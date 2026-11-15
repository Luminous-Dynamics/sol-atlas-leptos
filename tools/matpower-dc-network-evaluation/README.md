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
