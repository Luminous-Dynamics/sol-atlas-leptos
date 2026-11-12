# GMD protection envelope qualification v0.1

**Date:** 12 November 2026  
**Tracking:** [GMD-PROTECTION-ENVELOPE-001 #730](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/730)  
**Parent:** [GMD-IBR-PROTECTION-001 #726](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/726), [PR #729](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/729)  
**Claim ceiling:** E2 synthetic verifier logic only.

## The core rule

A passing point sample does not certify the surrounding operating region. One qualified counterexample is sufficient to show that a cell is not wholly misoperation-free; a finite set of passing points is not sufficient to prove that it is.

Let the declared parameter domain be partitioned into cells (C_i). The verifier assigns each cell one of three non-compensatory dispositions:

- **CERTIFIED:** complete digest-bound evidence covers the entire cell and declared uncertainty set; the required evidence capabilities are present; false-trip and failure-to-trip upper bounds are zero; coordination margin meets its owner-defined minimum; every required service-floor margin is non-negative.
- **EXCLUDED:** qualified evidence contains a counterexample or bound that violates a protection or service constraint.
- **UNKNOWN:** the cell lacks full-cell coverage, complete metrics, digest-bound evidence, or a capability required by the claim.

The certified set is therefore only the union of cells that meet the declared contract. Unknown cells are not silently interpolated, labelled safe, or removed from the denominator.

## Why this is more rigorous than a pointwise pass/fail grid

The initial PR #729 scenario matrix is an E0 design inventory. Its 60-Hz CSV fixture validates clock, column, unit/polarity metadata and digest exchange. It is not a CT/EMT measurement or physical relay result.

A parameter sweep of 100 points can still miss a narrow failure region between samples. The v0.1 contract therefore distinguishes *point observations* from *whole-cell bounds*. The first can reveal a counterexample; only the second can support a cell-level pass.

The current Rust verifier checks:
- strict typed JSON parsing with unknown fields rejected;
- parameter-axis uniqueness, dimensions and cell bounds within the declared domain;
- safe relative artifact paths and SHA-256 integrity for source, model and proof artifacts;
- valid, separately counted false-trip and failure-to-trip bounds;
- owner-defined coordination margin;
- individual service-floor margins, in native units;
- required versus available evidence capabilities;
- the three dispositions and deterministic receipt digest.

The tool does not validate the mathematical correctness of an interval proof producer. The synthetic certificate file exists only to exercise digest binding.

## The fidelity breakthrough: capabilities, not a scalar ladder

Do not reduce model evidence to one score such as RMS = 1, EMT = 2, HIL = 3. These methods answer different questions.

| Required claim | Minimum capability to declare |
|---|---|
| Fundamental-frequency dynamic response | Model validated for the declared RMS outputs |
| CT secondary distortion / saturation waveform | Waveform-resolving CT model or instrumented physical waveform |
| Software relay element response | Declared relay model, settings and model-validity evidence |
| Physical relay trip / restraint behaviour | Identified physical relay and measured response |
| Power-interface response | Relevant Power-HIL or laboratory evidence |
| Actual operating procedure / service exercise | Owner-authorised field exercise and independent outcome evidence |

The verifier uses separate capability flags for waveform resolution, independent validation, physical relay observation, physical power-interface observation and field-exercise observation. A capability missing for the requested claim produces UNKNOWN; it is not borrowed from a different test lane.

## From static parameter points to disturbance trajectories

The physical experiment should not be reduced to an unordered vector of scalar values. Protection response depends on time history and event ordering:

- GIC magnitude, polarity, waveform shape and duration;
- transformer initial magnetic/thermal state and CT remanence;
- fault inception relative to the GMD waveform;
- inverter current-control transient, limiting and momentary-cessation dynamics;
- relay filtering, timers, trip/restraint logic and backup coordination;
- sample-clock offsets and uncertainty.

Represent each experiment as a static configuration (	heta), a disturbance trajectory (u(t)), an initial state (s_0), and an ordered event schedule (e(t)). The operating domain is a bounded set of these inputs and their declared uncertainty, not merely a Cartesian product of independent steady values.

Matched RMS/EMT/HIL comparisons must hold topology, initial conditions, relay settings, fault, event timing and output definitions constant. Differences are reported per output: waveform distortion, pickup/reach, trip state and time, coordination margin, system response and service outcome.

## Adaptive boundary discovery

Use staged exploration rather than an indiscriminate Cartesian sweep:

1. **Screen:** identify influential axes using pre-registered sensitivity or designed-experiment methods. Use inexpensive models only for outputs they can represent.
2. **Bracket:** sample around threshold changes and model disagreements; retain the first counterexample and its exact configuration.
3. **Refine:** subdivide cells where uncertainty, a threshold crossing, or RMS/EMT/HIL disagreement remains material.
4. **Escalate fidelity:** use EMT for waveform-resolved transient questions, controller-HIL for physical relay/controller behaviour, and Power-HIL/laboratory for physical power interfaces.
5. **Prioritize experiments:** compare decision-relevant uncertainty reduction per simulation-hour or HIL-hour, while maintaining owner-defined hard constraints.
6. **Preserve unknowns:** publish the refinement budget, untested cells and exact evidence class. Do not use a smooth surrogate to fill a region unless its error bound itself covers that region.

The first implementation does not yet choose experiments or prove intervals. It verifies the declared cell evidence. Adaptive refinement is the next research increment after the verifier is qualified.

## Benchmark targets

All are targets, not achieved results:

| Benchmark | Target |
|---|---|
| Frozen known-answer contract corpus | 100% exact cell classifications |
| Digest / dimension / units / denominator / domain negative controls | 100% detected |
| Passing point-sampled cells | 0 falsely certified |
| False trip / failure to trip | Separate rates and denominators; zero on frozen certified fixtures |
| Service constraints | No required service-floor breach hidden by another service |
| Cell completeness | 100% of domain represented as certified / excluded / unknown when a true partition is declared |
| Fidelity mismatch | Reported per output and capability; no cross-fidelity averaging |
| Refinement efficiency | Aim for 2x fewer expensive EMT/HIL evaluations than uniform-grid baseline at matched boundary-localization error on the frozen synthetic benchmark |
| Reproducibility | Byte-identical input digest and deterministic JSON receipt |

The 2x refinement target is a research hypothesis. It becomes a result only after a matched budgeted experiment against a frozen uniform-grid baseline. No finite zero-failure corpus proves universal safety.

## Industry position

The direct comparator remains NGET's AIDPS programme (NIA2_NGET0068), which publicly reports detailed EMT models for SmartValves, STATCOMs, HVDC and Type-4 wind, and a 2-area / 4-machine / 11-bus RTDS benchmark with industrial relay HIL. Piesciorovsky, Patel & Reno (2026) already study 230-kV NBD, inverter-PV and distance-relay interactions, so simply combining those components is not novel.

The new research framing is narrower and more defensible:
- machine-checkable whole-cell evidence, not pointwise green sweeps;
- explicit unknown regions and counterexample-preserving refinement;
- evidence capabilities matched to the particular physical claim;
- cross-fidelity disagreement measured per output;
- protection constraints composed with owner-defined essential-service floors.

This complements IEEE C57.163-2023 transformer capability guidance, IEEE PES TR144 (2026) relay representation in fundamental-frequency phasor-domain dynamics, TR143 (2026) resilience metrics, and TR148 (2026) broader reliability/resilience framing. It does not replace the standards or reproduce NGET's hardware programme.

## South African path

The historical South African literature identifies a major empirical bottleneck: the 2008 network study explicitly noted limited measured GIC data, while the 2015 differential-magnetometer method paper describes a Botrivier measurement campaign close to Hermanus and comparison with transformer GIC at Bacchus. Those records are valuable leads, not evidence of present-day fleet coverage or current data access.

The next local experiment should be a single, owner-permissioned transformer/CT record with known grounding, CT DC response, calibration, load, timebase, polarity and a qualified earth/network model. A regional demonstration should follow only after this asset-level chain is reproducible.

## Claim ceiling and limitations

- The checked-in corpus and artifact payloads are entirely synthetic.
- A green result is a synthetic contract disposition at E2, not a physical safety envelope.
- The verifier validates a declared proof-artifact digest and coverage metadata; it does not prove the interval solver's mathematics or the physical model's validity.
- Cell counts are not volume-weighted coverage. A true percentage requires a disjoint complete partition and an explicit measure.
- Required coordination margins and service floors are owner-supplied; no universal threshold is invented.
- No switching, dispatch, protection-setting changes or execution authority are created.

## References

- [NGET AIDPS — NIA2_NGET0068](https://smarter.energynetworks.org/projects/nia2_nget0068/)
- [Piesciorovsky, Patel & Reno (2026), DOI 10.1016/j.epsr.2025.112190](https://doi.org/10.1016/j.epsr.2025.112190)
- [NREL/TP-5D00-94430 (2025)](https://doi.org/10.2172/3011876)
- [IEEE C57.163-2023](https://standards.ieee.org/ieee/C57.163/10203/)
- [IEEE PES TR143 (8 June 2026)](https://resourcecenter.ieee-pes.org/publications/technical-reports/pes_tr_143_amps_060826)
- [IEEE PES TR144 (8 July 2026)](https://resourcecenter.ieee-pes.org/publications/technical-reports/pes_tr_144_psdp_070826)
- [IEEE PES TR148 (1 October 2026)](https://resourcecenter.ieee-pes.org/publications/technical-reports/pes_tr_tr148_itslc_102026)
- [Texas A&M Texas2k Series25](https://electricgrids.engr.tamu.edu/texas2k-series25/)
- [South African differential-magnetometer GIC study](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1002/2015SW001289)
