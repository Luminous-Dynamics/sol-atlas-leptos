// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Cross-trade field-work UX prototype.
//!
//! All work orders, assets, readings, safety checks, and synchronization states
//! are synthetic. No live services, asset controllers, or safety systems are
//! connected. The reducer demonstrates workflow boundaries only; it is not a
//! validated operational safety control.

use leptos::prelude::*;
use leptos_router::components::A;
use sol_atlas_core::field_work::{
    AssetRefV1, AssetRelationship, EvidenceKind, EvidenceRefV1, FieldJobPhase,
    FieldJobProjectionV1, ProjectionFreshness, ProjectionSourceV1, SafetyGateStatus,
    SafetyGateSummaryV1, TradeKind, TransportState, ViewportIntentKind,
};
use sol_atlas_core::task_graph::{
    CompiledTaskGraph, RevalidationImpactKind, TASK_GRAPH_SCHEMA_VERSION, TaskAction,
    TaskGraphError, TaskGraphV1, TaskNodeV1, TaskRevalidationImpactV2, TaskStatus,
};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Trade {
    Electrical,
    Plumbing,
    Mechanical,
    Hvac,
    Construction,
}

impl Trade {
    fn projection_trade(self) -> TradeKind {
        match self {
            Self::Electrical => TradeKind::Electrical,
            Self::Plumbing => TradeKind::Plumbing,
            Self::Mechanical => TradeKind::MechanicalRepair,
            Self::Hvac => TradeKind::HvacRefrigeration,
            Self::Construction => TradeKind::Construction,
        }
    }

    fn short_label(self) -> &'static str {
        match self {
            Self::Electrical => "ELECTRICAL",
            Self::Plumbing => "PLUMBING",
            Self::Mechanical => "MECHANICAL",
            Self::Hvac => "HVAC",
            Self::Construction => "CONSTRUCTION",
        }
    }

    fn job_id(self) -> &'static str {
        match self {
            Self::Electrical => "DEMO-E-0248",
            Self::Plumbing => "DEMO-P-0249",
            Self::Mechanical => "DEMO-M-0250",
            Self::Hvac => "DEMO-H-0251",
            Self::Construction => "DEMO-C-0252",
        }
    }

    fn job_title(self) -> &'static str {
        match self {
            Self::Electrical => "Intermittent protective-device trip",
            Self::Plumbing => "Low water pressure at point of use",
            Self::Mechanical => "Vehicle fails to start consistently",
            Self::Hvac => "Cooling output below expected range",
            Self::Construction => "Inspect damaged stair handrail",
        }
    }

    fn asset_label(self) -> &'static str {
        match self {
            Self::Electrical => "Distribution board · sample asset",
            Self::Plumbing => "Internal supply · sample asset",
            Self::Mechanical => "Vehicle / powertrain · sample asset",
            Self::Hvac => "Air-conditioning plant · sample asset",
            Self::Construction => "Stair and handrail · sample asset",
        }
    }

    fn evidence_hint(self) -> &'static str {
        match self {
            Self::Electrical => {
                "Record instrument, circuit reference, units, test conditions, and applicable procedure revision."
            }
            Self::Plumbing => {
                "Record measurement point, pressure or flow units, fixture state, and instrument identity."
            }
            Self::Mechanical => {
                "Record symptoms, diagnostic codes, test conditions, part identity, and source of specifications."
            }
            Self::Hvac => {
                "Record operating mode, ambient conditions, instrument, units, and equipment operating state."
            }
            Self::Construction => {
                "Record location, dimensions, material or component identity, photographs, and inspection criteria."
            }
        }
    }

    fn isolation_label(self) -> &'static str {
        match self {
            Self::Electrical => "Electrical energy-control check (demo)",
            Self::Plumbing => "Water / pressure-control check (demo)",
            Self::Mechanical => "Mechanical energy-control check (demo)",
            Self::Hvac => "Electrical / pressure-control check (demo)",
            Self::Construction => "Site and equipment-control check (demo)",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkPhase {
    Assigned,
    Inspecting,
    Diagnosed,
    Planned,
    InProgress,
    VerificationDue,
    Verified,
    HandedOver,
}

impl WorkPhase {
    fn label(self) -> &'static str {
        match self {
            Self::Assigned => "Ready for site assessment",
            Self::Inspecting => "Inspection in progress",
            Self::Diagnosed => "Finding recorded · diagnosis unverified",
            Self::Planned => "Plan prepared · work not started",
            Self::InProgress => "Work reported in progress",
            Self::VerificationDue => "Work reported complete · verification due",
            Self::Verified => "Verification recorded · handover pending",
            Self::HandedOver => "Handover recorded in this demo",
        }
    }

    fn step(self) -> u8 {
        match self {
            Self::Assigned | Self::Inspecting => 0,
            Self::Diagnosed => 1,
            Self::Planned => 2,
            Self::InProgress => 3,
            Self::VerificationDue => 4,
            Self::Verified => 5,
            Self::HandedOver => 6,
        }
    }

    fn projection_phase(self) -> FieldJobPhase {
        match self {
            Self::Assigned => FieldJobPhase::Assigned,
            Self::Inspecting => FieldJobPhase::Assessing,
            Self::Diagnosed => FieldJobPhase::FindingRecorded,
            Self::Planned => FieldJobPhase::PlanApproved,
            Self::InProgress => FieldJobPhase::WorkReportedInProgress,
            Self::VerificationDue => FieldJobPhase::CompletionReported,
            Self::Verified => FieldJobPhase::VerificationRecorded,
            Self::HandedOver => FieldJobPhase::HandoverRecorded,
        }
    }

    fn next_action(self) -> WorkAction {
        match self {
            Self::Assigned => WorkAction::BeginAssessment,
            Self::Inspecting => WorkAction::RecordFinding,
            Self::Diagnosed => WorkAction::ApprovePlan,
            Self::Planned => WorkAction::StartWork,
            Self::InProgress => WorkAction::ReportWorkComplete,
            Self::VerificationDue => WorkAction::RecordVerification,
            Self::Verified => WorkAction::Handover,
            Self::HandedOver => WorkAction::Restart,
        }
    }

    fn next_action_label(self) -> &'static str {
        match self {
            Self::Assigned => "Begin site assessment",
            Self::Inspecting => "Record an observed finding",
            Self::Diagnosed => "Prepare and approve work plan",
            Self::Planned => "Start work (demo gate)",
            Self::InProgress => "Report work complete",
            Self::VerificationDue => "Record verification result",
            Self::Verified => "Record handover",
            Self::HandedOver => "Reset demonstration",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkAction {
    BeginAssessment,
    RecordFinding,
    ApprovePlan,
    StartWork,
    ReportWorkComplete,
    RecordVerification,
    Handover,
    Restart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DemoProgress {
    phase: WorkPhase,
    hazards_reviewed: bool,
    isolation_verified: bool,
    evidence_count: usize,
}

impl Default for DemoProgress {
    fn default() -> Self {
        Self {
            phase: WorkPhase::Assigned,
            hazards_reviewed: false,
            isolation_verified: false,
            evidence_count: 0,
        }
    }
}

fn switch_demo_session(
    current_trade: Trade,
    next_trade: Trade,
    progress: DemoProgress,
) -> (Trade, DemoProgress) {
    if current_trade == next_trade {
        (current_trade, progress)
    } else {
        (next_trade, DemoProgress::default())
    }
}

/// Pure demo state machine. Missing safety prerequisites fail closed.
///
/// In production, these booleans must not be treated as proof of physical
/// isolation, competence, permit approval, or compliance. They only exercise
/// the interaction's blocked/ready states.
fn transition(
    phase: WorkPhase,
    action: WorkAction,
    hazards_reviewed: bool,
    isolation_verified: bool,
) -> WorkPhase {
    use WorkAction as A;
    use WorkPhase as P;

    match (phase, action) {
        (P::Assigned, A::BeginAssessment) => P::Inspecting,
        (P::Inspecting, A::RecordFinding) => P::Diagnosed,
        (P::Diagnosed, A::ApprovePlan) => P::Planned,
        (P::Planned, A::StartWork) if hazards_reviewed && isolation_verified => P::InProgress,
        (P::InProgress, A::ReportWorkComplete) => P::VerificationDue,
        (P::VerificationDue, A::RecordVerification) => P::Verified,
        (P::Verified, A::Handover) => P::HandedOver,
        (_, A::Restart) => P::Assigned,
        _ => phase,
    }
}

/// Build a projection for this UI-only demonstration. Never use demo data as
/// canonical work state or proof that a physical prerequisite was satisfied.
fn demo_projection(
    trade: Trade,
    phase: WorkPhase,
    prerequisites_marked_in_demo: bool,
    evidence_count: usize,
) -> FieldJobProjectionV1 {
    let job_id = format!("demo:{}", trade.job_id());
    let asset_id = format!("{}:asset", trade.job_id());

    let mut evidence = Vec::new();
    if evidence_count >= 1 {
        evidence.push(EvidenceRefV1 {
            evidence_id: format!("{}:reported-finding", trade.job_id()),
            kind: EvidenceKind::Reported,
            source_ref: "urn:luminous:demo:user-report".into(),
            digest_sha256: "f".repeat(64),
            captured_at_unix_ms: None,
            unit: None,
            instrument_ref: None,
            procedure_revision: None,
            model_ref: None,
        });
    }
    if evidence_count >= 2 {
        evidence.push(EvidenceRefV1 {
            evidence_id: format!("{}:verification-demo", trade.job_id()),
            kind: EvidenceKind::Verified,
            source_ref: "urn:luminous:demo:verification-note".into(),
            digest_sha256: "e".repeat(64),
            captured_at_unix_ms: None,
            unit: None,
            instrument_ref: None,
            procedure_revision: Some(1),
            model_ref: None,
        });
    }

    let missing_requirements = if prerequisites_marked_in_demo {
        Vec::new()
    } else {
        vec![
            "Site-hazard check has not been marked in the demo".to_string(),
            "Energy/site-control check has not been marked in the demo".to_string(),
        ]
    };

    FieldJobProjectionV1 {
        schema_version: 1,
        job_id,
        job_revision: 1,
        trade: trade.projection_trade(),
        title: trade.job_title().to_string(),
        phase: phase.projection_phase(),
        transport: TransportState::LocalOnly,
        // Demo source values are synthetic, so do not call them fresh.
        freshness: ProjectionFreshness::Unknown,
        source: ProjectionSourceV1 {
            authority_id: "synthetic-demo-only".into(),
            source_revision: 1,
            source_digest_sha256: "d".repeat(64),
        },
        assets: vec![AssetRefV1 {
            asset_id,
            asset_revision: 1,
            label: trade.asset_label().to_string(),
            relationship: AssetRelationship::Primary,
        }],
        evidence,
        safety_gate: SafetyGateSummaryV1 {
            status: if prerequisites_marked_in_demo {
                SafetyGateStatus::AwaitingQualifiedReview
            } else {
                SafetyGateStatus::Blocked
            },
            policy_ref: Some("synthetic-demo-policy".into()),
            policy_revision: Some(1),
            missing_requirements,
        },
        offered_viewport_intents: vec![
            ViewportIntentKind::SelectAsset,
            ViewportIntentKind::FocusAsset,
            ViewportIntentKind::SetTimelineCursor,
        ],
    }
}

fn demo_compiled_task_graph(trade: Trade) -> CompiledTaskGraph {
    let graph = TaskGraphV1 {
        schema_version: TASK_GRAPH_SCHEMA_VERSION,
        graph_id: format!("{}:task-graph-demo", trade.job_id()),
        revision: 1,
        tasks: vec![
            TaskNodeV1 {
                task_id: "access".to_owned(),
                title: format!("Confirm access and work context · {}", trade.short_label()),
                depends_on: Vec::new(),
                priority: 100,
                status: TaskStatus::VerificationRecorded,
            },
            TaskNodeV1 {
                task_id: "inspection".to_owned(),
                title: format!("Inspect and record findings · {}", trade.short_label()),
                depends_on: vec!["access".to_owned()],
                priority: 80,
                status: TaskStatus::Pending,
            },
            TaskNodeV1 {
                task_id: "parts".to_owned(),
                title: "Confirm parts and materials availability".to_owned(),
                depends_on: vec!["access".to_owned()],
                priority: 90,
                status: TaskStatus::Pending,
            },
            TaskNodeV1 {
                task_id: "functional-test".to_owned(),
                title: "Record post-work functional test".to_owned(),
                depends_on: vec!["inspection".to_owned(), "parts".to_owned()],
                priority: 100,
                status: TaskStatus::Pending,
            },
            TaskNodeV1 {
                task_id: "handover".to_owned(),
                title: "Prepare handover and outcome record".to_owned(),
                depends_on: vec!["functional-test".to_owned()],
                priority: 70,
                status: TaskStatus::Pending,
            },
        ],
    };

    CompiledTaskGraph::compile(graph).expect("the synthetic task graph is valid")
}

#[derive(Clone)]
struct TaskGraphRow {
    task_id: String,
    title: String,
    status_label: &'static str,
    status_class: &'static str,
    dependency_text: String,
    action: Option<TaskAction>,
    action_label: &'static str,
}

fn task_action_for(status: TaskStatus, ready: bool) -> Option<TaskAction> {
    match status {
        TaskStatus::Pending if ready => Some(TaskAction::Start),
        TaskStatus::InProgress => Some(TaskAction::ReportCompletion),
        TaskStatus::CompletionReported => Some(TaskAction::RecordVerification),
        TaskStatus::Pending | TaskStatus::VerificationRecorded | TaskStatus::Cancelled => None,
    }
}

fn task_action_label(action: Option<TaskAction>, status: TaskStatus) -> &'static str {
    match action {
        Some(TaskAction::Start) => "Start task",
        Some(TaskAction::ReportCompletion) => "Report completion",
        Some(TaskAction::RecordVerification) => "Record verification",
        Some(TaskAction::Cancel) => "Cancel task",
        None if status == TaskStatus::Pending => "Await prerequisites",
        None if status == TaskStatus::VerificationRecorded => "Verified in demo",
        None => "Cancelled",
    }
}

fn task_status_presentation(status: TaskStatus) -> (&'static str, &'static str) {
    match status {
        TaskStatus::Pending => ("Pending", "pending"),
        TaskStatus::InProgress => ("In progress", "active"),
        TaskStatus::CompletionReported => ("Completion reported", "reported"),
        TaskStatus::VerificationRecorded => ("Verified in demo", "verified"),
        TaskStatus::Cancelled => ("Cancelled", "cancelled"),
    }
}

fn task_graph_rows(graph: &TaskGraphV1, ready_ids: &BTreeSet<String>) -> Vec<TaskGraphRow> {
    graph
        .tasks
        .iter()
        .map(|task| {
            let (status_label, status_class) = task_status_presentation(task.status);
            let dependency_titles: Vec<String> = task
                .depends_on
                .iter()
                .filter_map(|dependency_id| {
                    graph
                        .tasks
                        .iter()
                        .find(|dependency| dependency.task_id == *dependency_id)
                        .map(|dependency| dependency.title.clone())
                })
                .collect();
            let dependency_text = if dependency_titles.is_empty() {
                "No prerequisites".to_owned()
            } else {
                format!("Depends on: {}", dependency_titles.join(", "))
            };
            let action = task_action_for(task.status, ready_ids.contains(&task.task_id));
            let action_label = task_action_label(action, task.status);

            TaskGraphRow {
                task_id: task.task_id.clone(),
                title: task.title.clone(),
                status_label,
                status_class,
                dependency_text,
                action,
                action_label,
            }
        })
        .collect()
}

fn impact_kind_label(kind: RevalidationImpactKind) -> &'static str {
    match kind {
        RevalidationImpactKind::RequiresRevalidation => "Requires revalidation",
        RevalidationImpactKind::RemainsBlocked => "Remains blocked",
        RevalidationImpactKind::CancelledNoAction => "Cancelled · unchanged",
    }
}

fn task_status_label(status: TaskStatus) -> &'static str {
    task_status_presentation(status).0
}

#[component]
pub fn TradeWorkbench() -> impl IntoView {
    let selected_trade = RwSignal::new(Trade::Electrical);
    let simulated_offline = RwSignal::new(true);
    let phase = RwSignal::new(WorkPhase::Assigned);
    let hazards_reviewed = RwSignal::new(false);
    let isolation_verified = RwSignal::new(false);
    let evidence_count = RwSignal::new(0usize);
    let task_graph = RwSignal::new(demo_compiled_task_graph(Trade::Electrical));
    let impact_items = RwSignal::new(Vec::<TaskRevalidationImpactV2>::new());
    let impact_summary = RwSignal::new(
        "No correction-impact preview generated. The task graph is synthetic and read-only with respect to real work.".to_string(),
    );
    let status_message = RwSignal::new(
        "Synthetic job loaded. No real assignment, asset, or safety system is connected."
            .to_string(),
    );

    // Trade selection changes job identity, so reset state rather than carry
    // evidence/progress across two different synthetic jobs.
    let select_trade = move |trade: Trade| {
        let current_trade = selected_trade.get();
        let progress = DemoProgress {
            phase: phase.get(),
            hazards_reviewed: hazards_reviewed.get(),
            isolation_verified: isolation_verified.get(),
            evidence_count: evidence_count.get(),
        };
        let (next_trade, next_progress) = switch_demo_session(current_trade, trade, progress);
        if next_trade != current_trade {
            selected_trade.set(next_trade);
            phase.set(next_progress.phase);
            hazards_reviewed.set(next_progress.hazards_reviewed);
            isolation_verified.set(next_progress.isolation_verified);
            evidence_count.set(next_progress.evidence_count);
            task_graph.set(demo_compiled_task_graph(next_trade));
            impact_items.set(Vec::new());
            impact_summary
                .set("No correction-impact preview generated for this synthetic job.".to_string());
            status_message.set(
                "Trade profile changed. Demo state reset for the new synthetic job.".to_string(),
            );
        }
    };

    let online_label = move || {
        if simulated_offline.get() {
            "Offline mode simulated"
        } else {
            "Connected mode simulated"
        }
    };

    let step = move || phase.get().step();
    let safety_ready = move || hazards_reviewed.get() && isolation_verified.get();
    let projection = move || {
        demo_projection(
            selected_trade.get(),
            phase.get(),
            safety_ready(),
            evidence_count.get(),
        )
    };
    let next_disabled = move || phase.get() == WorkPhase::Planned && !safety_ready();

    let run_next_action = move |_| {
        let current = phase.get();
        let action = current.next_action();
        let next = transition(
            current,
            action,
            hazards_reviewed.get(),
            isolation_verified.get(),
        );

        if next != current {
            if action == WorkAction::RecordFinding || action == WorkAction::RecordVerification {
                evidence_count.update(|count| *count += 1);
            }
            if action == WorkAction::Restart {
                hazards_reviewed.set(false);
                isolation_verified.set(false);
                evidence_count.set(0);
            }
            phase.set(next);
            status_message.set(format!("Demo state changed: {}.", next.label()));
        } else if current == WorkPhase::Planned {
            status_message.set(
                "Blocked: both demo prerequisites must be marked before this simulated action. This is not a safety determination.".to_string(),
            );
        } else {
            status_message.set(
                "No state change: this action is not valid from the current workflow phase."
                    .to_string(),
            );
        }
    };

    let preview_revalidation = move |_| {
        let graph = task_graph.get();
        match graph.plan_verification_invalidation(
            "access",
            "demo_evidence_correction",
            "urn:luminous:demo:corrective-evidence",
        ) {
            Ok(plan) => {
                impact_summary.set(format!(
                    "Read-only preview at graph revision {}: {} downstream task(s) classified. No status or revision was changed.",
                    plan.expected_graph_revision,
                    plan.impacts.len()
                ));
                impact_items.set(plan.impacts);
                status_message.set(
                    "Correction impact preview generated locally. No authoritative record was written."
                        .to_string(),
                );
            }
            Err(error) => {
                impact_items.set(Vec::new());
                impact_summary.set(format!("Preview unavailable: {error}"));
            }
        }
    };

    view! {
        <main class="field-work-page trade-work-page">
            <header class="field-work-header">
                <div class="field-work-brand">
                    <span class="field-work-mark" aria-hidden="true">"TW"</span>
                    <div>
                        <p class="field-work-eyebrow">"SOL ATLAS · FIELD OPERATIONS"</p>
                        <h1>"Trade workbench"</h1>
                    </div>
                </div>
                <div class="field-work-header-actions">
                    <A href="/" attr:class="field-work-back">"← Globe"</A>
                    <button
                        class="field-work-mode"
                        aria-pressed=move || simulated_offline.get().to_string()
                        on:click=move |_| simulated_offline.update(|offline| *offline = !*offline)
                    >
                        {move || if simulated_offline.get() { "Simulate connected mode" } else { "Simulate offline mode" }}
                    </button>
                </div>
            </header>

            <section class="field-work-disclaimer" role="note" aria-label="Prototype data notice">
                <span class="field-work-disclaimer-icon" aria-hidden="true">"ⓘ"</span>
                <div>
                    <strong>"Interactive prototype — synthetic data only"</strong>
                    <p>"No live work orders, asset registry, instruments, manuals, notifications, service APIs, or machine controls are connected. All buttons change local demo state only. Do not use this prototype to authorize physical work."</p>
                </div>
            </section>

            <div class="field-work-layout">
                <aside class="field-work-sidebar" aria-label="Select trade">
                    <p class="field-work-eyebrow">"TRADE PROFILE"</p>
                    <nav class="field-work-nav">
                        <button class:active=move || selected_trade.get() == Trade::Electrical aria-pressed=move || (selected_trade.get() == Trade::Electrical).to_string() on:click=move |_| select_trade(Trade::Electrical)>
                            <span aria-hidden="true">"ϟ"</span><span>"Electrical"</span>
                        </button>
                        <button class:active=move || selected_trade.get() == Trade::Plumbing aria-pressed=move || (selected_trade.get() == Trade::Plumbing).to_string() on:click=move |_| select_trade(Trade::Plumbing)>
                            <span aria-hidden="true">"≈"</span><span>"Plumbing"</span>
                        </button>
                        <button class:active=move || selected_trade.get() == Trade::Mechanical aria-pressed=move || (selected_trade.get() == Trade::Mechanical).to_string() on:click=move |_| select_trade(Trade::Mechanical)>
                            <span aria-hidden="true">"⚙"</span><span>"Mechanics"</span>
                        </button>
                        <button class:active=move || selected_trade.get() == Trade::Hvac aria-pressed=move || (selected_trade.get() == Trade::Hvac).to_string() on:click=move |_| select_trade(Trade::Hvac)>
                            <span aria-hidden="true">"❄"</span><span>"HVAC"</span>
                        </button>
                        <button class:active=move || selected_trade.get() == Trade::Construction aria-pressed=move || (selected_trade.get() == Trade::Construction).to_string() on:click=move |_| select_trade(Trade::Construction)>
                            <span aria-hidden="true">"⌂"</span><span>"Construction"</span>
                        </button>
                    </nav>
                    <div class="field-work-sidebar-foot">
                        <span class="field-work-status-dot" aria-hidden="true"></span>
                        <div>
                            <strong>{online_label}</strong>
                            <p>"Mode toggle is visual only"</p>
                        </div>
                    </div>
                </aside>

                <section class="field-work-content">
                    <div class="field-work-page-intro">
                        <div>
                            <p class="field-work-eyebrow">{move || selected_trade.get().short_label()} " · SAMPLE WORK ORDER"</p>
                            <h2>"Your next useful action, without the dashboard maze."</h2>
                            <p class="field-work-muted">"One job, explicit prerequisites, evidence, and a state that never pretends an unverified report is a verified result."</p>
                        </div>
                        <span class="field-work-demo-pill">"SYNTHETIC EXAMPLE"</span>
                    </div>

                    <section class="field-work-attention-card trade-work-job-card" aria-label="Selected sample work order">
                        <div class="field-work-attention-symbol" aria-hidden="true">"↗"</div>
                        <div class="field-work-attention-body">
                            <div class="field-work-card-topline">
                                <span class="field-work-label">{move || projection().job_id}</span>
                                <span class="field-work-state field-work-state-unknown">"Synthetic · not assigned"</span>
                            </div>
                            <h3>{move || projection().title}</h3>
                            <p>{move || projection().assets.first().map(|asset| asset.label.clone()).unwrap_or_else(|| "Asset not available".to_string())}</p>
                            <div class="field-work-evidence-row">
                                <span>"Customer: Demo Property / Workshop"</span>
                                <span>"Schedule: not set"</span>
                                <span>"Live asset binding: none"</span>
                            </div>
                            <div class="trade-work-finding-hint">
                                <strong>"Evidence to capture for this trade"</strong>
                                <p>{move || selected_trade.get().evidence_hint()}</p>
                            </div>
                        </div>
                    </section>

                    <div class="field-work-section-heading">
                        <div>
                            <h3>"Work sequence"</h3>
                            <p>"A reported status is not the same as an accepted or verified outcome."</p>
                        </div>
                    </div>

                    <div class="field-work-stepper trade-work-stepper" aria-label="Job workflow progress">
                        <div class=move || {
                            let current_step = step();
                            if current_step == 0 {
                                "current"
                            } else if current_step > 0 {
                                "complete"
                            } else {
                                ""
                            }
                        }><span>"1"</span><small>"Assess"</small></div>
                        <div class=move || {
                            let current_step = step();
                            if current_step == 1 {
                                "current"
                            } else if current_step > 1 {
                                "complete"
                            } else {
                                ""
                            }
                        }><span>"2"</span><small>"Diagnose"</small></div>
                        <div class=move || {
                            let current_step = step();
                            if current_step == 2 {
                                "current"
                            } else if current_step > 2 {
                                "complete"
                            } else {
                                ""
                            }
                        }><span>"3"</span><small>"Plan"</small></div>
                        <div class=move || {
                            let current_step = step();
                            if current_step == 3 {
                                "current"
                            } else if current_step > 3 {
                                "complete"
                            } else {
                                ""
                            }
                        }><span>"4"</span><small>"Execute"</small></div>
                        <div class=move || {
                            let current_step = step();
                            if current_step == 4 {
                                "current"
                            } else if current_step > 4 {
                                "complete"
                            } else {
                                ""
                            }
                        }><span>"5"</span><small>"Verify"</small></div>
                        <div class=move || {
                            let current_step = step();
                            if current_step == 5 {
                                "current"
                            } else if current_step > 5 {
                                "complete"
                            } else {
                                ""
                            }
                        }><span>"6"</span><small>"Handover"</small></div>
                    </div>

                    <section class="field-work-workflow-status trade-work-status" aria-live="polite">
                        <div>
                            <span class="field-work-label">"CURRENT JOB STATE · DEMO"</span>
                            <strong>{move || phase.get().label()}</strong>
                        </div>
                        <span class="field-work-state field-work-state-neutral">{move || format!("{} synthetic evidence event(s)", evidence_count.get())}</span>
                    </section>

                    <div class="trade-work-dependency-grid">
                        <section class="field-work-panel trade-work-dependency-panel" aria-labelledby="trade-work-task-graph-title">
                            <div class="field-work-card-topline">
                                <span class="field-work-label">"DEPENDENCY-AWARE WORK"</span>
                                <span class="field-work-state field-work-state-neutral">
                                    {move || format!("Graph revision {}", task_graph.get().revision())}
                                </span>
                            </div>
                            <h3 id="trade-work-task-graph-title">"Parallel work, explicit prerequisites"</h3>
                            <p>"Inspection and parts can progress independently after the access prerequisite is verified. The functional test remains blocked until both branches have recorded verification."</p>
                            <ol class="trade-work-task-list" aria-label="Synthetic dependency task list">
                                {move || {
                                    let graph_snapshot = task_graph.get();
                                    let ready_ids = graph_snapshot
                                        .ready_task_ids()
                                        .into_iter()
                                        .map(str::to_owned)
                                        .collect::<BTreeSet<_>>();
                                    task_graph_rows(graph_snapshot.graph(), &ready_ids).into_iter().map(|row| {
                                        let task_id_for_action = row.task_id.clone();
                                        let button_accessible_name = format!("{}: {}", row.title, row.action_label);
                                        let action = row.action;
                                        let disabled = action.is_none();
                                        let status_class = row.status_class.to_owned();
                                        let action_label = row.action_label;
                                        view! {
                                            <li class="trade-work-task-row">
                                                <div class="trade-work-task-body">
                                                    <h4>{row.title}</h4>
                                                    <p>{row.dependency_text}</p>
                                                </div>
                                                <span class=format!("trade-work-task-status {status_class}")>
                                                    {row.status_label}
                                                </span>
                                                <button
                                                    class="trade-work-task-action"
                                                    disabled=disabled
                                                    aria-label=button_accessible_name
                                                    on:click=move |_| {
                                                        if let Some(action) = action {
                                                            task_graph.update(|graph| {
                                                                let _ = graph.apply(&task_id_for_action, action);
                                                            });
                                                            impact_items.set(Vec::new());
                                                            impact_summary.set(
                                                                "Task state changed locally; any previous impact preview was cleared."
                                                                    .to_string(),
                                                            );
                                                            status_message.set(
                                                                format!(
                                                                    "Synthetic task {} advanced locally. No authoritative write occurred.",
                                                                    task_id_for_action
                                                                )
                                                            );
                                                        }
                                                    }
                                                >
                                                    {action_label}
                                                </button>
                                            </li>
                                        }
                                    }).collect_view()
                                }}
                            </ol>
                            <p class="trade-work-safety-note">"Demo only: a task status or evidence label is not a physical measurement, worker authorization, or proof of safety."</p>
                        </section>

                        <section class="field-work-panel trade-work-dependency-panel" aria-labelledby="trade-work-revalidation-title">
                            <div class="field-work-card-topline">
                                <span class="field-work-label">"CORRECTION IMPACT"</span>
                                <span class="field-work-state field-work-state-neutral">"Read-only preview"</span>
                            </div>
                            <h3 id="trade-work-revalidation-title">"What becomes untrustworthy if a verification is corrected?"</h3>
                            <p>"Preview downstream effects of challenging the synthetic access verification. The planner preserves current statuses and binds the result to the graph revision; it does not apply the correction."</p>
                            <button class="field-work-secondary trade-work-preview-impact" on:click=preview_revalidation>
                                "Preview downstream impact"
                            </button>
                            <p class="trade-work-impact-summary" role="status" aria-live="polite">{move || impact_summary.get()}</p>
                            <ul class="trade-work-impact-list" aria-label="Downstream correction impact">
                                {move || {
                                    impact_items.get().into_iter().map(|impact| {
                                        let kind_label = impact_kind_label(impact.kind);
                                        let path = format!(
                                            "via {} · {} hop(s) from source",
                                            impact.predecessor_task_id,
                                            impact.path_depth
                                        );
                                        let prior_status = task_status_label(impact.prior_status);
                                        let kind_class = match impact.kind {
                                            RevalidationImpactKind::RequiresRevalidation => "requires-revalidation",
                                            RevalidationImpactKind::RemainsBlocked => "remains-blocked",
                                            RevalidationImpactKind::CancelledNoAction => "cancelled-no-action",
                                        };
                                        view! {
                                            <li class=format!("trade-work-impact-row {kind_class}")>
                                                <div>
                                                    <strong>{impact.task_id}</strong>
                                                    <span>{kind_label}</span>
                                                </div>
                                                <p>{format!("Prior state: {prior_status}. Path: {path}")}</p>
                                            </li>
                                        }
                                    }).collect_view()
                                }}
                            </ul>
                        </section>
                    </div>

                    <div class="trade-work-panels">
                        <section class="field-work-panel">
                            <div class="field-work-card-topline">
                                <span class="field-work-label">"SAFETY GATES"</span>
                                <span class=move || if safety_ready() { "trade-work-gate-ready" } else { "trade-work-gate-blocked" }>
                                    {move || if safety_ready() { "Demo checks marked" } else { "Action blocked until reviewed" }}
                                </span>
                            </div>
                            <h3>"Confirm context before work"</h3>
                            <p>"A production gate must be grounded in the correct site, worker authorization, current procedure, and required physical verification. These buttons only exercise the prototype state."</p>
                            <div class="trade-work-checklist">
                                <button class=move || if hazards_reviewed.get() { "trade-work-check trade-work-check-confirmed" } else { "trade-work-check" } aria-pressed=move || hazards_reviewed.get().to_string() on:click=move |_| hazards_reviewed.update(|v| *v = !*v)>
                                    <span aria-hidden="true">{move || if hazards_reviewed.get() { "✓" } else { "○" }}</span>
                                    <span><strong>"Site hazards reviewed (simulation)"</strong><small>"Does not record a permit or prove a site inspection."</small></span>
                                </button>
                                <button class=move || if isolation_verified.get() { "trade-work-check trade-work-check-confirmed" } else { "trade-work-check" } aria-pressed=move || isolation_verified.get().to_string() on:click=move |_| isolation_verified.update(|v| *v = !*v)>
                                    <span aria-hidden="true">{move || if isolation_verified.get() { "✓" } else { "○" }}</span>
                                    <span><strong>{move || selected_trade.get().isolation_label()}</strong><small>"Does not measure, isolate, lock out, or control any equipment."</small></span>
                                </button>
                            </div>
                            <p class="trade-work-safety-note">"Fail-closed prototype rule: 'ready' means only that both demo switches are set. It is not evidence of real-world safety or authorization."</p>
                        </section>

                        <section class="field-work-panel">
                            <div class="field-work-card-topline">
                                <span class="field-work-label">"NEXT ACTION"</span>
                                <span class="field-work-state field-work-state-neutral">"One primary action"</span>
                            </div>
                            <h3>{move || phase.get().next_action_label()}</h3>
                            <p>"The state machine disallows skipped steps. A production implementation must additionally validate authoritative records, applicable rules, and durable write outcomes."</p>
                            <button class="field-work-primary trade-work-primary" disabled=move || next_disabled() on:click=run_next_action>
                                {move || phase.get().next_action_label()}
                            </button>
                            <button class="field-work-secondary trade-work-reset" on:click=move |_| {
                                phase.set(WorkPhase::Assigned);
                                hazards_reviewed.set(false);
                                isolation_verified.set(false);
                                evidence_count.set(0);
                                task_graph.set(demo_compiled_task_graph(selected_trade.get()));
                                impact_items.set(Vec::new());
                                impact_summary.set(
                                    "No correction-impact preview generated. The task graph is synthetic and read-only with respect to real work.".to_string(),
                                );
                                status_message.set("Demonstration reset. No network action was performed.".to_string());
                            }>"Reset demo state"</button>
                            <p class="trade-work-status-message" role="status" aria-live="polite">{move || status_message.get()}</p>
                        </section>

                        <section class="field-work-panel">
                            <div class="field-work-card-topline">
                                <span class="field-work-label">"EVIDENCE LEDGER"</span>
                                <span class="field-work-state field-work-state-unknown">"Synthetic · unverified"</span>
                            </div>
                            <h3>"Capture once, reuse with provenance"</h3>
                            <p>{move || selected_trade.get().evidence_hint()}</p>
                            <div class="trade-work-evidence-summary">
                                <div><span>"Events in this demo"</span><strong>{move || evidence_count.get().to_string()}</strong></div>
                                <div><span>"Asset identity"</span><strong>"Not authenticated"</strong></div>
                                <div><span>"Instrument / units"</span><strong>"Not recorded"</strong></div>
                                <div><span>"Authoritative sync"</span><strong>"Not connected"</strong></div>
                            </div>
                            <p class="trade-work-safety-note">"Production evidence should preserve source, timestamp, actor, units, instrument where applicable, asset reference, procedure revision, and whether a value is reported, modelled, measured, or verified."</p>
                        </section>
                    </div>

                    <footer class="field-work-footer">
                        <span>"SOL ATLAS · CROSS-TRADE UX CONCEPT"</span>
                        <span>"Prototype only · no live work or safety decisions"</span>
                    </footer>
                </section>
            </div>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::{DemoProgress, Trade, WorkAction as A, WorkPhase as P, transition};

    #[test]
    fn complete_workflow_keeps_assessment_diagnosis_plan_execution_and_verification_distinct() {
        let mut phase = P::Assigned;
        phase = transition(phase, A::BeginAssessment, false, false);
        assert_eq!(phase, P::Inspecting);
        phase = transition(phase, A::RecordFinding, false, false);
        assert_eq!(phase, P::Diagnosed);
        phase = transition(phase, A::ApprovePlan, false, false);
        assert_eq!(phase, P::Planned);

        assert_eq!(transition(phase, A::StartWork, false, false), P::Planned);
        assert_eq!(transition(phase, A::StartWork, true, false), P::Planned);
        assert_eq!(transition(phase, A::StartWork, false, true), P::Planned);

        phase = transition(phase, A::StartWork, true, true);
        assert_eq!(phase, P::InProgress);
        phase = transition(phase, A::ReportWorkComplete, true, true);
        assert_eq!(phase, P::VerificationDue);
        phase = transition(phase, A::RecordVerification, true, true);
        assert_eq!(phase, P::Verified);
        phase = transition(phase, A::Handover, true, true);
        assert_eq!(phase, P::HandedOver);
    }

    #[test]
    fn physical_work_cannot_be_started_until_both_demo_prerequisites_are_set() {
        assert_eq!(
            transition(P::Planned, A::StartWork, false, false),
            P::Planned
        );
        assert_eq!(
            transition(P::Planned, A::StartWork, true, false),
            P::Planned
        );
        assert_eq!(
            transition(P::Planned, A::StartWork, false, true),
            P::Planned
        );
        assert_eq!(
            transition(P::Planned, A::StartWork, true, true),
            P::InProgress
        );
    }

    #[test]
    fn completion_report_is_not_verification_or_handover() {
        assert_eq!(
            transition(P::InProgress, A::ReportWorkComplete, true, true),
            P::VerificationDue
        );
        assert_eq!(
            transition(P::VerificationDue, A::Handover, true, true),
            P::VerificationDue
        );
        assert_eq!(
            transition(P::Verified, A::Handover, true, true),
            P::HandedOver
        );
    }

    #[test]
    fn unsupported_state_action_pairs_are_noops() {
        assert_eq!(
            transition(P::Assigned, A::Handover, true, true),
            P::Assigned
        );
        assert_eq!(
            transition(P::Diagnosed, A::ReportWorkComplete, true, true),
            P::Diagnosed
        );
        assert_eq!(
            transition(P::VerificationDue, A::StartWork, true, true),
            P::VerificationDue
        );
    }

    #[test]
    fn reset_is_explicit() {
        assert_eq!(
            transition(P::HandedOver, A::Restart, false, false),
            P::Assigned
        );
    }

    #[test]
    fn synthetic_job_identity_and_evidence_ids_are_trade_scoped() {
        let electrical = super::demo_projection(Trade::Electrical, P::Verified, true, 2);
        let plumbing_with_history = super::demo_projection(Trade::Plumbing, P::Assigned, false, 2);
        let new_plumbing_job = super::demo_projection(Trade::Plumbing, P::Assigned, false, 0);
        assert_ne!(electrical.job_id, plumbing_with_history.job_id);
        assert_ne!(
            electrical.assets[0].asset_id,
            plumbing_with_history.assets[0].asset_id
        );
        assert_ne!(
            electrical.evidence[0].evidence_id,
            plumbing_with_history.evidence[0].evidence_id
        );
        assert_eq!(new_plumbing_job.evidence.len(), 0);
    }

    #[test]
    fn switching_trade_resets_progress_but_reselecting_same_trade_preserves_it() {
        let in_progress = DemoProgress {
            phase: P::Verified,
            hazards_reviewed: true,
            isolation_verified: true,
            evidence_count: 2,
        };
        let (same_trade, preserved) =
            super::switch_demo_session(Trade::Electrical, Trade::Electrical, in_progress);
        assert_eq!(same_trade, Trade::Electrical);
        assert_eq!(preserved, in_progress);

        let (new_trade, reset) =
            super::switch_demo_session(Trade::Electrical, Trade::Plumbing, in_progress);
        assert_eq!(new_trade, Trade::Plumbing);
        assert_eq!(reset, DemoProgress::default());
    }

    #[test]
    fn demo_adapter_emits_valid_never_fresh_projection_for_each_trade() {
        for trade in [
            Trade::Electrical,
            Trade::Plumbing,
            Trade::Mechanical,
            Trade::Hvac,
            Trade::Construction,
        ] {
            let projection = super::demo_projection(trade, P::Assigned, false, 0);
            assert_eq!(projection.validate(), Ok(()));
            assert_eq!(projection.transport, super::TransportState::LocalOnly);
            assert_eq!(projection.freshness, super::ProjectionFreshness::Unknown);
            assert_eq!(
                projection.safety_gate.status,
                super::SafetyGateStatus::Blocked
            );
        }
    }

    #[test]
    fn task_graph_demo_is_valid_and_exposes_two_parallel_ready_branches_for_every_trade() {
        for trade in [
            Trade::Electrical,
            Trade::Plumbing,
            Trade::Mechanical,
            Trade::Hvac,
            Trade::Construction,
        ] {
            let graph = super::demo_compiled_task_graph(trade);
            assert_eq!(graph.graph().validate(), Ok(()));
            assert_eq!(graph.ready_task_ids(), vec!["parts", "inspection"]);
            assert_eq!(
                graph.graph().graph_id,
                format!("{}:task-graph-demo", trade.job_id())
            );
        }
    }

    #[test]
    fn demo_task_graph_keeps_join_blocked_until_both_branches_are_verified() {
        let mut graph = super::demo_compiled_task_graph(Trade::Electrical);
        for (task_id, action) in [
            ("inspection", super::TaskAction::Start),
            ("inspection", super::TaskAction::ReportCompletion),
            ("inspection", super::TaskAction::RecordVerification),
        ] {
            graph.apply(task_id, action).unwrap();
        }
        assert_eq!(graph.ready_task_ids(), vec!["parts"]);
        assert_eq!(
            graph.apply("functional-test", super::TaskAction::Start),
            Err(super::TaskGraphError::DependencyNotVerified {
                task_id: "functional-test".to_owned(),
                dependency_ids: vec!["parts".to_owned()],
            })
        );
    }

    #[test]
    fn previewing_impact_is_read_only_and_trade_switch_resets_graph() {
        let electrical = super::demo_compiled_task_graph(Trade::Electrical);
        let plan = electrical
            .plan_verification_invalidation(
                "access",
                "demo_evidence_correction",
                "urn:luminous:demo:corrective-evidence",
            )
            .unwrap();
        assert_eq!(plan.expected_graph_revision, 1);
        assert_eq!(plan.impacts.len(), 4);
        assert_eq!(electrical.revision(), 1);
        assert_eq!(electrical.ready_task_ids(), vec!["parts", "inspection"]);

        let plumbing = super::demo_compiled_task_graph(Trade::Plumbing);
        assert_eq!(plumbing.revision(), 1);
        assert_eq!(plumbing.ready_task_ids(), vec!["parts", "inspection"]);
        assert_ne!(electrical.graph().graph_id, plumbing.graph().graph_id);
    }

    #[test]
    fn each_trade_profile_has_distinct_work_order_identity_and_evidence_guidance() {
        let trades = [
            Trade::Electrical,
            Trade::Plumbing,
            Trade::Mechanical,
            Trade::Hvac,
            Trade::Construction,
        ];
        let ids = trades
            .iter()
            .map(|trade| trade.job_id())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(ids.len(), trades.len());
        for trade in trades {
            assert!(!trade.job_title().is_empty());
            assert!(!trade.asset_label().is_empty());
            assert!(!trade.evidence_hint().is_empty());
            assert!(!trade.isolation_label().is_empty());
        }
    }
}
