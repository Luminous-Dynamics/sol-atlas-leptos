# MATPOWER to DC Network Evaluation v0.1

An executable bridge from the strict MATPOWER v2 importer to the DC reference solver and independent receipt auditor.

## Pipeline

1. Parse and structurally admit literal MATPOWER v2 source using the PGLib case auditor.
2. Produce a typed, source-bound fixed-injection projection.
3. Map the admitted projection to the canonical DC network type.
4. Solve using the separate DC reference kernel.
5. Independently reconstruct branch flows, nodal balances, slack injections and Rate A limits.
6. Emit a deterministic JSON receipt bound to the exact source Git blob SHA-1.

## Run

    cargo test --manifest-path tools/matpower-dc-network-evaluation/Cargo.toml --locked --offline
    cargo run --manifest-path tools/matpower-dc-network-evaluation/Cargo.toml --locked --offline -- tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m

## Interpretation

A successful return means the independent auditor accepted the receipt's internal consistency against the mapped network and declared linear DC equations. The receipt separately reports whether nodal residual and Rate A checks pass. A thermal overload is an audited violation, not a solver failure and not a feasible result.

The type-3 reference bus injection is balanced by the solver. **The downstream DC model does not enforce the reference generator's aggregate Pmin/Pmax after balancing.** It also does not model redispatch, reserves, ramps, resistance losses, line charging, reactive power, voltage magnitude, shunt behavior, branch angle-difference limits, Rate B/Rate C or generator cost curves.

This is fixed-injection DC power flow only. It is not DC-OPF, AC power flow, security-constrained OPF, full N-1 security, a calibrated digital twin, physical reliability evidence or an actuation path.

The network is generated internally from the exact source text; the JSON receipt includes source byte identity and computed bus states and branch flows. Git blob SHA-1 is an identity binding, not proof of source authenticity or engineering truth.

## Pinned reference cases

PGLib-OPF v23.07 fixtures:
- IEEE 14 bus: 14 buses, 5 generators, 20 branches.
- IEEE RTS 24 bus: 24 buses, 33 generators, 38 branches.
- IEEE 57 bus: 57 buses, 7 generators, 80 branches.

Tests require successful DC solves and independent nodal/receipt checks for all three cases. They do not assert that all cases satisfy branch limits or claim AC/OPF equivalence.
