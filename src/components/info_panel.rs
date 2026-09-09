// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! The dossier: leads with one hero fact, then quiet secondary rows.
//! Natural events surface source evidence instead of assuming every record is
//! an observed snapshot with a generic magnitude.

use leptos::prelude::*;

use crate::data::types::SelectedItem;
use crate::state::data_state::DataState;
use crate::state::globe_state::GlobeState;

fn hero(value: String, unit: &'static str) -> impl IntoView {
    view! {
        <div class="hero-stat">
            <span class="hero-value">{value}</span>
            <span class="hero-unit">{unit}</span>
        </div>
    }
}

fn hero_dynamic(value: String, unit: String) -> impl IntoView {
    view! {
        <div class="hero-stat">
            <span class="hero-value">{value}</span>
            <span class="hero-unit">{unit}</span>
        </div>
    }
}

fn row(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="stat-row">
            <span class="stat-label">{label}</span>
            <span class="stat-value">{value}</span>
        </div>
    }
}

#[component]
pub fn InfoPanel() -> impl IntoView {
    let globe_state = expect_context::<GlobeState>();
    let data_state = expect_context::<DataState>();

    let visible = move || globe_state.selected.read().is_some();
    let close = move |_| globe_state.selected.set(None);

    view! {
        <div class=move || if visible() { "info-panel" } else { "info-panel hidden" }>
            <button class="close-btn" aria-label="Close" on:click=close>{"\u{2715}"}</button>
            {move || {
                let selected = globe_state.selected.read();
                match selected.as_ref() {
                    Some(SelectedItem::GeothermalNode(n)) => view! {
                        <div>
                            <h2>{n.name.clone()}</h2>
                            <div class="subtitle">"Geothermal Node · scenario"</div>
                            {hero(format!("{}", n.capacity_mw), "MW")}
                            {row("Region", n.region.clone())}
                            {row("Temperature", format!("{}\u{00b0}C", n.temperature_c))}
                            {row("Type", n.node_type.clone())}
                            {row("Status", n.status.clone())}
                        </div>
                    }.into_any(),
                    Some(SelectedItem::MaglevCorridor(c)) => view! {
                        <div>
                            <h2>{c.name.clone()}</h2>
                            <div class="subtitle">"Maglev Corridor · scenario"</div>
                            {hero(format!("{:.0}", c.distance_km), "km")}
                            {row("From", c.from_name.clone())}
                            {row("To", c.to_name.clone())}
                            {row("Travel Time", format!("{:.0} min", c.travel_time_min))}
                            {row("Type", if c.submarine { "Submarine" } else { "Land" }.to_string())}
                            {row("Seismic Risk", c.seismic_risk.clone())}
                            {row("Cost", format!("${:.0}B", c.cost_billion_usd))}
                            {row("Capacity", format!("{} pax/hr", c.capacity_pax_hr))}
                            {row("Geothermal", if c.geothermal_powered { "Yes" } else { "No" }.to_string())}
                        </div>
                    }.into_any(),
                    Some(SelectedItem::ResontiaVault(v)) => view! {
                        <div>
                            <h2>{v.name.clone()}</h2>
                            <div class="subtitle">"Resontia Vault · scenario"</div>
                            {hero(format!("{}", v.capacity_people), "people")}
                            {row("Heat Rejection", format!("{:.0} MW", v.heat_rejection_mw))}
                            {row("Blast Doors", v.blast_doors.to_string())}
                            {row("Status", v.status.clone())}
                            {row("Terra Lumina", v.terra_lumina_id.clone())}
                        </div>
                    }.into_any(),
                    Some(SelectedItem::TerraLuminaSite(s)) => view! {
                        <div>
                            <h2>{s.name.clone()}</h2>
                            <div class="subtitle">{format!("{} · Tier: {} · scenario", s.country, s.tier)}</div>
                            {hero(format!("{:.1}", s.total_renewable_gw), "GW renewable")}
                            {row("Geothermal", format!("{:.1} GW", s.geothermal_gw))}
                            {row("Solar", format!("{:.1} GW", s.solar_gw))}
                            {row("Hydro", format!("{:.1} GW", s.hydro_gw))}
                            {row("Phase 1", format!("\u{20ac}{:.0}B", s.phase1_billion_eur))}
                            {row("Total Cost", format!("\u{20ac}{:.0}B", s.total_billion_eur))}
                            {row("IRR", format!("{:.1}%", s.irr_percent))}
                        </div>
                    }.into_any(),
                    Some(SelectedItem::Site(s)) => view! {
                        <div>
                            <h2>{s.name.clone()}</h2>
                            <div class="subtitle">{format!("{:?} \u{00b7} {}", s.energy_type, s.country)}</div>
                            {hero(format!("{:.0}", s.capacity_mw), "MW")}
                            {row("Status", s.status.clone())}
                        </div>
                    }.into_any(),
                    Some(SelectedItem::EarthRegion(r)) => view! {
                        <div>
                            <h2>{r.name.clone()}</h2>
                            <div class="subtitle">"Earth Region · curated estimates"</div>
                            {hero(format!("{:.0}M", r.population_m), "people")}
                            {row("GDP/capita", format!("${:.0}", r.gdp_per_capita))}
                            {row("Education", format!("{:.0}%", r.education_index * 100.0))}
                            {row("Phi (mean)", format!("{:.2}", r.phi_mean))}
                            {row("Climate Risk", format!("{:.0}%", r.climate_vulnerability * 100.0))}
                            {row("Infrastructure", format!("{:.0}%", r.infrastructure * 100.0))}
                            {row("Spaceport", if r.spaceport { "Yes" } else { "No" }.to_string())}
                        </div>
                    }.into_any(),
                    Some(SelectedItem::NuclearSite(n)) => view! {
                        <div>
                            <h2>{n.name.clone()}</h2>
                            <div class="subtitle">{format!("{} \u{00b7} {}", n.reactor_type.label(), n.country)}</div>
                            {hero(format!("{:.0}", n.capacity_mw), "MW")}
                            {row("Operator", n.operator.clone())}
                            {row("Status", n.status.clone())}
                            {row("Commissioned", n.commission_year.to_string())}
                        </div>
                    }.into_any(),
                    Some(SelectedItem::FossilDeposit(d)) => {
                        let eroi = sol_atlas_core::economics::compute_eroi(d);
                        view! {
                            <div>
                                <h2>{d.name.clone()}</h2>
                                <div class="subtitle">{format!("{:?} \u{00b7} {}", d.fuel_type, d.country)}</div>
                                {match eroi {
                                    Some(e) => {
                                        let tier = sol_atlas_core::economics::EroiTier::from_eroi(e);
                                        hero(format!("{:.0}:1", e), tier.label()).into_any()
                                    }
                                    None => hero(format!("{:.0}", d.proven_reserves_mboe), "Mboe reserves").into_any(),
                                }}
                                {row("Reserves", format!("{:.0} Mboe", d.proven_reserves_mboe))}
                                {row("Production", format!("{:.0} Mboe/yr", d.annual_production_mboe))}
                                {row("Status", d.status.clone())}
                                {row("Discovered", d.discovery_year.to_string())}
                            </div>
                        }.into_any()
                    }
                    Some(SelectedItem::NaturalEvent(e)) => {
                        if let Some(record) = data_state.record_for_natural_event(e) {
                            let evidence = sol_atlas_core::summarize_event_evidence(&record);
                            let primary = match (
                                evidence.measurement_value.clone(),
                                evidence.measurement_unit.clone(),
                            ) {
                                (Some(value), Some(unit)) => hero_dynamic(value, unit).into_any(),
                                _ => hero_dynamic(
                                    evidence.class.to_string(),
                                    "evidence class".to_string(),
                                )
                                .into_any(),
                            };
                            view! {
                                <div>
                                    <h2>{e.name.clone()}</h2>
                                    <div class="subtitle">{format!("{:?} · {}", e.event_type, evidence.class)}</div>
                                    {primary}
                                    {row("Source", evidence.source)}
                                    {row("Source time", evidence.source_time)}
                                    {row("Uncertainty", evidence.uncertainty)}
                                    {row("Resource", evidence.resource)}
                                    {row("Evidence ID", evidence.observation_id)}
                                </div>
                            }
                            .into_any()
                        } else {
                            view! {
                                <div>
                                    <h2>{e.name.clone()}</h2>
                                    <div class="subtitle">{format!("{:?} · provenance unavailable", e.event_type)}</div>
                                    {hero(format!("{:.1}", e.magnitude), "legacy display value")}
                                    {row("Evidence", "No evidence record is bound to this event".to_string())}
                                </div>
                            }
                            .into_any()
                        }
                    }
                    Some(SelectedItem::MajorCity(c)) => view! {
                        <div>
                            <h2>{c.name.clone()}</h2>
                            <div class="subtitle">{format!("{} \u{00b7} curated", c.country)}</div>
                            {hero(c.population.to_string(), "people")}
                        </div>
                    }.into_any(),
                    Some(SelectedItem::Chokepoint(p)) => view! {
                        <div>
                            <h2>{p.name.clone()}</h2>
                            <div class="subtitle">{format!("{} \u{00b7} curated", p.chokepoint_type)}</div>
                            {hero(format!("{:.0}M", p.daily_barrels_m), "barrels/day")}
                        </div>
                    }.into_any(),
                    Some(SelectedItem::CriticalInfrastructure(i)) => view! {
                        <div>
                            <h2>{i.name.clone()}</h2>
                            <div class="subtitle">{format!("{} \u{00b7} curated", i.infra_type)}</div>
                            {hero(format!("{:.0}%", i.global_share * 100.0), "global share")}
                            {row("Risk", i.risk.clone())}
                        </div>
                    }.into_any(),
                    Some(SelectedItem::Confluence(cell)) => {
                        let rows: Vec<_> = cell
                            .layers
                            .iter()
                            .map(|layer| {
                                row(
                                    layer.label(),
                                    sol_atlas_core::audited_layer_provenance(*layer)
                                        .kind
                                        .label()
                                        .to_string(),
                                )
                            })
                            .collect();
                        view! {
                            <div>
                                <h2>"Confluence"</h2>
                                <div class="subtitle">
                                    "Derived signal · where eligible real systems co-locate · not a risk score"
                                </div>
                                {hero(cell.layers.len().to_string(), "real systems here")}
                                {row("Entities in this cell", cell.entity_count.to_string())}
                                {rows}
                            </div>
                        }
                        .into_any()
                    }
                    None => view! { <div /> }.into_any(),
                }
            }}
        </div>
    }
}
