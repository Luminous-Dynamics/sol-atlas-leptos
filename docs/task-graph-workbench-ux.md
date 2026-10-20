# Trade workbench: task graph and correction-impact UX

**Status:** synthetic Leptos prototype; no service, canonical record, or physical control integration  
**Route:** `/trade-work`  
**Component:** `src/components/trade_workbench.rs`

## User-facing change

The workbench now includes an ordinary ordered task list driven by the compiled Rust task-graph frontier. The example starts with one synthetic prerequisite recorded as verified in the demo, so site inspection and parts availability are independently ready. The functional test remains blocked until both branches have recorded verification; handover depends on the functional test.

Each row shows task title, prerequisite titles, task status, and the one action valid for the current state. Blocked tasks display a disabled action instead of silently accepting an invalid transition. Task changes are local and clear any prior correction-impact preview.

A second panel can preview the downstream impact of challenging the synthetic access verification. It shows downstream task IDs, their prior status, impact classification, and dependency path. The plan is read-only, bound to the observed graph revision, and not a rework command.

## Architecture boundary

- **Leptos** renders task semantics, keyboard-operable actions, blocked explanations, and the correction impact list.
- **CompiledTaskGraph** maintains deterministic ready-frontier state for this local demonstration.
- **Bevy** may later visualize spatial relationships, but it is not required to see, understand, or operate the task list.
- The client does not authorize work, verify corrective evidence, persist an event, mutate authoritative task records, or establish that any physical prerequisite is satisfied.

Everything on the page is synthetic. A “verified in demo” label means only that the local reducer has that enum state.

## Acceptance and benchmark targets

| ID | Measure | Initial target | Evidence |
|---|---|---|---|
| UI-DAG-01 | Ready action correctness | 100% agreement with core graph frontier | Component derives ready task IDs from `CompiledTaskGraph`; core parity tests remain the algorithm gate |
| UI-DAG-02 | Blocked-task explanation | ≥95% of participants correctly identify why the functional test is blocked | Moderated task study with no prior graph training |
| UI-DAG-03 | Correction-impact interpretation | ≥95% correctly distinguish “requires revalidation”, “remains blocked”, and “cancelled — unchanged” | Scenario test with downstream status differences |
| UI-DAG-04 | Keyboard reachability | 100% actions reachable and usable without pointer | Keyboard-only browser audit |
| UI-DAG-05 | Target size | Frequent task controls at least 44 CSS px high | Computed-style check at desktop and mobile breakpoints |
| UI-DAG-06 | Accessible list fallback | All information and actions remain available without canvas/WebGL/Bevy | Browser test with graphics surface disabled |
| UI-DAG-07 | Stale impact preview | 0 stale previews retained after graph mutation or trade switch | Component interaction tests |
| UX-01 | Task resumption | ≥30% lower median resumption time than a linear checklist, with correctness unchanged | Paired controlled tasks and recorded participant/task denominators |

These are proposed acceptance gates, not achieved usability measurements. The current code has not undergone real keyboard/screen-reader or tradesperson testing.

## Industry comparison

ServiceNow documents task dependencies including finish-to-start, start-after-start, start-together and lag constraints. This prototype does not yet implement those richer schedule constraints. Its current contribution is a deterministic client frontier and an explicit correction-impact explanation with revision context. Compare only after identical scenarios have been exercised in a configured commercial baseline; a five-task prototype is not evidence of superiority to an enterprise field-service product.

## Next steps

1. Validate labels, focus order, live status announcements and mobile layout in a browser.
2. Add component-level interaction tests for start/report/verify, parallel branches, blocked join, trade reset, and preview invalidation.
3. Connect task actions to an authoritative command service with expected-revision preconditions, operation IDs, idempotency, and role/evidence validation.
4. Only then consider spatial overlays in Bevy; the accessible list remains the primary operational surface.
