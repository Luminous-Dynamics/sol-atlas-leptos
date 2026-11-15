# Fixture provenance and attribution

The pinned cases are included from the IEEE PES Power Grid Library — Optimal Power Flow (PGLib-OPF):

- Release: v23.07
- Upstream repository: https://github.com/power-grid-lib/pglib-opf
- Immutable upstream release commit: `dc6be4b2f85ca0e776952ec22cbd4c22396ea5a3`

| Local fixture | Upstream path | Git blob SHA-1 |
|---|---|---|
| `pglib_opf_case14_ieee.m` | `pglib_opf_case14_ieee.m` | `2c8260fff0311c902c3d36dcabcbe861576fc97c` |
| `pglib_opf_case24_ieee_rts.m` | `pglib_opf_case24_ieee_rts.m` | `00b024c2e583b82452392514d2c22881c708ba7c` |
| `pglib_opf_case57_ieee.m` | `pglib_opf_case57_ieee.m` | `d8b0c9e8b45265e57f52b7ab96880ad3a874cde5` |

The upstream files retain their original headers and attribution. The case headers identify underlying IEEE benchmark sources and include the notice:

> Copyright (c) 1999 by Richard D. Christie, University of Washington. Electrical Engineering Licensed under the Creative Commons Attribution 4.0 International license.

The fixtures preserve upstream source text and attribution. PGLib-OPF is the source of the copied files; this importer does not claim ownership of the underlying benchmark data. See the upstream repository and its license for the governing terms.

The SHA-1 values are Git blob object identifiers, not authenticity proofs or replacements for the immutable upstream commit reference. They bind the bytes expected by this test harness.
