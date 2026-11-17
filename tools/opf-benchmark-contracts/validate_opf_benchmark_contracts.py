#!/usr/bin/env python3
"""CI-only schema and semantic checks for source-bound OPF benchmark contracts.

This validator does not execute a solver or issue an engineering qualification.
It checks that manifests and run receipts preserve identity and do not overclaim.
"""
from __future__ import annotations

import copy
import json
from pathlib import Path
import sys
from typing import Any

from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[2]
MANIFEST_SCHEMA = ROOT / "docs/schemas/opf-benchmark/benchmark-manifest-v0.1.schema.json"
RECEIPT_SCHEMA = ROOT / "docs/schemas/opf-benchmark/benchmark-run-receipt-v0.1.schema.json"
MANIFEST_FIXTURE = ROOT / "docs/research/fixtures/opf-benchmark/pglib-v23.07-typ-starter-manifest-v0.1.json"
RECEIPT_FIXTURE = ROOT / "docs/research/fixtures/opf-benchmark/benchmark-run-not-run-fixture-v0.1.json"


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ValueError(f"{path.relative_to(ROOT)}: cannot load JSON: {exc}") from exc


def schema_validate(schema_path: Path, instance_path: Path) -> dict[str, Any]:
    schema = load_json(schema_path)
    instance = load_json(instance_path)
    Draft202012Validator.check_schema(schema)
    validator = Draft202012Validator(schema, format_checker=FormatChecker())
    errors = sorted(
        validator.iter_errors(instance),
        key=lambda error: (list(map(str, error.absolute_path)), error.message),
    )
    if errors:
        rendered = [
            f"{instance_path.relative_to(ROOT)} at /{'/'.join(map(str, error.absolute_path))}: {error.message}"
            for error in errors
        ]
        raise ValueError("\n".join(rendered))
    print(f"PASS schema: {schema_path.relative_to(ROOT)}")
    print(f"PASS fixture: {instance_path.relative_to(ROOT)}")
    return instance


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def check_manifest(manifest: dict[str, Any]) -> None:
    cases = manifest["cases"]
    case_ids = [case["case_id"] for case in cases]
    require(len(case_ids) == len(set(case_ids)), "manifest contains duplicate case_id values")
    require(len(cases) == 3, "v0.1 TYP starter manifest must contain exactly the declared three-case panel")

    profiles = {profile["profile_id"]: profile for profile in manifest["formulations"]}
    for case in cases:
        case_path = ROOT / case["path"]
        require(case_path.is_file(), f"manifest case source does not exist in this repository head: {case['path']}")
        profile_ids = set(case["supported_profiles"])
        for profile_id in profile_ids:
            require(profile_id in profiles, f"case {case['case_id']} references unknown profile {profile_id}")
            require(
                profiles[profile_id]["status"] == "supported_reference_profile",
                f"case {case['case_id']} treats non-supported profile {profile_id} as supported",
            )
        # This starter-panel check binds the manifest to the exact fixture bytes.
        from hashlib import sha1

        data = case_path.read_bytes()
        header = f"blob {len(data)}\0".encode("ascii")
        import hashlib
        digest = hashlib.sha1(header + data).hexdigest()
        require(
            digest == case["source_git_blob_sha1"],
            f"case {case['case_id']} source Git blob identity mismatch: {digest}",
        )
        baseline = case["published_baseline_context"]
        require(
            baseline["comparison_role"] == "context_only_not_a_matpower_makeBdc_or_dc-mp_parity_target",
            f"case {case['case_id']} historical PGLib baseline is incorrectly marked parity-eligible",
        )

    evidence = manifest["observed_evidence_snapshot"]["evidence"]
    require(evidence, "manifest evidence snapshot is empty")
    require(
        all(record["conclusion"] == "success" for record in evidence),
        "manifest observed-evidence snapshot contains a non-success run",
    )


def check_receipt_semantics(manifest: dict[str, Any], receipt: dict[str, Any]) -> None:
    by_case = {case["case_id"]: case for case in manifest["cases"]}
    case = by_case.get(receipt["case_id"])
    require(case is not None, f"receipt references case outside the manifest: {receipt['case_id']}")
    require(
        receipt["manifest_id"] == manifest["manifest_id"],
        "receipt manifest_id does not match the exact manifest identity",
    )
    require(
        receipt["source_git_blob_sha1"] == case["source_git_blob_sha1"],
        "receipt source Git blob SHA-1 does not match the manifest case identity",
    )
    require(
        receipt["formulation"]["profile_id"] in case["supported_profiles"],
        "receipt uses a formulation not admitted for the referenced case",
    )

    execution = receipt["execution"]
    outcome = receipt["outcome"]
    qualification = receipt["qualification"]
    claim = receipt["claim_ceiling"]
    timings = execution["timings_ms"]
    run_status = execution["run_status"]

    if run_status == "not_run":
        require(outcome["solver_outcome"] == "unknown", "not_run receipt must not claim a solver outcome")
        require(qualification["disposition"] == "not_run", "not_run execution must have not_run qualification")
        require(qualification["method"] == "none", "not_run receipt cannot name a verification method")
        require(qualification["artifact"]["state"] == "absent", "not_run receipt cannot contain a verification artifact")
        require(all(value == 0 for value in timings.values()), "not_run receipt must have zero timings")
        require("objective_value" not in outcome, "not_run receipt must not contain an objective value")
        require("parameters_sha256" not in receipt["solver"], "not_run receipt must not invent a parameter digest")
        require(claim == "structural_only", "not_run fixture must retain a structural-only claim ceiling")
        return

    require(run_status == "completed" or qualification["disposition"] != "pass",
            "failed/interrupted execution cannot carry passing qualification")
    if run_status == "completed":
        require(execution["exit_code"] == 0, "completed execution must have exit_code=0")
        require("parameters_sha256" in receipt["solver"], "completed run must bind exact solver parameters")
        require(timings["end_to_end"] > 0, "completed run must report positive end-to-end time")
        staged = timings["model_build"] + timings["preprocess"] + timings["solve"] + timings["verify"]
        require(
            timings["end_to_end"] + 1e-9 >= staged,
            "end_to_end timing is smaller than the sum of declared stages",
        )

    if qualification["disposition"] == "pass":
        require(run_status == "completed", "passing qualification requires a completed execution")
        require(qualification["independent"], "a passing qualification must use an independent verifier")
        require(qualification["artifact"]["state"] == "present", "passing qualification requires an immutable evidence artifact")
        require(outcome["solver_outcome"] in {"optimal", "feasible"}, "solver outcome is not a feasible result")
        require({"source_identity", "formulation_identity"}.issubset(set(qualification["checks"])),
                "passing qualification must bind source and formulation identity")

    checks = set(qualification["checks"])
    method = qualification["method"]
    if claim == "numerical_feasibility":
        require(method in {"primal_audit", "numerical_kkt"}, "numerical feasibility claim requires a primal verifier")
        require("primal_feasibility" in checks, "numerical feasibility claim lacks primal_feasibility check")
    elif claim == "numerical_kkt":
        required = {
            "source_identity", "formulation_identity", "primal_feasibility",
            "dual_feasibility", "stationarity", "complementary_slackness", "duality_gap",
        }
        require(method == "numerical_kkt", "numerical KKT ceiling requires numerical_kkt method")
        require(required.issubset(checks), f"numerical KKT receipt lacks checks: {sorted(required - checks)}")
    elif claim == "exact_optimality":
        required = {
            "source_identity", "formulation_identity", "primal_feasibility",
            "dual_feasibility", "stationarity", "complementary_slackness",
            "duality_gap", "exact_optimality",
        }
        require(method in {"exact_rational", "interval_bound"}, "exact optimality ceiling requires exact or rigorous interval method")
        require(required.issubset(checks), f"exact optimality receipt lacks checks: {sorted(required - checks)}")
    elif claim == "relaxation_bound":
        require(method == "interval_bound", "relaxation-bound ceiling requires an interval/bound verifier")
        require({"source_identity", "formulation_identity", "duality_gap"}.issubset(checks),
                "relaxation-bound receipt lacks source/formulation/gap checks")
    elif claim == "physical_validation":
        target = receipt.get("target")
        require(target is not None, "physical validation claim requires exact target asset identity")
        require(
            method in {"laboratory_measurement", "hardware_in_loop", "field_observation"},
            "physical validation requires laboratory, HIL or field observation evidence",
        )
        require(
            {"asset_configuration_identity", "measurement_correlation"}.issubset(checks),
            "physical validation claim lacks configuration identity or measurement correlation",
        )
        require(qualification["artifact"]["state"] == "present", "physical validation requires an evidence artifact")


def expect_rejection(label: str, action: Any) -> None:
    try:
        action()
    except ValueError:
        print(f"PASS negative control: {label}")
        return
    raise ValueError(f"negative control was incorrectly accepted: {label}")


def check_negative_mutations(manifest: dict[str, Any], receipt: dict[str, Any]) -> None:
    bad_digest = copy.deepcopy(receipt)
    bad_digest["source_git_blob_sha1"] = "0" * 40
    expect_rejection("source digest mismatch", lambda: check_receipt_semantics(manifest, bad_digest))

    bad_profile = copy.deepcopy(receipt)
    bad_profile["formulation"]["profile_id"] = "powermodels_dcp"
    expect_rejection("unsupported or diagnostic-only formulation", lambda: check_receipt_semantics(manifest, bad_profile))

    false_pass = copy.deepcopy(receipt)
    false_pass["execution"]["run_status"] = "completed"
    false_pass["execution"]["exit_code"] = 0
    false_pass["execution"]["timings_ms"] = {
        "model_build": 1, "preprocess": 1, "solve": 1, "verify": 1, "end_to_end": 4
    }
    false_pass["solver"]["parameters_sha256"] = "a" * 64
    false_pass["qualification"]["disposition"] = "pass"
    false_pass["qualification"]["independent"] = False
    false_pass["qualification"]["artifact"] = {"state": "present", "sha256": "b" * 64, "uri": "https://example.invalid/evidence.json"}
    false_pass["outcome"]["solver_outcome"] = "optimal"
    expect_rejection("non-independent verifier cannot pass a run", lambda: check_receipt_semantics(manifest, false_pass))

    missing_kkt_checks = copy.deepcopy(false_pass)
    missing_kkt_checks["qualification"]["independent"] = True
    missing_kkt_checks["qualification"]["method"] = "numerical_kkt"
    missing_kkt_checks["qualification"]["checks"] = ["source_identity", "formulation_identity", "primal_feasibility"]
    missing_kkt_checks["claim_ceiling"] = "numerical_kkt"
    expect_rejection("numerical KKT cannot pass without all KKT checks", lambda: check_receipt_semantics(manifest, missing_kkt_checks))

    numerical_as_exact = copy.deepcopy(false_pass)
    numerical_as_exact["qualification"]["independent"] = True
    numerical_as_exact["qualification"]["method"] = "numerical_kkt"
    numerical_as_exact["qualification"]["checks"] = [
        "source_identity", "formulation_identity", "primal_feasibility", "dual_feasibility",
        "stationarity", "complementary_slackness", "duality_gap", "exact_optimality",
    ]
    numerical_as_exact["claim_ceiling"] = "exact_optimality"
    expect_rejection("floating-point KKT cannot be promoted to exact optimality", lambda: check_receipt_semantics(manifest, numerical_as_exact))

    fabricated_not_run = copy.deepcopy(receipt)
    fabricated_not_run["qualification"]["disposition"] = "pass"
    expect_rejection("not-run fixture cannot become a passing result", lambda: check_receipt_semantics(manifest, fabricated_not_run))


def main() -> int:
    manifest = schema_validate(MANIFEST_SCHEMA, MANIFEST_FIXTURE)
    receipt = schema_validate(RECEIPT_SCHEMA, RECEIPT_FIXTURE)
    check_manifest(manifest)
    check_receipt_semantics(manifest, receipt)
    check_negative_mutations(manifest, receipt)
    print("PASS: OPF benchmark manifest, source identities and run-receipt semantics")
    print("Claim ceiling: contract validation only; the not-run fixture contains no solver result.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except ValueError as exc:
        print(f"OPF benchmark contract validation failed: {exc}", file=sys.stderr)
        sys.exit(1)
