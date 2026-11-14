# EARTH-OBSERVABILITY-FEDERATED-EVIDENCE-001
## Rights-aware, source-local subsurface computation and evidence federation v0.1

**Parent:** Sol-Atlas #761 — EARTH-OBSERVABILITY-QUAL-001  
**Status:** architecture and qualification protocol; source-local adapters are not yet implemented  
**As-of:** 2026-11-13

## 1. Paradigm shift: from data hoarding to rights-aware evidence federation

A global subsurface platform cannot assume that every useful observation may be copied into a central warehouse. Public discovery, authorized viewing, processing, creation of derived products and redistribution are separate permissions.

The 2026-11-10 Open Well Data catalogue is a useful warning: it indexes 507 sources across 79 countries and territories, but only 139 are marked as having licences that allow copies; it explicitly states that the full catalogue check is not complete, and its Zenodo downloads were temporarily unavailable on 13 November 2026. Its index includes three South African source entries, none marked copy-allowed in that catalogue snapshot. This is not a claim that South African sources are inaccessible; it demonstrates why per-source, per-purpose rights qualification is necessary.

The architectural principle is:

> Compute where the data is held; move only the minimum authorized result and its reproducible evidence receipt.

This is a design pattern, not a claim that federated execution automatically guarantees privacy, lawful use, or measurement truth.

## 2. Permission dimensions are not interchangeable

An asset's source record must distinguish:

- **Discovery:** metadata can be found in a catalogue.
- **Read/access:** the user or service may open or query the source.
- **Processing:** the source may be used for a declared computation.
- **Derivative creation:** a derived dataset, model or interpretation may be produced.
- **Redistribution/publication:** source bytes or derivative outputs may be shared with a declared audience.
- **Custody and retention:** the source owner controls storage duration, replication, cache, and deletion/revocation conditions.

A public map viewer, API endpoint, or registration-based account does not automatically imply the right to download, process, derive, or republish every underlying layer.

No component should infer a broad right from a narrower permission. A user may have read access but no redistribution right. A custodian may permit an analysis to execute locally but prohibit raw source export. A derivative may have separate publication constraints from the source data.

## 3. Reference execution flow

```text
Source catalogue + exact asset identity
                |
                v
     Source owner / rights authority
                |
        signed, scoped grant receipt
                |
                v
      Mycelix receipt verification
                |
                v
       Data-use policy gate
                |
                v
     Source-local adapter / executor
                |
        bounded authorized result
                |
                v
     Evidence receipt + provenance
                |
                v
   Sol-Atlas claim graph / fusion layer
                |
                v
  Symthaea decision and investigation proposal
                |
                v
  Symtropy physical qualification / comparison
```

The source-local executor receives only the exact source identifiers and operations covered by the grant. Raw utility geometry, private borehole locations, mine workings, sensitive infrastructure or restricted site records do not leave the custodian by default.

The returned result should include, at minimum:

- exact source and asset identifiers;
- exact source bytes or immutable source snapshot digests where the custodian permits them to be recorded;
- data-use grant receipt identifier and digest;
- adapter and execution-code identity;
- model, configuration, transformation and dependency identities;
- declared query geometry/time range and spatial support;
- benchmark split identifier and target definition;
- output digest, units, uncertainty and abstention state;
- source-owner policy version, execution time and expiry/revocation status;
- explicit disclosure classification for the output.

A digest proves byte identity only when the referenced bytes are available for verification. A receipt does not make an unverified source scientifically correct.

## 4. Data-use grant contract

The first fail-closed Rust gate is implemented in:

- `sol-atlas-core/src/earth_observability.rs`
- `authorize_data_use(...)`
- `DataUseGrant`, `DataUseAction`, and `DataUseDenial`

It checks an upstream-issued grant bound to the **exact source, exact asset, requesting subject and declared purpose**, before allowing one of five operations:

1. `Read`
2. `Process`
3. `CreateDerivative`
4. `RedistributeSource` — permission to redistribute the original source asset.
5. `RedistributeDerivative` — permission to publish/redistribute a derivative product.

Source redistribution and derivative publication are independent grants; permission for one does not imply the other.

The gate requires non-empty grant/source/asset/subject/issuer identity, a syntactically valid SHA-256 receipt digest, exact source+asset+subject binding, explicit purpose membership, an active validity interval, satisfied declared conditions and operation-specific capability booleans. Missing or invalid grants fail closed.

**Important limits:** this code does not authenticate the issuing custodian, verify the grant signature, interpret licence text, prove that conditions were actually satisfied, implement revocation propagation, or enforce storage/network controls. Those responsibilities belong to the authoritative grant/custody and execution layers. The local Rust gate is a policy-consistency check over an already-qualified receipt, not legal advice or a cryptographic authorization service.

## 5. Federated benchmark protocol

Benchmark federated execution against both a source-local reference and an authorized centralized execution where copying is permitted.

| Dimension | Metric / test | Hard failure |
|---|---|---|
| Grant identity | exact source/asset/subject binding and receipt verification | grant for source A, asset A or subject A authorizes a different source, asset or subject |
| Purpose/action separation | Read / Process / Derive / RedistributeSource / RedistributeDerivative negative controls | read-only permission authorizes processing; raw-source grant authorizes derivative publication or vice versa |
| Validity | not-before, expiry and revocation tests | expired or revoked grant continues to authorize new access |
| Data locality | inspect outbound payloads, caches, logs and artifacts against restricted-source markers | unapproved raw source data leaves the owner boundary |
| Output disclosure | owner-defined geometry precision, aggregation, minimum cohort and sensitive-field policy | restricted geometry or sensitive attributes are disclosed |
| Reproducibility | repeat execution from frozen source version, code, configuration, policy and model digests | result cannot be replayed or material changes are not explained |
| Numerical parity | compare permitted federated result to the same computation on an authorized local reference | result differs beyond preregistered numerical tolerance |
| Coverage | fraction of requested sources that are available, authorized and actually executed | unknown or inaccessible sources silently count as covered |
| Freshness | source-version and grant-expiry lag | stale source or expired rights represented as current |
| Revocation | time to block new reads, processing, derivatives and redistribution after revocation | new processing succeeds after revocation acknowledgement |
| Cost/latency | data egress, computation, request count, time and owner burden | performance win hides unauthorized movement or omitted sources |

Report authorization coverage and scientific coverage separately. A system that can legally process 40% of a source list must not imply it has observed the remaining 60%. Likewise, an owner-approved execution result is not independent ground truth.

## 6. Security and privacy boundaries

- Federated execution is not automatically privacy-preserving. Metadata, query geometry, timing, model updates and output statistics may themselves leak information.
- Use source-owner-approved output schemas and disclosure review. Add differential privacy only where a defensible privacy model, sensitivity bound, contribution model and utility target exist; do not add noise indiscriminately to survey control or engineering coordinates.
- Avoid sending private coordinates or restricted geometry to general-purpose model prompts, logs, error telemetry or CI artifacts.
- Keep raw restricted inputs outside CI and synthetic public fixtures. Public tests must use abstract non-geographic coordinates.
- No cached derivative is assumed redistributable merely because it was computed locally. Its rights and disclosure status must be explicit.
- Unknown rights, expired grants, mismatched source IDs, withheld records and conditions not met must not be treated as permission.
- Source owners retain authority over access, corrections, retention, revocation and permitted disclosure.

## 7. Composition and ownership

- **Sol-Atlas** owns spatial evidence integration, exact source/asset references, claim lineage, observability semantics and federated benchmark orchestration.
- **Mycelix** owns identity, custody, rights receipts, grant authenticity, review and accountable revocation. Its valid record establishes the authorization trail, not the physical truth of the measured claim.
- **Symtropy** owns sensor/physical-model qualification, coordinate transforms and independent field/survey evidence.
- **Symthaea** can propose queries, hypotheses and next-best investigations within an explicitly granted action envelope. It cannot self-authorize data access, drilling, excavation or disclosure.
- Data owners, licensed custodians, landholders and competent authorities retain their legal and operational roles.

## 8. Initial pilot implications

- **Johannesburg / City GIS:** keep each aerial/LiDAR layer's licence, accuracy report, CRS, vertical datum, classification and release terms separate. If an asset is request-only, use a source-local or owner-delivered workflow until the terms permit wider processing.
- **DWS National Groundwater Archive:** treat account access, record visibility, download, analysis, derived mapping and publication as separately verified rights.
- **Council for Geoscience:** admit only exact, versioned layers/reports with terms and scale documented.
- **BGS SOBI:** its open, Open Government Licence index is a promising first cross-source metadata/coverage benchmark; any linked scanned record remains a separately identified artifact.
- **DINOloket/BRO/NLOG/USGS GeoLog Locator:** useful candidates for rights-qualified borehole, groundwater, seismic, geophysical-log and LAS interoperability; verify each record family and data type rather than assuming portal-wide terms.

## 9. Next implementation gates

**F0 — Contract:** exact source ID, asset ID, requesting subject, purpose, grant digest, issuer identity, validity and action matrix (Rust predicate added).  
**F1 — Receipt verification:** signed-grant authentication, issuer trust and revocation evidence through Mycelix.  
**F2 — Local executor:** one non-sensitive, permissioned source with a bounded query and no raw-data egress.  
**F3 — Reproducibility:** source-local rerun receipt, output digest, exact code/model/policy identity and independent local comparison.  
**F4 — Disclosure testing:** restricted-geometry sentinel corpus; verify raw coordinates and prohibited derivatives do not leave the owner boundary.  
**F5 — Multi-owner federation:** several independently administered source nodes with different rights/expiry/retention rules and robust failure handling.

No gate is passed by a synthetic rights fixture alone. No benchmark result authorizes access to a real source unless the source owner has issued the applicable grant.

## 10. Research anchors

- Open Well Data catalogue and source status: https://openwelldata.org/
- BGS Single Onshore Borehole Index: https://www.bgs.ac.uk/datasets/boreholes-index/
- Geological Survey of the Netherlands data portals: https://www.geologischedienst.nl/en/mapviewers-subsurface/
- NLOG borehole access and confidentiality: https://www.nlog.nl/en/boreholes
- USGS GeoLog Locator: https://webapps.usgs.gov/geologlocator/
- DWS groundwater data: https://www.dws.gov.za/Groundwater/data.aspx
- City of Johannesburg CGIS: https://eservices.joburg.org.za/sites/cgismaps/Pages/default.aspx
