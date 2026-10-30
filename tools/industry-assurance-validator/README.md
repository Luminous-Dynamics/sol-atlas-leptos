# Industry Assurance Contract validator

Standalone Rust tool for semantic consistency checks over IEAC v0.1 qualification profiles.

## Run

```sh
cargo test --manifest-path tools/industry-assurance-validator/Cargo.toml --locked
cargo run --manifest-path tools/industry-assurance-validator/Cargo.toml --locked -- \
  schemas/examples/industry-assurance-qualification-profile-synthetic-v0.1.json
```

Optionally provide a second positional path to write the machine-readable report:

```sh
cargo run --manifest-path tools/industry-assurance-validator/Cargo.toml --locked -- \
  path/to/profile.json path/to/report.json
```

Exit status is zero when the profile is internally consistent and one when semantic contract violations are found. Invalid file arguments or unreadable/invalid JSON return status two.

## Minimum mandatory dimensions

For a profile declared `qualified_for_profile`, the validator enforces:
- D0: software conformance, numerical verification and data integrity/rights.
- D1: D0 plus physical validation, uncertainty credibility, generalization, robustness, operational assurance and safety/security/authority.
- D2 and D3: all ten evidence dimensions.

A mandatory dimension must be established and include evidence. D2 additionally requires independent-verifier and rollback-plan artifact roles, plus human or independently tested fallback and a selection/constraint-gating model influence. D3 requires all ten dimensions established, direct-actuation semantics, independently tested fallback, and safety-case, operational-authorization, cybersecurity, rollback and independent-verifier artifact roles.

## What it checks

- Required profile, claim, risk, evidence-vector, artifact, authority and validity fields.
- Ten required evidence dimensions and permitted evidence-state/requirement combinations.
- Mandatory dimensions on a declared qualified profile must be established and carry evidence records.
- Every evidence artifact and operating-envelope source reference must resolve to an artifact binding.
- Artifact identity syntax: 40-character Git commit SHA, 40-character Git blob SHA-1, and optional 64-character content SHA-256. These fields are not interchangeable.
- Duplicate artifact and evidence IDs, repository/path sanity, canonical UTC validity timestamps and ordered review windows.
- Non-qualified profiles are read-only.
- D0 is descriptive only; D1 is advisory and human-reviewed; D2 requires a human or independently tested fallback, independent verifier, rollback plan and bounded scheduling; D3 requires direct-actuation risk semantics, independently tested fallback, all evidence dimensions established, and safety, operational-authorization, cybersecurity, rollback and independent-verifier artifact roles.

## What it does not prove

This validator does **not**:
- implement the complete JSON Schema 2020-12 evaluator;
- fetch repositories or verify remote commit/blob/content digests against actual bytes;
- prove source authenticity, data rights, measurement truth or evidence independence;
- reproduce a physical solver or establish model validation, uncertainty calibration or domain transfer;
- assess the adequacy of a safety case, cybersecurity assessment or operational authorization;
- certify regulatory conformity or grant permission to actuate.

The report always sets `independent_qualification_attested=false`. Even a semantically consistent profile with declared disposition `qualified_for_profile` is reported as `PROFILE_CONTRACT_VALID_QUALIFICATION_NOT_INDEPENDENTLY_ATTESTED`. This is a local contract check, not a qualification authority.

## Benchmark intent

The tests are adversarial contract controls, not a physical-model performance benchmark. They exercise false-promotion paths such as unqualified action permissions, D1 actuation, missing D2 rollback/verifier artifacts, incomplete D3 safety evidence, dangling artifact references, duplicate evidence IDs and invalid validity windows.

The profile schema and validator must evolve together. Any semantic change requires a versioned schema revision, new negative controls, and an independent review of the resulting claim ceiling.
