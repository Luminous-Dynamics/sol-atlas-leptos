# CityMaker change-impact — industry crosswalk v0.1

**Status:** source-backed capability map and proposed matched benchmark; not an independent product evaluation  
**Research date:** 8 November 2026  
**Related:** CITY-CHANGE-IMPACT-001 #643 / draft PR #644

## 1. Competitive conclusion

Revision tracking, model federation, clash detection, enterprise ontology, asset lifecycle management, virtual commissioning and supply-chain traceability are already established capabilities in the industry.

CityMaker must not claim to invent any of these.

The competitive hypothesis is narrower and more testable:

> Can CityMaker consume independently authoritative change, provenance, impact and revalidation reports from specialist owners, then prove which exact city/building/service claims are still applicable for a declared scope and evaluation time—without a central system taking ownership of those facts?

That is a composition and evidence-applicability problem, not a generic digital-twin feature checklist.

## 2. Current reference systems

| Reference | Documented capability | What CityMaker should reuse or benchmark | What CityMaker must not claim |
|---|---|---|---|
| Bentley iTwin Platform | Synchronizes CAD/BIM/engineering sources into iModels; creates incremental changesets; maintains synchronization changes; APIs identify and visualize added, removed and modified elements and properties | Source revision identity, change history, object-level difference, connector coverage, synchronization diagnostics, IFC export | That Bentley lacks change tracking, version history, or infrastructure federation |
| Autodesk Construction Cloud / Model Coordination | Aggregates discipline models, detects clashes, supports constructability review and issues; Autodesk's current Revit/coordination documentation exposes changed-element property comparisons between model versions | Exact element/property delta, model-set scope, clash/issue continuity after updates, false/new clash handling | That Autodesk cannot compare model revisions or identify changed properties |
| Palantir Foundry Ontology | Native federation of external datasets into objects without duplicating underlying data; relations, actions, processes, model versioning, security and lineage | External owner-source preservation, relationship/decision projection, source writeback boundaries, model lineage and operator workflows | That Palantir lacks federation, ontology, lineage or operational decision tooling |
| Siemens Digital Twin Composer | Factory-scale digital environments, simulation, virtual commissioning and robotics training | Product/process/factory simulation hand-off, virtual commissioning evidence and simulation configuration identity | That Siemens lacks industrial digital-twin composition or virtual commissioning |
| NIST IR 8536 | Industry-neutral manufacturing traceability meta-framework; linked provenance, temporal ordering, interoperability and selective disclosure | Product/lot lineage, cross-supplier traceability and minimal disclosure | That CityMaker's provenance design is novel merely because it links suppliers and lots |
| IETF RFC 9943 / SCITT | Signed statements, transparency-service receipts and append-only history | Issuer-statement accountability, receipt references and history | That a transparency receipt proves engineering correctness or source completeness |
| ISO 23247-6:2026 | Integrated, unified and federated manufacturing digital-twin composition | Interface classification, communication/interoperation assumptions and composition design | That federated composition itself is a new CityMaker invention or that manufacturing standard scope automatically certifies city twins |
| ISO 55000:2024 | Asset-management principles and lifecycle value orientation | Portfolio, assurance, adaptability and sustainability alignment | That asset-management vocabulary supplies an impact-graph algorithm |

### Primary sources

- Bentley iTwin platform and change history: https://www.bentley.com/products/itwin-platform/
- Bentley Synchronization & Exports: https://developer.bentley.com/api-groups/synchronization/
- Bentley Synchronization API: https://developer.bentley.com/apis/synchronization/
- Bentley Changed Elements / Review APIs: https://developer.bentley.com/api-groups/project-delivery/
- Autodesk Model Coordination: https://help.autodesk.com/cloudhelp/ENG/Coord-About/files/About_Model_Coord.html
- Autodesk Model Coordination version-change detail: https://help.autodesk.com/cloudhelp/2027/ENU/Revit-WhatsNew/files/GUID-B9284B64-E145-41D6-9242-9B243F262B2F.htm
- Autodesk support note on new clashes after changed model versions (5 May 2026): https://www.autodesk.com/support/technical/article/caas/sfdcarticles/sfdcarticles/BIM-360-Model-Coordination-reporting-preexisting-clashes-as-new-clashes.html
- Palantir Foundry Ontology, including Native Federation and model lineage: https://www.palantir.com/explore/platforms/foundry/ontology/
- Siemens Digital Twin Composer (announced at CES 2026): https://www.siemens.com/en-us/company/digital-transformation/industrial-metaverse/introducing-digital-twin-composer/
- NIST IR 8536 final: https://csrc.nist.gov/pubs/ir/8536/final
- RFC 9943: https://www.rfc-editor.org/rfc/rfc9943
- ISO 23247-6:2026: https://www.iso.org/standard/87426.html
- ISO 55000:2024: https://committee.iso.org/cms/live/live/en/sites/isoorg/contents/data/standard/08/30/83053.html

## 3. What to compare, fairly

Use a shared change set and the same rights-cleared exported dependency graph wherever each comparator can represent it without semantic loss.

### Test A — model-level change fidelity

Inputs: a frozen IFC/BIM or infrastructure model pair, identical coordinate frames, exact source revisions and declared model coverage.

Measures:
- added/removed/modified object precision and recall;
- property-change agreement;
- geometry-change agreement using independent geometry measurements;
- unit/coordinate-frame errors;
- synchronization and export loss;
- time to produce a reproducible change report.

This is a capability where Bentley iTwin and Autodesk Model Coordination are strong direct references. CityMaker should consume their output or use them as upstream specialists rather than rebuilding a weaker model-diff engine.

### Test B — downstream engineering and city-claim impact

Inputs: exact model-level changes plus an independently authored dependency graph covering requirements, products, manufacturing, delivery, installed assets, services and decisions.

Measures:
- missed downstream claims;
- over-invalidated claims;
- impact path completeness and explanation quality;
- claim-specific revalidation obligations;
- false no-impact conclusions when graph coverage is incomplete;
- exact post-change receipt matching.

This is the differentiating hypothesis for CityMaker. It must use the canonical Sol-Atlas/Symthaea impact owners and their independent oracles.

### Test C — bitemporal and trust lifecycle

Inputs: signed/unsigned corrections, source/lot revocations, rights updates, late arrivals, key changes and out-of-order events.

Measures:
- historical replay at earlier evaluation times;
- present-day decision-reliance correctness;
- issuer/owner authority preservation;
- rollback and stale-base rejection;
- difference between byte equality, signature/transparency, rights, technical validity and engineering applicability.

Use the Sol-Atlas IEAC/SCITT/TUF/VC crosswalk and NIST IR 8536. Do not invent another transparency log or signature protocol.

### Test D — product/process/factory change

Inputs: panel and service-cassette configuration changes, BOM/lot/fixture/tooling/firmware/calibration changes and exact manufacturing/commissioning receipts.

Measures:
- which production, installation and service assertions require requalification;
- whether unaffected installed-asset measurements remain usable;
- end-to-end requalification time and workflow labour;
- quality, first-pass yield and rejected-output accounting;
- virtual-commissioning/simulation versus measured physical evidence.

Use Symtropy's service cassette, panel-cell and built-asset robot-loop owners; Siemens is an industry reference for factory simulation/virtual commissioning, not evidence of construction-site performance.

### Test E — city service and asset portfolio

Inputs: the resulting product/asset claims projected into one scope-bound reference district.

Measures:
- affected building/service decisions and population/service scopes;
- time-dependent service consequences and recovery obligations;
- whether unrelated neighbourhood claims remain current;
- lifecycle, accessibility, affordability and environmental outcomes with their own boundaries;
- total human review and revalidation burden.

This is where Sol-Atlas adds the city-domain consumer. No synthetic pass becomes real district evidence.

## 4. CityMaker's defensible differentiation

A credible advantage is not more features, a larger ontology or more 3D detail.

It would be the demonstrated ability to:
1. preserve each specialist owner's source of truth;
2. bind each assertion to exact subject, revision, scope, time and provenance;
3. verify the owner-reported change/impact set independently;
4. identify the smallest justified revalidation frontier;
5. prove when a claim was actually revalidated against the post-change state;
6. preserve unaffected claims and historical knowledge;
7. expose Unknown when owner coverage, authority or dependency edges are incomplete;
8. trace the effect of the change through physical production and the city services people receive.

The system must also outperform a full-recompute baseline in cost without increasing false negatives or false no-impact conclusions. This is a Pareto comparison, not a contest to minimize the number of revalidations.

## 5. Minimum evidence before claiming advantage

- exact product and module versions for every comparator;
- published feature and API documentation pinned to evaluation date;
- common frozen input set, source revisions and graph coverage;
- semantic-loss report for each import/export adapter;
- independently authored expected changed-object and downstream-claim sets;
- raw outputs and compute/human-effort denominators;
- confidence/uncertainty and all abstentions;
- explicit unavailable/unsupported fields rather than inferred competitor weakness;
- independent replication on a second project or source-system family;
- physical verification before any construction, maintenance or service-performance claim.

An E1 synthetic PASS demonstrates only declared software semantics. E3+ measured trials and independent review are required for operational superiority claims.

## 6. Research references and boundaries

ISO 23247-6:2026 is specifically a manufacturing digital-twin composition standard. Its integrated, unified and federated categories are a useful reference for CityMaker's owner boundaries, but applying them to city/building evidence requires an explicit adaptation study.

NIST IR 8536 describes traceability across manufacturing supply chains and advocates interoperable links and selective disclosure. It supports, rather than replaces, Mycelix's lot/custody owner and the city consumer.

IDS 1.0 is limited to alphanumeric IFC information requirements and does not express geometry. IFC 4.3.2.0 is the current official IFC release. CityMaker's geometry-impact claims therefore need separate geometry/as-built verification, not an IDS-only check.

CityMaker should benchmark and compose with established software first. Reimplementation is justified only where a required owner contract, capability or independent validation path is demonstrably missing.
