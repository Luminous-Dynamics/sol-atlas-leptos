# EO-USABLE-CADENCE-001 — usable observation cadence benchmark v0.1

**Parent:** [Sol-Atlas #804 — EO-USABLE-CADENCE-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/804)  
**Integration:** [SATELLITE-CHANGE-EVIDENCE-001 / #795](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/795), [EO claim contract / PR #799](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/799), [Symthaea locked Sentinel witness / #194](https://github.com/Luminous-Dynamics/symthaea/issues/194)  
**Status:** v0.1 offline timeline model and synthetic tests drafted; Rust/schema CI not run  
**Claim ceiling:** structural/synthetic cadence semantics only; no real-world cadence, cloud statistics, sensor performance or actionable-detection advantage is claimed.

## 1. Why nominal revisit is the wrong unit

For any target/support and phenomenon class:

`nominal orbital revisit \ne actual acquisition cadence \ne usable observation cadence \ne event-time resolution \ne time-to-actionable-claim`

A nominal constellation schedule says when a pass may be available. It does not say that the target was acquired, that the data arrived, that the image passed QA, that the requested use was authorized, that the sensor could detect the phenomenon at the target's scale, or that an independent reviewer could act on the claim.

The unit of monitoring performance should therefore be **qualified useful observation opportunity**, with all missed/obscured/restricted states preserved.

## 2. Timeline record semantics

The Rust timeline binds each opportunity to:
- canonical timeline/support identity;
- source-registry and sensor identity;
- modality (optical, SAR, thermal, hyperspectral, multispectral or other);
- expected acquisition time, only when a source schedule is actually available;
- actual acquisition interval, if an acquisition occurred;
- canonical AssetId and source-native event identity, where acquired;
- acquisition/quality outcome;
- delivery/availability time, distinct from acquisition time;
- optional observation assessment;
- external coverage and detection-qualification receipt digests for promoted claims.

`RightsBlocked` is an upstream disposition from the authorized rights/custody adapter. The cadence scorer does not interpret licences or authorize use. `NoAcquisition` is distinct from cloud contamination, processing invalidity, out-of-domain, rights restriction and late delivery. Duplicate source-event identities are rejected so a mirrored/late catalogue row cannot shorten apparent revisit time.

Unix milliseconds are a normalized comparison representation only. Raw provider timestamps, time scale, conversion and time-normalization receipt remain in the source/processing evidence. The timeline does not authenticate those timestamps.

## 3. Metrics and exact meanings

### Per sensor
For each sensor identifier, report:
- expected opportunity count and expected schedule gaps (only when expected times are known);
- actual acquired count and actual acquisition gaps;
- qualified usable count and usable-observation gaps;
- rights-blocked opportunity count.

This is essential: a combined optical/SAR portfolio may provide useful coverage, but it must not conceal a long gap in either individual sensor's acquisition process.

### Fused portfolio
Report the same gap families over the combined set, clearly named as portfolio metrics. They are not a substitute for per-sensor distributions.

### Availability and delay
- usable fraction of known expected opportunities = usable observations with an expected time / opportunities with known expected times;
- usable fraction of acquired scenes = qualified usable / acquired;
- delivery latency = product-availability time minus acquisition-end time, reported as a distribution separately from revisit.

A missing expected schedule is not a zero revisit gap and is not silently synthesized from nominal constellation marketing.

### Event-time resolution
For each first positive change assessment, locate the latest prior qualified `NoChangeWithinEnvelope` assessment on the same support. Emit:
- exclusive lower bound: completion of that last qualified no-change acquisition;
- inclusive upper bound: start of the first qualified positive acquisition;
- interval width.

Intervening cloudy, invalid, rights-blocked, out-of-domain, duplicate or unobserved opportunities do not narrow the interval. If no qualified positive assessment exists, the scorer emits no interval; it does not invent a hidden change or assume stability. A short-lived event that occurs and reverses between usable acquisitions therefore remains a potential miss for a separate hidden-truth challenge, not an inferred event.

### Time to actionable claim
The scorer records the earliest delivery time of a qualified positive or negative assessment. It does not yet include reviewer latency, alert transport or decision-policy utility; those belong in later integrated tournament stages.

## 4. Initial deterministic synthetic case

The committed fixture `sol-atlas-core/tests/fixtures/earth_observation_cadence/optical_cloud_gap_sar_bridge.json` contains four expected opportunities over one abstract support:
1. usable optical scene;
2. cloud-contaminated optical scene;
3. usable SAR scene during the optical gap;
4. an expected optical opportunity with no acquisition.

The expected summary is:
- 4 opportunities;
- 3 acquired scenes;
- 2 quality-qualified usable observations;
- 1 no-acquisition opportunity;
- 50% usable fraction of known expected opportunities;
- one fused usable-observation gap of six synthetic days.

This validates the timeline contract and metric wiring only. It is not a claim about actual Sentinel-1/2 cadence, cloud availability or geographic coverage.

## 5. Adversarial conformance cases

The executable tests cover:
- a long optical cloud gap despite nominal five-day opportunities;
- SAR providing a distinct usable observation in that optical gap;
- event-time bounds using only qualified no-change and positive evidence;
- missing coverage/detection qualification blocking a no-change assessment;
- duplicate/replayed source-event identity rejection;
- rights-blocked and no-acquisition opportunities excluded from usable cadence;
- delivery latency kept separate from revisit;
- no positive evidence does not create an inferred transient event;
- deserialization re-runs opportunity and timeline invariants.

The next fixtures should cover seasonal phenology, false changes across sensor/processing-baseline transitions, short event/reversal between acquisitions, late-arriving corrections, source-rights revocation, conflicting sensors with common registration bias, and sensor-specific minimum mapping units.

## 6. Qualification boundaries and follow-on work

- Sol-Atlas owns timeline, support, source references, cadence metrics and event-time bounds.
- The EO claim contract (#799) carries source/processing/rights references and binds the independently qualified detection envelope.
- Symthaea #194 remains the authority for Sentinel data manifests, spatial/time splits, fitted artifacts, candidate selection and evaluation custody.
- Symthaea #12075 owns competing hypotheses and follow-up investigation.
- Symtropy qualifies applicable sensor/physical models and model discrepancy.
- Mycelix/custodian verifies purpose-specific rights/custody and external receipts.

This module does not authenticate the rights decision or detection-qualification receipt. Their digests are references; authoritative verification remains external.

## 7. Next benchmark gates (targets, not results)

1. 100% deterministic replay of the frozen synthetic timeline corpus.
2. 0 no-change assessment accepted without coverage and target/sensor detection qualification receipts.
3. 0 duplicate/replayed source events counted as independent acquisitions.
4. 0 no-acquisition, obscured, invalid, out-of-domain or rights-blocked opportunities counted as usable.
5. Report nominal/expected, actual, per-sensor usable and fused-portfolio cadence separately.
6. On hidden-event fixtures, every emitted interval must contain the planted onset; report interval width, unresolved-event rate and transient-miss rate.
7. Compare nominal scheduling, quality-aware scheduling, event-risk scheduling, entropy/EIG and existing EVSI/ENBS only under equal feasible action sets and equal eligible budget.
8. Proposed stretch hypothesis: reduce P90 time-to-actionable-claim by at least 20% versus nominal schedule at equal eligible cost, after baseline variance and statistical power are measured.

No superiority claim is promoted from this synthetic corpus. Retrospective/real Sentinel evaluation must use the locked Symthaea #194 protocol and independently admitted data/labels.
