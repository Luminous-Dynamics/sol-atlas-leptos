# Evidence-Bound Economic Intelligence Atlas

**Status:** Design proposal; not a statement that the capability is implemented or qualified.  
**Target repository:** `Luminous-Dynamics/sol-atlas-leptos`  
**Related work:** [SOL-ATLAS-007 / #90](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/90), [SOL-FIN-001 / #10](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/10), [Mycelix supply-chain provenance](https://github.com/Luminous-Dynamics/mycelix-supplychain), [Symthaea financial fragility profile / #5605](https://github.com/Luminous-Dynamics/symthaea/issues/5605)

## 1. Product boundary

Build a global, evidence-grounded atlas for exploring:

1. informal-economy estimates;
2. cross-border trade discrepancies and other illicit-financial-flow (IFF) indicators;
3. documented ownership and entity relationships;
4. relevant logistics and commodity-flow observations;
5. official designations, enforcement findings, and public allegations as distinct record classes.

The product must **not** claim to see every hidden transaction, equate informality with criminality, or turn anomalies into allegations of guilt. It is an investigative and statistical workbench: it shows what was observed, what was estimated, which method produced an inference, what conflicts with it, and what remains unknown.

UNCTAD and UNODC define IFFs for statistical purposes as cross-border flows reflecting an exchange of value that are illicit in origin, transfer, or use. Their framework exists partly because consistent data and measurement remain difficult. This definition is a useful baseline, not a warrant to classify every unusual flow as illicit.

## 2. First vertical slice: trade-mirror discrepancy workbench

Start with one reproducible, offline analysis over a small, frozen UN Comtrade fixture. Do not begin with a global risk score, a large live-ingestion program, or a claim about named suspects.

For a selected reporter pair, year, commodity classification, and flow direction:

1. retrieve or load the exporter-reported and importer-reported observations;
2. preserve original values, quantity, units, partner codes, classification revision, flow direction, and period;
3. pair records only when the dimensions match under a documented policy;
4. classify records as matched, orphaned, incompatible, or excluded, with a machine-readable reason;
5. compute a descriptive mirror-value gap only where preconditions are satisfied;
6. report coverage and sensitivity to exclusions separately from the measured gap;
7. export a deterministic result bundle with the exact inputs, method version, parameters, source snapshot identity, and limitations.

Global Financial Integrity's published method documents practical controls around non-zero values/volumes, compatible units, re-exports, and CIF-vs-FOB reporting. Reuse its method as a research reference, not as an automatic claim that every resulting value gap is criminal. A gap can also arise from legitimate freight/insurance treatment, timing, classification, re-exporting, reporting errors, or incomplete observations. Colluding parties may report matching false values, so absence of a discrepancy is not proof of absence of illicit activity either.

### Output vocabulary

Use neutral, method-bound labels such as:

- `MirrorValueGapObserved`: two reported values differ under the selected matching rule;
- `MirrorValueGapEstimated`: a stated model has adjusted or imputed the raw gap;
- `CoverageInsufficient`: source coverage or record compatibility prevents the calculation;
- `NotAssessed`: no supported evaluation has run;
- `CorroboratedContext`: independent records add relevant context without proving the hypothesis;
- `OfficialFinding`: a competent authority's specific published finding, linked to its source;
- `Allegation`: a source reports an allegation that has not been represented as an established finding.

These are proposed product semantics, not existing Rust enum variants. Their definitions, serialization, migration and exhaustive-match implications must be decided before implementation.

## 3. Keep independent semantic axes separate

The current Sol Atlas `DataKind` (Observed / Curated / Scenario) is useful for broad layer provenance but is insufficient by itself for a mixed intelligence product. Do not overload it into a criminality or confidence score. Add a separate, versioned observation/claim envelope when implementation begins, keeping the existing enum backward compatible unless an explicit migration is designed.

Each observation or claim should retain at least:

- stable `record_id`, `source_id`, source URL or lawful locator, source record ID, and source-revision identity;
- source-published time, retrieval/known-at time, and valid period/event time as separate fields;
- original value and unit, original geography/entity identifiers, and each normalized or derived value;
- `record_kind` (for example reported statistic, direct observation, modeled estimate, derived discrepancy, allegation, official finding, or scenario);
- validation status and its basis, separate from confidence or evidentiary disposition;
- source coverage, missingness, revision state, and known limitations;
- transformation/method identity, version, parameters, and parent input record hashes;
- licence/access policy, required attribution, and redistribution permission;
- claim ceiling: what the record supports and what it does not support.

**Do not collapse these into one “trust score.”** Source reliability, identity-match certainty, statistical uncertainty, independence of corroboration, and legal status answer different questions and must remain separately inspectable.

Model at least two times:
- **valid time** — the period the observation describes;
- **known time** — when this version of the information was available to the system.

A later investigation or corrected dataset must not silently rewrite what the system could have known at an earlier date.

## 4. Source onboarding and legal gates

Maintain a source registry. No source becomes an automated production dependency until its owner, terms, coverage, freshness, schema, attribution, redistribution rights, and use class are recorded.

Required source-registry fields:

```text
source_id, owner, canonical_url, source_type, schema_version,
data_license, terms_url, allowed_use_classes, redistribution_status,
required_attribution, auth_required, rate_limits, retention_policy,
coverage, temporal_resolution, geographic_resolution, known_biases,
reviewed_at, review_evidence_ref
```

Minimum initial source set:

| Source | Suitable first use | Gate / limitation |
|---|---|---|
| [UN Comtrade API documentation](https://uncomtrade.org/docs/un-comtrade-api/) | Bilateral merchandise-trade observations and metadata | Preview endpoints are capped and rate-limited; wider API/bulk access has account or subscription conditions. Confirm access and redistribution terms before scaling or promising refresh rates. |
| [UNCTAD/UNODC IFF framework](https://unctad.org/statistics/illicit-financial-flows) | Definitions and statistical measurement boundaries | An indicator or estimate is not a case-level adjudication of crime. |
| [GFI mirror-analysis methodology](https://gfintegrity.org/reports/methodology/trade-misinvoicing-mirror-analysis/) | Independently documented approach to qualifying, pairing and analyzing trade gaps | Reproduce the relevant assumptions and exclusions; do not label the gross gap as confirmed criminal proceeds. |
| [World Bank Informal Economy Database](https://www.worldbank.org/en/brief/2024/01/09/informal-economy-database) | Historical country-level context and alternative measures | Dataset covers up to 196 economies for 1990–2020 and mixes indirect model-based measures with survey/other indicators. It is not a current live map and not a measure of illegality. |
| [FATF trade-based money-laundering indicators](https://www.fatf-gafi.org/content/dam/fatf/documents/reports/Trade%20Based%20Money%20Laundering.pdf) | Candidate hypothesis features | Red flags guide review; any single indicator is non-dispositive and may have legitimate explanations. |
| [Open Ownership BODS](https://standard.openownership.org/en/latest/) | Interoperability reference for beneficial-ownership records | Pin the schema version and validate each jurisdiction's coverage and reuse terms. BODS 0.4 documentation says development is paused; do not assume an actively evolving standard. |
| [OpenSanctions](https://www.opensanctions.org/docs/) | Matching against publicly documented sanctions/PEP and other public-interest datasets | Bulk data is CC BY-NC 4.0 for non-commercial use; commercial use requires a data licence. Re-check exact terms before use. |
| [Global Fishing Watch APIs](https://api-doc.globalfishingwatch.org/our-apis/documentation/) | Research pilot for vessel-identity/behavior context | API is currently non-commercial under CC BY-NC 4.0 terms. Do not build a commercial dependency until explicit rights are established. |

The source register must distinguish **available for research** from **approved for public display**, **approved for commercial use**, and **approved for redistribution**. A public URL does not imply permission to republish its entire dataset. Keep credentials server-side; never embed tokens in the WASM bundle or public frontend.

## 5. Architecture boundaries

### Sol Atlas — map, inspect, and compare

- Keep renderer-independent Rust data contracts in `sol-atlas-core`.
- Render geographic features only at the resolution supported by source data. Do not fabricate point locations from country aggregates or put country-level estimates on H3 cells as if they were cell-level observations.
- Expose source, period, coverage, method, status, and caveats in the inspector.
- Use a country/region/sector table fallback when boundaries are unavailable or unlicensed.
- Keep the public aggregate map distinct from any permissioned case-workbench view.

### Symthaea — propose and test hypotheses

- Start with deterministic, understandable statistical and graph rules; measure any Symthaea contribution against these baselines.
- Keep hypotheses, supporting observations, contradictions, missing evidence, and competing benign explanations explicit.
- Allow abstention when identity resolution, coverage, or data quality is inadequate.
- Never convert an anomaly into a finding of guilt or make an enforcement decision.

### Mycelix — preserve evidence lineage and controlled sharing

- Bind each derived claim to exact source snapshots, transformation versions, parent hashes, and issuer identity where available.
- Preserve corrections and supersession instead of silently overwriting source history.
- Track source independence: duplicated reporting or syndicated allegations must not count as independent corroboration.
- Enforce purpose and audience boundaries. Do not publish private financial intelligence, personal data, credentials, or sensitive case-linkage edges merely because they are present in an internal graph.
- Treat cryptographic signatures as evidence of origin/integrity, not proof that the underlying statement is true.

## 6. Qualification plan

All initial fixtures should be frozen and deterministic. Include positive examples and negative controls before adding real-world alerting:

- exact bilateral pair with equal values;
- genuine numeric value difference with matching units and periods;
- missing value or zero volume;
- incompatible units and commodity-classification revisions;
- re-export/transshipment edge case;
- CIF/FOB asymmetry or documented freight/insurance explanation;
- duplicate/syndicated source mistaken for independent corroboration;
- company-name collision and ambiguous entity match;
- later source correction, with historical known-at replay;
- sanctioned entity candidate with incomplete identity descriptors;
- synthetic collusive case with no mirror discrepancy;
- ordinary lawful trade spike and legitimate shared corporate address.

Acceptance gates:

1. **Source qualification:** source terms, access plan, schema, attribution and coverage are recorded and reviewed.
2. **Parser qualification:** malformed input, unknown fields, duplicate keys/records, unit mismatch, missingness and schema drift fail explicitly or enter a named quarantine state; they must not silently become zero.
3. **Method qualification:** a frozen reference implementation and fixtures reproduce exact outputs, exclusions and counts.
4. **Evidence qualification:** every displayed result has a complete lineage bundle and explicit claim ceiling.
5. **Temporal qualification:** replay at an earlier known-at frontier does not consume later publications/corrections.
6. **UI qualification:** deterministic screenshots/capture review show observed, modeled, derived, allegation, official finding, stale, partial and missing-data states distinctly.
7. **Safety qualification:** the public view does not expose person-level suspicion rankings or sensitive site/route targeting; no automated adverse action is wired to analytic output.

A queued or green workflow is not evidence by itself. Record the exact tested commit, commands, fixture manifest digest, counts, exit status, and captured UI state.

## 7. Explicit non-goals

The initial slice does not:
- estimate the total dollar value of all illicit activity in the world;
- infer crime from country, ethnicity, poverty, cash use, or geographic proximity;
- create a universal “criminality score” for a country, company, or person;
- infer guilt from sanctions proximity, co-location, network adjacency, or a model score;
- promise real-time coverage from sources that publish with delays or offer limited access;
- ingest confidential bank, tax, law-enforcement or suspicious-transaction data without specific lawful authorization;
- make a public map of vulnerable people, operational enforcement targets, or sensitive facilities;
- replace specialist statistical methods or competent legal authorities.

## 8. Decision and next implementation step

Proceed in this order:

1. Agree the source-registry and claim-envelope contract with SOL-ATLAS-007 (#90), rather than duplicate its economic-observation model.
2. Add the minimal renderer-independent contract and frozen local fixtures in `sol-atlas-core`.
3. Implement only the offline mirror-pairing / value-gap calculation, with explicit exclusion reasons and deterministic tests.
4. Build the inspector and export bundle before adding further source connectors.
5. Add source connectors one at a time, only after terms and exact endpoint behavior are qualified.
6. Connect Mycelix provenance and Symthaea hypothesis generation only after the evidence contract is stable.

The first proof of value is not a dramatic global heat map. It is a reproducible analysis in which a user can explain exactly why a trade discrepancy appears, what assumptions affect it, which legitimate alternatives exist, and why the system does or does not permit a stronger claim.
