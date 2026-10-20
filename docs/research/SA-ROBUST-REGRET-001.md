# SA-ROBUST-REGRET-001
## Scenario-wise objective regret without replacing the outcome Pareto frontier

**Status:** Exact-head Rust workflow passed on commit `5ea2d34fb8add8de97e1a1f047de7cf8bbb4c1c2`: formatting, Clippy with warnings denied, 22 tests, and all three regret receipts succeeded. The repository `Check` workflow also completed successfully on that head.  
**Parent:** [Issue #239](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/239).  
**Input contracts:** [v0.1 frozen oracle](./SA-ECONOMIC-PORTFOLIO-001.md) and [v0.2 generalized scale suite](./SA-PORTFOLIO-SCALE-001.md).  
**Claim ceiling:** Synthetic decision-analysis methods only; no real project outcomes or investment decisions.

## 1. Decision-quality gap

The existing objective frontier answers which portfolios are non-dominated across every declared objective and stress scenario. It does not directly answer the related question: **how far could a portfolio fall short of the best portfolio for each objective if a different scenario materializes?**

A single weighted sum answers a different question and changes with chosen weights. A worst-case outcome vector can also be conservative without showing the distance from the scenario-specific attainable ideal. Robust regret adds a complementary view while keeping the original outcome vectors intact.

The literature contains multiobjective minimax-regret methods; e.g. Xidonas et al. (2017), *Robust multiobjective portfolio optimization: A minimax regret approach*, DOI 10.1016/j.ejor.2017.03.041. The cited paper is useful method context, not a claim that its financial-stock formulation is directly comparable to this discrete civic portfolio contract: https://www.sciencedirect.com/science/article/pii/S0377221717302631

## 2. Contract

For each declared scenario s and maximize-objective j, define the scenario ideal:

`ideal[s,j] = max(value[p,s,j])`

where p ranges over the feasible portfolios. Because the outcome Pareto frontier is complete under scenario-wise, componentwise dominance, it preserves every scenario/objective maximum; the maxima can therefore be computed from the complete frontier without retaining every dominated feasible portfolio.

For a portfolio p, absolute objective regret is:

`regret[p,s,j] = ideal[s,j] - value[p,s,j]`

The Rust API stores:
- `scenario_ideal_objectives`: attainable ideal vector for each declared scenario;
- `scenario_regret`: a separate nonnegative regret vector for every portfolio/scenario pair; and
- `worst_case_regret`: the componentwise maximum of each regret dimension over all declared scenarios.

The `minimax_regret_frontier` contains the non-dominated worst-case regret vectors under componentwise **minimization**. It does not overwrite, replace or narrow the original outcome frontier.

The scenario aggregate is an explicit max across the declared stress cases. This treats each supplied case as a stress case to guard against; it does not assert that cases have equal probability or any calibrated probability. The result is still a vector across four objectives in their own units. A scalar score is intentionally not supplied: cross-objective normalization, tolerances and weights would be a separate declared decision policy.

## 3. Frozen synthetic results

### V0.1: eight-project fixture

The four outcome-frontier alternatives remain all four members of the regret frontier in this fixture. Their worst-case absolute regret vectors (economic, essential service, paid-work opportunity, ecology) are:

| Portfolio | Worst-case regret vector |
|---|---:|
| A+F+J+R | (11, 3, 3, 1) |
| F+J+T+W | (10, 0, 6, 0) |
| I+J+R+T | (4, 5, 0, 8) |
| I+J+T+W | (0, 0, 3, 7) |

Scenario ideals from the four-member outcome frontier and from the full 22-feasible set must be identical. The source Rust tests pin both scenario ideal vectors, all four worst-case regret vectors, and equality between regret computed from the exact evaluator and the exact search frontier.

### V0.2: ten-project fixture

The v0.2 outcome frontier has eight portfolios. Under the componentwise worst-regret rule, seven survive the regret frontier; `P00+P03+P04+P05+P08` is dominated in worst-regret space by `P00+P01+P03+P05+P09`.

That reduction is not a universal ranking of public values. It means only that, under this synthetic fixture's two stress cases and four objective units, the dominated alternative never has lower regret in any objective. The original eight-member outcome frontier remains unchanged.

## 4. Qualification requirements

- Keep original outcome frontiers byte-structurally unchanged.
- Compare ideal objective values computed from the complete feasible set with values computed from the complete outcome frontier.
- Assert nonnegative regrets and that every scenario/objective ideal is attained by at least one feasible alternative.
- Compare regret results derived from the exhaustive oracle and dependency-aware search frontier.
- Pin v0.1/v0.2 regret-frontier memberships and vectors as known answers.
- Test row permutations, scenario order, ties, negative objective outcomes, scenario-coverage mismatch and no-feasible-portfolio inputs.
- Preserve an explicit input and reference digest whenever regret outputs are published.
- Do not select a scalar winner without a separate, predeclared decision-policy contract.

## 5. Exact-head validation record

The pinned Rust workflow completed successfully at commit `5ea2d34fb8add8de97e1a1f047de7cf8bbb4c1c2`:
- Rust unit tests: **22 passed, 0 failed**;
- `cargo fmt --check`: pass;
- Clippy with `-D warnings`: pass;
- v0.1 exact-regret receipt: 22 feasible portfolios and four regret-frontier members;
- v0.2 ten-project exact-regret receipt: 82 feasible portfolios, eight outcome-frontier members and seven regret-frontier members;
- v0.2 24-project search-regret receipt: full search accounting and frontier comparison pass, four regret-frontier members.

These are synthetic fixtures, and the test run validates implementation against those fixtures—not empirical regret quality or real-world planning superiority.

## 6. How to compare with industry / SOTA responsibly

The current tests establish exactness for two tiny synthetic corpora. They do not prove that regret-aware planning beats a public-sector incumbent, commercial portfolio platform, MILP/CP-SAT model or state-of-the-art robust optimizer.

A credible comparison should use matched input values, scenario sets, constraints, objective directions, solver configurations and compute budgets. Report outcome-frontier size, regret-frontier size, ideal-gap distributions, constraint failures, input-to-output digests, sensitivity to adding/removing stress cases, runtime, peak memory and (where available) exact optimality gaps. The World Bank's Infrastructure Prioritization Framework provides governance context for transparent criteria, budget constraints and sensitivity rather than opaque scoring: https://ppp.worldbank.org/library/infrastructure-prioritization-framework-ipf. COCO is useful for benchmark discipline in numerical multiobjective optimization but its numerical black-box suites do not directly represent this discrete dependency/resource portfolio task: https://coco-platform.org/data-archive/index.html.

The next robustness step, beyond this exact synthetic envelope, is uncertainty around the objective estimates themselves. That should use versioned uncertainty sets or calibrated distributions and keep a clear separation among (a) stress cases, (b) uncertain input intervals, and (c) probability-weighted forecasts. Do not silently reinterpret the current stress scenarios as a statistical model.

**Claim limit:** this output supports trade-off analysis only. It does not establish real ROI, jobs, causal service improvement, legal authorization, a legitimate civic priority ordering, or the optimal use of South African public resources.
