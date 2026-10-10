# Sol Atlas Rendering and Solar-System Architecture

**Status:** researched implementation decision / staged proposal  
**Research date:** 2026-10-10  
**Scope:** shared GPU rendering strategy, astronomical object model, ephemeris provenance, and staged validation

## Decision in one paragraph

Move toward a shared Rust renderer built on `wgpu`, not separate independent WebGPU and Vulkan implementations. In the browser, use WebGPU where available and preserve a tested WebGL2 path for browsers/devices where it is unavailable or fails initialization. For native Linux, let `wgpu` use Vulkan where available, with a tested fallback. Do not replace the production renderer until a bounded prototype demonstrates parity, resource lifecycle correctness, and measurable performance on the supported target set. Independently, replace duplicate hand-written solar-body lists and stylized free-running circular positions with one renderer-neutral object catalogue and time/frame-explicit ephemerides. Preserve an explicitly labeled presentation-scale transform so the full system remains explorable without presenting distorted display positions or radii as physical truth.

## What the current code actually does

Inspected on the repository's `main` branch on 2026-10-10:

- The root package is a Leptos/WASM application described as a WebGL globe. Its `web-sys` feature list includes `WebGl2RenderingContext`, and `src/renderer/mod.rs` imports it as `GL`; shader programs, buffers, textures, framebuffers, and draws are managed through direct WebGL2 calls.
- The renderer hard-codes a Sun, Moon, Venus, Mars, Jupiter and Saturn in a local `CelestialBody` vector. A separate `create_celestial_vao` also hard-codes six coloured point markers. These lists can drift apart.
- `sol-atlas-core/src/solar_system.rs` defines a different public `CelestialBody` model and returns six records: Sun, Moon, Venus, Mars, Jupiter and Saturn. Its `body_position` is a circular trigonometric animation driven by seconds and a hand-set angular speed/offset. The function does not accept a date, time scale, reference frame, centre, ephemeris source, or uncertainty.
- Earth is the globe itself, but the current solar-system catalogue does not contain Mercury, Uranus, Neptune or dwarf planets such as Pluto, Eris, Haumea, Makemake and Ceres. It also is not a catalogue of natural satellites, asteroids, comets or spacecraft.

Therefore the current bodies are an artistic/stylized visualization, not an ephemeris-accurate solar-system simulator. This is fine as an explicitly stylized starting view, but its output must not be described as real-time or physically accurate positions.

## Rendering backend decision

### Recommended direction: one renderer API, multiple backends

Rust's `wgpu` is the best first experiment because it provides one safe Rust API across native graphics backends and the browser. Its documented native backends include Vulkan, Metal, Direct3D 12 and OpenGL; on WebAssembly it supports browser WebGPU and a WebGL2 path. It is also the graphics API used by Bevy. This enables substantial reuse of render data, resource ownership and WGSL shaders, although browser and native initialization, surface handling, feature sets and packaging still need target-specific code.

- **Browser fast path:** request WebGPU in a secure context when the browser supports it and the adapter/device can be initialized with the required baseline features.
- **Browser compatibility path:** retain WebGL2 in production until a real browser matrix proves the `wgpu` WebGL backend, or the existing direct-WebGL renderer as an interim fallback. Treat unsupported features as a normal capability decision, not as a crash.
- **Native Linux path:** request Vulkan through `wgpu` where available. A native OpenGL/GLES fallback is useful where the runtime/driver configuration requires it. Native Vulkan is not an API a web page can call directly.
- **Shared logic:** projection, scene graph, body transforms, tile identity/selection, source provenance, camera-independent view state, and frame snapshots belong outside backend-specific code.
- **Backend-owned work:** GPU resources, bind groups, render pipelines, surface configuration, device/context loss, command submission, and shader compilation stay behind the renderer boundary.

Official `wgpu` documentation:
- Crate overview and backend support: https://wgpu.rs/doc/wgpu/
- Platform-specific WebGPU / WebGL2 builds: https://wgpu.rs/doc/wgpu/documentation/platforms/web/
- Feature flags and backend configuration: https://wgpu.rs/doc/wgpu/documentation/features/
- Browser API support and secure-context requirement: https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API

WebGPU remains limited-availability rather than universally supported. Do not remove the WebGL2 path based on one Chromium/Linux development machine.

### Staged migration (no big-bang rewrite)

1. **Freeze a renderer baseline.** Capture deterministic screenshots and interaction recordings for the existing Earth sphere, lighting/cloud/night textures, marker/arc layers, atmosphere, bloom, camera controls and context-loss behavior. Record startup-to-first-frame, frame-time distribution, GPU/CPU memory where available, shader/program count and texture residency on named devices/browsers.
2. **Define a narrow renderer contract.** Keep domain and astronomical data free of `web_sys` objects. Add explicit scene/view inputs and renderer-owned lifecycle APIs; do not leak `WebGlTexture` or `wgpu::Texture` into `sol-atlas-core`.
3. **Build a small `wgpu` proof.** Render one sphere, one texture, one camera and one selected object with WGSL. First prove the WebAssembly WebGPU path and native Linux Vulkan path independently. Verify whether a combined WebGPU + WebGL WASM artifact works reliably with the selected pinned `wgpu` release; if not, use distinct browser build variants or the existing WebGL2 fallback rather than claiming one artifact can cover all devices.
4. **Compare, don't assume.** Render the same frozen scene in the current WebGL2 implementation, browser `wgpu`, and native Vulkan. Compare projection, texture orientation, depth ordering, colour/tone mapping, picking, resize behavior, surface/device loss, and deterministic frame inputs. Do not remove the old renderer until the replacement passes the predeclared gate.
5. **Migrate capabilities in slices.** Start with sphere + textures + camera, then markers/arcs and picking, then atmosphere/post-processing, then camera-driven tile imagery. Keep the existing full-globe Earth texture as a fallback while tile scheduling, decoding, cache eviction, texture memory limits, parent-tile fallback, stale-request cancellation and attribution are proven.
6. **Pin and report the toolchain.** Pin the selected `wgpu` version and Rust toolchain; avoid copying feature flags from the moving `latest` docs without checking that version. Keep per-target CI separate and report results for the exact commit and build feature set.

The renderer should log the selected backend and a bounded summary of adapter capabilities. No user data or sensitive source URLs should be emitted in diagnostics. A request to use an unsupported GPU feature must fail closed to an explicitly tested path, not silently render a different scientific result.

## Solar-system model: catalogue first, renderer second

### One authoritative, renderer-neutral catalogue

Replace duplicate in-renderer lists and the core's toy-only body definitions with a versioned catalogue model. Each record should carry only values supported by a source, with optional fields left unknown rather than filled with aesthetic guesses.

Suggested minimum fields:

- stable object identifier and identifier namespace (for example, JPL Horizons designation / NAIF SPICE ID); display name and aliases;
- object class: star, planet, dwarf planet, natural satellite, asteroid, comet, spacecraft, dynamical point, or another explicitly supported class;
- parent/primary and reference centre where applicable; hierarchy must permit moons of planets and small-body satellites without flattening all objects into an unstructured list;
- known physical characteristics with units and source attribution, separated from display radius/colour/texture choices;
- source record, observation/computation provenance, retrieval/capture time, licence/usage terms for any texture or shape model, and validation state;
- optional geometry/texture assets and source credits; missing texture data must not make the object impossible to catalogue or crash scene initialization;
- catalogue/schema version and deterministic identity rules.

The model should not claim an image or material is authoritative body data merely because a file exists in the assets directory.

### Ephemeris is not an animation speed

A position query must make the following inputs explicit:

- target object identifier;
- epoch and time scale (do not silently conflate UTC with dynamical time);
- origin/observer centre and reference frame;
- position/velocity units and coordinate convention;
- ephemeris/solution source and version or query parameters;
- validity interval and available uncertainty/quality information.

The renderer consumes a timestamped, validated scene snapshot; it does not query an external service every frame. The acquisition adapter can request positions from JPL Horizons, validate and normalize the response, and cache a bounded snapshot with its query/source metadata. SPICE kernels are the stronger companion path for higher-fidelity mission geometry, orientation/lighting and offline replay when the appropriate kernels are available. Kernel availability and coverage differ by object/mission and must be recorded, not assumed.

- JPL Horizons API: https://ssd-api.jpl.nasa.gov/doc/horizons.html
- JPL Horizons object lookup (planets, satellites, spacecraft, asteroids, comets): https://ssd-api.jpl.nasa.gov/doc/horizons_lookup.html
- NASA/JPL NAIF SPICE concept and kernel model: https://naif.jpl.nasa.gov/naif/spiceconcept.html
- Horizons system manual and stated catalogue scope: https://ssd.jpl.nasa.gov/horizons/manual.html

Horizons is a source/query service, not a licence to bulk-fetch indefinitely or a guarantee that every object has ephemerides over every requested interval. Bound and cache requests; make the source and retrieval time visible; preserve errors, unavailable intervals and unresolved object names as explicit states. Freeze representative source responses for deterministic tests.

### Separate scientific coordinates from presentation coordinates

A single linear solar-system view is not useful for both planetary detail and the Kuiper belt. Provide distinct, visibly labeled navigation modes, for example:

1. **System overview:** non-linear/compressed distances and exaggerated body radii for legibility.
2. **Planetary system:** a chosen planet and its satellites, with selectable scale and optional orbit tracks.
3. **Observer/ephemeris view:** positions at a selected time/observer with the reference frame and source identified.
4. **Small-body explorer:** search/filter asteroid, comet, spacecraft and other catalogue entries; render only the viewport-relevant or user-selected set.

All display transforms must be applied at the presentation boundary. Preserve source physical values and ephemeris coordinates unchanged. The inspector should distinguish real physical radius and position from exaggerated display radius or logarithmically compressed distance. Never label simulated orbital motion as observed or up-to-date when no ephemeris query/snapshot backs it.

For GPU precision, keep astronomical positions in an appropriate high-precision CPU representation until applying a documented origin-relative transform. Convert to GPU-friendly local coordinates close to the renderer boundary; avoid feeding very large absolute coordinates directly to `f32` vertex positions and then hiding resulting jitter.

### Progressive catalogue coverage

“Every body” should mean every supported and discoverable body can be catalogued and searched—not that every object is rendered at once.

- **Launch baseline:** Sun, all eight planets, Earth’s Moon, and obvious parent relationships.
- **Next layer:** the five dwarf planets recognized by NASA's public reference page (Ceres, Pluto, Haumea, Makemake and Eris), major planetary satellites, ring systems and accessible shape/texture sources.
- **On-demand layer:** other natural satellites, asteroids, comets, near-Earth objects, selected spacecraft, and useful dynamical points, sourced via indexed queries.
- **Large-catalogue layer:** filtered/batched search and hierarchical visibility/LOD. Apply explicit request, memory, object-count and time-window budgets. Do not try to load or draw the whole small-body inventory as a single frame payload.

NASA/JPL's current references describe eight planets and five officially recognized dwarf planets. Horizons supports queries across planets, natural satellites, spacecraft, asteroids and comets; its catalogue is far larger than a feasible always-visible render set. Source scope, not a handcrafted array, should define what Sol Atlas can discover.

General reference: https://science.nasa.gov/solar-system/planets/

## Acceptance criteria

### Renderer

- Current and experimental renderers consume the same frozen, renderer-neutral scene input.
- Browser WebGPU, browser WebGL2 fallback, and native Vulkan selection each have separately recorded build and runtime outcomes.
- No claim of a passing target is based on a queued, skipped, missing, or cancelled check.
- Surface/context/device loss and failed adapter acquisition produce a recoverable, observable state.
- Textures, image requests, caches and in-flight work have explicit budgets and cancellation/eviction rules.
- A documented before/after comparison covers visual parity, frame times, memory, startup and supported-device matrix.

### Solar system

- The catalogue contains at least the eight planets and Sun, with Earth linked to the globe representation instead of duplicated as an unrelated visual body.
- The five listed dwarf planets are represented as catalogue entries; other moons/small bodies are discoverable by identifier and source query without requiring them in the initial render.
- Ephemeris requests explicitly record epoch, time scale, target, centre, frame, units and source; an artistic orbit cannot pass as an ephemeris result.
- Frozen inputs produce deterministic catalogue normalization and deterministic rendered placement.
- Missing texture/shape/ephemeris data remains explicit and does not suppress a catalogue record.
- Physical data, display scaling, rendering material and source/licence metadata remain separate.
- Tests cover object-ID aliases, parent/child hierarchy, missing data, unsupported time ranges, frame/origin conversion, source failures, and scale-transform invariants.

## Non-claims

This document records a researched direction, not an implementation or benchmark result. Sol Atlas has not yet been demonstrated to have a working `wgpu` renderer, WebGPU/WebGL fallback, Vulkan native application, JPL Horizons importer, SPICE integration, or comprehensive solar-system catalogue. Those claims require code, exact-target builds, and evidence from the tests above.
