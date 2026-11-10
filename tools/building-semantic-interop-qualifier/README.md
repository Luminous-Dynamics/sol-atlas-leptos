# Building semantic interoperability qualifier v0.1

This package validates a synthetic-only, version-pinned semantic mapping profile. It does not import, parse, author or certify real IFC, Brick, ASHRAE 223 or Haystack/XETO models.

## Standards registry

- IFC 4.3 / ISO 16739-1:2024: stable test lane.
- Brick 1.4.4: stable test lane.
- Brick 1.5.0-rc1: prerelease lane; cannot be promoted as stable.
- ASHRAE Standard 223: exact publication artifact/version remains to be pinned before use.
- Project Haystack/XETO: exact release remains to be pinned before use.

The fixture term identifiers are synthetic URNs. They are not normative class mappings.

## Mapping rules

An exact-equivalence claim requires the same declared subject, quantity kind, unit, time basis and evidence class, with no declared semantic loss. Related/narrower/broader/composed/derived mappings must state their semantic losses and cannot be promoted as exact. Unit conversion requires an explicit verified conversion rule and remains a declared transformation. Unknown standard releases and unknown/revoked rights fail closed.

## Owners

- Sol-Atlas #704 owns mapping registry, translation conformance and semantic-loss benchmark.
- Sol-Atlas #623 owns common scope binding.
- Mycelix #6025 owns instrument/measurand profile admission.
- Symtropy #1900/#4531 own physical model/room-scale test-cell work.

## Claim ceiling

The Rust oracle validates profile and 40 synthetic mapping cases only. It does not validate a real BIM/BAS model, field installation, calibration, building, safety, controls or operational performance.
