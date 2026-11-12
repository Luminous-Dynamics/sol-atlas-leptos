# GMD multi-rate trajectory and clock contract v0.1

**Date:** 12 November 2026  
**Tracking:** [GMD-TRAJECTORY-CLOCK-001 #732](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/732)  
**Parent:** [GMD-IBR-PROTECTION-001 #726](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/726), [PR #729](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/729)  
**Claim ceiling:** E2 synthetic contract logic only.

## Purpose

The GMD / IBR / CT / protection experiment is multi-rate and trajectory-dependent. GIC observation cadence, transformer magnetic response, CT secondary waveform sampling, IBR control transients, relay sampling and event clocks must not be collapsed into one timestamp or one frequency.

The generic 60-Hz waveform in PR #729 remains a valid data-exchange fixture. It must not be silently reused as a South African grid profile: Eskom states that the South African power system is balanced at nominal 50 Hz. The v0.1 profile therefore declares nominal AC frequency independently from rational sample rates and the GIC channel cadence.

## Contract

An experiment binds:
- a regional nominal-frequency profile;
- a TAI reference epoch and phase reference;
- source clock domains with anchor, bounded offset and bounded drift;
- observed calibration offset/drift that must remain inside the declared bounds;
- channels with signal kind, units, polarity, time column, sample period, rational sample rate, and artifact digest;
- explicit source-to-effective transformation chains;
- event-time intervals rather than point timestamps.

Clock mappings use an affine bounded model with integer ppb drift. The verifier computes conservative mapped intervals with outward floor/ceiling rounding. It does not silently normalize clocks.

For events A and B:
- A is before B only if `upper(A) < lower(B)`;
- B is before A only if `upper(B) < lower(A)`;
- otherwise their order is unresolved.

A partial order is the correct result when timing uncertainty overlaps. The verifier never forces a total event ordering.

## Frequency and sampling semantics

These are distinct typed quantities:

1. `nominal_frequency_millihz`: regional AC system frequency (50,000 for 50 Hz or 60,000 for 60 Hz in v0.1).
2. `sample_period_ns` and rational sample rate: channel acquisition cadence.
3. GIC channel cadence: its own channel sampling interval; it must not be mislabeled as AC frequency.

The South African profile identifier `south_africa` rejects a non-50-Hz context. Generic fixtures can declare 50 Hz or 60 Hz. This is a profile constraint, not a claim that every asset operates at exactly nominal frequency under all conditions.

## Clock and artifact integrity

- Offset and drift bounds are explicit; observed calibration values outside them are rejected.
- Drift must preserve a monotonic mapping.
- Sample timestamps must be strictly monotonic and match the declared rational sample rate.
- The effective CSV artifact's time column must exactly match the manifest time vector.
- Sample gaps and duplicate timestamps are rejected in v0.1; future versions may explicitly represent segmented or gapped records.
- Source/effective artifacts must be identical unless an explicit transformation chain links them.
- Resampling, filtering, polarity inversion, unit conversion and clock correction require input/output digests, method, implementation revision and parameter digest.
- All artifact paths are safe relative paths and all bytes are SHA-256 checked.

The contract uses TAI as its declared reference time scale to avoid hiding UTC leap-second handling inside an implicit conversion. An external UTC-to-TAI conversion must be recorded as a versioned transformation with provenance before a record is admitted.

## Fault phase uncertainty

For a `fault_inception` event, the verifier derives a circular phase interval from its mapped time interval, the declared phase-zero time/angle and nominal frequency. It reports wrap-around and full-cycle uncertainty separately. An interval wider than a full cycle is marked phase-ambiguous; it is not reduced to a falsely precise point phase.

## Initial synthetic corpus

The frozen v0.1 corpus includes three synthetic channels:
- GIC neutral current at 1 sample/s, with no AC frequency label;
- CT secondary waveform at 16 ksample/s, with a 50-Hz AC profile;
- relay input at 16 ksample/s, derived from an 8-ksample/s source through an explicit digest-bound synthetic resampling record.

The event corpus includes GIC onset, fault inception, relay pickup and relay trip on two clock domains. The fault phase result carries uncertainty. All artifacts and sample values are synthetic; no physical CT, relay, transformer or grid measurement is present.

## Frozen adversarial tests

The Rust test suite checks:
- valid 50-Hz South African profile;
- valid generic 60-Hz profile;
- South African 60-Hz mismatch rejection;
- distinct sample cadence vs nominal AC frequency;
- clock offset and drift calibration bounds;
- outward-rounded clock mapping;
- known event order vs overlapping unresolved order;
- sample gaps, duplicates and CSV/manifest time-column mismatch;
- undeclared source/effective differences;
- transformation-chain provenance;
- fault-phase interval uncertainty;
- digest mismatch rejection.

## Known limitations

- E2 synthetic logic only; no physical clock calibration, relay, CT or regional model qualification.
- TAI reference and clock bounds are declared inputs; this tool does not establish traceability to UTC or an instrument's reference clock.
- The parser checks the declared CSV time column, not every COMTRADE field. The exact provider-declared COMTRADE edition/exporter must remain separately recorded.
- Transformation provenance is checked; the mathematical correctness of the resampling or filtering implementation is not independently proven.
- v0.1 rejects sample gaps rather than representing them as recoverable segmented records.
- Phase intervals use a declared nominal frequency; dynamic instantaneous phase tracking and PLL uncertainty remain future work.
- No switching, dispatch, protection-setting changes or execution authority are introduced.

## Next research increment

The next step is a trajectory-aware benchmark against static snapshots. Use matched topology, initial magnetic state, GIC pulse, fault timing, IBR controller and relay settings. Compare:
- static snapshot vs trajectory-aware model;
- exact event-time vs bounded clock uncertainty;
- declared resampling vs silent resampling;
- sample-aligned vs deliberately misaligned waveform channels.

Measure newly discovered failure cases, event-order ambiguity, phase uncertainty, boundary-localization error and expensive EMT/HIL evaluations per discovered failure mode.

## References

- [Eskom company information: 50-Hz system frequency](https://www.eskom.co.za/about-eskom/company-information/)
- [IEEE C37.111 / COMTRADE project and status](https://standards.ieee.org/ieee/C37.111/11052/)
- [Liu et al. (2026), transformer transient saturation under GIC pulses](https://doi.org/10.1016/j.epsr.2026.113139)
- [IEEE C57.163-2023: transformer capability under GMD](https://standards.ieee.org/ieee/C57.163/10203/)
- [IEEE PES TR144 (2026): relay models in phasor-domain dynamic simulation](https://resourcecenter.ieee-pes.org/publications/technical-reports/pes_tr_144_psdp_070826)
