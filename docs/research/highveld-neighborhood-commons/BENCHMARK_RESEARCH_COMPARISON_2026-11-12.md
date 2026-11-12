# Highveld research and benchmark comparison — 12 November 2026

**Scope:** neighbourhood resilience commons / Highveld household service resilience.  
**Status:** research synthesis and proposed benchmark targets; no industry head-to-head or physical validation has been performed.

## 1. Current research changes the framing

### Community-scale resilience is a techno-legal and social-system problem

A November 2026 review of community-centric power-system resilience explicitly connects community-level resilience metrics, power-system enhancement strategies, AI/data-driven methods and law/governance. This supports treating the commons as a governed service network rather than just a microgrid or shared battery. It does not prove that a particular hub topology works in the Highveld context.

Reference: Nazari et al., *A review of community-centric power system resilience: Strategies, data-driven methods, and techno-legal perspectives*, Electric Power Systems Research 260 (Nov 2026), article 113285, DOI: https://doi.org/10.1016/j.epsr.2026.113285.

### The built-environment resilience evidence base is still fragmented

A November 2026 systematic scoping review maps 176 studies, identifies 29 resilience indicators, and highlights fragmented operational definitions and underrepresentation of Global South settings. The review is not a census of all industry practice. It gives Highveld a legitimate research opportunity: define operational service and access outcomes that are traceable across disciplines, while avoiding claims that incumbent products lack capabilities the review did not assess.

Reference: Ramesh & ur Rehman, *Energy resilience in the built environment under disruptive events: hazard-, climate- and equity-informed systematic scoping review*, Energy and Buildings 370 (1 Nov 2026), article 118111, DOI: https://doi.org/10.1016/j.enbuild.2026.118111.

### Adaptive surrogate UQ is advancing rapidly

A November 1, 2026 BIM-based adaptive surrogate study combines sparse Polynomial Chaos Expansion and sequential sampling for EnergyPlus-based uncertainty quantification across four Irish residential archetypes and a 14-variable input space. It reports holdout R² ≥ 0.99 for EUI and ≥ 0.93 for annual heating demand using 480–560 simulations per stage, with Monte Carlo comparison. Those are results for the paper's models, outputs, archetypes and assumptions—not a Highveld target or a universal SOTA threshold.

Highveld should reuse this class of multi-fidelity/surrogate method where appropriate, but evaluate **decision preservation**, especially false-feasible resident-service decisions, robust-feasibility changes, Pareto rank reversal, OOD detection and uncertainty calibration. A surrogate with excellent average R² can still be unsuitable near a hard service threshold.

Reference: Naamrudi & Aghamolaei, *A BIM-based adaptive surrogate framework for efficient uncertainty quantification in building energy simulation*, Energy and Buildings 370 (1 Nov 2026), article 118098, DOI: https://doi.org/10.1016/j.enbuild.2026.118098.

### VVUQ should be proportionate to decision consequence

NIST's October 5, 2026 report argues for harmonizing computational-model UQ with metrology practices and scaling VVUQ to model influence and decision consequence. This supports a risk-tiered qualification plan: a model used for early option screening needs a different evidence burden from one that can authorize an essential-service claim or a physical intervention.

Reference: Srinivasan, *On Methods of Uncertainty Quantification in Computational Modeling and Simulation: Harmonizing Modeling and Metrology Practices*, NIST AMS 100-85 (5 Oct 2026), DOI: https://doi.org/10.6028/NIST.AMS.100-85.

## 2. South African source implications

The DWS Gauteng dashboard published 31 May 2026 reports four strategic Rand Water reservoir-system series and says municipalities communicate localized supply outages separately. Its reservoir values are system context, not tap delivery at a household.

The 10 September 2026 joint government statement describes City Power interruptions affecting Rand Water's Eikenhof system and water-supply systems across municipalities. This is a credible cross-service dependency lead, but remains a government statement, not the issuer-native interval data required for a joined event trace.

Acquisition implication:
1. Preserve each DWS dashboard revision and its exact aggregation/date/units.
2. Seek Rand Water, City Power, Johannesburg Water, DWS and Eskom original event notices/telemetry for the Eikenhof episode.
3. Preserve each issuer's temporal and geographic scope before joining.
4. Do not infer local tap pressure/flow, household availability, causality or event probability from the reservoir dashboard or government statement.
5. Admit physical measurement only through the instrument owner, with a reference method, calibration, uncertainty, placement, clock and consent record.

References:
- DWS, *Monthly Dashboard for a Water Secure Gauteng*, Rev 02, 31 May 2026: https://www.dws.gov.za/documents/2026-05%20Gauteng%20Dashboard.pdf
- South African Government, joint statement on power trips and water-supply disruptions, 10 Sep 2026: https://www.gov.za/news/media-statements/government-measures-mitigate-water-supply-disruptions-resulting-power-trips

## 3. Incumbent comparison: compete at the boundary, not in every solver domain

| Capability domain | Specialist benchmark / incumbent | Highveld approach |
|---|---|---|
| Whole-building thermal/energy physics | EnergyPlus + applicable ASHRAE Standard 140 capability tests | Use the engine and standard; pin inputs, outputs, applicability and VVUQ. Do not build a competing thermal solver. |
| Building controls and fault detection | BOPTEST standardized emulator/control cases | Reuse matched eligible cases; benchmark access to service outcomes and evidence identity outside the emulator's physics scope. |
| Microgrid sizing/dispatch/protection | Specialist power-system and microgrid tools | Keep electrical engineering in its qualified solver; bind energy deliveries and failure modes to household service floors. |
| Water pressure/hydraulics | Qualified hydraulic/network models and utility records | Keep reservoir, bulk, municipal, distribution, tap and household scopes distinct. A reservoir balance is not delivered water. |
| Spatial/accessibility | GIS/network accessibility and route analysis | Bind access mode, route-time, mobility/access requirements, service hours and route disruption into service contracts. |
| Trajectories, recovery and adaptive measurement | Symtropy #1848/#1849 and its existing qualifiers | Reuse the native event clock, stable-recovery semantics, Pareto and decision-impact/adaptive-measurement qualifiers. |
| Evidence, rights, allocation and commitments | Mycelix owner systems | Preserve evidence/rights, consent, resident approval, human authority and appeal. No optimizer gets autonomous moral priority. |

The differentiator is not that Luminous is better than any of these specialist engines. It is the **evidence-preserving cross-domain decision contract**: exact common traces, matched design/service requirements, scope/rights/revision, resource conservation, access and queue constraints, resident floors, and replayable independent decisions.

## 4. Proposed benchmark target set

These targets apply first to finite synthetic known-answer cases. They are not physical performance claims.

### Hard synthetic correctness gates

| Metric | Target | Why it matters |
|---|---:|---|
| Household essential-floor false passes under seeded hub/common-cause loss | 0 | No hub-level success can conceal a household floor breach. |
| Local fallback independence known-answer agreement | 100% | Household guarantee survives commons loss. |
| Delivered-service false positives from hub availability, storage or proximity alone | 0 | Availability is not delivery. |
| Unqualified/inaccessible service allocated as delivered | 0 | Access is a precondition, not metadata. |
| Queue wait exceeding an admitted household floor but still marked served | 0 | The queue cannot silently consume the resident's service allowance. |
| Unmodelled peak-demand/capacity overcommit | 0 | Congestion must be bounded or unmet demand reported. |
| Resource ledger reserve breaches | 0 | Optimizer cannot spend a protected reserve. |
| Duplicate source-event resource credits | 0 | One meter/notice event cannot create multiple units of stock. |
| Unit-mismatch silent conversions | 0 | No L↔m³ or Wh↔kWh coercion without a versioned conversion. |
| Unapproved/appeal-less/automated-final allocation | 0 | Allocation authority remains resident-governed. |
| Exact oracle agreement on finite control suite | 100% | The suite is a deterministic software qualification corpus. |

### Comparative and decision-quality metrics

Report as vectors, never one universal score:
- per-household essential-service hours preserved;
- requested/admitted/delivered/refused/delayed/inaccessible/unknown service quantities;
- access time, queue time and unserved time separately;
- hub-loss autonomy by household/service;
- common-cause exposure count and failure-domain overlap;
- resource-balance residual by account and unit, alongside uncertainty;
- time to first floor breach, first restoration and stable recovery;
- repair/spares/operator access and re-verification time;
- Pareto precision/recall against exhaustive finite oracle;
- low-to-high-fidelity decision preservation and false-feasible rate;
- surrogate uncertainty coverage/calibration near each service threshold;
- human-hours/rework to a defensible decision at equivalent quality;
- consent, appeals, reversals and unresolved allocation disputes.

### Physical and model qualification

No universal R², MAE or residual threshold can qualify all physical claims. For each model/claim:
1. pin the model purpose, intended decision and consequence;
2. pass applicable ASHRAE 140 capability tests where relevant;
3. define held-out reference cases and measurement uncertainty before fitting;
4. report residuals by operating regime, not only pooled means;
5. check tails and classification errors at essential service thresholds;
6. declare out-of-domain, nonconverged, missing-input and extrapolated states;
7. scale VVUQ effort to model influence and decision consequence;
8. require an independent re-evaluation of claims that change a resident floor or physical safety decision.

## 5. Paradigm shifts to pursue

### 1. Resilience as a right-sized service guarantee
Move from capacity assets (kWh of battery, litres of tank) to a contract for resident task/service continuity, with a local fallback and bounded interruption. Assets are means; service delivery is the outcome.

### 2. Access-adjusted, not merely geographically shared
A nearby hub may be unreachable, overloaded, inaccessible, closed or unsafe to reach. Model the complete chain from request to delivery, including route, wait, refusal and outage. The "commons" is only resilient if residents can actually use it.

### 3. Resource provenance as a conservation law
The system must prove where each unit originated, where it was transferred, which loss was applied, which reserve remains and whether the same source event has been double-counted. No implicit units, duplicate credits or hidden reserve depletion.

### 4. Household autonomy plus collective option value
Keep local essential fallback, but make shared services optional augmentation. This preserves sovereignty while allowing repair capacity, logistics and large shared assets to increase capability when accessible.

### 5. Decision-centric multi-fidelity assurance
Use screening and surrogate methods to reduce expensive simulation, but spend high-fidelity effort where a plausible input/parameter change can alter a hard floor, Pareto set or access conclusion. Stop measurement only when the decision is stable across remaining admissible states and no critical blocker remains.

### 6. Governance is part of the engineered system
Allocation policies, appeals, consent, rights and resident oversight have measurable service implications. They cannot be appended as a disclaimer after the optimizer has chosen who receives water or power.

## 6. Next executable work

1. Add offline/manual-degraded-mode and maintenance/spares contracts to the synthetic reference schema and qualifier.
2. Add household data-rights/consent and explicit revocation controls, preserving no-sharing-by-default.
3. Add a deterministic 15-family commons stress trace with matched household/hub outages, queue/load peaks, repair delay, route interruption and repeat fault. Keep all quantities synthetic and include a finite oracle.
4. Bind its service-event projection to Symtropy's native owner interface; do not duplicate its timeline/recovery engine.
5. Add a queue/access fairness report that shows every household, not only neighbourhood average.
6. Only after owner review, extend the candidate comparison matrix. Keep #736's v0.1 three-candidate experiment unchanged until that decision.
7. Acquire South African source snapshots through the named owners and preserve system/municipal/dwelling scope differences.
8. Move to measurement only after Mycelix instrument admission and reviewed unoccupied test procedures.

## Claim ceiling

This synthesis informs research direction and benchmark design. The cited literature does not evaluate Highveld, the synthetic manifest does not model a real neighbourhood, and the hosted Rust checks do not validate building physics, hydraulic service, microgrid stability, accessibility, allocation equity, safety, cost or resilience. No claim of SOTA superiority is made.
