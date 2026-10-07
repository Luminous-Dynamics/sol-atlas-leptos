# GCS live external-effect qualification

This lane is the first concrete real-service qualification target for
SOL-ATLAS-020 / #38.

## Claim boundary

The qualified profile is intentionally narrow:

- Cloud Storage object generation is the mutation-time fence.
- Object metadata carries the exact execution identity, input fingerprint,
  attempt identity, fence generation, and stable idempotency key.
- Replay protection applies while that live object identity state remains retained;
  delete/recreate is outside the qualified replay guarantee.
- The adapter performs exact read-back after mutation and after an
  acknowledgement-discarded request.
- Media read-back is conditional on both generation and metageneration, and a
  bounded retry handles concurrent metadata churn without accepting a torn
  metadata/data snapshot.
- The evidence applies only to the tested adapter, bucket, object API, and
  captured run. It is not a universal exactly-once claim.

## GCP trust boundary

WIF profile v8 is exact rather than substring-matched. It requires google.subject
and the repository, repository ID, repository owner ID, environment, event,
workflow, ref, and workflow_ref mappings. The provider attribute condition must
equal the frozen conjunction in the profile.

The condition binds the live identity to refs/heads/main and the exact workflow
path on that ref. The workflow job independently refuses to run on any other
ref.

Before cloud authentication, the workflow first queries GitHub's server-side
workflow-run record with the read-only Actions API and verifies the exact run ID,
run attempt, repository ID, source SHA, branch, event, workflow identity, and
absence of unexpected reusable workflows. It also resolves the server-side
workflow record and verifies the exact workflow path and name. This evidence is
stored and later bound into the qualification report.

The workflow then requests a real GitHub OIDC JWT for
the exact Google provider-resource audience. A local parser checks the issuer,
audience, repository identity, protected environment, event, main ref,
workflow identity, workflow_ref, workflow_sha, source SHA, run ID, run attempt,
runner environment, and branch type against the runner context. The accepted
runner environment is exactly `github-hosted`. The parser also requires the
standard JWT `iat`, `nbf`, and `exp` NumericDate claims to describe a
non-empty validity interval and validates them at token receipt with a bounded
60-second clock-skew allowance. The verifier records a SHA-256 fingerprint of
the exact JWT bytes and writes the raw token only to the runner's transient
temp directory.

The verifier does not perform local JWT signature verification. The workflow
constructs a Google external-account credential configuration whose
file-sourced subject token is that exact JWT file. A dedicated verifier checks
the generated configuration's provider audience, JWT subject-token type, STS
URL, token-file path, and service-account impersonation URL, and checks that
the token-file fingerprint equals the fingerprint from the parsed JWT. The
workflow then runs `gcloud auth login --cred-file` against that configuration,
so the WIF exchange consumes the same verified JWT bytes rather than requesting
a second OIDC token.

The verifier also requires the provider's OIDC `allowedAudiences` to be empty,
which activates Google's provider-resource default audience rule. The workflow
passes that exact provider URL as its explicit audience, so an alternate
configured audience cannot silently widen the trust surface.

The verifier also checks the target service account identity and its direct IAM
policy: the service account must reside in the configured GCP project, and its
only direct Workload Identity User binding must be the repository-ID principal
set for the trusted pool. The project number and workload identity pool ID are
derived from the trusted provider resource.

This is intentionally a direct resource-policy assertion, not a proof of
effective IAM uniqueness across the Google Cloud resource hierarchy. Google
Cloud allow policies can apply at the project, folder, organization, and other
ancestor levels, so an inherited binding can affect effective access without
appearing in the service account's direct IAM policy. Covering that hierarchy
requires a separate privileged policy-analysis lane; this qualification does not
silently claim that stronger property. Freezing the exact provider-resource and
service-account identities is also tracked separately in #41.

This is trust configuration evidence only. It does not prove environment
approval honesty or Cloud Storage behavior. The workflow separately verifies
that `GITHUB_WORKFLOW_SHA` resolves to the same checked-out workflow blob, and
then verifies `GITHUB_SHA` against the checkout. These are source-provenance
gates for the captured run, not an IAM condition binding future workflow bytes. The
qualification pool is intentionally single-provider, and the service-account
WIF binding is intentionally exclusive to the repository-ID principal.

## Qualification cases

The active frozen case set is v2; v1 remains historical and is superseded:

1. current fence accepted;
2. exact stable-key replay does not mutate again;
3. stale generation rejected by Cloud Storage;
4. unestablished future generation rejected by Cloud Storage;
5. two concurrent writes with one generation permit exactly one service-side
   winner;
6. an idempotency key cannot be reused by another execution;
7. an execution identity cannot change the request under the same key;
8. an execution identity cannot silently change its idempotency key;
9. a metadata change between metadata and media reads is detected through
   metageneration preconditions and the read-back converges to one coherent
   snapshot;
10. an acknowledgement-discarded mutation is reconciled by exact read-back;
11. reconciliation distinguishes a genuinely absent pre-application point from
    the exact applied point; absence in the indeterminate-ack path remains
    insufficient to prove non-commit.

Adding, removing, reordering, or redefining these cases requires a new case-set identity.

## Evidence handling

The workflow report (schema v6) binds the exact checked-out source commit
(`GITHUB_SHA`), the workflow SHA/ref, adapter/harness/workflow blobs, GitHub
server workflow-run verification/digest, WIF verification/profile digest,
case-set identity, ordered observed results, OIDC
v4 claim record/digest, exact-token fingerprint, WIF credential-config
verification/digest, evidence digest, and report digest. The live workflow
asserts that `git rev-parse HEAD` equals `GITHUB_SHA` before cloud
authentication, and report verification checks the same equality.


After transient-credential cleanup, the qualification job uploads only an
explicit five-file allowlist of non-secret evidence through the pinned GitHub
artifact action and attests the report. The uploader's unique artifact ID is
passed as a small job output to a separate publication job. Publication has only `contents: read`,
downloads by that exact artifact ID with digest-mismatch failure enabled, and
re-verifies the report against the exact source checkout. Publication has
only `contents: read` and `attestations: read`; it cannot request a GitHub
OIDC token. Publication then runs `gh attestation verify` against the
downloaded report, requiring the exact qualification workflow as signer, the
captured `GITHUB_SHA` as source digest, the captured ref, and the GitHub OIDC
issuer, the explicit SLSA provenance predicate, and a GitHub-hosted-runner
requirement. The OIDC evidence itself also records and verifies the
`runner_environment` claim as `github-hosted`. This turns the attestation from
a generated side artifact into an independently checked evidence boundary.

A queued Actions run is not qualification evidence. The live claim advances
only when the complete case vector executes and the generated report
re-verifies against the exact checked-out source.

## Operations

Use a dedicated qualification bucket and a protected GitHub environment named
sol-atlas-gcs-qualification. Keep GCP values in protected environment/repository
variables.

The live workflow remains workflow_dispatch-only and intentionally runs only
from refs/heads/main. The qualification job has only `contents: read`, `actions: read`, and
`id-token: write`; it passes the provider-resource URL as the explicit OIDC
audience and pins third-party actions to immutable release commit SHAs. The
Cloud SDK is also pinned to the exact published version 586.0.0 rather than a
floating version constraint. Before cloud authentication it checks both the
triggering commit and workflow-file provenance. The setup-gcloud action runs
before OIDC token materialization. After token materialization, no external
action executes until transient credentials are removed. The pinned artifact
upload and attestation occur only after cleanup. Publication is a separate job
without `id-token: write`, and its attestation verification uses read-only
attestation access.

Google documents that google.subject is required for workload identity
providers and that service-account impersonation uses
roles/iam.workloadIdentityUser, which can be scoped to a principalSet based on
a mapped custom attribute. GitHub documents repository_id, repository_owner_id,
environment, event_name, workflow, ref, and workflow_ref claims for cloud
trust conditions.

## OIDC evidence version

The live OIDC evidence format is `sol-atlas:github-oidc-claims:v4`; OIDC v1,
v2, and v3 formats are historical. The outer GCS qualification report is schema
v5 because it now binds the exact verified JWT to the file-sourced WIF
credential configuration used for the exchange.


## Frozen case corpus

The live case definitions are checked in at
sol-atlas-policy-store-contract/conformance/gcs_external_effect_cases_v2.json.
The qualifier hashes the complete corpus, including descriptions and order,
and stores that digest in both evidence and report. A change to case meaning,
order, or membership therefore invalidates prior external evidence.

## Why GCS first

Cloud Storage exposes a resource-side generation guard checked at mutation time.
The adapter keeps idempotency identity and reconciliation semantics above that
primitive, so the qualification is one concrete resource boundary, not a
general exactly-once external-effect proof.
