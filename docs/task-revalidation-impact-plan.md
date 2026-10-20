# Task revalidation impact plan v1

**Status:** immutable impact-analysis preview; not a rework command, durable event, or authorization decision  
**Module:** `sol-atlas-core/src/task_graph.rs`  
**Fixture:** `sol-atlas-core/fixtures/task-revalidation-plan-v1.json`

## The paradigm shift

A dependency graph can tell us which tasks are ready today, but that is not enough for real maintenance. A measurement may be corrected after a later task was reported complete or verified. If the system only displays the latest status, it can quietly leave downstream conclusions looking valid after their premise has changed.

The proposed correction flow starts with an explicit **impact preview**, not a destructive cascade:

1. A previously `verification_recorded` task is challenged with a reason code and a reference to corrective evidence.
2. The planner traverses all downstream dependencies and returns an impact list bound to the observed graph revision.
3. Previously active, completion-reported or verified downstream tasks are flagged `requires_revalidation`.
4. Pending downstream tasks are flagged `remains_blocked`; cancelled tasks remain cancelled.
5. The original graph is not mutated. A canonical service must authenticate the actor, verify the evidence and permissions, check the revision is still current, append the correction event and decide the actual task transitions.

This separation prevents a UI preview from pretending to be an authoritative record update. It also preserves the exact statuses that existed when the plan was produced, which is essential for review and audit.

## API and invariants

- Source task must currently be `verification_recorded`.
- Reason code and correction-evidence reference must be non-empty.
- Plan includes schema version, graph ID, expected graph revision, source task/status, reason, correction evidence reference, previous downstream statuses, impact classes and deterministic dependency paths.
- Both the straightforward graph model and the compiled graph index emit the same plan.
- Planner returns shortest paths; equal-length paths are chosen deterministically by sorted dependency IDs. For a joined dependency graph, a node is included once, using the first deterministic shortest path.
- Generating a plan is read-only and must not increment graph revision.
- A real correction is rejected if the canonical graph revision changed, unless the authoritative service deliberately recomputes impact against that newer revision.

The dependency path is an explanation of impact, not proof that any physical work was safe, authorized or completed. The correction reference is a pointer, not a validation of evidence authenticity.

## Frozen acceptance fixture

The fixture includes a verified access task, verified inspection and parts branches, a completion-reported functional test that joins both branches, a pending handover, and a cancelled branch with a pending dependent. Its expected plan identifies every reachable descendant exactly once, preserves pre-plan statuses, distinguishes revalidation from blocked and cancelled work, and gives each task a deterministic path.

## Benchmarks and acceptance gates

| ID | Measure | Gate / initial target | Evidence |
|---|---|---|---|
| REV-01 | Plan parity | 100% same plan from reference and compiled graph for frozen traces | Exact equality against golden JSON fixture |
| REV-02 | Destructive preview | Zero graph/state/revision mutations when planning impact | Before/after equality assertions |
| REV-03 | Missing provenance | Zero plans with blank source, reason code or correction reference | Negative input suite |
| REV-04 | Incorrect root state | Zero plans from a source not currently verification-recorded | Source-status negative controls |
| REV-05 | Complete downstream closure | 100% of reachable descendants classified exactly once | DAGs with branching, joins, cancellation and multiple paths |
| REV-06 | Deterministic explanation | 100% identical dependency paths for the same graph revision regardless of input vector order | Reordered-node property cases |
| PERF-REV-01 | Impact-planning cost | p95 under 10 ms for 10,000 tasks and 40,000 edges (initial hypothesis) | Release build, fixed graph generator, named runner, 1,000 samples and raw distribution |

The timing target is not measured yet. The benchmark must include both a narrow chain-like frontier and a wide independent frontier, and should report graph shape and impact-set size. Report p50/p95/max and raw samples; don't turn algorithm measurements into claimed field productivity or safety outcomes.

## Industry comparison

ServiceNow documents work-order dependency relations including finish-to-start, start-after-start, start-together and lag constraints, and supports multi-agent tasks plus offline work. Microsoft Dynamics 365 Field Service incident types can standardize groups of service tasks, products and services. Thus, dependency modeling and work-order templates are established capabilities, not our novelty.

The proposed differential is explicit, deterministic correction-impact analysis with prior-status preservation and a revision-bound preview, paired with a fail-closed offline journal. This PR does not claim that vendor products lack equivalent rework controls; a future head-to-head study must evaluate the relevant vendor configuration and same correction scenarios.

References:
- [ServiceNow — Create dependencies between work order tasks](https://www.servicenow.com/docs/r/field-service-management/work-order-management/t_SetAnUpstreamTask.html)
- [ServiceNow — Manage work order tasks](https://www.servicenow.com/docs/r/field-service-management/work-order-management/c_WorkOrderTasks.html)
- [Microsoft Learn — Work order incident type overview](https://learn.microsoft.com/en-us/dynamics365/field-service/incident-type-overview)

## Production work still required

- Agree on the canonical owner of graph revision and rework events.
- Bind tasks and correction events to immutable evidence IDs/digests, procedure revision and qualified reviewer identity.
- Add an append-only event record in the authoritative service; this preview is not an event log.
- Implement stale-plan rejection and re-computation when concurrent work changes the graph.
- Only then wire the preview to Leptos. Bevy may visualize impact paths, but the accessible work list must remain available without spatial rendering.
