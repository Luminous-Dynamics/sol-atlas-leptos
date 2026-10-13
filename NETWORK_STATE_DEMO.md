# Network State demonstration: community energy decision

**Status:** draft public demo prototype; synthetic scenario only  
**Target route:** `/labs/community-energy`  
**Implementation:** Rust core + Leptos/WASM UI  
**No private monorepo dependency:** the demo is contained in this public repository.

## The story

A small community faces an evening outage. It needs to compare two energy-resilience configurations, understand which loads remain unserved, and record how a community decision might preserve dissent.

This is a deliberately bounded entry point for a Network State audience: a resource allocation problem with measurable consequences, not a pitch that the entire Mycelix CivOS is already deployed.

## What is implemented in this prototype

- A frozen eight-interval, 15-minute synthetic load / renewable-generation / grid-availability case.
- Two explicitly defined battery configurations: a baseline and an oversized resilience sensitivity candidate.
- Deterministic interval accounting for renewable-to-load, charge/discharge, grid import/export, curtailed generation, unserved load, and battery state of charge.
- A critical-first accounting split that reports critical and noncritical served/unserved energy separately.
- A separate verifier function that recomputes interval balances and aggregate totals from the candidate's rows without calling the simulation function again.
- Negative tests for forged aggregate totals, an import recorded during an outage, battery power-limit violations, and the frozen scenario's expected trade-off.
- A pre-seeded tally worksheet with 4 yes / 1 no / 0 abstain, demo-only 60% quorum and 75% approval thresholds, and visible dissent. It accepts no ballots and persists no vote. The [Mycelix Governance README](https://github.com/Luminous-Dynamics/mycelix/blob/main/mycelix-governance/README.md) lists 25% / 60% Standard proposal thresholds and MATL/stake/participation weighting, but [Mycelix issue #851](https://github.com/Luminous-Dynamics/mycelix/issues/851) records that executable voting paths diverge and the README is not an authority source. Neither the README nor this worksheet should be read as the authoritative current policy. The fixture arithmetic also fails closed on over-counted ballots, zero eligible population, or thresholds outside 0–100%, rather than allowing saturating integer arithmetic to produce an apparent pass.

## Frozen scenario result

The inputs are synthetic and are embedded in `sol-atlas-core/src/community_energy_demo.rs`. The run has 28 kWh total load and 6.1 kWh renewable generation across eight intervals, with intervals 5 and 6 (one-indexed) marked as grid outages.

| Result | Baseline equipment | Resilience sensitivity |
|---|---:|---:|
| Capacity / starting SOC / charge-discharge limit | 6 kWh / 3 kWh / 2 kW | 20 kWh / 16 kWh / 8 kW |
| Total grid import | 11.52 kWh | 4.24 kWh |
| Total unserved load | 8.6168 kWh | 5.82 kWh |
| Critical energy unserved | 1.6168 kWh | 0.00 kWh |
| Noncritical energy unserved | 7.00 kWh | 5.82 kWh |
| Accounting verifier | Pass | Pass |
| Zero-critical-shed constraint | Fail | Pass |

These are in-model results, not field measurements or performance predictions. The candidate's 20 kWh battery and 8 kW charge/discharge limits are an intentionally oversized sensitivity scenario. It also starts at 16 kWh state of charge versus 3 kWh for the baseline. The upstream source/cost of the precharged reserve is not modeled, so this is not a like-for-like battery-sizing result and is not a costed engineering recommendation. Costs, state-of-health degradation, inverter conversion, feeder limits, protection systems, weather uncertainty and calibrated real loads are not modeled.

The key teaching point is that **accounting consistency and policy success are different outcomes**. The baseline accounting can be correct while its result fails the declared critical-load constraint. The candidate meets the critical-load limit in this frozen fixture but still sheds noncritical load.

**Critical-first allocation assumption:** the model divides total unserved energy by allocating service to critical demand first, then noncritical demand. It does not simulate circuit topology, priority relays, selective load shedding, or a controller that enforces this ordering. The reported critical-unserved values therefore describe the declared accounting assumption; they do not establish that a physical site can guarantee critical-load service. A different service-allocation assumption could change the critical/noncritical split without changing total unserved energy.

## Evidence boundaries

- **Observed data:** none.
- **Scenario data:** all input profiles and the vote fixture.
- **Model:** deterministic Rust interval accounting.
- **Verifier:** separate function in the same Rust crate, not an independent process or organization. It does not authenticate the fixture's source or prove that any physical meter reported these values.
- **Governance:** pre-seeded illustrative tally only. The page accepts no ballots, persists no vote, and does not create a Holochain transaction.
- **Authority:** no physical actuator, inverter, breaker, payment or market integration exists in this demo.
- **Economics:** no cost, tariff, finance, ROI, or optimal sizing conclusion is claimed.

The public Mycelix README describes Mycelix as pre-alpha and identifies Energy as research-stage. This demo must not imply that the UI is connected to Mycelix Energy, that the vote uses production Mycelix governance, or that Holochain multi-agent conformance has been established.

## Run

From the repository root, use the project's Rust/WASM toolchain and Trunk configuration:

```sh
cargo test --manifest-path sol-atlas-core/Cargo.toml community_energy_demo
trunk serve --port 8091
```

Open `http://127.0.0.1:8091/labs/community-energy`.

The command above runs the focused core tests; also run the repository's required root formatting, browser/WASM build and native checks before merging. Do not call the demo qualified while any relevant exact-head checks are failed, queued or skipped.

## Proposed Network State walkthrough

1. Open the energy scenario and point out that every input is marked synthetic.
2. Select the baseline and show that accounting passes but the critical-load check fails under the declared critical-first allocation assumption.
3. Select the larger sensitivity candidate and compare unserved critical versus noncritical demand; call out both the critical-first allocation assumption and the higher precharged starting SOC (16 kWh versus 3 kWh), then explain that the candidate is not cost-optimized.
4. Inspect the interval rows across outage periods and ask attendees to identify which assumptions they would challenge.
5. Show the pre-seeded example tally including dissent; state that this page accepts no ballots, persists no vote, and does not send a transaction to Mycelix/Holochain.
6. Ask which evidence, privacy and appeal safeguards the community would require before deploying this workflow with real members and real meters.

## Acceptance gates before calling it a qualified demo

- Focused Rust tests pass for the exact commit.
- Root Rust format, native and WASM/browser build checks pass at the same exact commit.
- The browser route loads directly and at narrow/mobile widths.
- Both cases render the known-answer results above.
- Tampering with totals, outage grid flows or battery limits is rejected by the verifier.
- Evidence labels are visible without opening developer tools.
- The critical-first allocation assumption and absence of physical priority-control modelling remain visible.
- Malformed tally data (ballots exceeding the eligible population or thresholds above 100%) fails closed.
- There are no undocumented network requests, no real meter/identity collection, and no physical control path.
- Public pitch materials consistently say synthetic scenario, same-codebase verifier, illustrative vote, and no real actuation.
