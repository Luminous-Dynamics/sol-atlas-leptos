# Engineering evidence visualization contract

**Status:** proposed UI/data contract; no orbital-ring result is currently endorsed by this document.  
**Audience:** Sol Atlas data, state and rendering components.

## Purpose

Sol Atlas may help people inspect the civil infrastructure and space-research evidence landscape. It must not transform incomplete data into apparent certainty. The globe is a presentation surface, not a scientific authority or physical solver.

The existing dataset model already distinguishes observed, curated and scenario data through per-layer provenance. Engineering evidence needs a richer record because source class alone cannot say whether a result is fresh, blocked, reproducible or independently reviewed.

## Required evidence-card fields

Every engineering object displayed from a research feed should carry:

- stable claim/case ID and schema version;
- human-readable claim, with scope and explicit non-claims;
- evidence class (source fact, measurement, derived result, model result, assumption or independent check);
- verification state, never inferred from geometry, color or the presence of a source URL;
- source repository, commit, path, Git blob and SHA-256 where available;
- input-manifest and result-artifact digests;
- source capture/retrieval time and evidence-generation time, with time zone;
- model/frame/units summary and relevant uncertainty;
- reviewer disposition and a link to the immutable record.

Missing metadata must stay missing and be shown as such; do not fabricate a capture date or silently equate an absent timestamp to the current date.

## State vocabulary and display

| Machine state | User-facing meaning | Must not be shown as |
|---|---|---|
| Blocked | Required inputs or definitions are unresolved | Ready, safe, feasible |
| NotRun | No analysis result exists yet | Passed or zero risk |
| Running / Queued | Computation has not completed | Passed |
| PassedAtSpecifiedFidelity | Named checks passed within the stated scope | General feasibility |
| Inconclusive | Evidence cannot decide the claim | Safe |
| Failed | A declared acceptance condition failed | Merely unavailable |
| Stale | Current source or inputs differ from verified fingerprints | Current verified result |
| Unavailable | Source or artifact could not be fetched/validated | Empty inventory or zero infrastructure |
| Disputed | Conflicting evidence awaits resolution | Consensus or verified truth |

The UI should expose both the current status and the last status tied to a specific artifact. A stale pass is still historical evidence, not a current pass.

## Empty, malformed and unavailable datasets are different

A valid empty dataset may mean zero records. A parse error, missing file, network outage, stale cache or rejected signature does not.

The current loader logs malformed JSON and may return a default empty collection so rendering can continue. That resilience is useful for ordinary presentation, but an engineering-evidence view must also surface a machine-readable dataset health result instead of treating an empty default as proof that no evidence or infrastructure exists.

Recommended envelope:

- **Loaded:** parse and schema checks passed; all candidate records were accepted. Report accepted count and source fingerprint.
- **PartiallyLoaded:** the source parsed, but one or more records were rejected by semantic validation. Report accepted and rejected counts and the rejection reason.
- **ValidEmpty:** parse and schema checks passed and the source explicitly contains zero records.
- **Malformed:** parse/schema validation failed; preserve the diagnostic and leave accepted/rejected counts unknown rather than zero.
- **Unavailable:** source could not be retrieved.
- **StaleCache:** a prior snapshot is being shown because refresh failed; preserve its original timestamp and identity.
- **Rejected:** signature, hash, authority receipt or source policy validation failed.

Per-dataset health should travel with the data into UI state. A console warning alone is not sufficient for safety-relevant evidence.

## Rendering and interaction rules

1. Keep observations, derived results, simulations, assumptions and planning scenarios visually distinguishable and text-labelled. Color may reinforce a label but cannot be the only cue.
2. Selecting a marker opens its exact source/evidence record and reveals the digest, freshness, scope, uncertainty and review state.
3. The UI must not recalculate or infer physical acceptance from displayed values. It only renders a verification decision produced by the designated checker.
4. A hidden, filtered, failed-to-load or unrenderable layer must have a visible reason. Do not collapse “not displayed” into “does not exist”.
5. Data older than its explicit freshness policy is marked Stale; do not automatically renew timestamps on cached records.
6. If records conflict, show the conflict and both immutable references rather than silently selecting the most recent, popular or highly rated record.
7. Accessibility: state labels and icon/text distinctions remain understandable without color; the evidence details are keyboard navigable.

## Orbital-ring example

A visualization may display an ideal circular-orbit reference at a specified altitude as a **T0 reference fixture**. It must not use that orbit line to imply the existence, equilibrium, stability, support load or survivability of a constructed orbital ring.

If the corresponding scenario manifest is Blocked / NotRun, those exact states remain visible in every view that uses the scenario. No animation, Phi/coherence score, trust score, green color, marker density or successful page load may promote the engineering state.

## Tests required before enabling engineering-evidence overlays

- malformed JSON and invalid schema remain visibly distinct from a valid empty dataset;
- stale fingerprints remain labelled Stale even if their historical tests passed;
- an unverified signature or incorrect content hash is Rejected, not merely warned;
- queued/running records cannot render as PassedAtSpecifiedFidelity;
- cache fallback preserves source snapshot time and reports refresh failure;
- all status distinctions can be read without color;
- renderer changes cannot mutate the underlying verification state.

## Boundary with other projects

- Symthaea produces source-bound analyses and claim-limited verification reports.
- Symtropy provides geometry/physics primitives; the display must not treat generic transform success as proof of correct orbital dynamics.
- Mycelix-DeSci preserves signed provenance, authority receipts, supersession and review history.
- Sol Atlas renders these records read-only and exposes missing, stale, conflicting or rejected evidence rather than resolving them by visual inference.
