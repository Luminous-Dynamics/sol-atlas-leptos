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

- the explicit workflow file path reported by the running GitHub job;
- the SHA-256 of `tools/verify_qualification_execution_evidence.py`;
- the exact GitHub event name and triggering ref.

The verifier rejects duplicate JSON object keys so the packet has a single
unambiguous value for each field. It independently recomputes the workflow-file,
verifier, Cargo.lock, and test-inventory digests rather than trusting only the
packet's declared values. It also binds runner OS, runner architecture, Rust toolchain, rustc
version output, and cargo version output to the current qualification job;
the latter two are recomputed during verification rather than accepted solely
from the packet.

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
- current runner and toolchain identity/observation bindings;
- the workflow-run URL;
- the exact qualification command inventory;
- the packet SHA-256 sidecar.

The adversarial self-test exercises stale-sidecar rejection, workflow-file-path
context mismatch rejection, mutation of every packet execution-context scalar,
required-key-set rejection, command-inventory ordering rejection, canonical pull-request workflow-ref
rejection, verifier identity mismatch rejection, and independent tamper rejection
for duplicate JSON keys, the workflow file,
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

The packet uses JSON and SHA-256 for ordinary machine inspection. Duplicate
JSON object keys are rejected to keep every packet field semantically
unambiguous. The qualification contract pins Rust 1.99.0, Linux/x64 runner execution, and the `Check` / `.github/workflows/check.yml` /
`provenance-construction` workflow identity. The event is pinned to
`pull_request` and its ref to `refs/pull/<number>/merge`. The recorded `workflow_ref` must
also be the canonical GitHub `pull_request` merge-ref form for this repository
and PR (`.../.github/workflows/check.yml@refs/pull/<number>/merge`). The verifier rejects a self-consistent
packet that claims another Rust release or renames that qualification boundary. The schema
is application-defined and versioned. It does not claim RFC 8785/JCS canonical
JSON compatibility.


## Trusted post-run witness

The PR-local V2 verifier is intentionally not treated as the final trust
anchor: a malicious PR could change that verifier together with the workflow.
A separate Qualification Witness workflow therefore runs on the default
branch after a successful Check workflow run.

The witness:

- executes only the trusted default-branch copy of
  tools/verify_qualification_run.py;
- reads the qualification artifact as untrusted data and never executes code
  from that artifact or the PR;
- independently checks the GitHub workflow-run result, PR association,
  provenance job, and required successful qualification steps;
- fetches the exact PR workflow, verifier, and Cargo.lock at the recorded
  source revision and recomputes their SHA-256 digests;
- independently recomputes the PR workflow, verifier, and Cargo.lock digests;
  it does not treat the PR-local verifier as a trusted authority;
- checks the qualification workflow against a trusted policy, including the
  exact-head checkout, required qualification commands, pinned action refs, and
  absence of OIDC/attestation write privileges.

Because workflow_run uses the workflow version present on the default branch,
this witness is a post-merge trust anchor for subsequent runs, not a claim
that the current PR-local verifier can authenticate its own execution. GitHub
documents that workflow_run executes from the default branch and exposes the
completed workflow's run metadata; workflow-job APIs expose individual step
conclusions for independent inspection.
