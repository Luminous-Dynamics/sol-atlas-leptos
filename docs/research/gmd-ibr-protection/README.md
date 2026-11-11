# GMD / IBR / CT / protection benchmark contract v0.1

**Date:** 11 November 2026  
**Status:** Scenario and waveform contracts; planned synthetic cases only.  
**Claim ceiling:** E0 until exact model artifacts, known-answer cases and independent validation are recorded.  
**Tracking:** [GMD-IBR-PROTECTION-001 #726](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/726).

## Why this lane matters

Geomagnetically induced current is a slowly varying quasi-DC disturbance, but its consequences can reach subsystems operating on very different time scales. Transformer DC bias and harmonics can affect current-transformer waveforms; CT behaviour affects relay measurements; relay misoperation can change AC network topology; and inverter current control, current limiting, momentary cessation and grid strength affect fault response and protection performance.

The benchmark must therefore connect physical evidence across time scales without pretending that one model fidelity can answer every question.

## Research and industry baselines

1. **UK NGET AIDPS (NIA2_NGET0068):** a live project (April 2025–June 2027) building EMT models for power-electronic devices and an RTDS/industrial-relay HIL methodology to study DC components from GIC and power-electronic devices on CT saturation and protection/control. Its public 2025/26 progress report records component models including SmartValves, STATCOMs, HVDC and Type-4 wind, and an 2-area/4-machine/11-bus RTDS benchmark. This is a serious ongoing external baseline; Luminous does not claim access to its models, RTDS setup or relay data. https://smarter.energynetworks.org/projects/nia2_nget0068/

2. **Piesciorovsky, Patel & Reno (2026):** a 230-kV radial/non-radial study of GIC neutral-blocking devices, inverter-based PV and distance-relay response. It means merely combining NBDs and IBRs is not novel. The opportunity is an evidence-qualified benchmark across more converter/CT/relay variants, network redistribution, service outcomes and operator action windows. https://doi.org/10.1016/j.epsr.2025.112190

3. **IEEE PES TR144 (8 July 2026):** addresses relay representation in fundamental-frequency phasor-domain / RMS dynamic simulation. It is a distinct fidelity lane from waveform-level EMT and physical relay HIL. A phasor-domain relay model must not be treated as evidence of CT secondary waveform distortion or CT saturation unless those phenomena are explicitly represented and independently validated. https://resourcecenter.ieee-pes.org/publications/technical-reports/pes_tr_144_psdp_070826

4. **NREL/TP-5D00-94430 (2025):** a broad IBR modelling/control study finds the type of DC source, average-vs-switching inverter model and some power-level control modes do not necessarily change relay response as much as faster current controls and current limiting. This supports targeted sensitivity analysis rather than an indiscriminate Cartesian explosion. https://doi.org/10.2172/3011876

5. **Texas A&M Texas2k Series24/Series25:** synthetic 2,000-bus study cases with dynamic and GMD parameters; Series25 reflects 2025-level wind, solar, batteries and additional transmission. These are synthetic systems, not the real Texas grid or a South African utility network. https://electricgrids.engr.tamu.edu/activsg2000-dynamics-cases-2024/ and https://electricgrids.engr.tamu.edu/texas2k-series25/

6. **COMTRADE:** preserve the exact declared format edition and exporter/conversion provenance for all transient records. IEEE identifies the old C37.111-2013 edition as inactive-reserved and a newer PC37.111 revision project as active. Do not label 2013 as the current edition by assumption, and do not silently convert waveform files. https://standards.ieee.org/ieee/C37.111/11052/ and https://standards.ieee.org/ieee/C37.111/3795/

## Paradigm shift: fidelity-qualified causal chains

The benchmark is not a leaderboard for “relay accuracy” in the abstract. It is a set of typed causal handoffs:

GMD forcing / network injection  
-> transformer DC-bias and harmonic response  
-> CT primary waveform  
-> CT secondary waveform and saturation  
-> relay input channels / internal estimates  
-> relay element pickup and timing  
-> trip / restraint / blocking logic  
-> AC network response  
-> service and restoration consequences.

Every handoff binds units, polarity, time base, asset/configuration revision, model fidelity, source digest, validity domain and uncertainty. A successful upstream result does not certify the downstream interface.

## Fidelity contract

| Lane | Appropriate evidence | Explicit limitation |
|---|---|---|
| Phasor-domain RMS | Fundamental-frequency electromechanical dynamics, declared relay element models, system voltage/stability consequences | Does not by itself establish distorted CT secondary waveforms, harmonic content, sub-cycle behaviour or CT saturation |
| EMT software simulation | Time-domain converter/transformer/CT/relay input waveforms and detailed transient interactions, when those models are present and validated | A software relay model is not a physical relay test |
| RTDS with controller HIL | Real-time simulator I/O with the identified physical relay/controller and measured trip/logic response | Only the tested hardware, settings, interfaces and scenario envelope are qualified |
| Power-HIL / laboratory | Physical power interface, CT/relay hardware and controlled test measurements | Laboratory boundaries are not field-grid generalisation |
| Field exercise | Owner-authorised exercise with approved scope, procedures and safety controls | Exercise-specific evidence only; no autonomous live operation |

Claims must declare their minimum fidelity. CT saturation waveform claims require a waveform-resolving CT representation (EMT or appropriately instrumented hardware test); physical relay operation claims require physical relay evidence. RMS is useful and should be retained, but must not be promoted into an EMT/HIL claim.

## Time-scale and signal integrity

- GIC / transformer DC-bias inputs and relay/fault waveforms may use different time scales and sample rates.
- Preserve original sample clocks, synchronization group, clock uncertainty and per-channel start offsets.
- Do not resample or align channels without a declared transformation record and input/output digest.
- Polarity is explicit for every current channel. Never silently reverse signs to improve correlation.
- CT ratio, core/magnetization curve, burden and remanence state are explicit configuration inputs.
- Relay settings and relay firmware/model revisions are pinned independently of network revisions.
- Missing waveform channels or a configuration mismatch must lower the evidence class or reject the case.

The waveform manifest intentionally requires the provider-declared COMTRADE edition/exporter rather than assuming a fixed edition. IEEE's current C37.111 revision project and the inactive-reserved status of C37.111-2013 make version pinning important. The checked-in CSV example is a digest-pinned 60-Hz synthetic format fixture only; its ideal CT ratio and short trace are not a validated electrical/EMT response.

## Initial scenario set

The machine-readable scenario matrix contains twelve E0 synthetic design cases:
1. No-GMD conventional baseline.
2. GMD conventional baseline.
3. IBR-only grid-following weak-grid control.
4. GMD + grid-following IBR, no neutral blocker.
5. Matched radial GMD + IBR + neutral-blocking device.
6. Matched non-radial GMD + IBR + neutral blocker.
7. GMD + grid-forming IBR under low-SCR conditions.
8. GMD-related CT bias with a timed fault.
9. N-1 network / backup-protection condition.
10. Stale GIC telemetry negative control.
11. Common-mode sensor error negative control.
12. Current-control / current-limit / momentary-cessation sensitivity cases.

These are scenario definitions, not simulation outputs. No vendor model, owner relay setting, AIDPS data, utility topology or field waveform is included or implied.

## Metric families

**Transformer / CT:** GIC waveform amplitude and duration, transformer DC-bias/harmonic metrics, CT primary-secondary ratio error, saturation interval, waveform distortion, peak secondary error and phase displacement.

**Protection:** pickup and restraint state, apparent impedance/reach, directional decision, differential operating/restraint quantities, trip and backup-trip time, false-trip and failure-to-trip counts with denominators, protection coordination margin.

**IBR / network:** current-limit activation, positive/negative/zero-sequence injection where modelled, voltage and frequency excursion, reactive response, momentary cessation, weak-grid sensitivity, NBD-induced GIC redistribution and network-wide equipment stress.

**Service / action:** worst-service floor deficit, unserved quantity, action completion margin, resource and human authorization state, restoration time and independently evidenced recovery.

No single scalar may conceal a protection failure, a hard service-floor breach or a worse network-wide stress transfer.

## Required baselines and experimental design

- Matched No-GMD / GMD-only / IBR-only / GMD+IBR pairs.
- Neutral-blocker on/off under identical network, disturbance, relay settings and dispatch.
- Radial/non-radial topology pairs.
- GFL/GFM, fast/slow current control, current-limit and momentary-cessation variants.
- RMS/EMT model pairs where the same case can be represented faithfully in both.
- Published 230-kV NBD/IB-PV study as a reproduction target when sufficient published inputs/results exist.
- AIDPS public progress as a programme comparator, not as an assumed shared benchmark dataset.
- Owner-approved relay settings and physical devices only when explicitly supplied.

Use a staged design of experiments / sensitivity analysis to focus on high-influence parameters (current control, current limiting, weak-grid strength, CT remanence/burden, NBD state, relay settings). Avoid a full Cartesian explosion before screening. Apply event/parameter holdouts for generalisation claims.

## Negative controls

Wrong GIC polarity/unit; misaligned clocks; stale waveform; mismatched CT ratio or burden; missing/remanence-unqualified CT magnetization curve; relay settings from another revision; missing NBD model; wrong neutral path; wrong inverter mode/current limit; wrong grid strength; incorrect fault topology; sample-rate conversion without provenance; common-mode sensors declared independent; RMS-only claim of CT saturation; simulated relay output labelled as physical relay operation.

Positive controls are required as well: a valid paired case must be accepted, and the declared expected relationships must be reproduced. Rejecting all cases is not qualification.

## E0-E7 evidence boundary

- E0: scenario definition / planned matrix.
- E1: schema, units, timebase, version and provenance checks.
- E2: analytical known-answer and negative-control software tests.
- E3: published case reproduction with exact model/configuration and artifact digests.
- E4: independently validated EMT/RMS model comparison in a declared domain.
- E5: physical relay/CT controller-HIL, RTDS or Power-HIL evidence for the exact test setup.
- E6: owner-authorised bounded field or commissioning exercise.
- E7: repeated operational evidence across network, device and operating-condition variations.

A synthetic scenario matrix remains E0. Schema validation does not validate model physics.

## Next implementation sequence

1. Pin the exact Texas2k Series24/Series25 artifact files and hashes only after downloading and inspecting the provider release/terms.
2. Add concrete scenario manifests using the scenario schema; do not fill unknown relay/transformer/CT parameters with invented defaults.
3. Add waveform manifests when real or simulated waveform artifacts exist.
4. Build a dependency-light Rust contract checker for manifest-level unit, polarity, clock, CT-chain and fidelity/claim invariants.
5. Add analytic and published known-answer CT/relay cases.
6. Add the RMS-vs-EMT paired cases and show exactly which outputs diverge or become unobservable at lower fidelity.
7. Seek partner access for physical relays/RTDS and independent review.
8. Connect qualified results to #703 network mitigation, #717 service continuity and #723 decision value.

## Explicit non-claims

This contract does not claim access to NGET/AIDPS internal models or RTDS, physical relay settings, manufacturer CT magnetization data, a real Texas or South African grid model, measured GMD+IBR protection waveforms, or field qualification. It does not issue protection-setting changes, switching instructions or live control commands.
