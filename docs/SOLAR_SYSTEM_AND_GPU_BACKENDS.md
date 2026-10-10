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
before returning states. Its provider field must also match the source in the
validated response signature. These checks bind the recorded label to the
response bytes, but do not prove provider or transport authenticity. Do not
fetch the network from unit tests. The fixture is deliberately synthetic.

`scripts/capture-horizons-vector.sh` provides an operator path for one GET
sample: `--print-url` builds the URL offline and `--print-identity` prints the
canonical semantic-plus-transport identity without its trailing newline. Before
printing that identity or fetching, it binds known catalogue IDs to the matching
Horizons COMMAND/CENTER expressions and expected names; aggregate populations
are rejected as point targets/centres. An explicit capture writes both exact
`request.url` and `request.identity` bytes, the raw response, SHA-256 values,
receipt, and metadata to a new directory before schema validation. The request
digest covers the length-delimited identity (stable target/centre IDs, expected
names, and actual URL), while a separate URL digest records the transport URL.
The Rust parser uses the same identity format and checks it alongside the raw
response digest. It resolves its validator relative to the script path, so it
does not depend on the caller's working directory.
`scripts/validate-horizons-vector.py` checks response markers, signature,
target/centre, coordinate settings, column order, field count, finite state,
and epoch. `scripts/verify-horizons-capture.py CAPTURE_DIRECTORY` independently
recomputes the saved request-identity, URL, and raw-response SHA-256 values,
rebuilds the semantic identity from capture metadata, cross-checks COMMAND,
CENTER, TLIST, frame/plane, correction mode, units, table settings and format
against the actual URL, then reruns response-schema checks. It rejects internally
inconsistent packets and known catalogue ID relabeling; hashes still do not
prove that a capture came from an authentic provider. The
capture and verification tools use the shared
`sol-atlas-core/tests/fixtures/horizons/catalogue-bindings.json` manifest for
known IDs, names, and Horizons COMMAND/CENTER values; a Rust test checks every
manifest object against `SOLAR_SYSTEM_CATALOG`. Offline tests cover response
schema failures and capture-packet tampering, including a target relabel attempt
after recalculating the saved hash fields. CI checks the default URL and semantic
request identity byte-for-byte and verifies that known target/centre mismatches
and aggregate-as-point requests fail without network access. The verifier also
checks that provider and signature-version metadata match the raw response,
not just each other in the receipt. The capture script never overwrites a
capture or edits the committed fixture. A real response remains
`captured-not-yet-reviewed` until reviewed and promoted as a separate
byte-for-byte fixture.
For the complete operator sequence from offline preflight through capture,
packet verification, review, and immutable fixture promotion, see
[`docs/HORIZONS_CAPTURE_RUNBOOK.md`](HORIZONS_CAPTURE_RUNBOOK.md).

The provider docs are the contract for the adapter:
- https://ssd-api.jpl.nasa.gov/doc/horizons.html
- https://ssd.jpl.nasa.gov/horizons/manual.html
- https://ssd.jpl.nasa.gov/planets/orbits.html

## Hash-bound samples and scene composition

The parser returns `HashBoundStateVector`, a wrapper whose constructor is kept
inside the parser module. Callers can inspect its `StateVector`, but cannot
manufacture the wrapper through the public API. This narrows the path into the
scene composer: an input must have come through the parser's request-identity,
response-hash, signature, header, epoch, and column checks. It still does not
establish that the bytes came from the genuine JPL service.

`sol-atlas-core::ephemeris_scene::EphemerisScene` now composes those samples
into barycentric states. It recursively adds a body's centre-relative position
and velocity to its centre's resolved state. Missing centre samples, duplicate
targets, reference cycles, aggregate targets/centres, mismatched epochs, and
incompatible time/frame/plane/correction metadata fail closed. The composer
currently accepts only geometric vectors; light-time-corrected vectors are
rejected because they cannot be assumed to form a simple additive centre chain.
Each resolved body carries hashes for the complete centre chain used in
composition. Scene composition does not interpolate or propagate vectors:
every input must report the exact same parsed JDTDB epoch. Even a tiny mismatch
fails closed until a real propagation/interpolation step is implemented.

The renderer boundary is a separate step. `camera_relative_display_position`
subtracts a camera origin while values are still `f64` kilometre coordinates,
then converts the camera-relative result to `f32` only after range checks. A
display scale remains a visualization parameter and never mutates the
scientific state. The current WebGL2 renderer has not yet been wired to this
scene API.

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

The catalogue, query/parser, hash-bound sample, and renderer-neutral scene
composer are now present in the core crate, but these objects are not yet wired
into the current WebGL2 view. The next implementation should add a distinct
full-system scene mode backed by verified state vectors, while preserving the
existing Earth-focused globe as a separate view.
