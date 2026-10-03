# V5 Replay Receipt V1

V5ReplayReceiptV1 is an additive, content-addressed replay artifact for Sol Atlas cultural projection audits.

## Purpose

A V5 audit already proves that:

- the projection resolves to one canonical claim;
- evidence and source closure is admitted by the selected evidence frontier;
- typed CRMinf-aligned argumentation is bound to that claim and frontier;
- ontology mapping resolutions are bound to the same claim, evidence closure, qualification, and frontier;
- the audit semantic hash covers those components.

The replay receipt makes that proof inspectable without asking a consumer to reconstruct the closure from the audit object.

## Receipt contents

The receipt records:

1. the V5 audit semantic hash;
2. projection and canonical claim identity;
3. canonicalized evidence and source identities;
4. typed argumentation identities and their semantic hashes;
5. ontology resolution identities and their resolution hashes;
6. the ordered evidence-frontier lineage and each frontier manifest hash;
7. the verified leaf frontier;
8. the audit's existing qualification;
9. a receipt hash covering the complete receipt.

Evidence/source/argumentation/ontology collections are canonicalized so equivalent ordering does not produce a different receipt identity.

## Validation boundary

from_audit_and_chain first validates the V5 audit against the complete frontier chain. A receipt therefore cannot be created from an audit that is merely structurally valid but not chain-safe.

validate_against_audit_and_chain repeats the chain validation, reconstructs the expected receipt from the validated inputs, and requires exact structural equality plus a valid receipt hash.

The selected frontier must be the verified chain leaf. An audit authored for a child frontier therefore cannot be replayed against a parent-only chain.

## Epistemic boundary

The receipt is descriptive provenance, not an authority mechanism.

It does not:

- assign or upgrade qualification;
- calculate confidence;
- rank competing interpretations;
- infer historical truth;
- treat graph topology as evidence;
- replace the canonical claim/evidence authority outside Sol Atlas.

This separation follows the role of CRMinf as a model for documenting argumentation and inference relationships rather than collapsing provenance and reasoning into an unqualified truth score. CRMinf 1.2.1 explicitly models argumentation as a superclass for inference making, belief adoption, provenance assessment, and meaning comprehension.

## Relationship to external authority

Sol Atlas remains the projection/replay layer. Canonical claims and their evidence authority remain external inputs. The receipt records exactly what Sol Atlas verified and displayed; it does not become a second claim registry.

## Compatibility

This layer is additive to V5. Existing V1-V5 audit contracts remain unchanged. Consumers that do not need replay receipts can continue using the existing audit validation APIs.

## End-to-end reciprocal binding

Replay receipt validation has two layers:

- `validate_against_audit_and_chain` verifies that the receipt reconstructs exactly from the supplied V5 audit and verified frontier chain.
- `validate_against_projection` additionally verifies that the V5 audit itself reconstructs from the exact originating cultural projection and canonical claim before validating the receipt.
- `V5HistoricalReplayReceiptV1::validate_against_projection_at` carries the same end-to-end invariant through a historical prefix receipt selected at an earlier frontier.

This distinction is intentional: content hashes provide tamper evidence for the receipt, while reciprocal validation establishes the semantic derivation edge back to the source projection. The receipt remains descriptive provenance and does not become a new authority layer.
