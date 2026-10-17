// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! One vital sign, not a stat row. The HUD is a small breathing bloom —
//! the loading screen's consciousness motif carried into the living app —
//! whose pulse period tracks planetary coherence (population-weighted Φ
//! across earth regions). Node/corridor counts were debug telemetry and
//! now live where they belong: nowhere prominent.

use leptos::prelude::*;

use crate::state::data_state::DataState;
use crate::state::globe_state::GlobeState;

#[component]
pub fn Hud() -> impl IntoView {
    let data_state = expect_context::<DataState>();
    let globe_state = expect_context::<GlobeState>();
    let feed_status = data_state.usgs_feed_status;
    let eonet_status = data_state.eonet_feed_status;

    // Aggregate Phi from earth regions (population-weighted; None until
    // region data is present so we never render NaN).
    let global_phi = move || {
        let regions = data_state.earth_regions.read();
        let total_pop: f64 = regions.iter().map(|r| r.population_m).sum();
        if total_pop <= 0.0 {
            return None;
        }
        let weighted: f64 = regions.iter().map(|r| r.phi_mean * r.population_m).sum();
        Some(weighted / total_pop)
    };

    view! {
        <div class="vital" title="Planetary coherence (population-weighted mean Φ, curated regional estimates)">
            <span
                class="vital-bloom"
                // Higher coherence breathes faster: 9s at Φ=0 down to ~4.5s at Φ=1
                style=move || {
                    let phi = global_phi().unwrap_or(0.4);
                    format!("--vital-period: {:.2}s", 9.0 - phi * 4.5)
                }
            />
            <span class="vital-phi">
                {move || match global_phi() {
                    Some(phi) => format!("\u{03a6} {:.2}", phi),
                    None => "\u{03a6} \u{2014}".to_string(),
                }}
            </span>
            <span
                class=move || format!("live-feed-badge {}", feed_status.get().health.css_class())
                title=move || feed_status.get().description()
                aria-live="polite"
            >
                {move || feed_status.get().health.label()}
            </span>
            <span
                class=move || format!("live-feed-badge {}", eonet_status.get().css_class())
                title=move || eonet_status.get().description()
                aria-live="polite"
            >
                {move || eonet_status.get().label()}
            </span>
            <button
                class="aesthetic-cycle"
                aria-label="Cycle visual aesthetic"
                title=move || format!("Aesthetic: {} (click to cycle)", globe_state.aesthetic.get().label())
                on:click=move |_| globe_state.cycle_aesthetic()
            >
                {"\u{25c8}"}
            </button>
        </div>
    }
}
