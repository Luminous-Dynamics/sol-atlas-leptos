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
use std::collections::{BTreeMap, BTreeSet};
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

    /// Apply a local projection transition only if the caller's snapshot is current.
    ///
    /// The expected revision is an optimistic-concurrency precondition. A stale
    /// command is rejected before any task or graph revision is mutated.
    ///
    /// This mutates only the in-memory graph. The canonical service must still
    /// enforce an equivalent revision precondition, authorization and evidence
    /// rules before accepting any real work-order transition.
    pub fn apply(
        &mut self,
        expected_revision: u64,
        task_id: &str,
        action: TaskAction,
    ) -> Result<(), TaskGraphError> {
        if expected_revision != self.revision {
            return Err(TaskGraphError::RevisionConflict {
                expected_revision,
                actual_revision: self.revision,
            });
        }
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
    SelfDependency(String),
    DuplicateDependency {
        task_id: String,
        dependency_id: String,
    },
    CycleDetected,
    TaskNotFound(String),
    RevisionConflict {
        expected_revision: u64,
        actual_revision: u64,
    },
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
            Self::RevisionConflict {
                expected_revision,
                actual_revision,
            } => write!(
                formatter,
                "stale task-graph revision: expected {expected_revision}, current revision is {actual_revision}"
            ),
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
        task_graph.apply(1, "inspect", TaskAction::Start).unwrap();
        task_graph
            .apply(2, "inspect", TaskAction::ReportCompletion)
            .unwrap();

        assert_eq!(
            task_graph.blocked_by("test").unwrap(),
            vec!["inspect".to_owned()]
        );
        assert_eq!(task_graph.ready_tasks().unwrap().len(), 0);
        assert_eq!(
            task_graph.apply(3, "test", TaskAction::Start),
            Err(TaskGraphError::DependencyNotVerified {
                task_id: "test".to_owned(),
                dependency_ids: vec!["inspect".to_owned()],
            })
        );

        task_graph
            .apply(3, "inspect", TaskAction::RecordVerification)
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
            task_graph.apply(1, "inspect", TaskAction::RecordVerification),
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
        task_graph.apply(1, "permit", TaskAction::Cancel).unwrap();
        assert_eq!(
            task_graph.blocked_by("work").unwrap(),
            vec!["permit".to_owned()]
        );
        assert!(task_graph.ready_tasks().unwrap().is_empty());
    }

    #[test]
    fn stale_revision_action_is_rejected_without_mutating_graph() {
        let mut task_graph = graph(vec![node("inspect", &[], 1, TaskStatus::Pending)]);
        let stale_revision = task_graph.revision;

        task_graph
            .apply(stale_revision, "inspect", TaskAction::Start)
            .unwrap();
        let after_first_writer = task_graph.clone();

        assert_eq!(
            task_graph.apply(stale_revision, "inspect", TaskAction::ReportCompletion),
            Err(TaskGraphError::RevisionConflict {
                expected_revision: stale_revision,
                actual_revision: 2,
            })
        );
        assert_eq!(task_graph, after_first_writer);
    }

    #[test]
    fn graph_revision_advances_once_per_accepted_transition() {
        let mut task_graph = graph(vec![node("inspect", &[], 1, TaskStatus::Pending)]);
        task_graph.apply(1, "inspect", TaskAction::Start).unwrap();
        assert_eq!(task_graph.revision, 2);
        task_graph
            .apply(2, "inspect", TaskAction::ReportCompletion)
            .unwrap();
        assert_eq!(task_graph.revision, 3);
    }
}
