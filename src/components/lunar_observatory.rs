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
        SiteFilter::Touchdown => candidate.proposed_roles.iter().any(|role| {
            role.contains("touchdown") || role.contains("operations")
        }),
        SiteFilter::PowerAndRelay => candidate.proposed_roles.iter().any(|role| {
            role.contains("power") || role.contains("communications") || role.contains("relay")
        }),
        SiteFilter::RoboticsAndScience => candidate.proposed_roles.iter().any(|role| {
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
    match parsed {
        Ok(register) => {
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

                        <footer class="lunar-observatory-footer">
                            "Sol Atlas presents the register read-only. Site selection, geospatial analysis, and flight certification remain separate, evidence-bearing work products."
                        </footer>
                    </div>
                </main>
            }.into_any()
        }
        Err(error) => view! {
            <main class="lunar-observatory-page">
                <div class="lunar-observatory-shell">
                    <A href="/" attr:class="lunar-back-link">"← Back to Sol Atlas globe"</A>
                    <h1>"Lunar Site Observatory unavailable"</h1>
                    <p class="lunar-boundary">"The embedded candidate register could not be parsed. No fallback or inferred ranking is being displayed."</p>
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
    let observations = candidate.supported_observations.iter().map(|item| {
        let item = item.clone();
        view! { <li>{item}</li> }
    }).collect::<Vec<_>>();
    let unresolved = candidate.unresolved.iter().map(|item| {
        let item = item.clone();
        view! { <li>{item}</li> }
    }).collect::<Vec<_>>();
    let sources = candidate.sources.iter().enumerate().map(|(index, url)| {
        let url = url.clone();
        view! {
            <a href=url target="_blank" rel="noopener noreferrer">
                {format!("Source {}", index + 1)} " ↗"
            </a>
        }
    }).collect::<Vec<_>>();
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

#[cfg(test)]
mod tests {
    use super::{LunarCandidate, SiteFilter, filter_matches};

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
        assert!(!filter_matches(&candidate, SiteFilter::RoboticsAndScience, ""));
    }

    #[test]
    fn search_covers_evidence_and_open_questions() {
        let candidate = sample_candidate();
        assert!(filter_matches(&candidate, SiteFilter::All, "illumination"));
        assert!(filter_matches(&candidate, SiteFilter::All, "ellipse"));
        assert!(!filter_matches(&candidate, SiteFilter::All, "water reserve proven"));
    }
}
