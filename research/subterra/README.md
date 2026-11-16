# Subterra Atlas — planetary subterranean mapping research

Subterra Atlas is the Sol Atlas geospatial/evidence integration for planetary subterranean sites. It complements the Subterra geometry and robotics work in Luminous-Dynamics/symtropy.

## Documents and fixtures

- [Atlas integration specification v0.1](ATLAS_INTEGRATION_v0.1.md)
- [Planetary site schema v0.1](PLANETARY_SITE_SCHEMA_v0.1.json)
- [Synthetic Moon site fixture](fixtures/synthetic-moon-site.json)

## Design rule

Sol Atlas must not treat current Earth latitude/longitude types as universally valid planetary coordinates. Every location requires a body, reference frame, coordinate convention, frame epoch/model, source lineage and uncertainty. Subsurface interiors use a local 3D survey frame explicitly tied to a body-fixed entrance frame.

This increment is research/schema only. It does not change production types, coordinate transforms, renderer layers or existing Earth datasets.
