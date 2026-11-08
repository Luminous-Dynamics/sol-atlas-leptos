//! Bounded synthetic reference oracle for construction material delivery feasibility.
//! Checks supplied evidence and constraints. It does not perform routing,
//! inventory allocation, procurement, dispatch, certification, lift planning,
//! custody approval or installation.

use std::collections::{BTreeMap, BTreeSet};
use serde_json::{Value, json};

fn array<'a>(v: &'a Value, k: &str) -> Option<&'a [Value]> {
    v.get(k).and_then(Value::as_array).map(Vec::as_slice)
}
fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}
fn number(v: &Value, k: &str) -> Option<f64> {
    v.get(k).and_then(Value::as_f64).filter(|x| x.is_finite())
}
fn integer(v: &Value, k: &str) -> Option<i64> {
    v.get(k).and_then(Value::as_i64)
}
fn nested<'a>(v: &'a Value, a: &str, b: &str) -> &'a Value {
    v.get(a).and_then(|x| x.get(b)).unwrap_or(&Value::Null)
}
fn empty(case_id: &str, disposition: &str, reasons: BTreeSet<String>) -> Value {
    json!({
        "case_id": case_id,
        "disposition": disposition,
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "deliveries": []
    })
}

#[derive(Default)]
struct Stages {
    values: serde_json::Map<String, Value>,
    reasons: BTreeSet<String>,
}
impl Stages {
    fn set(&mut self, stage: &str, value: &str) {
        self.values.insert(stage.to_owned(), Value::String(value.to_owned()));
    }
    fn fail(&mut self, stage: &str, value: &str, reason: &str) {
        self.set(stage, value);
        self.reasons.insert(reason.to_owned());
    }
    fn disposition(&self, custody: &str, inspection: &str, received: f64, required: f64) -> &'static str {
        if self.reasons.contains("CUSTODY_DISPUTED") {
            "Disputed"
        } else if self.reasons.contains("LOT_NOT_QUALIFIED_FOR_DECLARED_USE")
            || self.reasons.contains("RECEIVING_INSPECTION_FAILED")
            || self.reasons.contains("CUSTODY_OR_BATCH_REVOKED") {
            "Rejected"
        } else if self.reasons.iter().any(|r|
            r.contains("OVERBOOKED") || r.contains("SHORTFALL") || r.contains("OVERALLOCATION")
            || r.contains("INFEASIBLE") || r.contains("CLOSED") || r.contains("UNAVAILABLE")
            || r.contains("REVISION_MISMATCH")) {
            "Blocked"
        } else if self.reasons.contains("DELIVERY_WINDOW_OR_DEADLINE_MISSED") {
            "Late"
        } else if !self.reasons.is_empty() {
            "Unknown"
        } else if custody == "partial" || (custody == "accepted" && received < required) {
            "Partial"
        } else if custody == "accepted" && inspection == "passed" && received >= required {
            "AcceptedDelivery"
        } else {
            "DeliveryFeasible"
        }
    }
}

fn validate<'a>(case: &Value, input: &'a Value) -> Result<Vec<&'a Value>, (String, BTreeSet<String>)> {
    let case_id = text(case, "case_id");
    let mut errors = BTreeSet::new();
    if text(input, "scope_digest").is_empty() {
        errors.insert("EVALUATION_SCOPE_DIGEST_MISSING".to_owned());
    }
    let Some(deliveries) = array(input, "deliveries") else {
        errors.insert("DELIVERIES_MISSING".to_owned());
        return Err((case_id.to_owned(), errors));
    };
    let Some(resources) = array(input, "resources") else {
        errors.insert("SHARED_RESOURCES_MISSING".to_owned());
        return Err((case_id.to_owned(), errors));
    };
    if deliveries.is_empty() {
        errors.insert("DELIVERIES_EMPTY".to_owned());
    }

    let mut ids = BTreeSet::new();
    let mut receipts = BTreeSet::new();
    for d in deliveries {
        let id = text(d, "delivery_id");
        if id.is_empty() || !ids.insert(id.to_owned()) {
            errors.insert("DUPLICATE_OR_MISSING_DELIVERY_ID".to_owned());
        }
        if text(d, "scope_digest") != text(input, "scope_digest") {
            return Err((case_id.to_owned(), BTreeSet::from(["DELIVERY_SCOPE_MISMATCH".to_owned()])));
        }
        let demand = d.get("demand").unwrap_or(&Value::Null);
        let source = d.get("source").unwrap_or(&Value::Null);
        let transport = d.get("transport").unwrap_or(&Value::Null);
        let site = d.get("site").unwrap_or(&Value::Null);
        if text(demand, "unit").is_empty() || text(source, "quantity_unit").is_empty()
            || text(demand, "unit") != text(source, "quantity_unit") {
            errors.insert("QUANTITY_UNIT_MISMATCH".to_owned());
        }
        for (v, k) in [
            (demand, "quantity"), (demand, "mass_kg"), (demand, "volume_m3"),
            (source, "available_quantity"), (transport, "payload_capacity_kg"),
            (transport, "volume_capacity_m3"), (nested(site, "unload", "capacity_kg"), "value"),
            (nested(site, "receiving_bay", "unused"), "unused"),
        ] {
            if k == "value" {
                if v.as_f64().is_none_or(|x| !x.is_finite() || x < 0.0) {
                    errors.insert("QUANTITY_OR_CAPACITY_INVALID".to_owned());
                }
            } else if v.get(k).is_some() && number(v, k).is_none_or(|x| x < 0.0) {
                errors.insert("QUANTITY_OR_CAPACITY_INVALID".to_owned());
            }
        }
        for (v, k) in [
            (demand, "required_by_tick"), (demand, "window_start_tick"),
            (demand, "window_end_tick"), (source, "ready_tick"),
            (transport, "route_duration_ticks"), (nested(site, "unload", "unused"), "unused"),
        ] {
            if v.get(k).is_some() && integer(v, k).is_none_or(|x| x < 0) {
                errors.insert("TIME_VALUE_INVALID".to_owned());
            }
        }
        if let (Some(start), Some(end)) =
            (integer(demand, "window_start_tick"), integer(demand, "window_end_tick")) {
            if end <= start {
                errors.insert("DELIVERY_WINDOW_INVALID".to_owned());
            }
        }
        for receipt in [
            text(source, "inventory_receipt_id"),
            text(transport, "route_receipt_id"),
            text(nested(site, "receiving", "custody_receipt_id"), "unused"),
        ] {
            if !receipt.is_empty() && !receipts.insert(receipt.to_owned()) {
                errors.insert("DUPLICATE_SOURCE_RECEIPT_ID".to_owned());
            }
        }
        let custody_receipt = text(nested(site, "receiving", "custody_receipt_id"), "value");
        if !custody_receipt.is_empty() && !receipts.insert(custody_receipt.to_owned()) {
            errors.insert("DUPLICATE_SOURCE_RECEIPT_ID".to_owned());
        }
    }

    let mut resource_keys = BTreeSet::new();
    for r in resources {
        let key = (text(r, "resource_id").to_owned(), text(r, "slot_id").to_owned());
        if key.0.is_empty() || key.1.is_empty() || !resource_keys.insert(key)
            || number(r, "capacity").is_none_or(|x| x < 0.0)
            || text(r, "unit").is_empty()
            || !matches!(text(r, "evidence_state"), "current" | "stale" | "unknown")
            || !matches!(text(r, "kind"), "receiving_bay" | "laydown") {
            errors.insert("RESOURCE_DEFINITION_INVALID".to_owned());
        }
    }
    if errors.is_empty() {
        Ok(deliveries.iter().collect())
    } else {
        Err((case_id.to_owned(), errors))
    }
}

fn resource_claims(input: &Value, deliveries: &[&Value])
    -> (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>) {
    let mut blocked = BTreeSet::new();
    let mut unknown = BTreeSet::new();
    let mut missing = BTreeSet::new();
    let resources = array(input, "resources").unwrap_or_default();

    for d in deliveries {
        let id = text(d, "delivery_id");
        for (field, kind) in [("receiving_bay", "receiving_bay"), ("laydown", "laydown")] {
            let claim = nested(d, "site", field);
            let rid = text(claim, "resource_id");
            let slot = text(claim, "slot_id");
            if rid.is_empty() || slot.is_empty() || number(claim, "use").is_none_or(|x| x <= 0.0) {
                missing.insert(id.to_owned());
                continue;
            }
            let Some(r) = resources.iter().find(|r|
                text(r, "resource_id") == rid && text(r, "slot_id") == slot && text(r, "kind") == kind) else {
                missing.insert(id.to_owned());
                continue;
            };
            if text(r, "evidence_state") != "current" {
                unknown.insert(id.to_owned());
            }
        }
    }

    for r in resources {
        let rid = text(r, "resource_id");
        let slot = text(r, "slot_id");
        let kind = text(r, "kind");
        let field = if kind == "receiving_bay" { "receiving_bay" } else { "laydown" };
        let mut total = 0.0;
        let mut contenders = BTreeSet::new();
        for d in deliveries {
            let claim = nested(d, "site", field);
            if text(claim, "resource_id") == rid && text(claim, "slot_id") == slot {
                total += number(claim, "use").unwrap_or(0.0);
                contenders.insert(text(d, "delivery_id").to_owned());
            }
        }
        if text(r, "evidence_state") == "current" && total > number(r, "capacity").unwrap_or(0.0) {
            blocked.extend(contenders);
        }
    }
    (blocked, unknown, missing)
}

fn overallocated_lots(deliveries: &[&Value]) -> BTreeSet<String> {
    let mut groups = BTreeMap::<(String, String), (f64, f64, BTreeSet<String>)>::new();
    for d in deliveries {
        let source = d.get("source").unwrap_or(&Value::Null);
        let demand = d.get("demand").unwrap_or(&Value::Null);
        let key = (text(source, "lot_id").to_owned(), text(source, "quantity_unit").to_owned());
        let entry = groups.entry(key).or_insert((0.0, number(source, "available_quantity").unwrap_or(0.0), BTreeSet::new()));
        entry.0 += number(demand, "quantity").unwrap_or(0.0);
        entry.1 = entry.1.min(number(source, "available_quantity").unwrap_or(0.0));
        entry.2.insert(text(d, "delivery_id").to_owned());
    }
    let mut blocked = BTreeSet::new();
    for (total, available, ids) in groups.into_values() {
        if total > available {
            blocked.extend(ids);
        }
    }
    blocked
}

pub fn evaluate_case(case: &Value) -> Value {
    let case_id = text(case, "case_id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let deliveries = match validate(case, input) {
        Ok(x) => x,
        Err((_, reasons)) => {
            let disposition = if reasons.contains("DELIVERY_SCOPE_MISMATCH") { "NotComparable" } else { "Unsupported" };
            return empty(case_id, disposition, reasons);
        }
    };
    let (blocked_resources, unknown_resources, missing_resources) = resource_claims(input, &deliveries);
    let overallocated = overallocated_lots(&deliveries);
    let mut outputs = Vec::new();

    for d in deliveries {
        let id = text(d, "delivery_id");
        let demand = d.get("demand").unwrap_or(&Value::Null);
        let source = d.get("source").unwrap_or(&Value::Null);
        let transport = d.get("transport").unwrap_or(&Value::Null);
        let site = d.get("site").unwrap_or(&Value::Null);
        let unload = nested(site, "unload", "unused");
        let unload_obj = site.get("unload").unwrap_or(&Value::Null);
        let receiving = site.get("receiving").unwrap_or(&Value::Null);
        let quantity = number(demand, "quantity").unwrap_or(0.0);
        let available = number(source, "available_quantity").unwrap_or(0.0);
        let mut result = Stages::default();

        if text(source, "evidence_state") != "current" || text(source, "inventory_evidence_state") != "current"
            || text(source, "inventory_receipt_id").is_empty() {
            result.fail("inventory", "unknown", "SOURCE_OR_INVENTORY_EVIDENCE_UNKNOWN");
        } else if available < quantity {
            result.fail("inventory", "blocked", "SOURCE_STOCK_SHORTFALL");
        } else if overallocated.contains(id) {
            result.fail("inventory", "blocked", "SOURCE_STOCK_OVERALLOCATION");
        } else {
            result.set("inventory", "sufficient");
        }

        if text(source, "qualification_evidence_state") != "current" || text(source, "qualification_state") == "unknown" {
            result.fail("qualification", "unknown", "QUALIFICATION_EVIDENCE_UNKNOWN");
        } else if text(source, "qualification_state") == "revoked" || text(source, "qualification_state") == "not_qualified"
            || text(source, "product_id") != text(demand, "product_id") || text(source, "grade") != text(demand, "grade") {
            result.fail("qualification", "rejected", "LOT_NOT_QUALIFIED_FOR_DECLARED_USE");
        } else {
            result.set("qualification", "qualified");
        }

        if text(source, "design_revision") != text(d, "design_revision") || text(source, "bom_digest") != text(d, "bom_digest") {
            result.fail("revision", "blocked", "DESIGN_OR_BOM_REVISION_MISMATCH");
        } else {
            result.set("revision", "matched");
        }

        let arrival = (integer(source, "ready_tick").unwrap_or(0) + integer(transport, "route_duration_ticks").unwrap_or(0))
            .max(integer(demand, "window_start_tick").unwrap_or(0))
            .max(integer(site, "gate_open_start_tick").unwrap_or(0));
        let acceptance = arrival + integer(unload_obj, "duration_ticks").unwrap_or(0)
            + integer(receiving, "inspection_duration_ticks").unwrap_or(0);
        let deadline = integer(demand, "required_by_tick").unwrap_or(i64::MAX)
            .min(integer(demand, "window_end_tick").unwrap_or(i64::MAX))
            .min(integer(site, "gate_open_end_tick").unwrap_or(i64::MAX));
        if acceptance >= deadline {
            result.fail("availability", "late", "DELIVERY_WINDOW_OR_DEADLINE_MISSED");
        } else {
            result.set("availability", "timely");
        }

        if text(transport, "evidence_state") != "current" || text(transport, "route_evidence_state") != "current"
            || text(transport, "route_receipt_id").is_empty() || text(transport, "route_state") == "unknown" {
            result.fail("transport", "unknown", "TRANSPORT_OR_ROUTE_EVIDENCE_UNKNOWN");
        } else if text(transport, "route_state") != "open" || text(transport, "vehicle_access") == "blocked"
            || number(transport, "payload_capacity_kg").unwrap_or(0.0) < number(demand, "mass_kg").unwrap_or(0.0)
            || number(transport, "volume_capacity_m3").unwrap_or(0.0) < number(demand, "volume_m3").unwrap_or(0.0) {
            result.fail("transport", "blocked", "VEHICLE_ROUTE_OR_LOAD_ENVELOPE_INFEASIBLE");
        } else if text(transport, "vehicle_access") != "allowed" {
            result.fail("transport", "unknown", "VEHICLE_ROUTE_PERMISSION_UNKNOWN");
        } else {
            result.set("transport", "feasible");
        }

        if text(site, "evidence_state") != "current" || text(site, "gate_state") == "unknown" {
            result.fail("site_access", "unknown", "SITE_ACCESS_EVIDENCE_UNKNOWN");
        } else if text(site, "gate_state") != "open" {
            result.fail("site_access", "blocked", "SITE_GATE_CLOSED");
        } else {
            result.set("site_access", "feasible");
        }

        if text(unload_obj, "evidence_state") != "current" {
            result.fail("unloading", "unknown", "UNLOAD_EQUIPMENT_EVIDENCE_UNKNOWN");
        } else if text(unload_obj, "status") != "available"
            || number(unload_obj, "capacity_kg").unwrap_or(0.0) < number(demand, "mass_kg").unwrap_or(0.0) {
            result.fail("unloading", "blocked", "UNLOAD_EQUIPMENT_UNAVAILABLE_OR_INSUFFICIENT");
        } else {
            result.set("unloading", "feasible");
        }

        if text(receiving, "evidence_state") != "current" || text(receiving, "labour_state") == "unknown" {
            result.fail("receiving", "unknown", "RECEIVING_LABOUR_EVIDENCE_UNKNOWN");
        } else if text(receiving, "labour_state") != "available" {
            result.fail("receiving", "blocked", "RECEIVING_LABOUR_UNAVAILABLE");
        } else {
            result.set("receiving", "feasible");
        }

        match text(receiving, "inspection_state") {
            "failed" => result.fail("inspection", "rejected", "RECEIVING_INSPECTION_FAILED"),
            "unknown" => result.fail("inspection", "unknown", "RECEIVING_INSPECTION_UNKNOWN"),
            "passed" | "pending" => result.set("inspection", text(receiving, "inspection_state")),
            _ => result.fail("inspection", "unknown", "RECEIVING_INSPECTION_UNKNOWN"),
        }

        if text(receiving, "custody_evidence_state") != "current" {
            result.fail("custody", "unknown", "CUSTODY_EVIDENCE_UNKNOWN");
        } else {
            match text(receiving, "custody_state") {
                "disputed" => result.fail("custody", "disputed", "CUSTODY_DISPUTED"),
                "rejected" => result.fail("custody", "rejected", "CUSTODY_OR_BATCH_REVOKED"),
                "accepted" | "partial" | "not_received" => result.set("custody", text(receiving, "custody_state")),
                _ => result.fail("custody", "unknown", "CUSTODY_EVIDENCE_UNKNOWN"),
            }
        }

        if blocked_resources.contains(id) {
            result.fail("shared_resources", "blocked", "SHARED_RECEIVING_OR_LAYDOWN_RESOURCE_OVERBOOKED");
        } else if unknown_resources.contains(id) || missing_resources.contains(id) {
            result.fail("shared_resources", "unknown", "SHARED_RESOURCE_EVIDENCE_UNKNOWN");
        } else {
            result.set("shared_resources", "feasible");
        }

        let received = number(receiving, "received_quantity").unwrap_or(0.0);
        let state = result.disposition(text(receiving, "custody_state"), text(receiving, "inspection_state"), received, quantity);
        outputs.push(json!({
            "delivery_id": id,
            "state": state,
            "reason_codes": result.reasons.into_iter().collect::<Vec<_>>(),
            "predicted_arrival_tick": arrival,
            "predicted_acceptance_tick": acceptance,
            "source_class": text(source, "source_class"),
            "local_productive_capability_claim": false,
            "lot_id": text(source, "lot_id"),
            "quantity_requested": quantity,
            "quantity_declared_available": available,
            "stages": result.values
        }));
        let _ = (unload, bay, laydown);
    }
    outputs.sort_by(|a, b| text(a, "delivery_id").cmp(text(b, "delivery_id")));
    json!({"case_id":case_id,"disposition":"Evaluated","reason_codes":[],"deliveries":outputs})
}

fn equivalent(a: &Value, b: &Value) -> bool {
    match (a,b) {
        (Value::Number(x),Value::Number(y)) => x.as_f64().zip(y.as_f64()).is_some_and(|(x,y)|x==y),
        (Value::Array(x),Value::Array(y)) => x.len()==y.len() && x.iter().zip(y).all(|(x,y)|equivalent(x,y)),
        (Value::Object(x),Value::Object(y)) => x.len()==y.len() && y.iter().all(|(k,v)|x.get(k).is_some_and(|a|equivalent(a,v))),
        _ => a==b,
    }
}
pub fn matches_expected(actual:&Value, expected:&Value)->bool {
    expected.as_object().is_some_and(|fields|fields.iter().all(|(k,v)|actual.get(k).is_some_and(|a|equivalent(a,v))))
}

#[cfg(test)]
mod tests {
    use super::{evaluate_case,matches_expected};
    use serde_json::Value;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn corpus()->Value {
        let path=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/research/city-construction-delivery/CITY_CONSTRUCTION_DELIVERY_CORPUS_V0_1.json");
        let bytes=std::fs::read(path).expect("delivery corpus must exist");
        serde_json::from_slice(&bytes).expect("delivery corpus must parse")
    }
    fn case(id:&str)->Value {
        corpus()["cases"].as_array().unwrap().iter().find(|x|x["case_id"]==id).unwrap().clone()
    }
    #[test] fn corpus_has_33_unique_synthetic_cases() {
        let c=corpus();
        assert_eq!(c["classification"],"synthetic_fixture_only");
        assert_eq!(c["corpus_id"],"CITY-CONSTRUCTION-DELIVERY-CORPUS-V0.1");
        let xs=c["cases"].as_array().unwrap();assert_eq!(xs.len(),33);
        let ids=xs.iter().map(|x|x["case_id"].as_str().unwrap().to_owned()).collect::<BTreeSet<_>>();
        assert_eq!(ids.len(),xs.len());
    }
    #[test] fn independent_oracle_matches_every_known_answer() {
        for x in corpus()["cases"].as_array().unwrap() {
            let actual=evaluate_case(x);
            assert!(matches_expected(&actual,&x["expected"]),"{} mismatch\nactual: {}\nexpected: {}",x["case_id"],serde_json::to_string_pretty(&actual).unwrap(),serde_json::to_string_pretty(&x["expected"]).unwrap());
        }
    }
    #[test] fn shared_lot_overallocation_blocks_all_contenders() {
        for d in evaluate_case(&case("CD-005"))["deliveries"].as_array().unwrap() {assert_eq!(d["state"],"Blocked");}
    }
    #[test] fn shared_resources_block_all_contenders() {
        for id in ["CD-023","CD-033"] {assert!(evaluate_case(&case(id))["deliveries"].as_array().unwrap().iter().all(|d|d["state"]=="Blocked"));}
    }
    #[test] fn acceptance_at_half_open_window_endpoint_is_late() {
        let r=evaluate_case(&case("CD-027"));assert_eq!(r["deliveries"][0]["predicted_acceptance_tick"],79);assert_eq!(r["deliveries"][0]["state"],"Late");
    }
    #[test] fn pending_and_partial_receipts_are_not_full_acceptance() {
        assert_eq!(evaluate_case(&case("CD-019"))["deliveries"][0]["state"],"DeliveryFeasible");
        assert_eq!(evaluate_case(&case("CD-020"))["deliveries"][0]["state"],"Partial");
    }
    #[test] fn imported_delivery_never_claims_local_productive_capability() {
        let r=evaluate_case(&case("CD-025"));assert_eq!(r["deliveries"][0]["state"],"AcceptedDelivery");assert_eq!(r["deliveries"][0]["local_productive_capability_claim"],false);
    }
    #[test] fn bom_revision_mismatch_blocks_delivery() {
        assert_eq!(evaluate_case(&case("CD-026"))["deliveries"][0]["state"],"Blocked");
    }
    #[test] fn stale_or_revoked_evidence_cannot_be_promoted() {
        for id in ["CD-002","CD-010","CD-013","CD-031"] {assert_ne!(evaluate_case(&case(id))["deliveries"][0]["state"],"AcceptedDelivery");}
        for id in ["CD-022","CD-032"] {assert_eq!(evaluate_case(&case(id))["deliveries"][0]["state"],"Rejected");}
    }
    #[test] fn delivery_permutation_does_not_change_normalized_output() {
        let mut x=case("CD-023");let before=evaluate_case(&x);x["input"]["deliveries"].as_array_mut().unwrap().reverse();assert_eq!(before,evaluate_case(&x));
    }
}
