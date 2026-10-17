# City Belonging Navigator — outcome and harm-aware evaluation contract v0.1

**Status:** proposed evaluation protocol; no user study, local service coverage, partner relationship, or efficacy outcome is claimed.  
**Companion implementation:** [synthetic admission-policy fixture](fixtures/city-belonging-admission-v0.1.md) and [standalone Rust policy checker](../../tools/city-belonging-admission/README.md).  
**Tracking:** [CITY-CONNECTION-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/123), [CITY-CONNECTION-002](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/125).

## 1. The paradigm shift

Do not measure success as “the app showed many listings,” “the user clicked,” or “the user stayed longer.” Treat the navigator as **reliable, consented access to a suitable real-world next step**.

That means separately evaluating:

1. **Data truth and currentness:** what claims can the source actually support today?
2. **Task success:** can a person find an option that fits their stated constraints and understand its limitations?
3. **Action success:** did the person choose and complete a next action they wanted, such as checking details or contacting a provider?
4. **Human outcome:** did the activity or referral work for this person, as self-reported on terms they chose?
5. **Harms and exclusion:** did the system mislead, stigmatize, expose, pressure, or systematically fail anyone?

These are separate outcomes. A technically valid record does not prove service quality; a good search session does not prove an actual connection; and a completed referral does not by itself prove that loneliness or social isolation decreased.

## 2. Research rationale

The [WHO Commission on Social Connection](https://www.who.int/groups/commission-on-social-connection) reported on 30 June 2025 that loneliness and social isolation are widespread and called for coordinated work across policy, research, intervention, measurement/data, and engagement. Its report also identifies important gaps in rigorous evaluation, including limited high-quality evidence for community interventions and research needs in low- and middle-income settings. This motivates local evidence generation rather than assuming that a digital directory is effective.

A 2025 BMJ Open umbrella review, [“Potential harms of social prescribing: a global umbrella review and dark logic model”](https://doi.org/10.1136/bmjopen-2025-108998), argues that unintended effects and governance deserve attention and cautions against relying on one outcome measure alone. Treat that as a reason to predefine plausible harms and multiple measures—not as proof that every social-prescribing intervention is harmful.

Open Referral's [HSDS schema reference](https://docs.openreferral.org/en/v3.3/hsds/schema_reference.html) specifies separate objects and their relationships; its documentation notes that object schemas define structure and required/optional fields. A schema check is therefore a format/structure gate, not evidence that a real service exists, remains open, or is suitable. This branch now pins nine HSDS 3.1 resource-schema files and checks selected row fields, required/tabular-required fields, types, formats, enums and relationships; it **does not yet claim full canonical HSDS conformance**, because nested reference semantics and all upstream schema rules have not been implemented or run through the official full schema/profile toolchain.

## 3. Evaluation layers and proposed measures

| Layer | Primary measure | Guardrail / reporting rule |
|---|---|---|
| Schema and integrity | Exact fixture bytes, manifest hash, schema/profile version | Hash mismatch or malformed input fails closed; never silently rewrite a manifest |
| Permission and provenance | Share of published records with disposition for reuse rights, upstream lineage, observation time, verification state and correction route | 100% for any displayed record; 0 displayed records with unresolved reuse permission |
| Service currentness | False-current rate on independently adjudicated sample; stale/closed detection sensitivity | Target 0 false-current results on frozen synthetic qualification fixtures; live results must include denominator and uncertainty |
| Claim-level uncertainty | Unknown-to-positive conversion rate for cost, eligibility, access, language, schedule, capacity and operation | 0 on regression fixtures; unknown must remain visible as unknown |
| Search task success | Share of representative tasks where the participant identifies at least one eligible option satisfying declared constraints | Establish baseline with a fixed, preregistered task bank; compare alternatives on the same tasks |
| Constraint comprehension | Correct understanding of cost, schedule, eligibility, location precision, accessibility and uncertainty | Report field-by-field; do not hide a failure inside an average |
| Actionability | Share who choose a wanted next action; share whose intended provider contact is confirmed as successful where measurement is consented and feasible | Report separately from search completion, listing clicks and outbound link opens |
| User agency | Unwanted-contact rate; successful opt-out; ability to correct a result and leave without explanation | 0 coerced referrals; 100% of recorded opt-outs honoured in the controlled suite |
| Harms | Incorrect or unsafe recommendation reports, distress/stigma, privacy incidents, false expectations, unmanageable provider workload | Publish incident categories and severity, remediation time, and denominator; pause affected claims on credible serious errors |
| Equity and accessibility | Task success and data coverage across language, disability/access needs, device/connection limits, transport/time constraints and pilot catchment | Report per stratum where sample and consent permit; disclose small/unknown groups rather than claiming fairness from a citywide average |
| Community reciprocity | Provider correction burden, time spent on stale/bad referrals, provider ability to withdraw/update information | Measure provider-side burden and feedback, not only user-side benefit |
| Longer-term social outcomes | Optional, validated measure of social connection/belonging at baseline and follow-up, chosen with independent research guidance | No loneliness-reduction claim without an appropriate design, preregistered analysis, participant consent, missingness analysis and limitations |

### Why a dashboard of a dozen metrics is not a single score

Do not average “currentness,” “safety,” “task completion,” “equity,” and “belonging” into one ranking. A high click-through rate must not cancel out a misleading cost claim; a high mean task score must not hide a group that cannot use the service. Use a small set of release gates plus separate outcome and harm reports.

## 4. Qualification targets

### Gate A — synthetic regression (required before exposing real records)

- 100% of fixture cases have deterministic expected dispositions and safety flags.
- 0 fixture cases with unresolved permission can be published.
- 0 stale, disputed or closed fixtures are surfaced as current.
- 0 unknown cost/accessibility/capacity fields become positive claims.
- 0 synthetic service-area/virtual records receive fabricated street pins.
- 100% of fixture decisions are reproducible for identical bytes, evaluation date and policy version.
- 100% of records have required lineage fields in the local profile; field values that are unknown remain explicit.
- 100% of negative controls in the frozen regression set are detected.

The thresholds apply only to the controlled synthetic set. They do not establish that the same error rate will hold in a live city.

### Gate B — baseline usability study

Before choosing a claimed percentage uplift, recruit a small, diverse, compensated usability cohort where feasible and use identical tasks against:

- a plain keyword/list baseline;
- the navigator's filters and evidence labels;
- optionally, a human-assisted discovery path only if a real partner accepts it.

Pre-register task wording, stop rules, inclusion/accessibility approach, missing-data handling and primary outcome. Report raw counts, denominators, confidence intervals and task-level confusion (false match, missed match, unknown constraint, unsuitable result). Do not choose only participants who already know the relevant service landscape.

### Gate C — bounded live pilot

Only begin after the specific source's reuse terms are established, providers or responsible source operators verify the fields claimed as current, and correction/withdrawal workflows have an owner. Set the size by the actual verified inventory, not a quota; an empty category is a coverage gap, not a reason to invent data.

At minimum, monitor record age by source/category, stale/incorrect listing rate, successful contact rate, false or failed referrals, opted-out contacts, correction resolution time, user-reported constraint comprehension, provider burden, and incident severity. Stop publishing the affected claim promptly when a credible report undermines critical currentness or safety.

### Gate D — outcome and causal evaluation

A directory-only pilot can usually establish feasibility, coverage and task outcomes. To claim improved social connection, loneliness, wellbeing or belonging, use appropriate validated measures and an independently reviewed design. If estimating a causal effect, include a defensible comparator or other suitable identification strategy, adequate sample-size reasoning, follow-up, attrition and adverse-event reporting. A before/after change without a counterfactual is not enough to attribute change to the navigator.

## 5. Non-goals and privacy boundaries

- No inferred loneliness, religion, trauma history, mental-health status or developmental stage.
- No sensitive intake before basic discovery; no account or exact location required for the simplest search.
- No retention of identifiable search histories merely to optimize engagement.
- No claim that a listing or referral is safe solely because a state agency, map or schema contains it.
- No provider partnership, municipal endorsement, citywide completeness or intervention efficacy claim without evidence.
- No gamified streaks or ranking people by participation, social confidence, spiritual beliefs, or perceived progress.

## 6. Current evidence and open engineering gates

The synthetic admission fixture currently includes eleven cases with manifest SHA-256 `fa66bc059b780ce6e3ea5ba37689c48b4c14952c7a386e7fe1c8403806145b79`. Its manifest pins the evaluation date (2026-10-17) and a 30-day provider-confirmation freshness window, making the stale-state decision reproducible. The 30-day threshold is an illustrative test policy, not a universal rule; production policies should be field-specific and locally reviewed. The standalone Rust checker now verifies the pinned schema-file bytes and selected HSDS row contract, with 20 authored golden and negative-control tests, but **has not been compiled or run here**. Any hand-executed mirror or static inspection is not a Rust test pass.

The next engineering gates remain:

1. pin the exact immutable upstream HSDS schema bundle and all referenced schema hashes;
2. implement and hash a versioned local admission profile for source lineage, reuse rights, field-level verification, and user corrections;
3. run upstream schema validation separately from local policy tests;
4. execute the standalone Rust tests at the exact commit and attach the command, environment, fixture digest, and receipt;
5. get independent local/community review of taxonomy, harms, language, data rights, and contact/withdrawal process before live ingestion.

## Sources

- World Health Organization, [Commission on Social Connection](https://www.who.int/groups/commission-on-social-connection) and the 2025 flagship report, published 30 June 2025.
- BMJ Open, [Potential harms of social prescribing: a global umbrella review and dark logic model](https://doi.org/10.1136/bmjopen-2025-108998), 2025.
- Open Referral, [HSDS schema reference](https://docs.openreferral.org/en/v3.3/hsds/schema_reference.html). Version 3.3 documentation used here to support the general distinction between schema structure and record truth; this is not a substitute for pinning HSDS 3.1 schema bytes for implementation.
