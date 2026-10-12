// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! External-contract tests for the public economic-intelligence API.
//!
//! These tests deliberately exercise the crate as a downstream consumer,
//! rather than relying only on private helpers inside the implementation module.

use serde::Deserialize;
use sol_atlas_core::economic_intelligence::{
    compare_mirror_pair, MirrorPairStatus, TradeAreaKind, TradeFlow, TradeObservation,
    TRADE_OBSERVATION_SCHEMA_VERSION,
};

fn observation(
    record_id: &str,
    reporter: &str,
    partner: &str,
    flow: TradeFlow,
    value_text: &str,
) -> TradeObservation {
    TradeObservation {
        schema_version: TRADE_OBSERVATION_SCHEMA_VERSION,
        record_id: record_id.to_string(),
        source_id: "synthetic-reference-fixture".to_string(),
        source_snapshot_id: "synthetic-reference-v1".to_string(),
        source_record_id: Some(format!("source-{record_id}")),
        source_url: Some("https://example.invalid/frozen-fixture".to_string()),
        reporter: reporter.to_string(),
        reporter_kind: TradeAreaKind::ReportingArea,
        partner: partner.to_string(),
        partner_kind: TradeAreaKind::ReportingArea,
        geography_code_system: "TEST_ALPHA3".to_string(),
        period: "2025".to_string(),
        commodity_code: "010121".to_string(),
        classification_revision: "HS-2022".to_string(),
        flow,
        trade_value_source_text: Some(value_text.to_string()),
        currency: "USD".to_string(),
        value_unit: "USD".to_string(),
        quantity: Some(10.0),
        quantity_unit: Some("kg".to_string()),
        source_published_at: Some("2026-01-15".to_string()),
        retrieved_at: "2026-02-01T00:00:00Z".to_string(),
    }
}

#[test]
fn public_api_preserves_source_lexeme_and_exact_gap() {
    let exporter = observation(
        "exp-1",
        "AAA",
        "BBB",
        TradeFlow::Export,
        "9007199254740992",
    );
    let importer = observation(
        "imp-1",
        "BBB",
        "AAA",
        TradeFlow::Import,
        "9007199254740993",
    );

    let serialized = serde_json::to_string(&exporter).expect("serialize observation");
    let decoded: TradeObservation =
        serde_json::from_str(&serialized).expect("round-trip observation");
    assert_eq!(
        decoded.trade_value_source_text.as_deref(),
        Some("9007199254740992")
    );

    let result = compare_mirror_pair(&[decoded], &[importer]);
    assert_eq!(result.status, MirrorPairStatus::Compared);
    assert_eq!(result.signed_difference_source_text.as_deref(), Some("1"));
    assert_eq!(result.absolute_difference_source_text.as_deref(), Some("1"));
    assert_eq!(result.currency.as_deref(), Some("USD"));
    assert_eq!(result.value_unit.as_deref(), Some("USD"));
}

#[test]
fn public_observation_contract_rejects_unknown_fields() {
    let observation = observation(
        "exp-1",
        "AAA",
        "BBB",
        TradeFlow::Export,
        "100.00",
    );
    let mut value =
        serde_json::to_value(observation).expect("serialize observation as JSON");
    value["unexpected_future_field"] = serde_json::json!("must not be silently dropped");

    assert!(
        serde_json::from_value::<TradeObservation>(value).is_err(),
        "schema drift must fail closed rather than silently discard evidence"
    );
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceCorpus {
    corpus_id: String,
    schema_version: u16,
    fixture_kind: String,
    note: String,
    cases: Vec<ReferenceCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceCase {
    case_id: String,
    exporter_value: Option<String>,
    importer_value: Option<String>,
    exporter_currency: Option<String>,
    importer_currency: Option<String>,
    exporter_value_unit: Option<String>,
    importer_value_unit: Option<String>,
    exporter_partner_kind: Option<TradeAreaKind>,
    importer_reporter_kind: Option<TradeAreaKind>,
    exporter_geography_code_system: Option<String>,
    importer_geography_code_system: Option<String>,
    expected_status: MirrorPairStatus,
    expected_signed_difference: Option<String>,
    expected_absolute_difference: Option<String>,
}

#[test]
fn frozen_synthetic_reference_corpus_matches_exact_expected_results() {
    let corpus: ReferenceCorpus = serde_json::from_str(include_str!(
        "fixtures/economic_intelligence_trade_mirror_v1.json"
    ))
    .expect("parse frozen reference corpus");
    assert_eq!(corpus.corpus_id, "economic-intelligence-trade-mirror-v1");
    assert_eq!(corpus.schema_version, 1);
    assert_eq!(corpus.fixture_kind, "synthetic");
    assert!(corpus.note.contains("not UN Comtrade records"));
    assert_eq!(corpus.cases.len(), 9);

    for case in corpus.cases {
        let exporter_id = format!("{}-export", case.case_id);
        let importer_id = format!("{}-import", case.case_id);
        let mut exporter = observation(
            &exporter_id,
            "AAA",
            "BBB",
            TradeFlow::Export,
            case.exporter_value.as_deref().unwrap_or("100"),
        );
        let mut importer = observation(
            &importer_id,
            "BBB",
            "AAA",
            TradeFlow::Import,
            case.importer_value.as_deref().unwrap_or("120"),
        );

        exporter.source_snapshot_id = "synthetic-reference-corpus-v1".to_string();
        importer.source_snapshot_id = "synthetic-reference-corpus-v1".to_string();
        if case.exporter_value.is_none() {
            exporter.trade_value_source_text = None;
        }
        if case.importer_value.is_none() {
            importer.trade_value_source_text = None;
        }
        exporter.currency = case.exporter_currency.unwrap_or_else(|| "USD".to_string());
        importer.currency = case.importer_currency.unwrap_or_else(|| "USD".to_string());
        exporter.value_unit = case.exporter_value_unit.unwrap_or_else(|| "USD".to_string());
        importer.value_unit = case.importer_value_unit.unwrap_or_else(|| "USD".to_string());
        exporter.partner_kind = case
            .exporter_partner_kind
            .unwrap_or(TradeAreaKind::ReportingArea);
        importer.reporter_kind = case
            .importer_reporter_kind
            .unwrap_or(TradeAreaKind::ReportingArea);
        exporter.geography_code_system = case
            .exporter_geography_code_system
            .unwrap_or_else(|| "TEST_ALPHA3".to_string());
        importer.geography_code_system = case
            .importer_geography_code_system
            .unwrap_or_else(|| "TEST_ALPHA3".to_string());

        let result = compare_mirror_pair(&[exporter], &[importer]);
        assert_eq!(
            result.status, case.expected_status,
            "unexpected status for fixture case {}",
            case.case_id
        );
        assert_eq!(
            result.signed_difference_source_text,
            case.expected_signed_difference,
            "unexpected signed value gap for fixture case {}",
            case.case_id
        );
        assert_eq!(
            result.absolute_difference_source_text,
            case.expected_absolute_difference,
            "unexpected absolute value gap for fixture case {}",
            case.case_id
        );
    }
}
