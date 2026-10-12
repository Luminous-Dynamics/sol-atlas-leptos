// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! The dossier: leads with one hero number (the fact you'd tell someone
//! first), then quiet secondary rows. Grows in rather than sliding a
//! rectangle.

use leptos::prelude::*;

use crate::data::types::SelectedItem;
use crate::state::data_state::DataState;
use crate::state::globe_state::GlobeState;

const NEARBY_INFRASTRUCTURE_RADIUS_KM: f64 = 100.0;
const MAX_NEARBY_INFRASTRUCTURE: usize = 4;

/// Hero stat block: the single number that defines the selected thing.
fn hero(value: String, unit: &'static str) -> impl IntoView {
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

fn format_timestamp_ms(timestamp_ms: i64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(timestamp_ms as f64));
    if !date.get_time().is_finite() {
        return "Invalid timestamp".to_string();
    }
    date.to_iso_string()
        .as_string()
        .unwrap_or_else(|| "Invalid timestamp".to_string())
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
                        // Geographic proximity is context only. The curated
                        // infrastructure inventory is neither complete nor an
                        // assessment of whether this event caused damage.
                        let infrastructure = data_state.critical_infrastructure.get();
                        let nearby = sol_atlas_core::hazard_context::nearby_infrastructure(
                            e,
                            &infrastructure,
                            NEARBY_INFRASTRUCTURE_RADIUS_KM,
                            MAX_NEARBY_INFRASTRUCTURE,
                        );
                        let nearby_count = nearby.len();
                        let source_label = e.source_name.clone().unwrap_or_else(|| {
                            if e.source_id.is_some() {
                                "Source-linked record".to_string()
                            } else {
                                "Source not identified".to_string()
                            }
                        });
                        let provenance_label = e
                            .data_kind
                            .map(|kind| kind.label().to_string())
                            .unwrap_or_else(|| "unclassified".to_string());
                        let measurement = match e.data_kind {
                            Some(sol_atlas_core::types::DataKind::Observed)
                                if e.source_name.as_deref() == Some("NASA EONET") =>
                            {
                                let label = match (
                                    e.magnitude_description.as_deref(),
                                    e.magnitude_unit.as_deref(),
                                ) {
                                    (Some(description), Some(unit)) => {
                                        format!("{description} ({:.1} {unit})", e.magnitude)
                                    }
                                    (Some(description), None) => description.to_string(),
                                    (None, Some(unit)) => format!("{:.1} {unit}", e.magnitude),
                                    (None, None) => {
                                        "No standardized measure across EONET categories".to_string()
                                    }
                                };
                                row("Reported measure", label).into_any()
                            }
                            Some(sol_atlas_core::types::DataKind::Observed)
                                if e.source_name.as_deref() == Some("USGS ComCat") =>
                            {
                                hero(format!("{:.1}", e.magnitude), "magnitude").into_any()
                            }
                            Some(sol_atlas_core::types::DataKind::Synthetic) => row(
                                "Measurement",
                                "Synthetic demo value; not a real-world observation".to_string(),
                            )
                            .into_any(),
                            Some(sol_atlas_core::types::DataKind::Mixed) => row(
                                "Measurement",
                                "Mixed provenance; value is not treated as a verified measurement".to_string(),
                            )
                            .into_any(),
                            Some(sol_atlas_core::types::DataKind::Curated) => row(
                                "Measurement",
                                "No source-validated measurement is available".to_string(),
                            )
                            .into_any(),
                            Some(sol_atlas_core::types::DataKind::Observed) => row(
                                "Measurement",
                                "No supported source-specific measure; numeric value suppressed".to_string(),
                            )
                            .into_any(),
                            _ => row(
                                "Measurement",
                                "Unknown provenance; value is not shown as verified".to_string(),
                            )
                            .into_any(),
                        };
                        let nearby_rows: Vec<_> = if nearby.is_empty() {
                            vec![view! {
                                <div class="stat-row">
                                    <span class="stat-label">"No mapped assets nearby"</span>
                                    <span class="stat-value">"None within 100 km"</span>
                                </div>
                            }.into_any()]
                        } else {
                            nearby.into_iter().map(|entry| {
                                let name = entry.asset.name.clone();
                                let detail = format!(
                                    "{} · {:.1} km",
                                    entry.asset.infra_type,
                                    entry.distance_km
                                );
                                view! {
                                    <div class="stat-row nearby-context-row">
                                        <span class="stat-label">{name}</span>
                                        <span class="stat-value">{detail}</span>
                                    </div>
                                }.into_any()
                            }).collect()
                        };

                        view! {
                            <div>
                                <h2>{e.name.clone()}</h2>
                                <div class="subtitle">{format!(
                                    "{:?} · {}",
                                    e.event_type,
                                    provenance_label
                                )}</div>
                                {measurement}
                                {row("Source", source_label)}
                                {row("Classification", provenance_label)}
                                {e.source_id.clone().map(|id| row("Source ID", id))}
                                {e.occurred_at_ms.map(|timestamp| row("Occurred (UTC)", format_timestamp_ms(timestamp)))}
                                {e.updated_at_ms.map(|timestamp| row("Updated (UTC)", format_timestamp_ms(timestamp)))}
                                {e.depth_km.map(|depth| row("Depth", format!("{depth:.1} km")))}
                                {e.geometry_note.clone().map(|note| row("Location representation", note))}
                                {match e.source_url.clone() {
                                    Some(url) if
                                        e.data_kind == Some(sol_atlas_core::types::DataKind::Observed)
                                        && (
                                            (e.source_name.as_deref() == Some("USGS ComCat")
                                                && url.starts_with("https://earthquake.usgs.gov/earthquakes/eventpage/"))
                                            || (e.source_name.as_deref() == Some("NASA EONET")
                                                && url.starts_with("https://eonet.gsfc.nasa.gov/api/v3/events/"))
                                        ) =>
                                        view! {
                                            <a class="stat-row source-link" href=url target="_blank" rel="noopener noreferrer">
                                                <span class="stat-label">"Evidence"</span>
                                                <span class="stat-value">"Open official source ↗"</span>
                                            </a>
                                        }.into_any(),
                                    _ => view! { <div /> }.into_any(),
                                }}
                                {row("Mapped assets", format!("{nearby_count} within 100 km"))}
                                <div class="context-note">
                                    "Curated geographic context only. Proximity is not an impact or risk assessment."
                                </div>
                                {nearby_rows}
                            </div>
                        }.into_any()
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
                            .map(|l| row(l.label(), l.provenance().kind.label().to_string()))
                            .collect();
                        view! {
                            <div>
                                <h2>"Confluence"</h2>
                                <div class="subtitle">
                                    "Derived signal \u{00b7} where real systems co-locate \u{00b7} not a risk score"
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
