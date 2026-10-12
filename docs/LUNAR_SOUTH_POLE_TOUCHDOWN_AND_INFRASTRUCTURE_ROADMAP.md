# Lunar South Pole: Touchdown-Site Decision and Infrastructure Roadmap

**Status:** research and architecture proposal, 11 October 2026  
**Scope:** first crewed south-polar touchdown, a sustainable lunar surface node, and the cislunar infrastructure needed to support it.  
**Decision boundary:** this is a candidate-screening framework, not an agency site selection or a flight-safety certification.

## Executive recommendation

Do **not** use the interior of Shackleton crater as the default first-touchdown or habitat location. Treat Shackleton and other permanently shadowed regions (PSRs) as high-value science/resource destinations to reach with robotic prospectors. Put the first crewed landing and initial surface operations on a separately verified, illuminated, low-slope landing ellipse with adequate approach lighting and communications, outside the coldest shadowed terrain.

The most useful initial comparison is not simply “Shackleton versus somewhere else.” It is a **network of different roles**:

1. **Touchdown and early operations:** assess the flatter Mons Mouton Plateau and nearby mapped terrain units as a serious baseline, alongside the current agency candidate regions. The 2025 IM-2 mission is useful operational evidence, not proof that every part of Mons Mouton is safe: the lander touched down roughly 400 m from its intended point, ended on its side inside a small crater, and the mission ended early. Precision navigation, hazard mapping and a larger safe ellipse still matter.
2. **Power and communications node:** investigate suitable terrain near, but not automatically on, the high Malapert Massif. NASA analysis identifies high illumination and exceptional Earth visibility around the massif rim. A high point may help a relay or communications node; it does not automatically make a good landing ellipse or habitat foundation.
3. **Science and resource prospecting:** send uncrewed rovers/drills toward selected PSRs near candidate regions such as Nobile and de Gerlache, after ground-truthing local terrain and volatiles. Do not assume a nearby shadowed crater contains economically recoverable ice.
4. **Long-term base:** select a site only after a seasonal illumination/thermal model, terrain and boulder hazard assessment, line-of-sight/relay analysis, and traversability study are tied to the exact landing ellipse and mission date.

The recommended concept is therefore a **South Polar Operations Network**, not a single “perfect crater”: a safe landing/operations node, a well-sited power/communications node, prospecting routes into shadowed terrain, and eventually connected logistics and resource nodes.

## Important programme context

NASA's latest public schedule page currently describes Artemis III as a 2027 low-Earth-orbit demonstration mission and Artemis IV as the target for the first Artemis lunar landing in early 2028. The date and lander readiness remain programmatic uncertainties. Do not write a site assessment as though a crewed touchdown at Shackleton—or at any other specific region—has already been selected.

NASA's refined south-polar candidate-region announcement listed nine regions: **Peak near Cabeus B, Haworth, Malapert Massif, Mons Mouton Plateau, Mons Mouton, Nobile Rim 1, Nobile Rim 2, de Gerlache Rim 2, and Slater Plain**. These are broad regions, not approved pinpoint touchdown coordinates. The public list is useful for screening; the eventual selection has to follow mission, lander, approach, lighting and landing-ellipse constraints.

## What makes Shackleton attractive—and why its interior is not the default base

Shackleton is valuable primarily because the rim/shadow geometry places illuminated terrain near permanently shadowed terrain of major scientific interest. Polar shadows may preserve water ice and other volatiles, while nearby high ground can have long illumination windows. Those are regional properties, not a guarantee of safe local terrain or a commercially viable resource deposit.

The crater floor/interior is a poor default for a first crewed touchdown because persistent shadow implies difficult solar-power access, severe cold, limited direct Earth line-of-sight in many locations, and hard terrain-access constraints. The exact value of a site depends on terrain horizons and date. An accessible rim or ridge near Shackleton is a different candidate from the crater interior; the two should never be conflated in Sol Atlas.

Historical NASA analyses show why illumination must be treated as a time-series and a power-system input rather than a binary “lit/dark” layer. One older model estimated significant energy-storage needs even at the Shackleton rim in a worst-case lunar day. Use that as a warning about storage requirements, not as a current power design number; recompute from the latest topography, exact location, horizon mask, season and load profile.

## Candidate-region screening

The table is qualitative and records what the cited public evidence supports. It does **not** rank landing safety. We deliberately leave numeric scores blank until all candidates have comparable, date-specific data and uncertainty estimates.

| Candidate / role | Evidence-based reason to keep in the comparison | Main unresolved question / hazard | Current posture |
|---|---|---|---|
| **Mons Mouton Plateau** — touchdown baseline to investigate | NASA describes it as a relatively flat south-polar region and used it as the target region for the IM-2 robotic mission. It offers an operationally relevant lesson set from an actual delivery near the pole. | The IM-2 lander ended on its side in a small crater after touchdown hundreds of metres from the intended point. Is the proposed crewed landing ellipse broad, slope/roughness compliant, illuminated along the approach, and tolerant of dispersion? | **Priority for detailed hazard/ellipse reassessment**, not “proven safe.” |
| **Malapert Massif region** — communications/power candidate | NASA's LOLA-based regional analysis reports high illumination around parts of the rim and exceptional Earth visibility. This is attractive for line-of-sight comms and some power concepts. | The massif is not uniformly flat. A summit/rim relay site and a safe landing/habitat site may need to be separate. What are the local slope, boulder, lighting and route constraints? | **Strong network-node candidate; touchdown remains data-gated.** |
| **de Gerlache Rim 2 / connecting-ridge corridor** — power-to-PSR science access | Ridge and rim terrain can place illuminated ground near shadowed areas; this supports a useful polar exploration architecture. | Narrow ridges, terrain horizon, rover traversability, and intermittent communication/illumination can make surface operations difficult. | **Strong prospecting-corridor candidate; local DEM study required.** |
| **Nobile Rim 1 / Nobile Rim 2** — prospecting and science corridor | NASA's earlier VIPER selection work identified the mountainous area west of Nobile for rover-accessible terrain and nearby permanently shadowed areas of scientific interest. | Remote sensing does not resolve deposit concentration, physical form, extraction depth or resource grade well enough to promise an ISRU supply. | **Strong robotic prospecting alternative; do not assume reserve.** |
| **Peak near Shackleton / Shackleton rim** — alternative landing or relay corridor | Several historical studies identify portions of Shackleton's rim or adjacent ridges as illumination/communications candidates. NASA's earlier candidate lists included Peak near Shackleton as a *region*. | Do not infer a safe ellipse from a region name, historic average illumination, or a map of the crater rim. Verify current seasonal lighting, slopes, boulders, approach geometry and distance to science zones. | **Keep as a comparator; do not privilege it by name alone.** |
| **Haworth / Slater Plain / Peak near Cabeus B** — diversity and mission-window alternatives | They appear in NASA's refined public list of candidate regions. Maintaining alternatives matters because illumination and launch/approach windows change feasibility. | Public candidate inclusion is not a quantitative performance score. Need comparable current terrain, power, comms, thermal, traverse and science datasets. | **Keep in a balanced down-select until common data are scored.** |

### Provisional conclusion

For an initial *crewed touchdown* baseline, **start the quantitative down-select with Mons Mouton Plateau and a mapped Malapert-adjacent alternative**, while retaining de Gerlache/Nobile routes for robotic resource scouting and Shackleton rim sites as comparators. Do not select the Malapert summit, Shackleton crater floor, or any named feature as a touchdown point merely because it is prominent or scientifically interesting.

This is a provisional engineering search strategy, not a verdict that Mons Mouton is safer than the other finalists. NASA's IM-2 result makes the distinction important: even a region described as relatively flat can contain small-crater hazards and suffer touchdown dispersions.

## Required data before site selection

Sol Atlas should not generate a weighted “winner” until each candidate has, at minimum:

- A common coordinate/reference-frame definition, datum, map epoch and immutable source fingerprints.
- High-resolution elevation/slope maps, slope uncertainty, rock/boulder hazard layers, small-crater density, landing dispersion footprint and terrain-relative-navigation observability.
- Time-resolved solar horizon/illumination for the proposed mission window plus a conservative multi-season design case; temperatures and cold-soak durations at touchdown, routes, habitat and power equipment.
- Direct-to-Earth visibility over time and relay-satellite coverage, including outages and the minimum autonomous safe mode.
- PSR boundaries, traverse distance/slope, route hazard and estimated science-access time; resource observations with uncertainty and explicit separation of remote-sensing evidence from in-situ confirmation.
- Descent/ascent vehicle limits, plume/dust zones, separation distances, emergency egress and safe havens.
- A verified surface power/load profile including storage efficiency, eclipses, survival loads, thermal control, dust degradation and redundancy.
- A source-bound uncertainty record for each metric, not just a point estimate.

### Decision method

Use a **constraint-first, Pareto-aware down-select**, not one opaque weighted score.

1. Fail candidates that cannot meet non-negotiable lander, slope, hazard, lighting, approach, communications or crew egress constraints for the actual launch/mission window.
2. Compare the feasible set on robust illumination/power, communications availability, traverse access to science targets, thermal conditions, geology, logistics and expansion potential.
3. Display a Pareto frontier and show which candidates win only because of a particular weighting. Let mission owners state the weights; preserve a sensitivity analysis rather than hiding value judgments in the software.
4. Bind every metric to source file/digest, acquisition date, processing version, spatial resolution, units, uncertainty and transformation lineage.
5. When evidence is missing, show **Unknown / Not assessed**, not zero and not green.
6. Keep “good for landing”, “good for power/communications”, “good for science” and “good for resource extraction” as separate suitability outputs. They are not the same optimization objective.

## Lunar infrastructure: what we should build, in dependency order

### Phase 0 — An evidence-grade lunar digital twin

**Build first.** Create a versioned, multi-layer lunar south-pole terrain and operations model in Sol Atlas. It should integrate LOLA elevation/slope and uncertainty, lighting horizons, temperature, communication visibility, candidate landing ellipses, PSRs, traverse networks, radiation and resource evidence. Every layer needs provenance, time coverage, resolution, uncertainty, quality/partial-load status and clear source-vs-derived-vs-simulation labels.

Symthaea can help coordinate model selection, anomaly triage, sensitivity analysis and candidate comparison, but its advisory output must not overrule a deterministic safety gate. Symtropy supplies checked geometry/kinematics primitives; those alone are not orbital mechanics or a landing simulator. Mycelix-DeSci binds immutable evidence, receipts, review and supersession. Sol Atlas presents the resulting status read-only and must never visually upgrade a blocked claim.

### Phase 1 — Robotic pathfinders and local ground truth

Before a crew base, send a coordinated robotic package: stereo imaging and terrain mapping; hazard/boulder mapping; neutron/hydrogen and volatile measurements; drill/sample handling; thermal sensors; radiation environment measurements; retroreflectors/beacons; communications tests; and a rover/short-hop robot capable of reaching shadow edges without risking crew.

The goal is not “prove there is ice nearby.” It is a spatially registered inventory with uncertainty: what material, at what depth, in what concentration, and how spatially continuous? NASA notes that orbital measurements cannot by themselves reveal exact surface-scale water distribution; in-situ prospecting is essential.

Use the IM-2 outcome as a systems-learning input: touchdown dispersion, crater/terrain interpretation, tipping stability, deployable payload accessibility and short-duration power/communications need to be analysed together. The lesson is to build hazard tolerance and observability, not to reject a whole geographic region from one landing.

### Phase 2 — Minimum viable polar surface node

Build only after the landing ellipse and operating environment are demonstrated.

- **Power:** solar arrays on well-characterized illuminated terrain plus storage; a separately evaluated fission surface-power option for firm power and shadow-period survival. Treat reactors as a separate safety, launch, siting, shielding, heat-rejection and regulatory programme, not a plug-in battery.
- **Power distribution:** modular DC bus, protected connectors, isolation/fault detection, metering and load-shedding priorities; power interfaces that can be serviced by robots.
- **Communications and navigation:** Earth links where geometry allows, a lunar relay/navigation service for blocked horizons, local ranging/beacons, interoperable time/frame conventions and autonomous contingency modes.
- **Landing surface and dust control:** robotic grading/compaction or regolith-derived pads/berms where proven; protected equipment lanes and stand-off distances between landers, habitats, science experiments and solar arrays. Verify plume/dust effects before scaling the infrastructure.
- **Thermal and radiation protection:** insulated equipment, night/survival modes and habitat shielding using local material only after structural and dust-handling tests. The habitat requires storm-shelter capability and independent safe-life support.
- **Mobility/logistics:** unpressurized cargo rover first, then pressurized crew mobility as warranted; route mapping, tow/recovery capability, tool interfaces and repairable wheels/joints.
- **Operations and safety:** landing/traffic scheduling, no-go polygons, lost-comms safe state, rescue/repair doctrine, spare-parts inventory, medical contingency and clear crew/robot zones.

### Phase 3 — Resource pilot, only after measured reserves

Run a closed-loop pilot for excavation, feedstock preparation, thermal extraction, condensation, water analysis and energy/accounting. Report kg of usable product per kWh, per kg of plant, per operating hour, rejected feedstock, maintenance, contamination and uncertainty. Then test electrolysis and storage if sufficient clean water is demonstrated.

No full-scale propellant economy should be assumed from orbital indications of hydrogen or from a single successful drill. Scale decisions should depend on measured resource grade, extraction yield, energy cost, dust wear, thermal losses, maintenance burden, throughput and independent replication.

### Phase 4 — Connected polar industrial network

Once single-node survival is boringly reliable, grow toward:

- Multiple power/relay nodes so a single outage cannot strand surface assets.
- Robotic construction of landing pads, berms, equipment foundations and graded routes using local material where life-cycle evidence supports it.
- Modular storage for water, oxygen, consumables and protected spares; standardized robotic cargo interfaces.
- Workshop/repair capability, calibration benches, component recycling and a manufacturing demonstrator for selected high-value parts.
- A science commons of time-stamped, source-bound resource and environmental data with access controls, provenance and public non-sensitive outputs.
- Resource-use, heritage protection, exclusion-zone and emergency-coordination rules that are published and auditable, not hidden in proprietary map layers.

### Parallel cislunar infrastructure

Some assets support every landing region and should be developed independent of the touchdown decision:

1. **Lunar communications, navigation and timing** — interoperable relays, surface positioning, time transfer and resilient links. South-polar terrain frequently blocks direct Earth line-of-sight.
2. **Cislunar transport and servicing** — reusable transfer tugs, standardized docking/refuelling interfaces and cargo aggregation where launch cadence and economics justify them. Fuel depots should follow proven demand and verified propellant supply, not lead the programme.
3. **Space weather and radiation monitoring** — solar-particle-event warning, local dosimetry and protected shelter protocols.
4. **Orbital traffic coordination and debris mitigation** — tracking, conjunction assessment, servicing interfaces and end-of-life rules for increasingly busy Earth and lunar orbits.
5. **Robotic construction and maintenance** — inspectable, replaceable systems first; autonomous excavation and assembly gradually, with safe stop/rollback states.
6. **Scientific infrastructure** — a lunar far-side radio-quiet astronomy demonstrator and distributed geophysics/solar-system monitoring, subject to environmental and spectrum coordination.
7. **Earth–Moon resource accounting and governance** — common units, interfaces, provenance and dispute-resolution protocols, avoiding a proprietary dependency on one operator for navigation, power or evidence.

## How this becomes interactive in Sol Atlas

Do not render one glowing “best site” with no audit trail. Build a *Lunar Site Observatory* view with:

- Candidate regions and actual candidate landing ellipses kept as different geometry types.
- Time control for solar illumination, temperature and Earth/relay visibility.
- Toggleable slope, roughness/boulders, PSRs, resource evidence, routes, power infrastructure and mission assets.
- A side-by-side site comparison table showing values, source date, resolution, uncertainty, evidence state and unknowns.
- Pareto scatter/parallel coordinates with user-visible weights and sensitivity; no single hidden “habitability score”.
- Separate suitability tracks for touchdown, crew operations, power/communications, science access and ISRU.
- Failure mode overlays: power outage, relay loss, rover route blocked, landing dispersion, thermal survival and inaccessible sample site.
- A replayable scenario snapshot binding the exact data, model, parameters, run artifacts and reviewer disposition.
- A hard rule that “queued” is not “passed”, a visualization is not an independent reviewer, and model output cannot upgrade the evidence state.

## Acceptance gates

A landing-region comparison is ready for decision review only when the exact map products, time window, landing ellipse, hazard/dispersion model, illumination/thermal simulation, comms/relay assumptions and uncertainty are frozen and digest-bound. It should then be independently compared against reference tools and reviewed by specialists for landing dynamics, surface geology, power, communications and crew operations.

A base location is not “proven” by passing a terrain model. A water-production plant is not proven by detecting hydrogen. A relay is not proven by visibility at one epoch. Every claim must stay at the highest fidelity actually tested.

## Sources

1. NASA, [Provides Update on Artemis III Moon Landing Regions](https://www.nasa.gov/news-release/nasa-provides-update-on-artemis-iii-moon-landing-regions/) — refined nine-region south-polar shortlist and selection criteria.
2. NASA, [Artemis III News and Updates](https://www.nasa.gov/artemis-iii-news-and-updates/) — current mission framing: Artemis III demonstration in 2027; Artemis IV targeted as the first lunar landing in early 2028.
3. NASA, [Lunar South Pole Region](https://www.nasa.gov/reference/moonbase-environment/) — terrain hazards, shadowed terrain, resource uncertainty and infrastructure challenges.
4. NASA Science, [Malapert region](https://science.nasa.gov/image-detail/20100618/) — LOLA-based illumination and Earth-visibility context for the Malapert Massif region.
5. NASA, [Receives Some Data Before Intuitive Machines Ends Lunar Mission](https://www.nasa.gov/news-release/nasa-receives-some-data-before-intuitive-machines-ends-lunar-mission/) — actual IM-2 landing offset, tipped lander and early end of mission near Mons Mouton.
6. NASA, [Water-Hunting Tool Will Help Scout Moon’s South Pole](https://www.nasa.gov/solar-system/moon/nasas-water-hunting-tool-will-help-scout-moons-south-pole/) — why in-situ prospecting is needed to map resource distribution.
7. NASA, [Lunar Surface Technology](https://www.nasa.gov/lunar-surface-technology/) — power, ISRU, construction/landing pads, robotics and dust mitigation technology areas.
8. ESA, [Moonlight](https://www.esa.int/Applications/Connectivity_and_Secure_Communications/Moonlight2) — lunar communications/navigation constellation concept, with south-polar coverage as a priority.
9. NASA Technical Reports Server, [Illumination Conditions at the Lunar Poles](https://ntrs.nasa.gov/archive/nasa/casi.ntrs.nasa.gov/20170007365.pdf) — older illumination modelling; use as historical context and recompute for current sites and mission dates.
10. NASA Goddard, [High-Resolution LOLA Topography for Lunar South Pole Sites](https://pgda.gsfc.nasa.gov/products/78) — public elevation, slope and uncertainty products for multiple south-polar candidate sites.

## Machine-enforced data boundary

The machine-readable candidate register is a **research-screening dataset**, not a landing-site recommendation. Its contract is enforced by the repository's independent `validate-lunar-site-screening` CI job.

Run the same standard-library-only checks locally:

```sh
python3 scripts/validate_lunar_site_screening.py
python3 -m unittest discover -s tests -p 'test_lunar_site_screening_contract.py'
```

The gate intentionally requires all eleven reviewed region/comparator records to remain identifiable, forbids point-coordinate and ranking fields, requires scalar scoring and comparison weights to remain null, and requires each `known_metrics` object to remain empty until comparable evidence can be bound to provenance. It also requires the comparison layer set, evidence state, unresolved questions, and HTTPS source references.

This is a protective boundary, not a claim that the regions are equally promising. A future version may add measurements only with explicit units, coordinate frame, spatial resolution, acquisition epoch/date, uncertainty, source/version, and evidence state; the accompanying analysis must explain comparability and preserve missing values as unknown. A site-specific coordinate must never be inferred from a broad region name or presented as an agency-approved landing ellipse.


## Geospatial data ingestion: actionable sources and historical comparators

The first useful data-ingestion pass should not start by assigning site scores. It should start by resolving exact region polygons and source-product identities, then joining terrain, uncertainty and illumination only where their coordinate references and time windows are compatible.

The repository's `docs/data/lunar_geospatial_source_catalog.v1.json` records seven source families, their limits, candidate-name crosswalk state and acceptance checks:

1. **NASA PGDA high-resolution LOLA site products** — [product 78](https://pgda.gsfc.nasa.gov/products/78) exposes per-region elevation, slope, LOLA-count and uncertainty rasters, clone ensembles and ROI shapefiles. Its page notes that around 90% of pixels in the cited 5 m/pixel LDEMs are interpolated; across the cited regions it reports median RMS height uncertainty around 0.30–0.50 m and slope uncertainty around 1.5–2.5°. These are not uniform per-pixel errors or operational safety limits.
2. **NASA PGDA lunar polar illumination** — [product 69](https://pgda.gsfc.nasa.gov/products/69) provides average solar illumination, permanent-shadow, Earth-visibility and sky-visibility layers at different polar resolutions. Some products average over an approximately 18.6-year nutation cycle; they are screening context, not mission-window power forecasts. Cloud-Optimized GeoTIFF overviews may average pixels, so quantitative sampling must use full-resolution data and product labels.
3. **South Pole LOLA DEM mosaic** — [product 81](https://pgda.gsfc.nasa.gov/products/81) provides 5 m/pixel products for 87–90°S, including elevation, slope and LOLA return counts. It is large and does not cover every candidate region; only use it after proving the region of interest intersects its documented bounds.
4. **USGS Artemis III candidate navigation grids** — [USGS data release page](https://www.usgs.gov/data/down-selected-artemis-iii-candidate-landing-site-navigational-grids) documents 1 km, 100 m and 10 m LGRS/Artemis Condensed Coordinates grids for nine refined candidate regions, with lunar LTM/LPS projection and DE421 Mean Earth datum requirements. Useful for consistent navigation references, but grid cells are not proof of candidate boundary geometry. The DOI download was inaccessible to this research tool, so the shapefiles remain unverified.
5. **Global LROC NAC DTM footprint inventory** — [NAC DTM shapefile page](https://data.lroc.im-ldi.com/lroc/view_rdr/SHAPEFILE_NAC_DTMS) provides separate 180° and 360° longitude-domain footprint shapefiles in a source-described Moon_2000 geographic coordinate system, with per-DTM resolution and error/coverage attributes. The listed footprint release runs through 15 September 2026. This can discover stereo-terrain coverage, but its footprints are not Artemis candidate boundaries; the ZIPs and README have not been ingested.
6. **LROC NAC Shackleton–de Gerlache Connecting Ridge DTM** — [SHACKRDGE02 product page](https://data.lroc.im-ldi.com/lroc/view_rdr/NAC_DTM_SHACKRDGE02) lists a 3.50 m/pixel stereo-derived DTM, a PDS label, color-slope product and confidence map. The stereo images were acquired in September 2010. This is a localized independent terrain cross-check, not a current Artemis-region crosswalk or a substitute for common-coverage LOLA/lighting data; none of its linked binary/label bytes has been retrieved or verified in this run.
7. **NASA SVS 2025–2028 illumination visualizations** — [SVS 5027](https://svs.gsfc.nasa.gov/5027) illustrates the temporal progression of sunlight and shadow at two-hour intervals. Treat these rendered videos/frames as qualitative context, not georeferenced numeric source rasters.

### Connecting Ridge: evidence to investigate, not a winner

NASA PGDA product 78 describes Site 01 (Connecting Ridge) as having the largest area with average illumination above 70% at 1 m above the surface among the sites compared for 1 January 2024–1 January 2026. It also identifies ROIs 4–6 as generally favorable when uncertainty is considered, and describes longer continuous illumination at 5 m for Site 01. This is a **source-reported historical illumination result**, not a new measurement by this project and not evidence of crewed touchdown safety.

A separate, locally targeted terrain source is also available: LROC's **SHACKRDGE02** stereo DTM. Its product page reports a 3.50 m/pixel grid and links a 32-bit GeoTIFF, PDS label, color-slope product and confidence map. This offers a useful second measurement lineage for checking local landforms against LOLA's interpolated polar DEM, but only after the label confirms the grid/reference frame and the confidence values are decoded. The page also reports Relative LE = 3.41 and LOLA_RMS = 1.06; their definitions and units must be checked in the product metadata before interpretation. The current research pass could inspect the product page but could not retrieve the linked PDS label or binary assets, so no checksum, label validation, raster comparison, or site statistic is claimed.

For candidate-coordinate handling, the [USGS down-selected Artemis III navigation-grid release](https://www.usgs.gov/data/down-selected-artemis-iii-candidate-landing-site-navigational-grids) adds standardized LGRS/ACC grid geometry for the nine refined regions at 1 km, 100 m and 10 m scales. USGS specifies lunar LTM/LPS projections and JPL DE421 Mean Earth datum requirements. These grids help anchor coordinate transformations and reproducibility; they are navigation grids, not an automatic substitute for the independently versioned candidate boundary polygons. The source page lists DOI 10.5066/P1MEQ6UK, but the DOI target was not accessible to this research tool and no shapefile was ingested.

A second route helps make that cross-check systematic: LROC's [Global NAC DTM footprint inventory](https://data.lroc.im-ldi.com/lroc/view_rdr/SHAPEFILE_NAC_DTMS) includes separate 180° and 360° longitude-domain shapefiles, uses a source-described Moon_2000 geographic coordinate system, and lists DTM resolution plus several error/coverage fields. Its published release cut-off is 15 September 2026. It can identify which stereo DTM footprints may overlap a region once the archive is ingested, but **it does not define the Artemis candidate polygons**. The ZIP/README links were inaccessible through this research browser, so no SHACKRDGE02 feature row or polygon intersection is claimed.

Therefore keep Connecting Ridge / Site 01–ROI 4–6 as a separate historical comparator for future, same-method evaluation. Before using it in candidate selection, retrieve and hash the source rasters, ROI shapefile and clone products; validate exact boundary correspondence; then recompute equivalent illumination, slope, roughness, communications, thermal and access metrics for a clearly declared mission epoch. Do not silently equate legacy Site 01 boundaries with any current Artemis landing-region label.

### Ingestion acceptance gate

No raster-derived number should enter the candidate register until the record contains a source URL/version, downloaded asset hash, retrieval time, product-label reference, coordinate frame, units, pixel registration, spatial resolution, nodata/scale-offset handling, ROI mask version, temporal window where applicable, uncertainty treatment, software version and replayable calculation receipt. Store source-reported observations separately from repository-computed measurements, and keep missing or incompatible data as unknown.


## Current infrastructure demand signal (checked 12 October 2026)

NASA's 8 September 2026 announcement for the NextSTEP-3 Broad Agency Announcement, Appendix A — Lunar Enabling Infrastructure Accelerator, was updated on 28 September 2026 to extend the response deadline. The published target capability areas are:

1. **Vertical solar arrays and power distribution/storage** — include deployment geometry, dust response, cable/connector standards, storage state-of-charge reserve, thermal cycling and power continuity under the intended surface timeline.
2. **Oxygen extraction from regolith** — require a measured feedstock assay and process energy/material balance. Do not assume that regolith oxygen production proves accessible water ice or that the product is immediately safe for crew use.
3. **Radioisotope Stirling power** — treat as a firm-power option requiring heat-flow, radiator, lifetime, maintenance, launch-safety and deployment analysis; the existence of a NASA call is not a finding that a given design is flight-ready.
4. **In-space advanced manufacturing** — prioritize repairable interfaces, qualified feedstocks and replacement parts that materially reduce Earth resupply, not broad “manufacturing capability” claims without demonstrated part requirements.
5. **Nanomaterials production** — keep a separate technology-demonstration track with product specifications, process yields, power/thermal demand and contamination controls. Do not assume it is a first-base prerequisite.

These are useful external alignment signals for the proposed infrastructure roadmap, not procurement eligibility or award advice. The solicitation is U.S.-led and its participation rules need to be checked in the live [SAM.gov opportunity](https://sam.gov/workspace/contract/opp/a6de3e86307449dbbf316af25147e35e/view); the NASA announcement states that industry, academia, non-profits and international partners participating through U.S.-led teams may be eligible. Recheck the live notice before acting because the response deadline and detailed terms can change.

**Design implication:** make the first lunar node a set of interoperable services with explicit interfaces: primary solar generation, firm-power contingency, energy storage, DC distribution, dust-tolerant connectors, thermal rejection, communications/navigation, robotic inspection/repair, and telemetry with independently checkable evidence. Tie later oxygen extraction/manufacturing modules to measured throughput, kWh/kg, useful yield, availability, maintenance and spares. Shared electrical/mechanical interfaces and evidence contracts are higher leverage than choosing a single “hero” reactor or factory prematurely.


### Local asset receipt workflow

When a source asset has been downloaded outside this constrained research environment, create a content-addressed receipt against the exact URL listed in the catalogue:

```sh
python3 scripts/lunar_asset_receipt.py \
  --asset-url 'https://imbrium.mit.edu/EXTRAS/ILLUMINATION/JP2/AVGVISIB_85S_060M_201608_JP2.LBL' \
  --file ./incoming/AVGVISIB_85S_060M_201608_JP2.LBL \
  --output ./evidence/lunar/AVGVISIB_85S_060M_201608_JP2.receipt.json
```

The helper opens one regular local file and hashes bytes from that file descriptor, records byte count and file-observation details, and binds the receipt to the exact catalogue bytes by catalogue SHA-256. It refuses symbolic-link inputs, non-regular or empty files, URLs not present in the reviewed catalogue, known-broken URLs recorded as 404, path replacement or detected content/metadata changes during hashing, and a catalogue that changed after parsing.

**A receipt is not source authentication.** The helper does not fetch the network, authenticate that local bytes came from the asserted URL, parse a PDS label, validate raster CRS/scale/offsets, or calculate site metrics. It deliberately sets those verification flags to false. A human/reviewer must examine source headers and product labels, compare the receipt with the retrieval record, and only then propose a separate reviewable catalog-state change. Do not change an asset to `downloaded_and_hashed` merely because a local file exists.

The same guard suite can be run locally with:

```sh
python3 scripts/validate_lunar_site_screening.py
python3 -m unittest discover -s tests -p 'test_lunar_site_screening_contract.py'
```
