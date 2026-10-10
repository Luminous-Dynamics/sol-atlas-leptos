# JPL Horizons capture and fixture-promotion runbook

Status: operational procedure for collecting a real vector response. No response
is considered a test fixture merely because the capture script returned success.

## 1. Offline preflight

Run from the repository root:

```sh
bash -n scripts/capture-horizons-vector.sh
python3 -m py_compile scripts/validate-horizons-vector.py
python3 -m py_compile scripts/verify-horizons-capture.py
python3 -m unittest discover -s scripts -p 'test_*.py' -v
scripts/capture-horizons-vector.sh --print-url
scripts/capture-horizons-vector.sh --print-identity
```

These checks must not contact Horizons. Confirm the printed URL has one explicitly
quoted TDB epoch, `EPHEM_TYPE='VECTORS'`, `VEC_TABLE='2'`, `VEC_CORR='NONE'`,
`OUT_UNITS='KM-S'`, and the reviewed target/centre. The identity output includes
stable Sol Atlas target/centre IDs and expected response-header names.

## 2. Capture into a new directory

Choose a new, non-existent output directory. The script deliberately refuses to
overwrite one:

```sh
capture_dir="$PWD/artifacts/horizons-mars-ssb-$(date -u +%Y%m%dT%H%M%SZ)"
HORIZONS_TARGET_ID=mars \
HORIZONS_CENTER_ID=ssb \
HORIZONS_COMMAND=499 \
HORIZONS_CENTER='@0' \
HORIZONS_EPOCH_JD=2461323.5 \
HORIZONS_EXPECTED_TARGET=Mars \
HORIZONS_EXPECTED_CENTER='Solar System Barycenter' \
scripts/capture-horizons-vector.sh "$capture_dir"
```

This performs a single HTTPS GET. The capture directory contains:
- `request.url`: exact URL bytes sent to the endpoint, without a trailing newline.
- `request.identity`: length-delimited semantic ID/name fields plus the exact URL.
- `response.raw.json`: unmodified response body bytes.
- `capture-receipt.json`: request, URL, and response digests and the declared
  provider/signature version.
- `capture-metadata.json`: target, centre, epoch, units, frame/plane, correction,
  response identity, and retrieval time.

The hashes show that the saved records agree byte-for-byte. They are not a digital
signature and do not prove that the provider or TLS endpoint was authentic. Keep
the entire directory together and retain the capture outside the source checkout
until review is complete. Do not normalize JSON, reformat the response, or edit
the receipt by hand.

## 3. Verify the saved packet offline

Run:

```sh
python3 scripts/verify-horizons-capture.py "$capture_dir"
```

The verifier checks the exact saved URL/identity/response digests, the canonical
URL encoding and parameter order, URL-vs-metadata consistency for target, centre,
epoch, units, reference system/plane and correction mode, the raw response
signature/source/version, and the parsed response schema. It also loads the
shared `catalogue-bindings.json` manifest and rejects known target/centre
relabeling, including a packet whose digests were recomputed after relabeling.

A verifier pass means internal byte consistency and schema checks passed; it is
not an authenticity verdict. Preserve any failed packet for investigation rather
than editing it in place.

## 4. Review and promote a real fixture

Only after step 3 passes, review:
1. The exact URL and canonical identity against the intended body, centre, epoch,
   frame/plane, correction mode, output units and vector table.
2. The unmodified signature source/version and target/centre labels in the body.
3. The column labels and each numeric state component; confirm position is km,
   velocity is km/s, and the JDTDB value equals the requested epoch.
4. All three hashes and the capture timestamp. Record the reviewer/commit in the
   pull request or review record; the hashes themselves do not identify a reviewer.

Add a **new** fixture and any query/receipt sidecar needed to reproduce it. Never
replace `mars_ssb_tdb_frame_km_s.json`: that fixture is synthetic and exists to
test parser behavior deterministically. Label the new data as provider-captured,
reviewed, and fixed to its exact request/response digest. The test should rebuild
`EphemerisProvenance` from the exact saved bytes and parse the response offline;
CI must not make network requests.

If the response signature version or header differs from the reviewed allowlist,
stop and review the contract first. Do not widen the allowlist or weaken column,
identity, epoch, or provenance validation just to make a new capture pass.

## 5. Evidence and promotion gate

Track these as separate states:

- `captured-unreviewed`: the bytes and hashes were saved.
- `packet-verified`: local URL, receipt, metadata, and response are internally
  consistent and schema-valid.
- `fixture-reviewed`: a human has reviewed the provider payload and query.
- `fixture-committed`: an immutable new fixture and its deterministic test are
  in version control.

None of these states means the renderer uses the sample. Full-scene integration,
the exact-head Rust/WASM checks, and visual/coordinate parity are separate gates.
Queued CI jobs are not passes, and a parser fixture is not evidence of a renderer
capture.
