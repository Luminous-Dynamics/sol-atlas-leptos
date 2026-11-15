# Ecological Reference Ensembles and Recovery Trajectories v0.1

**Status:** research protocol and synthetic qualification corpus; no real ecosystem reference model is yet admitted.
**Issue:** [#856](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/856)
**Related:** IOEQ #853 / PR #855; PCIT #845 / #846; Jukskei #847 / #852.

## 1. Why static baselines are insufficient

A restoration target should not be defined as a single historical map or a convenient modern green site. The SER Third Edition (2026), Principle 3, describes reference models as the approximate state a native ecosystem would exhibit in the absence of degradation while accounting for past and anticipated environmental change. It explicitly says reference models should not immobilize ecosystems at a fixed point in time.

Reference sites can themselves be degraded, may lack species or functions, and can create shifting/declining baselines. Ecosystems also respond to changing climate, hydrology and disturbance; successional pathways and alternative states may be plausible. The model must express these uncertainties and alternatives rather than force one endpoint.

## 2. Dynamic reference ensemble

A reference ensemble is a versioned set of compatible reference models and reference sites, not a single polygon. Each model records local ecosystem identity, spatial strata, multiple reference sites, site degradation screening, measurement provenance, climate/hydrological regime, succession pathways, six attribute indicator sets, knowledge provenance, rights and supersession history.

### 2.1 Six independent ecosystem attributes

Preserve six separately assessed dimensions:
1. Absence of threats.
2. Physical conditions.
3. Species composition.
4. Structural diversity.
5. Ecosystem function.
6. External exchanges and connectivity.

Where feasible, aim for at least three indicators per attribute and ideally at least one per sub-attribute. This is a design target, not permission to fabricate missing observations. Explicitly mark unmeasured dimensions and attribute-level uncertainty.

### 2.2 Reference-site qualification

Use multiple sites and other sources of evidence. Screen candidate sites for degradation drivers, invasive dominance, altered hydrology, missing native species/functions and other relevant pressures. Record the selection rationale and common-cause dependencies. A site is not a valid reference merely because it is nearby or visually greener.

### 2.3 Climate and hydrological conditioning

Record climate and hydrological conditions relevant to the ecosystem and indicator. If nonstationarity is detected, either adjust the reference envelope or explicitly report that only sensitivity analysis was performed. Include scenario IDs, time horizon, rainfall/temperature/flow context and model uncertainty. Do not silently compare future observations to a stationary past climate.

### 2.4 Succession and alternative trajectories

Reference models may require multiple stages or plausible states. Store path IDs, assumptions, environmental conditions, succession stage, plausibility and expected time scale. An assessment should not reject a scientifically plausible alternate path simply because it differs from one deterministic historical endpoint.

### 2.5 Shifting baselines and versioning

Screen historical and modern references for baseline drift. Version model changes, preserve the previous model, identify affected descendants, document the reason for changes and require an explicit impact review before reusing old recovery conclusions against a new reference model.

## 3. Recovery trajectory evidence

Observations are attached to an exact reference-model version, attribute, indicator, measurement profile, spatial/temporal support, method, source lineage and uncertainty. Report per-indicator and per-attribute trajectories, missingness and evidence dependencies.

Do not compute a single arithmetic ecosystem-health score by default. In particular, tree canopy gains must not silently compensate for worsening native species composition, hydrological function, threat levels or connectivity. If a project-specific composite is requested, publish raw attributes, weights, sensitivity and non-compensation rules.

## 4. Knowledge, consent and governance

Scientific, practitioner, local ecological and Indigenous knowledge can all inform a reference model. They are not interchangeable, nor should community observations be treated as free training labels. Record source identity, knowledge type, consent status, permitted purposes, attribution and withdrawal/supersession. Unknown or restricted consent is a hard use-purpose gate.

## 5. Jukskei application

Do not assume one catchment-wide pristine historical channel is the restoration target. First distinguish river reaches, riparian ecosystem types, modified urban reaches, wetlands, floodplain fragments and intervention sites. Establish local ecosystem classification and realistic restoration/rehabilitation intent per reach.

Candidate data streams include the Water for the Future / City of Johannesburg upper-headwaters monitoring station, SUNCASA's reported hydrological monitoring network, the Upper Jukskei EcoFilter, Landsat/Sentinel observations, community drone surveys, field surveys, laboratory analyses and local ecological knowledge. Every stream is a different measurement operator and must retain its own support, calibration, cadence, rights and dependencies.

The Water for the Future page documents a headwater station with water-quality sampling, integrated flow measurement and meteorological instrumentation, and says Bureau Veritas performs water analyses. Its public page also exposes sample E. coli and chemical reports. This is a concrete data-access lead, not an assertion that Sol-Atlas has obtained its raw time series, methods or reuse permission.

## 6. Benchmark and evaluator

The synthetic corpus tests:
- degraded reference sites and shifting baselines;
- climate nonstationarity without climate-conditioned reference adjustment;
- alternative succession pathways underrepresented by a single path;
- attribute trade-offs where one attribute worsens despite overall positive wording;
- undercoverage of indicators across the six attributes;
- local ecological knowledge with restricted purpose/consent;
- unresolved reference-model supersession;
- unknown degradation status of included reference sites;
- positive overall recovery claims despite unobserved attributes.

The offline Rust evaluator validates the schema, exports an agent-safe view without oracle labels and reports model qualification blockers, trajectory-coverage blockers and non-compensatory attribute violations. It does not interpret imagery or establish field conditions.

## 7. Benchmarks

Report separately:
- reference-model completeness by attribute and indicator;
- percentage of included reference sites with a completed degradation screen;
- baseline-shift detection rate and false-promotion rate;
- climate-adjustment sufficiency by scenario and time horizon;
- alternative trajectory recall/precision;
- attribute coverage and missingness;
- non-compensation violation rate;
- calibration/coverage of indicator envelopes when field references exist;
- sensitivity to reference-site selection and climate conditioning;
- consent/purpose completeness and model-version invalidation precision.

Do not report synthetic scores as field performance. Do not compare SER five-star trajectories as a universal cross-site leaderboard; reference models and indicators remain contextual.

## 8. References

- Gann et al. (2026), [SER International Principles and Standards for the Practice of Ecological Restoration, Third Edition](https://doi.org/10.1111/rec.70441), Principle 3, Principles 5–6 and Section 4 Part 1.
- GEO BON, [Essential Biodiversity Variables](https://geobon.org/ebvs/what-are-ebvs/).
- FAO, [Framework for Ecosystem Restoration Monitoring](https://www.fao.org/ecosystem-restoration-monitoring/ferm/en).
- IUCN, [Global Ecosystem Typology](https://global-ecosystems.org/).
- Water for the Future, [Jukskei water monitoring station](https://www.waterforthefuture.co.za/water-monitoring).

## 9. Non-claims

This protocol and synthetic corpus do not establish the ecological reference condition of any Jukskei reach, biodiversity status, current recovery trajectory, water-quality state, climate response or intervention effectiveness.
