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

WIF profile v4 is exact rather than substring-matched. It requires google.subject
and the repository, repository ID, repository owner ID, environment, event,
workflow, ref, and workflow_ref mappings. The provider attribute condition must
equal the frozen conjunction in the profile.

The condition binds the live identity to refs/heads/main and the exact workflow
path on that ref. The workflow job independently refuses to run on any other
ref.

The verifier also checks the target service account identity and IAM policy:
the service account must reside in the configured GCP project, and its only
Workload Identity User binding must be the repository-ID principal set for the
trusted pool. The project number and workload identity pool ID are derived from
the trusted provider resource.

This is trust configuration evidence only. It does not prove workflow source
immutability, environment approval honesty, or Cloud Storage behavior. The
qualification pool is intentionally single-provider, and the service-account
WIF binding is intentionally exclusive to the repository-ID principal.

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
9. a metadata change between metadata and media reads is detected through
   metageneration preconditions and the read-back converges to one coherent
   snapshot;
10. an acknowledgement-discarded mutation is reconciled by exact read-back;
11. reconciliation distinguishes the pre-application point from the applied
    point without treating absence as proof of non-commit.

Adding, removing, reordering, or redefining these cases requires a new case-set identity.

## Evidence handling

The workflow report binds the exact checked-out source commit (`GITHUB_SHA`),
the workflow SHA/ref, adapter/harness/workflow blobs, WIF verification/profile
digest, case-set identity, ordered observed results, evidence digest, and report
digest. The live workflow asserts that `git rev-parse HEAD` equals `GITHUB_SHA`
before cloud authentication, and report verification checks the same equality.

The artifact is also emitted through GitHub artifact attestation. That
attestation is provenance for the report; it does not prove that Cloud Storage
is truthful.

A queued Actions run is not qualification evidence. The live claim advances
only when the complete case vector executes and the generated report
re-verifies against the exact checked-out source.

## Operations

Use a dedicated qualification bucket and a protected GitHub environment named
sol-atlas-gcs-qualification. Keep GCP values in protected environment/repository
variables.

The live workflow remains workflow_dispatch-only and intentionally runs only
from refs/heads/main. It has id-token and attestations permissions and pins
third-party actions to immutable release commit SHAs.

Google documents that google.subject is required for workload identity
providers and that service-account impersonation uses
roles/iam.workloadIdentityUser, which can be scoped to a principalSet based on
a mapped custom attribute. GitHub documents repository_id, repository_owner_id,
environment, event_name, workflow, ref, and workflow_ref claims for cloud
trust conditions.

## Frozen case corpus

The live case definitions are checked in at
sol-atlas-policy-store-contract/conformance/gcs_external_effect_cases_v1.json.
The qualifier hashes the complete corpus, including descriptions and order,
and stores that digest in both evidence and report. A change to case meaning,
order, or membership therefore invalidates prior external evidence.

## Why GCS first

Cloud Storage exposes a resource-side generation guard checked at mutation time.
The adapter keeps idempotency identity and reconciliation semantics above that
primitive, so the qualification is one concrete resource boundary, not a
general exactly-once external-effect proof.
