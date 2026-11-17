// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Resident-facing architectural expression configurator.
//!
//! This route captures local inspiration preferences only. It deliberately
//! does not generate geometry, estimate cost, or qualify a building.

use leptos::prelude::*;
use leptos_router::components::A;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const EXPRESSION_CATALOG: &str =
    include_str!("../../docs/research/home-expression/HOME_EXPRESSION_CATALOG_V0.1.json");

#[derive(Clone)]
struct StyleOption {
    id: String,
    name: String,
    category: String,
    description: String,
}

#[derive(Clone)]
struct ExperienceOption {
    id: String,
    name: String,
    category: String,
    description: String,
}

fn load_catalog() -> (Vec<StyleOption>, Vec<ExperienceOption>) {
    let catalog: Value = serde_json::from_str(EXPRESSION_CATALOG).unwrap_or(Value::Null);

    let styles = catalog
        .get("style_catalog")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            Some(StyleOption {
                id: item.get("style_id")?.as_str()?.to_owned(),
                name: item.get("display_name")?.as_str()?.to_owned(),
                category: item.get("category")?.as_str()?.to_owned(),
                description: item.get("description")?.as_str()?.to_owned(),
            })
        })
        .collect();

    let experiences = catalog
        .get("experience_catalog")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            Some(ExperienceOption {
                id: item.get("package_id")?.as_str()?.to_owned(),
                name: item.get("display_name")?.as_str()?.to_owned(),
                category: item.get("category")?.as_str()?.to_owned(),
                description: item.get("description")?.as_str()?.to_owned(),
            })
        })
        .collect();

    (styles, experiences)
}

fn set_style_role(roles: RwSignal<BTreeMap<String, String>>, style_id: String, role: &'static str) {
    roles.update(|entries| {
        if entries.get(&style_id).is_some_and(|current| current == role) {
            entries.remove(&style_id);
            return;
        }

        if role == "PRIMARY" || role == "SECONDARY" {
            entries.retain(|_, current| current != role);
        }
        entries.insert(style_id, role.to_owned());
    });
}

fn set_experience_state(
    states: RwSignal<BTreeMap<String, String>>,
    package_id: String,
    state: &'static str,
) {
    states.update(|entries| {
        if entries.get(&package_id).is_some_and(|current| current == state) {
            entries.remove(&package_id);
        } else {
            entries.insert(package_id, state.to_owned());
        }
    });
}

#[component]
pub fn HomeExpressionPage() -> impl IntoView {
    let (styles, experiences) = load_catalog();
    let style_roles = RwSignal::new(BTreeMap::<String, String>::new());
    let experience_states = RwSignal::new(BTreeMap::<String, String>::new());
    let secondary_weight = RwSignal::new(35u8);
    let resident_statement = RwSignal::new(String::new());

    let preview = move || {
        let roles = style_roles.get();
        let package_states = experience_states.get();
        let secondary = roles.values().any(|role| role == "SECONDARY");
        let primary = roles.values().any(|role| role == "PRIMARY");
        let secondary_pct = secondary_weight.get();
        let primary_pct = 100u8.saturating_sub(secondary_pct);

        let style_entries: Vec<Value> = roles
            .iter()
            .map(|(style_id, role)| {
                let weight = match role.as_str() {
                    "PRIMARY" if secondary => Some(primary_pct),
                    "PRIMARY" => Some(100),
                    "SECONDARY" => Some(secondary_pct),
                    _ => None,
                };
                json!({
                    "style_id": style_id,
                    "role": role,
                    "blend_weight_pct": weight,
                    "resident_selected": true
                })
            })
            .collect();

        let package_entries: Vec<Value> = package_states
            .iter()
            .map(|(package_id, state)| {
                json!({
                    "package_id": package_id,
                    "resident_preference": state,
                    "optionality": "OPTIONAL",
                    "technical_status": "UNKNOWN_NOT_EVALUATED"
                })
            })
            .collect();

        let style_mode = if !primary {
            "NONE_UNDECIDED"
        } else if secondary {
            "BLENDED"
        } else {
            "SINGLE"
        };

        let draft = json!({
            "draft_version": "0.1.0",
            "maturity": "RESIDENT_WORKING_DRAFT_NOT_VALIDATED",
            "scope": "SINGLE_RESIDENT_EXPRESSION_EXPLORATION",
            "resident_expression": {
                "statement": resident_statement.get(),
                "style_preferences": {
                    "mode": style_mode,
                    "entries": style_entries
                },
                "experience_preferences": package_entries
            },
            "decision_state": "NO_RESIDENT_CHOICE_OR_DESIGN_APPROVAL_RECORDED",
            "claim_ceiling": "INSPIRATION_ONLY_NOT_A_BUILDABLE_PLAN",
            "persistence": "LOCAL_REACTIVE_PAGE_STATE_ONLY_NOT_UPLOADED_OR_SAVED"
        });

        serde_json::to_string_pretty(&draft).unwrap_or_else(|_| "{}".to_owned())
    };

    let selected_style_count = move || style_roles.get().len();
    let selected_experience_count = move || experience_states.get().len();

    view! {
        <main class="home-expression-page">
            <header class="home-expression-header">
                <A href="/" attr:class="home-expression-back">"← Back to Sol Atlas"</A>
                <p class="home-expression-eyebrow">"LUMINOUS DYNAMICS · HOME EXPRESSION LAB"</p>
                <h1>"Imagine the home you want to live in."</h1>
                <p class="home-expression-lead">
                    "Explore architectural languages and the spaces that make a home feel like yours. "
                    "Blend directions, reject what does not fit, and leave choices undecided."
                </p>
                <div class="home-expression-boundary">
                    <strong>"An expression tool, not a building approval."</strong>
                    " Your selections are a local working draft. They do not generate floor plans, "
                    "prices or engineering evidence, and they are not uploaded or saved."
                </div>
            </header>

            <section class="home-expression-section">
                <div class="home-expression-section-heading">
                    <div>
                        <p class="home-expression-step">"01 · ARCHITECTURAL LANGUAGE"</p>
                        <h2>"What should your home feel like?"</h2>
                        <p>"Choose one primary direction, optionally blend a second, or mark styles to avoid or keep acceptable."</p>
                    </div>
                    <span class="home-expression-count">
                        {move || format!("{} selected", selected_style_count())}
                    </span>
                </div>
                <div class="home-expression-style-grid">
                    {styles.into_iter().map(|style| {
                        let role_for_label = style_roles;
                        let role_id = style.id.clone();
                        let role = move || {
                            role_for_label.with(|entries| {
                                entries.get(&role_id).cloned().unwrap_or_else(|| "UNSET".to_owned())
                            })
                        };

                        let id_primary = style.id.clone();
                        let id_secondary = style.id.clone();
                        let id_avoid = style.id.clone();
                        let id_acceptable = style.id.clone();

                        view! {
                            <article class="home-expression-style-card">
                                <p class="home-expression-card-category">{style.category.clone()}</p>
                                <h3>{style.name.clone()}</h3>
                                <p class="home-expression-card-description">{style.description.clone()}</p>
                                <div class="home-expression-role-row" aria-label="Style preference">
                                    <button
                                        class:active=move || role() == "PRIMARY"
                                        aria-pressed=move || (role() == "PRIMARY").to_string()
                                        on:click=move |_| set_style_role(style_roles, id_primary.clone(), "PRIMARY")
                                    >"Primary"</button>
                                    <button
                                        class:active=move || role() == "SECONDARY"
                                        aria-pressed=move || (role() == "SECONDARY").to_string()
                                        on:click=move |_| set_style_role(style_roles, id_secondary.clone(), "SECONDARY")
                                    >"Blend"</button>
                                    <button
                                        class:active=move || role() == "AVOID"
                                        aria-pressed=move || (role() == "AVOID").to_string()
                                        on:click=move |_| set_style_role(style_roles, id_avoid.clone(), "AVOID")
                                    >"Avoid"</button>
                                    <button
                                        class:active=move || role() == "ACCEPTABLE"
                                        aria-pressed=move || (role() == "ACCEPTABLE").to_string()
                                        on:click=move |_| set_style_role(style_roles, id_acceptable.clone(), "ACCEPTABLE")
                                    >"Acceptable"</button>
                                </div>
                                <p class="home-expression-current-role">
                                    {move || match role().as_str() {
                                        "PRIMARY" => "Primary direction",
                                        "SECONDARY" => "Secondary blend",
                                        "AVOID" => "Explicitly avoided",
                                        "ACCEPTABLE" => "Acceptable alternative",
                                        _ => "Not yet considered",
                                    }}
                                </p>
                            </article>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <div class="home-expression-blend">
                    <div>
                        <strong>"Blend balance"</strong>
                        <p>"Used only when both a primary and secondary style are selected."</p>
                    </div>
                    <label>
                        <span>{move || format!("Primary {}% · Blend {}%", 100 - secondary_weight.get(), secondary_weight.get())}</span>
                        <input
                            type="range"
                            min="10"
                            max="90"
                            step="5"
                            prop:value=move || secondary_weight.get().to_string()
                            disabled=move || {
                                let roles = style_roles.get();
                                !roles.values().any(|role| role == "PRIMARY")
                                    || !roles.values().any(|role| role == "SECONDARY")
                            }
                            on:input=move |event| {
                                if let Ok(value) = event_target_value(&event).parse::<u8>() {
                                    secondary_weight.set(value.clamp(10, 90));
                                }
                            }
                        />
                    </label>
                </div>
            </section>

            <section class="home-expression-section">
                <div class="home-expression-section-heading">
                    <div>
                        <p class="home-expression-step">"02 · EXPERIENCES AND SPACES"</p>
                        <h2>"What would make it extraordinary?"</h2>
                        <p>"These are optional programme ideas, not promises of feasibility. Mark what attracts you and what you would rather avoid."</p>
                    </div>
                    <span class="home-expression-count">
                        {move || format!("{} considered", selected_experience_count())}
                    </span>
                </div>
                <div class="home-expression-package-grid">
                    {experiences.into_iter().map(|package| {
                        let state_signal = experience_states;
                        let state_id = package.id.clone();
                        let state = move || {
                            state_signal.with(|entries| {
                                entries.get(&state_id).cloned().unwrap_or_else(|| "NOT_SELECTED".to_owned())
                            })
                        };
                        let id_prefer = package.id.clone();
                        let id_consider = package.id.clone();
                        let id_avoid = package.id.clone();

                        view! {
                            <article class="home-expression-package-card">
                                <p class="home-expression-card-category">{package.category.clone()}</p>
                                <h3>{package.name.clone()}</h3>
                                <p class="home-expression-card-description">{package.description.clone()}</p>
                                <div class="home-expression-package-actions">
                                    <button
                                        class:active=move || state() == "PREFERRED"
                                        on:click=move |_| set_experience_state(experience_states, id_prefer.clone(), "PREFERRED")
                                    >"Prefer"</button>
                                    <button
                                        class:active=move || state() == "CONSIDERED"
                                        on:click=move |_| set_experience_state(experience_states, id_consider.clone(), "CONSIDERED")
                                    >"Consider"</button>
                                    <button
                                        class:active=move || state() == "AVOIDED"
                                        on:click=move |_| set_experience_state(experience_states, id_avoid.clone(), "AVOIDED")
                                    >"Avoid"</button>
                                </div>
                                <p class="home-expression-current-role">
                                    {move || match state().as_str() {
                                        "PREFERRED" => "Your preferred option",
                                        "CONSIDERED" => "Worth exploring",
                                        "AVOIDED" => "Explicitly avoided",
                                        _ => "Not selected",
                                    }}
                                </p>
                            </article>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            <section class="home-expression-section home-expression-intent-section">
                <p class="home-expression-step">"03 · YOUR WORDS"</p>
                <h2>"What matters most to you?"</h2>
                <p>"Describe atmosphere, daily rituals, privacy, gathering, quiet, nature, creativity or anything the catalogue cannot capture."</p>
                <textarea
                    rows="4"
                    prop:value=move || resident_statement.get()
                    placeholder="A place to gather friends, retreat into quiet, make things, watch the night sky…"
                    on:input=move |event| resident_statement.set(event_target_value(&event))
                ></textarea>
            </section>

            <section class="home-expression-section home-expression-preview-section">
                <div class="home-expression-section-heading">
                    <div>
                        <p class="home-expression-step">"04 · YOUR WORKING DRAFT"</p>
                        <h2>"A record of what you expressed"</h2>
                        <p>"This updates locally as you explore. It is not yet a validated Home Freedom brief or a design proposal."</p>
                    </div>
                </div>
                <pre class="home-expression-preview">{preview}</pre>
            </section>

            <footer class="home-expression-footer">
                <p>"Your home should reflect your life, not a universal design template."</p>
                <A href="/">"Return to Sol Atlas ↑"</A>
            </footer>
        </main>
    }
}
