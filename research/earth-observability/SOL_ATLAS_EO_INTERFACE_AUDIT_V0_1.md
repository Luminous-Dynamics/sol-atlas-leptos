# Sol-Atlas Earth Observation Interface Audit v0.1

**Programme:** [PLANETARY-CHANGE-EVIDENCE-001 / issue #795](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/795)  
**Status:** source inspection only; no runtime/build/test execution  
**Observed default-branch snapshot:** main at 1af10d891410cce31c1d4037e7c7129c16db5834  
**Audit date:** 2026-11-14

This audit exists to prevent a duplicate geospatial asset registry, rights system, GIS or evidence graph. It distinguishes what is present on the observed default branch from designs and code that exist only on unmerged research branches.

## 1. Executive conclusion

Sol-Atlas has a meaningful spatial/rendering foundation and several well-designed parallel workstreams, but **the default-branch application is not yet a qualified Earth-observation evidence system**.

The missing bridge is not another globe, another SourceAsset type or another EVSI engine. It is a small, explicit composition of:

1. a stable, versioned source-asset contract;
2. the existing purpose-specific rights and STAC ingestion rules;
3. optical/SAR/thermal EO observation semantics and quality masks;
4. time-bounded change claims and independent reference labels;
5. frozen spatial/temporal/sensor split manifests and benchmark receipts;
6. a typed handoff to Symthaea/Symtropy/Mycelix with clear claim ceilings.

Until the existing pending source-asset and rights contracts are reconciled and reviewed, **do not add a second source-asset domain type**.

## 2. Inspected default-branch implementation

| Path / artifact | Observed role | What it does not establish |
|---|---|---|
| DEMO.md | Human-readable planetary energy-transition demo narrative, data-layer counts and example calculations | Exact dataset versions/digests, source rights, independently reproduced EROI/emissions/economic values, or proof that animated P2P arcs are live trades |
| sol-atlas-core/src/types.rs | Existing layer types and compact DataKind / DataProvenance metadata (Observed, Curated, Scenario) | A source-asset registry, exact asset/snapshot identity, bytes digest, purpose-specific licence grant, QA state or independently qualified observation |
| sol-atlas-core/src/data.rs | Parses static JSON arrays and selected GeoJSON feeds into application layer types | A benchmark-grade ingestion receipt or complete error/state propagation |
| sol-atlas-core/src/geo.rs | Globe-oriented latitude/longitude conversion, marker math and haversine distance | General CRS transformations, datum/epoch handling, topology or geodetic uncertainty |
| sol-atlas-core/src/timeline.rs | Energy-transition narrative timeline | EO acquisition-time intervals, observation-validity windows or bounded event-time inference |
| sol-atlas-core/src/gibs.rs | File defines a configured NASA GIBS Web Mercator tile URL builder, with explicit warning that it does not fetch/validate assets | Live layer metadata discovery, tile availability/quality, independent source authenticity, scientific truth or redistribution rights |
| sol-atlas-core/src/lib.rs | Exports the core modules inspected on main | The observed file list includes gibs.rs, but lib.rs at this snapshot does not declare pub mod gibs; do not assume this file is currently part of the crate API |

### 2.1 A key display-versus-evidence distinction

In data.rs, the application parser logs a malformed dataset and returns a default value (often an empty collection) so the renderer can continue. That is a reasonable resilience behavior for a visualization surface, but it must not be reused as an evidence ingestion semantic.

For benchmark/evidence paths, an empty collection cannot mean “no change” or “nothing exists.” The caller needs an explicit disposition such as unavailable, invalid, unobserved, obscured, out-of-domain or valid-empty-within-declared-query. Keep visualization fallback behavior separate from scientific/evidence status.

### 2.2 Current demo claim ceiling

The existing DEMO.md describes reserve/EROI/carbon layers, depletion timelines, grid stress, economic calculations and P2P energy arcs. These are valuable product directions, but in the default-branch source snapshot the demo narrative alone does not bind all claims to a source item/version, bytes digest, retrieval receipt, processing run, uncertainty model or independent validation.

Issue #791 already starts a source-attributed planetary energy map projection and appropriately says those demo claims remain source-declared unless independently established. This Earth-observation programme must preserve that discipline and must not silently promote the current visualization into a validated Earth model.

## 3. Existing pending contracts — compose, do not fork

| Existing work | Observed state | Reuse / required reconciliation |
|---|---|---|
| Issue #65 — geospatial asset catalog | Open issue; acceptance criteria define stable AssetId, asset kind, source, licence review, footprint, time and resolution. | Canonical asset identity and metadata contract should live here / its PR, not in a second EO-specific registry. |
| PR #67 — validated geospatial asset catalog | Open draft PR against main; head 223f53b5208831670cbabfc7d0e6dbbfb07780f0. It adds a substantial renderer-neutral Rust model, including AssetId, footprint, observation-time kind, licence metadata, resolution and source locator. | Review and qualify this branch before designing another catalog. The visible draft implementation is still not a fetched-byte authenticity proof or a purpose-specific rights authorization decision. |
| Issue #240 — OGC/STAC discovery/ingestion | Open issue; defines read-only discovery, source/CRS/axis-order/QA/rights metadata, manifests and explicit OGC/STAC boundaries. | Make this the source-discovery adapter owner. Discovery is not admission; a public URL is not permission. |
| Issue #430 — rights-aware STAC/source-asset registry | Open issue; references source-asset and candidate schemas on the unmerged heritage-intelligence branch; defines purpose-separated permissions, immutable byte/metadata lineage and fail-closed egress. | Reconcile the schema with #65/#67; do not clone rights/grants/custody in the EO benchmark. Mycelix remains governance/custody authority. |
| Issue #771 — sensor observability qualification | Open programme; PR #775 is an open draft on research/earth-observability-soe-v0-1, with an analytic POD harness and explicitly unqualified fixture results. | Reuse its split isolation, non-detection semantics, uncertainty, negative-control and equal-budget tournament patterns. It concerns ground geophysical sensors; its GPR/ERT likelihoods must not be treated as satellite-image likelihoods. |
| PR #133 / issue #134 — investigation EVSI | Open draft implementation plus spatial/sequential investigation roadmap. | Reuse the existing discrete EVSI kernel and sampling research. Do not add another EVSI calculator to the EO path. |
| PR #138 — built-environment evidence graph | Open PR with exact artifact lineage, solver receipts, decision events, supersession/revocation and revalidation concepts. | Reuse relevant receipt/revision patterns by reference. It does not itself qualify EO source truth, labels or satellite observations. |
| Issue #791 — planetary-energy map projection | Open source-attributed projection and claim-boundary work. | Connect the existing energy map to the same source/claim discipline, but do not treat every current demo layer as EO evidence. |

**Important branch boundary:** PR #67, PR #429/#430 and PR #775 are not merged into the observed main snapshot. A design or implementation visible on those branches is not a default-branch capability. This audit does not claim that these branches are invalid; it says they need review and exact-head qualification before being treated as integrated.

## 3.1 Cross-project programme audit — important existing work

A broader repository search found that the satellite-specific programme must be composed with several already-active workstreams. These are not optional future ideas; they are existing issues/draft PRs and materially change the integration plan.

| Existing work | Current owner and scope | How this satellite programme composes |
|---|---|---|
| Sol-Atlas #761 / draft PR #762 — EARTH-OBSERVABILITY-QUAL-001 | Multimodal terrestrial/subsurface observability; coordinate/datum/source contracts, non-detection semantics, synthetic cases, POD/abstention, BGS/AGS/UK Geo data handoffs, rights-aware federation and investigation evaluation. The issue documents seven tracks and an 18-case challenge manifest. | Reuse the qualification, source, uncertainty, missingness, evidence-tier and decision principles. Satellite optical/SAR likelihoods, temporal semantics and change labels need their own modality-specific profile; do not reuse the current GPR/ERT / borehole transfer rates as satellite detection performance. |
| Sol-Atlas #798 — SCI-OBS-EARTH-001 | Mapping contract between Sol-Atlas Earth/terrestrial evidence and the federated Mycelix scientific-observability fabric. | Treat this as the Sol-Atlas↔Mycelix integration parent. The satellite programme is a modality-specific child, not a competing cross-project bridge. |
| Sol-Atlas #783 — UT-Gold-D admission | Blinded retrospective GPR field dataset admission, truth tiers, site-level split and rights/disclosure constraints. | Reuse its label-tier and cluster-leakage discipline. It is not satellite imagery ground truth and its GPR observations do not qualify optical/SAR change detection. |
| Symthaea #194 — Planetary Perception / Wetland Watch | Locked Sentinel evaluation witness: content-addressed data manifest, spatial/time holdouts, training/calibration/evaluation separation, fitted-artifact and model-selection integrity, custody/reveal controls, classic baselines and coverage-aware scores. | This is the existing Sentinel benchmark acceptance authority. Reuse its split, fit, selection and custody manifests; do not create a second evaluation-leakage framework. The present EO claim module is an input/claim contract, not a substitute for that locked witness. |
| Symthaea #12075 — SCI-OBS-SYM-001 | Evidence-bound anomaly triage, competing hypotheses, testable predictions and recommendation of admissible follow-up observations. | Symthaea owns reasoning and hypothesis competition. Sol-Atlas supplies qualified spatial/time claims and benchmark evidence; it must not implement another generic hypothesis engine. |
| Mycelix #6141 / draft PR #6142 — SCI-OBS-001 | Domain-neutral federated scientific-event provenance, evidence artifacts, ownership, custody, rights, signed events, dependency-aware corroboration, corrections/retractions and federation. | Mycelix owns the shared provenance/federation and authority layer. Sol-Atlas owns native Earth spatial semantics, observation support, change claims and EO evaluation; keep imagery payloads with source owners where possible. |
| Symtropy #1760 / #1728 — GEO-OBS / GEO-SIM | Active observation design and known-truth geophysical inverse-problem tests for the Earth Interior Observatory. | Reuse the general experimental-design and non-identifiability principles where relevant. These are geodynamo/geophysical inverse problems, not remote-sensing change-detection baselines. |

### Updated ownership map

- **Sol-Atlas:** native EO assets/observation references, spatial support, temporal/change claims, source-to-claim traceability, split manifests, metrics/receipts and map/timeline presentation.
- **Symthaea:** anomaly triage, competing causal hypotheses, predictions, model criticism and follow-up ranking; consume only qualified evidence and preserve abstention.
- **Symtropy:** modality- and domain-specific physical forward models, model applicability, simulation/measurement discrepancy and post-intervention outcome prediction; never turn simulation output into observation.
- **Mycelix:** signed scientific-event/provenance envelope, source rights/custody, subject/purpose authorization, actor/organization authority, dependency-aware corroboration and correction/retraction.
- **Planet / Copernicus / USGS / GEE / Microsoft Planetary Computer / other source platforms:** source data, imagery, discovery or compute as licensed; none becomes the sole truth authority.

The practical consequence: first complete the Sol-Atlas asset/STAC/rights and Sentinel evaluation contracts already in flight, then connect the narrow EO claims to them. The new satellite work should not absorb Mycelix's shared event ledger, Symthaea's investigation engine, Symtropy's physics models, or Sol-Atlas's existing geophysical sensor qualification.

## 4. What the EO programme should add

The EO-specific delta is much narrower than a second infrastructure stack:

1. **An observation/claim layer over canonical assets.** Reference the canonical asset ID and exact version/receipt; do not copy provider metadata into a parallel registry.
2. **EO sensor semantics.** Preserve modality, bands/polarization, product/collection, acquisition geometry, atmospheric/geometric quality flags, QA mask and sensor/processing version. Ground-sensor POD contracts are not satellite likelihoods.
3. **Space-time change statements.** Bind the support and temporal interval, class/ontology version, minimum mapping unit, geometry/area uncertainty, supporting and contradicting observations, and claim disposition.
4. **Independent reference-label lineage.** Distinguish field-assessed, independently adjudicated, catalogue-derived, weak/model-derived and unknown labels. Never promote unknown/unvisited to negative.
5. **Evaluation split and receipt manifests.** Freeze site/region/event, time and sensor groups before model selection. Record exact input digests, code/model versions, score outputs and independent adjudication.
6. **Decision handoff.** Pass qualified observation likelihoods and candidate investigations into the existing EVSI kernel only when both true-positive and false-positive behavior are qualified for the precise target, sensor, support, environment and configuration.
7. **Post-decision outcome links.** Bind authorized action and follow-up observation to the exact hypothesis/prediction; record predicted versus measured result and adverse outcomes.

## 5. Recommended implementation order

### P0 — contract reconciliation (documentation/review)

- [ ] Review PR #67 against issue #65 acceptance criteria; establish canonical stable AssetId, temporal metadata and serialization semantics.
- [ ] Review the #430 source-asset schema and rights rules against the #65/#67 model; resolve naming, purpose permissions, digest identity and unknown states once.
- [ ] Review #240 adapter boundaries so STAC raw JSON, provider IDs, extension fields and OGC service metadata survive ingestion.
- [ ] Specify an EO-specific observation/change-claim schema that references canonical asset IDs and Mycelix authority/custody references rather than re-defining them.
- [ ] Keep visualization fallback (parse_or_log) separate from qualification and evidence outcomes.

### P1 — small offline EO fixture/runner

- [ ] Build synthetic, frozen two-date and multi-date scenes (not real private data) with a known change polygon, no-change control, seasonal change, cloud/shadow, no-data, sensor-generation change and registration shift.
- [ ] Give the fixtures explicit spatial support, acquisition intervals, quality masks, source identities, and train/calibration/blind-test splits.
- [ ] Implement the first reference baseline as persistence/no-change plus a transparent spectral-index rule, where sensor/band support permits.
- [ ] Score known answers and negative controls in an offline runner; preserve NOT_RUN for any unimplemented statistical or physical qualification.
- [ ] Add spatial/temporal cluster-level uncertainty only after the independent unit design is frozen.

### P2 — public retrospective evaluation

- [ ] Admit only source assets whose exact product/collection, rights, labels and metadata are established.
- [ ] Compare matched baselines on identical splits and budgets.
- [ ] Run leave-region, leave-time, leave-sensor and resolution-shift evaluation.
- [ ] Publish pooled, per-class and worst-domain results; include calibration, selective risk, area error, runtime and cost.

### P3 — permissioned prospective pilot

- [ ] Use a bounded South African site/catchment after confirming land access, source/data rights, independent reference labels and qualified reviewers.
- [ ] Keep SANLC 2020 as weak/external comparison only until its authoritative data licence and its dependence on Sentinel-2-derived labels are resolved.
- [ ] Run the pre-registered protocol before opening blind labels.
- [ ] Compare later observations with predictions after any separately authorized action.

## 6. Immediate critical risks to close

1. **Asset model duplication:** #65/#67 and #430 are overlapping but not identical; pick one canonical identity and a narrow composition contract.
2. **Rights conflation:** licence metadata, Mycelix authority/consent, access tier, data custody and export permissions are distinct dimensions. A single licence_review enum must not imply all are authorized.
3. **Truth laundering:** DataKind::Observed means the source is observational, not that downstream interpretation is true or independently validated.
4. **No-data laundering:** renderer-friendly empty defaults must never enter the benchmark as a negative or no-change label.
5. **Time laundering:** acquisition instant, composite period, event-time bound and retrieval/processing time must remain different.
6. **Scale laundering:** map/display resolution, sensor native GSD and the minimum mapping unit of a claim must remain distinct.
7. **SOTA laundering:** synthetic fixtures, random pixel splits, source-derived labels and unverified vendor outputs cannot support broad superiority claims.

## 7. Exit criteria for this audit

This is complete as a source inspection note only. The audit becomes an implementation qualification only when:

- the canonical asset and rights schemas are reconciled;
- the EO observation/change-claim contract references these canonical records;
- deterministic fixtures exercise all required epistemic states;
- full schema validation and offline runner tests pass on an exact commit;
- independent held-out data and reference-label rights are documented;
- results distinguish synthetic, retrospective, blind and prospective evidence tiers.

No build, Cargo test, WASM check, satellite download, Planet API request, model evaluation or physical survey was run for this audit.
