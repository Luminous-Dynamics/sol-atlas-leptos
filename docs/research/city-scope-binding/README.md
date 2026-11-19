# CityMaker scope binding corpus v0.1

This is a finite, synthetic conformance corpus for the common scope binder. It checks that cross-owner evidence refers to the same declared space/time/population/scenario/functional unit/design context before independent domain outputs are composed.

## Scope

The fixture uses a synthetic 4x4 local grid with no real-world coordinates. It is not Johannesburg or another actual district. It carries synthetic population groups, a 12-month horizon, two scenarios, one weather profile, two functional-unit definitions and a design/BOM identity.

## Contract

- Canonical scope serialization recursively sorts JSON object keys and sorts the explicitly set-like arrays by stable identity: population group IDs, scenarios by scenario ID, each scenario's event IDs, service requirements by requirement ID, functional units by functional-unit ID, and design revisions by subject ID. Other arrays preserve their declared order.
- The scope digest is SHA-256 of the canonical scope object using `sha256-canonical-citymaker-scope-v1`; the digest field is external and does not hash itself.
- Each owner artifact is opaque to the binder. The binder checks owner/contract/version, schema artifact digest/algorithm, source revision, receipt ID, content digest, subject digest, scope, boundary, population frame, scenario, phase window, functional unit, weather profile, applicable design/BOM revision and evidence state.
- The owner payload digest is **not** verified by this binder because payload bytes are not retrieved. A supplied digest is only a reference. The result always reports payload_bytes_verified=false.
- Owner disposition is preserved unchanged. Binding a Blocked or ImportDependent result does not turn it into a successful or local-production claim.
- Exact draft owner schemas may be pinned for synthetic conformance, but draft status remains visible. The Mycelix Living-City schema in the corpus is an unmerged draft PR artifact.
- Missing canonical owner schemas remain Unknown; the binder does not infer schemas from familiar field names in consumer code.

## Binding states

- Bound: required metadata exactly matches the registered scope/schema/revision.
- Unknown: required schema/receipt evidence is missing, stale, revoked or unverifiable.
- NotComparable: a valid artifact belongs to another scope, population, horizon, scenario, functional unit, climate profile or design revision.
- Unsupported: malformed identity/digest, duplicate ID or unsupported contract version.

These are binding states only. Raw owner dispositions are not rewritten.

## Claim ceiling

A passing case proves only metadata compatibility for the declared synthetic scope. It does not validate the owner payload, a product, supply chain, city service, construction design, code compliance or a real district.
