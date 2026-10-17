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
- A map pin requires provider-confirmed coordinates as well as latitude/longitude; no coordinates means no pin.
- “Free” requires a separately confirmed free-cost state; missing/unknown cost does not qualify.
- “Accessible” requires a separately confirmed accessibility state; unknown accessibility does not qualify.
- Live capacity requires an explicitly live-confirmed field state and an authorised provider-live source; a historic or directory estimate cannot qualify.

## Failure behaviour and outputs

The verifier checks exact SHA-256 before parsing, requires newline-terminated JSONL, rejects blank lines, malformed JSON, duplicate fixture IDs and manifest count/disposition drift, then compares every expected decision and safety flag to the policy function. Errors return a non-zero CLI exit; manifests are never rewritten automatically.

The report says `SYNTHETIC_POLICY_FIXTURE_MATCH` only when the fixture matches this version of the policy. It is not evidence that a real listing is open, safe, affordable, accessible, available, or permitted for reuse. It does not yet validate against the canonical upstream HSDS 3.1 JSON Schemas; the fixture uses HSDS-shaped fragments and a separately versioned local evidence envelope. No real service data are imported.

## Qualification targets

- 100% policy agreement on the frozen synthetic fixture.
- 0 entries with unresolved reuse permission surfaced.
- 0 stale, disputed or closed cases labelled current, including a provider confirmation older than the pinned freshness window.
- 0 unknown cost/accessibility claims upgraded to positive statements.
- 0 virtual/service-area entries given fabricated street pins.
- Byte-identical receipt for identical fixture bytes and policy version.
- Human task-success and actual-contact outcomes measured separately from search/click completion.
