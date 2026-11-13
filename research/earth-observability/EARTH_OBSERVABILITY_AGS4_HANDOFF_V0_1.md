# EARTH-OBSERVABILITY-AGS4-HANDOFF-001
## Source-preserving geotechnical data exchange and authoritative validation boundary v0.1

**Parent:** EARTH-OBSERVABILITY-QUAL-001 (#761)  
**Status:** local structural preflight and synthetic tests added; external validator adapter not yet connected  
**As-of:** 2026-11-13

## 1. Why AGS is the next ingestion boundary

SOBI supplies a point-index record and optional links to an AGS log or scanned document. A geotechnical AGS file is a different evidence object: it can encode project metadata, locations, strata, samples, tests, groundwater and other depth-referenced observations.

Sol-Atlas should preserve this chain rather than collapsing it:

BGS SOBI index feature → linked AGS asset → format/schema validation → BGS submission validation → domain-specific review → qualified observations

A successful parser or validator does not establish that a borehole was positioned correctly, that depth references are correct, that a log describes the current site, or that a geotechnical interpretation is safe for design.

## 2. Reuse the authoritative validator

The British Geological Survey's AGS File Utilities tool/API performs two distinct checks:

- **AGS format/schema checker:** validates the file against the selected AGS 4.x dictionary and the official AGS rules.
- **BGS/NGDC submission checker:** evaluates BGS-specific data submission requirements.

The current BGS API catalogue identifies version **7.0** and endpoint:

https://agsapi.bgs.ac.uk/v7/validate/

Its request accepts AGS files or ZIP archives and a pinned dictionary selector: v4_0_3, v4_0_4, v4_1 or v4_1_1. It supports checker selection (ags, bgs) and returns JSON or text. The published response example includes the checker library/version, dictionary, validation timestamp, validity, errors, warnings and informational messages. BGS says uploaded files are not saved or stored by the validator.

The BGS user interface currently sets a 50 MB file-size limit. The download service is borehole-focused: users may request an individual borehole or up to 50 boreholes using a bounded spatial search. BGS-generated exports are AGS v4 and may use BGS-generated project/location identifiers rather than the IDs in the original source file.

**Do not duplicate the official validator.** Sol-Atlas should:
1. perform a cheap, offline structural preflight before any external submission;
2. pin the exact AGS dictionary;
3. submit only when source-owner rights and data-handling conditions allow;
4. retain the exact file digest, byte length, endpoint/API version, dictionary, checker selection, validator-library identity, request time and response digest;
5. treat the returned validation status as a separately sourced receipt;
6. keep geotechnical/data-quality and physical qualification separate.

## 3. What the local Rust preflight does—and does not do

The branch adds:

- sol-atlas-core/src/ags_preflight.rs
- sol-atlas-core::ags_preflight
- research/earth-observability/fixtures/ags4-synthetic-v0.1.ags
- src/bin/ags4_preflight.rs — local JSON-reporting command, no network access

The local preflight checks the basic file envelope:

- maximum file size of 50 MiB (the BGS UI limit);
- ASCII bytes and CRLF record delimiters;
- quoted cell parsing, including escaped double quotes and commas inside values;
- GROUP → HEADING → UNIT → TYPE → DATA descriptor ordering;
- unique/non-empty headings;
- unit/type/data row widths;
- at least one data row per group;
- explicit preservation of empty cells as empty strings;
- basic BGS required-group presence (PROJ, ABBR, TYPE, UNIT, LOCA or HOLE, and GEOL).

It can read TRAN_AGS and map only the four currently documented BGS dictionary IDs. Values such as 4, 4.0, empty, or unknown are not guessed or silently promoted to 4.1.1.

This local parser **does not** validate the full AGS dictionary or standard rules, unit semantics, parent/child references, sample uniqueness, coordinate truth, project-specific required headings, geotechnical test results, depth consistency, geological interpretation, BGS/NGDC submission requirements or engineering safety. The basic BGS required-group presence check is only an early warning, not a BGS submission pass.

The parser is deliberately a small Rust preflight, not an attempt to port the official AGS library into a second implementation that will drift. Run it from the repository root with cargo run --bin ags4_preflight -- path/to/file.ags. The CLI performs a bounded read of at most 50 MiB plus one byte before parsing, emits a JSON summary and always labels official validation, rights authorization and physical qualification separately. It does not compute the source digest; the later receipt-producing acquisition layer must hash the exact submitted bytes.

## 4. External validator receipt contract

BgsAgsValidationReceipt and validate_bgs_ags_receipt(...) record/check a normalized result bound to:

- exact API version and endpoint (7.0, /v7/validate/);
- exact submitted file SHA-256 and byte length;
- the explicit dictionary (v4_0_3, v4_0_4, v4_1, or v4_1_1);
- both ags and bgs checkers;
- validator library/version;
- request timestamp;
- exact response-body SHA-256;
- validity and error/warning/informational counts.

A mismatched file, size, dictionary, API version or endpoint fails closed. An error response cannot be promoted to valid by a caller. Warnings and FYIs remain recorded; they are not silently reclassified as errors.

**Receipt-authenticity limit:** the predicate checks the normalized receipt's declared fields. It does not independently hash the AGS bytes, verify TLS/network provenance, authenticate BGS signatures, or prove that the response body matches its digest. The retrieval adapter must hash the original file and response bytes and preserve the HTTP request/response context; Mycelix/custody should authenticate and preserve the receipt before any evidence admission. Do not present a locally constructed receipt as a real BGS validation result.

## 5. Validation is not physical qualification

Treat the following gates separately:

| Gate | What it establishes | What it does not establish |
|---|---|---|
| Local structural preflight | file envelope and basic row structure | full AGS conformance or correct values |
| Official AGS checker | compliance with the selected AGS 4.x format/dictionary | BGS submission acceptance or physical truth |
| BGS/NGDC checker | BGS submission-rule status for the submitted file | field accuracy or geotechnical design adequacy |
| Source/rights review | permitted use of the specific asset and derivatives | measurement correctness |
| Survey/metrology qualification | coordinate/depth reference, instrumentation, calibration and uncertainty | full geological model correctness |
| Geological/geotechnical review | competent interpretation for a defined use and domain | construction or excavation authorization outside the competent authority |

A file can pass AGS formatting while containing wrong coordinates, swapped depth references, incorrect units, stale measurements or unsupported lithological interpretations. Conversely, a warning should remain visible and be handled under a declared policy; it must not disappear from the receipt.

## 6. Source and rights boundaries

- An AGS URL found in a SOBI index is a pointer, not an access grant.
- The SOBI dataset licence does not automatically license the linked AGS file, scan, report or third-party content.
- BGS's validator service says uploaded files are not saved/stored; the exact request still needs source-owner permission, and its current terms/API policy must be followed.
- BGS's AGS Download Service only exposes records where donors have consented to access; legacy confidentiality restrictions can prevent an AGS Submission Record from being shown.
- BGS-generated AGS exports may subset original project data and replace project/location IDs with BGS-generated identifiers. Preserve both original and BGS identities, exact export context and original-source references where supplied.
- UKGEOS Glasgow GGC01 Final Borehole Information Pack is a promising single-borehole multimodal source. The exact BGS/NGDC cited-data record states OGL with a Natural Environment Research Council acknowledgement, while a separate data.gov.uk harvest still shows "Licence not set". Preserve the NGDC cited-data record, exact package manifest and attribution with the source receipt; verify the content/terms of each linked asset. The initial/intermediate releases, separate large image packages and other UKGEOS boreholes are distinct source records, not automatically covered by the final-pack admission.

## 7. Next implementation gates

**AGS-0 — Local preflight:** committed parser, synthetic fixture and negative controls; exact-head CI still pending.  
**AGS-1 — Dictionary pinning:** require an explicit supported TRAN_AGS value and record the matching dictionary ID; never auto-upgrade or default silently.  
**AGS-2 — Receipt adapter:** add a low-volume opt-in multipart client that hashes request file bytes and response bytes, pins /v7/validate/, checks the response envelope and retains the full request/response receipt. No private source data goes to CI.  
**AGS-3 — Rights and custody:** validate exact source/asset/subject/purpose grant before upload; preserve donor/confidentiality restrictions and expiry/revocation.  
**AGS-4 — Semantics:** map only dictionary-qualified fields/units/identifiers into source-preserving observations. Keep index metadata, raw measurements, processed outputs and interpretations distinct.  
**AGS-5 — Survey/geology:** qualify coordinate reference, depth reference, collar/trajectory, unit and calibration semantics; compare with independent reference data where available.  
**AGS-6 — Operational use:** competent geological/geotechnical review and applicable permitting/land/safety authority remain independent and cannot be inferred from parser or BGS validator success.

## 8. References

- BGS AGS File Utilities UI/API: https://agsapi.bgs.ac.uk/
- BGS API OpenAPI / ReDoc (v7.0): https://agsapi.bgs.ac.uk/redoc
- BGS AGS File Utilities API catalogue: https://www.api.gov.uk/bgs/ags-file-utilities-tool-and-api/
- Official pyagsapi documentation: https://britishgeologicalsurvey.github.io/pyagsapi/
- BGS AGS Download Service: https://www.bgs.ac.uk/technologies/geotechnical-data-services/ags-download-service/
- UKGEOS GGC01 final-pack NGDC cited-data record: https://www2.bgs.ac.uk/nationalgeosciencedatacentre/citedData/catalogue/e38c58a6-48ec-4ad1-a996-6c6144968d7d.html
- BGS API fair-use policy: https://www.bgs.ac.uk/technologies/web-services/api/
