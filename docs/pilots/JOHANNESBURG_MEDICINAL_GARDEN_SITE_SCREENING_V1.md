# Johannesburg Medicinal-Garden Site Screening Contract v1

**Status:** spatial-screening proposal only; no candidate site is declared safe, approved or selected.  
**Parent:** [Johannesburg/Highveld food-resilience profile PR #108](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/108), [Earth Systems Intelligence PR #101](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/101).

## 1. What Sol Atlas should answer

Sol Atlas can compare candidate areas by asking: Is there growing space and sunlight for a declared, human-reviewed horticultural use? What is known about seasonal rain/heat, water access, power, land permission and accessibility? Are there pollution pathways, flood/fire risks, invasive-species concerns, conservation constraints or ecological sensitivities? Which facts are measured, modelled, stale or missing? What tests or public review are needed before humans decide?

It cannot conclude that a plant is safe to consume, soil/water is contamination-free, land-use permission is unnecessary, or a species is clinically useful. Spatial proximity and satellite imagery provide context, not lab results or legal permission.

## 2. Reuse the existing agroclimate foundation

Build on the current Johannesburg/Highveld food-resilience profile and existing `sol-atlas-core` evidence/dataset contracts. Preserve study geometry, CRS/axis-order details, source version, artifact digest, validity period and independent review. Do not create a competing boundary, climate profile, dataset-health state machine or generic score.

Reference `agroclimate-johannesburg-highveld-v1` and keep screening-specific state additive. If a site is sensitive (private property, school, refuge, cultural site or threatened species location), generalize or access-control its coordinates. A public map must not expose household locations by default.

## 3. Screening dimensions — never collapse them into one score

### A. Site and lawful use
- Candidate polygon, declared resolution and positional uncertainty.
- Site custodian, access and permission-review reference.
- Land-use zoning/lease/owner-consent state with authority, version, effective date and scope.
- Accessibility, drainage/flood context and external setbacks where defined.

States: `Reviewed`, `Unknown`, `Blocked`, `Stale`, `Conflicting` or `OutOfScope`. A map layer alone never establishes permission.

### B. Water quantity and quality
- Source identity and lawful-access review.
- Seasonal quantity/reliability, storage and delivery constraints.
- Sampling plan and sample reference, chain of custody, date, laboratory/method, analytes, detection limits and reviewer disposition.
- External determination of suitability for the stated horticultural use.

Imagery, an upstream pollution report or a neighboring garden sample cannot certify this site. Missing/failed assessment blocks the claim that depends on it.

### C. Soil and contamination pathways
- Soil sampling campaign and representative sampling basis.
- Land-use history and plausible pathways (industrial activity, fill, traffic/runoff, mine-related material) based on current sources and expert review.
- Analytical report, analytes/methods, detection limits and exact supported spatial extent.
- Remediation/retesting references only where an authorized plan exists.

A fertility map is not contamination clearance. Do not combine fertility, contamination and land permission into one “safe soil” rating.

### D. Climate and growing conditions
- Source-backed seasonal windows and uncertainty; preserve differences among climate normals, forecast and actual observation.
- Sun/shade, slope/aspect, drainage and microclimate evidence where available.
- Declared system (open bed, raised bed, protected structure, container, etc.).
- Crop choice and any medicinal designation must come from a reviewed external catalogue; the model does not generate prescriptions.

Regional drought/El Niño outlooks motivate readiness, not deterministic plot-level forecasts.

### E. Biodiversity and knowledge rights
- Taxonomic identity and conservation-status references from competent sources.
- Invasive potential, propagation source, lawful access/collection and ecological-impact review.
- Separate reviewed rights/permit/benefit-sharing evidence where activity requires it.
- Sensitive traditional knowledge and rare-species coordinates stay non-public unless rights holders authorize bounded disclosure.

Permission to cultivate does not automatically permit bioprospecting or publication of related knowledge.

### F. Community operating capacity
- Maintenance coverage, skills/training and accessible participation.
- Tools/equipment, storage, power and connectivity dependencies.
- Operating budget and maintenance burden.
- Governance, grievance/withdrawal routes and stewardship obligations.

These are operations/resource facts, not a medicine safety rating.

## 4. Result contract

A future `GardenSiteScreeningV1` should be a vector of scoped assessments, not a scalar:

- site ID/generation and parent agroclimate profile reference;
- exact geometry snapshot reference and intended horticultural purpose;
- dimensions A–F, each with state, evidence, scope, observation/review dates, uncertainty and explanation;
- explicit veto/blocker and unresolved-information lists;
- source lineage, data-reuse/license review and correction/supersession references;
- independent review metadata;
- claim ceiling: “screening result for declared horticultural purpose,” never “safe medicinal site,” “medicine-grade material” or “treatment approved.”

No dimension inherits a sibling status. Good climate does not offset failed soil evidence; a clean sample does not grant land permission; legal access does not prove pharmacology.

## 5. Data quality and independence

Keep raw sample records and sensitive site identifiers under the source owner's access policy. The shared Atlas layer needs only minimal evidence refs, appropriate spatial resolution, assessment state, timestamps and content identity required for reproducibility.

Require:
- exact artifact/source version references, with captured bytes available where lawful;
- coordinate/CRS/axis-order validation;
- observation date distinct from retrieval date, plus a freshness/recheck policy per evidence class;
- duplicate/derivative-feed detection so copies of one measurement do not become independent confirmations;
- separate classes for field observation, remote-sensing inference, model projection and lab result;
- independent recomputation of spatial aggregates;
- no positive default when data are missing.

A digest establishes content identity only when computed over captured bytes; it does not establish source authenticity, truth, rights or suitability.

## 6. Johannesburg pilot sequence

1. Start with one consenting institutional/community site and coarse-public geometry, not household plots.
2. Confirm land access/use with the site custodian and retain the evidence reference.
3. Have a qualified environmental/agricultural partner define soil/water sampling before site approval. Sol Atlas does not choose thresholds or interpret lab safety.
4. Freeze boundary and dataset artifacts using the Highveld profile's source-review process.
5. Assess seasonal water reliability, drainage, shade, accessibility and maintenance capability.
6. Ask botanical/conservation experts and community knowledge holders to review the permitted crop catalogue, with rights/non-disclosure conditions first.
7. Publish only generalized status evidence approved by the operator; keep detailed samples, agreements and knowledge records under their proper controls.
8. Re-review after suspected contamination, flood, land-use change, source withdrawal, major infrastructure change or evidence expiry.

No live coordinates, soil/water samples, lab artifacts or institutional partnerships are represented here. This is a proposed pilot, not proof that a site or partner has been secured.

## 7. Hard stops

Return structured `Blocked` or `Unknown` if site permission is unresolved; required soil/water assessment is absent/failed/stale; a contamination path is unreviewed; species identity/conservation status/source is unresolved; relevant research/bioprospecting lacks required external authorization; evidence comes from the wrong site/sample/generation; reuse is restricted; or a model claims consumption/therapy/pharmaceutical safety.

Do not display a green badge when a necessary dimension is missing.

## 8. References

- [Johannesburg/Highveld food-resilience profile](JOHANNESBURG_HIGHVELD_FOOD_RESILIENCE.md)
- [SANBI medicinal garden and safety caution](https://www.sanbi.org/gardens/free-state/tours/medicinal-garden/)
- [National Environmental Management: Biodiversity Act 10 of 2004](https://www.gov.za/ss/documents/national-environmental-management-biodiversity-act-0)
- [DFFE BABS clearing house](https://www.dffe.gov.za/bc_projectsprogrammes_babs)
- [SAHPRA licence to manufacture/import/export](https://www.sahpra.org.za/document/licence-to-manufacture-import-or-export/)
- [WHO herbal-medicine contaminants and residues guidance](https://www.who.int/publications/i/item/9789241594448)
