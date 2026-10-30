# Qualification execution evidence V1

## Purpose

QualificationExecutionEvidenceV1 is an execution-context receipt emitted only
after the exact-head provenance qualification job completes successfully.

It is deliberately separate from:

- historical V5ReplayReceiptV1 identity;
- portable replay content identity;
- canonical historical qualification/authority.

Its purpose is to let an independent consumer reconstruct which source revision,
workflow definition, toolchain, dependency lock, and test inventory produced the
qualification result.

## Required bindings

The packet records:

- repository and pull-request identity;
- the exact checked-out source revision;
- the base revision used by the pull request;
- GitHub workflow name/ref/definition SHA;
- the SHA-256 of the checked-out workflow file;
- GitHub run ID, attempt, qualification job name, and check-run ID;
- a direct workflow-run URL;
- runner OS and architecture;
- requested Rust toolchain plus full rustc --version --verbose;
- full Cargo version;
- SHA-256 of Cargo.lock;
- SHA-256 of the captured sol-atlas-core test inventory;
- the exact qualification command;
- status passed.

The packet is emitted only after:

1. exact pull-request head verification;
2. test inventory inspection;
3. cargo test -p sol-atlas-core --locked;
4. Cargo.lock immutability verification;
5. clean working-tree verification.

A missing packet therefore cannot be interpreted as a successful qualification.

## Trust boundary

The packet is evidence of what the hosted workflow observed. It is not an
independent cryptographic attestation of the runner itself and does not claim
SLSA or in-toto conformance.

The workflow definition SHA is recorded from the GitHub Actions github.workflow_sha
context. The checked-out workflow file is separately hashed so consumers can
distinguish the workflow definition identity from the source-tree copy.

V1 packet semantics are intentionally preserved without the V2 verifier
identity extension. The current workflow emits the additive V2 packet instead;
V1 remains the compatibility contract for previously emitted V1 evidence.
The V1 schema therefore does not acquire new required fields retroactively.

Failure to publish the packet fails the qualification job rather than silently
degrading to an unrecorded pass.

## Interoperability

The packet uses JSON and SHA-256 for ordinary machine inspection. Its schema is
application-defined and versioned. It does not claim RFC 8785/JCS canonical
JSON compatibility.
