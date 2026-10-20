// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Versioned task-graph projection for multi-step field work.
//!
//! Readiness is derived from dependencies instead of stored as another state
//! flag. This prevents a reported completion from unlocking downstream tasks;
//! dependencies unlock only after verification has been recorded.
//!
//! This module is a planning/UI primitive, not a safety gate or authorization
//! engine. The canonical service must independently validate any state change.

use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

pub const TASK_GRAPH_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskGraphV1 {
    pub schema_version: u16,
    pub graph_id: String,
    pub revision: u64,
    pub tasks: Vec<TaskNodeV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskNodeV1 {
    pub task_id: String,
    pub title: String,
    /// All listed prerequisites must have VerificationRecorded status.
    pub depends_on: Vec<String>,
    /// Higher priority comes first among tasks that are currently ready.
    pub priority: u8,
    pub status: TaskStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    CompletionReported,
    VerificationRecorded,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskAction {
    Start,
    ReportCompletion,
    RecordVerification,
    Cancel,
}


pub const TASK_REVALIDATION_PLAN_SCHEMA_VERSION: u16 = 1;

/// Preview of the downstream consequences of challenging a verified task.
///
/// This is an immutable analysis result, not an invalidation command. Applying
/// it requires an authoritative service to re-check the graph revision,
/// corrected evidence, identity, permission and domain-specific rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskRevalidationPlanV1 {
    pub schema_version: u16,
    pub graph_id: String,
    pub expected_graph_revision: u64,
    pub source_task_id: String,
    pub source_status: TaskStatus,
    pub reason_code: String,
    pub correction_evidence_ref: String,
    pub impacts: Vec<TaskRevalidationImpactV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskRevalidationImpactV1 {
    pub task_id: String,
    pub prior_status: TaskStatus,
    pub kind: RevalidationImpactKind,
    /// Stable path from the challenged task to this downstream task.
    pub dependency_path: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevalidationImpactKind {
    /// The task is active, completion-reported, or previously verified.
    /// An authorized workflow must decide how to revalidate it.
    RequiresRevalidation,
    /// The task has not started and must remain unavailable until prerequisites
    /// are verified again.
    RemainsBlocked,
    /// The task was cancelled; this plan does not reopen it.
    CancelledNoAction,
}

impl TaskGraphV1 {
    pub fn validate(&self) -> Result<(), TaskGraphError> {
        if self.schema_version != TASK_GRAPH_SCHEMA_VERSION {
            return Err(TaskGraphError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        if self.graph_id.trim().is_empty() {
            return Err(TaskGraphError::EmptyGraphId);
        }
        if self.revision == 0 {
            return Err(TaskGraphError::ZeroRevision);
        }
        if self.tasks.is_empty() {
            return Err(TaskGraphError::EmptyTaskSet);
        }

        let mut by_id = BTreeMap::new();
        for (index, task) in self.tasks.iter().enumerate() {
            if task.task_id.trim().is_empty() {
                return Err(TaskGraphError::EmptyTaskId);
            }
            if task.title.trim().is_empty() {
                return Err(TaskGraphError::EmptyTaskTitle(task.task_id.clone()));
            }
            if by_id.insert(task.task_id.as_str(), index).is_some() {
                return Err(TaskGraphError::DuplicateTaskId(task.task_id.clone()));
            }

            let mut dependencies = BTreeSet::new();
            for dependency_id in &task.depends_on {
                if dependency_id == &task.task_id {
                    return Err(TaskGraphError::SelfDependency(task.task_id.clone()));
                }
                if !dependencies.insert(dependency_id.as_str()) {
                    return Err(TaskGraphError::DuplicateDependency {
                        task_id: task.task_id.clone(),
                        dependency_id: dependency_id.clone(),
                    });
                }
            }
        }

        for task in &self.tasks {
            for dependency_id in &task.depends_on {
                if !by_id.contains_key(dependency_id.as_str()) {
                    return Err(TaskGraphError::MissingDependency {
                        task_id: task.task_id.clone(),
                        dependency_id: dependency_id.clone(),
                    });
                }
            }
        }

        for task in &self.tasks {
            if matches!(
                task.status,
                TaskStatus::InProgress
                    | TaskStatus::CompletionReported
                    | TaskStatus::VerificationRecorded
            ) {
                let unresolved: Vec<String> = task
                    .depends_on
                    .iter()
                    .filter(|dependency_id| {
                        self.tasks[by_id[dependency_id.as_str()]].status
                            != TaskStatus::VerificationRecorded
                    })
                    .cloned()
                    .collect();
                if !unresolved.is_empty() {
                    return Err(TaskGraphError::ActiveTaskHasUnverifiedPrerequisites {
                        task_id: task.task_id.clone(),
                        dependency_ids: unresolved,
                    });
                }
            }
        }

        self.topological_order_unchecked().map(|_| ())
    }

    /// Return a stable topological order. Independent ties are resolved by task ID.
    pub fn topological_order(&self) -> Result<Vec<String>, TaskGraphError> {
        self.validate()?;
        self.topological_order_unchecked()
    }

    /// Return pending tasks whose full prerequisite set is verification-recorded.
    ///
    /// Results are deterministic: descending priority, then ascending task ID.
    pub fn ready_tasks(&self) -> Result<Vec<&TaskNodeV1>, TaskGraphError> {
        self.validate()?;
        let status_by_id = self.status_by_id();
        let mut ready: Vec<&TaskNodeV1> = self
            .tasks
            .iter()
            .filter(|task| {
                task.status == TaskStatus::Pending
                    && task.depends_on.iter().all(|dependency_id| {
                        status_by_id.get(dependency_id.as_str())
                            == Some(&TaskStatus::VerificationRecorded)
                    })
            })
            .collect();

        ready.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.task_id.cmp(&right.task_id))
        });
        Ok(ready)
    }

    /// Explain the downstream impact of challenging a previously verified task.
    ///
    /// The method does not mutate task status or graph revision. The returned
    /// plan is bound to the observed revision and must not be applied if the
    /// canonical graph has changed.
    pub fn plan_verification_invalidation(
        &self,
        source_task_id: &str,
        reason_code: &str,
        correction_evidence_ref: &str,
    ) -> Result<TaskRevalidationPlanV1, TaskGraphError> {
        self.validate()?;
        validate_revalidation_inputs(source_task_id, reason_code, correction_evidence_ref)?;

        let source_index = self
            .tasks
            .iter()
            .position(|task| task.task_id == source_task_id)
            .ok_or_else(|| TaskGraphError::TaskNotFound(source_task_id.to_owned()))?;
        if self.tasks[source_index].status != TaskStatus::VerificationRecorded {
            return Err(TaskGraphError::TaskNotVerified(source_task_id.to_owned()));
        }

        let id_to_index: BTreeMap<&str, usize> = self
            .tasks
            .iter()
            .enumerate()
            .map(|(index, task)| (task.task_id.as_str(), index))
            .collect();
        let mut dependents = vec![Vec::new(); self.tasks.len()];
        for (task_index, task) in self.tasks.iter().enumerate() {
            for dependency_id in &task.depends_on {
                let dependency_index = *id_to_index
                    .get(dependency_id.as_str())
                    .expect("validated dependency has an index");
                dependents[dependency_index].push(task_index);
            }
        }
        sort_dependents(&mut dependents, &self.tasks);
        let paths = collect_dependent_paths(source_index, &dependents);
        Ok(build_revalidation_plan(
            self,
            source_index,
            reason_code,
            correction_evidence_ref,
            paths,
        ))
    }

    /// Explain why a task is not dependency-ready, including cancelled or
    /// completion-reported prerequisites. This is explanation, not authorization.
    pub fn blocked_by(&self, task_id: &str) -> Result<Vec<String>, TaskGraphError> {
        self.validate()?;
        let task = self
            .tasks
            .iter()
            .find(|task| task.task_id == task_id)
            .ok_or_else(|| TaskGraphError::TaskNotFound(task_id.to_owned()))?;
        let status_by_id = self.status_by_id();

        Ok(task
            .depends_on
            .iter()
            .filter(|dependency_id| {
                status_by_id.get(dependency_id.as_str()) != Some(&TaskStatus::VerificationRecorded)
            })
            .cloned()
            .collect())
    }

    /// Apply a local projection transition with deterministic dependency checks.
    ///
    /// This mutates only the in-memory graph. Persisting this state or treating
    /// it as a confirmed physical/work-order transition requires an authoritative
    /// service response and revision check.
    pub fn apply(&mut self, task_id: &str, action: TaskAction) -> Result<(), TaskGraphError> {
        self.validate()?;
        let index = self
            .tasks
            .iter()
            .position(|task| task.task_id == task_id)
            .ok_or_else(|| TaskGraphError::TaskNotFound(task_id.to_owned()))?;
        let task = &self.tasks[index];
        let old_status = task.status;
        let task_id_owned = task.task_id.clone();

        let status_by_id = self.status_by_id();
        let unresolved_dependencies: Vec<String> = task
            .depends_on
            .iter()
            .filter(|dependency_id| {
                status_by_id.get(dependency_id.as_str()) != Some(&TaskStatus::VerificationRecorded)
            })
            .cloned()
            .collect();

        let next_status = match (old_status, action) {
            (TaskStatus::Pending, TaskAction::Start) if unresolved_dependencies.is_empty() => {
                TaskStatus::InProgress
            }
            (TaskStatus::Pending, TaskAction::Start) => {
                return Err(TaskGraphError::DependencyNotVerified {
                    task_id: task_id_owned,
                    dependency_ids: unresolved_dependencies,
                });
            }
            (TaskStatus::InProgress, TaskAction::ReportCompletion) => {
                TaskStatus::CompletionReported
            }
            (TaskStatus::CompletionReported, TaskAction::RecordVerification) => {
                TaskStatus::VerificationRecorded
            }
            (TaskStatus::Pending, TaskAction::Cancel) => TaskStatus::Cancelled,
            _ => {
                return Err(TaskGraphError::InvalidTransition {
                    task_id: task_id_owned,
                    action,
                    status: old_status,
                });
            }
        };

        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or(TaskGraphError::RevisionExhausted)?;
        self.tasks[index].status = next_status;
        self.revision = next_revision;
        Ok(())
    }

    fn status_by_id(&self) -> BTreeMap<&str, TaskStatus> {
        self.tasks
            .iter()
            .map(|task| (task.task_id.as_str(), task.status))
            .collect()
    }

    fn topological_order_unchecked(&self) -> Result<Vec<String>, TaskGraphError> {
        let mut indegree: BTreeMap<&str, usize> = self
            .tasks
            .iter()
            .map(|task| (task.task_id.as_str(), task.depends_on.len()))
            .collect();
        let mut dependents: BTreeMap<&str, Vec<&str>> = self
            .tasks
            .iter()
            .map(|task| (task.task_id.as_str(), Vec::new()))
            .collect();

        for task in &self.tasks {
            for dependency_id in &task.depends_on {
                if let Some(children) = dependents.get_mut(dependency_id.as_str()) {
                    children.push(task.task_id.as_str());
                }
            }
        }
        for children in dependents.values_mut() {
            children.sort_unstable();
        }

        let mut ready: BTreeSet<&str> = indegree
            .iter()
            .filter_map(|(task_id, count)| (*count == 0).then_some(*task_id))
            .collect();
        let mut ordered = Vec::with_capacity(self.tasks.len());

        while let Some(task_id) = ready.pop_first() {
            ordered.push(task_id.to_owned());
            if let Some(children) = dependents.get(task_id) {
                for child_id in children {
                    let degree = indegree
                        .get_mut(child_id)
                        .expect("validated graph has a degree entry for each task");
                    *degree -= 1;
                    if *degree == 0 {
                        ready.insert(child_id);
                    }
                }
            }
        }

        if ordered.len() != self.tasks.len() {
            return Err(TaskGraphError::CycleDetected);
        }
        Ok(ordered)
    }
}

/// A validated runtime index for repeated task-frontier queries and transitions.
///
/// The serialized graph remains the interchange format. This compiled form
/// indexes IDs and dependency edges once, then maintains the ready frontier
/// incrementally as tasks transition. It is a local projection, not an authority
/// boundary; callers still need the canonical service to check actor, revision,
/// evidence, permissions and durable idempotency.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledTaskGraph {
    graph: TaskGraphV1,
    id_to_index: BTreeMap<String, usize>,
    prerequisites: Vec<Vec<usize>>,
    dependents: Vec<Vec<usize>>,
    unresolved_prerequisites: Vec<usize>,
    ready: BTreeSet<(Reverse<u8>, String, usize)>,
}

impl CompiledTaskGraph {
    /// Validate and compile a graph once. The deterministic ready index stores
    /// descending priority, then task ID, independent of input vector order.
    pub fn compile(graph: TaskGraphV1) -> Result<Self, TaskGraphError> {
        graph.validate()?;

        let id_to_index: BTreeMap<String, usize> = graph
            .tasks
            .iter()
            .enumerate()
            .map(|(index, task)| (task.task_id.clone(), index))
            .collect();

        let mut prerequisites = vec![Vec::new(); graph.tasks.len()];
        let mut dependents = vec![Vec::new(); graph.tasks.len()];
        let mut unresolved_prerequisites = vec![0; graph.tasks.len()];

        for (task_index, task) in graph.tasks.iter().enumerate() {
            for dependency_id in &task.depends_on {
                let dependency_index = *id_to_index
                    .get(dependency_id)
                    .expect("validated graph dependency has an index");
                prerequisites[task_index].push(dependency_index);
                dependents[dependency_index].push(task_index);
                if graph.tasks[dependency_index].status != TaskStatus::VerificationRecorded {
                    unresolved_prerequisites[task_index] += 1;
                }
            }
        }

        sort_dependents(&mut dependents, &graph.tasks);

        let mut ready = BTreeSet::new();
        for (index, task) in graph.tasks.iter().enumerate() {
            if task.status == TaskStatus::Pending && unresolved_prerequisites[index] == 0 {
                ready.insert((Reverse(task.priority), task.task_id.clone(), index));
            }
        }

        Ok(Self {
            graph,
            id_to_index,
            prerequisites,
            dependents,
            unresolved_prerequisites,
            ready,
        })
    }

    pub fn graph(&self) -> &TaskGraphV1 {
        &self.graph
    }

    pub fn revision(&self) -> u64 {
        self.graph.revision
    }

    /// Compute the same revalidation impact plan using the compiled dependency index.
    /// This only previews consequences; it does not mutate graph state or authorize work.
    pub fn plan_verification_invalidation(
        &self,
        source_task_id: &str,
        reason_code: &str,
        correction_evidence_ref: &str,
    ) -> Result<TaskRevalidationPlanV1, TaskGraphError> {
        validate_revalidation_inputs(source_task_id, reason_code, correction_evidence_ref)?;
        let source_index = *self
            .id_to_index
            .get(source_task_id)
            .ok_or_else(|| TaskGraphError::TaskNotFound(source_task_id.to_owned()))?;
        if self.graph.tasks[source_index].status != TaskStatus::VerificationRecorded {
            return Err(TaskGraphError::TaskNotVerified(source_task_id.to_owned()));
        }

        let paths = collect_dependent_paths(source_index, &self.dependents);
        Ok(build_revalidation_plan(
            &self.graph,
            source_index,
            reason_code,
            correction_evidence_ref,
            paths,
        ))
    }

    /// Return the currently-ready task IDs in deterministic display order.
    /// The set has already been derived and updated by accepted transitions.
    pub fn ready_task_ids(&self) -> Vec<&str> {
        self.ready
            .iter()
            .map(|(_, task_id, _)| task_id.as_str())
            .collect()
    }

    pub fn ready_tasks(&self) -> Vec<&TaskNodeV1> {
        self.ready
            .iter()
            .map(|(_, _, index)| &self.graph.tasks[*index])
            .collect()
    }

    pub fn blocked_by(&self, task_id: &str) -> Result<Vec<String>, TaskGraphError> {
        let index = *self
            .id_to_index
            .get(task_id)
            .ok_or_else(|| TaskGraphError::TaskNotFound(task_id.to_owned()))?;

        Ok(self.prerequisites[index]
            .iter()
            .filter(|dependency_index| {
                self.graph.tasks[**dependency_index].status != TaskStatus::VerificationRecorded
            })
            .map(|dependency_index| self.graph.tasks[*dependency_index].task_id.clone())
            .collect())
    }

    /// Apply a local transition in indexed time. When a prerequisite is verified,
    /// only its direct dependents have their unresolved count adjusted. This
    /// implementation is valid for v1's monotone status transitions; adding
    /// verification invalidation/rework must update the frontier in both directions.
    pub fn apply(&mut self, task_id: &str, action: TaskAction) -> Result<(), TaskGraphError> {
        let index = *self
            .id_to_index
            .get(task_id)
            .ok_or_else(|| TaskGraphError::TaskNotFound(task_id.to_owned()))?;
        let task = &self.graph.tasks[index];
        let old_status = task.status;
        let task_id_owned = task.task_id.clone();
        let priority = task.priority;

        let next_status = match (old_status, action) {
            (TaskStatus::Pending, TaskAction::Start)
                if self.unresolved_prerequisites[index] == 0 =>
            {
                TaskStatus::InProgress
            }
            (TaskStatus::Pending, TaskAction::Start) => {
                let dependency_ids = self.prerequisites[index]
                    .iter()
                    .filter(|dependency_index| {
                        self.graph.tasks[**dependency_index].status
                            != TaskStatus::VerificationRecorded
                    })
                    .map(|dependency_index| self.graph.tasks[*dependency_index].task_id.clone())
                    .collect();
                return Err(TaskGraphError::DependencyNotVerified {
                    task_id: task_id_owned,
                    dependency_ids,
                });
            }
            (TaskStatus::InProgress, TaskAction::ReportCompletion) => {
                TaskStatus::CompletionReported
            }
            (TaskStatus::CompletionReported, TaskAction::RecordVerification) => {
                TaskStatus::VerificationRecorded
            }
            (TaskStatus::Pending, TaskAction::Cancel) => TaskStatus::Cancelled,
            _ => {
                return Err(TaskGraphError::InvalidTransition {
                    task_id: task_id_owned,
                    action,
                    status: old_status,
                });
            }
        };

        let next_revision = self
            .graph
            .revision
            .checked_add(1)
            .ok_or(TaskGraphError::RevisionExhausted)?;

        if old_status == TaskStatus::Pending {
            self.ready
                .remove(&(Reverse(priority), task_id_owned.clone(), index));
        }

        self.graph.tasks[index].status = next_status;
        self.graph.revision = next_revision;

        if old_status != TaskStatus::VerificationRecorded
            && next_status == TaskStatus::VerificationRecorded
        {
            for dependent_index in self.dependents[index].iter().copied() {
                if self.unresolved_prerequisites[dependent_index] > 0 {
                    self.unresolved_prerequisites[dependent_index] -= 1;
                }
                let dependent = &self.graph.tasks[dependent_index];
                if dependent.status == TaskStatus::Pending
                    && self.unresolved_prerequisites[dependent_index] == 0
                {
                    self.ready.insert((
                        Reverse(dependent.priority),
                        dependent.task_id.clone(),
                        dependent_index,
                    ));
                }
            }
        }

        Ok(())
    }
}


fn validate_revalidation_inputs(
    source_task_id: &str,
    reason_code: &str,
    correction_evidence_ref: &str,
) -> Result<(), TaskGraphError> {
    if source_task_id.trim().is_empty() {
        return Err(TaskGraphError::EmptyTaskId);
    }
    if reason_code.trim().is_empty() {
        return Err(TaskGraphError::EmptyRevalidationReason);
    }
    if correction_evidence_ref.trim().is_empty() {
        return Err(TaskGraphError::EmptyCorrectionEvidenceRef);
    }
    Ok(())
}

fn sort_dependents(dependents: &mut [Vec<usize>], tasks: &[TaskNodeV1]) {
    for children in dependents {
        children.sort_by(|left, right| {
            tasks[*left]
                .task_id
                .cmp(&tasks[*right].task_id)
        });
    }
}

fn collect_dependent_paths(
    source_index: usize,
    dependents: &[Vec<usize>],
) -> Vec<(usize, Vec<usize>)> {
    let mut seen = BTreeSet::new();
    seen.insert(source_index);
    let mut queue = VecDeque::new();
    let mut result = Vec::new();

    for dependent_index in &dependents[source_index] {
        if seen.insert(*dependent_index) {
            queue.push_back((*dependent_index, vec![source_index, *dependent_index]));
        }
    }

    while let Some((task_index, path)) = queue.pop_front() {
        result.push((task_index, path.clone()));
        for dependent_index in &dependents[task_index] {
            if seen.insert(*dependent_index) {
                let mut child_path = path.clone();
                child_path.push(*dependent_index);
                queue.push_back((*dependent_index, child_path));
            }
        }
    }

    result
}

fn build_revalidation_plan(
    graph: &TaskGraphV1,
    source_index: usize,
    reason_code: &str,
    correction_evidence_ref: &str,
    paths: Vec<(usize, Vec<usize>)>,
) -> TaskRevalidationPlanV1 {
    let impacts = paths
        .into_iter()
        .map(|(task_index, path)| {
            let task = &graph.tasks[task_index];
            let kind = match task.status {
                TaskStatus::InProgress
                | TaskStatus::CompletionReported
                | TaskStatus::VerificationRecorded => {
                    RevalidationImpactKind::RequiresRevalidation
                }
                TaskStatus::Pending => RevalidationImpactKind::RemainsBlocked,
                TaskStatus::Cancelled => RevalidationImpactKind::CancelledNoAction,
            };
            TaskRevalidationImpactV1 {
                task_id: task.task_id.clone(),
                prior_status: task.status,
                kind,
                dependency_path: path
                    .into_iter()
                    .map(|index| graph.tasks[index].task_id.clone())
                    .collect(),
            }
        })
        .collect();

    TaskRevalidationPlanV1 {
        schema_version: TASK_REVALIDATION_PLAN_SCHEMA_VERSION,
        graph_id: graph.graph_id.clone(),
        expected_graph_revision: graph.revision,
        source_task_id: graph.tasks[source_index].task_id.clone(),
        source_status: graph.tasks[source_index].status,
        reason_code: reason_code.to_owned(),
        correction_evidence_ref: correction_evidence_ref.to_owned(),
        impacts,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskGraphError {
    UnsupportedSchemaVersion(u16),
    EmptyGraphId,
    ZeroRevision,
    EmptyTaskSet,
    EmptyTaskId,
    EmptyTaskTitle(String),
    DuplicateTaskId(String),
    MissingDependency {
        task_id: String,
        dependency_id: String,
    },
    ActiveTaskHasUnverifiedPrerequisites {
        task_id: String,
        dependency_ids: Vec<String>,
    },
    TaskNotVerified(String),
    EmptyRevalidationReason,
    EmptyCorrectionEvidenceRef,
    SelfDependency(String),
    DuplicateDependency {
        task_id: String,
        dependency_id: String,
    },
    CycleDetected,
    TaskNotFound(String),
    DependencyNotVerified {
        task_id: String,
        dependency_ids: Vec<String>,
    },
    InvalidTransition {
        task_id: String,
        action: TaskAction,
        status: TaskStatus,
    },
    RevisionExhausted,
}

impl fmt::Display for TaskGraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(version) => {
                write!(formatter, "unsupported task-graph schema version {version}")
            }
            Self::EmptyGraphId => formatter.write_str("task-graph ID must not be empty"),
            Self::ZeroRevision => formatter.write_str("task-graph revision must be positive"),
            Self::EmptyTaskSet => formatter.write_str("task graph must contain at least one task"),
            Self::EmptyTaskId => formatter.write_str("task ID must not be empty"),
            Self::EmptyTaskTitle(task_id) => {
                write!(formatter, "task {task_id} has an empty title")
            }
            Self::DuplicateTaskId(task_id) => write!(formatter, "duplicate task ID {task_id}"),
            Self::MissingDependency {
                task_id,
                dependency_id,
            } => write!(
                formatter,
                "task {task_id} depends on missing task {dependency_id}"
            ),
            Self::ActiveTaskHasUnverifiedPrerequisites {
                task_id,
                dependency_ids,
            } => write!(
                formatter,
                "active task {task_id} has unverified prerequisites: {}",
                dependency_ids.join(", ")
            ),
            Self::TaskNotVerified(task_id) => {
                write!(formatter, "task {task_id} is not currently verified")
            }
            Self::EmptyRevalidationReason => {
                formatter.write_str("revalidation reason code must not be empty")
            }
            Self::EmptyCorrectionEvidenceRef => {
                formatter.write_str("correction evidence reference must not be empty")
            }
            Self::SelfDependency(task_id) => {
                write!(formatter, "task {task_id} cannot depend on itself")
            }
            Self::DuplicateDependency {
                task_id,
                dependency_id,
            } => write!(
                formatter,
                "task {task_id} lists dependency {dependency_id} more than once"
            ),
            Self::CycleDetected => formatter.write_str("task graph contains a dependency cycle"),
            Self::TaskNotFound(task_id) => write!(formatter, "task {task_id} was not found"),
            Self::DependencyNotVerified {
                task_id,
                dependency_ids,
            } => write!(
                formatter,
                "task {task_id} has prerequisites without recorded verification: {}",
                dependency_ids.join(", ")
            ),
            Self::InvalidTransition {
                task_id,
                action,
                status,
            } => write!(
                formatter,
                "action {action:?} is invalid for task {task_id} in state {status:?}"
            ),
            Self::RevisionExhausted => formatter.write_str("task-graph revision counter exhausted"),
        }
    }
}

impl std::error::Error for TaskGraphError {}

#[cfg(test)]
mod tests {
    use super::{
        TASK_GRAPH_SCHEMA_VERSION, TaskAction, TaskGraphError, TaskGraphV1, TaskNodeV1, TaskStatus,
    };

    fn node(id: &str, dependencies: &[&str], priority: u8, status: TaskStatus) -> TaskNodeV1 {
        TaskNodeV1 {
            task_id: id.to_owned(),
            title: format!("Task {id}"),
            depends_on: dependencies
                .iter()
                .map(|value| (*value).to_owned())
                .collect(),
            priority,
            status,
        }
    }

    fn graph(tasks: Vec<TaskNodeV1>) -> TaskGraphV1 {
        TaskGraphV1 {
            schema_version: TASK_GRAPH_SCHEMA_VERSION,
            graph_id: "job-graph-test".to_owned(),
            revision: 1,
            tasks,
        }
    }

    #[test]
    fn frozen_graph_fixture_deserializes_and_validates() {
        let fixture: TaskGraphV1 =
            serde_json::from_str(include_str!("../fixtures/task-graph-v1.json"))
                .expect("frozen task graph fixture matches schema");
        assert_eq!(fixture.validate(), Ok(()));
        assert_eq!(
            fixture.topological_order().unwrap(),
            vec![
                "access-approval",
                "inspection",
                "parts-procurement",
                "functional-test",
                "handover"
            ]
        );
    }

    #[test]
    fn independent_tasks_can_be_ready_in_parallel_with_deterministic_priority() {
        let task_graph = graph(vec![
            node("low", &[], 1, TaskStatus::Pending),
            node("high-b", &[], 8, TaskStatus::Pending),
            node("high-a", &[], 8, TaskStatus::Pending),
        ]);
        let ready: Vec<&str> = task_graph
            .ready_tasks()
            .unwrap()
            .iter()
            .map(|task| task.task_id.as_str())
            .collect();
        assert_eq!(ready, vec!["high-a", "high-b", "low"]);
    }

    #[test]
    fn completion_report_does_not_unlock_dependents_until_verification_is_recorded() {
        let mut task_graph = graph(vec![
            node("inspect", &[], 10, TaskStatus::Pending),
            node("test", &["inspect"], 20, TaskStatus::Pending),
        ]);
        task_graph.apply("inspect", TaskAction::Start).unwrap();
        task_graph
            .apply("inspect", TaskAction::ReportCompletion)
            .unwrap();

        assert_eq!(
            task_graph.blocked_by("test").unwrap(),
            vec!["inspect".to_owned()]
        );
        assert_eq!(task_graph.ready_tasks().unwrap().len(), 0);
        assert_eq!(
            task_graph.apply("test", TaskAction::Start),
            Err(TaskGraphError::DependencyNotVerified {
                task_id: "test".to_owned(),
                dependency_ids: vec!["inspect".to_owned()],
            })
        );

        task_graph
            .apply("inspect", TaskAction::RecordVerification)
            .unwrap();
        assert_eq!(
            task_graph
                .ready_tasks()
                .unwrap()
                .iter()
                .map(|task| task.task_id.as_str())
                .collect::<Vec<_>>(),
            vec!["test"]
        );
    }

    #[test]
    fn invalid_transition_does_not_mutate_task_or_revision() {
        let mut task_graph = graph(vec![node("inspect", &[], 1, TaskStatus::Pending)]);
        let before = task_graph.clone();
        assert_eq!(
            task_graph.apply("inspect", TaskAction::RecordVerification),
            Err(TaskGraphError::InvalidTransition {
                task_id: "inspect".to_owned(),
                action: TaskAction::RecordVerification,
                status: TaskStatus::Pending,
            })
        );
        assert_eq!(task_graph, before);
    }

    #[test]
    fn graph_rejects_missing_duplicate_self_and_cyclic_dependencies() {
        assert!(matches!(
            graph(vec![node("a", &["missing"], 0, TaskStatus::Pending)]).validate(),
            Err(TaskGraphError::MissingDependency { .. })
        ));
        assert!(matches!(
            graph(vec![node("a", &["a"], 0, TaskStatus::Pending)]).validate(),
            Err(TaskGraphError::SelfDependency(_))
        ));
        assert!(matches!(
            graph(vec![
                node("a", &["b"], 0, TaskStatus::Pending),
                node("b", &["a"], 0, TaskStatus::Pending),
            ])
            .validate(),
            Err(TaskGraphError::CycleDetected)
        ));
        assert!(matches!(
            graph(vec![
                node("a", &["b", "b"], 0, TaskStatus::Pending),
                node("b", &[], 0, TaskStatus::Pending)
            ])
            .validate(),
            Err(TaskGraphError::DuplicateDependency { .. })
        ));
    }

    #[test]
    fn cancelled_prerequisite_does_not_unlock_downstream_task() {
        let mut task_graph = graph(vec![
            node("permit", &[], 1, TaskStatus::Pending),
            node("work", &["permit"], 1, TaskStatus::Pending),
        ]);
        task_graph.apply("permit", TaskAction::Cancel).unwrap();
        assert_eq!(
            task_graph.blocked_by("work").unwrap(),
            vec!["permit".to_owned()]
        );
        assert!(task_graph.ready_tasks().unwrap().is_empty());
    }

    #[test]
    fn graph_revision_advances_once_per_accepted_transition() {
        let mut task_graph = graph(vec![node("inspect", &[], 1, TaskStatus::Pending)]);
        task_graph.apply("inspect", TaskAction::Start).unwrap();
        assert_eq!(task_graph.revision, 2);
        task_graph
            .apply("inspect", TaskAction::ReportCompletion)
            .unwrap();
        assert_eq!(task_graph.revision, 3);
    }
    #[test]
    fn revalidation_plan_fixture_matches_reference_and_compiled_analysis_without_mutation() {
        let tasks = vec![
            node("access", &[], 10, TaskStatus::VerificationRecorded),
            node("cancelled-branch", &["access"], 1, TaskStatus::Cancelled),
            node("inspection", &["access"], 8, TaskStatus::VerificationRecorded),
            node("new-work", &["access"], 6, TaskStatus::Pending),
            node("parts", &["access"], 7, TaskStatus::VerificationRecorded),
            node("after-cancel", &["cancelled-branch"], 1, TaskStatus::Pending),
            node(
                "functional-test",
                &["inspection", "parts"],
                9,
                TaskStatus::CompletionReported,
            ),
            node("handover", &["functional-test"], 5, TaskStatus::Pending),
        ];
        let mut graph = graph(tasks.clone());
        graph.graph_id = "revalidation-demo-001".to_owned();
        graph.revision = 12;
        let mut reordered = graph(tasks.into_iter().rev().collect());
        reordered.graph_id = "revalidation-demo-001".to_owned();
        reordered.revision = 12;
        let before = graph.clone();

        let reference_plan = graph
            .plan_verification_invalidation(
                "access",
                "evidence_correction",
                "urn:demo:evidence:access-correction",
            )
            .unwrap();
        let compiled = super::CompiledTaskGraph::compile(graph.clone()).unwrap();
        let compiled_plan = compiled
            .plan_verification_invalidation(
                "access",
                "evidence_correction",
                "urn:demo:evidence:access-correction",
            )
            .unwrap();
        let expected: super::TaskRevalidationPlanV1 =
            serde_json::from_str(include_str!("../fixtures/task-revalidation-plan-v1.json"))
                .expect("frozen revalidation plan fixture must deserialize");

        assert_eq!(reference_plan, expected);
        assert_eq!(compiled_plan, expected);
        assert_eq!(
            reordered
                .plan_verification_invalidation(
                    "access",
                    "evidence_correction",
                    "urn:demo:evidence:access-correction",
                )
                .unwrap(),
            expected
        );
        assert_eq!(graph, before);
        assert_eq!(compiled.graph(), &before);
    }

    #[test]
    fn revalidation_plan_rejects_unverified_source_and_missing_correction_context() {
        let graph = graph(vec![node("access", &[], 1, TaskStatus::Pending)]);
        assert_eq!(
            graph.plan_verification_invalidation(
                "access",
                "evidence_correction",
                "urn:demo:evidence:correction",
            ),
            Err(TaskGraphError::TaskNotVerified("access".to_owned()))
        );
        assert_eq!(
            graph.plan_verification_invalidation(
                "access",
                " ",
                "urn:demo:evidence:correction",
            ),
            Err(TaskGraphError::EmptyRevalidationReason)
        );
        assert_eq!(
            graph.plan_verification_invalidation("access", "evidence_correction", " "),
            Err(TaskGraphError::EmptyCorrectionEvidenceRef)
        );
    }

    #[test]
    fn compiled_frontier_matches_reference_reducer_through_a_multi_branch_job() {
        let graph: TaskGraphV1 =
            serde_json::from_str(include_str!("../fixtures/task-graph-v1.json"))
                .expect("task graph fixture must deserialize");
        let mut reference = graph.clone();
        let mut compiled = super::CompiledTaskGraph::compile(graph).unwrap();

        let check_parity = |reference: &TaskGraphV1, compiled: &super::CompiledTaskGraph| {
            assert_eq!(
                reference
                    .ready_tasks()
                    .unwrap()
                    .iter()
                    .map(|task| task.task_id.as_str())
                    .collect::<Vec<_>>(),
                compiled.ready_task_ids()
            );
            assert_eq!(reference, compiled.graph());
            assert_eq!(reference.revision, compiled.revision());
        };

        check_parity(&reference, &compiled);
        for (task_id, action) in [
            ("inspection", TaskAction::Start),
            ("inspection", TaskAction::ReportCompletion),
            ("inspection", TaskAction::RecordVerification),
            ("parts-procurement", TaskAction::Start),
            ("parts-procurement", TaskAction::ReportCompletion),
            ("parts-procurement", TaskAction::RecordVerification),
            ("functional-test", TaskAction::Start),
        ] {
            assert_eq!(
                reference.apply(task_id, action),
                compiled.apply(task_id, action)
            );
            check_parity(&reference, &compiled);
        }
        assert!(compiled.ready_task_ids().is_empty());
    }

    #[test]
    fn compiled_frontier_rejects_invalid_transition_without_mutation() {
        let graph: TaskGraphV1 =
            serde_json::from_str(include_str!("../fixtures/task-graph-v1.json"))
                .expect("task graph fixture must deserialize");
        let mut compiled = super::CompiledTaskGraph::compile(graph).unwrap();
        let before = compiled.clone();
        assert!(matches!(
            compiled.apply("inspection", TaskAction::RecordVerification),
            Err(TaskGraphError::InvalidTransition { .. })
        ));
        assert_eq!(compiled, before);
    }

    #[test]
    fn graph_rejects_active_snapshot_with_unverified_prerequisite() {
        let invalid = graph(vec![
            node("permit", &[], 1, TaskStatus::Pending),
            node("work", &["permit"], 1, TaskStatus::InProgress),
        ]);
        assert!(matches!(
            invalid.validate(),
            Err(TaskGraphError::ActiveTaskHasUnverifiedPrerequisites {
                task_id,
                dependency_ids,
            }) if task_id == "work" && dependency_ids == vec!["permit".to_owned()]
        ));
    }

    #[test]
    fn compiled_frontier_matches_reference_across_generated_parallel_and_chained_tasks() {
        let mut tasks = Vec::new();
        for index in 0..64 {
            tasks.push(node(
                &format!("task-{index:03}"),
                &[],
                ((index * 19) % 101) as u8,
                TaskStatus::Pending,
            ));
        }
        for index in 64..128 {
            let dependency = format!("task-{:03}", index % 64);
            tasks.push(node(
                &format!("task-{index:03}"),
                &[dependency.as_str()],
                ((index * 19) % 101) as u8,
                TaskStatus::Pending,
            ));
        }
        for index in 128..256 {
            let access_dependency = format!("task-{:03}", index % 64);
            let previous_dependency = format!("task-{:03}", index - 1);
            let dependencies = [access_dependency.as_str(), previous_dependency.as_str()];
            tasks.push(node(
                &format!("task-{index:03}"),
                &dependencies,
                ((index * 19) % 101) as u8,
                TaskStatus::Pending,
            ));
        }

        let mut reference = graph(tasks);
        let mut compiled = super::CompiledTaskGraph::compile(reference.clone()).unwrap();
        let assert_frontier_parity =
            |reference: &TaskGraphV1, compiled: &super::CompiledTaskGraph| {
                assert_eq!(
                    reference
                        .ready_tasks()
                        .unwrap()
                        .iter()
                        .map(|task| task.task_id.as_str())
                        .collect::<Vec<_>>(),
                    compiled.ready_task_ids()
                );
                assert_eq!(reference, compiled.graph());
                assert_eq!(reference.revision, compiled.revision());
            };

        assert_frontier_parity(&reference, &compiled);
        for index in 0..256 {
            let task_id = format!("task-{index:03}");
            for action in [
                TaskAction::Start,
                TaskAction::ReportCompletion,
                TaskAction::RecordVerification,
            ] {
                assert_eq!(
                    reference.apply(&task_id, action),
                    compiled.apply(&task_id, action),
                    "transition {action:?} for {task_id}"
                );
                assert_frontier_parity(&reference, &compiled);
            }
        }
    }

    #[test]
    fn compiled_revision_overflow_does_not_mutate_status_or_frontier() {
        let mut graph = graph(vec![node("inspect", &[], 1, TaskStatus::Pending)]);
        graph.revision = u64::MAX;
        let mut compiled = super::CompiledTaskGraph::compile(graph).unwrap();
        let before = compiled.clone();
        assert_eq!(
            compiled.apply("inspect", TaskAction::Start),
            Err(TaskGraphError::RevisionExhausted)
        );
        assert_eq!(compiled, before);
    }

    #[test]
    fn compiled_frontier_is_independent_of_input_task_vector_order() {
        let tasks = vec![
            node("z", &["a"], 2, TaskStatus::Pending),
            node("b", &[], 9, TaskStatus::Pending),
            node("a", &[], 3, TaskStatus::VerificationRecorded),
            node("y", &["b"], 1, TaskStatus::Pending),
        ];
        let first = graph(tasks.clone());
        let second = graph(tasks.into_iter().rev().collect());
        let first_compiled = super::CompiledTaskGraph::compile(first).unwrap();
        let second_compiled = super::CompiledTaskGraph::compile(second).unwrap();
        assert_eq!(
            first_compiled.ready_task_ids(),
            second_compiled.ready_task_ids()
        );
        assert_eq!(first_compiled.ready_task_ids(), vec!["b", "z"]);
    }
}
