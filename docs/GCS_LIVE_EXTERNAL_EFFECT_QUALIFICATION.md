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

WIF profile v5 is exact rather than substring-matched. It requires google.subject
and the repository, repository ID, repository owner ID, environment, event,
workflow, ref, and workflow_ref mappings. The provider attribute condition must
equal the frozen conjunction in the profile.

The condition binds the live identity to refs/heads/main and the exact workflow
path on that ref. The workflow job independently refuses to run on any other
ref.

Before cloud authentication, the workflow requests a real GitHub OIDC JWT for
the exact Google provider-resource audience. A local parser checks the issuer,
audience, repository identity, protected environment, event, main ref,
workflow identity, workflow_ref, workflow_sha, source SHA, run ID, run attempt,
and branch type against the runner context. The parser records only those safe
claims and explicitly delegates JWT signature acceptance to the subsequent
Google WIF exchange; decoding the JWT locally is not itself cryptographic
verification. The selected claims and their digest are embedded in the final
qualification report, so later report verification can prove that the captured
run's token claims matched the recorded workflow/source context.

The verifier also requires the provider's OIDC `allowedAudiences` to be empty,
which activates Google's provider-resource default audience rule. The workflow
passes that exact provider URL as its explicit audience, so an alternate
configured audience cannot silently widen the trust surface.

The verifier also checks the target service account identity and IAM policy:
the service account must reside in the configured GCP project, and its only
Workload Identity User binding must be the repository-ID principal set for the
trusted pool. The project number and workload identity pool ID are derived from
the trusted provider resource.

This is trust configuration evidence only. It does not prove environment
approval honesty or Cloud Storage behavior. The workflow separately verifies
that `GITHUB_WORKFLOW_SHA` resolves to the same checked-out workflow blob, and
then verifies `GITHUB_SHA` against the checkout. These are source-provenance
gates for the captured run, not an IAM condition binding future workflow bytes. The
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
from refs/heads/main. It has id-token and attestations permissions, passes the
provider-resource URL as the explicit OIDC audience, and pins third-party
actions to immutable release commit SHAs. Before cloud authentication it
checks both the triggering commit and workflow-file provenance.

Google documents that google.subject is required for workload identity
providers and that service-account impersonation uses
roles/iam.workloadIdentityUser, which can be scoped to a principalSet based on
a mapped custom attribute. GitHub documents repository_id, repository_owner_id,
environment, event_name, workflow, ref, and workflow_ref claims for cloud
trust conditions.

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
