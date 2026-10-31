# IEAC value-of-information control v0.1

**Status:** Frozen synthetic, hand-calculated counterexample fixture. No acquisition policy implementation or field experiment is claimed.  
**Tracks:** [IEAC-VOI-001 / #365](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/365).

## 1. Why this fixture exists

An assurance system should not request the measurement that merely reduces uncertainty the most. It should request the measurement that can most improve the decision after accounting for observation cost, delay and permissions.

The companion fixture `schemas/examples/ieac-voi-entropy-vs-decision-value-synthetic-v0.1.json` demonstrates a minimal case where the two objectives disagree.

## 2. Frozen decision problem

There are two independent hidden bits:

- A safety-relevant hazard state, with prior failure probability 0.1.
- A nuisance operating mode, equally likely to be either mode.

There are two diagnostic actions:

| Action | Healthy state | Failure state |
|---|---:|---:|
| Continue normal | 0 | 100 |
| Protective shutdown | 12 | 12 |

Before making an observation, expected loss is:

- Continue normal: (0.1 	imes 100 = 10).
- Protective shutdown: (12).

Therefore the minimum expected loss is 10, selecting continue normal in this synthetic decision problem. This is not a real-world safety policy and is not an IEAC authorization.

Two observations are available, each at cost 1 and zero delay/operator cost.

**Mode probe:** perfectly observes the nuisance mode, which is independent of the hazard. It yields exactly 1 bit of information, but the hazard posterior remains 0.1 and the decision loss remains 10. Gross decision value is zero; net value after cost is -1.

**Hazard probe:** observes the hazard with sensitivity 0.9 and specificity 0.9. Its positive outcome probability is 0.18 and posterior failure probability after a positive is 0.5. Its negative outcome probability is 0.82 and posterior failure probability after a negative is (1/82 approx 0.0121951).

After a positive observation, the minimum expected loss is 12 (protective shutdown). After a negative observation, the minimum expected loss is (100/82 approx 1.2195122) (continue normal). The expected post-observation loss is therefore:

[
0.18 	imes 12 + 0.82 	imes rac{100}{82} = 3.16
]

The hazard probe reduces expected loss from 10 to 3.16: gross value 6.84, net value after cost 5.84. Its mutual information is only about 0.211081 bits.

## 3. The counterexample

**Decision-value winner: hazard probe**

| Policy | Information gain | Expected loss after observation | Observation cost | Total expected loss |
|---|---:|---:|---:|---:|
| No query | 0 bits | 10.00 | 0 | 10.00 |
| Entropy-maximizing mode probe | 1.000 bits | 10.00 | 1 | 11.00 |
| Decision-value hazard probe | 0.211 bits | 3.16 | 1 | 4.16 |
| Fixed checklist (both probes) | 1.211 bits | 3.16 | 2 | 5.16 |

The entropy-maximizing policy chooses the irrelevant mode probe and performs worse than asking nothing once its cost is counted. The expected-value-of-information policy chooses the noisier hazard probe and obtains the largest reduction in expected decision loss.

This does not show that entropy/information gain is useless. It shows that information gain is an intermediate diagnostic quantity, not a substitute for decision value.

## 4. Benchmark objective

For an allowed observation request (q), define:

[
operatorname{NetVoI}(q) =
min_d mathbb{E}[L(d,S)]
-
mathbb{E}_{o sim q}
left[
min_d mathbb{E}[L(d,S)mid o]
ight]
-
C(q)
]

where:

- (S) is the hidden state in the benchmark simulator;
- (d) is a bounded decision option;
- (L) is the preregistered decision-loss function;
- (o) is the prospective observation;
- (C(q)) includes observation, delay, operator and resource costs.

The expected observation distribution must include source reliability, missingness, delay and correlated errors. The policy must not see hidden simulator state. A bounded oracle may see it but must be reported only as an upper bound.

The next executable benchmark should compare no-query, fixed-checklist, maximum-information-gain, expected-NetVoI, specialist diagnostic and bounded-oracle methods with matched information and compute budgets. Preserve all per-case prospective requests, realized measurements, admission results, costs and outcomes in the replay ledger.

## 5. Safety and assurance boundary

- A measurement request is not an admission and cannot authorize an action.
- Any action after an observation requires a fresh IEAC admission against the new snapshot and current authority/revocation state.
- An observation with no permitted source, incompatible units, invalid calibration, missing provenance or expired authority cannot become admissible merely because it has high expected information value.
- Hard safety, service and product-quality constraints remain gates; average expected utility cannot compensate for their violation.
- The hidden-state simulator is a benchmark oracle only; it is never available to the acquisition policy or an operational twin.

## 6. Acceptance and next steps

- Exact hand-calculated values in this fixture must be reproduced by at least two independent implementations.
- Entropy-only selection must choose `mode_probe`; expected-NetVoI must choose `hazard_probe`.
- The acquisition planner must not change decision authority or bypass fresh admission.
- Future benchmarks must use pre-registered cost/latency models, held-out task instances and paired comparisons; report uncertainty in utility differences.
- Extend to multi-sensor correlation, adversarial observations, calibration drift, stale measurements, source outages and expensive specialist-solver requests.

**Claim ceiling:** this fixture is a decision-theory counterexample, not a calibrated industrial model, physical risk estimate, safety case or production evidence.
