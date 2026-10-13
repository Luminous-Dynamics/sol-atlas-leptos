// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Small public-facing civic energy demo: explicit synthetic scenario,
//! deterministic Rust simulation, independent accounting verification,
//! and a separately-labelled illustrative community vote worksheet.

use leptos::prelude::*;
use leptos_router::components::A;

use sol_atlas_core::community_energy_demo::{
    simulate, verify, BatteryConfiguration, GovernanceVoteFixture, SCENARIO_INPUTS, CASE_ID,
    EVIDENCE_CLASS,
};

#[component]
pub fn CommunityEnergyDemo() -> impl IntoView {
    let baseline = simulate(BatteryConfiguration::BASELINE)
        .expect("fixed community-energy baseline fixture must be valid");
    let candidate = simulate(BatteryConfiguration::RESILIENCE_CANDIDATE)
        .expect("fixed community-energy sensitivity fixture must be valid");

    let baseline_report = verify(BatteryConfiguration::BASELINE, &SCENARIO_INPUTS, &baseline)
        .expect("baseline simulation output must satisfy accounting invariants");
    let candidate_report = verify(
        BatteryConfiguration::RESILIENCE_CANDIDATE,
        &SCENARIO_INPUTS,
        &candidate,
    )
    .expect("candidate simulation output must satisfy accounting invariants");

    let selected_candidate = RwSignal::new(true);
    // Copy the values used by individual reactive closures. This avoids moving
    // one non-Copy helper closure into several UI closures.
    let baseline_accounting = baseline_report.accounting_consistent;
    let candidate_accounting = candidate_report.accounting_consistent;
    let baseline_critical_pass = baseline_report.critical_service_limit_satisfied;
    let candidate_critical_pass = candidate_report.critical_service_limit_satisfied;
    let baseline_load = baseline.totals.total_load_kwh;
    let candidate_load = candidate.totals.total_load_kwh;
    let baseline_generation = baseline.totals.total_renewable_generation_kwh;
    let candidate_generation = candidate.totals.total_renewable_generation_kwh;
    let baseline_import = baseline.totals.total_grid_import_kwh;
    let candidate_import = candidate.totals.total_grid_import_kwh;
    let baseline_unserved = baseline.totals.total_unserved_load_kwh;
    let candidate_unserved = candidate.totals.total_unserved_load_kwh;
    let baseline_critical_unserved = baseline.totals.total_critical_unserved_kwh;
    let candidate_critical_unserved = candidate.totals.total_critical_unserved_kwh;
    let baseline_noncritical_unserved = baseline.totals.total_noncritical_unserved_kwh;
    let candidate_noncritical_unserved = candidate.totals.total_noncritical_unserved_kwh;
    let baseline_rows = baseline.intervals.clone();
    let candidate_rows = candidate.intervals.clone();
    let vote = GovernanceVoteFixture::NETWORK_STATE_DEMO;

    view! {
        <div class="community-energy-page">
            <div class="community-energy-topbar">
                <A href="/">"← Back to Sol Atlas"</A>
                <span class="community-energy-kicker">"PUBLIC DEMONSTRATOR / V0"</span>
            </div>

            <header class="community-energy-hero">
                <p class="community-energy-eyebrow">"CIVIC CAPACITY · ENERGY RESILIENCE"</p>
                <h1>"One community. One outage. Two choices."</h1>
                <p class="community-energy-intro">
                    "See how a community could compare resilience options, inspect the evidence, and record a decision without giving an AI system the authority to make that decision."
                </p>
                <div class="community-energy-disclaimer" role="note">
                    <strong>"Scenario-only demo."</strong>
                    " All inputs are synthetic. This page is not live meter data, a real microgrid forecast, a financial feasibility study, an executed Mycelix vote, or a controller for physical equipment."
                </div>
            </header>

            <section class="community-energy-section">
                <div class="community-energy-section-heading">
                    <span class="community-energy-step">"01"</span>
                    <div>
                        <h2>"Choose a resilience scenario"</h2>
                        <p>"Same eight 15-minute intervals; only the battery configuration changes."</p>
                    </div>
                </div>

                <div class="community-energy-choice-row">
                    <button
                        class=move || if !selected_candidate.get() {
                            "community-energy-choice community-energy-choice-active"
                        } else {
                            "community-energy-choice"
                        }
                        on:click=move |_| selected_candidate.set(false)
                    >
                        <span class="community-energy-choice-label">"BASELINE"</span>
                        <strong>"Existing equipment"</strong>
                        <span>"6 kWh capacity · 3 kWh starting SOC · 2 kW charge/discharge"</span>
                    </button>
                    <button
                        class=move || if selected_candidate.get() {
                            "community-energy-choice community-energy-choice-active"
                        } else {
                            "community-energy-choice"
                        }
                        on:click=move |_| selected_candidate.set(true)
                    >
                        <span class="community-energy-choice-label">"SENSITIVITY CANDIDATE"</span>
                        <strong>"Larger reserve"</strong>
                        <span>"20 kWh capacity · 16 kWh starting SOC · 8 kW charge/discharge"</span>
                    </button>
                </div>

                <p class="community-energy-candidate-warning">
                    "The larger case is intentionally oversized and starts with 16 kWh already stored; the baseline starts at 3 kWh. The source and cost of precharging that reserve are not modeled, so this does not isolate battery size. Cost, degradation, inverter behavior, network constraints, and real load calibration are also outside scope."
                </p>

                <p class="community-energy-candidate-warning">
                    "The critical-first service split is an accounting assumption. This demo has no circuit-level load model, priority controls, or load-shedding controller, so it does not demonstrate that a real site could physically protect critical loads in this way."
                </p>

                <div class="community-energy-status-row">
                    <div class="community-energy-status-card">
                        <span class="community-energy-label">"ACCOUNTING VERIFICATION"</span>
                        <strong class="community-energy-status-pass">{move || if (if selected_candidate.get() { candidate_accounting } else { baseline_accounting }) { "PASS" } else { "FAIL" }}</strong>
                        <p>"Balances, storage transitions, grid boundaries, and reported totals were rechecked from interval rows."</p>
                    </div>
                    <div class="community-energy-status-card">
                        <span class="community-energy-label">"CRITICAL-LOAD POLICY"</span>
                        <strong class=move || if (if selected_candidate.get() { candidate_critical_pass } else { baseline_critical_pass }) {
                                "community-energy-status-pass"
                            } else {
                                "community-energy-status-fail"
                            }>
                            {move || if (if selected_candidate.get() { candidate_critical_pass } else { baseline_critical_pass }) { "PASS" } else { "FAIL" }}
                        </strong>
                        <p>"Modelled check: zero critical-load energy remains unserved under the declared critical-first allocation assumption. No circuit-priority controller or load-shedding hardware is represented."</p>
                    </div>
                </div>

                <div class="community-energy-metrics">
                    <div class="community-energy-metric">
                        <span>"Total load"</span>
                        <strong>{move || format!("{:.2} kWh", if selected_candidate.get() { candidate_load } else { baseline_load })}</strong>
                    </div>
                    <div class="community-energy-metric">
                        <span>"Renewable input"</span>
                        <strong>{move || format!("{:.2} kWh", if selected_candidate.get() { candidate_generation } else { baseline_generation })}</strong>
                    </div>
                    <div class="community-energy-metric">
                        <span>"Grid import"</span>
                        <strong>{move || format!("{:.2} kWh", if selected_candidate.get() { candidate_import } else { baseline_import })}</strong>
                    </div>
                    <div class="community-energy-metric">
                        <span>"Unserved load"</span>
                        <strong>{move || format!("{:.2} kWh", if selected_candidate.get() { candidate_unserved } else { baseline_unserved })}</strong>
                    </div>
                    <div class="community-energy-metric">
                        <span>"Critical unserved"</span>
                        <strong>{move || format!("{:.2} kWh", if selected_candidate.get() { candidate_critical_unserved } else { baseline_critical_unserved })}</strong>
                    </div>
                    <div class="community-energy-metric">
                        <span>"Noncritical unserved"</span>
                        <strong>{move || format!("{:.2} kWh", if selected_candidate.get() { candidate_noncritical_unserved } else { baseline_noncritical_unserved })}</strong>
                    </div>
                </div>

                <div class="community-energy-table-wrap">
                    <table class="community-energy-table">
                        <thead>
                            <tr>
                                <th>"Interval"</th>
                                <th>"Grid"</th>
                                <th>"Load"</th>
                                <th>"Renewables"</th>
                                <th>"Battery out"</th>
                                <th>"Grid import"</th>
                                <th>"Unserved"</th>
                                <th>"Critical unserved"</th>
                                <th>"Battery SOC"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || if selected_candidate.get() { candidate_rows.clone() } else { baseline_rows.clone() }
                                key=|row| row.input.interval
                                children=move |row| view! {
                                    <tr class=if row.input.grid_available { "" } else { "community-energy-outage" }>
                                        <td>{row.input.interval + 1}</td>
                                        <td>{if row.input.grid_available { "UP" } else { "OUTAGE" }}</td>
                                        <td>{format!("{:.2}", row.input.load_kwh)}</td>
                                        <td>{format!("{:.2}", row.input.renewable_generation_kwh)}</td>
                                        <td>{format!("{:.2}", row.battery_discharge_to_load_kwh)}</td>
                                        <td>{format!("{:.2}", row.grid_import_kwh)}</td>
                                        <td>{format!("{:.2}", row.unserved_load_kwh)}</td>
                                        <td>{format!("{:.2}", row.critical_unserved_kwh)}</td>
                                        <td>{format!("{:.2}", row.battery_soc_kwh)}</td>
                                    </tr>
                                }
                            />
                        </tbody>
                    </table>
                </div>

                <p class="community-energy-evidence-note">
                    <strong>"Evidence boundary."</strong>
                    " The verifier is a separate function, but it is in the same crate as the simulator and shares the frozen input fixture. This is a reproducible accounting check—not an independent organization, source authentication, or evidence of physical meter truth."
                </p>
            </section>

            <section class="community-energy-section">
                <div class="community-energy-section-heading">
                    <span class="community-energy-step">"02"</span>
                    <div>
                        <h2>"Inspect a governance tally example"</h2>
                        <p>"Pre-seeded illustration with a visible minority; this page does not accept ballots, persist a vote, or submit a Holochain transaction."</p>
                    </div>
                </div>

                <div class="community-energy-vote">
                    <div class="community-energy-vote-result">
                        <span class="community-energy-label">"ILLUSTRATIVE RULE RESULT"</span>
                        <strong>{if vote.passed() { "PASSED" } else { "NOT PASSED" }}</strong>
                        <p>"A simple one-person/one-vote rule is used only for this exhibit. Mycelix Governance describes a different, experimental MATL/stake-weighted model; this page does not claim conformance to that policy."</p>
                    </div>
                    <div class="community-energy-vote-numbers">
                        <div><span>"YES"</span><strong>{vote.yes}</strong></div>
                        <div><span>"NO"</span><strong>{vote.no}</strong></div>
                        <div><span>"ABSTAIN"</span><strong>{vote.abstain}</strong></div>
                        <div><span>"ELIGIBLE"</span><strong>{vote.eligible_members}</strong></div>
                    </div>
                    <div class="community-energy-vote-rule">
                        <span>"Declared quorum: "{vote.quorum_percent}"%"</span>
                        <span>"Declared approval: "{vote.approval_percent}"%"</span>
                        <span>"Recorded dissent: 1 member voted no."</span>
                    </div>
                </div>

                <div class="community-energy-governance-boundary">
                    <strong>"What this does—and does not—show"</strong>
                    <p>"This is a pre-seeded tally display, not a voting interface: the page accepts no ballots and persists no decision. No vote was sent to Holochain; there is no authentication, member eligibility verification, delegation, appeal workflow, or governance execution."</p>
                </div>
            </section>

            <footer class="community-energy-footer">
                <div>
                    <span class="community-energy-label">"FROZEN CASE ID"</span>
                    <code>{CASE_ID}</code>
                </div>
                <div>
                    <span class="community-energy-label">"INPUT CLASS"</span>
                    <code>{EVIDENCE_CLASS}</code>
                </div>
                <div>
                    <span class="community-energy-label">"TIME AXIS"</span>
                    <code>"8 intervals × 15 minutes"</code>
                </div>
            </footer>
        </div>
    }
}

