# City in-use outcomes: standards and industry crosswalk v0.1

This crosswalk clarifies which external methods already exist and where the CityMaker adapter adds integration. It is not a claim of certification or a substitute for the current full standards, project-specific methods or South African statutory requirements.

## External references are complementary, not interchangeable

| Reference | What it contributes | What it does not prove alone | CityMaker use |
|---|---|---|---|
| CIBSE TM63 (2020, active) | Methodological framework for measurement and verification of building energy in use; calibrated energy-model comparison and identification of performance-gap causes | Not a universal residential POE standard, not a complete comfort/health/safety method, and not a proof that any particular building is compliant | Use for energy model calibration/operational comparison where applicable; pin edition and declare its UK-origin assumptions before applying in South Africa |
| CIBSE TM61 | Operational performance of buildings | Not a building commissioning record or field metrology system | Use as a performance/operational framing reference |
| CIBSE TM62 | Surveying occupant satisfaction | Self-report cannot establish structural, fire, IAQ, energy-meter or clinical claims | Use for a separate experience stream with clear sampling, consent and privacy |
| CIBSE TM64 | Indoor-air-quality emissions sources and mitigation | Not a general health diagnosis or proof of all indoor environmental quality dimensions | Map relevant indoor-air variables and source evidence |
| IPMVP Core Concepts 2022 | M&V framework for energy/water efficiency projects, baseline and post-period measurement/adjustment, defined options | An IPMVP-aligned plan or status does not itself prove savings; project-specific data and method matter | Mark eligibility for savings review only when method, baseline, post-period, coverage and adjustments are present; do not emit a savings amount from this adapter |
| ASHRAE / applicable building energy modelling validation standards | Software capability/reference-case checks and model quality concepts | Software validation does not establish a particular model's site-specific calibration or predict resident experience | Use specialist owner's validation/certification output, with exact software version and scenario |
| South Africa SANS 10400-XA:2026 Edition 3 (latest edition identified by SAIAT) | Current identified national energy-use standard edition; exact requirements must be taken from the official SABS text | SAIAT's specialist CPD record is secondary evidence; it does not establish project-specific effective/transition treatment, applicability or compliance | Keep regulatory/compliance evidence separate from measured in-use performance; verify the exact SABS publication and project applicability with the competent authority and qualified professional before encoding checks |
| POE systematic reviews | Evidence that residential POE remains inconsistent; design-stage planning, continuous collection and user engagement matter | No one universal POE protocol is globally dominant | Preserve a versioned multi-method record and map each local method to its exact question and limits |
| 2026 building-health review | Synthesizes six dimensions: safety, environmental quality, occupant-centred performance, system/infrastructure, energy and economic performance | A six-dimension literature taxonomy is not an accepted universal compliance score | Report dimension-specific evidence and missingness; avoid one blended health score |

### South African edition-status correction — 8 November 2026

SAIAT's professional CPD portal identifies **SANS 10400-XA:2026, Edition 3** and states it was published early in 2026, with a change to the energy-efficient hot-water compliance method. Its presenter is identified as chair of SABS TC 060 and a member of the latest review workgroup. This is specialist secondary evidence of the edition, not a substitute for the copyrighted SABS publication or an applicability determination.

The official SABS catalogue entry, lawful full text, effective/transition date and project-specific application have not been independently verified here. Therefore:
- do not describe the 2021 Edition 2 as the latest identified edition;
- keep the regulatory applicability state Unknown until the official SABS edition and project context are checked;
- do not encode clause-level pass/fail logic from secondary summaries;
- do not infer that an as-designed energy model, an in-use meter reading or this synthetic oracle establishes statutory compliance.

SAIAT source: https://cpd.saiat.org.za/sans-10400-xa-3rd-edition-cc

## Paradigm shift: a longitudinal evidence graph

Existing POE methods often focus on a single data collection event or a limited set of outcomes. The CityMaker addition is the exact, version-aware connection across:
- brief and design target;
- design-stage prediction;
- as-built and BOM;
- installation and commissioning;
- actual use, weather, occupancy and operations;
- technical measurements;
- resident/user-reported experience;
- maintenance, repair, correction and recommissioning;
- measured post-intervention outcomes and future design lessons.

The comparison can only be admitted when the scope, design/BOM revision, period, functional unit, operating context, source owner/contract/revision and instrument or consent requirements match.

## Required benchmark comparisons

1. Reproduce one CIBSE TM63-style calibration case with a pinned model and a separate observation/validation period; compare static prediction gap with calibrated dynamic gap.
2. Use an IPMVP-aligned synthetic baseline/post case. Verify that the evaluator reports only eligibility—not savings—without an accepted M&V report.
3. Compare technical IEQ measures and resident feedback as separate outcomes. Test non-response, consent denial, seasonal coverage and subgroup missingness.
4. Introduce service faults, repair events and recommissioning. Compare maintenance events with actual post-repair service receipts, not an operator's "closed" work order alone.
5. Compare the same accepted building functional unit across seasons and operating regimes; report confidence and missingness, not only a building-wide mean.
6. Keep regulatory compliance, energy performance, indoor environmental quality, accessibility, affordability and lived experience non-compensatory where they are hard floors.

## Suggested metrics

- Energy: kWh/m2/year by energy carrier and end use; peak demand; weather/occupancy normalized and raw series separately.
- Indoor environment: operative-temperature distribution, occupied comfort-exceedance hours, humidity, ventilation/CO2 where justified, and moisture events.
- Service: hours available/adequate, outage duration, restored-service evidence, repeat-fault rate and response/recovery time.
- Experience: response coverage, response bias, reported comfort/usability/accessibility/safety, consent state and group differences.
- Lifecycle: maintenance hours/cost, replacement, downtime, material/energy/water use, rework and recovery by grade/destination.
- Model quality: calibration residuals, independent validation, uncertainty coverage and model-envelope applicability.
- Rights and reliability: instrument calibration status, source freshness, missing interval coverage, consent and publication rights.

Do not convert a failed comfort floor or inaccessible service into an acceptable result through lower energy consumption or a lower cost.

## References

- CIBSE TM63 (active; published Aug 2020): https://www.cibse.org/knowledge-research/knowledge-portal/operational-performance-building-performance-modelling-and-calibration-for-evaluation-of-energy-in-use-tm63/
- CIBSE TM61–TM64 are complementary operational-performance memoranda; TM62 covers occupant satisfaction and TM64 indoor air quality: https://www.cibse.org/knowledge-research/knowledge-resources/engineering-guidance/cibse-technical-memoranda-tm/
- EVO IPMVP Core Concepts 2022; document access is registration-gated through EVO's library: https://www.evo-world.org/en/library/download-protocol-documents-mainmenu-en
- IEA listing for South African SANS 10400-XA:2021 states in force, national jurisdiction and new-building energy-use requirements (verify the current legally applicable edition with the competent authority): https://www.iea.org/policies/8412-sans-10400-xa2021-national-building-regulations-energy-usage-in-buildings
- Pereira et al. (2025), residential POE review: https://doi.org/10.1080/09613218.2025.2538167
- Building health systematic review (2026), 106 papers/six dimensions: https://doi.org/10.3390/buildings16112096
- State-of-the-art POE protocol review (2018): https://www.sciencedirect.com/science/article/pii/S0360132318300957

## Claim boundary

This profile classifies evidence compatibility and review eligibility only. It does not certify regulatory compliance, building safety, energy savings, health, comfort or causal impact. Those claims must be made by the appropriate technical/authority owner using the applicable method and independently reviewed evidence.
