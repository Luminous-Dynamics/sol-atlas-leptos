# Protected qualification environment

The credentialed GCS qualification job uses the GitHub environment `sol-atlas-gcs-qualification`. The workflow file alone does not prove that the environment is protected.

## Desired state

The canonical desired state is `.github/governance/protected-qualification-environment.json`.

The environment should require at least one reviewer, prevent self-review, and use an explicit custom deployment branch policy permitting exactly `main`.

`scripts/verify_github_qualification_environment.py` reads the environment configuration and its deployment branch policies through GitHub's read-only REST endpoints. It fails closed on missing or malformed protection state.

`scripts/test_verify_github_qualification_environment.py` and `scripts/test_verify_github_qualification_environment_integration.py` exercise the semantic and control-flow boundaries.

The manual workflow `.github/workflows/audit-qualification-environment.yml` runs the auditor with only the normal read-only `GITHUB_TOKEN`. It deliberately does not accept a repository secret containing an administrator token, because mutable repository workflow code must never be trusted with a governance-admin credential.

For an audit that needs privileged visibility beyond the default workflow token, run the script outside repository-controlled automation with an independently governed GitHub token.

## Security relationship

The environment is a separate credential-release gate from the main-branch governance ruleset. The intended sequence is:

`organization governance -> main branch -> protected qualification environment -> OIDC credential materialization -> cloud trust verification -> external effect qualification`

A failure at any earlier boundary must remain fail-closed.

GitHub documents that required reviewers can gate workflow jobs referencing an environment, that self-review can be prevented, and that custom deployment branch policies can restrict which branch names may deploy to an environment. The branch-policy API exposes the configured patterns for read-back.

These controls prove GitHub configuration only. They do not prove reviewer intent, Cloud IAM state, or external-effect correctness.