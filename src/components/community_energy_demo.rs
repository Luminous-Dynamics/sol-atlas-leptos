// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Small public-facing civic energy demo: explicit synthetic scenario,
//! deterministic Rust simulation, independent accounting verification,
//! and a separately-labelled illustrative community vote worksheet.

use leptos::prelude::*;
use leptos_router::components::A;

use sol_atlas_core::community_energy_demo::{
    simulate, verify, BatteryConfiguration, EnergyIntervalInput, SimulationResult,
    VerificationReport, GovernanceVoteFixture, SCENARIO_INPUTS, CASE_ID, EVIDENCE_CLASS,
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
    let selected_run = move || {
        if selected_candidate.get() { candidate.clone() } else { baseline.clone() }
    };
    let selected_report = move || {
        if selected_candidate.get() { candidate_report.clone() } else { baseline_report.clone() }
    };
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
                        class:community-energy-choice-active=move || !selected_candidate.get()
                        class="community-energy-choice"
                        on:click=move |_| selected_candidate.set(false)
                    >
                        <span class="community-energy-choice-label">"BASELINE"</span>
                        <strong>"Existing equipment"</strong>
                        <span>"6 kWh battery · 2 kW charge/discharge"</span>
                    </button>
                    <button
                        class:community-energy-choice-active=move || selected_candidate.get()
                        class="community-energy-choice"
                        on:click=move |_| selected_candidate.set(true)
                    >
                        <span class="community-energy-choice-label">"SENSITIVITY CANDIDATE"</span>
                        <strong>"Larger reserve"</strong>
                        <span>"20 kWh battery · 8 kW charge/discharge"</span>
                    </button>
                </div>

                <p class="community-energy-candidate-warning">
                    "The larger battery is an intentionally oversized sensitivity case, not a priced recommendation. Cost, degradation, inverter behavior, network constraints, and real load calibration are outside this model."
                </p>

                <div class="community-energy-status-row">
                    <div class="community-energy-status-card">
                        <span class="community-energy-label">"ACCOUNTING VERIFICATION"</span>
                        <strong class="community-energy-status-pass">{move || if selected_report().accounting_consistent { "PASS" } else { "FAIL" }}</strong>
                        <p>"Balances, storage transitions, grid boundaries, and reported totals were rechecked from interval rows."</p>
                    </div>
                    <div class="community-energy-status-card">
                        <span class="community-energy-label">"CRITICAL-LOAD POLICY"</span>
                        <strong class:community-energy-status-pass=move || selected_report().critical_service_limit_satisfied
                                class:community-energy-status-fail=move || !selected_report().critical_service_limit_satisfied>
                            {move || if selected_report().critical_service_limit_satisfied { "PASS" } else { "FAIL" }}
                        </strong>
                        <p>"Hard requirement: no critical-load energy may remain unserved during the frozen outage scenario."</p>
                    </div>
                </div>

                <div class="community-energy-metrics">
                    <div class="community-energy-metric">
                        <span>"Total load"</span>
                        <strong>{move || format!("{:.2} kWh", selected_run().totals.total_load_kwh)}</strong>
                    </div>
                    <div class="community-energy-metric">
                        <span>"Renewable input"</span>
                        <strong>{move || format!("{:.2} kWh", selected_run().totals.total_renewable_generation_kwh)}</strong>
                    </div>
                    <div class="community-energy-metric">
                        <span>"Grid import"</span>
                        <strong>{move || format!("{:.2} kWh", selected_run().totals.total_grid_import_kwh)}</strong>
                    </div>
                    <div class="community-energy-metric">
                        <span>"Unserved load"</span>
                        <strong>{move || format!("{:.2} kWh", selected_run().totals.total_unserved_load_kwh)}</strong>
                    </div>
                    <div class="community-energy-metric">
                        <span>"Critical unserved"</span>
                        <strong>{move || format!("{:.2} kWh", selected_run().totals.total_critical_unserved_kwh)}</strong>
                    </div>
                    <div class="community-energy-metric">
                        <span>"Noncritical unserved"</span>
                        <strong>{move || format!("{:.2} kWh", selected_run().totals.total_noncritical_unserved_kwh)}</strong>
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
                                each=move || selected_run().intervals
                                key=|row| row.input.interval
                                children=move |row| view! {
                                    <tr class:community-energy-outage=move || !row.input.grid_available>
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
                        <h2>"Record the community’s choice"</h2>
                        <p>"Illustrative vote fixture with a visible minority—not an executed Mycelix governance transaction."</p>
                    </div>
                </div>

                <div class="community-energy-vote">
                    <div class="community-energy-vote-result">
                        <span class="community-energy-label">"ILLUSTRATIVE DECISION"</span>
                        <strong>"{if vote.passed() { "PASSED" } else { "NOT PASSED" }}"</strong>
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
                    <p>"It demonstrates a possible interface between a scenario result and a recorded decision. No vote was sent to Holochain; there is no authentication, member eligibility verification, delegation, appeal workflow, or governance execution in this demo."</p>
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

#[allow(dead_code)]
fn _types_are_explicitly_bound(_input: EnergyIntervalInput, _run: SimulationResult, _report: VerificationReport) {}
