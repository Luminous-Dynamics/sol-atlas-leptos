# SOL-CIV-Power: Evidence-Bound Institutional Dependency Atlas V0

**Status:** Research / projection contract proposal  
**Tracking:** [SOL-CIV-POWER-001 — #970](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/970)  
**Related evidence envelope:** [#450](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/450) and draft [PR #452](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/452)  
**Upstream authorities:** Mycelix Civilization Twin; Symthaea power/dependency analysis; Symtropy physical and service simulation

This document specifies a spatial and temporal projection of qualified institutional relationships. It does not establish that any particular real-world actor has power, that a displayed relationship is legally valid, or that proximity implies dependency.

---

## 1. Why this is a distinct capability

Sol Atlas already brings resource, infrastructure, service, trade, supply-chain and governance-related views into a spatial interface. Its existing confluence design explicitly avoids hidden coefficients and warns that co-location is not danger. That is an important foundation.

The public core also contains illustrative Mycelix flow models: TEND time-exchange routes, governance participation pulses, resource-sharing examples and other cross-cluster relationships. These examples are useful for exercising renderers, but the hard-coded simulate_* outputs are synthetic fixtures, not live institutional observations. A boolean verified field alone does not establish provenance, verification method or evidence quality.

The opportunity is to connect spatial context to a source-qualified institutional graph without turning the map into a truth engine.

**Core transition:**

~~~text
independent map layers
  -> evidence-linked spatial entities
  -> qualified directional relationships
  -> dependency paths and substitute analysis
  -> effective-power dimensions and service consequences
  -> human inspection, challenge and temporal replay
~~~

The atlas should make visible where physical infrastructure, institutional authority, resource access, information and dependence intersect. It must keep the source and limits of every claim inspectable.

## 2. Ownership and non-duplication

This is a projection/adapter contract, not another graph authority.

| Concern | Authority |
|---|---|
| Institutional claims, canonical relationships, identity, provenance, rights, disputes and revocation | Mycelix / Civilizational Systems Science interchange |
| Power, dependency, decision-right and mechanism analysis | Symthaea |
| Physical resource, service, disruption and transition trajectories | Symtropy |
| Spatial-temporal projection, rendering, exploration and user inspection | Sol Atlas |

The Sol Atlas projection consumes upstream relationship and evidence references. It does not recreate centrality calculations, institutional interpretation, routing, legal adjudication or governance execution.

Compose the existing work:
- Sol Atlas #450 and draft PR #452 for layer evidence envelopes and scenario/observed distinctions.
- Sol Atlas #11/#12 for historical claims, temporal evidence frontiers and reversible projection.
- Sol Atlas #937 for cross-system territorial rights and stewardship qualification.
- Mycelix #4816 for Civilization Twin subsystem boundaries.
- Mycelix #4814 for production/input-output and critical-supply network fabric.
- Mycelix #5699 for institutional feedback and lock-in.
- Mycelix #2976 for public-integrity mechanism analysis without actor blame.
- Symthaea #7190 for power, agency, dependency and rights-floor observables.
- Symthaea #3054/#3095 for institutional game semantics and experiment receipts.
- Symtropy #6441 for qualified regime / attractor / transition outputs.

If one of those authorities does not yet supply a necessary field, this contract should create a narrow upstream interface task. It must not silently implement a parallel truth source.

## 3. Relationship model

The canonical relationship is directed, typed, time-bounded and evidence-linked. Illustrative relation types include:

- depends_on;
- controls_access_to;
- operates;
- owns_or_stewards;
- funds;
- supplies;
- substitutes_for;
- can_veto;
- sets_agenda_for;
- verifies_or_qualifies;
- delegates_to;
- routes_through;
- bears_cost_of;
- benefits_from.

A relationship record should carry or reference:

1. stable relationship ID and schema revision;
2. subject and object entity references;
3. relation type and direction;
4. scope (jurisdiction, domain, service, resource, geographic extent);
5. valid-time interval;
6. source-known / publication / observation times where applicable;
7. evidence claim and source snapshot references;
8. epistemic class and qualification status;
9. spatial geometry reference, where grounded;
10. uncertainty, limitations, competing claims and disputes;
11. access rights, consent / purpose and redaction policy;
12. transformation receipt if derived from upstream evidence.

The projection must not infer an edge from two entities sharing a map cell, being in the same supply chain category, or appearing near one another.

### 3.1 Formal authority and effective leverage

Formal authority and effective leverage are different dimensions.

An institution may formally authorize a decision. A separate provider may control a bottleneck asset. A verifier may determine whether a resource claim is accepted. A lender may constrain available choices through contract terms. A substitute provider may reduce that leverage.

Show those relationships separately. Do not collapse them into one power score.

Candidate dimensions, consumed only from qualified upstream metrics:

- resource and infrastructure control;
- decision authority;
- information and verification advantage;
- agenda-setting and veto rights;
- substitution depth;
- switching cost;
- exit feasibility;
- bargaining leverage;
- essential-service dependency;
- common-cause exposure.

A dimension without evidence is unknown or not assessed, not zero.

### 3.2 Non-equivalences

The UI and data contracts must preserve:

~~~text
formal authority != effective leverage
ownership != operation != stewardship != jurisdiction
resource balance != control of access
participation != decision authority
reputation / expertise != rights / authorization
network centrality != legitimacy
co-location != dependency
missing evidence != no relationship
recorded claim != adjudicated right
scenario value != observed measurement
~~~

No individual or institution should be branded as corrupt, malicious, illegitimate or dangerous based on network position or an indicator.

## 4. Projection and visualization views

### 4.1 Dependency path explorer

A user selects a service or resource and follows its qualified upstream dependencies:

- source and provider;
- physical asset or route;
- operator and steward;
- settlement or verification bridge;
- decision or authorization requirement;
- substitute or fallback;
- common-cause failure domain.

Every hop is a source-backed relationship. A path is not a guarantee that the full service is available; the UI must distinguish dependency evidence from operational service evidence.

### 4.2 Chokepoint inspector

Selecting a bridge, provider or asset should reveal:

- the set of services and communities that depend on it;
- alternative paths and their qualified capacity;
- substitute depth and switching costs;
- whether alternatives share the same upstream asset, energy source, jurisdiction or verifier;
- interruption and recovery scenarios from Symtropy, when available;
- missing, disputed or stale relationship evidence.

Do not assume two routes are independent simply because they are geometrically distinct.

### 4.3 Authority / leverage comparison

Show separate views for:

- formal decision rights;
- agenda and veto rights;
- control of critical resources;
- operational and verification dependencies;
- alternatives and exit options.

The display should make disagreement possible. For example, a formal authority claim may be disputed while the operational dependency is well documented. The user must be able to inspect both without one silently replacing the other.

### 4.4 Resource and value-flow view

A flow is only aggregatable when its quantity, unit, direction, time basis, accounting boundary and source semantics match.

Do not sum:
- stocks and flows;
- installed capacity and delivered service;
- gross output and value added;
- monetary balances and labor hours;
- physical resource flows and monetary value;
- unpaid care hours and financial transactions.

Show source denominators and aggregation rules. If units or accounting boundaries are incompatible, keep flows separate and explain why.

### 4.5 Rights and service-floor overlay

Where upstream evidence supports it, show which declared cohorts or service units depend on a given pathway and which service floors may be affected by a disruption.

Default to aggregated cohorts and service-level effects. Do not display private individual dependency, voting, health, negotiation or transaction data just because it could create a richer graph.

### 4.6 Historical and temporal replay

Support at least:
- valid time of the relationship;
- time the evidence was observed;
- publication / known-by time;
- assessment time;
- map epoch and evidence frontier.

A view at evidence frontier T must exclude claims first known after T, including downstream inferences built from later evidence. Revisions must be reversible to the source snapshots that existed at the selected frontier.

### 4.7 Explainability inspector

Every node, edge, flow, overlay and aggregate should answer:

- What exactly is being shown?
- Is it observed, reported, inferred, disputed or a scenario assumption?
- Which source and snapshot support it?
- Which transformation or metric produced it?
- What time and geographic scope does it cover?
- What is missing or contested?
- What counterevidence or alternative interpretation exists?
- What information is withheld and why?
- Who is allowed to correct or challenge it?

## 5. Synthetic and scenario data policy

The existing mycelix_flows.rs example functions should be clearly identified as synthetic demo fixtures wherever rendered. The examples may remain useful for UI development and negative controls.

Requirements:
- simulate_tend_flows() is not displayed as actual TEND exchange.
- simulate_governance_pulses() is not displayed as real participation.
- verified = true is not treated as a verification receipt.
- Scenario data has a visible scenario class and cannot enter observed-only summaries.
- Model inference is distinct from curated secondary data.
- Missing values do not default to zero or a plausible location.
- A failed parser does not produce a visually authoritative empty layer.
- Source reuse and attribution rules travel with the data.

The existing provenance work in #450/#452 should provide the shared source/epistemic envelope. This work should extend its relationship projection semantics rather than create another evidence envelope.

## 6. Adversarial qualification fixture matrix

| Fixture | Input condition | Required result |
|---|---|---|
| SP-01 proximity negative control | Two entities share an H3 cell but have no relationship claim | No dependency edge is created |
| SP-02 coordinate invariance | Same graph, different coordinates | Structural metrics remain unchanged |
| SP-03 equal-balance bridge capture | Equal balances, one critical settlement bridge | Balance equality does not hide bridge concentration |
| SP-04 authority vs centrality | Same centrality, different formal decision rights | Separate authority and topology views |
| SP-05 operational bottleneck | No formal veto, but exclusive provider with no substitute | Effective leverage is shown as a separate qualified dimension |
| SP-06 common-cause substitutes | Multiple routes share one upstream failure domain | Alternatives are not counted as independent |
| SP-07 temporal transfer | Ownership / operation / delegation changes over time | Historical and current views remain distinct |
| SP-08 missing edge | Relationship is withheld, stale or unresolved | Render unknown/restricted state, not absence |
| SP-09 participation only | A governance participation pulse exists without decision-right evidence | Show participation only; infer no power |
| SP-10 disputed tenure | Competing ownership or stewardship claims exist | Both remain inspectable with dispute status |
| SP-11 scenario substitution | Synthetic flow is mislabelled as observed | Validator rejects or visibly quarantines it |
| SP-12 privacy boundary | Personal or negotiation-sensitive fields enter public projection | Export/render is blocked or redacted under the access policy |

The test corpus should include both positive and negative controls. The test oracle must not be derived from the renderer being evaluated.

## 7. Metrics and acceptance

Report dimensions separately:

- relationship completeness and source coverage;
- temporal and spatial scope completeness;
- provenance and claim-to-render reversibility;
- unresolved / disputed / withheld relationship share;
- path reachability and substitute depth under declared graph semantics;
- common-cause overlap across nominal alternatives;
- authority and effective-leverage dimensions with source/method references;
- user comprehension of evidence class and relationship type;
- privacy and access-control violations;
- synthetic-versus-observed classification errors.

Initial hard gates:
- zero renderer-created relationships;
- zero scenario-to-observed upgrades;
- zero public disclosure violations in frozen tests;
- every visible relationship links to its canonical claim and source lineage;
- spatial and temporal scope survive export;
- no single power, legitimacy, corruption or institutional-quality score is emitted;
- withheld and missing evidence remain distinguishable from zero;
- same input receipt produces the same projection data under the declared deterministic contract.

These gates qualify projection integrity. They do not prove the upstream claims true or the upstream power metric valid.

## 8. First implementation tranche

1. Review and compose the evidence-envelope work in #450 / PR #452.
2. Add renderer-independent relationship projection types with typed relation, source claim reference, temporal extent, spatial extent, qualification and access policy.
3. Add synthetic relationship fixtures for SP-01 through SP-12.
4. Add validation and round-trip tests for source links, time boundaries, coordinates, missingness and rights.
5. Export a deterministic graph projection for one bounded resource/service dependency case.
6. Add a small UI view only after the pure-Rust model and fixtures pass.
7. Integrate one source-pinned public case after licence, provenance and coverage are checked.

Do not build a universal institutional graph database, a new route engine, a second asset registry, or a power-scoring system.

## 9. External comparison plan

Compare Sol Atlas's spatial projection against established geospatial and network-analysis workflows, not against a generic map screenshot.

Use matched inputs and measure:
- correct rendering of relationship direction and semantic type;
- temporal and evidence-frontier correctness;
- ability to inspect alternate paths and common-cause dependencies;
- preservation of CRS, geometry uncertainty and data provenance;
- aggregation correctness and missingness behavior;
- user task accuracy and time-to-find a source;
- performance at matched graph sizes and spatial densities.

A conventional GIS may be better at general spatial operations, a network package may be better at specialized graph analytics, and an evidence/provenance platform may be better at source custody. Sol Atlas should claim advantage only where its integrated, reversible institutional-spatial inspection demonstrably improves a measured task without weakening those specialist capabilities.

### 9.1 Interoperability and industry benchmark anchors

The first release should be standards-compatible, not another closed geospatial format:

- **OGC API – Features:** use as the feature-query and CRS interop baseline. Part 1 Core and Part 2 CRS by Reference define the baseline for feature access and coordinate-system handling; the benchmark should include conformance checks and exact round-trips. [OGC standard](https://www.ogc.org/standards/ogcapi-features/).
- **OGC API – Common – Part 2: Geospatial Data:** incorporate the newly published standard (approved 15 October 2026; published 12 November 2026) for *collection discovery and description*, including collection metadata and spatial/temporal/multidimensional extents. Part 2 does **not** define feature-item access or data-export operations. Pair it with **OGC API – Features Part 1: Core** for feature collection/item access and **Part 2: CRS by Reference** for alternate coordinate reference systems. Do not claim conformance until the relevant validators and exact test reports exist. [Part 2 official document](https://docs.ogc.org/is/20-024/20-024.pdf); [OGC API – Features](https://www.ogc.org/standards/ogcapi-features/).
- **STAC 1.1.0 / STAC API 1.0.0:** use for discovery and metadata of spatiotemporal assets such as imagery, lidar, point clouds, DEMs and vector datasets. Sol Atlas may extend these with evidence/rights metadata, but should not redefine STAC's asset catalog semantics. [OGC STAC standard page](https://www.ogc.org/standards/stac/).
- **W3C PROV-DM:** crosswalk source, entity, activity, agent and derivation lineage to a domain-agnostic provenance model. Keep rights, epistemic class and claim qualification as domain extensions rather than pretending PROV alone establishes truth. [W3C PROV-DM](https://www.w3.org/TR/prov-dm/).
- **ArcGIS Knowledge:** treat as a direct industry comparison for knowledge graphs that combine entities, relationships, spatial/temporal analysis and network/link analysis. Our differentiation must be measured in source/claim reversibility, explicit epistemic and missingness semantics, historical evidence frontiers, privacy/rights preservation and model-to-decision claim boundaries—not in the mere ability to draw a graph on a map. [ArcGIS Knowledge overview](https://doc.esri.com/en/arcgis-knowledge/latest/introduction/what-is-arcgis-knowledge.html).

The comparison suite should distinguish at least four layers of conformance:
1. **Format interchange:** can the same features, geometries, CRS and spatiotemporal assets round-trip through the applicable OGC/STAC interfaces?
2. **Graph semantics:** are relationship direction, entity identity, time and scope preserved without adjacency inference?
3. **Evidence semantics:** can consumers retain claim lineage, epistemic class, missingness, dispute, rights and known-by frontier?
4. **Decision safety:** can a consumer distinguish a source claim, spatial projection, simulation output, authorized decision and independently observed outcome?

A result can pass the first layer and fail the other three. Publish separate reports; do not compress them into an "interoperability score".

---

## References

- [Sol Atlas evidence-aware planetary resource layers (#450)](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/450)
- [Draft evidence metadata PR (#452)](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/452)
- [Sol Atlas temporal historical projection (#12)](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/12)
- [Symthaea power, agency and dependency stress lab (#7190)](https://github.com/Luminous-Dynamics/symthaea/issues/7190)
- [Mycelix Civilization Twin (#4816)](https://github.com/Luminous-Dynamics/mycelix/issues/4816)
- [Mycelix institutional feedback / lock-in (#5699)](https://github.com/Luminous-Dynamics/mycelix/issues/5699)
- [Symtropy attractor and regime qualification (#6441)](https://github.com/Luminous-Dynamics/symtropy/issues/6441)
