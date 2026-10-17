// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Rust-only USGS refresh adapter. Validation and source policy live in sol-atlas-core.

use crate::data::live_feed_http::{LiveFeedSource, fetch_feed};
use crate::data::types::{NaturalEvent, NaturalEventType};
use crate::state::data_state::DataState;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::{Get, Set};
use sol_atlas_core::usgs::{FeedHealth, MAX_FEED_AGE_MS, UsgsFeedStatus, parse_feed};

const REFRESH_INTERVAL_MS: u32 = 75_000;

pub fn start(data_state: DataState) {
    wasm_bindgen_futures::spawn_local(async move {
        loop {
            if let Err(error) = refresh_once(&data_state).await {
                let now_ms = js_sys::Date::now() as i64;
                mark_refresh_failure(&data_state, now_ms);
                log::warn!("USGS refresh failed; retaining last accepted snapshot: {error}");
            }
            TimeoutFuture::new(REFRESH_INTERVAL_MS).await;
        }
    });
}

async fn refresh_once(data_state: &DataState) -> Result<(), String> {
    let body = fetch_feed(LiveFeedSource::UsgsEarthquakes).await?;
    let fetched_at_ms = js_sys::Date::now() as i64;
    let snapshot = parse_feed(&body, fetched_at_ms).map_err(|error| error.to_string())?;

    let live_events: Vec<NaturalEvent> = snapshot
        .earthquakes
        .iter()
        .map(|quake| NaturalEvent {
            lat: quake.latitude,
            lon: quake.longitude,
            event_type: NaturalEventType::Earthquake,
            magnitude: quake.magnitude,
            name: format!("M{:.1} — {}", quake.magnitude, quake.place),
            magnitude_unit: None,
            magnitude_description: None,
            source_name: Some("USGS ComCat".to_string()),
            data_kind: Some(crate::data::types::DataKind::Observed),
            record_temporal_origin: Some(crate::data::types::RecordTemporalOrigin::LiveFeed),
            source_id: Some(quake.id.clone()),
            source_url: quake.event_url.clone(),
            occurred_at_ms: Some(quake.occurred_at_ms),
            updated_at_ms: quake.updated_at_ms,
            depth_km: quake.depth_km,
            geometry_note: None,
        })
        .collect();

    // Replace only USGS-owned earthquake records. Bundled non-USGS earthquake
    // entries, fires, storms, and volcano inventory remain untouched.
    let mut combined = data_state.natural_events.get();
    combined.retain(|event| event.source_name.as_deref() != Some("USGS ComCat"));
    combined.extend(live_events);
    data_state.natural_events.set(combined);

    data_state.usgs_feed_status.set(UsgsFeedStatus {
        health: FeedHealth::Live,
        last_fetched_at_ms: Some(snapshot.fetched_at_ms),
        generated_at_ms: Some(snapshot.generated_at_ms),
        event_count: snapshot.earthquakes.len(),
        skipped_features: snapshot.skipped_features,
    });
    data_state.mark_changed();
    log::info!(
        "USGS feed accepted: {} earthquake(s), {} rejected feature(s), generated_at_ms={}",
        snapshot.earthquakes.len(),
        snapshot.skipped_features,
        snapshot.generated_at_ms
    );
    Ok(())
}

fn mark_refresh_failure(data_state: &DataState, now_ms: i64) {
    let mut status = data_state.usgs_feed_status.get();
    status.health = match status.generated_at_ms {
        None => FeedHealth::Unavailable,
        Some(generated) if generated >= now_ms.saturating_sub(MAX_FEED_AGE_MS) => {
            FeedHealth::Degraded
        }
        Some(_) => FeedHealth::Stale,
    };
    data_state.usgs_feed_status.set(status);
    data_state.mark_changed();
}
