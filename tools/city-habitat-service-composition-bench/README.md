# Habitat service evidence composition v0.1

## What it does

A small Rust reference oracle composes three independent result families on one canonical evaluation scope:

1. **Restoration** — the service owner's declared continuity and stable-recovery conclusion.
2. **Access potential** — CityMorph's group-specific reachable service capacity, price/eligibility/access envelope and unknowns.
3. **Operational delivery** — City In-Use Outcomes' service-operation metric, source receipt, window and coverage.

It validates a canonical SHA-256 digest over the complete evaluation scope, exact owner/source identities, #623-style common-scope binding receipts, source authority/rights, and valid/recorded/registered/ingested time cuts. It does not recompute any owner model.

## The critical non-equivalence

- A route is not a usable service.
- Reachable capacity is not realized access.
- Available clinic hours are not clinic visits delivered to each person.
- A consented survey is not a representative population denominator.
- A restored asset is not proof of sustained service unless its owner supplies the required current continuity receipt.

The v0.1 output therefore keeps `realized_use_state = NotMeasured` even when all three evidence families pass. It never emits a number of people served or visits unmet. Those require a separate, lawful, scope-bound service-usage evidence owner and a defensible denominator.

## Frozen benchmark

The 24 deterministic cases cover:
- fully bound restoration + group access potential + service operation;
- non-restored and unstable restoration;
- group capacity shortfall, unreachable service and Unknown access;
- service-operation failure, non-comparable observations and inadequate coverage;
- subject/service/population-frame/owner-scope mismatches;
- late ingestion, revoked evidence, restricted rights and unauthorized issuers;
- malformed scope and artifact digests;
- missing/duplicate owner lanes, duplicate metrics and internally inconsistent owner reports.

The expected result objects are frozen. The oracle reports an independent state vector and non-compensatory overall disposition. It has no action authority.

## Current owner boundaries

- Restoration bridge: #657 / PR #667.
- CityMorph access: PR #564.
- In-use outcomes: PR #635.
- Common scope binding: #623 / PR #624.

The fixtures refer to those contract families but are not the owners' live receipts and are not evidence of a real Johannesburg facility.

## Qualification

E0: 24/24 exact known-answer cases; canonical scope hash; deterministic owner-order invariance; adversarial negative controls.

E1: exact-head Rust formatting, locked native tests, Clippy, WASM compile, JSON contract checks and a receipt bound to exact source/corpus digests.

E2: replay actual owner-produced receipts through independently implemented adapters without erasing native identity.

E3: rights-cleared local service, network, population and in-use evidence plus privacy-safe field validation.

## Claim ceiling

Synthetic cross-owner service evidence composition only. Not actual access, realized service use, people served, unmet need, resident outcomes, regulatory compliance, restoration authorization or a qualified Johannesburg district.
