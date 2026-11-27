use serde_json::Value;
use std::{collections::{BTreeMap, BTreeSet}, env, fs, process};

fn fail(message: impl AsRef<str>) -> ! {
    panic!("INVALID: {}", message.as_ref());
}

fn read_json(path: &str) -> Value {
    let bytes = fs::read(path).unwrap_or_else(|e| fail(format!("cannot read {path}: {e}")));
    serde_json::from_slice(&bytes).unwrap_or_else(|e| fail(format!("invalid JSON in {path}: {e}")))
}

fn string<'a>(v: &'a Value, path: &str) -> &'a str {
    v.as_str().unwrap_or_else(|| fail(format!("{path} must be a string")))
}

fn array<'a>(v: &'a Value, path: &str) -> &'a [Value] {
    v.as_array().map(Vec::as_slice).unwrap_or_else(|| fail(format!("{path} must be an array")))
}

fn required<'a>(v: &'a Value, key: &str, path: &str) -> &'a Value {
    v.get(key).unwrap_or_else(|| fail(format!("{path}.{key} is required")))
}

fn validate(feature_collection: &Value, evidence_package: &Value) -> Value {
    if string(required(feature_collection, "type", "featureCollection"), "featureCollection.type") != "FeatureCollection" {
        fail("root type must be FeatureCollection");
    }
    let conforms = array(required(feature_collection, "conformsTo", "featureCollection"), "featureCollection.conformsTo");
    if !conforms.iter().any(|x| x.as_str() == Some("http://www.opengis.net/spec/json-fg-1/1.0/conf/core")) {
        fail("JSON-FG core conformance declaration is missing");
    }

    let records = array(required(evidence_package, "records", "evidencePackage"), "evidencePackage.records");
    let mut record_map = BTreeMap::<String, &Value>::new();
    for (i, record) in records.iter().enumerate() {
        let path = format!("records[{i}]");
        let id = string(required(record, "recordId", &path), &format!("{path}.recordId")).to_owned();
        if record_map.insert(id.clone(), record).is_some() {
            fail(format!("duplicate evidence record ID: {id}"));
        }
        let rights = required(record, "rights", &path);
        for capability in ["recordVisibility", "metadataVisibility", "contentRetrieval", "publicDisclosure", "redistribution", "commercialUse", "modelTraining"] {
            let state = string(required(rights, capability, &format!("{path}.rights")), &format!("{path}.rights.{capability}"));
            if !["allowed", "prohibited", "pending", "unknown"].contains(&state) {
                fail(format!("invalid rights state at {path}.rights.{capability}"));
            }
        }
        let content_retrieval = string(required(rights, "contentRetrieval", &format!("{path}.rights")), &format!("{path}.rights.contentRetrieval"));
        let public_disclosure = string(required(rights, "publicDisclosure", &format!("{path}.rights")), &format!("{path}.rights.publicDisclosure"));
        if (content_retrieval != "allowed" || public_disclosure != "allowed") && record.get("syntheticPayload").is_some() {
            fail(format!("record {id} embeds content without both retrieval and disclosure permission"));
        }
    }

    let features = array(required(feature_collection, "features", "featureCollection"), "featureCollection.features");
    let mut semantic_features = Vec::new();
    let mut total_associations = 0usize;
    for (i, feature) in features.iter().enumerate() {
        let path = format!("features[{i}]");
        let properties = required(feature, "properties", &path);
        let profile = required(properties, "solAtlas", &format!("{path}.properties"));
        if string(required(profile, "profileVersion", &format!("{path}.solAtlas")), &format!("{path}.solAtlas.profileVersion")) != "0.1" {
            fail(format!("unsupported profile version at {path}"));
        }
        let claim = required(profile, "claim", &format!("{path}.solAtlas"));
        let temporal = required(profile, "temporal", &format!("{path}.solAtlas"));
        for role in ["validTime", "observationTime", "recordedTime", "decisionTime"] {
            required(temporal, role, &format!("{path}.temporal"));
        }
        let uncertainty = required(profile, "uncertainty", &format!("{path}.solAtlas"));
        let horizontal = required(uncertainty, "horizontalAccuracy", &format!("{path}.uncertainty"));
        let accuracy = required(horizontal, "value", &format!("{path}.uncertainty.horizontalAccuracy"))
            .as_f64().unwrap_or_else(|| fail("horizontal accuracy value must be numeric"));
        if accuracy < 0.0 { fail("horizontal accuracy cannot be negative"); }
        string(required(horizontal, "unit", &format!("{path}.uncertainty.horizontalAccuracy")), "horizontalAccuracy.unit");
        string(required(horizontal, "method", &format!("{path}.uncertainty.horizontalAccuracy")), "horizontalAccuracy.method");

        let associations = required(profile, "evidenceAssociations", &format!("{path}.solAtlas"));
        let buckets = [("supports", "supports"), ("contradicts", "contradicts"), ("contextOnly", "context_only")];
        let mut ids = BTreeSet::new();
        let mut semantic_associations = Vec::new();
        for (bucket, relation) in buckets {
            for (j, association) in array(required(associations, bucket, &format!("{path}.evidenceAssociations")), bucket).iter().enumerate() {
                let ap = format!("{path}.evidenceAssociations.{bucket}[{j}]");
                let id = string(required(association, "recordId", &ap), &format!("{ap}.recordId"));
                if !ids.insert(id.to_owned()) { fail(format!("evidence association {id} appears in multiple buckets")); }
                let declared_relation = string(required(association, "relation", &ap), &format!("{ap}.relation"));
                if declared_relation != relation { fail(format!("association {id} relation does not match bucket {bucket}")); }
                let record = record_map.get(id).unwrap_or_else(|| fail(format!("association {id} has no evidence record dependency")));
                if string(required(record, "relation", &format!("record {id}")), &format!("record {id}.relation")) != relation {
                    fail(format!("association {id} relation differs from its evidence record"));
                }
                let digest = string(required(association, "recordDigest", &ap), &format!("{ap}.recordDigest"));
                let record_digest = string(required(record, "payloadDigest", &format!("record {id}")), &format!("record {id}.payloadDigest"));
                if digest != record_digest { fail(format!("association {id} digest differs from evidence record")); }
                semantic_associations.push(serde_json::json!({"bucket":bucket,"recordId":id,"relation":relation,"recordDigest":digest}));
                total_associations += 1;
            }
        }
        let provenance = required(profile, "provenance", &format!("{path}.solAtlas"));
        let input_digests = array(required(provenance, "inputDigests", &format!("{path}.provenance")), &format!("{path}.provenance.inputDigests"));
        let mut declared_inputs: Vec<String> = input_digests.iter().map(|x| string(x, "inputDigests[]").to_owned()).collect();
        let mut expected_inputs: Vec<String> = ids.iter().map(|id| {
            string(required(record_map[id], "payloadDigest", "evidenceRecord"), "payloadDigest").to_owned()
        }).collect();
        declared_inputs.sort();
        expected_inputs.sort();
        if declared_inputs != expected_inputs { fail(format!("{path} provenance inputs do not match associated evidence")); }
        let hypotheses = required(profile, "hypothesisSet", &format!("{path}.solAtlas"));
        let alternatives = array(required(hypotheses, "alternatives", &format!("{path}.hypothesisSet")), "hypothesisSet.alternatives");
        if alternatives.is_empty() { fail(format!("{path} must retain at least one hypothesis")); }

        semantic_associations.sort_by(|a,b| a.to_string().cmp(&b.to_string()));
        semantic_features.push(serde_json::json!({
            "featureId": feature.get("id"),
            "geometry": feature.get("geometry"),
            "claim": claim,
            "temporal": temporal,
            "uncertainty": uncertainty,
            "geometryOrigin": profile.get("geometryOrigin"),
            "evidenceAssociations": semantic_associations,
            "provenance": provenance,
            "hypothesisSet": hypotheses
        }));
    }
    semantic_features.sort_by(|a,b| a.get("featureId").map(Value::to_string).cmp(&b.get("featureId").map(Value::to_string)));
    let _ = (total_associations, records.len());
    serde_json::json!({"profileVersion":"0.1","features":semantic_features})
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: sol-atlas-evidence-validator <feature-collection.json> <evidence-records.json>");
        process::exit(64);
    }
    let feature_collection = read_json(&args[1]);
    let evidence_package = read_json(&args[2]);
    let projection = validate(&feature_collection, &evidence_package);
    println!("VALID: semantic projection emitted");
    println!("{}", serde_json::to_string(&projection).expect("semantic projection serializes"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> (Value, Value) {
        let feature = include_str!("../../fixtures/feature-collection.json");
        let evidence = include_str!("../../fixtures/evidence-records.json");
        (serde_json::from_str(feature).unwrap(), serde_json::from_str(evidence).unwrap())
    }

    #[test]
    fn accepts_registered_synthetic_fixture() {
        let (feature, evidence) = fixtures();
        let projection = validate(&feature, &evidence);
        assert_eq!(projection["features"].as_array().unwrap().len(), 1);
    }

 
    #[test]
    fn restricted_unreferenced_input_does_not_change_public_semantic_projection() {
        let (feature, evidence) = fixtures();
        let baseline = validate(&feature, &evidence);
        let mut changed = evidence.clone();
        changed["records"].as_array_mut().unwrap().push(serde_json::json!({
            "recordId":"private-added-after-baseline",
            "recordType":"other",
            "relation":"context_only",
            "summary":"A secret record with no public metadata permission.",
            "payloadDigest":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "rights":{
                "decisionStatus":"pending",
                "recordVisibility":"prohibited",
                "metadataVisibility":"prohibited",
                "contentRetrieval":"prohibited",
                "publicDisclosure":"prohibited",
                "redistribution":"prohibited",
                "commercialUse":"unknown",
                "modelTraining":"prohibited"
            }
        }));
        let after = validate(&feature, &changed);
        assert_eq!(baseline, after, "restricted-only package changes must not alter the public semantic projection");
    }

    #[test]
    #[should_panic(expected = "appears in multiple buckets")]
    fn rejects_support_counter_overlap() {
        let (mut feature, evidence) = fixtures();
        let duplicate = feature["features"][0]["properties"]["solAtlas"]["evidenceAssociations"]["supports"][0].clone();
        feature["features"][0]["properties"]["solAtlas"]["evidenceAssociations"]["contradicts"].as_array_mut().unwrap().push(duplicate);
        let _ = validate(&feature, &evidence);
    }

    #[test]
    #[should_panic(expected = "has no evidence record dependency")]
    fn rejects_missing_record_dependency() {
        let (mut feature, evidence) = fixtures();
        feature["features"][0]["properties"]["solAtlas"]["evidenceAssociations"]["contextOnly"][0]["recordId"] = Value::String("missing".into());
        let _ = validate(&feature, &evidence);
    }

    #[test]
    #[should_panic(expected = "relation does not match bucket")]
    fn rejects_counter_evidence_promoted_to_support() {
        let (mut feature, evidence) = fixtures();
        feature["features"][0]["properties"]["solAtlas"]["evidenceAssociations"]["supports"][0]["relation"] = Value::String("contradicts".into());
        let _ = validate(&feature, &evidence);
    }

    #[test]
    #[should_panic(expected = "embeds content without both retrieval and disclosure permission")]
    fn rejects_content_embedded_without_rights() {
        let (feature, mut evidence) = fixtures();
        evidence["records"][1]["syntheticPayload"] = Value::String("restricted body".into());
        let _ = validate(&feature, &evidence);
    }
}
