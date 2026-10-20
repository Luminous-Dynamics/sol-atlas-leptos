# Water localization topology split audit v0.1

**Status:** synthetic split-integrity test only. No hydraulic model is run and no leak-localization score is measured.

## Why this is a useful benchmark gate

Randomly splitting correlated simulations can place the same topology, scenario seed, or identical observation signature in both training and test. That can make a model appear to generalize when it has only learned variants of an already-seen network.

This tranche adds an executable precondition for the broader study in [WATER-LOCALIZATION-GEN-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/174): train on one topology family, calibrate on another, and test on a held-out family. It validates split membership before a classifier or localizer is scored.

The 2026 research picture makes that distinction important. A May 2026 unified EPANET/WNTR Net3 benchmark reported over 47,000 simulated scenarios, over 95% binary detection accuracy, and approximately 75–77% Top-1 localization for ensemble baselines; it also evaluated sensitivity to leak magnitude, sensor density and noise: https://www.sciencedirect.com/science/article/pii/S2214714426006720. A separate October 2026 transient-based study found a pronounced simulation-to-experiment gap and improvement after fine-tuning with limited experimental data; that is a different sensing task and cannot be combined with Net3 localization scores: https://www.sciencedirect.com/science/article/pii/S0043135426011395.

## What the evaluator enforces

- All cases in one declared topology family stay in exactly one of train, calibration or test.
- Family lists are non-empty, disjoint and fully represented by cases.
- An exact graph digest cannot be declared under multiple family IDs.
- Graph identity digesting is order-independent for listed nodes/links and endpoint order, but intentionally preserves names.
- A second label-invariant 1-WL-style structural signature ignores node and link IDs and is used to propose candidate cross-split topology-family collisions.
- If a structural-signature candidate crosses split families, the receipt uses `NeedsTopologyReview`, not `Passed`, until a curator adjudicates the candidate.
- Scenario seeds and exact observation signatures cannot cross splits.
- Scenario specification and observation signatures are fingerprinted from canonical JSON.
- Duplicate case IDs, inconsistent fault labels, invalid sensor states, missing readings encoded as zero, unknown nodes and malformed graph references are rejected.
- The CLI emits a receipt containing the raw plan digest, computed fingerprints, split counts, issues and the revision supplied by the workflow.

Topology family IDs remain curator-reviewed identities. The exact topology digest preserves node/link labels, while the new structural fingerprint runs an iterative 1-dimensional Weisfeiler–Lehman-style color refinement over the undirected graph and ignores names. A cross-split structural match is a **review candidate only**: 1-WL signatures are not a complete graph-isomorphism test, and non-isomorphic regular graphs can collide. The audit uses a fixed maximum of 32 refinement rounds to bound work for larger graphs. The bounded signature can therefore produce additional false-positive review candidates on large or symmetric topologies. The audit deliberately stops at `NeedsTopologyReview` rather than asserting that a candidate pair is isomorphic or silently accepting the benchmark for scoring.

This is a conservative new pre-scoring gate, not automatic family assignment. Curators must decide whether each candidate is genuinely the same/related topology and then update the declared family plan accordingly. A family may contain multiple exact graph variants (for example, a reviewed small topology edit or relabeling) as long as all variants remain in one split. The receipt preserves both the label-sensitive exact digest and label-invariant candidate signature so reviewers can reproduce what was compared.

## Frozen toy fixture

The 9-case file contains three cases in each of three topology families. A frozen mutation test adds a renamed copy of the training tree in the test split and requires `NeedsTopologyReview`, while a separate test confirms that exact identity digests remain label-sensitive and structural candidate signatures are label-invariant. Values are hand-authored synthetic pressure deltas and are included only to exercise split auditing (including one missing-sensor state). They are not EPANET outputs and must never be used to claim localization accuracy.

## Next gates

1. Replace the toy family set with generated scenario manifests from the existing EPANET runner, keeping split assignment upstream of feature generation.
2. Add whole-topology holdout and seed disjointness to the actual large corpus; freeze test families before tuning.
3. Add operating-condition, leak-magnitude, sensor-density, noise and missingness shift, each as a separately reported stratum.
4. Score matched physical-model, classical ML and graph-model baselines only after exact data and solver artifacts are available.
5. Keep binary detection, pipe/node localization, graph-distance error, abstention/calibration, false dispatch and field transfer on separate compatible scorecards.

## Benchmark ladder and proposed targets

These are **proposed go/no-go targets**, not results, guarantees, or universal industry standards. The published Net3 figures above are context only; they do not establish a matched held-out-topology baseline.

| Gate | Metric and protocol | Proposed target |
|---|---|---|
| Split integrity | Exact digests, seed reuse, observation signatures, family membership, structural-collision review | Zero unresolved integrity findings in a scored run; 100% deterministic replay from exact manifest bytes |
| Binary detection | Recall at a fixed false-positive rate; AUPRC under declared prevalence and per-topology strata | Stretch target: ≥95% recall at ≤1% false-positive rate on held-out topology families, with confidence intervals |
| Localization | Top-1 / Top-3 node-or-pipe identification on families never used for training or calibration | Stretch target: Top-1 >80% and Top-3 >90%; require a paired comparison on identical cases and a confidence interval over the strongest matched baseline |
| Structural generalization | Difference between random-row and topology-family-held-out performance | Report the gap; target ≥50% reduction versus the strongest matched baseline without increasing false-positive rate |
| Spatial error | Shortest-path distance to the true affected node/pipe, normalized by graph size and reported in physical units where meaningful | Report median, p90 and worst-stratum distance; no one scalar may replace Top-1/Top-3 |
| Confidence and abstention | Risk–coverage curve; calibration by topology family, sensor density and fault magnitude | At ≥80% coverage, target ≤10% selective Top-1 error; also report false-abstention and high-confidence error rates |
| Robustness | Independent shifts in leak magnitude, sensor placement/density, noise, missingness and demand regime | Publish per-stratum results and worst stratum; no overall average may hide a failed stratum |
| Cost | Solver time, training/inference latency, memory, simulations used and calibration-data budget | Compare quality-versus-budget curves on the same hardware and data; disclose preprocessing and tuning budgets |
| Physical validity | Locked physical-bench cases from an independent acquisition, separate from EPANET/WNTR simulations | Establish a field/bench baseline first; report domain gap and failure rate before any deployment claim |

The >80% Top-1 and >90% Top-3 targets are stretch hypotheses inspired by the published 75–77% Net3 ensemble Top-1 context. They are **not directly comparable** until that baseline is reproduced on the same exact topology-family holdout, target labels, sensor layout, fault distribution and tuning budget. A model should be called better only when the paired comparison supports the improvement and the data split has no unresolved structural-review candidates.

The first scorecard must separate: (1) synthetic random split, (2) synthetic topology-family holdout, (3) unseen operating conditions on known topology, and (4) independent physical/field validation. Progress on one tier does not qualify the next.

## Nonclaims

Passing this audit means only that the declared toy split plan meets its explicit structural leakage checks. It is not a hydraulic validation, benchmark reproduction, state-of-the-art comparison, calibrated model, field experiment or utility decision recommendation.
