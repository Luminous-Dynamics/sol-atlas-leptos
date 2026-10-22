# SA-WORKFORCE-CHAIN-CONFORMANCE-001 — End-to-end evidence-chain conformance v0.1

**Tracking issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/282  
**Composes with:** diagnosis #268, source profile #272, outcome evaluation #277, active evidence acquisition #279 and source robustness #281.  
**Architectural precedent:** Sol-Atlas cross-repository landscape conformance #273.  
**Status:** synthetic chain contract. No real workforce shortage or employment-impact validation.

## Paradigm shift: claims carry proof obligations

A stack of schema-valid documents is not necessarily a valid evidence chain. Each stage must refer to exact, pinned inputs; keep the same intended scope; preserve evidence class and uncertainty; and satisfy the gates for the claim it emits. A later stage must never increase claim strength just because an intermediate record parses.

The workforce evidence chain is:

1. **Source profile:** evidence origin, population, geography, reference period, denominator, freshness and access class.
2. **Active acquisition:** what new evidence was requested, the permission basis, query result and any unresolved/unknown outcomes.
3. **Diagnosis:** competing causes and uncertainty, including the distinction between funded demand and raw vacancy proxies.
4. **Intervention:** action preconditions and a legitimate human authorization boundary.
5. **Outcome:** independently verified service restoration and, where claimed, evidence of payment and job conditions.
6. **Robustness:** source perturbations, alternative mappings, missingness and whether the conclusion is fragile.
7. **Claim decision:** strongest claim justified by the weakest required input/authority gate.

This evaluator owns only the chain manifest, adapter conformance and chain-level negative controls. It does not replace the upstream algorithms or write back credential, authorization, scheduler, work-order or payment state.

## Required invariants

- Artifact digest and schema digest must be valid and the observed digest must match the manifest's pinned digest. A stale input fails closed.
- Every required stage appears exactly once, and every required adjacent link joins the intended stage IDs in order.
- Linked stages must agree on case scope, task scope, geography, observation period and decision cutoff. Units, denominators and any normalization basis remain explicit; relevant comparison logic must enforce their comparability.
- A query contributes evidence only if its ID appears in the permitted set and authorization is positive. Unknown or denied permission cannot support the claim.
- Synthetic-only inputs impose a hard ceiling of `synthetic_contract_only`; downstream artifacts cannot relabel them as observed, verified outcomes or causal evidence.
- Unknown, stale, contradictory or missing is distinct from observed zero, confirmed absence, a failed action and successful completion.
- A service-restoration claim requires independent verification; work-order closure is not sufficient by itself.
- A paid-work claim requires the declared payment evidence class; training completion or a job offer does not suffice.
- A causal paid-work claim additionally requires a net-additionality estimate and explicit displacement, substitution, deadweight and spillover assessment. Gross paid events cannot silently become net jobs or hours created.
- A causal-impact claim requires a registered credible comparison design, a preregistered primary outcome and a declared counterfactual basis, in addition to adequate outcome evidence.
- A SHA-256 digest proves byte identity, not consent, permission, competence, authorization or truth.
- A failed or incomplete chain must state a machine-readable reason and demote the claim to its supported ceiling; no silent auto-repair or ranking.

## Claim-strength lattice

The strength vocabulary is intentionally small:

- `synthetic_contract_only`: validates data shape and internal invariants on synthetic fixtures only.
- `descriptive_association_only`: summarizes authorized observed evidence without causal attribution.
- `verified_operational_outcome`: independently verifies a defined operational result.
- `causal_impact`: reports an effect supported by a credible preregistered comparison design and outcome follow-up.

Not every claim is interchangeable: a paid-work claim has an additional payment-evidence gate even if its overall strength is not higher than an operational outcome. Gates are conjunctive; a higher strength is not earned by averaging scores across stages.

### Digest semantics in v0.1

The verifier now reads each synthetic stage artifact and its referenced stage schema from the local working tree, calculates SHA-256 over the actual bytes, compares actual bytes against both the reported and pinned digests, validates the artifact against its stage schema, and verifies stage ID/type/evidence-class/claim-ceiling identity. The fixture includes separate negative controls for stale pins and byte-valid but wrong-stage artifacts. The paths are restricted to the synthetic artifact directory and one known schema path; verification performs no network reads.

The current artifacts are deliberately synthetic stand-ins for upstream adapter payloads; passing these tests establishes byte-level integrity for these fixture files, not integrity or truth of the real upstream PR artifacts. A production adapter still needs to pin and verify actual upstream repository revisions and artifacts. The claim ceiling remains synthetic-contract-only.

## Negative controls

The v0.1 corpus must reject:
- mismatched observed versus pinned artifact digest;
- mismatched task scope, geography, observation period or decision cutoff;
- a missing or duplicate required stage, or broken/stale stage link;
- a used query absent from the permitted set, or unknown/denied authorization;
- a downstream claim stronger than the weakest stage's claim ceiling;
- synthetic inputs relabelled as observed or causal;
- service restoration without an independent verification status;
- paid work without payment evidence;
- a causal claim without credible preregistration and counterfactual basis.

## Metrics and acceptance gates

- 100% detection of frozen negative controls, with exact expected failure codes.
- Zero silent claim promotion; zero synthetic-to-observed upgrades.
- Zero authorization bypass, missing payment proof on paid-work claims, and unverified service outcomes represented as verified.
- 100% of required stages and links pinned by exact digests on conformant cases.
- Deterministic result under artifact input-order permutation.
- Explicit weakest-link claim ceiling and no omitted failure denominator.
- The harness performs no network reads at evaluation time; its manifest pins the artifacts before running.

## Research basis

- NIST, *TEVV-Athlon Framework for Evaluating AI Systems*: https://www.nist.gov/artificial-intelligence/ai-research/tevv-athlon-framework-evaluating-ai-systems
- Aronsson, Rahbar & Chehreghani, *A Survey on Active Feature Acquisition Strategies*, IJCAI 2026: https://www.ijcai.org/proceedings/2026/860
- Sol-Atlas landscape artifact-chain conformance #273: https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/273

## Claim ceiling

The chain verifier tests synthetic artifact conformance. It does not validate actual South African labour-market source representativeness, establish a shortage, prove real service improvement, create jobs, or establish causal impact/SOTA superiority. Those claims require authorized observations, a legitimate data steward and independent evaluation.
