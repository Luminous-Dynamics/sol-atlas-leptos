# Energy Dispatch Standalone Schedule Auditor v0.1

A standalone Rust auditor for domain-level energy dispatch schedules. This
crate is intentionally separate from the LP translator and solver pipeline.
It does not import LP parsers, LP model types, variable names, solver result
parsers or optimizer code.

## Paradigm

A solver should return a dispatch trajectory. An independent auditor should
evaluate that trajectory against the source request contract, not reconstruct
domain meaning from LP variable identifiers.

## Run

Generate the standalone lockfile and run the crate independently:

    cargo generate-lockfile --manifest-path tools/energy-dispatch-schedule-auditor/Cargo.toml
    cargo fmt --manifest-path tools/energy-dispatch-schedule-auditor/Cargo.toml --all
    cargo test --manifest-path tools/energy-dispatch-schedule-auditor/Cargo.toml --locked
    cargo clippy --manifest-path tools/energy-dispatch-schedule-auditor/Cargo.toml --all-targets --locked -- -D warnings

Audit an exact request and schedule:

    cargo run --release --manifest-path tools/energy-dispatch-schedule-auditor/Cargo.toml --locked -- request.json schedule.json audit-receipt.json

Exit status is 0 for an accepted schedule, 1 for a parsed but rejected schedule,
and 2 when parsing, I/O, or receipt construction fails. A receipt is written
for accepted and rejected schedules when both inputs pass strict parsing and
typed schema decoding. When interval counts differ, the receipt remains
rejected and marks all six objective fields null because a partial trajectory
cannot support complete objective recomputation; it is not converted into a
fake zero-valued result.


## CLI input resource boundary

The CLI accepts at most 1 MiB for the source request and 1 MiB for the
dispatch schedule. It opens each input first, inspects regular-file metadata
on that same opened handle, rejects an over-limit size, and then reads through
a bounded stream capped at limit+1 bytes. The final byte count is checked
before parsing. This prevents metadata-only checks from being bypassed by
file growth or path replacement between stat and read. Directories, FIFOs and
other non-regular inputs are rejected.

The unit tests cover the exact-limit and one-byte-over stream boundary. These
controls establish bounded local file ingestion; they do not authenticate the
input producer or protect the receipt output directory from an untrusted
filesystem namespace.

## Schedule contract

The schedule DTO is defined in src/lib.rs and the JSON Schema is
schedule-schema-v0.1.json. The top-level schedule binds:

- schema version and source case ID;
- SHA-256 of the exact source request bytes;
- one ordered dispatch record per source interval;
- terminal SOC reported by the schedule.

Each interval reports domain quantities: grid import, charge input, discharge
withdrawal, SOC after the interval, essential/flexible served and unserved
energy, PV used and curtailed. It does not expose solver-specific names such as
charge_mode_000, charge_gain_000 or soc_after_000.

## Independent checks

The auditor strict-parses raw request and schedule bytes, rejecting duplicate
object member names, invalid numeric values and trailing JSON before typed
deserialization. It independently checks:

- exact source-request digest, schema, case ID, interval count and ordered IDs;
- unique interval identifiers, positive duration, non-overlapping chronology;
- initial SOC, capacity and terminal reserve;
- battery charge/discharge limits, SOC availability and no simultaneous
  charge/discharge;
- exact integer floor charge and discharge efficiencies;
- SOC transition at every interval and final reported terminal SOC;
- grid import limits and zero import during outage;
- essential and flexible served/unserved partitions;
- PV use/curtailment partition and interval bus energy balance;
- all six independently recomputed objective values.

The objective values are recomputed from source coefficients and dispatch
actions. Terminal SOC objective uses the independently propagated SOC state;
it does not trust the schedule's reported SOC fields.

## Receipt

The receipt binds exact request and schedule SHA-256 values, source and
schedule schema IDs, case ID, synthetic-source flag, candidate-expansion
metadata, interval count, every pass/fail predicate with interval localization,
the six recomputed objective values, auditor identity/version, the SHA-256
of the actual running executable and a SHA-256 receipt-subject digest. CI
checks that the receipt's binary digest equals the exact release CLI binary
whose hash is published with the run.

The digest binds the serialized receipt subject, not an external signature.
Input authenticity and receipt signing are separate concerns.

## Tests

The current unit panel includes:
- a known-answer H=2 charge-from-PV followed by essential-load service during
  a grid outage;
- valid H=3 idle-recovery and H=4 post-outage-recovery trajectories;
- SOC mutation at each H=2 interval;
- outage import, charge/discharge conflict, load partition and PV partition
  negative controls;
- interval reordering and request/schedule digest cross-association;
- fractional 2/3 charge and discharge efficiencies with exact floor behavior;
- nonzero grid cost/carbon recomputation;
- terminal reserve failure;
- duplicate-key, unknown-field and malformed-association input controls.

The auditor tests are independent known-answer and mutation tests. They do not
call the LP validator. Cross-checks against the LP validator's exhaustive H=2
and sampled H=3/H=4 panels can be added in a separate integration harness,
without sharing production model-building or verification code.

## Reproducible latency benchmark

The dependency-free Rust benchmark measures horizons H=2, H=3 and H=4 with
50 warmups and 1,000 recorded samples per metric and horizon. It reports
strict duplicate-aware parsing of the exact request and schedule bytes
separately from end-to-end audit plus receipt generation. Each row contains
p50/p95/p99, minimum/maximum nanoseconds and source/schedule byte lengths.
These are small synthetic implementation baselines; they are not field
dispatch latencies or an industry performance comparison.

## Claim ceiling

A pass establishes independent conformance to this schedule schema and the
declared integer battery/PV/grid model for the exact request bytes. It does
not prove physical-model completeness, source evidence authenticity, solver
optimality, field reliability, grid-interconnection compliance, equipment
safety or authorization to execute the schedule.

## Independent essential-service floor evaluator v0.1

The package also builds a second executable, `energy-dispatch-service-floor-auditor`. It is a distinct process from the schedule search and from the schedule-auditor CLI. It accepts five inputs:

```sh
cargo run --release --locked \
  --manifest-path tools/energy-dispatch-schedule-auditor/Cargo.toml \
  --bin energy-dispatch-service-floor-auditor -- \
  request.json schedule.json service-floor-policy.json \
  schedule-audit-receipt.json service-floor-receipt.json
```

The evaluator does not accept optimizer-supplied service or objective claims. It strict-parses the exact request, schedule, policy and prior schedule-audit receipt; recomputes the source-aligned schedule findings and all objective values; verifies the receipt's canonical subject digest and exact request/schedule bindings; then evaluates the pinned policy.

The v0.1 policy contains:
- minimum essential-service fraction per interval (basis points, 0–10,000);
- maximum cumulative essential unserved energy;
- maximum consecutive intervals below the per-interval floor;
- minimum terminal battery SOC.

This is intentionally time-indexed. A daily/whole-horizon energy total cannot conceal a critical interval in which essential service falls below its declared floor. The service-floor receipt binds the exact policy, request, schedule, preceding schedule-audit receipt, schedule-auditor executable and service-floor-auditor executable. The receipt digest is a content identifier, not an external signature or proof of binary authenticity.

The checked-in synthetic PV/outage policy requires 100% essential service in every interval, zero cumulative essential unserved energy, zero consecutive service-floor misses and a zero-quanta minimum terminal reserve. It is a deliberately strict synthetic positive control, not a universal policy for all sites or operating contexts. A real deployment needs a separately authored, risk-approved policy with site-specific service classes, uncertainty, operating envelope and operator authority.

The profiler workflow now runs both executables for each of its 30 repeated synthetic schedules and requires 30 accepted schedule receipts plus 30 passed service-floor receipts. The existing v0.2 solver trace remains unchanged; these separate service-floor receipts are uploaded as independent artifacts until the v0.3 trace-runner migration composes them.

## Non-anticipativity auditor v0.3: causal observation and forecast histories

The additive `energy-dispatch-nonanticipativity-auditor` binary evaluates
whether complete scenario schedules can represent one implementable policy
under the information actually declared available at each decision time. It
is separate from the single-scenario schedule auditor and the service-floor
evaluator.

### Two evidence streams, not one omniscient scenario request

The scenario request remains the full realized trajectory used by the domain
schedule auditor and by after-the-fact scoring. **It is not used to derive the
policy's information set.** This distinction prevents future realized PV,
load, outage or price values in the request from silently becoming information
available to an earlier decision.

The v0.3 manifest lists 2–256 scenarios and, for each scenario and decision
interval, references two exact, time-stamped artifacts:

1. an observation bundle (`energy-dispatch-observation-bundle-v0.1`) with
   `observed_at_unix_seconds` and a non-empty `observation_payload`;
2. a forecast bundle (`energy-dispatch-forecast-bundle-v0.1`) with issue time,
   validity window and non-empty `forecast_payload`.

Both artifacts bind the same `scenario_set_id` as the manifest. The manifest
contains file paths and declared timestamps, never caller-chosen information
set hashes. The CLI loads both exact byte streams using bounded readers and
canonical path resolution beneath the manifest directory. It rejects duplicate
JSON object keys, unknown top-level fields, path traversal, symlink escape,
oversize inputs, future-dated observations and forecasts, expired forecasts,
and cross-scenario-set bundles.

### Causal information-history derivation

At interval (t), the evaluator chains only the observation artifacts through
that decision:

[
O_{s,t}=H(
  \textext{history schema},
  \textext{scenario-set ID},
  t,
  O_{s,t-1},
  \textext{observed-at}_t,
  H(\text{canonical JSON observation payload}_t)
)
]

It then derives the information-set identity:

[
I_{s,t}=H(
  \textext{information-set schema},
  \textext{scenario-set ID},
  \textext{decision time}_t,
  O_{s,t},
  \textext{forecast vintage}_t,
  H(\textext{exact forecast bundle bytes}_t)
)
]

The initial observation-history seed is a fixed, domain-separated digest.
Every subsequent history digest includes the previous one, the current
observation timestamp and the SHA-256 of the canonical serialized
`observation_payload`. Bundle ID and JSON formatting do not create a new
information state. The receipt separately records the SHA-256 of the exact raw
observation bundle bytes for provenance. Forecast bytes are hashed separately
and included in the information-set identity.

This design has three important consequences:

- Future realized intervals in the full scenario request cannot change an
  earlier information-set identity.
- Scenarios with identical cumulative observation histories at a decision time
  must use the same forecast vintage and exact forecast bytes. A caller cannot
  manufacture a branch by selecting a different forecast file for one scenario.
- A genuine new observation changes the cumulative history and may permit
  scenario schedules to choose different actions from that decision onward.

The evaluator still requires a common source schema, evidence class, search
budget, energy quantum, battery configuration and interval chronology. It
requires one common initial root history, rejects vacuous success, validates
each complete trajectory with the existing standalone domain predicates and
recomputes its objectives. For every interval, all scenarios with equal derived
information-set identities must take identical dispatch actions, including
grid import, charge/discharge, SOC after the interval, essential/flexible
service, unserved energy and PV use/curtailment.

### Manifest, receipt and execution

The v0.3 manifest is defined by
`nonanticipativity-manifest-schema-v0.3.json`. The observation and forecast
artifact contracts are `observation-bundle-schema-v0.1.json` and
`forecast-bundle-schema-v0.1.json`. The receipt schema,
`nonanticipativity-receipt-schema-v0.3.json`, binds the scenario-set ID,
exact request/schedule digests, exact observation/forecast digests and
timestamps, cumulative observation-history identities, derived information-set
identities, findings, receipt digest and auditor executable digest.

Run the auditor with the manifest and receipt path:

```sh
cargo run --release --locked \
  --manifest-path tools/energy-dispatch-schedule-auditor/Cargo.toml \
  --bin energy-dispatch-nonanticipativity-auditor -- \
  nonanticipativity-manifest.json nonanticipativity-receipt.json
```

Exit 0 means the complete scenario set passed; exit 1 means a well-formed but
non-anticipative policy was rejected; exit 2 means an input or audit error.
The receipt is content-addressed, not externally signed.

### Controls and claim boundary

The checked-in controls under `schemas/examples/nonanticipativity-v0.1/`
exercise:

- **Positive:** both branches share the exact root observation and forecast,
  take the same root PV-charge action, then receive distinct decision-time
  essential-demand observations and choose different subsequent actions.
- **Negative:** both schedules are individually valid and share the same root
  observation and forecast, but choose different root actions; the auditor
  must reject the policy.
- **Future-data isolation:** different full realized request trajectories do
  not alter an earlier information-set digest when the observation/forecast
  artifacts available at that earlier decision are identical.
- **Forecast escape control:** two scenarios with identical observation
  histories cannot select different forecast artifacts to obtain different
  actions.
- **Freshness controls:** future-dated observations/forecasts and bundles from
  another scenario set are rejected.

**Critical limit:** the raw artifacts are content-bound and time-checked, and
observation semantics are canonicalized, but this verifier does not authenticate
the observation publisher, prove the measurements were actually available to
the controller, authenticate the forecast producer, or establish forecast
calibration/probability validity.
Those require independent provenance, signed publisher identity, clock and
freshness assurance, revocation handling and calibration evidence. The current
auditor cannot prove the semantic truth of caller-supplied observation payloads.

This is the causal-information verifier for
[ENERGY-UNCERTAINTY-001 / #312](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/312),
not yet a scenario-tree optimizer, a stochastic optimality proof or a
deployable rolling-horizon controller.