# Home DfMA delivery benchmark v0.1

**Study:** HOME-DfMA-DELIVERY-001  
**Parent:** [Sol-Atlas #814](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/814)  
**Maturity:** `RESEARCH_CONTRACT_WITH_SYNTHETIC_CASE`  
**Execution:** `NOT_EXECUTED`

## 1. Research question

Can resident-directed homes retain architectural variety while being delivered through repeatable local component and assembly systems—without moving cost, risk or rework from the building site into a factory, transport operation or lifting plan?

The benchmark compares conventional site-built, locally panelized, volumetric modular and hybrid component-kit delivery using the same frozen design, BOM, service requirements, site and quality plan.

## 2. Paradigm shift: manufacturing-aware design with change-safe release

A design should carry a production and installation contract:
- exact design, assembly, component and interface revisions;
- fabrication routing, tooling and work instructions;
- datums, tolerances, stack-up analysis and calibrated measurement;
- incoming, in-process and final quality hold points;
- material batch/lot and product genealogy;
- packaging, storage, dimensions, mass, centre of gravity and lift points;
- transport route, clearance, offload, crane/handling and site laydown;
- crew skill, temporary works, installation order and commissioning;
- post-install inspection, as-built deviations, maintenance and repair.

The design must remain editable before the safety-critical release boundary. After release, a change must identify its impact on stock, work-in-progress, fabrication, QA, certification, quotes, packaging, route, lifting, installation, inspection and warranty. A changed drawing must not silently inherit receipts from the prior revision.

## 3. Delivery modes and matched boundary

1. **Conventional site-built:** include procurement, delivery, storage, site labour, plant, temporary works, waste, inspection and rework.
2. **Local panelized:** include DfMA engineering, factory labour, equipment, setup, overhead, QA, packaging, delivery, offload, lift, site closure and inspection.
3. **Volumetric modular:** include module manufacture and fit-out, transport envelope, route/permit costs, crane, module joints, services, foundations/interfaces, commissioning and warranty.
4. **Hybrid component kit:** include both factory and site labour, multiple handoffs, interface tolerances, logistics, sequencing and temporary protection.

The functional unit is one dwelling design revision and its declared assembly/BOM built to identical service and quality requirements. Do not compare factory gate costs with complete installed dwelling costs. Freeze all comparator scope, quality, labour, tax, transport, handling and inspection boundaries.

## 4. Current industry position

- The U.S. GAO's November 10, 2026 technology assessment identifies planning/design tools, construction means/methods and materials/components as housing-technology categories. It describes possible productivity and waste benefits from controlled off-site production, while also identifying code fragmentation, workforce gaps and high initial capital costs as adoption barriers.
- DOE's Building Construction Technology and Innovation Program supports research around modular/panelized construction, design/construction automation, materials innovation, code processes and workforce development.
- The National Housing Crisis Task Force describes industrialized housing delivery as integrated planning, factory manufacture and onsite assembly—not merely the purchase of a prefabricated product.
- IFC 4.3 and IDS provide the open exchange baseline for geometry, assets, materials, quantities and information requirements. They do not prove fabrication readiness, tolerances or crane/transport feasibility.
- Agrément South Africa offers an evidence source for non-standardised building systems and products, but its exact certificate scope does not replace site-specific design or regulatory approval.

The industry already has strong specialist design, BIM, manufacturing and construction tools. The potential Luminous contribution is a resident-directed evidence bridge connecting design revisions to manufacturing, batch/lot QA, transport, installation and in-use repair—without duplicating specialist engines.

## 5. Benchmark measurements

Report all outcomes separately:

| Domain | Measurements |
|---|---|
| Cost | Total installed ZAR per functional unit; factory, site, transport, crane, temporary works, inspection and rework separated |
| Time | End-to-end p50/p95 days; factory queue, fabrication, transport, site install and commissioning windows |
| Labour | Factory and site person-hours by trade/skill; setup and supervision hours |
| Quality | First-time-right, tolerance deviations, failed inspections, defects and warranty returns |
| Materials | Yield, scrap, damage, waste mass and reuse/recovery route |
| Logistics | Transport envelope, route feasibility, payload, handling, lift plan, crane time and site storage |
| Change | Engineering change order latency/cost; work-in-progress obsolescence; invalidation completeness |
| Resilience | Supplier failure recovery, alternate-source qualification time, weather delay and repairability |
| Resident service | Occupancy disruption, unserved essential-service hours, safe phase transitions and future adaptation cost |

Do not set arbitrary numerical improvement targets for cost or schedule before a measured local baseline exists. Publish the full cost/time boundary and all negative results.

## 6. P0 qualification gates

- Every production item binds to the exact design, assembly and component revision.
- No panel/module is released without the required QA/tolerance records.
- No transport/lift feasibility claim without dimensions, mass, centre of gravity, lifting points, route/site envelope and authorized plan.
- No occupied phase passes with a critical safety or essential-service floor breached.
- Every seeded post-release design change invalidates the correct downstream claims.
- An independent known-answer oracle agrees on route, tolerance, quantity, sequence and change-impact controls.
- Positive controls remain admissible; an always-blocked plan does not qualify.
- The qualifier authorizes no purchases, fabrication, lifting, installation or permits.

## 7. Synthetic case

The companion case enumerates the four delivery modes against one synthetic design revision. It deliberately leaves factory capacity, route clearance, component mass, crane/lift plan, labour, cost, duration, QA and batch genealogy unknown. All options remain `NOT_EVALUATED` and the design remains `NOT_RELEASED`.

This is a positive test for representing delivery hypotheses without manufacturing a false transport, cost or safety result.

## 8. Research references

- U.S. GAO, *Housing Construction Innovations: Technologies and Policy Options to Help Increase Supply and Affordability*, GAO-27-108037, 10 Nov 2026: https://www.gao.gov/products/gao-27-108037
- DOE Building Construction Technology and Innovation Program: https://www.energy.gov/cmei/buildings/building-construction-technology-and-innovation-program
- National Housing Crisis Task Force, *Industrialized Housing Delivery*: https://nationalhousingcrisis.org/toolkit/regulation-policy/industrialized-housing-delivery-in-the-u-s-recommendations-for-state-and-local-governments/
- IFC 4.3: https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/
- IDS: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- Agrément South Africa: https://agrement.co.za/certification-process/

## Claim ceiling

No factory capability, local supplier capacity, structural design, route, lift, permit, schedule, installed cost, quality improvement or constructed dwelling is represented. No physical DfMA benefit or superiority is claimed.
