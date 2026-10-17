// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Read-only lunar south-pole site observatory.
//!
//! This view intentionally renders the reviewed evidence register, not a map
//! of invented point coordinates or a scalar site ranking. Geographic layers
//! are gated on comparable, provenance-bound terrain and illumination data.

use leptos::prelude::*;
use leptos_router::components::A;
use serde::Deserialize;

const LUNAR_REGISTER_JSON: &str =
    include_str!("../../docs/data/lunar_south_pole_site_screening.v1.json");
const LUNAR_SOURCE_CATALOG_JSON: &str =
    include_str!("../../docs/data/lunar_geospatial_source_catalog.v1.json");

#[derive(Clone, Deserialize)]
struct LunarRegister {
    status: String,
    decision_rule: String,
    coordinate_policy: String,
    current_program_context: ProgramContext,
    candidates: Vec<LunarCandidate>,
    required_comparison_layers: Vec<String>,
    comparison_policy: ComparisonPolicy,
}

#[derive(Clone, Deserialize)]
struct ProgramContext {
    artemis_iii: String,
    artemis_iv: String,
    evidence_url: String,
}

#[derive(Clone, Deserialize)]
struct ComparisonPolicy {
    output: String,
    missing_data: String,
}

#[derive(Clone, Deserialize)]
struct LunarCandidate {
    id: String,
    name: String,
    status: String,
    evidence_state: String,
    proposed_roles: Vec<String>,
    supported_observations: Vec<String>,
    unresolved: Vec<String>,
    sources: Vec<String>,
    #[serde(default)]
    interpretation_boundary: Option<String>,
}

#[derive(Clone, Deserialize)]
struct LunarSourceCatalog {
    status: String,
    decision_rule: String,
    metric_policy: String,
    historical_leads: Vec<LunarHistoricalLead>,
    sources: Vec<LunarSource>,
    ingestion_sequence: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct LunarHistoricalLead {
    id: String,
    title: String,
    related_region: String,
    finding: String,
    evidence_state: String,
    source_url: String,
    time_window: String,
    #[serde(default)]
    heights_above_surface_m: Vec<f64>,
    next_action: String,
    interpretation_limits: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct LunarSource {
    id: String,
    title: String,
    publisher: String,
    url: String,
    product_families: Vec<String>,
    spatial_resolution: String,
    coordinate_reference: String,
    temporal_scope: String,
    uncertainty_and_limitations: Vec<String>,
    recommended_use: String,
    required_acceptance_checks: Vec<String>,
    candidate_crosswalk: Vec<LunarSourceCrosswalk>,
    #[serde(default)]
    sample_direct_assets: Vec<LunarDirectAsset>,
    #[serde(default)]
    asset_access_notes: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct LunarDirectAsset {
    title: String,
    url: String,
    asset_type: String,
    verification_state: String,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Clone, Deserialize)]
struct LunarSourceCrosswalk {
    candidate_id: String,
    product_site: Option<String>,
    mapping_state: String,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum SiteFilter {
    All,
    Touchdown,
    PowerAndRelay,
    RoboticsAndScience,
}

impl SiteFilter {
    fn label(self) -> &'static str {
        match self {
            Self::All => "All regions",
            Self::Touchdown => "Touchdown and operations",
            Self::PowerAndRelay => "Power and relay",
            Self::RoboticsAndScience => "Robotics and science",
        }
    }
}

fn filter_matches(candidate: &LunarCandidate, filter: SiteFilter, query: &str) -> bool {
    let search_blob = format!(
        "{} {} {} {} {} {}",
        candidate.name,
        candidate.id,
        candidate.proposed_roles.join(" "),
        candidate.evidence_state,
        candidate.supported_observations.join(" "),
        candidate.unresolved.join(" "),
    )
    .to_lowercase();

    let normalized_query = query.to_lowercase();
    if !normalized_query.is_empty() && !search_blob.contains(normalized_query.as_str()) {
        return false;
    }

    match filter {
        SiteFilter::All => true,
        SiteFilter::Touchdown => candidate
            .proposed_roles
            .iter()
            .any(|role| role.contains("touchdown") || role.contains("operations")),
        SiteFilter::PowerAndRelay => candidate
            .proposed_roles
            .iter()
            .any(|role| {
                role.contains("power") || role.contains("communications") || role.contains("relay")
            }),
        SiteFilter::RoboticsAndScience => candidate
            .proposed_roles
            .iter()
            .any(|role| {
                role.contains("robotic") || role.contains("prospecting") || role.contains("science")
            }),
    }
}

fn evidence_label(state: &str) -> &'static str {
    match state {
        "candidate_list_only" => "Public candidate list; site-level evidence still needed",
        "regional_illumination_comms_evidence_available" => {
            "Regional illumination / Earth-visibility context available"
        }
        "robotic_touchdown_outcome_available" => {
            "Robotic landing outcome available; not crew-safety proof"
        }
        "regional_science_rationale_available" => {
            "Regional science rationale available; resource grade unknown"
        }
        "historical_regional_model_evidence" => {
            "Historical regional model; re-evaluate with current inputs"
        }
        _ => "Evidence state not recognized",
    }
}

#[component]
pub fn LunarObservatory() -> impl IntoView {
    let parsed = serde_json::from_str::<LunarRegister>(LUNAR_REGISTER_JSON);
    let parsed_catalog = serde_json::from_str::<LunarSourceCatalog>(LUNAR_SOURCE_CATALOG_JSON);
    match (parsed, parsed_catalog) {
        (Ok(register), Ok(source_catalog)) => {
            let total_candidates = register.candidates.len();
            let layer_count = register.required_comparison_layers.len();
            let candidates = RwSignal::new(register.candidates);
            let filter = RwSignal::new(SiteFilter::All);
            let query = RwSignal::new(String::new());
            let status = register.status;
            let decision_rule = register.decision_rule;
            let coordinate_policy = register.coordinate_policy;
            let artemis_iii = register.current_program_context.artemis_iii;
            let artemis_iv = register.current_program_context.artemis_iv;
            let program_url = register.current_program_context.evidence_url;
            let comparison_output = register.comparison_policy.output;
            let missing_data_policy = register.comparison_policy.missing_data;
            let layers = register.required_comparison_layers;
            let catalog_status = source_catalog.status;
            let catalog_decision_rule = source_catalog.decision_rule;
            let catalog_metric_policy = source_catalog.metric_policy;
            let historical_leads = source_catalog.historical_leads;
            let source_products = source_catalog.sources;
            let ingestion_sequence = source_catalog.ingestion_sequence;

            view! {
                <main class="lunar-observatory-page">
                    <div class="lunar-observatory-shell">
                        <A href="/" attr:class="lunar-back-link">"← Back to Sol Atlas globe"</A>
                        <header class="lunar-observatory-header">
                            <p class="lunar-eyebrow">"SOL ATLAS / LUNAR SOUTH POLE"</p>
                            <h1>"Lunar Site Observatory"</h1>
                            <p class="lunar-lede">
                                "Choose a touchdown region by evidence and mission role, not by a single magic score."
                            </p>
                        </header>

                        <section class="lunar-boundary" aria-label="Decision boundary">
                            <strong>"Research register — not a site selection or safety approval"</strong>
                            <p>{decision_rule}</p>
                            <p>{coordinate_policy}</p>
                            <div class="lunar-boundary-facts">
                                <span><b>{total_candidates}</b>" regions and comparators"</span>
                                <span><b>{layer_count}</b>" required evidence layers"</span>
                                <span><b>"Unknown"</b>" is not zero"</span>
                                <span><b>"No score"</b>" and no coordinate-based winner"</span>
                            </div>
                        </section>

                        <section class="lunar-program-context">
                            <h2>"Mission context"</h2>
                            <p>{artemis_iii}</p>
                            <p>{artemis_iv}</p>
                            <a href=program_url target="_blank" rel="noopener noreferrer">
                                "NASA Artemis updates ↗"
                            </a>
                        </section>

                        <section class="lunar-explorer" aria-label="Candidate region explorer">
                            <div class="lunar-section-heading">
                                <div>
                                    <p class="lunar-eyebrow">"EXPLORE THE REGISTER"</p>
                                    <h2>"Regions by role"</h2>
                                </div>
                                <span class="lunar-register-state">{status.replace('_', " ")}</span>
                            </div>
                            <div class="lunar-filter-row" role="group" aria-label="Filter regions by mission role">
                                <button
                                    class="lunar-filter"
                                    class:active=move || filter.get() == SiteFilter::All
                                    aria-pressed=move || (filter.get() == SiteFilter::All).to_string()
                                    on:click=move |_| filter.set(SiteFilter::All)
                                >{SiteFilter::All.label()}</button>
                                <button
                                    class="lunar-filter"
                                    class:active=move || filter.get() == SiteFilter::Touchdown
                                    aria-pressed=move || (filter.get() == SiteFilter::Touchdown).to_string()
                                    on:click=move |_| filter.set(SiteFilter::Touchdown)
                                >{SiteFilter::Touchdown.label()}</button>
                                <button
                                    class="lunar-filter"
                                    class:active=move || filter.get() == SiteFilter::PowerAndRelay
                                    aria-pressed=move || (filter.get() == SiteFilter::PowerAndRelay).to_string()
                                    on:click=move |_| filter.set(SiteFilter::PowerAndRelay)
                                >{SiteFilter::PowerAndRelay.label()}</button>
                                <button
                                    class="lunar-filter"
                                    class:active=move || filter.get() == SiteFilter::RoboticsAndScience
                                    aria-pressed=move || (filter.get() == SiteFilter::RoboticsAndScience).to_string()
                                    on:click=move |_| filter.set(SiteFilter::RoboticsAndScience)
                                >{SiteFilter::RoboticsAndScience.label()}</button>
                            </div>
                            <label class="lunar-search-label" for="lunar-region-search">"Search names, roles, evidence, or open questions"</label>
                            <input
                                id="lunar-region-search"
                                class="lunar-search"
                                type="search"
                                placeholder="e.g. Malapert, landing ellipse, illumination"
                                prop:value=move || query.get()
                                on:input=move |event| query.set(event_target_value(&event))
                            />
                            <p class="lunar-match-count" aria-live="polite">
                                {move || {
                                    let selected_filter = filter.get();
                                    let search_text = query.get();
                                    let count = candidates.get().iter()
                                        .filter(|candidate| filter_matches(candidate, selected_filter, &search_text))
                                        .count();
                                    format!("{count} of {total_candidates} register records match")
                                }}
                            </p>
                            <div class="lunar-candidate-grid">
                                {move || {
                                    let selected_filter = filter.get();
                                    let search_text = query.get();
                                    let matching = candidates.get().into_iter()
                                        .filter(|candidate| filter_matches(candidate, selected_filter, &search_text))
                                        .collect::<Vec<_>>();
                                    if matching.is_empty() {
                                        vec![view! {
                                            <p class="lunar-empty-state">
                                                "No candidate records match this filter. Try a broader role or search term."
                                            </p>
                                        }.into_any()]
                                    } else {
                                        matching.into_iter()
                                            .map(|candidate| view! { <LunarCandidateCard candidate=candidate/> }.into_any())
                                            .collect::<Vec<_>>()
                                    }
                                }}
                            </div>
                            <p class="lunar-filter-note">
                                "Role filters are navigation aids, not rankings. A region can serve more than one role; suitability depends on the exact lander, epoch, approach corridor, and verified terrain."
                            </p>
                        </section>

                        <section class="lunar-evidence-layers">
                            <p class="lunar-eyebrow">"BEFORE NUMERIC COMPARISON"</p>
                            <h2>"Required evidence layers"</h2>
                            <p>"A future comparison must bind each value to its source, acquisition date, coordinate frame, units, spatial resolution, uncertainty, and evidence state."</p>
                            <ul class="lunar-layer-list">
                                {layers.into_iter().map(|layer| view! { <li>{layer.replace('_', " ")}</li> }).collect::<Vec<_>>()}
                            </ul>
                            <p class="lunar-policy-note">
                                {comparison_output} ". Missing data policy: " {missing_data_policy.replace('_', " ")} "."
                            </p>
                        </section>

                        <section class="lunar-source-catalog">
                            <p class="lunar-eyebrow">"AUTHORITATIVE INPUTS"</p>
                            <div class="lunar-section-heading">
                                <h2>"Geospatial products to ingest"</h2>
                                <span class="lunar-register-state">{catalog_status.replace('_', " ")}</span>
                            </div>
                            <p class="lunar-catalog-intro">{catalog_decision_rule}</p>
                            <p class="lunar-policy-note">{catalog_metric_policy}</p>
                            <h3 class="lunar-subsection-title">"Research leads to test — not site rankings"</h3>
                            <div class="lunar-historical-lead-grid">
                                {historical_leads.into_iter().map(|lead| view! {
                                    <LunarHistoricalLeadCard lead=lead/>
                                }).collect::<Vec<_>>()}
                            </div>
                            <h3 class="lunar-subsection-title">"Source products and ingestion requirements"</h3>
                            <div class="lunar-source-grid">
                                {source_products.into_iter().map(|source| view! {
                                    <LunarSourceCard source=source/>
                                }).collect::<Vec<_>>()}
                            </div>
                            <div class="lunar-ingestion-sequence">
                                <h3>"Recommended ingestion sequence"</h3>
                                <ol>
                                    {ingestion_sequence.into_iter().map(|step| view! { <li>{step}</li> }).collect::<Vec<_>>()}
                                </ol>
                            </div>
                        </section>

                        <footer class="lunar-observatory-footer">
                            "Sol Atlas presents the register read-only. Site selection, geospatial analysis, and flight certification remain separate, evidence-bearing work products."
                        </footer>
                    </div>
                </main>
            }.into_any()
        }
        (Err(error), _) => view! {
            <main class="lunar-observatory-page">
                <div class="lunar-observatory-shell">
                    <A href="/" attr:class="lunar-back-link">"← Back to Sol Atlas globe"</A>
                    <h1>"Lunar Site Observatory unavailable"</h1>
                    <p class="lunar-boundary">"The embedded candidate register could not be parsed. No fallback or inferred ranking is being displayed."</p>
                    <pre class="lunar-parse-error">{error.to_string()}</pre>
                </div>
            </main>
        }.into_any(),
        (_, Err(error)) => view! {
            <main class="lunar-observatory-page">
                <div class="lunar-observatory-shell">
                    <A href="/" attr:class="lunar-back-link">"← Back to Sol Atlas globe"</A>
                    <h1>"Lunar Site Observatory unavailable"</h1>
                    <p class="lunar-boundary">"The geospatial source catalogue could not be parsed. Product metadata must be corrected before this view is served."</p>
                    <pre class="lunar-parse-error">{error.to_string()}</pre>
                </div>
            </main>
        }.into_any(),
    }
}

#[component]
fn LunarCandidateCard(candidate: LunarCandidate) -> impl IntoView {
    let status_class = if candidate.status == "historical_candidate_region_comparator" {
        "historical"
    } else {
        "candidate"
    };
    let status_label = if candidate.status == "historical_candidate_region_comparator" {
        "Historical comparator"
    } else {
        "Candidate region"
    };
    let observations = candidate
        .supported_observations
        .iter()
        .map(|item| {
            let item = item.clone();
            view! { <li>{item}</li> }
        })
        .collect::<Vec<_>>();
    let unresolved = candidate
        .unresolved
        .iter()
        .map(|item| {
            let item = item.clone();
            view! { <li>{item}</li> }
        })
        .collect::<Vec<_>>();
    let sources = candidate
        .sources
        .iter()
        .enumerate()
        .map(|(index, url)| {
            let url = url.clone();
            view! {
                <a href=url target="_blank" rel="noopener noreferrer">
                    {format!("Source {}", index + 1)} " ↗"
                </a>
            }
        })
        .collect::<Vec<_>>();
    let role_text = candidate.proposed_roles.join(" · ");
    let evidence = evidence_label(&candidate.evidence_state);
    let boundary = candidate.interpretation_boundary.clone();

    view! {
        <article class="lunar-candidate-card">
            <div class="lunar-card-topline">
                <span class=format!("lunar-candidate-state {status_class}")>{status_label}</span>
                <span class="lunar-candidate-id">{candidate.id}</span>
            </div>
            <h3>{candidate.name}</h3>
            <p class="lunar-role-label">"Proposed roles"</p>
            <p class="lunar-role-text">{role_text}</p>
            <p class="lunar-evidence-state">{evidence}</p>
            {boundary.map(|text| view! { <p class="lunar-interpretation-boundary">{text}</p> })}
            <div class="lunar-card-section">
                <h4>"Supported observations"</h4>
                <ul>{observations}</ul>
            </div>
            <div class="lunar-card-section unresolved">
                <h4>"Unresolved before a decision"</h4>
                <ul>{unresolved}</ul>
            </div>
            <div class="lunar-card-sources">
                {sources}
            </div>
        </article>
    }
}

#[component]
fn LunarHistoricalLeadCard(lead: LunarHistoricalLead) -> impl IntoView {
    let heights = if lead.heights_above_surface_m.is_empty() {
        "not specified".to_string()
    } else {
        lead.heights_above_surface_m
            .iter()
            .map(|height| format!("{height} m"))
            .collect::<Vec<_>>()
            .join(" / ")
    };
    let limits = lead
        .interpretation_limits
        .iter()
        .map(|item| {
            let item = item.clone();
            view! { <li>{item}</li> }
        })
        .collect::<Vec<_>>();

    view! {
        <article class="lunar-historical-lead">
            <div class="lunar-card-topline">
                <span class="lunar-candidate-id">{lead.id}</span>
                <span class="lunar-register-state">{lead.evidence_state.replace('_', " ")}</span>
            </div>
            <h4>{lead.title}</h4>
            <p class="lunar-role-label">{lead.related_region}</p>
            <p class="lunar-historical-finding">{lead.finding}</p>
            <dl class="lunar-source-metadata">
                <dt>"Model window"</dt>
                <dd>{lead.time_window}</dd>
                <dt>"Heights above surface"</dt>
                <dd>{heights}</dd>
            </dl>
            <div class="lunar-card-section">
                <h4>"Interpretation limits"</h4>
                <ul>{limits}</ul>
            </div>
            <div class="lunar-card-section">
                <h4>"Next action"</h4>
                <p class="lunar-role-text">{lead.next_action}</p>
            </div>
            <a class="lunar-source-open" href=lead.source_url target="_blank" rel="noopener noreferrer">
                "Read source evidence ↗"
            </a>
        </article>
    }
}

#[component]
fn LunarSourceCard(source: LunarSource) -> impl IntoView {
    let product_families = source
        .product_families
        .iter()
        .map(|item| {
            let item = item.clone();
            view! { <li>{item}</li> }
        })
        .collect::<Vec<_>>();
    let limitations = source
        .uncertainty_and_limitations
        .iter()
        .map(|item| {
            let item = item.clone();
            view! { <li>{item}</li> }
        })
        .collect::<Vec<_>>();
    let acceptance_checks = source
        .required_acceptance_checks
        .iter()
        .map(|item| {
            let item = item.clone();
            view! { <li>{item}</li> }
        })
        .collect::<Vec<_>>();
    let access_notes = source
        .asset_access_notes
        .iter()
        .map(|note| {
            let note = note.clone();
            view! { <li>{note}</li> }
        })
        .collect::<Vec<_>>();
    let direct_assets = source
        .sample_direct_assets
        .iter()
        .map(|asset| {
            let title = asset.title.clone();
            let url = asset.url.clone();
            let asset_type = asset.asset_type.clone();
            let verification_state = asset.verification_state.replace('_', " ");
            let note = asset.note.clone();
            view! {
                <li class="lunar-direct-asset">
                    <div class="lunar-direct-asset-heading">
                        <a href=url target="_blank" rel="noopener noreferrer">{title}</a>
                        <span>{asset_type}</span>
                    </div>
                    <p class="lunar-asset-verification">{verification_state}</p>
                    {note.map(|text| view! { <p>{text}</p> })}
                </li>
            }
        })
        .collect::<Vec<_>>();
    let crosswalks = source
        .candidate_crosswalk
        .iter()
        .map(|entry| {
            let candidate_id = entry.candidate_id.clone();
            let product_site = entry
                .product_site
                .clone()
                .unwrap_or_else(|| "No exact product-site mapping asserted".to_string());
            let mapping_state = entry.mapping_state.replace('_', " ");
            let note = entry.note.clone();
            view! {
                <li class="lunar-crosswalk-entry">
                    <strong>{candidate_id}</strong>
                    <span>{product_site}</span>
                    <span class="lunar-crosswalk-state">{mapping_state}</span>
                    {note.map(|text| view! { <p>{text}</p> })}
                </li>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <article class="lunar-source-card">
            <div class="lunar-card-topline">
                <span class="lunar-candidate-id">{source.id}</span>
            </div>
            <h3>{source.title}</h3>
            <p class="lunar-source-publisher">{source.publisher}</p>
            <a class="lunar-source-open" href=source.url target="_blank" rel="noopener noreferrer">
                "Open source product ↗"
            </a>
            <dl class="lunar-source-metadata">
                <dt>"Resolution"</dt>
                <dd>{source.spatial_resolution}</dd>
                <dt>"Coordinate reference"</dt>
                <dd>{source.coordinate_reference}</dd>
                <dt>"Temporal scope"</dt>
                <dd>{source.temporal_scope}</dd>
            </dl>
            <div class="lunar-card-section">
                <h4>"Available products"</h4>
                <ul>{product_families}</ul>
            </div>
            <div class="lunar-card-section">
                <h4>"Asset / release links and retrieval state"</h4>
                {if direct_assets.is_empty() {
                    view! { <p class="lunar-role-text">"No asset URLs independently audited in this catalogue entry yet."</p> }.into_any()
                } else {
                    view! { <ul class="lunar-direct-asset-list">{direct_assets}</ul> }.into_any()
                }}
            </div>
            <div class="lunar-card-section unresolved">
                <h4>"Access audit notes"</h4>
                {if access_notes.is_empty() {
                    view! { <p class="lunar-role-text">"No access audit note recorded."</p> }.into_any()
                } else {
                    view! { <ul>{access_notes}</ul> }.into_any()
                }}
            </div>
            <div class="lunar-card-section">
                <h4>"Candidate coverage / crosswalk state"</h4>
                <ul class="lunar-crosswalk-list">{crosswalks}</ul>
            </div>
            <div class="lunar-card-section unresolved">
                <h4>"Limitations to preserve"</h4>
                <ul>{limitations}</ul>
            </div>
            <div class="lunar-card-section">
                <h4>"Acceptance checks before use"</h4>
                <ul>{acceptance_checks}</ul>
            </div>
            <div class="lunar-card-section">
                <h4>"Recommended use"</h4>
                <p class="lunar-role-text">{source.recommended_use}</p>
            </div>
        </article>
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LUNAR_REGISTER_JSON, LUNAR_SOURCE_CATALOG_JSON, LunarCandidate, LunarSourceCatalog,
        SiteFilter, filter_matches,
    };

    fn sample_candidate() -> LunarCandidate {
        LunarCandidate {
            id: "malapert-massif".to_string(),
            name: "Malapert Massif".to_string(),
            status: "candidate_region".to_string(),
            evidence_state: "regional_illumination_comms_evidence_available".to_string(),
            proposed_roles: vec![
                "power_node_alternative".to_string(),
                "communications_node_alternative".to_string(),
                "touchdown_alternative".to_string(),
            ],
            supported_observations: vec!["regional illumination context".to_string()],
            unresolved: vec!["exact landing ellipse".to_string()],
            sources: vec!["https://science.nasa.gov/image-detail/20100618/".to_string()],
            interpretation_boundary: None,
        }
    }

    #[test]
    fn filters_are_role_navigation_not_a_numeric_ranking() {
        let candidate = sample_candidate();
        assert!(filter_matches(&candidate, SiteFilter::Touchdown, ""));
        assert!(filter_matches(&candidate, SiteFilter::PowerAndRelay, ""));
        assert!(!filter_matches(
            &candidate,
            SiteFilter::RoboticsAndScience,
            ""
        ));
    }

    #[test]
    fn search_covers_evidence_and_open_questions() {
        let candidate = sample_candidate();
        assert!(filter_matches(&candidate, SiteFilter::All, "illumination"));
        assert!(filter_matches(&candidate, SiteFilter::All, "ellipse"));
        assert!(!filter_matches(
            &candidate,
            SiteFilter::All,
            "water reserve proven"
        ));
    }

    #[test]
    fn embedded_source_catalog_declares_real_products_and_limitations() {
        let catalog: LunarSourceCatalog = serde_json::from_str(LUNAR_SOURCE_CATALOG_JSON)
            .expect("embedded source catalogue must parse");
        assert_eq!(catalog.sources.len(), 7);

        // Bind assertions by stable source IDs, not array position. New source
        // families must not silently redirect existing regression checks.
        let topography = catalog
            .sources
            .iter()
            .find(|source| source.id == "pgda-lola-high-resolution-site-products")
            .expect("primary LOLA topography source must be present");
        let illumination = catalog
            .sources
            .iter()
            .find(|source| source.id == "pgda-lola-polar-illumination")
            .expect("LOLA illumination source must be present");
        let connecting_ridge_dtm = catalog
            .sources
            .iter()
            .find(|source| source.id == "lroc-nac-shackleton-de-gerlache-connecting-ridge-dtm")
            .expect("LROC Connecting Ridge DTM source must be present");
        let lroc_inventory = catalog
            .sources
            .iter()
            .find(|source| source.id == "lroc-global-nac-dtm-footprint-shapefiles")
            .expect("LROC global DTM inventory must be present");
        let usgs_navigation = catalog
            .sources
            .iter()
            .find(|source| source.id == "usgs-artemis-iii-candidate-navigation-grids")
            .expect("USGS candidate navigation-grid source must be present");

        assert_eq!(topography.sample_direct_assets.len(), 6);
        assert_eq!(illumination.sample_direct_assets.len(), 7);
        assert!(illumination
            .asset_access_notes
            .iter()
            .any(|note| note.contains("HTML 404")));
        assert!(illumination
            .sample_direct_assets
            .iter()
            .any(|asset| asset.url.ends_with("_JP2.LBL")));
        assert_eq!(topography.candidate_crosswalk.len(), 11);

        assert_eq!(connecting_ridge_dtm.sample_direct_assets.len(), 5);
        assert!(connecting_ridge_dtm
            .sample_direct_assets
            .iter()
            .all(|asset| !asset.verification_state.contains("downloaded_and_hashed")));
        assert_eq!(lroc_inventory.sample_direct_assets.len(), 3);
        assert_eq!(usgs_navigation.candidate_crosswalk.len(), 9);
        assert_eq!(usgs_navigation.sample_direct_assets.len(), 1);
        assert!(usgs_navigation.sample_direct_assets[0]
            .asset_type
            .contains("not a direct archive URL"));
        assert!(!usgs_navigation.sample_direct_assets[0]
            .verification_state
            .contains("downloaded_and_hashed"));

        assert!(catalog
            .sources
            .iter()
            .all(|source| !source.candidate_crosswalk.is_empty()));
        assert_eq!(catalog.ingestion_sequence.len(), 6);
        assert_eq!(catalog.historical_leads.len(), 3);
        assert!(catalog.metric_policy.contains("No derived site metrics"));
        assert!(catalog.historical_leads[0]
            .title
            .contains("Connecting Ridge"));
        assert!(catalog.historical_leads[0]
            .interpretation_limits
            .iter()
            .any(|item| item.contains("not a new measurement")));
        assert!(catalog
            .sources
            .iter()
            .any(|source| source.title.contains("High-Resolution LOLA Topography")));
        assert!(catalog.sources.iter().any(|source| {
            source
                .uncertainty_and_limitations
                .iter()
                .any(|item| item.contains("interpolated"))
        }));
    }

    #[test]
    fn embedded_register_stays_unranked_and_metric_free() {
        let register: serde_json::Value =
            serde_json::from_str(LUNAR_REGISTER_JSON).expect("embedded register must parse");
        let candidates = register["candidates"]
            .as_array()
            .expect("candidate list must be an array");
        assert_eq!(candidates.len(), 11);
        assert!(register["scoring"].is_null());
        assert!(register["comparison_policy"]["weights"].is_null());
        for candidate in candidates {
            assert!(
                candidate["known_metrics"]
                    .as_object()
                    .is_some_and(|m| m.is_empty())
            );
            assert!(candidate.get("latitude").is_none());
            assert!(candidate.get("longitude").is_none());
            assert!(candidate.get("coordinates").is_none());
            assert!(candidate.get("score").is_none());
        }
    }
}
