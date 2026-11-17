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
    let catalog: Value = serde_json::from_str(EXPRESSION_CATALOG)
        .expect("embedded home expression catalogue must be valid JSON");

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

fn current_preference(preferences: RwSignal<BTreeMap<String, String>>, item_id: &str) -> String {
    preferences.with(|entries| {
        entries
            .get(item_id)
            .cloned()
            .unwrap_or_else(|| "NOT_SELECTED".to_owned())
    })
}

fn promote_secondary_if_orphaned(entries: &mut BTreeMap<String, String>) {
    if entries.values().any(|role| role == "PRIMARY") {
        return;
    }
    if let Some(style_id) = entries
        .iter()
        .find_map(|(style_id, role)| (role == "SECONDARY").then(|| style_id.clone()))
    {
        entries.insert(style_id, "PRIMARY".to_owned());
    }
}

fn can_select_secondary(
    roles: RwSignal<BTreeMap<String, String>>,
    style_id: &str,
) -> bool {
    roles.with(|entries| {
        entries.get(style_id).is_some_and(|role| role == "SECONDARY")
            || entries
                .iter()
                .any(|(candidate, role)| candidate != style_id && role == "PRIMARY")
    })
}

fn apply_style_role(
    entries: &mut BTreeMap<String, String>,
    style_id: String,
    role: &'static str,
) {
    let was_primary = entries
        .get(&style_id)
        .is_some_and(|current| current == "PRIMARY");

    if entries
        .get(&style_id)
        .is_some_and(|current| current == role)
    {
        entries.remove(&style_id);
        if role == "PRIMARY" {
            promote_secondary_if_orphaned(entries);
        }
        return;
    }

    match role {
        "PRIMARY" => {
            entries.retain(|key, current| {
                current != "PRIMARY" && !(key == &style_id && current == "SECONDARY")
            });
            entries.insert(style_id, "PRIMARY".to_owned());
        }
        "SECONDARY" => {
            let has_other_primary = entries
                .iter()
                .any(|(candidate, current)| candidate != &style_id && current == "PRIMARY");
            if has_other_primary {
                entries.retain(|key, current| {
                    current != "SECONDARY" && !(key == &style_id && current == "PRIMARY")
                });
                entries.insert(style_id, "SECONDARY".to_owned());
            }
        }
        "AVOID" | "ACCEPTABLE" => {
            entries.insert(style_id, role.to_owned());
            if was_primary {
                promote_secondary_if_orphaned(entries);
            }
        }
        _ => {}
    }
}

fn set_style_role(roles: RwSignal<BTreeMap<String, String>>, style_id: String, role: &'static str) {
    roles.update(|entries| apply_style_role(entries, style_id, role));
}

fn apply_experience_state(
    entries: &mut BTreeMap<String, String>,
    package_id: String,
    state: &'static str,
) {
    if entries
        .get(&package_id)
        .is_some_and(|current| current == state)
    {
        entries.remove(&package_id);
    } else {
        entries.insert(package_id, state.to_owned());
    }
}

fn set_experience_state(
    states: RwSignal<BTreeMap<String, String>>,
    package_id: String,
    state: &'static str,
) {
    states.update(|entries| apply_experience_state(entries, package_id, state));
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
                        let id_primary_active = style.id.clone();
                        let id_primary_pressed = style.id.clone();
                        let id_primary_click = style.id.clone();
                        let id_secondary_active = style.id.clone();
                        let id_secondary_pressed = style.id.clone();
                        let id_secondary_click = style.id.clone();
                        let id_secondary_disabled = style.id.clone();
                        let id_avoid_active = style.id.clone();
                        let id_avoid_pressed = style.id.clone();
                        let id_avoid_click = style.id.clone();
                        let id_acceptable_active = style.id.clone();
                        let id_acceptable_pressed = style.id.clone();
                        let id_acceptable_click = style.id.clone();
                        let role_label_id = style.id.clone();

                        view! {
                            <article class="home-expression-style-card">
                                <p class="home-expression-card-category">{style.category.clone()}</p>
                                <h3>{style.name.clone()}</h3>
                                <p class="home-expression-card-description">{style.description.clone()}</p>
                                <div class="home-expression-role-row" aria-label="Style preference">
                                    <button
                                        class:active=move || current_preference(style_roles, &id_primary_active) == "PRIMARY"
                                        aria-pressed=move || (current_preference(style_roles, &id_primary_pressed) == "PRIMARY").to_string()
                                        on:click=move |_| set_style_role(style_roles, id_primary_click.clone(), "PRIMARY")
                                    >"Primary"</button>
                                    <button
                                        disabled=move || !can_select_secondary(style_roles, &id_secondary_disabled)
                                        class:active=move || current_preference(style_roles, &id_secondary_active) == "SECONDARY"
                                        aria-pressed=move || (current_preference(style_roles, &id_secondary_pressed) == "SECONDARY").to_string()
                                        on:click=move |_| set_style_role(style_roles, id_secondary_click.clone(), "SECONDARY")
                                    >"Blend"</button>
                                    <button
                                        class:active=move || current_preference(style_roles, &id_avoid_active) == "AVOID"
                                        aria-pressed=move || (current_preference(style_roles, &id_avoid_pressed) == "AVOID").to_string()
                                        on:click=move |_| set_style_role(style_roles, id_avoid_click.clone(), "AVOID")
                                    >"Avoid"</button>
                                    <button
                                        class:active=move || current_preference(style_roles, &id_acceptable_active) == "ACCEPTABLE"
                                        aria-pressed=move || (current_preference(style_roles, &id_acceptable_pressed) == "ACCEPTABLE").to_string()
                                        on:click=move |_| set_style_role(style_roles, id_acceptable_click.clone(), "ACCEPTABLE")
                                    >"Acceptable"</button>
                                </div>
                                <p class="home-expression-current-role">
                                    {move || match current_preference(style_roles, &role_label_id).as_str() {
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
                        let id_prefer_active = package.id.clone();
                        let id_prefer_click = package.id.clone();
                        let id_consider_active = package.id.clone();
                        let id_consider_click = package.id.clone();
                        let id_avoid_active = package.id.clone();
                        let id_avoid_click = package.id.clone();
                        let package_label_id = package.id.clone();

                        view! {
                            <article class="home-expression-package-card">
                                <p class="home-expression-card-category">{package.category.clone()}</p>
                                <h3>{package.name.clone()}</h3>
                                <p class="home-expression-card-description">{package.description.clone()}</p>
                                <div class="home-expression-package-actions">
                                    <button
                                        class:active=move || current_preference(experience_states, &id_prefer_active) == "PREFERRED"
                                        on:click=move |_| set_experience_state(experience_states, id_prefer_click.clone(), "PREFERRED")
                                    >"Prefer"</button>
                                    <button
                                        class:active=move || current_preference(experience_states, &id_consider_active) == "CONSIDERED"
                                        on:click=move |_| set_experience_state(experience_states, id_consider_click.clone(), "CONSIDERED")
                                    >"Consider"</button>
                                    <button
                                        class:active=move || current_preference(experience_states, &id_avoid_active) == "AVOIDED"
                                        on:click=move |_| set_experience_state(experience_states, id_avoid_click.clone(), "AVOIDED")
                                    >"Avoid"</button>
                                </div>
                                <p class="home-expression-current-role">
                                    {move || match current_preference(experience_states, &package_label_id).as_str() {
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_catalog_has_expected_style_and_experience_counts() {
        let (styles, experiences) = load_catalog();
        assert_eq!(styles.len(), 20);
        assert_eq!(experiences.len(), 26);
        assert!(styles.iter().all(|item| {
            !item.id.is_empty()
                && !item.name.is_empty()
                && !item.category.is_empty()
                && !item.description.is_empty()
        }));
        assert!(experiences.iter().all(|item| {
            !item.id.is_empty()
                && !item.name.is_empty()
                && !item.category.is_empty()
                && !item.description.is_empty()
        }));
    }

    #[test]
    fn style_roles_allow_only_one_primary_and_one_secondary() {
        let mut roles = BTreeMap::new();
        apply_style_role(&mut roles, "MODERNIST".to_owned(), "PRIMARY");
        apply_style_role(&mut roles, "SOLARPUNK".to_owned(), "SECONDARY");
        apply_style_role(&mut roles, "BIOPHILIC_ORGANIC".to_owned(), "PRIMARY");

        assert_eq!(roles.get("MODERNIST"), None);
        assert_eq!(
            roles.get("BIOPHILIC_ORGANIC").map(String::as_str),
            Some("PRIMARY")
        );
        assert_eq!(roles.get("SOLARPUNK").map(String::as_str), Some("SECONDARY"));
    }

    #[test]
    fn secondary_style_requires_a_different_primary() {
        let mut roles = BTreeMap::new();
        apply_style_role(&mut roles, "SOLARPUNK".to_owned(), "SECONDARY");
        assert!(roles.is_empty());
        assert!(!can_select_secondary(RwSignal::new(roles), "SOLARPUNK"));
    }

    #[test]
    fn removing_primary_promotes_existing_secondary() {
        let mut roles = BTreeMap::new();
        apply_style_role(&mut roles, "MODERNIST".to_owned(), "PRIMARY");
        apply_style_role(&mut roles, "SOLARPUNK".to_owned(), "SECONDARY");
        apply_style_role(&mut roles, "MODERNIST".to_owned(), "PRIMARY");
        assert_eq!(roles.len(), 1);
        assert_eq!(roles.get("SOLARPUNK").map(String::as_str), Some("PRIMARY"));
    }

    #[test]
    fn replacing_primary_with_avoid_promotes_secondary() {
        let mut roles = BTreeMap::new();
        apply_style_role(&mut roles, "MODERNIST".to_owned(), "PRIMARY");
        apply_style_role(&mut roles, "SOLARPUNK".to_owned(), "SECONDARY");
        apply_style_role(&mut roles, "MODERNIST".to_owned(), "AVOID");
        assert_eq!(roles.get("MODERNIST").map(String::as_str), Some("AVOID"));
        assert_eq!(roles.get("SOLARPUNK").map(String::as_str), Some("PRIMARY"));
    }

    #[test]
    fn selecting_avoid_or_acceptable_does_not_displace_primary_blend() {
        let mut roles = BTreeMap::new();
        apply_style_role(&mut roles, "MODERNIST".to_owned(), "PRIMARY");
        apply_style_role(&mut roles, "SOLARPUNK".to_owned(), "SECONDARY");
        apply_style_role(&mut roles, "GOTHIC_REVIVAL".to_owned(), "AVOID");
        apply_style_role(&mut roles, "BIOPHILIC_ORGANIC".to_owned(), "ACCEPTABLE");

        assert_eq!(roles.get("MODERNIST").map(String::as_str), Some("PRIMARY"));
        assert_eq!(roles.get("SOLARPUNK").map(String::as_str), Some("SECONDARY"));
        assert_eq!(roles.get("GOTHIC_REVIVAL").map(String::as_str), Some("AVOID"));
        assert_eq!(
            roles.get("BIOPHILIC_ORGANIC").map(String::as_str),
            Some("ACCEPTABLE")
        );
    }

    #[test]
    fn clicking_the_same_style_role_clears_it() {
        let mut roles = BTreeMap::new();
        apply_style_role(&mut roles, "MODERNIST".to_owned(), "PRIMARY");
        apply_style_role(&mut roles, "MODERNIST".to_owned(), "PRIMARY");
        assert!(roles.is_empty());
    }

    #[test]
    fn experience_preferences_toggle_back_to_not_selected() {
        let mut states = BTreeMap::new();
        apply_experience_state(&mut states, "PRIVATE_OBSERVATORY".to_owned(), "PREFERRED");
        assert_eq!(
            states.get("PRIVATE_OBSERVATORY").map(String::as_str),
            Some("PREFERRED")
        );
        apply_experience_state(&mut states, "PRIVATE_OBSERVATORY".to_owned(), "PREFERRED");
        assert!(states.is_empty());
    }
}
