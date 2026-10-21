# SA-WORKFORCE-ACTIVE-EVIDENCE-001 — Rights-aware value-of-information benchmark v0.1

**Tracking issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/278  
**Composes with:** diagnosis benchmark #268, source-aware evidence profile #272, and diagnosis-to-outcome evaluation #277.  
**Status:** benchmark contract + synthetic scenarios; no real labour data or model-effectiveness result.

## Thesis: the next best question is part of the decision

A one-shot diagnosis is forced to make its best guess from whatever evidence happens to be present. In practice, uncertainty itself has operational structure: some missing fact is cheap and safe to verify; some facts are expensive or unavailable; some hypotheses imply the same safe action and do not need to be distinguished yet.

We instead treat evidence collection as a sequential decision problem:

**plausible hypotheses → choose a permitted observation → update the remaining hypotheses → stop when the safe decision is determined → otherwise ask the next question or abstain.**

This is the workforce-specific application of active feature acquisition (AFA). The IJCAI 2026 survey describes AFA as sequential selection of features to balance prediction quality and acquisition costs, frames existing approaches using a POMDP view, and highlights the need for stronger guarantees beyond heuristics. The goal here is deliberately narrower and auditable: begin with deterministic, finite, small cases whose exact policy frontier can be enumerated.

## Non-negotiable distinction: root-cause certainty vs decision certainty

A policy may stop even when it has not identified the unique underlying cause. If every remaining plausible hypothesis implies the same safe, authorized operational decision, further investigation is not necessary for that decision. Conversely, a high-confidence-looking prediction is not sufficient when unresolved hypotheses imply materially different actions. The fixture’s expected_remaining_hypotheses field records the residual set on the relevant unresolved or unknown-outcome branch; a policy that resolves all fully observed branches may still have to abstain if an unmodelled outcome occurs.

The stopping rule is therefore based on **decision-class agreement**, not only the number of remaining root-cause labels.

- If all remaining hypotheses imply the same permitted disposition, stop and expose the unresolved cause set.
- If materially different decision classes remain, continue only with a permitted query that may distinguish them.
- If no permitted query can resolve the decision within declared cost/budget limits, abstain and report why.
- An unknown/unmodelled query outcome does not eliminate hypotheses. It triggers revalidation or a safe hold.

Do not assign probabilities to authored hypotheses. Do not use uncalibrated expected value or arbitrary weighted sums. Compare policies using a **Pareto frontier** over staff effort, cash cost and elapsed delay. Rights, privacy, consent, authorization, safety and essential-service constraints are hard eligibility gates, not dimensions that can be traded away for speed.

## Roles and boundaries

- Sol-Atlas: scenario/evidence provenance and query-cost / geography context; this benchmark only owns acquisition decisions.
- Symthaea: optional diagnostic assistance, with explicit uncertainty and no authority over qualification, approval or execution.
- Mycelix: compose the existing scoped skill-evidence/provenance path; do not create another credential registry or worker score.
- Symtropy: later simulate queue/delay and capacity where the declared model has been validated; simulation is not observed reality.
- Human data owner / worker / community authority: approve access, consent and scope under legitimate governance.

Do not collect private data merely because it is informative. No personal-account scraping, social-media inference, invasive worker monitoring, sensitive-trait inference, automated adverse hiring decisions or general employability scoring. An assessment of practical competence must be scope-specific, appropriately authorized and not treated as a universal rating of a person.

## Model contract

Each synthetic case contains:
1. A finite set of hypotheses. Each has a stable ID and a permitted **decision class**.
2. Candidate evidence queries. Each carries an explicit permission flag, an access basis, an exclusion reason where forbidden, and separate direct costs (staff minutes, cash in ZAR cents, and elapsed delay hours).
3. A set of mutually exclusive known outcomes for each query. Every known outcome has the hypothesis subset compatible with that result; the known branches must exactly partition the current hypothesis set under the declared model.
4. Optional unknown/unmodelled outcome. It may be compatible with every hypothesis and never narrows the set.
5. Resource budgets and a hand-calculated expected disposition or exact reference frontier.

Costs along a branch add. A policy's conservative vector is the component-wise worst case over its possible paths. Pareto dominance requires being no worse in all dimensions and strictly better in at least one. Two policies with incomparable cost vectors remain on the frontier. Do not average cost/benefit across unweighted synthetic scenario counts.

Queries must not be repeated in a single path. The tiny-instance exact Rust solver enumerates permitted adaptive policies up to a declared cap; it emits an explicit limit status rather than returning a silently truncated frontier. Its resolution tiers distinguish (a) guaranteed resolution on every possible branch, (b) conditional resolution on every modeled observed branch with abstention permitted for unknown/unmodelled outcomes, (c) partial resolution where some observed branches remain unresolved, and (d) no observed-branch resolution. Pareto cost vectors are kept separate within each tier so a cheap partial policy cannot masquerade as an equally useful conditional policy.

## Baselines and comparisons

- **B0 — fixed-order questioning:** a disclosed static query order.
- **B1 — cheapest permitted query first:** simple direct-cost heuristic.
- **B2 — immediate split per staff minute:** transparent greedy heuristic; only a comparator.
- **B3 — collect every permitted observation:** useful negative-control baseline for waste; it does not justify over-collection.
- **B4 — exact adaptive oracle:** exhaustive policy enumeration for tiny fixtures.
- **Candidate — robust Pareto policy:** chooses among exact-oracle optimal/non-dominated policies without inventing probabilities or silently collapsing costs.

Compare on identical case definitions, query permissions, outcome partitions and budgets. Report both decision-resolved cases and cases that correctly stop on a shared decision class. An abstention is not automatically a failure: report whether resolution was possible with permitted queries and within budget.

## Hand-calculated reference: scenario A

Possible hypotheses:
- H-FUND: payer/funding absent → HOLD / establish lawful funding before representing work as funded.
- H-TOOLS: critical spare missing → HOLD / procure or qualify a compatible part.
- H-SKILL: task-ready capacity insufficient → HOLD / scoped capacity assessment and a permitted workforce pathway.

Three permitted evidence queries have deterministic synthetic partitions:

| Query | Known outcomes | Staff minutes | Cash cost | Delay |
|---|---|---:|---:|---:|
| Confirm approved payer/budget | H-FUND vs {H-TOOLS, H-SKILL} | 20 | R0 | 1 h |
| Check exact spare inventory | H-TOOLS vs {H-FUND, H-SKILL} | 10 | R50 | 0 h |
| Scoped task-ready capacity check | H-SKILL vs {H-FUND, H-TOOLS} | 50 | R0 | 8 h |

Hand-calculated policy examples:
- **A1 — less staff-time / less delay:** check inventory first; if the spare is missing, stop. Otherwise confirm payer/budget. Cost vector: **(30 staff-min, R50, 1 h)** worst case.
- **A2 — no cash expenditure in the modeled checks:** confirm payer/budget first; if confirmed and tools are available under this hypothesis set, check scoped task-ready capacity; if no payer exists, stop. Cost vector: **(70 staff-min, R0, 9 h)** worst case.

Neither vector dominates the other: A1 uses less staff time and delay but spends R50, while A2 spends no cash but requires more staff time and delay. The example makes no statement about actual South African costs. It shows why an undisclosed scalar “best query” score would hide a real trade-off.

The reference frontier must be generated and checked by an independent exact oracle before making claims that these are the complete non-dominated set for the encoded problem. These two are hand-calculated reference policies, not evidence that a production solver has been implemented.

## Adversarial and negative controls

The fixture includes:
- hypotheses that require different dispositions but have no permitted discriminating query → explicit abstention;
- a forbidden, apparently informative query → never selected regardless of its apparent information gain or low cost;
- stale/contradictory evidence and an unknown branch → hold or revalidate; no silent narrowing;
- hypotheses that share one safe decision → stop early without unnecessary data collection;
- budget exhaustion → preserve unresolved hypotheses and state why the decision remains indeterminate;
- permuted case, hypothesis, query and outcome order → identical canonical receipt and frontier.

Further controls should add access changes, new competing hypotheses, outcome-partition defects, duplicate query IDs, negative or incomparable cost units, source-version drift, and a query that distinguishes labels without changing the operational decision.

## Metrics and acceptance gates

- Exact agreement with independently hand-enumerated feasible policies and Pareto frontier on the frozen tiny instances.
- 0 forbidden query selections; 0 consent/scope/authorization bypasses; 0 sensitive-proxy inference.
- 100% deterministic canonical output under row-order permutations.
- Correct early stopping whenever all live hypotheses share one decision class.
- Correct abstention when material decisions cannot be distinguished by permitted, affordable evidence.
- Unknown/unmodelled outcomes never narrow the hypothesis set by themselves.
- Show Pareto frontier in each of staff minutes, cash cost and delay hours; no implicit weighted sum.
- Report decision resolution, residual decision classes, query count, worst-case query cost, unmodelled-outcome rate, abstention reason and exact-oracle cap.
- Emit corpus/schema/code digests and a claim ceiling in every receipt.
- Benchmark runtime, peak memory and policies explored as fixture size grows; do not set a performance superiority target until a reproducible baseline exists.

## Research anchors

- Aronsson, Rahbar & Chehreghani, *A Survey on Active Feature Acquisition Strategies*, IJCAI 2026: https://www.ijcai.org/proceedings/2026/860
- Provost et al., *Active Feature-Value Acquisition*, Management Science: https://pubsonline.informs.org/doi/10.1287/mnsc.1080.0952
- NIST, *TEVV-Athlon Framework for Evaluating AI Systems* (2026 initial draft): https://www.nist.gov/artificial-intelligence/ai-research/tevv-athlon-framework-evaluating-ai-systems
- World Bank, *From Demand to Delivery: Strengthening South Africa's Digital Skills Pipeline* (14 Aug 2026): https://www.worldbank.org/en/news/feature/2026/08/14/from-demand-to-delivery-strengthening-south-africas-digital-skills

## Claim ceiling

This is a measurement contract with synthetic cases. It is not a shortage estimate, a field-tested acquisition policy, an employment recommender, a real cost estimate or evidence of causal impact/SOTA. Empirical evaluation needs an authorized partner, documented source access, a preregistered comparison design and independent outcome verification.
