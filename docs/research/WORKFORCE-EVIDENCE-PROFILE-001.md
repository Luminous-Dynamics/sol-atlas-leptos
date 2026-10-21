# SA-WORKFORCE-EVIDENCE-001 — Labour-market evidence profile v0.1

**Tracking issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/271  
**Related benchmark:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/268  
**Status:** schema and synthetic examples only; not an official labour-market dataset.

## 1. Why this data layer exists

A listed high-demand occupation, a vacancy advertisement, an unfilled job, a queue of maintenance tasks, or a large pool of unemployed people is not, by itself, proof of a local skills shortage.

South Africa's official evidence illustrates why the model must preserve denominators and coverage:
- Statistics South Africa's QLFS for Q2 2026 reports an official unemployment rate of **33.6%**, 8.5 million unemployed people, 16.7 million employed people, LU3 of 43.8%, and LU4 of 46.3%. These are national survey indicators for the report's population and definitions; they do not imply that every occupation or locality has surplus workers.
- DHET's 2024 National List of Occupations in High Demand has **350 occupations**. Its ordering follows the OFO framework, not a priority ranking, and DHET explicitly says this list is **not a scarce-skills list**. It notes that some occupations with unusually high numbers of registered unemployed people require further investigation.
- The World Bank's August 2026 report describes simultaneous high unemployment and unfilled digital vacancies, with bottlenecks spanning foundational skills, programme alignment, and workplace learning.

Sources:
- Stats SA QLFS Q2 2026 key findings: https://www.statssa.gov.za/?PPN=P0211&SCH=74512&page_id=1856
- Stats SA QLFS 2026 data and metadata index: https://isibaloweb.statssa.gov.za/pages/surveys/pss/qlfs/2026/qlfs2026.php
- DHET 2024 National List / official notice: https://www.gov.za/sites/default/files/gcis_document/202404/50510gen2414.pdf
- World Bank, *From Demand to Delivery: Strengthening South Africa's Digital Skills Pipeline to Deliver More and Better Jobs* (14 Aug 2026): https://www.worldbank.org/en/news/feature/2026/08/14/from-demand-to-delivery-strengthening-south-africas-digital-skills
- O*NET database and task/skill descriptors: https://www.onetcenter.org/database.html
- European Commission ESCO classification: https://esco.ec.europa.eu/en/classification
- NIST, *The TEVV-Athlon Framework for Evaluating AI Systems* (initial public draft, Aug 2026): https://www.nist.gov/artificial-intelligence/ai-research/tevv-athlon-framework-evaluating-ai-systems

## 2. Paradigm shift: evidence statements, not one shortage score

The profile must keep separate:
1. **Labour supply:** who is in the measured population, with what experience, training and task-relevant competence.
2. **Employer demand:** which vacancies are real, budgeted, approved, paid, located and still open—not merely advertised.
3. **Task-ready capacity:** who has current, scope-relevant evidence of practical competence and is actually available under the stated conditions.
4. **Work quality and access:** compensation, shift, safety, equipment, transport, supervision, location, accommodation and retention.
5. **Service/work demand:** distinct, validated, funded work items and their verified completion status.
6. **Pipeline transition:** training entry, completion, demonstrated skill, paid placement, earnings/payment timing, and 6/12-month retention.
7. **Evidence quality:** source authority, release/observation date, population/denominator, geographic coverage, missingness, uncertainty and licensing/permission.

Emit separate statuses for each dimension and for the final diagnosis. Do not collapse them into one weighted score. A shortage claim should remain indeterminate unless independently evidenced paid demand and scoped task-ready supply are comparable in place and time, and plausible alternatives have been checked.

## 3. Source evidence contract

Every evidence record must include:
- immutable record ID, metric ID, unit, denominator, population, geography and time window;
- source ID, publisher, source version/release or capture date, retrieval date, access class, source URL where applicable, and provenance note;
- observation status: official survey estimate, employer-verified, administrative record, public taxonomy reference, model estimate, forecast, synthetic example, or unknown;
- freshness, uncertainty/interval status, taxonomy system/version, and any crosswalk status;
- explicit evidence limits and a claim ceiling.

A crosswalk between OFO, O*NET, ESCO or another taxonomy is versioned and lossy unless proven otherwise. It must expose ambiguity; same label or string similarity does not imply equivalent tasks, licences or proficiency. A credential is not evidence of current practical performance outside the assessed scope.

The strict schema is [workforce_evidence_record_v0_1.schema.json](schemas/workforce_evidence_record_v0_1.schema.json). Its companion [synthetic fixture](fixtures/workforce_evidence_records_v0_1.json) uses only invented test values and must never be described as an actual occupational or regional estimate.

## 4. Evidence statuses and prohibited inferences

| Evidence signal | Permitted inference | Not permitted |
|---|---|---|
| Occupation on DHET high-demand list | Relevant occupation for education/planning investigation | Confirmed shortage, priority ranking, or proof that a local vacancy is funded |
| Public vacancy advertisement | An advertisement was observed within a given period | Approved budget, genuine active opening, viable pay/conditions, or successful hire |
| Employer-verified funded vacancy | A named payer and approval are evidenced for a role on that date | Task-ready worker scarcity, durable job creation, or eventual fill |
| Registered unemployed occupation count | Registered supply inside that data system, subject to its coverage | Total unemployed workforce, current availability, or task readiness |
| Qualification/certificate | Credential asserted or verified within a stated scope | Current proficiency, willingness/availability, or authorization beyond scope |
| Completed training | Training completion in a defined cohort | Competence, paid placement, earnings, retention or causal employment impact |
| Service backlog | Reported workload under stated deduplication/validation rules | Distinct jobs, funded demand, skills shortage, or service loss without verification |
| Unknown, stale or contradictory evidence | Need to collect/revalidate evidence | A precise probability or confident root-cause label |

## 5. Benchmark baselines

- **B0 — local incumbent:** documented workforce planning/hiring process where a data owner supplies a valid baseline.
- **B1 — DHET list proxy:** assumes a high-demand list entry proves a shortage; expected to fail negative controls.
- **B2 — public-advert count:** counts public advertisements without proving funding, deduplication or currentness.
- **B3 — taxonomy match:** matches occupation/skills labels without proving local task-ready capability.
- **B4 — transparent evidence rules:** exposes source coverage and uncertainty, returns unknown where key evidence is missing.
- **Candidate — triangulated profile:** compares comparable, time-and-place-scoped paid demand with task-ready supply and work conditions; proposes evidence-gathering actions when the causes cannot be distinguished.

Use the same frozen cases and source versions for all baselines. Tune no thresholds on the final evaluation set.

## 6. Metrics to aim for

Synthetic contract targets:
- 100% schema validity and deterministic outputs under record-order permutation;
- zero promotion of synthetic values to observed claims;
- zero high-demand-list or raw-advert-only promotion to a confirmed shortage;
- explicit rejection or abstention for missing, stale, contradictory or incompatible denominators;
- full provenance and claim ceiling on every emitted result.

Real-world metrics, only after data-owner authorization and preregistration:
- false-shortage and false-surplus rates against independently reviewed local cases;
- interval coverage and calibration when a probabilistic model is statistically justified;
- diagnosis time and additional evidence needed to resolve uncertainty;
- funded-vacancy validation rate, time-to-fill, acceptance/decline, pay/shift conditions and retention;
- skills assessment to paid placement conversion and 6/12-month retention;
- verified service restoration, repeat work and fully accounted cost per valid resolution;
- error / access slices by geography and evidence coverage, subject to privacy and lawful-use constraints.

Do not claim that a model caused job creation or productivity gains from a before/after count alone. A real impact study needs a credible comparison design and independently verifiable outcomes.

## 7. Privacy, rights and adoption constraints

- Do not create individual employability/risk scores or automated adverse hiring decisions from this evidence profile.
- Avoid protected/sensitive attributes as proxy signals; evaluate aggregate accessibility and differential error only where lawful and justified.
- Use restricted survey microdata only through authorized access and in line with disclosure rules; public aggregate indicators do not authorize individual-level linkage.
- No scraping restricted vacancy systems or private accounts. Record licences and permitted uses for every source.
- Provide correction/appeal routes and worker/community input; do not let a source's missingness become a negative individual inference.
- A job counts as a paid-work outcome only when the payer, paid hours/earnings, work authorization, safe conditions, completed work and follow-up are evidenced.

## 8. First implementation sequence

1. Validate this schema and the entirely synthetic fixture with a strict parser.
2. Add source adapters only after allowed access, source version and release semantics have been reviewed.
3. Keep public aggregates and employer-validated records at separate evidence levels.
4. Build a crosswalk audit with unresolved/one-to-many mappings represented explicitly.
5. Run negative controls first: unfunded advert, stale vacancy, expired/scope-mismatched certificate, out-of-region candidate, nonviable job terms, duplicate opening, and unmatched denominator.
6. Add a holdout cohort from a legitimate local data partner before claiming predictive quality.

## 9. Claim ceiling

This is a measurement contract and synthetic fixture. It is not a real shortage estimate, hiring recommender, individual employability assessment, job-creation claim, or causal impact model. The QLFS and occupational-list figures cited above provide context only; they do not establish a local occupational shortage or authorize intervention.
