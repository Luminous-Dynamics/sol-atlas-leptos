//! Fail-closed train/validation/test split integrity for methane benchmark corpora.
//!
//! The caller supplies the exact corpus JSON bytes and a separately versioned split
//! manifest. This module binds the manifest to the raw corpus bytes, requires a
//! complete one-to-one assignment, and rejects source, cluster, or exact-case-artifact
//! leakage across partitions. It validates split integrity; it does not prove labels
//! are true or that the chosen split design measures every intended form of generalization.

use crate::sha256_hex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationSplit {
    Train,
    Validation,
    Test,
    BlindTest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SplitUniverseRecord {
    pub case_id: String,
    /// Dependence unit such as site × overpass or controlled-release campaign.
    pub cluster_id: String,
    /// Stable source identity; use an explicit per-case sentinel if attribution is unknown.
    pub source_group_id: String,
    /// Digest of the exact case-level input artifact, not a large export containing many cases.
    pub case_artifact_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SplitCorpus {
    pub schema_version: u32,
    pub records: Vec<SplitUniverseRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SplitAssignment {
    pub case_id: String,
    pub cluster_id: String,
    pub source_group_id: String,
    pub case_artifact_sha256: String,
    pub split: EvaluationSplit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SplitManifest {
    pub schema_version: u32,
    /// SHA-256 of the exact bytes of the corpus JSON file.
    pub corpus_sha256: String,
    /// Human-readable, predeclared grouping policy, not a post-hoc explanation.
    pub split_strategy: String,
    pub assignments: Vec<SplitAssignment>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SplitViolation {
    pub record_id: String,
    pub code: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct SplitValidationReport {
    pub computed_corpus_sha256: Option<String>,
    pub corpus_records: usize,
    pub assignments: usize,
    pub valid: bool,
    pub violations: Vec<SplitViolation>,
}

impl SplitValidationReport {
    pub fn is_valid(&self) -> bool {
        self.valid && self.violations.is_empty()
    }

    fn push(&mut self, record_id: &str, code: &'static str) {
        self.violations.push(SplitViolation {
            record_id: record_id.to_owned(),
            code,
        });
    }
}

pub fn validate_split_manifest(
    corpus_bytes: &[u8],
    manifest: &SplitManifest,
) -> SplitValidationReport {
    let mut report = SplitValidationReport::default();
    let corpus_digest = sha256_hex(corpus_bytes);
    report.computed_corpus_sha256 = Some(corpus_digest.clone());

    let corpus: SplitCorpus = match serde_json::from_slice(corpus_bytes) {
        Ok(corpus) => corpus,
        Err(_) => {
            report.push("__corpus__", "invalid_corpus_json");
            report.violations.sort_by(|a, b| (&a.record_id, a.code).cmp(&(&b.record_id, b.code)));
            return report;
        }
    };
    report.corpus_records = corpus.records.len();
    report.assignments = manifest.assignments.len();

    if corpus.schema_version != 1 {
        report.push("__corpus__", "unsupported_corpus_schema_version");
    }
    if manifest.schema_version != 1 {
        report.push("__manifest__", "unsupported_split_manifest_schema_version");
    }
    if manifest.split_strategy.trim().is_empty() {
        report.push("__manifest__", "empty_split_strategy");
    }
    if !is_canonical_sha256(&manifest.corpus_sha256) {
        report.push("__manifest__", "invalid_corpus_sha256");
    } else if manifest.corpus_sha256 != corpus_digest {
        report.push("__manifest__", "corpus_digest_mismatch");
    }
    if corpus.records.is_empty() {
        report.push("__corpus__", "empty_split_corpus");
    }

    let mut expected = BTreeMap::new();
    for record in &corpus.records {
        if record.case_id.trim().is_empty() {
            report.push("__corpus__", "empty_case_id");
        }
        if record.cluster_id.trim().is_empty() {
            report.push(&record.case_id, "empty_cluster_id");
        }
        if record.source_group_id.trim().is_empty() {
            report.push(&record.case_id, "empty_source_group_id");
        }
        if !is_canonical_sha256(&record.case_artifact_sha256) {
            report.push(&record.case_id, "invalid_case_artifact_sha256");
        }
        if expected.insert(record.case_id.as_str(), record).is_some() {
            report.push(&record.case_id, "duplicate_corpus_case_id");
        }
    }

    let mut seen_assignments = BTreeSet::new();
    let mut cluster_split: BTreeMap<&str, EvaluationSplit> = BTreeMap::new();
    let mut source_split: BTreeMap<&str, EvaluationSplit> = BTreeMap::new();
    let mut artifact_split: BTreeMap<&str, EvaluationSplit> = BTreeMap::new();
    let mut populated_splits = BTreeSet::new();

    for assignment in &manifest.assignments {
        if assignment.case_id.trim().is_empty() {
            report.push("__manifest__", "empty_assignment_case_id");
        }
        if !seen_assignments.insert(assignment.case_id.as_str()) {
            report.push(&assignment.case_id, "duplicate_case_assignment");
        }
        populated_splits.insert(assignment.split);

        let Some(record) = expected.get(assignment.case_id.as_str()) else {
            report.push(&assignment.case_id, "assignment_for_unknown_case");
            continue;
        };

        if record.cluster_id != assignment.cluster_id {
            report.push(&assignment.case_id, "assignment_cluster_mismatch");
        }
        if record.source_group_id != assignment.source_group_id {
            report.push(&assignment.case_id, "assignment_source_group_mismatch");
        }
        if record.case_artifact_sha256 != assignment.case_artifact_sha256 {
            report.push(&assignment.case_id, "assignment_artifact_digest_mismatch");
        }

        register_split_owner(
            &mut report,
            &mut cluster_split,
            &assignment.cluster_id,
            assignment.split,
            &assignment.case_id,
            "cluster_crosses_splits",
        );
        register_split_owner(
            &mut report,
            &mut source_split,
            &assignment.source_group_id,
            assignment.split,
            &assignment.case_id,
            "source_group_crosses_splits",
        );
        register_split_owner(
            &mut report,
            &mut artifact_split,
            &assignment.case_artifact_sha256,
            assignment.split,
            &assignment.case_id,
            "case_artifact_crosses_splits",
        );
    }

    for case_id in expected.keys() {
        if !seen_assignments.contains(case_id) {
            report.push(case_id, "missing_case_assignment");
        }
    }

    for split in [
        EvaluationSplit::Train,
        EvaluationSplit::Validation,
        EvaluationSplit::Test,
    ] {
        if !populated_splits.contains(&split) {
            report.push("__manifest__", missing_split_code(split));
        }
    }

    report.violations.sort_by(|a, b| (&a.record_id, a.code).cmp(&(&b.record_id, b.code)));
    report.violations.dedup();
    report.valid = report.violations.is_empty();
    report
}

fn register_split_owner<'a>(
    report: &mut SplitValidationReport,
    owners: &mut BTreeMap<&'a str, EvaluationSplit>,
    identity: &'a str,
    split: EvaluationSplit,
    record_id: &str,
    violation_code: &'static str,
) {
    if let Some(previous_split) = owners.insert(identity, split) {
        if previous_split != split {
            report.push(record_id, violation_code);
        }
    }
}

fn missing_split_code(split: EvaluationSplit) -> &'static str {
    match split {
        EvaluationSplit::Train => "missing_train_split",
        EvaluationSplit::Validation => "missing_validation_split",
        EvaluationSplit::Test => "missing_test_split",
        EvaluationSplit::BlindTest => "missing_blind_test_split",
    }
}

fn is_canonical_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        byte.to_string().repeat(64)
    }

    fn corpus() -> SplitCorpus {
        SplitCorpus {
            schema_version: 1,
            records: vec![
                SplitUniverseRecord {
                    case_id: "train-a".into(),
                    cluster_id: "site-a".into(),
                    source_group_id: "source-a".into(),
                    case_artifact_sha256: digest('a'),
                },
                SplitUniverseRecord {
                    case_id: "train-b".into(),
                    cluster_id: "site-a".into(),
                    source_group_id: "source-b".into(),
                    case_artifact_sha256: digest('b'),
                },
                SplitUniverseRecord {
                    case_id: "validation-a".into(),
                    cluster_id: "site-b".into(),
                    source_group_id: "source-c".into(),
                    case_artifact_sha256: digest('c'),
                },
                SplitUniverseRecord {
                    case_id: "test-a".into(),
                    cluster_id: "site-c".into(),
                    source_group_id: "source-d".into(),
                    case_artifact_sha256: digest('d'),
                },
                SplitUniverseRecord {
                    case_id: "blind-a".into(),
                    cluster_id: "site-d".into(),
                    source_group_id: "source-e".into(),
                    case_artifact_sha256: digest('e'),
                },
            ],
        }
    }

    fn manifest_for(corpus_bytes: &[u8]) -> SplitManifest {
        SplitManifest {
            schema_version: 1,
            corpus_sha256: sha256_hex(corpus_bytes),
            split_strategy: "group by site and source before model selection".into(),
            assignments: vec![
                assignment("train-a", "site-a", "source-a", 'a', EvaluationSplit::Train),
                assignment("train-b", "site-a", "source-b", 'b', EvaluationSplit::Train),
                assignment(
                    "validation-a",
                    "site-b",
                    "source-c",
                    'c',
                    EvaluationSplit::Validation,
                ),
                assignment("test-a", "site-c", "source-d", 'd', EvaluationSplit::Test),
                assignment("blind-a", "site-d", "source-e", 'e', EvaluationSplit::BlindTest),
            ],
        }
    }

    fn assignment(
        case_id: &str,
        cluster_id: &str,
        source_group_id: &str,
        artifact: char,
        split: EvaluationSplit,
    ) -> SplitAssignment {
        SplitAssignment {
            case_id: case_id.into(),
            cluster_id: cluster_id.into(),
            source_group_id: source_group_id.into(),
            case_artifact_sha256: digest(artifact),
            split,
        }
    }

    fn corpus_bytes() -> Vec<u8> {
        serde_json::to_vec(&corpus()).unwrap()
    }

    #[test]
    fn accepts_exact_complete_partition_with_separated_groups() {
        let bytes = corpus_bytes();
        let manifest = manifest_for(&bytes);
        let report = validate_split_manifest(&bytes, &manifest);
        assert!(report.is_valid(), "{report:?}");
        assert_eq!(report.corpus_records, 5);
        assert_eq!(report.assignments, 5);
        assert_eq!(report.computed_corpus_sha256.as_deref(), Some(manifest.corpus_sha256.as_str()));
    }

    #[test]
    fn rejects_modified_corpus_bytes_after_manifest_was_pinned() {
        let original = corpus_bytes();
        let manifest = manifest_for(&original);
        let modified = [original.as_slice(), b" "].concat();
        let report = validate_split_manifest(&modified, &manifest);
        assert!(!report.is_valid());
        assert!(report.violations.iter().any(|v| v.code == "corpus_digest_mismatch"));
    }

    #[test]
    fn rejects_cluster_leakage_across_train_and_test() {
        let bytes = corpus_bytes();
        let mut manifest = manifest_for(&bytes);
        manifest.assignments
            .iter_mut()
            .find(|a| a.case_id == "test-a")
            .unwrap()
            .cluster_id = "site-a".into();
        // The assignment must also match the corpus identity, so both violations are expected.
        let report = validate_split_manifest(&bytes, &manifest);
        assert!(report.violations.iter().any(|v| v.code == "assignment_cluster_mismatch"));
        assert!(report.violations.iter().any(|v| v.code == "cluster_crosses_splits"));
    }

    #[test]
    fn rejects_source_group_leakage_even_when_clusters_differ() {
        let bytes = corpus_bytes();
        let mut manifest = manifest_for(&bytes);
        manifest.assignments
            .iter_mut()
            .find(|a| a.case_id == "test-a")
            .unwrap()
            .source_group_id = "source-a".into();
        let report = validate_split_manifest(&bytes, &manifest);
        assert!(report.violations.iter().any(|v| v.code == "source_group_crosses_splits"));
    }

    #[test]
    fn rejects_identical_case_artifact_in_different_splits() {
        let bytes = corpus_bytes();
        let mut corpus_value = corpus();
        corpus_value.records[3].case_artifact_sha256 = digest('a');
        let changed_bytes = serde_json::to_vec(&corpus_value).unwrap();
        let mut manifest = manifest_for(&changed_bytes);
        let test_assignment = manifest.assignments.iter_mut().find(|a| a.case_id == "test-a").unwrap();
        test_assignment.case_artifact_sha256 = digest('a');
        let report = validate_split_manifest(&changed_bytes, &manifest);
        assert!(report.violations.iter().any(|v| v.code == "case_artifact_crosses_splits"));
    }

    #[test]
    fn rejects_missing_duplicate_and_extra_assignments() {
        let bytes = corpus_bytes();
        let mut manifest = manifest_for(&bytes);
        manifest.assignments.pop();
        manifest.assignments.push(manifest.assignments[0].clone());
        manifest.assignments.push(assignment(
            "unknown-case", "site-z", "source-z", 'f', EvaluationSplit::Test
        ));
        let report = validate_split_manifest(&bytes, &manifest);
        assert!(report.violations.iter().any(|v| v.code == "missing_case_assignment"));
        assert!(report.violations.iter().any(|v| v.code == "duplicate_case_assignment"));
        assert!(report.violations.iter().any(|v| v.code == "assignment_for_unknown_case"));
    }

    #[test]
    fn rejects_identity_mismatch_between_corpus_and_manifest() {
        let bytes = corpus_bytes();
        let mut manifest = manifest_for(&bytes);
        manifest.assignments[0].case_artifact_sha256 = digest('f');
        let report = validate_split_manifest(&bytes, &manifest);
        assert!(report.violations.iter().any(|v| v.code == "assignment_artifact_digest_mismatch"));
    }

    #[test]
    fn rejects_absent_validation_or_train_partitions() {
        let bytes = corpus_bytes();
        let mut manifest = manifest_for(&bytes);
        manifest.assignments.iter_mut().for_each(|assignment| {
            if assignment.split == EvaluationSplit::Validation {
                assignment.split = EvaluationSplit::Train;
            }
        });
        let report = validate_split_manifest(&bytes, &manifest);
        assert!(report.violations.iter().any(|v| v.code == "missing_validation_split"));
    }

    #[test]
    fn rejects_malformed_corpus_json_without_panicking() {
        let manifest = SplitManifest {
            schema_version: 1,
            corpus_sha256: digest('a'),
            split_strategy: "predeclared".into(),
            assignments: Vec::new(),
        };
        let report = validate_split_manifest(b"{not json", &manifest);
        assert!(!report.is_valid());
        assert!(report.violations.iter().any(|v| v.code == "invalid_corpus_json"));
    }
}
