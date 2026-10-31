# Temporal Dispatch Qualification v0.1

Status: contract plus H=2 exhaustive test-only schedule oracle (65,536 bounded assignments), H=3 deterministic sampled oracle (50,000 assignments), H=4 deterministic sampled oracle (120,000 assignments across three seeds), all-six-objective independent recomputation, and interval-index/SOC/energy negative controls. Production schedule API and exact-head CI qualification remain open.

## 1. Assurance boundary

The current translation validator checks that a restricted LP file has the same objective, variables, bounds and rows as a separately derived model for one source request. That is a compiler-correctness check, but it does not independently prove that the physical interpretation of a multi-period schedule is complete.

The next increment adds a second oracle: a schedule evaluator written against the source request schema and candidate schedule. It must not import the LP emitter, LP parser, expected-model builder, or solver result parser. Its core input is exact source request bytes, a candidate per-interval dispatch schedule, and an explicit unit/time-base profile.

Its output is a structured list of feasibility predicates, exact integer residuals, objective recomputations and an overall accept/reject decision. It does not trust solver status or translation receipts as schedule-feasibility evidence.

## 2. Temporal semantics to freeze

For each interval t, define the model's integer units and independently test:

- SOC carry: SOC[t+1] is derived from SOC[t], charge[t], discharge[t] and the documented floor-efficiency rules.
- Mode exclusivity: charge[t] and discharge[t] cannot both be positive when the request uses a binary mode.
- Power/energy conversion: interval duration, power limits and energy quanta are dimensionally consistent.
- Grid exchange: import/export bounds and outage state are enforced at the same interval index as the input profile.
- PV balance: PV used, curtailed PV and exported PV cannot exceed available PV.
- Load partition: essential and flexible served/unserved components reconcile exactly to demand.
- Bus balance: supply, storage flow, grid exchange, PV and load reconcile in the declared sign convention.
- Reserve: minimum SOC and terminal SOC conditions apply at the documented boundaries.
- Time coupling: no interval may omit, duplicate, reorder or borrow state from another interval.

Define interval-boundary behavior explicitly. Initial SOC is an input; terminal SOC applies after the final interval; outage state is tied to its own interval.

## 3. Independent oracle structure

Keep three layers separate:

1. Request decoding and normalization: strict source-schema decoding, units and profile checks.
2. Schedule semantics: direct arithmetic over request and candidate schedule; no LP objects or shared equation-generation helpers.
3. Evidence binding: hash exact request, normalized schedule, evaluator version, unit profile, interval count and result record.

Use checked integer arithmetic. Overflow is a hard rejection, not saturation or floating-point fallback. Report each predicate separately so a rejection identifies the violated constraint family and interval.

Do not deserialize solver LP names into domain semantics. LP variable naming is a compiler artifact, not the authoritative schedule API.

## 4. Exhaustive small-horizon oracle

Start with horizons H=2, 3 and 4, bounded to keep full enumeration tractable. Enumerate all admissible integer action combinations independently, including zero and upper-bound values. Compare feasibility classification, exact objective values, minimum/maximum objective, number of feasible schedules, boundary states and per-interval residuals.

Include known-answer controls: feasible zero-flow; forced charging followed by discharge; zero charge/discharge limit; outage with no grid import; impossible essential-load service; terminal-SOC failure; floor-efficiency boundary; profile length mismatch; interval reorder.

The enumerator and evaluator must not share transition helpers. An oracle repeating production evaluator code is not independent.

### Implemented first increment

The follow-on branch extends the test-only schedule specification to walk
sequential request intervals and carries independently recomputed SOC between
them. The H=2 fixture charges from PV, then serves essential demand during a
grid outage. Its 16 binary-bounded free variables are exhaustively enumerated:
65,536 complete assignments are compared by LP feasibility and the
source-request schedule evaluator. A separate negative control corrupts the
preceding interval's SOC and must be rejected.

The H=3 fixture adds an idle recovery interval after the outage and compares
50,000 deterministic assignments generated with a frozen LCG seed. This is
sampled coverage, not exhaustive enumeration. For every H=2 exhaustive and
H=3 sampled assignment, all six LP objectives are independently recomputed
from the source request and compared exactly: essential unserved, flexible
unserved, operating cost, grid carbon, curtailed PV and terminal SOC. The
oracle also asserts that every objective-specific model has identical
variables/bounds and constraints, so objective changes cannot silently alter
the feasible set.

The H=3 negative control also swaps the grid-outage and post-outage idle
intervals while retaining the original candidate trajectory; the independent
source-request evaluator must reject the now-misaligned schedule.

The H=4 fixture adds a post-outage recovery interval with available PV and a
bounded grid-import allowance. Three frozen LCG seeds each exercise 40,000
assignments, for 120,000 sampled assignments total. The model/schedule
feasibility and all six objective values are compared on every sample. A
separate known-feasible trajectory is mutated at each of its four SOC
transition outputs; the schedule oracle must reject all four interval-local
corruptions. H=4 is sampled, not exhaustive; a production schedule receipt
and broader generated request domains remain open.

## 5. Mutation panel

Generate deterministic mutations with recorded seed and mutation ID:

- SOC[t] to SOC[t+1] off-by-one;
- omission of one transition;
- one interval duplicated or removed;
- outage bit shifted by one interval;
- PV/load vectors reordered independently;
- interval duration changed while energy/power values remain fixed;
- coefficient, objective weight, import/export bound or terminal condition altered;
- floor efficiency replaced with round-to-nearest or ceiling;
- charge/discharge exclusivity omitted;
- request/schedule/evaluator digest cross-association.

Each mutation has an expected target predicate. Report detection by category and interval, not only an aggregate score. A crash or generic parse error is not semantic detection unless malformed input is the mutation target.

## 6. Evidence record

A schedule acceptance receipt should bind source request SHA-256, source schema/profile ID, schedule canonical-byte SHA-256, interval count and time-base/unit profile, evaluator identifier/version and binary SHA-256, evaluator result SHA-256, translator receipt SHA-256, LP SHA-256 and semantic-model SHA-256, solver identity/version/binary digest, solver certificate digest and independent checker result.

The evidence graph must state which links were checked and which were simply recorded. A digest association does not make an unverified assertion true.

## 7. Benchmark tiers

Tier A — exhaustive semantics: zero false-feasible and false-infeasible cases, exact objective agreement and reproducible enumeration counts on every frozen tiny-horizon case.

Tier B — compiler mutation: report per-family mutation detection, interval localization, false rejection and false acceptance. Target 100% on the finite frozen mutation panel and publish the panel.

Tier C — proof/evidence: tamper with every source, receipt, LP, semantic digest, certificate, checker and schedule association. Target 100% rejection of the finite tamper panel.

Tier D — operational realism: on held-out multi-day traces, report essential unserved energy, longest continuous interruption, recovery time, flexible-load shed, reserve margin, operating cost, emissions, time-to-first-feasible, optimality gap, solver time, independent-check time, proof size and memory.

Do not present Tier A synthetic speed as field performance. Report optimization, proof verification and schedule validation time separately.

## 8. Comparison protocol

REopt is an appropriate external reference for distributed energy-resource sizing, microgrids, resilience and techno-economic objectives when assumptions match. PGLib-OPF is a benchmark collection for AC optimal power flow and related network variants; it should not be substituted for a behind-the-meter storage-dispatch benchmark. Freeze dataset version, time series, timestep, units, tariffs, degradation, service priorities, outage assumptions and hardware before comparisons.

A fair comparison requires the same model class, objective, constraints, input data, feasibility tolerances and stopping criterion. When these differ, report a capability matrix instead of declaring one solver or platform faster/better.

## 9. Claim ceiling

A pass may establish exact agreement with the frozen finite exhaustive oracle, correct rejection of listed mutations, and independent feasibility of an evaluated schedule for the stated model. It does not establish input authenticity, physical-model completeness, field reliability, safety certification, or authorization to control equipment.
