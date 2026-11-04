# Streaming source payload and schema verification v0.1

**Status:** Rust Read-based implementation, shared digest engine, streaming receipt schema, synthetic schema fixture, and deterministic parity/error tests implemented; exact-head format, Clippy, unit tests and schema workflow pending.
**Parent:** [#527 — bounded-memory streaming verification](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/527).
**Base byte verifier:** [PR #526](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/526).
**Descriptor layer:** [PR #496](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/496).
**Trust/coverage/impact dependencies:** #347, #489/#490, #397, #484.

## Architecture

The in-memory and streaming interfaces feed the same internal verifier.

1. An input reader is keyed by the stable typed record identity or by (schema ID, schema version).
2. The streaming adapter hashes one reader at a time with a bounded buffer.
3. It records only the computed SHA-256 and exact byte length in an input manifest map.
4. The shared verifier compares those facts with the exact case-bound descriptor catalogue.
5. The streaming wrapper receipt binds the unchanged inner verification receipt plus the execution profile and buffer size.

The streaming API is generic over caller-supplied Rust Read implementations. It never retrieves a URL, resolves a filename, or chooses an object-store path. A caller may supply a file, object-store stream, memory reader, or other explicit Read handle after applying its own access/rights policy.

## API

`verify_catalog_payload_streams(case, catalog, payload_readers, schema_readers, buffer_size_bytes)`

Payload reader keys are:
- `evidence:<id>`;
- `artifact:<id>`;
- `engineering_receipt:<id>`;
- `decision:<id>`.

Schema reader keys are `(schema_id, schema_version)`.

Keys are held in BTreeMaps and consumed in lexicographic key order. The v0.1 buffer is constrained to 4 KiB through 4 MiB. The default benchmark profile is 64 KiB.

The API returns:
- a StreamingByteVerificationReceipt containing the unchanged PayloadByteVerificationReceipt and a streaming execution profile; or
- ByteInputReadError identifying the exact input key, bytes consumed, error kind, and message.

On any read error, no partial-success receipt is returned. Partial digest state stays local to the failed call.

## Receipt and parity

The inner PayloadByteVerificationReceipt is deliberately identical to the result from the in-memory verifier when given the same bytes:
- same per-record payload and schema outcomes;
- same exact payload and schema input manifest SHA-256 values;
- same counts and findings;
- same claim ceiling and nonclaims;
- same inner receipt digest.

The outer streaming receipt adds:
- `io_mode = streaming_reader`;
- requested buffer size;
- working buffer bound;
- deterministic key-order profile;
- exact SHA-256 binding to the inner receipt digest and execution profile.

Changing the buffer size must not change the content digest or semantic result. The outer execution receipt changes because it records the runtime profile; the inner proof of supplied bytes remains identical.

## Memory and error model

The implementation hashes readers sequentially. It allocates one application-owned buffer per active reader (one reader is processed at a time), plus O(number of descriptors) digest metadata and receipt state. The working read buffer is capped at 4 MiB. Caller-owned reader/object-store buffers are outside that bound and must be measured separately.

A reader error returns:
- stable input key;
- bytes successfully consumed before error;
- error kind;
- error message.

The system does not convert short reads, stream errors, or early EOF into a verified result. If EOF arrives before the descriptor's declared byte length, the shared verifier produces a length mismatch. If no length was declared, a digest mismatch still catches truncation when a trusted expected digest exists.

## Tests

The implementation adds:
- in-memory vs streaming inner-receipt parity;
- input manifest parity at 4 KiB, 64 KiB, 1 MiB, and 4 MiB buffers;
- explicit streaming execution profile checks;
- injected read failure after a known byte offset;
- stable failure input key and exact consumed-byte count;
- invalid buffer size rejected before any reader is consumed.

The existing digest mutation, byte-length, missing byte, schema mismatch, owner-attested, external-reference, unavailable digest, and unreferenced-input controls remain shared by both APIs.

## Qualification targets

P0:
- zero payload or schema digest mismatch accepted;
- zero declared byte-length mismatch accepted;
- zero read errors promoted to a successful receipt;
- 100% inner semantic receipt parity between in-memory and streaming paths;
- exact input manifest parity across buffer sizes;
- invalid buffer bounds rejected before reading;
- read failures identify exact source key and consumed length;
- no change to source-owner, attestation, rights, source-universe completeness, engineering correctness or action-authority nonclaims.

P1:
- 1 KiB, 1 MiB, 100 MiB, 1 GiB and 10 GiB profiles where runners/storage permit;
- p50/p95/p99 and worst-case throughput;
- MB/s and CPU ns/byte;
- peak RSS and allocation count;
- I/O wait, bytes read, storage type and cache state;
- SHA implementation and CPU feature profile;
- concurrent worker scaling without digest/state contamination;
- end-to-end object retrieval plus verification throughput.

Target: application-owned streaming read buffer <=4 MiB per active worker, excluding caller-owned source/object-store buffers. This is a design target, not yet a measured result.

## Nonclaims

Streaming improves memory behavior; it does not improve source authenticity. A matching digest still does not establish owner identity, signatures, rights, schema semantic validity, source-universe completeness, engineering correctness, professional approval, or action authority.