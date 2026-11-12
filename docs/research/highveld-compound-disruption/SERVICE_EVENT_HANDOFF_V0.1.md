# Highveld evidence-to-service-event handoff v0.1

**Status:** proposed consumer boundary for review by Symtropy #1848/#1849 and Sol-Atlas #700. This file does not alter or redefine either owner contract and does not implement an adapter.

## The critical semantic boundary

An upstream disruption is not automatically a household-service loss.

- Grid interruption is an electricity-network condition.
- Municipal water notice is a source/area notice, not a tap-delivery measurement.
- Component fault is an equipment condition.
- Heat, cold, rainfall and outdoor-air events are external conditions.
- A household service is the outcome delivered to a named resident/task under the declared initial state, resource reserve, controls and evidence scope.

The system must not translate an upstream condition to a service loss unless an owner-qualified service trajectory or admitted outcome evidence shows that the named service crossed its declared floor. Likewise, upstream restoration is not service restoration.

## Owner-native boundary

Symtropy #1848/#1849 remains the sole owner of its deterministic event clock, paired-policy trajectories, service floors and stable-recovery evaluation.

The inspected owner schema and implementation define the native timeline event as:

- `id`: string;
- `generation`: unsigned 64-bit integer (`u64`);
- `tick`: unsigned 32-bit integer (`u32`);
- `sequence`: unsigned 32-bit integer (`u32`), unique within a tick;
- `kind`: exactly `restored` or `lost`.

Implementation references: [owner schema](https://github.com/Luminous-Dynamics/symtropy/blob/research/spatial-resilience-trajectory-benchmark-2026-10/docs/research/spatial-design/SPATIAL_RESILIENCE_TIMELINE_ORACLE_V0_1.schema.json) and [owner Rust implementation](https://github.com/Luminous-Dynamics/symtropy/blob/research/spatial-resilience-trajectory-benchmark-2026-10/tools/spatial-resilience-qualifier/src/lib.rs).

The owner implementation also establishes these semantics:

- Exact duplicate events with identical `id`, `generation` and payload are ignored; a conflicting payload for the same `(id, generation)` is rejected.
- A reused `(tick, sequence)` ordering slot is rejected as ambiguous; simultaneous transitions need a declared, unique sequence.
- A loss at the candidate stable-recovery tick invalidates that recovery candidate. The stability window must be positive, and events beyond the declared horizon are rejected.
- The implementation's timeline is a synthetic finite-state availability oracle, not a physics/service-quality measurement model.

This handoff does not derive those owner-native values from source intervals. In particular:

- never assume one external second equals one owner tick;
- never map source event IDs directly into native event identity without an explicit adapter identity rule;
- never synthesize `generation` or `sequence` from source ordering;
- never add new `kind` values for grid, water, heat, parts or component faults;
- preserve raw source/evidence events and digests in the separate evidence crosswalk; the native event is not a replacement for source provenance.

Any concrete executable translation requires an exact owner-approved adapter contract and exact-head test receipt. Until that exists, the state is OWNER_MAPPING_NOT_IMPLEMENTED.

## Projection decision table

| Upstream evidence or state | Direct service event allowed? | Required interpretation |
|---|---|---|
| Regional heat/cold or weather record | No | External condition only. A thermal service/comfort floor can be evaluated only by its qualified model or measurement owner. |
| Eskom national/system-level supply record | No | System context only; not a feeder, dwelling or household electricity observation. |
| Feeder/dwelling grid interruption record | No, not by itself | Electricity availability may be interrupted while a service remains available through reserve, local generation or alternate control. |
| Johannesburg Water area/system notice | No | Area/system context does not prove tap or end-use delivery. |
| Tap/connection water-flow interruption | Not by itself | Water service loss requires a declared end use, admitted quantity/quality profile and a service-floor breach; reserve and permitted alternative sources matter. |
| HeatBank, AirCare, WaterLoop or shared-plant component fault | No, not by itself | Fault is not proof that delivered service crossed its floor. |
| Qualified service trajectory crosses a declared hard floor | Eligible for owner mapping | Only the named service/resident/task and exact owner-approved transition semantics may be projected. |
| Qualified service resumes above the declared floor | Eligible for owner mapping | A restored transition may be emitted only under the owner's exact event and clock rules. |
| First restoration without stable recovery | Do not conflate | First restored transition and stable recovery remain distinct outcomes; repeated faults can follow. |
| Missing, censored, conflicting, stale, revoked or out-of-scope evidence | No | Preserve UNKNOWN/PARTIAL/BLOCKED semantics; do not fabricate a transition. |

## Required preconditions before any projection

1. **Identity:** exact named service, resident/task scope, requirement/profile revision and architecture candidate revision are known.
2. **Outcome:** a qualified owner-native trajectory or admitted measurement shows the service state, not merely an upstream event.
3. **Floor:** the service floor, units, interval and resident authority are present and unchanged for the decision.
4. **Evidence:** source/model/instrument identity, revision, rights, coverage, time basis, uncertainty and claim ceiling are valid under the relevant owner.
5. **Causality:** no causal edge is inferred from overlap. A service may be affected by an upstream event only where the dependency is modelled or supported by evidence.
6. **Time mapping:** a deterministic owner-approved mapping exists between evidence time and the owner's tick/generation/sequence rules. Ambiguous or unqualified time blocks projection.
7. **Pairing:** every architecture receives the same exact exogenous trace and declared information budget; policy-visible information is filtered by availability time.
8. **Replay:** duplicate/out-of-order delivery behaviour and canonical replay are qualified by the owner; this package does not invent idempotency semantics.

## Service-specific implications

- **HeatBank:** grid loss may cause reduced electrical input while stored heat continues to meet hot-water draws. Record electrical availability, thermal reserve, delivered draw and service floor separately. Grid return does not prove stored reserve has recovered.
- **AirCare:** grid/network loss may affect fan/control capability, but a component or network fault is not itself proof of pollutant-specific clean-air delivery failure. The outcome requires the named occupied position, pollutant, reference method and service profile.
- **WaterLoop:** municipal interruption, source-tank status, treatment fault and tap delivery are distinct. A local reserve may preserve an authorized non-potable end use; no potable or health-protection claim is implied.
- **Shared plant:** a single fault can affect several services, but each delivered service trajectory and per-resident floor remains separately represented. Do not infer a common cause from overlapping intervals.

## Required owner review questions

Before executable integration, request explicit answers from Symtropy #1848/#1849 and Sol-Atlas #700:

1. Which exact trajectory output proves a named service floor crossing and what evidence class/uncertainty does it carry?
2. What exact identity rules govern native event `id`, `generation`, `tick` and `sequence`?
3. What clock/tick mapping and duplicate/out-of-order semantics are already qualified?
4. Is service-floor breach represented at resident/task, service, or both scopes? How are simultaneous multi-service losses represented?
5. What exact owner receipt binds the projected event to the source evidence and trajectory result?
6. How is first restoration distinguished from stable recovery and repeated failure?
7. What happens when resident requirements change during a trace or when an evidence dependency is revoked?

Unknown answers remain OWNER_CONTRACT_NOT_YET_EXPOSED; do not guess the field names or add parallel native types.

## Negative controls

An executable adapter must reject or refuse projection for at least:

- grid loss while the declared service remains above its floor on reserve;
- grid restoration while the service remains below its floor;
- municipal water notice with no tap/end-use evidence;
- component fault with no delivered-service consequence;
- heat/cold event without a qualified thermal outcome;
- coarse system interval overlapping a fine local record with incompatible scope;
- future event present in the full trace but unavailable at decision time;
- source revision, rights state or model/geometry change invalidating the supporting outcome;
- conflicting or censored evidence that cannot establish a transition;
- two service trajectories with a shared upstream interruption but different service outcomes.

## Acceptance target

- 100% exact agreement with the owner-approved reference mapping on finite synthetic adapter cases.
- 0 direct source-event-to-service-event promotions without qualified service outcome evidence.
- 0 source/system scope escalations.
- 0 guessed tick/generation/sequence mappings.
- 0 future-information leaks.
- 100% source evidence digest and owner receipt binding for every projected native event.
- First restoration, stable recovery and repeated failure remain distinct in all reports.

## Claim ceiling

This handoff specifies proposed semantic safeguards only. It establishes no physical service outcome, causal dependency, household reliability, health/safety result or qualified executable adapter. The authoritative native contracts and implementation remain with their existing owners.
