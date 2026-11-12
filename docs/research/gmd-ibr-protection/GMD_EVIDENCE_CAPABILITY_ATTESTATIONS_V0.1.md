# GMD evidence capability attestations v0.1

**Date:** 12 November 2026  
**Tracking:** [GMD-EVIDENCE-CAPABILITY-001 #733](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/733)  
**Parent:** [GMD protection envelope verifier PR #731](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/731)  
**Claim ceiling:** E2 synthetic contract verification only.

## Purpose

A capability cannot be established because a manifest says `physical_relay_observed=true`. A signature is also not proof that a statement is true: it establishes that the holder of a trusted issuer key signed exact payload bytes, subject to trust policy and validity time.

The v0.1 verifier separates artifact integrity, issuer trust, Ed25519 signature verification, time validity, evidence-reference integrity, exact subject/configuration/scope matching, qualification class and independence.

## Contract

The required claim specifies claim type, subject kind/identity, configuration digest, scope digest, accepted qualification classes, accepted issuer roles and whether independence is required.

The detached payload specifies the issuer, role, qualification class, subject/configuration, scope, evidence artifact digest, method, protocol revision, issue/expiry times, independence statement and limitations. The verifier checks the signature over the original payload bytes without reserializing JSON.

The trust store includes issuer status, permitted roles, independence assessment and a DER-encoded Ed25519 public key artifact. OpenSSL 3 performs detached Ed25519 verification using an argv-based process invocation (no shell interpolation).

## Status semantics

- **SATISFIED:** active trusted issuer, valid signature, valid time window, intact evidence bytes, and exact claim/scope/configuration/qualification/role/independence match.
- **UNKNOWN:** valid attestation does not establish the requested claim, such as wrong scope or inadequate qualification class.
- **REJECTED:** invalid signature, revoked/untrusted issuer, or expired/not-yet-valid attestation.
- Artifact digest mismatches are hard verifier errors, not UNKNOWN.

Qualification classes are claim-specific. A synthetic contract signature cannot satisfy a physical relay hardware-test requirement. This is not a scalar fidelity ranking.

## Synthetic fixture

The checked-in fixture contains one Ed25519-signed E2 synthetic waveform capability. The public key and evidence payload are synthetic. The private test key is not included. The fixture proves that signature and claim-scope checks work; it does not establish a real laboratory, issuer or physical asset qualification.

The trust-store digest demonstrates integrity pinning within the fixture. Production deployment must pin the trust root out-of-band and establish issuer enrollment, key rotation, revocation, compromise recovery and delegated issuer policy.

## Limitations

- Signature validity proves control of the corresponding test private key, not a real-world issuer's identity or competence.
- A signed evidence digest does not establish that the measurement is physically accurate.
- The verifier does not prove the scientific truth of a model or method.
- No physical relay, CT, power interface, field exercise or operational procedure is qualified.
- No switching, dispatch or protection-setting changes are authorised.

## Next steps

1. Production trust-root governance and independent issuer enrollment.
2. Key rotation, revocation and supersession registries.
3. Calibration and method accreditation attestations.
4. Claim-specific model-validity and mathematical proof-checking attestations.
5. Mycelix provenance integration without allowing the ledger to self-authorize its claims.
