#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Real-service qualification harness for the GCS generation-fenced adapter."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import uuid
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
from pathlib import Path

from scripts.gcs_generation_fenced_adapter import (
    GcsGenerationFencedObject,
    MutationRequest,
    access_token,
    sha256_prefixed,
)


SCHEMA = "sol-atlas:recovery-execution-effect-external-report:v1"
CASE_SET_PATH = (
    "sol-atlas-policy-store-contract/conformance/"
    "gcs_external_effect_cases_v1.json"
)
EXPECTED_CASE_SET = (
    "sol-atlas:recovery-execution-effect-external-conformance-cases:"
    "gcs-generation-v1"
)
CASE_SET_SCHEMA = "sol-atlas:recovery-execution-effect-case-set:v1"
ADAPTER_PATH = "scripts/gcs_generation_fenced_adapter.py"
HARNESS_PATH = "scripts/qualify_gcs_external_effect.py"
WORKFLOW_PATH = ".github/workflows/qualify-gcs.yml"
ADAPTER_ID = "gcs-generation-fenced-object"
HARNESS_ID = "sol-atlas-gcs-external-conformance"
CLAIM_CEILING = (
    "GCS generation-precondition evidence only; replay safety applies while "
    "the qualified live object state remains retained; no universal "
    "exactly-once claim."
)
def load_case_set() -> dict[str, object]:
    case_set = json.loads(
        Path(CASE_SET_PATH).read_text(encoding="utf-8")
    )
    if (
        case_set.get("schema") != CASE_SET_SCHEMA
        or case_set.get("case_set") != EXPECTED_CASE_SET
    ):
        raise AssertionError("invalid GCS case-set identity")
    cases = case_set.get("cases")
    if not isinstance(cases, list) or not cases:
        raise AssertionError("GCS case-set has no cases")
    case_ids = [case.get("id") for case in cases if isinstance(case, dict)]
    if (
        len(case_ids) != len(cases)
        or any(not isinstance(case_id, str) or not case_id for case_id in case_ids)
        or len(set(case_ids)) != len(case_ids)
    ):
        raise AssertionError("GCS case-set has invalid or duplicate case IDs")
    return case_set


def case_ids(case_set: dict[str, object]) -> list[str]:
    return [case["id"] for case in case_set["cases"]]


def github_execution_context() -> dict[str, str]:
    required = {
        "repository": os.environ.get("GITHUB_REPOSITORY", ""),
        "repository_id": os.environ.get("GITHUB_REPOSITORY_ID", ""),
        "environment": os.environ.get("GITHUB_ENVIRONMENT", ""),
        "workflow": os.environ.get("GITHUB_WORKFLOW", ""),
        "workflow_ref": os.environ.get("GITHUB_WORKFLOW_REF", ""),
        "workflow_sha": os.environ.get("GITHUB_WORKFLOW_SHA", ""),
        "event": os.environ.get("GITHUB_EVENT_NAME", ""),
        "ref": os.environ.get("GITHUB_REF", ""),
        "run_id": os.environ.get("GITHUB_RUN_ID", ""),
        "sha": os.environ.get("GITHUB_SHA", ""),
        "run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT", ""),
    }
    missing = [name for name, value in required.items() if not value]
    if missing:
        raise AssertionError(
            "missing GitHub execution context: " + ", ".join(missing)
        )
    expected = {
        "repository": "Luminous-Dynamics/sol-atlas-leptos",
        "repository_id": "1195997641",
        "environment": "sol-atlas-gcs-qualification",
        "workflow": "Qualify GCS external effect",
        "event": "workflow_dispatch",
        "ref": "refs/heads/main",
        "workflow_ref": (
            "Luminous-Dynamics/sol-atlas-leptos/.github/workflows/"
            "qualify-gcs.yml@refs/heads/main"
        ),
    }
    for name, expected_value in expected.items():
        if required[name] != expected_value:
            raise AssertionError(
                f"GitHub execution context drift for {name}: "
                f"{required[name]!r} != {expected_value!r}"
            )
    return required


def verify_checked_out_source_commit() -> str:
    expected = os.environ.get("GITHUB_SHA", "")
    if not expected:
        raise AssertionError("GITHUB_SHA is missing")
    actual = git_head()
    if actual != expected:
        raise AssertionError(
            f"checked-out source drift: {actual!r} != GITHUB_SHA {expected!r}"
        )
    return actual


def load_wif_verification(path: str) -> dict[str, object]:
    verification = json.loads(Path(path).read_text(encoding="utf-8"))
    if verification.get("schema") != "sol-atlas:gcs-wif-trust-verification:v4":
        raise AssertionError("wrong WIF trust verification schema")
    if not verification.get("attribute_mapping_verified"):
        raise AssertionError("WIF attribute mapping was not verified")
    if not verification.get("attribute_condition_verified"):
        raise AssertionError("WIF attribute condition was not verified")
    if not verification.get("service_account_binding_verified"):
        raise AssertionError("WIF service-account binding was not verified")
    if not verification.get("provider_pool_exclusive"):
        raise AssertionError("WIF provider pool was not verified exclusive")
    if not verification.get("service_account_project_verified"):
        raise AssertionError("WIF service account project was not verified")
    return verification


def validate_case_set() -> str:
    case_set = load_case_set()
    for case in case_set["cases"]:
        if (
            not isinstance(case, dict)
            or not isinstance(case.get("id"), str)
            or not isinstance(case.get("semantic"), str)
            or not case["semantic"].strip()
        ):
            raise AssertionError("GCS case-set contains an incomplete case")
    case_set_digest = digest(case_set)
    print(
        "verified case-set: "
        + EXPECTED_CASE_SET
        + " "
        + case_set_digest
    )
    return case_set_digest


def git_sha(path: str) -> str:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD:" + path],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def git_head() -> str:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def profile() -> dict[str, str]:
    return {
        "schema": "sol-atlas:recovery-execution-effect-safety-profile:v1",
        "fencing": "EnforcedAtMutationBoundary",
        "idempotency": "StableKey",
        "reconciliation": "StrongReadBack",
        "claim_ceiling": CLAIM_CEILING,
    }


def profile_digest(value: dict[str, str]) -> str:
    material = (
        f"schema={len(value['schema'])}:{value['schema']}|"
        "fencing=enforced|"
        "idempotency=stable-key|"
        "reconciliation=strong-read-back|"
        f"claim={len(value['claim_ceiling'])}:{value['claim_ceiling']}"
    )
    return "sha256:" + hashlib.sha256(material.encode()).hexdigest()


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    ).encode("utf-8")


def digest(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical(value)).hexdigest()


def request(
    name: str,
    generation: int,
    *,
    execution: str | None = None,
    key: str | None = None,
    body: bytes | None = None,
) -> MutationRequest:
    payload = body or f"sol-atlas:{name}:v1".encode()
    execution_id = execution or f"gcs-{name}-{uuid.uuid4().hex[:12]}"
    idempotency_key = key or f"gcs-key-{name}-{uuid.uuid4().hex[:12]}"
    return MutationRequest(
        execution_id=execution_id,
        input_fingerprint=sha256_prefixed(payload),
        attempt_id=f"attempt-{name}",
        fence_generation=generation,
        idempotency_key=idempotency_key,
        body=payload,
    )


def create_setup(resource: GcsGenerationFencedObject, name: str) -> int:
    setup = request(
        name,
        0,
        execution=f"setup-{name}-{uuid.uuid4().hex[:12]}",
        key=f"setup-key-{name}-{uuid.uuid4().hex[:12]}",
    )
    result = resource.raw_put(setup)
    if result.status not in {200, 201}:
        raise RuntimeError(
            f"setup object creation failed with HTTP {result.status}"
        )
    state = resource.state()
    if state is None:
        raise RuntimeError("setup object was not observable after creation")
    return state.generation


def record(case_id: str, passed: bool, observed: dict[str, object]) -> dict[str, object]:
    return {
        "id": case_id,
        "passed": passed,
        "observed": observed,
    }


def run_qualification(
    bucket: str,
    object_prefix: str,
    wif_verification_path: str,
) -> dict[str, object]:
    case_set = load_case_set()
    wif_verification = load_wif_verification(wif_verification_path)
    github_context = github_execution_context()
    source_commit = verify_checked_out_source_commit()
    token = access_token()
    run_id = os.environ.get("GITHUB_RUN_ID", "local")
    run_attempt = os.environ.get("GITHUB_RUN_ATTEMPT", "1")
    unique = uuid.uuid4().hex[:12]
    root = (
        object_prefix.rstrip("/")
        + f"/run-{run_id}-attempt-{run_attempt}-{unique}"
    )
    main_name = root + "/main.bin"
    point_name = root + "/point-in-time.bin"
    race_name = root + "/metadata-race.bin"
    resource = GcsGenerationFencedObject(bucket, main_name, token)
    point_resource = GcsGenerationFencedObject(bucket, point_name, token)
    race_resource = GcsGenerationFencedObject(bucket, race_name, token)
    cases: list[dict[str, object]] = []

    try:
        generation = create_setup(resource, "main")

        current = request(
            "advance",
            generation,
            execution="execution-gcs-advance",
            key="idempotency-gcs-advance",
            body=b"sol-atlas:gcs:advance:v1",
        )
        applied = resource.apply(current)
        after_current = resource.state()
        if applied != "Applied" or after_current is None:
            raise AssertionError("current-fence mutation did not apply")
        cases.append(
            record(
                "current_fence_accepted",
                True,
                {
                    "result": applied,
                    "precondition_generation": generation,
                    "result_generation": after_current.generation,
                },
            )
        )
        generation = after_current.generation

        before_replay = generation
        replay = resource.apply(current)
        after_replay = resource.state()
        if replay != "AlreadyAppliedSameRequest" or after_replay is None:
            raise AssertionError("stable-key replay was not recognized")
        cases.append(
            record(
                "stable_key_replay_safe",
                after_replay.generation == before_replay,
                {
                    "result": replay,
                    "generation_before": before_replay,
                    "generation_after": after_replay.generation,
                },
            )
        )

        stale = request(
            "stale",
            generation - 1,
            execution="execution-gcs-stale",
            key="idempotency-gcs-stale",
        )
        stale_result = resource.raw_put(stale)
        cases.append(
            record(
                "stale_fence_rejected",
                stale_result.status == 412,
                {
                    "http_status": stale_result.status,
                    "precondition_generation": stale.fence_generation,
                    "current_generation": generation,
                },
            )
        )
        if stale_result.status != 412:
            raise AssertionError("GCS accepted a stale generation precondition")

        future = request(
            "future",
            generation + 1,
            execution="execution-gcs-future",
            key="idempotency-gcs-future",
        )
        future_result = resource.raw_put(future)
        cases.append(
            record(
                "future_fence_rejected",
                future_result.status == 412,
                {
                    "http_status": future_result.status,
                    "precondition_generation": future.fence_generation,
                    "current_generation": generation,
                },
            )
        )
        if future_result.status != 412:
            raise AssertionError("GCS accepted an unestablished future generation")

        before_identity = resource.state()
        same_key_other_execution = request(
            "same-key-other-execution",
            generation,
            execution="execution-gcs-other",
            key=current.idempotency_key,
            body=current.body,
        )
        identity_result = resource.apply(same_key_other_execution)
        after_identity = resource.state()
        if (
            identity_result != "RejectedIdentityMismatch"
            or before_identity is None
            or after_identity is None
            or after_identity.generation != before_identity.generation
        ):
            raise AssertionError("cross-execution idempotency collision was not blocked")
        cases.append(
            record(
                "different_request_same_key_rejected",
                True,
                {
                    "result": identity_result,
                    "generation_unchanged": True,
                },
            )
        )

        changed_same_key = request(
            "same-execution-different-request",
            generation,
            execution=current.execution_id,
            key=current.idempotency_key,
            body=b"sol-atlas:gcs:changed-request:v1",
        )
        changed_result = resource.apply(changed_same_key)
        cases.append(
            record(
                "changed_request_same_key_rejected",
                changed_result == "RejectedIdentityMismatch",
                {"result": changed_result},
            )
        )
        if changed_result != "RejectedIdentityMismatch":
            raise AssertionError("same-key request mutation was not rejected")

        changed_key = request(
            "changed-key",
            generation,
            execution=current.execution_id,
            key="different-idempotency-key",
            body=current.body,
        )
        changed_key_result = resource.apply(changed_key)
        cases.append(
            record(
                "changed_idempotency_key_rejected",
                changed_key_result == "RejectedIdentityMismatch",
                {"result": changed_key_result},
            )
        )
        if changed_key_result != "RejectedIdentityMismatch":
            raise AssertionError("idempotency-key drift was not rejected")

        concurrent_a = request(
            "concurrent-a",
            generation,
            execution="execution-gcs-concurrent-a",
            key="idempotency-gcs-concurrent-a",
            body=b"sol-atlas:gcs:concurrent:a:v1",
        )
        concurrent_b = request(
            "concurrent-b",
            generation,
            execution="execution-gcs-concurrent-b",
            key="idempotency-gcs-concurrent-b",
            body=b"sol-atlas:gcs:concurrent:b:v1",
        )
        with ThreadPoolExecutor(max_workers=2) as pool:
            futures = [
                pool.submit(resource.raw_put, concurrent_a),
                pool.submit(resource.raw_put, concurrent_b),
            ]
            concurrent_results = [future.result() for future in futures]
        statuses = sorted(result.status for result in concurrent_results)
        concurrency_ok = statuses == [200, 412]
        winning_request = (
            concurrent_a if concurrent_results[0].status == 200 else concurrent_b
        )
        cases.append(
            record(
                "concurrent_fencing_preserved",
                concurrency_ok,
                {
                    "http_statuses": statuses,
                    "winner_execution": winning_request.execution_id,
                },
            )
        )
        if not concurrency_ok:
            raise AssertionError(
                f"expected exactly one GCS 200 and one 412, got {statuses}"
            )
        winning_state = resource.state()
        if winning_state is None:
            raise AssertionError("concurrent winner was not observable")
        generation = winning_state.generation

        race_generation = create_setup(race_resource, "metadata-race")
        baseline = race_resource.state()
        if baseline is None:
            raise AssertionError("metadata-race setup was not observable")

        def mutate_metadata(observed):
            result = race_resource.update_metadata(
                observed.generation,
                observed.metageneration,
                {"reconciliation-marker": "race-v1"},
            )
            if result.status != 200:
                raise AssertionError(
                    f"metadata race update failed: HTTP {result.status}"
                )

        coherent_state = race_resource.state(
            between_metadata_and_data=mutate_metadata
        )
        metadata_coherence_ok = (
            coherent_state is not None
            and coherent_state.generation == race_generation
            and coherent_state.metageneration > baseline.metageneration
            and coherent_state.metadata.get("reconciliation-marker") == "race-v1"
            and coherent_state.data_sha256 == baseline.data_sha256
        )
        cases.append(
            record(
                "metadata_readback_coherence",
                metadata_coherence_ok,
                {
                    "generation_unchanged": (
                        coherent_state.generation == baseline.generation
                        if coherent_state
                        else False
                    ),
                    "metageneration_before": baseline.metageneration,
                    "metageneration_after": (
                        coherent_state.metageneration
                        if coherent_state
                        else None
                    ),
                    "marker_observed": (
                        coherent_state.metadata.get("reconciliation-marker")
                        if coherent_state
                        else None
                    ),
                    "body_digest_unchanged": (
                        coherent_state.data_sha256 == baseline.data_sha256
                        if coherent_state
                        else False
                    ),
                    "retry_on_precondition_mismatch": True
                },
            )
        )
        if not metadata_coherence_ok:
            raise AssertionError(
                "metadata/data read-back did not converge to one coherent snapshot"
            )

        lost_ack = request(
            "lost-ack",
            generation,
            execution="execution-gcs-lost-ack",
            key="idempotency-gcs-lost-ack",
            body=b"sol-atlas:gcs:lost-ack:v1",
        )
        before_lost_ack = resource.state()
        if before_lost_ack is None:
            raise AssertionError("lost-ack object disappeared before the request")
        resource.raw_put(lost_ack, discard_response=True)
        reconciled = resource.reconcile(lost_ack)
        after_lost_ack = resource.state()
        if after_lost_ack is None:
            raise AssertionError("lost-ack object disappeared during reconciliation")
        if reconciled == "ObservedAppliedSameRequest":
            no_second_mutation = after_lost_ack.generation != before_lost_ack.generation
        else:
            no_second_mutation = after_lost_ack.generation == before_lost_ack.generation
        lost_ack_observation_ok = reconciled in {
            "ObservedAppliedSameRequest",
            "ObservedNotApplied",
            "ObservedDifferentRequest",
        }
        cases.append(
            record(
                "indeterminate_ack_reconciled",
                no_second_mutation and lost_ack_observation_ok,
                {
                    "acknowledgement": "response_discarded_before_read",
                    "reconciliation": reconciled,
                    "commit_status": (
                        "observed_applied"
                        if reconciled == "ObservedAppliedSameRequest"
                        else "indeterminate_at_reconciliation"
                    ),
                    "generation_before": before_lost_ack.generation,
                    "generation_after": after_lost_ack.generation,
                    "absence_is_not_non_commit": True,
                    "blind_retry_performed": False,
                },
            )
        )
        if not (no_second_mutation and lost_ack_observation_ok):
            raise AssertionError(
                "lost-ack reconciliation did not produce a bounded observation"
            )
        point_generation = create_setup(point_resource, "point-in-time")
        point_request = request(
            "point-in-time-target",
            point_generation,
            execution="execution-gcs-point-in-time",
            key="idempotency-gcs-point-in-time",
            body=b"sol-atlas:gcs:point-in-time:v1",
        )
        before = point_resource.reconcile(point_request)
        applied_point = point_resource.apply(point_request)
        after = point_resource.reconcile(point_request)
        point_ok = (
            before == "ObservedDifferentRequest"
            and applied_point == "Applied"
            and after == "ObservedAppliedSameRequest"
        )
        cases.append(
            record(
                "point_in_time_semantics_explicit",
                point_ok,
                {
                    "before_apply": before,
                    "mutation": applied_point,
                    "after_apply": after,
                },
            )
        )
        if not point_ok:
            raise AssertionError("point-in-time reconciliation semantics changed")

        if [case["id"] for case in cases] != case_ids(case_set):
            raise AssertionError("case-set execution order drifted")

        evidence = {
            "case_set": EXPECTED_CASE_SET,
            "case_set_digest": digest(case_set),
            "cases": cases,
            "all_passed": all(bool(case["passed"]) for case in cases),
            "cleanup_succeeded": False,
        }
        return {
            "schema": SCHEMA,
            "wif_verification": wif_verification,
            "wif_verification_digest": digest(wif_verification),
            "github_execution_context": github_context,
            "github_execution_context_digest": digest(github_context),
            "checked_out_source_commit": source_commit,
            "status": "qualified",
            "service": "Google Cloud Storage",
            "adapter_id": ADAPTER_ID,
            "adapter_revision": git_sha(ADAPTER_PATH),
            "harness_id": HARNESS_ID,
            "harness_revision": git_sha(HARNESS_PATH),
            "workflow_path": WORKFLOW_PATH,
            "workflow_revision": git_sha(WORKFLOW_PATH),
            "source_commit": git_head(),
            "profile": profile(),
            "profile_digest": profile_digest(profile()),
            "evidence_digest": digest(evidence),
            "case_set": EXPECTED_CASE_SET,
            "case_set_digest": digest(case_set),
            "case_set_path": CASE_SET_PATH,
            "object_names": [main_name, point_name],
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "evidence": evidence,
        }
    finally:
        cleanup_errors = []
        for target in (resource, point_resource, race_resource):
            try:
                state = target.state()
                if state is not None:
                    result = target.delete(state.generation)
                    if not 200 <= result.status < 300:
                        cleanup_errors.append(
                            f"{target.object_name}: HTTP {result.status}"
                        )
            except Exception as exc:
                cleanup_errors.append(f"{target.object_name}: {exc}")
        if cleanup_errors:
            raise RuntimeError(
                "qualification cleanup failed: " + "; ".join(cleanup_errors)
            )


def finalize_report(report: dict[str, object]) -> dict[str, object]:
    evidence = dict(report["evidence"])
    evidence["cleanup_succeeded"] = True
    evidence["all_passed"] = all(bool(case["passed"]) for case in evidence["cases"])
    report["evidence"] = evidence
    report["evidence_digest"] = digest(evidence)
    report["status"] = "qualified" if evidence["all_passed"] else "unqualified"
    report["report_digest"] = digest(report)
    return report


def write_report(path: str, report: dict[str, object]) -> None:
    destination = Path(path)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def verify_report(path: str) -> None:
    report = json.loads(Path(path).read_text(encoding="utf-8"))
    expected_case_set = load_case_set()
    if report.get("schema") != SCHEMA:
        raise AssertionError("wrong report schema")
    if report.get("status") != "qualified":
        raise AssertionError("report is not qualified")
    if report.get("case_set") != EXPECTED_CASE_SET:
        raise AssertionError("wrong case-set identity")
    if report.get("case_set_path") != CASE_SET_PATH:
        raise AssertionError("wrong case-set path")
    expected_case_set_digest = digest(expected_case_set)
    if report.get("case_set_digest") != expected_case_set_digest:
        raise AssertionError("case-set digest mismatch")
    wif_verification = report.get("wif_verification")
    if not isinstance(wif_verification, dict):
        raise AssertionError("missing WIF trust verification")
    github_context = report.get("github_execution_context")
    if not isinstance(github_context, dict):
        raise AssertionError("missing GitHub execution context")
    expected_context = {
        "repository": "Luminous-Dynamics/sol-atlas-leptos",
        "repository_id": "1195997641",
        "environment": "sol-atlas-gcs-qualification",
        "workflow": "Qualify GCS external effect",
        "event": "workflow_dispatch",
    }
    for name, expected_value in expected_context.items():
        if github_context.get(name) != expected_value:
            raise AssertionError(
                f"GitHub execution context mismatch for {name}"
            )
    if report.get("github_execution_context_digest") != digest(github_context):
        raise AssertionError("GitHub execution context digest mismatch")
    checked_out_source = report.get("checked_out_source_commit")
    if checked_out_source != github_context.get("sha"):
        raise AssertionError("checked-out source does not match GITHUB_SHA")
    if checked_out_source != git_head():
        raise AssertionError("checked-out source drift detected")
    if report.get("wif_verification_digest") != digest(wif_verification):
        raise AssertionError("WIF verification digest mismatch")
    if report.get("adapter_revision") != git_sha(ADAPTER_PATH):
        raise AssertionError("adapter revision drift detected")
    if report.get("harness_revision") != git_sha(HARNESS_PATH):
        raise AssertionError("harness revision drift detected")
    if report.get("workflow_revision") != git_sha(WORKFLOW_PATH):
        raise AssertionError("workflow revision drift detected")
    if report.get("source_commit") != git_head():
        raise AssertionError("source commit drift detected")

    expected_profile = profile()
    if report.get("profile") != expected_profile:
        raise AssertionError("profile drift detected")
    if report.get("profile_digest") != profile_digest(expected_profile):
        raise AssertionError("profile digest mismatch")

    evidence = report.get("evidence")
    if not isinstance(evidence, dict):
        raise AssertionError("missing evidence")
    expected_ids = case_ids(expected_case_set)
    if [case.get("id") for case in evidence.get("cases", [])] != expected_ids:
        raise AssertionError("case-set vector mismatch")
    if evidence.get("case_set") != EXPECTED_CASE_SET:
        raise AssertionError("evidence case-set identity mismatch")
    if evidence.get("case_set_digest") != expected_case_set_digest:
        raise AssertionError("evidence case-set digest mismatch")
    if not evidence.get("cleanup_succeeded"):
        raise AssertionError("cleanup was not successful")
    if not evidence.get("all_passed"):
        raise AssertionError("one or more qualification cases failed")
    if report.get("evidence_digest") != digest(evidence):
        raise AssertionError("evidence digest mismatch")

    report_digest = report.pop("report_digest", None)
    if report_digest != digest(report):
        raise AssertionError("report digest mismatch")

    print("verified: exact source, profile, case set, evidence, and report")


def main() -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("validate-case-set")
    qualify = sub.add_parser("qualify")
    qualify.add_argument("--bucket", required=True)
    qualify.add_argument("--object-prefix", default="sol-atlas/qualification")
    qualify.add_argument("--wif-verification", required=True)
    qualify.add_argument("--output", required=True)
    verify = sub.add_parser("verify")
    verify.add_argument("--report", required=True)
    args = parser.parse_args()

    if args.command == "validate-case-set":
        validate_case_set()
        return 0

    if args.command == "verify":
        verify_report(args.report)
        return 0

    report = run_qualification(
        args.bucket,
        args.object_prefix,
        args.wif_verification,
    )
    report = finalize_report(report)
    write_report(args.output, report)
    verify_report(args.output)
    print(
        "QUALIFIED "
        + report["adapter_id"]
        + " "
        + report["source_commit"]
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
