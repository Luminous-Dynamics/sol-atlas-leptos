# Cost-of-place and whole-household affordability benchmark v0.1

**Study:** HOME-COST-OF-PLACE-001  
**Parent:** [Sol-Atlas #816](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/816)  
**Maturity:** `RESEARCH_CONTRACT_WITH_SYNTHETIC_FIXTURE`  
**Execution:** `NOT_EVALUATED`

## Why construction price is insufficient

A home can be inexpensive to build and expensive to inhabit. A lower land price may be offset by bulk service connection, unreliable water, higher transport cost, longer commute, maintenance, insurance or insecure tenure. A low monthly loan payment does not prove whole-household affordability.

The compiler should compare a resident's housing options over a declared horizon, preserving one-time and recurring costs separately, and showing both cash burden and delivered services. It must not infer loan eligibility, target households by protected traits or issue financial recommendations.

## South African evidence context

- Stats SA's **General Household Survey 2025** reports 87.4% of households had access to piped water in the dwelling, yard or communal tap and 84.0% had improved sanitation. National access does not establish service reliability, in-house supply, quality, affordability or a particular parcel's connection capacity.
- The **South Africa National Urban Agenda report 2026** reports housing and transport together account for more than half of household expenditure using 2022 data. For decile 1 it reports housing/utilities at 38.1% of income and transport at 6.3%; those are historic national distribution estimates, not current costs for a specific family or site.
- A South African Government speech in November 2026 emphasizes that land becomes a housing opportunity only when essential services are available and can be sustained, and connects serviced land/location/transport with cost of living.
- National Treasury's 2026 Estimates of National Expenditure describe bulk infrastructure and serviced-site programmes. Programme targets and funding are not proof that a particular site's services are available or funded.
- Stats SA's Income and Expenditure Survey 2022/23 provides income/expenditure distributions. Use median and distributional baselines with their survey period; do not use averages as a representative household.

## Whole-household model boundary

Report separately:
- land purchase/lease, title, deposit, tenure, rates, levies and common-property obligations;
- bulk service capacity, connection fee, connection delays, reliability, outages, quality and alternative service cost;
- design/professional fees, permits, construction, tax, contingency, finance during construction and commissioning;
- deposit, finance principal, interest, fees, term and reset/refinance stress assumptions;
- monthly utilities, insurance, maintenance, replacement, repairs and backup systems;
- commuting costs/time and access to work, education, healthcare, food, care and transport;
- relocation, displacement and tenure-security exposure;
- cost and service continuity for future phases/adaptations.

One-time and recurring flows should not be collapsed without explicit conversion and a disclosed time horizon/discount method. Preserve currency, price date, source, unit, tax, geography, owner and uncertainty on every input.

## No scalar affordability verdict

The model returns a transparent distribution and a Pareto set. It does not rank households or infer what they should accept. A cheap site must not compensate for absent water, sanitation, lawful tenure, safe occupancy or workable access to opportunities. A low monthly payment must not hide a large deposit, connection charge, transport burden, repair liability or cost escalation risk.

Tenure models—owner-occupied, rental, cooperative/shared equity, rent-to-own and serviced-site/incremental—have different rights and risks and must be compared with separate contracts. They cannot be treated as interchangeable versions of a mortgage.

## Benchmark and claim ceiling

Compare quantity-surveyor/tender estimates, site/pro-forma tools, lender payment calculators (narrowly, as finance baselines), public housing-programme criteria, manual household lifecycle workbooks and the Luminous evidence composition. Freeze site, date, household frame, service floors, tenure, tax, finance, horizon and access requirements.

Proposed P0 hard gates: 100% cost provenance; zero unknown-to-zero coercion; zero unverified utility capacity promoted to service; zero historic index used as current tariff/quote; zero site claims inferred from national averages; 100% known-answer agreement and positive-control preservation; no credit/subsidy/tenancy action.

The synthetic fixture contains three distinct options but no real income, site, title, quote, tariff, finance, transport or service evidence. It deliberately leaves all costs and service states unknown. No affordability, loan eligibility, subsidy eligibility, service readiness or savings result is represented.

## Research references

- Stats SA, General Household Survey 2025: https://www.statssa.gov.za/?PPN=P0318&SCH=74563&page_id=1856
- South Africa National Urban Agenda Report 2026: https://www.urbanagendaplatform.org/sites/default/files/2026-04/South-Africa-National-Report-2026.pdf
- South African Government, serviced land and municipal infrastructure (Nov 2026): https://www.gov.za/news/speeches/minister-thembi-simelane-distinction-dialogue-strengthening-municipal-infrastructure
- National Treasury, 2026 Estimates of National Expenditure: https://www.treasury.gov.za/documents/National%20Budget/2026/ene/FullENE.pdf
- Stats SA, Income & Expenditure Survey 2022/23: https://www.statssa.gov.za/?p=17995

No individual affordability, credit, legal, investment, subsidy, real-site or household-service claim is established.
