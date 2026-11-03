# ITIP federated evidence reference v0.1

**Status:** Source-native reference contract and synthetic schema fixture. No production connector or verified data ingestion is claimed.
**Tracks:** [ITIP-FED-001 / #366](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/366).

## 1. Design principle

Do not replace a vendor's asset graph or force its native data model into a Luminous-owned universal twin schema. Preserve the source-native system, namespace, object ID, schema identity/version, record ID, exact payload digest, sequence and timestamps. Any normalized view is a separate derived evidence artifact.

The evidence reference is deliberately not a copy of the source payload. It carries stable source identity and a payload URI/digest, then adds only explicit semantic references, measurement summaries, rights and transformation lineage. A consumer can dereference the permitted source record or audit the original payload independently.

## 2. Contract

The companion schema `schemas/industry-federated-evidence-reference-v0.1.schema.json` defines:

- **Source identity:** system ID, namespace, native object ID, native schema URI/version/digest, source record ID, payload URI/digest, source sequence, observation time and ingestion time.
- **Rights:** rights state, basis, permitted uses and optional export permission. Unknown rights require an empty permitted-use list; the adapter must not infer permission from successful retrieval.
- **Observations:** native property path, semantic concept, value, unit, uncertainty interval, observation time, evidence state and semantic mapping status.
- **Transformations:** input digests, operation/version, parameter digest, implementation digest, output digest and explicit lossiness classification.
- **Unsupported semantics:** fields or concepts retained in the native payload but not yet mapped to a reviewed Luminous semantic contract.

Every normalized or converted quantity must retain the native property path and original unit/value in its source payload. Converted observations are marked `derived`, identify the conversion rule and bind the derived-value digest. An unsupported or opaque semantic must remain `unknown`; it cannot be silently treated as a valid decision variable.

## 3. Cross-object validation requirements

JSON Schema validates structure, not a whole transformation graph. A runtime adapter/verifier must additionally prove:

1. The first transformation input binds the declared native payload digest.
2. Each subsequent transformation input binds the previous output digest; no unreferenced or skipped transform is accepted.
3. The final transformation output digest matches the derived observation digest.
4. The exact operation, parameters, implementation and version are identified for every conversion.
5. No unit conversion is implicit. A conversion requires a reviewed rule, explicit source/target units and a replayable transformation.
6. Source sequence and timestamps satisfy the relevant freshness and anti-rollback policy.
7. Rights/permissions are scoped to the requested evaluation use and destination. A data-read permission is not automatically a redistribution or operational-use permission.
8. Unsupported semantics remain opaque. If a decision depends on an unsupported semantic, the runtime must abstain or reject rather than guess.

## 4. Synthetic example

`schemas/examples/ieac-federated-evidence-reference-synthetic-v0.1.json` shows a synthetic pump vibration observation. The example uses a native object ID and property path, a source schema digest, a source payload digest, explicit `mm/s` units and uncertainty bounds. No conversion is applied, so the transformation chain is empty and the original representation remains authoritative.

The example's `urn:synthetic:*` identifiers and digests are fixtures, not actual OPC UA, Siemens, AWS, Bentley or Palantir source artifacts.

## 5. Benchmark plan

Use adapter-specific frozen conformance suites. Report results by source system, schema version, field type and transformation family; never hide a weak connector behind a pooled score.

- **Identity preservation:** all native IDs, schema versions and source digests survive import/reference/export exactly.
- **Semantic fidelity:** known-answer unit, enum, timestamp, uncertainty and relationship mappings; unknown mappings remain opaque.
- **Lineage completeness:** every derived value is replayable from the exact source payload and transformation inputs.
- **Metamorphic invariance:** property ordering and lossless serialization do not change decision-relevant semantics or admission.
- **Mutation resistance:** one-bit digest changes, namespace collision, unit substitution, schema drift, stale source sequence, rights revocation and transform parameter mutation are detected.
- **Federation scale:** measure p50/p95/p99 lookup/admission latency, memory and throughput across increasing source counts, graph depth, evidence count and schema diversity.
- **Partition behavior:** stale cache, missing source, unavailable authority and revoked source remain distinct; cached data cannot renew its own authority.
- **Independent agreement:** independently implemented adapters/verifiers agree on the exact source identity, semantic status and admission result.

## 6. Competitive boundary

OPC UA/UAFX, AWS IoT TwinMaker, Siemens Digital Twin Composer, Bentley iTwin and Palantir Foundry already have significant interoperability, ontology, engineering and connector capabilities. The competitive claim is not that Luminous can replace them. It is that Luminous can qualify a particular decision across sources which retain separate ownership, native schemas and authority, and provide a portable replayable assurance receipt.

Do not claim production integration until API terms, licenses, access rights, customer authorization and version compatibility are established. Start with public schemas, generated fixtures and operator-authorized exports.

## 7. Claim ceiling

This is an evidence-reference/lineage contract. It does not authenticate the source, prove a measurement true, grant data rights, verify a physical model or authorize a decision. Signed source authority belongs to IEAC trust v0.2; decision admission remains a separate gate.

## References

- ISO 23247-6:2026, Digital twin composition: https://www.iso.org/standard/84677.html
- OPC UA overview: https://opcfoundation.org/about/opc-technologies/opc-ua/
- AWS IoT TwinMaker developer guide: https://docs.aws.amazon.com/iot-twinmaker/latest/guide/what-is-twinmaker.html
- Bentley iTwin platform: https://www.bentley.com/software/itwin/
- Siemens Digital Twin Composer: https://www.siemens.com/global/en/products/automation/topic-areas/digital-enterprise/digital-twin-composer.html