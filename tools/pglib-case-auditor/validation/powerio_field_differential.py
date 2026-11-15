#!/usr/bin/env python3
"""Field-by-field MATPOWER differential against the pinned PowerIO parser.

The production Sol-Atlas importer emits a source-bound projection receipt.
PowerIO parses the same original file independently. This script compares the
supported MATPOWER v2 columns, re-aggregating PowerIO's normalized load/shunt
tables back onto bus rows. It is validation glue only; it is not a production
parser and never repairs either result.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any

import powerio

EXPECTED_POWERIO_VERSION = "0.12.1"
REL_TOL = 1.0e-10
ABS_TOL = 1.0e-9


class Comparison:
    def __init__(self) -> None:
        self.mismatches: list[dict[str, Any]] = []
        self.comparisons = 0
        self.unlimited_rating_equivalences = 0

    def fail(self, field: str, expected: Any, actual: Any, note: str = "") -> None:
        self.mismatches.append(
            {"field": field, "sol_atlas": expected, "powerio": actual, "note": note}
        )

    def equal(self, field: str, expected: Any, actual: Any) -> None:
        self.comparisons += 1
        if expected != actual:
            self.fail(field, expected, actual)

    def number(self, field: str, expected: float, actual: float) -> None:
        self.comparisons += 1
        if not math.isfinite(float(expected)) or not math.isfinite(float(actual)):
            if expected == actual:
                return
            self.fail(field, expected, actual)
            return
        if not math.isclose(
            float(expected), float(actual), rel_tol=REL_TOL, abs_tol=ABS_TOL
        ):
            self.fail(field, expected, actual)

    def rating(self, field: str, expected: float, actual: float) -> None:
        self.comparisons += 1
        # MATPOWER uses 0 in RATE_A/B/C to mean "unlimited". PowerIO's typed
        # network uses +Inf for that semantic. This is the one intentional
        # representation equivalence; negative infinity and NaN never pass.
        if expected == 0.0 and math.isinf(float(actual)) and actual > 0:
            self.unlimited_rating_equivalences += 1
            return
        if not math.isclose(
            float(expected), float(actual), rel_tol=REL_TOL, abs_tol=ABS_TOL
        ):
            self.fail(field, expected, actual)


def git_blob_sha1(source: bytes) -> str:
    header = f"blob {len(source)}\0".encode("ascii")
    return hashlib.sha1(header + source).hexdigest()


def projection_for(case_path: Path, binary: str) -> dict[str, Any]:
    completed = subprocess.run(
        [binary, str(case_path)],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(
            f"Sol-Atlas projection failed for {case_path.name}: "
            f"{completed.stderr.strip()}"
        )
    try:
        value = json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(
            f"Sol-Atlas projection emitted invalid JSON for {case_path.name}: {error}"
        ) from error
    if value.get("schema_version") != "matpower-dc-projection.v0.1.0":
        raise RuntimeError("unexpected Sol-Atlas projection schema version")
    return value


def bus_aggregates(
    network: powerio.BalancedNetwork,
) -> tuple[dict[int, tuple[float, float]], dict[int, tuple[float, float]]]:
    loads: dict[int, list[float]] = defaultdict(lambda: [0.0, 0.0])
    for row in network.loads:
        if row["in_service"]:
            loads[int(row["bus"])][0] += float(row["p"])
            loads[int(row["bus"])][1] += float(row["q"])

    shunts: dict[int, list[float]] = defaultdict(lambda: [0.0, 0.0])
    for row in network.shunts:
        if row["in_service"]:
            shunts[int(row["bus"])][0] += float(row["g"])
            shunts[int(row["bus"])][1] += float(row["b"])

    return (
        {key: (value[0], value[1]) for key, value in loads.items()},
        {key: (value[0], value[1]) for key, value in shunts.items()},
    )


def compare_case(case_path: Path, binary: str) -> dict[str, Any]:
    source = case_path.read_bytes()
    digest = git_blob_sha1(source)
    expected = projection_for(case_path, binary)
    comparison = Comparison()

    comparison.equal("source_git_blob_sha1", digest, expected["source_git_blob_sha1"])
    comparison.number("base_mva", float(expected["base_mva"]), 0.0) if False else None

    module = powerio.parse(case_path, format="matpower")
    network = module.value
    if not isinstance(network, powerio.BalancedNetwork):
        raise RuntimeError(
            f"PowerIO returned {type(network).__name__}, not BalancedNetwork"
        )

    version = str(getattr(powerio, "__version__", "unknown"))
    if version != EXPECTED_POWERIO_VERSION:
        raise RuntimeError(
            f"PowerIO version drift: expected {EXPECTED_POWERIO_VERSION}, got {version}"
        )

    comparison.number("base_mva", float(expected["base_mva"]), float(network.base_mva))
    comparison.equal("bus_count", len(expected["buses"]), len(network.buses))
    comparison.equal(
        "generator_count", len(expected["generators"]), len(network.generators)
    )
    comparison.equal("branch_count", len(expected["branches"]), len(network.branches))
    comparison.equal("cost_row_count", len(expected["costs"]), len(network.generators))

    loads, shunts = bus_aggregates(network)

    for index, (want, got) in enumerate(zip(expected["buses"], network.buses)):
        prefix = f"bus[{index}]"
        comparison.equal(f"{prefix}.id", int(want["id"]), int(got["id"]))
        kinds = {"PQ": 1, "PV": 2, "REF": 3, "ISOLATED": 4}
        actual_kind = str(got["kind"]).upper()
        comparison.equal(
            f"{prefix}.bus_type", int(want["bus_type"]), kinds.get(actual_kind, -1)
        )
        for field, key in [
            ("area", "area"),
            ("zone", "zone"),
        ]:
            comparison.equal(f"{prefix}.{field}", int(want[field]), int(got[key]))
        for field, key in [
            ("voltage_magnitude_initial_pu", "vm"),
            ("voltage_angle_initial_degrees", "va"),
            ("base_kv", "base_kv"),
            ("voltage_max_pu", "vmax"),
            ("voltage_min_pu", "vmin"),
        ]:
            comparison.number(
                f"{prefix}.{field}", float(want[field]), float(got[key])
            )

        bus_id = int(want["id"])
        load_p, load_q = loads.get(bus_id, (0.0, 0.0))
        shunt_g, shunt_b = shunts.get(bus_id, (0.0, 0.0))
        comparison.number(
            f"{prefix}.demand_mw", float(want["demand_mw"]), load_p
        )
        comparison.number(
            f"{prefix}.reactive_demand_mvar",
            float(want["reactive_demand_mvar"]),
            load_q,
        )
        comparison.number(
            f"{prefix}.shunt_conductance_mw",
            float(want["shunt_conductance_mw"]),
            shunt_g,
        )
        comparison.number(
            f"{prefix}.shunt_susceptance_mvar",
            float(want["shunt_susceptance_mvar"]),
            shunt_b,
        )

        online = [
            generator
            for generator in network.generators
            if int(generator["bus"]) == bus_id and generator["in_service"]
        ]
        aggregate_pg = sum(float(row["pg"]) for row in online)
        aggregate_pmin = sum(float(row["pmin"]) for row in online)
        aggregate_pmax = sum(float(row["pmax"]) for row in online)
        comparison.number(
            f"{prefix}.online_generation_mw",
            float(want["online_generation_mw"]),
            aggregate_pg,
        )
        comparison.number(
            f"{prefix}.online_generation_p_min_mw",
            float(want["online_generation_p_min_mw"]),
            aggregate_pmin,
        )
        comparison.number(
            f"{prefix}.online_generation_p_max_mw",
            float(want["online_generation_p_max_mw"]),
            aggregate_pmax,
        )
        actual_injection = aggregate_pg - load_p
        specified = want["specified_injection_mw"]
        if specified is None:
            comparison.equal(f"{prefix}.reference_slack", True, int(want["bus_type"]) == 3)
        else:
            comparison.number(
                f"{prefix}.specified_injection_mw",
                float(specified),
                actual_injection,
            )

    for index, (want, got) in enumerate(zip(expected["generators"], network.generators)):
        prefix = f"generator[{index}]"
        for field, key in [
            ("bus_id", "bus"),
        ]:
            comparison.equal(f"{prefix}.{field}", int(want[field]), int(got[key]))
        for field, key in [
            ("pg_mw", "pg"),
            ("qg_mvar", "qg"),
            ("q_max_mvar", "qmax"),
            ("q_min_mvar", "qmin"),
            ("voltage_setpoint_pu", "vg"),
            ("m_base_mva", "mbase"),
            ("p_max_mw", "pmax"),
            ("p_min_mw", "pmin"),
        ]:
            comparison.number(f"{prefix}.{field}", float(want[field]), float(got[key]))
        comparison.equal(
            f"{prefix}.in_service", bool(want["in_service"]), bool(got["in_service"])
        )

        cost = got.get("cost")
        if cost is None:
            comparison.fail(f"{prefix}.cost", want, None, "PowerIO cost row is absent")
        else:
            comparison.equal(
                f"{prefix}.cost.model", int(want["model"]), int(cost["model"])
            )
            comparison.number(
                f"{prefix}.cost.startup_cost",
                float(want["startup_cost"]),
                float(cost["startup"]),
            )
            comparison.number(
                f"{prefix}.cost.shutdown_cost",
                float(want["shutdown_cost"]),
                float(cost["shutdown"]),
            )
            comparison.equal(
                f"{prefix}.cost.point_or_coefficient_count",
                int(want["point_or_coefficient_count"]),
                int(cost["ncost"]),
            )
            comparison.equal(
                f"{prefix}.cost.coefficients",
                list(want["coefficients"]),
                list(cost["coeffs"]),
            )

    for index, (want, got) in enumerate(zip(expected["branches"], network.branches)):
        prefix = f"branch[{index}]"
        for field, key in [
            ("from_bus", "from_id"),
            ("to_bus", "to_id"),
        ]:
            comparison.equal(f"{prefix}.{field}", int(want[field]), int(got[key]))
        for field, key in [
            ("resistance_pu", "r"),
            ("reactance_pu", "x"),
            ("line_charging_susceptance_pu", "b"),
            ("tap_ratio", "tap"),
            ("phase_shift_degrees", "shift"),
            ("angle_min_degrees", "angmin"),
            ("angle_max_degrees", "angmax"),
        ]:
            comparison.number(f"{prefix}.{field}", float(want[field]), float(got[key]))
        for field, key in [
            ("rate_a_mva", "rate_a"),
            ("rate_b_mva", "rate_b"),
            ("rate_c_mva", "rate_c"),
        ]:
            comparison.rating(f"{prefix}.{field}", float(want[field]), float(got[key]))
        comparison.equal(
            f"{prefix}.in_service", bool(want["in_service"]), bool(got["in_service"])
        )

    diagnostics = powerio.diagnostic_records(module.diagnostics)
    errors = [
        finding
        for finding in diagnostics
        if str(finding.get("severity", "")).lower() == "error"
    ]

    return {
        "case": case_path.name,
        "powerio_version": version,
        "source_git_blob_sha1": digest,
        "counts": {
            "buses": len(network.buses),
            "generators": len(network.generators),
            "branches": len(network.branches),
            "loads": len(network.loads),
            "shunts": len(network.shunts),
            "cost_rows": len(network.generators),
        },
        "field_comparisons": comparison.comparisons,
        "unlimited_rating_semantic_equivalences": comparison.unlimited_rating_equivalences,
        "powerio_diagnostics": diagnostics,
        "powerio_error_diagnostic_count": len(errors),
        "mismatches": comparison.mismatches,
        "passed": not comparison.mismatches and not errors,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("cases", nargs="+", type=Path)
    parser.add_argument(
        "--projection-binary",
        default=os.environ.get("PGLIB_PROJECTION_BIN"),
        help="path to the already-built matpower-projection CLI",
    )
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    if not args.projection_binary:
        parser.error("--projection-binary or PGLIB_PROJECTION_BIN is required")

    results = [compare_case(path, args.projection_binary) for path in args.cases]
    report = {
        "schema_version": "sol-atlas-powerio-differential.v0.1.0",
        "powerio_version": EXPECTED_POWERIO_VERSION,
        "cases": results,
        "passed": all(result["passed"] for result in results),
    }
    rendered = json.dumps(report, indent=2, sort_keys=True, allow_nan=False) + "\n"
    if args.output:
        args.output.write_text(rendered, encoding="utf-8")
    print(rendered, end="")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
