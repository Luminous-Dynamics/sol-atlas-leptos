# Shared Energy Engineering and Discovery Contract v1

**Status:** Proposed research contract. Documentation-only; no engineering model is qualified by this document.  
**Scope:** Symthaea + Mycelix + Sol Atlas.  
**Baseline reviewed:** Sol Atlas main at `8c51d4a53b3fcb219dbd67eda2f96244704ccecf` (2026-10-10). Cross-project references below are moving development artifacts; verify their exact heads before consuming them.

## 1. The objective

Build a reproducible loop that moves from energy-system observations to explicit engineering hypotheses, then to controlled simulation, independent checking, and eventually field evidence:

```text
observations + provenance
    -> versioned study / frozen inputs
    -> physical model + constraints
    -> baseline policies + candidate policies
    -> raw step receipts
    -> independent recomputation + external-reference comparison
    -> uncertainty / trade-off report
    -> reviewed recommendation or explicitly rejected hypothesis
```

The loop must make it easy to discover improvements and equally easy to show that a candidate did not improve. A simulator output is a model result, not a meter observation; a model result is not an engineering certification; a recommendation is not an actuator command.

Near-term focus should be energy reliability and access: solar/wind resource and site screening, battery and thermal storage, inverter/microgrid behavior, critical-load service during outages, and lifecycle economics. New energy-material or process candidates should use the same evidence discipline but a domain-appropriate experiment/holdout protocol.

## 2. Current project boundaries and observed gaps

This is a source review, not a claim that the latest CI or a live installation has been verified.

### Symthaea — physics, simulation, and hypothesis ranking

The public `symthaea-grid-physics` crate already separates grid physics from the cognitive stack. Its current module surface covers battery state/degradation, a simplified radial feeder, inverter droop, trip envelopes, islanding, solar/wind generation, and scheduling. The scheduler comments explicitly describe the reserve-aware advisor as a rule-based interface demonstration, not yet the full HDC-driven policy.

The highest-value existing qualification work is already tracked:
- [#7248 — qualify energy policies against a deterministic scenario corpus](https://github.com/Luminous-Dynamics/symthaea/issues/7248)
- [#7239 — harden scheduling timing and dispatch invariants](https://github.com/Luminous-Dynamics/symthaea/pull/7239)
- [#7304 — frozen scenario qualification receipts](https://github.com/Luminous-Dynamics/symthaea/pull/7304), currently open and based on the scheduling-hardening branch
- [#7305 — cross-validate battery accounting against NREL SAM](https://github.com/Luminous-Dynamics/symthaea/issues/7305)
- [#7306 — make the versioned per-load registry the simulator source of truth](https://github.com/Luminous-Dynamics/symthaea/issues/7306)

Compose with that work. Do not build another scheduler-evaluation framework beside it. The current simplified feeder is not a production AC power-flow, protection, or grid-code qualification.

### Mycelix — identity, coordination, and evidence records

The Mycelix Energy SDK already defines project/participant, production-reading, trade, settlement, storage, and priority concepts. The current public repository describes the Energy domain as pre-alpha/schema-backend work, without a deployed Energy UI. Treat those types as a starting contract rather than a deployed grid service.

The current `ProductionReading` shape includes a `verified` boolean and optional verifier identity. That alone cannot establish meter authenticity, calibration, time correctness, non-replay, or independent verification. Strengthen the claim with references to signed/source-bound evidence and verification receipts; do not equate “a record says verified” with verified physical delivery.

Relevant work already in flight: [Mycelix #4939 — enforce grid-trade input invariants](https://github.com/Luminous-Dynamics/mycelix/pull/4939). Compose with it rather than duplicate basic numeric validation.

### Sol Atlas — geography, resource inventory, and decision surface

Sol Atlas is the appropriate place to connect resource and asset inventories to geography and expose scenario comparisons, with source and freshness metadata visible to the user. Existing work is already addressing important pieces:
- [#66 — stop inferring EROI from financial ratios](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/66)
- [#68 — deterministic microgrid dispatch model](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/68)
- [#69 — discounted lifecycle LCOE and NPV](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/69)

The current `sol-atlas-core/src/energy_trading.rs` generator chooses synthetic trade pairs from site indices and a time-dependent phase; it does not calculate geographic distance, feeder reachability, or physical delivery. Its grid-stress values come from fixed illustrative city baselines and simple year formulas, not from metered loads, a calibrated demand forecast, or a power-flow solution. Those are acceptable demo/scenario mechanisms only when labelled as such.

The current `DataKind` vocabulary distinguishes observed, curated, and scenario data. Energy-system outputs also need a separate **modelled/derived** classification: model results are not necessarily fictional scenarios and must not be silently labelled as observed. Audit every exhaustive match before extending the enum. Layer-wide provenance should also be reconciled with per-dataset provenance and individual snapshots; a single layer label must not imply every item shares the same source or observation date.

## 3. Shared study identity: EnergyStudyV1

Every comparative run should be identified by one immutable manifest. Version the schema instead of silently changing the interpretation of a field.

Required manifest fields:

- `schema_version`, `study_id`, `case_id`, creation timestamp, and canonical `case_digest`.
- Geographic scope and CRS; site/feeder/asset registry version and digest. Avoid publishing precise residential coordinates or meter identities without authorization.
- Time axis: start/end, timezone, interval duration in integer seconds, and explicit treatment of irregular final intervals.
- Inputs: load, weather/irradiance, wind, tariffs, asset state, topology, and outage schedule. Each dataset records source URI/ID, source revision or snapshot time, content hash, units, quality flags, and evidence class.
- Asset and network assumptions: rated/effective power and energy, charge/discharge limits, efficiency convention, reserve floor, degradation parameters, feeder model class, loss assumptions, and any omitted physics.
- Model identity: source repository and exact commit, model/version, configuration digest, parameter mapping, and solver/toolchain identity where it affects results.
- Policy identity: baseline/candidate name, exact revision, configuration digest, and whether it is a rule policy, optimizer, learned policy, or external reference.
- Preregistered metrics, hard constraints, evaluation horizon, train/calibration/holdout partitions where applicable, and error tolerances fixed before the comparison.

Use a deterministic canonical encoding and cryptographic digest. Document canonical field ordering, string/number encoding, units, missing values, and schema-version transitions. Hash identity and provenance separately from outcome metrics so changing a result cannot rewrite the identity of its inputs.

## 4. Raw step receipts and independent checking

Aggregate numbers are not enough. Retain one receipt per interval, in an ordered and complete sequence, with at least:

- exact case digest, policy identity, source revision, step index, start instant, and duration;
- observed measurements, forecasts, and model-derived values in separate typed fields;
- AC-bus generation/import/export, served load, battery charge/discharge, curtailed generation, and declared electrical losses, with units and sign conventions;
- battery inventory and SoC before/after the step, using one explicit energy boundary;
- served and unserved energy split by load class, including critical medical, water, communications, food preservation, and other declared service priorities;
- all active constraints, rejected/limited requested setpoints, outage/islanding state, missing/stale input flags, and numerical error flags;
- power-balance residual and source references for every input.

Use a declared AC-bus balance, with losses stated rather than hidden. For example:

`generation + grid_import + battery_discharge = served_load + battery_charge + grid_export + curtailment + declared_losses + residual`

The independent checker must validate step identity/order/completeness, finite values, unit consistency, state bounds, registry membership, and power/energy reconciliation. It must recompute all reported metrics from raw receipts instead of trusting the simulator's summary. A separately written checker catches shared-summary bugs; stronger independence comes from a different implementation/runtime or external reference.

At minimum, mutation tests must show rejection of: changed inputs with an unchanged digest; duplicate, missing, and reordered steps; forged totals; non-finite values; negative durations; impossible SoC; charge/discharge limit violations; inconsistent energy balances; priority breaches hidden by correct aggregate totals; and mismatched policy revisions.

## 5. Fair comparison protocol

Every policy in a comparison receives identical inputs, initial state, clock, topology, and constraints. Establish a baseline ladder rather than comparing only against a weak straw-man:

1. No-storage / fixed dispatch baseline where applicable.
2. Current simple greedy policy.
3. Reserve-aware policy.
4. A well-specified conventional optimization baseline when the same model assumptions support it (for example, LP/MILP or model-predictive control).
5. Symthaea-derived candidate policy.
6. Optional external-model/reference result, with an explicit parameter mapping.

Report each metric independently. Do not allow cheap energy or low average cost to compensate for unserved critical energy or hard-constraint violations. Use a constraint-first/Pareto report, with cost and emissions considered only after reliability and safety constraints are satisfied.

Required metrics include total and critical-load unserved energy, outage service duration, cost/import/export credits, renewable curtailment and self-consumption, battery cycles and capacity assumptions, peak import, constraint violations, recovery time, and data/forecast quality. Report per-case results, worst cases and distribution summaries; include calibrated uncertainty only where calibration has been demonstrated. A single composite score may be supplemental but cannot be the sole qualification gate.

For statistical comparisons, preregister the candidate set and held-out scenarios. Keep holdout truth unavailable to the policy and to any tuning loop until the evaluation is frozen. Record failures and regressions as well as wins.

## 6. Three implementation lanes

### Lane A — near-term resilient microgrids

Compose the Symthaea receipts and load registry with Sol Atlas's in-progress deterministic dispatch and lifecycle-economics work. Produce one small, frozen benchmark with solar/weather variation, evening peak, constrained/degraded battery, tariff boundaries, critical loads, and a grid outage. A second case must include poor resource conditions and inadequate storage; the system must report shortages honestly rather than inventing supply.

### Lane B — site and technology discovery

For a proposed site or technology, keep the following quantities separate:
- resource potential and its source/uncertainty;
- installed/nameplate capacity;
- expected generation under a stated model and weather trace;
- delivered energy after curtailment, losses, and availability;
- lifecycle cost/LCOE/NPV under explicit finance and lifetime assumptions;
- physical EROI only when energy-in and energy-out boundaries are compatible and sourced;
- emissions estimates with a named boundary and emissions factors;
- interconnection, land/water/material, permitting, maintenance, and supply-chain constraints.

For energy materials and new conversion/storage concepts, Symthaea may rank hypotheses or equations; every candidate needs exact training/search provenance, a leakage-controlled benchmark, uncertainty disclosures, and a feasible experimental or reference-model validation path. Symbolic consistency or a plausible simulation is not evidence of material performance.

### Lane C — Mycelix evidence and community coordination

Mycelix should bind a production/consumption or project claim to its source observation, responsible identity, time interval, calibrated meter/device identity where available, and the verification receipt. Preserve disputes and supersession rather than overwriting history. Separate physical metering, contractual entitlement, and financial settlement: a settlement receipt does not itself prove power physically traversed a particular path.

Privacy is a system requirement: use scoped disclosure and pseudonymous site identifiers, minimize exact-location retention, and retain enough hashes and public evidence for reproducibility without publishing household load traces.

## 7. External references and what they are for

- [NREL System Advisor Model (SAM) — battery storage](https://sam.nrel.gov/battery-storage/): candidate external reference for matched battery dispatch trajectories. SAM is not a standalone battery executable; its Generic System battery configuration and loss boundary must be pinned and documented. Symthaea's simplified constant-efficiency / linear cycle-fade model must not inherit SAM's chemistry, temperature, or lifetime accuracy claims.
- [CityLearn release documentation](https://www.citylearn.net/guides/releases.html): a useful optional multi-building/community energy-management benchmark. Its May 28, 2026 release notes describe a synchronized multi-community environment with per-community and portfolio metrics. Keep this optional so ordinary Rust tests remain small and offline.
- [OpenEMS Edge architecture](https://openems.github.io/openems.io/openems/latest/edge/architecture.html): useful architecture precedent for a frozen process image per control cycle, device abstractions, separated controllers, and priority-preserving outputs. Its documentation explicitly notes that soft real-time software is not appropriate for critical safety measures requiring hard timing guarantees.
- [NIST SP 1800-32](https://csrc.nist.gov/pubs/sp/1800/32/final): security reference for distributed energy resources, including communications/data integrity, authentication/access control, network monitoring, and protection of the control path.

These references support experimental design and architecture; they do not certify this project or mean the integrations have been executed.

## 8. Evidence gates and authority boundary

- **E0 — Contract valid:** schema, units, source identity, and deterministic fixtures validated.
- **E1 — Kernel internally consistent:** bounds, conservation/accounting properties, mutation tests, and independent receipt checking pass.
- **E2 — Reference compared:** pinned SAM / CityLearn / another justified model comparison executed; mapping differences and non-comparable quantities retained explicitly.
- **E3 — Generalization checked:** held-out weather/load/outage scenarios and sensitivity analysis; candidate beats the agreed baselines without reliability or constraint regressions.
- **E4 — Shadow mode:** read-only comparison against appropriately authorized real data; no setpoints sent to devices.
- **E5 — Physical deployment:** separate professional review, hardware-in-the-loop and protection testing, grid-code/regulatory analysis, operational monitoring, and an independently controlled safety layer.

An E0–E3 result authorizes research claims only. This contract grants no physical control authority. Initial work must remain offline/simulated or read-only. No model may promote itself, write actuator setpoints, bypass a local guard, or convert a missing observation into a pass. Each promotion requires its own evidence, and a queued or authored test is not a pass.

## Implementation status update — 2026-10-11

The first executable follow-up is drafted in [Sol Atlas PR #82](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/82), stacked on the microgrid dispatch model in [PR #68](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/68). It adds `EnergyStudyManifestV1`, fail-closed field validation, a domain-separated canonical SHA-256 encoding, and a synthetic cloud/evening-outage fixture with hashes for every listed input, registry, constraint set, model configuration, and policy configuration.

A separate [independent microgrid-result checker is drafted in PR #83](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/83). It rechecks interval balances, battery inventory transitions, input binding, grid availability boundaries, and every aggregate total from raw interval rows rather than rerunning the dispatcher. Its adversarial cases include forged summary totals and a fabricated grid import during an outage. The verifier is independent code within this repository, not an independent organization/runtime or external reference model.

[Mycelix PR #4956](https://github.com/Luminous-Dynamics/mycelix/pull/4956) adds the matching TypeScript manifest verifier and a byte-level checker for each referenced dataset and configuration artifact. An independent source-level SHA-256 pass reproduced the manifest's declared artifact hashes and canonical golden digest, and the Sol Atlas/Mycelix manifest fixture contents are byte-identical. Rust and TypeScript builds/tests have **not** been executed in this environment; this is static/hash verification, not a passing code qualification. The fixture is synthetic and does not establish real-world performance.

A newly recorded [issue #84](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/84) tracks a separate limitation: aggregate-load dispatch does not currently model service of critical versus noncritical loads. A constraint declared in a manifest must not be called satisfied unless its semantics are explicitly represented and independently checked. Manifest hashes do not authenticate sources, attest to runtime execution, or prove simulation correctness.

## 9. Immediate next deliverables

1. **Do not fork another scheduling harness.** Complete and review Symthaea #7239 / #7304, then continue #7305's optional SAM reference lane and #7306's load-registry integration with exact-head receipts.
2. **Qualify the Sol Atlas kernels.** Keep #66 / #68 / #69 focused; verify dispatch conservation, interval edge cases, and external-reference mapping. Add per-dataset provenance plus explicit modelled/scenario labels to derived layers. Audit the illustrative trade/stress simulation so geographic reachability and physical delivery are never implied when not modeled.
3. **Harden Mycelix source claims.** Compose with #4939; replace trust-by-boolean where consequential with verifiable source/evidence references, replay-resistant intervals, and explicit dispute/supersession behavior.
4. **Close the adapter loop.** Feed this frozen manifest fixture into Symthaea's per-step receipt/reporting contract, then bind the same case digest into Mycelix's evidence records and Sol Atlas's visible presentation. Add a cross-language golden-vector check before calling the interchange stable.

The success criterion is not the number of energy features. It is a reproducible, honestly bounded answer to: *which policy or technology performed better, on which identical conditions, by how much, with what uncertainty, under which constraints, and with what independent evidence?*
