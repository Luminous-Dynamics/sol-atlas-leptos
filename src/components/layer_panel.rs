// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! The layer dock: a quiet vertical arc of glyph-dots hugging the left
//! edge, replacing the former 13-checkbox panel. Active layers are lit and
//! breathe; hovering (or focusing) a dot reveals its name and provenance.
//! Scenario layers carry an amber halo — speculative fiction is visible at
//! a glance, without a docked disclaimer paragraph.

use leptos::prelude::*;

use crate::data::types::{DataKind, Layer};
use crate::state::globe_state::GlobeState;

/// Dock grouping: real observed/curated data first, scenario fiction below.
const REAL_LAYERS: [Layer; 15] = [
    Layer::Energy,
    Layer::Nuclear,
    Layer::FossilDeposits,
    Layer::Regions,
    Layer::SupplyChain,
    Layer::Climate,
    Layer::Emergency,
    Layer::Health,
    Layer::Earthquakes,
    Layer::Fires,
    Layer::Storms,
    Layer::Volcanoes,
    Layer::Chokepoints,
    Layer::Infrastructure,
    Layer::MajorCities,
];

const SCENARIO_LAYERS: [Layer; 5] = [
    Layer::Geothermal,
    Layer::Maglev,
    Layer::ResontiaVaults,
    Layer::TerraLumina,
    Layer::Robotics,
];

#[component]
fn DockDot(layer: Layer) -> impl IntoView {
    let globe_state = expect_context::<GlobeState>();
    let gs_active = globe_state.clone();
    let gs_toggle = globe_state.clone();

    let active = move || gs_active.active_layers.read().contains(&layer);
    let prov = layer.provenance();
    let is_scenario = prov.kind == DataKind::Scenario;

    let toggle = move || {
        gs_toggle.toggle_layer(layer);
        // Whisper the provenance whenever a layer wakes
        if gs_toggle.active_layers.read().contains(&layer) {
            gs_toggle
                .whisper
                .set(Some(format!("{} — {}", layer.label(), prov.summary())));
        }
    };
    let toggle_kb = toggle.clone();

    view! {
        <button
            class="dock-dot"
            class:active=active
            class:scenario=is_scenario
            aria-label=layer.label()
            aria-pressed=move || active().to_string()
            on:click=move |_| toggle()
            on:keydown=move |e| {
                if e.key() == "Enter" || e.key() == " " {
                    e.prevent_default();
                    toggle_kb();
                }
            }
        >
            <span
                class="dock-dot-core"
                style=move || {
                    if active() {
                        format!("background: {}", layer.css_color())
                    } else {
                        String::new()
                    }
                }
            />
            <span class="dock-flyout">
                <span class="dock-flyout-name">{layer.label()}</span>
                <span class="dock-flyout-prov">{prov.summary()}</span>
            </span>
        </button>
    }
}

#[component]
fn ConfluenceDot() -> impl IntoView {
    let globe_state = expect_context::<GlobeState>();
    let gs_active = globe_state.clone();
    let gs_toggle = globe_state.clone();

    let active = move || gs_active.show_confluence.get();

    let toggle = move || {
        gs_toggle.show_confluence.update(|v| *v = !*v);
        if gs_toggle.show_confluence.get() {
            gs_toggle.whisper.set(Some(
                "Confluence — where 2+ real (Observed/Curated) systems co-locate. Not a risk score."
                    .to_string(),
            ));
        }
    };
    let toggle_kb = toggle.clone();

    view! {
        <button
            class="dock-dot confluence"
            class:active=active
            aria-label="Confluence"
            aria-pressed=move || active().to_string()
            on:click=move |_| toggle()
            on:keydown=move |e| {
                if e.key() == "Enter" || e.key() == " " {
                    e.prevent_default();
                    toggle_kb();
                }
            }
        >
            <span
                class="dock-dot-core"
                style=move || {
                    if active() {
                        "background: rgb(var(--sa-confluence))".to_string()
                    } else {
                        String::new()
                    }
                }
            />
            <span class="dock-flyout">
                <span class="dock-flyout-name">"Confluence"</span>
                <span class="dock-flyout-prov">
                    "derived \u{00b7} where real systems overlap, never scenario data"
                </span>
            </span>
        </button>
    }
}

#[component]
pub fn LayerPanel() -> impl IntoView {
    let globe_state = expect_context::<GlobeState>();
    let gs_fossil = globe_state.clone();
    let gs_core = globe_state.clone();
    let gs_core2 = globe_state.clone();

    let fossil_active = move || {
        gs_fossil
            .active_layers
            .read()
            .contains(&Layer::FossilDeposits)
    };

    view! {
        <nav class="layer-dock" aria-label="Data layers">
            <div class="dock-group">
                {REAL_LAYERS.into_iter().map(|l| view! { <DockDot layer=l/> }).collect::<Vec<_>>()}
            </div>
            <div class="dock-divider" title="Below: a derived signal computed from the real layers above, not raw data"></div>
            <div class="dock-group">
                <ConfluenceDot/>
            </div>
            <div class="dock-divider" title="Below: speculative planning fiction"></div>
            <div class="dock-group">
                {SCENARIO_LAYERS.into_iter().map(|l| view! { <DockDot layer=l/> }).collect::<Vec<_>>()}
            </div>
            <div class="dock-divider"></div>
            <button
                class="dock-dot core-cutaway"
                class:active=move || gs_core.show_core.get()
                aria-label="Core cutaway"
                aria-pressed=move || gs_core.show_core.get().to_string()
                on:click=move |_| gs_core2.show_core.update(|v| *v = !*v)
            >
                <span
                    class="dock-dot-core"
                    style=move || {
                        if gs_core.show_core.get() {
                            "background: rgb(var(--sa-vital))".to_string()
                        } else {
                            String::new()
                        }
                    }
                />
                <span class="dock-flyout">
                    <span class="dock-flyout-name">"Core cutaway"</span>
                    <span class="dock-flyout-prov">"see through the ocean to the inner earth"</span>
                </span>
            </button>

            // Keep classification uncertainty visible instead of implying that
            // the legacy numerical thresholds are universal system boundaries.
            <Show when=fossil_active>
                <div
                    class="dock-eroi"
                    role="note"
                    aria-label="EROI tiers withheld until threshold methodology and system boundary are identified"
                    title="EROI tiers are withheld until the threshold methodology and energy boundary are named and versioned"
                >
                    <span class="eroi-step" style="--c: 184,134,11"></span>
                    <span class="eroi-label">"EROI TIERS WITHHELD"</span>
                </div>
            </Show>
        </nav>
    }
}
