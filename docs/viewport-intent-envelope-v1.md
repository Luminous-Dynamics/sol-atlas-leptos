# Revision-bound viewport intent envelope v1

**Status:** framework-neutral protocol DTO and synthetic conformance fixture  
**Module:** `sol-atlas-core/src/field_work.rs`  
**Fixture:** `sol-atlas-core/fixtures/viewport-intent-envelope-v1.json`

## Purpose

Leptos owns the accessible work list, forms, evidence review, workflow and approvals. Bevy owns interactive spatial presentation and simulation. Neither framework owns canonical work-order truth. They need an explicit message contract when a click on a spatial object asks the application to focus an asset, select a component, move a timeline or request a comparison.

A typed intent alone is insufficient if the underlying view changes between render and click. The envelope binds the intent to the exact job revision and source snapshot the user saw.

## Envelope fields

- `schemaVersion`: protocol version independent of either UI/framework version.
- `requestId`: stable identity for exact retries; the local request journal should enforce uniqueness and preserve the original packet bytes/digest.
- `jobId`, `expectedJobRevision`: identify the work item and revision from the rendered projection.
- `expectedSourceRevision`, `expectedSourceDigestSha256`: bind the action to the source snapshot, not just an asset label.
- `intent`: the existing typed `ViewportIntentV1` command-shaped request.

The client validator rejects unsupported schema versions, missing request identity, malformed digest encoding, incorrect job identity, stale job/source revisions, stale source digests, and intents that were not offered by or do not target the same projection.

## Trust boundary

This envelope is **not authorization** and does not prove the actor is qualified, the site is safe, or the operation is valid. The authoritative command handler must load canonical state and independently check actor permissions, access policy, expected revisions, target existence, safety/compliance rules and evidence. If rejected as stale, the caller must fetch the new projection and require a fresh user action rather than retargeting the old click.

Stable retries reuse the same `requestId` and exact packet. A changed intent or revised projection requires a new request ID. The server should deduplicate by actor/request ID plus a canonical packet digest and reject reuse with a different payload; that server-side journal is not part of this DTO.

## Leptos and Bevy usage

- **Leptos:** receives the projection, displays the full textual identity/status and accessible controls, creates revision-bound intent envelopes, and handles stale/rejected responses visibly.
- **Bevy:** maps a spatially selected entity to a stable `assetId` already present in the projection, and emits an intent envelope through the host/application bridge. It must not submit raw ECS entity IDs as domain identity.
- **Host bridge:** serializes/deserializes the framework-neutral DTO and routes it to the application. It is not a privileged side channel.
- **Authority:** revalidates every command; a client-side `validate_for_projection` pass is only a stale-view and presentation-consistency check.

## Spatial placement is intentionally separate

The current `AssetRefV1` provides stable asset ID, asset revision, label and relationship. It does **not** carry geospatial coordinates, building-local transforms, floor/room anchors, coordinate reference systems or uncertainty bounds. The renderer must not invent a position or put an unlocated asset at the origin. Spatial rendering is enabled only when a separately versioned, source-backed spatial anchor is available, with its coordinate frame, units, transform/provenance and confidence known. Unlocated assets remain available in the Leptos list and can be selected textually.

## Acceptance tests and next work

The unit suite covers the frozen envelope fixture, exact revision/source matching, stale revision rejection, digest mismatch, wrong job, empty request IDs, unknown schema version, invalid digest encoding and unknown-field rejection. This does not yet test a live browser-to-native bridge or server authority.

Next integration steps:
1. Define a shared application event channel between Leptos and Bevy, with request/response correlation.
2. Add server receipt/rejection/conflict responses bound to request ID and authoritative revision.
3. Add fault injection for a stale scene click, late responses, duplicated events and changed source digests.
4. Only then introduce optional spatial anchors and a Bevy field-asset viewport.
