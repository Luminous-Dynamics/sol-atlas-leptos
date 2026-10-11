# Horizons fixtures

`mars_ssb_tdb_frame_km_s.json` is a **synthetic parser fixture**, not a response
captured from NASA/JPL Horizons. Its response envelope and vector-table text are
authored to exercise the reviewed parser contract deterministically.

The synthetic marker is documented here rather than embedded as an extra key in
the JSON response itself: the payload is intended to model the provider's
response shape, and both Rust and Python validators reject unknown envelope
fields. The values therefore must not be cited as a scientific ephemeris or
promoted as provider evidence.

A genuine provider capture must be saved as a separate immutable packet, verified
against its exact request/response bytes, reviewed, and added as a distinct
fixture with the associated receipt and provenance hashes.
