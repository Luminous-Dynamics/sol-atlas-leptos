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
for accepted and rejected schedules when the typed input rows are complete.
A schedule with a missing or extra interval row is rejected before objective
receipt construction because six complete horizon objectives cannot be
recomputed from a partial trajectory.

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
the six recomputed objective values, auditor identity/version and a SHA-256
receipt-subject digest.

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

## Claim ceiling

A pass establishes independent conformance to this schedule schema and the
declared integer battery/PV/grid model for the exact request bytes. It does
not prove physical-model completeness, source evidence authenticity, solver
optimality, field reliability, grid-interconnection compliance, equipment
safety or authorization to execute the schedule.
