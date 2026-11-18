# Home Expression Grammar v0.2 — research and benchmark protocol

**Issue:** [HOME-ARCHITECTURAL-GRAMMAR-002 #934](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/934)  
**Parent work:** [Home Expression Library #904 / PR #906](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/906)  
**Status:** Contract, catalogue and synthetic fixture proposal; not a design generator  
**Date:** 2026-11-18

## 1. Paradigm shift: from style presets to a composable architectural grammar

A style is an expressive reference, not a building system. Home Expression v0.2 therefore separates five axes:

1. **Spatial typology:** detached, courtyard, row/terrace, duplex, apartment, pavilion, clustered/multi-generational, live-work, incremental-core and retrofit-first arrangements.
2. **Architectural expression:** form, composition, proportion, ornament, colour, light, thresholds and symbolic intent. The catalogue supplies reference families; residents can blend, reject, rename or leave them undecided.
3. **Material and tectonic intent:** stone, brick, earth, timber, steel, hybrid and repairable assembly preferences. A material name is not a specified assembly, EPD, structural detail or procurement proof.
4. **Environmental strategy intent:** orientation, shading, daylight, ventilation, thermal mass, water strategies and landscape relationships. A style's appearance never proves that one of these strategies works on a specific site.
5. **Experience and programme:** optional spaces and services that change area, structure, fire, services, access, cost, water, energy, sourcing and maintenance.

The composition contract preserves unresolved conflicts. It does not collapse axes into an aesthetic score, infer a style from resident identity, or automatically map a collage of references into geometry. The v0.2 catalogue now publishes the shared vocabularies as versioned arrays: 11 spatial typologies, 34 expression families, 8 material/tectonic options, 10 environmental strategies and 26 experience packages. The configurator consumes these same IDs rather than maintaining a separate UI-only list; Symthaea adapters should consume this versioned catalogue when their owner accepts the contract.

### New catalogue families

The 20 v0.1 IDs are retained. Fourteen additional expression families are added: Cape Dutch historic reference; Cape Georgian influence; Victorian; Edwardian; Georgian/Neoclassical; Arts & Crafts/Craftsman; Art Nouveau/Jugendstil; Prairie School; Bauhaus; International Style; Nordic/Scandinavian; Spanish Colonial/Mission; Alpine/Chalet; and Postmodern.

Two important classification choices:
- **Karoo corbelled-stone architecture is not a general-purpose style preset.** It is a separately typed, conservation-only Great Karoo heritage reference. UCT research documents its nineteenth-century form, complex frontier context and contested/multiple heritage associations. Exact asset provenance, local community engagement and conservation review remain necessary. Sources: Patricia Kramer (2012), [UCT thesis](https://open.uct.ac.za/handle/11427/12087); Caroline Hancock (2018), [UCT thesis](https://open.uct.ac.za/items/5d247a02-4bab-4a92-8f6f-2ce9d5fb04fb).
- **No generic "African vernacular" preset is introduced.** Community-specific grammar modules require attribution, community-authorized description, source provenance, rights, review and a correction/withdrawal route. Cape vernacular scholarship itself describes a layered tradition with place, material, technology and cultural influences, including Georgian shifts; it should not be flattened into an unchanging motif set. [University of Pretoria source](https://repository.up.ac.za/server/api/core/bitstreams/fc087af1-69b5-4f2f-9e74-9ed9fe58716e/content).

## 2. What is new versus what is deliberately not built

This increment adds:
- additive v0.2 catalogue, keeping v0.1 byte-for-byte and digest-pinned;
- separate JSON Schema for the catalogue and five-axis grammar composition;
- a synthetic Highveld grammar example whose site/climate/jurisdiction evidence remains UNKNOWN;
- the resident configurator's catalogue selection updated to v0.2, with an explicitly unvalidated local draft;
- deterministic Rust integrity checks and mutation self-tests.

This increment does **not** add:
- floor-plan or 3D geometry generation;
- structural, fire, moisture, energy, water or acoustic calculations;
- quotes, cost estimates, bills of quantities or supplier availability;
- municipal approval, heritage consent or professional sign-off;
- persistent resident data, user authentication or household consensus.

The UI remains local reactive state only. The v0.2 grammar fixture is a contract example, not a buildable proposal. The existing v0.1 profile, decision receipt, synthetic source and qualifier remain pinned to v0.1.

## 3. Industry and research baseline — matched scopes only

| Comparator | Demonstrated product/research scope | Fair test | Our differentiation target |
|---|---|---|---|
| Finch | Generative building layouts, firm plan libraries, unit/floor-plan exploration and BIM workflows | Same brief, floor plate, plan-library access, code assumptions, output editability and human setup budget | Preserve resident intent and provenance across candidate, decision and later evidence revisions |
| TestFit | Site/development feasibility, building configuration, unit mix, quantities and early cost workflows | Same parcel, zoning evidence/date, unit mix, quantity scope and rate source | Whole-household service, lifecycle and affordability outcomes; no opaque assumptions |
| Autodesk Forma | Early massing/site analysis including sun, daylight, wind, microclimate, noise, solar and carbon studies | Same geometry, site, weather file, date, terrain and analysis scope | Carry site evidence into resident-led alternatives and downstream qualification |
| Hypar | Programmable design logic, test-fit and space planning with metrics and Revit handoff | Same program, geometry exchange, team edits and output handoff | Keep the grammar declarative, open, resident-owned and independently verifiable |
| LayoutGen (2026 research) | Retrieval-grounded floor-plan generation with deterministic checks for selected topology/geometric/dimensional rules | Same dataset split, plan type, encoded rules, generation budget and blind evaluator | Expanded evidence ownership, multiple hard-gate domains, uncertainty and resident control; report scope mismatch |
| Talami et al. (2026) | Surrogate-assisted multi-objective office optimization; paper reports 93.8% fewer simulations than exhaustive runs in one case | Re-run on same case/model and compare against exact oracle and declared exhaustive budget | Test robust multi-fidelity search without weakening high-fidelity final verification |
| EnergyPlus / OpenStudio | Building energy and water simulation | Identical geometry, constructions, weather, occupancy, schedules, systems and solver version | Do not claim to beat physics engines; improve the search/decision process around them |
| IFC / IDS and professional BIM workflow | Open model exchange and computer-interpretable information requirements | Identical models, IDS files, software versions and round-trip task | Exact lineage, loss reports and claim-scoped evidence that survives tool changes |

Official/product descriptions: [Finch](https://www.finch3d.com/product), [TestFit](https://www.testfit.io/), [Autodesk Forma](https://www.autodesk.com/za/products/forma/environmental-impact-analysis), [Hypar](https://docs.hypar.io/). Product marketing statements are not independent benchmark results.

Relevant published baselines:
- LayoutGen reports 3.7 to 0.3 measured violations per plan across 200 runs, but only for the specific checks in its loop; the paper explicitly excludes some whole-building/site checks from that count. [Kembhavi et al., 2026-10-15](https://link.springer.com/article/10.1007/s44163-026-02481-9).
- Talami, Costa and Stouffs report 93.8% fewer simulations in one Nottingham office case, with surrogate accuracy and low deviation reported for their studied objectives. This is a strong search-efficiency baseline, not a residential/South-African generalization claim. [Energy and Buildings, online 2026-10-20](https://www.sciencedirect.com/science/article/pii/S0378778826014775).
- The available [Talami et al. manuscript](https://papers.ssrn.com/sol3/Delivery.cfm/cf644962-4e07-468a-8c8f-d49ec1862029-MECA.pdf?abstractid=6981724&mirid=1&type=2) reports 85–92% coverage of the true Pareto front; use the manuscript's exact definition when reproducing this benchmark.
- EnergyPlus 26.2.0 was released 2026-09-30. Pin its executable/package digest and all inputs. [EnergyPlus release notes](https://energyplus.net/epupdate.htm).
- ASHRAE Standard 140 is a method for evaluating building-performance simulation software; it should anchor solver verification rather than an ad hoc model leaderboard. [ASHRAE/IBPSA resources](https://www.ashrae.org/technical-resources/bookstore/ashrae-building-energy-simulation-and-performance-modeling-resources).
- IFC 4.3.2.0 is the official IFC 4.3 release, published as ISO 16739-1:2024. IDS 1.0 is an official information-requirement standard, but its first version does not validate geometric rules; complement it with geometry-aware checks rather than overclaiming. [buildingSMART IFC](https://www.buildingsmart.org/standards/bsi-standards/industry-foundation--classes/), [ISO 16739-1:2024](https://www.iso.org/standard/84123.html), [IDS](https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/), [IDS limitations](https://github.com/buildingSMART/IDS/blob/development/Documentation/UserManual/specifications.md).

## 4. Benchmark suite and proposed targets

All targets below are **research targets**, not observed results. Report each separately; never combine them into one weighted score.

### P0 — correctness, rights and resident intent
- 0 false-feasible / false-PASS cases on frozen hard-gate oracles.
- 100% preservation of explicit non-negotiables, avoided motifs, undecided fields and dissent.
- 100% source/revision/method identity for any claim-bearing evidence.
- 100% rejection or quarantine of missing rights, withdrawn references, unbound heritage claims and stale evidence.
- 0 automatic promotion from style/intent to geometry, cost, climate performance, code compliance or heritage approval.
- Positive controls must demonstrate that valid, evidence-complete candidates can pass; an always-blocking system is not acceptable.

### P1 — grammar quality and compositional behaviour
- 100% of valid grammar fixtures preserve independent axis identity; 0 silent conversion of typology into style or material preference into a specified assembly.
- 100% detection of seeded incompatible/underspecified combinations; unresolved cases remain visible rather than silently "fixed".
- At least 3 genuinely distinct topology/functional candidates on a tractable non-overconstrained brief, with duplication measured structurally rather than by render similarity.
- ≥95% retention of resident-stated requirements across candidate revision; 100% of removed or transformed requirements must have a traceable explanation and resident-visible status.
- Counterfactual locality: changes should propagate only through declared dependencies; publish the exact affected/unaffected receipt set.

### P2 — search and physical performance
- **Stretch target beyond the 2026 office-optimization baseline:** on a matched evaluation protocol, target at least 95% exact-oracle Pareto-front coverage and no more than 5% of exhaustive high-fidelity simulation calls. Talami, Costa and Stouffs report 93.8% fewer simulations in one Nottingham office case; the available manuscript reports 85–92% true-front coverage. Our threshold is a research target for held-out residential/context scenarios, not a claim of superiority. Reproduce the reference case first and compare using the paper's exact coverage definition before making a cross-paper claim.
- Against an exact-enumeration oracle on small discrete cases: ≥95% of exact-oracle hypervolume and ≥95% exact-front coverage, while using ≤5% of exhaustive high-fidelity simulation calls. Use the same published front-coverage definition as the strongest matched paper baseline, fixed reference points, seeds, budgets and uncertainty sets. This is an aspirational target, not an achieved result.
- Calibrate surrogate uncertainty by climate, typology and assembly; out-of-distribution inputs must abstain. Independently re-run the final Pareto candidates with the high-fidelity solver.
- Report hypervolume, IGD+, epsilon indicator, feasible-set recall, scenario-wise regret, wall time, energy use by end use, peak demand, comfort hours, water balance and lifecycle cost separately.
- Complete ASHRAE 140 testing before claiming building-simulation fidelity. A surrogate is an exploration aid, not a replacement for the qualified solver.

### P3 — interoperability and professional workflow
- For frozen IFC+IDS fixtures, report import/export round-trip property retention, geometry deviations, classification preservation and semantic-loss receipts. Initial target: ≥99.5% required-property retention, with zero silent loss of required items.
- Keep IDS information checks distinct from geometry, dynamic calculations and jurisdiction-specific professional reviews.
- Compare against an incumbent architect/engineer workflow on the same brief, time budget and deliverable maturity; report human setup time, intervention count, rework, error classes and review outcome.
- Do not make permit, safety or professional-practice claims from benchmark fixtures.

### P4 — resident experience and legitimacy
- Use consented and compensated residents; pre-register tasks and analysis.
- Measure requirement-comprehension, ability to express "none/undecided", preference correction, perceived influence, accessibility task completion, privacy comprehension and dissent preservation.
- Compare decisions to resident-declared priorities, not synthetic LLM personas or a universal satisfaction score.
- Determine sample size by a power analysis for the chosen outcomes; do not invent a user-study result from a prototype.

## 5. Execution order

1. Freeze v0.1 and make v0.2 additive.
2. Validate the v0.2 catalogue and grammar schema with deterministic mutation tests.
3. Add the new catalogue to the local-only configurator; retain an explicit unvalidated draft status.
4. Extend the grammar compiler in Symthaea #7790/#7801 only after its owner accepts the contract; do not add a second optimizer in Sol-Atlas.
5. Route realized geometry, simulation and spatial constraints to Symtropy #1766/#1790/#1848 and existing owner APIs; no duplicate solver.
6. Compose with Sol-Atlas Home Freedom and #832 revision invalidation; exact changes, not intent labels alone, determine re-evaluation.
7. Build a small exact oracle and independent benchmark fixtures before large simulation campaigns.
8. Conduct a resident/architect evaluation only with consent, compensation and a pre-registered protocol.

## 6. Status ledger

- Catalogue v0.2 and grammar contract: **authored / not yet exact-head CI verified**.
- UI: still an inspiration-only, local working draft; no save, geometry, price, or qualification.
- Synthetic grammar fixture: **synthetic / not evaluated**.
- Physical, cost, climate, code, heritage and professional validation: **not performed**.
- Benchmark comparisons: **protocol and targets only; no SOTA victory claimed**.
