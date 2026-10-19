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
- Graph digesting is order-independent for nodes/links and link endpoint order.
- Scenario seeds and exact observation signatures cannot cross splits.
- Scenario specification and observation signatures are fingerprinted from canonical JSON.
- Duplicate case IDs, inconsistent fault labels, invalid sensor states, missing readings encoded as zero, unknown nodes and malformed graph references are rejected.
- The CLI emits a receipt containing the raw plan digest, computed fingerprints, split counts, issues and the revision supplied by the workflow.

Topology family IDs are curator-reviewed identities: the exact graph digest preserves node/link labels and cannot discover every graph-isomorphic or near-duplicate relabeling. Curators must group known variants before running the audit.

## Frozen toy fixture

The 9-case file contains three cases in each of three topology families. Values are hand-authored synthetic pressure deltas and are included only to exercise split auditing (including one missing-sensor state). They are not EPANET outputs and must never be used to claim localization accuracy.

## Next gates

1. Replace the toy family set with generated scenario manifests from the existing EPANET runner, keeping split assignment upstream of feature generation.
2. Add whole-topology holdout and seed disjointness to the actual large corpus; freeze test families before tuning.
3. Add operating-condition, leak-magnitude, sensor-density, noise and missingness shift, each as a separately reported stratum.
4. Score matched physical-model, classical ML and graph-model baselines only after exact data and solver artifacts are available.
5. Keep binary detection, pipe/node localization, graph-distance error, abstention/calibration, false dispatch and field transfer on separate compatible scorecards.

## Nonclaims

Passing this audit means only that the declared toy split plan meets its explicit structural leakage checks. It is not a hydraulic validation, benchmark reproduction, state-of-the-art comparison, calibrated model, field experiment or utility decision recommendation.
