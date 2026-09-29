// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::data::evidence_projection::{
    AtlasEvidenceProjectionV1, ClaimKind, ContradictionRef, EpistemicState, LineageCompleteness,
    LineageNodeRef, LineageResolution, TerminalQueryV1, Visibility,
};

fn fixture_projections() -> Vec<AtlasEvidenceProjectionV1> {
    vec![
        AtlasEvidenceProjectionV1 {
            projection_id: "view:entity:01".into(),
            entity_ref: "entity:fin:ns-energy-01".into(),
            claim_ref: "claim:obs:7f31".into(),
            claim_kind: ClaimKind::Observed,
            statement: "Reported operating capacity increased during the declared period.".into(),
            evidence_refs: vec!["evidence:4a90".into()],
            derivation_ref: None,
            frontier_ref: "ef:demo:9d7b".into(),
            qualification_ref: "profile:fin-001c0".into(),
            qualification: EpistemicState::Qualified,
            contradictions: vec![ContradictionRef {
                claim_ref: "claim:obs:alt-22".into(),
                evidence_refs: vec!["evidence:alt-91".into()],
                summary: "A second provider reports a materially different observation for the same period.".into(),
            }],
            source_snapshot_refs: vec!["source:filing:v3".into()],
            visibility: Visibility::Public,
        },
        AtlasEvidenceProjectionV1 {
            projection_id: "view:derived:02".into(),
            entity_ref: "entity:fin:ns-energy-01".into(),
            claim_ref: "claim:derived:42ac".into(),
            claim_kind: ClaimKind::Derived,
            statement: "Normalized capacity series is admissible at this frontier.".into(),
            evidence_refs: vec!["evidence:4a90".into(), "evidence:factor:fx-17".into()],
            derivation_ref: Some("derivation:fin:42ac".into()),
            frontier_ref: "ef:demo:9d7b".into(),
            qualification: EpistemicState::Qualified,
            contradictions: vec![],
            source_snapshot_refs: vec!["source:filing:v3".into(), "source:fx:v7".into()],
            visibility: Visibility::Public,
        },
        AtlasEvidenceProjectionV1 {
            projection_id: "view:hypothesis:03".into(),
            entity_ref: "entity:fin:ns-energy-01".into(),
            claim_ref: "hypothesis:sym:91e2".into(),
            claim_kind: ClaimKind::Hypothesis,
            statement: "A supply-side transition may explain part of the observed change.".into(),
            evidence_refs: vec!["evidence:4a90".into()],
            derivation_ref: Some("reasoning:symthaea:91e2".into()),
            frontier_ref: "ef:demo:9d7b".into(),
            qualification: EpistemicState::ObservedUnqualified,
            contradictions: vec![],
            source_snapshot_refs: vec!["source:filing:v3".into()],
            visibility: Visibility::Public,
        },
    ]
}

fn claim_kind_label(kind: ClaimKind) -> &'static str {
    match kind {
        ClaimKind::Observed => "OBSERVED CLAIM",
        ClaimKind::Derived => "DERIVED PROJECTION",
        ClaimKind::Hypothesis => "SYMTHAEA CANDIDATE",
    }
}

fn state_label(state: EpistemicState) -> &'static str {
    match state {
        EpistemicState::Qualified => "QUALIFIED",
        EpistemicState::ObservedUnqualified => "OBSERVED / UNQUALIFIED",
        EpistemicState::Conflicting => "CONFLICTING",
        EpistemicState::Stale => "STALE",
        EpistemicState::Protected => "PROTECTED",
        EpistemicState::Unknown => "UNKNOWN",
        EpistemicState::FutureInaccessible => "FUTURE-INACCESSIBLE",
    }
}

fn state_class(state: EpistemicState) -> &'static str {
    match state {
        EpistemicState::Qualified => "qualified",
        EpistemicState::ObservedUnqualified => "observed",
        EpistemicState::Conflicting => "conflicting",
        EpistemicState::Stale => "stale",
        EpistemicState::Protected => "protected",
        EpistemicState::Unknown => "unknown",
        EpistemicState::FutureInaccessible => "future",
    }
}

/// Projection-only terminal surface. Mycelix remains the semantic authority.
#[component]
pub fn EvidenceTerminal() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();

    let terminal_query = move || {
        TerminalQueryV1::from_url_parts(
            params.read().get("entity_ref"),
            query.read().get("claim"),
            query.read().get("frontier"),
            query.read().get("view"),
        )
    };

    let projections = fixture_projections();
    let primary = projections.first().cloned().expect("fixture is non-empty");

    let selected_entity = move || {
        terminal_query()
            .entity_ref
            .unwrap_or_else(|| primary.entity_ref.clone())
    };
    let selected_frontier = move || {
        terminal_query()
            .frontier_ref
            .unwrap_or_else(|| primary.frontier_ref.clone())
    };
    let selected_claim = move || terminal_query().claim_ref;
    let selected_view = move || terminal_query().view.as_str();
    let selected_node = move || LineageNodeRef::parse(query.read().get("node").as_deref());
    let selected_node_status = move || {
        selected_node()
            .map(|node| primary.resolve_lineage_node(&node))
            .unwrap_or(LineageResolution::Incomplete)
    };
    let lineage_href = move |node: LineageNodeRef| {
        format!(
            "/terminal/entity/{}?frontier={}&claim={}&view=lineage&node={}",
            selected_entity(),
            selected_frontier(),
            primary.claim_ref,
            node.query_value()
        )
    };
    let replay_href = move || {
        format!(
            "/terminal/entity/{}?frontier={}&claim={}&view=evidence",
            selected_entity(),
            selected_frontier(),
            primary.claim_ref
        )
    };

    view! {
        <main class="evidence-terminal">
            <header class="terminal-header">
                <div>
                    <div class="terminal-kicker">"MYCELIX / SYMTHAEA / SOL ATLAS"</div>
                    <h1>"Evidence Terminal"</h1>
                    <p>"Evidence first. Frontier explicit. Reasoning reversible."</p>
                </div>
                <div class="terminal-frontier">
                    <span class="frontier-label">"INFORMATION FRONTIER"</span>
                    <strong>"2026-09-29T14:00:00Z"</strong>
                    <span class="frontier-id">{selected_frontier}</span>
                </div>
            </header>

            <section class="terminal-command" aria-label="Research query">
                <span class="command-mark">">"</span>
                <input type="text" value="Why is this entity shown?" aria-label="Research query"/>
                <kbd>"⌘K"</kbd>
            </section>

            <div class="terminal-grid">
                <section class="terminal-main">
                    <div class="entity-heading">
                        <div>
                            <span class="eyebrow">"ENTITY / ORGANIZATION"</span>
                            <h2>"Northstar Energy Holdings"</h2>
                            <span class="entity-id">{selected_entity}</span>
                        </div>
                        <span class="status-chip qualified">"QUALIFIED"</span>
                    </div>

                    {projections.into_iter().map(|projection| {
                        let kind_class = match projection.claim_kind {
                            ClaimKind::Observed => "",
                            ClaimKind::Derived => " derived",
                            ClaimKind::Hypothesis => " hypothesis",
                        };
                        let status_class = state_class(projection.qualification);
                        let kind_label = claim_kind_label(projection.claim_kind);
                        let status = state_label(projection.qualification);
                        view! {
                            <article class=format!("evidence-card{kind_class}")>
                                <div class="card-head">
                                    <span>{kind_label}</span>
                                    <span class="mono">{projection.claim_ref.clone()}</span>
                                </div>
                                <h3>{projection.statement.clone()}</h3>
                                <div class="claim-meta">
                                    <span>{status}</span>
                                    <span>{format!("{} evidence ref(s)", projection.evidence_refs.len())}</span>
                                    <span>{format!("frontier {}", projection.frontier_ref)}</span>
                                </div>
                            </article>
                        }
                    }).collect_view()}

                    <div class="terminal-section">
                        <div class="section-title">
                            <span>"CONTRADICTION"</span>
                            <span class="status-chip conflicting">"CONFLICTING"</span>
                        </div>
                        <p>{primary.contradictions.first().map(|c| c.summary.clone()).unwrap_or_else(|| "No contradiction recorded.".into())}</p>
                        <A class="text-action" href=move || format!("/terminal/entity/{}?frontier={}&claim={}&view=lineage", selected_entity(), selected_frontier(), primary.claim_ref)>"Inspect competing evidence →"</A>
                    </div>
                </section>

                <aside class="terminal-aside">
                    <section class="terminal-section why-shown">
                        <div class="section-title">
                            <span>"WHY IS THIS SHOWN?"</span>
                            <button class="icon-action" aria-label="Close evidence drawer">"×"</button>
                        </div>
                        <ol class="lineage">
                            {primary.lineage_nodes().into_iter().map(|node| {
                                let label = match &node {
                                    LineageNodeRef::Projection(_) => "Rendered projection",
                                    LineageNodeRef::Claim(_) => "Canonical claim",
                                    LineageNodeRef::Evidence(_) => "Evidence",
                                    LineageNodeRef::SourceSnapshot(_) => "Source snapshot",
                                    LineageNodeRef::Qualification(_) => "Qualification profile",
                                    LineageNodeRef::Frontier(_) => "Information frontier",
                                    LineageNodeRef::Derivation(_) => "Derivation receipt",
                                    LineageNodeRef::ReasoningReceipt(_) => "Reasoning receipt",
                                };
                                let href = lineage_href(node.clone());
                                view! {
                                    <li>
                                        <span>{label}</span>
                                        <A class="lineage-link" href=href>{node.id().to_string()}</A>
                                    </li>
                                }
                            }).collect_view()}
                        </ol>
                        <div class="lineage-selection">
                            <span>"INSPECTED NODE"</span>
                            <code>{move || selected_node().map(|node| node.query_value()).unwrap_or_else(|| "none".into())}</code>
                            <span class="lineage-resolution">
                                {move || match selected_node_status() {
                                    LineageResolution::Resolved => "RESOLVED BY DECLARED PROJECTION",
                                    LineageResolution::Unavailable => "UNAVAILABLE · NO FALLBACK",
                                    LineageResolution::Protected => "PROTECTED · CONTENT NOT DISCLOSED",
                                    LineageResolution::Incomplete => "INCOMPLETE · NO NODE SELECTED",
                                }}
                            </span>
                        </div>
                        {match primary.lineage_completeness() {
                            LineageCompleteness::Complete => view! {
                                <div class="lineage-integrity complete">"LINEAGE COMPLETE · all declared dependencies are addressable"</div>
                            }.into_any(),
                            LineageCompleteness::Incomplete => view! {
                                <div class="lineage-integrity incomplete">"LINEAGE INCOMPLETE · no missing dependency is inferred"</div>
                            }.into_any(),
                        }}
                        <A class="replay-action" href=replay_href>"↻  Reconstruct at this frontier"</A>
                        <p class="lineage-note">"Inspection links address declared dependencies only; an unresolved node is shown as unavailable rather than replaced with current or inferred data."</p>
                        <div class="terminal-route-state">
                            <span>"URL view"</span><code>{selected_view}</code>
                            <span>"Claim filter"</span><code>{move || selected_claim().unwrap_or_else(|| "none".into())}</code>
                        </div>
                    </section>

                    <section class="terminal-section status-map">
                        <div class="section-title"><span>"EPISTEMIC STATES"</span></div>
                        <div class="state-list">
                            <span class="status-chip qualified">"Qualified"</span>
                            <span class="status-chip observed">"Observed / Unqualified"</span>
                            <span class="status-chip conflicting">"Conflicting"</span>
                            <span class="status-chip stale">"Stale"</span>
                            <span class="status-chip protected">"Protected"</span>
                            <span class="status-chip unknown">"Unknown"</span>
                            <span class="status-chip future">"Future-Inaccessible"</span>
                        </div>
                    </section>

                    <section class="terminal-section research-panel">
                        <div class="section-title">
                            <span>"SYMTHAEA RESEARCH"</span>
                            <span class="status-note">{state_label(primary.qualification)}</span>
                        </div>
                        <p>"Competing explanations, missing information, scenarios and forecasts will enter here through a typed ResearchResult boundary."</p>
                        <A class="text-action" href=move || format!("/terminal/entity/{}?frontier={}&claim={}&view=research", selected_entity(), selected_frontier(), primary.claim_ref)>"Show information gaps →"</A>
                    </section>
                </aside>
            </div>
        </main>
    }
}
