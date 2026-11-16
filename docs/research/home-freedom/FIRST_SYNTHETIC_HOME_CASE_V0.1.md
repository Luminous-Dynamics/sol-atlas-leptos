# First synthetic home case v0.1

**Study:** HOME-FREEDOM-COMPILER-001  
**Maturity:** SYNTHETIC_SCREENING_ONLY  
**Execution:** NOT_SIMULATED

This fixture exercises the resident brief and candidate-set data contracts with two competing Highveld dwelling archetypes: a courtyard home and a compact row home with service yard.

It deliberately does not contain a real site, personal information, actual Johannesburg weather, local costs or building-control approvals. The two room programmes are symbolic allocations only. Their areas add to their stated internal floor-area targets, but they are not drawn plans and do not claim those areas fit a real plot or comply with any code.

The candidate set therefore keeps every safety/legal/technical gate at `UNKNOWN`, cost state `NOT_ESTIMATED`, thermal state `NOT_SIMULATED`, and future changes at `RESERVED_SPACE_ONLY`. Both options remain unranked. This is a positive control for representing choices without manufacturing a false-feasible design.

## Case contents

- `HOME_DESIGN_BRIEF_SYNTHETIC_EXAMPLE_V0.1.json`: resident intent, hard requirements, preferences, and explicit unknown site/budget evidence.
- `HOME_DESIGN_CANDIDATE_SET_SYNTHETIC_EXAMPLE_V0.1.json`: two unranked concept-space programmes with the same household brief and blocking unknowns.
- `HOME_DESIGN_CANDIDATE_SET_V0.1.schema.json`: machine-readable output contract for this candidate-set profile.

## P0 expected disposition

- Two candidates represented: PASS.
- Every required resident brief requirement has a traceable gate row: structural representation only; no requirement is marked technically satisfied.
- Unknown site/code/rights/soil/structural/fire/accessibility states preserved: PASS.
- Unknown costs preserved as null with `NOT_ESTIMATED`: PASS.
- Thermal performance not inferred from a pool or architectural label: PASS.
- No candidate promoted to buildable or ranked: PASS.
- Independent JSON Schema validation and Rust semantic qualification: NOT YET IMPLEMENTED.
