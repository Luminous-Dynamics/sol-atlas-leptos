# Thermal refuge network-flow feasibility benchmark v0.1

**Study ID:** HIGHVELD-REFUGE-NETWORK-FLOW-001  
**Parent:** HIGHVELD-REFUGE-AVAILABILITY-001 / HIGHVELD-THERMAL-SERVICE-REFUGE-001  
**Maturity:** SYNTHETIC_NETWORK_FLOW_ORACLE_NOT_FIELD_VALIDATED

## 1. Why site-by-site eligibility is not enough

The availability filter asks whether one site could serve one anonymous demand envelope. A neighborhood has multiple demand groups competing for the same finite site capacity. Summing each site's apparent availability independently can double-count a refuge's seats, accessible places or safe water.

This benchmark adds an aggregate capacitated network-flow oracle. It consumes:
- anonymous demand groups with total and accessible-place demand;
- eligible site receipts with remaining total capacity, accessible capacity and safe water;
- group-to-site route evidence and accessible-route evidence from Sol-Atlas;
- one common safe-water allowance per place over a fixed service horizon.

It does not calculate routes, identify people, book places, allocate named residents or actuate refuge services.

## 2. Two-stage allocation-feasibility policy

The v0.1 policy is explicit and conservative:

1. **Accessible demand first.** Max-flow all accessible-place demand through qualified accessible routes into the accessible capacity of eligible refuge sites.
2. **General demand second.** Max-flow remaining non-accessible demand over reachable routes into remaining accessible places or general places.
3. **Joint site cap.** Convert a site's free safe water into water-supported places using the shared litres-per-place quota. Effective total site capacity is the minimum of remaining seats and water-supported places. Effective accessible capacity is the minimum of remaining accessible places and effective total capacity.
4. **No double counting.** The second flow uses only residual accessible places and non-accessible capacity after the first flow. Total assignment cannot exceed the effective site cap.
5. **No hidden assignment.** Results are group-level feasibility counts, not a commitment, reservation, route instruction or list of individuals.

This policy gives accessible demand a clear service priority before general demand. It is a benchmark policy choice, not an ethical allocation decision. Real deployment requires resident/asset-owner authority and a separately governed allocation process.

## 3. V0.1 resource simplification

All groups share the same safe-water litres-per-place quota for the declared service horizon. This permits an exact deterministic flow formulation with site capacities bounded by both seats and water.

Do not use v0.1 for groups with different water-per-person needs, duration, food, medical resource or other heterogeneous resource constraints. Such cases require a multi-resource integer optimizer with independently checked fairness, infeasibility certificates and a policy approved by the responsible authority. The current algorithm must not pretend those cases are equivalent.

## 4. Deterministic algorithm

The implementation uses a deterministic maximum-flow network:
- source to each group's accessible-demand and general-demand nodes;
- route-qualified group nodes to site accessible/general capacity nodes;
- site capacity nodes to the sink.

Accessible and general flows are solved separately in that order. Groups and sites are sorted by stable synthetic IDs for deterministic tie-breaking. Missing, duplicate or malformed group/site/route identities reject the input; unknown/unreachable or inaccessible route edges simply do not qualify for the corresponding flow.

The tool reports:
- total and accessible demand;
- total, accessible and general places served;
- unmet total and accessible demand;
- per-group served/unmet counts;
- per-site allocated total/accessibility counts;
- effective total capacity after water, remaining accessible/general capacities and water use implied by the common quota;
- worst group fulfillment ratio;
- case/input/evidence references and algorithm policy version.

A maximum-flow result proves feasibility against the supplied capacity/route graph only. It is not proof that a route, refuge, water source or capacity measurement is physically current.

## 5. Negative and positive controls

The synthetic known-answer suite covers:
- full feasible coverage with two groups and two eligible sites;
- demand surge beyond shared capacity, proving seats cannot be double-counted;
- accessible-route failure with unmet accessible demand despite general seats remaining;
- safe-water limitation reducing effective total capacity;
- duplicate route-edge identities rejected;
- ineligible/unverified site excluded;
- valid positive controls preserved;
- deterministic result under input-array permutation.

Targets:
- zero assignments above effective total capacity;
- zero assignments above effective accessible capacity;
- zero water use above admitted safe water after commitments/reserves;
- zero accessible-demand service via non-accessible route/capacity;
- zero accepted duplicate route/site/group IDs;
- 100% agreement with the finite independent flow oracle;
- report group-level unmet demand and worst group fulfillment, not only network total.

## 6. Industry and SOTA comparison

A classical maximum-flow solver is an established algorithmic baseline, not a novelty claim. The contribution is its application as a privacy-preserving, rights-aware, service-evidence-bound oracle with accessible-demand priority and water-constrained finite capacity.

Compare the policy against:
- unconstrained nearest-site assignment (negative/naive baseline; may overbook);
- total-seat-only max-flow (omits accessibility and water);
- accessible-first max-flow (v0.1);
- later, a policy-approved min-cost/fairness-aware multi-resource integer programme.

No shortest-path or geographic optimization is performed here. Sol-Atlas provides route evidence. The network-flow oracle must not fabricate travel times or infer route eligibility from Euclidean distance.

## 7. Claim ceiling

**Status:** SYNTHETIC_NETWORK_FLOW_ORACLE_NOT_FIELD_VALIDATED. The input and cases are synthetic. No site, demand group, route, accessible facility, actual safe-water volume, booking or resident assignment is admitted. The output is capacity-feasibility analysis only.
