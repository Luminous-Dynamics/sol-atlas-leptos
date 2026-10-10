# Solar-system model and GPU backend roadmap

Status: design note for the next implementation increments. This is not a claim
that the WebGPU renderer or a full physical solar-system scene is already shipped.

## Verified current state

- The active browser renderer is WebGL2 through `web-sys` in
  `src/renderer/mod.rs`; the application mounts this renderer from the Leptos
  client entry point.
- The current renderer hard-codes six bodies for its globe-adjacent visual
  animation: Sun, Moon, Venus, Mars, Jupiter, and Saturn. It is not a complete
  solar-system model and its circular motion is not an ephemeris.
- `sol-atlas-core::solar_system` is a visual-scale helper that uses
  `f32` animation time and hand-authored angular speeds. Keep it useful for
  decorative animation, but do not treat its output as scientifically
  authoritative coordinates.
- `sol-atlas-core::system_catalog` now defines a renderer-neutral catalogue:
  all eight planets, five recognized dwarf planets, representative major
  satellites, and aggregate layers for small-body populations and spacecraft.
  It also defines a validated, provenance-carrying state-vector contract. The
  catalogue's rounded size/orbit values are for metadata and rough display
  scale only; catalogue membership does not mean that an object is currently
  rendered or has a live ephemeris.

## Solar-system data model

Separate three layers that must not be conflated:

1. **Catalogue metadata**: stable object ID, display name, class, parent, known
   texture, and rounded physical reference values. Population layers (asteroid
   belt, near-Earth objects, comets, Centaurs, Kuiper belt, scattered disc,
   Oort cloud, spacecraft) are query domains, not single bodies.
2. **Ephemeris state**: target ID, centre ID, epoch, time scale, reference
   system and plane, vector correction, position in km, velocity in km/s, and
   provenance. Every imported
   sample must bind to the exact canonical request and raw response with
   SHA-256 digests. Validate finite values and explicit units before the state
   reaches a renderer.
3. **Display transform**: parent-child hierarchy, camera-relative coordinates,
   lens and level-of-detail policy, and explicitly named visual exaggeration.
   It may transform scientific positions for legibility, but must never
   overwrite the underlying state or label an exaggerated view as scale-true.

Use JPL Horizons for authoritative time-specific states rather than expanding
the hand-authored orbit-speed table. For vector queries, explicitly request the
target, centre, epoch range, time scale, reference system and plane,
vector-correction mode, and units. Preserve
those parameters with each captured response: a vector is meaningless without
its centre, frame, and epoch. The new horizons module builds a deterministic
GET URL with explicit target, centre, discrete TDB epochs (each individually
quoted as Horizons requires), reference system and plane, vector correction,
VEC_TABLE=2, CSV output, and KM-S units. It rejects URLs above a conservative
7,500-byte budget so oversized batches route to the official file-based API
instead of dropping the request. `HorizonsRequestPlan` returns either a GET URL
or a file POST plan with deterministic batch input (including
`TABLE_TYPE='VECTOR'`, `VECT_TABLE='2'`, `VECT_CORR='NONE'`, units,
frame/plane, and individually quoted TLIST epochs). The file API uses the
legacy batch syntax: `TABLE_TYPE='VECTOR'` (singular) and the
`VECT_TABLE`/`VECT_CORR` setting names rather than the GET API's
`EPHEM_TYPE='VECTORS'`, `VEC_TABLE`, and `VEC_CORR` parameters. Its canonical request identity
excludes transient multipart boundaries while binding the exact endpoint,
format, form field, and input bytes. See the official
[file API docs](https://ssd-api.jpl.nasa.gov/doc/horizons_file.html).

The GET docs are labelled 1.3 while their JSON examples still show signature
version 1.0. The file API docs are labelled 1.0 while examples show 0.2. The
parser uses separate, narrow version allowlists for each transport; all other
versions fail closed. Catalogue entries carry explicit Horizons COMMAND IDs
(including semicolon-qualified small-body IDs) rather than relying on ambiguous
name lookup. The parser validates labelled column order, target/centre,
reference frame/plane, correction mode, units, epoch order, fields, and finite
values. It does no HTTP. EphemerisProvenance computes SHA-256 over the exact
canonical request identity and raw response bytes; both digests are checked
before returning states. This binds bytes but does not prove provider or
transport authenticity. Do not fetch the network from unit tests. The fixture
is deliberately synthetic.

`scripts/capture-horizons-vector.sh` provides an operator path for one GET
sample: `--print-url` builds the URL offline, and an explicit capture writes the
exact URL, raw response, hashes, receipt, and metadata to a new directory before
schema validation. It resolves its validator relative to the script path, so it
does not depend on the caller's working directory.
`scripts/validate-horizons-vector.py` checks exact table markers, signature,
target/centre, coordinate settings, column order, field count, finite state,
and epoch. Seven offline regression tests cover the synthetic fixture and
failure cases such as target mismatch, unknown signature version, missing
markers, reordered columns, non-finite values, and epoch drift. CI also checks
the default URL byte-for-byte; none of these checks calls Horizons. The capture
script never overwrites a capture or edits the committed fixture. A real
response remains `captured-not-yet-reviewed` until reviewed and promoted as a
separate byte-for-byte fixture.
The provider docs are the contract for the adapter:
- https://ssd-api.jpl.nasa.gov/doc/horizons.html
- https://ssd.jpl.nasa.gov/horizons/manual.html
- https://ssd.jpl.nasa.gov/planets/orbits.html

## GPU decision: WebGPU in the browser, Vulkan on native Linux

The direction should be a shared Rust renderer built on `wgpu`, not two
independent rendering implementations.

- **Browser**: use the browser WebGPU backend where the runtime exposes it.
  Retain the existing WebGL2 renderer as an explicit compatibility fallback
  during migration and for unsupported contexts.
- **Native Linux**: enable `wgpu`'s Vulkan backend and test on a real Vulkan
  adapter. Vulkan is a native backend choice; it is not an API that browsers
  expose directly.
- **Shared core**: scene objects, body hierarchy, camera-relative transforms,
  tile/asset budgets, picking IDs, and ephemeris records belong outside the GPU
  backend. Convert to renderer precision only after subtracting a camera or
  local-system origin to reduce precision loss at planetary distances.

`wgpu` documents support for native Vulkan and for browser WebGPU; its API
is based on WebGPU while exposing cross-platform Rust interfaces:
https://wgpu.rs/doc/wgpu/ . The W3C WebGPU specification is still evolving, so
runtime capability checks and a fallback are required:
https://www.w3.org/TR/webgpu/ .

## Migration gates

1. **Freeze the contracts**: tests for parent resolution, unit-labelled vector
   fields, bad hashes, non-finite state, response-header mismatches, malformed
   markers, unexpected columns, canonical query encoding, and epoch matching. The parser now
   covers the documented JSON envelope/CSV marker shape using a synthetic
   fixture; before claiming provider-format conformance, capture a real
   response for a pinned exact request, verify its metadata and hashes, and add
   that byte-for-byte payload as a separate fixture. Renderer integration is
   still future work.
2. **Add an isolated `wgpu` feature**: do not replace the working WebGL2
   renderer in the same change. Implement adapter/device/surface lifecycle,
   resize, lost/outdated surface recovery, depth, texture loading, and a clear
   backend-selection result.
3. **Compile both targets**: wasm32 browser-WebGPU compile/check plus native
   Linux Vulkan compile/check. Keep the native backend optional on platforms
   where it is unavailable. A compile-only check is not evidence of GPU
   rendering correctness.
4. **Compare deterministic scenes**: stable seed, frozen ephemeris fixture,
   identical camera and display transforms; compare object positions and
   picking IDs numerically, then review captured images for visual parity.
   Record adapter, browser/driver, dimensions, colour format, and commit SHA.
5. **Promote only after evidence**: keep WebGL2 as the fallback until the new
   path passes the exact-head build and capture gates on supported environments.
   Queued checks, an initialized device, or a successful blank frame are not a
   rendering pass.

## Body coverage roadmap

- **Baseline system**: Sun and all eight planets, including Earth as a separate
  object from the Earth globe UI.
- **Natural satellites**: selected major moons first, represented as children
  of their parent body instead of independent heliocentric circles.
- **Dwarf planets**: Ceres, Pluto, Haumea, Makemake, and Eris.
- **Populations**: queryable asteroid/NEO, comet, Centaur, Kuiper-belt,
  scattered-disc, and Oort-cloud layers. Stream bounded result windows rather
  than allocating every known small body in a single frame.
- **Missions and infrastructure**: spacecraft and mission events with explicit
  observation/ephemeris epochs. Lagrange-point markers require a named
  primary system and must not be treated as physical objects with independent
  orbital radii.

The first catalogue increment does not yet display these objects in the
current WebGL2 view. The next implementation should add a distinct full-system
scene mode backed by the catalogue and ephemeris contract, while preserving the
existing Earth-focused globe as a separate view.
