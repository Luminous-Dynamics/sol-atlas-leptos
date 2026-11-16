# MATPOWER to DC Network Evaluation v0.2

An executable bridge from strict MATPOWER v2 intake to the DC reference solver, independent receipt auditor and fixed-injection single-branch contingency screen.

## Pipelines

### Intact DC power flow

1. Parse and structurally admit literal MATPOWER v2 source using the PGLib case auditor.
2. Produce a typed, source-bound fixed-injection projection.
3. Map the projection to the canonical DC network type.
4. Solve using the separate DC reference kernel.
5. Independently reconstruct branch flows, nodal balances, slack injections and Rate A limits.
6. Emit a deterministic JSON receipt bound to the exact source Git blob SHA-1.

Run:

    cargo run --manifest-path tools/matpower-dc-network-evaluation/Cargo.toml --locked --offline -- tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m

### Fixed-injection single-branch outage screen

The second API runs the intact network and every in-service single-branch outage. Each contingency is solved independently and each returned candidate is audited. Outcomes remain distinct:
- independently audited with no declared nodal/Rate A violation;
- independently audited with declared violations;
- solver unknown (including islanding or unsupported topology);
- candidate rejected by the independent auditor.

The sweep has a hard scenario budget. An exceeded budget returns no partial report.

Run:

    cargo run --manifest-path tools/matpower-dc-network-evaluation/Cargo.toml --locked --offline --bin matpower-dc-contingency -- tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m

## Interpretation

A successful return means the independent auditor accepted internal consistency against the mapped network and declared linear DC equations. The report separately states whether nodal residual and Rate A checks pass. A thermal overload is an audited violation, not a solver failure and not a feasible result. A solver failure or islanding is unknown, never a pass.

The type-3 reference bus injection is balanced by the solver. **The downstream DC model does not enforce the reference generator's aggregate Pmin/Pmax after balancing.** It also does not model redispatch, reserves, ramps, resistance losses, line charging, reactive power, voltage magnitude, shunt behavior, branch angle-difference limits, Rate B/Rate C or generator cost curves.

This is fixed-injection linear DC power flow and single-line outage screening only. It is not DC-OPF, AC power flow, security-constrained OPF, full N-1 security, post-contingency redispatch, a calibrated digital twin, physical reliability evidence or an actuation path.

The network is generated internally from the exact source text; JSON receipts include source byte identity and computed bus states/branch flows or complete contingency records. Git blob SHA-1 is an identity binding, not proof of source authenticity or engineering truth.

## Pinned reference cases

PGLib-OPF v23.07 fixtures:
- IEEE 14 bus: 14 buses, 5 generators, 20 branches.
- IEEE RTS 24 bus: 24 buses, 33 generators, 38 branches.
- IEEE 57 bus: 57 buses, 7 generators, 80 branches.

The tests require successful intact DC solves and independently audited complete single-branch outage coverage. They do not assert that every contingency satisfies Rate A, and they do not claim AC/OPF equivalence or full N-1 security.


## Independent parser and numerical references

The integration now has two external comparison paths, kept out of the production dependency graph.

### PowerIO 0.12.1 — source-field differential

The separate nested workspace at tools/powerio-matpower-differential parses the same original MATPOWER bytes with PowerIO 0.12.1 and compares the typed fields against Sol-Atlas' source-bound projection. It checks bus/generator/branch fields, bus-level demand and shunt aggregates, online generation bounds and generator-cost rows. The only explicit semantic conversion is MATPOWER zero branch rating = unlimited ↔ PowerIO +Infinity. Mismatches are evidence, not silently normalized.

Run:

    cargo test --manifest-path tools/powerio-matpower-differential/Cargo.toml

This is a parser/model differential, not a numerical solver comparison or physical validation.

### MATPOWER 8.1 — independent DC numerical reference

The workflow .github/workflows/matpower-8.1-dc-reference-differential.yml pins MATPOWER commit 1a828c7af590714499284e36ee9c81273388c594 and invokes MATPOWER's own rundcpf on the unchanged PGLib source files. A separate Rust binary compares the output against the Sol-Atlas DC evaluation.

It compares:
- exact source Git blob SHA-1, base MVA and reference-bus identity;
- every bus angle relative to the reference bus;
- every bus net active injection, including the solved slack balancing injection;
- branch row/endpoint identity and status;
- branch PF and PT flow values.

Initial tolerances are fixed at 1e-7 degrees for bus angle, 1e-6 MW for bus injection and branch flow, and 1e-10 MVA for system base. The workflow produces TSV and JSON comparison receipts for IEEE 14, IEEE RTS 24 and IEEE 57 cases.

With MATPOWER 8.1 and Octave installed:

    cargo build --manifest-path tools/matpower-dc-network-evaluation/Cargo.toml --locked --offline --bins
    octave-cli --quiet --no-gui --eval "addpath('tools/matpower-reference-differential'); run_matpower_dc('tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m', '/tmp/case14.tsv', '/path/to/matpower-8.1', '$(git hash-object tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m)')"
    tools/matpower-dc-network-evaluation/target/debug/compare-matpower-dc tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m /tmp/case14.tsv

A zero mismatch is scoped to these exact inputs, MATPOWER version, fixed-injection DC equations, reported values and tolerances. It is not AC power-flow validation, AC-OPF parity, full N-1 security, physical calibration or operational authority.


## PTDF/LODF fast outage screen

The standalone library at tools/dc-lodf-outage-screen computes a source-bound base DC solution and predicts every in-service single-branch outage using PTDF/LODF sensitivities. It factorizes one reduced susceptance matrix per energized island and reuses it for each branch transaction.

Run:

    cargo run --manifest-path tools/matpower-dc-network-evaluation/Cargo.toml --locked --offline --bin matpower-dc-lodf-screen -- tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m

The JSON receipt has one record per in-service outage. An ordinary record includes predicted post-outage flows and Rate A overloads. A bridge or near-bridge record contains no predicted flows and explicitly requires an exact re-solve.

The PGLib integration tests compare every resolved prediction to a direct outage re-solve for all 20 IEEE 14-bus, 38 IEEE RTS 24-bus and 80 IEEE 57-bus outages. They require zero flow mismatches at 1e-6 MW, zero false-safe branch/outage results, zero unexpected exact-solver failures and complete coverage. Bridge/near-bridge cases are counted as abstentions.

The screen and exact comparison are separate APIs. The fast screen does not execute the exhaustive solver sweep. The validation API deliberately does, and its cost must not be included in fast-screen runtime measurements.

This is fixed-injection linear DC screening only. It is not redispatch, DC-OPF, SCLOPF, AC contingency analysis, or N-1 operating-security qualification.
