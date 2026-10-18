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
- Attribution, license/terms state, redistribution constraints and review timestamp.

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

### C2 — bathymetry source-quality display
Implement metadata/TID handling before detailed rendering. Make directly measured, predicted, interpolated, mixed and unknown source types distinguishable. Do not imply navigation safety.

### C3 — temporal and instrument view
Show valid time, retrieval time, source document and change history separately. Compare snapshots while preserving prior versions, corrections and withdrawals.

### C4 — biological/environment evidence index
Add public observation/study references with methodology, spatial/temporal coverage, sampling effort, uncertainty and limitations. Never turn missing records into negative species labels.

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
