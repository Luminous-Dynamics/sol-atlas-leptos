# Water intervention decision contract v0.1

**Status:** research and schema contract; not a deployed decision engine or a validated engineering method.  
**Research date:** 2026-10-15.

## The paradigm shift

The default workflow in many engineering projects is model -> dashboard -> recommendation. Sol-Atlas should instead treat the **intervention decision packet** as the unit of engineering evidence:

`authorised observations -> model/run evidence -> feasible interventions -> uncertainty analysis -> constraint-first comparison -> human decision -> independently verified outcome`

A hydraulic model remains a specialised calculation, not a digital twin by declaration. A good dashboard can make a bad assumption easier to see, but cannot make it true. The new contract therefore binds alternatives to named run bundles and requires each outcome to retain its unit, scope, time window, evidence class, comparability state and uncertainty state.

This addresses an active gap: the American Water Works Association's 2026 survey reports interest in AI, predictive analytics and digital twins, while noting that many utilities lack well-defined implementation plans. A defensible engineering plan needs explicit qualification and decision gates, not a promise of “real-time intelligence.” [AWWA, *Model or Digital Twin?* (15 April 2026)](https://awwa.onlinelibrary.wiley.com/doi/full/10.1002/awwa.70075).

## Five design shifts

1. **Optimise service, not a proxy.** Pipe velocity or simulated headloss is a diagnostic. Decision metrics should include service reliability, pressure compliance, unmet demand where represented, water quality where represented, energy, whole-life cost, environmental effects and affected populations. Do not count renewal kilometres as proof of better service.
2. **Unknown is a first-class result.** Missing pressure data, incomparable annual indicators, an uncalibrated roughness estimate, and a failed solver are not zero values. The packet preserves unavailable and non-comparable metrics and prevents them from becoming ranked gains.
3. **Hard constraints before Pareto trade-offs.** Applicable health, safety, regulatory and minimum-service constraints must be pass/fail/unknown before a candidate enters any trade-off frontier. A lower cost or energy metric must never compensate for an unresolved mandatory safety/service constraint. Keep the outcome vector; do not hide policy judgments in an unreviewed weighted score.
4. **Decide what to measure next.** Record material unknowns and the next evidence actions that could reduce them. Once deterministic runs are reproduced, declared sensitivity/sweep cases should test whether a candidate remains preferable when demand, roughness, boundary conditions and other uncertain inputs vary. Quantitative value-of-information estimates can come later; this v0.1 contract records the actionable evidence backlog without pretending it has computed one.
5. **Close the loop with independently observed outcomes.** Preserve the pre-intervention baseline. Recommendation, human authorization, completed work, and verified service outcome remain separate states. A repair record is not itself proof of water saved or improved service.

## Why the physical model boundary matters

The US EPA describes EPANET as a model for extended-period hydraulic and water-quality analysis in pressurised distribution networks. Its user manual distinguishes demand-driven analysis from pressure-driven analysis; under low-pressure conditions, that choice can materially affect how demand shortfall is represented. Sol-Atlas must record the analysis mode rather than silently treating the two interpretations as interchangeable. Sources: [US EPA EPANET project](https://www.epa.gov/water-research/epanet), [EPANET 2.2 User Manual](https://usepa.github.io/EPANET2.2/).

Keep SWMM as a separate adapter for appropriate rainfall-runoff, stormwater and sewer cases. Do not force drinking-water distribution, building plumbing, and stormwater hydraulics into one generic graph solver simply because they share a map. Shared evidence contracts are useful; interchangeable physics models are not.

Local service outcomes matter. South Africa's 2023 No Drop reporting described national non-revenue water increasing from 37% in 2014 to 47% in 2023, while the March 2026 Green/Blue/No Drop programme update reiterates the need to assess drinking-water quality, wastewater management and water-use efficiency together. These are reasons to retain distinct outcome dimensions—not proof that this prototype can explain the causes or select the right pipe. Sources: [South African Government, 2023 No Drop findings](https://www.gov.za/news/media-statements/water-and-sanitation-releases-2023-full-blue-drop-report-05-dec-2023), [South African Government, 27 March 2026](https://www.gov.za/news/media-advisories/government-activities/minister-pemmy-majodina-releases-green-drop-report-and).

## What this commit adds

- `water-intervention-decision-v0.1.schema.json`: a Draft 2020-12 contract for the baseline, candidate alternatives, typed outcomes, hard constraints, uncertainty review, equity review, comparison policy, decision state and nonclaims.
- `w0-intervention-decision.synthetic.json`: a deliberately **not-run** fixture. It uses unavailable metrics with null values and explicit reasons; no invented hydraulic results or ranking.
- A CI schema/fixture pair so the contract shape is checked alongside the existing water-engineering and public-observation bundles.

The schema is a shape gate, not a semantic verifier. Cross-record ID uniqueness, constraint/result consistency, unit compatibility, Pareto calculation, uncertainty statistics and decision-state transitions remain follow-up implementation work. Do not display a ranked recommendation as qualified until those checks, actual pinned-solver evidence, and independent engineering review exist.

## Qualification ladder

- **Contract-only:** schema/fixture validation passes; no model result is implied.
- **Reproduced synthetic:** exact inputs, solver build identity, reports, continuity checks and output hashes agree on an exact run. This proves the controlled test was reproduced, not that a municipal model is accurate.
- **Uncertainty-screened synthetic:** declared parameter/range sweeps report their coverage and failures; do not cherry-pick only favourable samples.
- **Empirically validated model:** calibration and withheld observations, error distributions and known limitations are reviewed by a qualified engineer.
- **Shadow decision pilot:** historical replay and prospective recommendations are checked against later evidence; decisions and physical actions remain under accountable human authority.
- **Verified intervention outcome:** post-work observations are compared with a frozen baseline, accounting for changing demand, operating conditions and measurement uncertainty.

The long-term opportunity is not to automate every engineering decision. It is to make engineering claims **reproducible, comparable where justified, uncertainty-aware, and accountable from source observation through to service outcome**.
