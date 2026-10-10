#!/usr/bin/env bash
# Capture one exact JPL Horizons VECTORS request for reproducible parser tests.
# This script writes raw bytes and hashes; it never edits committed fixtures.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

die() {
  printf 'error: %s\n' "$*" >&2
  exit 2
}

usage() {
  cat >&2 <<'USAGE'
Usage:
  scripts/capture-horizons-vector.sh OUTPUT_DIRECTORY
  scripts/capture-horizons-vector.sh --print-url
  scripts/capture-horizons-vector.sh --print-identity

Defaults (override with environment variables):
  HORIZONS_TARGET_ID=mars
  HORIZONS_CENTER_ID=ssb
  HORIZONS_COMMAND=499
  HORIZONS_CENTER=@0
  HORIZONS_EPOCH_JD=2461323.5
  HORIZONS_EXPECTED_TARGET=Mars
  HORIZONS_EXPECTED_CENTER='Solar System Barycenter'
  HORIZONS_REF_SYSTEM=ICRF
  HORIZONS_REF_PLANE=FRAME
  HORIZONS_VEC_CORR=NONE

Requirements: bash, python3, curl, jq, sha256sum.
The captured response is not automatically promoted to a test fixture. Review its
signature, target, centre, units, frame, columns, requested epoch, and hashes first.
USAGE
}

env_or_default() {
  local value
  value=$(printenv "$1" 2>/dev/null || true)
  if [[ -z "$value" ]]; then value=$2; fi
  printf '%s' "$value"
}

command -v python3 >/dev/null 2>&1 || die "required tool not found: python3"

target_id=$(env_or_default HORIZONS_TARGET_ID mars)
center_id=$(env_or_default HORIZONS_CENTER_ID ssb)
target=$(env_or_default HORIZONS_COMMAND 499)
center=$(env_or_default HORIZONS_CENTER '@0')
epoch=$(env_or_default HORIZONS_EPOCH_JD 2461323.5)
expected_target=$(env_or_default HORIZONS_EXPECTED_TARGET Mars)
expected_center=$(env_or_default HORIZONS_EXPECTED_CENTER 'Solar System Barycenter')
ref_system=$(env_or_default HORIZONS_REF_SYSTEM ICRF)
ref_plane=$(env_or_default HORIZONS_REF_PLANE FRAME)
vec_corr=$(env_or_default HORIZONS_VEC_CORR NONE)

url=$(
  python3 - "$target" "$center" "$epoch" "$ref_system" "$ref_plane" "$vec_corr" <<'PY'
import math
import sys
from decimal import Decimal
from urllib.parse import quote

target, center, epoch, ref_system, ref_plane, vec_corr = sys.argv[1:]
allowed = set(" -_.@;()=")

def safe_token(value: str) -> bool:
    return bool(value) and value.strip() == value and all(
        ch.isascii() and (ch.isalnum() or ch in allowed) for ch in value
    )

if not safe_token(target) or not safe_token(center):
    raise SystemExit("target/center contains a character outside the allowed set")
if ref_system not in {"ICRF", "B1950"}:
    raise SystemExit("REF_SYSTEM must be ICRF or B1950")
if ref_plane not in {"ECLIPTIC", "FRAME", "BODY EQUATOR"}:
    raise SystemExit("REF_PLANE must be ECLIPTIC, FRAME, or BODY EQUATOR")
if vec_corr not in {"NONE", "LT", "LT+S"}:
    raise SystemExit("VEC_CORR must be NONE, LT, or LT+S")

try:
    value = float(epoch)
except ValueError:
    raise SystemExit("HORIZONS_EPOCH_JD must be a Julian-date number")
if not math.isfinite(value) or value <= 0:
    raise SystemExit("HORIZONS_EPOCH_JD must be finite and positive")
# Match Rust f64::to_string()'s non-exponent notation for ordinary JD
# magnitudes while preserving the shortest decimal spelling of the float.
epoch_text = format(Decimal(str(value)).normalize(), "f")

def q(value: str) -> str:
    return "'" + value + "'"

parameters = [
    ("COMMAND", q(target)),
    ("CENTER", q(center)),
    ("CSV_FORMAT", q("YES")),
    ("EPHEM_TYPE", q("VECTORS")),
    ("MAKE_EPHEM", q("YES")),
    ("OBJ_DATA", q("YES")),
    ("OUT_UNITS", q("KM-S")),
    ("REF_PLANE", q(ref_plane)),
    ("REF_SYSTEM", q(ref_system)),
    ("TIME_TYPE", q("TDB")),
    ("TLIST", q(epoch_text)),
    ("TLIST_TYPE", q("JD")),
    ("VEC_CORR", q(vec_corr)),
    ("VEC_LABELS", q("YES")),
    ("VEC_TABLE", q("2")),
    ("format", "json"),
]
query = "&".join(f"{key}={quote(value, safe='-_.~')}" for key, value in parameters)
url = "https://ssd.jpl.nasa.gov/api/horizons.api?" + query
if len(url.encode("utf-8")) > 7500:
    raise SystemExit("URL exceeds 7500 bytes; use the file-based Horizons API")
print(url, end="")
PY
) || die "failed to build canonical Horizons query"

# Keep request.url as the exact URL sent over HTTP, but hash a canonical
# identity that also binds Sol Atlas target/centre IDs and response expectations.
# This is the same length-delimited format used by HorizonsVectorRequest.
identity=$(
  python3 - "$target_id" "$center_id" "$target" "$center" "$expected_target" "$expected_center" "$url" "$script_dir/../sol-atlas-core/tests/fixtures/horizons/catalogue-bindings.json" <<'PY'
import json
import sys
from pathlib import Path
from urllib.parse import quote

target_id, center_id, command, provider_center, expected_target, expected_center, url, manifest_path = sys.argv[1:]
encode = lambda value: quote(value, safe="-_.~")

# One manifest is shared by the capture and verification tools; Rust unit tests
# compare the same file against SOLAR_SYSTEM_CATALOG to detect mapping drift.
try:
    manifest = json.loads(Path(manifest_path).read_text(encoding="utf-8"))
except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
    raise SystemExit(f"cannot read Horizons catalogue bindings manifest: {error}")
if manifest.get("schema_version") != 1 or not isinstance(manifest.get("objects"), list):
    raise SystemExit("unsupported Horizons catalogue bindings manifest")
objects = manifest["objects"]
target_bindings = {
    obj["id"]: (obj["command"], obj["name"])
    for obj in objects
    if obj.get("command") is not None
}
aggregate_ids = {
    obj["id"]
    for obj in objects
    if obj.get("kind") in {"small_body_population", "spacecraft_population"}
}
center_bindings = {
    obj["id"]: (obj["center"], obj["name"])
    for obj in objects
    if obj.get("center") is not None
}
for special in manifest.get("special_centers", []):
    center_bindings[special["id"]] = (
        special["provider_center"],
        special["name"],
    )
if target_id in aggregate_ids:
    raise SystemExit(f"aggregate catalogue layer cannot be queried as a point target: {target_id}")
if target_id in target_bindings:
    bound_command, bound_name = target_bindings[target_id]
    if command != bound_command or expected_target.casefold() != bound_name.casefold():
        raise SystemExit(f"catalogue target binding mismatch for {target_id}")
if center_id in aggregate_ids:
    raise SystemExit(f"aggregate catalogue layer cannot be used as a point centre: {center_id}")
if center_id in center_bindings:
    bound_center, bound_name = center_bindings[center_id]
    if provider_center != bound_center or expected_center.casefold() != bound_name.casefold():
        raise SystemExit(f"catalogue centre binding mismatch for {center_id}")
transport_bytes = len(url.encode("utf-8"))
print(
    "SOL-ATLAS-HORIZONS-REQUEST-V1\n"
    f"target_id={encode(target_id)}\n"
    f"center_id={encode(center_id)}\n"
    f"expected_target_name={encode(expected_target)}\n"
    f"expected_center_name={encode(expected_center)}\n"
    f"transport_bytes={transport_bytes}\n"
    f"{url}",
    end="",
)
PY
) || die "failed to build canonical request identity"

if [[ $# -eq 1 && "$1" == "--print-url" ]]; then
  printf '%s\n' "$url"
  exit 0
fi
if [[ $# -eq 1 && "$1" == "--print-identity" ]]; then
  printf '%s\n' "$identity"
  exit 0
fi

[[ $# -eq 1 ]] || { usage; die "provide an output directory"; }
for tool in curl jq sha256sum; do
  command -v "$tool" >/dev/null 2>&1 || die "required tool not found: $tool"
done
out_dir=$1
[[ ! -e "$out_dir" ]] || die "output path already exists; choose a fresh directory"
mkdir -p "$(dirname "$out_dir")"
mkdir "$out_dir"

url_file="$out_dir/request.url"
identity_file="$out_dir/request.identity"
response_file="$out_dir/response.raw.json"
metadata_file="$out_dir/capture-metadata.json"
receipt_file="$out_dir/capture-receipt.json"

# No newline is written to either canonical file. request.url is the exact URL
# sent over HTTP; request.identity is the same length-delimited semantic-plus-
# transport identity the Rust parser hashes for EphemerisProvenance.
printf '%s' "$url" > "$url_file"
printf '%s' "$identity" > "$identity_file"
request_sha=$(sha256sum "$identity_file" | awk '{print $1}')
url_sha=$(sha256sum "$url_file" | awk '{print $1}')

printf 'Canonical request identity SHA-256: %s\n' "$request_sha"
printf 'Canonical URL SHA-256: %s\n' "$url_sha"
printf 'Fetching one vector state from JPL Horizons...\n'
http_status=$(
  curl \
    --proto '=https' \
    --tlsv1.2 \
    --silent \
    --show-error \
    --connect-timeout 10 \
    --max-time 60 \
    --output "$response_file" \
    --write-out '%{http_code}' \
    "$url"
) || die "HTTP transfer failed; inspect the capture directory before retrying"

# Seal the raw bytes and write a receipt before schema checks. If the provider
# changes format, operators still retain the response hash and requested query.
response_sha=$(sha256sum "$response_file" | awk '{print $1}')
retrieved_at=$(date -u +'%Y-%m-%dT%H:%M:%SZ')
source=$(jq -r '.signature.source // "unparsed"' "$response_file" 2>/dev/null || printf 'unparsed')
version=$(jq -r '.signature.version // "unparsed"' "$response_file" 2>/dev/null || printf 'unparsed')
jq -n \
  --arg request_sha256 "$request_sha" \
  --arg canonical_url_sha256 "$url_sha" \
  --arg response_sha256 "$response_sha" \
  --arg retrieved_at_utc "$retrieved_at" \
  --arg http_status "$http_status" \
  --arg provider_source "$source" \
  --arg api_version "$version" \
  '{
    receipt_status: "captured-unreviewed",
    canonical_request_sha256: $request_sha256,
    canonical_url_sha256: $canonical_url_sha256,
    raw_response_sha256: $response_sha256,
    retrieved_at_utc: $retrieved_at_utc,
    http_status: $http_status,
    http_status: $http_status,
    reported_provider_source: $provider_source,
    reported_api_version: $api_version
  }' > "$receipt_file"

jq -n \
  --arg requested_target_id "$target_id" \
  --arg requested_center_id "$center_id" \
  --arg requested_target "$target" \
  --arg requested_center "$center" \
  --arg requested_epoch_jd "$epoch" \
  --arg expected_target "$expected_target" \
  --arg expected_center "$expected_center" \
  --arg ref_system "$ref_system" \
  --arg ref_plane "$ref_plane" \
  --arg vector_correction "$vec_corr" \
  --arg request_sha256 "$request_sha" \
  --arg canonical_url_sha256 "$url_sha" \
  --arg response_sha256 "$response_sha" \
  --arg retrieved_at_utc "$retrieved_at" \
  --arg http_status "$http_status" \
  --arg api_source "$source" \
  --arg api_version "$version" \
  '{
    fixture_status: "captured-not-yet-reviewed",
    provider: $api_source,
    api_signature_version: $api_version,
    requested_target_id: $requested_target_id,
    requested_center_id: $requested_center_id,
    requested_target: $requested_target,
    requested_center: $requested_center,
    expected_target_name: $expected_target,
    expected_center_name: $expected_center,
    requested_epoch_jd_tdb: $requested_epoch_jd,
    reference_system: $ref_system,
    reference_plane: $ref_plane,
    vector_correction: $vector_correction,
    output_units: "KM-S",
    canonical_request_sha256: $request_sha256,
    canonical_url_sha256: $canonical_url_sha256,
    raw_response_sha256: $response_sha256,
    retrieved_at_utc: $retrieved_at_utc
  }' > "$metadata_file"

[[ "$http_status" == "200" ]] || die "unexpected HTTP status $http_status; raw response, receipt and metadata retained"

# Fail closed on signature drift but preserve the raw response and receipt.
[[ "$source" == "NASA/JPL Horizons API" ]] || die "unexpected provider source: $source"
case "$version" in
  1.0|1.3) ;;
  *) die "unreviewed Horizons signature version: $version; raw response and receipt retained" ;;
esac
if jq -e '(.error // "") != ""' "$response_file" >/dev/null; then
  jq -r '.error' "$response_file" >&2
  die "Horizons returned an application-level error; raw response and receipt retained"
fi
python3 "$script_dir/validate-horizons-vector.py" \
  "$response_file" \
  "$expected_target" \
  "$expected_center" \
  "$ref_system" \
  "$ref_plane" \
  "$vec_corr" \
  "$epoch" || die "captured response failed the schema contract; raw response and receipt retained"


printf '\nCapture files written:\n  %s\n  %s\n  %s\n  %s\n  %s\n' "$url_file" "$identity_file" "$response_file" "$receipt_file" "$metadata_file"
printf 'Status: captured, not yet reviewed or promoted into test fixtures.\n'
printf 'Next: inspect response.raw.json and compare its header/column labels with the parser contract.\n'
