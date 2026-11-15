# Highveld Assurance-to-Campaign Interoperability Contract v0.1

**Status:** proposed benchmark-only interface; no production owner schema binding claimed  
**Parent:** [HPT-CROSS-OWNER-REPLAY-001 #878](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/878)  
**Related owners:** Symtropy [#5164](https://github.com/Luminous-Dynamics/symtropy/issues/5164), Mycelix [#6252](https://github.com/Luminous-Dynamics/mycelix/issues/6252), Symthaea [#12046](https://github.com/Luminous-Dynamics/symthaea/issues/12046)

## 1. Why this contract exists

Three existing owners already cover most of the planning problem. The Highveld integration must preserve those boundaries.

- **Mycelix #6252** is designing assurance-obligation evidence acquisition: which unresolved proof obligations a candidate task can address, and whether it is admissible under authority, rights/source-use, safety, schedule, physical-realizability and data-access gates. The plan only identifies tasks capable of producing evidence; it does not mean evidence has been acquired or accepted. The issue is open research work; no completed production integration is claimed.
- **Symtropy #5164 / PR #5240** owns resource- and space-time-constrained observation campaign planning, including bounded exact set-cover and its existing reservation/occupancy owners.
- **Symtropy #5250 / draft PR #5260** already owns the **production-option plus option-specific evidence-campaign composition and joint Pareto layer**. PR #5260 calls the existing optioneering engine #5238 and observation-campaign engine #5240; its issue explicitly says not to duplicate option gates, set-cover search, scheduler, reservation calendar, occupancy logic or accepted-output ledger. Its current issue record describes a Rust implementation and 14 synthetic tests, but the referenced exact-head CI was still pending in the last status recorded there.

The Highveld integration must therefore **not build another campaign optimizer, option-evidence package evaluator or joint Pareto layer**.

The remaining cross-owner problem is narrower:

> Can Mycelix's assurance-obligation and acquisition-admissibility result be bound to the exact option-specific evidence needs and candidate acquisition tasks already handled by Symtropy #5260, so that a blocked, Unknown, rights-ineligible or unbound task is never presented as a valid route to satisfying a proof obligation?

When an admissible task requires a physical observation, its relation to the Symtropy campaign action must be explicit and owner-reviewed. Remote source audits, document review and authority review remain Mycelix tasks and must not be forced into a physical observation scheduler.

The benchmark tests this **assurance-admissibility extension** and evidence lifecycle boundary. Symtropy #5260 remains the owner of option/campaign composition, objective ordering and its Pareto frontier. The Sol-Atlas layer preserves references, reports cross-owner mismatches and measures the resulting assurance outcome; it does not recalculate the frontier.

## 2. Non-goals and binding status

This is a **benchmark-only normalized envelope**, not a production contract and not a new evidence or resource planner.

Before a real adapter is accepted, it must record:
- source repository and exact commit SHA;
- owner profile/schema identifier and version;
- exact source artifact/reference and content digest;
- transformation/adapter version and its own digest;
- semantic mapping review and owner approval;
- rights to use and disclose each source projection.

Until all of those are known, the projection is **UNBOUND**. A matching field name, JSON shape or issue title does not establish semantic equivalence.

No values in this contract authorize an inspection, resource booking, site entry, material receipt, work release, robot command, acceptance decision, professional approval, construction or occupancy.

## 3. The owner-preserving composition model

### Layer A — assurance-obligation admissibility (Mycelix #6252)

For each unresolved proof obligation, consume the Mycelix owner's result about candidate acquisition tasks. Preserve the independent gate dispositions and source evidence:
- obligation coverage / relevance and exact decision scope;
- authority and signer scope;
- legal rights and permitted source use;
- safety permission;
- physical realizability;
- schedule admissibility;
- data access / disclosure permission;
- prerequisite closure;
- source competence and scope.

A Blocked or Unknown task remains rejected with its owner-provided reason. It must not be made selectable by adding a cost or information-value bonus. Source review, clerical review and authority review may be non-physical acquisitions; keep them in the Mycelix layer.

### Layer B — option-specific evidence campaign (Symtropy #5250/#5260)

Consume the already-existing option-evidence package from Symtropy #5260. Its current design:
- evaluates Avoid/Switch/Improve options before spending campaign search on infeasible options;
- retains robust-feasible options whose evidence burden may alter the option-level Pareto set;
- requires option-specific work needs and campaign inputs bound to functional contract, option/design digest, project context, floor, site epoch, coordinate frame, clock domain and currency;
- invokes the existing bounded exact observation-campaign solver #5240 for each option;
- delegates reservation/occupancy and action conflicts to existing owners;
- distinguishes proven optimality, feasible-but-not-proven-optimal, proven infeasibility and search-budget exhaustion;
- handles campaign-cost-included versus campaign-cost-additional explicitly;
- computes the option/evidence Pareto frontier without collapsing distinct objectives.

The current implementation details are sourced from [Symtropy issue #5250](https://github.com/Luminous-Dynamics/symtropy/issues/5250) and [draft PR #5260](https://github.com/Luminous-Dynamics/symtropy/pull/5260). The Highveld integration must consume this owner result; it must not reimplement it.

### Highveld's cross-owner bridge

The proposed extension is only the reviewed binding between Mycelix proof obligations / admissible acquisition tasks and the option-specific evidence needs and campaign actions in Symtropy #5260.

- A remote acquisition stays in Mycelix and is not converted into a scan.
- A physical acquisition may bind to a Symtropy observation action only through an exact owner-reviewed relation.
- An option-evidence package remains identified by its functional contract, option design, project/site epoch, frame, clock and currency bindings.
- The bridge preserves Mycelix gate results and Symtropy's campaign completeness/optimality result; it does not rewrite either.
- If Mycelix rejects a task, that task cannot satisfy the linked obligation in the cross-owner replay. The affected option/evidence package must be re-evaluated by the owning Symtropy engine; Sol-Atlas must not patch its result locally.
- If no exact mapping exists, the relation is UNBOUND and cannot count as obligation coverage.

No additional joint Pareto frontier or campaign selector belongs in Sol-Atlas. The relevant question is whether the owner-native package remains admissible and decision-sufficient after the assurance obligations, rights and authority constraints are attached.

## 4. Benchmark-only normalized envelope

The following are semantic concepts for test fixtures, not asserted fields in Symtropy or Mycelix production schemas.

| Concept | Purpose | Authority / owner |
|---|---|---|
| Decision episode | Exact decision scope and cutoff time | Decision owner; source refs only |
| Unresolved obligation | Claim/proof-obligation ID, criticality, exact subject/revision and missing evidence type | Mycelix |
| Candidate acquisition | How evidence might be obtained; cost/currency; prerequisites; source and scope | Mycelix |
| Acquisition admissibility | Separate gate dispositions and reasons; Unknown remains Unknown | Mycelix |
| Observation action | Physical task, coverage, site/epoch, times, crew/instrument/resource use and energy | Symtropy |
| Acquisition/action binding | Reviewed relation for physical acquisitions only | Owner-approved adapter |
| Existing reservations / occupancy | Current context and constraints used for scratch admission | Symtropy |
| Candidate plan | Selected acquisition and observation references, rejected alternatives, remaining obligations | Composition result |
| Observation result | Actual acquired measurement with source, time, calibration, coverage and digest | Physical observation owner |
| Qualification result | Independent decision whether acquired evidence satisfies the obligation | Relevant claim/acceptance owner |

The envelope carries references to these records; it does not copy or replace them. Exact property names, enumeration values, temporal conventions and identifiers must be mapped from pinned owner profiles before any non-synthetic adapter can exist.

## 5. Assurance-admissibility and result semantics

### Hard gates first

Mycelix remains authoritative for the admissibility result it owns. An Unknown or Blocked authority, rights/source-use, safety, schedule, physical-realizability or data-access gate is non-compensatory. It cannot be overridden by an option score, lower campaign cost, greater information gain, higher coverage or a different Pareto position.

The Highveld bridge reports the owner result; it does not reinterpret an Unknown as Allowed or merge gates into a weighted score.

### Preserve Symtropy's option/campaign Pareto output

Symtropy #5260 already owns the production-option × evidence-campaign package and joint objective/Pareto computation. Keep its cost boundary, objective ordering, completeness and optimality proof status intact.

The Highveld evaluator must not:
- schedule or replace observation actions;
- recompute spatial occupancy or resource conflicts;
- change campaign objective ordering or search budget;
- combine money across different currencies;
- add campaign cost twice;
- create a second construction/observation Pareto frontier;
- make an incomplete or budget-exhausted campaign appear proven optimal.

### Cross-owner obligation closure

Let U be the unresolved Mycelix obligation set for one exact decision episode. A benchmark package is **assurance-admissible** only if:
1. each task claimed to address an obligation is explicitly mapped to that obligation by the Mycelix output or a reviewed adapter;
2. the task passes every required Mycelix admissibility gate;
3. any physical observation is bound to an exact Symtropy #5260/#5240 action and the option's same project/site/epoch/frame/clock/design context;
4. the Symtropy owner result declares the corresponding work need covered by a feasible campaign;
5. the evidence state remains **planned**, not observed, verified, accepted or qualified.

If any mapping or proof obligation is absent, the corresponding obligation remains unresolved. A campaign plan is not acquired evidence. An acquired observation is not automatically sufficient proof.

### Report, do not re-optimize

The bridge report records:
- original Mycelix obligation and acquisition disposition, including all hard-gate reasons;
- exact relation to the Symtropy option-evidence package and observation action, or UNBOUND;
- Symtropy option/campaign identity, feasibility, completeness, optimality proof status and owner-computed objective fields;
- obligations still uncovered after admissibility gating;
- new physical evidence still required and who owns its acquisition/verification;
- exact source/profile/commit/digest and adapter identities.

Keep acquisition cost, observation cost, crew time, energy, duration and source currency as separately reported owner dimensions. No cross-owner weighted scalar or new Pareto calculation is authorized by this contract.

## 6. Required adversarial fixtures

Start with at least 15 deterministic synthetic fixtures. Each should carry an independent expected outcome and be tested under input permutation.

1. **Admissible remote source audit:** resolves one obligation but has no physical observation; it must remain a valid acquisition without a fake Symtropy action.
2. **Admissible physical survey:** explicit owner binding exists and a feasible campaign covers the declared obligation.
3. **Cheap authority-blocked acquisition:** excluded even though it would make the nominal plan cheaper.
4. **Rights-Unknown source:** excluded; no public redistribution or use by virtue of a digest.
5. **Safety-Unknown observation:** excluded even if its geometry/coverage looks complete.
6. **Unbound acquisition/action pair:** physical task has no reviewed owner binding and contributes no obligation coverage.
7. **Two individually feasible observations with a shared instrument conflict:** joint selection must reject the conflict.
8. **Spatially incompatible observations:** both are excluded from one simultaneous campaign under the owner's occupancy semantics.
9. **Shared resource capacity exceeded:** individually valid actions cannot be selected together.
10. **Prerequisite blocked:** a dependent action cannot be selected until the prerequisite is admissibly covered.
11. **Common-cause sensor evidence:** two reports sharing one calibration/controller/source cannot be counted as independent corroboration.
12. **Currency mismatch:** aggregate comparison is rejected or reported in separate ledgers; no arithmetic across currencies.
13. **Budget/node cap exhausted:** report SEARCH_INCOMPLETE and retain uncovered obligations; never claim infeasible or optimal.
14. **No feasible complete plan:** partial coverage and exact blockers are reported.
15. **Complete plan exists only through non-greedy combination:** exact tiny oracle finds it; greedy baseline reports its actual gap.
16. **Inconclusive measurement:** planning coverage is recorded, but the obligation remains unresolved after the outcome.
17. **Late correction or revocation:** the as-of-decision result is recomputed; historical events are preserved.
18. **Duplicate action / obligation / binding ID:** typed input rejection, not silent overwrite.
19. **Plan permutation:** objective values and Pareto frontier remain invariant under row/action order; digest still binds the exact byte representation.
20. **Owner output tampering:** changed source digest, schema version, site epoch or adapter digest invalidates the bound projection.

The first 15 form the v0.1 tiny-oracle minimum. Cases 16–20 expand to outcome-aware and adversarial owner-replay tests.

## 7. Baseline arms and benchmark method

Run the same frozen decision episode, obligation set, option catalogue, source rights/authority/safety dispositions, project/site context and observation constraints through:

- **B0 — no acquisition:** no admissible acquisition is selected; all obligations remain unresolved.
- **B1 — Mycelix-only:** run the assurance-obligation acquisition planner without attaching a physical campaign.
- **B2 — Symtropy-native:** run the existing #5260 option-evidence package evaluator with its #5238 option and #5240 observation campaign owners; no external Mycelix admissibility extension.
- **B3 — sequential bridge:** obtain Mycelix admissibility first, then pass only reviewed, admissible option-evidence task bindings to the existing Symtropy #5260 package evaluator. Do not modify either owner engine.
- **B4 — independent cross-owner oracle:** for tiny frozen fixtures, independently verify obligation-to-acquisition mapping, gate admissibility, physical-action binding and the expected owner package/remaining-obligation result.
- **B5 — candidate bridge:** the proposed Highveld projection/evaluator, with no access to oracle labels.

The oracle must not call the same owner implementation's gate, feasibility or objective functions to generate its expected answer. For small fixtures, enumerate the cross-owner bindings and hard-gate dispositions independently, then compare exact mappings, rejected tasks, remaining obligations and the referenced Symtropy package status. Do not recompute Symtropy's Pareto objective in Sol-Atlas.

Report performance by fixture family. A single blocked/Unknown selection or false obligation closure is a hard failure and cannot be averaged away.

## 8. Metrics and prospective targets

These are proposed targets, not results.

| Measure | Target |
|---|---:|
| Exact agreement with independent cross-owner oracle on tiny fixtures | 100% |
| Blocked/Unknown/rights-ineligible acquisition treated as admissible | 0 |
| Unbound physical acquisition/action relation counted as obligation coverage | 0 |
| Wrong option/design/site/epoch/frame/clock/currency binding accepted | 0 |
| Search-budget exhaustion or incomplete Symtropy package presented as proven optimal | 0 |
| Planned observation promoted to acquired evidence | 0 |
| Acquired evidence promoted to qualified proof without verifier | 0 |
| Owner-native reservation/occupancy mutations from planning | 0 |
| Unresolved obligation silently removed from the denominator | 0 |
| Same input, same semantic bridge report under input permutation | 100% |

Also report:
- Mycelix admissibility precision and recall against independently adjudicated obligations;
- percentage of critical obligations with a genuinely admissible physical or non-physical acquisition route;
- number of blocked/unknown tasks retained with explicit reasons;
- percentage of physical tasks with exact owner-reviewed binding to Symtropy;
- false obligation closure and evidence-laundering counts;
- Symtropy option/campaign result completeness and search-proof status, as emitted by that owner;
- decision time, expert review effort, rights/disclosure burden and evidence still missing;
- accepted outcome rates only after actual evidence is acquired and independently verified.

Do not score the Highveld bridge for generating a new Pareto frontier. That is already owned and benchmarked by Symtropy #5260. Benchmark the correctness of the assurance extension and the preservation of the owner result.

## 9. Execution sequence

1. **Review the existing Symtropy composition:** pin and review #5260's exact branch/head, option-evidence schema, objective contract, tests and CI. Do not recreate its option/campaign layer.
2. **Review the acquisition owner:** inspect Mycelix #6252's E0 proposal and eventual implementation; pin exact schema/profile/commit, gate semantics, cost units and source references. Until implemented and pinned, mark it UNBOUND.
3. **Approve the cross-owner mapping:** define reviewed obligation-to-acquisition and physical-acquisition-to-observation bindings without field-name inference.
4. **Build synthetic replay fixtures:** replay the same Highveld panel decision episode through Mycelix and Symtropy owner outputs and the thin Highveld bridge. Preserve all original owner reasons and result statuses.
5. **Run independent adjudication:** two reviewers label the cross-owner expected outcomes; a third resolves disagreements; freeze digests and hidden holdout with an access-controlled custodian.
6. **Run controlled physical observation only afterward:** a separate competent workcell/site safety and metrology process is required. Planning remains non-authorizing.

## 10. Claim ceiling

A perfect benchmark result shows only that the candidate reproduces the independently specified planning result for the frozen synthetic candidate universe and owner projections. It does not prove actual observation coverage, instrument calibration, legal admissibility, source authenticity, material acceptance, robot safety, physical installation, building compliance, professional approval, or fitness for occupancy.

