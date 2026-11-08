# City Service Restoration Feasibility v0.1

Offline Rust reference oracle for [CITY-SERVICE-RESTORATION-001 #639](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/639).

## What it answers

Given an interruption and a proposed ordered repair plan, does the proposed plan have current, scope-matched evidence for its necessary parts/lots, reservations, deliveries, workforce, skills, tools, calibration, access, isolation, tolerance and repair-envelope prerequisites—and are there any proven resource or sequence conflicts?

This is a read-only evidence-composition evaluator. It does not schedule crews, route vehicles, dispatch work, allocate inventory, reserve stock, calculate continuity timelines, authorize isolation, control robots, approve engineering work or commission equipment.

## The five separate axes

1. **Plan readiness:** `feasible`, `infeasible`, or `unknown`.
2. **Work execution:** `not_started`, `in_progress`, `failed`, `completed`, or `unknown`.
3. **Inspection:** `not_performed`, `pass`, `fail`, or `unknown`.
4. **Commissioning:** `not_performed`, `pass`, `fail`, or `unknown`.
5. **Service state:** `adequate`, `constrained`, `inaccessible`, or `unknown`.

A feasible plan does not mean that work has started. Completed work is not accepted without the relevant inspection evidence. Inspection does not imply commissioning. Commissioning does not prove that essential service has been restored and remained adequate.

Only the continuity owner (#574) may establish final service state and a stable-restoration interval. This evaluator never synthesizes a recovery timestamp.

## Verdict precedence

- A proven hard contradiction makes the plan `infeasible`, even when unrelated evidence is missing.
- If no hard contradiction is proven but any required evidence is absent, stale, ambiguous or not authoritative, the plan is `unknown`.
- Only when all required evidence is current, scope-matched and non-conflicting is the plan `feasible`.
- Unknown is never treated as Feasible. No weighted score is emitted.

Examples of proven conflicts include revoked lots, wrong product revisions, double-booked exclusive resources, impossible delivery or worker windows, expired calibration, denied access, protected-service isolation without mitigation, out-of-envelope adjustments or repairs, and explicit work-precedence violations.

Examples of Unknown include missing custody/receiving acceptance, absent reservations, unqualified substitutions, unavailable skill evidence, unverified bytes, missing owner rights/authority, missing whole evidence domains and incomplete rework accounting.

## Owner boundaries

The input corpus models owner-native facts and their states, but it does not claim those source schemas have been integrated. Production composition must consume:
- #574 for continuity timelines and stable restoration semantics;
- #623/#624 for common district scope and owner envelopes;
- #500/#637 for exact payload-byte reports where required;
- #573 for delivery, route, unloading and receiving feasibility;
- #555 for accessibility and affected-group constraints;
- Symtropy #4181 for service cassette interfaces, isolation, repair and recommissioning;
- Symtropy #4043 for as-built inspection and repair/re-verification;
- Symtropy #3605 for accepted output, labour, time, defects and rework;
- Mycelix #4814 for supplier, production, inventory, substitution and skills;
- Mycelix #5149 for lot identity, custody, approvals, correction and revocation.

No duplicate inventory ledger, scheduler, robot controller, dispatch system, continuity model or professional approval engine is introduced.

## Synthetic corpus

`CITY_SERVICE_RESTORATION_EVAL_CORPUS_V0_1.json` is the executable 32-case input fixture. The original descriptive proposal remains in `CITY_SERVICE_RESTORATION_CORPUS_V0_1.json`. The evaluator computes from each case's `input` object and compares its complete output against the frozen `expected` object; it never reads expected values when deriving a verdict.

The corpus includes positive controls, hard conflicts, missing-evidence controls, lifecycle-stage separation and overclaim negatives. All resources, quantities, times, qualifications, access, approvals and states are synthetic mechanics only. They are not Johannesburg or South African operating data, engineering thresholds, safety rules or workforce facts.

The companion JSON Schema pins the v0.1 input/output contract. New required fields or changed verdict semantics require a versioned contract update and explicit fixture changes.

## Reproduction

From repository root:

```sh
cargo fmt --manifest-path tools/city-service-restoration-bench/Cargo.toml --all -- --check
cargo test --locked --manifest-path tools/city-service-restoration-bench/Cargo.toml
cargo clippy --locked --manifest-path tools/city-service-restoration-bench/Cargo.toml --all-targets -- -D warnings
cargo run --locked --quiet --manifest-path tools/city-service-restoration-bench/Cargo.toml -- \
  docs/research/city-service-restoration/CITY_SERVICE_RESTORATION_EVAL_CORPUS_V0_1.json \
  --source-commit <40-hex-commit>
```



## Owner-native service-state enum

The projected continuity service state is restricted to `adequate`, `constrained`, `inaccessible` or `unknown`. Unsupported strings do not pass through as service outcomes; they remain Unknown. A non-Unknown state additionally requires current continuity-owner evidence, exact scope match and the stable-adequacy predicate where appropriate.

## Part availability interval semantics

Part/lot availability is checked against the exact half-open interval during which the consuming work package requires that lot (`use_start_at`, `use_end_at`), not against the entire planning window. The planning window constrains the plan; it is not itself a part-consumption interval. If the use interval is missing or malformed, the plan remains Unknown or Infeasible according to whether the data is absent or contradictory. This prevents rejecting a lot that arrives after planning begins but before its actual use, while still rejecting a lot that arrives late or expires before use is complete.

## Benchmarks

E1 targets:
- 100% known-answer agreement on the frozen corpus;
- zero false-Feasible results on hard-conflict and missing-evidence controls;
- proven hard conflicts take precedence over unrelated Unknown evidence;
- zero exclusive allocation conflicts missed for overlapping half-open resource intervals;
- zero promotion from plan readiness to execution, inspection, commissioning, service restoration or action authority;
- deterministic exact-head receipt reproduction.

Report plan-verdict counts and reason codes. Do not collapse Unknown and Infeasible. Do not optimize by rejecting every plan: preserve positive controls and stage-separation cases.

For E2, compare the projection against source-owner adapters and a separately authored domain oracle. For E3, use a guarded non-hazardous mock-up and the accepted-output owner. E4/E5 require qualified professional design, risk assessment, statutory approvals, independent commissioning, sustained operational measurements and replication.

## Competitive comparison

The baseline is not “software versus no software.” Compare the integrated workflow against:
- a conventional facilities/maintenance-management plan;
- manual planner + supplier/procurement coordination + access/work-order systems;
- a CMMS/enterprise asset-management workflow;
- a human-led best-of-breed digital twin + delivery + inventory + work-package + inspection toolchain;
- the Luminous owner-composed evidence chain.

Use identical synthetic and later rights-cleared project fixtures. Compare plan false-Feasible/false-Infeasible/Unknown rates, blocker-localization completeness, hidden resource conflicts, evidence completeness, total human review time, time-to-decision, rework and service-deficit hours. Do not infer missing vendor capabilities from inaccessible records. The evaluator must not claim superior delivery performance until real, matched, independent field outcomes exist.

## Claim ceiling

A passing result establishes only that the evaluator follows the declared synthetic feasibility and lifecycle-separation rules on the frozen corpus. It does not prove that a real part exists, a worker is qualified, a site is accessible, a repair is safe, a delivery will occur, work will succeed, or a city service will be restored. `action_authority` is always `none`; causal effect is always `false`.
