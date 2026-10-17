// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Compact, accessible status for bundled dataset parsing.

use leptos::prelude::*;
use sol_atlas_core::data::{DatasetLoadHealth, DatasetLoadState};

use crate::state::data_state::DataState;

fn label(state: &DatasetLoadState) -> &'static str {
    match state {
        DatasetLoadState::Loaded => "Loaded",
        DatasetLoadState::PartiallyLoaded => "Partially loaded",
        DatasetLoadState::ValidEmpty => "Valid empty",
        DatasetLoadState::Malformed => "Malformed",
        DatasetLoadState::Unavailable => "Unavailable",
        DatasetLoadState::StaleCache => "Stale cache",
        DatasetLoadState::Rejected => "Rejected",
    }
}

#[component]
pub fn DatasetHealthPanel() -> impl IntoView {
    let data_state = expect_context::<DataState>();
    let health = data_state.static_dataset_health;
    let health_for_summary = health.clone();
    let health_for_rows = health.clone();

    let summary = move || {
        let items = health_for_summary.get();
        if items.is_empty() {
            return "Static assets: loading".to_string();
        }
        let loaded = items
            .iter()
            .filter(|item| item.state == DatasetLoadState::Loaded)
            .count();
        let partial = items
            .iter()
            .filter(|item| item.state == DatasetLoadState::PartiallyLoaded)
            .count();
        let empty = items
            .iter()
            .filter(|item| item.state == DatasetLoadState::ValidEmpty)
            .count();
        let malformed = items
            .iter()
            .filter(|item| item.state == DatasetLoadState::Malformed)
            .count();
        format!(
            "Static assets: {loaded} loaded · {partial} partial · {empty} valid-empty · {malformed} malformed"
        )
    };

    let rows = move || {
        health_for_rows
            .get()
            .into_iter()
            .map(|item: DatasetLoadHealth| {
                let malformed = item.state == DatasetLoadState::Malformed;
                let partially_loaded = item.state == DatasetLoadState::PartiallyLoaded;
                let state = label(&item.state);
                let rejected = item.rejected_record_count.unwrap_or(0);
                let count = match item.record_count {
                    Some(accepted) if rejected > 0 => {
                        format!("accepted: {accepted} · rejected: {rejected}")
                    }
                    Some(accepted) => format!("records: {accepted}"),
                    None => "records: unknown".to_string(),
                };
                let diagnostic = item.diagnostic.unwrap_or_default();
                view! {
                    <li class:malformed=malformed class:partially-loaded=partially_loaded>
                        <span class="data-health-name">{item.dataset}</span>
                        <span class="data-health-state">{state}</span>
                        <span class="data-health-count">{format!("records: {count}")}</span>
                        {(!diagnostic.is_empty()).then(|| view! {
                            <p class="data-health-diagnostic">{diagnostic}</p>
                        })}
                    </li>
                }
            })
            .collect_view()
    };

    view! {
        <aside class="data-health-panel" aria-label="Static data integrity" aria-live="polite">
            <details>
                <summary>{summary}</summary>
                <p class="data-health-note">
                    "Parsing status only. It does not establish source freshness, authenticity, or factual correctness."
                </p>
                <ul class="data-health-list">{rows}</ul>
            </details>
        </aside>
    }
}
