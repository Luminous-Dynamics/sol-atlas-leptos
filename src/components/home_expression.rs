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
    include_str!("../../docs/research/home-expression/HOME_EXPRESSION_CATALOG_V0.2.json");

#[derive(Clone)]
struct StyleOption {
    id: String,
    name: String,
    category: String,
    description: String,
    reference_policy: String,
    authenticity_note: String,
    reference_sources: Vec<String>,
}

#[derive(Clone)]
struct ExperienceOption {
    id: String,
    name: String,
    category: String,
    description: String,
}

#[derive(Clone)]
struct IntentOption {
    id: String,
    name: String,
    description: String,
}

fn load_intent_options() -> (Vec<IntentOption>, Vec<IntentOption>, Vec<IntentOption>) {
    let typologies = vec![
        ("DETACHED", "Detached home", "A standalone dwelling; exact setbacks, access and services remain site-specific."),
        ("SEMI_DETACHED", "Semi-detached", "A paired dwelling arrangement with shared-boundary and acoustic questions."),
        ("ROW_TERRACE", "Row or terrace", "A connected series of dwellings with shared-wall, daylight and privacy questions."),
        ("COURTYARD", "Courtyard home", "A dwelling organized around one or more protected outdoor rooms."),
        ("DUPLEX", "Duplex", "A multi-level or paired-unit arrangement; circulation and accessibility need exact review."),
        ("APARTMENT", "Apartment", "A dwelling within a larger multi-unit building and shared-services context."),
        ("PAVILION", "Pavilion", "A dispersed or clustered set of volumes connected through outdoor or covered space."),
        ("MULTIGENERATIONAL_CLUSTER", "Multi-generational cluster", "Related private and shared spaces for multiple household members."),
        ("LIVE_WORK", "Live-work", "A dwelling combining residential and work or studio activity."),
        ("INCREMENTAL_CORE", "Incremental core", "A phased dwelling intended to support carefully planned future expansion."),
        ("RETROFIT_FIRST", "Retrofit-first", "An existing building adapted before considering replacement or new construction."),
    ]
    .into_iter()
    .map(|(id, name, description)| IntentOption { id: id.to_owned(), name: name.to_owned(), description: description.to_owned() })
    .collect();

    let materials = vec![
        ("STONE_MASONRY", "Stone masonry", "Material intent only; stone type, assembly, sourcing and structural design are unqualified."),
        ("BRICK_MASONRY", "Brick masonry", "Material intent only; unit, mortar, wall build-up and thermal properties require specification."),
        ("EARTH_MASONRY", "Earth construction", "Explore earth-based assemblies with moisture, durability, code and craft review."),
        ("TIMBER", "Timber", "Explore timber structure or finishes; species, treatment, fire and sourcing remain open."),
        ("STEEL", "Steel", "Explore steel structure or detailing; sections, protection and embodied impacts require evidence."),
        ("HYBRID_MATERIALS", "Hybrid materials", "Combine materials only after interfaces, movement, moisture and lifecycle are reviewed."),
        ("RECLAIMED_REUSED", "Reclaimed or reused", "Reuse intent does not establish availability, condition, certification or safe reuse."),
        ("REPAIRABLE_MODULAR", "Repairable modular assemblies", "Prefer assemblies designed for access, replacement and repair; evidence remains pending."),
    ]
    .into_iter()
    .map(|(id, name, description)| IntentOption { id: id.to_owned(), name: name.to_owned(), description: description.to_owned() })
    .collect();

    let environmental = vec![
        ("ORIENTATION_SOLAR_RESPONSE", "Orientation and solar response", "Site-relative intent; requires verified location, orientation and solar analysis."),
        ("EXTERNAL_SHADING", "External shading", "A shading strategy preference, not a measured reduction in heat gain."),
        ("CROSS_VENTILATION", "Cross-ventilation", "A ventilation intent requiring exact openings, wind, security and airflow analysis."),
        ("THERMAL_MASS", "Thermal mass", "A strategy preference requiring assembly, climate and operating-schedule evaluation."),
        ("DAYLIGHT_CONTROL", "Daylight and glare control", "A daylight intent requiring geometry, glazing, shading and occupancy context."),
        ("RAINWATER_HARVESTING", "Rainwater harvesting", "A water strategy requiring rainfall, catchment, storage, quality and demand data."),
        ("GREYWATER_REUSE", "Greywater reuse", "A reuse strategy requiring treatment, plumbing, health and jurisdictional review."),
        ("ON_SITE_SOLAR", "On-site solar generation", "A generation preference requiring roof/site, resource, grid and storage assessment."),
        ("PRODUCTIVE_LANDSCAPE", "Productive landscape", "Landscape intent requiring water, soil, biodiversity and maintenance context."),
        ("PASSIVE_FIRST", "Passive-first design", "A design priority, not a claim that active systems can be omitted."),
    ]
    .into_iter()
    .map(|(id, name, description)| IntentOption { id: id.to_owned(), name: name.to_owned(), description: description.to_owned() })
    .collect();

    (typologies, materials, environmental)
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
                reference_policy: item.get("reference_policy")?.as_str()?.to_owned(),
                authenticity_note: item.get("authenticity_note")?.as_str()?.to_owned(),
                reference_sources: item
                    .get("reference_sources")
                    .and_then(Value::as_array)
                    .map(|sources| {
                        sources.iter().filter_map(Value::as_str).map(str::to_owned).collect()
                    })
                    .unwrap_or_default(),
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

fn can_select_secondary_from_entries(entries: &BTreeMap<String, String>, style_id: &str) -> bool {
    entries
        .get(style_id)
        .is_some_and(|role| role == "SECONDARY")
        || entries
            .iter()
            .any(|(candidate, role)| candidate != style_id && role == "PRIMARY")
}

fn can_select_secondary(roles: RwSignal<BTreeMap<String, String>>, style_id: &str) -> bool {
    roles.with(|entries| can_select_secondary_from_entries(entries, style_id))
}

fn apply_style_role(entries: &mut BTreeMap<String, String>, style_id: String, role: &'static str) {
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
fn HomeIntentAxisPicker(
    title: &'static str,
    description: &'static str,
    options: Vec<IntentOption>,
    preferences: RwSignal<BTreeMap<String, String>>,
) -> impl IntoView {
    let selected_count = move || preferences.get().len();

    view! {
        <section class="home-expression-section">
            <div class="home-expression-section-heading">
                <div>
                    <p class="home-expression-step">{title}</p>
                    <h2>{description}</h2>
                    <p>"These are resident intent signals, not specified assemblies or performance claims."</p>
                </div>
                <span class="home-expression-count">
                    {move || format!("{} selected", selected_count())}
                </span>
            </div>
            <div class="home-expression-style-grid">
                {options.into_iter().map(|option| {
                    let id_prefer_active = option.id.clone();
                    let id_prefer_pressed = option.id.clone();
                    let id_prefer_click = option.id.clone();
                    let id_consider_active = option.id.clone();
                    let id_consider_pressed = option.id.clone();
                    let id_consider_click = option.id.clone();
                    let id_avoid_active = option.id.clone();
                    let id_avoid_pressed = option.id.clone();
                    let id_avoid_click = option.id.clone();
                    view! {
                        <article class="home-expression-style-card">
                            <p class="home-expression-card-category">"RESIDENT-AUTHORED AXIS OPTION"</p>
                            <h3>{option.name.clone()}</h3>
                            <p class="home-expression-card-description">{option.description.clone()}</p>
                            <div class="home-expression-role-row" aria-label="Intent preference">
                                <button
                                    class:active=move || current_preference(preferences, &id_prefer_active) == "PREFERRED"
                                    aria-pressed=move || (current_preference(preferences, &id_prefer_pressed) == "PREFERRED").to_string()
                                    on:click=move |_| set_experience_state(preferences, id_prefer_click.clone(), "PREFERRED")
                                >"Prefer"</button>
                                <button
                                    class:active=move || current_preference(preferences, &id_consider_active) == "CONSIDERED"
                                    aria-pressed=move || (current_preference(preferences, &id_consider_pressed) == "CONSIDERED").to_string()
                                    on:click=move |_| set_experience_state(preferences, id_consider_click.clone(), "CONSIDERED")
                                >"Consider"</button>
                                <button
                                    class:active=move || current_preference(preferences, &id_avoid_active) == "AVOIDED"
                                    aria-pressed=move || (current_preference(preferences, &id_avoid_pressed) == "AVOIDED").to_string()
                                    on:click=move |_| set_experience_state(preferences, id_avoid_click.clone(), "AVOIDED")
                                >"Avoid"</button>
                            </div>
                        </article>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </section>
    }
}

#[component]
pub fn HomeExpressionPage() -> impl IntoView {
    let (styles, experiences) = load_catalog();
    let (typology_options, material_options, environmental_options) = load_intent_options();
    let style_sources: BTreeMap<String, Vec<String>> = styles
        .iter()
        .map(|style| (style.id.clone(), style.reference_sources.clone()))
        .collect();
    let style_roles = RwSignal::new(BTreeMap::<String, String>::new());
    let experience_states = RwSignal::new(BTreeMap::<String, String>::new());
    let typology_states = RwSignal::new(BTreeMap::<String, String>::new());
    let material_states = RwSignal::new(BTreeMap::<String, String>::new());
    let environmental_states = RwSignal::new(BTreeMap::<String, String>::new());
    let secondary_weight = RwSignal::new(35u8);
    let resident_statement = RwSignal::new(String::new());

    let preview = move || {
        let roles = style_roles.get();
        let package_states = experience_states.get();
        let typology_preferences = typology_states.get();
        let material_preferences = material_states.get();
        let environmental_preferences = environmental_states.get();
        let secondary = roles.values().any(|role| role == "SECONDARY");
        let secondary_pct = secondary_weight.get();
        let primary_pct = 100u8.saturating_sub(secondary_pct);

        let grammar_style_entries: Vec<Value> = roles
            .iter()
            .map(|(style_id, role)| {
                let weight = match role.as_str() {
                    "PRIMARY" if secondary => Some(primary_pct),
                    "PRIMARY" => Some(100),
                    "SECONDARY" => Some(secondary_pct),
                    _ => None,
                };
                let mut entry = json!({
                    "style_id": style_id,
                    "role": role,
                    "resident_selected": true,
                    "source_refs": style_sources.get(style_id).cloned().unwrap_or_default()
                });
                if let Some(weight) = weight {
                    entry["blend_weight_pct"] = json!(weight);
                }
                entry
            })
            .collect();

        let typology_entries: Vec<Value> = typology_preferences
            .iter()
            .map(|(item_id, preference)| json!({"item_id": item_id, "preference": preference}))
            .collect();
        let material_entries: Vec<Value> = material_preferences
            .iter()
            .map(|(item_id, preference)| json!({"item_id": item_id, "preference": preference}))
            .collect();
        let environmental_entries: Vec<Value> = environmental_preferences
            .iter()
            .map(|(item_id, preference)| json!({"item_id": item_id, "preference": preference}))
            .collect();
        let experience_entries: Vec<Value> = package_states
            .iter()
            .map(|(package_id, preference)| {
                json!({"package_id": package_id, "preference": preference})
            })
            .collect();

        let preferred_typology_count = typology_preferences
            .values()
            .filter(|preference| preference.as_str() == "PREFERRED")
            .count();
        let typology_state = if typology_entries.is_empty() {
            "UNDECIDED"
        } else if preferred_typology_count > 1 {
            "CONFLICT_UNRESOLVED"
        } else {
            "INTENT_ONLY_NOT_EVALUATED"
        };
        let intent_axis = |entries: Vec<Value>| {
            json!({
                "state": if entries.is_empty() { "UNDECIDED" } else { "INTENT_ONLY_NOT_EVALUATED" },
                "entries": entries
            })
        };
        let expression_state = if grammar_style_entries.is_empty() {
            "UNDECIDED"
        } else {
            "INTENT_SELECTED"
        };
        let experience_state = if experience_entries.is_empty() {
            "UNDECIDED"
        } else {
            "OPTIONAL_IDEAS_ONLY"
        };

        let draft = json!({
            "schema_version": "0.2.0",
            "grammar_id": "LOCAL-HOME-EXPRESSION-DRAFT",
            "maturity": "RESIDENT_WORKING_DRAFT_NOT_VALIDATED",
            "catalog_revision": "HOME-EXPRESSION-CATALOG-0.2.0",
            "design_revision_ref": "UNBOUND_LOCAL_WORKING_DRAFT",
            "resident_statement": resident_statement.get(),
            "context": {
                "site_context_ref": null,
                "climate_profile_ref": null,
                "jurisdiction_ref": null,
                "context_state": "UNKNOWN"
            },
            "composition": {
                "spatial_typology": {"state": typology_state, "entries": typology_entries},
                "architectural_expression": {
                    "state": expression_state,
                    "entries": grammar_style_entries
                },
                "material_tectonic_intent": intent_axis(material_entries),
                "environmental_strategy_intent": intent_axis(environmental_entries),
                "experience_programme": {
                    "state": experience_state,
                    "entries": experience_entries
                }
            },
            "decision_state": "NO_RESIDENT_CHOICE_OR_DESIGN_APPROVAL_RECORDED",
            "evidence_state": "UNKNOWN_NOT_EVALUATED",
            "claim_ceiling": "INTENT_ONLY_NO_GEOMETRY_COST_PERFORMANCE_OR_APPROVAL"
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
                    "prices or engineering evidence, and they are not uploaded or saved. This page "
                    "currently edits expression and experience preferences only; the v0.2 grammar "
                    "contract also separates spatial typology, materials and environmental strategies."
                </div>
                <p class="home-expression-card-description">
                    "Historical and community-associated references are exploration prompts only. "
                    "They require source, rights, local-context and (where applicable) heritage review. "
                    "No identity or culture is inferred from a resident or site."
                </p>
            </header>

            <HomeIntentAxisPicker
                title="01 · SPATIAL TYPOLOGY"
                description="What kind of dwelling arrangement fits your life?"
                options=typology_options
                preferences=typology_states
            />

            <section class="home-expression-section">
                <div class="home-expression-section-heading">
                    <div>
                        <p class="home-expression-step">"02 · ARCHITECTURAL LANGUAGE"</p>
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
                                <p class="home-expression-card-description">
                                    {format!("{} · {}", style.reference_policy, style.authenticity_note)}
                                </p>
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

            <HomeIntentAxisPicker
                title="03 · MATERIAL AND TECTONIC INTENT"
                description="What materials and ways of building appeal to you?"
                options=material_options
                preferences=material_states
            />

            <HomeIntentAxisPicker
                title="04 · ENVIRONMENTAL STRATEGY INTENT"
                description="Which environmental strategies should the design explore?"
                options=environmental_options
                preferences=environmental_states
            />

            <section class="home-expression-section">
                <div class="home-expression-section-heading">
                    <div>
                        <p class="home-expression-step">"05 · EXPERIENCES AND SPACES"</p>
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
                        let id_prefer_pressed = package.id.clone();
                        let id_prefer_click = package.id.clone();
                        let id_consider_active = package.id.clone();
                        let id_consider_pressed = package.id.clone();
                        let id_consider_click = package.id.clone();
                        let id_avoid_active = package.id.clone();
                        let id_avoid_pressed = package.id.clone();
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
                                        aria-pressed=move || (current_preference(experience_states, &id_prefer_pressed) == "PREFERRED").to_string()
                                        on:click=move |_| set_experience_state(experience_states, id_prefer_click.clone(), "PREFERRED")
                                    >"Prefer"</button>
                                    <button
                                        class:active=move || current_preference(experience_states, &id_consider_active) == "CONSIDERED"
                                        aria-pressed=move || (current_preference(experience_states, &id_consider_pressed) == "CONSIDERED").to_string()
                                        on:click=move |_| set_experience_state(experience_states, id_consider_click.clone(), "CONSIDERED")
                                    >"Consider"</button>
                                    <button
                                        class:active=move || current_preference(experience_states, &id_avoid_active) == "AVOIDED"
                                        aria-pressed=move || (current_preference(experience_states, &id_avoid_pressed) == "AVOIDED").to_string()
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
                <p class="home-expression-step">"06 · YOUR WORDS"</p>
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
                        <p class="home-expression-step">"07 · YOUR WORKING DRAFT"</p>
                        <h2>"A record of what you expressed"</h2>
                        <p>"This updates locally as you explore. It is not yet a validated Home Freedom brief or a design proposal."</p>
                    </div>
                    <button
                        class="home-expression-reset"
                        on:click=move |_| {
                            style_roles.set(BTreeMap::new());
                            experience_states.set(BTreeMap::new());
                            typology_states.set(BTreeMap::new());
                            material_states.set(BTreeMap::new());
                            environmental_states.set(BTreeMap::new());
                            secondary_weight.set(35);
                            resident_statement.set(String::new());
                        }
                    >"Reset draft"</button>
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
        assert_eq!(styles.len(), 34);
        assert_eq!(experiences.len(), 26);
        assert!(styles.iter().all(|item| {
            !item.id.is_empty()
                && !item.name.is_empty()
                && !item.category.is_empty()
                && !item.description.is_empty()
                && !item.reference_policy.is_empty()
                && !item.authenticity_note.is_empty()
        }));
        assert!(experiences.iter().all(|item| {
            !item.id.is_empty()
                && !item.name.is_empty()
                && !item.category.is_empty()
                && !item.description.is_empty()
        }));
        assert!(styles.iter().all(|item| !item.reference_policy.is_empty()));
        assert!(styles.iter().all(|item| !item.authenticity_note.is_empty()));

        let (typologies, materials, environmental) = load_intent_options();
        assert_eq!(typologies.len(), 11);
        assert_eq!(materials.len(), 8);
        assert_eq!(environmental.len(), 10);
        assert!(typologies.iter().all(|item| !item.id.is_empty()));
        assert!(materials.iter().all(|item| !item.id.is_empty()));
        assert!(environmental.iter().all(|item| !item.id.is_empty()));
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
        assert_eq!(
            roles.get("SOLARPUNK").map(String::as_str),
            Some("SECONDARY")
        );
    }

    #[test]
    fn secondary_style_requires_a_different_primary() {
        let mut roles = BTreeMap::new();
        apply_style_role(&mut roles, "SOLARPUNK".to_owned(), "SECONDARY");
        assert!(roles.is_empty());
        assert!(!can_select_secondary_from_entries(&roles, "SOLARPUNK"));

        apply_style_role(&mut roles, "MODERNIST".to_owned(), "PRIMARY");
        assert!(can_select_secondary_from_entries(&roles, "SOLARPUNK"));
        assert!(!can_select_secondary_from_entries(&roles, "MODERNIST"));
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
        assert_eq!(
            roles.get("SOLARPUNK").map(String::as_str),
            Some("SECONDARY")
        );
        assert_eq!(
            roles.get("GOTHIC_REVIVAL").map(String::as_str),
            Some("AVOID")
        );
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
