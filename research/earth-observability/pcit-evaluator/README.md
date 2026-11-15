# PCIT evaluator v0.1

Standalone, offline Rust CLI for the public PCIT metadata-only development corpus.

## Commands

Validate the full case corpus against JSON Schema 2020-12:

```sh
cargo run --manifest-path research/earth-observability/pcit-evaluator/Cargo.toml -- \
  validate \
  --schema schemas/pcit-case-manifest-v0.1.schema.json \
  --corpus research/earth-observability/PCIT_SYNTHETIC_CORPUS_V0_1.json
```

Export the agent-visible metadata view. The export omits case family, truth state/note, injected perturbations and expected dispositions:

```sh
cargo run --manifest-path research/earth-observability/pcit-evaluator/Cargo.toml -- \
  agent-view \
  --corpus research/earth-observability/PCIT_SYNTHETIC_CORPUS_V0_1.json \
  --out /tmp/pcit-agent-view.json
```

Score an agent result envelope:

```sh
cargo run --manifest-path research/earth-observability/pcit-evaluator/Cargo.toml -- \
  score \
  --corpus-schema schemas/pcit-case-manifest-v0.1.schema.json \
  --corpus research/earth-observability/PCIT_SYNTHETIC_CORPUS_V0_1.json \
  --result-schema schemas/pcit-result-envelope-v0.1.schema.json \
  --results /tmp/pcit-results.json \
  --out /tmp/pcit-report.json
```

Use `--out -` to write a report to standard output.

## Security and scientific boundary

- The corpus is public development data, not a blind or held-out set.
- The agent view is metadata-only. It contains no rasters, imagery, observation bytes, sample coordinates or pixel outputs.
- Rights, reference independence, source metadata conflicts and numeric reconciliation are hard gates; agent-authored boolean assertions cannot override them.
- The evaluator's SHA-256 binds results to the exact corpus file bytes.
- A passed synthetic disposition score does not establish source authenticity, physical truth, sensor equivalence, field performance, calibrated uncertainty or intervention impact.
- JSON Schema validation and disposition scoring are separate from the independent replay attestation in #843.

## Development

```sh
cargo fmt --manifest-path research/earth-observability/pcit-evaluator/Cargo.toml -- --check
cargo test --manifest-path research/earth-observability/pcit-evaluator/Cargo.toml
```


## Ecological reference trajectory audit

Validate the dynamic reference ensemble and recovery trajectory schema/corpus:

```sh
cargo run --manifest-path research/earth-observability/pcit-evaluator/Cargo.toml -- \
  reference-validate \
  --schema schemas/ecological-reference-trajectory-v0.1.schema.json \
  --corpus research/earth-observability/ECOLOGICAL_REFERENCE_TRAJECTORY_SYNTHETIC_CORPUS_V0_1.json
```

Export an agent-visible view with the case family and expected audit oracle removed:

```sh
cargo run --manifest-path research/earth-observability/pcit-evaluator/Cargo.toml -- \
  reference-view \
  --corpus research/earth-observability/ECOLOGICAL_REFERENCE_TRAJECTORY_SYNTHETIC_CORPUS_V0_1.json \
  --out /tmp/reference-agent-view.json
```

Audit reference-model qualification and non-compensatory recovery-claim gates:

```sh
cargo run --manifest-path research/earth-observability/pcit-evaluator/Cargo.toml -- \
  reference-audit \
  --schema schemas/ecological-reference-trajectory-v0.1.schema.json \
  --corpus research/earth-observability/ECOLOGICAL_REFERENCE_TRAJECTORY_SYNTHETIC_CORPUS_V0_1.json \
  --out /tmp/reference-audit.json
```

The audit checks included reference-site degradation screening, multiple reference sites, at least three synthetic indicators per attribute for a qualified model, shifting-baseline review, climate/nonstationarity adjustment, alternative succession-path coverage, reference-model supersession, rights/consent-purpose restrictions, six-attribute coverage and non-compensatory treatment of worsening/unobserved attributes.

This audit scores only synthetic metadata contracts. It does not validate ecological field conditions, remote-sensing observations, climate projections, or restoration impact.
