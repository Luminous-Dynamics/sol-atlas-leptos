// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Deterministic project-cluster split construction for physical benchmarks.
//!
//! A project is the indivisible split unit. Activities, radargrams, traces,
//! annotations and trench references inherit their parent project's role.
//! This guards against acquisition-level leakage but does not replace spatial
//! overlap, near-duplicate or source-lineage audits.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectActivityGroup {
    pub project_id: String,
    pub activity_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectDatasetIndex {
    pub dataset_id: String,
    pub projects: Vec<ProjectActivityGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSplitFold {
    pub fold_id: String,
    pub fit_project_ids: Vec<String>,
    pub calibration_project_ids: Vec<String>,
    pub blind_project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSplitManifest {
    pub manifest_id: String,
    pub dataset_id: String,
    pub split_policy: String,
    pub calibration_projects_per_fold: usize,
    pub project_count: usize,
    pub folds: Vec<ProjectSplitFold>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectSplitError {
    EmptyIdentifier(&'static str),
    DatasetIdentityMismatch,
    InvalidSplitPolicy,
    DuplicateProject(String),
    DuplicateActivity(String),
    ProjectWithoutActivities(String),
    TooFewProjects,
    InvalidCalibrationProjectCount,
    FoldCountMismatch,
    DuplicateFold(String),
    UnknownProject(String),
    ProjectAssignedMultipleRoles(String),
    ProjectNotAssigned(String),
    BlindProjectRepeated(String),
    CalibrationCountMismatch(String),
    FoldIdMismatch(String),
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn canonical_projects(
    index: &ProjectDatasetIndex,
) -> Result<Vec<ProjectActivityGroup>, Vec<ProjectSplitError>> {
    let mut errors = Vec::new();
    if !non_empty(&index.dataset_id) {
        errors.push(ProjectSplitError::EmptyIdentifier("dataset_id"));
    }

    let mut project_ids = BTreeSet::new();
    let mut activity_ids = BTreeSet::new();
    for project in &index.projects {
        if !non_empty(&project.project_id) {
            errors.push(ProjectSplitError::EmptyIdentifier("project_id"));
        }
        if !project_ids.insert(project.project_id.clone()) {
            errors.push(ProjectSplitError::DuplicateProject(
                project.project_id.clone(),
            ));
        }
        if project.activity_ids.is_empty() {
            errors.push(ProjectSplitError::ProjectWithoutActivities(
                project.project_id.clone(),
            ));
        }
        let mut local_activities = BTreeSet::new();
        for activity_id in &project.activity_ids {
            if !non_empty(activity_id) {
                errors.push(ProjectSplitError::EmptyIdentifier("activity_id"));
            }
            if !local_activities.insert(activity_id.clone())
                || !activity_ids.insert(activity_id.clone())
            {
                errors.push(ProjectSplitError::DuplicateActivity(
                    activity_id.clone(),
                ));
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut projects = index.projects.clone();
    projects.sort_by(|left, right| left.project_id.cmp(&right.project_id));
    for project in &mut projects {
        project.activity_ids.sort();
    }
    Ok(projects)
}

/// Build deterministic leave-one-project-out folds. Each fold uses exactly
/// one blind project, a disjoint rotating calibration-project set, and all
/// remaining projects for fitting. Output order is lexicographically stable.
pub fn build_leave_one_project_out(
    index: &ProjectDatasetIndex,
    calibration_projects_per_fold: usize,
) -> Result<ProjectSplitManifest, Vec<ProjectSplitError>> {
    let projects = canonical_projects(index)?;
    let count = projects.len();

    if count < 3 {
        return Err(vec![ProjectSplitError::TooFewProjects]);
    }
    if calibration_projects_per_fold == 0 || calibration_projects_per_fold >= count - 1 {
        return Err(vec![ProjectSplitError::InvalidCalibrationProjectCount]);
    }

    let project_ids: Vec<String> = projects
        .iter()
        .map(|project| project.project_id.clone())
        .collect();
    let mut folds = Vec::with_capacity(count);

    for blind_index in 0..count {
        let blind_project_id = project_ids[blind_index].clone();
        let calibration_project_ids: Vec<String> = (1..=calibration_projects_per_fold)
            .map(|offset| project_ids[(blind_index + offset) % count].clone())
            .collect();
        let calibration_set: BTreeSet<&str> = calibration_project_ids
            .iter()
            .map(String::as_str)
            .collect();
        let fit_project_ids: Vec<String> = project_ids
            .iter()
            .filter(|project_id| {
                project_id.as_str() != blind_project_id
                    && !calibration_set.contains(project_id.as_str())
            })
            .cloned()
            .collect();

        folds.push(ProjectSplitFold {
            fold_id: format!("leave-project-out:{blind_project_id}"),
            fit_project_ids,
            calibration_project_ids,
            blind_project_id,
        });
    }

    let manifest = ProjectSplitManifest {
        manifest_id: format!("{}:leave-project-out-v0.1", index.dataset_id),
        dataset_id: index.dataset_id.clone(),
        split_policy: "leave_one_project_out_with_rotating_project_calibration".to_string(),
        calibration_projects_per_fold,
        project_count: count,
        folds,
    };

    validate_project_split_manifest(index, &manifest)?;
    Ok(manifest)
}

/// Check that every project is assigned exactly one role per fold and is the
/// blind project exactly once across the full leave-one-project-out manifest.
pub fn validate_project_split_manifest(
    index: &ProjectDatasetIndex,
    manifest: &ProjectSplitManifest,
) -> Result<(), Vec<ProjectSplitError>> {
    let projects = canonical_projects(index)?;
    let project_ids: BTreeSet<String> = projects
        .iter()
        .map(|project| project.project_id.clone())
        .collect();
    let mut errors = Vec::new();

    if !non_empty(&manifest.manifest_id) {
        errors.push(ProjectSplitError::EmptyIdentifier("manifest_id"));
    }
    if manifest.dataset_id != index.dataset_id {
        errors.push(ProjectSplitError::DatasetIdentityMismatch);
    }
    if manifest.split_policy != "leave_one_project_out_with_rotating_project_calibration" {
        errors.push(ProjectSplitError::InvalidSplitPolicy);
    }
    if manifest.calibration_projects_per_fold == 0
        || manifest.calibration_projects_per_fold >= project_ids.len().saturating_sub(1)
    {
        errors.push(ProjectSplitError::InvalidCalibrationProjectCount);
    }
    if manifest.project_count != project_ids.len()
        || manifest.folds.len() != project_ids.len()
    {
        errors.push(ProjectSplitError::FoldCountMismatch);
    }

    let mut fold_ids = BTreeSet::new();
    let mut blind_projects = BTreeSet::new();

    for fold in &manifest.folds {
        if !fold_ids.insert(fold.fold_id.as_str()) {
            errors.push(ProjectSplitError::DuplicateFold(fold.fold_id.clone()));
        }
        if fold.fold_id != format!("leave-project-out:{}", fold.blind_project_id) {
            errors.push(ProjectSplitError::FoldIdMismatch(fold.fold_id.clone()));
        }

        let mut assigned = BTreeSet::new();
        for project_id in &fold.fit_project_ids {
            if !project_ids.contains(project_id) {
                errors.push(ProjectSplitError::UnknownProject(project_id.clone()));
            }
            if !assigned.insert(project_id.as_str()) {
                errors.push(ProjectSplitError::ProjectAssignedMultipleRoles(
                    project_id.clone(),
                ));
            }
        }
        for project_id in &fold.calibration_project_ids {
            if !project_ids.contains(project_id) {
                errors.push(ProjectSplitError::UnknownProject(project_id.clone()));
            }
            if !assigned.insert(project_id.as_str()) {
                errors.push(ProjectSplitError::ProjectAssignedMultipleRoles(
                    project_id.clone(),
                ));
            }
        }
        if !project_ids.contains(&fold.blind_project_id) {
            errors.push(ProjectSplitError::UnknownProject(
                fold.blind_project_id.clone(),
            ));
        }
        if !assigned.insert(fold.blind_project_id.as_str()) {
            errors.push(ProjectSplitError::ProjectAssignedMultipleRoles(
                fold.blind_project_id.clone(),
            ));
        }
        if assigned.len() != project_ids.len() {
            for project_id in &project_ids {
                if !assigned.contains(project_id.as_str()) {
                    errors.push(ProjectSplitError::ProjectNotAssigned(
                        project_id.clone(),
                    ));
                }
            }
        }
        if fold.calibration_project_ids.len() != manifest.calibration_projects_per_fold {
            errors.push(ProjectSplitError::CalibrationCountMismatch(
                fold.fold_id.clone(),
            ));
        }
        if !blind_projects.insert(fold.blind_project_id.as_str()) {
            errors.push(ProjectSplitError::BlindProjectRepeated(
                fold.blind_project_id.clone(),
            ));
        }
    }

    if blind_projects.len() != project_ids.len() {
        errors.push(ProjectSplitError::FoldCountMismatch);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROJECT_COUNT: usize = 13;

    const FIXTURE: &str = include_str!(
        "../../research/earth-observability/fixtures/project-split-input-synthetic-v0.1.json"
    );

    fn index() -> ProjectDatasetIndex {
        serde_json::from_str(FIXTURE).expect("project split fixture parses")
    }

    #[test]
    fn thirteen_project_leave_one_out_is_deterministic_and_disjoint() {
        let index = index();
        let first = build_leave_one_project_out(&index, 3).unwrap();
        let second = build_leave_one_project_out(&index, 3).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.folds.len(), 13);
        for fold in &first.folds {
            assert_eq!(fold.fit_project_ids.len(), 9);
            assert_eq!(fold.calibration_project_ids.len(), 3);
            assert!(!fold.fit_project_ids.contains(&fold.blind_project_id));
            assert!(!fold.calibration_project_ids.contains(&fold.blind_project_id));
            assert!(
                fold.fit_project_ids
                    .iter()
                    .all(|id| !fold.calibration_project_ids.contains(id))
            );
        }
    }

    #[test]
    fn each_project_is_blind_exactly_once() {
        let index = index();
        let manifest = build_leave_one_project_out(&index, 3).unwrap();
        let blind: BTreeSet<&str> = manifest
            .folds
            .iter()
            .map(|fold| fold.blind_project_id.as_str())
            .collect();
        assert_eq!(blind.len(), PROJECT_COUNT);
        assert_eq!(
            blind,
            index.projects.iter().map(|project| project.project_id.as_str()).collect()
        );
    }

    #[test]
    fn duplicate_activity_across_projects_is_rejected() {
        let mut index = index();
        index.projects[1].activity_ids[0] = index.projects[0].activity_ids[0].clone();
        assert!(canonical_projects(&index)
            .unwrap_err()
            .contains(&ProjectSplitError::DuplicateActivity(
                "activity-01-a".to_string()
            )));
    }

    #[test]
    fn insufficient_project_clusters_are_rejected() {
        let mut index = index();
        index.projects.truncate(2);
        assert_eq!(
            build_leave_one_project_out(&index, 1),
            Err(vec![ProjectSplitError::TooFewProjects])
        );
    }

    #[test]
    fn invalid_calibration_count_is_rejected() {
        let index = index();
        assert_eq!(
            build_leave_one_project_out(&index, 12),
            Err(vec![ProjectSplitError::InvalidCalibrationProjectCount])
        );
    }

    #[test]
    fn a_manually_contaminated_fold_is_rejected() {
        let index = index();
        let mut manifest = build_leave_one_project_out(&index, 3).unwrap();
        let blind_project_id = manifest.folds[0].blind_project_id.clone();
        manifest.folds[0].fit_project_ids.push(blind_project_id.clone());
        assert!(
            validate_project_split_manifest(&index, &manifest)
                .unwrap_err()
                .contains(&ProjectSplitError::ProjectAssignedMultipleRoles(blind_project_id))
        );
    }
}
