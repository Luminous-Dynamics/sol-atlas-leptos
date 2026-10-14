// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! The dossier: leads with one hero number (the fact you'd tell someone
//! first), then quiet secondary rows. Grows in rather than sliding a
//! rectangle.

use leptos::prelude::*;

use crate::data::types::SelectedItem;
use crate::state::globe_state::GlobeState;

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

#[component]
pub fn InfoPanel() -> impl IntoView {
    let globe_state = expect_context::<GlobeState>();

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
                        let qualified_eroi = sol_atlas_core::economics::qualified_eroi(d);
                        // Render supplied metadata even when malformed so the evidence gap
                        // remains inspectable; only qualified evidence affects classification.
                        let evidence = d.eroi_evidence.as_ref();
                        view! {
                            <div>
                                <h2>{d.name.clone()}</h2>
                                <div class="subtitle">{format!("{:?} \u{00b7} {} \u{00b7} curated", d.fuel_type, d.country)}</div>
                                {match eroi {
                                    Some(e) => hero(
                                        format!("{:.1}:1", e),
                                        if qualified_eroi.is_some() { "EROI estimate" } else { "EROI value — unqualified" },
                                    ).into_any(),
                                    None => hero("Unknown".to_string(), "physical EROI").into_any(),
                                }}
                                {row(
                                    "EROI classification",
                                    if eroi.is_none() {
                                        "Unknown — physical EROI not supplied".to_string()
                                    } else if qualified_eroi.is_none() {
                                        "Unqualified — source or boundary metadata missing".to_string()
                                    } else {
                                        "Not tiered — threshold methodology lacks a named system boundary".to_string()
                                    },
                                )}
                                {match evidence {
                                    Some(evidence) => row(
                                        "EROI metadata",
                                        if !evidence.is_qualified() {
                                            "Incomplete or invalid — not comparable".to_string()
                                        } else if eroi.is_some_and(|value| !evidence.supports_value(value)) {
                                            "Value outside stated uncertainty interval — not comparable".to_string()
                                        } else {
                                            "Shape valid; source not authenticated".to_string()
                                        },
                                    ),
                                    None => row("EROI metadata", "Not supplied".to_string()),
                                }}
                                {match evidence {
                                    Some(evidence) => row("EROI stage", evidence.stage.label().to_string()),
                                    None => row("EROI stage", "Not qualified".to_string()),
                                }}
                                {match evidence {
                                    Some(evidence) if evidence.methodology_id.trim().is_empty() => {
                                        row("EROI methodology", "Missing — not qualified".to_string())
                                    }
                                    Some(evidence) => row("EROI methodology", evidence.methodology_id.clone()),
                                    None => row("EROI methodology", "Not supplied".to_string()),
                                }}
                                {match evidence {
                                    Some(evidence) if evidence.methodology_version.trim().is_empty() => {
                                        row("EROI method version", "Missing — not qualified".to_string())
                                    }
                                    Some(evidence) => row("EROI method version", evidence.methodology_version.clone()),
                                    None => row("EROI method version", "Not supplied".to_string()),
                                }}
                                {match evidence {
                                    Some(evidence) => row("EROI output", evidence.output_definition.clone()),
                                    None => row("EROI output definition", "Not supplied".to_string()),
                                }}
                                {match evidence {
                                    Some(evidence) => row("EROI input", evidence.input_definition.clone()),
                                    None => row("EROI input definition", "Not supplied".to_string()),
                                }}
                                {match evidence {
                                    Some(evidence) => row("EROI geography", evidence.geographic_scope.clone()),
                                    None => row("EROI geography", "Not supplied".to_string()),
                                }}
                                {match evidence {
                                    Some(evidence) => row(
                                        "EROI period",
                                        format!("{}–{}", evidence.period_start_year, evidence.period_end_year),
                                    ),
                                    None => row("EROI period", "Not supplied".to_string()),
                                }}
                                {match evidence {
                                    Some(evidence) => row("EROI source", evidence.source_citation.clone()),
                                    None => row("EROI source", "Not supplied".to_string()),
                                }}
                                {match evidence {
                                    Some(evidence) => row("EROI basis", evidence.basis.label().to_string()),
                                    None => row("EROI basis", "Not supplied".to_string()),
                                }}
                                {match evidence {
                                    Some(evidence) => row(
                                        "EROI publication date",
                                        evidence.publication_date.clone().unwrap_or_else(|| "Not supplied".to_string()),
                                    ),
                                    None => row("EROI publication date", "Not supplied".to_string()),
                                }}
                                {match evidence {
                                    Some(evidence) => row(
                                        "EROI interval",
                                        evidence.uncertainty.map(|range| {
                                            let interpretation = range
                                                .interpretation
                                                .map(|kind| kind.label())
                                                .unwrap_or("Interpretation missing — not qualified");
                                            let level = match (range.interpretation, range.probability_level) {
                                                (
                                                    Some(sol_atlas_core::types::EroiUncertaintyKind::ConfidenceInterval),
                                                    Some(value),
                                                ) => format!(" ({:.0}% confidence)", value * 100.0),
                                                (
                                                    Some(sol_atlas_core::types::EroiUncertaintyKind::CredibleInterval),
                                                    Some(value),
                                                ) => format!(" ({:.0}% credible mass)", value * 100.0),
                                                (
                                                    Some(sol_atlas_core::types::EroiUncertaintyKind::ConfidenceInterval),
                                                    None,
                                                ) => " (confidence level missing)".to_string(),
                                                (
                                                    Some(sol_atlas_core::types::EroiUncertaintyKind::CredibleInterval),
                                                    None,
                                                ) => " (credible mass missing)".to_string(),
                                                (_, Some(_)) => " (unexpected level)".to_string(),
                                                _ => String::new(),
                                            };
                                            format!("{}{} — {:.2}–{:.2}", interpretation, level, range.lower, range.upper)
                                        }).unwrap_or_else(|| "Not supplied".to_string()),
                                    ),
                                    None => row("EROI interval", "Not supplied".to_string()),
                                }}
                                {row("Reserves", format!("{:.0} Mboe", d.proven_reserves_mboe))}
                                {row("Production", format!("{:.0} Mboe/yr", d.annual_production_mboe))}
                                {row("Status", d.status.clone())}
                                {row("Discovered", d.discovery_year.to_string())}
                            </div>
                        }.into_any()
                    }
                    Some(SelectedItem::NaturalEvent(e)) => view! {
                        <div>
                            <h2>{e.name.clone()}</h2>
                            <div class="subtitle">{format!("{:?} \u{00b7} observed snapshot", e.event_type)}</div>
                            {hero(format!("{:.1}", e.magnitude), "magnitude")}
                        </div>
                    }.into_any(),
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
