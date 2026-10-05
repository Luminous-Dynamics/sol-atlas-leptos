# GCS live external-effect qualification

This lane is the first concrete real-service qualification target for
SOL-ATLAS-020 / #38.

## Claim boundary

The qualified profile is intentionally narrow:

- Cloud Storage object generation is the mutation-time fence.
- Object metadata carries the exact execution identity, input fingerprint,
  attempt identity, fence generation, and stable idempotency key.
- The adapter performs an exact read-back after mutation and after an
  acknowledgement-discarded request.
- The evidence applies only to the tested adapter, bucket, object API, and
  captured run. It is not a universal exactly-once claim.

Cloud Storage documents ifGenerationMatch as a service-side precondition:
a mutation proceeds only when the target generation matches, otherwise the
service returns HTTP 412. Cloud Storage also documents generation-preconditioned
object mutations as conditionally idempotent/retry-safe.

## Qualification cases

The frozen case set is:

1. current fence accepted;
2. exact stable-key replay does not mutate again;
3. stale generation rejected by Cloud Storage;
4. unestablished future generation rejected by Cloud Storage;
5. two concurrent writes with one generation permit exactly one service-side
   winner;
6. an idempotency key cannot be reused by another execution;
7. an execution identity cannot change the request under the same key;
8. an execution identity cannot silently change its idempotency key;
9. an acknowledgement-discarded mutation is reconciled by exact read-back;
10. reconciliation distinguishes the pre-application point from the applied
    point without treating absence as proof of non-commit.

Adding, removing, reordering, or redefining these cases requires a new case-set
identity.

## GCP setup

Use a dedicated qualification bucket. The service account needs object read,
create, overwrite, and delete capability only on that bucket. Cloud Storage
currently documents roles/storage.objectUser as providing create, read, update,
and delete access to objects; a narrower custom role is also suitable.

Configure GitHub repository or protected-environment variables:

- SOL_ATLAS_GCP_PROJECT_ID
- SOL_ATLAS_GCP_WORKLOAD_IDENTITY_PROVIDER
- SOL_ATLAS_GCP_SERVICE_ACCOUNT
- SOL_ATLAS_GCS_QUALIFICATION_BUCKET

Create the GitHub environment named sol-atlas-gcs-qualification and require
approval for it before granting access to the live qualification.

Use GitHub OIDC / Google Workload Identity Federation rather than a long-lived
service-account key. The trust policy should at minimum constrain:

- repository identity;
- the protected qualification environment;
- the expected workflow path;
- the expected event/ref policy.

Do not authorize arbitrary pull-request code to obtain the GCP identity.

## Evidence handling

The workflow produces a JSON report that binds:

- exact source commit;
- exact adapter Git blob;
- exact harness Git blob;
- exact workflow Git blob;
- exact profile digest;
- exact case-set identity;
- ordered observed case results;
- evidence digest;
- report digest.

The artifact is also emitted through GitHub artifact attestation. That
attestation is build provenance for the report; it does not prove that the
Cloud Storage service is truthful.

A queued GitHub Actions run is not qualification evidence. The live claim only
advances when the workflow executes the complete case vector and the generated
report re-verifies against the exact checked-out source.

## Local operation

A local operator with gcloud authentication can run:

    python3 scripts/qualify_gcs_external_effect.py \
      qualify \
      --bucket QUALIFICATION_BUCKET \
      --output artifacts/gcs-external-effect-report.json

Then verify the captured artifact:

    python3 scripts/qualify_gcs_external_effect.py \
      verify \
      --report artifacts/gcs-external-effect-report.json

The object names are unique to the run and are deleted with an exact generation
precondition after qualification. A cleanup failure makes the qualification
run fail rather than silently reporting success.

## Why GCS first

Cloud Storage exposes the exact primitive this boundary needs: a resource-side
generation guard checked when the mutation occurs. This is stronger evidence
than a client-side compare followed by a write, because a stale resumed caller
cannot bypass the fence merely by observing a fresh state earlier.

The adapter still keeps idempotency identity and reconciliation semantics above
that primitive. GCS generation fencing therefore qualifies one concrete
resource boundary without being mistaken for a general external-effect
transaction protocol.

## Frozen case corpus and source execution

The live case definitions are checked in at
`sol-atlas-policy-store-contract/conformance/gcs_external_effect_cases_v1.json`.
The qualifier hashes the complete corpus, including case descriptions, and
stores that digest in both the evidence and report. A change to case meaning,
order, or membership therefore invalidates prior external evidence even when
its case-set name is unchanged.

The qualification workflow pins its GitHub Actions dependencies to immutable
release commit SHAs. The live lane is manual and protected rather than
pull-request triggered because it has authority to obtain a real GCP identity.

GitHub documents that `workflow_dispatch` only receives events when the
workflow file exists on the default branch. After this workflow is merged to
that branch, run it first from the trusted default-branch workflow, then use a
GitHub API/CLI dispatch against a specific trusted ref when exact-head
qualification is required. Do not change this lane to a pull-request trigger
merely to make credentials available to review code.
