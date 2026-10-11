# Humanity Atlas run consumer v0.1

This package has two distinct validation layers:

- `ScenarioRunEnvelope` validates generic run metadata, required nullable keys,
  declared identity scope, metric bounds, evidence-gated classifications, and
  content-addressed artifact references.
- `technology_tree::import_technology_tree` independently checks the
  source-specific tree definition, exact artifact bytes, manifest and resolved
  input digests, scenario and metric bindings, and the milestone dependency graph.

Neither layer downloads artifact URIs, authenticates a publisher, executes the
simulator, calibrates probabilities, or establishes scientific truth. Passing
means the artifact is structurally valid and its declared cross-layer bindings
are consistent; the classification remains `unvalidated_simulation`.

## Capture and verify one exact artifact

From the canonical Luminous Dynamics monorepo root, capture a real run. The CLI
writes only JSON to stdout, so shell redirection preserves the artifact without
capturing progress diagnostics:

```sh
cargo run --manifest-path mycelix-multiworld-sim/Cargo.toml --release --bin luminous_sim -- --years 150 --seed 42 --format humanity-atlas-json > /tmp/humanity-atlas-run.json
```

From the Symtropy repository, produce the generic envelope over those exact bytes:

```sh
python3 docs/ops/humanity_atlas_scenario_contract_v0_1/adapt_multiworld_output.py /tmp/humanity-atlas-run.json --output /tmp/humanity-atlas-envelope.v0.1.json
```

From the Sol-Atlas repository root, verify the envelope and those *same source
bytes*. Do not pretty-print or otherwise reserialize the source artifact between
these steps; `run_id` and the artifact reference bind its exact byte sequence.

```sh
cargo run --manifest-path sol-atlas-core/Cargo.toml --example verify_humanity_atlas_artifact -- /tmp/humanity-atlas-envelope.v0.1.json /tmp/humanity-atlas-run.json source-simulator-output
```

A successful verifier prints a JSON summary containing the source artifact digest,
tree definition digest, run horizon, survival outcome, milestone count and
dependency count. It explicitly reports that scientific validation is not
established. A failed digest or cross-layer binding exits non-zero.

## Focused qualification commands

Run the producer-backed simulator/importer integration test:

```sh
cargo test --manifest-path mycelix-multiworld-sim/Cargo.toml --test humanity_atlas_roundtrip
```

Run Sol-Atlas's offline importer and envelope unit tests:

```sh
cargo test --manifest-path sol-atlas-core/Cargo.toml
```

Run the Python adapter's fail-closed regression tests from the Symtropy repository:

```sh
cd docs/ops/humanity_atlas_scenario_contract_v0_1
python3 -m unittest -v test_adapt_multiworld_output.py
```

The adapter's JSON Schema integration assertion runs when Python's `jsonschema`
package is installed; otherwise that one test reports a skip. Treat a queued CI
run as pending, not as a pass. Until the commands above have actually completed
and the captured artifact is verified end-to-end, this boundary is not
build-qualified.
