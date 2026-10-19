# Cross-Trade Field Workbench: UX, Evidence, and Benchmarks

**Status:** research-informed design contract and synthetic Leptos prototype  
**Route:** `/trade-work`  
**Evidence state:** prototype only; not yet compiled, rendered, accessibility-audited, or tested with tradespeople  
**Scope:** electricians, plumbers, mechanics, HVAC/refrigeration technicians, construction and adjacent field trades

## Product thesis

Do not build a small office dashboard. Build a field workbench that makes the next safe and useful step obvious, captures evidence once, preserves uncertainty, works through connectivity loss, and produces a trustworthy handover.

The experience should optimize completed work and durable professional knowledge—not screen density, AI activity, clicks alone, or the number of records created.

The route in this branch is deliberately synthetic. Its local toggles do not inspect a site, verify energy isolation, prove a worker's qualification, connect an asset, call a service, or authorize physical work. The prototype demonstrates interface and reducer boundaries only.

## Paradigm shifts

1. **Dashboard → work sequence.** Prioritize one next action with its preconditions, not a wall of KPIs. Keep deeper asset, inventory, drawing, and history views one step away.
2. **Forms → evidence timeline.** Capture an observation once and reuse it for diagnosis, estimate, test record, customer report, and asset history. Preserve what was reported, measured, modelled, independently verified, and accepted as different evidence states.
3. **Cloud session → local-first work.** The technician can read cached context and safely draft records without signal. Show local persistence, pending submission, authoritative receipt, and ambiguous outcome as different states.
4. **Generic AI → source-grounded assistance.** Recommendations cite the exact manual or procedure revision, asset identity, assumptions, units, and evidence that support them. AI can suggest; it cannot certify physical safety or silently approve a consequential action.
5. **Generic screens → trade packs.** Share the shell, job lifecycle, evidence model, offline queue, and accessibility primitives. Make task language, units, instruments, procedures, mandatory evidence, and tests trade-specific.
6. **Single job → living asset history.** Verified work contributes to a versioned service history so the next technician inherits evidence and known uncertainty instead of re-entering everything.
7. **Status label → verifiable state transition.** A click, local draft, transmitted request, server receipt, customer acceptance, and verified outcome are not interchangeable.

## Shared field-job envelope

Production work should use a typed, versioned contract rather than infer meaning from display text. An initial UI-facing projection and viewport-intent model is now implemented in `sol-atlas-core/src/field_work.rs`, with frozen fixtures under `sol-atlas-core/fixtures/`. It is not yet a canonical field-service data model, a production authorization contract, or a cross-project integration. The owning service and extension boundaries still require review.

Minimum concepts:

- **Identity:** stable `job_id`, immutable event ID, `job_revision`, actor/workgroup reference, and explicit asset reference.
- **Scope:** trade profile, job type, site reference, jurisdiction, locale/units, customer-approved scope, and the applicable procedure or policy revision.
- **Execution:** independent workflow phase, current owner, required preconditions, allowed next actions, due/window details, and reason for any blocked transition.
- **Evidence:** evidence ID, source kind (user report, measured, imported, modelled, or independently verified), actor, timestamp, units where applicable, instrument/calibration metadata where applicable, asset reference, procedure revision, and provenance link/digest.
- **Transport:** local journal state, submission state, authoritative response state, retry/idempotency key, conflict state, and last successful synchronization receipt.
- **Outcome:** technician-reported completion, verification result, customer/owner acceptance, rework or dispute, and handover remain distinct.

Do not combine transport and work lifecycle into one status enum. A job can be locally saved while the network result is unknown; it can be remotely received while the physical work remains incomplete.

### Suggested lifecycle

`Assigned → Assessing → FindingRecorded → PlanApproved → WorkReportedInProgress → CompletionReported → VerificationRecorded → Handover`

Side paths include `Blocked`, `NeedsInformation`, `ReworkRequired`, `Disputed`, and `Cancelled`, with explicit reasons and allowed transitions. The screen must never infer that a reported completion is verified, that an accepted quote is a booking, or that a transmitted request succeeded when the response is ambiguous.

## Safety and authority boundaries

- Missing, stale, mismatched, or unknown prerequisites are **not ready**. The state must explain what is missing and how an authorized person can resolve it.
- Safety rules must be versioned by jurisdiction, asset, task, procedure, and role/qualification. Do not embed one country's rules as globally valid defaults.
- The UI must identify the source and freshness of safety-critical procedure content. If the applicable source cannot be resolved, do not present a green or approved state.
- A checklist click is a human claim, not a sensor reading or independent verification. A model prediction is not physical proof. A photo is not a substitute for a required instrument test.
- Safety-critical controls need negative controls: absent qualification, incomplete isolation, stale procedure, conflicting asset identity, missing required test, unauthorized actor, and wrong trade pack must not pass.
- Prototype switches intentionally simulate prerequisites. They must be visibly called **simulation** and must never be reused as production proof fields.

These are software guardrails, not a declaration of legal compliance or physical safety.

## Offline and recovery contract

Use a durable local journal and explicit transport states. The expected production protocol should define write-ahead preparation, durable commit/read-back before sending, immutable operation identity, typed authoritative results, and reconciliation for ambiguous outcomes. Do not silently retry a request that might already have caused an external side effect.

At minimum, test these situations:

1. The device loses connectivity before save, after local save, during upload, and after the server applies a request but before the response arrives.
2. The app is killed or the device restarts between every journal transition.
3. A retry arrives twice, an operation ID is reused with a different payload, or the server returns malformed/unknown response data.
4. A second technician edits the same job or a newer work-order revision supersedes a cached plan.
5. An asset, procedure, credential, or trade profile is missing, stale, revoked, or mismatched.
6. Local evidence survives app restart and remains clearly pending until reconciliation is authoritative.

Keep these user-facing distinctions visible: **Saved on this device**, **Ready to submit**, **Submission outcome unknown**, **Received by service**, **Rejected**, and **Conflict needs review**. Never show “sent”, “booked”, “verified”, or “complete” as a convenient synonym for a pending state.

## Cross-project responsibilities

| Project | Responsibility in the end-to-end flow | Explicit non-responsibility |
|---|---|---|
| **Mycelix** | Identity/authorization, work assignment and lifecycle records, evidence/approval references, durable receipts, auditability, and controlled collaboration. | A UI state change alone must not become an authoritative work receipt. |
| **Symthaea** | Evidence-linked diagnostic hypotheses, calculations, manual/procedure retrieval, uncertainty, and next-test suggestions. | A generated answer must not bypass authorization, fabricate a measurement, or assert a verified repair without evidence. |
| **Symtropy** | Spatial/mechanical context, component relationships, site geometry, system models, and scenario comparison. | A simulated model state is not a measured on-site condition. |
| **Sol-Atlas** | Geographic and infrastructure context, mapped assets/corridors, jurisdictional lookup, and the field-facing workbench shell/prototype. | Geographic proximity or a map marker alone does not establish access, travel time, ownership, or safe work authority. |

This is a contract between layers, not a claim that the integrations currently exist. Mycelix already has work-order revision-binding work in [PR #4730](https://github.com/Luminous-Dynamics/mycelix/pull/4730); reuse and reconcile with that contract instead of creating a competing notion of work-order identity.

## Additional paradigm shifts to test

### Offline sync → recoverable field journal

Offline-first should mean more than a cached screen plus background sync. Microsoft documents offline sync status and record-level conflicts in its Field Service mobile workflow; depending on configuration, a conflict can favor one whole-record version over the other. This is a useful baseline comparator, not a claim that every Microsoft product/version behaves identically. See [offline sync and conflict handling](https://learn.microsoft.com/en-us/dynamics365/field-service/mobile/offline-data-sync).

Our target should be stronger in the following measurable ways:

- Persist an immutable local operation identity before any external side effect.
- Distinguish local save, submit-ready, unknown outcome, authoritative receipt, rejection, and conflict.
- Preserve both conflicting revisions and show a field-aware explanation of what differs.
- Suggest a safe merge only when fields are independent and merge policy is explicit; safety-relevant measurements, permissions, procedure revisions, prices, commitments, and asset identity require deliberate resolution rather than silent last-write-wins.
- Retain a durable correction history. Correcting a record should append a correction, not erase the prior claim.

**Gate:** zero lost acknowledged writes, zero duplicate logical effects, zero silent overwrites, and zero promotion of an ambiguous response to success in the frozen fault corpus. These are software conformance claims, not proof that a field service never loses data in production.

### Status checklist → task graph

A linear workflow is a useful first demonstration but does not model real multi-trade jobs well. A repair can require parallel procurement, access approval, specialist inspection, customer sign-off, and rework branches. The next domain iteration should model a directed task graph with explicit prerequisites, evidence requirements, owners, revisions, and retry/rework edges; derive the visible "next action" from that graph rather than hard-code one universal sequence.

**Gate:** every offered action has a valid prerequisite path; no invalid edge is accepted; rework preserves the initial failure and subsequent correction; independent tasks can proceed without mutating unrelated state. Measure time-to-resume after interruption and task completion time against a linear baseline.

### AI answer → next-best diagnostic action

Reuse Symthaea's existing finite-hypothesis/decision-value work for trade-specific diagnosis rather than building a second recommendation scorer in the UI. The helpful outcome is not more AI messages; it is a well-supported next measurement, test, or escalation that changes a decision.

**Gate:** evaluate fixed synthetic fault traces with the same evidence and costs across baseline and candidate; report diagnostic decision accuracy, unnecessary-test count, abstention quality, total diagnostic cost, and all safety-gate bypasses. Require zero accepted safety-gate bypasses in the defined negative-control suite. Never claim real incident reduction from synthetic traces.

### One input method → multimodal, evidence-preserving capture

Support touch, physical keyboard, barcode/QR scan, photo, speech-to-text, and compatible instrument input as optional ways to capture the same typed information. They should create the same provenance-aware data model; none should silently turn a low-confidence transcription or visual estimate into a verified measurement. Always offer review/correction and an accessible manual route.

**Gate:** compare completion time and transcription/correction error rates by input method, trade, environment/noise, and experience level. Require 100% traceability for accepted evidence and preserve the original source alongside normalized text/values.

### Accessibility baseline → field ergonomics

WCAG 2.2 AA is a baseline. For frequent controls in this field UI, target the WCAG 2.2 enhanced 44 × 44 CSS pixel size where practical ([W3C target-size guidance](https://www.w3.org/WAI/WCAG22/Understanding/target-size-enhanced)). Also test glove-like accuracy constraints, bright light, low brightness, one-handed use, noisy environments, zoomed text, keyboard-only operation, and screen readers. Do not advertise "glove mode" or voice accessibility without task testing.

**Gate:** no critical action depends on color alone or a hover-only interaction; all primary workflows remain available without the spatial viewport; participants can correctly identify job identity, freshness, transport status, and safety-gate meaning at small-screen and zoomed-text configurations.

## Trade-specific first scenarios

| Profile | Synthetic scenario | Evidence that the UI should prompt for | Important negative case |
|---|---|---|---|
| Electrical | Intermittent protective-device trip | Circuit/asset reference, qualified actor, applicable procedure revision, instrument and test conditions, result and units | Work-start action remains blocked when required authorization/isolation evidence is absent or stale |
| Plumbing | Low pressure at a fixture | Measurement point, pressure/flow units, fixture/system conditions, instrument identity, asset context | Unknown pressure is never rendered as zero; symptom is not automatically promoted to a confirmed leak |
| Mechanics | Intermittent no-start | Diagnostic-code source, symptoms, test conditions, battery/part identity where applicable, verified repair procedure | A cleared fault code is not proof that the fault is fixed |
| HVAC/refrigeration | Output outside expected range | Operating mode, ambient conditions, instrument/units, equipment state, applicable service procedure | A modelled estimate is not displayed as a measured pressure or temperature |
| Construction/carpentry | Damaged handrail inspection | Site/location, dimensions, material/component identity, images, inspection criteria, required authorization | A photo or visual estimate is not promoted to a passed structural check without the required method |

These are workflow test examples, not trade procedures. Real procedural content and thresholds must be reviewed against the applicable manufacturer information, standards, and local requirements by appropriately qualified people.

## Benchmark protocol

Record a baseline before claiming improvement. Freeze scenario IDs, scripts, test inputs, device/network profile, app revision, and scoring rubric. Report the numerator/denominator and raw per-scenario results alongside aggregates. Separate synthetic conformance results from usability results and field outcomes.

### Level 1 — interaction and task performance

| ID | Measure | Initial target | Method |
|---|---|---|---|
| UX-01 | Core-task success | ≥95% completion without moderator intervention after brief orientation, once the design has passed formative iteration | Use the same task scripts against the current baseline and prototype; report results by trade and experience |
| UX-02 | Time to complete common flow | ≥30% lower median elapsed time than baseline, with no regression in required evidence or correctness | Timed assessment → finding → plan → verification → handover scenarios |
| UX-03 | Duplicate entry | Zero re-entry of an unchanged field already captured in the same job/evidence chain | Instrumented task scenarios and state diff |
| UX-04 | Primary touch targets | Aim for ≥44 × 44 CSS px for frequent touch targets; meet WCAG 2.2 AA requirements and document any exceptions | Automated geometry check plus manual mobile review |
| UX-05 | Comprehension of states | ≥95% correct interpretation of the difference between local save, submitted, unknown outcome, received, verified, and accepted | Ask participants to explain the state and next safe action without coaching |
| UX-06 | Recovery from interruption | ≥95% task recovery without data loss or silent state change | Scripted interruptions mid-task; check state and evidence identity afterwards |

### Level 2 — deterministic software and offline correctness

| ID | Measure | Required gate | Method |
|---|---|---|---|
| REL-01 | Acknowledged local writes lost after restart/reconnect | **0** in the frozen fault-injection corpus | Kill/restart and connectivity changes at every journal boundary |
| REL-02 | Duplicate side effects from duplicate delivery | **0** for the idempotency corpus | Re-submit identical operation IDs and verify one logical effect |
| REL-03 | False success under ambiguous response | **0** promotions to received/accepted/verified without authoritative evidence | Simulate applied-but-response-lost, timeout, malformed response, and unknown future status |
| REL-04 | Silent conflict overwrite | **0**; every unresolved conflict remains visible and recoverable | Concurrent revision fixture with contradictory edits |
| REL-05 | Stale or mismatched context accepted | **0** unflagged admissions in the frozen negative-control suite | Wrong asset ID, old work-order revision, stale procedure, revoked capability, wrong trade pack |
| REL-06 | Deterministic reducer conformance | 100% of enumerated state/action cases match the table | Exhaustive state/action matrix with digest-pinned expected result |

### Level 3 — domain safety, evidence and real outcomes

| ID | Measure | Initial target | Method |
|---|---|---|---|
| DOM-01 | Critical gate bypass | **0** accepted bypasses in the specified negative-control corpus | Test missing/stale/revoked/mismatched authorization, procedure, isolation and required test evidence |
| DOM-02 | Evidence provenance | 100% of submitted evidence has required source and time; applicable measurements retain units and instrument metadata | Schema validation plus semantic checks; missing data stays missing |
| DOM-03 | Verified claim lineage | 100% of verified outcomes reference the required evidence and procedure revision | Graph/reference integrity validation and deliberate broken-link fixtures |
| DOM-04 | Safety-status comprehension | No participant interprets a demo or reported state as an authoritative physical safety clearance in the comprehension test | Adversarial usability script that includes misleading status and stale-source cases |
| DOM-05 | Field usability coverage | Formative testing includes apprentices and experienced workers; initially test all five profile families before broad promotion | Moderated sessions, observations, error analysis, and worker feedback |
| DOM-06 | Outcome claims | Do not claim reduced incidents, rework, or cost from UI tests alone | Require a separately designed, consented, appropriately reviewed field evaluation with baseline and denominators |

### Device and network profile

The first performance baseline should use a documented lower/mid-range Android device, a desktop browser, a small-screen layout, constrained bandwidth, high latency, packet loss, and complete disconnect. For cached work-order context, aim for a p95 render-to-interactive time under 2 seconds on the named reference device. Publish the model, browser/runtime, workload size, cache state, and percentile method with the result; do not quote the threshold without that context.

### Pilot design

Start formative tests with electricians and plumbers because they exercise different evidence and isolation concepts. Include novice/apprentice and experienced users. Expand to mechanics, HVAC, and construction only after the shared sequence is understandable without teaching a user to trust the demo as a real safety authority. A small formative sample finds defects; it does not establish population-wide usability or prove safety.

For every run, archive the exact source SHA, fixture digest, device and network profile, run command, raw results, failures, and verifier version. A queued CI run is not a pass; no prototype benchmark result is a real-world impact claim.

## Implementation backlog

**P0 — close the UX and safety seams**

- [x] Add a separate cross-trade route with explicitly synthetic data and trade-specific prompt copy.
- [x] Add a pure workflow reducer with blocked start, distinct completion/verification/handover phases, and tests for invalid shortcuts.
- [ ] Run Rust format/native/WASM checks on the exact candidate commit.
- [ ] Browser-review keyboard focus, mobile overflow, target geometry, color contrast, reduced motion, and screen-reader announcements.
- [ ] Review the safety copy with qualified workers; verify that no demo control can be mistaken for an actual site check.
- [ ] Create a versioned, machine-readable workflow/negative-control corpus and executable offline journal harness.

**P1 — shared integration contract**

- [ ] Agree on the typed work envelope and separate work phase from transport state.
- [ ] Bind evidence to exact work-order and procedure revisions.
- [ ] Add an authoritative reconciliation flow for ambiguous outcomes and concurrent edits.
- [ ] Connect Mycelix work records, Symthaea evidence-linked assistance, Symtropy models, and Sol-Atlas site context behind explicit adapters.
- [ ] Add localized units, privacy-preserving photo handling, consent, data export, and retention rules.

**P2 — operational validation**

- [ ] Run formative tests across all five trade families, then preregister larger usability and field-effectiveness evaluations.
- [ ] Compare task time, first-time correctness, rework, missing evidence, user-reported cognitive burden, and recovery after outages.
- [ ] Promote features only against published positive and negative controls; keep unsupported claims at prototype level.

## Research references

- [Microsoft Learn: Work offline and update offline data](https://learn.microsoft.com/en-us/dynamics365/field-service/mobile/work-offline) — offline data, synchronization and conflict-handling concerns.
- [Microsoft Learn: Configure offline data synchronization](https://learn.microsoft.com/en-us/dynamics365/field-service/mobile/offline-data-sync) — offline-first behavior and sync-status/conflict operations.
- [W3C: Web Content Accessibility Guidelines (WCAG) 2.2](https://www.w3.org/TR/WCAG22/) — accessibility baseline and target-size criteria.
- [OSHA: Qualified employee requirements for electrical equipment maintenance](https://www.osha.gov/laws-regs/standardinterpretations/1993-09-09-3) — US-specific illustration of why qualification and test evidence are separate from a UI status; not a global rule set.
