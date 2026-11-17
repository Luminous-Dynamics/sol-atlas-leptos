# Home Expression: 2026 research frontier, benchmark design and competitive strategy

**Study:** HOME-EXPRESSION-SOTA-REVIEW-001  
**Checked:** 2026-11-16  
**Status:** Research synthesis and proposed targets; no new system-level SOTA claim  
**Owners:** Sol-Atlas #904 / #911; reuse Symtropy #1766 / #1848 / #1790 / #1900, Symthaea #7790 / #7801, Mycelix #5149

## Executive conclusion

The field is advancing along three tracks:
1. **Neuro-symbolic plan synthesis:** generate layouts, retrieve legal/architectural precedents, run deterministic geometric checks, and refine from feedback.
2. **Performance optimization:** couple physical simulation to surrogate-assisted search, uncertainty quantification, clustering and multi-objective decision support.
3. **Evaluation science:** move beyond FID and image similarity toward element validity, performance and human preference.

Sol-Atlas should not try to win by creating the largest catalogue, by generating the most floor plans, or by claiming a universal best aesthetic. The differentiator should be an end-to-end, resident-directed **design decision and evidence compiler**:

`resident intent -> candidate programme -> alternatives -> independent evidence -> robust Pareto set -> explicit human decision -> revision-bound delivery`

The decisive advantage is not that the system draws a more attractive house. It is that every candidate is traceable, that missing evidence stays unknown, that hard constraints cannot be traded away, and that a design change cannot inherit an obsolete qualification.

This architecture should compose the existing Sol-Atlas/Symtropy/Symthaea/Mycelix owners, not duplicate them.

## 1. Current research state

### 1.1 LayoutGen: a current neuro-symbolic layout baseline

Kembhavi et al., *An evaluation guided multiagent framework for generating floor plans that comply with municipal building regulations*, was published on 15 October 2026 in *Discover Artificial Intelligence*. It describes LayoutGen, a four-phase pipeline (spatial reasoning, coordinate mapping, SVG rendering and iterative refinement) with dual retrieval over architectural precedents and municipal rules, deterministic geometry checks, and evaluation-gated agent transitions.

On 200 runs, the paper reports a reduction from 3.7 to 0.3 detected violations per plan (92%) relative to an unretrieved, unrefined single-shot baseline using the same backbone. This is an important matched baseline and validates the generate–check–refine pattern.

But the measured compliance envelope is narrow and explicitly declared:
- single-storey residential plans, studio–3BHK, axis-aligned rectangular rooms;
- topological/geometric validity plus encoded minimum room area, minimum room width and critical width;
- setbacks, FAR, egress path length, stair geometry, parking and ventilation are not covered by the reported in-loop dimensional-compliance measure;
- the three jurisdiction examples (BBMP, CMDA, GHMC) share comparable dimensional-rule structure; this is not proof of international portability;
- structural, electrical and mechanical systems are outside the implementation;
- the authors identify blind correlation against practising architects as future work and state that their three scored dimensions are internal operationalisations.

**What to adopt:** dual-source grounding; deterministic checks; explicit state transitions; itemised feedback; ablations; identical-backbone comparisons.

**What to improve:** keep statutory and safety constraints as non-compensable gates instead of a weighted grade; version and verify the authority corpus; qualify a wider set of spatial/service/physical evidence; add resident-specific preference and dissent; and make each evidence record exact-revision-bound.

A grade such as A+ from an internal scoring function is not a permit or a professional certification. The paper itself bounds what the compliance measure establishes.

### 1.2 Surrogate-assisted multi-objective building design

Talami, Costa and Stouffs, *A fast, interpretable and uncertainty-informed design-support framework for multi-objective building performance optimization*, was available online on 20 October 2026 in *Energy and Buildings* (DOI 10.1016/j.enbuild.2026.118417; article in press at the time checked).

The study combines physics-based simulation, radial-basis-function surrogate-assisted optimization, uncertainty quantification, graph-based clustering and decision-support indices. Its Nottingham office case uses 11 geometry/envelope/HVAC/control variables and four climate/occupancy uncertainty scenarios. The journal abstract reports 93.8% fewer simulations than exhaustive search, R² around 0.95 for energy demand and around 0.93 for discomfort hours, with interpretable clusters of robust design pathways.

This is a much more relevant efficiency baseline than a raw “generative AI is faster” claim. It shows that the next design-system breakthrough can be **selective use of expensive physics**, not replacing physics with an opaque model.

Important qualification: the study’s stakeholder personas are LLM-simulated. Those can test interaction or decision-support interfaces, but they are not actual resident participation, consent, community approval or human-preference evidence.

### 1.3 Evaluation metrics are themselves a research problem

Brama et al., *Challenges in the evaluation of machine learning techniques in generative urban design*, was first published online 14 June 2026 in the *International Journal of Architectural Computing* (DOI 10.1177/14780771261459705).

The study finds that common generative-design evaluation approaches can respond more strongly to visual disruption than to meaningful urban/architectural logic. Generic FID and current VLM judgements have limitations; the authors propose domain-guided feature representations and structured VLM violation identification, and a four-level evaluation framework: model quality, design-element validity, performance KPIs and human evaluation. They report that some VLM scores overlap between benign and disrupted plans; more structured prompts that ask for specific violations can improve discrimination.

**Implication:** never use FID, image similarity, a single VLM score or a composite “beauty” score as a feasibility or design-success oracle. A VLM can help triage or explain candidate issues, but its findings need deterministic checks or appropriately qualified human review.

### 1.4 Standards and physics baseline

- **IFC:** buildingSMART lists IFC 4.3.2.0 (IFC 4.3 ADD2) as the current official schema release, published as ISO 16739-1:2024. IFC 4.4 and IFC 5 remain in development. Use the official schema release, not a draft, in conformance tests.
- **IDS:** Information Delivery Specification 1.0 is an approved buildingSMART standard for machine-readable alphanumeric information requirements on IFC objects (classifications, properties, materials, quantities and relations). IDS does not replace geometry validation; pair it with explicit geometry/relationship checks and report semantic loss separately.
- **EnergyPlus:** version 26.2.0 was released 30 September 2026. Pin the exact release, executable/package digest, EpJSON schema, weather file and model inputs for every physical-model receipt.

### 1.5 Additional 2026 evidence: interactive surrogates, residential decision fronts and participation

Three further findings sharpen the benchmark strategy:

**Interactive surrogate + NSGA-III (September 2026).** Uddin, Khanna and Aman describe a browser-based early-design framework trained on 600 Latin-Hypercube EnergyPlus simulations of a small commercial office in ASHRAE Climate Zone 4A. Their multi-output DNN reports R² above 0.95 for EUI, peak heating demand and spatial daylight autonomy; the authors report 3–4 orders of magnitude reduction in per-evaluation time versus direct simulation and Pareto front generation within 60–180 seconds. This is a useful interactive-search comparator, but it is one small commercial-office case, one climate zone and a 600-simulation training population. It is not evidence of cross-climate residential generalization or that the surrogate can safely replace final EnergyPlus runs.

**Residential optimization / decision support (1 October 2026).** Romeos et al. couple a high-fidelity EnergyPlus model and a multi-objective evolutionary algorithm for residential buildings in Mediterranean climates. Their reported front includes 59 non-dominated solutions trading annual space-conditioning energy against construction material cost. This is a relevant residential baseline, but those two objectives do not constitute a complete household utility, accessibility, water, service continuity or multi-resident preference model.

**Community participation gap (1 October 2026 review).** A systematic review of 108 generative urban-design studies reports community engagement absent in 77% of the surveyed studies. This finding motivates a deliberate participation/evaluation track; it does not demonstrate that a particular AI system gives residents meaningful agency. Luminous must test resident task completion, comprehension, privacy, influence over decisions and preservation of dissent with actual consented participants—not count prompts or simulated personas as engagement.

These findings suggest we need two different leaderboards:
- **search performance:** exact-oracle regret/hypervolume, feasibility, expensive simulation calls, wall time, surrogate calibration and high-fidelity confirmation;
- **decision legitimacy and usability:** source attribution, explicit preference state, choice comprehension, privacy-scope compliance, dissent preservation, and the fraction of resident-stated requirements that visibly influence accepted alternatives.

Do not combine these leaderboards into one weighted scalar. A very fast optimizer cannot compensate for a system that misrepresents or excludes the people whose home is being designed.

## 2. Paradigm shifts worth pursuing

### A. From a generated plan to a resident-owned design decision

Represent a household's intent as a structured, revisable problem, not a prompt to be averaged into a single “ideal” design. Preserve non-negotiables, preferences, protected/avoided expression, shared/private spaces, and unresolved disagreements. A candidate is a proposal; acceptance is a separate resident decision.

### B. From weighted grade to hard gates + partial order

Separate:
- binding constraints (must pass, otherwise block or remain unknown);
- resident preferences (belong to the resident; not an optimizer's truth);
- measured/estimated objectives (units, scope and uncertainty);
- unresolved/incomparable dimensions.

Only candidates that pass the relevant hard gates enter the feasible Pareto set. Do not allow a high daylight or beauty score to compensate for a failed egress/accessibility/structural requirement. Do not force a winner when two candidates are incomparable or the resident's preference ordering is incomplete.

### C. From “run every simulator” to multi-fidelity, value-of-information search

Compose:
1. inexpensive deterministic checks and symbolic constraints;
2. low-cost conceptual models;
3. calibrated surrogate screening;
4. high-fidelity, pinned external simulations for candidates near the decision boundary or Pareto front;
5. independent high-fidelity re-evaluation of the final candidate set.

Use active sampling/VOI to decide which simulation or measurement can change the decision. Track surrogate error and calibration by archetype, climate, operating regime and objective. Out-of-domain prediction must abstain. Never use a surrogate as the final authority for safety or code compliance.

### D. From static performance to robust habitability trajectories

A dwelling is experienced over time. Measure service levels through heat/cold, power/water outages, equipment faults, material delays, household changes and recovery. Keep essential service floors and resident tasks visible. Compose this work with Symtropy #1848; do not create another timeline or physical solver.

### E. From one “best style” to plural architectural expression

Allow single, blended, custom and undecided styles; residents can rename, reject and define. Keep cultural/historical references resident-authored, rights-bound and context-reviewed. Style labels are prompts for exploration, not demographic inferences or authenticity claims.

### F. From amenity icons to fully costed service packages

An observatory, skybridge, indoor pool, hidden room or underground wellness suite is a design programme change, not a decorative checkbox. Ask what it changes in space, structure, fire, moisture, access, water, energy, cost, sourcing, repair and whole-life operation. Unknown is a valid answer until the responsible owner supplies evidence.

### G. From “design version” to causal evidence lineage

An intent-only change does not automatically invalidate unrelated physical evidence. An actual geometry, assembly, service or equipment change may invalidate several dependent receipts. The single revision owner (#832) must decide from the exact dependency graph. This prevents both stale PASS leakage and indiscriminate invalidation.

### H. From optimizer recommendation to a resident-owned decision receipt

The current synthetic Home Freedom candidates have all ten source gates at `UNKNOWN`. A useful system should not hide that state behind an attractive Pareto chart. It should emit a revision-bound decision receipt which says what can be compared, what remains conditional, which evidence is missing, and whether the resident has actually chosen.

The new Sol-Atlas #917 work implements the contract-side guard:
- exact profile, brief, candidate-set and design-revision binding;
- exact preservation of every source hard-gate ID/state/reason/evidence reference;
- `FAIL -> BLOCKED`, `UNKNOWN -> CONDITIONAL`, all `PASS -> eligible for resident-authorized preference comparison`;
- no universal score, automatic rank, automatic selection or inferred resident choice.

This composes Symthaea #7790/#7801 and Symtropy #1766/#1848; it is not a second Pareto optimizer or physical evaluator. The current synthetic receipt correctly leaves both source candidates conditional and the resident choice unrecorded.

### I. 2026 openBIM conformance gap: IDS checker results can differ

buildingSMART's 2026 IDS community survey received 186 responses; 58% reported using IDS on real projects. The survey reports strong demand for checker/authoring certification and notes that the same IDS file can produce different checking results across tools. IDS 1.0 is a final standard for **alphanumeric** IFC information requirements; it does not specify geometric checking.

This is an opportunity for a differential conformance benchmark, not a reason to replace IFC/IDS. For identical frozen IFC + IDS fixtures, compare checker outputs across tools, capture exact versions, and require every discrepancy to become a reproducible counterexample. Separate:
- IDS schema conformance;
- information requirement evaluation;
- geometry / spatial checking (outside IDS's alphanumeric scope);
- downstream engineering evidence.

A tool's “IDS pass” must not be promoted to geometry, building-code or physical qualification.

## 3. Competitive map and fair comparison

| Comparator | Strongest relevant capability | Fair matched task | Where Luminous should differentiate |
|---|---|---|---|
| Finch3D | Generative room/plan layouts and design-standards workflows | Same brief, code assumptions, input budget, editable output and review maturity | Resident agency, plural intent, evidence lifecycle, uncertainty and downstream service qualification |
| TestFit | Site feasibility, building configuration and pro-forma workflows | Same parcel, zoning/rights evidence, unit mix, rate date and quantity scope | Household-level outcomes, service continuity, whole-life affordability and exact evidence provenance |
| Autodesk Forma | Early-stage site, environmental and massing studies | Same location, weather, geometry and analysis scope | Connect context to resident-led dwelling alternatives and qualified downstream evidence |
| Autodesk Revit | Detailed parametric BIM, documentation and collaboration | Same BIM exchange and revision tasks | Do not replace Revit; provide an open, traceable intent/decision/evidence layer that can hand off to it |
| LayoutGen (research baseline) | Retrieval-grounded, evaluation-guided residential plan generation | Same RPLAN/MSD/Tell2Design case, geometry class, rule set and generation budget | Broader hard-gate/evidence coverage; residents' constraints and choices; robust whole-life and revision assurance |
| EnergyPlus / OpenStudio | Whole-building energy and water simulation | Same geometry, weather, constructions, occupancy/schedules and systems | Never claim to outperform the solver without a matched calibration/validation study; improve decision search around it |
| IFC / IDS | Open BIM data exchange and information requirements | Same source models, export/import versions and property/geometry scope | Report exact round-trip preservation, semantic loss and claim-specific evidence links |

Do not say “we beat Revit” or “we beat Forma” on a synthetic schema test. Compare each tool only on functions it actually claims and supports. For a South African trial, verify local inputs and municipal sources; do not assume U.S. parcel integrations or Indian municipal rules transfer to Johannesburg.

## 4. Benchmark programme

The current Home Freedom 100-case corpus (#819) remains the feasibility oracle. The expression benchmark extension (#904/#906) is a test extension, not a second corpus. The spatial-resilience benchmark belongs to Symtropy #1848; intent-to-design and counterfactual robustness belong to Symthaea #7790/#7801.

### P0 — integrity and hard gates
- zero false-feasible / false-PASS outcomes on the frozen adversarial oracle;
- 100% non-negotiable resident-constraint retention;
- 100% unknown/stale/conflicted evidence remains explicit;
- zero silent zero-fill of cost, water, energy or physical performance;
- 100% exact source-revision and method-version binding;
- 100% detection of seeded stale revisions, wrong units, wrong jurisdiction, unsupported scope and revoked/mismatched evidence;
- positive-control recall proves that valid candidates can pass; always-blocked is a failing system.

### P1 — alternative quality and search
- exact enumeration oracle on tractable small cases;
- hypervolume (with a frozen reference point), IGD+, epsilon indicator and Pareto-front coverage;
- feasibility rate and hard-gate false-PASS reported separately from optimization metrics;
- candidate diversity at the spatial/topological/functional level, not merely colour or material variation;
- counterfactual locality: changing one input should affect only dependent outputs/evidence;
- preference sensitivity and Pareto reversals reported instead of hiding them in a single scalar score;
- scenario-wise regret and robust-feasibility fraction under frozen uncertainty sets.

### P2 — physics, cost and lifecycle
- EnergyPlus 26.2.0 or another declared engine with exact version/input/weather digests;
- thermal comfort hours/degree-hours by declared metric and occupancy;
- energy by end use, peak demand and reserve dependence;
- water source/withdrawal/reuse/discharge and service continuity;
- whole-life cost with dated local rates, quantity provenance, exclusions and uncertainty;
- embodied carbon with lifecycle boundary and EPD/data source;
- maintenance access, replacement time, repair labour/parts and disruption;
- adaptive change cost across declared household/climate/material scenarios.

### P3 — resident and professional evaluation
- resident-specific requirement coverage and preference retention;
- blinded pairwise comparisons of alternatives with consent and declared participant cohorts;
- task success, accessibility journey completion, comprehension of uncertainty and revision acceptance;
- private/shared intent and dissent preservation for multi-resident cases;
- qualified professional review for actual geometry, fire, structure, services, accessibility and municipal approval;
- VLM judgements used only as one calibrated advisory measure, compared against professional/resident annotations and prompt perturbations.

### Proposed efficiency target: research against the October 2026 surrogate result

A proposed research target—not an achieved result—is to reduce expensive simulation calls by at least 90% versus exhaustive search **while retaining at least 95% of the exact high-fidelity hypervolume on tractable test problems**, with calibrated uncertainty and independent high-fidelity re-evaluation of final alternatives. Report both savings and loss of solution quality; a low simulation count alone is not success.

## 5. Baseline matrix and experimental protocol

For each benchmark task, freeze:
- exact source brief, site/scenario, jurisdiction and climate profile;
- exact product/model/solver version, settings, rule repository and date;
- candidate encoding and decision variables;
- objective definitions, units, uncertainty and hard/soft boundaries;
- wall-clock and solver-call budgets;
- information available to each comparator;
- allowed human setup/intervention time;
- test split and any training/reference corpus overlap.

At minimum compare:
1. incumbent professional workflow;
2. relevant commercial design workflow (Finch3D/TestFit/Forma/Revit only where matched);
3. one-shot generator;
4. generator + deterministic verifier;
5. generator + verifier + iterative feedback (LayoutGen-style);
6. multi-objective evolutionary baseline (NSGA-II; include a second suitable algorithm such as MOEA/D or NSGA-III where applicable);
7. multi-fidelity/VOI search;
8. hybrid professional + compiler workflow.

Do not reuse the same cases for model selection and final evaluation. Publish failure cases and confidence intervals, not only best runs. Freeze external datasets/splits and comply with their licenses; RPLAN is not assumed redistributable.

## 6. What the current code increment now adds

The Sol-Atlas expression branch adds:
- 20 style-expression families;
- 26 optional experience packages;
- one resident-authored synthetic Highveld/Solarpunk expression profile;
- ten impact domains per selected package, with UNKNOWN preserved;
- a 24-case/4-pair contract-mutation extension;
- a proposed 46-row style/package-to-revision crosswalk which refers changes back to #832 and forbids automatic invalidation/approval;
- Rust qualification and CI checks for schema/catalog/benchmark/crosswalk integrity.

The crosswalk is only a proposed owner-review mapping. It does not run the revision invalidation engine, generate geometry, measure physical performance or approve a design. All physical impact values in the synthetic profile remain UNKNOWN.

## 7. Immediate execution sequence

1. **Completed in the current feature branch:** versioned 20-style/26-package catalog, 46-row proposed revision-impact crosswalk, deterministic owner/catalog checks, and exact catalog/brief digest binding in the resident expression profile.
2. **Completed for the currently extant synthetic source only:** all 24 expression mutation rows are bound to `SYNTH-HV-CASE-001` and the exact Home Freedom brief/candidate-set blob digests. They are counterfactual mutations of one baseline, not 24 independent households. The #819 manifest still says the planned 100-case corpus is `NOT_EXECUTED`; extend that population only when independently authored, versioned source cases actually exist.
3. **Decision receipt contract added in Sol-Atlas #917:** exact profile/brief/candidate-set/revision binding and fail-closed candidate dispositions; the current source candidates remain conditional with no resident winner.
4. Compose Symthaea #7790/#7801's multi-objective search with Symtropy #1766/#1848's physical/resilience receipts; keep one optimizer and one physical-evaluation authority.
5. Build exact-enumeration oracles on small, independently authored candidate spaces and report hypervolume/IGD+/regret only after all required gates pass.
6. Create an IDS differential conformance suite under the existing openBIM owner; compare identical IFC+IDS fixtures across checker versions because 2026 buildingSMART survey users report cross-tool result differences.
7. **Interactive resident configurator prototype added:** `/homes/express` reads the versioned 20-style/26-package catalogue, captures local-only style/experience preferences and shows an unvalidated JSON working draft. It is not a floor-plan generator or validated resident study.
8. **Resident agency evaluation protocol opened in Sol-Atlas #925:** pre-register consented tasks for preference comprehension, UNKNOWN/conditional understanding, choice influence, privacy, accessibility, correction and withdrawal. No benefit claim until the study is conducted and independently analysed.
9. Select one tractable starter typology and run matched high-fidelity EnergyPlus 26.2.0 scenarios only after exact input package qualification.
10. Run a paid, consented resident/architect trial before human-benefit claims or product comparisons.

## 8. Research and standards sources

- Kembhavi et al. (2026-10-15), LayoutGen: https://link.springer.com/article/10.1007/s44163-026-02481-9
- Uddin, Khanna & Aman (2026-09), interactive DNN + NSGA-III building optimization: https://www.sciencedirect.com/science/article/pii/S3051052X26000768
- Romeos et al. (2026-10-01), residential EnergyPlus + multi-objective decision support: https://www.sciencedirect.com/science/article/pii/S0378778826009370
- Generative urban design systematic review (2026-10-01), community engagement absent in 77% of 108 studies: https://doi.org/10.1016/j.scs.2026.107864
- Huang et al. (2025-12-17), 153-study performance-driven generative design systematic review: https://doi.org/10.3390/buildings15244556
- Talami, Costa & Stouffs (online 2026-10-20), uncertainty-informed multi-objective building design: https://doi.org/10.1016/j.enbuild.2026.118417
- Brama et al. (online 2026-06-14), evaluation challenges in generative urban design: https://doi.org/10.1177/14780771261459705
- buildingSMART IFC 4.3.2.0: https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/
- buildingSMART IDS 1.0: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- ISO 16739-1:2024: https://www.iso.org/standard/84123.html
- EnergyPlus 26.2.0: https://energyplus.net/epupdate.htm
- buildingSMART IDS 2026 survey (186 responses; checker interoperability concerns): https://www.buildingsmart.org/ids-survey-results-2026/

