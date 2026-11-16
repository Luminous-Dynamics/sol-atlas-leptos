# Infrastructure Interdependency Industry Comparison
## Matched evaluation protocol v0.1

**Status:** literature-grounded benchmark design; no head-to-head execution or superiority claim.  
**Parent programme:** DEEP-TIME-INFRASTRUCTURE-001 #886.  
**Related:** #890 service-continuity oracles; #894 topology renewal; #895 Durban–Gauteng demonstrator; Symtropy #3894/#5452/#5474; Mycelix #5923/#5762.

## 1. What the current industry landscape says

Shamsi, Helmrich and Deegan published an open-access comparative review on 7 November 2026. The paper's declared scope is energy, water and transport infrastructure. It identifies 24 tools in that review scope and seven that explicitly support system-of-systems interdependency assessment. It concludes that sector-specific tools dominate, physical interdependencies receive more attention than non-physical dimensions, complexity competes with practical usability, and no single tool captures the full breadth of interdependency behavior.

Reference: N. Shamsi, A. Helmrich and M. Deegan, *Toward system-of-systems resilience: a comparative framework for infrastructure interdependency assessment tools*, Environment Systems and Decisions, published 7 November 2026, DOI https://doi.org/10.1007/s10669-026-10162-2 .

This is a literature sample and a structured assessment, not a live market census, common runtime leaderboard or evidence that any one tool is universally best.

## 2. Seven direct comparators identified by the review

| Tool | Primary approach in the review | Strength / appropriate use | Comparison caution |
|---|---|---|---|
| **Kaleidoscope** | Cloud spatial analytics and predictive algorithms | Spatial co-location, asset vulnerability patterns, strategic capital prioritization | Relationships are often spatial/implicit rather than explicit dynamic cascade semantics |
| **CIrcle** | Participatory expert workshops and causal mapping | Stakeholder elicitation, organizational/functional dependency discovery, interpretable causal pathways | Qualitative to semi-quantitative outputs; analyst facilitation and elicitation quality matter |
| **REFIT** | Probabilistic Monte Carlo cascading simulation with synthetic networks and domain solvers | Scenario-based cascade/risk exploration and investment analysis; can couple to domain tools | Probabilistic inputs and synthetic network assumptions need calibration/qualification |
| **PRIIA** | Research-stage hypergraph representation and optimization-based restoration prioritization | Post-disaster restoration planning and network dependency prioritization | Scope is focused on restoration/prioritization; verify maturity and access for a matched run |
| **CISIA-Pro Simulator** | MATLAB-based fault propagation through predefined dependencies and failure parameters | Deterministic/parameterized cascade exploration under declared network mappings | Review reports narrower breadth and validation than the broader tools; do not infer unsupported dimensions |
| **InfraRisk** | Modular simulation of sequential hazard → failure → recovery at asset level | Asset-level resilience and restoration trajectory analysis; comparatively low data burden in the review | Requires compatible network, operational and restoration inputs; a scoped tool, not a universal dependency platform |
| **IN-CORE** | Integrated community-resilience environment combining hazards, infrastructure, fragility, recovery and socio-economic processes | Broad community-scale system-of-systems modelling, including physical, functional, geographic, economic and social dimensions | High data requirements and context-specific model calibration; output truth remains bounded by model/data assumptions |

The review evaluates four technical dimensions—interdependency breadth, data requirements, validation strength and integration potential—and five categorical dimensions—purpose fit, methodological variety, context suitability, application readiness and sectoral scope. These scores are ordinal tool-selection guides, not a universal performance ranking. IN-CORE is described as the broadest integrated perspective in the sample, but the paper emphasizes that strengths and trade-offs differ by application.

The review's full text and figures should be consulted before implementing any detailed numeric scorecard. This protocol intentionally does not copy an approximate radar-chart score into a Luminous-owned leaderboard.

## 3. The opportunity: compose models, own the evidence and decision boundary

Luminous should not build an eighth monolithic interdependency simulator.

The defensible research direction is an **evidence-qualified federation**:
- specialized tools continue to own their validated hazard, hydraulic, fragility, network, social or restoration models;
- their outputs enter through explicit adapters and exact source/model lineage;
- dependency classes and time validity remain typed, not flattened;
- service obligations, beneficiary floors, ecological constraints and legal authority are represented separately;
- adaptive transition policies are compared against fixed and expert baselines;
- independent exact oracles qualify small instances and adversarial controls;
- all results are reproducible and bounded by the imported model's context of use.

Existing Luminous owners:
- Symtropy #3894 — service-cascade propagation/recovery;
- Symtropy #5452 — matched external resilience evaluation and claim discipline;
- Symtropy #5474 — external model/dataset evidence envelope;
- Mycelix #5923/#5762 — time-qualified dependency hypergraphs and transitive closure;
- Sol-Atlas #406 — common-cause-aware evidence independence;
- Symthaea #7636 — adaptive pathways, triggers and value of information;
- Sol-Atlas #400 / Symtropy #1631 — TRIM rail evidence intake and source-qualified corridor graph.

This work owns the *comparison protocol* for the deep-time service/topology problem, not the implementations above.

## 4. Matched benchmark tracks

A tool should be scored only on tasks that match its published capability and context of use. Unsupported is a valid result; do not turn an out-of-scope task into a zero score.

### Track A — spatial dependency discovery

**Question:** Can the tool identify geographically co-located assets and shared hazard exposure without claiming physical independence?

Compare Kaleidoscope-style spatial analytics with a source-qualified GIS baseline. Measure:
- asset/section coverage and spatial-source provenance;
- co-location and shared hazard-domain precision/recall against a frozen synthetic oracle;
- false independence when routes share flood, subsidence, fire or coastal footprints;
- resolution, datum and positional uncertainty sensitivity;
- analyst effort and data burden.

This track tests geography, not dynamic service-cascade correctness.

### Track B — elicited causal/organizational dependencies

**Question:** Can stakeholder knowledge reveal dependencies that a physical graph misses?

Compare CIrcle-style facilitated causal mapping with a frozen expert dependency set and a typed cross-domain register. Measure:
- dependency-class coverage;
- inter-rater agreement and disagreement retention;
- source/role/authority provenance;
- time to construct and update a model;
- false certainty and missing/contested dependencies.

Do not treat an elicited causal link as a measured probability.

### Track C — exact small-network cascades

**Question:** Does the model correctly propagate faults through declared AND/OR/K-of-N dependencies and shared causes?

Compare REFIT, CISIA-Pro, InfraRisk and IN-CORE where adapters and rights permit. Use:
- exact finite-state fixtures with hand-computable cascade outcomes;
- matched common-cause and independent-failure controls;
- sequence/time-order cases;
- service loss by beneficiary rather than asset count alone;
- restoration precedence and recovery-resource contention.

The exact oracle is independent of every candidate production simulator.

### Track D — community-scale service consequences

**Question:** Do hazard/asset changes produce defensible consequences for essential community functions?

IN-CORE is a serious reference where a rights-cleared, scope-compatible testbed and outputs are available. Preserve hazard, asset inventory, fragility, functionality, restoration, socio-economic inputs, source revision and run lineage. NIST Community Resilience Planning Guide is a planning/performance-goal comparator, not a numerical simulator.

Do not invent an IN-CORE score from a paper description or substitute a Luminous synthetic case for a matched testbed.

### Track E — restoration and topology renewal

**Question:** Can a service network be deliberately reconfigured without avoidable essential-service interruption?

Compare PRIIA-style restoration prioritization and the existing specialist cascade/recovery tools against the deep-time topology-renewal oracle. Test:
- replacement commissioning before old-asset retirement;
- route transfer and shared-section capacity;
- recovery action prerequisites and shared crews;
- terminal/port relocation;
- canal retirement and multimodal substitution;
- ecological restoration as a separate state transition;
- time-to-reconfigure and cumulative service deficit.

This is the likely differentiator for the deep-time programme: not a stronger static hazard map, but a verified comparison of service-preserving topology transitions.

## 5. Shared scorecard

Keep the following dimensions separate; do not collapse them into one weighted "resilience score":

1. **Semantic coverage:** supported dependency classes, relation types, time validity and unsupported/unknown count.
2. **Correctness:** exact-oracle agreement, false-safe, false-independent, false authority acceptance and branch leakage.
3. **Cascade quality:** service deficit, failure-path correctness, dependency traceability and event-order determinism.
4. **Recovery:** restoration trajectory, resource precedence, time under service threshold and reconfiguration latency.
5. **Adaptation:** trigger timeliness, information delay, option preservation, regret and worst-case service loss.
6. **Evidence quality:** exact source/model/version/dataset lineage, datum/unit validation, uncertainty and rights.
7. **Validation maturity:** analytical, synthetic, laboratory, historical field and prospective evidence—reported separately.
8. **Integration:** adapter surface, native-unit preservation, data contract completeness, round-trip reproducibility.
9. **Usability:** analyst hours, training burden, configuration complexity, explanation quality and repeatability.
10. **Resource cost:** runtime p50/p95, memory, simulation evaluations, compute/energy and licence/access burden.

A model with no valid data for a field must report unknown/not supported, not zero risk.

## 6. Proposed performance targets for Luminous

These are qualification hypotheses, not current results:

- 100% exact agreement with the independent oracle on every supported finite synthetic case.
- Zero false-safe or false-independent outcomes on frozen common-cause, rights, ecology, datum, missing-source and future-information negative controls.
- 100% source/model/configuration traceability for material consumed claims.
- 100% deterministic replay for pinned finite synthetic cases.
- No unsupported physical/sector model domain silently treated as qualified.
- At least 20% lower integrated critical-service deficit at equal fully burdened lifecycle budget **or** at least 20% lower lifecycle cost while preserving all agreed mandatory service/safety/ecological floors on a preregistered matched test—only if the external case and data can resolve an effect of that size.
- Report cost/analyst effort and performance trade-offs rather than claiming a universal model winner.

If the test does not support a 20% materiality threshold, publish confidence intervals, sample precision and the smallest detectable effect instead.

## 7. What “better than SOTA” should mean here

Do not claim to beat IN-CORE at community hazard/fragility/recovery modelling, REFIT at probabilistic cascade simulation, or Kaleidoscope at geospatial analytics without matched tests in those domains.

A meaningful advantage would be demonstrated if the federation:
- preserves the specialist model's original semantics and evidence;
- reveals a material dependency or time transition that the matched baseline omitted;
- prevents a false-safe portfolio decision;
- produces a lower-regret, service-preserving transition at equal cost/information;
- can be independently replayed from exact evidence;
- exposes uncertainty and unknowns without turning them into invented probabilities.

A correct abstention or "not supported" result is a capability, not a failure to optimize.

## 8. Literature and comparator references

- Shamsi, Helmrich & Deegan (2026), [comparative framework for infrastructure interdependency assessment tools](https://doi.org/10.1007/s10669-026-10162-2).
- NIST / IN-CORE, [Center of Excellence for Risk-Based Community Resilience Planning](https://www.nist.gov/community-resilience/center-excellence).
- NIST, [Community Resilience Planning Guide](https://www.nist.gov/community-resilience/planning-guide).
- NIST, [IN-CORE architecture publication](https://www.nist.gov/publications/interdependent-networked-community-resilience-modeling-environment-core).

## 9. Claim ceiling

This is a literature-grounded comparison protocol. It is not a reproduced comparison, tool validation, SOTA result or claim that Luminous has broader/stronger capabilities today. Each external tool must be evaluated on a task it supports, with matched inputs and independent outcomes.

## 10. Additional 2026 evidence: federation, lifecycle semantics and adaptive transport

Two recent reviews reinforce why the comparison should be about composable, qualified model workflows rather than building a single general-purpose twin.

### Infrastructure BIM/digital-twin lifecycle review (30 October 2026)

Pisciotta and Inzerillo review 97 primary studies selected from 531 records. In that corpus, 65 studies emphasize interoperability/data modelling, 49 include digital twin/IoT/structural-health-monitoring technologies, and 47 include open/semantic interoperability standards. The technology categories overlap, and the authors explicitly warn that labels do not establish operational maturity. They identify gaps in semantic continuity, updating, governance and long-term operational validation. Their review also notes heterogeneous case evidence and the presence of conceptual and one-way-update systems.

Reference: A. Pisciotta & L. Inzerillo (2026), *Building Information Modeling for Infrastructure Lifecycle Management: A Systematic Review of Interoperability, Asset Management, and Digital Twins*, Applied Sciences 16(21), 10749, published 30 October 2026. https://doi.org/10.3390/app162110749 .

**Implication:** connector count is not interoperability. The federation must preserve units, coordinate/datum semantics, valid-time, revision lineage, update direction, model boundary, rights and claim ceiling. A model that only imports one way or has no operational update/replay evidence must be labelled accordingly.

### Adaptive transport under deep uncertainty (November 2026)

Jittrapirom et al. propose an adaptive policy framework for sustainable transport that links monitoring, contingency planning and adaptive actions, illustrates the approach with a stylized urban redevelopment case in the Netherlands, and treats accessibility as an outcome of interaction across multiple spatial scales.

Reference: P. Jittrapirom, V. Marchau, R. van der Heijden & H. Meurs (2026), *Decision-supportive framework for sustainable transport under deep uncertainty*, Transport Policy, Article 104389, November 2026. https://doi.org/10.1016/j.tranpol.2026.104389 .

**Implication:** the Durban–Gauteng and deep-time comparisons must evaluate transport accessibility and service continuity across scales, not only vehicle or section efficiency. Monitoring, detection delay, implementation lead time and contingencies must be part of the matched policy—not assumed instantaneous.

### Multiplex infrastructure criticality (September 2026)

Chatzistefanou et al. propose a multiplex-network framework for interdependent infrastructure asset criticality under natural hazards, explicitly considering multiple interaction types, multiple criticality dimensions and cascading effects.

Reference: G. A. Chatzistefanou et al. (2026), *A multiplex network-based framework to assess interdependent infrastructure asset criticality and priority under natural hazard scenarios*, International Journal of Disaster Risk Reduction, 143, 106319, September 2026. https://doi.org/10.1016/j.ijdrr.2026.106319 .

**Implication:** the comparison suite should include multiplex dependency representations as a direct method baseline. It should still compare each model against typed hypergraph, time-valid dependency and exact small-instance oracles, not assume that more layers alone produce better causal fidelity.

### Standards scope caution

ISO 23247-6:2026 describes digital-twin composition methods (integrated, unified and federated) for manufacturing environments. It is a useful architectural reference for composing independently built twins, but its stated scope is manufacturing—not a direct civil-infrastructure certification or conformance requirement.

Reference: ISO 23247-6:2026, *Digital twin framework for manufacturing — Part 6: Digital twin composition*. https://www.iso.org/obp/ui?_escaped_fragment_=iso%3Astd%3Aiso%3A23247%3A-6%3Aed-1%3Av1%3Aen .

## 11. Revised differentiation statement

The defensible target is an evidence-qualified federation across specialist models, with:
- exact source/model/version and transformation lineage;
- typed physical, digital, geographic, logical, economic, institutional, ecological and evidentiary dependencies;
- explicit model-form disagreement, not default averages;
- time-valid observations, service floors and governance authority;
- adaptive pathway comparison with monitoring and action latency;
- independently checkable finite oracles and immutable replay.

The opportunity is to outperform **the integration and decision-assurance workflow on matched tasks**, not to claim that one Luminous model replaces every specialist model.