# Spaceport Geography and Earth-to-Space Logistics Benchmark v0.1

**Status:** research specification; synthetic-first; not a site approval or launch-system design
**Owner:** Sol-Atlas spatial opportunity / infrastructure Foundry
**Tracking:** [SPACEPORT-GEOGRAPHY-001 #393](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/393)
**Composition:** [terrestrial-to-cislunar explorer #106](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/106), [Port-City Foundry #370](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/370), Symtropy [ORBITAL-ACCESS-001 #3202](https://github.com/Luminous-Dynamics/symtropy/issues/3202), [FREIGHT-MAGLEV-001 #3201](https://github.com/Luminous-Dynamics/symtropy/issues/3201)
**Version:** 0.1.0

## 1. Executive statement

The decision object is a **spaceport ecosystem**, not a point with a low absolute latitude. A viable ecosystem can include geographically separate assets: launch pad and range, vehicle/payload integration, manufacturing, storage and handling, tracking and communications stations, emergency response, workforce/training, energy and water supply, and port/rail/road connections. These assets may need separation because public safety, downrange trajectory, environment, land rights and urban form can conflict with heavy freight logistics.

The question is: which architecture-specific Earth-side ecosystem provides safe and useful access to demanded orbital services, resilient logistics and durable local value, compared with existing launch infrastructure and other credible alternatives?

Do not assume every spaceport should be equatorial, an equatorial site serves every orbit, a cargo port and launch pad should share a footprint, or a space elevator, orbital ring, skyhook, partial ring and launch loop share the same geometry.

## 2. State of the art and location constraints

### Equatorial launch advantage is mission-dependent

NASA's launch-site primer explains that an equatorial launch can take advantage of Earth's rotational velocity, stated as more than 1,650 km/h at the equator. The benefit depends on launch direction and target orbit. High-inclination missions do not receive the same advantage, and the downrange path must avoid populated areas. Source: https://science.nasa.gov/learn/basics-of-space-flight/chapter14-1/

Therefore, evaluate the target orbit and launch azimuth before giving a site credit for latitude. A polar or sun-synchronous launch programme may rationally prefer a higher latitude and a different downrange corridor.

A further caution: once sites are already close to the equator, latitude differences alone become a weak discriminator. Using a spherical reference model with Earth's rotation rate 7.2921150e-5 rad/s and equatorial radius 6,378,137 m, eastward surface speed is about 464.66 m/s at 2.5° latitude and 463.33 m/s at 5.0° latitude, a difference of only about 1.33 m/s. The 2.5° value is only an illustrative regional coordinate, not a surveyed Kipini launch-site coordinate. This difference is tiny relative to orbital speed and can be overwhelmed by target-orbit, vehicle, range, weather, logistics, ecological, safety and governance factors. Do not claim a meaningful payload advantage between two near-equatorial sites from latitude alone.

### Public safety and range access

The US FAA identifies access to useful orbits and public safety as primary spaceport-location considerations. Its FAQ notes that sites are normally away from major population centres and commonly near water to reduce public risk in a launch failure. This is a useful comparator, not a substitute for the applicable national licensing regime, range-safety rules, environmental law, airspace/maritime coordination or consultation. Source: https://www.faa.gov/space/additional_information/faq

### Reference sites, not automatic winners

- **Kourou, French Guiana:** mature operational reference in the equatorial region, approximately 5° north. ESA describes its location as advantageous for GEO transfer trajectories. https://www.esa.int/Enabling_Support/Space_Transportation/Europe_s_Spaceport/Europe_s_Spaceport2
- **Alcântara, Brazil:** Brazilian Space Agency material places the launch centre at approximately 2°18′ south and describes launch/tracking functions. https://www.gov.br/aeb/pt-br/acoes-e-programas/aplicacoes-espaciais/infraestrutura-de-solo/centros-de-lancamento
- **Kipini/Kenya proposal:** on 1 October 2026, Kenya's PPP Directorate announced funding for a feasibility study for a proposed spaceport. A 28 October 2026 report by Space in Africa says the transaction-advisory consortium has since been appointed and identifies the proposed general area as Kipini, between Malindi and Lamu; the study is to assess sites, launch infrastructure, demand, environmental/social impacts and financial/economic viability. This remains a feasibility exercise, not a build decision or proof of bankability. https://pppkenya.go.ke/kenya-advances-plans-for-proposed-spaceport-project/ ; https://spaceinafrica.com/2026/10/28/kenya-appoints-consortium-to-advise-on-proposed-commercial-spaceport/
- **Biak/Indonesia planning-stage proposal:** on 2 July 2026, Indonesia's BRIN reported early discussions involving URPI and Glavkosmos on possible Biak spaceport development, following the signing of a national spaceport regulation. A May 2026 Jakarta Post report described Biak's near-equatorial location and Pacific downrange access as motivations and reported international company interest. Treat this as a government/industry planning reference, not an operational launch range, final site approval, finance close or proven payload-performance claim. Do not import promotional percentage estimates for payload advantage without independent orbit/vehicle calculations. https://www.brin.go.id/en/news/129109/didukung-regulasi-baru-brin-dan-glavkosmos-bahas-langkah-awal-pengembangan-bandar-antariksa ; https://www.thejakartapost.com/business/2026/05/15/russia-india-show-interest-in-indonesias-biak-spaceport-project/

Use these as reference cases only. Verify current operating status, supported mission/orbit envelope, safety corridors, data currency and source rights before admitting a site to a benchmark. Follow Kipini and Biak as separate live project-evidence streams; do not assume either broad regional location is a final surveyed site or that either public announcement establishes a recommended launch architecture.

### Kipini/Tana Delta: mandatory avoidance-first screen

The general Kipini study area sits within the wider Tana Delta landscape. This is a material environmental and climate-screening issue, not a minor score penalty:

- The Ramsar Sites Information Service lists the Tana River Delta as site 2082, 163,600 ha, designated 9 July 2012, with freshwater, floodplain, estuarine and coastal habitats, mangroves and important threatened-species/migratory-bird functions. https://rsis.ramsar.org/sites/default/files/rsiswp_search/exports/Ramsar-Sites-annotated-summary-Kenya.pdf
- The confirmed Tana River Delta Key Biodiversity Area is 1,576.2 km², with elevation 0–37 m and 85.6% protected-area coverage. Its description says the low-lying delta coastal base extends from Kipini toward Mto Kilifi. https://www.keybiodiversityareas.org/en/site/factsheet/6412
- Tana River County's Participatory Climate Risk Assessment identifies Kipini East and West among wards affected by flooding and seawater intrusion and describes coastal erosion, wetland/freshwater-ecosystem loss and livelihood impacts. https://maarifa.cog.go.ke/sites/default/files/2024-12/Tana%20River%20County%20PCRA%20Report%20DEC%202024%283%29.pdf
- UNESCO IHP-WINS publishes a Tana River Basin historical-inundation dataset based on 1999–2020 satellite observations. https://ihp-wins.unesco.org/en/dataset/flood-risk

The actual future project footprint is not established by the general place name Kipini. Do not assert exact Ramsar-polygon overlap until official current boundaries and actual candidate polygons have been intersected. Nevertheless, the wider-area evidence requires early avoidance/alternatives analysis before any latitude, port-access or financial score. Track this explicitly in [KIPINI-SPACEPORT-SCREEN-001 #395](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/395). A separate manufacturing, integration, tracking or logistics site outside the sensitive delta may have a different impact profile from a launch range and must be evaluated as a distinct alternative.

## 3. System boundary: ecosystem, not pin

Represent distinct but connectable asset classes:

1. **Launch range:** pad/vehicle integration perimeter, range control, launch azimuth, flight-safety corridors, telemetry and recovery/response assets.
2. **Manufacturing and integration ecosystem:** component manufacturing, clean assembly, test, quality, suppliers, workforce, training and research.
3. **Logistics gateway:** seaport, airport, rail terminal, road freight hub, customs/bonded facilities and secure cargo handling.
4. **Utilities:** electrical supply, storage, water, communications, emergency services, housing and health services.
5. **Tracking and communications network:** geographically distributed ground stations and downlink capacity. Communications connectivity is not physical cargo connectivity.
6. **Environmental and social geography:** protected areas, sensitive habitats, fisheries, cultural heritage, flood/storm exposure, land/water rights, affected communities and jurisdiction.
7. **Future orbital infrastructure terminals:** possible elevator Earth ports, orbital-ring skyhook/ladder interfaces, partial-ring endpoints or launch-loop terminals. Each has its own architecture and geometry.

A port may import heavy components to an inland integration hub, while the launch range occupies a separate lower-density site. Connect the facilities with a sourced, capacity-bounded transport leg; do not create one giant co-located node just to shorten a straight line. Kourou is a concrete reference for this distributed pattern: the Vega C User Manual describes a sea-arrival/docking area at Pariacabo on the Kourou River, linked by road to the Space Centre (about 9 km), alongside separate payload-preparation facilities and dedicated launch sites. This is a useful logistics/functional-separation case, not a universal layout template. Source: https://www.avio.com/sites/avio.com/files/downloads/UM%20Vega%20C%20-%20Iss1%20Rev0%20Erratum_1.pdf

## 4. Architecture-specific location logic

Never use one universal best-launch-or-tether latitude score.

| Architecture | Governing location geometry | Boundary |
|---|---|---|
| Chemical/reusable rocket | Target orbit, launch azimuth, inclination, Earth rotation, downrange safety and range availability | Pad and range corridor |
| Air-launch | Aircraft base, airspace, release envelope, weather and target orbit | Aircraft operations and release corridor |
| Classical geostationary space elevator | Equatorial Earth terminal, GEO node, counterweight beyond GEO, tether strength and dynamics | Tether/Earth-port system |
| Full orbital-ring system | Selected orbital plane/altitude, ring dynamics, skyhook placement, precession and ladder/terminal geometry | Ring/skyhook-specific interface |
| Partial orbital-ring system | Coupled orbital arc, paired endpoints, moving cable/mass-stream and ground terminal dynamics | Both endpoints jointly |
| Orbital skyhook | Orbit, tether length, rotation/velocity, release/capture state and payload transfer | Moving orbital asset, not fixed ground cable |
| Launch loop | Ground endpoint pair, high-speed moving mass stream, guideway/sheath and safety corridor | Ground-supported launcher, not orbital ring |

### Elevator versus orbital ring

A classical geostationary elevator depends on Earth's rotation and a tether/counterweight system; its conventional surface terminal is equatorial. It faces extreme materials, deployment, dynamics and collision-protection challenges. A 2023 review of tether-material candidates reports that no candidate material yet exists in sufficient quantity for the required tether, despite laboratory samples and work on carbon nanotubes, graphene-based materials and hexagonal boron nitride. It describes a roughly 100,000 km tether manufacturing/continuity challenge. This is a separate material-industrialization gate; it must not be inferred solved by a high tensile-strength laboratory coupon. Source: https://doi.org/10.1016/j.actaastro.2023.04.008

Paul Birch's 1982 Journal of the British Interplanetary Society paper describes a different orbital-ring concept using a moving low-Earth-orbit ring and skyhooks/Jacob's Ladders. Its preliminary analysis discusses orbital-plane choices beyond the equatorial plane and partial-ring variants with ground endpoints. This is a concept source, not a validated design or demonstration of construction feasibility. Source: https://nss.org/wp-content/uploads/Orbital-Rings.pdf

**Implication:** do not force an orbital-ring terminal search to the equator because a classical elevator anchor is equatorial. Symtropy #3202 owns the orbital dynamics/architecture benchmark. Sol-Atlas consumes a qualified terminal/access envelope as an evidence-bearing projection.

### Avoid technology conflation

A short MagRail freight link is not a launch loop. Linear-motor propulsion is not necessarily levitation. An orbital skyhook is not a stationary tether terminal. An orbital ring is not a space elevator with a different name. Every candidate declares its architecture and model-validity envelope.

## 5. Earth-side opportunity search

### Orbital-service demand and access

Record a target-orbit portfolio, not one generic orbit: inclination and altitude/regime; launch azimuth/release direction and allowed corridor; payload mass and cadence; time windows and seasonal/weather availability; mission classes such as GEO transfer, low-inclination LEO, polar/sun-synchronous, crew/cargo inclination classes, lunar injection, deep-space escape and suborbital/test; recovery/reuse and payload-integration service requirements.

Use vehicle performance data only within its stated vehicle, launch-site, payload and mission envelope. Do not apply a generic latitude-savings factor.

### Terrain, geotechnical, coastal and climate context

Capture elevation, slope, geomorphology, terrain roughness, drainage, geology, faults, seismicity, landslide/liquefaction potential, coastal erosion, storm surge, cyclone/wind, sea-level rise, flooding and tsunami exposure. Add winds aloft and surface winds, lightning, precipitation, cloud, humidity, temperature extremes, salt/corrosion exposure and visibility. Include protected ecology, marine/freshwater systems, groundwater and freshwater supply.

Global/coarse datasets are screening context only. Preserve datum, CRS, resolution, epoch, uncertainty, source type, coverage and licence. A coarse raster cannot establish pad foundation conditions or a range-safety assessment.

### Range, airspace and maritime safety

Represent launch azimuth families and target-orbit compatibility; population/settlement density and overflight exposure; land/ocean downrange corridors and emergency response envelopes; airspace, maritime traffic and fishing grounds; exclusion zones and their effect on local livelihoods and rights; telemetry coverage, tracking handovers and lost-link response boundaries. Unknown corridors or missing population/traffic coverage are not empty space.

### Industry, logistics and local value

Measure component mass/volume, port handling capability, secure storage and customs; road/rail/port service capacity, reliability and seasonality; manufacturing, assembly, test, training, maintenance and research capacity; supplier concentration, imported content, skills transfer and local value-add; electricity, storage, water, fibre and emergency services; housing, healthcare, schools and worker access; and lifecycle cost of new utilities/logistics infrastructure.

A nearby port is an advantage only if the required components can be safely handled and carried on a verified route at required dimensions and service frequency.

## 6. Multi-ledger appraisal and hard gates

Keep separate ledgers for private project cash flows/bankability; public fiscal exposure and contingent liabilities; economic resource costs/benefits including time/reliability; ecological/environmental effects; distributional impacts and durable local value; and resilience, lifecycle replacement and real-option value.

No weighted economic score can buy through legal/rights, range safety, protected ecology or engineering feasibility gates. Missing evidence yields an explicit unknown/needs-evidence state, not safe-by-default. Report robust Pareto alternatives. If a scalar preference is used for a policy decision, publish its weights, sensitivity envelope and rank reversals.

## 7. Connection to ground transport and port-city infrastructure

Compare existing port/airport/rail improvements; conventional road and freight rail; MagRail/linear-motor systems on existing rail where supported; dedicated underground freight systems; new road/rail/tunnel links; reuse of existing tunnels/brownfields where verified; and no-build/phased demand. In South Africa, Transnet describes ContainerCor as a 688 km Durban–Gauteng rail corridor serving inland terminals including City Deep, Kascon, Pretcon and Kaalfontein. That existing corridor is an important baseline for any port-to-inland or space-industry supply chain; compare capacity recovery, service reliability, terminals and bottlenecks before proposing a new tunnel or maglev. Source: https://www.transnet.net/SubsiteRender.aspx?id=4492347

Symtropy #1601 owns multimodal transport optimization, #1951 owns reuse-first underground asset evaluation and #3201 owns the MagRail/underground freight comparison. This document does not add a second freight optimizer.

A plausible ecosystem may import components near a major port, integrate/test elsewhere, and launch from a separate range. Only evidence-backed routes connect those functions. Do not co-locate freight terminals and launch hazards simply to reduce straight-line distance.

## 8. Benchmark suite

### Frozen case families

1. Equatorial launch to low-inclination orbit versus higher-latitude alternatives.
2. Equatorial versus high-latitude access to polar/sun-synchronous missions.
3. Similar latitude but different downrange, maritime and overflight constraints.
4. Coastal site with strong logistics access but poor storm/surge exposure.
5. Remote safe range with weak power/rail/port support.
6. Existing launch site with underused infrastructure versus new build.
7. Multi-site network with shared port, grid, tracking or corridor dependencies.
8. Proposed site with incomplete environmental, rights or community evidence.
9. Elevator terminal search with equatorial-anchor negative controls.
10. Orbital-ring search with non-equatorial orbital-plane hypotheses supplied by qualified Symtropy model outputs.
11. Partial-ring paired-endpoint selection; endpoints cannot be optimized independently.
12. Port-to-integration-to-launch logistics with cargo dimensional limits, transfer delays and outages.

### Metrics

Measure target-orbit/launch-azimuth feasibility and useful-orbit coverage; payload/cadence service envelope; time-window/weather availability only where calibrated; downrange/overflight safety coverage and evidence completeness; lifecycle cost per delivered kg/mission; door-to-door logistics cost/time and cargo compatibility; energy/water/material requirements and lifecycle emissions; ecological and social burden; local benefits and imported dependency; resilience under shared grid/tracking/port/road/fibre/range outages; ranking stability/regret under demand, capex, weather and policy uncertainty; and value of further site investigation.

### Proposed acceptance targets (not achieved results)

- 100% rejection of planted invalid-orbit, no-downrange, populated-overflight, missing-rights, stale-source and unsupported-architecture negative controls.
- Exact agreement with independently coded analytic geometry fixtures for simple, declared launch-plane/inclination cases.
- Every candidate retains evidence coverage, rights, uncertainty, source edition/freshness, model suitability and claim-ceiling metadata.
- 100% deterministic replay for fixed source/scenario/policy manifests.
- No hard-gate violations among candidates declared eligible.
- Exact agreement with exhaustive multi-objective reference evaluation on small finite cases.
- For larger search spaces, report Pareto recall, hypervolume gap, rank regret, compute/memory and candidate-evaluation budget against transparent baselines.
- Zero claims that synthetic scores demonstrate launch safety, bankability or physical feasibility.

### Baselines and fair comparison

Compare latitude-only ranking (weak baseline), conventional multi-criteria site selection, exact enumeration on small cases, GIS/network shortest-path and multi-objective baselines, established mission/orbit-analysis tools on matched inputs, and documented operator/site service outcomes. Do not claim superiority over professional mission-design or range-analysis workflows without matched tasks, data, tolerances, hardware and independent reference results.

## 9. Data admission and rights

Every artifact needs a manifest with publisher, stable URL/identifier, edition/revision, acquisition time and observation/effective period; byte count, MIME/container, digest and licence/terms; geography, resolution, datum/CRS, uncertainty, missingness and coverage; observed/estimated/derived/proposed/synthetic classification; allowed use and redistribution obligations; transformation code/version and output digest.

Candidate source classes include national space agencies and range operators, aviation/maritime authorities, meteorological agencies, geospatial agencies, environmental authorities, national statistics, port/rail operators and peer-reviewed orbital/launch publications. An article licence does not automatically license its described dataset.

## 10. Integration boundaries

- **Sol-Atlas:** candidate geography, source/coverage views, spatial evidence, range/catchment overlays, Pareto comparison and investigation-value surfaces.
- **Symtropy:** orbital/launch dynamics, mission-service envelope, ground transport scenarios and deterministic replay; #3201 and #3202 own the new adjacent benchmark tracks.
- **Symthaea:** requirement-to-design hypotheses, research planning, candidate generation and evidence-gap detection; not self-certification.
- **Mycelix:** identity, provenance, rights/consent, commitments and governance evidence; signatures do not establish physical truth or constitute a permit.

A displayed line or orbital animation is not a qualified trajectory. A map overlay is not a range-safety determination. Each projection retains model time, ephemeris time, coordinate frame and source revision.

## 11. Phased execution

**Phase A — synthetic contract:** stable IDs for ecosystem, launch range, integration/manufacturing node, gateway, tracking node, utility dependency and architecture; distinguish source/observation/derived/scenario values; freeze negative controls.

**Phase B — global references:** qualify known operational/historic sites and supported service envelopes without ranking sites from incomplete public data.

**Phase C — East African evidence inventory:** follow the Kenya PPP feasibility-study and transaction-advisor work for the proposed Kipini area; compare its published candidate geography, orbit/range requirements and regional transport/industrial connections with reference sites; involve local stakeholders and authorities before elevating any candidate.

**Phase D — Southern African opportunity screen:** do not assume South Africa is suitable for orbital launch merely because it has ports and rail. First establish launch azimuth, downrange, weather, environment and governance constraints. Evaluate whether manufacturing, integration, tracking, testing or launch support has more defensible regional value than a new range.

**Phase E — specialist validation:** only shortlisted concepts proceed to launch-range, orbital-dynamics, geotechnical, environmental, safety, community, legal and financial study by qualified specialists.

## 12. Claim ceiling

The benchmark produces transparent **site and ecosystem investigation hypotheses**. It does not determine launch licensing, flight safety, environmental permission, land/water rights, public consent, bankability, orbital-ring feasibility, tether-material readiness or construction approval. No location is best without a declared mission/architecture, admissible evidence, jurisdiction/stakeholder context and independent safety review.