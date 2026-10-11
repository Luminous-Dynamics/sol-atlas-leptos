// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Allow-listed Rust/WASM transport for bounded public JSON feeds.
//! Callers cannot provide an arbitrary URL or relax the byte/time limits.

use gloo_timers::callback::Timeout;
use sol_atlas_core::{eonet, usgs};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AbortController, ReadableStreamDefaultReader, RequestInit, Response};

const REQUEST_TIMEOUT_MS: u32 = 12_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveFeedSource {
    UsgsEarthquakes,
    NasaEonet,
}

impl LiveFeedSource {
    fn configuration(self) -> (&'static str, usize) {
        match self {
            Self::UsgsEarthquakes => (usgs::USGS_M1_DAY_FEED_URL, usgs::MAX_FEED_BYTES),
            Self::NasaEonet => (eonet::EONET_OPEN_EVENTS_URL, eonet::MAX_EONET_BODY_BYTES),
        }
    }
}

/// Fetch an allow-listed feed and cap the body while streaming it into Rust
/// memory. Content-Length is an early rejection hint only; the streaming cap
/// remains authoritative for compressed or chunked responses.
pub async fn fetch_feed(source: LiveFeedSource) -> Result<String, String> {
    let (url, maximum_bytes) = source.configuration();
    let window = web_sys::window().ok_or_else(|| "browser window unavailable".to_string())?;
    let controller = AbortController::new()
        .map_err(|_| "could not create request abort controller".to_string())?;
    let mut init = RequestInit::new();
    init.set_signal(Some(&controller.signal()));

    let timeout_controller = controller.clone();
    let timeout = Timeout::new(REQUEST_TIMEOUT_MS, move || {
        timeout_controller.abort();
    });

    let response_value = JsFuture::from(window.fetch_with_str_and_init(url, &init))
        .await
        .map_err(|_| format!("{source:?} request failed or timed out"))?;
    let response: Response = response_value
        .dyn_into()
        .map_err(|_| format!("{source:?} request returned a non-Response value"))?;

    if !response.ok() {
        controller.abort();
        return Err(format!("{source:?} returned HTTP {}", response.status()));
    }
    if response
        .headers()
        .get("content-length")
        .ok()
        .flatten()
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|length| length > maximum_bytes)
    {
        controller.abort();
        return Err(format!("{source:?} response exceeded the {maximum_bytes}-byte limit"));
    }

    let body_stream = response
        .body()
        .ok_or_else(|| format!("{source:?} response had no readable body"))?;
    let reader: ReadableStreamDefaultReader = body_stream
        .get_reader()
        .dyn_into()
        .map_err(|_| format!("{source:?} body has no default reader"))?;
    let mut bytes = Vec::with_capacity(64 * 1024);

    loop {
        let read_result = JsFuture::from(reader.read())
            .await
            .map_err(|_| format!("{source:?} response stream failed or timed out"))?;
        let done = js_sys::Reflect::get(
            &read_result,
            &wasm_bindgen::JsValue::from_str("done"),
        )
        .map_err(|_| format!("{source:?} stream returned an invalid read result"))?
        .as_bool()
        .ok_or_else(|| format!("{source:?} stream omitted the read completion flag"))?;
        if done {
            break;
        }

        let chunk_value = js_sys::Reflect::get(
            &read_result,
            &wasm_bindgen::JsValue::from_str("value"),
        )
        .map_err(|_| format!("{source:?} stream returned a chunk without a value"))?;
        let chunk = js_sys::Uint8Array::new(&chunk_value);
        let next_len = bytes
            .len()
            .checked_add(chunk.length() as usize)
            .ok_or_else(|| format!("{source:?} response length overflow"))?;
        if next_len > maximum_bytes {
            controller.abort();
            let _ = reader.cancel();
            return Err(format!("{source:?} response exceeded the {maximum_bytes}-byte limit"));
        }

        let start = bytes.len();
        bytes.resize(next_len, 0);
        chunk.copy_to(&mut bytes[start..]);
    }
    drop(timeout);

    String::from_utf8(bytes).map_err(|_| format!("{source:?} response was not valid UTF-8"))
}
