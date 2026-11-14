# Home assembly substitution blast-radius benchmark v0.1

**Study:** HOME-ASSEMBLY-SUBSTITUTION-001  
**Parent:** HOME-ASSEMBLY-DEMAND-001 (#803)  
**Maturity:** SYNTHETIC_KNOWN_ANSWER  
**Execution:** NO_PRODUCT_EQUIVALENCE_DECISION

## Research question

When a product, certificate, interface, batch, supplier, quote, geometry revision or manufacturing method changes, can we identify exactly which prior claims must be withdrawn for requalification—and preserve claims that genuinely do not depend on the changed input?

This is dependency propagation, not a product-equivalence solver. It must not claim that two products are physically or legally equivalent.

## Algorithm

Each case declares the assembly, original/replacement product references, dependency catalogue, changed or now-unknown inputs, prior claims and the exact dependencies each claim consumed. Independently authored expected sets identify invalidated and preserved claims.

The baseline algorithm computes the union of changed/unknown dependency IDs, then invalidates every claim whose dependency set intersects that union. A claim with no intersecting changed dependency retains its previous state; the evaluator does not promote it.

If the exact product identity changes, the case must carry a PRODUCT_IDENTITY change. If no dependency changes, the result is NO_DEPENDENCY_CHANGE. Otherwise, REQUALIFICATION_REQUIRED.

This is a transparent baseline. Later work may add typed dependency edges and transitive closure, but it must beat the baseline on exact known answers and cannot weaken owner gates.

## Frozen initial corpus

The first corpus contains 18 synthetic cases:
- no-change positive control;
- SKU identity change;
- fire certificate revision;
- interface tolerance and fixing pattern;
- approval application scope;
- thermal result;
- moisture result;
- acoustic result;
- batch/receiving/warranty;
- local availability and lead time becoming unknown;
- price quote and tax scope;
- manufacturing process/tool capability;
- repair/disassembly method;
- multi-property change;
- fire evidence becoming unknown;
- geometry revision/takeoff dependence;
- supplier region and lead time;
- EPD/embodied impact revision.

These are 18 authored known-answer fixtures; the P0 target remains at least 40 independent cases before broad evaluation. Do not count generated permutations as independent evidence.

## Metrics

- Invalidated-claim recall: all dependent claims withdrawn.
- Invalidation precision: unrelated claims preserved.
- Missed invalidation count.
- Extra invalidation count.
- Exact set agreement with independent known answers.
- Review/requalification work count by claim owner.
- Blast-radius size and distribution by change type.
- False equivalence and unauthorized re-promotion count (both target zero; this evaluator has no authority to issue either).

No product comparison or engineering performance result is produced by this benchmark.

## Claim ceiling

Synthetic dependency propagation only. It does not decide product equivalence, engineering validity, South African approval, physical safety, supplier availability, manufacturability, price or resident acceptance.
