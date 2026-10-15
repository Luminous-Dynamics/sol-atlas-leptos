# Conflict-to-Supply-Chain Impact Propagation Pilot

- **Status:** `design_only`; source inventory and model qualification are not complete.
- **Owner surface:** Sol-Atlas research catalog, composing transport projection and trade-evidence work.
- **Catalog entry:** `global-conflict-and-supply-chain-resilience` in [the Earth systems domain catalog](../earth-systems-domain-catalog.json).
- **Purpose:** Trace evidence-supported pathways from disruption reports to commodity exposure and downstream humanitarian/market indicators without turning correlation into causality.

## 1. Research question

For one preregistered historical disruption window and one selected commodity family, can we reconstruct a reproducible chain from a documented event to direct transport evidence, aggregate trade exposure, and a downstream outcome series—and can we distinguish the affected case from matched unaffected controls?

The first case is deliberately **not selected yet**. Selection occurs only after exact source windows, rights, geographic support and an independent outcome series are checked. Do not select a case after inspecting which one produces the most dramatic output.

## 2. Layer model

Keep these layers independently toggleable and inspectable:

1. **Conflict / event observations.** Source-specific event identity, event definition, actors where appropriate, coordinates and spatial accuracy, event time, publication/reporting time, retrieval time, source release and known coverage limits.
2. **Direct disruption evidence.** Reports or records of a port, road, rail segment, facility, border crossing or other link being open, restricted, delayed or unavailable. Conflict near infrastructure is not a substitute for direct disruption evidence.
3. **Trade exposure.** Aggregate commodity flows retaining reporter, partner, flow direction, classification, period, aggregation, reporting/estimated state, unit, currency and valuation basis. Country-level flow values must not be represented as exact shipments.
4. **Physical logistics and facility context.** Typed routes/segments, ports, facilities, milestones and dependencies from Sol Atlas's existing renderer-neutral projection. Distinguish reported, proposed, unknown, stale, conflicted and scenario-only data.
5. **Downstream outcome context.** Humanitarian indicators, prices, availability or other selected outcomes only when exact indicators, temporal cadence, geography and upstream lineage are known.
6. **Counterfactual scenarios.** Explicit interventions such as a hypothetical route closure, delay or partial capacity loss. Each scenario retains the exact base snapshot, assumptions, model/profile version and outputs; it must never overwrite observed state.

Do not create one untyped graph where a news report, a port, a commodity, a physical shipment and a causal claim are interchangeable nodes. Cross-layer links need a typed relation and evidence/qualification state.

## 3. Data-source roles and rights gates

| Source | Intended role | Gate before use |
|---|---|---|
| UCDP | Versioned historical event observations | Freeze release/codebook and citations; preserve event definitions and attribution |
| ACLED | Separate event series for comparison | Verify API entitlement, version-specific terms, allowed uses and redistribution |
| GDELT | High-cadence report discovery | Deduplicate underlying reporting where possible; do not treat mentions as confirmed events |
| IMF PortWatch | Port/maritime-activity context | Freeze exact collection/query metadata; determine what activity metric means and whether reuse is permitted |
| UN Comtrade | Country/partner/product/period aggregates | Freeze classification and dimensions; preserve completeness, estimates, valuation and source-vintage semantics |
| HDX HAPI | Selected humanitarian indicators | Review underlying provider, exact indicator, licence, geography, release and beta/version caveats |
| Open Supply Hub | Facility discovery/matching for eligible supply chains | Obtain an eligible token/package; review terms for display, derived data, export and redistribution |
| GS1 EPCIS 2.0 | Event-interoperability profile for partner-provided records | Use only authorized partner feeds; record profile/version and mapping loss; do not assume a public data feed exists |

The catalog's source records are **metadata**, not acquired-data receipts. A source being listed, an API endpoint being visible, a request succeeding, or a checksum being present does not by itself establish an unrestricted licence, independent corroboration, publisher authenticity or scientific truth.

## 4. Claim ladder

Every node, relation and displayed impact path must identify its evidence class:

- **Observed:** directly measured or recorded by the described measurement/event system, with the measurement's own limits intact.
- **Reported:** a source reports the claim; its issuer and original reporting lineage remain visible.
- **Derived:** a deterministic or statistical transformation of exact parent evidence and a versioned method.
- **Inferred:** a model-supported hypothesis, including uncertainty and validity domain.
- **Scenario:** a hypothetical intervention and simulated consequence, visibly separate from current or historical state.
- **Unknown / conflicted / stale / unavailable:** first-class states; these must not collapse to zero, false or "no impact."

An inferred edge should carry parent evidence IDs, transformation/model identity, assumptions, interval/time basis, and a reason it was generated. Multi-source agreement does not become independent corroboration when the sources repeat the same upstream report.

## 5. Integration boundaries

- **Sol-Atlas:** render time-aware layers, dependency paths, coverage and "why shown?" evidence inspectors. Compose [economic geography #90](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/90), [trade comparison #91](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/91), [logistics explorer #106](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/106), and the in-progress [transport projection contract #111](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/111). Do not build a second renderer or canonical transport graph.
- **Mycelix:** own canonical evidence/artifact references, claim history, access/consent scope and domain-native supply-chain/logistics semantics. Sol Atlas consumes a versioned, read-only projection. See Mycelix's existing supply-chain, EPCIS/DCSA/ONE Record and logistics work; confirm exact current maturity before integration.
- **Symthaea:** compare explicit hypotheses and scenario alternatives using only available evidence; propose useful additional observations; expose disagreement and sensitivity. It must not infer a missing supplier, fill a blank trade record, or promote a model confidence score into truth.
- **Symtropy:** support deterministic route/resource-allocation experiments only after the network model, units, capacities and constraints are appropriate and independently checked. Do not treat rigid-body physics as a trade-flow or economic model.
- **Data services:** keep ingestion, storage, model execution and map rendering separate. The browser/WASM client consumes bounded, versioned projections rather than downloading large source rasters or unrestricted live feeds.

The first candidate for a concrete source-readiness pass is the [Black Sea Grain Initiative 2022–2023](BLACK_SEA_GRAIN_INITIATIVE_READINESS.md). It is intentionally selected for data-source inspection only; its case state remains `design_only` until exact source bytes, rights review, a qualified trade baseline and independent controls are available.

## 6. Validation protocol

1. Preregister the event window, commodity family, outcome indicators, expected direction(s), baseline, controls and exclusion rules before calculating results.
2. Freeze all source artifacts / API responses where permitted; record source IDs/releases, query filters, retrieval times, exact-byte hashes, parser/profile versions, time/CRS/unit metadata and rights review. If a source cannot be retained, record the exact access limitation and a reproducible locator without pretending there is an immutable snapshot.
3. Test temporal order. Event occurrence, public report, source publication, observation window and outcome publication are distinct times.
4. Compare the affected corridor/commodity with matched unaffected controls and a predeclared baseline. Control for seasonality and known unrelated shocks where data permits.
5. Run source ablation: remove GDELT, then each coded event provider; remove the port proxy; perturb classification/valuation assumptions; and inspect whether conclusions depend on a single source or an unsupported edge.
6. Preserve negative controls: unrelated commodities, unaffected routes/ports, placebo event dates, missing/late reports and contradictory event records.
7. Ask an independent reviewer to inspect source lineage, methods, geographic/time support, outcome independence, false positives and whether the conclusion exceeds the evidence.

A successful backtest qualifies only the named case, source vintages, model and outcome. It does not certify a general conflict-impact predictor.

## 7. Acceptance criteria

- [ ] The selected case and outcomes are chosen by a documented readiness gate, not visual appeal or post-hoc performance.
- [ ] Source/API versions, terms, rights, query parameters, temporal coverage, CRS and units are recorded per artifact.
- [ ] Event reports, direct disruption observations, aggregate trade values, inferred edges and scenario effects remain different types.
- [ ] A proximity-only relation cannot become a reported closure or causal edge.
- [ ] Missing/partial/stale inputs never become zero, unaffected, available or complete by default.
- [ ] Source-dependent or duplicated reporting is not counted as independent corroboration.
- [ ] A country-level trade record cannot be rendered as a specific shipment, manufacturer or bill of materials.
- [ ] Baseline, held-out period/case, unaffected controls, negative controls, calibration and error analysis are frozen and independently reviewable.
- [ ] Every displayed impact path can explain its parent sources, transformations, assumptions, time range and unresolved links.
- [ ] No live actuation, dispatch, automatic policy, tactical targeting, price forecast or causal-impact claim is enabled by this pilot.
- [ ] Structural tests, parser tests, independent calculation checks and exact-head CI are reported separately; queued jobs are not passes.

## 8. Immediate milestones

- **C0 — source readiness:** inventory and rights review only; no model output.
- **C1 — frozen historical evidence slice:** capture/validate the minimum source artifacts and outcome data for one case.
- **C2 — deterministic graph projection:** render the case with evidence classes, timestamps, coverage and unresolved links.
- **C3 — reproducible baseline/scenario comparison:** run explicit disruption scenarios without overwriting observed history.
- **C4 — independent retrospective evaluation:** controls, ablations, false positives, uncertainty and claim ceiling reviewed.
- **C5 — wider connector rollout:** only after C4, add another commodity, source or geography under the same source-qualified contract.

## References

- UCDP Dataset Download Center: https://ucdp.uu.se/downloads/
- ACLED API documentation: https://acleddata.com/acled-api-documentation
- GDELT Project: https://www.gdeltproject.org/
- IMF PortWatch Search API: https://portwatch.imf.org/api/search/definition/
- UN Comtrade API: https://comtradeapi.un.org/
- HDX HAPI documentation: https://hdx-hapi.readthedocs.io/
- Open Supply Hub API documentation: https://info.opensupplyhub.org/resources/api-documentation
- GS1 EPCIS: https://www.gs1.org/standards/epcis
