# Farmer-first Field Workbench: UX, Evidence, and Benchmarks

**Status:** research-informed product proposal plus clickable Leptos concept route  
**Scope:** Sol Atlas, with later integrations to Symthaea, Symtropy, and Mycelix  
**Evidence state:** not yet field-validated

## Product thesis

Do not sell farmers an AI dashboard. Give them a dependable workbench that helps them decide what to inspect, what to do next, and whether an action is worth its costs. The product must remain useful with incomplete records, irregular data entry, poor connectivity, different farming systems, and a recommendation that turns out to be wrong.

A recent farmer-centred grazing digital-twin study conducted 36 farmer and consultant interviews with prototype sessions. It emphasizes decision practices, spatial navigation, high utility despite irregular data entry, and differences in perceived usefulness by management intensity. That supports an operator-first map and workflow, not a model-first dashboard. Reference: https://doi.org/10.1007/s10460-025-10752-x

FAO case-study research also identifies cost, skills and knowledge, enabling infrastructure, connectivity, and adaptation to diverse farm types as adoption issues. References: https://doi.org/10.4060/cc2912en and https://doi.org/10.4060/cc2914en

## Paradigm shifts

| From | To |
|---|---|
| More charts and alerts | A short, ranked work queue with a useful next step |
| AI score first | Place, task, consequence, evidence, uncertainty |
| Assumed-perfect telemetry | Useful with manual, irregular, stale, or missing observations |
| One generic smart farm | Crop, grazing, mixed-farm, and herd-specific workflows |
| Cloud lock-in | Offline capture, visible sync state, exportable records, controlled sharing |
| Yield as the only success | Margin, labour, resilience, animal welfare, soil/water stewardship, and operator agency |
| Black-box recommendation | Sources, assumptions, uncertainty, downside, and what would change the answer |
| Logins as adoption | Repeated use because a real job became easier |

## UX rules

1. Lead with Today: show the few jobs that need attention and explain why they appear.
2. Start from place when location affects the decision. Keep map, history, time, and plan linked.
3. Make unknown a real state. Missing is not zero; stale is not current; estimates are not observations.
4. Capture at the point of work. Use large touch targets, short forms, repeatable group actions, useful defaults, and no compulsory long entry form.
5. Offline is a product requirement. Save locally, make pending sync visible, reconcile conflicts, and never silently drop a note.
6. Explain consequences in farm language: time, inputs, costs, operational risk, timing, and trade-offs before model internals.
7. Keep people in control. Clearly distinguish observation, recommendation, and action. No automatic animal diagnosis, chemical application, irrigation control, or machinery actuation from unvalidated model output.
8. Put data rights in the interface: inspect source, export records, correct errors, control recipients, and revoke future sharing. Explain that revocation cannot erase a copy already received by another party.
9. Design for mixed digital skills, outdoor contrast, keyboard and screen-reader access, reduced motion, and low-cost phones.
10. Integrate one workflow at a time. Add satellite, weather, machinery, animal, soil, or market sources only with visible provenance and freshness.

## First interface slice

The /farm route is a concept prototype with illustrative data only. It is not connected to a farm account, sensors, Symthaea, Mycelix, or live Sol Atlas sources.

- Today: short work queue and one demo-only record interaction.
- Land & grazing: schematic paddock selector to test location-first navigation.
- Herd: observation workflow principles; no simulated diagnosis.
- Costs & outcomes: prompts for baseline and evidence; no fabricated yield or profit estimates.
- Trust labels: demo data, no live data, and human control are visible.

This is a test of information architecture and vocabulary, not a field-ready product claim.

## Responsibilities across the stack

- Sol Atlas: place-based view, field/paddock context, time, environmental layers, and scenario visualization.
- Symthaea: optional decision support or anomaly triage with source provenance, freshness, uncertainty/calibration where available, and inspectable rationale. It should abstain when prerequisites are missing.
- Symtropy: counterfactual scenarios only when physical assumptions, units, and calibration are explicit. A simulation is not a validated forecast.
- Mycelix: identity, permissions, provenance, consented sharing, cooperative coordination, and auditability. Farmers should not need to learn blockchain or currency concepts to finish a routine job.
- Interoperability: use open exports and adapters. Assess the AgGateway ADAPT Standard for agricultural production-data exchange rather than inventing a closed canonical format: https://github.com/AgGateway-ADAPT/Standard

## Existing cross-project work to compose

This UI is the **operator-facing layer**, not a replacement for the existing agriculture workstreams. Keep the canonical model and validation logic in their current owners:

- Sol Atlas site evidence and local climate/food resilience profile: [AGRO-CLIMATE-001 #107](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/107).
- Shared agricultural research and systems boundaries: [Symthaea draft research PR #7796](https://github.com/Luminous-Dynamics/symthaea/pull/7796).
- Robot feasibility/platform selection and safety evidence: [Symthaea #7797](https://github.com/Luminous-Dynamics/symthaea/issues/7797), [Symthaea #7818](https://github.com/Luminous-Dynamics/symthaea/issues/7818), [AGHRI evaluation #7828](https://github.com/Luminous-Dynamics/symthaea/issues/7828).
- Measurement-first value of information and spatially conditioned farm-service economics: [Symtropy #1661](https://github.com/Luminous-Dynamics/symtropy/issues/1661), [Symtropy #1814](https://github.com/Luminous-Dynamics/symtropy/issues/1814), [Symtropy #1816](https://github.com/Luminous-Dynamics/symtropy/issues/1816).
- Producer commitments, product units, cooperative fulfillment, and private/public data boundaries: [Mycelix MYC-FOOD-001 #5001](https://github.com/Luminous-Dynamics/mycelix/issues/5001).

The route should consume versioned outputs from these contracts when they become available; it must not create a second geospatial truth, safety case, economic optimizer, producer ledger, or farm identity system. UI integration is blocked when a source contract is missing or unqualified; demo data remains visibly synthetic.

## Proposed benchmark ladder

These are pilot gates to validate, not claims of current performance or universal agricultural targets.

### Usability and value
- At least 8/10 first-time pilot users complete each critical task without coaching: record an observation, find current field/paddock status, inspect recommendation evidence, and export their data.
- Median time to capture a simple observation: at most 30 seconds after onboarding.
- Median daily brief preparation: at most 2 minutes, with an obvious way to inspect why a job appears.
- At least 70% of participating operators voluntarily use the selected workflow weekly by week 4. Publish denominator, farm type, and reasons for non-use.
- Interview non-frequent users and workers, not only enthusiastic owners.

### Reliability and data integrity
- Zero silent loss of a saved observation across offline/online transition tests.
- Every queued write has a visible status and stable identifier; retry and conflict tests are deterministic and auditable.
- 100% of displayed values identify whether they are observed, estimated, stale, missing, or synthetic demo data.
- Missing prerequisites result in “cannot assess yet,” not a confident recommendation.
- Complete export/re-import preserves timestamps, units, source, correction history, and permissions metadata.
- No externally consequential action can be triggered by demo or unvalidated model output.

### Economic and environmental outcomes
- Choose one primary outcome per pilot with the farmer: e.g. labour time per task, irrigation water or energy per hectare, feed cost per kg gain, grazing recovery compliance, crop loss, or gross margin per hectare.
- Record baseline and relevant covariates. Use a matched comparison, phased rollout, or another credible reference where feasible.
- Report absolute results, uncertainty, adverse outcomes, implementation cost, farmer time, and data gaps. Before/after coincidence alone is not evidence that AI improved yield.
- Aim for a practically meaningful improvement in a farm-selected metric without breaching agreed animal-welfare, safety, or soil/water guardrails. Set the effect threshold after baseline measurement; do not force one percentage across very different farm systems.

### System performance
- Proposed local-capture feedback target: p95 under 200 ms on the pilot device.
- Proposed first useful view target: under 3 seconds on an agreed reference phone/tablet and declared test network; measure offline launch separately.
- Core workflow works during a declared offline window; start testing at 24 hours and extend.
- Measure energy, storage growth, sync bytes, recovery, and performance on low-cost devices before adding larger models or continuous sensors.

### Trust and governance
- A farmer can find data export and revoke future sharing without contacting the developer.
- Every cross-party share has a purpose, recipient, scope, and timestamp.
- Zero unresolved critical permissions or data-loss findings before a live pilot.
- Publish data dictionary and model/rule cards including limits, provenance, version, uncertainty, and escalation procedure.
- Track overrides, skipped recommendations, abstentions, and complaints rather than hiding them as negative engagement.

## Field research plan

**Discovery:** recruit a varied group across small and commercial operations, crop and livestock, intensive and extensive grazing, connectivity conditions, and owners plus workers/advisers. Ask people to demonstrate their last real workflow instead of merely asking what features they want.

Observe: time-sensitive expensive decisions; current notebooks/spreadsheets/chat/machinery displays; phone use with gloves, dirt, rain, heat, low battery, and no signal; information producers refuse to share; evidence required before acting on advice; and repetitive data entry that should not be duplicated.

**Prototype sessions:** give operators fictional tasks in this interface without explaining the intended route. Measure task completion, time, misunderstandings, trust-label comprehension, correction behavior, and what the user chooses not to enter. Aesthetic preference is not a substitute for task success.

**Small pilot:** select a repeatable, low-safety-risk workflow with a measurable cost. Start with observation → task assignment → completion record or a human-approved paddock plan. Do not start with autonomous animal-health or equipment control.

Before collection, agree on ownership, retention, access, sharing, support, incident response, outcome metrics, and an easy exit process.

**Release gates:** (1) operator comprehension, (2) deterministic offline/capture/export tests using synthetic fixtures, (3) permissions/security review, (4) supervised field pilot with baseline and adverse-outcome tracking, (5) broader deployment only after results and failure modes are published.

## Anti-goals and risks

- No claim of “revolutionary” outcomes without independently reviewed field results.
- Do not commercialize farm observations without explicit terms and a clear producer benefit.
- Do not assume remote sensing, weather, sensors, or tags are always available or correct.
- Never present synthetic data as live.
- Phi, engagement, or a general intelligence score is not a proxy for agricultural truth, yield, welfare, or trustworthiness.
- Do not optimize only for high-capital farms.
- Do not add marketplace, token, robot, or autonomous-control features before one daily workflow reliably helps.

## References

1. Lee, T., Ramp, D., & Bless, A. (2025). “Unlocking digital twin planning for grazing industries with farmer centred design.” Agriculture and Human Values 42, 2055–2075. 36 interviews with prototype sessions; map and work practices, irregular entry, and management intensity were key themes. https://doi.org/10.1007/s10460-025-10752-x
2. FAO (2022). “Leveraging automation and digitalization for precision agriculture: Evidence from the case studies.” Cost, knowledge/skills, infrastructure, and farm diversity shape adoption. https://doi.org/10.4060/cc2912en
3. FAO (2022). “Agricultural digitalization and automation in low- and middle-income countries: Evidence from ten case studies.” Cost, digital skills, and enabling infrastructure are key adoption constraints. https://doi.org/10.4060/cc2914en
4. AgGateway ADAPT Standard, data schema and type definitions for agricultural production-data exchange. https://github.com/AgGateway-ADAPT/Standard
5. FAO AgriTech Observatory, “Smallholder-Oriented Data Governance Principles.” https://agritechobservatory.review.fao.org/en/smallholder-oriented-data-governance-principles

## Current acceptance status

- [x] Research-informed information architecture documented.
- [x] Rust/Leptos concept route added; sample values are clearly illustrative.
- [x] No-live-integration and human-control notices included.
- [ ] Rust/WASM compile and browser rendering verified.
- [ ] Keyboard/screen-reader and responsive browser review completed.
- [ ] Farmer/operator usability sessions completed.
- [ ] Offline persistence, real provenance, and field integrations implemented.
- [ ] Outcome pilot completed and independently reviewed.
