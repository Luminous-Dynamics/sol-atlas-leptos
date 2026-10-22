# ENERGY-PORTS-V0.1 — multi-output conversion without double counting

**Status:** reference implementation and frozen synthetic graph; CI results determine executable verification status.  
**Scope:** one time-bounded directed acyclic graph of energy-conversion/service nodes, with explicit ports, stream allocations and terminal dispositions.  
**Not in scope:** thermal transients, storage/cycles, grid optimization, unit commitment, chemistry/mass-flow reaction modelling, detailed converter physics, second-law validation of every converter, or authenticated telemetry.

## Paradigm shift

A single efficiency number hides the difference between primary output, useful co-product heat, exhaust, rejected energy, and curtailed resource. A linear converter chain also fails to express that a co-product can be routed to a downstream user.

This contract uses typed input/output ports and directed edges. It retains each node boundary and first-law balance while ensuring that an output portion routed downstream is not also counted as a terminal loss. The graph is a conservation oracle, not a replacement for a process simulator or dispatch optimizer.

## Contract

For each node, at the declared boundary:

\[
\sum E_{\mathrm{input}} = \sum E_{\mathrm{output}}
\]

For each input port, declared accepted energy equals external supply plus energy delivered by incoming edges. For each output port, all outgoing edge allocations must be no greater than the declared output quantity. The un-routed remainder has exactly one terminal disposition: useful service, rejected, unconverted resource or curtailed.

At graph level:

\[
E_{\mathrm{external}} = E_{\mathrm{useful}} + E_{\mathrm{rejected}} + E_{\mathrm{unconverted}} + E_{\mathrm{curtailed}}
\]

Internal edges appear in the input/output node ledgers but are not counted again as external supply or terminal service. The receipt report separately exposes routed energy for diagnostics; do not add that diagnostic total to graph-level terminal energy.

### Identity, typing and chronology

- Each node has a non-empty boundary description and an evidence class; non-assumption nodes require a reference.
- Every positive external input stream has a unique external_source_id. The same source identity may not be declared as two independent external supplies. To split a shared source, represent it once as a source/converter node output and allocate it over graph edges.
- Energy form must match at both ends of every edge.
- Thermal-heat ports require an explicit finite temperature above 0 K after Celsius conversion. Non-thermal forms must not include a temperature in this v0.1 contract. Heat edges preserve the declared temperature grade; a grade change needs an explicit converter/transfer node.
- All nodes and edges form a DAG. Cycles fail closed; carrying energy over time belongs in the stateful storage model.
- Unknown JSON fields, duplicate identifiers, negative/non-finite quantities, over-routed outputs, under-specified inputs, node-balance mismatch and global-balance mismatch fail closed.
- The energy tolerance is scale-aware: absolute 1e-9 kWh plus relative 1e-12 of the larger magnitude. It is not evidence of measurement accuracy.
- Node first-law closure is necessary but not sufficient for physical feasibility. A converter can still violate a second-law constraint unless a component-specific model/bound is applied. Heat-pump/Carnot and exergy checks stay in their respective contracts.

External source identity is an explicit anti-double-counting control, not a signature or proof that the upstream measurement is true. The CLI receipt binds the exact input bytes and serialized assessment with SHA-256; it does not authenticate metering, calibration, ownership or evidence.

## Frozen synthetic cogeneration fixture

Synthetic control volume for a cogeneration unit and process heat user:

| Quantity | Energy |
|---|---:|
| External chemical-energy input | 100 kWh |
| Net electricity terminal service | 35 kWh |
| Heat co-product output | 45 kWh at 80 C |
| Exhaust/rejected heat output | 20 kWh at 150 C |
| Co-product heat routed to a process node | 15 kWh |
| Process heat terminal service | 14 kWh |
| Process boundary rejection | 1 kWh |
| Co-product heat remainder rejected | 30 kWh |

Node balances: CHP input 100 = outputs 35 + 45 + 20; process input 15 = outputs 14 + 1. Graph terminal quantities: 49 kWh useful service (35 electricity + 14 heat), 51 kWh rejected (30 + 20 + 1), and 100 kWh total terminal disposition against 100 kWh external input. The 15 kWh heat edge is not independently added to terminal output. These values are a known-answer synthetic fixture, not evidence about any real CHP plant.

## Benchmarks and negative controls

| Benchmark | Acceptance target |
|---|---|
| Frozen analytical CHP graph | 100 kWh external input = 100 kWh terminal disposition; residual within scale-aware tolerance |
| Co-product attribution | 15 kWh routed once; 30 kWh remainder explicitly rejected; never report 45 kWh rejected too |
| Duplicate/over-allocated output edges | reject before portfolio totals |
| Input routing | every input is filled by explicit external energy plus its incoming edges |
| Local energy balance | every node closes within scale-aware tolerance |
| Identity and form checks | duplicate node/port/edge/source identities and carrier mismatch fail closed |
| Temperature-grade controls | missing absolute temperature, nonpositive kelvin or changed grade on an edge fail closed |
| Graph topology | cycles are rejected; deterministic sorted report order |
| Numerical stress | negative, non-finite and overflowed totals reject |
| Evidence | assumption/model/test/field/plant claims retain evidence class; non-assumption without a reference rejects |

Future property-based tests should generate small acyclic port graphs and mutate one invariant at a time. For small graphs, enumerate route allocations exactly and compare the evaluator's service/rejection totals; include adversarial controls for duplicate edges and duplicated external sources. The evaluator should never convert missing, unknown or unmetered energy into a zero-loss claim.

## Industry comparison and what “better” would mean

- **SAM:** compare compatible component/plant balances only where boundaries and stream outputs align; use technology-specific models for actual plant behaviour.
- **REopt:** compare facility/microgrid sizing, dispatch, cost and outage outcomes, rather than comparing the graph evaluator to an optimizer.
- **PyPSA:** compare chronological network/sector portfolios under identical time series, carrier definitions, storage states and constraints.
- **Independent oracle:** use hand-derived small graphs first, then a separately implemented exact small-instance flow oracle.

Pin tool/version, scenario inputs, units, source IDs, time interval, boundary, weather/load data, constraints and raw-input digests. Report solve time, balance residuals, service delivered, unserved service, curtailment, evidence coverage, Pareto fronts and regret against an exact small-instance oracle. Do not assert superiority over SAM/REopt/PyPSA until matched held-out tests establish it.

## Next steps

1. Couple output temperature grade with the thermal-exergy oracle and component-specific second-law constraints.
2. Add time-indexed edges only through explicit storage-state boundaries; keep chronology and state transitions auditable.
3. Add shared-source capacity/evidence objects and signed meter ingestion without treating a signature as calibration.
4. Run the same frozen graph and later real plant traces through independently implemented reference calculations.
5. Only then build dispatch/portfolio optimization that maximizes essential service under cost, water, carbon, materials, grid and resilience constraints.
