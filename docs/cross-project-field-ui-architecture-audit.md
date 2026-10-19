# Cross-Project Field UI Architecture Audit

**Snapshot:** 2026-10-19  
**Purpose:** Align field-work UI and engineering visualization across Symthaea, Mycelix, Symtropy, and Sol-Atlas with the agreed Rust-first Leptos/Bevy direction.  
**Method:** Targeted source, manifest, repository-tree, and open-PR inspection through the public GitHub API. This is not a full code review or a local build.  
**Validation ceiling:** No Rust toolchain or local clone was available in the execution environment, so this audit does not claim compilation, browser rendering, or runtime integration.

## Decision

Use **Leptos as the default application and operator interface** and **Bevy as the spatial/3D viewport and simulation runtime**. Keep domain state in framework-independent Rust contracts and authoritative project services.

```text
Authoritative domain services / versioned canonical records
                  |
          typed Rust projections
            /             \
           v               v
   Leptos application    Bevy viewport
   shell and workflows   geometry, 2D/3D,
   forms, evidence,       simulation and
   keyboard/accessibility spatial interaction
            \             /
             typed user-intent events
                     |
           validated domain commands
```

Neither Leptos signals nor Bevy ECS resources should become canonical work-order, asset, evidence, approval, or safety truth. Rendering selections express user intent; domain owners decide whether a command is authorized and valid.

This matches the existing Mycelix architecture direction in [issue #2053 — Leptos shell with projection-driven Bevy topology viewport](https://github.com/Luminous-Dynamics/mycelix/issues/2053). Reuse that architectural pattern for field operations instead of designing a parallel integration scheme.

## Repository audit

### Sol-Atlas Leptos

**Observed:** `Cargo.toml` uses Leptos 0.8 CSR. The current web client has a custom WebGL2 globe renderer and renderer-specific code under `src/renderer/`; the app composes DOM controls through Leptos components. It does not currently depend on Bevy.

**Implication:** The new `/trade-work` route correctly uses Leptos for job navigation, work sequencing, status labels, evidence prompts, and mobile interaction. Do not turn trade forms, dense textual procedures, or accessibility-critical workflows into Bevy-only UI.

**Next seam:** Keep the existing globe as-is while defining a typed viewport contract. If a Bevy viewport is introduced, treat it as an independently tested rendering surface with a non-Bevy accessible fallback. Do not add Bevy to the browser app merely to satisfy a framework preference; first demonstrate the wasm target, GPU fallback, bundle-size, input, accessibility, and low-end-device budgets.

Relevant source: `Cargo.toml`, `src/app.rs`, `src/components/globe_canvas.rs`, `src/renderer/`, and the current trade-workbench branch.

### Symtropy

**Observed:** The production root manifest currently uses Bevy 0.19 and has separate crates for physics-only Bevy integration, the Symtropy Bevy plugin, and the physics-to-render bridge. The render bridge exposes projection, physics synchronization, materials, and telemetry buffers.

Open work already supplies useful foundations:
- [PR #2107 — purpose-built multidimensional view profiles](https://github.com/Luminous-Dynamics/symtropy/pull/2107) is a draft describing strategic, operational, tactical, scientific, and diagnostic view intent, plus 2D/2.5D/3D/4D projection contracts. It does not claim all camera controls are installed.
- [PR #2009 — frozen PBR render benchmark and frame-time receipts](https://github.com/Luminous-Dynamics/symtropy/pull/2009) defines reproducible render profiles and frame-time reporting. Reuse its fixture and measurement discipline rather than inventing a second renderer benchmark.
- [PR #2012 — isolated Bevy 0.20 graphics probe](https://github.com/Luminous-Dynamics/symtropy/pull/2012) reports a successful isolated probe on exact head `ad9362549eb4a15c5e5c353eed83045c9515081d`. That proves the probe compiles; it does not mean the production workspace has migrated.

**Implication:** Build new Symtropy integrations against the current 0.19 production boundary until the repository's migration gate, companion crates, and toolchain are aligned. Do not mix Bevy 0.19 and 0.20 types across a bridge, and do not upgrade the root workspace just because 0.20 has been released. A migration must include Lightyear, egui/inspector, task, custom-shader/material/buffer APIs, lockfile, CI and Rust toolchain checks.

**Next seam:** A Bevy plugin should receive immutable/versioned domain projections and emit typed selection/navigation intents. It should not write directly to work-order state or treat local ECS state as authoritative.

Relevant source: root `Cargo.toml`, `crates/core/symtropy-bevy-core/`, `crates/bridges/symtropy-bevy/`, and `crates/bridges/symtropy-render-bridge/`.

### Symthaea

**Observed:** The repository already has a lightweight Leptos 0.8 `symthaea-ui` client for the Symthaea service gateway. Its README explicitly keeps the heavy `symthaea` crate out of the WASM UI; the client calls the service's HTTP and live-telemetry surfaces instead. That is a good precedent for a narrow diagnostic-assistance boundary.

The existing `symthaea-bevy-dash` manifest uses Bevy 0.15, while Symtropy's production workspace uses 0.19. This is a real version boundary; it is not safe to assume that the dashboard can be embedded as a plugin in Symtropy without migration.

**Implication:** Expose diagnostic hypotheses, evidence references, confidence/uncertainty, and next-test suggestions through a small stable API/projection. Keep the Leptos interface thin. Reuse Symthaea's current diagnosis/decision-value kernels rather than implementing new UI-specific inference, and do not couple a browser build to the entire Symthaea workspace.

**Next seam:** Review whether `symthaea-bevy-dash` is a legacy standalone tool or needs its own version-migration plan. Do not silently pull the Bevy 0.15 dashboard into the Symtropy 0.19/0.20 type graph.

Relevant source: `crates/bridges/symthaea-ui/Cargo.toml`, `crates/bridges/symthaea-ui/src/api.rs`, `crates/bridges/symthaea-ui/README.md`, and `crates/bridges/symthaea-bevy-dash/Cargo.toml`.

### Mycelix

**Observed:** Mycelix has Holochain-backed work-order patterns, including manufacturing work orders with status-transition validation and references to BOM/routing revisions. Those records are manufacturing-oriented (product, quantity, due date, BOM/routing); do not force a repair/service visit into that model just because the word “work order” is shared.

The Mycelix field-work direction already includes:
- [Issue #2053](https://github.com/Luminous-Dynamics/mycelix/issues/2053): Leptos owns routes, inspectors, forms, evidence, timelines and accessibility; Bevy owns the topology viewport, camera and spatial selection. The issue explicitly says the Bevy viewport must have a usable accessible fallback and must emit user intent, not mutate domain truth.
- [Issue #5209 and PR #5217](https://github.com/Luminous-Dynamics/mycelix/pull/5217): restoration of the missing public `mycelix-leptos-ui` path package is actively being worked on. The current public `mycelix-leptos-core` manifest references that package, but it is absent from the inspected main-tree snapshot. Do not add a cross-repository dependency on this design system until PR #5217 and downstream builds pass on their exact heads.
- The current shared `FormField` helper in `crates/mycelix-leptos-core/src/forms.rs` emits a label separately from its child control without a `for`/ID contract. Audit/fix label association before reusing this helper for trade forms.

Mycelix Hearth's Leptos manifest describes a Leptos + Bevy app, but `mycelix-hearth/apps/leptos/src/bevy/mod.rs` is currently a stub and its Leptos manifest does not declare a Bevy dependency. Treat it as a future direction, not as an existing embeddable Bevy viewport.

**Implication:** Mycelix should own identity/authorization and authoritative service/workflow records, but it should not require the rest of the stack to import a whole Holochain workspace simply to render a job card.

**Next seam:** Define a distinct versioned field-service contract with clear owner and revision semantics; retain manufacturing BOM/routing bindings as a domain-specific extension when a service job actually needs them.

Relevant source: `crates/mycelix-leptos-core/src/`, `mycelix-manufacturing/zomes/workorders/`, `mycelix-hearth/apps/leptos/src/bevy/mod.rs`, and [PR #5217](https://github.com/Luminous-Dynamics/mycelix/pull/5217).

## Framework-neutral contract: initial implementation

The initial **presentation projection and viewport-intent** contract now lives in `sol-atlas-core/src/field_work.rs`, which already provides shared renderer-facing types and depends on Serde without importing Leptos, Bevy, HDK/HDI, or the Symthaea runtime. Frozen examples live in `sol-atlas-core/fixtures/`. This is an initial interop seam, not a claim that the cross-project owner has been agreed or that the types are authoritative canonical work-order records.

The contract currently includes schema versioning, source revision/digest, work phase separated from transport state, trade and asset references, evidence provenance, a cautious safety-gate summary, offered viewport affordances, and typed viewport intents. Its validation and fixtures should be reviewed as a starting point before other repositories implement matching adapters.

- `FieldJobProjectionV1`: stable job ID, job revision, trade/task kind, assigned role, scope, site/asset references, applicable jurisdiction/procedure revisions, current phase, allowed next intents, and freshness.
- `AssetRefV1`: stable asset identity, revision, parent/child relationship and source authority.
- `EvidenceRefV1`: evidence ID, source kind (reported/measured/imported/modelled/verified), actor, timestamp, units, instrument/calibration reference where applicable, procedure revision, and integrity/provenance reference.
- `MeasurementV1`: typed quantity/unit, measurement context, instrument source, quality/freshness, and a distinction between missing, invalid, estimated and observed values.
- `SafetyGateProjectionV1`: rule/procedure identity and revision, jurisdiction/asset/task scope, required evidence, missing/stale/conflicting reasons, and authoritative evaluation status. It is not a client-controlled Boolean.
- `ViewportIntentV1`: select/focus asset, select flow/component, move timeline cursor, request a path explanation, or ask for a compare view. These are intents; authoritative handlers validate resulting commands.

Keep work lifecycle separate from transport lifecycle. For example, “saved on this device”, “submission outcome unknown”, “received by service”, “work reported complete”, “verified”, and “customer accepted” mean different things.

## Leptos / Bevy ownership contract

### Leptos is the default for
- mobile-first navigation and task queues;
- textual procedures, checklists and form input;
- estimates, changes in scope, customer approval and handover;
- evidence provenance and source/freshness labels;
- keyboard, screen-reader, contrast, scaling and reduced-motion behavior;
- offline/connected/error/conflict states and durable receipt display;
- an accessible table/list fallback for all essential 3D information.

### Bevy is for
- interactive site/building/machine geometry;
- spatial selection, camera and scene navigation;
- network/pipe/circuit/component overlays when a spatial view materially helps;
- digital-twin scenarios, time playback, animation and engineering visualization;
- frame-budgeted visual modes (strategic, operational, tactical, scientific and diagnostic) that retain identity and provenance.

### Bridge rules
1. Canonical values are outside both frameworks.
2. Leptos and Bevy consume the same versioned projection; they must not independently invent facts.
3. Bevy emits typed intents; the domain/authority layer validates authorization and state transitions.
4. The selected object in Bevy drives a Leptos inspector and evidence panel.
5. A non-Bevy fallback must preserve task completion if WebGPU/WebGL or the viewport is unavailable.
6. No critical procedure, safety gate, or confirmation exists only inside the Bevy viewport.
7. Do not add JS/npm UI frameworks. Prefer Leptos, Rust crates, and the existing Bevy renderer; use minimal browser glue only where the platform requires it.

## Immediate engineering sequence

**P0 — stabilize and audit the current prototype**
- Run format, native tests and the WASM build on the exact PR #169 head; queued is not pass.
- Browser-check the route for mobile overflow, focus order, accessible names/status messages, reduced motion and target dimensions.
- Keep every job, safety, evidence and connectivity example explicitly synthetic.

**P1 — define the cross-project contract before live integration**
- Agree who owns `FieldJobProjectionV1` and the authoritative service lifecycle; do not use the manufacturing product work-order schema as a universal service schema.
- Freeze a small synthetic cross-trade fixture and negative controls for stale/wrong asset, wrong revision, missing authorization, unknown request outcome, conflicting edits, and reported-vs-verified completion.
- Add exhaustive reducer/transition tests and an offline/restart journal harness; every result should be tied to an exact source SHA and fixture digest.

**P2 — reuse the actual rendering substrate**
- Use Symtropy's existing render bridge and the render benchmark work as the basis for a Bevy viewport, instead of writing a new renderer in the workbench.
- Keep the first view 2D engineering topology / 2.5D context, then add 3D only when it improves a task.
- Align Bevy versions through explicit migration gates. The existing Bevy 0.20 probe is evidence for an isolated compile, not permission to mix the versions.

**P3 — integrate authoritative data and intelligence**
- Mycelix owns authorized lifecycle commands and receipts.
- Symthaea returns source-grounded diagnostic hypotheses, uncertainties and test suggestions.
- Symtropy returns model projections and simulation results.
- Sol-Atlas hosts geographic context and the field-facing Leptos shell.
- Validate each integration against synthetic/negative controls before any field pilot.

## References

- [Leptos forms and controlled inputs](https://book.leptos.dev/view/05_forms.html)
- [Leptos routing](https://book.leptos.dev/router/index.html)
- [Bevy 0.20 release notes (8 October 2026)](https://bevy.org/news/bevy-0-20/)
- [Bevy UI documentation](https://docs.rs/bevy/latest/bevy/ui/index.html)
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/)
