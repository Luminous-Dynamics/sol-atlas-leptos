# MATPOWER 8.1 DC Reference Differential

This harness compares Sol-Atlas fixed-injection DC power flow against MATPOWER 8.1 rundcpf on the exact same PGLib MATPOWER source files.

## Independent implementations

- Sol-Atlas parses the source with its strict MATPOWER profile, creates a typed source-bound projection, solves with the separate Rust DC reference kernel, and independently audits the resulting receipt.
- MATPOWER 8.1 independently loads the original .m file and solves it with rundcpf.
- The comparator binds both paths to the exact Git blob SHA-1 and checks the corresponding results.

The numerical reference stack is pinned to exact release commits:
- MATPOWER 8.1: 1a828c7af590714499284ec9c81273388c594.
- MP-Opt-Model 5.0: 35e81cdc147436e843d69b143b32ec5bacfa14d2.
- MP-Test 8.1: a918a838ee421af41d64775508a8a6608e75e569.

MATPOWER 8.1 requires MP-Opt-Model and uses MP-Test utilities. The CI workflow checks out and verifies each exact commit and release tag. The reference function resolves every path to an absolute path because MATPOWER's loadcase helper temporarily changes the Octave working directory while evaluating a case file; relative library paths are not stable across that operation.

## Compared quantities

- system base MVA;
- reference-bus identity;
- bus voltage angles, normalized by subtracting the MATPOWER reference-bus angle;
- bus net active injections, including the MATPOWER-calculated reference-bus balancing injection;
- branch endpoint identity and in-service status;
- forward and reverse branch active-power flow (MATPOWER PF/PT).

Frozen comparison tolerances:
- bus angle: 1e-7 degrees;
- active power / branch flow: 1e-6 MW;
- base MVA: 1e-10 MVA.

Any mismatch fails the differential job and is retained in a JSON receipt. Missing rows, duplicate bus/branch identities, version drift, non-finite values and source digest mismatches fail closed.

## Run

With MATPOWER 8.1, MP-Opt-Model 5.0, MP-Test 8.1 and GNU Octave installed:

    cargo build --manifest-path tools/matpower-dc-network-evaluation/Cargo.toml --locked --offline --bins

    octave-cli --quiet --no-gui --eval "repo=pwd; addpath(fullfile(repo,'tools','matpower-reference-differential')); run_matpower_dc(fullfile(repo,'tools','pglib-case-auditor','fixtures','pglib_opf_case14_ieee.m'), fullfile(repo,'/tmp','case14.tsv'), fullfile(repo,'vendor','matpower'), fullfile(repo,'vendor','mp-opt-model'), fullfile(repo,'vendor','mptest'), '$(git hash-object tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m)')"

    tools/matpower-dc-network-evaluation/target/debug/compare-matpower-dc tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m /tmp/case14.tsv

The dedicated workflow runs the same path for IEEE 14, IEEE RTS 24 and IEEE 57 and publishes TSV plus JSON comparison receipts as a workflow artifact.

## Claim boundary

A zero-mismatch result means agreement for the declared fixed-injection DC equations and reported variables on these exact source cases, versions, tolerances and model assumptions. It does not establish AC power-flow equivalence, AC-OPF, generator redispatch feasibility, contingency security, dynamic stability, protection coordination, calibrated field accuracy or operational authorization.

MATPOWER's successful solve is a numerical reference, not independent physical measurement. The Sol-Atlas auditor independently checks the numerical receipt; it does not make the underlying DC approximation an AC or field model.
