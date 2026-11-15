# MATPOWER 8.1 DC Reference Differential

This harness compares Sol-Atlas fixed-injection DC power flow against MATPOWER 8.1 \`rundcpf\` on the exact same PGLib MATPOWER source files.

## Independent implementations

- Sol-Atlas parses the source with its strict MATPOWER profile, creates a typed source-bound projection, solves with the separate Rust DC reference kernel, and independently audits the resulting receipt.
- MATPOWER 8.1 independently loads the original \`.m\` file and solves it with \`rundcpf\`.
- The comparator binds both paths to the exact Git blob SHA-1 and checks the corresponding results.

MATPOWER is pinned to commit \`1a828c7af590714499284e36ee9c81273388c594\`, the official MATPOWER 8.1 release tag. The workflow checks out and verifies that exact commit and tag before running the comparison.

## Compared quantities

- system base MVA;
- reference-bus identity;
- bus voltage angles, normalized by subtracting the MATPOWER reference-bus angle;
- bus net active injections, including the MATPOWER-calculated reference-bus balancing injection;
- branch endpoint identity and in-service status;
- forward and reverse branch active-power flow (MATPOWER PF/PT).

Frozen comparison tolerances:
- bus angle: \(10^{-7}\) degrees;
- active power / branch flow: \(10^{-6}\) MW;
- base MVA: \(10^{-10}\) MVA.

Any mismatch fails the differential job and is retained in a JSON receipt. Missing rows, duplicate bus/branch identities, version drift, non-finite values and source digest mismatches fail closed.

## Run

With MATPOWER 8.1 and GNU Octave installed:

    cargo build --manifest-path tools/matpower-dc-network-evaluation/Cargo.toml --locked --offline --bins
    octave-cli --quiet --no-gui --eval "addpath('tools/matpower-reference-differential'); run_matpower_dc('tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m', '/tmp/case14.tsv', '/path/to/matpower-8.1', '$(git hash-object tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m)')"
    tools/matpower-dc-network-evaluation/target/debug/compare-matpower-dc tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m /tmp/case14.tsv

The dedicated workflow runs the same path for IEEE 14, IEEE RTS 24 and IEEE 57 and publishes the TSV plus JSON comparison receipts as a downloadable workflow artifact.

## Claim boundary

A zero-mismatch result means agreement for the declared fixed-injection DC equations and reported variables on these exact source cases, versions, tolerances and model assumptions. It does not establish AC power-flow equivalence, AC-OPF, generator redispatch feasibility, contingency security, dynamic stability, protection coordination, calibrated field accuracy or operational authorization.

MATPOWER's successful solve is a numerical reference, not independent physical measurement. The Sol-Atlas auditor independently checks the numerical receipt; it does not make the underlying DC approximation an AC or field model.
