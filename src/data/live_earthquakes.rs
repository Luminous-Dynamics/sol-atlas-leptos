// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Rust-only browser adapter for the USGS recent-earthquakes feed.
//! The URL is a compile-time constant; failed refreshes never clear last-good data.

use crate::data::types::{NaturalEvent, NaturalEventType};
use crate::state::data_state::DataState;
use gloo_timers::callback::Timeout;
use gloo_timers::future::TimeoutFuture;
use sol_atlas_core::usgs::{
    FeedHealth, MAX_FEED_AGE_MS, MAX_FEED_BYTES, UsgsFeedStatus, USGS_M1_DAY_FEED_URL,
    parse_feed,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AbortController, ReadableStreamDefaultReader, RequestInit, Response};

const REFRESH_INTERVAL_MS: u32 = 75_000;
const REQUEST_TIMEOUT_MS: u32 = 12_000;

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
    let window = web_sys::window().ok_or_else(|| "browser window unavailable".to_string())?;
    let controller = AbortController::new()
        .map_err(|_| "could not create request abort controller".to_string())?;
    let mut init = RequestInit::new();
    init.set_signal(Some(&controller.signal()));

    // Bound the complete request, including body consumption.
    let timeout_controller = controller.clone();
    let timeout = Timeout::new(REQUEST_TIMEOUT_MS, move || {
        timeout_controller.abort();
    });
    let response_value = JsFuture::from(
        window.fetch_with_str_and_init(USGS_M1_DAY_FEED_URL, &init),
    )
    .await
    .map_err(|_| "USGS request failed or timed out".to_string())?;
    let response: Response = response_value
        .dyn_into()
        .map_err(|_| "USGS request returned a non-Response value".to_string())?;

    if !response.ok() {
        return Err(format!("USGS returned HTTP {}", response.status()));
    }
    if response
        .headers()
        .get("content-length")
        .ok()
        .flatten()
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|length| length > MAX_FEED_BYTES)
    {
        controller.abort();
        return Err(format!("USGS response exceeded the {MAX_FEED_BYTES}-byte limit"));
    }

    // Stream bytes into a bounded buffer. Response::text() would allocate the
    // entire body before the parser could reject an oversized chunked response.
    let body_stream = response
        .body()
        .ok_or_else(|| "USGS response had no readable body".to_string())?;
    let reader: ReadableStreamDefaultReader = body_stream
        .get_reader()
        .dyn_into()
        .map_err(|_| "USGS response body has no default reader".to_string())?;
    let mut bytes = Vec::with_capacity(64 * 1024);

    loop {
        let read_result = JsFuture::from(reader.read())
            .await
            .map_err(|_| "USGS response stream failed or timed out".to_string())?;
        let done = js_sys::Reflect::get(
            &read_result,
            &wasm_bindgen::JsValue::from_str("done"),
        )
        .map_err(|_| "USGS stream returned an invalid read result".to_string())?
        .as_bool()
        .ok_or_else(|| "USGS stream omitted the read completion flag".to_string())?;
        if done {
            break;
        }

        let chunk_value = js_sys::Reflect::get(
            &read_result,
            &wasm_bindgen::JsValue::from_str("value"),
        )
        .map_err(|_| "USGS stream returned a chunk without a value".to_string())?;
        let chunk = js_sys::Uint8Array::new(&chunk_value);
        let chunk_len = chunk.length() as usize;
        let next_len = bytes.len().checked_add(chunk_len)
            .ok_or_else(|| "USGS response length overflow".to_string())?;
        if next_len > MAX_FEED_BYTES {
            controller.abort();
            let _ = reader.cancel();
            return Err(format!("USGS response exceeded the {MAX_FEED_BYTES}-byte limit"));
        }

        let start = bytes.len();
        bytes.resize(next_len, 0);
        chunk.copy_to(&mut bytes[start..]);
    }
    drop(timeout);

    let body = String::from_utf8(bytes)
        .map_err(|_| "USGS response body was not valid UTF-8".to_string())?;
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
            source_id: Some(quake.id.clone()),
            source_url: quake.event_url.clone(),
            occurred_at_ms: Some(quake.occurred_at_ms),
            updated_at_ms: quake.updated_at_ms,
            depth_km: quake.depth_km,
        })
        .collect();

    // Replace only earthquake records. Preserve fire, storm, and volcano data.
    // Invalid/network-failed refreshes never reach this mutation path.
    let mut combined = data_state.natural_events.get();
    combined.retain(|event| event.event_type != NaturalEventType::Earthquake);
    combined.extend(live_events);
    data_state.natural_events.set(combined);

    data_state.usgs_feed_status.set(UsgsFeedStatus {
        health: FeedHealth::Live,
        last_fetched_at_ms: Some(snapshot.fetched_at_ms),
        generated_at_ms: Some(snapshot.generated_at_ms),
        event_count: snapshot.earthquakes.len(),
        skipped_features: snapshot.skipped_features,
    });
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
        Some(generated) if generated >= now_ms.saturating_sub(MAX_FEED_AGE_MS) => FeedHealth::Degraded,
        Some(_) => FeedHealth::Stale,
    };
    data_state.usgs_feed_status.set(status);
}
