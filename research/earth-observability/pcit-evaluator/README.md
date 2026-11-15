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
