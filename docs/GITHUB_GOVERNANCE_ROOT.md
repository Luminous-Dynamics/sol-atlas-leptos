# GitHub governance root

This repository treats GitHub governance as an external trust root rather than as a repository-local correctness claim.

## Desired organization ruleset

The canonical desired state is `.github/governance/protect-main.json`.

It targets this repository ID (`1195997641`) and `refs/heads/main` with an organization-owned active branch ruleset. The required protections are:

- pull request before merge;
- at least one approving review;
- dismissal of stale approvals on new pushes;
- approval of the latest push by someone other than the pusher;
- required conversation resolution;
- verified commit signatures;
- deletion protection;
- force-push protection;
- strict required status checks;
- `Check` as the required context, sourced from GitHub Actions integration ID `15368`;
- no configured bypass actors.

## Audit

`scripts/verify_github_governance_root.py` is a read-only auditor. It reads all repository-applicable branch rulesets, including inherited rulesets, then reads the full rule definitions and fails closed unless an active organization-owned ruleset satisfies the desired root.

`scripts/test_verify_github_governance_root.py` and `scripts/test_verify_github_governance_root_integration.py` exercise the semantic and control-flow boundaries.

The manual workflow `.github/workflows/audit-governance-root.yml` exists so the live governance state can be checked independently of the normal PR Check lane.

## Current observed state

As of 2026-10-07, the repository's applicable branch-ruleset API response is empty (`[]`). The connected GitHub integration also cannot read legacy branch protection for `main` (HTTP 403: resource not accessible by the integration), so legacy branch protection is not claimed as evidence.

This means the external governance root is **not established yet**. The repository-local auditor and this document do not substitute for creating and independently reading back the organization ruleset.

## Security boundary

The ruleset is intentionally not part of the ordinary `Check` gate. A missing ruleset should remain a visible governance failure, not cause every source-validation run to become semantically self-referential.

GitHub documents that required status checks can be bound to a specific GitHub App, and that rulesets can enforce pull requests, signed commits, deletion protection, and non-fast-forward protection. The repository-level ruleset API can include inherited organization rulesets when `includes_parents=true` is used.

These controls are governance evidence only. They do not establish Cloud IAM, workload identity, or external-effect correctness; those remain separate boundaries in the GCS qualification architecture.