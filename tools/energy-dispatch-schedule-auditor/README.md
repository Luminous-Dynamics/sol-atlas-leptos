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

## Non-anticipativity auditor v0.1

The additive `energy-dispatch-nonanticipativity-auditor` binary audits whether
a set of complete scenario schedules can be interpreted as one policy under
the caller-declared information sets. It is separate from the single-scenario
schedule auditor and from the service-floor evaluator.

A strict manifest lists 2–256 scenarios and binds a safe `scenario_set_id`. Each entry binds:
- a unique scenario ID, equal to the request case ID;
- one exact source-request JSON file;
- one exact complete schedule JSON file;
- one 64-character information-set SHA-256 label per source interval.

The receipt records the scenario-set ID, exact request/schedule digests, information-set labels, shared information-set count, action-comparison count, per-group findings, receipt digest and auditor executable digest. Its JSON contract is defined in `nonanticipativity-receipt-schema-v0.1.json`.

The CLI is invoked with a manifest and receipt output path:

```sh
cargo run --release --locked \
  --manifest-path tools/energy-dispatch-schedule-auditor/Cargo.toml \
  --bin energy-dispatch-nonanticipativity-auditor -- \
  nonanticipativity-manifest.json nonanticipativity-receipt.json
```

The CLI strict-parses the manifest and each source input, rejects duplicate
JSON object keys, bounds every input read, and resolves request/schedule paths
beneath the manifest directory (including canonical-path checks against
symlink escape). It writes a canonical receipt and returns exit 0 for accepted,
1 for a well-formed but rejected policy, or 2 for an audit/input error.

The library evaluator first requires every scenario request and schedule to
pass the existing standalone domain predicates/objective recomputation. It
then requires all scenarios to share the same source schema, synthetic/evidence
class, search budget, energy quantum, battery configuration and interval
chronology. It also requires every scenario to share one root information set
at the first decision; scenario trees without a common initial decision are
rejected in this v0.1 contract. For each interval it groups scenarios by
information-set digest. Every group with two or more members must have:
1. identical source-observation prefixes through that interval; and
2. identical dispatch action fields at that interval, including grid import,
   battery charge/discharge, SOC after the interval, essential/flexible served
   and unserved quantities, and PV use/curtailment.

Actions may diverge after the information-set labels diverge. A set with no
shared information sets is rejected rather than passing vacuously. The receipt
binds every request/schedule digest, the information-set labels, all findings
and the exact auditor executable digest.

**Critical limit:** the auditor proves equality only for the information-set
labels supplied in the manifest. It cannot prove that these labels are
authentic, complete, or genuinely available to an operator at the decision
time. The CLI also does not prove probabilistic forecast calibration,
stochastic optimality, service-floor compliance, physical/network safety,
or actuation authority. This is the first verifier primitive for
[ENERGY-UNCERTAINTY-001 / #312](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/312), not yet a complete scenario-tree optimizer or rolling-horizon controller.

The v0.1 tests include:
- an accepted two-scenario example where both branches take the same 2-quanta
  PV charging action at the root and diverge to 2-quanta versus 1-quanta
  battery discharge after branch information is revealed;
- rejection when two scenarios with the same pre-branch information choose
  different actions;
- acceptance when pre-branch actions match and actions diverge after the
  information-set labels split;
- rejection when a shared label hides different declared observation prefixes;
- rejection when no information set is shared;
- receipt digest/binary provenance checks and bounded/path-safe CLI controls.
