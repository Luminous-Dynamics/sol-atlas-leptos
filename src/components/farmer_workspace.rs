// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Field-work UX prototype for the farmer decision-to-outcome loop.
//!
//! Every farm, observation, quote, provider and status in this component is
//! synthetic. All interactions are in-memory UI simulations: no request,
//! notification, booking, payment, or physical action is performed.

use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn FarmerWorkspace() -> impl IntoView {
    let selected = RwSignal::new("today");
    let simulated_offline = RwSignal::new(true);
    // 0: no local report; 1: local report; 2: synthetic quote;
    // 3: quote acceptance simulated; 4: booking confirmation simulated;
    // 5: provider-reported completion; 6: farmer accepts; 7: dispute/rework.
    let workflow = RwSignal::new(0_u8);

    let online_label = move || {
        if simulated_offline.get() {
            "Offline mode simulated"
        } else {
            "Connected mode simulated"
        }
    };

    let state_label = move || match workflow.get() {
        0 => "No sample request recorded",
        1 => "Saved on this device — not sent",
        2 => "Synthetic quote available — not accepted",
        3 => "Quote acceptance simulated — awaiting provider confirmation",
        4 => "Booking confirmation simulated — work not completed",
        5 => "Provider reports completion — farmer decision pending",
        6 => "Farmer acceptance simulated",
        7 => "Problem reported — demo rework path",
        _ => "Unknown demo state",
    };

    view! {
        <main class="field-work-page">
            <header class="field-work-header">
                <div class="field-work-brand">
                    <span class="field-work-mark" aria-hidden="true">"SA"</span>
                    <div>
                        <p class="field-work-eyebrow">"SOL ATLAS · FIELD WORK"</p>
                        <h1>"Farm workspace"</h1>
                    </div>
                </div>
                <div class="field-work-header-actions">
                    <A href="/" attr:class="field-work-back">"← Globe"</A>
                    <button
                        class="field-work-mode"
                        aria-pressed=move || simulated_offline.get().to_string()
                        on:click=move |_| simulated_offline.update(|offline| *offline = !*offline)
                    >
                        {move || if simulated_offline.get() { "Simulate connected mode" } else { "Simulate offline mode" }}
                    </button>
                </div>
            </header>

            <section class="field-work-disclaimer" role="note" aria-label="Prototype data notice">
                <span class="field-work-disclaimer-icon" aria-hidden="true">"ⓘ"</span>
                <div>
                    <strong>"Interactive prototype — synthetic data only"</strong>
                    <p>"No live farm, weather, map, route, provider, or payment service is connected. Buttons below change local demo state only."</p>
                </div>
            </section>

            <div class="field-work-layout">
                <aside class="field-work-sidebar" aria-label="Farm workspace navigation">
                    <p class="field-work-eyebrow">"WORKSPACE"</p>
                    <nav class="field-work-nav">
                        <button class:active=move || selected.get() == "today" on:click=move |_| selected.set("today")>
                            <span aria-hidden="true">"⌂"</span><span>"Today"</span>
                        </button>
                        <button class:active=move || selected.get() == "land" on:click=move |_| selected.set("land")>
                            <span aria-hidden="true">"▧"</span><span>"Land"</span>
                        </button>
                        <button class:active=move || selected.get() == "work" on:click=move |_| selected.set("work")>
                            <span aria-hidden="true">"✓"</span><span>"Work"</span>
                        </button>
                        <button class:active=move || selected.get() == "services" on:click=move |_| selected.set("services")>
                            <span aria-hidden="true">"↔"</span><span>"Services"</span>
                            <span class="field-work-nav-count" aria-label="one demo workflow">"1"</span>
                        </button>
                        <button class:active=move || selected.get() == "sales" on:click=move |_| selected.set("sales")>
                            <span aria-hidden="true">"$"</span><span>"Sales"</span>
                        </button>
                    </nav>
                    <div class="field-work-sidebar-foot">
                        <span class="field-work-status-dot" aria-hidden="true"></span>
                        <div>
                            <strong>{online_label}</strong>
                            <p>"State is simulated"</p>
                        </div>
                    </div>
                </aside>

                <section class="field-work-content">
                    <Show when=move || selected.get() == "today">
                        <div class="field-work-page-intro">
                            <div>
                                <p class="field-work-eyebrow">"SUNDAY · DEMO FARM"</p>
                                <h2>"Good day. What needs attention?"</h2>
                                <p class="field-work-muted">"A small list of decisions and work — not a wall of charts."</p>
                            </div>
                            <span class="field-work-demo-pill">"SYNTHETIC EXAMPLE"</span>
                        </div>

                        <div class="field-work-attention-card">
                            <div class="field-work-attention-symbol" aria-hidden="true">"!"</div>
                            <div class="field-work-attention-body">
                                <div class="field-work-card-topline">
                                    <span class="field-work-label">"NEEDS ATTENTION"</span>
                                    <span class="field-work-state field-work-state-unknown">"Reported · unverified"</span>
                                </div>
                                <h3>"Irrigation pump may not be delivering water"</h3>
                                <p>"Synthetic report: a farm worker noticed reduced flow. Pump condition and cause have not been independently verified."</p>
                                <div class="field-work-evidence-row">
                                    <span>"Source: sample user report"</span>
                                    <span>"As of: demo timestamp"</span>
                                </div>
                                <div class="field-work-card-actions">
                                    <button class="field-work-primary" on:click=move |_| selected.set("services")>"Review request"</button>
                                    <button class="field-work-secondary" on:click=move |_| selected.set("land")>"View field context"</button>
                                </div>
                            </div>
                        </div>

                        <div class="field-work-section-heading">
                            <div><h3>"Today's work"</h3><p>"Example tasks, not live assignments."</p></div>
                            <button class="field-work-text-button" on:click=move |_| selected.set("work")>"View all work →"</button>
                        </div>
                        <div class="field-work-task-list">
                            <div class="field-work-task-row">
                                <span class="field-work-task-icon" aria-hidden="true">"◷"</span>
                                <div class="field-work-task-main"><strong>"Check irrigation line"</strong><p>"Window: not set · Owner: not assigned"</p></div>
                                <span class="field-work-state field-work-state-neutral">"Needs planning"</span>
                            </div>
                            <div class="field-work-task-row">
                                <span class="field-work-task-icon" aria-hidden="true">"≋"</span>
                                <div class="field-work-task-main"><strong>"Review water availability"</strong><p>"No current water-system feed connected."</p></div>
                                <span class="field-work-state field-work-state-unknown">"Unknown"</span>
                            </div>
                        </div>

                        <div class="field-work-section-heading">
                            <div><h3>"Quick actions"</h3><p>"Nothing is sent outside this prototype."</p></div>
                        </div>
                        <div class="field-work-quick-actions">
                            <button on:click=move |_| { workflow.set(1); selected.set("services"); }><span aria-hidden="true">"+" </span>"Record a problem"</button>
                            <button on:click=move |_| selected.set("land")><span aria-hidden="true">"▧" </span>"Inspect land evidence"</button>
                            <button on:click=move |_| selected.set("services")><span aria-hidden="true">"↔" </span>"Review service options"</button>
                        </div>
                    </Show>

                    <Show when=move || selected.get() == "land">
                        <div class="field-work-page-intro">
                            <div><p class="field-work-eyebrow">"LAND · SYNTHETIC SITE"</p><h2>"Field context"</h2><p class="field-work-muted">"Spatial context should help a decision; it should never manufacture access or certainty."</p></div>
                        </div>
                        <div class="field-work-land-grid">
                            <div class="field-work-map-placeholder" role="img" aria-label="Illustrative field map placeholder; not geospatial data">
                                <div class="field-work-map-grid"></div>
                                <div class="field-work-map-field"><span>"DEMO FIELD A"</span></div>
                                <span class="field-work-map-caption">"Illustration · no real coordinates"</span>
                            </div>
                            <div class="field-work-panel">
                                <p class="field-work-eyebrow">"SELECTED SITE"</p>
                                <h3>"Demo Field A"</h3>
                                <p class="field-work-muted">"Opaque sample reference: SITE-DEMO-A"</p>
                                <dl class="field-work-facts">
                                    <div><dt>"Crop"</dt><dd>"Not specified"</dd></div>
                                    <div><dt>"Soil moisture"</dt><dd><span class="field-work-state field-work-state-unknown">"Unknown"</span></dd></div>
                                    <div><dt>"Route time"</dt><dd>"Not available"</dd></div>
                                    <div><dt>"Water feed"</dt><dd>"Not connected"</dd></div>
                                </dl>
                                <p class="field-work-note">"Straight-line distance, if later available, is not a measured or validated travel time."</p>
                            </div>
                        </div>
                    </Show>

                    <Show when=move || selected.get() == "work">
                        <div class="field-work-page-intro"><div><p class="field-work-eyebrow">"WORK QUEUE"</p><h2>"Tasks and commitments"</h2><p class="field-work-muted">"Planned work is not completed work."</p></div></div>
                        <div class="field-work-panel field-work-full-panel">
                            <div class="field-work-card-topline"><span class="field-work-label">"SAMPLE TASK"</span><span class="field-work-state field-work-state-neutral">"Not assigned"</span></div>
                            <h3>"Inspect irrigation pump and line"</h3>
                            <p>"No technician, arrival time, or accepted quote exists in this prototype state."</p>
                            <ul class="field-work-list">
                                <li>"Scope: inspection only; diagnosis not established."</li>
                                <li>"Required parts and total cost: unknown."</li>
                                <li>"Completion evidence: none."</li>
                            </ul>
                            <button class="field-work-primary" on:click=move |_| selected.set("services")>"Continue to service workflow"</button>
                        </div>
                    </Show>

                    <Show when=move || selected.get() == "services">
                        <div class="field-work-page-intro">
                            <div><p class="field-work-eyebrow">"SERVICES · LOCAL INTERACTION DEMO"</p><h2>"From reported problem to farmer decision"</h2><p class="field-work-muted">"Every step below is simulated locally; no provider is contacted."</p></div>
                            <span class="field-work-demo-pill">"NO BACKEND"</span>
                        </div>

                        <div class="field-work-workflow-status" aria-live="polite">
                            <div><span class="field-work-label">"CURRENT DEMO STATE"</span><strong>{state_label}</strong></div>
                            <span class="field-work-state field-work-state-unknown">"Local simulation"</span>
                        </div>

                        <div class="field-work-stepper" aria-label="Service lifecycle steps">
                            <div class:current=move || workflow.get() == 1 class:complete=move || workflow.get() > 1><span>"1"</span><small>"Request"</small></div>
                            <div class:current=move || workflow.get() == 2 class:complete=move || workflow.get() > 2><span>"2"</span><small>"Quote"</small></div>
                            <div class:current=move || workflow.get() == 3 || workflow.get() == 4 class:complete=move || workflow.get() > 4><span>"3"</span><small>"Booking"</small></div>
                            <div class:current=move || workflow.get() == 5 class:complete=move || workflow.get() > 5><span>"4"</span><small>"Provider report"</small></div>
                            <div class:current=move || workflow.get() == 6 || workflow.get() == 7><span>"5"</span><small>"Farmer decision"</small></div>
                        </div>

                        <div class="field-work-panel field-work-full-panel">
                            <Show when=move || workflow.get() == 0>
                                <p class="field-work-eyebrow">"STEP 1 · REPORT"</p>
                                <h3>"Record the irrigation problem"</h3>
                                <p>"Choose this when you need help. This demo saves a synthetic record locally and does not transmit it."</p>
                                <div class="field-work-summary"><strong>"Reported issue"</strong><span>"Reduced water flow at Demo Field A"</span><small>"Source: sample farmer report · Cause: unknown"</small></div>
                                <button class="field-work-primary" on:click=move |_| workflow.set(1)>"Save sample request on this device"</button>
                            </Show>

                            <Show when=move || workflow.get() == 1>
                                <p class="field-work-eyebrow">"STEP 1 · LOCAL RECEIPT"</p>
                                <h3>"Saved on this device"</h3>
                                <p>"The sample record is in the prototype's memory. It has not reached a provider or backend."</p>
                                <div class="field-work-receipt"><strong>"LOCAL-DEMO-001"</strong><span>"Local only · no request sent"</span></div>
                                <button class="field-work-primary" on:click=move |_| workflow.set(2)>"Review synthetic quote"</button>
                                <button class="field-work-secondary" on:click=move |_| workflow.set(0)>"Discard demo and restart"</button>
                            </Show>

                            <Show when=move || workflow.get() == 2>
                                <p class="field-work-eyebrow">"STEP 2 · SYNTHETIC OFFER"</p>
                                <h3>"Example quote — not from a real provider"</h3>
                                <div class="field-work-quote">
                                    <div><span>"Inspection labor"</span><strong>"R 350.00"</strong></div>
                                    <div><span>"Travel fee"</span><strong>"Unknown"</strong></div>
                                    <div><span>"Parts"</span><strong>"Excluded / not estimated"</strong></div>
                                    <div><span>"Validity"</span><strong>"Demo offer only"</strong></div>
                                </div>
                                <p class="field-work-note">"This illustrative quote is intentionally incomplete. The real workflow must block acceptance until material terms are clear."</p>
                                <button class="field-work-primary" on:click=move |_| workflow.set(3)>"Simulate requesting complete terms"</button>
                                <button class="field-work-secondary" on:click=move |_| workflow.set(1)>"Back to request"</button>
                            </Show>

                            <Show when=move || workflow.get() == 3>
                                <p class="field-work-eyebrow">"STEP 2 · TERMS REQUESTED"</p>
                                <h3>"The quote is not accepted or booked"</h3>
                                <p>"A missing travel fee or unknown parts cost is material. This prototype will not pretend the full price is known."</p>
                                <div class="field-work-receipt"><strong>"Awaiting complete terms"</strong><span>"No booking · no payment · no provider contacted"</span></div>
                                <button class="field-work-primary" on:click=move |_| workflow.set(4)>"Simulate receipt of a complete quote and booking"</button>
                                <button class="field-work-secondary" on:click=move |_| workflow.set(2)>"Review original example"</button>
                                <p class="field-work-note">"The next action skips two external actors for demo navigation only; it does not claim that a real quote was received or accepted."</p>
                            </Show>

                            <Show when=move || workflow.get() == 4>
                                <p class="field-work-eyebrow">"STEP 3 · BOOKING DEMO"</p>
                                <h3>"Booking confirmation simulated"</h3>
                                <p>"For this local demo only, assume complete terms were received and explicitly accepted. In a real system, booking confirmation must come from the provider."</p>
                                <div class="field-work-receipt"><strong>"SIMULATED BOOKING"</strong><span>"Work not completed · payment not made"</span></div>
                                <button class="field-work-primary" on:click=move |_| workflow.set(5)>"Simulate provider reports work complete"</button>
                            </Show>

                            <Show when=move || workflow.get() == 5>
                                <p class="field-work-eyebrow">"STEP 4 · PROVIDER CLAIM"</p>
                                <h3>"The provider reports completion"</h3>
                                <p>"This is a provider claim in the demo, not independent proof of quality and not farmer acceptance."</p>
                                <div class="field-work-receipt"><strong>"Awaiting your decision"</strong><span>"Not accepted · payment state unchanged"</span></div>
                                <div class="field-work-card-actions">
                                    <button class="field-work-primary" on:click=move |_| workflow.set(6)>"Simulate accepting work"</button>
                                    <button class="field-work-secondary" on:click=move |_| workflow.set(7)>"Report a problem"</button>
                                </div>
                            </Show>

                            <Show when=move || workflow.get() == 6>
                                <p class="field-work-eyebrow">"STEP 5 · FARMER DECISION"</p>
                                <h3>"Acceptance recorded in this prototype"</h3>
                                <p>"The demo now records a separate farmer decision. That is still not a payment receipt or proof of independently verified repair quality."</p>
                                <div class="field-work-receipt"><strong>"FARMER ACCEPTANCE · SIMULATED"</strong><span>"Payment: not recorded"</span></div>
                                <button class="field-work-secondary" on:click=move |_| workflow.set(5)>"Review completion claim again"</button>
                                <button class="field-work-secondary" on:click=move |_| workflow.set(0)>"Restart demo workflow"</button>
                            </Show>

                            <Show when=move || workflow.get() == 7>
                                <p class="field-work-eyebrow">"STEP 5 · DISPUTE"</p>
                                <h3>"Problem reported — unresolved"</h3>
                                <p>"The report is preserved in this local demo. No one is labelled at fault and no automatic penalty is applied."</p>
                                <div class="field-work-receipt"><strong>"REWORK REQUEST · SIMULATED"</strong><span>"Resolution: pending · payment: not recorded"</span></div>
                                <button class="field-work-primary" on:click=move |_| workflow.set(5)>"Return to completion review"</button>
                                <button class="field-work-secondary" on:click=move |_| workflow.set(0)>"Restart demo workflow"</button>
                            </Show>
                        </div>
                    </Show>

                    <Show when=move || selected.get() == "sales">
                        <div class="field-work-page-intro"><div><p class="field-work-eyebrow">"SALES · FUTURE VERTICAL"</p><h2>"Harvest and sales"</h2><p class="field-work-muted">"Build this only after product units, lot state and fulfilment semantics are integrated with the canonical Mycelix food contracts."</p></div></div>
                        <div class="field-work-panel field-work-full-panel">
                            <h3>"No live inventory is available"</h3>
                            <p>"Planned harvest, forecast supply, reported stock, reserved stock and delivered quantity must remain distinct."</p>
                            <p class="field-work-note">"This screen intentionally contains no invented crop quantity, price or buyer."</p>
                        </div>
                    </Show>

                    <footer class="field-work-footer">
                        <span>"Prototype v0.1 · local state only"</span>
                        <span>"Truthful states · farmer choice · no real transactions"</span>
                    </footer>
                </section>
            </div>
        </main>
    }
}
