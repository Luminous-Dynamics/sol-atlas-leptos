# CCZ-COMMONS-001 — Open Clarion–Clipperton Zone Commons Atlas

**Status:** design proposal; no real dataset ingested or legal question adjudicated  
**Parent issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/165  
**Parent architecture:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/115  
**Cross-project assurance:** https://github.com/Luminous-Dynamics/symthaea/issues/8016

## 1. Purpose

Create an open-source, reproducible spatial evidence index for the Clarion–Clipperton Zone (CCZ), then generalize it to other international seabed regions. The first release makes official instruments, exploration-contract areas, environmental protections, public scientific observations, source quality and data gaps inspectable by researchers, Pacific Island communities, developing countries, regulators, civil society, journalists and industry.

The Atlas is not an extraction planner, mining-site optimizer, reserve estimator, navigational product or legal adjudicator. It exposes what a source says, when it was valid, how data were transformed and what the evidence does not establish.

This work extends existing Sol-Atlas maritime asset/provenance facilities. It must not introduce a second generic geospatial asset, provenance, evidence-validation or maritime-data system.

## 2. Legal and scientific framing

The word “claims” must not collapse different classes of evidence.

UNCLOS Article 136 describes the international seabed Area and its resources as the common heritage of mankind. Article 137 says no State may claim sovereignty or sovereign rights over the Area or its resources. Rights to mineral resources in the Area are governed through Part XI and applicable Authority rules. An ISA exploration contract is therefore not sovereign ownership and does not itself constitute a commercial-exploitation authorization.

**Regulatory status is time-sensitive.** In its 19 September 2026 update, the ISA said exploitation-regulation negotiations had continued during the 31st Session; the Council requested a roadmap and an advanced negotiating text for the 32nd Session. The ISA listed Council meetings for 8–19 March and 12–23 July 2027 and the 32nd Assembly for 26–30 July 2027. Therefore a current atlas snapshot should record the retrieval/effective date and source state, rather than imply that draft exploitation regulations have already been adopted. Source: https://isa.org.jm/news/from-the-31st-to-the-32nd-session-advancing-the-mining-code-and-strengthening-the-common-heritage-of-humankind/

Official references:
- UNCLOS Part XI, Articles 136–143: https://www.un.org/depts/los/convention_agreements/texts/unclos/part112.htm
- ISA exploration contracts: https://isa.org.jm/exploration-contracts/
- ISA exploration areas: https://isa.org.jm/exploration-contracts/exploration-areas/
- ISA CCZ Environmental Management Plan and APEIs: https://isa.org.jm/protection-of-the-marine-environment/regional-environmental-management-plans/ccz/
- ISA DeepData: https://isa.org.jm/deepdata-database/about-deepdata/
- GEBCO_2026 Grid and Type Identifier Grid: https://www.gebco.net/data-products-gridded-bathymetry-data/gebco2026-grid
- OBIS data access: https://portal.obis.org/data/access/

Keep distinct typed classes for ISA exploration contract, ISA reserved area, APEI/environmental-management polygon, national maritime zone, domestic-law application, public permit/order, scientific survey footprint, company-reported resource estimate, and unknown/conflicting record. An application or corporate filing must never be presented as an adjudicated right. The Atlas describes the source and instrument; it does not rule on contested legal interpretations.

Scientific baselines are nonuniform. Global gridded bathymetry can include predicted or interpolated cells; an occurrence record shows an observation rather than a complete census; different survey methods, effort and resolution may make data incomparable.

## 3. Product principles

1. **Evidence over color:** every geometry links to the issuing/source document, status, date and provenance.
2. **No implied ownership:** do not use a single owner field for heterogeneous claims, instruments, permits and assertions.
3. **Resolution-aware:** a global bathymetry prior is not close-range clearance truth; a scientific observation is not a navigation product.
4. **Time-aware:** preserve issue, renewal, expiry, replacement, revocation and retrieval history; do not overwrite previous source states.
5. **Visible uncertainty:** missing, stale, preliminary, disputed, conflicting and not-reviewed are explicit states. Unobserved is not absent; unmeasured is not zero.
6. **Open but rights-aware:** check source-specific terms and attribution. Open-source software does not make every input dataset redistributable.
7. **Interoperable:** manifests and normalized exports work without the Atlas UI or one operator's platform.
8. **Public-interest by design:** support environmental stewardship, research, regulatory inspection and transparent economic analysis without implementing extraction optimization.

## 4. Existing architecture to reuse

| Concern | Existing owner | Boundary |
|---|---|---|
| Spatial assets and coordinate semantics | Sol-Atlas geospatial owner #65 | Do not create parallel spatial identity. |
| Provenance graph and source lineage | Existing provenance graph #54 | Every derived artifact identifies source snapshots and transformation. |
| Validation reports and claim boundaries | Existing validation-report owner #59 | Schema conformance is not legal or scientific qualification. |
| Maritime metadata, validity and time coverage | MARITIME-ATLAS-001 #115 | Extend its profile narrowly; do not fork it. |
| Environmental observations / causal history | Symtropy #1240 and #1245 | Read-only; source authorities retain state ownership. |
| Ocean robotics mission contract | Symthaea #8016 | Atlas evidence can inform a mission but cannot authorize actuation. |
| Provenance/custody/public governance | Mycelix where applicable | A hash or signature does not prove physical truth. |

## 5. Minimum feature envelope

This is a conceptual schema, not a compiled Rust type. Reconcile exact field names with #65, #54, #59 and #115 before implementation.

### Feature identity
- Stable feature ID independent of row order and URL.
- Feature class and vocabulary/schema version.
- Geometry, coordinate reference system, geometry precision and uncertainty.
- Source-native identifier and source revision.
- Valid time (when the source says the record applies) distinct from retrieval/transaction time (when the Atlas acquired it).

### Issuer and instrument
- Issuer/publisher as stated by the source.
- Jurisdiction or institutional regime.
- Instrument class and source-stated status.
- Source document/dataset ID and URL.
- Issue, effective, expiry and revocation dates where present.
- Attribution, license/terms state and redistribution constraints.
- A terms-review record ID, reviewer identity and timestamp; the timestamp must include a timezone.
- Source retrieval timestamp with explicit timezone, plus stable source version where available.
- SHA-256 format for archived source/geometry artifacts. The V0 validator checks digest syntax and provenance references; it does not calculate hashes from artifact bytes or authenticate the publisher. The ingestion path must compute a digest from exact bytes before asserting a pinned artifact.

### Measurement and quality
- Observation interval, spatial footprint and sampling effort when available.
- Product edition, method/instrument, units and taxonomic/measurement resolution.
- Horizontal and vertical reference/datum where relevant.
- Coverage, resolution, uncertainty and source lineage.
- Quality status: NotReviewed, ValidForDeclaredUse, Incomplete, Stale, Conflicting, Withdrawn, OutOfScope or Unknown.
- Explicit allowed-use profile (display/context, regional research, habitat evidence, etc.) and source for that classification.

Missing fields remain unknown. Do not synthesize a confidence level merely because the UI needs a value.

## 6. Source registry for V0

V0 uses a small manually reviewed manifest and fixtures. Do not download multi-gigabyte grids or ingest the whole ocean just to prove the interface.

| Source family | Initial use | Critical limit |
|---|---|---|
| ISA contract/area pages and public contract records | Contract geometry/status and source links | Verify status against an exact snapshot; contract is not sovereignty or an exploitation permit. |
| ISA CCZ environmental plan and APEI documents | Environmental-management polygons and instrument references | Preserve version; do not infer that all ecological variation is represented. |
| GEBCO_2026 grid plus TID | Bathymetry context and source-type overlay | Direct sounding, predicted, interpolated, mixed and unknown source types remain distinct. Not close-range clearance truth. |
| OBIS public occurrence exports | Observation provenance and source-attribution handling | Occurrence data do not provide unbiased absence labels or complete habitat coverage. Preserve provider metadata. |
| Public baseline/monitoring papers | Evidence graph and study-site metadata | Record method, place, time, sampling effort, rights and applicability; studies may not be directly comparable. |
| National-authority public applications/orders | Separate domestic-law document layer | Label the actual instrument; do not merge with ISA records or decide contested legal effect. |

Each source manifest entry records official URL, retrieval date, stable version/identifier, coverage, license/terms and attribution notes, allowed use, artifact digest if archived, and rationale for the proposed use. If terms do not allow mirroring, store citation/metadata only.

## 7. Delivery sequence

### C0 — semantic fixtures
Add deterministic fixtures for: ISA exploration contract, reserved area, APEI, national maritime zone, domestic-law application, corporate-reported resource estimate, scientific occurrence record, expired/revoked record, conflicting issuer/status and unknown instrument type. Unless independently sourced and verified, fixtures are synthetic and cannot be represented as a current CCZ inventory.

### C1 — source provenance and versioning
Bind each snapshot to exact artifact digest, retrieval/version identity and terms review. Derived features bind parent digests, transform version/parameters and output digest. A URL that now serves different bytes is a new snapshot, not the same evidence.


### C1a — exact-byte verification receipt

The core now exposes `sha256_hex` and `verify_artifact_bytes`. The latter emits a deterministic receipt containing the source snapshot pointer, expected digest, digest computed from the actual supplied bytes, byte length and one of four explicit states: `matched`, `missing_expected_digest`, `invalid_expected_digest`, or `mismatch`. A URL or version label never overrides a byte mismatch. A missing or malformed expected digest cannot be reported as verified.


**Terms must be endpoint-specific.** The GEBCO product page says its grid is public domain subject to its conditions and asks for acknowledgment; the CEDA archive catalogue for the same GEBCO_2026 product lists the Open Government Licence v3 and requires citation. Preserve these as separate source statements with their own URLs and retrieval times. They may be compatible, but the Atlas must not assume equivalence or mark a particular downloaded artifact `OpenRedistributionReviewed` based on a single landing page. Keep publication at `ReviewRequired` until the source notices and the terms governing the chosen endpoint/artifact have been reviewed. Sources: [GEBCO product terms](https://www.gebco.net/data-products-gridded-bathymetry-data/gebco2026-grid) and [CEDA archive catalogue](https://catalogue.ceda.ac.uk/uuid/fb236c93aa574cb9b5154db45dfdd784/).


The receipt now has a composite `can_publish_artifact(snapshot)` gate: source snapshot identity and expected/actual digest agreement must match, and the existing source validation must independently grant `OpenDataAndMetadata`. This prevents a correct hash from overriding unreviewed/restricted terms or a receipt from being reused against another snapshot identity. A serialized receipt is still not publisher authentication or a trusted signature, so callers must preserve the trusted ingestion/provenance chain.

The helper is intentionally not a downloader or an archive: the caller must obtain bytes through a reviewed ingestion path and retain the payload and receipt. It does not authenticate publisher identity, prove a legal interpretation, parse a data format, or grant publication permission. The existing terms/publication gate remains separate.

**First source candidate:** GEBCO_2026 Grid plus its Type Identifier (TID) Grid. The official GEBCO page says the grid is published at 15 arc-second intervals, supports user-selected area subsets and OPeNDAP access, and is public domain subject to its terms/disclaimer and required source acknowledgment. The cited edition reference is GEBCO Bathymetric Compilation Group 2026, DOI [10.5285/4f68d5c7-45eb-f999-e063-7086abc036fa](https://doi.org/10.5285/4f68d5c7-45eb-f999-e063-7086abc036fa). The global grid is multi-gigabyte; this change does **not** download it or claim the source artifact is ingested or rights-reviewed. Select a bounded CCZ subset, save the exact source bytes and source documentation, compute the digest locally, record exact terms/attribution review and compare the TID cells before adding any spatial claim.

Source: [GEBCO_2026 Grid and terms of use](https://www.gebco.net/data-products-gridded-bathymetry-data/gebco2026-grid) and [Gridded Bathymetry Data access methods](https://www.gebco.net/data-products-gridded-bathymetry-data).

### C2 — bathymetry source-quality display
Implement metadata/TID handling before detailed rendering. Make directly measured, predicted, interpolated, mixed and unknown source types distinguishable. Do not imply navigation safety.

### C3 — temporal and instrument view
Show valid time, retrieval time, source document and change history separately. Compare snapshots while preserving prior versions, corrections and withdrawals.

### C4 — biological/environment evidence index
Add public observation/study references with methodology, spatial/temporal coverage, sampling effort, uncertainty and limitations. Never turn missing records into negative species labels.


### C4a — field-evidence benchmark anchors (not universal thresholds)

Use published trials to stress the monitoring and inference pipeline, while keeping each study’s site, collection method, observation window and uncertainty attached. These results are not transferable “safe limits” for another site or operation.

- Valls Domedel et al., *Nature Ecology & Evolution* (2026), compared two years of pre-trial data with sampling two months after a large-scale eastern Pacific trial. The authors reported a 37% decrease in macrofaunal density and a 32% reduction in species richness directly inside mining tracks, while also finding significant natural background variation. This is a regression anchor for pre-impact baselines, control sites, species-level analysis, and separating background variation from impact.
  Source: https://www.nature.com/articles/s41559-025-02911-4
- Gazis et al., *Nature Communications* (2025), reported a benthic plume/current-monitoring trial where the maximum suspended-particle concentration at 50 m from mining lanes was up to four orders of magnitude above ambient; bottom currents dominated dispersion to the 4.5 km monitoring boundary, with about 3 cm sediment redeposition next to lanes and a minimum estimated erosion depth of 5 cm. This is an anchor for current-aware spatial sampling, calibrated multi-sensor observations, and detecting impacts beyond the machine track.
  Source: https://www.nature.com/articles/s41467-025-56311-0
- *Nature* (2025) reported that biological impacts persisted in many organism groups 44 years after an older nodule-removal experiment, despite re-establishment in some groups and persistent physical seafloor changes. This is a long-horizon anchor: short post-trial monitoring cannot by itself establish recovery.
  Source: https://www.nature.com/articles/s41586-025-08921-3

Benchmark implications:
1. A valid environmental assessment needs a declared baseline window, matched reference/control sites, spatial replication, method and detection limits, sensor calibration and observation timestamps. Missing baseline/control data must return `Indeterminate`, not “no effect”.
2. Simulations must include current-driven plume transport, deposition, sensor blind spots, calibration drift, sparse sampling and post-operation effects. Detect when a plume leaves the observation footprint; do not convert “not detected here” into “absent”.
3. Report direct-track, near-field, far-field and longer-term outcomes separately. Preserve physical disturbance, biological responses, uncertainty and recovery evidence as distinct measures.
4. Do not derive operationally safe plume/biodiversity thresholds from these three studies alone. Thresholds need a broader evidence base, independent ecological review, explicit regulatory authority and a precautionary response to unknowns.
5. Fleet-scale accounting must include cumulative sound, light, sediment disturbance and repeated passes over time, not only each individual vehicle’s local contact.

### C5 — reproducible export
Provide a source manifest and deterministic normalized metadata export so another team can reproduce the same result offline from the same permission-cleared inputs.

## 8. Benchmark protocol and proposed gates

Freeze fixture versions and independent expected outputs before evaluation. Legal-semantics tests must be separate from geometry/provenance tests; successful rendering cannot compensate for a misclassified instrument.

**Semantic/legal status**
- 100% of fixtures remain distinct through parse, normalization, export and rendering.
- Zero test cases classify an ISA exploration contract as sovereign ownership or a commercial-exploitation authorization.
- 100% of unknown, conflicting, expired or revoked fixtures preserve their declared status without favorable fallback.

**Provenance**
- 100% of rendered features link to source snapshot/document and review state.
- 100% of transformations preserve parent identities, transform identity and output digest.
- 100% detection in the frozen corpus for changed bytes behind the same URL, wrong revision, duplicate/replayed source, missing attribution and disallowed redistribution.
- Deterministic reruns over frozen inputs produce identical normalized metadata and manifest digest where deterministic transformation is required.

**Spatial integrity**
- Preserve coordinate reference, vertical datum and valid time through transformation and export.
- Test polygon validity, boundary intersections, overlapping distinct feature classes, antimeridian cases, coordinate transformations and area calculations against a separate geometric oracle.
- Never report greater precision than the source supports.

**Scientific comparability**
- 100% of metadata tests preserve missingness and stated sampling-effort distinctions.
- No unobserved cell is converted to confirmed absence.
- Benchmark splits group by source, site/campaign and time where known to reduce spatial/source leakage.

**Reproducibility and usability**
- Offline deterministic reproduction works from the frozen source manifest.
- The source drawer exposes citation, attribution, terms state, validity/currentness, provenance and claim ceiling.
- Core browsing works without AI/GPU services or network calls during rendering.
- Exact local commands/results and independent-oracle identity are recorded; queued CI is not a pass.

These targets apply to the explicit finite corpus, not all real legal or scientific questions.

## 9. Required negative controls

- Changed bytes behind the same URL create a new source snapshot and preserve the old one.
- A fixture that classifies an ISA exploration contract as national sovereign title fails semantic validation.
- A domestic application mislabeled as an issued permit fails unless source evidence supports that type.
- Missing CRS, vertical datum or valid time required by a selected use becomes unknown/incomplete or rejects that use.
- A GEBCO predicted/interpolated cell relabeled as directly measured fails provenance validation.
- An unobserved biodiversity cell relabeled as confirmed species absence fails.
- Missing license/attribution evidence blocks publication or redistribution under policy.
- A withdrawn/corrected record cannot replace historical state without retaining history.
- Overlapping features with different instruments/statuses remain separate; the map does not infer a legal winner.

## 10. Governance and contribution

Keep the project public and vendor-neutral. Invite review by marine scientists, Pacific Island institutions, developing-state research programmes, legal experts and open-data maintainers.

Contributors may propose metadata and interpretations, but authoritative records remain attributed to their issuers. Review decisions and transformations are public and versioned. Sensitive species coordinates, contract-confidential records and non-redistributable data must not be exposed in the name of openness.

Before advertising the map as complete, obtain independent review of the source registry, instrument classifications, scientific comparability rules and limitations.

## 11. Claim ceiling

A passing fixture proves that the implementation handled declared synthetic/source metadata as specified. It does not establish that the inventory is complete, a legal status is uncontested, a data product is accurate, a habitat is absent/present beyond observation scope, a site is navigable, or extraction is safe or authorized.

The public-good objective is to make the basis for decisions inspectable, comparable, challengeable and reproducible—not to confer permission to mine.


## 12. Material sufficiency before primary extraction

The Atlas must not turn a concentration of nodules or a mapped resource occurrence into an implied need to extract. It is a spatial evidence commons; material demand and substitution analysis belongs to the existing Symthaea [CRITMAT-001 programme #5200](https://github.com/Luminous-Dynamics/symthaea/issues/5200).

For any later public decision study, compare the same delivered function and time horizon across distinct pathways:

1. lower material intensity, longer service life, repair and reuse;
2. technically qualified material or architecture substitution;
3. manufacturing scrap, end-of-life recovery, recycling and industrial/byproduct feedstocks;
4. existing terrestrial primary sources where comparable source and life-cycle evidence exists;
5. seabed-derived material only as a separate, unresolved scenario with its own legal status, environmental, energy, transport, refining and benefit-sharing evidence.

Keep separate axes for technical equivalence, mass per functional unit, recovery yield and grade, lifecycle emissions/energy/water/reagents/waste, habitat/biodiversity impact, uncertainty, cost and benefit distribution. Do not collapse those into a single “green metal” score without a declared decision rule. Missing high-impact burdens remain Unknown/Indeterminate, not zero.

This is a paradigm shift from “where is the metal and how can it be extracted?” to “what is the least harmful, technically adequate and fairly governed way to provide the required function?” It is a cross-project decision principle, not an extraction planner.


### Structural validity is not publication authority

The Rust validator exposes separate structural validity and publication-scope decisions. Consumers must check the relevant publication permission as well as structural errors; a readable record with restricted or unreviewed terms is not publishable. Metadata-only permission never authorizes redistributing the underlying artifact. Source-review fields preserve review provenance but are not cryptographic authentication or independent legal advice.
