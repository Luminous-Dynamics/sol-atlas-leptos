# Site Investigation as a Decision: Expected Value of Information

Status: implementation and research note v0.1. Synthetic economics are illustrative only; the code does not perform geotechnical analysis or certify an investigation plan.

## Why this is a useful paradigm shift

Conventional project planning can treat surveys as fixed checklist items or budget line items. Risk-informed decision analysis asks a more precise question:

**Which piece of missing information could materially change the design or site decision, and is the expected benefit worth its cost?**

For a site, this might compare geotechnical boreholes, utility-capacity verification, title/servitude checks, flood modelling, contamination sampling, biodiversity surveys, traffic studies, or additional structural analysis. The best investigation depends on the decision being made: an investigation is useful when its potential outcomes could change the best action.

Recent geotechnical research combines spatial random fields, Bayesian networks and expected-value-of-sample-information (EVSI) analysis to compare site-investigation strategies. One 2026 study models slope stability and shallow foundation settlement and finds that useful sampling locations can depend on the failure or serviceability mechanism; that supports mechanism-aware investigation planning, not a universal borehole-placement rule. See Fartosy, Jawad and Hasan (2026), *Optimizing Geotechnical Site Characterization*, [DOI 10.3311/PPci.43308](https://doi.org/10.3311/PPci.43308). Earlier work has similarly evaluated site-investigation programs before carrying them out, including [Hu et al. (2021)](https://doi.org/10.1016/j.compgeo.2020.103938).

## Mathematical contract

Given mutually exclusive states of the world s, prior probabilities p(s), available decisions d and losses L(d,s):

prior_loss = min_d Σ_s p(s) L(d,s)

For a test/investigation with outcome o and likelihood P(o | s), update with Bayes' rule:

P(s | o) = P(o | s) P(s) / Σ_s' P(o | s') P(s')

Expected post-investigation loss is:

posterior_loss = Σ_o P(o) min_d Σ_s P(s | o) L(d,s)

Then:

- Expected Value of Sample Information (EVSI) = prior loss − posterior loss.
- Expected Value of Perfect Information (EVPI) = prior loss − expected loss if the true state were known before choosing.
- Expected net value of the investigation = EVSI − investigation cost.

Under a coherent model, EVSI should be non-negative and no greater than EVPI. The code checks probability normalization, complete loss tables, test likelihoods, costs, and this upper bound within numeric tolerance.

## Implementation boundary

The Rust module sol-atlas-core::site_investigation currently implements a discrete-state pre-posterior calculation. It requires the project team to provide:
- credible prior state probabilities;
- a decision set and loss for every decision/state pair;
- investigation costs;
- likelihoods for every investigation outcome under every state.

It does not estimate these probabilities or losses from actual borehole logs, sensor data, geotechnical models, or historical projects. It does not model continuous random fields, spatial correlation, failure probabilities, multiple sequential investigations, or interactions between sampling locations. Those are future specialist extensions.

All losses must currently use one common unit. Monetary cost is a convenient pilot unit, but life safety, statutory obligations, ecological rights, displacement and non-compensable constraints must not be turned into tradeable values simply to fit an economic equation. Mandatory safety/legal investigations remain required even when an economic model assigns low or negative net value. Use explicit gates and professional judgment alongside the model.

## Synthetic reference benchmark

The fixture at docs/research/fixtures/site-investigation-voi-benchmark-v0.1.jsonl is fully synthetic. It contains:
1. Perfect information: EVSI equals EVPI; an illustrative $25,000 investigation has $490,000 EVSI and $465,000 net value.
2. An uninformative test: EVSI is zero and the test's cost makes net value negative.
3. Partial geotechnical information: EVSI is between zero and EVPI, supporting comparison against the test cost.

The values do not estimate any real site, construction cost, failure probability, borehole price, or foundation design.

## Proposed promotion gates

Before this tool is used to influence real project spending, require:

1. At least 30 independently hand-calculated reference cases with two through many states, three or more decisions, rare outcomes, zero-probability outcomes, imperfect tests, extreme costs, ties, and contradictory inputs.
2. An independent implementation or calculation spreadsheet on the same pinned fixtures.
3. Property tests that EVSI is non-negative, EVSI ≤ EVPI within tolerance, posterior probabilities normalize, and uninformative evidence has zero EVSI.
4. Calibrated test likelihoods and priors, with provenance and elicitation methods recorded; no invented precision.
5. Sensitivity analysis over priors, losses, costs and likelihoods; report when the recommendation reverses.
6. For spatially distributed investigations, model where information is sampled, spatial correlation, decision mechanism and the exact engineering model. Compare to an independent Monte Carlo or validated specialist reference where feasible.
7. Keep mandatory safety investigations outside cost-benefit discretion; no synthetic or economic benchmark constitutes professional engineering approval.

## Integration

- Sol-Atlas supplies site candidates, evidence gaps, and candidate investigations.
- Symthaea can explain the decision model, generate alternative investigation plans, and recommend which unknowns matter most, without inventing priors or final authority.
- Symtropy may help with appropriate deterministic geometry/physics exploration, but specialist engineering solvers provide structural, geotechnical, wind, seismic, fire and similar results.
- Mycelix can preserve the data versions, model assumptions, investigation purchase/authorization, sampled location, results, review and decision revision as an auditable record.

A strong future workflow compares whole investigation programs—not just one test—by estimated decision improvement, cost, time, safety obligations and how the result can change the design. It then updates the design only when validated new evidence is returned.

## Sources

- Fartosy, S. H., Jawad, F. W., Hasan, H. A. (2026). “Optimizing Geotechnical Site Characterization: A Value of Information Approach Using Coupled Spatial Random Fields and Bayesian Networks.” *Periodica Polytechnica Civil Engineering*, 70(2), 598–611. https://doi.org/10.3311/PPci.43308
- Hu, J. Z. et al. (2021). “Value of information analysis of site investigation program for slope design.” *Computers and Geotechnics*. https://doi.org/10.1016/j.compgeo.2020.103938
- Hu, J. Z. et al. (2022). “Assessing expected benefit of site investigation program for reliability-based design of slope.” *Engineering Geology*. https://doi.org/10.1016/j.enggeo.2022.106749
- Spatial preference uncertainty and Sobol sensitivity analysis (published 2026). *Environmental Modelling & Software*, 205, 107137. https://doi.org/10.1016/j.envsoft.2026.107137

The research supports the general decision-analysis approach. It does not validate this code, its synthetic inputs, or any real site's engineering model.
