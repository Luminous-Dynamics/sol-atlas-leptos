# Qualification execution evidence V2

## Purpose

QualificationExecutionEvidenceV2 is an additive execution-context receipt built
on the V1 boundary. It adds explicit identity for the repository verifier that
interpreted the packet, without changing V1 packet semantics.

It remains deliberately separate from:

- historical V5ReplayReceiptV1 identity;
- portable replay content identity;
- canonical historical qualification/authority.

Its purpose is to let an independent consumer reconstruct which source revision,
workflow definition, verifier implementation, toolchain, dependency lock, and
test inventory produced the qualification result.

## V2 bindings

In addition to the V1 bindings, the V2 packet records:

- the SHA-256 of `tools/verify_qualification_execution_evidence.py`.

The verifier independently recomputes the workflow-file, verifier, Cargo.lock,
and test-inventory digests rather than trusting only the packet's declared
values.

## Emission and verification boundary

The packet is emitted only after:

1. exact pull-request head verification;
2. test inventory inspection;
3. `cargo test -p sol-atlas-core --locked`;
4. adversarial self-tests of the independent verifier;
5. Cargo.lock immutability verification;
6. clean working-tree verification.

The V2 verifier is invoked with the current GitHub execution context and checks:

- exact schema and key set;
- source/base/workflow/workflow-file-path/run/check-run bindings;
- SHA-256 recomputation of all recorded repository file identities;
- the workflow-run URL;
- the exact qualification command inventory;
- the packet SHA-256 sidecar.

The adversarial self-test exercises stale-sidecar rejection, verifier identity
mismatch rejection, and independent tamper rejection for the workflow file,
verifier file, Cargo.lock, and test inventory.

A missing packet cannot be interpreted as a successful qualification.

## Trust boundary

The packet is evidence of what the hosted workflow observed. It is not an
independent cryptographic attestation of the runner itself and does not claim
SLSA or in-toto conformance.

GitHub documents `github.workflow_sha` as the commit SHA for the workflow file
and `job.check_run_id` as the check-run ID of the current job. V2 records these
values alongside the separate SHA-256 of the checked-out workflow bytes.

## Interoperability

The packet uses JSON and SHA-256 for ordinary machine inspection. The schema is
application-defined and versioned. It does not claim RFC 8785/JCS canonical
JSON compatibility.
