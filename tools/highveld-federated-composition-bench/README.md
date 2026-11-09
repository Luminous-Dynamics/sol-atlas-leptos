# Highveld Federated Composition Reference v0.1

Offline Rust reference evaluator for the 18-case synthetic corpus in PR #652.

## Exact claim ceiling

A pass means only that the frozen synthetic cases agree with this independent reference evaluator under the declared owner-profile, temporal-replay, identity, dependency, disclosure and service-claim mechanics.

It does not:
- authenticate source payload bytes or issuers;
- verify owner authority or live source completeness;
- establish that an engineering claim is professionally approved or applicable;
- validate IFC geometry or physical as-built condition;
- prove panel installation, building commissioning or actual service restoration;
- grant disclosure or action authority.

Every output retains the claim ceiling: synthetic_federated_composition_mechanics_only.

## Inputs

The evaluator accepts three exact files:
- the v0.1 federated twin composition corpus;
- the matching v0.1 JSON Schema document;
- HIGHVELD_FEDERATED_COMPOSITION_OWNER_PROFILES_V0_1.json.

The owner profile pins the shared record envelope and a field allowlist for each owner/schema pair: design, geometry, solver, product, provenance and workcell. Unknown nested fields fail closed. A known owner/schema ID with an unrecognized schema version is retained as an unsupported evidence condition, not interpreted by silently applying a newer or older contract.

## Strict JSON boundary

The original JSON byte stream is scanned before serde_json constructs maps. Duplicate object-member keys are rejected after decoding JSON escape sequences, so the literal key name and its Unicode-escaped equivalent collide. The standard parser then performs complete JSON syntax validation. Strict field checks run on root, case, input, temporal cut, owner inventory, owner record, identity relation, dependency edge, disclosure request, building-service claim and expected result objects.

This is a narrow duplicate-key scanner plus explicit profile validation. It is not a general JSON Schema 2020-12 implementation.

## Temporal semantics

Historical reliance uses all four declared cutoffs:
- valid time;
- source-recorded time;
- source-registered time;
- evaluator-ingested time.

Current reliance uses the corpus's separate current cut. Backdated corrections may make the retrospective assessment of a historical decision incomplete, but they never rewrite what was actually visible at the original decision cut. Late ingestion cannot leak into historical replay.

HFC-016 retains both revisions of the same logical revocation record. The evaluator does not deduplicate by record_id alone. Both versions remain in the receipt; either visible revocation version blocks current reliance. The frozen v0.1 case does not define a general revision-ordering contract, so the evaluator does not infer sequence from the textual revision label.

## Owner and authority boundaries

- Similar labels and geometry never merge distinct owner-native identities.
- Owner-authorized identity succession preserves both old and new identities; it is not a merge.
- Byte-state, issuer-authority, rights, evidence-admission, applicability and owner authentication remain separate.
- BOM and geometry dependency changes trigger current revalidation; they do not rewrite the historical as-known view.
- A panel acceptance receipt does not imply building commissioning or continuity.
- A building-service claim cannot pass when its commissioning or continuity owner references are missing.
- Disclosure returns only the requested/authorized intersection; partial disclosure is not treated as full permission.

## Run

From repository root:

~~~sh
cargo fmt --manifest-path tools/highveld-federated-composition-bench/Cargo.toml --all -- --check
cargo test --locked --manifest-path tools/highveld-federated-composition-bench/Cargo.toml
cargo clippy --locked --manifest-path tools/highveld-federated-composition-bench/Cargo.toml --all-targets -- -D warnings
cargo run --locked --manifest-path tools/highveld-federated-composition-bench/Cargo.toml -- \
  docs/research/highveld-keep/HIGHVELD_KEEP_FEDERATED_TWIN_COMPOSITION_CORPUS_V0_1.json \
  docs/research/highveld-keep/HIGHVELD_KEEP_FEDERATED_TWIN_COMPOSITION_CORPUS_V0_1.schema.json \
  docs/research/highveld-keep/HIGHVELD_FEDERATED_COMPOSITION_OWNER_PROFILES_V0_1.json \
  --source-commit "$(git rev-parse HEAD)"
~~~

The package has its own empty Cargo workspace so it can be tested without building the parent Leptos application. Dependencies are pinned and captured by Cargo.lock.

## Test coverage

- all 18 frozen known-answer scenarios;
- duplicate literal and escaped-equivalent JSON keys;
- malformed UTF-8 and trailing bytes;
- unknown root and nested owner fields;
- multiple revisions of one logical revocation record;
- four-cutoff temporal replay and future-ingestion exclusion;
- late backdated correction versus post-decision correction;
- no fuzzy identity merge;
- selective disclosure;
- unsupported schema revision;
- incomplete source inventory;
- separation of panel acceptance from building commissioning/service;
- automatic-merge negative control.

## E1 limitation

Passing these tests demonstrates deterministic synthetic composition mechanics only. Real owner data, authenticated source bytes, independent owner authorization, professional review, physical metrology and occupied service outcomes remain unqualified.
