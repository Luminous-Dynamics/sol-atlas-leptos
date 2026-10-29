mod reference_effect_adapter;

use reference_effect_adapter::{LostAckResource, ReferenceFencedResource};
use sol_atlas_policy_store_contract::{
    RecoveryExecutionEffectConformanceEvidenceV1, RecoveryExecutionEffectConformanceReportV1,
    RecoveryExecutionEffectSafetyProfileV1, RecoveryExecutionFencedResource,
    RecoveryExecutionFencingCapabilityV1, RecoveryExecutionIdempotencyCapabilityV1,
    RecoveryExecutionProtectedMutationNextActionV1,
    RecoveryExecutionProtectedMutationOrchestrationStateV1,
    RecoveryExecutionProtectedMutationReconciler,
    RecoveryExecutionProtectedMutationReconciliationOutcome,
    RecoveryExecutionProtectedMutationResult, RecoveryExecutionProtectedMutationV1,
    RecoveryExecutionReconciliationCapabilityV1,
};
use std::process::Command;
use std::sync::{Arc, Barrier};
use std::thread;

const ADAPTER_PATH: &str = concat!(
    "sol-atlas-policy-store-contract/tests/",
    "reference_effect_adapter.rs"
);
const HARNESS_PATH: &str = concat!(
    "sol-atlas-policy-store-contract/tests/",
    "reference_effect_conformance.rs"
);
fn qualified_profile() -> RecoveryExecutionEffectSafetyProfileV1 {
    RecoveryExecutionEffectSafetyProfileV1 {
        schema: RecoveryExecutionEffectSafetyProfileV1::SCHEMA.into(),
        fencing: RecoveryExecutionFencingCapabilityV1::EnforcedAtMutationBoundary,
        idempotency: RecoveryExecutionIdempotencyCapabilityV1::StableKey,
        reconciliation: RecoveryExecutionReconciliationCapabilityV1::StrongReadBack,
        claim_ceiling: "Reference adapter evidence only.".into(),
    }
}

fn mutation(
    execution_id: &str,
    fingerprint: &str,
    attempt_id: &str,
    fence_epoch: u64,
    idempotency_key: Option<&str>,
) -> RecoveryExecutionProtectedMutationV1 {
    RecoveryExecutionProtectedMutationV1 {
        execution_id: execution_id.into(),
        execution_input_snapshot: fingerprint.into(),
        attempt_id: attempt_id.into(),
        fence_epoch,
        idempotency_key: idempotency_key.map(str::to_owned),
    }
}

fn git_blob_sha(path: &str) -> String {
    let output = Command::new("git")
        .args(["rev-parse", &format!("HEAD:{path}")])
        .output()
        .expect("git must be available for source provenance qualification");
    assert!(
        output.status.success(),
        "git could not resolve qualified source path {path}"
    );
    String::from_utf8(output.stdout)
        .expect("git output must be UTF-8")
        .trim()
        .to_owned()
}

fn run_reference_conformance() -> RecoveryExecutionEffectConformanceEvidenceV1 {
    let resource = ReferenceFencedResource::default();
    resource.set_epoch(2);
    assert_eq!(resource.current_epoch(), 2);

    let current = mutation(
        "execution-conformance-current",
        "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        "attempt-conformance",
        2,
        Some("conformance-key"),
    );
    let stale = RecoveryExecutionProtectedMutationV1 {
        fence_epoch: 1,
        ..current.clone()
    };
    let future = RecoveryExecutionProtectedMutationV1 {
        fence_epoch: 3,
        ..current.clone()
    };
    let changed_same_key = mutation(
        &current.execution_id,
        "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        &current.attempt_id,
        current.fence_epoch,
        current.idempotency_key.as_deref(),
    );
    let changed_idempotency_key = RecoveryExecutionProtectedMutationV1 {
        idempotency_key: Some("different-key".into()),
        ..current.clone()
    };
    let same_key_other_execution = mutation(
        "execution-conformance-other",
        "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        "attempt-other",
        2,
        Some("conformance-key"),
    );

    let current_fence_accepted = resource
        .mutate_if_fence_is_current(&current)
        .expect("current")
        == RecoveryExecutionProtectedMutationResult::Applied;
    let stable_key_replay_safe = resource
        .mutate_if_fence_is_current(&current)
        .expect("replay")
        == RecoveryExecutionProtectedMutationResult::AlreadyAppliedSameRequest;
    let stale_fence_rejected = resource.mutate_if_fence_is_current(&stale).expect("stale")
        == RecoveryExecutionProtectedMutationResult::RejectedStaleFence;
    let future_fence_rejected = resource
        .mutate_if_fence_is_current(&future)
        .expect("future")
        == RecoveryExecutionProtectedMutationResult::RejectedFutureFence;
    let same_execution_different_request_rejected = resource
        .mutate_if_fence_is_current(&changed_same_key)
        .expect("changed request")
        == RecoveryExecutionProtectedMutationResult::RejectedIdentityMismatch;
    let different_execution_same_key_rejected = resource
        .mutate_if_fence_is_current(&same_key_other_execution)
        .expect("cross-execution key collision")
        == RecoveryExecutionProtectedMutationResult::RejectedIdentityMismatch;
    let different_request_same_key_rejected =
        same_execution_different_request_rejected && different_execution_same_key_rejected;
    let changed_idempotency_key_rejected = resource
        .mutate_if_fence_is_current(&changed_idempotency_key)
        .expect("changed idempotency key")
        == RecoveryExecutionProtectedMutationResult::RejectedIdentityMismatch;

    let exact_reconciliation = resource
        .reconcile_mutation(&current)
        .expect("exact read-back")
        == RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedAppliedSameRequest;
    assert_eq!(
        resource
            .reconcile_mutation(&same_key_other_execution)
            .expect("cross-execution key reconciliation"),
        RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedDifferentRequest
    );

    let point_in_time = mutation(
        "execution-point-in-time",
        "sha256:abababababababababababababababababababababababababababababababab",
        "attempt-point-in-time",
        2,
        None,
    );
    let observed_before_apply = resource
        .reconcile_mutation(&point_in_time)
        .expect("point-in-time before")
        == RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedNotApplied;
    assert_eq!(
        resource
            .mutate_if_fence_is_current(&point_in_time)
            .expect("point-in-time apply"),
        RecoveryExecutionProtectedMutationResult::Applied
    );
    let observed_after_apply = resource
        .reconcile_mutation(&point_in_time)
        .expect("point-in-time after")
        == RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedAppliedSameRequest;
    let point_in_time_semantics_explicit = observed_before_apply && observed_after_apply;

    let lost_ack = LostAckResource::default();
    lost_ack.set_epoch(2);
    lost_ack.arm_lost_ack();
    let lost_ack_result = lost_ack
        .mutate_if_fence_is_current(&current)
        .expect("lost-ack mutation");
    let lost_ack_reconciled = lost_ack
        .reconcile_mutation(&current)
        .expect("lost-ack reconciliation");
    let indeterminate_ack_reconciled = lost_ack_result
        == RecoveryExecutionProtectedMutationResult::Indeterminate
        && lost_ack_reconciled
            == RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedAppliedSameRequest;

    let concurrent = Arc::new(ReferenceFencedResource::default());
    concurrent.set_epoch(2);
    let barrier = Arc::new(Barrier::new(32));
    let current_concurrent = mutation(
        "execution-conformance-concurrent-current",
        "sha256:abababababababababababababababababababababababababababababababab",
        "attempt-current",
        2,
        None,
    );
    let stale_concurrent = mutation(
        "execution-conformance-concurrent-stale",
        "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        "attempt-stale",
        1,
        None,
    );
    let mut joins = Vec::new();
    for index in 0..32 {
        let store = Arc::clone(&concurrent);
        let gate = Arc::clone(&barrier);
        let request = if index == 0 {
            current_concurrent.clone()
        } else {
            stale_concurrent.clone()
        };
        joins.push(thread::spawn(move || {
            gate.wait();
            store.mutate_if_fence_is_current(&request)
        }));
    }

    let mut stale_rejected_count = 0usize;
    let mut current_applied_count = 0usize;
    for join in joins {
        match join
            .join()
            .expect("concurrent join")
            .expect("concurrent result")
        {
            RecoveryExecutionProtectedMutationResult::RejectedStaleFence => {
                stale_rejected_count += 1;
            }
            RecoveryExecutionProtectedMutationResult::Applied => {
                current_applied_count += 1;
            }
            other => panic!("unexpected concurrent outcome: {other:?}"),
        }
    }
    let concurrent_fencing_preserved = current_applied_count == 1 && stale_rejected_count == 31;

    RecoveryExecutionEffectConformanceEvidenceV1 {
        current_fence_accepted,
        stale_fence_rejected,
        future_fence_rejected,
        concurrent_fencing_preserved,
        stable_key_replay_safe,
        different_request_same_key_rejected,
        changed_idempotency_key_rejected,
        indeterminate_ack_reconciled,
        transactionally_coupled_retry_safe: false,
        exact_reconciliation,
        point_in_time_semantics_explicit,
        strong_read_back_verified: exact_reconciliation && indeterminate_ack_reconciled,
        eventually_consistent_read_back_verified: false,
    }
}

#[test]
fn reference_conformance_report_is_bound_to_current_source_and_profile() {
    let profile = qualified_profile();
    let report: RecoveryExecutionEffectConformanceReportV1 = serde_json::from_str(include_str!(
        "../conformance/reference_effect_report_v1.json"
    ))
    .expect("checked-in report must be valid JSON");

    assert!(report.is_well_formed());
    assert_eq!(report.adapter_id, "reference-memory-resource");
    assert_eq!(report.harness_id, "sol-atlas-protected-effect-conformance");
    assert_eq!(report.adapter_revision, git_blob_sha(ADAPTER_PATH));
    assert_eq!(report.harness_revision, git_blob_sha(HARNESS_PATH));
    assert_eq!(report.profile_digest, profile.digest());

    let observed = run_reference_conformance();
    assert_eq!(report.evidence, observed);
    assert_eq!(report.evidence_digest, observed.digest());
    assert!(report.supports_profile(&profile));

    let mut drifted_profile = profile.clone();
    drifted_profile.claim_ceiling = "Profile drift must not widen permission.".into();
    assert!(!report.supports_profile(&drifted_profile));

    let state = RecoveryExecutionProtectedMutationOrchestrationStateV1::ObservedNotApplied;
    assert_eq!(
        profile.next_action_for_mutation(
            &mutation(
                "execution-recovery",
                "sha256:1111111111111111111111111111111111111111111111111111111111111111",
                "attempt-recovery",
                2,
                None,
            ),
            state,
        ),
        RecoveryExecutionProtectedMutationNextActionV1::RequireIdempotencyOrManualRecovery
    );
}

#[test]
fn indeterminate_ack_is_reconciled_without_a_second_mutation() {
    let resource = LostAckResource::default();
    resource.set_epoch(2);
    resource.arm_lost_ack();

    let request = mutation(
        "execution-indeterminate",
        "sha256:2222222222222222222222222222222222222222222222222222222222222222",
        "attempt-indeterminate",
        2,
        Some("stable-indeterminate-key"),
    );
    assert_eq!(
        resource
            .mutate_if_fence_is_current(&request)
            .expect("mutation"),
        RecoveryExecutionProtectedMutationResult::Indeterminate
    );
    assert_eq!(
        resource.reconcile_mutation(&request).expect("reconcile"),
        RecoveryExecutionProtectedMutationReconciliationOutcome::ObservedAppliedSameRequest
    );
}
