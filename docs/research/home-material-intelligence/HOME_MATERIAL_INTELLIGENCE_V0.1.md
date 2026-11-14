# Home material intelligence and substitution benchmark v0.1

**Study:** HOME-MATERIAL-INTELLIGENCE-001  
**Parent:** [Sol-Atlas #811](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/811) / Home Freedom Compiler #800  
**Maturity:** `RESEARCH_CONTRACT_WITH_SYNTHETIC_FIXTURE`  
**Execution:** `NOT_PHYSICALLY_EVALUATED`

## Research thesis

A design is not economically buildable merely because a generic material label has a unit rate. A component that appears dimensionally similar may change structural capacity, fire performance, moisture behavior, acoustic performance, thermal bridges, fixings, installation skills, inspection, warranty, maintenance and lifecycle impacts.

The system should search for alternatives when a supplier disappears or a quote changes, but must be willing to return "no qualified substitute". A cost optimizer ranks only options that first satisfy required performance and evidence constraints.

## Industry capability and standards

- **IFC 4.3.2.0 / ISO 16739-1:2024** is the official IFC release. Use IFC material/product relationships, layers and quantities where they fit. Do not invent a replacement BIM ontology.
- **IDS 1.0** provides computer-interpretable delivery requirements for checking IFC classifications, materials, properties and values. IDS checks information delivery; it does not prove technical equivalence or legal acceptance.
- **bSDD** provides versioned dictionary concepts. Preserve exact dictionary URIs and versions.
- **Agrément South Africa** evaluates non-standardised products, materials and systems against performance-based criteria and monitors quality systems after certification. Bind any real certificate to its exact current status, licensee, product/system revision, scope, intended use and conditions. A certificate discovery is not universal building approval.
- **Stats SA P0151.1** publishes construction input price indices. The September 2026 total construction index was 113.6 (Dec 2023=100) and increased 7.7% year-on-year. This is an escalation/sensitivity input—not a supplier quotation or direct unit price.
- **Digital Product Passport** is a useful interoperability pattern for origin, safety, material composition, repairability and circularity. EU requirements are not assumed to apply to South African projects.
- **DfMA / industrialized housing:** the U.S. GAO's 10 November 2026 assessment and DOE construction innovation programme identify off-site manufacture, modular/panelized methods, automation, materials innovation, training and code pathways as relevant levers. Industrialized delivery integrates planning, factory manufacture and onsite assembly. Prefabrication does not automatically save money: transport, handling, factory utilization, finance, design freeze, inspection, rework and warranty must be measured.

## Evidence contract

For each component preserve exact manufacturer/source owner, family, SKU, configuration, revision, intended use, declared unit, geometry/interface, IFC and bSDD references, source document, evidence class, publication date, digest and provenance.

Treat fire, structural, thermal, moisture, acoustic, durability, water, health, installation and maintenance as separate property claims. Each requires value or explicit UNKNOWN, unit, comparison direction, method/version, tested specimen, product revision, assembly scope, source/date/owner and uncertainty. Marketing text, an EPD or a product-family name is not a test report or certificate.

Certification records need authority, certificate ID, holder/licensee, product revision, covered assembly/configuration, conditions, issue/expiry dates, current status and verification receipt. A certificate only supports its own scope; it is not a substitute for all municipal or professional approvals.

Quotes are time-bound observations. Store supplier/site, product revision, quote revision/date/expiry, currency, unit rate/unit, quantity breaks, minimum order, VAT, delivery, stock, lead time, region, exclusions and uncertainty. Keep catalogue prices, written quotes, tendered prices, awarded rates and invoices separate. Keep Stats SA indices separate from each.

Circularity records should preserve EPD/LCA scope, recycled-content evidence, repairability, parts availability, disassembly, take-back, recovery, waste and end-of-life routes. Unknown content remains null/UNKNOWN, never zero.

## Substitution qualification

1. Confirm exact role, revision and intended use.
2. Confirm geometry, tolerance, substrate, interface and connection method.
3. Compare every critical property using compatible test methods, units and evidence scope.
4. Verify certificate/authority scope and current validity.
5. Verify regional supply, stock, lead-time and delivery.
6. Compare like-for-like quote basis: currency, VAT, freight, unit, quantity breaks and validity.
7. Re-evaluate the complete assembly, not just an individual component.
8. Invalidate dependent claims and request new owner evidence.
9. Leave professional/authority/client acceptance with the responsible decision-makers.

Outcomes: `QUALIFIED_IN_SCOPE`, `BLOCKED_UNKNOWN`, `REJECTED`, `NOT_COMPARABLE`. A scalar equivalence score must never override a failed critical property.

## Dependency invalidation

A component revision/substitution can invalidate BOM quantities, cost estimates, structural connections, fire strategy, thermal model, moisture/condensation, acoustics, certification scope, installation/warranty, maintenance/repair, EPD/LCA and construction sequence. Unrelated evidence may be preserved only where exact scope and revision binding prove it is unaffected.

## Benchmark protocol

**Functional unit:** one installed m², linear metre, unit or assembly at a named design revision, including materials, fasteners, labour, handling, waste and inspection. Also report dwelling-level roll-up without hiding component-level provenance.

Compare matched manual architect + quantity-surveyor + supplier quotation workflows, BIM product libraries and quantity takeoffs, IFC/IDS/bSDD information delivery, offsite/panelized DfMA and conventional onsite construction. Freeze product/tool version, quote date, geography, unit scope, freight, installation labour, factory overhead, handling, inspection, rework and warranty boundaries. Do not compare catalogue price with installed assembly rate.

### P0 acceptance targets

| Gate | Proposed target |
|---|---:|
| Expired/mismatched/out-of-scope critical evidence promoted to PASS | 0 |
| Price indices treated as direct unit prices | 0 |
| Silent currency/unit/VAT/freight/quantity-break mismatch | 0 |
| Seeded critical property mismatches detected | 100% |
| Revision changes invalidate expected dependent claims | 100% |
| Independent known-answer agreement on frozen cases | 100% |
| Positive controls preserved | 100% |
| Automated purchasing or substitution authority | 0 |

Measure quote freshness, price coverage, quote-to-award variance, installed cost per functional unit, stock/lead-time uncertainty, substitution and rejection rates, qualification time, supplier concentration, waste, rework, factory/site labour, installation first-time-right, defects, maintenance, spare-part availability, lifecycle cost and embodied carbon. Establish numerical improvement targets after a measured local baseline exists.

## Synthetic fixture

The machine-readable fixture contains a baseline panel, a compatible positive-control alternative, an alternative with a critical UNKNOWN fire property, and one with a known synthetic fire shortfall. It also contains a synthetic wall assembly, fictional supplier quotes and one genuine Stats SA public index entry typed as escalation-only.

The synthetic control outcomes exercise qualification logic only; they do not assert physical product equivalence. No supplier, certificate, stock state, product performance or quote is real.

## Claim ceiling

No real product certification, supplier availability, live quote, structural/fire equivalence, regulatory acceptance, environmental benefit, procurement saving or construction performance is established. Real use requires verification against source-native evidence, exact product revision, intended-use scope and qualified professional/authority decisions.

## Research references

- Agrément South Africa: https://agrement.co.za/certification-process/ and https://agrement.co.za/active-certificates/
- Stats SA P0151.1, September 2026: https://www.statssa.gov.za/publications/P01511/P01511September2026.pdf
- IFC 4.3: https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/
- IDS: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- bSDD in IDS and IFC: https://technical.buildingsmart.org/services/bsdd/referencing-bsdd-in-ids-and-ifc/
- EU Digital Product Passport: https://single-market-economy.ec.europa.eu/single-market/digital-product-passport_en
- U.S. GAO GAO-27-108037 (10 Nov 2026): https://www.gao.gov/products/gao-27-108037
- DOE Construction Technology and Innovation Program: https://www.energy.gov/cmei/buildings/building-construction-technology-and-innovation-program
- National Housing Crisis Task Force, Industrialized Housing Delivery: https://nationalhousingcrisis.org/toolkit/regulation-policy/industrialized-housing-delivery-in-the-u-s-recommendations-for-state-and-local-governments/
