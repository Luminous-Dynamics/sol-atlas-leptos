# Federated Industrial Twin Evidence v0.1

**Status:** Source-native evidence and bounded observation contract. This is not a protocol implementation, universal interoperability claim, safety case, or standards conformity assessment.

## 1. Architectural shift

Treat a composed industrial twin as a federation of evidence-bearing systems, not a flattened copy. Preserve the source record and source semantics. Normalized values are an additional projection with a declared transform artifact; they never replace the native observation.

`native source → read-only connector → observation capability boundary → source-native evidence envelope → IEAC admission`

The connector declares exactly what it can observe within a pinned inventory and scope. The evidence envelope records what it actually observed. Those are different contracts and must be verified against each other.

## 2. Source-native evidence envelope

The envelope binds the source system, protocol, endpoint, native schema URI/version, connector build digest, record identity, raw payload URI and digest, original field path/type/unit/value, source time, receipt time, clock domain/synchronization/uncertainty, provenance lineage, source rights and observation scope.

A normalized semantic projection is separate:
- `source_native_only` or `preserved`: no normalization claim.
- `exact_converted`: a versioned transform artifact and digest are mandatory, with normalized unit/value and uncertainty transformation status.
- `lossy_projection`: a transform artifact, loss description and uncertainty state are mandatory. The envelope cannot claim complete observation.
- `unavailable`: no semantic projection is made.

Unit, time and coordinate-frame conversions require explicit transform artifacts. A transform name without artifact identity/digest is insufficient.

## 3. Bounded observation completeness

The connector profile declares:
- source inventory snapshot identity/digest and trust state;
- the inventory and asset/time/network/permission scope;
- every declared channel and native field path;
- supported, partial or unsupported channel capability;
- sampling, time, unit and duplicate semantics;
- pagination/history completeness, loss counters, watermarks and unknown-field policy;
- known blind spots.

The evidence envelope separately records declared, observed, omitted and unsupported channel IDs. `complete_within_declared_inventory` is permitted only when the inventory is authenticated or operator-pinned, no declared channels are omitted or unsupported, there are no blind spots within that inventory, and the capture-loss counter is exactly zero.

This is intentionally not a claim of whole-system observability. An adapter cannot prove that a source inventory itself is complete merely by reading that inventory. The inventory root and update process require external trust. A source can also have hidden dependencies outside the declared scope. Such limitations remain explicit in `known_blind_spots` and `scope_definition`.

Unknown, unavailable, partially visible, paginated-but-incomplete or lossy channels must not be silently called complete. If the connector cannot establish completeness, it must emit partial or invalid state and IEAC should abstain where the missing scope matters to the decision.

## 4. Protocol profiles

The v0.1 contract is protocol-neutral but names likely adapter families:
- OPC UA / OPC UA FX: use native node identity, namespace URI/version, status/quality, source timestamp, sequence and subscription semantics.
- MTConnect: preserve device/component/data-item identity, stream/sequence and native units.
- QIF: preserve product/feature/measurement identities, schema version, units, tolerances and measurement lineage.
- Vendor API/file exchange: preserve native record identifiers, cursor/page semantics, schema versions and payload bytes/digests.

This work does not implement these connectors. It defines what an adapter must attest to before its observations can become decision evidence.

## 5. Test matrix and benchmark

Hand-calculated golden fixtures and bounded oracles should cover:
- namespace/schema evolution, unknown fields and identifier collisions;
- SI/imperial conversions, dimensional mismatch, affine offsets and precision loss;
- timezones, source/receipt clock skew, unsynchronized clocks, sequence gaps, duplicates, reordering and replay;
- coordinate-frame and datum transforms;
- uncertainty propagation and missing uncertainty;
- incomplete pagination, subscription loss, connector restart, watermark regression and partial source outage;
- rights revocation, unverified source inventory, provenance gaps and connector build drift;
- unsupported channels, omitted channels, declared inventory shrink/expansion and hidden common-mode dependencies.

Acceptance:
- 100% preservation of declared decision-relevant source-native fields in round-trip fixtures.
- Zero silent drops, unit guesses, identifier merges or timestamp reinterpretations.
- Every transform is versioned, digest-bound and reproducible; lossy conversion is explicit and cannot satisfy complete-observation state.
- 100% of unsupported/unobserved declared channels surface as partial/incomplete.
- Exact payload, native schema, connector build and inventory snapshot identities are retained.
- Zero semantic verifier disagreements on golden vectors.
- Per-protocol, per-composition-mode and per-failure-family reporting. No pooled score may hide weak adapters.

Latency, throughput and bandwidth targets must be measured on pinned hardware and networks before assigning thresholds. Correctness is the first gate; do not invent performance targets in the absence of a real test environment.

## 6. Composition modes and industry position

ISO 23247-6:2026 distinguishes integrated, unified and federated twin composition. Qualify all three:
- integrated where common ownership/schema and tight coupling are appropriate;
- unified where multiple twins retain autonomy but share a governed interface;
- federated where independent parties/systems remain autonomous and exchange bounded evidence.

Use OPC UA FX, MTConnect and QIF as existing protocols and information models rather than inventing another field transport. Integrate with Siemens simulation/visualization, AWS TwinMaker, Palantir Foundry and Bentley iTwin as evidence sources or execution environments; this project is not trying to replace them.

## 7. Claim ceiling

Synthetic contract fixtures demonstrate only that the stated contract behaves as tested. They do not prove source inventory completeness, source truth, physical validity, field safety, protocol certification, cyber assurance, live performance, universal interoperability or compliance with ISO/OPC specifications. Those require source-specific assessment and independent field verification.
