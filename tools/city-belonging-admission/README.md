# City Belonging Admission Policy Checker

Standalone Rust CLI for checking a **synthetic admission-policy fixture** used by the Sol-Atlas City Belonging Navigator work. It is intentionally separate from the Sol-Atlas UI and does not need to join the repository root workspace.

## Run

From the repository root, with Rust and dependencies available:

    cargo run --manifest-path tools/city-belonging-admission/Cargo.toml -- verify \
      docs/research/fixtures/city-belonging-admission-v0.1.jsonl \
      docs/research/fixtures/city-belonging-admission-v0.1.manifest.json

The manifest pins both `policy_as_of_date` and `provider_confirmation_max_age_days` (30 days for this synthetic fixture). The 30-day value is illustrative and must be revisited with local operators and field-specific risk; it is not a universal standard.

Run its tests:

    cargo test --manifest-path tools/city-belonging-admission/Cargo.toml

## Policy precedence

1. Unresolved reuse rights block publication and yield `CandidateOnly`.
2. Material source conflicts, failed contact checks, or an unverified correction yield `ReviewRequired`.
3. A provider-confirmed closed service yields `ClosedOrWithdrawn`.
4. Expired schedule, explicitly past event, stale capacity, or provider confirmation older than the manifest-pinned freshness window yields `StaleOrDisputed`.
5. Only a currently active service with provider-supplied source, non-empty provider-confirmed fields, a provider-verification timestamp within the manifest-pinned age window, and explicitly confirmed operational status may yield `ProviderConfirmed`.
6. Everything else remains `CandidateOnly`.

The precedence is intentionally conservative and must be reviewed with local operators before any production use. The fixture's `approved_for_fixture_only` value is synthetic test-only permission, not a real data licence.

## Derived display claims

- Only `ProviderConfirmed` entries may be surfaced as current by this fixture policy.
- A map pin requires a physical location type plus provider-confirmed coordinates and latitude/longitude; virtual/service-area entries never receive a street pin, even if coordinates are supplied.
- “Free” requires a separately confirmed free-cost state; missing/unknown cost does not qualify.
- “Accessible” requires a separately confirmed accessibility state; unknown accessibility does not qualify.
- Live capacity requires an explicitly live-confirmed field state and an authorised provider-live source; a historic or directory estimate cannot qualify.

## Failure behaviour and outputs

The verifier checks the fixture SHA-256 before parsing records, confirms the pinned schema-manifest SHA-256 and each vendored source-schema digest, checks the selected HSDS 3.1 row contract and cross-row IDs, requires newline-terminated JSONL, rejects blank lines, malformed JSON, duplicate fixture IDs and manifest count/disposition drift, then compares every expected admission decision and display-safety flag to the policy function. Errors return a non-zero CLI exit; manifests are never rewritten automatically.

The report says `SYNTHETIC_POLICY_FIXTURE_MATCH` only when the synthetic fixture matches this policy. It also verifies byte hashes for nine vendored Open Referral HSDS 3.1 resource-schema files and validates selected row properties: allowed fields, required plus tabular-required fields, declared scalar types, formats, enums, and cross-row relationships. The input resource rows are normalized (not a full nested HSDS API document). This is **not full HSDS conformance**: nested `$ref` semantics and every upstream schema rule are not fully implemented, and the standard's full schema/profile toolchain has not been run. A pass is not evidence that any real listing is open, safe, affordable, accessible, available, or permitted for reuse. No real service data are imported.

## Qualification targets

- 100% policy agreement on the frozen synthetic fixture.
- 0 entries with unresolved reuse permission surfaced.
- 0 stale, disputed or closed cases labelled current, including a provider confirmation older than the pinned freshness window.
- 0 unknown cost/accessibility claims upgraded to positive statements.
- 0 virtual/service-area entries given fabricated street pins.
- Byte-identical receipt for identical fixture bytes and policy version.
- Human task-success and actual-contact outcomes measured separately from search/click completion.

## Pinned upstream sources

- [Selected HSDS contract manifest](../../docs/research/fixtures/hsds-3.1-selected-contract-v0.1.json)
- [Local admission evidence profile](../../docs/research/fixtures/city-belonging-admission-profile-v0.1.json)
- [Vendored HSDS 3.1 schema files](../../docs/research/fixtures/upstream/hsds-3.1/schema/)
- [Open Referral HSDS license](../../docs/research/fixtures/upstream/hsds-3.1/LICENSE) — CC BY-SA 4.0
- Canonical upstream source: https://github.com/openreferral/specification/tree/3.1/schema

The vendored schema bytes are pinned and used for this branch's selected-resource field contract. Full HSDS/profile-tool validation, including nested references, remains a separate gate.
