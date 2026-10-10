#!/usr/bin/env bash
# Capture one exact JPL Horizons VECTORS request for reproducible parser tests.
# This script writes raw bytes and hashes; it never edits committed fixtures.
set -euo pipefail

die() {
  printf 'error: %s\n' "$*" >&2
  exit 2
}

usage() {
  cat >&2 <<'USAGE'
Usage:
  scripts/capture-horizons-vector.sh OUTPUT_DIRECTORY
  scripts/capture-horizons-vector.sh --print-url

Defaults (override with environment variables):
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

for tool in python3 curl jq sha256sum; do
  command -v "$tool" >/dev/null 2>&1 || die "required tool not found: $tool"
done

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
epoch_text = str(int(value)) if value.is_integer() else repr(value)

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

if [[ $# -gt 0 && "$1" == "--print-url" ]]; then
  printf '%s\n' "$url"
  exit 0
fi

[[ $# -eq 1 ]] || { usage; die "provide an output directory"; }
out_dir=$1
[[ ! -e "$out_dir" ]] || die "output path already exists; choose a fresh directory"
mkdir -p "$(dirname "$out_dir")"
mkdir "$out_dir"

url_file="$out_dir/request.url"
response_file="$out_dir/response.raw.json"
metadata_file="$out_dir/capture-metadata.json"
receipt_file="$out_dir/capture-receipt.json"

# No newline is written to the canonical URL file; its digest therefore hashes
# exactly the same URL bytes that the GET client sends.
printf '%s' "$url" > "$url_file"
request_sha=$(sha256sum "$url_file" | awk '{print $1}')

printf 'Request SHA-256: %s\n' "$request_sha"
printf 'Fetching one vector state from JPL Horizons...\n'
curl \
  --proto '=https' \
  --tlsv1.2 \
  --fail \
  --silent \
  --show-error \
  --connect-timeout 10 \
  --max-time 60 \
  --output "$response_file" \
  "$url" || die "HTTP request failed; inspect the capture directory before retrying"

# Seal the raw bytes and write a receipt before schema checks. If the provider
# changes format, operators still retain the response hash and requested query.
response_sha=$(sha256sum "$response_file" | awk '{print $1}')
retrieved_at=$(date -u +'%Y-%m-%dT%H:%M:%SZ')
source=$(jq -r '.signature.source // "unparsed"' "$response_file" 2>/dev/null || printf 'unparsed')
version=$(jq -r '.signature.version // "unparsed"' "$response_file" 2>/dev/null || printf 'unparsed')
jq -n \
  --arg request_sha256 "$request_sha" \
  --arg response_sha256 "$response_sha" \
  --arg retrieved_at_utc "$retrieved_at" \
  --arg provider_source "$source" \
  --arg api_version "$version" \
  '{
    receipt_status: "captured-unreviewed",
    canonical_request_sha256: $request_sha256,
    raw_response_sha256: $response_sha256,
    retrieved_at_utc: $retrieved_at_utc,
    reported_provider_source: $provider_source,
    reported_api_version: $api_version
  }' > "$receipt_file"

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
result=$(jq -er '.result' "$response_file") || die "response lacks result text; raw response and receipt retained"

# Enforce the same body-name, coordinate metadata, epoch, and column contract
# as the parser. A mismatch leaves response.raw.json plus capture-receipt.json
# for inspection but does not create capture-metadata.json.
python3 - "$result" "$expected_target" "$expected_center" "$ref_system" "$ref_plane" "$vec_corr" "$epoch" <<'PY' \
  || die "captured response did not match the parser contract; raw response and receipt retained"
import csv
import math
import sys

(result, expected_target, expected_center, expected_frame, expected_plane,
 expected_correction, epoch_text) = sys.argv[1:]

lines = result.splitlines()
starts = [i for i, line in enumerate(lines) if line.strip() == "$SOE"]
ends = [i for i, line in enumerate(lines) if line.strip() == "$EOE"]
if len(starts) != 1 or len(ends) != 1 or starts[0] >= ends[0]:
    raise SystemExit("missing or duplicated $SOE/$EOE markers")

header = lines[:starts[0]]
def metadata(label):
    for line in header:
        if ":" in line:
            key, value = line.split(":", 1)
            if key.strip().casefold() == label.casefold():
                return value.strip()
    raise SystemExit(f"missing response metadata: {label}")

def body_name(value):
    value = value.split("{", 1)[0].split("(", 1)[0].strip()
    first, separator, remainder = value.partition(" ")
    if separator and first.isascii() and first.isdigit():
        return remainder.strip()
    return value

actual_target = body_name(metadata("Target body name"))
actual_center = body_name(metadata("Center body name"))
if actual_target.casefold() != expected_target.casefold():
    raise SystemExit(f"target mismatch: expected {expected_target!r}, got {actual_target!r}")
if actual_center.casefold() != expected_center.casefold():
    raise SystemExit(f"center mismatch: expected {expected_center!r}, got {actual_center!r}")

for label, expected in [
    ("Reference frame", expected_frame),
    ("Reference plane", expected_plane),
    ("Aberration corrections", expected_correction),
    ("Output units", "KM-S"),
]:
    actual = metadata(label)
    if label == "Reference frame" and expected == "B1950":
        valid = actual.casefold() in {"b1950", "fk4/b1950"}
    else:
        valid = actual.casefold() == expected.casefold()
    if not valid:
        raise SystemExit(f"{label} mismatch: expected {expected!r}, got {actual!r}")

column_line = next(
    (line for line in reversed(header) if "JDTDB" in line.upper() and "," in line),
    None,
)
if column_line is None:
    raise SystemExit("missing labelled JDTDB vector-column header")
columns = [field.strip().upper() for field in next(csv.reader([column_line]))]
expected_columns = [
    "JDTDB", "CALENDAR DATE (TDB)", "X", "Y", "Z", "VX", "VY", "VZ"
]
if columns != expected_columns:
    raise SystemExit(f"unexpected vector columns: {columns!r}")

data_lines = [line for line in lines[starts[0] + 1:ends[0]] if line.strip()]
if len(data_lines) != 1:
    raise SystemExit(f"expected one output row, got {len(data_lines)}")
fields = next(csv.reader([data_lines[0]]))
if len(fields) != 8:
    raise SystemExit(f"expected 8 CSV fields, got {len(fields)}")
try:
    output_epoch = float(fields[0])
    expected_epoch = float(epoch_text)
    components = [float(field) for field in fields[-6:]]
except ValueError:
    raise SystemExit("epoch or state components are not numeric")
if not math.isfinite(output_epoch) or any(not math.isfinite(v) for v in components):
    raise SystemExit("epoch or state components are non-finite")
if abs(output_epoch - expected_epoch) > 1.0e-8:
    raise SystemExit(f"epoch mismatch: expected {expected_epoch}, got {output_epoch}")
PY

jq -n \
  --arg requested_target "$target" \
  --arg requested_center "$center" \
  --arg requested_epoch_jd "$epoch" \
  --arg expected_target "$expected_target" \
  --arg expected_center "$expected_center" \
  --arg ref_system "$ref_system" \
  --arg ref_plane "$ref_plane" \
  --arg vector_correction "$vec_corr" \
  --arg request_sha256 "$request_sha" \
  --arg response_sha256 "$response_sha" \
  --arg retrieved_at_utc "$retrieved_at" \
  --arg api_source "$source" \
  --arg api_version "$version" \
  '{
    fixture_status: "captured-not-yet-reviewed",
    provider: $api_source,
    api_signature_version: $api_version,
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
    raw_response_sha256: $response_sha256,
    retrieved_at_utc: $retrieved_at_utc
  }' > "$metadata_file"

printf '\nCapture files written:\n  %s\n  %s\n  %s\n  %s\n' "$url_file" "$response_file" "$receipt_file" "$metadata_file"
printf 'Status: captured, not yet reviewed or promoted into test fixtures.\n'
printf 'Next: inspect response.raw.json and compare its header/column labels with the parser contract.\n'
