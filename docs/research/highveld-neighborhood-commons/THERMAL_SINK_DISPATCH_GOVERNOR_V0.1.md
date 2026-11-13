# Thermal sink dispatch governor — safety and entitlement contract v0.1

**Study:** HIGHVELD-THERMAL-SINK-CASCADE-001  
**Maturity:** GOVERNOR_CONTRACT_NOT_PHYSICALLY_VALIDATED  
**Purpose:** prevent an optimizer or controller from requesting an active sink path that violates known physical, water, safety or evidence constraints.

## Paradigm

Split dispatch into two layers:

1. A deterministic feasibility/entitlement governor enumerates only physically and legally eligible options.
2. A separate bounded optimiser may compare eligible options on comfort, water, electricity, peak demand, wear and cost.

The governor does not choose the morally preferred allocation, actuate equipment, or replace resident authority. It cannot be overridden by an optimizer. It is deliberately conservative: unknown or failed evidence blocks the affected active path.

## Input contract

Each evaluation binds:
- model applicability and instrument state;
- availability of local control and manual safe mode;
- network state (a network partition does not block a locally safe path when local control remains available);
- a per-path qualification state and sensor state;
- effective sink temperature, store temperature, minimum approach, available/requsted heat-rejection capacity and auxiliary power;
- for evaporative paths only: source rights, source quality, restriction state, account balance, commitments, protected essential reserve, requested draw, aerosol controls and humidity state.

All operating thresholds and equipment ratings come from qualified design/model inputs. The governor does not invent universal temperature approaches, humidity limits, reserve quantities or capacity ratings.

## Decision semantics

A sink path is eligible for downstream optimizer consideration only when:
- the overall model is in domain, required instruments are qualified, local control is available and manual safe mode exists;
- the path itself and its sensors are qualified;
- store-to-sink temperature headroom exceeds the path's declared minimum approach;
- available rejection capacity is at least the requested rate;
- auxiliary power is available at the required level;
- for evaporation, water rights and quality are qualified, current restrictions permit use, source/account quantities cover commitments plus the protected reserve plus requested draw, and aerosol/humidity gates are within limits.

A missing field, unknown state, failed qualification, negative temperature headroom, over-capacity request or inadequate auxiliary power blocks that path. The report explains why.

A network partition is not itself a rejection when local control and safe fallback are available. Conversely, a connected network is not enough to authorize an action when the local evidence is invalid.

The result is an eligibility list, not an actuation command. It always records that no equipment was actuated by this tool.

## Benchmark and negative controls

The committed synthetic case demonstrates three distinct outcomes:
- a qualified radiative path can be eligible when a valid local controller remains available during a network partition;
- a dry cooler is blocked when its declared temperature approach is not met;
- an evaporative path is blocked when water restrictions prohibit discretionary use and the available account cannot cover reserve plus demand.

Required tests include preservation of a genuinely valid evaporative positive control when restrictions/rights/quality/quantity/health gates are satisfied, as well as missing-instrument, out-of-domain, sensor fault, insufficient sink headroom, insufficient capacity, insufficient auxiliary power, essential-reserve breach and network-partition cases.

Hard targets:
- zero optimizer-visible options that fail a hard governor gate;
- zero discretionary draw against protected essential water reserves;
- zero unknown state treated as qualified;
- 100% preservation of valid positive controls;
- 100% deterministic agreement with finite known-answer input cases.

## Claim ceiling

The Rust implementation is a deterministic contract/eligibility filter. It is not an MPC solver, CFD model, EnergyPlus model, hydraulic simulation, forecast model, rights authority or equipment controller. It does not actuate hardware or establish the physical truth of the supplied sensor/model state. Operational use requires source-owner admission, independent review and a separately qualified physical control system.


## 7. Two-sided thermal pinch: load-to-store admission

The governor now evaluates a separate \`BUILDING_LOAD_TO_STORE\` charge path in addition to store-to-sink regeneration. This prevents a design from qualifying a cold sink while silently assuming that building heat can enter the storage medium at an arbitrary temperature grade.

For the load-to-store fluid boundary:

\[
\Delta T_{\mathrm{load-store}} =
T_{\mathrm{load,return}} - T_{\mathrm{store}}
\]

Admission requires the measured/qualified approach to exceed the declared minimum exchanger approach, the available heat-transfer power to cover the requested transfer rate, and the requested interval energy to fit the currently qualified store headroom:

\[
Q_{\mathrm{requested}} =
\dot Q_{\mathrm{requested}}\,\Delta t
\le E_{\mathrm{capacity}}-E_{\mathrm{state}}
\]

The v0.1 filter deliberately gives **no credit for concurrent regeneration** when checking this headroom. That is conservative and prevents unverified overlap between load-side charge and sink-side discharge from qualifying an otherwise full store. A future coupled temporal controller can credit simultaneous regeneration only with synchronized, qualified flow/temperature trajectories and independent store-balance closure.

The committed synthetic positive control has a 9 K load-to-store approach, 3 K minimum approach, 1.5 kW requested transfer over 2 hours, 3 kW available exchanger capacity and 4 kWh measured headroom. It is eligible for optimizer review only; the tool does not actuate the exchanger.

Negative controls block insufficient temperature grade, load-side exchanger capacity, sensor qualification and finite store headroom. The result exposes approach, requested transfer energy, remaining headroom and the explicit \`NOT_ASSUMED\` concurrent-regeneration policy.

This check is separate from ASHRAE occupied-zone comfort and from PCM phase-change physics. It is a boundary feasibility gate around a qualified fluid/store model, not a thermal simulation.
