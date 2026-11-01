use serde_json::Value;
use sol_atlas_core::port_city::PortStudyManifest;
use std::collections::BTreeSet;

const CORPUS: &str = include_str!("fixtures/port_city/manifest_cases.jsonl");

fn nonempty(value: Option<&Value>) -> bool {
    value.and_then(Value::as_str).is_some_and(|s| !s.trim().is_empty())
}

fn valid_digest(value: Option<&Value>) -> bool {
    value.and_then(Value::as_str).is_some_and(|s| {
        s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
    })
}

/// Deliberately independent JSON-value oracle. It does not call PortStudyManifest::validate.
fn reference_valid(manifest: &Value) -> bool {
    if manifest.get("schema_version").and_then(Value::as_u64) != Some(1)
        || manifest.get("claim_ceiling").and_then(Value::as_str)
            != Some("concept_description_only")
        || !nonempty(manifest.get("study_id"))
        || !nonempty(manifest.get("study_region_id"))
    {
        return false;
    }

    let Some(alternatives) = manifest.get("alternatives").and_then(Value::as_array) else {
        return false;
    };
    if alternatives.len() < 2 {
        return false;
    }

    let mut ids = BTreeSet::new();
    let mut baseline_count = 0usize;
    for alt in alternatives {
        if !nonempty(alt.get("alternative_id")) || !nonempty(alt.get("label")) {
            return false;
        }
        let Some(id) = alt.get("alternative_id").and_then(Value::as_str) else {
            return false;
        };
        if !ids.insert(id.to_owned()) {
            return false;
        }
        let Some(family) = alt.get("family").and_then(Value::as_str) else {
            return false;
        };
        const FAMILIES: &[&str] = &[
            "no_build_operational", "existing_port_upgrade", "brownfield_expansion",
            "offshore_terminal", "barrier_island_network", "inland_basin_canal",
            "inland_waterway_port", "dry_port_rail", "distributed_network",
        ];
        if !FAMILIES.contains(&family) {
            return false;
        }
        if family == "no_build_operational" {
            baseline_count += 1;
        }

        match alt.get("design_geometry") {
            None | Some(Value::Null) if family == "no_build_operational" => {}
            None | Some(Value::Null) => return false,
            Some(g) => {
                if !nonempty(g.get("artifact_id"))
                    || !valid_digest(g.get("sha256"))
                    || !nonempty(g.get("horizontal_crs"))
                    || !nonempty(g.get("linear_unit"))
                    || !matches!(g.get("axis_order").and_then(Value::as_str), Some("east_north" | "north_east"))
                {
                    return false;
                }
            }
        }

        let evidence = match alt.get("evidence") {
            None => &[][..],
            Some(v) => match v.as_array() { Some(a) => a.as_slice(), None => return false },
        };
        let mut evidence_ids = BTreeSet::new();
        for ev in evidence {
            if !nonempty(ev.get("record_id")) || !nonempty(ev.get("revision_id"))
                || !valid_digest(ev.get("sha256"))
            {
                return false;
            }
            let id = ev.get("record_id").and_then(Value::as_str).unwrap();
            if !evidence_ids.insert(id.to_owned()) {
                return false;
            }
        }

        let parameters = match alt.get("design_parameters") {
            None => &[][..],
            Some(v) => match v.as_array() { Some(a) => a.as_slice(), None => return false },
        };
        let mut parameter_ids = BTreeSet::new();
        for p in parameters {
            if !nonempty(p.get("parameter_id")) || !nonempty(p.get("unit")) {
                return false;
            }
            let id = p.get("parameter_id").and_then(Value::as_str).unwrap();
            if !parameter_ids.insert(id.to_owned()) {
                return false;
            }
            let (Some(lo), Some(hi)) = (p.get("lower").and_then(Value::as_f64), p.get("upper").and_then(Value::as_f64)) else {
                return false;
            };
            if !lo.is_finite() || !hi.is_finite() || lo > hi {
                return false;
            }
        }

        let models = match alt.get("required_models") {
            None => &[][..],
            Some(v) => match v.as_array() { Some(a) => a.as_slice(), None => return false },
        };
        let mut model_ids = BTreeSet::new();
        for m in models {
            let Some(id) = m.as_str() else { return false; };
            if !model_ids.insert(id.to_owned()) { return false; }
        }

        let assumptions = match alt.get("assumptions") {
            None => &[][..],
            Some(v) => match v.as_array() { Some(a) => a.as_slice(), None => return false },
        };
        if assumptions.iter().any(|a| !a.as_str().is_some_and(|s| !s.trim().is_empty())) {
            return false;
        }
    }
    baseline_count == 1
}

#[test]
fn frozen_port_study_corpus_matches_independent_oracle() {
    let mut seen = BTreeSet::new();
    let mut count = 0usize;
    for (line_no, line) in CORPUS.lines().enumerate() {
        if line.trim().is_empty() { continue; }
        let case: Value = serde_json::from_str(line).expect("fixture JSON must parse");
        let case_id = case["case_id"].as_str().expect("case_id");
        assert!(seen.insert(case_id.to_owned()), "duplicate case_id at line {}", line_no + 1);
        let expected = case["expected_valid"].as_bool().expect("expected_valid");
        let manifest_value = &case["manifest"];
        let reference = reference_valid(manifest_value);
        let production = serde_json::from_value::<PortStudyManifest>(manifest_value.clone())
            .map(|m| m.validate().is_ok())
            .unwrap_or(false);
        assert_eq!(reference, expected, "reference oracle disagreed for {case_id}");
        assert_eq!(production, expected, "production validator disagreed for {case_id}");
        count += 1;
    }
    assert_eq!(count, 34, "frozen corpus size changed without manifest update");
}

#[test]
fn canonical_output_is_invariant_to_alternative_order() {
    let case: Value = serde_json::from_str(CORPUS.lines().find(|line| {
        line.contains("valid_permuted_collection_order")
    }).expect("permutation fixture")).unwrap();
    let manifest: PortStudyManifest = serde_json::from_value(case["manifest"].clone()).unwrap();
    let mut reversed = manifest.clone();
    reversed.alternatives.reverse();
    for alt in &mut reversed.alternatives {
        alt.evidence.reverse();
        alt.design_parameters.reverse();
        alt.required_models.reverse();
        alt.assumptions.reverse();
    }
    assert_eq!(manifest.canonical_json().unwrap(), reversed.canonical_json().unwrap());
}
