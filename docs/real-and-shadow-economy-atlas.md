# Real and Shadow Economy Atlas — Measurement Architecture

**Status:** Research-backed product and data-contract proposal; not yet implemented or empirically validated.  
**Target:** Sol Atlas; related provenance and modelling boundaries for Mycelix and Symthaea.  
**Research snapshot:** 13 October 2026.

## 1. Why this needs a larger model

Yes: the opportunity is materially larger than a map of an estimated “shadow economy percentage.” A useful system should show how people produce, exchange, finance, consume and sustain life; what official statistics already capture; what is incompletely observed; and which estimates are method-dependent.

The phrase *shadow economy* is not a single, directly observed quantity. The IMF's review of estimates across 158 countries and 1991–2015 compares direct surveys and administrative/statistical methods with indirect approaches such as currency-demand and MIMIC models. Its reported historical average is a result of that specific sample, period and method—not a current universal parameter. Different approaches have different assumptions and error modes. ([IMF, 2018](https://www.imf.org/en/publications/wp/issues/2018/01/25/shadow-economies-around-the-world-what-did-we-learn-over-the-last-20-years-45583))

The IMF's framework for measuring the informal economy aligns the scope to national accounts and identifies several components of production: informal-sector units; goods for own final use; domestic workers; and informal employment within otherwise formal enterprises. The ILO's indicator framework goes wider than a firm-registration count: people, jobs, production units, sectors, own-use production, working hours, earnings, working conditions and social protection all matter. ([IMF, 2021](https://www.imf.org/en/publications/policy-papers/issues/2021/02/02/measuring-the-informal-economy-50057); [ILO indicator framework](https://www.ilo.org/informal_economy_indicator_framework))

**Core design rule: observability, formality, and legality are independent dimensions.** Informal activity is not automatically illegal. Activity missing from one dataset is not necessarily hidden from all institutions. An anomalous payment is not a measure of production, and a statistical gap is not evidence of a crime.

## 2. One economic system, several non-interchangeable lenses

Treat the economy as linked but distinct activity, value, finance, work and ecological accounts.

| Lens | Unit of analysis | Questions it should answer | Values that must not be confused |
|---|---|---|---|
| Production and value added | Industry, enterprise, production unit, region, national accounts | What goods and services are produced? Where are inputs transformed into output? | Gross sales versus value added; intermediate inputs versus final output |
| Work and livelihoods | Person/job/household, only at privacy-safe aggregate levels | Who works, how many hours, for what earnings, and with what protections? | Informal sector enterprises versus informal jobs; employment versus hours; earnings versus output |
| Household and community production | Household/community, aggregate time-use measures | What is produced for own use, shared, cared for, repaired or maintained outside priced markets? | Unpaid labour time versus monetary output; what falls inside versus outside the national-accounts production boundary |
| Enterprise and value chains | Enterprise, supplier/customer relationship, sector and commodity chain | Where do micro-enterprises fit into formal supply chains? Which bottlenecks suppress productivity, wages and market access? | Invoice value, physical volume, gross output, margins and value added |
| Financial circulation and credit | Instrument, institution, aggregated flow or balance sheet | How do payments, cash, credit, remittances, debt and asset ownership enable or constrain activity? | Payment turnover versus production; credit stocks versus credit flows; transfers versus newly produced value |
| Cross-border measurement and IFFs | Reporter/partner/period/commodity, tax or commercial method, case-specific evidence | Where do trade asymmetries, tax/commercial estimates and other cross-border value-flow indicators appear? | Raw mirror gaps versus adjusted estimates, measured IFFs, or adjudicated findings |
| Materials, energy and ecosystems | Physical flow, stock, sector and location | Which energy, water, land, minerals and materials support production? What waste, depletion or ecosystem service is involved? | Physical quantities versus monetary value; flow during a period versus a stock at a date |
| Institutions and formalization | Worker, job, enterprise or policy/region cohort | Which rules, registration burdens, benefits, financing barriers and protections shape participation? | Registration counts versus decent work, productivity or welfare outcomes |

Financial activity belongs in the model, but it is not a separate “fake economy”: it funds, prices, insures and transfers claims on real activity. It can also circulate repeatedly without each transaction adding equivalent GDP. Keep it linked to production, but don't add payment turnover to value added.

Unpaid care and own-use production deserve visibility, but must be measured on their own appropriate basis. Most unpaid domestic services are not priced market output; care hours should not be arbitrarily converted to currency and added to GDP. Where a time-use or satellite-account valuation is offered, show its method and keep it separate from core GDP totals.

## 3. The independent axes of every record

Do not create one field called `shadow_status` that collapses different concepts. An observation, estimate or claim should carry independently versioned fields like these. Names are conceptual and not current Rust types.

### A. Economic subject and unit

`unit_of_analysis`: `person`, `job`, `household`, `enterprise`, `transaction`, `production_unit`, `sector`, `region`, `jurisdiction`, `supply_chain`, `asset_stock`, `physical_flow`.

A worker can hold an informal job at a registered firm. A registered company may use informal labour. A household may produce for its own use. Do not assign a single informal/formal label to an entire country, person or multi-activity business if the evidence is about a particular job or activity.

### B. Statistical visibility

`recording_state`: `directly_recorded`, `partially_covered`, `model_estimated`, `residual_or_discrepancy`, `coverage_unknown`.

This dimension describes a dataset's relationship to the activity, not whether the activity is illegal. “Not found in this source” should never be rendered as “hidden from authorities.”

### C. Formality / institutional coverage

`formality_state`: `formal`, `informal`, `mixed`, `not_applicable`, `unknown`.

Keep legal registration, tax coverage, enterprise record-keeping, employment status, social-insurance coverage and labour protections as separate attributes where available. Their definitions depend on the applicable statistical standard and reference period. Do not infer all of them from one registration flag.

### D. Legal / evidentiary disposition

`legal_assessment_state`: `not_assessed`, `reported_allegation`, `administrative_concern`, `official_finding`, `adjudicated_finding`, `disputed_or_under_review`.

This is a provenance field, never an ML prediction. An allegation or investigative signal is not an official finding; an official finding does not automatically support broader claims about connected people or firms. Never infer a legal state from informality, poverty, cash use, location, network proximity or a statistical anomaly.

### E. Value and account boundary

`value_kind`: `gross_output`, `value_added`, `wages_or_earnings`, `household_consumption`, `financial_transfer`, `credit_flow`, `asset_stock`, `labour_time`, `physical_flow`, `ecosystem_service`, `estimated_iff`, `other_defined_measure`.

Each value needs its unit and meaning: currency, price basis/year, current versus constant prices, geographic scope, reference period, denominator and conversion method. Null means unknown or not reported; it must never mean zero. A derived metric should list its parent values and whether the parents overlap.

### F. Evidence method and uncertainty

`evidence_method`: `administrative_record`, `household_survey`, `enterprise_survey`, `labour_force_survey`, `national_accounts`, `audit_or_enforcement_data`, `physical_proxy`, `macro_econometric_model`, `direct_calculation`, `scenario`.

Retain the method version, specification, model parameters, uncertainty interval or error description, known coverage bias, source snapshot, and all exclusions. Keep source reliability, sampling error, model uncertainty, entity resolution confidence and legal status separate. Do not compress them into a universal trust score.

### G. Time and revision

Keep at least:
- **Reference/valid time:** the period the record describes.
- **Known/retrieved time:** when this exact revision was available to the system.
- **Publication/revision time:** only where the source separately reports it.

A revised survey or corrected national account should create a new version and preserve the older known-at state, not silently rewrite historical dashboards.

## 4. Measurement rules that stop false totals

1. **No global shadow-economy total by default.** First publish estimates by method, population, geography, period, value concept and scope. Present a range or scenario envelope only where a documented synthesis model justifies it.
2. **No summing unlike methods.** Currency-demand, MIMIC, electricity/light proxies, survey results, national-accounts discrepancies and tax-audit data can measure different populations or respond to different causes. Triangulate; do not add them as if each were a separate pile of output.
3. **Do not double count through value chains.** Adding the full sale of a supplier to the purchaser's full output counts intermediate goods more than once. Use value-added, supply-use/input-output reconciliations, or an explicitly gross-flow view.
4. **Keep stocks and flows apart.** Debt outstanding, cash in circulation, wealth and registered assets are stocks. Wages, payments, production and transfers are flows over a period. Never add them.
5. **Keep the national-account boundary explicit.** Distinguish market production, own-use production that is within the production boundary, and unpaid domestic/care services represented in supplementary time-use or satellite accounts.
6. **Preserve zeros, missingness, suppression and non-response separately.** Missing data are not zero; small samples suppressed for privacy are not zero; a rejected source row is not a valid observation.
7. **Handle changes in standards as potential breaks.** If a survey definition, sampling frame, weighting scheme or classification changes, show the breakpoint and document whether historical values were backcast.
8. **Use denominator labels everywhere.** “Percentage of jobs,” “percentage of employed people,” “share of enterprises,” and “share of GDP” are different measures. The denominator belongs in the metric name and exported metadata.

## 5. A valuable South Africa pilot

**Data-use gate (reviewed 13 October 2026):** Stats SA's official [QLFS Q4 2025 metadata](https://isibaloweb.statssa.gov.za/metadata/QLFS/2025/Qtr04/Quarterly%20Labour%20Force%20Survey%204th%20Quarter%202025%20metadata.pdf) states that data applications must acknowledge Stats SA and identify the application/analysis as independently processed. It also says the basic data and any reprocessed version or application may not be sold or offered for sale in any form without prior permission. We therefore do not put QLFS unit-record data or reprocessed data in this public repository, and any commercial offering based on such data is blocked pending permission. This does **not** by itself resolve the rights to reuse the separately published headline-summary page or its two percentages: that summary stays `terms_review_required` until its own terms are reviewed.


South Africa is a practical first case because Stats SA exposes downloadable Quarterly Labour Force Survey (QLFS) datasets and metadata, and UNCTAD lists South Africa among countries in which methods for tax and commercial illicit financial-flow estimates were pilot-tested. Start with public, aggregated, legally reusable data rather than trying to infer individual activity.

A current example shows why the multidimensional model matters. Stats SA's Q4 2025 public summary reports that **21.4% of jobs were in the informal sector**, while **33.5% of employed people were in informal employment** as defined by lack of particular protections such as pension access and paid annual/sick leave. These aren't contradictory: one describes the production unit/sector, the other the job's protection status, including jobs inside formal-sector enterprises. The same summary says the updated definitions were adopted in Q3 2025, and the enterprise test now places more emphasis on tax/CIPC registration and record-keeping than on enterprise size. A naïve pre/post chart can therefore mistake a definitional change for an economic shock. See [Stats SA, 4 March 2026](https://www.statssa.gov.za/?p=19240) and the [QLFS 2025 data portal](https://isibaloweb.statssa.gov.za/pages/surveys/pss/qlfs/2025/qlfs2025.php).

### Suggested source ladder

| Source class | First use | Mandatory caveat / gate |
|---|---|---|
| Stats SA QLFS, QLFS metadata and Labour Market Dynamics | Jobs, employment status, sector, wages/earnings where available, demographic and industry aggregates | Follow survey weights, uncertainty, metadata and Q3 2025 definition changes; do not redistribute microdata without checking its terms |
| Stats SA national accounts, supply/use tables, household surveys and environmental-economic accounts | Output, value added, household conditions, inputs and physical resource context | Align units, price bases, period and production boundary before comparison |
| ILO indicator framework and ILOSTAT | Comparable informality, decent work, own-use production and unpaid care indicators | Preserve each indicator's definition, population, denominator and standard version |
| IMF informal-economy measurement framework | Reconciliation to national-accounts concepts and multi-source estimates | A framework is not itself a live country estimate |
| UNCTAD/UNODC IFF framework and SDG 16.4.1 reporting templates | Separate, method-specific tax/commercial IFF estimates and metadata | Estimate only the categories and methods actually supported; do not label all informal activity an IFF |
| UN Comtrade | Cross-border merchandise-trade observations and raw mirror comparison | Respect preview limits and licensing; no mirror gap is a criminality finding |
| Public enterprise surveys / annual reports / sector surveys | Enterprise formation, financing, input constraints, productivity and supply chains | Avoid treating an absent firm record as evidence of a hidden or unlawful firm |
| Payments, cash, credit and energy proxies | Aggregate sensitivity indicators and stress tests | Proxy is not a direct observation of informal output; lawful privacy and reuse review required |

Key references:
- [ILO informal economy indicator framework](https://www.ilo.org/informal_economy_indicator_framework)
- [ILO Resolution on promoting transitions to formality, 13 June 2025](https://www.ilo.org/resource/record-decisions/resolution-concerning-general-discussion-addressing-informality-and)
- [ILO, *Using Firm-level and Integrated Data Sources for Enterprise Policy Development*, 8 October 2026](https://www.ilo.org/publications/using-firm-level-and-integrated-data-sources-enterprise-policy-development)
- [IMF, *Measuring the Informal Economy*, 2021](https://www.imf.org/en/publications/policy-papers/issues/2021/02/02/measuring-the-informal-economy-50057)
- [World Bank, *The Long Shadow of Informality*](https://www.worldbank.org/en/research/publication/informal-economy)
- [UNCTAD/UNODC conceptual framework for statistical measurement of IFFs](https://unctad.org/publication/conceptual-framework-statistical-measurement-illicit-financial-flows)
- [UNCTAD IFF statistics, SDG 16.4.1 and reporting templates](https://unctad.org/statistics/illicit-financial-flows)
- [Stats SA QLFS Q4 2025 publication page](https://www.statssa.gov.za/?PPN=P0211&SCH=74047&page_id=1854)

## 6. What Sol Atlas, Mycelix and Symthaea should do

- **Sol Atlas — explore and compare:** spatial and temporal dashboards with named units, definitions, denominators, evidence coverage, method ranges and data-break warnings. A map should visualize aggregated evidence, not create a league table of suspected criminals.
- **Mycelix — preserve provenance and corrections:** source licence and permitted-use metadata; raw snapshot digests; transformation identities; parent-record links; correction, supersession and revocation history; reproducible result bundles. Provenance proves lineage/integrity, not the truth of an input.
- **Symthaea — propose hypotheses and test sensitivity:** detect inconsistencies, identify which extra data would discriminate between competing explanations, model productivity/credit/energy/policy scenarios, and report when several models disagree. It must not convert an anomaly into a legal conclusion or use opaque “shadow activity” scores against people.
- **Human/statistical authority — assess claims:** official statistics, legal determinations and domain-expert assessments remain distinct source classes. Model output never silently upgrades an allegation to a finding.

## 7. First implementation sequence

### Phase 1 — Aggregate evidence map
Create a versioned metric registry and source registry. Load one small, reproducible set of public Stats SA/ILO aggregates with exact source dates, definitions, units, denominators, source URLs, terms, and method metadata. Render formal-sector jobs, informal-sector jobs, informal employment and job-protection measures as separate series.

### Phase 2 — Account reconciliation
Link labour and enterprise data to national accounts using compatible industry, geography and reference-period mappings. Show direct observations separately from residual/model estimates. Implement coverage and comparability checks before calculating totals.

### Phase 3 — Livelihood and real-economy measures
Add earnings distributions, hours, social protection, enterprise finance access, productivity, household own-use production/time use and supply-chain relationships. Separate the core production-boundary output from supplementary unpaid-care indicators.

### Phase 4 — Financial and cross-border layer
Add cash/payment/credit aggregates, remittances and method-qualified cross-border indicators. Keep value-flow volumes, assets, output and value-added as separate account types. For IFFs, require a cited measurement method and the corresponding metadata/reporting template.

### Phase 5 — Stress tests and policy comparison
Run deterministic scenarios for productivity support, registration-cost changes, social-insurance access, credit availability, market access and enforcement approaches. Evaluate impacts on earnings, survival, coverage, productivity and fiscal capacity. Formalization should not be scored by registrations alone: improving decent work and economic resilience is the objective.

## 8. Machine-readable South Africa pilot seed

A first source-attributed aggregate registry now lives at:

- `data/economic_metrics/south_africa_qlfs_q4_2025_v1.json`
- Contract verifier: `scripts/verify_economic_metric_registry.py`
- CI executes both `py_compile` and the registry verifier; this checks the contract, not statistical recomputation.

The registry contains two official headline values from Stats SA's Q4 2025 summary: 21.4% for jobs in the informal sector and 33.5% for employed people in informal employment. They are separate metric IDs with different units of analysis, denominators, account boundaries and interpretation ceilings.

The summary's headline rates do not provide exact numerator/denominator counts or uncertainty intervals, so those fields are deliberately null rather than reverse-engineered from rounded percentages. The source-body digest is also null and labelled not captured; a human-readable snapshot ID is not a cryptographic digest. The official [QLFS Q4 2025 metadata](https://isibaloweb.statssa.gov.za/metadata/QLFS/2025/Qtr04/Quarterly%20Labour%20Force%20Survey%204th%20Quarter%202025%20metadata.pdf) states that applications must acknowledge Stats SA and identify the analysis as independently processed, and that the underlying data and reprocessed versions/applications may not be sold or offered for sale without prior permission. The repository does not include unit-record QLFS data. Reuse status for the two values copied from the separately published summary remains `terms_review_required` pending review of that summary's own reuse terms; the metadata restriction is not treated as a definitive ruling on the standalone summary. The registry therefore does not claim to be a complete QLFS extract or a fully qualified reusable dataset.

### Cross-country source candidate: World Bank Informal Economy Database

The active machine-readable source qualification record is `data/economic_sources/world_bank_informal_economy_database.json` (registry v7); v1–v6 remain in the same directory as preserved historical snapshots. `scripts/verify_economic_source_registry.py` and CI check its fail-closed metadata contract, including the edition discrepancy guard. It is deliberately **metadata-only**: no workbook values are copied into the repository, no source-body digest is claimed, and ingestion/redistribution remain unapproved.

The 2024 [World Bank source brief](https://www.worldbank.org/en/brief/2024/01/09/informal-economy-database) describes up to 196 economies over 1990–2020 and claims 11 measures, but its bullets list two output estimates (DGE and MIMIC), four employment indicators and six perception indicators—12 listed series if DGE and MIMIC are counted separately. A separate [World Bank Data360 metadata PDF](https://data360files.worldbank.org/data360-data/datasetmetadata/WB_INFECDB.pdf) describes the `WB_INFECDB` dataset over 1990–2018 and claims 12 measures; the [Data360 catalogue search](https://data360.worldbank.org/en/search) also displays the Informal Economy Database with 12 indicators. This 12/2018 metadata claim conflicts with the brief's 11/2020 claim. It may reflect a different edition or metadata taxonomy; the metadata version date and the exact current workbook inventory remain unverified. The [QoG Datafinder catalogue](https://datafinder.qog.gu.se/dataset/ied) exposes `ied_dge` and `ied_mimic` as separate variables, and a separate World Bank reproducibility package documents `DGE_p` and `MIMIC_p` sheets in a processed derivative ([catalogue record](https://reproducibility.worldbank.org/catalog/330)). None of these records establishes the current Excel workbook's exact inventory. Capture and inspect the exact `Informal economy data (Excel)` bytes, digest, edition and worksheet/column inventory before resolving the count or combining time series. Keep DGE, MIMIC, survey, employment and perception series separate; do not sum, average or imply they estimate the same quantity. The record remains metadata-only; no Data360 API result, workbook bytes, hash, or observations were captured in this review.

The licence evidence is contradictory at the catalogue-metadata level, including within one page. The [Jobless Development catalogue record](https://reproducibility.worldbank.org/catalog/265) names `informal-economy-database.xlsx`, links to the official source page, and labels that dataset CC BY 4.0; its page-level overview simultaneously says the parent reproduction ZIP and its reproduction data/code remain temporarily embargoed. A separate dataset row calls the original Excel file publicly available and included. The active v6 record preserves those scopes separately rather than using the embargoed ZIP as a download route. Another package lists `DGE_Raw.xlsx` as publicly available with “Custom license” ([catalogue record](https://reproducibility.worldbank.org/catalog/583)); a third CC BY 4.0 label applies only to a processed derivative ([catalogue record](https://reproducibility.worldbank.org/catalog/330)). I attempted the official direct Excel URL: the web reader rejected the XLSX content type, while the container fetch failed DNS resolution before any HTTP response. This is an execution-environment limitation, not evidence of a source outage. No bytes, HTTP status, digest, byte length, or sheet inventory are claimed. The rights decision remains unresolved; this is a precautionary project gate, not a claim that reuse is legally prohibited. Before public/commercial reuse, capture the exact official workbook and reconcile the current terms with the catalogue records.

A narrower alternative is documented in the v6 registry: the World Bank's 2025 reproducibility package for *(In)Formalizing Jobs in Latin America and the Caribbean* identifies a processed derivative, `informal-economy-database-with-fixes.dta`, reports the source sheets `DGE_p` and `MIMIC_p`, lists the DGE/MIMIC period-average variables for 1993–2019, and labels that processed dataset CC BY 4.0 ([catalogue record](https://reproducibility.worldbank.org/catalog/330)). The same record also lists `INFORMALITY DATA SET.dta` as a separate author-provided file described as not publicly available from its underlying source, while noting it is included in the reproduction package for verification. Do not conflate its provenance or rights with the public-link-derived derivative. The candidate package and target file have not yet been downloaded and hashed. This is a promising bounded route for two historical output estimates, not a replacement for the official workbook's full eleven/twelve-indicator inventory, and not permission to apply the derivative's license metadata to the current Excel file.

The Q3 2025 informality-definition change is represented as a versioned comparability event affecting the informal-sector classification measure. The panel should show the marker, and the system must not calculate an adjacent-quarter economic change from mixed definitions unless a documented bridge/backcast exists. The second measure remains a distinct employment-protection measure; the summary alone does not establish that the enterprise-classification change should be attached to it as the same event.

Source evidence: [Stats SA Q4 2025 informality summary, published 4 March 2026](https://www.statssa.gov.za/?p=19240) and the [official QLFS 2025 data and metadata portal](https://isibaloweb.statssa.gov.za/pages/surveys/pss/qlfs/2025/qlfs2025.php). The portal makes quarter-specific datasets, questionnaires and concepts/definitions available. The seed is intentionally based on the published summary only; a future qualified data ingestion should reconcile it against Q4 source metadata and table definitions.

## 9. Minimum acceptance tests before public conclusions

- Same activity, different formality and visibility states remain separately representable.
- An informal job at a formal firm does not force the firm to be classified informal.
- A formal-status record does not overwrite a separate allegation or official finding; no legal state is inferred from location, sector or cash use.
- A null value stays null and does not enter sums as zero.
- A gross-output series cannot be summed with value-added or transaction turnover without an explicit reconciliation method.
- Overlapping macro estimates cannot be totalled by a generic `sum()` path; aggregation requires compatible scope fingerprints and a declared method.
- Survey revisions and classification breaks produce explicit comparability warnings and preserve old revisions.
- Model results preserve method version, inputs, uncertainty and exclusions and can be replayed from frozen snapshots.
- Privacy thresholds block display of sparse household/job cells; no person-level “shadow economy” ranking or criminality score is generated.
- Scenario results are clearly labelled as counterfactual, never observed facts.

## 10. Explicit non-goals for the first release

No universal shadow-economy percentage; no inferred guilt or “suspicious country” score; no person-level payment surveillance; no sale of worker-level traces; no automated tax-enforcement referrals; no summing incompatible estimates; no live connector before terms/access are qualified; no claim that better visibility alone improves welfare.

**Recommended first milestone:** a South Africa aggregate “labour, informality and livelihood” panel with a visible definition-break marker at Q3 2025, built before any global shadow-economy total. This is a smaller feature that can be validated against published Stats SA tables and immediately demonstrates why a multidimensional economic model is better than a single shadow-economy slider.
