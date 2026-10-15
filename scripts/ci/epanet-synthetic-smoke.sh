#!/usr/bin/env bash
set -euo pipefail
out="$PWD/artifacts/epanet-w0"
solver="$PWD/.epanet-build/bin/runepanet"
scenario_file="$PWD/sol-atlas-core/fixtures/water-engineering/w0-p2-diameter-restriction.synthetic.json"
scenario_file_sha="$(sha256sum "$scenario_file" | awk '{print $1}')"
input_rel="$(jq -er '.model_package.input_snapshot.path | strings' "$scenario_file")"
input="$PWD/$input_rel"
declared_input_sha="$(jq -er '.model_package.input_snapshot.sha256 | strings' "$scenario_file")"
declared_input_bytes="$(jq -er '.model_package.input_snapshot.byte_length | numbers' "$scenario_file")"
base_scenario_sha="$(jq -er '.scenario.base_input_snapshot_sha256 | strings' "$scenario_file")"
scenario_bundle_id="$(jq -er '.bundle_id | strings' "$scenario_file")"
scenario_id="$(jq -er '.scenario.scenario_id | strings' "$scenario_file")"
asset_id="$(jq -er '.scenario.changes[0].asset_id | strings' "$scenario_file")"
parameter="$(jq -er '.scenario.changes[0].parameter | strings' "$scenario_file")"
baseline_diameter="$(jq -er '.scenario.changes[0].baseline_value | numbers' "$scenario_file")"
candidate_diameter="$(jq -er '.scenario.changes[0].candidate_value | numbers' "$scenario_file")"
change_count="$(jq -er '.scenario.changes | length' "$scenario_file")"

# This runner intentionally accepts one simple synthetic restriction case,
# not arbitrary user-supplied paths, parameters, or a municipal model.
jq -e '
  (.classification == "synthetic_public") and
  (.model_package.input_snapshot.synthetic == true) and
  (.model_package.solver_profile.execution_state == "NotRun") and
  (.model_package.solver_profile.analysis_mode == "DemandDriven") and
  (.model_package.solver_profile.headloss_model == "HazenWilliams") and
  (.model_package.unit_system == "EPANET LPS (length m, diameter mm, flow L/s, head m)") and
  (.scenario.base_model_id == .model_package.model_id) and
  (.model_package.source_artifacts | length == 1) and
  (.model_package.input_snapshot.artifact_id == .model_package.source_artifacts[0].artifact_id) and
  (.model_package.source_artifacts[0].synthetic == true) and
  (.run_result.state == "NotRun") and
  (.scenario.changes | length == 1) and
  (.scenario.changes[0].parameter == "diameter_mm") and
  (.scenario.changes[0].unit == "mm") and
  (.scenario.changes[0].baseline_value | type == "number") and
  (.scenario.changes[0].candidate_value | type == "number") and
  (.scenario.changes[0].candidate_value > 0) and
  (.scenario.changes[0].candidate_value < .scenario.changes[0].baseline_value) and
  (.scenario.base_input_snapshot_sha256 == .model_package.input_snapshot.sha256) and
  (.model_package.input_snapshot.path == .model_package.source_artifacts[0].path) and
  (.model_package.input_snapshot.sha256 == .model_package.source_artifacts[0].sha256) and
  (.model_package.input_snapshot.byte_length == .model_package.source_artifacts[0].byte_length) and
  (.scenario.changes[0].asset_id as $asset | any(.model_package.asset_index[]; .asset_id == $asset and .asset_type == "pipe"))
' "$scenario_file" >/dev/null || {
  echo "SCENARIO_BUNDLE_CONTRACT_FAILED: runner requires a single internally consistent synthetic pipe-diameter restriction"
  exit 1
}
case "$input_rel" in
  sol-atlas-core/fixtures/water-engineering/*.inp) ;;
  *) echo "SCENARIO_INPUT_PATH_REJECTED: $input_rel"; exit 1 ;;
esac
case "$input_rel" in
  *".."*) echo "SCENARIO_INPUT_PATH_TRAVERSAL_REJECTED: $input_rel"; exit 1 ;;
esac
case "$asset_id" in
  ""|*[!A-Za-z0-9_-]*) echo "SCENARIO_ASSET_ID_REJECTED: $asset_id"; exit 1 ;;
esac
test "$parameter" = "diameter_mm"
test "$change_count" = "1"
test "$declared_input_sha" = "$base_scenario_sha"
grep -Eq '^[[:space:]]*UNITS[[:space:]]+LPS([[:space:]]|$)' "$input" || {
  echo "SCENARIO_INPUT_UNIT_SYSTEM_MISMATCH: expected LPS"
  exit 1
}
grep -Eq '^[[:space:]]*HEADLOSS[[:space:]]+H-W([[:space:]]|$)' "$input" || {
  echo "SCENARIO_INPUT_HEADLOSS_MODEL_MISMATCH: expected Hazen-Williams"
  exit 1
}
actual_input_sha="$(sha256sum "$input" | awk '{print $1}')"
actual_input_bytes="$(wc -c < "$input" | tr -d '[:space:]')"
test "$actual_input_sha" = "$declared_input_sha" || {
  echo "SCENARIO_INPUT_SHA256_MISMATCH: expected=$declared_input_sha actual=$actual_input_sha"
  exit 1
}
test "$actual_input_bytes" = "$declared_input_bytes" || {
  echo "SCENARIO_INPUT_BYTE_LENGTH_MISMATCH: expected=$declared_input_bytes actual=$actual_input_bytes"
  exit 1
}
started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
cp "$input" "$out/w0-baseline.inp"

# Mutate only the declared asset's diameter field and require an exact
# single match with the scenario's declared baseline value.
awk -v asset="$asset_id" -v baseline="$baseline_diameter" -v candidate="$candidate_diameter" '
  $1 == asset && $1 !~ /^;/ {
    if (($5 + 0) != (baseline + 0)) {
      printf "declared baseline mismatch for %s: input=%s scenario=%s\n", asset, $5, baseline > "/dev/stderr"
      exit 2
    }
    $5 = candidate
    changed++
  }
  { print }
  END {
    if (changed != 1) {
      printf "expected exactly one target-pipe diameter mutation; changed=%d\n", changed > "/dev/stderr"
      exit 1
    }
  }
' "$input" > "$out/w0-candidate.inp" || exit 1

awk -v asset="$asset_id" -v expected="$candidate_diameter" '$1 == asset && $1 !~ /^;/ && ($5 + 0) == (expected + 0) { found++ } END { exit !(found == 1) }' "$out/w0-candidate.inp" || {
  echo "SCENARIO_CANDIDATE_MUTATION_VERIFICATION_FAILED"
  exit 1
}
case "$scenario_id" in
  ""|*[!A-Za-z0-9._-]*) echo "SCENARIO_ID_REJECTED: $scenario_id"; exit 1 ;;
esac
candidate_stem="$out/$scenario_id-candidate"
candidate_input="$candidate_stem.inp"
candidate_report="$candidate_stem.rpt"
candidate_binary="$candidate_stem.bin"
candidate_stdout="$candidate_stem.stdout"
cp "$out/w0-candidate.inp" "$candidate_input"

"$solver" "$out/w0-baseline.inp" "$out/w0-baseline.rpt" "$out/w0-baseline.bin" > "$out/w0-baseline.stdout" 2>&1
"$solver" "$candidate_input" "$candidate_report" "$candidate_binary" > "$candidate_stdout" 2>&1

grep -q "EPANET ran successfully" "$out/w0-baseline.stdout" || {
  echo "BASELINE_EPANET_FAILED: solver stdout and report follow"
  cat "$out/w0-baseline.stdout"
  cat "$out/w0-baseline.rpt" 2>/dev/null || true
  exit 1
}
grep -q "EPANET ran successfully" "$candidate_stdout" || {
  echo "CANDIDATE_EPANET_FAILED: solver stdout and report follow"
  cat "$candidate_stdout"
  cat "$candidate_report" 2>/dev/null || true
  exit 1
}
test -s "$out/w0-baseline.rpt"
test -s "$out/w0-baseline.bin"
test -s "$candidate_report"
test -s "$candidate_binary"
grep -q "$asset_id" "$out/w0-baseline.rpt" || {
  echo "BASELINE_REPORT_MISSING_TARGET_ASSET: $asset_id"
  cat "$out/w0-baseline.rpt"
  exit 1
}
grep -q "$asset_id" "$candidate_report" || {
  echo "CANDIDATE_REPORT_MISSING_TARGET_ASSET: $asset_id"
  cat "$candidate_report"
  exit 1
}
# Parse hydraulic rows only inside EPANET's result sections; an asset
# appearing in the input summary is not proof of a computed result.
downstream_node="$(awk -v asset="$asset_id" '
  /^\[PIPES\]/ { in_pipes = 1; next }
  /^\[/ { in_pipes = 0 }
  in_pipes && $1 == asset { print $3; found++ }
  END { if (found != 1) exit 1 }
' "$input")" || {
  echo "SCENARIO_PIPE_ENDPOINT_LOOKUP_FAILED: $asset_id"
  exit 1
}
case "$downstream_node" in
  ""|*[!A-Za-z0-9_-]*) echo "SCENARIO_DOWNSTREAM_NODE_REJECTED: $downstream_node"; exit 1 ;;
esac
downstream_degree="$(awk -v node="$downstream_node" '
  /^\[PIPES\]/ { in_pipes = 1; next }
  /^\[/ { in_pipes = 0 }
  in_pipes && NF > 0 && $1 !~ /^;/ && ($2 == node || $3 == node) { degree++ }
  END { print degree + 0 }
' "$input")"
test "$downstream_degree" = "1" || {
  echo "SCENARIO_TERMINAL_NODE_TOPOLOGY_UNSUPPORTED: node=$downstream_node degree=$downstream_degree"
  exit 1
}
downstream_demand_lps="$(awk -v node="$downstream_node" '
  /^\[JUNCTIONS\]/ { in_junctions = 1; next }
  /^\[/ { in_junctions = 0 }
  in_junctions && NF > 0 && $1 !~ /^;/ && $1 == node { print $3; found++ }
  END { if (found != 1) exit 1 }
' "$input")" || {
  echo "SCENARIO_DOWNSTREAM_JUNCTION_DEMAND_MISSING: $downstream_node"
  exit 1
}
total_junction_demand_lps="$(awk '
  /^\[JUNCTIONS\]/ { in_junctions = 1; next }
  /^\[/ { in_junctions = 0 }
  in_junctions && NF > 0 && $1 !~ /^;/ { total += $3; count++ }
  END { if (count < 1) exit 1; printf "%.6f\n", total }
' "$input")" || {
  echo "SCENARIO_TOTAL_JUNCTION_DEMAND_MISSING"
  exit 1
}
reservoir_id="$(awk '
  /^\[RESERVOIRS\]/ { in_reservoirs = 1; next }
  /^\[/ { in_reservoirs = 0 }
  in_reservoirs && NF > 0 && $1 !~ /^;/ { print $1; found++ }
  END { if (found != 1) exit 1 }
' "$input")" || {
  echo "SCENARIO_RESERVOIR_TOPOLOGY_UNSUPPORTED"
  exit 1
}
source_pipe_id="$(awk -v reservoir="$reservoir_id" '
  /^\[PIPES\]/ { in_pipes = 1; next }
  /^\[/ { in_pipes = 0 }
  in_pipes && NF > 0 && $1 !~ /^;/ && ($2 == reservoir || $3 == reservoir) { print $1; found++ }
  END { if (found != 1) exit 1 }
' "$input")" || {
  echo "SCENARIO_SOURCE_PIPE_TOPOLOGY_UNSUPPORTED: reservoir=$reservoir_id"
  exit 1
}

extract_link_results() {
  local report="$1"
  local requested_asset="$2"
  awk -v asset="$requested_asset" '
    /Link Results at / { in_links = 1; next }
    /Node Results at / { in_links = 0 }
    in_links && $1 == asset { print $2, $3, $4; found = 1; exit }
    END { if (!found) exit 1 }
  ' "$report"
}
extract_node_pressure() {
  local report="$1"
  awk -v node="$downstream_node" '
    /Node Results at / { in_nodes = 1; next }
    /Link Results at / { in_nodes = 0 }
    in_nodes && $1 == node { print $4; found = 1; exit }
    END { if (!found) exit 1 }
  ' "$report"
}

baseline_pipe_results="$(extract_link_results "$out/w0-baseline.rpt" "$asset_id")" || {
  echo "BASELINE_TARGET_PIPE_HYDRAULIC_ROW_MISSING: $asset_id"
  cat "$out/w0-baseline.rpt"
  exit 1
}
candidate_pipe_results="$(extract_link_results "$candidate_report" "$asset_id")" || {
  echo "CANDIDATE_TARGET_PIPE_HYDRAULIC_ROW_MISSING: $asset_id"
  cat "$candidate_report"
  exit 1
}
read -r baseline_pipe_flow baseline_pipe_velocity baseline_pipe_headloss <<< "$baseline_pipe_results"
read -r candidate_pipe_flow candidate_pipe_velocity candidate_pipe_headloss <<< "$candidate_pipe_results"

baseline_source_results="$(extract_link_results "$out/w0-baseline.rpt" "$source_pipe_id")" || {
  echo "BASELINE_SOURCE_PIPE_HYDRAULIC_ROW_MISSING: $source_pipe_id"
  cat "$out/w0-baseline.rpt"
  exit 1
}
candidate_source_results="$(extract_link_results "$candidate_report" "$source_pipe_id")" || {
  echo "CANDIDATE_SOURCE_PIPE_HYDRAULIC_ROW_MISSING: $source_pipe_id"
  cat "$candidate_report"
  exit 1
}
read -r baseline_source_flow _ _ <<< "$baseline_source_results"
read -r candidate_source_flow _ _ <<< "$candidate_source_results"

baseline_downstream_pressure="$(extract_node_pressure "$out/w0-baseline.rpt")" || {
  echo "BASELINE_DOWNSTREAM_PRESSURE_ROW_MISSING: $downstream_node"
  cat "$out/w0-baseline.rpt"
  exit 1
}
candidate_downstream_pressure="$(extract_node_pressure "$candidate_report")" || {
  echo "CANDIDATE_DOWNSTREAM_PRESSURE_ROW_MISSING: $downstream_node"
  cat "$candidate_report"
  exit 1
}

# Never let awk coerce an absent or malformed report field to zero.
# Reject missing, non-numeric, or placeholder/overflow values before
# running arithmetic assertions or writing an evidence manifest.
number_re='^-?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$'
for field_name in \
  baseline_source_flow candidate_source_flow \
  baseline_pipe_flow candidate_pipe_flow \
  baseline_pipe_velocity candidate_pipe_velocity \
  baseline_pipe_headloss candidate_pipe_headloss \
  baseline_downstream_pressure candidate_downstream_pressure \
  total_junction_demand_lps downstream_demand_lps; do
  field_value="${!field_name}"
  if [[ ! "$field_value" =~ $number_re ]]; then
    echo "REPORT_NUMERIC_FIELD_INVALID: field=$field_name value=[$field_value]"
    exit 1
  fi
done

printf '%s flow [L/s]: baseline=%s candidate=%s\n' "$asset_id" "$baseline_pipe_flow" "$candidate_pipe_flow"
printf '%s velocity [m/s]: baseline=%s candidate=%s\n' "$asset_id" "$baseline_pipe_velocity" "$candidate_pipe_velocity"
printf '%s headloss [m per 1000 m]: baseline=%s candidate=%s\n' "$asset_id" "$baseline_pipe_headloss" "$candidate_pipe_headloss"
printf '%s downstream pressure [m]: baseline=%s candidate=%s\n' "$downstream_node" "$baseline_downstream_pressure" "$candidate_downstream_pressure"

awk -v b="$baseline_pipe_flow" -v c="$candidate_pipe_flow" 'BEGIN { d = c - b; if (d < -0.02 || d > 0.02) exit 1 }' || {
  echo "FAIL: demand-driven flow in $asset_id should remain invariant within report precision"
  exit 1
}
for pair in \
  "baseline source $baseline_source_flow $total_junction_demand_lps" \
  "candidate source $candidate_source_flow $total_junction_demand_lps" \
  "baseline target $baseline_pipe_flow $downstream_demand_lps" \
  "candidate target $candidate_pipe_flow $downstream_demand_lps"; do
  read -r label phase observed expected <<< "$pair"
  awk -v observed="$observed" -v expected="$expected" 'BEGIN { delta = observed - expected; if (delta < -0.02 || delta > 0.02) exit 1 }' || {
    echo "FAIL: synthetic continuity check $phase/$label observed=$observed expected=$expected tolerance=0.02 L/s"
    exit 1
  }
done
awk -v b="$baseline_pipe_velocity" -v c="$candidate_pipe_velocity" 'BEGIN { exit !(c > b) }' || {
  echo "FAIL: reducing $asset_id diameter did not increase reported velocity"
  exit 1
}
awk -v b="$baseline_pipe_headloss" -v c="$candidate_pipe_headloss" 'BEGIN { exit !(c > b) }' || {
  echo "FAIL: reducing $asset_id diameter did not increase reported headloss"
  exit 1
}
awk -v b="$baseline_downstream_pressure" -v c="$candidate_downstream_pressure" 'BEGIN { exit !(c < b) }' || {
  echo "FAIL: restricting $asset_id did not reduce downstream $downstream_node pressure"
  exit 1
}
echo "EPANET_DIRECTIONAL_HYDRAULICS_PASS: flow invariant; velocity/headloss increase; downstream pressure decreases."

if cmp -s "$out/w0-baseline.rpt" "$candidate_report"; then
  echo "FAIL: diameter-restriction scenario did not change the EPANET report"
  exit 1
fi
if cmp -s "$out/w0-baseline.bin" "$candidate_binary"; then
  echo "FAIL: diameter-restriction scenario did not change the EPANET binary output"
  exit 1
fi

finished_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
solver_sha="$(sha256sum "$solver" | awk '{print $1}')"
baseline_input_sha="$(sha256sum "$out/w0-baseline.inp" | awk '{print $1}')"
candidate_input_sha="$(sha256sum "$candidate_input" | awk '{print $1}')"
baseline_report_sha="$(sha256sum "$out/w0-baseline.rpt" | awk '{print $1}')"
candidate_report_sha="$(sha256sum "$candidate_report" | awk '{print $1}')"
baseline_binary_sha="$(sha256sum "$out/w0-baseline.bin" | awk '{print $1}')"
candidate_binary_sha="$(sha256sum "$candidate_binary" | awk '{print $1}')"
source_input_sha="$(sha256sum "$input" | awk '{print $1}')"
cmake_version="$(cmake --version | head -n 1)"
compiler_version="$(cc --version | head -n 1)"

jq -n \
  --arg schema_version "epanet-synthetic-smoke/v0.1" \
  --arg state "Completed" \
  --arg source_commit "4d8d82ddc260fce216af9321fc3d9a4646ac6827" \
  --arg engine_version "EPANET 2.2 source revision" \
  --arg solver_sha256 "$solver_sha" \
  --arg source_input_sha256 "$source_input_sha" \
  --arg input_rel "$input_rel" \
  --arg declared_input_sha "$declared_input_sha" \
  --arg downstream_node "$downstream_node" \
  --arg scenario_bundle_sha256 "$scenario_file_sha" \
  --arg scenario_bundle_id "$scenario_bundle_id" \
  --arg scenario_id "$scenario_id" \
  --arg scenario_asset_id "$asset_id" \
  --arg scenario_baseline_diameter "$baseline_diameter" \
  --arg scenario_candidate_diameter "$candidate_diameter" \
  --arg baseline_input_sha256 "$baseline_input_sha" \
  --arg candidate_input_sha256 "$candidate_input_sha" \
  --arg baseline_report_sha256 "$baseline_report_sha" \
  --arg candidate_report_sha256 "$candidate_report_sha" \
  --arg baseline_binary_sha256 "$baseline_binary_sha" \
  --arg candidate_binary_sha256 "$candidate_binary_sha" \
  --arg source_pipe_id "$source_pipe_id" \
  --arg total_junction_demand_lps "$total_junction_demand_lps" \
  --arg downstream_demand_lps "$downstream_demand_lps" \
  --arg baseline_source_flow "$baseline_source_flow" \
  --arg candidate_source_flow "$candidate_source_flow" \
  --arg baseline_pipe_flow "$baseline_pipe_flow" \
  --arg candidate_pipe_flow "$candidate_pipe_flow" \
  --arg baseline_pipe_velocity "$baseline_pipe_velocity" \
  --arg candidate_pipe_velocity "$candidate_pipe_velocity" \
  --arg baseline_pipe_headloss "$baseline_pipe_headloss" \
  --arg candidate_pipe_headloss "$candidate_pipe_headloss" \
  --arg baseline_downstream_pressure "$baseline_downstream_pressure" \
  --arg candidate_downstream_pressure "$candidate_downstream_pressure" \
  --arg cmake_version "$cmake_version" \
  --arg compiler_version "$compiler_version" \
  --arg started_at "$started_at" \
  --arg finished_at "$finished_at" \
  '{
    schema_version: $schema_version,
    run_state: $state,
    synthetic_only: true,
    solver: {
      engine: "EPANET",
      version: $engine_version,
      source_commit: $source_commit,
      executable_sha256: $solver_sha256,
      cmake: $cmake_version,
      compiler: $compiler_version
    },
    source_fixture: {
      path: $input_rel,
      sha256: $source_input_sha256,
      declared_sha256: $declared_input_sha
    },
    scenario_bundle: {
      path: "sol-atlas-core/fixtures/water-engineering/w0-p2-diameter-restriction.synthetic.json",
      sha256: $scenario_bundle_sha256,
      downstream_node_id: $downstream_node,
      bundle_id: $scenario_bundle_id,
      scenario_id: $scenario_id,
      target_asset_id: $scenario_asset_id,
      parameter: "diameter_mm",
      baseline_value_mm: ($scenario_baseline_diameter | tonumber),
      candidate_value_mm: ($scenario_candidate_diameter | tonumber)
    },
    baseline: {
      change: "none",
      input_sha256: $baseline_input_sha256,
      report_sha256: $baseline_report_sha256,
      binary_output_sha256: $baseline_binary_sha256
    },
    candidate: {
      change: ("pipe " + $scenario_asset_id + " diameter " + $scenario_baseline_diameter + " mm -> " + $scenario_candidate_diameter + " mm"),
      input_sha256: $candidate_input_sha256,
      report_sha256: $candidate_report_sha256,
      binary_output_sha256: $candidate_binary_sha256
    },
    hydraulic_observations: {
      scope: "first reported hydraulic period; report-rounded values",
      continuity: {
        declared_total_junction_demand_lps: ($total_junction_demand_lps | tonumber),
        downstream_junction_id: $downstream_node,
        declared_downstream_demand_lps: ($downstream_demand_lps | tonumber),
        source_pipe_id: $source_pipe_id,
        source_flow_lps: {
          baseline: ($baseline_source_flow | tonumber),
          candidate: ($candidate_source_flow | tonumber)
        },
        target_pipe_flow_matches_downstream_demand_within_0_02_lps: true,
        source_flow_matches_total_demand_within_0_02_lps: true,
        terminal_node_has_single_pipe_connection: true
      },
      units: {
        flow: "L/s",
        velocity: "m/s",
        headloss: "m per 1000 m",
        pressure: "m"
      },
      target_pipe: {
        baseline: {
          flow_lps: ($baseline_pipe_flow | tonumber),
          velocity_m_per_s: ($baseline_pipe_velocity | tonumber),
          headloss_m_per_1000_m: ($baseline_pipe_headloss | tonumber)
        },
        candidate: {
          flow_lps: ($candidate_pipe_flow | tonumber),
          velocity_m_per_s: ($candidate_pipe_velocity | tonumber),
          headloss_m_per_1000_m: ($candidate_pipe_headloss | tonumber)
        }
      },
      downstream_node_pressure_m: {
        baseline: ($baseline_downstream_pressure | tonumber),
        candidate: ($candidate_downstream_pressure | tonumber)
      },
      checks: {
        target_pipe_flow_invariant_within_0_02_lps: true,
        target_pipe_velocity_increased: true,
        target_pipe_headloss_increased: true,
        downstream_node_pressure_decreased: true
      }
    },
    execution: {
      started_at: $started_at,
      finished_at: $finished_at,
      baseline_exit_code: 0,
      candidate_exit_code: 0,
      analysis_mode: "DemandDriven",
      headloss_model: "Hazen-Williams",
      unit_system: "EPANET LPS (length m, diameter mm, flow L/s, head m)",
      command: "runepanet <input.inp> <report.rpt> <output.bin>"
    },
    nonclaims: [
      "This is a synthetic solver smoke test only.",
      "A changed report confirms a scenario affects this model run; it does not qualify a municipal model or design.",
      "No empirical calibration, uncertainty quantification, engineering review, or real-network validation is implied."
    ]
  }' > "$out/epanet-synthetic-smoke-evidence.json"

sha256sum "$out"/* | tee "$out/artifact-sha256.txt"
echo "EPANET_SYNTHETIC_SMOKE_PASS: pinned solver ran scenario-bundle-derived restriction case; declared digests, simple-network continuity and directional hydraulics were verified."
