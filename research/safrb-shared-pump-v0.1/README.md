# SAFRB shared-pump read-only projection fixture

A renderer-neutral, synthetic projection candidate for Sol-Atlas #106 / #895, linked to Symtropy's frozen SAFRB reference https://github.com/Luminous-Dynamics/symtropy/pull/7543.

The important visual invariant is that two logical route references converge on one physical pump identity. The map must not depict two independently available pumps or infer geographic location from this schematic network.

Projection is read-only. It exposes evidence maturity, the physical-resource alias, lower/upper bounds, time window and claim scope. It has no dispatch, stock mutation, reservation, legal recognition or service-qualification authority.

For a real geospatial adapter, pin exact editions: OGC API - Features Part 1: Core 1.0.1 (read-only core), OGC API - Records Part 1: Core 1.0.0, and JSON-FG Part 1: Core 1.0.0. Features are spatial feature access, Records is metadata catalogue discovery, and JSON-FG is the richer feature/geometry interchange layer; they are not interchangeable. Preserve CRS, datum and source lineage.

## Independent projection checks

The projection fixture carries both the single-window and two-window Symtropy scenarios, per-window capacity/service values, one shared pump identity and an explicit read-only claim boundary.

Run:

```sh
python3 research/safrb-shared-pump-v0.1/verify_read_only_projection.py
```

The test-only verifier checks that exactly one physical pump node exists, both routes refer to it, each time window remains separate, both source digests and certificates are preserved, and the projection cannot claim geographic coordinates or operational authority. Eight hostile mutations cover topology, aliases, coordinate frame, time-window aggregation, verdict promotion and write authority.

This does not implement a live Sol-Atlas UI integration, nor does it validate any real map position or field infrastructure. No CI pass is claimed for this draft head.
