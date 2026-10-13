// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Pinned JSON wire-contract fixture for cross-repository adapter compatibility.
//!
//! The adjacent fixture is synthetic test data, not a real municipal reading.
//! Changes to serialized enum names, tagged-value shape, timestamps, or field
//! semantics should be treated as an observation-contract compatibility change.

use sol_atlas_core::utility_telemetry::{
    DataTransport, Freshness, ObservationSource, UtilityObservation,
};

#[test]
fn synthetic_v1_fixture_validates_and_round_trips_without_wire_drift() {
    let fixture = include_str!("../fixtures/utility-observation-v1.synthetic.json");
    let expected: serde_json::Value =
        serde_json::from_str(fixture).expect("the pinned JSON fixture must be valid JSON");

    let observation: UtilityObservation = serde_json::from_value(expected.clone())
        .expect("the v1 fixture must deserialize into the public Rust contract");
    observation
        .validate()
        .expect("the v1 fixture must satisfy structural and semantic validation");

    assert_eq!(observation.source, ObservationSource::ScadaReadOnlyExport);
    assert_eq!(observation.transport, DataTransport::StoreAndForward);
    assert_eq!(observation.observed_at_unix_ms, 1_000);
    assert_eq!(observation.received_at_unix_ms, 1_100);
    assert_eq!(observation.freshness_at(1_200, 500), Freshness::Fresh);

    let actual = serde_json::to_value(&observation)
        .expect("the validated observation must serialize back to JSON");
    assert_eq!(actual, expected, "serialized v1 wire representation changed");
    assert_eq!(actual["source"], "scada_read_only_export");
    assert_eq!(actual["transport"], "store_and_forward");
    assert_eq!(actual["value"]["kind"], "numeric");
    assert_eq!(actual["value"]["unit"], "kilopascal");
    assert_eq!(
        actual["provenance"]["source_id"],
        "synthetic-fixture-not-a-live-source"
    );
}
