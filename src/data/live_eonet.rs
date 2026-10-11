// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Rust-only NASA EONET refresh adapter. Replaces only records this source owns.

use crate::data::live_feed_http::{fetch_feed, LiveFeedSource};
use crate::data::types::{NaturalEvent, NaturalEventType};
use crate::state::data_state::DataState;
use gloo_timers::future::TimeoutFuture;
use sol_atlas_core::eonet::{parse_feed, EonetEventType, EonetFeedStatus};
use sol_atlas_core::usgs::FeedHealth;

const REFRESH_INTERVAL_MS: u32 = 120_000;
const MAX_LAST_KNOWN_AGE_MS: i64 = 10 * 60 * 1_000;

pub fn start(data_state: DataState) {
    wasm_bindgen_futures::spawn_local(async move {
        loop {
            if let Err(error) = refresh_once(&data_state).await {
                let now_ms = js_sys::Date::now() as i64;
                mark_refresh_failure(&data_state, now_ms);
                log::warn!("NASA EONET refresh failed; retaining last accepted snapshot: {error}");
            }
            TimeoutFuture::new(REFRESH_INTERVAL_MS).await;
        }
    });
}

async fn refresh_once(data_state: &DataState) -> Result<(), String> {
    let body = fetch_feed(LiveFeedSource::NasaEonet).await?;
    let fetched_at_ms = js_sys::Date::now() as i64;
    let snapshot = parse_feed(&body, fetched_at_ms).map_err(|error| error.to_string())?;

    let live_events: Vec<NaturalEvent> = snapshot
        .events
        .iter()
        .map(|event| NaturalEvent {
            lat: event.latitude,
            lon: event.longitude,
            event_type: match event.event_type {
                EonetEventType::Fire => NaturalEventType::Fire,
                EonetEventType::Storm => NaturalEventType::Storm,
                EonetEventType::Volcano => NaturalEventType::Volcano,
            },
            // The value is retained for possible category-specific display only;
            // EONET measurements are not comparable across event categories.
            magnitude: event.magnitude_value.unwrap_or(1.0),
            name: event.title.clone(),
            source_name: Some("NASA EONET".to_string()),
            source_id: Some(event.id.clone()),
            source_url: event.event_url.clone(),
            occurred_at_ms: Some(event.occurred_at_ms),
            updated_at_ms: None,
            depth_km: None,
            geometry_note: Some(event.geometry_kind.label().to_string()),
        })
        .collect();

    let mut combined = data_state.natural_events.get();
    combined.retain(|event| event.source_name.as_deref() != Some("NASA EONET"));
    // A validated empty open-event catalog is a real snapshot and intentionally
    // removes expired EONET markers. Failed/invalid snapshots never reach here.
    combined.extend(live_events);
    data_state.natural_events.set(combined);
    data_state.eonet_feed_status.set(EonetFeedStatus {
        health: FeedHealth::Live,
        last_fetched_at_ms: Some(snapshot.fetched_at_ms),
        event_count: snapshot.events.len(),
        skipped_events: snapshot.skipped_events,
    });

    log::info!(
        "NASA EONET feed accepted: {} event(s), {} rejected event(s), fetched_at_ms={}",
        snapshot.events.len(),
        snapshot.skipped_events,
        snapshot.fetched_at_ms
    );
    Ok(())
}

fn mark_refresh_failure(data_state: &DataState, now_ms: i64) {
    let mut status = data_state.eonet_feed_status.get();
    status.health = match status.last_fetched_at_ms {
        None => FeedHealth::Unavailable,
        Some(fetched) if fetched >= now_ms.saturating_sub(MAX_LAST_KNOWN_AGE_MS) => {
            FeedHealth::Degraded
        }
        Some(_) => FeedHealth::Stale,
    };
    data_state.eonet_feed_status.set(status);
}
