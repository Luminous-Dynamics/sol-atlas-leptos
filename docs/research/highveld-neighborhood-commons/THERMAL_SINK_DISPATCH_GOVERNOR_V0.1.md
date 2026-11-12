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
