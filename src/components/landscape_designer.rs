// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! First vertical slice of the multi-scale landscape design workbench.
//!
//! This is a schematic editor, not a survey/GIS source or an engineering solver.
//! All canvas coordinates are local metres with a screen-oriented positive Y axis.
//! Georeferencing and standards-based exchange must be explicit future operations.

use leptos::prelude::*;
use leptos_router::components::A;
use serde_json::{Value, json};
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FeatureKind {
    Tree,
    PlantingBed,
    Water,
    Path,
    BuiltElement,
}

impl FeatureKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Tree => "Canopy tree",
            Self::PlantingBed => "Planting bed",
            Self::Water => "Water feature",
            Self::Path => "Path",
            Self::BuiltElement => "Built element",
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Tree => "tree",
            Self::PlantingBed => "planting-bed",
            Self::Water => "water",
            Self::Path => "path",
            Self::BuiltElement => "built-element",
        }
    }

    const fn color(self) -> &'static str {
        match self {
            Self::Tree => "#438b66",
            Self::PlantingBed => "#86a96f",
            Self::Water => "#5caab4",
            Self::Path => "#c8b785",
            Self::BuiltElement => "#b4aaa0",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DesignFeature {
    id: u32,
    x_du: u16,
    y_du: u16,
    kind: FeatureKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LandscapeScale {
    Site,
    District,
    Watershed,
    Bioregion,
}

impl LandscapeScale {
    const fn label(self) -> &'static str {
        match self {
            Self::Site => "Site / parcel",
            Self::District => "District",
            Self::Watershed => "Watershed",
            Self::Bioregion => "Bioregion",
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Site => "site-parcel",
            Self::District => "district",
            Self::Watershed => "watershed",
            Self::Bioregion => "bioregion",
        }
    }
}

fn starter_features() -> Vec<DesignFeature> {
    vec![
        DesignFeature {
            id: 1,
            x_du: 180,
            y_du: 160,
            kind: FeatureKind::Tree,
        },
        DesignFeature {
            id: 2,
            x_du: 245,
            y_du: 195,
            kind: FeatureKind::Tree,
        },
        DesignFeature {
            id: 3,
            x_du: 365,
            y_du: 250,
            kind: FeatureKind::PlantingBed,
        },
        DesignFeature {
            id: 4,
            x_du: 430,
            y_du: 285,
            kind: FeatureKind::Water,
        },
        DesignFeature {
            id: 5,
            x_du: 540,
            y_du: 360,
            kind: FeatureKind::Path,
        },
        DesignFeature {
            id: 6,
            x_du: 740,
            y_du: 190,
            kind: FeatureKind::BuiltElement,
        },
        DesignFeature {
            id: 7,
            x_du: 785,
            y_du: 420,
            kind: FeatureKind::Tree,
        },
    ]
}

fn snapshot_json(project_name: &str, scale: LandscapeScale, features: &[DesignFeature]) -> Value {
    json!({
        "schema": "luminous.landscape-design.v0",
        "project_name": project_name,
        "scope": scale.key(),
        "coordinate_frame": {
            "kind": "LOCAL_CARTESIAN_DRAWING_UNITS",
            "origin": "canvas top-left",
            "x_axis": "right",
            "y_axis": "down",
            "georeferenced": false
        },
        "geometry_status": "schematic-design-intent",
        "features": features.iter().map(|feature| json!({
            "id": format!("feature-{:04}", feature.id),
            "kind": feature.kind.key(),
            "label": feature.kind.label(),
            "x_du": feature.x_du,
            "y_du": feature.y_du,
            "status": "design-intent",
            "source": "user-authored-schematic"
        })).collect::<Vec<_>>()
    })
}

#[component]
pub fn LandscapeDesigner() -> impl IntoView {
    let (features, set_features) = signal(starter_features());
    let (active_tool, set_active_tool) = signal(FeatureKind::Tree);
    let (active_scale, set_active_scale) = signal(LandscapeScale::Site);
    let (show_grid, set_show_grid) = signal(true);
    let (show_context, set_show_context) = signal(true);
    let (export_text, set_export_text) = signal(String::new());

    let place_feature = move |event: web_sys::MouseEvent| {
        // Use the SVG element's measured bounds so pointer placement remains
        // correct when the responsive canvas is scaled by CSS.
        if let Some(target) = event.current_target() {
            let element: web_sys::Element = target.unchecked_into();
            let rect = element.get_bounding_client_rect();
            if rect.width() > 0.0 && rect.height() > 0.0 {
                let x = ((f64::from(event.client_x()) - rect.left()) * 1000.0 / rect.width())
                    .clamp(0.0, 1000.0)
                    .round() as u16;
                let y = ((f64::from(event.client_y()) - rect.top()) * 650.0 / rect.height())
                    .clamp(0.0, 650.0)
                    .round() as u16;
                let next_id = features
                    .get_untracked()
                    .iter()
                    .map(|item| item.id)
                    .max()
                    .unwrap_or(0)
                    + 1;
                set_features.update(|items| {
                    items.push(DesignFeature {
                        id: next_id,
                        x_du: x,
                        y_du: y,
                        kind: active_tool.get_untracked(),
                    })
                });
                set_export_text.set(String::new());
            }
        }
    };

    let export_project = move |_| {
        let value = snapshot_json(
            "Untitled landscape",
            active_scale.get_untracked(),
            &features.get_untracked(),
        );
        set_export_text.set(
            serde_json::to_string_pretty(&value)
                .unwrap_or_else(|error| format!("Snapshot serialization failed: {error}")),
        );
    };

    let undo = move |_| {
        set_features.update(|items| {
            items.pop();
        });
        set_export_text.set(String::new());
    };

    let reset = move |_| {
        set_features.set(starter_features());
        set_active_tool.set(FeatureKind::Tree);
        set_active_scale.set(LandscapeScale::Site);
        set_show_grid.set(true);
        set_show_context.set(true);
        set_export_text.set(String::new());
    };

    view! {
        <main class="landscape-workbench" on:keydown=move |event: web_sys::KeyboardEvent| {
            if let Some(target) = event.target() {
                let element: web_sys::Element = target.unchecked_into();
                let tag = element.tag_name();
                if tag == "INPUT" || tag == "TEXTAREA" { return; }
            }
            match event.key().to_lowercase().as_str() {
                "t" => set_active_tool.set(FeatureKind::Tree),
                "p" => set_active_tool.set(FeatureKind::PlantingBed),
                "w" => set_active_tool.set(FeatureKind::Water),
                "l" => set_active_tool.set(FeatureKind::Path),
                "b" => set_active_tool.set(FeatureKind::BuiltElement),
                _ => return,
            }
            event.prevent_default();
        }>
            <header class="landscape-topbar">
                <div class="landscape-brand">
                    <A href="/" attr:class="landscape-back">"← Sol Atlas"</A>
                    <div>
                        <p class="landscape-kicker">"LUMINOUS DYNAMICS / LAND SYSTEMS"</p>
                        <h1>"Landscape Workbench"</h1>
                    </div>
                </div>
                <div class="landscape-top-actions">
                    <span class="landscape-draft-mark"><span></span>" SCHEMATIC DRAFT"</span>
                    <button class="landscape-button quiet" type="button" on:click=undo>"Undo"</button>
                    <button class="landscape-button primary" type="button" on:click=export_project>"Export snapshot"</button>
                </div>
            </header>

            <div class="landscape-layout">
                <aside class="landscape-sidebar" aria-label="Design tools and project scope">
                    <section class="landscape-section">
                        <p class="landscape-section-label">"01 / DESIGN SCALE"</p>
                        <h2>"Think in systems"</h2>
                        <p class="landscape-muted">"Switch planning context from a single property to its wider living landscape. This changes the declared scope, not the coordinate frame."</p>
                        <div class="landscape-scale-list">
                            <button type="button" class:chosen=move || active_scale.get() == LandscapeScale::Site on:click=move |_| set_active_scale.set(LandscapeScale::Site)><span>"01"</span>"Site / parcel"</button>
                            <button type="button" class:chosen=move || active_scale.get() == LandscapeScale::District on:click=move |_| set_active_scale.set(LandscapeScale::District)><span>"02"</span>"District"</button>
                            <button type="button" class:chosen=move || active_scale.get() == LandscapeScale::Watershed on:click=move |_| set_active_scale.set(LandscapeScale::Watershed)><span>"03"</span>"Watershed"</button>
                            <button type="button" class:chosen=move || active_scale.get() == LandscapeScale::Bioregion on:click=move |_| set_active_scale.set(LandscapeScale::Bioregion)><span>"04"</span>"Bioregion"</button>
                        </div>
                    </section>

                    <section class="landscape-section">
                        <p class="landscape-section-label">"02 / PLACE ELEMENTS"</p>
                        <h2>"Design palette"</h2>
                        <p class="landscape-muted">"Choose an element, then click the plan to place it."</p>
                        <div class="landscape-tool-list">
                            <button type="button" class:chosen=move || active_tool.get() == FeatureKind::Tree on:click=move |_| set_active_tool.set(FeatureKind::Tree) aria-pressed=move || (active_tool.get() == FeatureKind::Tree).to_string()><span class="tool-swatch tree-swatch"></span><span>"Canopy tree"</span><kbd>"T"</kbd></button>
                            <button type="button" class:chosen=move || active_tool.get() == FeatureKind::PlantingBed on:click=move |_| set_active_tool.set(FeatureKind::PlantingBed) aria-pressed=move || (active_tool.get() == FeatureKind::PlantingBed).to_string()><span class="tool-swatch bed-swatch"></span><span>"Planting bed"</span><kbd>"P"</kbd></button>
                            <button type="button" class:chosen=move || active_tool.get() == FeatureKind::Water on:click=move |_| set_active_tool.set(FeatureKind::Water) aria-pressed=move || (active_tool.get() == FeatureKind::Water).to_string()><span class="tool-swatch water-swatch"></span><span>"Water feature"</span><kbd>"W"</kbd></button>
                            <button type="button" class:chosen=move || active_tool.get() == FeatureKind::Path on:click=move |_| set_active_tool.set(FeatureKind::Path) aria-pressed=move || (active_tool.get() == FeatureKind::Path).to_string()><span class="tool-swatch path-swatch"></span><span>"Path / connection"</span><kbd>"L"</kbd></button>
                            <button type="button" class:chosen=move || active_tool.get() == FeatureKind::BuiltElement on:click=move |_| set_active_tool.set(FeatureKind::BuiltElement) aria-pressed=move || (active_tool.get() == FeatureKind::BuiltElement).to_string()><span class="tool-swatch built-swatch"></span><span>"Built element"</span><kbd>"B"</kbd></button>
                        </div>
                    </section>

                    <section class="landscape-section">
                        <p class="landscape-section-label">"03 / DRAWING LAYERS"</p>
                        <h2>"Reading the site"</h2>
                        <label class="landscape-toggle"><input type="checkbox" prop:checked=move || show_grid.get() on:change=move |_| set_show_grid.update(|visible| *visible = !*visible)/><span>"Measurement grid"</span></label>
                        <label class="landscape-toggle"><input type="checkbox" prop:checked=move || show_context.get() on:change=move |_| set_show_context.update(|visible| *visible = !*visible)/><span>"Illustrative contours"</span></label>
                        <p class="landscape-note">"Context lines are decorative teaching geometry, not a terrain survey or hydrology result."</p>
                    </section>

                    <section class="landscape-section landscape-summary">
                        <p class="landscape-section-label">"CURRENT PLAN"</p>
                        <div class="landscape-stat"><strong>{move || features.get().len().to_string()}</strong><span>"placed elements"</span></div>
                        <div class="landscape-stat"><strong>{move || active_scale.get().label()}</strong><span>"declared scale"</span></div>
                        <button class="landscape-button quiet full-width" type="button" on:click=reset>"Reset starter plan"</button>
                    </section>
                </aside>

                <section class="landscape-stage" aria-label="Landscape plan editor">
                    <div class="landscape-stage-head">
                        <div>
                            <p class="landscape-kicker">"UNTITLED LANDSCAPE / CONCEPT 01"</p>
                            <h2>"Living systems plan"</h2>
                        </div>
                        <div class="landscape-stage-meta"><span>"LOCAL GRID"</span><span>"1000 × 650 schematic units"</span></div>
                    </div>
                    <div class="landscape-canvas-frame">
                        <svg class="landscape-canvas" viewBox="0 0 1000 650" tabindex="0" role="application" aria-label="Schematic landscape canvas. Click to place the selected design element; press Enter or Space to place at the center." on:click=place_feature on:keydown=move |event: web_sys::KeyboardEvent| {
                            if event.key() == "Enter" || event.key() == " " {
                                event.prevent_default();
                                let next_id = features.get_untracked().iter().map(|item| item.id).max().unwrap_or(0) + 1;
                                set_features.update(|items| items.push(DesignFeature {
                                    id: next_id,
                                    x_du: 500,
                                    y_du: 325,
                                    kind: active_tool.get_untracked(),
                                }));
                                set_export_text.set(String::new());
                            }
                        }>
                            <defs>
                                <pattern id="landscape-grid" width="50" height="50" patternUnits="userSpaceOnUse">
                                    <path d="M 50 0 L 0 0 0 50" fill="none" stroke="#7f9384" stroke-width="1" stroke-opacity="0.22"/>
                                </pattern>
                            </defs>
                            <rect x="0" y="0" width="1000" height="650" fill="#e7e7d7"/>
                            <rect x="0" y="0" width="1000" height="650" fill="url(#landscape-grid)" style:opacity=move || if show_grid.get() { "1" } else { "0" }/>
                            <g style:opacity=move || if show_context.get() { "0.62" } else { "0" }>
                                <path d="M -20 130 C 180 55, 260 185, 430 125 S 720 65, 1020 145" fill="none" stroke="#a7b49c" stroke-width="2"/>
                                <path d="M -20 166 C 180 91, 260 221, 430 161 S 720 101, 1020 181" fill="none" stroke="#a7b49c" stroke-width="2"/>
                                <path d="M -20 202 C 180 127, 260 257, 430 197 S 720 137, 1020 217" fill="none" stroke="#a7b49c" stroke-width="2"/>
                                <path d="M 80 670 C 120 500, 230 470, 330 360 S 520 280, 650 340 S 820 540, 1020 440" fill="none" stroke="#9da995" stroke-width="2"/>
                                <path d="M 105 670 C 145 525, 245 495, 350 380 S 525 305, 665 365 S 835 565, 1020 465" fill="none" stroke="#9da995" stroke-width="2"/>
                            </g>
                            <path d="M 45 45 L 935 45 L 955 595 L 65 610 Z" fill="#a6b59b" fill-opacity="0.12" stroke="#6f8a76" stroke-width="3" stroke-dasharray="12 8"/>
                            <text x="70" y="80" fill="#566a5b" font-size="17" letter-spacing="3">"CONCEPTUAL SITE BOUNDARY"</text>
                            <For each=move || features.get() key=|feature| feature.id children=move |feature: DesignFeature| {
                                let x = feature.x_du.to_string();
                                let y = feature.y_du.to_string();
                                let left = feature.x_du.saturating_sub(17).to_string();
                                let top = feature.y_du.saturating_sub(12).to_string();
                                let title = feature.kind.label();
                                match feature.kind {
                                    FeatureKind::Tree => view! {
                                        <g>
                                            <circle cx=x.clone() cy=y.clone() r="17" fill="#438b66" fill-opacity="0.78" stroke="#315d47" stroke-width="2"/>
                                            <circle cx=x.clone() cy=y.clone() r="7" fill="#b8d3a8" fill-opacity="0.75"/>
                                            <title>{title}</title>
                                        </g>
                                    }.into_any(),
                                    FeatureKind::PlantingBed => view! {
                                        <g>
                                            <rect x=left.clone() y=top.clone() width="34" height="24" rx="5" fill="#86a96f" stroke="#4f744d" stroke-width="2"/>
                                            <path d=format!("M {} {} l 10 16 m 5 -16 l 10 16", feature.x_du.saturating_sub(12), feature.y_du.saturating_sub(8)) fill="none" stroke="#e6efd9" stroke-width="2"/>
                                            <title>{title}</title>
                                        </g>
                                    }.into_any(),
                                    FeatureKind::Water => view! {
                                        <g>
                                            <path d=format!("M {} {} C {} {}, {} {}, {} {} C {} {}, {} {}, {} {}", feature.x_du.saturating_sub(18), feature.y_du, feature.x_du.saturating_sub(10), feature.y_du.saturating_sub(19), feature.x_du.saturating_add(8), feature.y_du.saturating_sub(16), feature.x_du.saturating_add(18), feature.y_du, feature.x_du.saturating_add(25), feature.y_du.saturating_add(15), feature.x_du.saturating_sub(12), feature.y_du.saturating_add(17), feature.x_du.saturating_sub(18), feature.y_du) fill="#5caab4" stroke="#347780" stroke-width="2"/>
                                            <title>{title}</title>
                                        </g>
                                    }.into_any(),
                                    FeatureKind::Path => view! {
                                        <g>
                                            <path d=format!("M {} {} l 35 18", feature.x_du.saturating_sub(16), feature.y_du.saturating_sub(8)) fill="none" stroke="#c8b785" stroke-width="12" stroke-linecap="round"/>
                                            <path d=format!("M {} {} l 35 18", feature.x_du.saturating_sub(16), feature.y_du.saturating_sub(8)) fill="none" stroke="#f1e7c6" stroke-width="2" stroke-linecap="round"/>
                                            <title>{title}</title>
                                        </g>
                                    }.into_any(),
                                    FeatureKind::BuiltElement => view! {
                                        <g>
                                            <rect x=left.clone() y=top.clone() width="34" height="24" rx="2" fill="#b4aaa0" stroke="#716961" stroke-width="2"/>
                                            <path d=format!("M {} {} l 17 -11 l 17 11", feature.x_du.saturating_sub(17), feature.y_du.saturating_sub(12)) fill="none" stroke="#716961" stroke-width="2"/>
                                            <title>{title}</title>
                                        </g>
                                    }.into_any(),
                                }
                            }/>
                            <g transform="translate(900 75)">
                                <path d="M 0 22 L 0 -12 M -7 -2 L 0 -14 L 7 -2" fill="none" stroke="#4d6356" stroke-width="3"/>
                                <text x="0" y="39" text-anchor="middle" font-size="14" fill="#4d6356">"N"</text>
                            </g>
                        </svg>
                        <div class="landscape-canvas-legend"><span><i class="legend-tree"></i>"Canopy"</span><span><i class="legend-bed"></i>"Planting"</span><span><i class="legend-water"></i>"Water"</span><span><i class="legend-path"></i>"Circulation"</span><span><i class="legend-built"></i>"Built"</span></div>
                    </div>
                    <footer class="landscape-stage-footer">
                        <span><i class="status-dot"></i>"Editor ready"</span>
                        <span>"Selected tool: " {move || active_tool.get().label()}</span>
                        <span>"Local drawing units · not georeferenced"</span>
                        <span>"Focus plan + Enter/Space places at center · T/P/W/L/B switch tools"</span>
                    </footer>
                    <section class="landscape-export-panel">
                        <div class="landscape-export-header"><div><p class="landscape-section-label">"PORTABLE PROJECT SNAPSHOT"</p><h3>"Draft data"</h3></div><p>"Native JSON · schema v0"</p></div>
                        {move || if export_text.get().is_empty() {
                            view! { <p class="landscape-muted">"Export a snapshot to inspect the exact design-intent data and coordinate declaration. No remote save or network upload occurs."</p> }.into_any()
                        } else {
                            view! { <pre class="landscape-json">{move || export_text.get()}</pre> }.into_any()
                        }}
                    </section>
                </section>
            </div>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_declares_local_frame_and_never_claims_georeferencing() {
        let features = starter_features();
        let value = snapshot_json("fixture", LandscapeScale::Watershed, &features);
        assert_eq!(value["schema"], "luminous.landscape-design.v0");
        assert_eq!(value["scope"], "watershed");
        assert_eq!(
            value["coordinate_frame"]["kind"],
            "LOCAL_CARTESIAN_DRAWING_UNITS"
        );
        assert_eq!(value["coordinate_frame"]["georeferenced"], false);
        assert_eq!(value["features"].as_array().unwrap().len(), 7);
    }

    #[test]
    fn snapshot_round_trips_as_json_and_preserves_feature_ids() {
        let features = starter_features();
        let value = snapshot_json("fixture", LandscapeScale::Site, &features);
        let serialized = serde_json::to_string(&value).unwrap();
        let parsed: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(parsed, value);
        assert_eq!(parsed["features"][0]["id"], "feature-0001");
    }

    #[test]
    fn every_design_tool_has_a_stable_machine_key() {
        let kinds = [
            FeatureKind::Tree,
            FeatureKind::PlantingBed,
            FeatureKind::Water,
            FeatureKind::Path,
            FeatureKind::BuiltElement,
        ];
        assert_eq!(
            kinds.map(FeatureKind::key),
            ["tree", "planting-bed", "water", "path", "built-element"]
        );
    }
}
