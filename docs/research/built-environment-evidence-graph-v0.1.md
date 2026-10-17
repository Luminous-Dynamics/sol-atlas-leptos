# Built Environment Evidence Graph and Decision Receipts

Status: implementation proposal v0.1. This is a provenance/integrity layer for project artifacts. It does not certify the truth of source facts, approve a building under law, or establish that a structure is safe.

## Problem

Site selection, design generation, solver results, model exchanges, review comments, decisions and construction records are often stored as separate documents. A change to the design can silently invalidate a solver result or an earlier approval if the exact input revision is not pinned.

This module provides a typed graph that connects:

1. source evidence and its state/rights/jurisdiction;
2. immutable artifact IDs and content digests, with explicit parent artifact digests;
3. engineering receipts that bind a solver, version, declared scope and review level to the exact input artifact;
4. decision events that bind an actor, authority evidence, the exact target digest and the receipts the decision relied on;
5. supersession/revocation links so a record can be corrected without rewriting history.

The aim is a verifiable provenance chain, not a database that merely records that someone clicked “approve.”

## Core invariants

- **Exact artifact binding:** a review/approval of a digest cannot be reused on a different model revision.
- **Pinned parent lineage:** child artifacts record both the parent artifact ID and its SHA-256; mismatch is explicit.
- **No inferred solver success:** statuses such as NotRun, Failed and NonConverged are not passing results; Indeterminate is not PassWithinDeclaredScope.
- **Review level is explicit:** a policy can require a qualified professional, independent verification, or a lower level for non-safety exploration. The validator does not assign professional credentials or decide who is legally qualified.
- **Evidence has state and rights:** Verified, Estimated, Unknown, Disputed, Stale and Restricted are distinct. Data rights and jurisdiction are separate dimensions.
- **Unknowns remain visible:** missing referenced evidence is a validation error. Unknown, stale, disputed, restricted, or unlicensed evidence cannot silently satisfy an approval policy that does not explicitly accept it.
- **No wrong-version approval:** an approval targeting a superseded revision in the same design/family is flagged.
- **Conditional means conditional:** a record with unresolved conditions cannot be recorded as an unconditional Approve.
- **Revocation is linked:** a revocation names an existing approval event for the same design and exact artifact digest.
- **Policy is project-specific:** required engineering disciplines and acceptable evidence classes must be set according to the project's jurisdiction, scope and intended use. There is no universal receipt list for every pavilion, stadium or tower.

## What the Rust validator checks

The new sol-atlas-core::built_environment_case module checks:
- duplicate IDs for evidence, artifacts, receipts and decisions;
- required identifiers, digests and timestamps;
- artifact evidence links, parent artifact links and pinned parent digests;
- cycles in the artifact-dependency graph;
- solver receipt input IDs/digests, run status, conclusion, solver/version, declared scope, review level and reviewer identity;
- receipt assumption-evidence references and admissibility for the configured approval policy;
- decision targets and exact digests;
- authority evidence, required discipline receipts, and review level for approval;
- superseded approval targets and supersession/revocation lineage.

It reports stable issue codes and deterministic order so that callers can create machine-readable receipts and UI explanations.

## What this does not do

- It does not fetch/hash external payload bytes. It checks the supplied 64-character SHA-256 representation and compares declared digests. The ingestion/storage layer must hash exact bytes and protect immutability.
- It does not parse IFC, verify IDS requirements, evaluate code, interpret land title/zoning, check source licensing with a lawyer, verify credentials, or establish professional competence.
- It does not run structural, geotechnical, fire, wind, seismic, accessibility or energy analyses. It only checks the provenance and declared disposition of receipt records supplied by those tools.
- It does not imply that passing engineering receipts across the configured disciplines exhausts all requirements for a real project. The policy and the review team determine which analysis types and standards apply.
- A synthetic valid graph is only an internal consistency test. It is not an actual building approval, certificate of occupancy, planning permission or safety certification.

## Integration path

- **Sol-Atlas:** stores site/design candidates, source evidence, artifacts, receipts and the decision timeline.
- **Symthaea:** proposes alternatives and summarizes evidence/trade-offs; it must not fabricate a validation receipt or independently grant approval.
- **Symtropy:** may produce exploratory geometry/simulation outputs; they remain research artifacts until the supported solver and review policy says otherwise.
- **Specialist solvers:** run the engineering analysis and produce result receipts that identify their exact inputs, version, scope, warnings and convergence status.
- **Mycelix:** may transport/preserve provenance and decisions across organizations, with permissions and correction/revocation semantics. Replication does not make a claim true.
- **Open exchange:** IFC 4.3 supports interoperable building/infrastructure information; IDS expresses machine-checkable requirements for IFC information delivery; OGC API - Features is an interface for geospatial features. These are interface standards, not evidence of safety.

## Promotion benchmarks

Before using the graph to support a real construction release:
1. At least 30 adversarial fixtures: duplicate IDs, malformed digests, dangling evidence/parents, stale parent hashes, cyclic lineage, stale solver input, failed/nonconverged solver, insufficient review level, wrong discipline, unknown/restricted evidence, unauthorized approval, stale approval, revocation, and time-reversed supersession.
2. Exact digest verification over real byte streams and an immutable content-addressed store; SHA syntax validation alone is not enough.
3. Independent verifier implementation over the same pinned fixtures.
4. Property tests: any input digest change invalidates dependent receipts; any superseded target cannot remain current; no excluded/failed receipt can satisfy a required discipline.
5. Permission and data-rights review, privacy controls and appropriate treatment of security-sensitive infrastructure and cultural/ecological information.
6. IFC/IDS round-trip tests where supported, plus an integration test linking site evidence, the Pareto frontier, investigation updates, design revision, solver receipts and decisions.
7. A built pilot compared against the approved model, with measured construction discrepancies and an independent professional review.

## Standards and references

- buildingSMART IFC schema specifications: https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/
- buildingSMART Information Delivery Specification: https://technical.buildingsmart.org/projects/information-delivery-specification-ids/
- OGC API - Features: https://www.ogc.org/standards/ogcapi-features/
- NIST, Building Digitization and Semantic Interoperability: https://www.nist.gov/programs-projects/building-digitization-and-semantic-interoperability

These standards help exchange structured information. They do not certify this module or any building.
