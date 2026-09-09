// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root
use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_router::components::{A, Route, Router, Routes};
use leptos_router::path;

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
    crate::vitality::provide_vitality();
    provide_meta_context();

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
    let globe_state = GlobeState::new();
    let data_state = DataState::new();

    let data_for_effect = data_state.clone();
    Effect::new(move |_| {
        let loaded = static_data::load_all();
        let event_records = static_data::load_natural_event_records();
        log::info!(
            "Static data: {} sites, {} geothermal, {} corridors, {} vaults, {} fossil deposits, {} nuclear, {} evidence-bearing natural events",
            loaded.sites.len(),
            loaded.geothermal_nodes.len(),
            loaded.maglev_corridors.len(),
            loaded.resontia_vaults.len(),
            loaded.fossil_deposits.len(),
            loaded.nuclear_sites.len(),
            event_records.len(),
        );
        data_for_effect.set_all(loaded);
        data_for_effect.set_natural_event_records(event_records);
    });

    #[cfg(feature = "holochain")]
    {
        let data_for_hc = data_state.clone();
        Effect::new(move |_| {
            let ds = data_for_hc.clone();
            wasm_bindgen_futures::spawn_local(async move {
                use crate::data::holochain;
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
        <Timeline />
        <Whisper />
        <Tooltip />
        <InfoPanel />
        <PlanetNav />
    }
}
