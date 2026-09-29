// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root.

use leptos::prelude::*;

/// Deterministic presentation fixture for the first evidence-native terminal slice.
///
/// This deliberately contains no canonical evidence semantics. It is a renderer
/// fixture whose identifiers are stable enough to exercise the future Mycelix
/// projection boundary.
#[component]
pub fn EvidenceTerminal() -> impl IntoView {
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
                    <span class="frontier-id">"ef:demo:9d7b"</span>
                </div>
            </header>

            <section class="terminal-command" aria-label="Research query">
                <span class="command-mark">">"</span>
                <input
                    type="text"
                    value="Why is this entity shown?"
                    aria-label="Research query"
                />
                <kbd>"⌘K"</kbd>
            </section>

            <div class="terminal-grid">
                <section class="terminal-main">
                    <div class="entity-heading">
                        <div>
                            <span class="eyebrow">"ENTITY / ORGANIZATION"</span>
                            <h2>"Northstar Energy Holdings"</h2>
                            <span class="entity-id">"entity:fin:ns-energy-01"</span>
                        </div>
                        <span class="status-chip qualified">"QUALIFIED"</span>
                    </div>

                    <div class="evidence-card">
                        <div class="card-head">
                            <span>"OBSERVED CLAIM"</span>
                            <span class="mono">"claim:obs:7f31"</span>
                        </div>
                        <h3>"Reported operating capacity increased during the declared period."</h3>
                        <div class="claim-meta">
                            <span>"Observed"</span>
                            <span>"Published 2026-09-28"</span>
                            <span>"Available before frontier"</span>
                        </div>
                    </div>

                    <div class="evidence-card derived">
                        <div class="card-head">
                            <span>"DERIVED PROJECTION"</span>
                            <span class="mono">"derivation:fin:42ac"</span>
                        </div>
                        <h3>"Normalized capacity series is admissible at this frontier."</h3>
                        <div class="claim-meta">
                            <span>"Derived"</span>
                            <span>"Recipe norm-v2"</span>
                            <span>"Factor evidence bound"</span>
                        </div>
                    </div>

                    <div class="evidence-card hypothesis">
                        <div class="card-head">
                            <span>"SYMT HAEA CANDIDATE"</span>
                            <span class="mono">"hypothesis:sym:91e2"</span>
                        </div>
                        <h3>"A supply-side transition may explain part of the observed change."</h3>
                        <div class="claim-meta">
                            <span>"Hypothesis"</span>
                            <span>"Not admitted as fact"</span>
                            <span>"Needs additional evidence"</span>
                        </div>
                    </div>

                    <div class="terminal-section">
                        <div class="section-title">
                            <span>"CONTRADICTION"</span>
                            <span class="status-chip conflicting">"CONFLICTING"</span>
                        </div>
                        <p>"A second provider reports a materially different observation for the same period."</p>
                        <button class="text-action">"Inspect competing evidence →"</button>
                    </div>
                </section>

                <aside class="terminal-aside">
                    <section class="terminal-section why-shown">
                        <div class="section-title">
                            <span>"WHY IS THIS SHOWN?"</span>
                            <button class="icon-action" aria-label="Close evidence drawer">"×"</button>
                        </div>
                        <ol class="lineage">
                            <li><span>"Rendered element"</span><code>"view:entity:01"</code></li>
                            <li><span>"Canonical claim"</span><code>"claim:obs:7f31"</code></li>
                            <li><span>"Evidence"</span><code>"evidence:4a90"</code></li>
                            <li><span>"Source snapshot"</span><code>"source:filing:v3"</code></li>
                            <li><span>"Qualification"</span><code>"profile:fin-001c0"</code></li>
                            <li><span>"Information frontier"</span><code>"ef:demo:9d7b"</code></li>
                        </ol>
                        <button class="replay-action">"↻  Reconstruct at this frontier"</button>
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
                        <div class="section-title"><span>"SYMT HAEA RESEARCH"</span><span class="status-note">"CANDIDATE"</span></div>
                        <p>"Competing explanations, missing information, scenarios and forecasts will enter here through a typed ResearchResult boundary."</p>
                        <button class="text-action">"Show information gaps →"</button>
                    </section>
                </aside>
            </div>
        </main>
    }
}
