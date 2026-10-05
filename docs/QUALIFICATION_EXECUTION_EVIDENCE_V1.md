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

The artifact is uploaded only after the qualification job has passed. Before
publication, the workflow invokes the repository's independent verifier at
tools/verify_qualification_execution_evidence.py with the current GitHub
execution context. The verifier checks the exact schema, execution bindings, SHA shapes, run URL, qualification command inventory,
independent SHA-256 recomputation of the checked-out workflow/Cargo.lock/test
inventory files, and the JSON sidecar digest. This keeps packet production and
packet validation as separate implementations.
The upload contains both qualification-execution-v1.json and a SHA-256 sidecar
named qualification-execution-v1.json.sha256, allowing the JSON bytes to be
archived and checked independently of GitHub's artifact container.

Failure to publish the packet fails the qualification job rather than silently
degrading to an unrecorded pass.

## Interoperability

The packet uses JSON and SHA-256 for ordinary machine inspection. Its schema is
application-defined and versioned. It does not claim RFC 8785/JCS canonical
JSON compatibility.
