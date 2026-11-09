# CityMorph service-scoped opportunity receipt v0.1

## Purpose

The current CityMorph reference evaluator returns a group-level access outcome and aggregate reachable capacity potential. It also lists accessible and unknown service IDs. That is adequate for category-level accessibility, but a downstream restoration composition may bind one named clinic, school, shelter or other service. It must not infer the facility's individual capacity or state from the group/category aggregate.

This contract adds a deterministic, service-scoped evidence row created during the existing group evaluation. It does not perform a second route calculation, change aggregate capacity accounting, allocate slots, ration capacity, or claim actual service delivery.

## Output location

Each group result retains its existing fields and adds:

- `service_opportunities`: deterministic array, one row per same-category service considered for that group.

The existing fields remain authoritative for the group aggregate:
- `status`
- `accessible_service_ids`
- `unknown_service_ids`
- `reachable_capacity_potential`
- `demand` and `demand_unit`
- group-level `reason_codes`

No existing aggregate field is redefined by the new rows.

## Per-service row

Each row reports:

- group ID, service ID, service node and category;
- route state: known within envelope, unknown within envelope, known outside envelope, unknown outside envelope, or no known path;
- known route minutes where a known in-envelope path exists;
- separate availability, eligibility, affordability and step-free assessments;
- capacity evidence state, declared capacity and unit;
- reachable capacity potential only where all required route/service constraints are known and pass;
- service-specific reason codes, sorted deterministically.

A row-level `AccessiblePotential` means only that the service is a known usable opportunity under the declared group constraints. It does not mean adequate group-wide supply or that the group received the service. Group adequacy remains the existing group-level decision.

## Non-substitution rules

- A route does not prove that a service is open, affordable, eligible, step-free or capacity-adequate.
- Unknown route or facility metadata remains Unknown, never a favorable default.
- A closed, unaffordable, ineligible or inaccessible service contributes zero reachable potential for the group, but its declared capacity is preserved separately where known.
- Unknown capacity is null/Unknown, not numeric zero.
- Per-service capacities are not summed again by downstream consumers that already use the group aggregate.
- A group/category aggregate must never be reverse-engineered into an individual facility capacity.
- Potential capacity is not a booking, allocation, trip served, visit delivered or resident outcome.

## Determinism and invariants

- Rows are sorted by service ID regardless of input order.
- Duplicate service IDs remain Unsupported.
- Sum of per-service reachable potential for a group must equal the current group-level reachable capacity potential.
- Each service contributes at most once to that sum.
- Permuting the service input does not change normalized per-group service rows or aggregate results.
- Changing one service's node, capacity, price, eligibility, availability or step-free state changes only the affected row and any group/category aggregate that logically depends on it.

## Composition boundary

The follow-on effective-access composition (#683) needs a named-service binding. It may consume one matching service row by service ID, group ID, service node and declared source/scope identity. Until this owner receipt is qualified, a synthetic service-scoped binding in #683 is not a direct executable integration with CityMorph #564.

Claim ceiling: synthetic per-group service-opportunity composition only.
