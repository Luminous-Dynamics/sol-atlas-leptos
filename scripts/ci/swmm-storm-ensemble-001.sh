#!/usr/bin/env bash
set -Eeuo pipefail
umask 077

repo_root="$PWD"
out="$repo_root/artifacts/water-storm-ensemble-001"
solver="$repo_root/.swmm-build/bin/Release/runswmm"
if [[ ! -x "$solver" && -x "$repo_root/.swmm-build/src/run/runswmm" ]]; then
  solver="$repo_root/.swmm-build/src/run/runswmm"
fi
plan_file="$repo_root/sol-atlas-core/fixtures/water-engineering/water-storm-ensemble-001.synthetic.json"
input_file="$repo_root/sol-atlas-core/fixtures/water-engineering/s0_stormwater.inp"
source_commit="7952ca837988b1c32f791812eccc9fd64547e093"
continuity_limit="1.0"
case_out="$out/cases"

mkdir -p "$case_out"
test -x "$solver"
test -s "$plan_file"
test -s "$input_file"

expected_source_sha="$(jq -er '.source_input.sha256' "$plan_file")"
expected_source_bytes="$(jq -er '.source_input.byte_length' "$plan_file")"
source_sha="$(sha256sum "$input_file" | awk '{print $1}')"
source_bytes="$(wc -c < "$input_file" | tr -d '[:space:]')"
if [[ "$source_sha" != "$expected_source_sha" || "$source_bytes" != "$expected_source_bytes" ]]; then
  echo "WATER_STORM_SOURCE_IDENTITY_MISMATCH: expected=$expected_source_sha/$expected_source_bytes actual=$source_sha/$source_bytes" >&2
  exit 1
fi
plan_sha="$(sha256sum "$plan_file" | awk '{print $1}')"
solver_sha="$(sha256sum "$solver" | awk '{print $1}')"
github_sha="${GITHUB_SHA:-}"

write_case_result() {
  local case_id="$1" profile="$2" multiplier="$3" diameter="$4" baseline_id="$5"
  local expected_rain_depth="$6" state="$7" exit_code="$8" derived_sha="$9" report_sha="${10}" output_sha="${11}" stdout_sha="${12}" metrics_json="${13}" dir="${14}"
  local metrics
  metrics="$(jq -c '.' <<<"$metrics_json")"
  jq -n \
    --arg case_id "$case_id" \
    --arg profile "$profile" \
    --arg multiplier "$multiplier" \
    --arg diameter "$diameter" \
    --arg baseline_id "$baseline_id" \
    --arg expected_rain_depth "$expected_rain_depth" \
    --arg state "$state" \
    --arg exit_code "$exit_code" \
    --arg derived_sha "$derived_sha" \
    --arg report_sha "$report_sha" \
    --arg output_sha "$output_sha" \
    --arg stdout_sha "$stdout_sha" \
    --argjson metrics "$metrics" \
    'def num_or_null: if . == "" then null else tonumber end;
     def hash_or_null: if test("^[a-f0-9]{64}$") then . else null end;
     {
       case_id: $case_id,
       rainfall_profile_id: $profile,
       rainfall_depth_multiplier: ($multiplier | tonumber),
       conduit_diameter_m: ($diameter | tonumber),
       paired_baseline_case_id: $baseline_id,
       expected_rainfall_depth_mm: ($expected_rain_depth | tonumber),
       case_state: $state,
       exit_code: ($exit_code | num_or_null),
       derived_input_sha256: ($derived_sha | hash_or_null),
       report_sha256: ($report_sha | hash_or_null),
       binary_output_sha256: ($output_sha | hash_or_null),
       stdout_sha256: ($stdout_sha | hash_or_null),
       metrics: $metrics,
       paired_deltas: {
         peak_j1_depth_m: null,
         peak_conduit_velocity_m_s: null,
         peak_outfall_flow_m3_s: null,
         flooding_loss_million_liters: null
       }
     }' > "$dir/result.json"
}

empty_metrics='{"runoff_continuity_error_percent":null,"routing_continuity_error_percent":null,"rainfall_depth_mm":null,"total_runoff_depth_mm":null,"total_runoff_volume_million_liters":null,"peak_runoff_m3_s":null,"peak_j1_depth_m":null,"peak_j1_time":null,"peak_conduit_flow_m3_s":null,"peak_conduit_velocity_m_s":null,"conduit_max_full_flow_ratio":null,"conduit_max_full_depth_ratio":null,"peak_outfall_flow_m3_s":null,"outfall_volume_million_liters":null,"flooding_loss_million_liters":null,"surcharged":null,"flooded":null}'

while IFS= read -r case_json; do
  case_id="$(jq -er '.case_id' <<<"$case_json")"
  profile="$(jq -er '.rainfall_profile_id' <<<"$case_json")"
  multiplier="$(jq -er '.rainfall_depth_multiplier' <<<"$case_json")"
  diameter="$(jq -er '.conduit_diameter_m' <<<"$case_json")"
  baseline_id="$(jq -er '.paired_baseline_case_id' <<<"$case_json")"
  expected_rain_depth="$(jq -er '.expected_rainfall_depth_mm' <<<"$case_json")"
  mapfile -t rain_values < <(jq -er '.rainfall_intensity_mm_hr[]' <<<"$case_json")
  case_dir="$case_out/$case_id"
  mkdir -p "$case_dir"
  derived="$case_dir/model.inp"
  report="$case_dir/model.rpt"
  binary="$case_dir/model.out"
  stdout="$case_dir/model.stdout"
  derived_sha="" report_sha="" output_sha="" stdout_sha=""
  exit_code=""
  state="Failed"
  metrics_json="$empty_metrics"

  if [[ "${#rain_values[@]}" -ne 6 ]]; then
    echo "WATER_STORM_RAIN_VECTOR_INVALID: case=$case_id count=${#rain_values[@]}" >&2
    write_case_result "$case_id" "$profile" "$multiplier" "$diameter" "$baseline_id" "$expected_rain_depth" "Failed" "64" "" "" "" "$metrics_json" "$case_dir"
    continue
  fi

  if awk \
    -v diameter="$diameter" \
    -v r0="${rain_values[0]}" -v r1="${rain_values[1]}" -v r2="${rain_values[2]}" \
    -v r3="${rain_values[3]}" -v r4="${rain_values[4]}" -v r5="${rain_values[5]}" '
      /^\[XSECTIONS\]/ { section = "x"; print; next }
      /^\[TIMESERIES\]/ { section = "t"; print; next }
      /^\[/ { section = ""; print; next }
      section == "x" && $1 == "C1" && $1 !~ /^;/ {
        if (($3 + 0) != 0.6) {
          printf "WATER_STORM_BASE_DIAMETER_MISMATCH: input=%s expected=0.6\n", $3 > "/dev/stderr"
          exit 2
        }
        $3 = diameter
        xsections++
        print
        next
      }
      section == "t" && $1 == "RAIN1" && $2 == "01/01/2020" {
        if ($3 == "00:00") { $4 = r0; changed++ }
        else if ($3 == "00:05") { $4 = r1; changed++ }
        else if ($3 == "00:10") { $4 = r2; changed++ }
        else if ($3 == "00:15") { $4 = r3; changed++ }
        else if ($3 == "00:20") { $4 = r4; changed++ }
        else if ($3 == "00:25") { $4 = r5; changed++ }
        print
        next
      }
      { print }
      END {
        if (xsections != 1 || changed != 6) {
          printf "WATER_STORM_INPUT_TRANSFORM_INCOMPLETE: xsections=%d rainfall_rows=%d\n", xsections, changed > "/dev/stderr"
          exit 1
        }
      }
    ' "$input_file" > "$derived"; then
    derived_sha="$(sha256sum "$derived" | awk '{print $1}')"
  else
    echo "WATER_STORM_INPUT_TRANSFORM_FAILED: case=$case_id" >&2
    write_case_result "$case_id" "$profile" "$multiplier" "$diameter" "$baseline_id" "$expected_rain_depth" "Failed" "65" "" "" "" "$metrics_json" "$case_dir"
    continue
  fi

  if "$solver" "$derived" "$report" "$binary" > "$stdout" 2>&1; then
    exit_code="0"
  else
    exit_code="$?"
  fi
  if [[ "$exit_code" != "0" ]] || grep -Fq "There are errors." "$stdout" || grep -Fq "There are warnings." "$stdout" || [[ ! -s "$report" || ! -s "$binary" ]]; then
    echo "WATER_STORM_SOLVER_CASE_FAILED: case=$case_id exit_code=$exit_code" >&2
    cat "$stdout" >&2 || true
    state="Failed"
  else
    parsed="$(
      awk '
        /Continuity Error \(%\)/ { continuity[++continuity_count] = $NF }
        /Flooding Loss/ { flooding_loss = $NF }
        /Subcatchment Runoff Summary/ { section = "subrunoff"; next }
        /Node Depth Summary/ { section = "depth"; next }
        /Link Flow Summary/ { section = "link"; next }
        /Outfall Loading Summary/ { section = "outfall"; next }
        section == "subrunoff" && $1 == "S1" && NF >= 11 {
          rainfall = $2; runoff_depth = $8; runoff_volume = $9; peak_runoff = $10
          section = ""
        }
        section == "depth" && $1 == "J1" && $2 == "JUNCTION" {
          peak_j1_depth = $4; peak_j1_time = $7; section = ""
        }
        section == "link" && $1 == "C1" && $2 == "CONDUIT" {
          peak_conduit_flow = $3; peak_conduit_velocity = $6; full_flow_ratio = $7; full_depth_ratio = $8
          section = ""
        }
        section == "outfall" && $1 == "O1" && NF >= 5 {
          peak_outfall_flow = $4; outfall_volume = $5; section = ""
        }
        END {
          if (continuity_count != 2 || rainfall == "" || runoff_depth == "" || runoff_volume == "" ||
              peak_runoff == "" || peak_j1_depth == "" || peak_j1_time == "" ||
              peak_conduit_flow == "" || peak_conduit_velocity == "" || full_flow_ratio == "" ||
              full_depth_ratio == "" || peak_outfall_flow == "" || outfall_volume == "" || flooding_loss == "") {
            print "WATER_STORM_REPORT_METRIC_PARSE_FAILED" > "/dev/stderr"
            exit 2
          }
          printf "%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s\n",
            continuity[1], continuity[2], rainfall, runoff_depth, runoff_volume, peak_runoff,
            peak_j1_depth, peak_j1_time, peak_conduit_flow, peak_conduit_velocity,
            full_flow_ratio, full_depth_ratio, peak_outfall_flow, outfall_volume, flooding_loss
        }
      ' "$report"
    )" || parsed=""
    if [[ -z "$parsed" ]]; then
      state="Failed"
      echo "WATER_STORM_REPORT_PARSE_FAILED: case=$case_id" >&2
    else
      IFS='|' read -r runoff_cont routing_cont rain_mm runoff_depth runoff_volume peak_runoff peak_j1_depth peak_j1_time peak_conduit_flow peak_vel full_flow_ratio full_depth_ratio peak_outfall outfall_volume flooding_loss <<< "$parsed"
      rainfall_matches="true"
      if ! awk -v actual="$rain_mm" -v expected="$expected_rain_depth" 'BEGIN { d=actual-expected; if (d<0) d=-d; exit !(d<=0.02) }'; then
        rainfall_matches="false"
        echo "WATER_STORM_RAINFALL_DEPTH_MISMATCH: case=$case_id expected=$expected_rain_depth actual=$rain_mm" >&2
      fi
      if [[ "$rainfall_matches" == "false" ]]; then
        state="Failed"
      elif awk -v a="$runoff_cont" -v b="$routing_cont" -v lim="$continuity_limit" 'BEGIN { exit !((a<=lim && a>=-lim) && (b<=lim && b>=-lim)) }'; then
        state="Completed"
        exit_code="0"
      else
        state="Failed"
        echo "WATER_STORM_CONTINUITY_LIMIT_EXCEEDED: case=$case_id runoff=$runoff_cont routing=$routing_cont limit=$continuity_limit" >&2
      fi
      surcharged="true"
      flooded="true"
      grep -Fq "No nodes were surcharged." "$report" && surcharged="false"
      grep -Fq "No nodes were flooded." "$report" && flooded="false"
      metrics_json="$(jq -n \
        --arg rc "$runoff_cont" --arg fc "$routing_cont" --arg rain "$rain_mm" \
        --arg rd "$runoff_depth" --arg rv "$runoff_volume" --arg pr "$peak_runoff" \
        --arg jd "$peak_j1_depth" --arg jt "$peak_j1_time" --arg cf "$peak_conduit_flow" \
        --arg cv "$peak_vel" --arg fr "$full_flow_ratio" --arg dr "$full_depth_ratio" \
        --arg of "$peak_outfall" --arg ov "$outfall_volume" --arg fl "$flooding_loss" \
        --arg sur "$surcharged" --arg fld "$flooded" '
        def n: if . == "" then null else tonumber end;
        {
          runoff_continuity_error_percent: ($rc | n),
          routing_continuity_error_percent: ($fc | n),
          rainfall_depth_mm: ($rain | n),
          total_runoff_depth_mm: ($rd | n),
          total_runoff_volume_million_liters: ($rv | n),
          peak_runoff_m3_s: ($pr | n),
          peak_j1_depth_m: ($jd | n),
          peak_j1_time: $jt,
          peak_conduit_flow_m3_s: ($cf | n),
          peak_conduit_velocity_m_s: ($cv | n),
          conduit_max_full_flow_ratio: ($fr | n),
          conduit_max_full_depth_ratio: ($dr | n),
          peak_outfall_flow_m3_s: ($of | n),
          outfall_volume_million_liters: ($ov | n),
          flooding_loss_million_liters: ($fl | n),
          surcharged: ($sur == "true"),
          flooded: ($fld == "true")
        }')"
    fi
  fi

  [[ -f "$report" ]] && report_sha="$(sha256sum "$report" | awk '{print $1}')" || true
  [[ -f "$binary" ]] && output_sha="$(sha256sum "$binary" | awk '{print $1}')" || true
  [[ -f "$stdout" ]] && stdout_sha="$(sha256sum "$stdout" | awk '{print $1}')" || true
  write_case_result "$case_id" "$profile" "$multiplier" "$diameter" "$baseline_id" "$expected_rain_depth" "$state" "$exit_code" "$derived_sha" "$report_sha" "$output_sha" "$stdout_sha" "$metrics_json" "$case_dir"
done < <(jq -c '.cases[]' "$plan_file")

cases_json="$(jq -s '.' "$case_out"/*/result.json)"
completed_count="$(jq '[.[] | select(.case_state == "Completed")] | length' <<<"$cases_json")"
failed_count="$(jq '[.[] | select(.case_state != "Completed")] | length' <<<"$cases_json")"
paired_cases="$(
  jq -c --argjson runs "$cases_json" '
    def delta($a; $b):
      if ($a | type) == "number" and ($b | type) == "number" then $a - $b else null end;
    $runs | map(
      . as $case |
      ($runs | map(select(.case_id == $case.paired_baseline_case_id)) | first) as $base |
      $case + {
        paired_deltas: {
          peak_j1_depth_m: (if $base == null then null else delta($case.metrics.peak_j1_depth_m; $base.metrics.peak_j1_depth_m) end),
          peak_conduit_velocity_m_s: (if $base == null then null else delta($case.metrics.peak_conduit_velocity_m_s; $base.metrics.peak_conduit_velocity_m_s) end),
          peak_outfall_flow_m3_s: (if $base == null then null else delta($case.metrics.peak_outfall_flow_m3_s; $base.metrics.peak_outfall_flow_m3_s) end),
          flooding_loss_million_liters: (if $base == null then null else delta($case.metrics.flooding_loss_million_liters; $base.metrics.flooding_loss_million_liters) end)
        }
      }
    )
  ' <<<"$cases_json"
)"
if [[ "$completed_count" == "54" && "$failed_count" == "0" ]]; then
  run_state="Passed"
else
  run_state="Failed"
fi

jq -n \
  --arg revision "$github_sha" \
  --arg source_sha "$source_sha" \
  --arg plan_sha "$plan_sha" \
  --arg solver_sha "$solver_sha" \
  --arg source_commit "$source_commit" \
  --arg run_state "$run_state" \
  --arg completed_count "$completed_count" \
  --arg failed_count "$failed_count" \
  --argjson cases "$paired_cases" \
  'def maybe_revision: if test("^[a-f0-9]{40}$") then . else null end;
   {
     schema_version: "water-storm-ensemble-run/v0.1",
     run_state: $run_state,
     synthetic_only: true,
     benchmark_id: "water-storm-ensemble-001",
     implementation_revision: ($revision | maybe_revision),
     plan_sha256: $plan_sha,
     source_input_sha256: $source_sha,
     source_input_byte_length: 1660,
     solver: {
       engine: "EPA_SWMM",
       version: "5.2.4",
       source_commit: $source_commit,
       executable_sha256: $solver_sha
     },
     case_count: 54,
     completed_case_count: ($completed_count | tonumber),
     failed_case_count: ($failed_count | tonumber),
     cases: $cases,
     claim_ceiling: "SyntheticScenarioEnvelopeOnly",
     nonclaims: [
       "Synthetic scenario envelope only; not a calibrated catchment or observed event.",
       "The rainfall profiles are not return-period or design storms and imply no annual exceedance probability.",
       "Low continuity error is numerical consistency, not empirical predictive accuracy or flood safety.",
       "No environmental quality, sewer compliance, engineering approval or real-world intervention benefit is established.",
       "No operational control, fieldwork or automated actuation is authorized."
     ]
   }' > "$out/receipt.json"
sha256sum "$out/receipt.json" > "$out/receipt.sha256"
if [[ "$run_state" == "Passed" ]]; then
  echo "WATER_STORM_ENSEMBLE_PASS: 54 synthetic paired cases, 6 rainfall shapes x 3 depth multipliers x 3 conduit diameters; continuity gate <= ${continuity_limit}%; no field-performance claim."
  exit 0
fi
echo "WATER_STORM_ENSEMBLE_FAILED: completed=$completed_count failed_or_not_run=$failed_count; see receipt and per-case reports." >&2
exit 1
