# South African Durban–Gauteng Corridor Demonstrator
## Scope and source audit v0.1

**As-of date:** 16 November 2026  
**Status:** public-source scoping only; no corridor capacity, investment, engineering or feasibility result.  
**Work item:** [DEEP-TIME-SA-CORRIDOR-001 #895](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/895).

## 1. Why this is a useful demonstrator

The Durban–Gauteng container corridor is a real national logistics system with port, rail, road, inland-terminal, energy, communications, labour, security and governance dependencies. It lets the deep-time programme test service continuity and topology change in an actual policy/geographic context while keeping initial benchmark execution synthetic.

It is not a reason to assume the corridor requires a canal, another megaproject or a specific port expansion.

## 2. Public source snapshot

The companion JSON source register records six public sources and, more importantly, what each source does *not* authorize us to infer.

### Official port planning

Transnet's KZN Logistics Hub page describes 33 projects across Durban and Richards Bay masterplans and states an estimated investment above R154 billion. It lists Durban current container capacity as 2.9m TEUs and planned future capacity as 11.4m TEUs. These are Transnet planning/capacity statements; do not treat the planned future capacity as operating capacity, actual throughput or guaranteed delivery. [Transnet KZN Logistics Hub](https://www.transnet.net/SubsiteRender.aspx?id=5723346).

### Rail reform and access

The Minister of Transport reported on 8 July 2026 that the first 11 private Train Operating Companies were approved for network access on 13 March 2026. The speech says operations are expected to commence on 1 April 2027 and that the operators plan up to 24 million tonnes of freight per year collectively. These are dated official milestone and operator-plan claims—not evidence that services are already operating or that those tonnes are reserved for this corridor. [Ministerial statement](https://www.gov.za/news/speeches/minister-barbara-creecy-africa-rail-convention-08-jul-2026).

### Port statistics

Transnet's Port Statistics page says Cargo Statistics are based on actual cargo dues invoiced. At the 16 November 2026 retrieval snapshot, its public page displayed monthly series through September 2026. This is promising for reconstructing historic port cargo, but files must still be downloaded, hashed, labelled by metric/unit, and reconciled to annual totals before use. The page itself disclaims responsibility for inconsistencies and interpretations. Do not confuse cargo statistics with designed capacity or hinterland rail throughput. [TNPA Port Statistics](https://www.transnet.net/SubsiteRender.aspx?id=24332214).

### Coastal adaptation

South Africa's Coastal Adaptation Response Plan launch statement describes five components—ecosystems, communities, infrastructure, livelihoods and governance—21 measurable outcomes and more than 150 interventions. This provides policy context, not local proof that a Durban asset has a funded or completed intervention. Obtain the complete plan and relevant KwaZulu-Natal/municipal implementation records before scoring local adaptations. [Government statement, 5 June 2025](https://www.gov.za/news/media-statements/minister-dion-george-launches-climate-change-coastal-adaptation-response-plan).

### Water and ecological constraints

National Water Resource Strategy III sets policy direction for sustainable and equitable water management, climate adaptation, water quality and ecological infrastructure. It is not a catchment water balance, Reserve determination, water-use licence or donor-basin surplus. Any future waterway or transfer option must remain blocked until the applicable catchment evidence and competent-authority requirements are verified. [DWS NWRS-3](https://www.dws.gov.za/documents/Other/Strategic%20Plan/2023/Approved%20National%20Water%20Resource%20Strategy%20Third%20Edition%20%28NWRS3%29%202023.pdf).

### Rail network statement

The public government notice links a 2024/2025 network statement and rail access tariffs. It is a dated reference. Check for later network statements, tariff changes, access conditions and segment restrictions before numeric use. [Government notice](https://www.gov.za/documents/notices/roadmap-freight-logistics-system-south-africa-final-network-statement-and-rail).

## 3. Benchmark alternatives

Initial comparison should include:
1. no-build / observed current operation;
2. operational reliability and scheduling improvements;
3. road-dominant baseline;
4. rail with road drayage;
5. rail plus inland terminal / dry-port;
6. staged open-access rail and terminal capacity with commissioning uncertainty;
7. distributed gateway alternatives only where cargo type, terminal capability, network access and capacity are evidenced;
8. demand management / shipment consolidation.

An artificial canal, inter-basin transfer or major new waterway is not a default candidate. It stays as an unsupported hypothesis until water, environmental, legal, geotechnical, sediment, cost and alternatives evidence exists.

## 4. Topology-renewal questions

- Can new rail or inland-terminal service be commissioned before old capacity is withdrawn?
- What happens if open-access implementation is delayed or allocated paths do not become operational?
- How much nominal redundancy shares a port, feeder, telecom, control, road crossing, flood footprint, workforce or upstream supplier?
- Can freight transfer across road/rail/terminal alternatives while reporting unmet demand honestly?
- What happens when coastal or port functions are constrained but inland service nodes remain available?
- Which measures protect affected communities, workers, ecosystems and essential access rather than merely maximizing TEUs?

## 5. Data-intake gates

Before numeric corridor assessment:
- separate throughput, rated capacity, planned capacity, slot allocation, train path, actual service, dwell time and reliability;
- pin exact artifact bytes, source revision, unit, period, geography and rights;
- obtain route/segment geometry and network restrictions;
- obtain origin–destination / commodity demand with the right granularity and permissions;
- qualify port, terminal and rail operating observations;
- capture multi-hazard and vertical-datum evidence for coastal and inland assets;
- map physical, digital, geographic, economic, ecological and institutional dependencies using existing owner schemas;
- bind water and environmental flows to applicable catchment and legal evidence;
- retain unknown / not evaluated for missing values; do not use synthetic defaults in real-world outputs.

## 6. Benchmark sequence

**Stage A — synthetic exact corpus:** at least 20 cases and five exact network-flow/service known answers, including capacity bottleneck, route failure, service transfer, common-cause outage and transition gap.

**Stage B — public-data reproduction:** reproduce port cargo time series from pinned TNPA downloads and compare with annual public totals. Keep planned capacity and policy milestones in separate data classes.

**Stage C — matched multimodal evaluation:** compare road, rail/road, dry-port and distributed portfolios at the same demand, cost boundary and scenario support. Compose Symtropy #1601 rather than create a second transport optimizer.

**Stage D — adaptive transition experiment:** compare fixed plans with staged pathway policies using the TNR-001/002/008/009 semantics and the Symthaea #7636 policy owner.

**Stage E — physical/local qualification:** only after rights-cleared local topography, bathymetry, flooding, structural, geotechnical, ecology and community evidence has been admitted.

## 7. Claim ceiling

This is a public-source audit and demonstrator scope. No route capacity, port performance, preferred investment, canal viability, water entitlement, navigation safety, site suitability, financing, local consent or construction approval is inferred.