# Highveld Cross-Owner Binding and Readiness Register v0.1

**As-of:** 15 November 2026 (UTC)  
**Parent:** [HPT-CROSS-OWNER-REPLAY-001 #878](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/878)  
**Corpus:** [#870](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/870)  
**Qualifier / benchmark protocol:** [#877](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/877)

## 1. Purpose

This register distinguishes four very different states:

1. Code exists and has exact-head synthetic CI evidence.
2. A draft implementation exists but its exact-head qualification is incomplete.
3. A research owner has a design/issue but has not published a consumable schema/profile.
4. A real cross-owner adapter has been reviewed, version-pinned and replayed.

**A CI pass for an individual owner crate is not an integration pass.** Likewise, a GitHub issue URL, source commit, file digest, valid signature, or similarly named JSON field does not establish that a consumer has the right semantic binding.

At the time of this register, the Highveld programme has a validated synthetic corpus and independent evaluator, plus several independently implemented owner components. It does **not** yet have a production-bound Mycelix–Symtropy–Symthaea–Sol-Atlas adapter chain.

## 2. Status vocabulary

| Status | Meaning |
|---|---|
| `CODE_PRESENT_CI_PASS_SYNTHETIC` | The named owner implementation passed the listed exact-head software checks for synthetic fixtures only |
| `CODE_PRESENT_CI_INCOMPLETE` | Implementation exists, but exact-head checks are cancelled, pending, skipped or incomplete |
| `DRAFT_PROFILE_NOT_BOUND` | A draft schema/interface exists but its consumer mapping is not owner-approved or not version-pinned |
| `RESEARCH_ONLY_UNBOUND` | An issue/design exists; there is no verified consumable implementation/profile in this replay |
| `INTEGRATION_NOT_RUN` | No same-input cross-repository replay has been demonstrated |
| `PHYSICAL_NOT_EVIDENCED` | No competent physical measurement/acceptance evidence is included |

These statuses may coexist. Never promote an item simply because a downstream PR references it.

## 3. Owner readiness matrix

| Owner responsibility | Source artifact / exact reference | Exact-head evidence | Current binding status | What the evidence actually establishes |
|---|---|---|---|---|
| Sol-Atlas panel-thread corpus | [PR #870](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/870), branch `research/highveld-panel-thread-composition-v0-1`, composition commit `59f546c4e9d289d491f661ad71d903d815f39b3b` | Repository Check run [#43714320563](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/43714320563) passed | `CODE_PRESENT_CI_PASS_SYNTHETIC`; labels remain unadjudicated | 24 synthetic scenarios and a closed corpus schema; not formal JSON Schema meta-validation, independent gold labels, owner conformance or physical proof |
| Sol-Atlas qualifier, bridge and industry protocol | [PR #877](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/877), exact validated head `d175f4d8bf9828c92f77b6847141a98573139120` | Focused qualifier run [#43766996384](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/43766996384) and repository Check [#43766996387](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/43766996387) passed. Focused: formatting, 49/49 library tests and strict Clippy. Repository: 7/7 application tests, 49/49 qualifier library tests, strict Clippy, formatting and WASM check | `CODE_PRESENT_CI_PASS_SYNTHETIC`; Mycelix/Symtropy production profile mappings remain UNBOUND | Deterministic evaluation of the public provisional corpus plus a synthetic assurance-to-campaign envelope. The bridge tests route admissibility, context/profile binding, temporal cutoff, bilateral review, candidate path retention and non-closure of proof obligations. No real owner adapter, authenticated owner output, source-byte/signature verification, adjudication, vendor conformance or physical acceptance |
| Symtropy bounded exact observation campaign | [PR #5240](https://github.com/Luminous-Dynamics/symtropy/pull/5240), exact implementation head `706a207d08769a45a92037659ebe50caff651678` | Workflow [#43699505549](https://github.com/Luminous-Dynamics/symtropy/actions/runs/43699505549) passed on Ubuntu, Windows and macOS; 15/15 tests and strict Clippy on all three; Ubuntu exact-vs-greedy scale benchmark passed | `CODE_PRESENT_CI_PASS_SYNTHETIC`; campaign input/output profile binding still requires an owner-approved cross-repo projection | Bounded exact set-cover under resource/space constraints, proof-of-search status and synthetic latency; not actual sensor coverage or observed evidence |
| Symtropy material-readiness adapter | [PR #5323](https://github.com/Luminous-Dynamics/symtropy/pull/5323), exact implementation head `d58ed77aa58a429be523c1fbbfd4532f792213ea` | Workflow [#43699151577](https://github.com/Luminous-Dynamics/symtropy/actions/runs/43699151577) passed on Ubuntu, Windows and macOS; 16/16 tests and strict Clippy on all three | `CODE_PRESENT_CI_PASS_SYNTHETIC`; source payloads and real inventory remain external | Synthetic exact-lot/custody/quality/reservation readiness semantics; not real material certification or warehouse truth |
| Symtropy option + evidence-campaign co-design | [PR #5260](https://github.com/Luminous-Dynamics/symtropy/pull/5260), stated head `4de9edc401081a795fb94009d2b11d6e5aa0e483` | Recorded workflow [#43636707270](https://github.com/Luminous-Dynamics/symtropy/actions/runs/43636707270) was **cancelled**; PR describes 14 synthetic tests and a Linux benchmark but those claims are not a current exact-head pass | `CODE_PRESENT_CI_INCOMPLETE`; do not make this the verified joint-composition owner yet | Draft implementation composes #5238 option selection with #5240 campaign planning, preserves multiple objectives and explicit campaign-cost treatment; no verified current-head result from the evidence checked here |
| Mycelix provenance/custody trace | [PR #5708](https://github.com/Luminous-Dynamics/mycelix/pull/5708) and [parent #5149](https://github.com/Luminous-Dynamics/mycelix/issues/5149) | Draft PR describes a 28-test Rust verifier; this register has not established its exact PR head and successful exact-head run | `DRAFT_PROFILE_NOT_BOUND` | The described verifier checks supplied reference metadata and subject links; it explicitly does not fetch/authenticate source bytes, verify signatures or determine legal/professional authority |
| Mycelix typed decision episode / assurance graph | [Issue #6247](https://github.com/Luminous-Dynamics/mycelix/issues/6247) | Open research design; no verified consumable profile or exact-head code result established here | `RESEARCH_ONLY_UNBOUND` | Intended owner for canonical episode scope, typed claims, proof obligations, exact artifact digests and independently verified terminal disposition |
| Mycelix assurance-obligation acquisition / decision-conditioned VoI | [Issue #6252](https://github.com/Luminous-Dynamics/mycelix/issues/6252) | Open research design; no verified consumable production output profile established here | `RESEARCH_ONLY_UNBOUND` | Intended owner for candidate acquisition tasks, separate hard-gate dispositions, admissibility, prerequisites and evidence requests; plan coverage is not accepted evidence |
| Symthaea engineering change impact / evidence applicability | [Issue #10543](https://github.com/Luminous-Dynamics/symthaea/issues/10543) | Open benchmark/research work; exact owner profile output not established here | `RESEARCH_ONLY_UNBOUND` | Intended owner for change-to-claim impact and selective requalification; Highveld must not recreate the dependency engine |
| Symthaea decision-sufficient evidence request | [Issue #12046](https://github.com/Luminous-Dynamics/symthaea/issues/12046) | Open research work; exact owner profile output not established here | `RESEARCH_ONLY_UNBOUND` | Intended owner for the minimum competent evidence request and correct abstention; not an autonomous external requester |
| Symthaea benign workcell integration | [Issue #6040](https://github.com/Luminous-Dynamics/symthaea/issues/6040) | Open integration demonstrator; no verified physical or source-bound replay output included here | `RESEARCH_ONLY_UNBOUND`; `PHYSICAL_NOT_EVIDENCED` | Intended low-risk end-to-end workcell thread; not production capability or autonomous construction |

### Important owner-resolution rule

Symtropy #5240 and #5260 are different responsibilities: #5240 owns the bounded observation-campaign optimizer; #5260 owns the option-specific campaign package and joint Pareto result. Highveld consumes the latter only after its own exact-head qualification is current. Do not alias a related issue number to a PR merely because the topics overlap.

Likewise, Mycelix #6247 owns the decision episode / typed assurance graph; #6252 owns assurance-obligation acquisition. Symthaea #10543 and #12046 remain distinct engineering-change and evidence-request responsibilities. The cross-owner layer should carry opaque references, not clone the owner's graph or planner.

## 4. Useful measured owner baseline: exact vs greedy observation campaigns

Symtropy #5240 reports the following synthetic Linux-runner medians from its exact-vs-greedy benchmark (milliseconds):

| Candidate actions | Exact p50 | Exact p95 | Greedy p50 | Greedy p95 |
|---:|---:|---:|---:|---:|
| 8 | 1.577 | 1.744 | 0.685 | 0.731 |
| 12 | 9.309 | 9.466 | 1.790 | 1.800 |
| 16 | 50.115 | 50.485 | 3.592 | 3.717 |
| 20 | 251.941 | 253.847 | 6.380 | 6.481 |

This is a useful baseline for the **campaign-search** part of the programme. In that benchmark workload, exact and greedy found the same crew-minute score at each size. A separate frozen counterexample demonstrates that greedy is not generally optimal. These measurements are not production SLOs, field productivity, actual survey coverage or an independent cross-repository benchmark.

The Highveld differentiator should not be to beat Symtropy's optimizer at its own task. It should test whether upstream proof obligations, authority/rights gates, identity, freshness and change-impact are correctly bound to that optimizer without changing its objective or falsely closing an obligation.

## 5. Research landscape and fair SOTA positioning

### NIST SI for Manufacturing / TIDAL

NIST's current Super Intelligence for Manufacturing initiative is organized around Teamed, Integrated, DT-assured and Layered Intelligence (TIDAL). It explicitly plans an integration test bench comparing industrial information architectures such as UNS/MQTT plus knowledge graphs/type models, I3X and OPC UA profiles, initially using the Tennessee Eastman Process and later extending to NIST physical digital-twin/additive-manufacturing equipment. Its planned evaluation includes contextualization, accuracy, latency, cost, grounding, human-machine teaming and digital-twin VVUQ.

This is a direct methodological comparator. We should align with and learn from this test-bench approach—not claim that Highveld already beats it. Our distinct challenge is cross-owner construction/industrialized-asset decisions with exact configuration, evidence applicability, acquisition admissibility and physical acceptance boundaries.

Reference: https://www.nist.gov/programs-projects/super-intelligence-si-manufacturing

### Decision-focused active sensing

The 30 October 2026 IEEE Transactions on Systems, Man, and Cybernetics: Systems article *Value-Based Active Sensing for Improving Sampling Efficiency* frames sensing as observe-then-plan, comparing expected opportunity loss with observation cost and adding a risk premium and model-ignorance penalty. It reports temperature-alert and CO₂-aware office use cases under a fixed one-step downstream decision.

This is a current SOTA comparator for decision-value-aware sensing. Highveld must not claim to outperform it without reproducing its baseline and objective. Our extension hypothesis is broader, not yet proven: multiple linked proof obligations; non-compensatory rights/authority/safety admissibility; owner-preserving provenance; resource-constrained campaigns; independent evidence qualification; and lifecycle/accepted-output outcomes.

Reference: https://doi.org/10.1109/TSMC.2026.3745855

### Industrialized construction adoption

Autodesk's 2026 Spotlight survey (869 industry respondents plus supplemental Dodge research) reports regular/standard adoption of productization, DfMA and prefabrication at 37–40%, and traditional procurement as the leading barrier to broader productization/DfMA adoption (51%). This is vendor-sponsored survey context, not an independent global market census or a technical-performance benchmark.

The implication for Highveld is to measure repeatability across projects, supplier hand-offs, procurement, change, accepted output and lifecycle evidence—not just one successful prefabrication/robotics demonstration.

Reference: https://www.autodesk.com/design-make/articles/2026-state-of-design-make-spotlight-on-industrialized-construction

### Construction robotics maturity

A June 2026 systematic review of 375 construction-robotics studies reports operator-led workflows remain prevalent, with research concentrated in structural construction and planning, maintenance and demolition relatively underexplored. It reports about 19.3% of the reviewed publications as field deployment or on-site trials. That is a literature classification, not an industry-wide deployment rate.

The first Highveld physical article remains a guarded non-load-bearing panel cell. Dynamic sites, mobile robots, public access, occupied buildings and work-at-height are separate ODDs that cannot inherit that cell's qualification.

Reference: https://doi.org/10.1016/j.rineng.2026.110406

## 6. What we can claim now—and cannot

**Supported now:**
- Sol-Atlas has a 24-case synthetic composition corpus and a deterministic non-authorizing evaluator with exact-head CI.
- Symtropy has independently validated synthetic building blocks for exact observation campaigns and material readiness.
- A draft option/evidence-campaign composition implementation exists in Symtropy.
- Mycelix and Symthaea have identified research owners for provenance, decision-assurance, acquisition, engineering change and evidence requests.

**Not supported yet:**
- End-to-end Mycelix–Symtropy–Symthaea–Sol-Atlas adapter integration.
- Current-head CI success for Symtropy #5260.
- Owner-approved semantic profile mappings for #6247/#6252/#10543/#12046.
- Vendor interoperability superiority over Autodesk/Bentley/Siemens/PTC/Dassault.
- Physical material, process, robot, geometry, building or production qualification.
- A blind SOTA result or an independently adjudicated gold corpus.

## 7. Highest-value next work

1. **Close the Symtropy #5260 exact-head gate.** Do not treat the cancelled run as a pass. Once current exact-head formatting/tests/Clippy/benchmark succeed, pin its output package and cost-boundary semantics.
2. **Consume Mycelix #6247 as the episode/obligation source.** Bind the exact episode scope and obligation IDs; do not create a Highveld assurance graph.
3. **Consume Mycelix #6252 as the acquisition/admissibility source.** Preserve authority, rights, safety, schedule, physical-realizability and data-access dispositions independently. Unknown/Blocked never become selectable.
4. **Consume Symthaea #10543 and #12046 outputs when published.** Use them for change impact and minimum competent evidence requests; keep both UNBOUND until versioned source profiles exist.
5. **Build a read-only replay of the exact same decision episode.** Compare owner-native results against an independent small-case oracle. Preserve source digests, scope/time and owner result statuses. No new planner, scheduler, ledger or Pareto layer.
6. **Adjudicate the public corpus and then create an access-controlled hidden holdout.** The public redacted packet is not hidden.
7. **Only after synthetic replay:** proceed to the guarded, non-occupied panel cell with independent metrology, applicable risk assessment and relevant professional acceptance.

## 8. Qualification and reporting rule

Every future replay receipt must identify, at minimum, the repository, exact commit, source artifact reference and digest, owner schema/profile ID and version, adapter/mapping version and digest, decision scope/as-of cutoff, rights/disclosure scope, and the exact evaluator/corpus/submission digests.

If the owner profile or mapping is unavailable, record **UNBOUND** and do not score the corresponding obligation as covered. If the exact source bytes cannot be retrieved or verified, say so. A successful CI run validates the code it tested; it does not validate an unbound cross-owner mapping.

## 9. New Design-for-Evidence benchmark workstream

[HIGHVELD-DESIGN-FOR-EVIDENCE-001 #885](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/885) is now open under the Highveld panel programme. It is a **research/specification workstream**, not an implemented optimizer or physical inspection capability.

Its ownership map is intentionally compositional:
- Symtropy #4165 supplies the DfI / inspectability-method precedent (aerospace-specific until a construction mapping is approved).
- Symtropy #4043 supplies the as-built inspection / repair / reverification lifecycle owner.
- Symtropy #5240 / #5250/#5260 own observation campaign and option/evidence composition.
- Symtropy #3773 owns accepted-output and manual/assisted/robotic economics.
- Sol-Atlas #149 / #814 supply reversible assembly and DfMA delivery concepts.
- Symthaea #10543 / #12046 own engineering change applicability and decision-sufficient evidence requests.
- Mycelix #5149 supplies provenance, material custody and scoped source/approval references.

No second design optimizer, NDE engine, evidence scheduler, repair engine or authority graph is proposed. New adapters remain UNBOUND until the actual owner schema/profile and exact commit are published and reviewed.

Research references used in the issue:
- NASA DFI / Inspectability Metric (NTRS 20240003953 / 20240007593): a published aerospace methodology precedent for treating inspectability as a design variable; not reproduced or claimed equivalent.
- ISO 20887:2020, confirmed current in 2025, for building design-for-disassembly/adaptability principles; it does not establish universal numeric performance levels.
- NIST Smart Manufacturing Systems Test Bed for a later physical matched test combining CAx, CNC, CMM and MTConnect.
- NIST 2026 SI-for-Manufacturing TIDAL programme for industrial architecture ablation and digital-twin assurance methodology.

## 10. Validated synthetic assurance-to-campaign bridge increment

**Exact validated head:** `d175f4d8bf9828c92f77b6847141a98573139120`  
**Focused workflow:** [#43766996384](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/43766996384) — passed formatting, 49/49 Rust library tests and strict Clippy.  
**Repository workflow:** [#43766996387](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/43766996387) — passed formatting, 7/7 application tests, 49/49 qualifier tests, strict Clippy and WASM check.

The qualifier library contains 16 original corpus/evaluator tests and 33 bridge tests. The bridge test families cover:
- remote source review versus physical observation;
- exact Mycelix episode/acquisition snapshot and Symtropy campaign snapshot consistency;
- exact project/design/site epoch/frame/clock/currency context;
- separate Mycelix and Symtropy mapping review references;
- pre-decision availability/review cutoffs;
- non-compensatory authority, rights, safety, schedule, data-access and source-competence gates;
- incomplete, infeasible and search-incomplete campaign outcomes;
- deterministic path selection, rejection visibility and input-order invariance;
- action ID namespace scoped to campaign package;
- a runnable positive synthetic example and a negative example keeping the real owner profiles UNBOUND.

**Semantic invariant:** a candidate acquisition route never reduces the unresolved-proof-obligation denominator. Planned work remains `PLANNED_NOT_OBSERVED`. Symtropy campaign search/optimality is recorded as owner-reported; the bridge sets `owner_outputs_authenticated=false`. The report also hard-codes source-byte verification=false, signature verification=false, physical observations=0, qualified obligations=0 and authority effects=NONE.

**Important:** both positive owner bindings and bilateral approvals in the runnable example are explicitly synthetic placeholders. They test the normalized bridge envelope only. The negative example represents the current real-owner state with Mycelix episode/acquisition and Symtropy campaign projections UNBOUND. Neither example is a real owner adapter replay.

The input and report JSON files were parsed and their key fields cross-checked against the Rust implementation. This is structural validation, not formal JSON Schema meta-validation. No independent label adjudication, real source authentication, actual cross-repository owner output, physical measurement, safety qualification or production test has been performed.

