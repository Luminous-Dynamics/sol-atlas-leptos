// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Plural archaeological annotations, explicit abstention and panel agreement.
//!
//! Independent annotations are preserved as observations. Plurality and
//! adjudication are separate derived records; neither is asserted to be
//! historical truth. Fleiss' kappa is computed only for complete, balanced,
//! nominal human-rating panels within one task, context and ontology version.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const HERITAGE_ANNOTATION_SCHEMA_VERSION: &str =
    "luminous.heritage-annotation.v0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RaterKind {
    HumanExpert,
    Practitioner,
    CitizenScientist,
    Model,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotationRater {
    pub rater_id: String,
    pub kind: RaterKind,
    /// Task-specific scope tags, not a global expertise score.
    pub task_scope_tags: Vec<String>,
    /// Optional pseudonymous qualification or training receipt.
    pub qualification_receipt_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", content = "label_id", rename_all = "snake_case")]
pub enum AnnotationDecision {
    Assigned(String),
    Unsure,
    NotObservable,
    Abstained,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrdinalConfidence {
    pub score: u8,
    pub scale_min: u8,
    pub scale_max: u8,
    pub scale_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeritageAnnotation {
    pub annotation_id: String,
    pub task_id: String,
    pub sample_id: String,
    pub context_id: String,
    pub ontology_version: String,
    pub rubric_sha256: String,
    pub source_artifact_sha256: String,
    pub assessment_round_id: String,
    pub rater_id: String,
    pub decision: AnnotationDecision,
    /// Ordinal self-reported certainty; never interpreted as a probability.
    pub confidence: Option<OrdinalConfidence>,
    pub alternative_label_ids: Vec<String>,
    pub rationale_sha256: Option<String>,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "label_id", rename_all = "snake_case")]
pub enum AdjudicationOutcome {
    Resolved(String),
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotationAdjudication {
    pub adjudication_id: String,
    pub task_id: String,
    pub sample_id: String,
    pub context_id: String,
    pub ontology_version: String,
    pub source_annotation_ids: Vec<String>,
    pub adjudicator_rater_id: String,
    pub outcome: AdjudicationOutcome,
    pub rationale_sha256: String,
    /// True means this was not a blind adjudication relative to model output.
    pub model_outputs_shown: bool,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeritageAnnotationDataset {
    pub schema_version: String,
    pub ontology_version: String,
    pub label_ids: Vec<String>,
    pub confidence_scale_sha256: String,
    pub raters: Vec<AnnotationRater>,
    pub annotations: Vec<HeritageAnnotation>,
    pub adjudications: Vec<AnnotationAdjudication>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", content = "label_id", rename_all = "snake_case")]
pub enum PluralityOutcome {
    Resolved(String),
    Tie,
    NoAssignedVotes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluralityConsensusRecord {
    pub task_id: String,
    pub sample_id: String,
    pub context_id: String,
    pub ontology_version: String,
    pub source_annotation_ids: Vec<String>,
    pub method: String,
    pub outcome: PluralityOutcome,
    pub assigned_vote_count: usize,
    pub unsure_count: usize,
    pub not_observable_count: usize,
    pub abstained_count: usize,
    pub vote_counts: BTreeMap<String, usize>,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleissKappaReport {
    pub task_id: String,
    pub context_id: String,
    pub ontology_version: String,
    pub item_count: usize,
    pub raters_per_item: usize,
    pub category_prevalence: BTreeMap<String, f64>,
    pub mean_observed_agreement: f64,
    pub expected_chance_agreement: f64,
    /// None when chance agreement is 1.0 and kappa is undefined.
    pub fleiss_kappa: Option<f64>,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeritageAnnotationAuditReport {
    pub schema_version: String,
    pub annotation_count: usize,
    pub human_annotation_count: usize,
    pub model_annotation_count: usize,
    pub assigned_count: usize,
    pub unsure_count: usize,
    pub not_observable_count: usize,
    pub abstained_count: usize,
    pub adjudication_count: usize,
    pub model_output_shown_adjudication_count: usize,
    pub rater_count: usize,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageAnnotationError {
    SchemaVersionMismatch,
    EmptyIdentity(String),
    DuplicateIdentity(String),
    InvalidDigest(String),
    InvalidOntology,
    UnknownRater(String),
    UnknownLabel(String),
    DuplicateAssessmentForRaterAndRound(String),
    InvalidConfidence(String),
    InvalidAlternativeLabel(String),
    DifferentAnnotationContext(String),
    InvalidTimestamp(String),
    UnknownAnnotation(String),
    AdjudicationSetTooSmall(String),
    AdjudicationReferencesDifferentSubject(String),
    AdjudicatorMustBeHuman(String),
    InvalidAdjudicationOutcome(String),
    NoAnnotationsForPlurality,
    NoHumanPanel,
    TooFewItemsForAgreement,
    IncompleteOrAbstainingPanel(String),
    UnequalRaterCounts,
    RepeatedRaterWithinItem(String),
    InsufficientCategories,
    UndefinedChanceAgreement,
}

impl fmt::Display for HeritageAnnotationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for HeritageAnnotationError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_utc_timestamp(value: &str) -> bool {
    value.ends_with('Z')
        && value.contains('T')
        && value.len() >= 20
        && !value.chars().any(char::is_whitespace)
}

fn is_human(kind: RaterKind) -> bool {
    matches!(
        kind,
        RaterKind::HumanExpert | RaterKind::Practitioner | RaterKind::CitizenScientist
    )
}

pub fn audit_heritage_annotations(
    dataset: &HeritageAnnotationDataset,
) -> Result<HeritageAnnotationAuditReport, HeritageAnnotationError> {
    if dataset.schema_version != HERITAGE_ANNOTATION_SCHEMA_VERSION {
        return Err(HeritageAnnotationError::SchemaVersionMismatch);
    }
    if !non_empty(&dataset.ontology_version) || !valid_sha256(&dataset.confidence_scale_sha256) {
        return Err(HeritageAnnotationError::InvalidOntology);
    }

    let mut labels = BTreeSet::new();
    for label in &dataset.label_ids {
        if !non_empty(label) || !labels.insert(label.as_str()) {
            return Err(HeritageAnnotationError::InvalidOntology);
        }
    }
    if labels.len() < 2 {
        return Err(HeritageAnnotationError::InvalidOntology);
    }

    let mut raters = BTreeMap::<&str, &AnnotationRater>::new();
    for rater in &dataset.raters {
        if !non_empty(&rater.rater_id) {
            return Err(HeritageAnnotationError::EmptyIdentity("rater_id".to_owned()));
        }
        if raters.insert(&rater.rater_id, rater).is_some() {
            return Err(HeritageAnnotationError::DuplicateIdentity(
                rater.rater_id.clone(),
            ));
        }
        if rater.task_scope_tags.iter().any(|tag| !non_empty(tag)) {
            return Err(HeritageAnnotationError::EmptyIdentity(
                rater.rater_id.clone(),
            ));
        }
    }

    let mut annotation_ids = BTreeMap::<&str, &HeritageAnnotation>::new();
    let mut assessment_keys = BTreeSet::new();
    let mut annotation_subject_sources = BTreeMap::<(&str, &str, &str), (&str, &str)>::new();
    let mut human_annotation_count = 0usize;
    let mut model_annotation_count = 0usize;
    let mut assigned_count = 0usize;
    let mut unsure_count = 0usize;
    let mut not_observable_count = 0usize;
    let mut abstained_count = 0usize;

    for annotation in &dataset.annotations {
        for (field, value) in [
            ("annotation_id", annotation.annotation_id.as_str()),
            ("task_id", annotation.task_id.as_str()),
            ("sample_id", annotation.sample_id.as_str()),
            ("context_id", annotation.context_id.as_str()),
            ("ontology_version", annotation.ontology_version.as_str()),
            ("assessment_round_id", annotation.assessment_round_id.as_str()),
        ] {
            if !non_empty(value) {
                return Err(HeritageAnnotationError::EmptyIdentity(field.to_owned()));
            }
        }
        if annotation.ontology_version != dataset.ontology_version {
            return Err(HeritageAnnotationError::InvalidOntology);
        }
        if !valid_sha256(&annotation.rubric_sha256)
            || !valid_sha256(&annotation.source_artifact_sha256)
        {
            return Err(HeritageAnnotationError::InvalidDigest(
                annotation.annotation_id.clone(),
            ));
        }
        let subject_key = (
            annotation.task_id.as_str(),
            annotation.sample_id.as_str(),
            annotation.context_id.as_str(),
        );
        let subject_signature = (
            annotation.source_artifact_sha256.as_str(),
            annotation.rubric_sha256.as_str(),
        );
        if annotation_subject_sources
            .get(&subject_key)
            .is_some_and(|existing| *existing != subject_signature)
        {
            return Err(HeritageAnnotationError::DifferentAnnotationContext(
                annotation.annotation_id.clone(),
            ));
        }
        annotation_subject_sources
            .entry(subject_key)
            .or_insert(subject_signature);
        if !valid_utc_timestamp(&annotation.created_at_utc) {
            return Err(HeritageAnnotationError::InvalidTimestamp(
                annotation.annotation_id.clone(),
            ));
        }
        if annotation_ids
            .insert(&annotation.annotation_id, annotation)
            .is_some()
        {
            return Err(HeritageAnnotationError::DuplicateIdentity(
                annotation.annotation_id.clone(),
            ));
        }
        let rater = raters
            .get(annotation.rater_id.as_str())
            .ok_or_else(|| HeritageAnnotationError::UnknownRater(annotation.rater_id.clone()))?;
        if is_human(rater.kind) {
            human_annotation_count += 1;
        } else {
            model_annotation_count += 1;
        }
        let key = (
            annotation.task_id.as_str(),
            annotation.sample_id.as_str(),
            annotation.context_id.as_str(),
            annotation.rater_id.as_str(),
            annotation.assessment_round_id.as_str(),
        );
        if !assessment_keys.insert(key) {
            return Err(HeritageAnnotationError::DuplicateAssessmentForRaterAndRound(
                annotation.annotation_id.clone(),
            ));
        }

        match &annotation.decision {
            AnnotationDecision::Assigned(label_id) => {
                if !labels.contains(label_id.as_str()) {
                    return Err(HeritageAnnotationError::UnknownLabel(label_id.clone()));
                }
                assigned_count += 1;
            }
            AnnotationDecision::Unsure => unsure_count += 1,
            AnnotationDecision::NotObservable => not_observable_count += 1,
            AnnotationDecision::Abstained => abstained_count += 1,
        }

        let mut alternatives = BTreeSet::new();
        for alternative in &annotation.alternative_label_ids {
            if !labels.contains(alternative.as_str())
                || !alternatives.insert(alternative.as_str())
                || matches!(&annotation.decision, AnnotationDecision::Assigned(primary) if primary == alternative)
            {
                return Err(HeritageAnnotationError::InvalidAlternativeLabel(
                    annotation.annotation_id.clone(),
                ));
            }
        }

        if let Some(confidence) = &annotation.confidence {
            if confidence.scale_sha256 != dataset.confidence_scale_sha256
                || confidence.scale_min >= confidence.scale_max
                || confidence.score < confidence.scale_min
                || confidence.score > confidence.scale_max
            {
                return Err(HeritageAnnotationError::InvalidConfidence(
                    annotation.annotation_id.clone(),
                ));
            }
        }
        if annotation
            .rationale_sha256
            .as_deref()
            .is_some_and(|digest| !valid_sha256(digest))
        {
            return Err(HeritageAnnotationError::InvalidDigest(
                annotation.annotation_id.clone(),
            ));
        }
    }

    let mut adjudication_ids = BTreeSet::new();
    let mut model_output_shown_adjudication_count = 0usize;
    for adjudication in &dataset.adjudications {
        if !non_empty(&adjudication.adjudication_id)
            || !non_empty(&adjudication.task_id)
            || !non_empty(&adjudication.sample_id)
            || !non_empty(&adjudication.context_id)
            || !valid_sha256(&adjudication.rationale_sha256)
            || !valid_utc_timestamp(&adjudication.created_at_utc)
        {
            return Err(HeritageAnnotationError::EmptyIdentity(
                adjudication.adjudication_id.clone(),
            ));
        }
        if !adjudication_ids.insert(adjudication.adjudication_id.as_str()) {
            return Err(HeritageAnnotationError::DuplicateIdentity(
                adjudication.adjudication_id.clone(),
            ));
        }
        if adjudication.source_annotation_ids.len() < 2 {
            return Err(HeritageAnnotationError::AdjudicationSetTooSmall(
                adjudication.adjudication_id.clone(),
            ));
        }
        let mut source_ids = BTreeSet::new();
        for annotation_id in &adjudication.source_annotation_ids {
            if !source_ids.insert(annotation_id.as_str()) {
                return Err(HeritageAnnotationError::AdjudicationSetTooSmall(
                    adjudication.adjudication_id.clone(),
                ));
            }
            let annotation = annotation_ids.get(annotation_id.as_str()).ok_or_else(|| {
                HeritageAnnotationError::UnknownAnnotation(annotation_id.clone())
            })?;
            if annotation.task_id != adjudication.task_id
                || annotation.sample_id != adjudication.sample_id
                || annotation.context_id != adjudication.context_id
                || annotation.ontology_version != adjudication.ontology_version
            {
                return Err(HeritageAnnotationError::AdjudicationReferencesDifferentSubject(
                    adjudication.adjudication_id.clone(),
                ));
            }
        }
        let adjudicator = raters
            .get(adjudication.adjudicator_rater_id.as_str())
            .ok_or_else(|| {
                HeritageAnnotationError::UnknownRater(adjudication.adjudicator_rater_id.clone())
            })?;
        if !is_human(adjudicator.kind) {
            return Err(HeritageAnnotationError::AdjudicatorMustBeHuman(
                adjudication.adjudication_id.clone(),
            ));
        }
        if let AdjudicationOutcome::Resolved(label_id) = &adjudication.outcome {
            if !labels.contains(label_id.as_str()) {
                return Err(HeritageAnnotationError::InvalidAdjudicationOutcome(
                    adjudication.adjudication_id.clone(),
                ));
            }
        }
        if adjudication.model_outputs_shown {
            model_output_shown_adjudication_count += 1;
        }
    }

    Ok(HeritageAnnotationAuditReport {
        schema_version: HERITAGE_ANNOTATION_SCHEMA_VERSION.to_owned(),
        annotation_count: dataset.annotations.len(),
        human_annotation_count,
        model_annotation_count,
        assigned_count,
        unsure_count,
        not_observable_count,
        abstained_count,
        adjudication_count: dataset.adjudications.len(),
        model_output_shown_adjudication_count,
        rater_count: dataset.raters.len(),
        claim_ceiling: "annotation_observations_and_adjudication_records_structurally_valid_only"
            .to_owned(),
    })
}

pub fn derive_plurality_consensus(
    dataset: &HeritageAnnotationDataset,
    task_id: &str,
    sample_id: &str,
    context_id: &str,
) -> Result<PluralityConsensusRecord, HeritageAnnotationError> {
    audit_heritage_annotations(dataset)?;
    let mut matching: Vec<&HeritageAnnotation> = dataset
        .annotations
        .iter()
        .filter(|annotation| {
            annotation.task_id == task_id
                && annotation.sample_id == sample_id
                && annotation.context_id == context_id
        })
        .collect();
    if matching.is_empty() {
        return Err(HeritageAnnotationError::NoAnnotationsForPlurality);
    }
    matching.sort_by(|a, b| a.annotation_id.cmp(&b.annotation_id));

    let mut vote_counts = BTreeMap::<String, usize>::new();
    let mut unsure_count = 0usize;
    let mut not_observable_count = 0usize;
    let mut abstained_count = 0usize;
    for annotation in &matching {
        match &annotation.decision {
            AnnotationDecision::Assigned(label) => {
                *vote_counts.entry(label.clone()).or_default() += 1;
            }
            AnnotationDecision::Unsure => unsure_count += 1,
            AnnotationDecision::NotObservable => not_observable_count += 1,
            AnnotationDecision::Abstained => abstained_count += 1,
        }
    }
    let max_votes = vote_counts.values().copied().max().unwrap_or(0);
    let winners: Vec<String> = vote_counts
        .iter()
        .filter(|(_, votes)| **votes == max_votes && max_votes > 0)
        .map(|(label, _)| label.clone())
        .collect();
    let outcome = match winners.as_slice() {
        [] => PluralityOutcome::NoAssignedVotes,
        [winner] => PluralityOutcome::Resolved(winner.clone()),
        _ => PluralityOutcome::Tie,
    };

    Ok(PluralityConsensusRecord {
        task_id: task_id.to_owned(),
        sample_id: sample_id.to_owned(),
        context_id: context_id.to_owned(),
        ontology_version: dataset.ontology_version.clone(),
        source_annotation_ids: matching
            .iter()
            .map(|annotation| annotation.annotation_id.clone())
            .collect(),
        method: "unweighted_plurality_v0.1".to_owned(),
        outcome,
        assigned_vote_count: vote_counts.values().sum(),
        unsure_count,
        not_observable_count,
        abstained_count,
        vote_counts,
        claim_ceiling: "derived_vote_summary_not_archaeological_truth".to_owned(),
    })
}

/// Fleiss' kappa for complete, balanced nominal panels of human raters.
/// Abstentions/unsure/not-observable ratings are preserved in the dataset but
/// make this complete-panel statistic undefined for the selected task/context.
pub fn fleiss_kappa_report(
    dataset: &HeritageAnnotationDataset,
    task_id: &str,
    context_id: &str,
) -> Result<FleissKappaReport, HeritageAnnotationError> {
    audit_heritage_annotations(dataset)?;
    let raters = dataset
        .raters
        .iter()
        .map(|rater| (rater.rater_id.as_str(), rater.kind))
        .collect::<BTreeMap<_, _>>();
    let mut grouped = BTreeMap::<&str, Vec<&HeritageAnnotation>>::new();
    for annotation in dataset.annotations.iter().filter(|annotation| {
        annotation.task_id == task_id && annotation.context_id == context_id
    }) {
        if is_human(*raters.get(annotation.rater_id.as_str()).expect("validated rater")) {
            grouped
                .entry(annotation.sample_id.as_str())
                .or_default()
                .push(annotation);
        }
    }
    if grouped.is_empty() {
        return Err(HeritageAnnotationError::NoHumanPanel);
    }
    if grouped.len() < 2 {
        return Err(HeritageAnnotationError::TooFewItemsForAgreement);
    }

    let mut rater_count_per_item = None;
    let categories = dataset.label_ids.iter().cloned().collect::<Vec<_>>();
    let mut total_category_counts = vec![0usize; categories.len()];
    let mut mean_observed_agreement = 0.0;
    let mut item_count = 0usize;

    for (sample_id, annotations) in &grouped {
        let mut seen_raters = BTreeSet::new();
        let mut category_counts = vec![0usize; categories.len()];
        for annotation in annotations {
            if !seen_raters.insert(annotation.rater_id.as_str()) {
                return Err(HeritageAnnotationError::RepeatedRaterWithinItem(
                    (*sample_id).to_owned(),
                ));
            }
            let AnnotationDecision::Assigned(label_id) = &annotation.decision else {
                return Err(HeritageAnnotationError::IncompleteOrAbstainingPanel(
                    (*sample_id).to_owned(),
                ));
            };
            let category_index = categories
                .iter()
                .position(|category| category == label_id)
                .expect("validated ontology label");
            category_counts[category_index] += 1;
        }
        let n = annotations.len();
        if n < 2 {
            return Err(HeritageAnnotationError::IncompleteOrAbstainingPanel(
                (*sample_id).to_owned(),
            ));
        }
        if rater_count_per_item.is_some_and(|expected| expected != n) {
            return Err(HeritageAnnotationError::UnequalRaterCounts);
        }
        rater_count_per_item = Some(n);
        let observed = (category_counts
            .iter()
            .map(|count| count * count)
            .sum::<usize>() as f64
            - n as f64)
            / ((n * (n - 1)) as f64);
        mean_observed_agreement += observed;
        for (total, count) in total_category_counts.iter_mut().zip(category_counts) {
            *total += count;
        }
        item_count += 1;
    }

    let n_items = item_count as f64;
    let total_ratings = (item_count * rater_count_per_item.expect("panel count")) as f64;
    let mut category_prevalence = BTreeMap::new();
    let mut expected_chance_agreement = 0.0;
    for (category, count) in categories.iter().zip(total_category_counts) {
        let prevalence = count as f64 / total_ratings;
        expected_chance_agreement += prevalence * prevalence;
        category_prevalence.insert(category.clone(), prevalence);
    }
    mean_observed_agreement /= n_items;
    let fleiss_kappa = if (1.0 - expected_chance_agreement).abs() < f64::EPSILON {
        None
    } else {
        Some(
            (mean_observed_agreement - expected_chance_agreement)
                / (1.0 - expected_chance_agreement),
        )
    };

    Ok(FleissKappaReport {
        task_id: task_id.to_owned(),
        context_id: context_id.to_owned(),
        ontology_version: dataset.ontology_version.clone(),
        item_count,
        raters_per_item: rater_count_per_item.expect("non-empty group"),
        category_prevalence,
        mean_observed_agreement,
        expected_chance_agreement,
        fleiss_kappa,
        claim_ceiling: "nominal_inter_rater_agreement_for_selected_complete_balanced_panel_only"
            .to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(_value: &[u8]) -> String {
        "a".repeat(64)
    }

    fn assigned(id: &str, sample: &str, rater: &str, label: &str) -> HeritageAnnotation {
        HeritageAnnotation {
            annotation_id: id.to_owned(),
            task_id: "stone-wall-classification".to_owned(),
            sample_id: sample.to_owned(),
            context_id: "orthophoto-context-1".to_owned(),
            ontology_version: "stone-wall-ontology-1.0".to_owned(),
            rubric_sha256: digest(b"rubric-v1"),
            source_artifact_sha256: digest(b"orthophoto-1"),
            assessment_round_id: "round-1".to_owned(),
            rater_id: rater.to_owned(),
            decision: AnnotationDecision::Assigned(label.to_owned()),
            confidence: Some(OrdinalConfidence {
                score: 4,
                scale_min: 1,
                scale_max: 5,
                scale_sha256: digest(b"confidence-scale-1-5"),
            }),
            alternative_label_ids: Vec::new(),
            rationale_sha256: Some(digest(b"rationale")),
            created_at_utc: "2026-11-05T08:00:00Z".to_owned(),
        }
    }

    fn dataset() -> HeritageAnnotationDataset {
        HeritageAnnotationDataset {
            schema_version: HERITAGE_ANNOTATION_SCHEMA_VERSION.to_owned(),
            ontology_version: "stone-wall-ontology-1.0".to_owned(),
            label_ids: vec!["wall".to_owned(), "natural".to_owned()],
            confidence_scale_sha256: digest(b"confidence-scale-1-5"),
            raters: vec![
                AnnotationRater {
                    rater_id: "expert-1".to_owned(),
                    kind: RaterKind::HumanExpert,
                    task_scope_tags: vec!["dry-stone-wall".to_owned()],
                    qualification_receipt_ref: Some("qualification://expert-1".to_owned()),
                },
                AnnotationRater {
                    rater_id: "expert-2".to_owned(),
                    kind: RaterKind::HumanExpert,
                    task_scope_tags: vec!["dry-stone-wall".to_owned()],
                    qualification_receipt_ref: Some("qualification://expert-2".to_owned()),
                },
                AnnotationRater {
                    rater_id: "expert-3".to_owned(),
                    kind: RaterKind::Practitioner,
                    task_scope_tags: vec!["dry-stone-wall".to_owned()],
                    qualification_receipt_ref: None,
                },
            ],
            annotations: vec![
                assigned("a-1", "sample-1", "expert-1", "wall"),
                assigned("a-2", "sample-1", "expert-2", "wall"),
                assigned("a-3", "sample-1", "expert-3", "natural"),
                assigned("a-4", "sample-2", "expert-1", "wall"),
                assigned("a-5", "sample-2", "expert-2", "natural"),
                assigned("a-6", "sample-2", "expert-3", "natural"),
            ],
            adjudications: Vec::new(),
        }
    }

    #[test]
    fn valid_annotation_dataset_preserves_independent_votes() {
        let report = audit_heritage_annotations(&dataset()).expect("valid annotation data");
        assert_eq!(report.annotation_count, 6);
        assert_eq!(report.human_annotation_count, 6);
        assert_eq!(report.assigned_count, 6);
        assert_eq!(report.unsure_count, 0);
    }

    #[test]
    fn plurality_tie_remains_unresolved() {
        let mut input = dataset();
        input.annotations.truncate(2);
        input.annotations[1].decision = AnnotationDecision::Assigned("natural".to_owned());
        let consensus = derive_plurality_consensus(
            &input,
            "stone-wall-classification",
            "sample-1",
            "orthophoto-context-1",
        )
        .expect("plurality");
        assert_eq!(consensus.outcome, PluralityOutcome::Tie);
        assert_eq!(consensus.assigned_vote_count, 3);
        assert_eq!(
            consensus.claim_ceiling,
            "derived_vote_summary_not_archaeological_truth"
        );
    }

    #[test]
    fn plurality_winner_is_a_derived_record_not_a_replacement_label() {
        let mut input = dataset();
        input.annotations[2].decision = AnnotationDecision::Assigned("wall".to_owned());
        let consensus = derive_plurality_consensus(
            &input,
            "stone-wall-classification",
            "sample-1",
            "orthophoto-context-1",
        )
        .expect("plurality winner");
        assert_eq!(
            consensus.outcome,
            PluralityOutcome::Resolved("wall".to_owned())
        );
        assert_eq!(input.annotations[2].decision, AnnotationDecision::Assigned("wall".to_owned()));
        assert_eq!(consensus.source_annotation_ids.len(), 3);
    }

    #[test]
    fn unsure_not_observable_and_abstained_are_not_negative_labels() {
        let mut input = dataset();
        input.annotations[0].decision = AnnotationDecision::Unsure;
        input.annotations[1].decision = AnnotationDecision::NotObservable;
        input.annotations[2].decision = AnnotationDecision::Abstained;
        let report = audit_heritage_annotations(&input).expect("explicit missingness states");
        assert_eq!(report.unsure_count, 1);
        assert_eq!(report.not_observable_count, 1);
        assert_eq!(report.abstained_count, 1);
    }

    #[test]
    fn fleiss_kappa_reports_complete_balanced_nominal_panel_only() {
        let report = fleiss_kappa_report(
            &dataset(),
            "stone-wall-classification",
            "orthophoto-context-1",
        )
        .expect("balanced Fleiss panel");
        assert_eq!(report.item_count, 2);
        assert_eq!(report.raters_per_item, 3);
        assert!((report.mean_observed_agreement - 1.0 / 3.0).abs() < 1e-12);
        assert!((report.expected_chance_agreement - 0.5).abs() < 1e-12);
        assert!((report.fleiss_kappa.expect("defined kappa") + 1.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn fleiss_kappa_refuses_incomplete_or_abstaining_panels() {
        let mut input = dataset();
        input.annotations[0].decision = AnnotationDecision::Unsure;
        assert!(matches!(
            fleiss_kappa_report(&input, "stone-wall-classification", "orthophoto-context-1"),
            Err(HeritageAnnotationError::IncompleteOrAbstainingPanel(_))
        ));
    }

    #[test]
    fn fleiss_kappa_refuses_unequal_rater_counts() {
        let mut input = dataset();
        input.annotations.pop();
        assert_eq!(
            fleiss_kappa_report(&input, "stone-wall-classification", "orthophoto-context-1"),
            Err(HeritageAnnotationError::UnequalRaterCounts)
        );
    }

    #[test]
    fn same_sample_context_cannot_mix_source_assets_or_rubrics() {
        let mut input = dataset();
        input.annotations[1].rubric_sha256 = digest(b"other-rubric");
        assert!(matches!(
            audit_heritage_annotations(&input),
            Err(HeritageAnnotationError::DifferentAnnotationContext(_))
        ));
    }

    #[test]
    fn repeated_ratings_are_allowed_in_distinct_rounds_but_not_same_round() {
        let mut input = dataset();
        let mut repeat = input.annotations[0].clone();
        repeat.annotation_id = "a-7".to_owned();
        repeat.assessment_round_id = "round-2".to_owned();
        repeat.created_at_utc = "2026-11-06T08:00:00Z".to_owned();
        input.annotations.push(repeat);
        assert!(audit_heritage_annotations(&input).is_ok());

        input.annotations[6].assessment_round_id = "round-1".to_owned();
        assert!(matches!(
            audit_heritage_annotations(&input),
            Err(HeritageAnnotationError::DuplicateAssessmentForRaterAndRound(_))
        ));
    }

    #[test]
    fn alternative_labels_must_belong_to_ontology_and_differ_from_primary() {
        let mut input = dataset();
        input.annotations[0].alternative_label_ids = vec!["unknown".to_owned()];
        assert!(matches!(
            audit_heritage_annotations(&input),
            Err(HeritageAnnotationError::InvalidAlternativeLabel(_))
        ));
        input.annotations[0].alternative_label_ids = vec!["wall".to_owned()];
        assert!(matches!(
            audit_heritage_annotations(&input),
            Err(HeritageAnnotationError::InvalidAlternativeLabel(_))
        ));
    }

    #[test]
    fn ordinal_confidence_is_not_silently_treated_as_probability() {
        let mut input = dataset();
        input.annotations[0].confidence.as_mut().unwrap().score = 6;
        assert!(matches!(
            audit_heritage_annotations(&input),
            Err(HeritageAnnotationError::InvalidConfidence(_))
        ));
    }

    #[test]
    fn adjudication_preserves_source_annotations_and_model_visibility() {
        let mut input = dataset();
        input.adjudications.push(AnnotationAdjudication {
            adjudication_id: "adj-1".to_owned(),
            task_id: "stone-wall-classification".to_owned(),
            sample_id: "sample-1".to_owned(),
            context_id: "orthophoto-context-1".to_owned(),
            ontology_version: "stone-wall-ontology-1.0".to_owned(),
            source_annotation_ids: vec!["a-1".to_owned(), "a-2".to_owned(), "a-3".to_owned()],
            adjudicator_rater_id: "expert-1".to_owned(),
            outcome: AdjudicationOutcome::Unresolved,
            rationale_sha256: digest(b"adjudication-rationale"),
            model_outputs_shown: false,
            created_at_utc: "2026-11-05T09:00:00Z".to_owned(),
        });
        let report = audit_heritage_annotations(&input).expect("valid unresolved adjudication");
        assert_eq!(report.adjudication_count, 1);
        assert_eq!(input.annotations.len(), 6);
        assert_eq!(report.model_output_shown_adjudication_count, 0);
    }

    #[test]
    fn model_rater_cannot_issue_human_adjudication() {
        let mut input = dataset();
        input.raters.push(AnnotationRater {
            rater_id: "model-1".to_owned(),
            kind: RaterKind::Model,
            task_scope_tags: vec!["stone-wall-detection".to_owned()],
            qualification_receipt_ref: None,
        });
        input.adjudications.push(AnnotationAdjudication {
            adjudication_id: "adj-model".to_owned(),
            task_id: "stone-wall-classification".to_owned(),
            sample_id: "sample-1".to_owned(),
            context_id: "orthophoto-context-1".to_owned(),
            ontology_version: "stone-wall-ontology-1.0".to_owned(),
            source_annotation_ids: vec!["a-1".to_owned(), "a-2".to_owned()],
            adjudicator_rater_id: "model-1".to_owned(),
            outcome: AdjudicationOutcome::Resolved("wall".to_owned()),
            rationale_sha256: digest(b"adjudication-rationale"),
            model_outputs_shown: true,
            created_at_utc: "2026-11-05T09:00:00Z".to_owned(),
        });
        assert!(matches!(
            audit_heritage_annotations(&input),
            Err(HeritageAnnotationError::AdjudicatorMustBeHuman(_))
        ));
    }

    #[test]
    fn annotation_schema_and_synthetic_fixture_parse_and_audit() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/heritage-annotation-v0.1.schema.json"
        ))
        .expect("annotation schema JSON");
        let fixture: HeritageAnnotationDataset = serde_json::from_str(include_str!(
            "../../schemas/examples/heritage-annotation-synthetic-v0.1.json"
        ))
        .expect("annotation fixture");
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(audit_heritage_annotations(&fixture).is_ok());
    }

    #[test]
    fn annotation_cannot_claim_unknown_label_or_rater() {
        let mut input = dataset();
        input.annotations[0].decision = AnnotationDecision::Assigned("burial_mound".to_owned());
        assert!(matches!(
            audit_heritage_annotations(&input),
            Err(HeritageAnnotationError::UnknownLabel(_))
        ));
        input.annotations[0].decision = AnnotationDecision::Assigned("wall".to_owned());
        input.annotations[0].rater_id = "missing-rater".to_owned();
        assert!(matches!(
            audit_heritage_annotations(&input),
            Err(HeritageAnnotationError::UnknownRater(_))
        ));
    }
}
