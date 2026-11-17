# Maintenance-to-dispatch coupling v0.1

**Status:** research implementation; synthetic software test only  
**Owner:** Sol-Atlas  
**Composes:** maintenance availability trace + bounded exact dispatch oracle  
**Scope:** one PV inverter represented as the complete PV availability series for one site-microgrid case

## Purpose

The maintenance availability projection answers: when is this exact asset represented as available, planned-outage, or unknown in the study trace?

The bounded dispatch oracle answers: what finite integer-quantized site schedule is feasible and Pareto-nondominated for the declared source and demand envelope?

This coupling applies the former to the latter. It does not invent a new optimizer, lifecycle ledger, or network solver.

## Deterministic transformation

For each dispatch interval [start, end):

1. Validate the maintenance trace schema, source-reference uniqueness, and complete contiguous coverage of the declared study window.
2. Bind the maintenance trace to the exact PV inverter ID and the configuration digest for which the PV production envelope was prepared.
3. Require the complete dispatch interval to be contained in exactly one maintenance availability interval. If an interval crosses an availability boundary, reject it; do not average PV availability across an interval.
4. If the asset state is Available, preserve the declared PV availability quanta.
5. If the asset state is PlannedOutage or Unknown, set the bound inverter's PV availability to zero. Keep the two source states distinct in the binding receipt.
6. Reject any post-maintenance configuration digest that differs from the dispatch profile's bound configuration. A replacement, repair, firmware or controller-setting change requires a new profile and requalification before the asset may contribute PV again.
7. Pass the adjusted finite request to assess_energy_dispatch. The exactness claim is inherited from that oracle and remains limited to the declared discrete model.

The grid-outage flag is not changed. A local PV inverter maintenance event is not silently converted into a grid outage.

## Synthetic known answer

Study window: [0, 10800) seconds. Three one-hour dispatch intervals. The single bound PV source has 2 integer energy quanta available in each interval before maintenance is applied.

| Interval | Maintenance state | Input PV | Effective PV | Grid outage | Essential demand |
|---|---|---:|---:|---|---:|
| 00:00–01:00 | Available | 2 | 2 | No | 1 |
| 01:00–02:00 | PlannedOutage | 2 | 0 | No | 1 |
| 02:00–03:00 | Available | 2 | 2 | No | 1 |

Battery capacity and grid import are both zero for this fixture. The exact finite dispatch assessment therefore has one essential-service quantum unmet in the maintenance interval and zero unmet in the other two intervals. That is an arithmetic known answer, not a resilience, grid, or field result.

## Adversarial tests

- Missing maintenance authorization remains Unknown and zeros PV supply.
- A wrong asset ID is rejected.
- Any configuration change invalidates the old PV production profile.
- An interval crossing a maintenance boundary is rejected.
- A dispatch horizon outside the maintenance study is rejected.
- The source maintenance state does not modify the independent grid-outage flag.
- The exact finite dispatch result exposes the induced essential-service deficit.

## Industry comparison and claim ceiling

This adapter is narrower than industry scheduling/control platforms:

- PyPSA's maintenance scheduling functionality is a model-side component scheduling capability for supported component classes (including Generator, Link and Process); it is not the same as a lifecycle-authenticated maintenance event or proof of as-installed asset configuration.
- Grid2Op exposes maintenance actions in sequential grid-operation environments; it is a controller/agent evaluation platform, not an independent asset manufacturing and acceptance ledger.
- NREL REopt addresses site/campus/microgrid techno-economic sizing and dispatch/resilience; it is not a per-asset work-order provenance system.

This slice should be compared only on the same declared site-microgrid case and formulation. It does not claim superiority, and it does not model line or transformer outages, network topology, AC/DC power flow, multiple PV arrays, partial output derating, forecast uncertainty, degradation, crew/parts logistics, RMS/EMT dynamics, protection, HIL, or field operation.

## Next qualification boundary

The next broader maintenance/service study must use a network model that explicitly represents the maintained element and its topology cut. It must then independently replay per-location essential/flexible service, ENS/EENS, restoration and reliability indices. A single-source PV availability overlay is not an acceptable substitute for a 3-bus line/transformer outage model.
