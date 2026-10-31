# Energy Dispatch MILP Translation Validation v0.1

**Status:** design contract for the independent validator; not an implementation or a qualification result.  
**Parent work:** Sol-Atlas #333, building on #319, #321 and #330.  
**Claim boundary:** this document defines what must be proven before a SCIP/VIPR certificate is accepted as evidence about the original dispatch request.

## 1. Why this layer exists

SCIP/VIPR can prove that a particular integer linear program has a claimed optimum. That proof does not, by itself, establish that the LP represents the source request. A perfectly valid certificate for a mistranslated model is still a proof of the wrong problem.

The assurance chain therefore has separate obligations:

1. **Source contract:** define the exact integer dispatch semantics of the validated request.
2. **Independent reference model:** derive a canonical integer model directly from those semantics.
3. **Translation validation:** parse the emitted LP and show semantic equivalence with the reference model.
4. **Optimization:** solve only the LP whose source and semantic identities are bound into the validation receipt.
5. **Proof checking:** independently validate the certificate for that exact LP.
6. **Decision witness:** bind selected outcomes to the complete feasible frontier and versioned policy.

The validator is deliberately independent of the LP emitter. It must not call or reuse `emit_lp`, `objective_terms`, `append_constraint`, the emitter's variable-name helpers, or its bound-generation helpers. Sharing the strict request DTO is acceptable; sharing the implementation of the translation is not.

## 2. Canonical semantic model

The validator should represent a model as a deterministic, versioned structure:

- model schema ID;
- request byte SHA-256 and parsed request identity;
- objective ID and direction;
- sorted variable declarations with domain and exact lower/upper bounds;
- sorted constraints with stable semantic IDs, exact integer coefficients, relation and RHS;
- canonical semantic-model SHA-256.

Each linear expression is a finite map from variable ID to signed integer coefficient. Zero coefficients are forbidden. Variable names are stable identifiers, not display labels.

For v0.1 all decision variables are integer-valued; the charge-mode variables are binary. No continuous variable is permitted. The model parser must reject any undeclared variable or any unused declaration not in the reference inventory.

### 2.1 Per-interval variables

For interval index `i` (rendered in the LP with the three-digit zero-padded suffix), define:

| Variable | Meaning | Domain / upper bound |
|---|---|---|
| `grid_import_i` | Energy imported from grid | integer, outage ? 0 : grid import limit |
| `charge_input_i` | Energy sent into battery | integer, charge input limit |
| `soc_withdrawal_i` | Battery energy withdrawn from stored SOC | integer, min(discharge limit, capacity) |
| `charge_gain_i` | Integer SOC gained after charging efficiency | integer, floor(charge limit × charge numerator / denominator) |
| `discharge_delivered_i` | Energy delivered after discharge efficiency | integer, floor(discharge limit × discharge numerator / denominator) |
| `soc_after_i` | Battery state of charge at interval end | integer, battery capacity |
| `essential_served_i` | Essential demand served | integer, essential demand |
| `essential_unserved_i` | Essential demand not served | integer, essential demand |
| `flexible_served_i` | Flexible demand served | integer, flexible demand |
| `flexible_unserved_i` | Flexible demand not served | integer, flexible demand |
| `pv_used_i` | PV energy used | integer, PV availability |
| `pv_curtailed_i` | PV energy curtailed | integer, PV availability |
| `charge_mode_i` | 1 permits charging; 0 permits withdrawal | binary |

All listed variables have lower bound zero. The binary mode variable is declared only in the LP Binaries section; all other per-interval variables are declared in Generals. Each must also appear exactly once in the bound inventory.

## 3. Required constraints

The reference builder must derive each row from the request fields, without copying row construction from the emitter. For each interval `i), with `prev_soc = initial_soc` for the first interval and `soc_after_(i-1)` otherwise, the model contains the following semantic rows.

| Row ID | Mathematical relation | Purpose |
|---|---|---|
| `soc_transition_i` | `soc_after_i + soc_withdrawal_i - charge_gain_i - prev_soc = 0` | SOC state transition |
| `withdrawal_prev_soc_i` | `soc_withdrawal_i - prev_soc <= 0` | Cannot withdraw more than opening SOC |
| `charge_mode_limit_i` | `charge_input_i - charge_limit × charge_mode_i <= 0` | Charging only in charge mode |
| `discharge_mode_limit_i` | `soc_withdrawal_i + discharge_ceiling × charge_mode_i <= discharge_ceiling` | Withdrawal only outside charge mode |
| `charge_floor_lower_i` | `charge_den × charge_gain_i - charge_num × charge_input_i <= 0` | Integer floor lower side |
| `charge_floor_upper_i` | `charge_num × charge_input_i - charge_den × charge_gain_i <= charge_den - 1` | Integer floor upper side |
| `discharge_floor_lower_i` | `discharge_den × discharge_delivered_i - discharge_num × soc_withdrawal_i <= 0` | Integer floor lower side |
| `discharge_floor_upper_i` | `discharge_num × soc_withdrawal_i - discharge_den × discharge_delivered_i <= discharge_den - 1` | Integer floor upper side |
| `essential_partition_i` | `essential_served_i + essential_unserved_i = essential_demand_i` | Essential demand accounting |
| `flexible_partition_i` | `flexible_served_i + flexible_unserved_i = flexible_demand_i` | Flexible demand accounting |
| `pv_partition_i` | `pv_used_i + pv_curtailed_i = pv_available_i` | PV accounting |
| `bus_balance_i` | `pv_used_i + grid_import_i + discharge_delivered_i - essential_served_i - flexible_served_i - charge_input_i = 0` | Interval energy balance |

There is exactly one horizon constraint:

| Row ID | Mathematical relation | Purpose |
|---|---|---|
| `terminal_reserve` | `soc_after_last >= terminal_reserve` | Hard terminal reserve |

The reference builder must use checked integer arithmetic. Overflow is a validation failure, never a saturating or wrapping result.

### 3.1 Integer efficiency semantics

The two inequalities for a ratio (n/d), where (0 < n <= d), are equivalent to:

[
q = \left\lfloor \frac{n x}{d} \right\rfloor
]

only because both (q) and (x) are integers. The validator must preserve both sides and the strict integer offset (d-1). Replacing these rows with a floating-point equality or a rounded coefficient changes the feasible set.

The validator should include hand-calculated cases for:
- zero input;
- input below the first non-zero efficiency quantum;
- exact integer quotient;
- non-zero remainder;
- maximum admissible input;
- numerator equal to denominator;
- numerator one less than denominator.

## 4. Objective contracts

Generate all twelve objective/direction instances: six objective IDs × minimize/maximize.

| Objective ID | Expression |
|---|---|
| essential unserved | sum of `essential_unserved_i` |
| flexible unserved | sum of `flexible_unserved_i` |
| operating cost | sum of `grid_cost_units_per_quantum_i × grid_import_i` |
| grid carbon | sum of `grid_carbon_grams_per_quantum_i × grid_import_i` |
| curtailed PV | sum of `pv_curtailed_i` |
| terminal SOC | `soc_after_last` |

The objective ID, direction, and exact objective term map are part of the canonical model identity. An objective swap must fail even if the altered model remains feasible.

## 5. Strict LP parser

Implement a parser for the emitted LP subset, not a permissive general-purpose LP reader. The accepted grammar is deliberately small:

1. Initial comment lines.
2. Exactly one Minimize or Maximize section with one named objective expression.
3. Exactly one Subject To section with uniquely named linear rows.
4. Exactly one Bounds section with one lower/upper interval per non-binary variable, plus the binary bounds as required by the emitted profile.
5. Exactly one Binaries section.
6. Exactly one Generals section.
7. Exactly one terminal End marker and no trailing non-whitespace content.

The parser must reject duplicate sections, duplicate declarations, duplicate row names, unknown tokens, malformed signs, zero coefficients, integer overflow, unknown symbols, undeclared references, missing declarations and unsupported syntax. It must not silently ignore an unknown section or an unparsed suffix.

The accepted profile should document whether harmless differences in whitespace, line wrapping, expression term order and comments are permitted. Such differences may produce different LP byte hashes while preserving the same canonical semantic-model digest.

## 6. Equivalence algorithm

The independent reference builder and LP parser produce two canonical models. Compare them structurally:

- exact objective ID, sense, coefficients and constant;
- exact variable set and domain;
- exact lower and upper bounds;
- exact named constraint set;
- exact relation, coefficient map and RHS for every row.

For v0.1, prefer exact normalized row equality over broad algebraic simplification. If positive integer row scaling is accepted, divide all coefficients and RHS by their positive GCD. Never allow a normalization step to erase a changed variable domain, bound, objective, relation, or RHS. Equality sign normalization may flip all coefficients and RHS together. Inequality sign normalization is permitted only when the relation is also reversed.

The result must be a deterministic report with:
- request digest;
- source model schema;
- LP byte digest;
- objective identity;
- reference-model digest;
- parsed-model digest;
- equality verdict;
- stable mismatch codes and row/variable IDs;
- parser/verifier source and build identities;
- claim ceiling.

## 7. Certificate binding and trust boundary

A solver proof is admissible only when all identities match:

[
(request\_sha256, objective, direction, lp\_sha256,
semantic\_model\_sha256, validator\_identity, certificate\_sha256)
]

The VIPR certificate verifier still checks the solver proof. Translation validation establishes that the proof's LP is the intended model. These are distinct claims and must be reported independently.

The current #330 workflow disables presolving and cutting-plane separation for the original-model certificate path. If future runs enable separation, require the documented completion step before certificate checking; do not silently accept incomplete logs or artifacts. Any future presolved-model path must bind the presolved model identity and a separately checked mapping back to the source model.

## 8. Adversarial corpus

Create a deterministic mutation corpus with stable mutation IDs, seed, target path, input digest, output digest and expected verdict. Start with at least 1,000 mutations.

Required mutations:

- delete, duplicate, rename or substitute each row family;
- alter a coefficient sign, magnitude, RHS or comparison relation;
- remove or widen a lower/upper bound;
- change an integer declaration to continuous or binary;
- remove a binary declaration or add a fake one;
- remove a variable declaration or add an unknown variable use;
- change objective coefficient, direction or identity;
- replace the request, source digest, LP file, model index or certificate association;
- perturb capacity, reserve, efficiency numerator/denominator, outage, grid limit, PV and both demand classes;
- truncate the file, inject trailing text, repeat a section, or insert unsupported LP syntax.

A negative-control mutation is not merely "test passed" because parsing failed. Record whether the expected failure happened at byte parsing, grammar parsing, semantic comparison, feasibility checking, or certificate binding.

Harmless serialization variants should be accepted only if the parsed semantic model remains identical. Report false accepts, false rejects and unsupported grammar coverage separately.

## 9. Acceptance gates

- Every frozen fixture and all twelve objective/direction models pass independent translation validation.
- At least 1,000 deterministic semantic mutations; zero false accepts.
- 100% of declared LP grammar tokens accounted for; no silently ignored content.
- All arithmetic uses checked integer operations.
- 100% of certificates bind to the exact request and semantic-model receipt.
- Independent validator compiles without linking the LP emitter module.
- Small-instance feasibility agreement across source-level integer evaluator, canonical model, parsed LP and independent exhaustive enumeration.
- Separate timing for parse, canonicalization, semantic compare, solver execution and proof checking; report p50/p95/p99.
- Latest-head commit SHA, Cargo.lock SHA-256, toolchain, SCIP release/checksum, VIPR source commit and checker build identity accompany every qualification.

## 10. Explicit non-claims

Passing this validator does not prove that the dispatch model is physically complete, that source data are authentic, that forecasts are accurate, that grid constraints match a real network, or that an operational action is safe or authorized. It proves only the exact semantic correspondence of a request contract and emitted integer LP under this versioned model schema, and it binds subsequent optimization evidence to that model.

## 11. Industry comparison

SCIP exact solving and VIPR are the proof-production/proof-checking comparison. OR-Tools CP-SAT remains a useful independent solver comparison, with FEASIBLE, OPTIMAL, INFEASIBLE and UNKNOWN kept distinct. Neither solver comparison substitutes for translation validation.

For later scale-up, compare operational value and model coverage against REopt, PyPSA and GenX only on matched cases and equivalent constraints. Network-constrained tests should use public power-flow/OPF datasets (for example PGLib-OPF or SimBench), not reinterpret the current two-interval synthetic microgrid fixture as a network-planning benchmark.

## References

- SCIP exact solving and certificate generation: https://scipopt.org/doc-10.0.3/html/EXACT.php
- SCIP proof/presolving caveats: https://www.scipopt.org/doc-10.0.3/html/FAQ.php
- OR-Tools CP-SAT statuses: https://github.com/google/or-tools/blob/stable/ortools/sat/docs/troubleshooting.md
- REopt platform/API: https://developer.nlr.gov/docs/energy-optimization/reopt/
- PyPSA storage/optimization constraints: https://docs.pypsa.org/latest/api/networks/constraints/
- PGLib-OPF benchmark: https://github.com/power-grid-lib/pglib-opf
