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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DemoWorkflowState {
    Empty,
    LocalRequest,
    IncompleteQuote,
    TermsRequested,
    CompleteQuote,
    QuoteAccepted,
    BookingConfirmed,
    ProviderReportedComplete,
    FarmerAccepted,
    Disputed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DemoAction {
    SaveRequest,
    ReviewQuote,
    RequestCompleteTerms,
    ReceiveCompleteQuote,
    AcceptQuote,
    ConfirmBooking,
    ReportProviderComplete,
    FarmerAccept,
    FarmerDispute,
    ReturnToRequest,
    DeclineQuote,
    Restart,
}

impl DemoWorkflowState {
    fn label(self) -> &'static str {
        match self {
            Self::Empty => "No sample request recorded",
            Self::LocalRequest => "Saved on this device — not sent",
            Self::IncompleteQuote => "Synthetic incomplete quote available — not accepted",
            Self::TermsRequested => "Complete terms requested — no quote accepted",
            Self::CompleteQuote => "Complete synthetic quote received — not accepted",
            Self::QuoteAccepted => "Quote acceptance simulated — awaiting provider confirmation",
            Self::BookingConfirmed => "Booking confirmation simulated — work not completed",
            Self::ProviderReportedComplete => {
                "Provider reports completion — farmer decision pending"
            }
            Self::FarmerAccepted => "Farmer acceptance simulated",
            Self::Disputed => "Problem reported — demo rework path",
        }
    }

    fn step(self) -> u8 {
        match self {
            Self::Empty => 0,
            Self::LocalRequest => 1,
            Self::IncompleteQuote | Self::TermsRequested | Self::CompleteQuote => 2,
            Self::QuoteAccepted | Self::BookingConfirmed => 3,
            Self::ProviderReportedComplete => 4,
            Self::FarmerAccepted | Self::Disputed => 5,
        }
    }
}

/// Independent demo reducer: impossible shortcuts leave state unchanged.
fn transition(state: DemoWorkflowState, action: DemoAction) -> DemoWorkflowState {
    use DemoAction as A;
    use DemoWorkflowState as S;

    match (state, action) {
        (S::Empty, A::SaveRequest) => S::LocalRequest,
        (S::LocalRequest, A::ReviewQuote) => S::IncompleteQuote,
        (S::IncompleteQuote, A::RequestCompleteTerms) => S::TermsRequested,
        (S::TermsRequested, A::ReceiveCompleteQuote) => S::CompleteQuote,
        (S::CompleteQuote, A::AcceptQuote) => S::QuoteAccepted,
        (S::QuoteAccepted, A::ConfirmBooking) => S::BookingConfirmed,
        (S::BookingConfirmed, A::ReportProviderComplete) => S::ProviderReportedComplete,
        (S::ProviderReportedComplete, A::FarmerAccept) => S::FarmerAccepted,
        (S::ProviderReportedComplete, A::FarmerDispute) => S::Disputed,
        (S::IncompleteQuote, A::ReturnToRequest) => S::LocalRequest,
        (S::CompleteQuote, A::DeclineQuote) => S::LocalRequest,
        (_, A::Restart) => S::Empty,
        _ => state,
    }
}

#[component]
pub fn FarmerWorkspace() -> impl IntoView {
    let selected = RwSignal::new("today");
    let simulated_offline = RwSignal::new(true);
    let workflow = RwSignal::new(DemoWorkflowState::Empty);

    let dispatch =
        move |action: DemoAction| workflow.update(|state| *state = transition(*state, action));

    let online_label = move || {
        if simulated_offline.get() {
            "Offline mode simulated"
        } else {
            "Connected mode simulated"
        }
    };

    let state_label = move || workflow.get().label();

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
                            <button on:click=move |_| { dispatch(DemoAction::SaveRequest); selected.set("services"); }><span aria-hidden="true">"+" </span>"Record a problem"</button>
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
                            <div class:current=move || { workflow.get() == DemoWorkflowState::LocalRequest } class:complete=move || { workflow.get().step() > 1 }><span>"1"</span><small>"Request"</small></div>
                            <div class:current=move || { matches!(workflow.get(), DemoWorkflowState::IncompleteQuote | DemoWorkflowState::TermsRequested | DemoWorkflowState::CompleteQuote) } class:complete=move || { workflow.get().step() > 2 }><span>"2"</span><small>"Quote"</small></div>
                            <div class:current=move || { matches!(workflow.get(), DemoWorkflowState::QuoteAccepted | DemoWorkflowState::BookingConfirmed) } class:complete=move || { workflow.get().step() > 3 }><span>"3"</span><small>"Booking"</small></div>
                            <div class:current=move || { workflow.get() == DemoWorkflowState::ProviderReportedComplete } class:complete=move || { workflow.get().step() > 4 }><span>"4"</span><small>"Provider report"</small></div>
                            <div class:current=move || { matches!(workflow.get(), DemoWorkflowState::FarmerAccepted | DemoWorkflowState::Disputed) }><span>"5"</span><small>"Farmer decision"</small></div>
                        </div>

                        <div class="field-work-panel field-work-full-panel">
                            <Show when=move || workflow.get() == DemoWorkflowState::Empty>
                                <p class="field-work-eyebrow">"STEP 1 · REPORT"</p>
                                <h3>"Record the irrigation problem"</h3>
                                <p>"Choose this when you need help. This demo saves a synthetic record locally and does not transmit it."</p>
                                <div class="field-work-summary"><strong>"Reported issue"</strong><span>"Reduced water flow at Demo Field A"</span><small>"Source: sample farmer report · Cause: unknown"</small></div>
                                <button class="field-work-primary" on:click=move |_| dispatch(DemoAction::SaveRequest)>"Save sample request on this device"</button>
                            </Show>

                            <Show when=move || workflow.get() == DemoWorkflowState::LocalRequest>
                                <p class="field-work-eyebrow">"STEP 1 · LOCAL RECEIPT"</p>
                                <h3>"Saved on this device"</h3>
                                <p>"The sample record is in the prototype's memory. It has not reached a provider or backend."</p>
                                <div class="field-work-receipt"><strong>"LOCAL-DEMO-001"</strong><span>"Local only · no request sent"</span></div>
                                <button class="field-work-primary" on:click=move |_| dispatch(DemoAction::ReviewQuote)>"Review synthetic quote"</button>
                                <button class="field-work-secondary" on:click=move |_| dispatch(DemoAction::Restart)>"Discard demo and restart"</button>
                            </Show>

                            <Show when=move || workflow.get() == DemoWorkflowState::IncompleteQuote>
                                <p class="field-work-eyebrow">"STEP 2 · SYNTHETIC OFFER"</p>
                                <h3>"Example quote — not from a real provider"</h3>
                                <div class="field-work-quote">
                                    <div><span>"Inspection labor"</span><strong>"R 350.00"</strong></div>
                                    <div><span>"Travel fee"</span><strong>"Unknown"</strong></div>
                                    <div><span>"Parts"</span><strong>"Excluded / not estimated"</strong></div>
                                    <div><span>"Validity"</span><strong>"Demo offer only"</strong></div>
                                </div>
                                <p class="field-work-note">"This illustrative quote is intentionally incomplete. The real workflow must block acceptance until material terms are clear."</p>
                                <button class="field-work-primary" on:click=move |_| dispatch(DemoAction::RequestCompleteTerms)>"Simulate requesting complete terms"</button>
                                <button class="field-work-secondary" on:click=move |_| dispatch(DemoAction::ReturnToRequest)>"Back to request"</button>
                            </Show>

                            <Show when=move || workflow.get() == DemoWorkflowState::TermsRequested>
                                <p class="field-work-eyebrow">"STEP 2 · TERMS REQUESTED"</p>
                                <h3>"The quote is not accepted or booked"</h3>
                                <p>"A missing travel fee or unknown parts cost is material. This prototype will not pretend the full price is known."</p>
                                <div class="field-work-receipt"><strong>"Awaiting complete terms"</strong><span>"No booking · no payment · no provider contacted"</span></div>
                                <button class="field-work-primary" on:click=move |_| dispatch(DemoAction::ReceiveCompleteQuote)>"Simulate receiving a complete quote"</button>
                            </Show>

                            <Show when=move || workflow.get() == DemoWorkflowState::CompleteQuote>
                                <p class="field-work-eyebrow">"STEP 2 · COMPLETE SYNTHETIC QUOTE"</p>
                                <h3>"Review full terms before deciding"</h3>
                                <p>"This is a synthetic example with every price field stated. It is still not an offer from a real provider."</p>
                                <div class="field-work-quote">
                                    <div><span>"Inspection and labor"</span><strong>"R 350.00"</strong></div>
                                    <div><span>"Travel fee"</span><strong>"R 75.00"</strong></div>
                                    <div><span>"Parts allowance"</span><strong>"R 150.00 maximum"</strong></div>
                                    <div><span>"Total cap (synthetic)"</span><strong>"R 575.00"</strong></div>
                                    <div><span>"Scope"</span><strong>"Inspect pump; replace one seal if required"</strong></div>
                                    <div><span>"Validity"</span><strong>"Until demo expiry"</strong></div>
                                </div>
                                <button class="field-work-primary" on:click=move |_| dispatch(DemoAction::AcceptQuote)>"Simulate accepting complete terms"</button>
                                <button class="field-work-secondary" on:click=move |_| dispatch(DemoAction::DeclineQuote)>"Decline and return to request"</button>
                            </Show>

                            <Show when=move || workflow.get() == DemoWorkflowState::QuoteAccepted>
                                <p class="field-work-eyebrow">"STEP 3 · QUOTE DECISION"</p>
                                <h3>"Quote acceptance simulated; awaiting provider confirmation"</h3>
                                <p>"A farmer decision has been modelled, but no booking is yet confirmed. This prototype has not contacted any provider."</p>
                                <div class="field-work-receipt"><strong>"QUOTE ACCEPTED · SIMULATED"</strong><span>"Booking: pending provider confirmation · payment: not authorized"</span></div>
                                <button class="field-work-primary" on:click=move |_| dispatch(DemoAction::ConfirmBooking)>"Simulate provider confirms booking"</button>
                            </Show>

                            <Show when=move || workflow.get() == DemoWorkflowState::BookingConfirmed>
                                <p class="field-work-eyebrow">"STEP 3 · BOOKING DEMO"</p>
                                <h3>"Booking confirmation simulated"</h3>
                                <p>"This state is explicitly separate from quote acceptance. In a real system, a provider acknowledgement would be required."</p>
                                <div class="field-work-receipt"><strong>"SIMULATED BOOKING"</strong><span>"Work not completed · payment not made"</span></div>
                                <button class="field-work-primary" on:click=move |_| dispatch(DemoAction::ReportProviderComplete)>"Simulate provider reports work complete"</button>
                            </Show>

                            <Show when=move || workflow.get() == DemoWorkflowState::ProviderReportedComplete>
                                <p class="field-work-eyebrow">"STEP 4 · PROVIDER CLAIM"</p>
                                <h3>"The provider reports completion"</h3>
                                <p>"This is a provider claim in the demo, not independent proof of quality and not farmer acceptance."</p>
                                <div class="field-work-receipt"><strong>"Awaiting your decision"</strong><span>"Not accepted · payment state unchanged"</span></div>
                                <div class="field-work-card-actions">
                                    <button class="field-work-primary" on:click=move |_| dispatch(DemoAction::FarmerAccept)>"Simulate accepting work"</button>
                                    <button class="field-work-secondary" on:click=move |_| dispatch(DemoAction::FarmerDispute)>"Report a problem"</button>
                                </div>
                            </Show>

                            <Show when=move || workflow.get() == DemoWorkflowState::FarmerAccepted>
                                <p class="field-work-eyebrow">"STEP 5 · FARMER DECISION"</p>
                                <h3>"Acceptance recorded in this prototype"</h3>
                                <p>"The demo now records a separate farmer decision. That is still not a payment receipt or proof of independently verified repair quality."</p>
                                <div class="field-work-receipt"><strong>"FARMER ACCEPTANCE · SIMULATED"</strong><span>"Payment: not recorded"</span></div>
                                <button class="field-work-secondary" on:click=move |_| dispatch(DemoAction::Restart)>"Restart demo workflow"</button>
                            </Show>

                            <Show when=move || workflow.get() == DemoWorkflowState::Disputed>
                                <p class="field-work-eyebrow">"STEP 5 · DISPUTE"</p>
                                <h3>"Problem reported — unresolved"</h3>
                                <p>"The report is preserved in this local demo. No one is labelled at fault and no automatic penalty is applied."</p>
                                <div class="field-work-receipt"><strong>"REWORK REQUEST · SIMULATED"</strong><span>"Resolution: pending · payment: not recorded"</span></div>
                                <button class="field-work-primary" on:click=move |_| dispatch(DemoAction::Restart)>"Restart demo workflow"</button>
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

#[cfg(test)]
mod tests {
    use super::{DemoAction as A, DemoWorkflowState as S, transition};

    #[test]
    fn every_state_action_pair_matches_the_explicit_transition_allowlist() {
        let states = [
            S::Empty,
            S::LocalRequest,
            S::IncompleteQuote,
            S::TermsRequested,
            S::CompleteQuote,
            S::QuoteAccepted,
            S::BookingConfirmed,
            S::ProviderReportedComplete,
            S::FarmerAccepted,
            S::Disputed,
        ];
        let actions = [
            A::SaveRequest,
            A::ReviewQuote,
            A::RequestCompleteTerms,
            A::ReceiveCompleteQuote,
            A::AcceptQuote,
            A::ConfirmBooking,
            A::ReportProviderComplete,
            A::FarmerAccept,
            A::FarmerDispute,
            A::ReturnToRequest,
            A::DeclineQuote,
            A::Restart,
        ];

        for state in states {
            for action in actions {
                let expected = match (state, action) {
                    (S::Empty, A::SaveRequest) => S::LocalRequest,
                    (S::LocalRequest, A::ReviewQuote) => S::IncompleteQuote,
                    (S::IncompleteQuote, A::RequestCompleteTerms) => S::TermsRequested,
                    (S::TermsRequested, A::ReceiveCompleteQuote) => S::CompleteQuote,
                    (S::CompleteQuote, A::AcceptQuote) => S::QuoteAccepted,
                    (S::QuoteAccepted, A::ConfirmBooking) => S::BookingConfirmed,
                    (S::BookingConfirmed, A::ReportProviderComplete) => S::ProviderReportedComplete,
                    (S::ProviderReportedComplete, A::FarmerAccept) => S::FarmerAccepted,
                    (S::ProviderReportedComplete, A::FarmerDispute) => S::Disputed,
                    (S::IncompleteQuote, A::ReturnToRequest) => S::LocalRequest,
                    (S::CompleteQuote, A::DeclineQuote) => S::LocalRequest,
                    (_, A::Restart) => S::Empty,
                    _ => state,
                };
                assert_eq!(
                    transition(state, action),
                    expected,
                    "unexpected transition from {state:?} via {action:?}"
                );
            }
        }
    }

    #[test]
    fn incomplete_quote_cannot_be_accepted() {
        assert_eq!(
            transition(S::IncompleteQuote, A::AcceptQuote),
            S::IncompleteQuote
        );
        assert_eq!(
            transition(S::TermsRequested, A::AcceptQuote),
            S::TermsRequested
        );
    }

    #[test]
    fn quote_acceptance_and_booking_confirmation_are_distinct_states() {
        assert_eq!(
            transition(S::CompleteQuote, A::AcceptQuote),
            S::QuoteAccepted
        );
        assert_eq!(
            transition(S::QuoteAccepted, A::ConfirmBooking),
            S::BookingConfirmed
        );
        assert_ne!(S::QuoteAccepted, S::BookingConfirmed);
    }

    #[test]
    fn booking_is_required_before_provider_completion() {
        assert_eq!(
            transition(S::QuoteAccepted, A::ReportProviderComplete),
            S::QuoteAccepted
        );
    }

    #[test]
    fn provider_completion_is_not_farmer_acceptance() {
        assert_eq!(
            transition(S::BookingConfirmed, A::ReportProviderComplete),
            S::ProviderReportedComplete
        );
        assert_eq!(
            transition(S::ProviderReportedComplete, A::FarmerAccept),
            S::FarmerAccepted
        );
        assert_ne!(S::ProviderReportedComplete, S::FarmerAccepted);
    }

    #[test]
    fn dispute_does_not_mean_acceptance_or_resolution() {
        assert_eq!(
            transition(S::ProviderReportedComplete, A::FarmerDispute),
            S::Disputed
        );
        assert_ne!(S::Disputed, S::FarmerAccepted);
    }

    #[test]
    fn invalid_action_does_not_advance_and_restart_is_explicit() {
        assert_eq!(transition(S::Empty, A::ConfirmBooking), S::Empty);
        assert_eq!(transition(S::Disputed, A::Restart), S::Empty);
    }
}
