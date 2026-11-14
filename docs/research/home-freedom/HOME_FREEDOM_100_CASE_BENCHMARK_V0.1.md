# Home Freedom 100-case corpus and independent known-answer benchmark v0.1

**Study:** HOME-FREEDOM-100-CASE-CORPUS-001  
**Parent:** [Sol-Atlas #819](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/819) / Home Freedom #800  
**Maturity:** `SYNTHETIC_FROZEN_CORPUS`  
**Execution:** `NOT_EXECUTED`

## Purpose

The first home compiler demonstration used one synthetic brief and two symbolic room programmes. This corpus expands P0 qualification to 100 matched household-intent / site-scenario pairs.

It measures whether a design workflow preserves resident-stated non-negotiables and distinguishes screening eligibility, unknown/conditional situations, hard failures and non-comparable evidence. It does not generate geometry or approve building designs.

## Household profiles

Ten synthetic profiles represent different functional programmes and preferences:
1. compact single-occupant home;
2. courtyard-oriented couple;
3. family with quiet and study zones;
4. multigenerational home;
5. explicit step-free-first home;
6. live-work / maker home;
7. shared/cooperative home;
8. care and guest flexibility;
9. rural resource-resilient home;
10. repair/retrofit-first home.

The corpus uses only non-identifying design requirements. Accessibility is included as a resident-stated functional requirement, not inferred from medical or demographic characteristics.

## Site/authority scenarios

Ten synthetic scenarios cover complete reference conditions, unknown survey, unknown rights, unknown soil, unknown municipal/code profile, failed fire access, failed step-free internal route, unknown utility capacity, failed sanitation and out-of-domain climate applicability.

Each profile declares the set of gates it requires. A failed non-negotiable required gate blocks that case; an unknown required gate leaves it conditional. A step-free failure blocks profiles that explicitly require that route, but it is not silently imposed as a personal attribute on other synthetic briefs.

## Oracle semantics

- `SCREENING_ELIGIBLE`: all required P0 gate observations are PASS. This only permits the next screening layer; it is not a buildability, compliance, safety or approval claim.
- `CONDITIONAL_CONCEPT_ONLY`: one or more required gate observations are UNKNOWN. Unknown is not failure, but it is never PASS.
- `BLOCKED`: one or more required gate observations are FAIL.
- `NOT_COMPARABLE`: required evidence is missing, duplicated, malformed, or has a non-applicable state where the profile requires it.

The fixture carries expected dispositions and reason codes separately from candidate code. A deterministic Rust evaluator recomputes the result and must agree exactly with the frozen answer.

## Qualification targets

- exactly 10 household profiles, 10 scenarios and 100 matched cases;
- 100% exact known-answer agreement;
- zero false screening-eligible results under unknown or failed required evidence;
- 100% positive-control preservation;
- deterministic result under case and observation permutation;
- duplicate IDs, missing required gates and scope mismatches become NOT_COMPARABLE or fail the corpus audit;
- no scalar score can compensate for a failed hard constraint.

Report confusion matrices by household profile and scenario, not just overall accuracy. Count false eligibility, conditional cases incorrectly promoted, hard failures missed and valid positive controls rejected separately.

## Industry comparator boundary

Compare scope-matched architect-led review, TestFit, Finch3D, Autodesk Forma, IFC/IDS information checks and the Luminous evidence bridge. Do not compare a 100-case synthetic qualifier against a professional stamped design or imply that a software product lacks capabilities outside the inspected product scope. The innovation hypothesis is traceability and hard-gate semantics across resident requirements, site evidence and design revisions.

## Claim ceiling

This corpus is synthetic. A pass only establishes agreement with the frozen P0 contract. It does not prove real buildability, code compliance, professional design, affordability, comfort, energy/water performance or resident acceptance.
