# City morphology effective-access corpus v0.1

Parent: Sol-Atlas [CITY-MORPH-ACCESS-001 #555](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/555) and [CITY-MAKER-001 #549](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/549).

The Johannesburg source register contains 22 candidate records, including the JDA's 2026 Universal Design Access Plan completion announcement. The plan is recorded as a public method/roadmap source only; its full audit records and facility-level data remain unverified.

This 24-case corpus is entirely synthetic. It tests network access, usable-service constraints, individual group thresholds, reachable capacity potential and once-counted aggregate category capacity. It is not a Johannesburg data set or a real accessibility evaluation.

## Interpretation

- Network route cost is computed from declared graph edges, not Euclidean distance.
- A known route uses current network evidence, open edges and known step-free edges where required.
- An unknown route can yield Unknown but never a favorable accessible result.
- Service opening window, eligibility, affordability, step-free support and capacity evidence are independent filters.
- Group-level reachable capacity is a potential, not an assignment. The same facility can be a potential option for multiple groups, so the city/category total separately counts each facility once and sums demand across groups.
- No rationing, equity-allocation or composite accessibility score is implemented.
- Group status: AccessibleAndAdequatePotential, AccessibleButCapacityConstrained, Unknown, Inaccessible.
- Case disposition: FeasibleForSyntheticCase, Infeasible, Unknown, Unsupported.

The expected fields are known-answer fixtures only. The Rust reference oracle must recompute from inputs and must never read expected results as its decision path. Synthetic values must not be used as local travel-time, population, service-capacity, price or policy estimates.

The next evidence tier is reproduction of published UN-Habitat / SDG calculations and one published multidimensional accessibility method on a rights-cleared published case, followed by a source-readiness audit before any Johannesburg profile is populated.
