# CityMaker digital product passport interoperability crosswalk v0.1

## 13. Digital product passport and current built-asset data interoperability

Construction-method and district comparisons should preserve product data identity across design, manufacturing, installation, use, maintenance and recovery. Keep the following external standards/legislation as versioned projections, never as internal truth or substitutes for local professional approval:

- IFC 4.3.2.0 / ISO 16739-1:2024 is the current official IFC 4.3 release; pin exact schema release, view/profile, serializer, source revision and artifact digest.
- IDS 1.0 is an official buildingSMART standard for machine-readable information delivery requirements. IDS pass means the specified fields/objects are present in an IFC exchange; it does not certify physical performance or safety.
- EN 15978:2026 is the current CEN method reference for life-cycle environmental performance of new and existing buildings and refurbishment, including design-for-circularity information. Retain declared unit, lifecycle modules, geography, data vintage, EPD validity, cut-offs and uncertainty.
- EU CPR 2024 (Regulation (EU) 2024/3110) became applicable from 8 January 2026, with product-family transition and coexistence of old/new rules. A product's actual obligations depend on the relevant harmonised technical specification and product-family transition.
- The European Commission DPP timeline states that the EU DPP Registry became operational on 20 July 2026 and that construction-product DPP requirements are scheduled for Q2 2027. The timeline is indicative. As of 7 November 2026, this is not a blanket construction-product passport mandate already in force.
- EU CPR/DPP is an export/interoperability reference only; it is not South African law. The applicable South African building-control, product, NHBRC, SANS, municipal and professional approval paths remain separate.

A DPP/QR/URI is an information carrier, not proof that the delivered batch matches, was inspected, is suitable for a project, is installed correctly or has been commissioned. Preserve external passport URI, issuer, product/batch identity, schema/legal revision, performance/conformity documents, material composition, EPD/LCA references, repair/disassembly data, role-based access, correction/revocation and digest. Bind it to the physical lot, custody, receiving inspection and installation evidence owned by the canonical systems.

References:
- European Commission DPP overview and indicative timeline: https://single-market-economy.ec.europa.eu/single-market/digital-product-passport_en
- European Commission CPR overview and transition: https://single-market-economy.ec.europa.eu/sectors/construction/construction-products-regulation-cpr/cpr-overview_en
- EUR-Lex Regulation (EU) 2024/3110: https://eur-lex.europa.eu/eli/reg/2024/3110/oj/eng
- CEN EN 15978:2026: https://www.cencenelec.eu/news-events/news/2026/en-in-the-spotlight/2026-04-17-en-15978-2026/
- buildingSMART IFC 4.3 and IDS 1.0: https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/ ; https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
