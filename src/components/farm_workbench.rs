// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Farmer-first workbench concept. All records and labels are illustrative.
//! This route is not connected to a farm, sensor, Symthaea, Mycelix, or live
//! Sol Atlas data source; interactions are local demo state only.

use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn FarmWorkbench() -> impl IntoView {
    let tab = RwSignal::new(0usize);
    let water_status = RwSignal::new(0u8); // 0 unknown, 1 observed OK, 2 issue observed, 3 unable to inspect
    let paddock = RwSignal::new(1usize);

    view! {
        <main class="farm-workbench">
            <header class="farm-topbar">
                <A href="/" attr:class="farm-back">"← Sol Atlas"</A>
                <div class="farm-brand"><span class="farm-dot"></span>"FIELD WORKBENCH"</div>
                <span class="farm-demo-tag">"CONCEPT · DEMO DATA"</span>
            </header>

            <section class="farm-hero">
                <p class="farm-eyebrow">"FARM OPERATIONS / HUMAN IN CONTROL"</p>
                <h1>"Make the next good decision obvious."</h1>
                <p class="farm-hero-copy">"A practical workspace for land, livestock, jobs, and costs — built around how a farm actually runs, not around an AI model."</p>
                <div class="farm-trust-strip">
                    <span><strong>"Example farm"</strong>" · all values are illustrative"</span>
                    <span>"No live sensors, diagnosis, or remote control"</span>
                </div>
            </section>

            <section class="farm-workspace" aria-label="Farm workbench concept">
                <nav class="farm-tabs" role="group" aria-label="Farm workspace sections">
                    <button class="farm-tab" class:active=move || tab.get() == 0 aria-pressed=move || tab.get() == 0 on:click=move |_| tab.set(0)>"Today"</button>
                    <button class="farm-tab" class:active=move || tab.get() == 1 aria-pressed=move || tab.get() == 1 on:click=move |_| tab.set(1)>"Land & grazing"</button>
                    <button class="farm-tab" class:active=move || tab.get() == 2 aria-pressed=move || tab.get() == 2 on:click=move |_| tab.set(2)>"Herd"</button>
                    <button class="farm-tab" class:active=move || tab.get() == 3 aria-pressed=move || tab.get() == 3 on:click=move |_| tab.set(3)>"Costs & outcomes"</button>
                </nav>

                {move || match tab.get() {
                    0 => view! {
                        <section class="farm-panel" aria-label="Today's work">
                            <div class="farm-section-heading">
                                <div><p class="farm-eyebrow">"DAILY BRIEF"</p><h2>"What needs attention?"</h2></div>
                                <span class="farm-chip">"3 example jobs"</span>
                            </div>
                            <div class="farm-summary-grid">
                                <article class="farm-summary-card"><span>"Open jobs"</span><strong>"03"</strong><small>"Illustrative queue"</small></article>
                                <article class="farm-summary-card"><span>"Data freshness"</span><strong>"Unknown"</strong><small>"No farm data connected"</small></article>
                                <article class="farm-summary-card"><span>"Decision status"</span><strong>"Your call"</strong><small>"No autonomous actions"</small></article>
                            </div>
                            <div class="farm-work-list">
                                <article class="farm-work-card" class:done=move || water_status.get() == 1 class:attention=move || water_status.get() == 2>
                                    <div class="farm-work-icon">"W"</div>
                                    <div class="farm-work-copy">
                                        <div class="farm-work-meta"><span>"WATER"</span><span class="farm-priority">"FIELD CHECK"</span></div>
                                        <h3>"Confirm trough flow"</h3>
                                        <p>"West paddock · Walk the line and record a quick observation before the next stock move."</p>
                                        <div class="farm-work-foot">
                                            <span class="farm-record-state" role="status">{move || match water_status.get() {
                                                1 => "Demo only: flow reported OK",
                                                2 => "Demo only: problem reported",
                                                3 => "Demo only: unable to inspect",
                                                _ => "Not recorded",
                                            }}</span>
                                            <div class="farm-record-actions" role="group" aria-label="Water check result, demo only">
                                                <button class="farm-action" class:selected=move || water_status.get() == 1 aria-pressed=move || water_status.get() == 1 on:click=move |_| water_status.set(1)>"Flow OK"</button>
                                                <button class="farm-action issue" class:selected=move || water_status.get() == 2 aria-pressed=move || water_status.get() == 2 on:click=move |_| water_status.set(2)>"Problem found"</button>
                                                <button class="farm-action secondary" class:selected=move || water_status.get() == 3 aria-pressed=move || water_status.get() == 3 on:click=move |_| water_status.set(3)>"Unable to inspect"</button>
                                                {move || (water_status.get() != 0).then(|| view! {
                                                    <button class="farm-action secondary" on:click=move |_| water_status.set(0)>"Clear demo result"</button>
                                                })}
                                            </div>
                                        </div>
                                    </div>
                                </article>
                                <article class="farm-work-card">
                                    <div class="farm-work-icon land">"L"</div>
                                    <div class="farm-work-copy">
                                        <div class="farm-work-meta"><span>"GRAZING"</span><span class="farm-chip">"REVIEW"</span></div>
                                        <h3>"Review the next paddock"</h3>
                                        <p>"North block → East block · Compare forage, recovery time, water access, and stock needs before deciding."</p>
                                        <div class="farm-work-foot"><span class="farm-muted">"Inputs required before a recommendation"</span><button class="farm-action secondary" on:click=move |_| tab.set(1)>"Open land view →"</button></div>
                                    </div>
                                </article>
                                <article class="farm-work-card">
                                    <div class="farm-work-icon herd">"H"</div>
                                    <div class="farm-work-copy">
                                        <div class="farm-work-meta"><span>"LIVESTOCK"</span><span class="farm-chip">"OBSERVE"</span></div>
                                        <h3>"Review herd observations"</h3>
                                        <p>"Keep individual observations and group context together. This concept does not diagnose animals."</p>
                                        <div class="farm-work-foot"><span class="farm-muted">"No animal records connected"</span><button class="farm-action secondary" on:click=move |_| tab.set(2)>"Open herd view →"</button></div>
                                    </div>
                                </article>
                            </div>
                            <p class="farm-footnote">"Demo interactions are not saved or synced. A real deployment must work offline, preserve an audit trail, and label stale or missing data."</p>
                        </section>
                    }.into_any(),
                    1 => view! {
                        <section class="farm-panel" aria-label="Land and grazing">
                            <div class="farm-section-heading"><div><p class="farm-eyebrow">"LAND & GRAZING"</p><h2>"Start with the place."</h2></div><span class="farm-chip">"Schematic only"</span></div>
                            <div class="farm-land-layout">
                                <article class="farm-map-card">
                                    <div class="farm-card-heading"><div><strong>"Paddock sketch"</strong><small>"Not geographic or live data"</small></div><span>"N ↑"</span></div>
                                    <div class="farm-paddock-map" role="group" aria-label="Select an example paddock">
                                        <button class="farm-paddock" class:selected=move || paddock.get() == 1 aria-pressed=move || paddock.get() == 1 on:click=move |_| paddock.set(1)><strong>"North 1"</strong><small>"Resting"</small></button>
                                        <button class="farm-paddock" class:selected=move || paddock.get() == 2 aria-pressed=move || paddock.get() == 2 on:click=move |_| paddock.set(2)><strong>"North 2"</strong><small>"Review"</small></button>
                                        <button class="farm-paddock" class:selected=move || paddock.get() == 3 aria-pressed=move || paddock.get() == 3 on:click=move |_| paddock.set(3)><strong>"East 1"</strong><small>"Availability?"</small></button>
                                        <button class="farm-paddock" class:selected=move || paddock.get() == 4 aria-pressed=move || paddock.get() == 4 on:click=move |_| paddock.set(4)><strong>"West 1"</strong><small>"Water check"</small></button>
                                        <button class="farm-paddock wide" class:selected=move || paddock.get() == 5 aria-pressed=move || paddock.get() == 5 on:click=move |_| paddock.set(5)><strong>"South reserve"</strong><small>"Verify access rules"</small></button>
                                    </div>
                                    <p class="farm-footnote">"A live map needs farmer-verified boundaries, source provenance, and a way to correct errors."</p>
                                </article>
                                <aside class="farm-detail-card">
                                    <p class="farm-eyebrow">"SELECTED PLACE"</p>
                                    <h3>{move || match paddock.get() { 1 => "North 1", 2 => "North 2", 3 => "East 1", 4 => "West 1", _ => "South reserve" }}</h3>
                                    <p>{move || match paddock.get() {
                                        1 => "Example: resting. Check last grazing date and actual recovery before allocating stock.",
                                        2 => "Example: review. Compare forage, water access, and the current mob plan.",
                                        3 => "Example: availability unverified. Confirm conditions in the field.",
                                        4 => "Example: water check due. Verify trough and line condition on site.",
                                        _ => "Example: protected area. Confirm conservation and landholder rules before planning work.",
                                    }}</p>
                                    <dl class="farm-detail-list">
                                        <div><dt>"Last observation"</dt><dd>"Not connected"</dd></div>
                                        <div><dt>"Recovery estimate"</dt><dd>"Not calculated"</dd></div>
                                        <div><dt>"Source freshness"</dt><dd>"Unknown"</dd></div>
                                    </dl>
                                    <div class="farm-notice">"A map should support judgment, not silently replace it."</div>
                                </aside>
                            </div>
                        </section>
                    }.into_any(),
                    2 => view! {
                        <section class="farm-panel" aria-label="Herd">
                            <div class="farm-section-heading"><div><p class="farm-eyebrow">"HERD & ANIMAL CARE"</p><h2>"Keep observations usable."</h2></div><span class="farm-chip">"Example only"</span></div>
                            <div class="farm-land-layout">
                                <article class="farm-detail-card"><p class="farm-eyebrow">"GROUP RECORD"</p><h3>"Mob 04"</h3><p>"Illustrative group label only. No real count, animal ID, location, or health information is present."</p>
                                    <dl class="farm-detail-list"><div><dt>"Group size"</dt><dd>"Not provided"</dd></div><div><dt>"Last movement"</dt><dd>"Not provided"</dd></div><div><dt>"Water / feed"</dt><dd>"Not assessed"</dd></div></dl>
                                </article>
                                <article class="farm-detail-card"><p class="farm-eyebrow">"OBSERVATION DESIGN"</p><h3>"Record what was actually seen."</h3>
                                    <ul class="farm-principles"><li>"Who observed it and when"</li><li>"Animal or mob ID, when known"</li><li>"Observation separate from interpretation"</li><li>"Images only with explicit permission"</li><li>"Clear escalation to a qualified veterinarian"</li></ul>
                                    <div class="farm-notice">"No automated diagnosis or treatment advice in this prototype."</div>
                                </article>
                            </div>
                        </section>
                    }.into_any(),
                    _ => view! {
                        <section class="farm-panel" aria-label="Costs and outcomes">
                            <div class="farm-section-heading"><div><p class="farm-eyebrow">"ECONOMICS & EVIDENCE"</p><h2>"Show the trade-off, not a magic score."</h2></div><span class="farm-chip">"No estimates invented"</span></div>
                            <div class="farm-economics-grid">
                                <article class="farm-economics-card"><span>"Input cost"</span><strong>"Not connected"</strong><p>"Track fuel, feed, water, labour, and maintenance in the farm's own units."</p></article>
                                <article class="farm-economics-card"><span>"Operational time"</span><strong>"Baseline needed"</strong><p>"Compare time spent on the same job before and after a workflow change."</p></article>
                                <article class="farm-economics-card"><span>"Land / herd outcome"</span><strong>"Baseline needed"</strong><p>"Use farm-selected measures such as recovery days, weight gain, yield, or condition."</p></article>
                                <article class="farm-economics-card"><span>"Recommendation evidence"</span><strong>"Not available"</strong><p>"Show sources, assumptions, uncertainty, and what missing observation would change the result."</p></article>
                            </div>
                            <article class="farm-method-card"><h3>"The pilot rule"</h3><p>"Agree on one outcome with the farmer, capture a baseline, compare against a credible reference, include costs and adverse effects, and publish the method. No universal yield or profit claim is assumed."</p></article>
                        </section>
                    }.into_any(),
                }}
            </section>
            <footer class="farm-footer"><p>"Respect the operator's expertise, farm data rights, and reality on the ground."</p><span>"Prototype route · no live farm integration"</span></footer>
        </main>
    }
}
