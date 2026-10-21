# SA-WORKFORCE-SOURCE-ROBUSTNESS-001 — Labour-source robustness benchmark v0.1

**Tracking issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/280  
**Composes with:** evidence profile #272, bottleneck diagnosis #268, outcome evaluation #277, and active evidence acquisition #279.  
**Status:** synthetic source-perturbation contract; not a validated South African shortage estimator.

## Paradigm shift: a conclusion should survive plausible ways the data could be wrong

A demand indicator is produced by a source-generating process, not dropped from the sky. Online job ads omit some hiring channels; one position can be copied or reposted; employer plans are not the same as funded vacancies; occupational taxonomies do not map perfectly; and high vacancy activity can be caused by turnover or nonviable work conditions rather than too few qualified people.

The benchmark asks not only what the base evidence says, but how the conclusion changes under predeclared, plausible perturbations to coverage, duplication, mapping, freshness, survey selection and job quality. If an admissible perturbation changes the decision class, the system must surface fragility or abstain. It must not silently convert a fragile ranking into a precise shortage claim.

## Research grounding

The OECD's current review of shortage measurement notes that indirect indicators can change with business cycles, labour-market institutions and external shocks, and that vacancy-based measures alone omit information such as the time positions remain unfilled. It also notes that high vacancy counts may arise from turnover, low pay, poor prospects or ineffective HR practices. Its 2024 comparison of Lightcast data against official vacancy sources in Australia, Canada, the UK and the US found regional representativeness generally better than occupational and sectoral representativeness in those countries. These results motivate tests; they do not automatically characterise South African data.

The 2024 DHET National List of Occupations in High Demand combines multiple secondary indicators, including QLFS, LMDS and consolidated private vacancy data, with an employer survey distributed through DHET stakeholder networks. This is useful national planning evidence, but it is not a local, time-specific measure of funded task-ready work. Official Stats SA labour-force measures describe defined populations and should not be used as if they directly measured a vacancy-level skill gap.

Sources:
- OECD, *Mapping Skill Shortages in OECD Countries*, methodological considerations: https://www.oecd.org/en/publications/mapping-skill-shortages-in-oecd-countries_735f9485-en/full-report/component-5.html
- OECD, *How well do online job postings match national sources in large English speaking countries?* (2024): https://www.oecd.org/en/publications/how-well-do-online-job-postings-match-national-sources-in-large-english-speaking-countries_c17cae09-en.html
- DHET, 2024 National List of Occupations in High Demand: https://www.gov.za/sites/default/files/gcis_document/202404/50510gen2414.pdf
- Eurostat, skills measurement dimensions and data sources: https://ec.europa.eu/eurostat/web/skills/methodology
- Stats SA, QLFS Q2 2026: https://www.statssa.gov.za/?PPN=P0211&SCH=74512&page_id=1856

## Scope and non-goals

This first version owns only source-perturbation cases, the source-sensitivity contract and a synthetic result manifest. It does not create a new source registry, publish a composite shortage score, scrape restricted vacancy systems, identify individual workers, or infer an individual's employability.

It must compose the source metadata/access limits from PR #272, not duplicate source ownership. It must compose the diagnosis labels and action gates from PR #268, not authorize action. If task-ready supply cannot be measured using authorized comparable evidence, say so; do not fill the gap with credential counts or demographic proxies.

## Source record contract

Every source must state:
- source family, source period, geography and population;
- measure, value and unit, with a named denominator where applicable;
- access class and freshness/age of the source;
- coverage or sampling limitation and taxonomy/version caveat;
- whether a number is observed, administrative, survey-reported, or a derived proxy in a real evaluation.

For this v0.1 corpus every numeric value is synthetic. Values are not local estimates, sample weights or probabilities.

## Perturbation families

1. **Duplicate/repost pressure:** multiply online ad records while keeping the independently supported number of funded positions constant. A high raw ad count must not automatically increase funded headcount.
2. **Missing hiring channel / coverage shift:** reduce or remove one posting source, employer group or informal hiring channel. If this can materially change the ranking, report fragile/indeterminate instead of pretending the sample is representative.
3. **Employer intention vs funded demand:** remove the funded-vacancy source while retaining employer statements about future hiring. An intention-to-hire report is not current funded demand.
4. **Job-quality confounding:** introduce nonviable pay/shift/conditions, high offer declines or turnover. A conclusion should consider job quality instead of labelling all persistent vacancies a skill shortage.
5. **Taxonomy crosswalk uncertainty:** evaluate alternate plausible OFO/provider mappings. If the rank or decision changes, publish a mapping-sensitive range or mark the comparison indeterminate.
6. **Staleness/reference-period mismatch:** age a key record or misalign source periods. Do not combine stale and current measures without an explicit sensitivity result.
7. **Leave-one-source-family-out:** remove each source family in turn, reporting what conclusion changes. Never let a large, high-volume source dominate solely because it yields many records.
8. **Denominator/population mismatch:** compare similar units only. National workforce supply does not directly establish local task-ready availability, and course completion does not directly establish competence or placement.

## Decision vocabulary

- **confirmed_task_capacity_gap:** only when source contract supports current, funded work and comparable task-ready capacity evidence shows a gap. This remains a synthetic decision label in v0.1, not a real claim.
- **no_confirmed_shortage:** evidence fails to establish a shortage under the defined claim rule. This is not proof that no shortage exists.
- **job_quality_confounding:** evidence shows material job-quality/retention issues that could explain repeated vacancies; this does not establish that skills are sufficient.
- **fragile_or_indeterminate:** conclusions change under a permitted plausible perturbation, source support is insufficient, source periods/populations mismatch, or taxonomy uncertainty changes the decision.

Do not use a single numeric score as the answer. If multi-indicator summaries are later added, expose raw indicators, denominators, weights, source contributions, alternate-weight sensitivity and the decision rule.

## Baselines

- B0: raw vacancy/ad/backlog count, retained only as a weak negative control.
- B1: deduplicated posting count, still without coverage adjustment.
- B2: transparent source-aware rule with explicit comparability and abstention.
- B3: perturbation envelope / leave-one-source-family-out analysis.
- B4: independently adjudicated reference decisions for a frozen synthetic corpus or authorized retrospective cohort.

Use the same input cutoff, occupation mapping and decision definition for every baseline. A vendor comparison must be limited to the task the vendor actually performs and must use matched data and budgets.

## Metrics

- **Decision stability:** fraction of predeclared perturbations that preserve the decision class, with the perturbation denominator exposed.
- **Fragility detection:** rate of correctly flagging a decision when an admissible perturbation changes its class.
- **Proxy inflation error:** change in estimated demand when ad duplicates are added but independent funded-position evidence is unchanged; target zero change to the funded-position count.
- **Leave-one-source-out sensitivity:** per-source-family delta in any published indicator/rank/decision.
- **Rank stability:** Kendall/Spearman correlation and top-k set overlap only where ranking is meaningful and the alternatives are comparable.
- **Denominator integrity:** zero comparisons whose geography, population, reference period or unit mismatch has been silently ignored.
- **Job-quality confounding:** rate at which high turnover/nonviable conditions induce an unsupported skills-shortage claim.
- **Freshness and mapping sensitivity:** cases that are correctly marked indeterminate when stale data or alternate taxonomy mappings change the conclusion.
- **Cost/compute:** runtime and manual review burden as the number of sources and perturbations grows.

## Acceptance gates

- Zero confirmed shortage decisions from ads/backlog alone or employer intention alone.
- Repeating/duplicating ads does not increase verified funded-position counts absent independent evidence.
- Dropping a pivotal source cannot silently preserve a precise claim if the required evidence contract is no longer satisfied.
- If a predeclared plausible source perturbation changes the decision class, report fragile/indeterminate unless a declared independent anchor keeps the claim identified.
- Alternate taxonomy mappings that change the claim must be visible in the result receipt.
- Source deletion, record permutation and fixed perturbation manifests are deterministic.
- Every output includes source IDs/digests, period, geography, population, denominator, mapping version, freshness, perturbation ID and claim ceiling.
- Synthetic results stay synthetic-only and do not imply local vacancy rates or job creation.

## Implemented evaluation slice

The Rust crate `tools/workforce-source-robustness-verifier/` now validates the JSON Schema and cross-field source/perturbation IDs, replays the declared synthetic perturbations, derives one of the four explicit decision classes, and emits a receipt with source-fixture and schema SHA-256 digests. The dedicated workflow gates formatting, unit tests and receipt generation.

The first evaluator includes deterministic tests for all six baseline cases and eleven perturbations, extreme ad-volume inflation, removal of funded-demand evidence, taxonomy/time-period fragility, invalid schema fields, duplicate source IDs and input-order invariance. Post-perturbation sources are rechecked for finite non-negative values, bounded age, valid date shape/order, unique IDs and required comparability metadata. Unsupported operations, malformed parameters and absent source references fail closed.

These tests validate a synthetic contract, not empirical source bias. Do not convert the deterministic rule thresholds into a production shortage index; later work needs a legitimate source owner, coverage analysis and independent real-data evaluation.

## Claim ceiling

The benchmark tests robustness against authored distortions. It does not validate a real source's national/local representativeness, produce actual shortages, estimate job creation, demonstrate causal effects, or establish superiority over state of the art. Empirical claims require authorized source records, sampling/coverage analysis, independent validation and a preregistered comparison design.
