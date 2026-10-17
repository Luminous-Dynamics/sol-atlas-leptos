// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, MouseEvent, TouchEvent, WebGl2RenderingContext, WheelEvent};

use crate::data::geo;
use crate::data::types::*;
use crate::renderer::geometry;
use crate::renderer::math::Vec3;
use crate::renderer::picking;
use crate::renderer::{GlobeRenderer, MarkerInstance};
use crate::state::data_state::DataState;
use crate::state::globe_state::GlobeState;

/// Flat list of marker positions + metadata for picking.
struct PickableMarker {
    position: Vec3,
    info: HoverInfo,
    selected: SelectedItem,
}

#[component]
pub fn GlobeCanvas() -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let globe_state = expect_context::<GlobeState>();
    let data_state = expect_context::<DataState>();

    Effect::new(move |_| {
        let Some(canvas) = canvas_ref.get() else {
            log::warn!("Canvas ref not yet available");
            return;
        };
        let canvas_el: HtmlCanvasElement = canvas.into();

        let window = web_sys::window().unwrap();
        let w = window.inner_width().unwrap().as_f64().unwrap() as u32;
        let h = window.inner_height().unwrap().as_f64().unwrap() as u32;
        canvas_el.set_width(w);
        canvas_el.set_height(h);

        let gl: WebGl2RenderingContext = match canvas_el.get_context("webgl2") {
            Ok(Some(ctx)) => ctx.dyn_into().unwrap(),
            Ok(None) => {
                log::error!("WebGL2 not supported");
                show_globe_unavailable_message(
                    &canvas_el,
                    "Sol Atlas needs WebGL2, which this browser does not support or has disabled.",
                );
                return;
            }
            Err(e) => {
                log::error!("WebGL2 context error: {:?}", e);
                show_globe_unavailable_message(
                    &canvas_el,
                    "Sol Atlas needs WebGL2, which this browser does not support or has disabled.",
                );
                return;
            }
        };

        log::info!("WebGL2 context created");

        let renderer = match GlobeRenderer::init(gl) {
            Ok(r) => {
                log::info!("GlobeRenderer initialized successfully");
                r
            }
            Err(e) => {
                log::error!("GlobeRenderer init failed: {e}");
                show_globe_unavailable_message(
                    &canvas_el,
                    "Sol Atlas failed to initialize its renderer. Try reloading the page.",
                );
                return;
            }
        };

        let renderer = Rc::new(RefCell::new(renderer));

        // Shared pickable markers list (rebuilt when layers change)
        let pickables: Rc<RefCell<Vec<PickableMarker>>> = Rc::new(RefCell::new(Vec::new()));

        // Track mouse-down position to distinguish click from drag
        let mouse_down_pos: Rc<RefCell<Option<(f32, f32)>>> = Rc::new(RefCell::new(None));

        // H3 hex hover: the currently outlined cell, so repeated hover
        // events over the same hex skip rebuilding its boundary VAO.
        let last_hex_cell: Rc<RefCell<Option<h3o::CellIndex>>> = Rc::new(RefCell::new(None));

        // Clone handles for closures
        let r_md = renderer.clone();
        let mdp = mouse_down_pos.clone();

        // ── Mouse Down ──
        let mouse_down = Closure::wrap(Box::new(move |e: MouseEvent| {
            let x = e.client_x() as f32;
            let y = e.client_y() as f32;
            r_md.borrow_mut().camera.on_mouse_down(x, y);
            *mdp.borrow_mut() = Some((x, y));
        }) as Box<dyn FnMut(MouseEvent)>);
        canvas_el
            .add_event_listener_with_callback("mousedown", mouse_down.as_ref().unchecked_ref())
            .unwrap();
        mouse_down.forget();

        // ── Mouse Move (drag + hover picking) ──
        let r_mm = renderer.clone();
        let gs_mm = globe_state.clone();
        let pick_mm = pickables.clone();
        let canvas_mm = canvas_el.clone();
        let last_hex_mm = last_hex_cell.clone();

        let mouse_move = Closure::wrap(Box::new(move |e: MouseEvent| {
            let x = e.client_x() as f32;
            let y = e.client_y() as f32;

            r_mm.borrow_mut().camera.on_mouse_move(x, y);
            gs_mm.mouse_pos.set((x, y));

            // Hover picking (only when not dragging)
            if !r_mm.borrow().camera.is_dragging() {
                let cw = canvas_mm.client_width() as f32;
                let ch = canvas_mm.client_height() as f32;
                let r = r_mm.borrow();
                let proj = r.camera.projection_matrix(cw / ch);
                let view = r.camera.view_matrix(r.time_secs());

                if let Some((origin, dir)) = picking::screen_to_ray(x, y, cw, ch, &proj, &view) {
                    if let Some(hit) = picking::ray_sphere_intersect(origin, dir, Vec3::ZERO, 1.02)
                    {
                        let picks = pick_mm.borrow();
                        let positions: Vec<Vec3> = picks.iter().map(|p| p.position).collect();
                        if let Some(idx) = picking::find_nearest_marker(hit, &positions, 0.08) {
                            gs_mm.hovered.set(Some(picks[idx].info.clone()));
                        } else {
                            gs_mm.hovered.set(None);
                        }
                    } else {
                        gs_mm.hovered.set(None);
                    }
                }
                drop(r);
                update_hex_hover(x, y, &canvas_mm, &r_mm, &last_hex_mm);
            }
        }) as Box<dyn FnMut(MouseEvent)>);
        canvas_el
            .add_event_listener_with_callback("mousemove", mouse_move.as_ref().unchecked_ref())
            .unwrap();
        mouse_move.forget();

        // ── Mouse Up (click detection) ──
        let r_mu = renderer.clone();
        let gs_mu = globe_state.clone();
        let pick_mu = pickables.clone();
        let mdp_mu = mouse_down_pos.clone();
        let canvas_mu = canvas_el.clone();

        let mouse_up = Closure::wrap(Box::new(move |e: MouseEvent| {
            r_mu.borrow_mut().camera.on_mouse_up();

            // Detect click (not drag): mouse moved less than 5px
            let x = e.client_x() as f32;
            let y = e.client_y() as f32;
            if let Some((dx, dy)) = mdp_mu.borrow().as_ref() {
                let dist = ((x - dx).powi(2) + (y - dy).powi(2)).sqrt();
                if dist < 5.0 {
                    click_select_or_drill(x, y, &canvas_mu, &r_mu, &pick_mu, &gs_mu);
                }
            }
            *mdp_mu.borrow_mut() = None;
        }) as Box<dyn FnMut(MouseEvent)>);
        canvas_el
            .add_event_listener_with_callback("mouseup", mouse_up.as_ref().unchecked_ref())
            .unwrap();
        mouse_up.forget();

        // ── Wheel ──
        let r_wh = renderer.clone();
        let wheel = Closure::wrap(Box::new(move |e: WheelEvent| {
            e.prevent_default();
            r_wh.borrow_mut().camera.on_wheel(e.delta_y() as f32);
        }) as Box<dyn FnMut(WheelEvent)>);
        let mut opts = web_sys::AddEventListenerOptions::new();
        opts.passive(false);
        canvas_el
            .add_event_listener_with_callback_and_add_event_listener_options(
                "wheel",
                wheel.as_ref().unchecked_ref(),
                &opts,
            )
            .unwrap();
        wheel.forget();

        // ── Touch: one-finger drag rotates, two-finger pinch zooms, tap picks ──
        // Reuses the camera's mouse path for rotation and the wheel path for
        // zoom, so touch and mouse stay behaviorally identical.
        let touch_start_pos: Rc<RefCell<Option<(f32, f32)>>> = Rc::new(RefCell::new(None));
        let pinch_dist: Rc<RefCell<Option<f32>>> = Rc::new(RefCell::new(None));

        fn touch_xy(e: &TouchEvent, i: u32) -> Option<(f32, f32)> {
            e.touches()
                .item(i)
                .map(|t| (t.client_x() as f32, t.client_y() as f32))
        }
        fn pinch_distance(e: &TouchEvent) -> Option<f32> {
            let (x0, y0) = touch_xy(e, 0)?;
            let (x1, y1) = touch_xy(e, 1)?;
            Some(((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt())
        }

        let mut touch_opts = web_sys::AddEventListenerOptions::new();
        touch_opts.passive(false);

        let r_tst = renderer.clone();
        let tsp_st = touch_start_pos.clone();
        let pd_st = pinch_dist.clone();
        let touch_start = Closure::wrap(Box::new(move |e: TouchEvent| {
            e.prevent_default();
            match e.touches().length() {
                1 => {
                    if let Some((x, y)) = touch_xy(&e, 0) {
                        r_tst.borrow_mut().camera.on_mouse_down(x, y);
                        *tsp_st.borrow_mut() = Some((x, y));
                    }
                }
                2 => {
                    // Second finger: stop rotating, start pinching
                    r_tst.borrow_mut().camera.on_mouse_up();
                    *tsp_st.borrow_mut() = None;
                    *pd_st.borrow_mut() = pinch_distance(&e);
                }
                _ => {}
            }
        }) as Box<dyn FnMut(TouchEvent)>);
        canvas_el
            .add_event_listener_with_callback_and_add_event_listener_options(
                "touchstart",
                touch_start.as_ref().unchecked_ref(),
                &touch_opts,
            )
            .unwrap();
        touch_start.forget();

        let r_tmv = renderer.clone();
        let pd_mv = pinch_dist.clone();
        let touch_move = Closure::wrap(Box::new(move |e: TouchEvent| {
            e.prevent_default();
            match e.touches().length() {
                1 => {
                    if let Some((x, y)) = touch_xy(&e, 0) {
                        r_tmv.borrow_mut().camera.on_mouse_move(x, y);
                    }
                }
                2 => {
                    if let Some(dist) = pinch_distance(&e) {
                        let mut prev = pd_mv.borrow_mut();
                        if let Some(p) = *prev {
                            // Pinch-in (shrinking distance) zooms out, matching
                            // positive wheel delta. ~6x scales finger pixels to
                            // wheel-notch magnitudes.
                            r_tmv.borrow_mut().camera.on_wheel((p - dist) * 6.0);
                        }
                        *prev = Some(dist);
                    }
                }
                _ => {}
            }
        }) as Box<dyn FnMut(TouchEvent)>);
        canvas_el
            .add_event_listener_with_callback_and_add_event_listener_options(
                "touchmove",
                touch_move.as_ref().unchecked_ref(),
                &touch_opts,
            )
            .unwrap();
        touch_move.forget();

        let r_ten = renderer.clone();
        let gs_ten = globe_state.clone();
        let pick_ten = pickables.clone();
        let canvas_ten = canvas_el.clone();
        let tsp_en = touch_start_pos.clone();
        let pd_en = pinch_dist.clone();
        let touch_end = Closure::wrap(Box::new(move |e: TouchEvent| {
            r_ten.borrow_mut().camera.on_mouse_up();
            if e.touches().length() < 2 {
                *pd_en.borrow_mut() = None;
            }
            // Tap (short, low-movement single touch) = pick, like a click
            if let Some((sx, sy)) = tsp_en.borrow_mut().take() {
                if let Some(t) = e.changed_touches().item(0) {
                    let (x, y) = (t.client_x() as f32, t.client_y() as f32);
                    let dist = ((x - sx).powi(2) + (y - sy).powi(2)).sqrt();
                    if dist < 10.0 {
                        click_select_or_drill(x, y, &canvas_ten, &r_ten, &pick_ten, &gs_ten);
                    }
                }
            }
        }) as Box<dyn FnMut(TouchEvent)>);
        canvas_el
            .add_event_listener_with_callback("touchend", touch_end.as_ref().unchecked_ref())
            .unwrap();
        touch_end.forget();

        // ── WebGL context loss/restore ──
        // prevent_default on loss keeps the context restorable; on restore the
        // cleanest recovery for this stateless app is a reload (all GL objects
        // are gone and the renderer has no re-init path mid-session).
        let ctx_lost = Closure::wrap(Box::new(move |e: web_sys::Event| {
            e.prevent_default();
            log::error!("WebGL context lost — rendering halted, awaiting restore");
        }) as Box<dyn FnMut(web_sys::Event)>);
        canvas_el
            .add_event_listener_with_callback("webglcontextlost", ctx_lost.as_ref().unchecked_ref())
            .unwrap();
        ctx_lost.forget();

        let ctx_restored = Closure::wrap(Box::new(move |_: web_sys::Event| {
            log::warn!("WebGL context restored — reloading to reinitialize renderer");
            if let Some(w) = web_sys::window() {
                let _ = w.location().reload();
            }
        }) as Box<dyn FnMut(web_sys::Event)>);
        canvas_el
            .add_event_listener_with_callback(
                "webglcontextrestored",
                ctx_restored.as_ref().unchecked_ref(),
            )
            .unwrap();
        ctx_restored.forget();

        // Build initial data
        update_renderer_data(&renderer, &pickables, &data_state, &globe_state);

        // ── Animation Loop ──
        let canvas_for_frame = canvas_el.clone();
        let r_frame = renderer.clone();
        let ds = data_state.clone();
        let gs2 = globe_state.clone();
        let picks_frame = pickables.clone();
        // Thermodynamic coupling: device energy dims the heartbeat, deep
        // torpor halves the frame rate. Optional so the canvas still works
        // if vitality isn't provided (e.g. isolated component tests).
        let thermo = use_context::<crate::vitality::ThermodynamicState>();

        let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
        let g = f.clone();
        let mut last_layer_hash = 0u64;
        let mut last_data_revision = ds.data_revision.get_untracked();
        let mut frame_count = 0u32;
        let mut tick = 0u64;
        let mut last_aesthetic = globe_state.aesthetic.get_untracked();
        // Apply the initial preset immediately (renderer defaults are the
        // Holographic values already, but this keeps the two paths honest).
        r_frame.borrow_mut().set_aesthetic(last_aesthetic);

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move |time: f64| {
            tick += 1;
            if let Some(th) = thermo {
                let torpor = th.torpor_level.get_untracked();
                if torpor > 0.5 && tick % 2 == 1 {
                    // Deep torpor: render every other frame to halve GPU cost
                    let window = web_sys::window().unwrap();
                    let _ = window.request_animation_frame(
                        f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                    );
                    return;
                }
                r_frame
                    .borrow_mut()
                    .set_vitality(th.device_energy.get_untracked() as f32);
            }
            let layer_hash = {
                let layers = gs2.active_layers.read();
                let mut h = 0u64;
                for l in layers.iter() {
                    h ^= *l as u64 * 2654435761;
                }
                h ^= gs2.timeline_year.get() as u64 * 1000003; // timeline changes trigger rebuild
                h
            };
            let data_revision = ds.data_revision.get_untracked();
            if layer_hash != last_layer_hash || data_revision != last_data_revision {
                last_layer_hash = layer_hash;
                last_data_revision = data_revision;
                update_renderer_data(&r_frame, &picks_frame, &ds, &gs2);
            }

            let aesthetic = gs2.aesthetic.get_untracked();
            if aesthetic != last_aesthetic {
                last_aesthetic = aesthetic;
                r_frame.borrow_mut().set_aesthetic(aesthetic);
            }

            let cw = canvas_for_frame.client_width() as u32;
            let ch = canvas_for_frame.client_height() as u32;
            if canvas_for_frame.width() != cw || canvas_for_frame.height() != ch {
                canvas_for_frame.set_width(cw);
                canvas_for_frame.set_height(ch);
            }

            {
                let mut renderer = r_frame.borrow_mut();
                if renderer.is_context_lost() {
                    // Keep the rAF loop alive so rendering resumes after the
                    // context-restored reload path, but skip GL work.
                    let window = web_sys::window().unwrap();
                    let _ = window.request_animation_frame(
                        f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                    );
                    return;
                }
                renderer.show_core = gs2.show_core.get();

                // Handle planet fly-to navigation
                use crate::components::planet_nav::PlanetTarget;
                if let Some(planet) = gs2.focused_planet.get() {
                    if !renderer.camera.is_flying() {
                        let t = (time / 1000.0) as f32;
                        let (dist, speed, offset) = planet.orbit_params();
                        if dist > 0.01 {
                            // Compute current planet position
                            let angle = t * speed + offset;
                            let pos = crate::renderer::math::Vec3::new(
                                dist * angle.cos(),
                                0.0,
                                dist * angle.sin(),
                            );
                            renderer.camera.fly_to(pos, planet.camera_distance());
                        } else {
                            // Earth: fly home
                            renderer.camera.fly_home();
                        }
                    }
                }

                renderer.frame(time, cw, ch);
            }

            frame_count += 1;
            if frame_count == 1 {
                log::info!("First frame rendered ({}x{})", cw, ch);
            }

            let window = web_sys::window().unwrap();
            let _ = window
                .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref());
        }) as Box<dyn FnMut(f64)>));

        let window = web_sys::window().unwrap();
        let _ =
            window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref());

        log::info!("Animation loop started");
    });

    view! {
        <canvas id="globe-canvas" node_ref=canvas_ref />
    }
}

/// Ray-pick the nearest marker at screen position (x, y); shared by mouse
/// click and touch tap.
fn pick_selected_at(
    x: f32,
    y: f32,
    canvas: &HtmlCanvasElement,
    renderer: &Rc<RefCell<GlobeRenderer>>,
    pickables: &Rc<RefCell<Vec<PickableMarker>>>,
) -> Option<SelectedItem> {
    let cw = canvas.client_width() as f32;
    let ch = canvas.client_height() as f32;
    let r = renderer.borrow();
    let proj = r.camera.projection_matrix(cw / ch);
    let view = r.camera.view_matrix(r.time_secs());
    let (origin, dir) = picking::screen_to_ray(x, y, cw, ch, &proj, &view)?;
    let hit = picking::ray_sphere_intersect(origin, dir, Vec3::ZERO, 1.02)?;
    let picks = pickables.borrow();
    let positions: Vec<Vec3> = picks.iter().map(|p| p.position).collect();
    picking::find_nearest_marker(hit, &positions, 0.08).map(|idx| picks[idx].selected.clone())
}

/// Insert a visible message where the globe would have been, in place of
/// a permanently blank canvas. Tests the REAL canvas actually used for
/// rendering — unlike an earlier index.html-level probe (a throwaway
/// detached `<canvas>` element) that gave false positives in headless
/// Chromium and evidently some real mobile configurations too, and — far
/// worse — blocked the loading screen from ever hiding on EVERY route,
/// including WebGL-free ones like /labs/reactor-twin (found live,
/// 2026-07-11: the globe worked but the reactor-twin page rendered as a
/// permanently blank black screen for every visitor).
fn show_globe_unavailable_message(canvas_el: &HtmlCanvasElement, message: &str) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Ok(div) = document.create_element("div") else {
        return;
    };
    div.set_class_name("globe-unavailable");
    div.set_text_content(Some(message));
    if let Some(parent) = canvas_el.parent_node() {
        let _ = parent.append_child(&div);
    }
}

/// H3 cell under a screen position, plus the ray-hit point (used as the
/// drill-in fly-to target). Raycasts against radius 1.0 — the actual earth
/// mesh surface (`Self::create_sphere_vao(&gl, 128, 128, 1.0)`), not the
/// 1.02 markers sit at.
fn hex_hit_at(
    x: f32,
    y: f32,
    canvas: &HtmlCanvasElement,
    renderer: &Rc<RefCell<GlobeRenderer>>,
) -> Option<(h3o::CellIndex, Vec3)> {
    let cw = canvas.client_width() as f32;
    let ch = canvas.client_height() as f32;
    let r = renderer.borrow();
    let proj = r.camera.projection_matrix(cw / ch);
    let view = r.camera.view_matrix(r.time_secs());
    let camera_distance = r.camera.distance;
    let (origin, dir) = picking::screen_to_ray(x, y, cw, ch, &proj, &view)?;
    let hit = picking::ray_sphere_intersect(origin, dir, Vec3::ZERO, 1.0)?;
    let cell = crate::renderer::hex::cell_at_hit(hit, camera_distance)?;
    Some((cell, hit))
}

/// Update the hovered H3 cell outline, rebuilding the boundary VAO only
/// when the hovered cell actually changes (not on every mousemove tick).
fn update_hex_hover(
    x: f32,
    y: f32,
    canvas: &HtmlCanvasElement,
    renderer: &Rc<RefCell<GlobeRenderer>>,
    last_hex_cell: &Rc<RefCell<Option<h3o::CellIndex>>>,
) {
    let hit = hex_hit_at(x, y, canvas, renderer);
    let cell = hit.map(|(c, _)| c);
    if cell == *last_hex_cell.borrow() {
        return;
    }
    *last_hex_cell.borrow_mut() = cell;

    let mut r = renderer.borrow_mut();
    match cell {
        Some(c) => {
            // Slightly above the surface (radius 1.0) so the outline never
            // z-fights with the earth mesh.
            let boundary = crate::renderer::hex::boundary_positions(c, 1.01);
            r.update_hex_boundary(Some(&boundary));
            r.set_hex_outline_alpha(1.0);
        }
        None => {
            r.set_hex_outline_alpha(0.0);
        }
    }
}

/// Click/tap handling shared by mouse and touch: a data marker always
/// wins (opens its dossier, unchanged existing behavior); an empty-globe
/// click on a valid H3 cell instead drills the camera in — each click
/// reveals progressively finer hexes since resolution tracks distance.
fn click_select_or_drill(
    x: f32,
    y: f32,
    canvas: &HtmlCanvasElement,
    renderer: &Rc<RefCell<GlobeRenderer>>,
    pickables: &Rc<RefCell<Vec<PickableMarker>>>,
    globe_state: &GlobeState,
) {
    let selected = pick_selected_at(x, y, canvas, renderer, pickables);
    if selected.is_some() {
        globe_state.selected.set(selected);
        return;
    }
    if let Some((_cell, hit)) = hex_hit_at(x, y, canvas, renderer) {
        let mut r = renderer.borrow_mut();
        let target_distance = crate::renderer::hex::drill_target_distance(r.camera.distance);
        r.camera.fly_to(hit, target_distance);
    }
}

fn update_renderer_data(
    renderer: &Rc<RefCell<GlobeRenderer>>,
    pickables: &Rc<RefCell<Vec<PickableMarker>>>,
    data_state: &DataState,
    globe_state: &GlobeState,
) {
    let layers = globe_state.active_layers.read();
    let mut markers = Vec::new();
    let mut picks = Vec::new();
    let mut arc_datas = Vec::new();

    if layers.contains(&Layer::Geothermal) {
        let nodes = data_state.geothermal_nodes.read();
        for node in nodes.iter() {
            let pos = geo::lat_lon_to_xyz(node.lat, node.lon, 1.005);
            let size = geo::marker_size_from_capacity(node.capacity_mw) * 2.5;
            markers.push(MarkerInstance {
                position: pos,
                color: Vec3::new(0.937, 0.267, 0.267),
                size,
                marker_type: 1.0,
            });
            picks.push(PickableMarker {
                position: pos,
                info: HoverInfo::GeothermalNode(node.clone()),
                selected: SelectedItem::GeothermalNode(node.clone()),
            });
        }
    }

    let timeline_year = globe_state.timeline_year.get();

    if layers.contains(&Layer::ResontiaVaults) {
        let vaults = data_state.resontia_vaults.read();
        // Vaults appear progressively: first at year 50, all 12 by year 150
        let max_vaults = match timeline_year {
            0..=49 => 0,
            50..=74 => 2,
            75..=99 => 4,
            100..=124 => 7,
            125..=149 => 10,
            _ => 12,
        };
        for (idx, vault) in vaults.iter().enumerate() {
            if idx >= max_vaults {
                break;
            }
            let pos = geo::lat_lon_to_xyz(vault.lat, vault.lon, 1.005);
            let color = if timeline_year >= 150 {
                Vec3::new(0.204, 0.827, 0.600) // operational (green)
            } else {
                Vec3::new(0.984, 0.749, 0.141) // under construction (amber)
            };
            markers.push(MarkerInstance {
                position: pos,
                color,
                size: 0.04,
                marker_type: 2.0,
            });
            picks.push(PickableMarker {
                position: pos,
                info: HoverInfo::ResontiaVault(vault.clone()),
                selected: SelectedItem::ResontiaVault(vault.clone()),
            });
        }
    }

    if layers.contains(&Layer::TerraLumina) {
        let sites = data_state.terra_lumina_sites.read();
        for site in sites.iter() {
            let pos = geo::lat_lon_to_xyz(site.lat, site.lon, 1.005);
            markers.push(MarkerInstance {
                position: pos,
                color: Vec3::new(0.655, 0.545, 0.984),
                size: 0.032,
                marker_type: 3.0,
            });
            picks.push(PickableMarker {
                position: pos,
                info: HoverInfo::TerraLuminaSite(site.clone()),
                selected: SelectedItem::TerraLuminaSite(site.clone()),
            });
        }
    }

    if layers.contains(&Layer::Energy) {
        let sites = data_state.sites.read();
        for site in sites.iter() {
            let pos = geo::lat_lon_to_xyz(site.lat, site.lon, 1.005);
            let c = site.energy_type.rgb();
            let size = geo::marker_size_from_capacity(site.capacity_mw);
            markers.push(MarkerInstance {
                position: pos,
                color: Vec3::new(c[0], c[1], c[2]),
                size,
                marker_type: 0.0,
            });
            picks.push(PickableMarker {
                position: pos,
                info: HoverInfo::Site(site.clone()),
                selected: SelectedItem::Site(site.clone()),
            });
        }
    }

    // Earth region markers (large, Phi-colored)
    if layers.contains(&Layer::Regions) {
        let regions = data_state.earth_regions.read();
        for region in regions.iter() {
            let pos = geo::lat_lon_to_xyz(region.lat, region.lon, 1.005);
            // Color by Phi: low=blue, mid=cyan, high=gold
            let phi = region.phi_mean as f32;
            let color = Vec3::new(
                phi * 2.0,       // red increases with Phi
                0.4 + phi * 0.8, // green grows
                1.0 - phi * 0.5, // blue fades with Phi
            );
            // Size by population (logarithmic)
            let size = (region.population_m as f32).ln() * 0.005 + 0.015;
            markers.push(MarkerInstance {
                position: pos,
                color,
                size,
                marker_type: 0.0, // use default energy style for now
            });
            picks.push(PickableMarker {
                position: pos,
                info: HoverInfo::EarthRegion(region.clone()),
                selected: SelectedItem::EarthRegion(region.clone()),
            });
        }
    }

    if layers.contains(&Layer::Maglev) {
        let corridors = data_state.maglev_corridors.read();
        // Corridors appear progressively: first at year 25, all by year 200
        let max_corridors = match timeline_year {
            0..=24 => 0,
            25..=49 => 3,
            50..=99 => 6,
            100..=149 => 10,
            150..=199 => 13,
            _ => 15,
        };
        for (i, corridor) in corridors.iter().enumerate() {
            if i >= max_corridors {
                break;
            }
            let from = geo::lat_lon_to_xyz(corridor.from_lat, corridor.from_lon, 1.0);
            let to = geo::lat_lon_to_xyz(corridor.to_lat, corridor.to_lon, 1.0);
            let peak = geo::arc_peak_height(corridor.distance_km);
            let verts = geometry::generate_arc_with_progress(from, to, peak, 32);
            let color = if corridor.submarine {
                Vec3::new(0.0, 0.4, 0.8)
            } else {
                Vec3::new(0.0, 1.0, 0.533)
            };
            arc_datas.push((verts, color, i as f32 * 0.7));
        }
    }

    // Supply chain trade routes (amber arcs)
    if layers.contains(&Layer::SupplyChain) {
        let routes = data_state.supply_routes.read();
        for (i, route) in routes.iter().enumerate() {
            let from = geo::lat_lon_to_xyz(route.from_lat, route.from_lon, 1.0);
            let to = geo::lat_lon_to_xyz(route.to_lat, route.to_lon, 1.0);
            let peak = 0.06 + route.capacity as f32 * 0.03;
            let verts = geometry::generate_arc_with_progress(from, to, peak, 32);
            // Color by transport mode
            let color = match route.mode.as_str() {
                "Maritime" => Vec3::new(0.96, 0.62, 0.04), // amber
                "Air" => Vec3::new(0.8, 0.8, 0.95),        // light blue-white
                "Land" => Vec3::new(0.6, 0.4, 0.1),        // brown
                _ => Vec3::new(0.96, 0.62, 0.04),
            };
            arc_datas.push((verts, color, (i as f32 + 20.0) * 0.5));
        }
    }

    // Climate projects (green markers)
    if layers.contains(&Layer::Climate) {
        let projects = data_state.climate_projects.read();
        for project in projects.iter() {
            let pos = geo::lat_lon_to_xyz(project.lat, project.lon, 1.005);
            // Color by type
            let color = match project.project_type.as_str() {
                "reforestation" | "conservation" => Vec3::new(0.06, 0.73, 0.51), // emerald
                "solar" | "wind" => Vec3::new(0.98, 0.84, 0.0),                  // gold
                "carbon_capture" => Vec3::new(0.0, 0.87, 1.0),                   // cyan
                "ocean" => Vec3::new(0.0, 0.5, 0.8),                             // ocean blue
                _ => Vec3::new(0.06, 0.73, 0.51),
            };
            let size = (project.co2_offset_mt as f32).sqrt() * 0.004 + 0.008;
            markers.push(MarkerInstance {
                position: pos,
                color,
                size,
                marker_type: 0.0,
            });
        }
    }

    // Emergency shelters (red markers)
    if layers.contains(&Layer::Emergency) {
        let shelters = data_state.emergency_shelters.read();
        for shelter in shelters.iter() {
            let pos = geo::lat_lon_to_xyz(shelter.lat, shelter.lon, 1.005);
            markers.push(MarkerInstance {
                position: pos,
                color: Vec3::new(0.94, 0.27, 0.27), // red
                size: 0.018,
                marker_type: 0.0,
            });
        }
    }

    // Health facilities (pink markers)
    if layers.contains(&Layer::Health) {
        let facilities = data_state.health_facilities.read();
        for facility in facilities.iter() {
            let pos = geo::lat_lon_to_xyz(facility.lat, facility.lon, 1.005);
            let size = if facility.beds > 1000 { 0.015 } else { 0.010 };
            markers.push(MarkerInstance {
                position: pos,
                color: Vec3::new(0.93, 0.28, 0.60), // pink
                size,
                marker_type: 0.0,
            });
        }
    }

    // Robotics dispatch (purple markers)
    if layers.contains(&Layer::Robotics) {
        let dispatch = data_state.robotics_dispatch.read();
        for unit in dispatch.iter() {
            let pos = geo::lat_lon_to_xyz(unit.lat, unit.lon, 1.005);
            markers.push(MarkerInstance {
                position: pos,
                color: Vec3::new(0.55, 0.36, 0.96), // purple
                size: 0.020,
                marker_type: 0.0,
            });
        }
    }

    if layers.contains(&Layer::FossilDeposits) {
        let deposits = data_state.fossil_deposits.read();
        for deposit in deposits.iter() {
            let pos = geo::lat_lon_to_xyz(deposit.lat, deposit.lon, 1.005);
            let eroi = sol_atlas_core::economics::compute_eroi(deposit).unwrap_or(5.0);
            let c = sol_atlas_core::economics::eroi_color(eroi);
            let emissive = sol_atlas_core::geo::fossil_emissive_factor(&deposit.status);
            let scale = sol_atlas_core::geo::fossil_scale_factor(&deposit.status);
            let size = sol_atlas_core::geo::marker_size_from_reserves(deposit.proven_reserves_mboe)
                * scale;
            markers.push(MarkerInstance {
                position: pos,
                color: Vec3::new(c[0] * emissive, c[1] * emissive, c[2] * emissive),
                size,
                marker_type: 0.0,
            });
            picks.push(PickableMarker {
                position: pos,
                info: HoverInfo::FossilDeposit(deposit.clone()),
                selected: SelectedItem::FossilDeposit(deposit.clone()),
            });
        }
    }

    if layers.contains(&Layer::Nuclear) {
        let sites = data_state.nuclear_sites.read();
        let nc = Layer::Nuclear.rgb();
        for site in sites.iter() {
            let pos = geo::lat_lon_to_xyz(site.lat, site.lon, 1.005);
            let size = sol_atlas_core::geo::marker_size_from_capacity(site.capacity_mw);
            let brightness = if site.reactor_type.is_smr() {
                1.4_f32
            } else {
                1.0
            };
            markers.push(MarkerInstance {
                position: pos,
                color: Vec3::new(nc[0] * brightness, nc[1] * brightness, nc[2] * brightness),
                size,
                marker_type: 0.0,
            });
            picks.push(PickableMarker {
                position: pos,
                info: HoverInfo::NuclearSite(site.clone()),
                selected: SelectedItem::NuclearSite(site.clone()),
            });
        }
    }

    // Natural events (USGS earthquakes, NASA FIRMS fires, NASA EONET storms,
    // curated volcanoes) share one dataset with a type discriminant, but
    // each gets its own Layer toggle — filter per layer.
    for (layer, event_type) in [
        (Layer::Earthquakes, NaturalEventType::Earthquake),
        (Layer::Fires, NaturalEventType::Fire),
        (Layer::Storms, NaturalEventType::Storm),
        (Layer::Volcanoes, NaturalEventType::Volcano),
    ] {
        if !layers.contains(&layer) {
            continue;
        }
        let c = layer.rgb();
        let events = data_state.natural_events.read();
        for event in events.iter().filter(|e| e.event_type == event_type) {
            let pos = geo::lat_lon_to_xyz(event.lat, event.lon, 1.005);
            let size = (0.008 + (event.magnitude as f32 / 10.0).clamp(0.0, 1.0) * 0.018).max(0.006);
            markers.push(MarkerInstance {
                position: pos,
                color: Vec3::new(c[0], c[1], c[2]),
                size,
                marker_type: 0.0,
            });
            picks.push(PickableMarker {
                position: pos,
                info: HoverInfo::NaturalEvent(event.clone()),
                selected: SelectedItem::NaturalEvent(event.clone()),
            });
        }
    }

    if layers.contains(&Layer::MajorCities) {
        let cities = data_state.major_cities.read();
        let c = Layer::MajorCities.rgb();
        for city in cities.iter() {
            let pos = geo::lat_lon_to_xyz(city.lat, city.lon, 1.005);
            let size = (0.006 + ((city.population as f32 + 1.0).ln() * 0.0016)).clamp(0.008, 0.02);
            markers.push(MarkerInstance {
                position: pos,
                color: Vec3::new(c[0], c[1], c[2]),
                size,
                marker_type: 0.0,
            });
            picks.push(PickableMarker {
                position: pos,
                info: HoverInfo::MajorCity(city.clone()),
                selected: SelectedItem::MajorCity(city.clone()),
            });
        }
    }

    if layers.contains(&Layer::Chokepoints) {
        let points = data_state.chokepoints.read();
        let c = Layer::Chokepoints.rgb();
        for point in points.iter() {
            let pos = geo::lat_lon_to_xyz(point.lat, point.lon, 1.005);
            let size = 0.01 + (point.daily_barrels_m as f32 / 25.0).clamp(0.0, 1.0) * 0.015;
            markers.push(MarkerInstance {
                position: pos,
                color: Vec3::new(c[0], c[1], c[2]),
                size,
                marker_type: 0.0,
            });
            picks.push(PickableMarker {
                position: pos,
                info: HoverInfo::Chokepoint(point.clone()),
                selected: SelectedItem::Chokepoint(point.clone()),
            });
        }
    }

    if layers.contains(&Layer::Infrastructure) {
        let sites = data_state.critical_infrastructure.read();
        let c = Layer::Infrastructure.rgb();
        for site in sites.iter() {
            let pos = geo::lat_lon_to_xyz(site.lat, site.lon, 1.005);
            let size = 0.01 + (site.global_share as f32).clamp(0.0, 1.0) * 0.02;
            markers.push(MarkerInstance {
                position: pos,
                color: Vec3::new(c[0], c[1], c[2]),
                size,
                marker_type: 0.0,
            });
            picks.push(PickableMarker {
                position: pos,
                info: HoverInfo::CriticalInfrastructure(site.clone()),
                selected: SelectedItem::CriticalInfrastructure(site.clone()),
            });
        }
    }

    // Confluence: where 2+ distinct REAL (Observed/Curated) layers
    // co-locate — computed fresh from a snapshot of every real layer
    // currently loaded, independent of which layers are toggled visible.
    // Deliberately not gated by `layers` above: it answers "where do real
    // systems overlap" regardless of what the viewer has chosen to look at
    // right now.
    if globe_state.show_confluence.get() {
        let snapshot = data_state.snapshot();
        use sol_atlas_core::usgs::FeedHealth;
        let usgs_health = data_state.usgs_feed_status.get_untracked().health;
        let eonet_health = data_state.eonet_feed_status.get_untracked().health;
        let freshness = sol_atlas_core::confluence::LiveSourceFreshness {
            usgs_is_current: matches!(usgs_health, FeedHealth::Live | FeedHealth::Degraded),
            eonet_is_current: matches!(eonet_health, FeedHealth::Live | FeedHealth::Degraded),
        };
        let cells =
            sol_atlas_core::confluence::compute_with_live_source_freshness(&snapshot, 2, freshness);
        for cell in &cells {
            let pos = geo::lat_lon_to_xyz(cell.lat, cell.lon, 1.008);
            // Size and brightness scale with how many distinct real
            // systems co-locate — an honest visual echo of the plain
            // `layers.len()` count, not a hidden severity score.
            let weight = (cell.layers.len() as f32 - 2.0).clamp(0.0, 4.0);
            let size = 0.045 + weight * 0.01;
            markers.push(MarkerInstance {
                position: pos,
                // Near-white — converging colors read as white light, and
                // it can't be confused with any single layer's own hue
                // (or with gold, reserved for Φ/vitals elsewhere in the UI).
                color: Vec3::new(0.92, 0.94, 1.0),
                size,
                marker_type: 4.0,
            });
            picks.push(PickableMarker {
                position: pos,
                info: HoverInfo::Confluence(cell.clone()),
                selected: SelectedItem::Confluence(cell.clone()),
            });
        }
    }

    // Build Φ hotspots from Terra Lumina sites for earth shader warping
    {
        let tl = data_state.terra_lumina_sites.read();
        let mut hotspots: Vec<[f32; 4]> = tl
            .iter()
            .take(12)
            .map(|s| {
                let pos = geo::lat_lon_to_xyz(s.lat, s.lon, 1.0);
                [pos.x, pos.y, pos.z, s.score as f32 / 100.0]
            })
            .collect();
        // Pad to 12 if needed
        while hotspots.len() < 12 {
            hotspots.push([0.0, 0.0, 0.0, 0.0]);
        }
        renderer.borrow_mut().set_phi_hotspots(hotspots);
    }

    log::info!(
        "Updated: {} markers, {} pickable, {} arcs",
        markers.len(),
        picks.len(),
        arc_datas.len()
    );

    *pickables.borrow_mut() = picks;
    let mut r = renderer.borrow_mut();
    r.update_markers(&markers);
    r.update_arcs(arc_datas);
}
