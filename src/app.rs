// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root
use leptos::prelude::*;
use leptos_router::components::{A, Route, Router, Routes};
use leptos_router::path;
use sol_atlas_core::data::DatasetLoadState;

use crate::components::dataset_health_panel::DatasetHealthPanel;
use crate::components::globe_canvas::GlobeCanvas;
use crate::components::hud::Hud;
use crate::components::info_panel::InfoPanel;
use crate::components::layer_panel::LayerPanel;
use crate::components::planet_nav::PlanetNav;
use crate::components::reactor_twin_demo::ReactorTwinDemo;
use crate::components::timeline::Timeline;
use crate::components::tooltip::Tooltip;
use crate::components::whisper::Whisper;
use crate::data::static_data;
use crate::state::data_state::DataState;
use crate::state::globe_state::GlobeState;

#[component]
pub fn App() -> impl IntoView {
    // Vitality first: battery -> torpor, circadian phase, idle homeostasis.
    // Provided above the router so every route (globe, labs) breathes with
    // the same substrate.
    crate::vitality::provide_vitality();

    view! {
        <Router>
            <Routes fallback=|| view! { <GlobeApp/> }>
                <Route path=path!("/") view=GlobeApp/>
                <Route path=path!("/labs/reactor-twin") view=ReactorTwinDemo/>
            </Routes>
        </Router>
    }
}

#[component]
fn GlobeApp() -> impl IntoView {
    // Initialize reactive state
    let globe_state = GlobeState::new();
    let data_state = DataState::new();

    // Load static data immediately (always available as fallback)
    let data_for_effect = data_state.clone();
    Effect::new(move |_| {
        let report = static_data::load_all_with_health();
        let malformed = report.health.iter()
            .filter(|item| item.state == DatasetLoadState::Malformed)
            .count();
        let valid_empty = report.health.iter()
            .filter(|item| item.state == DatasetLoadState::ValidEmpty)
            .count();
        log::info!(
            "Static data: {} sites, {} geothermal, {} corridors, {} vaults, {} fossil deposits, {} nuclear; {} datasets valid-empty; {} malformed",
            report.data.sites.len(),
            report.data.geothermal_nodes.len(),
            report.data.maglev_corridors.len(),
            report.data.resontia_vaults.len(),
            report.data.fossil_deposits.len(),
            report.data.nuclear_sites.len(),
            valid_empty,
            malformed,
        );
        for item in report.health.iter().filter(|item| item.state == DatasetLoadState::Malformed) {
            log::warn!("Static dataset '{}' is malformed: {}", item.dataset, item.diagnostic.as_deref().unwrap_or("no parser diagnostic"));
        }
        data_for_effect.set_all_with_health(report);
    });

    // When holochain feature is enabled, try to fetch live data and merge
    #[cfg(feature = "holochain")]
    {
        let data_for_hc = data_state.clone();
        Effect::new(move |_| {
            let ds = data_for_hc.clone();
            wasm_bindgen_futures::spawn_local(async move {
                use crate::data::holochain;
                // Try each data source — on success, replace static data
                let sites = holochain::fetch_all_sites().await;
                if !sites.is_empty() {
                    ds.sites.set(sites);
                }
                let nodes = holochain::fetch_geothermal_nodes().await;
                if !nodes.is_empty() {
                    ds.geothermal_nodes.set(nodes);
                }
                let corridors = holochain::fetch_maglev_corridors().await;
                if !corridors.is_empty() {
                    ds.maglev_corridors.set(corridors);
                }
                let vaults = holochain::fetch_vaults().await;
                if !vaults.is_empty() {
                    ds.resontia_vaults.set(vaults);
                }
                let tl = holochain::fetch_terra_lumina_sites().await;
                if !tl.is_empty() {
                    ds.terra_lumina_sites.set(tl);
                }
                let deposits = holochain::fetch_fossil_deposits().await;
                if !deposits.is_empty() {
                    ds.fossil_deposits.set(deposits);
                }
            });
        });
    }

    // Provide state via context
    provide_context(globe_state.clone());
    provide_context(data_state.clone());

    view! {
        <GlobeCanvas />
        <div class="globe-title">
            <h1>"Sol Atlas"</h1>
            <p class="subtitle">"Symthaea Planetary Coordination Layer"</p>
            <A href="/labs/reactor-twin" attr:class="labs-link">"Labs: Reactor Digital Twin demo \u{2192}"</A>
        </div>
        <Hud />
        <LayerPanel />
        <DatasetHealthPanel />
        <Timeline />
        <Whisper />
        <Tooltip />
        <InfoPanel />
        <PlanetNav />
    }
}
