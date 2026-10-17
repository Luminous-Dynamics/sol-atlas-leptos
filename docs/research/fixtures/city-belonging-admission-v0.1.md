# City Belonging Admission Fixture v0.1

**Status:** synthetic research fixture; not production data and not yet upstream-schema validated.  
**Parent:** [CITY-CONNECTION-002](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/125).  
**Companion corpus:** `city-belonging-admission-v0.1.jsonl`; exact byte hash is in its manifest.

## Paradigm shift: an evidence-qualified access system

Treat the directory as a **real-world access workflow**, not a scraped map or an engagement funnel. A location pin, an organization record, a current service, a working contact path, provider confirmation, permission to republish, and actual live capacity are distinct facts. They must not be compressed into one trust score.

The fixture contains eleven synthetic decision cases. Every organization, service, URL and location is synthetic; no vulnerable-client data, personal contact details, or real coordinates are included.

## Fixture cases

| Case | Expected disposition | Negative control |
|---|---|---|
| CBF-001 | ProviderConfirmed | Named fields only; unknown values stay unknown |
| CBF-002 | StaleOrDisputed | Expired event cannot be shown as current |
| CBF-003 | ReviewRequired | Broken contact blocks reliable next action |
| CBF-004 | ProviderConfirmed | Unknown cost/accessibility never become positive claims |
| CBF-005 | ReviewRequired | Conflicting claims preserve both source lineages |
| CBF-006 | ReviewRequired | User correction is not auto-authoritative |
| CBF-007 | StaleOrDisputed | Historical capacity is not live availability |
| CBF-008 | ClosedOrWithdrawn | Provider-confirmed closed service is suppressed |
| CBF-009 | CandidateOnly | Unresolved reuse permission blocks publication |
| CBF-010 | ProviderConfirmed | Service-area/virtual option has no fabricated street pin |
| CBF-011 | StaleOrDisputed | Provider-confirmed status older than the manifest-pinned 30-day window is not current |

Each JSONL record contains synthetic HSDS-shaped fragments (organization, service, location, service-at-location and schedule where relevant), a separate local admission-evidence envelope, and an expected decision. Local evidence fields are intentionally kept out of the HSDS fragments so a future profile can be versioned separately.

**Schema honesty gate:** the manifest names the HSDS 3.1 source reference but leaves its schema-bundle hash blank. The exact upstream schema bytes have not been retrieved and hashed in this change, and the fixture has not been validated against the canonical schema. The fixture hash establishes corpus integrity only, not HSDS conformance.

## Admission invariants

1. **Separate permission from verification.** Permission to reuse data does not verify a service; provider confirmation does not grant permission.
2. **Field-scoped verification.** Verification applies only to named fields and the time evidenced. At the fixture's pinned evaluation date, provider confirmation older than 30 days is stale; that is a test policy, not a universal freshness standard.
3. **Fail closed on material ambiguity.** Conflicts affecting address, operation, eligibility, access, contactability or suitability require review.
4. **Unknown never becomes positive.** Unknown cost, accessibility, eligibility, language, schedule, capacity and operation remain explicit unknowns.
5. **No single trust score.** Preserve provenance and independent claim states.
6. **Currentness is not capacity.** An active service does not imply live availability.
7. **Schema validity is not truth.** Valid data says nothing by itself about service quality, safety, affordability or whether people feel welcome.
8. **Privacy by default.** Basic discovery needs no account, exact location, sensitive profile, spiritual/religious identity, inferred loneliness or search-history retention.
9. **List-first and low-bandwidth.** Lack of coordinates must not erase a useful virtual or service-area option.
10. **Corrections are evidence, not hidden edits.** Keep a reviewable, privacy-conscious correction lineage.

## Benchmark gates

| Metric | Initial controlled-suite target | Qualification |
|---|---:|---|
| Cases with explicit expected disposition | 100% | No implicit acceptance |
| Published records with unresolved permission | 0 | Rights gate |
| False-positive current-service rate | 0% target | On frozen, adjudicated regression cases only |
| Detection of stale/closed/conflicting fixtures | 100% | Not a real-world guarantee |
| Unknown turned into a positive claim | 0 | For example, unknown cost must not become “free” |
| Material conflict silently auto-resolved | 0 | No recency-only tie breaker |
| Virtual/service-area record given fake pin | 0 | No fabricated precision |
| Repeatability for same input/policy version | 100% | Deterministic receipts |
| Provenance completeness on published claims | 100% | Field-level lineage and permission disposition |
| Real-world task success | Baseline first, then test improvement with confidence intervals | No arbitrary uplift before baseline |
| Successful contact/action rate | Report separately from search completion | Clicks are not successful connections |
| Loneliness-reduction claims | None without a consented, appropriate longitudinal evaluation | Listings alone are not causal evidence |

The zero-error target is a strict synthetic regression gate, not a promise about a live city. Pilot reports must include denominators, uncertainty intervals, source coverage, field-missingness and access gaps.

## Next engineering gates

1. Retrieve the official HSDS 3.1 schema bundle from an immutable upstream commit; pin exact bytes and hashes for every referenced schema; include attribution/licence.
2. Implement and hash a minimal namespaced local admission profile.
3. Validate fixtures separately for schema conformance and admission-policy outcomes.
4. Add deterministic negative tests for duplicate IDs, missing lineage, malformed input, stale events, conflicting sources, permission unknown, closed services, and unknown-field propagation.
5. Only then consider ingestion; review actual source terms and rights first.

## Source basis

- [Open Referral HSDS 3.1 overview](https://docs.openreferral.org/en/3.1/hsds/overview.html)
- [HSDS 3.1 change notes](https://openreferral.org/upgrading-our-standards-introducing-version-3-1-of-the-human-service-data-specifications/)
- [WHO Commission on Social Connection and 2025 report](https://www.who.int/groups/commission-on-social-connection)
