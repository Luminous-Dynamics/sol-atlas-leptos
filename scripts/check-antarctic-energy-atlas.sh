#!/usr/bin/env bash
# Copyright (C) 2026 Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Validates Antarctic station source-support semantics. It does not verify the
# scientific truth of source metadata or establish that raw observations exist.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
data_file="${1:-$repo_root/research/antarctic-energy-atlas/sites-v0.json}"

command -v jq >/dev/null 2>&1 || {
  echo "ERROR: jq is required to validate the Antarctic atlas source manifest" >&2
  exit 2
}
test -f "$data_file" || {
  echo "ERROR: missing manifest: $data_file" >&2
  exit 2
}

audit_manifest() {
  local candidate="$1"
  jq -e '
    .schema_version == "antarctic-energy-atlas-sites-v0.1"
    and .status == "source_pinned_locator_metadata_only"
    and (.sites | length) == 4
    and ([.sites[].id] | unique | length) == (.sites | length)
    and all(.sites[];
      (.id | type) == "string"
      and (.name | type) == "string"
      and (.lat | type) == "number"
      and (.lon | type) == "number"
      and (.coordinate_precision_decimal_places == 4)
      and (.measured_energy_observations_ingested == false)
    )
    and ([.sites[] | select(.id == "aad-casey") | [.lat,.lon]] == [[-66.2818,110.5276]])
    and ([.sites[] | select(.id == "aad-davis") | [.lat,.lon]] == [[-68.5766,77.9673]])
    and ([.sites[] | select(.id == "aad-mawson") | [.lat,.lon]] == [[-67.6026,62.8738]])
    and ([.sites[] | select(.id == "aad-macquarie") | .site_class] == ["subantarctic_island"])
    and ([.observation_sources[] | select(.id == "dcceew-aad-fy2024-25-table73") | .spatial_support] == ["australian_antarctic_division_program_aggregate"])
    and ([.observation_sources[] | select(.id == "dcceew-aad-fy2024-25-table73") | .site_ids] == [[]])
    and ([.observation_sources[] | select(.id == "aad-soe-electricity-monthly") | .metadata_conflict.status] == ["unresolved_until_raw_file_and_source_record_reconciliation"])
    and ([.observation_sources[] | select(.id == "aad-soe-electricity-monthly") | .metadata_conflict.linked_SOE_SFU_record.archive_description_span] == ["1993-01 through 2016-02"])
    and ([.observation_sources[] | select(.id == "aad-soe-electricity-monthly") | .metadata_conflict.display_indicator_page.displayed_span] == ["1986-01 through 2016-02"])
    and ([.observation_sources[] | select(.id == "aad-soe-electricity-monthly") | .metadata_conflict.display_indicator_page.same_page_metadata_start] == ["1990-01-01"])
    and ([.observation_sources[] | select(.id == "aad-soe-electricity-monthly") | .doi] == ["10.26179/5d38f549336a7"])
    and ([.observation_sources[] | select(.id == "aad-soe-generator-boiler-fuel-monthly") | .metadata_conflict.status] == ["unresolved_until_raw_file_and_originator_reconciliation"])
    and ([.observation_sources[] | select(.id == "aad-soe-generator-boiler-fuel-monthly") | .metadata_conflict.citation_years_reported] == [[2001,2019]])
    and ([.observation_sources[] | select(.id == "aad-soe-generator-boiler-fuel-monthly") | .citation_records | map(.citation_year)] == [[2001,2019]])
    and ([.metadata_stewardship.status] == ["not_confirmed"])
    and ([.observation_sources[] | select(.id == "copernicus-era5-hourly-single-levels") | .evidence_class] == ["reanalysis_not_site_measurement"])
    and ([.observation_sources[] | select(.id == "copernicus-era5-hourly-single-levels") | .spatial_support] == ["gridded_reanalysis"])
    and all(.controls_required_before_ui[]; type == "string")
  ' "$candidate" >/dev/null
}

expect_rejected() {
  local label="$1"
  local filter="$2"
  local mutated
  mutated="$(mktemp)"
  if ! jq "$filter" "$data_file" >"$mutated"; then
    rm -f "$mutated"
    echo "ERROR: could not construct atlas negative control: $label" >&2
    return 1
  fi
  if audit_manifest "$mutated" >/dev/null 2>&1; then
    rm -f "$mutated"
    echo "ERROR: invalid atlas manifest was accepted: $label" >&2
    return 1
  fi
  rm -f "$mutated"
  echo "PASS: rejected atlas negative control — $label"
}

audit_manifest "$data_file"
echo "PASS: source-only station locator manifest; program aggregate and gridded reanalysis remain unassigned to stations."

expect_rejected "Macquarie Island reclassified as continental Antarctic station" '.sites |= map(if .id == "aad-macquarie" then .site_class = "antarctic_station" else . end)'
expect_rejected "programme annual aggregate joined to Casey" '.observation_sources |= map(if .id == "dcceew-aad-fy2024-25-table73" then .site_ids = ["aad-casey"] else . end)'
expect_rejected "historical temporal conflict silently resolved" '.observation_sources |= map(if .id == "aad-soe-electricity-monthly" then .metadata_conflict.status = "resolved" else . end)'
expect_rejected "electricity data span overwritten by another record" '.observation_sources |= map(if .id == "aad-soe-electricity-monthly" then .metadata_conflict.linked_SOE_SFU_record.archive_description_span = "1986-01 through 2016-02" else . end)'
expect_rejected "fuel citation-year conflict silently collapsed" '.observation_sources |= map(if .id == "aad-soe-generator-boiler-fuel-monthly" then .metadata_conflict.citation_years_reported = [2019] else . end)'
expect_rejected "AADC originator contact falsely claimed" '.metadata_stewardship.status = "confirmed"'
expect_rejected "ERA5 relabeled as site measurement" '.observation_sources |= map(if .id == "copernicus-era5-hourly-single-levels" then .evidence_class = "site_measured" else . end)'
expect_rejected "source-only locator falsely marked as ingested observations" '.sites[0].measured_energy_observations_ingested = true'
expect_rejected "coordinate altered from source metadata" '.sites |= map(if .id == "aad-casey" then .lat = -60.0 else . end)'

echo "BOUNDARY: integrity tests only; source locators are metadata and historical raw observations have not been ingested."
sha256sum "$data_file"
