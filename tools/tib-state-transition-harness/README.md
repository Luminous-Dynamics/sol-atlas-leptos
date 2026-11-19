# TIB Property-State Transition Harness

**Status:** executable synthetic host-safety reference model v0.1.

This harness runs the existing nine-scenario `TIB-PROPERTY-STATE-TRANSITIONS-001` corpus against a conservative reference disposition model. It is separate from `sol-atlas-core::territorial`: that module validates territorial assertion structure and provenance links; this harness tests whether a host workflow should hold, reject a retry or require review when external evidence and internal state disagree.

## Invariants exercised

- An internal transfer/property projection is not an external registration receipt.
- A digest or signature reference alone does not prove issuer competence, jurisdiction, scope or currentness.
- Stale/revoked authority, active challenge and surface/subsurface representation mismatch remain review-required.
- An external timeout with unknown outcome is not evaluable.
- A second retry before reconciling a timed-out first request is structurally rejected.
- No scenario authorizes a national-register or other authoritative mutation.

## Run

```sh
cargo fmt --manifest-path tools/tib-state-transition-harness/Cargo.toml --all -- --check
cargo test --manifest-path tools/tib-state-transition-harness/Cargo.toml
cargo clippy --manifest-path tools/tib-state-transition-harness/Cargo.toml --all-targets -- -D warnings
```

The Rust test suite loads the JSON corpus directly. It does not duplicate scenarios in a second fixture file.

## Claim ceiling

The harness validates only its synthetic expected dispositions and host hold/retry behavior. It does not authenticate a registry, decide title, determine South African legal status, verify representative legitimacy or consent, or execute any external mutation. The reference model intentionally contains no positive authoritative-mutation path.
