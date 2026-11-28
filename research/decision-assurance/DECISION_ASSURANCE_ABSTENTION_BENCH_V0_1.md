# Decision Assurance and Abstention Benchmark v0.1

**Status:** research specification; synthetic reference not yet implemented  
**Related issue:** [#997](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/997)  
**Scope:** disposition semantics, robustness envelopes, abstention, and benchmark scoring

## 1. Objective

Qualify whether a spatially grounded decision is justified by the evidence available for its exact support, time, authority, and intended use.

This benchmark is not another prediction or inference engine. It consumes already-qualified outputs from spatial calibration, applicability, spatial functionals, hypothesis disagreement, evidence governance, and robust EVSI.

The object being qualified is a **decision envelope**: the set of actions and their consequences across admissible physical hypotheses, evidence states, registered stakeholder utilities, and declared assumption perturbations.

## 2. Formal contract

Let:

- \(S\) be the exact spatial/temporal support of the decision;
- \(\mathcal H\) be the non-empty set of admissible hypotheses or joint worlds;
- \(\mathcal U\) be the registered set of stakeholder utility functions;
- \(\mathcal A\) be the finite or explicitly bounded action set;
- \(C_k\) be hard constraints (rights, safety, authority, evidence integrity, applicability, support completeness);
- \(L(a,h;u)\) be the loss of action \(a\) under world \(h\) and utility profile \(u\).

For each \((h,u)\), define the optimal action set:

\[
A^*(h,u)=\arg\min_{a\in\mathcal A} L(a,h;u).
\]

For a candidate action \(a\), define robust regret:

\[
R_{\max}(a)=\sup_{h\in\mathcal H,\,u\in\mathcal U}
\left[L(a,h;u)-\inf_{b\in\mathcal A}L(b,h;u)\right].
\]

The robust action set is:

\[
A_{\mathrm{rob}}(\epsilon)=
\{a\in\mathcal A:R_{\max}(a)\leq\epsilon\}.
\]

A decision-boundary margin is the smallest declared perturbation of assumptions or utility parameters that changes the selected action. It is a sensitivity measure, not a probability.

The reference must report these quantities separately. It must not collapse them into an opaque confidence score.

## 3. Disposition state machine

Apply hard gates before scoring:

1. **REJECT** if a non-waivable hard constraint is violated, including unauthorized evidence, invalid authority, prohibited disclosure, or a declared safety-limit breach.
2. **ABSTAIN** if required evidence, support, applicability, provenance, utility, or admissible-world information is missing or unresolved.
3. **RECOMMEND** only if all hard gates pass and the action satisfies the preregistered robust-risk envelope.
4. **CONDITIONAL** if the action is defensible only under explicit, monitorable assumptions or restrictions.
5. **INVESTIGATE** if a permitted observation has positive decision value and could change the action or disposition.

Precedence is intentional: no expected-value, calibration, or model-confidence score can override REJECT or ABSTAIN. INVESTIGATE is not a recommendation to perform a physical intervention.

Every disposition receipt must include:
- exact support ID/revision and time window;
- source/evidence and provider-receipt digests;
- rights, authority, and disclosure decision references;
- applicable validation regime and claim ceiling;
- admissible hypotheses/worlds and their weight semantics (or explicitly unweighted set);
- utility profile IDs and action-loss table digest;
- hard-gate results and unresolved conditions;
- selected disposition, rationale, robust regret and sensitivity envelope;
- next evidence actions and their cost/value assumptions;
- invalidation conditions, expiry, and deterministic evaluator version.

## 4. Decision-directed acquisition

For candidate observation \(q\), define disposition-aware value:

\[
\mathrm{DVOI}(q)=
\mathbb E[\,V(D\mid Y_q)-V(D)\,]-c(q),
\]

where \(D\) is the decision/disposition state, \(Y_q\) is the possible observation, and \(c(q)\) includes acquisition, delay, access, processing, and operational costs.

The expectation is valid only under a declared observation model. Where probabilities are not justified, report scenario-wise bounds or robust value instead of inventing a distribution.

A useful acquisition is one that changes a decision-relevant quantity: reduces worst-case regret, moves ABSTAIN to a justified CONDITIONAL/RECOMMEND, resolves an action disagreement, or rules out an unsafe action. Reducing map variance alone is not sufficient.

Correlated repeat observations must not be counted as independent information. Reuse #985/#987's dependence and robust-EVSI owners.

## 5. Frozen synthetic challenge suite

| ID | Challenge | Required disposition / invariant |
|---|---|---|
| DAAB-01 | Wide prediction interval; same optimal action in every world | No unnecessary investigation solely to narrow the interval |
| DAAB-02 | Narrow interval straddles a high-consequence threshold | INVESTIGATE or ABSTAIN; never infer safety from interval width |
| DAAB-03 | Nominal winner has catastrophic worst-case regret | No RECOMMEND under the declared robust envelope |
| DAAB-04 | Small stakeholder-utility perturbation flips the action | Expose preference sensitivity; no universal recommendation |
| DAAB-05 | Unknown support intersects the decision-critical corridor | ABSTAIN or bounded CONDITIONAL; never treat unknown as safe |
| DAAB-06 | Rights-denied evidence changes the action | REJECT that evidence path and recompute without unauthorized evidence |
| DAAB-07 | Correlated repeat measurements appear numerous | No false information gain from duplication |
| DAAB-08 | Cheap observation changes INVESTIGATE to RECOMMEND; expensive one has negative net value | Correct DVOI ordering under the declared observation model |
| DAAB-09 | No permitted observation can resolve disagreement | Report irreducible ambiguity; ABSTAIN or CONDITIONAL |
| DAAB-10 | Permutations and duplicate references | Deterministic order invariance and duplicate non-inflation |

## 6. Benchmark scorecard

Report, do not hide in one aggregate:
- false-RECOMMEND rate on hard-gate and catastrophic-regret cases;
- false-ABSTAIN and unnecessary-INVESTIGATE rates;
- expected regret and worst-case regret;
- robust-action-set size and action stability;
- decision-boundary margin under registered perturbations;
- disposition-aware value per unit acquisition cost;
- critical-support unknown fraction and overlap with action-sensitive regions;
- stakeholder utility dispersion and preference-robustness frontier;
- rights/provenance/applicability violations;
- runtime, peak memory, determinism, and receipt reproducibility.

Probability calibration of dispositions is reportable only when repeated, representative outcomes define a defensible reference frequency. Synthetic case pass rates are not empirical risk estimates.

## 7. Baselines and matched comparisons

Compare against:
1. nominal expected-loss / Bayes action where probabilities are justified;
2. minimax action;
3. minimax-regret action;
4. CVaR policy with preregistered tail mass;
5. threshold-only and pointwise-variance acquisition;
6. spatial-diversity and entropy acquisition;
7. nominal EVSI and robust EVSI;
8. decision-focused conformal baselines;
9. documented incumbent specialist workflow and expert plan.

All methods receive identical source bytes, support, admissible worlds, utility, rights, action set, acquisition budget, hardware envelope, and sealed evaluation cases. Record product version, license/module, settings, operator effort, unsupported functions, and failed runs. NOT_RUN is not evidence of inferiority.

## 8. Qualification ladder

- **D0 — mathematical:** hand-computable finite loss tables and state-machine cases.
- **D1 — executable synthetic:** independent Rust oracle, schema, frozen fixtures, negative controls, deterministic CLI receipt.
- **D2 — composed:** consume real qualified receipts from #991/#993/#987 without changing their ownership boundaries.
- **D3 — retrospective:** whole-site/campaign holdout, independent reference outcomes, stakeholder-approved utility, permissioned data.
- **D4 — prospective:** preregistered decision tournament with blinded scoring, power analysis, real acquisition costs, and post-decision outcome follow-up.

No lower tier can be promoted by presentation or documentation language.

## 9. Research baselines and limitations

- Utility-Directed Conformal Prediction (ICLR 2025) demonstrates conformal prediction sets shaped by downstream decision loss. It is a required comparator, not a novelty claim: https://proceedings.iclr.cc/paper_files/paper/2025/hash/0c6b452f1bbfb6905f6bac957d73b321-Abstract-Conference.html
- LSCP (ICML 2026) combines local quantile regression and conformal calibration. Its coverage claims depend on the stated spatial exchangeability, stationarity, and spatial-mixing conditions: https://proceedings.mlr.press/v306/jiang26at.html
- GeoSIMCP combines geographic and feature-space similarity. Its localized joint weighted-exchangeability assumptions, tuning procedure, and scale sensitivity require explicit stress tests: https://doi.org/10.1080/24694452.2026.2639721
- Neighbourhood- and threshold-aligned groundwater monitoring acquisition is a strong comparator to pointwise uncertainty heuristics; reported case-study gains are not assumed to transfer: https://doi.org/10.1016/j.envsoft.2026.107091
- Tiered spatial validation shows why random-fold performance cannot stand in for external geographic transfer: https://doi.org/10.1016/j.mex.2026.104072

## 10. Ownership boundaries

- #983 / Mycelix: evidence exchange, rights, provenance, disclosure.
- #986: pointwise calibration and metrology.
- #988: likelihood applicability and support/rights/coordinate/temporal binding.
- #991: spatial functional identity, support, units, missingness, joint-world functionals.
- #993: hypothesis/action disagreement and regret.
- #985 / #987: correlated observations and robust EVSI.
- Symthaea: alternative-hypothesis reasoning and investigation planning.
- Symtropy: qualified physical and engineering models.

This programme owns only disposition semantics, decision envelopes, abstention, and benchmark scoring. It does not create a second posterior, covariance, geometry, topology, or evidence-graph engine.

## 11. Claim ceiling

The initial implementation will be a synthetic finite reference. Passing it demonstrates deterministic contract behavior only. It does not establish real-world risk probabilities, physical safety, legal authority, causal completeness, stakeholder legitimacy, or superiority over commercial systems.
