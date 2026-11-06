# HDB-11A — explicit detection-probability estimands v0.2

## Why this is a contract change

A probability of detection is not a free-floating number. It estimates a probability under a particular sampling frame and conditioning set. A POD estimated over every independently sampled survey PSU is not interchangeable with POD estimated only after a target is known to have survived, been exposed, been observable and fallen inside the actual sensor footprint.

HDB-11 v0.1 did not encode that distinction. Passing its Wilson POD into HDB-22's conditional-detection factor and then multiplying separate survival, exposure, observability and coverage probabilities could count the same mechanisms twice.

v0.2 therefore makes the estimand explicit and binds every calibration trial to evidence for every declared conditioning event.

## Supported scope types

- **joint_per_survey_unit** — probability of detecting the known-positive target over the complete survey-unit process. Its conditioning set is empty. All eligible target PSUs enter the denominator, including those that fail upstream opportunity conditions.
- **conditional_on_target_survived_exposed_observable_and_covered** — probability of detection conditional on all four upstream events: survival, exposure, observability, and actual footprint coverage. Only PSUs for which all four are evidenced as satisfied enter the POD denominator.
- **conditional_on_target_presented_to_sensor** — probability of detection conditional on survival, exposure, observability, actual footprint coverage, and target presentation to the sensor. The profile requires all five explicit event records.
- **other_declared_conditioning_set** — an explicitly specified, non-empty set for an estimand that does not match the predeclared scopes. Its events must be unique and canonically ordered.

The profile event vector is canonically ordered by the enum order in the schema. The calibration profile and every resulting report also carry method identity, calibration-data and split-manifest digests, validation receipt, rights decision and receipt, and access tier; these are required to construct a later HDB-22 evidence stage without inventing missing provenance. Every trial must carry exactly one state for every declared event, in that same order. Each state also carries a non-empty source evidence reference and a SHA-256 digest of the evidence artifact. Missing, extra, duplicate or reordered event evidence fails closed.

## Estimation rules

Let S be the declared conditioning set. For calibration PSU i, define eligibility as 1 when every event in S is evidenced satisfied, otherwise 0.

Only eligible trials enter the empirical estimate and its Wilson interval:

POD_S = (sum of eligible detected trials) / (number of eligible trials).

The report separately records total independent PSUs, eligible PSUs, and PSUs excluded because their conditioning set was not satisfied. It also emits a SHA-256 commitment to the complete calibration-trial manifest (canonical PSU order, including each detection outcome, conditioning assertion, evidence reference/digest, reference tier and rights receipt). Each trial-level condition assertion is linked to an evidence reference and digest; the evaluator validates the reference structure and digest syntax but does not authenticate the source or rights. It also reports the scope and the exact event set. Zero eligible PSUs is a typed error. A trial attributed as a true detection while a declared conditioning event is false is rejected because it cannot be a true-positive calibration observation under that estimand; false-positive evidence belongs in its separate evaluation.

The profile's minimum sample requirement is applied to eligible PSUs, not the raw input array length. The existing 20-PSU floor remains an engineering guard, not a universal power recommendation.

## HDB-22 adapter safety

- A joint_per_survey_unit POD is a **composite** survey-process probability. Do not multiply separate survival, exposure, observability or footprint-coverage factors into it again.
- A conditional POD can be used as the sensor-detection factor only when HDB-22 receipts independently establish every conditioning event required by the declared scope.
- other_declared_conditioning_set is not automatically mappable to HDB-22. It requires an explicit reviewed mapping for every event.
- The scope enum does not authenticate trial evidence, prove the target-location distribution, or prove independence between upstream stages. The manifest digest establishes byte-level commitment to the serialized trial set, not issuer authenticity or correctness of the underlying evidence.
- HDB-15 swept-area fraction remains a geometric fraction of a survey unit. It is not a target-location coverage probability without candidate-specific geometry or an explicit target-location distribution.

This report is a source-level estimand description, not yet a complete HDB-24 adapter receipt.

## Qualification tests added

The Rust evaluator tests:
- joint scope has no conditioning events;
- conditional scope uses only trials satisfying every declared conditioning event;
- total, eligible and excluded PSU counts are separate;
- exact event set is required for built-in scopes;
- missing or reordered trial evidence fails;
- duplicate profile events fail;
- a detected true-positive trial with a failed conditioning event fails;
- zero eligible trials fail;
- presented-to-sensor scope includes all five upstream/presentation events;
- custom scopes cannot be empty and are reported literally.

A v0.2 input schema, output schema, and synthetic fixtures are provided. A dedicated workflow validates Draft 2020-12 schemas and fixtures with negative controls, then runs the core tests and formatting on the exact checkout.

## Claim limits

This contract does not establish that the conditioning events were correctly observed, that event evidence is authentic, that PSUs are independent, or that a conditional probability transfers between sites, sensors or seasons. It does not make archaeological absence or identification claims. No field data or real archaeological probability is included in the synthetic fixtures.
