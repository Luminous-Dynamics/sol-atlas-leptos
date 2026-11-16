// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Deterministic finite reference operations for infrastructure decisions
//! compared across competing Earth-history model families.
//!
//! This module does not implement geological reconstruction, sea-level
//! physics, hydraulic simulation, or probability estimation. It provides
//! exact integer and set operations for synthetic benchmark qualification.
//!
//! Invariants:
//! - model families are never implicitly averaged or assigned probabilities;
//! - mandatory floors are non-compensatory;
//! - missing model/floor evidence produces an explicit unresolved result;
//! - shoreline output is a synthetic threshold oracle, not a physical model;
//! - minimax regret uses an explicit unweighted model set.

use std::collections::{BTreeMap, BTreeSet};

/// Per-claim result across a declared set of qualified model families.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionClass {
    /// The claim passes every required floor in every model.
    Invariant,
    /// The claim passes some models and fails others.
    Conditional,
    /// Pairwise decision ordering changes across model families.
    ModelSensitive,
    /// At least one mandatory floor fails in every model.
    UniformlyIneligible,
    /// A required model or floor result is unknown.
    Unresolved,
}

/// Result of evaluating one mandatory floor for one model family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateResult {
    Pass,
    Fail,
    Unknown,
}

/// Mandatory-floor outcomes for one model family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelEligibility {
    pub model_id: String,
    pub floors: BTreeMap<String, GateResult>,
}

/// Integer utility values for one strategy across named model families.
///
/// Utilities have no implicit physical unit or probability interpretation.
/// All strategies in a comparison must use the same declared objective,
/// system boundary and units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyUtilities {
    pub strategy_id: String,
    pub by_model: BTreeMap<String, i64>,
}

/// Exact minimax-regret output over an unweighted finite model set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimaxRegretReport {
    pub best_utility_by_model: BTreeMap<String, i64>,
    pub maximum_regret_by_strategy: BTreeMap<String, i64>,
    pub selected_strategy: String,
    pub minimum_maximum_regret: i64,
}

/// Errors returned when a finite reference input is incomplete or ambiguous.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RobustnessError {
    EmptyModelSet,
    EmptyStrategySet,
    EmptyFloorSet,
    DuplicateModelId(String),
    DuplicateStrategyId(String),
    MissingFloor {
        model_id: String,
        floor_id: String,
    },
    MissingUtility {
        strategy_id: String,
        model_id: String,
    },
    UnexpectedModel {
        strategy_id: String,
        model_id: String,
    },
    RegretOverflow {
        strategy_id: String,
        model_id: String,
    },
    EmptyShorelineCellSet,
    EmptyShorelineModelSet,
}

/// Classify whether one candidate satisfies all mandatory floors across a
/// declared model set.
///
/// A known floor failure makes that model ineligible even if another floor
/// is unknown. If no floor is known to fail but at least one is unknown, the
/// model is unresolved. Across models:
/// - all eligible => Invariant;
/// - all ineligible => UniformlyIneligible;
/// - eligible and ineligible models => Conditional;
/// - any unresolved model => Unresolved.
///
/// Empty model/floor sets and missing floor keys are errors.
pub fn classify_eligibility(
    model_results: &[ModelEligibility],
    required_floors: &[String],
) -> Result<DecisionClass, RobustnessError> {
    if model_results.is_empty() {
        return Err(RobustnessError::EmptyModelSet);
    }
    if required_floors.is_empty() {
        return Err(RobustnessError::EmptyFloorSet);
    }

    let mut seen_models = BTreeSet::new();
    let mut eligible_count = 0usize;
    let mut ineligible_count = 0usize;
    let mut unresolved_count = 0usize;

    for model in model_results {
        if !seen_models.insert(model.model_id.as_str()) {
            return Err(RobustnessError::DuplicateModelId(
                model.model_id.clone(),
            ));
        }

        let mut failed = false;
        let mut unknown = false;

        for floor_id in required_floors {
            let result = model.floors.get(floor_id).ok_or_else(|| {
                RobustnessError::MissingFloor {
                    model_id: model.model_id.clone(),
                    floor_id: floor_id.clone(),
                }
            })?;

            match result {
                GateResult::Pass => {}
                GateResult::Fail => failed = true,
                GateResult::Unknown => unknown = true,
            }
        }

        if failed {
            ineligible_count += 1;
        } else if unknown {
            unresolved_count += 1;
        } else {
            eligible_count += 1;
        }
    }

    if unresolved_count > 0 {
        Ok(DecisionClass::Unresolved)
    } else if eligible_count == model_results.len() {
        Ok(DecisionClass::Invariant)
    } else if ineligible_count == model_results.len() {
        Ok(DecisionClass::UniformlyIneligible)
    } else {
        Ok(DecisionClass::Conditional)
    }
}

/// Classify whether the pairwise ordering of strategies is stable across
/// model families.
///
/// Every model must provide one utility for every strategy, using identical
/// utility semantics. A strategy pair whose relative ordering changes in
/// any model makes the overall ranking ModelSensitive. This is ordinal
/// stability only; it does not select a strategy or assign model weights.
pub fn classify_ranking_stability(
    model_ids: &[String],
    strategies: &[StrategyUtilities],
) -> Result<DecisionClass, RobustnessError> {
    if model_ids.is_empty() {
        return Err(RobustnessError::EmptyModelSet);
    }
    if strategies.is_empty() {
        return Err(RobustnessError::EmptyStrategySet);
    }

    let mut model_set = BTreeSet::new();
    for model_id in model_ids {
        if !model_set.insert(model_id.as_str()) {
            return Err(RobustnessError::DuplicateModelId(model_id.clone()));
        }
    }

    let mut strategy_set = BTreeSet::new();
    for strategy in strategies {
        if !strategy_set.insert(strategy.strategy_id.as_str()) {
            return Err(RobustnessError::DuplicateStrategyId(
                strategy.strategy_id.clone(),
            ));
        }
        for model_id in model_ids {
            if !strategy.by_model.contains_key(model_id) {
                return Err(RobustnessError::MissingUtility {
                    strategy_id: strategy.strategy_id.clone(),
                    model_id: model_id.clone(),
                });
            }
        }
        for model_id in strategy.by_model.keys() {
            if !model_set.contains(model_id.as_str()) {
                return Err(RobustnessError::UnexpectedModel {
                    strategy_id: strategy.strategy_id.clone(),
                    model_id: model_id.clone(),
                });
            }
        }
    }

    for left_index in 0..strategies.len() {
        for right_index in (left_index + 1)..strategies.len() {
            let left = &strategies[left_index];
            let right = &strategies[right_index];
            let mut first_ordering: Option<std::cmp::Ordering> = None;

            for model_id in model_ids {
                let ordering = left
                    .by_model
                    .get(model_id)
                    .expect("validated above")
                    .cmp(right.by_model.get(model_id).expect("validated above"));

                match first_ordering {
                    None => first_ordering = Some(ordering),
                    Some(previous) if previous != ordering => {
                        return Ok(DecisionClass::ModelSensitive);
                    }
                    Some(_) => {}
                }
            }
        }
    }

    Ok(DecisionClass::Invariant)
}

/// Compute exact minimax regret over a finite, unweighted model set.
///
/// The caller must qualify mandatory floors before invoking this function.
/// Ties are resolved deterministically by lexicographically smallest
/// strategy ID. No probability or expected-value input is accepted.
pub fn minimax_regret(
    model_ids: &[String],
    strategies: &[StrategyUtilities],
) -> Result<MinimaxRegretReport, RobustnessError> {
    if model_ids.is_empty() {
        return Err(RobustnessError::EmptyModelSet);
    }
    if strategies.is_empty() {
        return Err(RobustnessError::EmptyStrategySet);
    }

    let mut model_set = BTreeSet::new();
    for model_id in model_ids {
        if !model_set.insert(model_id.as_str()) {
            return Err(RobustnessError::DuplicateModelId(model_id.clone()));
        }
    }

    let mut strategy_ids = BTreeSet::new();
    for strategy in strategies {
        if !strategy_ids.insert(strategy.strategy_id.as_str()) {
            return Err(RobustnessError::DuplicateStrategyId(
                strategy.strategy_id.clone(),
            ));
        }

        for model_id in model_ids {
            if !strategy.by_model.contains_key(model_id) {
                return Err(RobustnessError::MissingUtility {
                    strategy_id: strategy.strategy_id.clone(),
                    model_id: model_id.clone(),
                });
            }
        }
        for model_id in strategy.by_model.keys() {
            if !model_set.contains(model_id.as_str()) {
                return Err(RobustnessError::UnexpectedModel {
                    strategy_id: strategy.strategy_id.clone(),
                    model_id: model_id.clone(),
                });
            }
        }
    }

    let mut best_utility_by_model = BTreeMap::new();
    for model_id in model_ids {
        let best = strategies
            .iter()
            .map(|strategy| {
                *strategy
                    .by_model
                    .get(model_id)
                    .expect("validated above")
            })
            .max()
            .expect("non-empty strategy set checked above");
        best_utility_by_model.insert(model_id.clone(), best);
    }

    let mut maximum_regret_by_strategy = BTreeMap::new();
    for strategy in strategies {
        let mut maximum_regret = i64::MIN;
        for model_id in model_ids {
            let best = *best_utility_by_model
                .get(model_id)
                .expect("best utility computed for every model");
            let utility = *strategy
                .by_model
                .get(model_id)
                .expect("validated above");
            let regret = best.checked_sub(utility).ok_or_else(|| {
                RobustnessError::RegretOverflow {
                    strategy_id: strategy.strategy_id.clone(),
                    model_id: model_id.clone(),
                }
            })?;
            maximum_regret = maximum_regret.max(regret);
        }

        maximum_regret_by_strategy.insert(
            strategy.strategy_id.clone(),
            maximum_regret,
        );
    }

    let (selected_strategy, minimum_maximum_regret) = maximum_regret_by_strategy
        .iter()
        .min_by(|(left_id, left_regret), (right_id, right_regret)| {
            left_regret
                .cmp(right_regret)
                .then_with(|| left_id.cmp(right_id))
        })
        .expect("non-empty strategy set checked above");

    Ok(MinimaxRegretReport {
        best_utility_by_model,
        maximum_regret_by_strategy,
        selected_strategy: selected_strategy.clone(),
        minimum_maximum_regret: *minimum_maximum_regret,
    })
}

/// Exact partition of cells under a deliberately synthetic threshold model.
///
/// Elevations and offsets use integer millimetres to avoid floating-point
/// boundary ambiguity. A cell is wet when elevation <= water offset. This
/// is a software reference only, not a physical shoreline calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShorelineEnvelope {
    pub wet_cells_by_model: BTreeMap<String, BTreeSet<usize>>,
    pub invariant_wet_cells: BTreeSet<usize>,
    pub model_sensitive_cells: BTreeSet<usize>,
    pub invariant_dry_cells: BTreeSet<usize>,
}

/// Compute exact agreement/disagreement cells for explicit synthetic offsets.
pub fn synthetic_shoreline_envelope(
    elevations_mm: &[i64],
    water_offsets_by_model_mm: &BTreeMap<String, i64>,
) -> Result<ShorelineEnvelope, RobustnessError> {
    if elevations_mm.is_empty() {
        return Err(RobustnessError::EmptyShorelineCellSet);
    }
    if water_offsets_by_model_mm.is_empty() {
        return Err(RobustnessError::EmptyShorelineModelSet);
    }

    let mut wet_cells_by_model = BTreeMap::new();
    let mut union_wet = BTreeSet::new();
    let mut invariant_wet: Option<BTreeSet<usize>> = None;

    for (model_id, offset_mm) in water_offsets_by_model_mm {
        let wet: BTreeSet<usize> = elevations_mm
            .iter()
            .enumerate()
            .filter_map(|(index, elevation)| {
                (*elevation <= *offset_mm).then_some(index)
            })
            .collect();

        union_wet.extend(wet.iter().copied());
        invariant_wet = Some(match invariant_wet {
            None => wet.clone(),
            Some(current) => current.intersection(&wet).copied().collect(),
        });
        wet_cells_by_model.insert(model_id.clone(), wet);
    }

    let invariant_wet_cells = invariant_wet.unwrap_or_default();
    let model_sensitive_cells = union_wet
        .difference(&invariant_wet_cells)
        .copied()
        .collect();

    let all_cells: BTreeSet<usize> = (0..elevations_mm.len()).collect();
    let invariant_dry_cells = all_cells.difference(&union_wet).copied().collect();

    Ok(ShorelineEnvelope {
        wet_cells_by_model,
        invariant_wet_cells,
        model_sensitive_cells,
        invariant_dry_cells,
    })
}

/// Exact overlap of upstream observation and calibration identifiers.
///
/// Shared identifiers expose lineage overlap. They do not quantify
/// statistical dependence or prove that unshared sources are independent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedLineage {
    pub shared_observations: BTreeSet<String>,
    pub shared_calibrations: BTreeSet<String>,
}

/// Return the identifiers shared by two model input/calibration manifests.
pub fn shared_lineage(
    left_observations: &[String],
    right_observations: &[String],
    left_calibrations: &[String],
    right_calibrations: &[String],
) -> SharedLineage {
    let left_obs: BTreeSet<&str> =
        left_observations.iter().map(String::as_str).collect();
    let right_obs: BTreeSet<&str> =
        right_observations.iter().map(String::as_str).collect();
    let left_cal: BTreeSet<&str> =
        left_calibrations.iter().map(String::as_str).collect();
    let right_cal: BTreeSet<&str> =
        right_calibrations.iter().map(String::as_str).collect();

    SharedLineage {
        shared_observations: left_obs
            .intersection(&right_obs)
            .map(|value| (*value).to_owned())
            .collect(),
        shared_calibrations: left_cal
            .intersection(&right_cal)
            .map(|value| (*value).to_owned())
            .collect(),
    }
}


/// Origin/meaning of a scenario supplied to a shoreline evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EarthScenarioKind {
    PaleogeographicReconstruction,
    GlobalEustaticEstimate,
    LocalRelativeSeaLevelReconstruction,
    SyntheticModernTerrainOffset,
    FutureEarthBoundaryCondition,
}

/// Result of checking whether a scenario has the minimum context needed to
/// enter a local shoreline physical-model evaluation.
///
/// Ready means only that required input categories are present and declared
/// Pass by the caller. It is not proof that the data or physical model is
/// accurate, calibrated, or valid for a particular place or geological age.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalShorelineInputStatus {
    ReadyForPhysicalModelEvaluation,
    RejectedTypeMismatch,
    BlockedByFailedGate,
    Unresolved,
}

/// Gate report for local shoreline evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalShorelineInputReport {
    pub status: LocalShorelineInputStatus,
    pub missing_or_unknown_fields: BTreeSet<String>,
    pub failed_fields: BTreeSet<String>,
}

/// Required evidence dimensions for local shoreline evaluation.
pub const LOCAL_SHORELINE_REQUIRED_GATES: [&str; 8] = [
    "horizontal_paleo_transform",
    "vertical_reference",
    "paleotopography",
    "local_vertical_land_motion",
    "geoid_gravitational_rotational_fingerprint",
    "ocean_dynamics",
    "chronology_alignment",
    "source_lineage_and_uncertainty",
];

/// Check whether a scenario may enter a local shoreline model evaluation.
///
/// Synthetic modern-terrain offsets and future-Earth climate boundary
/// conditions are rejected as type mismatches. Other scenario kinds require
/// every local transformation/evidence gate to be explicitly Pass. Missing
/// or Unknown gates return Unresolved; any known failed gate blocks evaluation.
pub fn local_shoreline_input_report(
    scenario_kind: EarthScenarioKind,
    gates: &BTreeMap<String, GateResult>,
) -> LocalShorelineInputReport {
    if matches!(
        scenario_kind,
        EarthScenarioKind::SyntheticModernTerrainOffset
            | EarthScenarioKind::FutureEarthBoundaryCondition
    ) {
        return LocalShorelineInputReport {
            status: LocalShorelineInputStatus::RejectedTypeMismatch,
            missing_or_unknown_fields: BTreeSet::new(),
            failed_fields: BTreeSet::new(),
        };
    }

    let mut missing_or_unknown_fields = BTreeSet::new();
    let mut failed_fields = BTreeSet::new();

    for field in LOCAL_SHORELINE_REQUIRED_GATES {
        match gates.get(field) {
            Some(GateResult::Pass) => {}
            Some(GateResult::Fail) => {
                failed_fields.insert(field.to_owned());
            }
            Some(GateResult::Unknown) | None => {
                missing_or_unknown_fields.insert(field.to_owned());
            }
        }
    }

    let status = if !failed_fields.is_empty() {
        LocalShorelineInputStatus::BlockedByFailedGate
    } else if !missing_or_unknown_fields.is_empty() {
        LocalShorelineInputStatus::Unresolved
    } else {
        LocalShorelineInputStatus::ReadyForPhysicalModelEvaluation
    };

    LocalShorelineInputReport {
        status,
        missing_or_unknown_fields,
        failed_fields,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model_ids(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn utility_map(values: &[(&str, i64)]) -> BTreeMap<String, i64> {
        values
            .iter()
            .map(|(model, utility)| ((*model).to_owned(), *utility))
            .collect()
    }

    #[test]
    fn dtr_004_shared_proxy_and_calibration_are_detected() {
        let result = shared_lineage(
            &model_ids(&["PROXY-1", "PROXY-2"]),
            &model_ids(&["PROXY-2", "PROXY-3"]),
            &model_ids(&["CAL-7"]),
            &model_ids(&["CAL-7"]),
        );

        assert_eq!(
            result.shared_observations,
            BTreeSet::from(["PROXY-2".to_owned()])
        );
        assert_eq!(
            result.shared_calibrations,
            BTreeSet::from(["CAL-7".to_owned()])
        );
    }

    #[test]
    fn dtr_006_shoreline_partition_matches_known_answer() {
        let offsets = BTreeMap::from([
            ("MODEL-A".to_owned(), 5_000_i64),
            ("MODEL-B".to_owned(), 8_000_i64),
        ]);
        let result = synthetic_shoreline_envelope(
            &[0, 2_000, 5_000, 8_000, 12_000],
            &offsets,
        )
        .expect("valid synthetic shoreline fixture");

        assert_eq!(
            result.wet_cells_by_model["MODEL-A"],
            BTreeSet::from([0, 1, 2])
        );
        assert_eq!(
            result.wet_cells_by_model["MODEL-B"],
            BTreeSet::from([0, 1, 2, 3])
        );
        assert_eq!(
            result.invariant_wet_cells,
            BTreeSet::from([0, 1, 2])
        );
        assert_eq!(
            result.model_sensitive_cells,
            BTreeSet::from([3])
        );
        assert_eq!(
            result.invariant_dry_cells,
            BTreeSet::from([4])
        );
    }

    #[test]
    fn dtr_007_minimax_regret_matches_known_answer() {
        let models = model_ids(&["FLOODING_A", "STRONTIUM_B", "SEQUENCE_C"]);
        let strategies = vec![
            StrategyUtilities {
                strategy_id: "coastal_fixed".to_owned(),
                by_model: utility_map(&[
                    ("FLOODING_A", 100),
                    ("STRONTIUM_B", 30),
                    ("SEQUENCE_C", 50),
                ]),
            },
            StrategyUtilities {
                strategy_id: "adaptive_dual".to_owned(),
                by_model: utility_map(&[
                    ("FLOODING_A", 75),
                    ("STRONTIUM_B", 75),
                    ("SEQUENCE_C", 70),
                ]),
            },
            StrategyUtilities {
                strategy_id: "inland_rail".to_owned(),
                by_model: utility_map(&[
                    ("FLOODING_A", 60),
                    ("STRONTIUM_B", 80),
                    ("SEQUENCE_C", 90),
                ]),
            },
        ];

        let result = minimax_regret(&models, &strategies)
            .expect("complete unweighted utility matrix");

        assert_eq!(
            result.best_utility_by_model,
            BTreeMap::from([
                ("FLOODING_A".to_owned(), 100),
                ("SEQUENCE_C".to_owned(), 90),
                ("STRONTIUM_B".to_owned(), 80),
            ])
        );
        assert_eq!(
            result.maximum_regret_by_strategy,
            BTreeMap::from([
                ("adaptive_dual".to_owned(), 25),
                ("coastal_fixed".to_owned(), 50),
                ("inland_rail".to_owned(), 40),
            ])
        );
        assert_eq!(result.selected_strategy, "adaptive_dual");
        assert_eq!(result.minimum_maximum_regret, 25);
    }

    #[test]
    fn dtr_008_mean_utility_cannot_override_a_floor_failure() {
        let floors = vec!["water_ecology".to_owned()];
        let models = vec![
            ModelEligibility {
                model_id: "A".to_owned(),
                floors: BTreeMap::from([(
                    "water_ecology".to_owned(),
                    GateResult::Pass,
                )]),
            },
            ModelEligibility {
                model_id: "B".to_owned(),
                floors: BTreeMap::from([(
                    "water_ecology".to_owned(),
                    GateResult::Fail,
                )]),
            },
            ModelEligibility {
                model_id: "C".to_owned(),
                floors: BTreeMap::from([(
                    "water_ecology".to_owned(),
                    GateResult::Pass,
                )]),
            },
        ];

        assert_eq!(
            classify_eligibility(&models, &floors).unwrap(),
            DecisionClass::Conditional
        );
        assert_ne!(
            classify_eligibility(&models, &floors).unwrap(),
            DecisionClass::Invariant
        );
    }

    #[test]
    fn unknown_required_floor_forces_unresolved() {
        let floors = vec!["water_ecology".to_owned()];
        let models = vec![
            ModelEligibility {
                model_id: "A".to_owned(),
                floors: BTreeMap::from([(
                    "water_ecology".to_owned(),
                    GateResult::Pass,
                )]),
            },
            ModelEligibility {
                model_id: "B".to_owned(),
                floors: BTreeMap::from([(
                    "water_ecology".to_owned(),
                    GateResult::Unknown,
                )]),
            },
        ];

        assert_eq!(
            classify_eligibility(&models, &floors).unwrap(),
            DecisionClass::Unresolved
        );
    }

    #[test]
    fn pairwise_ranking_changes_are_model_sensitive() {
        let models = model_ids(&["A", "B"]);
        let strategies = vec![
            StrategyUtilities {
                strategy_id: "coastal".to_owned(),
                by_model: utility_map(&[("A", 100), ("B", 20)]),
            },
            StrategyUtilities {
                strategy_id: "inland".to_owned(),
                by_model: utility_map(&[("A", 50), ("B", 90)]),
            },
        ];

        assert_eq!(
            classify_ranking_stability(&models, &strategies).unwrap(),
            DecisionClass::ModelSensitive
        );
    }

    #[test]
    fn incomplete_utility_matrix_fails_closed() {
        let models = model_ids(&["A", "B"]);
        let strategies = vec![StrategyUtilities {
            strategy_id: "only_a".to_owned(),
            by_model: utility_map(&[("A", 10)]),
        }];

        assert_eq!(
            minimax_regret(&models, &strategies),
            Err(RobustnessError::MissingUtility {
                strategy_id: "only_a".to_owned(),
                model_id: "B".to_owned(),
            })
        );
    }

    #[test]
    fn empty_floor_set_is_rejected() {
        let models = vec![ModelEligibility {
            model_id: "A".to_owned(),
            floors: BTreeMap::new(),
        }];

        assert_eq!(
            classify_eligibility(&models, &[]),
            Err(RobustnessError::EmptyFloorSet)
        );
    }

    fn complete_shoreline_gates() -> BTreeMap<String, GateResult> {
        LOCAL_SHORELINE_REQUIRED_GATES
            .iter()
            .map(|field| ((*field).to_owned(), GateResult::Pass))
            .collect()
    }

    #[test]
    fn dtr_001_synthetic_modern_offset_is_rejected_as_paleoshoreline_input() {
        let result = local_shoreline_input_report(
            EarthScenarioKind::SyntheticModernTerrainOffset,
            &complete_shoreline_gates(),
        );
        assert_eq!(
            result.status,
            LocalShorelineInputStatus::RejectedTypeMismatch
        );
    }

    #[test]
    fn dtr_002_global_eustatic_curve_needs_local_vertical_motion_and_other_gates() {
        let mut gates = complete_shoreline_gates();
        gates.insert(
            "local_vertical_land_motion".to_owned(),
            GateResult::Unknown,
        );
        let result = local_shoreline_input_report(
            EarthScenarioKind::GlobalEustaticEstimate,
            &gates,
        );
        assert_eq!(result.status, LocalShorelineInputStatus::Unresolved);
        assert!(result
            .missing_or_unknown_fields
            .contains("local_vertical_land_motion"));
    }

    #[test]
    fn dtr_009_future_supercontinent_boundary_is_not_a_sea_level_offset() {
        let result = local_shoreline_input_report(
            EarthScenarioKind::FutureEarthBoundaryCondition,
            &complete_shoreline_gates(),
        );
        assert_eq!(
            result.status,
            LocalShorelineInputStatus::RejectedTypeMismatch
        );
    }

    #[test]
    fn local_shoreline_gate_failure_blocks_evaluation() {
        let mut gates = complete_shoreline_gates();
        gates.insert("vertical_reference".to_owned(), GateResult::Fail);
        let result = local_shoreline_input_report(
            EarthScenarioKind::PaleogeographicReconstruction,
            &gates,
        );
        assert_eq!(
            result.status,
            LocalShorelineInputStatus::BlockedByFailedGate
        );
        assert!(result.failed_fields.contains("vertical_reference"));
    }

}
