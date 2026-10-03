// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use leptos::prelude::*;
use sol_atlas_core::bootstrap::water_purification_fixture;
use sol_atlas_core::capability::{CapabilityId, EvidenceKind};

#[component]
pub fn CapabilityAtlasDemo() -> impl IntoView {
    let graph = water_purification_fixture();
    let root = CapabilityId("water.purification".into());
    let unavailable = CapabilityId("energy.electricity".into());
    let resilience = graph.resilience_assessment(&root, &unavailable);
    let resilience_binding = resilience.is_exactly_bound_to_graph(&graph);
    let (closure, closure_status) = match graph.required_closure(&root) {
        Ok(closure) => (closure, "Required dependency closure resolved.".to_string()),
        Err(error) => {
            let mut details = Vec::new();
            if !error.missing.is_empty() {
                details.push(format!(
                    "{} missing required capability(s)",
                    error.missing.len()
                ));
            }
            if !error.duplicate_ids.is_empty() {
                details.push(format!(
                    "{} duplicate capability ID(s)",
                    error.duplicate_ids.len()
                ));
            }

            (
                Vec::new(),
                format!(
                    "Required dependency closure unresolved: {}.",
                    details.join("; ")
                ),
            )
        }
    };

    let capabilities = closure
        .iter()
        .filter_map(|id| graph.capabilities.iter().find(|c| &c.id == id))
        .collect::<Vec<_>>();

    view! {
        <main class="capability-atlas">
            <header class="capability-atlas-header">
                <div>
                    <p class="capability-kicker">"Humanity / AI Bootstrap Atlas"</p>
                    <h1>"What would it take to establish this capability?"</h1>
                    <p class="capability-lede">
                        "A deterministic, synthetic dependency walk. This surface visualizes semantics; it does not qualify them."
                    </p>
                </div>
                <a class="capability-back" href="/">"← Return to globe"</a>
            </header>

            <section class="capability-warning" aria-label="Evidence status">
                <span class="capability-badge scenario">"SCENARIO"</span>
                <span>
                    "Synthetic fixture only — not field validation, operational proof, or CIV-BOOT qualification."
                </span>
            </section>

            <section class="capability-grid">
                <article class="capability-card capability-root">
                    <p class="capability-label">"ROOT CAPABILITY"</p>
                    <h2>"Water purification"</h2>
                    <p>"Produces potable water from an identified source."</p>
                    <div class="capability-facts">
                        <span>{closure_status.clone()}</span>
                        <span>
                            "Resolved closure entries: "
                            {closure.len()}
                        </span>
                        <span>"Qualification: none"</span>
                    </div>
                </article>

                <article class="capability-card">
                    <p class="capability-label">"DEPENDENCY CLOSURE"</p>
                    <ol class="capability-chain">
                        {capabilities.iter().map(|cap| view! {
                            <li class:root-cap=cap.id == root>
                                <span class="capability-node">{cap.name.clone()}</span>
                                <span class="capability-id">{cap.id.0.clone()}</span>
                            </li>
                        }).collect_view()}
                    </ol>
                </article>
            </section>

            <section class="capability-card capability-resilience">
                <div class="capability-section-title">
                    <div>
                        <p class="capability-label">"STRUCTURAL RESILIENCE ANALYSIS"</p>
                        <h2>"Alternatives are reported, never inferred"</h2>
                    </div>
                    <span class="capability-principle">
                        {format!("Graph binding: {}", if resilience_binding { "exact" } else { "drift detected" })}
                    </span>
                </div>

                <div class="capability-resilience-summary">
                    <span>
                        "Unavailable: "
                        <code>{unavailable.0.clone()}</code>
                    </span>
                    <span>
                        "Structurally affected: "
                        {resilience.affected.len()}
                    </span>
                    <span>
                        "Scope missing prerequisites: "
                        {resilience.scope_snapshot.missing_nodes.len()}
                    </span>
                    <span>
                        "Unresolved impact: "
                        {resilience.unresolved.len()}
                    </span>
                    <span>
                        "Declared alternatives: "
                        {resilience.alternatives.len()}
                    </span>
                    <span>
                        "Unresolved alternatives: "
                        {resilience.unresolved_alternatives.len()}
                    </span>
                </div>

                <div class="capability-resilience-list">
                    {resilience.alternatives.iter().map(|candidate| view! {
                        <div class="capability-resilience-row">
                            <span class="capability-badge scenario">"DECLARED"</span>
                            <span>
                                <strong>{candidate.candidate.0.clone()}</strong>
                                <small>{format!("for {}", candidate.for_dependency.0)}</small>
                            </span>
                            <span>"Closure resolved; selection and interchangeability are not established."</span>
                        </div>
                    }).collect_view()}
                    {resilience.unresolved_alternatives.iter().map(|candidate| view! {
                        <div class="capability-resilience-row unresolved">
                            <span class="capability-badge scenario">"UNRESOLVED"</span>
                            <span>
                                <strong>{candidate.candidate.0.clone()}</strong>
                                <small>{format!("for {}", candidate.for_dependency.0)}</small>
                            </span>
                            <span>
                                {format!("Missing: {}", candidate.missing_capabilities.iter().map(|id| id.0.as_str()).collect::<Vec<_>>().join(", "))}
                            </span>
                        </div>
                    }).collect_view()}
                </div>

                <p class="capability-resilience-ceiling">
                    {resilience.claim_ceiling.clone()}
                </p>
            </section>

            <section class="capability-card">
                <div class="capability-section-title">
                    <div>
                        <p class="capability-label">"EVIDENCE / PROVENANCE / AUTHORITY"</p>
                        <h2>"Nothing is inferred"</h2>
                    </div>
                    <span class="capability-principle">"visual presence ≠ qualification"</span>
                </div>

                <div class="capability-table-wrap">
                    <table class="capability-table">
                        <thead>
                            <tr>
                                <th>"Capability"</th>
                                <th>"Relations"</th>
                                <th>"Evidence"</th>
                                <th>"Location"</th>
                                <th>"Qualification"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {capabilities.iter().map(|cap| {
                                let relation_count = cap.dependencies.iter()
                                    .filter(|d| d.relation.is_required())
                                    .count();
                                let evidence = cap.evidence.first()
                                    .map(|e| match e.kind {
                                        EvidenceKind::Observed => "Observed",
                                        EvidenceKind::Curated => "Curated",
                                        EvidenceKind::Scenario => "Scenario",
                                    })
                                    .unwrap_or("None");
                                let location = if cap.has_location() { "Present" } else { "None" };
                                let qualification = if cap.is_qualified() { "Qualified" } else { "None" };
                                view! {
                                    <tr>
                                        <td>
                                            <strong>{cap.name.clone()}</strong>
                                            <small>{cap.id.0.clone()}</small>
                                        </td>
                                        <td>{relation_count}</td>
                                        <td><span class="capability-badge scenario">{evidence}</span></td>
                                        <td>{location}</td>
                                        <td>{qualification}</td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            </section>

            <section class="capability-card capability-semantics">
                <p class="capability-label">"SEMANTIC GUARDRAILS"</p>
                <div class="capability-principles">
                    <span>"evidence ≠ qualification"</span>
                    <span>{"geographic presence ≠ availability"}</span>
                    <span>"deployment ≠ reproducibility"</span>
                    <span>"capability amplification ≠ authority amplification"</span>
                </div>
            </section>

            <footer class="capability-footer">
                "Synthetic reference: sol-atlas:bootstrap-fixture:v1 · Deterministic closure · Renderer-neutral core"
            </footer>
        </main>
    }
}
