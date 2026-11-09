# South African Mine-Water Thermal Service Benchmark v0.1

**Status:** research and site-screening contract, 9 November 2026. No mine has been declared thermally suitable and no abstraction/reinjection approval is implied.

Related issue: [SA-MINE-WATER-THERMAL-SERVICE-001 #690](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/690). Parent portfolio owner: [ENERGY-GEN-PORTFOLIO-001 #293](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/293).

## 1. Why this is a better first geothermal/thermal use case

South African geothermal research to date identifies possible low-enthalpy anomalies and explicitly calls for more temperature, structural, hydrogeological and economic work. It does not establish an electricity-grade geothermal reservoir.

Mine-water thermal systems provide a different route: use existing water-management infrastructure as a low-grade thermal source/sink; exchange heat through a controlled secondary circuit; use heat pumps only where the required temperature lift warrants their compressor work; and carry heat/cold through storage or a network to a real consumer.

This is **useful heat/cooling infrastructure**, not geothermal electricity generation. It can still be a major energy-system improvement because industrial heat and cooling demand may be met without first converting all input energy into electricity.

## 2. Evidence currently available

| Evidence | What it supports | What it does not prove |
|---|---|---|
| Council for Geoscience mine-water management project | South African mine-water ingress, pump-and-treat, acid-mine-drainage and water-management context | Stable thermal flow, water temperature, thermal capacity or a consumer connection |
| GreenCape industrial heating brief (28 Aug 2026) | South African process-heat and steam decarbonization context | Site-specific process duty or LCOH |
| CSIR/SANEDI thermal systems support | Local thermal test and R&D capabilities, including industrial solar heat, heat pumps and thermal storage work | That every listed planned rig is already commissioned or that a product has passed qualification |
| Oppelt et al. (2025) mine-water review | Flow renewal, temperature, pumping head, consumer proximity, reinjection and chemistry are critical | A South African mine's measured thermal output |
| 2026 operating mine-water heat-supply case | Heat-network losses and exchanger mineralization can dominate optimization; 14–22-year payback in one case's scenarios | A transferable South African payback |
| Freiberg mine-water seasonal storage pilot | More than two years of pilot operation, reported 45–65% storage efficiency and 2–40°C mine-water temperature in the German case | 45–65% as a universal storage efficiency or South African district self-sufficiency |

The source register preserves these records with separate evidence classes: South African government/project context, policy report, international literature review and published field/pilot observation. All remain context-only; no raw data are ingested.

## 3. Physical architecture to compare

1. **Direct heat exchange:** possible only where the source temperature and the consumer's required temperature make direct transfer technically useful.
2. **Heat pump:** mine water supplies source heat; compressor electricity raises the temperature grade. Report source heat, compressor work and auxiliaries separately.
3. **Heat pump plus thermal storage:** time-shift production away from peak electricity periods or match intermittent industrial waste heat and solar thermal.
4. **Heat pump plus distribution network:** compare centralized and decentralized heat-pump placement against network distance, pumping and heat losses.
5. **Hybrid heat source:** add industrial waste heat, wastewater or solar thermal as separate, metered source ports; do not double-count the same energy stream.

The clean customer loop should be isolated from mine water through a suitably engineered heat-exchanger boundary unless site-specific materials, chemistry, water quality, corrosion and approvals justify otherwise. The source-side mine-water management and remediation obligations remain primary.

## 4. The variables that determine feasibility

### Mine-water source

- stable inlet/outlet flow and renewal/replenishment chronology;
- temperatures at multiple mine levels and seasons;
- source drawdown and thermal recovery;
- hydraulic head and pump/treatment electricity;
- pH, salinity, hardness, dissolved metals, suspended solids and scaling/fouling;
- legal ownership, abstraction/reinjection/discharge rights and treatment requirements.

### Consumer and network

- hourly/seasonal useful heat and cooling demand;
- minimum and maximum supply/return temperatures;
- production continuity and acceptable service interruption;
- source-to-consumer distance, elevation, pipe routing and insulation;
- heat-network loss, circulation energy, maintenance access and backup system.

### Conversion and storage

- heat-pump compressor maps over the actual temperature lift and part-load range;
- COP at a declared boundary and whole-system COP including circulation and treatment;
- thermal-store state, temperature grade, capacity, standing loss and cycle efficiency;
- storage/source interaction, seasonal recovery and thermal stratification;
- emergency bypass and service recovery.

A mine's water volume alone is not a thermal-resource measure. The usable potential depends on flow renewal, temperature, pump head, consumer proximity and permitted reinjection/discharge temperature.

## 5. Matched comparison set

For each candidate site and identical load chronology, compare:

- existing boiler/chiller or present supply arrangement;
- no-build / efficiency-only / heat-recovery-only;
- direct heat exchange (where feasible);
- heat pump alone;
- heat pump + thermal storage;
- heat pump + waste-heat recovery;
- solar thermal + thermal storage + heat-pump top-up;
- centralized district/industrial network versus decentralized heat pumps.

Use existing Sol-Atlas evaluators:
- #295 for pathway conservation and auxiliaries;
- #296 for heat-pump source heat, compressor work and Carnot check;
- #297 for chronological thermal/electrical storage state and losses;
- #300 for temperature-grade exergy;
- #301 for multi-output port graph;
- #303 for useful service at the required temperature;
- #307 for chronological service reliability and unserved heat/cooling;
- #309 for bounded exact small-instance dispatch/Pareto comparison.

Do not implement a second heat-pump or thermal-store kernel in this use-case branch.

## 6. Scorecard — preserve the denominators

| Dimension | Required metric |
|---|---|
| Useful heat | MWh-thermal delivered at each required temperature grade |
| Useful cooling | MWh cooling delivered by interval and supply/return conditions |
| Source extraction | source heat extracted/rejected, flow and thermal recovery |
| Heat pump | compressor electricity, auxiliaries, COP and whole-system COP |
| Network | delivered/source heat ratio, circulation energy, distribution temperature and loss |
| Storage | charge/discharge energy, usable capacity, duration, standing loss, temperature grade and state |
| Water management | pumping/treatment energy, water chemistry, reinjection/discharge compliance and service impact |
| Reliability | service availability, unserved demand, longest interruption, recovery time and backup operation |
| Economics | LCOH / levelized cooling cost, CAPEX, OPEX, replacement, finance and tariff sensitivity |
| Environment | water impacts, treatment chemicals, materials, refrigerant leakage, carbon and remediation compatibility |

COP alone is not a product result. Report source-side pumping and treatment electricity as well as compressor and distribution energy. A high heat-pump COP cannot compensate for an uneconomic mine-water pumping or long heat network.

## 7. First negative controls

- no source-flow record → no assumed thermal supply;
- no source renewal/recovery evidence → no sustainable seasonal extraction claim;
- temperature lift beyond a measured device envelope → no COP/power credit;
- missing compressor electricity or auxiliary load → no whole-system COP;
- heat below the consumer's minimum temperature → no useful heat service;
- network losses omitted or set to zero by default → reject cost/service result;
- mine-water chemistry and fouling evidence missing → no exchanger lifetime or maintenance claim;
- no rights/approval evidence → no project feasibility or construction authority;
- thermal storage state/chronology gap → no exact dispatch result;
- any remediation, containment, water-quality or environmental service breach → candidate is infeasible regardless of energy value.

## 8. Next evidence gate

The first deliverable is a **candidate pair shortlist** (mine-water zone + consumer cluster), not a plant design. Every candidate needs operator/authority data access and measurement planning before engineering selection.

For the first shortlisted case:
1. pin original water and consumer data with timebase and uncertainty;
2. establish pumping/treatment and heat-system baselines;
3. construct the smallest source-to-service model using #295–#309;
4. compare direct exchange, heat pump, heat pump + storage and no-build;
5. run the Symtropy time-series twin and independently re-audit first law, exergy, thermal-store chronology and service delivery;
6. request CSIR/SANEDI or qualified laboratory collaboration for test and instrumentation planning;
7. retain unresolved source, chemistry, rights, cost and maintenance questions as explicit unknowns.

## 9. Claim ceiling

A model or international pilot reproduction does not establish mine-water rights, reservoir sustainability, safe reinjection, thermal-network constructability, water quality, local LCOH or environmental approval. Field sampling, pumping, heat-exchanger installation and reinjection require competent licensed professionals, mine/water operator authorization, environmental review and independent acceptance.
