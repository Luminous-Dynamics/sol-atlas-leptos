# DC PTDF / LODF Outage Screen v0.1

A fast, fail-closed single-branch outage screen for the bounded fixed-injection DC reference model.

## Method

1. Solve the intact fixed-injection DC base case.
2. Partition the active network into energized islands and factorize one reduced susceptance matrix per island.
3. For every in-service branch k, solve the one-MW from-to transaction and compute its branch PTDF column.
4. Compute the LODF denominator 1 - PTDF[k,k].
5. For a well-conditioned denominator, predict post-outage flows using f_l(post k) = f_l(base) + LODF[l,k] * f_k(base).
6. If the denominator is non-positive or within the declared near-bridge tolerance, do not produce flows. Mark the outage as requiring an exact re-solve.

The transaction matrix is formed in MW/radian units using the same tap and reactance conventions as the reference kernel. Fixed phase-shifter offsets affect the base flow, not the linear sensitivity. The sensitivity model is factored once per connected component and reused for all branch transactions.

## Run against PGLib

    cargo run --manifest-path tools/matpower-dc-network-evaluation/Cargo.toml --locked --offline --bin matpower-dc-lodf-screen -- tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m

The source-bound receipt includes the original MATPOWER Git blob SHA-1, case function and version, base-case summary, every in-service branch outage, predicted branch flows, predicted Rate A overloads and explicit exact-solve abstentions.

## Exact-solve validation path

The compare_screen_to_exact function is deliberately separate from the fast screening path. It disables each branch in turn, re-runs the full DC reference solver, and compares every predicted branch flow. The PGLib integration tests apply this check to:
- IEEE 14 bus: 20 branch outages;
- IEEE RTS 24 bus: 38 branch outages;
- IEEE 57 bus: 80 branch outages.

The frozen initial acceptance tolerance is 1e-6 MW. Tests also require zero false-safe branch predictions, zero false-safe outage classifications, zero unexpected exact-solver failures, and complete coverage. Bridge / near-bridge abstentions are expected and are not counted as predictions.

## Claim boundary

This is **linear fixed-injection DC screening only**. It is not redispatch, DC-OPF, security-constrained OPF, AC contingency analysis, voltage/reactive validation, dynamic stability, protection coordination, reserve/ramp qualification or an actuation path. A predicted-safe result applies only to the declared DC model and input snapshot. It does not make an operational security claim.

A denominator below the threshold is an abstention, not a safe outage. The caller must escalate it to an exact model that can re-establish island references and balance each island or retain the outcome as unknown. This library does not silently invent new slack generators.
