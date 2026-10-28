# ENERGY-DESIGN-ADAPTER-001 — candidate to bounded dispatch v0.1

Status: initial implementation spike, synthetic-only. No field or SOTA claim is made.

## Contract provenance

The candidate schema snapshot in `schemas/vendor/symtropy-energy-system-candidate-v0.1.schema.json` was fetched from Symtropy `schemas/energy-system-candidate-v0.1.schema.json`, blob SHA `697e7068bd3a086403b04376567a90f570733176`. The frozen example at `schemas/examples/energy-design-pv-battery-grid.synthetic.json` comes from the shared candidate contract and remains fully marked synthetic. If the upstream schema changes, refresh the snapshot deliberately, document the upstream blob SHA, and re-run cross-repository validation; do not silently drift the contract.

## What the adapter does

`sol-atlas-core/src/energy_design_adapter.rs` compiles only a narrow representable subset of the candidate graph into the exact finite model in `energy_dispatch.rs`:

- One site and one contiguous scenario with uniform interval duration.
- Electrical-service requests expressed in kWh.
- At most one PV source, one grid-import source and one battery.
- Explicit per-interval source availability, PV interval-energy/rated-power caps, grid import-energy/power caps, storage capacity, opening SOC, terminal reserve, charge/discharge power and efficiency claims.
- An explicitly declared kWh quantum, candidate expansion cap, and grid cost/carbon factors from the mapping configuration.
- A fully connected shared-bus topology for the supported components: each source reaches each serviced sink; when a battery exists, every source has a declared path to its charge port and the battery has a discharge path to each serviced sink.

The compiler rejects, rather than approximates away, unknown source technologies, extra storage, unsupported component kinds, thermal/process carriers, services outside electricity/kWh, inconsistent outage availability, missing battery/reserve/power/efficiency inputs, broken connectivity, nonuniform timestep, missing source samples, non-quantized energies and missing factor-basis text.

The mapping policy only accepts synthetic analyst-assumption candidates today. The mapping configuration must explicitly state a basis for cost/carbon factors. Numeric zero is allowed only as an explicit synthetic test value; absent values are not defaulted to zero.

## Output and claim ceiling

Run the compiler with the candidate and mapping configuration:

```text
cargo run -p sol-atlas-core --bin energy_design_dispatch_compile -- schemas/examples/energy-design-pv-battery-grid.synthetic.json schemas/examples/energy-dispatch-mapping-config.synthetic.json
```

The output includes candidate, config and compiled-request SHA-256 digests; the reduced dispatch request; supported-model boundary; objective coverage; and every requested evaluation that remains unexecuted. It is a **compiled problem, not a dispatch result**.

Objective coverage is explicit:
- essential/flexible unserved energy: mapped to dispatch quanta × declared quantum;
- PV curtailment: partial, because the evaluator does not represent every possible source's spill or lifecycle boundary;
- operating cost: partial, limited to explicit grid-import cost units (not full-system currency, capital cost, maintenance, fuel or replacement);
- emissions: partial, limited to the declared grid-import factor in grams (not lifecycle CO2e);
- all other objectives: preserved as `not_scored`, not silently dropped from the output.

The reduced subproblem being feasible does not prove the whole candidate graph is feasible. It does not model AC power flow, voltage/frequency, protection, feeder limits, converter switching, thermal systems, multi-site flows, uncertainty trees, degradation, forecast error, lifecycle economics or field hardware. It cannot actuate equipment.

## Frozen expected behavior

The fixture contains PV, grid, one battery and one essential electrical sink. The first hourly interval has 2 kWh of PV; the second interval is a grid outage with 2 kWh essential demand. With the explicitly idealized lossless 2 kWh battery, the compiled dispatch request has:
- 2 PV quanta and 2 grid-import quanta available in the first interval;
- zero grid-import allowance in the outage interval;
- a 2-quanta battery with zero opening SOC, explicit zero terminal reserve and 2-quanta per-interval charge/discharge limits;
- 2 essential-demand quanta in the outage interval.

The downstream exact oracle should recover a schedule that charges before the outage and serves all essential demand with zero outage grid import. This is a synthetic hand-calculated transfer test, not a real battery-performance or reliability claim.

## Negative controls

- Unsupported component or source type must fail.
- Removing the PV-to-load edge must fail; the adapter may not create an unconnected supply route.
- Removing the grid-to-battery or battery-to-load path must fail for a battery architecture.
- An outage interval declaring positive grid availability must fail.
- Missing terminal reserve, missing power limit or missing efficiency must fail.
- A non-integer-quantum availability/demand must fail rather than round.
- Non-synthetic inputs or non-assumption evidence are unsupported in v0.1.
- Operating cost/carbon factors with an empty basis must fail.
- Every schedule remains subject to the downstream exact oracle's own hard constraints; compilation itself does not claim those checks were run.

## Benchmark goals

Targets, not measured results:
- 100% recovery of the mapped frozen known answer.
- 100% rejection of the enumerated unsupported/invalid mappings.
- Zero energy-balance, outage-import, storage-state or reserve violations in any downstream schedule labelled feasible.
- Identical candidate bytes and config produce identical compiled-request bytes and digest.
- Report adapter rejection rate and supported-feature coverage alongside service shortfall, objective coverage, search expansions and runtime.
- Hold at least 10,000 generated cases for later validation; never claim performance from only the one frozen fixture.
- Keep essential-service failure separate from cost/carbon; don't rank complete designs with partial objective coverage.

## Industry comparison lane

The adapter is not a replacement for [REopt](https://developer.nlr.gov/docs/energy-optimization/reopt/), [PyPSA](https://docs.pypsa.org/latest/) or [GenX](https://genxproject.github.io/GenX.jl/stable/User_Guide/workflow/). Its job is to make one subset of a candidate architecture precisely comparable to an exact reference problem. External comparisons require identical time boundaries, resource availability, constraints, storage efficiency, terminal SOC, objective semantics and discretization. Publish translation limitations and solver status; don't equate a tiny quantized result with system-scale optimality.

## Follow-on

1. Expand the deterministic generated corpus and run independent schedule verification.
2. Add adapters for thermal and multi-carrier candidates only after the corresponding typed-port, exergy, storage and dynamics contracts can be translated without loss.
3. Connect the compiled request to Symtropy trajectory/fault evaluation and preserve all candidate digests and model versions.
4. Only then begin matched REopt/PyPSA subproblem comparisons and held-out regret analysis.
