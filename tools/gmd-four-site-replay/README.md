# GMD four-site replay verifier

This is a deliberately isolated, standard-library-only Rust CLI for the May 2024 four-site GIC benchmark. It does not change the Sol-Atlas application or implement a physical GIC model.

## Purpose and claim ceiling

The verifier checks whether the exact public source snapshot, source files, UTC aggregation, timestamp joins, sensor-polarity handling and metrics reproduce the final publisher Table 1 for eight model/site rows. A PASS is **E1 data-pipeline reproduction only**. It is not validation of the TVA or GMU physical models, transformer heating/damage, grid stability, operational resilience, regulatory compliance or safety.

## Requirements

- Rust stable with Edition 2024 support.
- Git available on PATH.
- A local checkout of the public SWERVE-2024-05-10 companion repository at commit 089617756920bcc9df621f7f2299ba0644320cc3.
- The 20 pinned source files must be present in that checkout. No dataset files are copied into this repository.

## Prepare a minimal pinned dataset checkout

The companion repository is large. Use a blob-filtered sparse checkout so Git only materializes the 20 required source files:

```sh
git clone --filter=blob:none --no-checkout https://github.com/lucywilkerson/SWERVE-2024-05-10.git /tmp/SWERVE-2024-05-10
git -C /tmp/SWERVE-2024-05-10 sparse-checkout init --no-cone
git -C /tmp/SWERVE-2024-05-10 sparse-checkout set --no-cone \
  data_original/tva/gic/GIC-measured/gic-bullrun_20240510.csv \
  data_original/tva/gic/GIC-measured/gic-montgomery_20240510.csv \
  data_original/tva/gic/GIC-measured/gic-union_20240510.csv \
  data_original/tva/gic/GIC-measured/gic-widowscreek2_20240510.csv \
  data_original/gmu/tva/site_108137138.csv \
  data_original/gmu/tva/site_708931360.csv \
  data_original/gmu/tva/site_98699542.csv \
  data_original/gmu/tva/site_14133615.csv \
  data_original/tva/gic/GIC-calculated/20240510_BullRunXfrmGIC.dat \
  data_original/tva/gic/GIC-calculated/20240510_MontgomeryGIC.dat \
  data_original/tva/gic/GIC-calculated/20240510_UnionGIC.dat \
  data_original/tva/gic/GIC-calculated/20240510_WidowsCreek2GIC.dat \
  data_original/tva/gic/GIC-calculated/20240511_BullRunXfrmGIC.dat \
  data_original/tva/gic/GIC-calculated/20240511_MontgomeryGIC.dat \
  data_original/tva/gic/GIC-calculated/20240511_UnionGIC.dat \
  data_original/tva/gic/GIC-calculated/20240511_WidowsCreek2GIC.dat \
  data_original/tva/gic/GIC-calculated/20240512_BullRunXfrmGIC.dat \
  data_original/tva/gic/GIC-calculated/20240512_MontgomeryGIC.dat \
  data_original/tva/gic/GIC-calculated/20240512_UnionGIC.dat \
  data_original/tva/gic/GIC-calculated/20240512_WidowsCreek2GIC.dat
git -C /tmp/SWERVE-2024-05-10 checkout 089617756920bcc9df621f7f2299ba0644320cc3
```

The replay treats timezone-free source timestamps as UTC in accordance with the publisher's UT evaluation window. It rejects explicit non-UTC offsets rather than guessing an offset from the machine's local timezone.

## Run

From the Sol-Atlas repository root:

    cargo test --manifest-path tools/gmd-four-site-replay/Cargo.toml --locked
    cargo run --release --locked --manifest-path tools/gmd-four-site-replay/Cargo.toml -- /path/to/SWERVE-2024-05-10

The CLI writes a deterministic JSON receipt to stdout and exits non-zero for a commit mismatch, missing/changed file, invalid or incomplete minute, duplicate timestamp, wrong substation, non-UTC time, unmatched prediction minute or known-answer mismatch. It performs no network calls.

## Integrity properties

- Dataset HEAD must equal the pinned commit.
- All 20 inputs must be regular files with the pinned byte length and Git blob SHA-1.
- The compiled pin constants are checked against the machine-readable replay manifest embedded from docs/research/results/gmd_may2024_four_site_replay_v0_1.json.
- Measured series are aggregated only from exactly 60 unique one-second UTC samples per minute.
- TVA elapsed seconds are interpreted relative to each pinned UTC date and must lie on minute boundaries.
- GMU predictions must carry the exact expected substation ID and UTC minute.
- Joins are exact-minute only; there is no interpolation, forward fill or time shift.
- Raw signed Pearson correlation is preserved. When it is negative, the predicted series is inverted only for residual metrics, matching the publisher's sensor-orientation convention.
- The receipt includes the dataset commit, tool commit, all source blob pins, sample counts, eight metric rows and the known-answer decision.

## Source/data licensing

The companion repository declares MIT, but embedded third-party measurements can retain provider-specific terms. This tool does not redistribute the source measurements. Users must verify the current terms for their intended use and must not publish protected utility data.

## Known limitation

The source-data replay was previously recomputed interactively. This CLI is the independent executable implementation intended to reproduce it. Until it has been run against the complete pinned dataset checkout and the receipt reviewed, its results must be described as pending—not as independently reproduced.
