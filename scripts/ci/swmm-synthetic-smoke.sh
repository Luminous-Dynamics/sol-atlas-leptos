#!/usr/bin/env bash
set -Eeuo pipefail
umask 077

repo_root="$PWD"
out="$repo_root/artifacts/swmm-s0"
solver="$repo_root/.swmm-build/bin/Release/runswmm"
if [[ ! -x "$solver" && -x "$repo_root/.swmm-build/src/run/runswmm" ]]; then
  solver="$repo_root/.swmm-build/src/run/runswmm"
fi
bundle_file="$repo_root/sol-atlas-core/fixtures/water-engineering/s0-stormwater-restricted-conduit.synthetic.json"
input_default="$repo_root/sol-atlas-core/fixtures/water-engineering/s0_stormwater.inp"
source_commit="7952ca837988b1c32f791812eccc9fd64547e093"
max_abs_continuity_percent="1.0"

github_sha="${GITHUB_SHA:-}"
github_ref="${GITHUB_REF:-}"
workflow_run_id="${GITHUB_RUN_ID:-}"
workflow_run_attempt="${GITHUB_RUN_ATTEMPT:-}"

failure_stage="initialization"
failure_line=""
bundle_id="unknown"
scenario_id="unknown"
asset_id="C1"
baseline_diameter="0.6"
candidate_diameter="0.25"
declared_input_sha=""
declared_input_bytes=""
actual_input_sha=""
actual_input_bytes=""
bundle_sha=""
solver_sha=""
digest_match=""
baseline_state="NotRun"
candidate_state="NotRun"
baseline_exit_code=""
candidate_exit_code=""
baseline_input_sha=""
candidate_input_sha=""
baseline_report_sha=""
candidate_report_sha=""
baseline_output_sha=""
candidate_output_sha=""
baseline_stdout_sha=""
candidate_stdout_sha=""
baseline_runoff=""
baseline_routing=""
candidate_runoff=""
candidate_routing=""
reports_differ=""
outputs_differ=""
continuity_pass=""

mkdir -p "$out"

write_manifest() {
  local state="$1" code="$2" line="$3" message="$4"
  jq -n \
    --arg schema_version "swmm-synthetic-run/v0.1" \
    --arg run_state "$state" \
    --arg failure_stage "$failure_stage" \
    --arg failure_code "$code" \
    --arg failure_line "$line" \
    --arg failure_message "$message" \
    --arg source_commit "$source_commit" \
    --arg solver_sha "$solver_sha" \
    --arg github_sha "$github_sha" \
    --arg github_ref "$github_ref" \
    --arg workflow_run_id "$workflow_run_id" \
    --arg workflow_run_attempt "$workflow_run_attempt" \
    --arg input_path "sol-atlas-core/fixtures/water-engineering/s0_stormwater.inp" \
    --arg input_sha "$actual_input_sha" \
    --arg input_bytes "$actual_input_bytes" \
    --arg bundle_path "sol-atlas-core/fixtures/water-engineering/s0-stormwater-restricted-conduit.synthetic.json" \
    --arg bundle_sha "$bundle_sha" \
    --arg digest_match "$digest_match" \
    --arg bundle_id "$bundle_id" \
    --arg scenario_id "$scenario_id" \
    --arg asset_id "$asset_id" \
    --arg baseline_diameter "$baseline_diameter" \
    --arg candidate_diameter "$candidate_diameter" \
    --arg baseline_state "$baseline_state" \
    --arg candidate_state "$candidate_state" \
    --arg baseline_exit_code "$baseline_exit_code" \
    --arg candidate_exit_code "$candidate_exit_code" \
    --arg baseline_input_sha "$baseline_input_sha" \
    --arg candidate_input_sha "$candidate_input_sha" \
    --arg baseline_report_sha "$baseline_report_sha" \
    --arg candidate_report_sha "$candidate_report_sha" \
    --arg baseline_output_sha "$baseline_output_sha" \
    --arg candidate_output_sha "$candidate_output_sha" \
    --arg baseline_stdout_sha "$baseline_stdout_sha" \
    --arg candidate_stdout_sha "$candidate_stdout_sha" \
    --arg baseline_runoff "$baseline_runoff" \
    --arg baseline_routing "$baseline_routing" \
    --arg candidate_runoff "$candidate_runoff" \
    --arg candidate_routing "$candidate_routing" \
    --arg reports_differ "$reports_differ" \
    --arg outputs_differ "$outputs_differ" \
    --arg continuity_pass "$continuity_pass" \
    --arg max_abs "$max_abs_continuity_percent" \
    'def num_or_null: if . == "" then null else tonumber end;
     def bool_or_null: if . == "" then null else . == "true" end;
     def hash_or_null: if test("^[a-f0-9]{64}$") then . else null end;
     {
       schema_version: $schema_version,
       run_state: $run_state,
       synthetic_only: true,
       failure_stage: (if $run_state == "Completed" then null else $failure_stage end),
       failure_exit_code: ($failure_code | num_or_null),
       failure_line: ($failure_line | num_or_null),
       failure_message: (if $run_state == "Completed" then null else $failure_message end),
       solver: {
         engine: "EPA_SWMM",
         version: "5.2.4",
         source_commit: $source_commit,
         executable_sha256: ($solver_sha | hash_or_null)
       },
       execution_identity: {
         github_sha: (if ($github_sha | test("^[a-f0-9]{40}$")) then $github_sha else null end),
         github_ref: (if $github_ref == "" then null else $github_ref end),
         workflow_run_id: (if ($workflow_run_id | test("^[0-9]+$")) then $workflow_run_id else null end),
         workflow_run_attempt: (if ($workflow_run_attempt | test("^[0-9]+$")) then $workflow_run_attempt else null end)
       },
       source_fixture: {
         path: $input_path,
         sha256: ($input_sha | hash_or_null),
         byte_length: ($input_bytes | num_or_null)
       },
       scenario_bundle: {
         path: $bundle_path,
         sha256: ($bundle_sha | hash_or_null),
         bundle_id: $bundle_id,
         scenario_id: $scenario_id,
         target_asset_id: $asset_id,
         parameter: "circular_diameter_m",
         baseline_value_m: ($baseline_diameter | num_or_null),
         candidate_value_m: ($candidate_diameter | num_or_null)
       },
       baseline: {
         state: $baseline_state,
         exit_code: ($baseline_exit_code | num_or_null),
         input_sha256: ($baseline_input_sha | hash_or_null),
         report_sha256: ($baseline_report_sha | hash_or_null),
         output_sha256: ($baseline_output_sha | hash_or_null),
         stdout_sha256: ($baseline_stdout_sha | hash_or_null)
       },
       candidate: {
         state: $candidate_state,
         exit_code: ($candidate_exit_code | num_or_null),
         input_sha256: ($candidate_input_sha | hash_or_null),
         report_sha256: ($candidate_report_sha | hash_or_null),
         output_sha256: ($candidate_output_sha | hash_or_null),
         stdout_sha256: ($candidate_stdout_sha | hash_or_null)
       },
       continuity: {
         max_abs_percent: ($max_abs | tonumber),
         baseline: {
           runoff_percent: ($baseline_runoff | num_or_null),
           routing_percent: ($baseline_routing | num_or_null)
         },
         candidate: {
           runoff_percent: ($candidate_runoff | num_or_null),
           routing_percent: ($candidate_routing | num_or_null)
         }
       },
       checks: {
         input_digest_matches: ($digest_match | bool_or_null),
         baseline_outputs_present: ($baseline_report_sha != "" and $baseline_output_sha != ""),
         candidate_outputs_present: ($candidate_report_sha != "" and $candidate_output_sha != ""),
         reports_differ: ($reports_differ | bool_or_null),
         binary_outputs_differ: ($outputs_differ | bool_or_null),
         continuity_within_tolerance: ($continuity_pass | bool_or_null)
       },
       claim_ceiling: "synthetic_swmm_single_event_adapter_smoke",
       nonclaims: [
         "Synthetic solver-adapter smoke only; not a calibrated catchment.",
         "No flood safety, sewer compliance, receiving-water quality, treatment efficacy, or intervention-benefit claim.",
         "No fieldwork, operational control, or automated actuation is authorized."
       ]
     }' > "$out/swmm-synthetic-run.v0.1.json"
}

on_exit() {
  local status="$1" line="$2"
  trap - EXIT
  if (( status != 0 )); then
    failure_line="$line"
    [[ -x "$solver" && ! "$solver_sha" =~ ^[a-f0-9]{64}$ ]] && solver_sha="$(sha256sum "$solver" | awk '{print $1}')" || true
    [[ -f "$input_default" ]] && actual_input_sha="$(sha256sum "$input_default" | awk '{print $1}')" || true
    [[ -f "$input_default" ]] && actual_input_bytes="$(wc -c < "$input_default" | tr -d '[:space:]')" || true
    [[ -f "$bundle_file" ]] && bundle_sha="$(sha256sum "$bundle_file" | awk '{print $1}')" || true
    for stem in s0-baseline s0-candidate; do
      if [[ -f "$out/$stem.inp" ]]; then
        local ih
        ih="$(sha256sum "$out/$stem.inp" | awk '{print $1}')"
        if [[ "$stem" == "s0-baseline" && -z "$baseline_input_sha" ]]; then baseline_input_sha="$ih"; fi
        if [[ "$stem" == "s0-candidate" && -z "$candidate_input_sha" ]]; then candidate_input_sha="$ih"; fi
      fi
      if [[ -f "$out/$stem.rpt" ]]; then
        local rh
        rh="$(sha256sum "$out/$stem.rpt" | awk '{print $1}')"
        if [[ "$stem" == "s0-baseline" && -z "$baseline_report_sha" ]]; then baseline_report_sha="$rh"; fi
        if [[ "$stem" == "s0-candidate" && -z "$candidate_report_sha" ]]; then candidate_report_sha="$rh"; fi
      fi
      if [[ -f "$out/$stem.out" ]]; then
        local oh
        oh="$(sha256sum "$out/$stem.out" | awk '{print $1}')"
        if [[ "$stem" == "s0-baseline" && -z "$baseline_output_sha" ]]; then baseline_output_sha="$oh"; fi
        if [[ "$stem" == "s0-candidate" && -z "$candidate_output_sha" ]]; then candidate_output_sha="$oh"; fi
      fi
      if [[ -f "$out/$stem.stdout" ]]; then
        local sh
        sh="$(sha256sum "$out/$stem.stdout" | awk '{print $1}')"
        if [[ "$stem" == "s0-baseline" && -z "$baseline_stdout_sha" ]]; then baseline_stdout_sha="$sh"; fi
        if [[ "$stem" == "s0-candidate" && -z "$candidate_stdout_sha" ]]; then candidate_stdout_sha="$sh"; fi
      fi
    done
    echo "SWMM_RUNNER_FAILURE: stage=$failure_stage exit_code=$status line=$line" >&2
    write_manifest "Failed" "$status" "$line" "Runner failed at stage: $failure_stage" || true
    if [[ -f "$out/swmm-synthetic-run.v0.1.json" ]]; then cat "$out/swmm-synthetic-run.v0.1.json" >&2; fi
  fi
  exit "$status"
}
trap 'rc=$?; line=$LINENO; on_exit "$rc" "$line"' EXIT

failure_stage="read_scenario"
bundle_sha="$(sha256sum "$bundle_file" | awk '{print $1}')"
bundle_id="$(jq -er '.bundle_id | strings' "$bundle_file")"
scenario_id="$(jq -er '.scenario.scenario_id | strings' "$bundle_file")"
input_rel="$(jq -er '.model_package.input_snapshot.path | strings' "$bundle_file")"
declared_input_sha="$(jq -er '.model_package.input_snapshot.sha256 | strings' "$bundle_file")"
declared_input_bytes="$(jq -er '.model_package.input_snapshot.byte_length | numbers' "$bundle_file")"
asset_id="$(jq -er '.scenario.changes[0].asset_id | strings' "$bundle_file")"
parameter="$(jq -er '.scenario.changes[0].parameter | strings' "$bundle_file")"
baseline_diameter="$(jq -er '.scenario.changes[0].baseline_value | numbers' "$bundle_file")"
candidate_diameter="$(jq -er '.scenario.changes[0].candidate_value | numbers' "$bundle_file")"

failure_stage="validate_scenario"
jq -e '
  (.classification == "synthetic_public") and
  (.model_package.domain == "stormwater") and
  (.model_package.input_snapshot.synthetic == true) and
  (.model_package.solver_profile.engine == "EPA_SWMM") and
  (.model_package.solver_profile.engine_version == "5.2.4") and
  (.model_package.solver_profile.execution_state == "NotRun") and
  (.model_package.unit_system == "SWMM CMS (length m; runoff flow m3/s; rainfall intensity mm/hr)") and
  (.scenario.base_model_id == .model_package.model_id) and
  (.scenario.base_input_snapshot_sha256 == .model_package.input_snapshot.sha256) and
  (.model_package.source_artifacts | length == 1) and
  (.model_package.input_snapshot.artifact_id == .model_package.source_artifacts[0].artifact_id) and
  (.model_package.input_snapshot.path == .model_package.source_artifacts[0].path) and
  (.model_package.input_snapshot.sha256 == .model_package.source_artifacts[0].sha256) and
  (.model_package.input_snapshot.byte_length == .model_package.source_artifacts[0].byte_length) and
  (.model_package.source_artifacts[0].synthetic == true) and
  (.run_result.state == "NotRun") and
  (.run_result.metrics | length == 0) and
  (.run_result.result_artifacts | length == 0) and
  (.validation.accepted_for_engineering_use == false) and
  (.scenario.scenario_type == "intervention") and
  (.scenario.changes | length == 1) and
  (.scenario.changes[0].asset_id == "C1") and
  (.scenario.changes[0].parameter == "circular_diameter_m") and
  (.scenario.changes[0].unit == "m") and
  (.scenario.changes[0].baseline_value == 0.6) and
  (.scenario.changes[0].candidate_value == 0.25) and
  (.scenario.changes[0].candidate_value > 0) and
  (.scenario.changes[0].candidate_value < .scenario.changes[0].baseline_value) and
  (.model_package.asset_index | any(.[]; .asset_id == "C1" and .asset_type == "conduit"))
' "$bundle_file" >/dev/null

case "$input_rel" in
  "sol-atlas-core/fixtures/water-engineering/s0_stormwater.inp") ;;
  *) echo "SWMM_SCENARIO_INPUT_PATH_REJECTED: $input_rel"; exit 1 ;;
esac
case "$input_rel" in
  *".."*) echo "SWMM_SCENARIO_INPUT_PATH_TRAVERSAL_REJECTED: $input_rel"; exit 1 ;;
esac
case "$asset_id" in
  ""|*[!A-Za-z0-9_-]*) echo "SWMM_SCENARIO_ASSET_ID_REJECTED: $asset_id"; exit 1 ;;
esac
case "$scenario_id" in
  ""|*[!A-Za-z0-9._-]*) echo "SWMM_SCENARIO_ID_REJECTED: $scenario_id"; exit 1 ;;
esac
test "$parameter" = "circular_diameter_m"
test -x "$solver"

failure_stage="verify_input_digest"
input="$repo_root/$input_rel"
actual_input_sha="$(sha256sum "$input" | awk '{print $1}')"
actual_input_bytes="$(wc -c < "$input" | tr -d '[:space:]')"
if [[ "$actual_input_sha" != "$declared_input_sha" || "$actual_input_bytes" != "$declared_input_bytes" ]]; then
  digest_match="false"
  echo "SWMM_INPUT_IDENTITY_MISMATCH: sha expected=$declared_input_sha actual=$actual_input_sha bytes_expected=$declared_input_bytes actual_bytes=$actual_input_bytes"
  exit 1
fi
digest_match="true"
failure_stage="verify_solver_profile"
grep -Eq '^[[:space:]]*FLOW_UNITS[[:space:]]+CMS([[:space:]]|$)' "$input" || { echo "SWMM_INPUT_FLOW_UNITS_UNSUPPORTED: expected CMS"; exit 1; }
grep -Eq '^[[:space:]]*FLOW_ROUTING[[:space:]]+DYNWAVE([[:space:]]|$)' "$input" || { echo "SWMM_INPUT_ROUTING_UNSUPPORTED: expected DYNWAVE"; exit 1; }
grep -Eq '^[[:space:]]*\[REPORT\][[:space:]]*$' "$input" || { echo "SWMM_REPORT_SECTION_MISSING"; exit 1; }
grep -Eq '^[[:space:]]*CONTINUITY[[:space:]]+YES([[:space:]]|$)' "$input" || { echo "SWMM_CONTINUITY_REPORTING_NOT_ENABLED"; exit 1; }

failure_stage="prepare_model_mutation"
cp "$input" "$out/s0-baseline.inp"
awk -v asset="$asset_id" -v baseline="$baseline_diameter" -v candidate="$candidate_diameter" '
  /^\[XSECTIONS\]/ { section = 1; print; next }
  /^\[/ { section = 0 }
  section && $1 == asset && $1 !~ /^;/ {
    if (($3 + 0) != (baseline + 0)) {
      printf "declared baseline diameter mismatch for %s: input=%s scenario=%s\n", asset, $3, baseline > "/dev/stderr"
      exit 2
    }
    $3 = candidate
    changed++
  }
  { print }
  END {
    if (changed != 1) {
      printf "expected exactly one xsection diameter mutation; changed=%d\n", changed > "/dev/stderr"
      exit 1
    }
  }
' "$input" > "$out/s0-candidate.inp"
awk -v asset="$asset_id" -v expected="$candidate_diameter" '
  /^\[XSECTIONS\]/ { section = 1; next }
  /^\[/ { section = 0 }
  section && $1 == asset && $1 !~ /^;/ && ($3 + 0) == (expected + 0) { found++ }
  END { exit !(found == 1) }
' "$out/s0-candidate.inp"

baseline_input_sha="$(sha256sum "$out/s0-baseline.inp" | awk '{print $1}')"
candidate_input_sha="$(sha256sum "$out/s0-candidate.inp" | awk '{print $1}')"
test "$baseline_input_sha" = "$declared_input_sha"

failure_stage="baseline_swmm_run"
if "$solver" "$out/s0-baseline.inp" "$out/s0-baseline.rpt" "$out/s0-baseline.out" > "$out/s0-baseline.stdout" 2>&1; then
  baseline_state="Completed"
  baseline_exit_code="0"
else
  baseline_exit_code="$?"
  baseline_state="Failed"
  cat "$out/s0-baseline.stdout" >&2 || true
  exit "$baseline_exit_code"
fi

# SWMM CLI returns exit status zero even when the model run reports errors.
# Process exit alone is not evidence of a complete simulation.
if grep -Fq "There are errors." "$out/s0-baseline.stdout" || grep -Fq "There are warnings." "$out/s0-baseline.stdout" || [[ ! -s "$out/s0-baseline.rpt" || ! -s "$out/s0-baseline.out" ]]; then
  baseline_state="Failed"
  echo "SWMM_BASELINE_DID_NOT_PRODUCE_CLEAN_RESULT: stdout and report follow" >&2
  cat "$out/s0-baseline.stdout" >&2 || true
  cat "$out/s0-baseline.rpt" >&2 || true
  exit 1
fi

failure_stage="candidate_swmm_run"
if "$solver" "$out/s0-candidate.inp" "$out/s0-candidate.rpt" "$out/s0-candidate.out" > "$out/s0-candidate.stdout" 2>&1; then
  candidate_state="Completed"
  candidate_exit_code="0"
else
  candidate_exit_code="$?"
  candidate_state="Failed"
  cat "$out/s0-candidate.stdout" >&2 || true
  exit "$candidate_exit_code"
fi

if grep -Fq "There are errors." "$out/s0-candidate.stdout" || grep -Fq "There are warnings." "$out/s0-candidate.stdout" || [[ ! -s "$out/s0-candidate.rpt" || ! -s "$out/s0-candidate.out" ]]; then
  candidate_state="Failed"
  echo "SWMM_CANDIDATE_DID_NOT_PRODUCE_CLEAN_RESULT: stdout and report follow" >&2
  cat "$out/s0-candidate.stdout" >&2 || true
  cat "$out/s0-candidate.rpt" >&2 || true
  exit 1
fi

failure_stage="validate_solver_reports"
for stem in s0-baseline s0-candidate; do
  test -s "$out/$stem.rpt"
  test -s "$out/$stem.out"
  grep -q "Runoff Quantity Continuity" "$out/$stem.rpt"
  grep -q "Flow Routing Continuity" "$out/$stem.rpt"
  if grep -Eiq '(^|[[:space:]])ERROR[[:space:]]+[0-9]+' "$out/$stem.rpt"; then
    echo "SWMM_REPORT_CONTAINS_ERROR_RECORD: $stem"
    exit 1
  fi
done

extract_continuity() {
  local report="$1"
  awk '
    /Continuity Error \(%\)/ {
      value = $NF
      if (value !~ /^[-+]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][-+]?[0-9]+)?$/) {
        printf "invalid continuity number: [%s]\n", value > "/dev/stderr"
        exit 2
      }
      values[++count] = value
    }
    END {
      if (count != 2) {
        printf "expected exactly two continuity error values, found %d\n", count > "/dev/stderr"
        exit 3
      }
      printf "%s %s\n", values[1], values[2]
    }
  ' "$report"
}

failure_stage="parse_continuity"
continuity_values="$(extract_continuity "$out/s0-baseline.rpt")"
read -r baseline_runoff baseline_routing <<< "$continuity_values"
continuity_values="$(extract_continuity "$out/s0-candidate.rpt")"
read -r candidate_runoff candidate_routing <<< "$continuity_values"
number_re='^[-+]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][-+]?[0-9]+)?$'
for value in "$baseline_runoff" "$baseline_routing" "$candidate_runoff" "$candidate_routing"; do
  [[ "$value" =~ $number_re ]] || { echo "SWMM_CONTINUITY_VALUE_INVALID: [$value]"; exit 1; }
done
if awk -v br="$baseline_runoff" -v bt="$baseline_routing" -v cr="$candidate_runoff" -v ct="$candidate_routing" -v limit="$max_abs_continuity_percent" 'BEGIN { exit !((br <= limit && br >= -limit) && (bt <= limit && bt >= -limit) && (cr <= limit && cr >= -limit) && (ct <= limit && ct >= -limit)) }'; then
  continuity_pass="true"
else
  continuity_pass="false"
  echo "SWMM_CONTINUITY_LIMIT_EXCEEDED: limit_abs_percent=$max_abs_continuity_percent baseline_runoff=$baseline_runoff baseline_routing=$baseline_routing candidate_runoff=$candidate_runoff candidate_routing=$candidate_routing"
  exit 1
fi

failure_stage="compare_outputs"
if cmp -s "$out/s0-baseline.rpt" "$out/s0-candidate.rpt"; then
  reports_differ="false"
  echo "SWMM_REPORT_UNCHANGED_AFTER_DECLARED_DIAMETER_INTERVENTION"
  exit 1
else
  reports_differ="true"
fi
if cmp -s "$out/s0-baseline.out" "$out/s0-candidate.out"; then
  outputs_differ="false"
  echo "SWMM_BINARY_OUTPUT_UNCHANGED_AFTER_DECLARED_DIAMETER_INTERVENTION"
  exit 1
else
  outputs_differ="true"
fi

failure_stage="hash_success"
solver_sha="$(sha256sum "$solver" | awk '{print $1}')"
baseline_report_sha="$(sha256sum "$out/s0-baseline.rpt" | awk '{print $1}')"
candidate_report_sha="$(sha256sum "$out/s0-candidate.rpt" | awk '{print $1}')"
baseline_output_sha="$(sha256sum "$out/s0-baseline.out" | awk '{print $1}')"
candidate_output_sha="$(sha256sum "$out/s0-candidate.out" | awk '{print $1}')"
baseline_stdout_sha="$(sha256sum "$out/s0-baseline.stdout" | awk '{print $1}')"
candidate_stdout_sha="$(sha256sum "$out/s0-candidate.stdout" | awk '{print $1}')"
write_manifest "Completed" "" "" ""
sha256sum "$out/swmm-synthetic-run.v0.1.json" > "$out/swmm-synthetic-run.v0.1.sha256"
echo "SWMM_SYNTHETIC_SMOKE_PASS: source=$source_commit; absolute runoff/routing continuity error <= ${max_abs_continuity_percent}%; synthetic adapter only."
 "$input" || { echo "SWMM_REPORT_SECTION_MISSING"; exit 1; }
grep -Eq '^[[:space:]]*CONTINUITY[[:space:]]+YES([[:space:]]|$)' "$input" || { echo "SWMM_CONTINUITY_REPORTING_NOT_ENABLED"; exit 1; }

failure_stage="prepare_model_mutation"
cp "$input" "$out/s0-baseline.inp"
awk -v asset="$asset_id" -v baseline="$baseline_diameter" -v candidate="$candidate_diameter" '
  /^\[XSECTIONS\]/ { section = 1; print; next }
  /^\[/ { section = 0 }
  section && $1 == asset && $1 !~ /^;/ {
    if (($3 + 0) != (baseline + 0)) {
      printf "declared baseline diameter mismatch for %s: input=%s scenario=%s\n", asset, $3, baseline > "/dev/stderr"
      exit 2
    }
    $3 = candidate
    changed++
  }
  { print }
  END {
    if (changed != 1) {
      printf "expected exactly one xsection diameter mutation; changed=%d\n", changed > "/dev/stderr"
      exit 1
    }
  }
' "$input" > "$out/s0-candidate.inp"
awk -v asset="$asset_id" -v expected="$candidate_diameter" '
  /^\[XSECTIONS\]/ { section = 1; next }
  /^\[/ { section = 0 }
  section && $1 == asset && $1 !~ /^;/ && ($3 + 0) == (expected + 0) { found++ }
  END { exit !(found == 1) }
' "$out/s0-candidate.inp"

baseline_input_sha="$(sha256sum "$out/s0-baseline.inp" | awk '{print $1}')"
candidate_input_sha="$(sha256sum "$out/s0-candidate.inp" | awk '{print $1}')"
test "$baseline_input_sha" = "$declared_input_sha"

failure_stage="baseline_swmm_run"
if "$solver" "$out/s0-baseline.inp" "$out/s0-baseline.rpt" "$out/s0-baseline.out" > "$out/s0-baseline.stdout" 2>&1; then
  baseline_state="Completed"
  baseline_exit_code="0"
else
  baseline_exit_code="$?"
  baseline_state="Failed"
  cat "$out/s0-baseline.stdout" >&2 || true
  exit "$baseline_exit_code"
fi

failure_stage="candidate_swmm_run"
if "$solver" "$out/s0-candidate.inp" "$out/s0-candidate.rpt" "$out/s0-candidate.out" > "$out/s0-candidate.stdout" 2>&1; then
  candidate_state="Completed"
  candidate_exit_code="0"
else
  candidate_exit_code="$?"
  candidate_state="Failed"
  cat "$out/s0-candidate.stdout" >&2 || true
  exit "$candidate_exit_code"
fi

failure_stage="validate_solver_reports"
for stem in s0-baseline s0-candidate; do
  test -s "$out/$stem.rpt"
  test -s "$out/$stem.out"
  grep -q "Runoff Quantity Continuity" "$out/$stem.rpt"
  grep -q "Flow Routing Continuity" "$out/$stem.rpt"
  if grep -Eiq '(^|[[:space:]])ERROR[[:space:]]+[0-9]+' "$out/$stem.rpt"; then
    echo "SWMM_REPORT_CONTAINS_ERROR_RECORD: $stem"
    exit 1
  fi
done

extract_continuity() {
  local report="$1"
  awk '
    /Continuity Error \(%\)/ {
      value = $NF
      if (value !~ /^[-+]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][-+]?[0-9]+)?$/) {
        printf "invalid continuity number: [%s]\n", value > "/dev/stderr"
        exit 2
      }
      values[++count] = value
    }
    END {
      if (count != 2) {
        printf "expected exactly two continuity error values, found %d\n", count > "/dev/stderr"
        exit 3
      }
      printf "%s %s\n", values[1], values[2]
    }
  ' "$report"
}

failure_stage="parse_continuity"
continuity_values="$(extract_continuity "$out/s0-baseline.rpt")"
read -r baseline_runoff baseline_routing <<< "$continuity_values"
continuity_values="$(extract_continuity "$out/s0-candidate.rpt")"
read -r candidate_runoff candidate_routing <<< "$continuity_values"
number_re='^[-+]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][-+]?[0-9]+)?$'
for value in "$baseline_runoff" "$baseline_routing" "$candidate_runoff" "$candidate_routing"; do
  [[ "$value" =~ $number_re ]] || { echo "SWMM_CONTINUITY_VALUE_INVALID: [$value]"; exit 1; }
done
if awk -v br="$baseline_runoff" -v bt="$baseline_routing" -v cr="$candidate_runoff" -v ct="$candidate_routing" -v limit="$max_abs_continuity_percent" 'BEGIN { exit !((br <= limit && br >= -limit) && (bt <= limit && bt >= -limit) && (cr <= limit && cr >= -limit) && (ct <= limit && ct >= -limit)) }'; then
  continuity_pass="true"
else
  continuity_pass="false"
  echo "SWMM_CONTINUITY_LIMIT_EXCEEDED: limit_abs_percent=$max_abs_continuity_percent baseline_runoff=$baseline_runoff baseline_routing=$baseline_routing candidate_runoff=$candidate_runoff candidate_routing=$candidate_routing"
  exit 1
fi

failure_stage="compare_outputs"
if cmp -s "$out/s0-baseline.rpt" "$out/s0-candidate.rpt"; then
  reports_differ="false"
  echo "SWMM_REPORT_UNCHANGED_AFTER_DECLARED_DIAMETER_INTERVENTION"
  exit 1
else
  reports_differ="true"
fi
if cmp -s "$out/s0-baseline.out" "$out/s0-candidate.out"; then
  outputs_differ="false"
  echo "SWMM_BINARY_OUTPUT_UNCHANGED_AFTER_DECLARED_DIAMETER_INTERVENTION"
  exit 1
else
  outputs_differ="true"
fi

failure_stage="hash_success"
solver_sha="$(sha256sum "$solver" | awk '{print $1}')"
baseline_report_sha="$(sha256sum "$out/s0-baseline.rpt" | awk '{print $1}')"
candidate_report_sha="$(sha256sum "$out/s0-candidate.rpt" | awk '{print $1}')"
baseline_output_sha="$(sha256sum "$out/s0-baseline.out" | awk '{print $1}')"
candidate_output_sha="$(sha256sum "$out/s0-candidate.out" | awk '{print $1}')"
baseline_stdout_sha="$(sha256sum "$out/s0-baseline.stdout" | awk '{print $1}')"
candidate_stdout_sha="$(sha256sum "$out/s0-candidate.stdout" | awk '{print $1}')"
write_manifest "Completed" "" "" ""
sha256sum "$out/swmm-synthetic-run.v0.1.json" > "$out/swmm-synthetic-run.v0.1.sha256"
echo "SWMM_SYNTHETIC_SMOKE_PASS: source=$source_commit; absolute runoff/routing continuity error <= ${max_abs_continuity_percent}%; synthetic adapter only."
