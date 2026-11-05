# HDB-20: Heritage Provenance, Fixity and Rights Ledger v0.1

## Purpose

HDB-20 makes every source and derived artifact traceable through explicit entity, activity and agent records, with byte-level fixity, purpose-specific rights and a deterministic manifest commitment.

## Internal model

- Artifact/entity: exact-byte SHA-256, byte length, media type, role, access tier, primary/derived status and rights decision.
- Activity: purpose, typed operation, exact inputs/outputs, responsible agents, time bounds and (for automated transformations/inference/simulation) software build, environment and parameter digests.
- Agent: person, organization, software or instrument, identified by opaque reference. Identity is not authenticated here.
- Rights decision: explicit allowed, restricted, prohibited or undetermined status and purpose sets. Undetermined is deny-by-default. Research permission does not imply model training, commercial use or redistribution.
- Fixity event: expected versus observed digest; mismatch events remain in the ledger and are reported rather than hidden.

Derived artifacts must have exactly one generating activity. Activities must reference existing objects and agents. A cycle in the artifact lineage is rejected. The canonical manifest digest is computed after sorting record arrays and their set-like reference fields.

## Standards mapping

- W3C PROV-O: Entity / Activity / Agent and used/generated/derived/association relationships. This is a compact internal subset, not full RDF/PROV-O conformance: https://www.w3.org/TR/prov-o/.
- RO-Crate 1.3: package-level metadata and contextual entities for files, people, software, equipment, workflows and reuse licences. Future export should use RO-Crate rather than replace it: https://www.researchobject.org/ro-crate/specification/1.3/.
- PREMIS 3.0: preservation Objects, Events, Agents and Rights. Fixity checks and migrations can be mapped to PREMIS events: https://www.loc.gov/standards/premis/v3/.
- BagIt RFC 8493: payload manifests and checksum validation for transfer/preservation packages. HDB-20 can export a BagIt payload manifest, but does not implement a complete BagIt package validator: https://www.rfc-editor.org/rfc/rfc8493.html.

## Claim ceilings

- Recomputed SHA-256 equality establishes byte equality with the declared digest, not authenticity or authorship.
- A canonical manifest SHA-256 commits to the serialized metadata representation after canonical ordering; it is not a digital signature.
- An agent URI is not identity proof.
- A rights receipt is an auditable reference, not legal interpretation or authority authentication.
- A cycle-free provenance graph may still describe an inaccurate observation or false historical interpretation.
- The v0.1 timestamp validator checks a UTC Z-suffixed representation and lexicographic order; it is not a full RFC 3339 parser.
- JSON Schema fixtures are parsed as JSON in the unit tests; full Draft 2020-12 schema conformance needs a dedicated validator.

## Benchmark gates

Controlled payload mutation must be detected in 100% of byte-verifier tests. Invalid agent/artifact/activity references, orphan outputs, multiple generators, cycles and forbidden rights purposes must fail closed. Canonical digest output must be invariant to permitted record ordering and change for semantic modifications.

## Out of scope

Digital signatures, key management, trust roots, notarization, repository access controls, legal adjudication, raw acquisition validation, external custody assurance and source authenticity are not implemented.
