// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Threshold-function robust controlled invariance for a restricted class of
//! monotone finite-grid systems.
//!
//! The safe set is lower-closed under the componentwise product order. Each
//! transition rule is a saturating affine map with non-negative integer
//! coefficients, which is monotone by construction. The invariant set is
//! represented by one height threshold per column, not by an explicit list of
//! all grid states.
//!
//! This module is intentionally narrower than arbitrary infrastructure games.
//! Non-monotone dependencies, negative couplings, discontinuous institutional
//! rules or hard floors that are not lower-closed must use the explicit
//! finite-state kernel or another sound method. Do not coerce such systems
//! into this representation.

/// One monotone saturating affine transition rule.
///
/// For output coordinate j:
/// y[j] = clamp(sum_i(coefficients_by_output[j][i] * x[i])
///              + offset_by_output[j], 0, axis_sizes[j] - 1).
///
/// All coefficients are unsigned, hence non-negative. Negative offsets are
/// allowed; they do not break monotonicity. Arithmetic saturates in u128
/// before clamping to the finite output axis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonotoneTransitionRule {
    pub coefficients_by_output: Vec<Vec<u64>>,
    pub offset_by_output: Vec<i64>,
}

/// Product-grid problem with a lower-closed initial safe set.
///
/// The final coordinate is the height axis. The safe set is represented by
/// initial_safe_thresholds, one signed height threshold per projected column;
/// -1 denotes an empty column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonotoneGridProblem {
    pub axis_sizes: Vec<usize>,
    /// Outer index is controller action; inner index is disturbance branch.
    pub rules_by_action: Vec<Vec<MonotoneTransitionRule>>,
    pub initial_safe_thresholds: Vec<i64>,
}

/// Explicit computational limits for threshold-function synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonotoneGridLimits {
    pub max_state_universe: u128,
    pub max_columns: usize,
    pub max_fixed_point_iterations: usize,
    pub max_predecessor_queries: usize,
}

impl Default for MonotoneGridLimits {
    fn default() -> Self {
        Self {
            max_state_universe: 100_000_000_000_000,
            max_columns: 10_000_000,
            max_fixed_point_iterations: 10_000,
            max_predecessor_queries: 100_000_000,
        }
    }
}

/// Certificate that the representation assumptions were checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonotonicityCertificate {
    pub transition_family: String,
    pub coefficients_nonnegative: bool,
    pub saturating_componentwise_clamp: bool,
    pub initial_safe_set_lower_closed: bool,
    pub height_axis: usize,
}

/// Exact greatest-fixed-point result in compressed threshold form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonotoneInvariantReport {
    pub problem_fingerprint: u64,
    pub axis_sizes: Vec<usize>,
    pub thresholds: Vec<i64>,
    pub column_count: usize,
    pub state_universe_size: u128,
    pub invariant_state_count: u128,
    pub fixed_point_iterations: usize,
    pub predecessor_queries: usize,
    pub certificate: MonotonicityCertificate,
}

/// A bounded run contains no partial threshold set and makes no invariant
/// classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonotoneGridExecution {
    Exact(MonotoneInvariantReport),
    Bounded {
        resource: String,
        observed: u128,
        limit: u128,
    },
}

/// Shape, ordering or coordinate errors in the structured monotone model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonotoneGridError {
    AtLeastTwoDimensionsRequired,
    EmptyAxis { axis: usize },
    StateUniverseOverflow,
    ColumnCountOverflow,
    NoControllerActions,
    NoDisturbances,
    UnequalDisturbanceCounts {
        action: usize,
        expected: usize,
        observed: usize,
    },
    WrongRuleOutputCount {
        action: usize,
        disturbance: usize,
        expected: usize,
        observed: usize,
    },
    WrongRuleInputCount {
        action: usize,
        disturbance: usize,
        output_axis: usize,
        expected: usize,
        observed: usize,
    },
    WrongThresholdCount {
        expected: usize,
        observed: usize,
    },
    ThresholdOutOfRange {
        column: usize,
        value: i64,
        maximum: i64,
    },
    SafeSetNotLowerClosed {
        column: usize,
        greater_column: usize,
        lower_threshold: i64,
        greater_threshold: i64,
    },
    CoordinateDimensionMismatch {
        expected: usize,
        observed: usize,
    },
    CoordinateOutOfRange {
        axis: usize,
        value: usize,
        maximum: usize,
    },
    ProblemFingerprintMismatch,
}

/// Decode row-major flat index into componentwise coordinates.
fn decode_index(mut index: usize, axis_sizes: &[usize]) -> Vec<usize> {
    let mut coordinates = vec![0; axis_sizes.len()];
    for axis in (0..axis_sizes.len()).rev() {
        coordinates[axis] = index % axis_sizes[axis];
        index /= axis_sizes[axis];
    }
    coordinates
}

/// Encode componentwise coordinates in row-major order.
fn encode_index(coordinates: &[usize], axis_sizes: &[usize]) -> usize {
    coordinates
        .iter()
        .zip(axis_sizes)
        .fold(0usize, |index, (coordinate, size)| {
            index * size + coordinate
        })
}

fn checked_products(axis_sizes: &[usize]) -> Result<(u128, usize), MonotoneGridError> {
    let state_universe = axis_sizes.iter().try_fold(1u128, |acc, size| {
        acc.checked_mul(*size as u128)
            .ok_or(MonotoneGridError::StateUniverseOverflow)
    })?;
    let column_count_u128 = axis_sizes[..axis_sizes.len() - 1]
        .iter()
        .try_fold(1u128, |acc, size| {
            acc.checked_mul(*size as u128)
                .ok_or(MonotoneGridError::ColumnCountOverflow)
        })?;
    let column_count = usize::try_from(column_count_u128)
        .map_err(|_| MonotoneGridError::ColumnCountOverflow)?;
    Ok((state_universe, column_count))
}

fn validate_problem_shape(
    problem: &MonotoneGridProblem,
) -> Result<(u128, usize, usize), MonotoneGridError> {
    if problem.axis_sizes.len() < 2 {
        return Err(MonotoneGridError::AtLeastTwoDimensionsRequired);
    }
    for (axis, size) in problem.axis_sizes.iter().enumerate() {
        if *size == 0 {
            return Err(MonotoneGridError::EmptyAxis { axis });
        }
        if *size - 1 > i64::MAX as usize {
            return Err(MonotoneGridError::StateUniverseOverflow);
        }
    }
    if problem.rules_by_action.is_empty() {
        return Err(MonotoneGridError::NoControllerActions);
    }
    let disturbance_count = problem.rules_by_action[0].len();
    if disturbance_count == 0 {
        return Err(MonotoneGridError::NoDisturbances);
    }

    for (action_index, rules) in problem.rules_by_action.iter().enumerate() {
        if rules.len() != disturbance_count {
            return Err(MonotoneGridError::UnequalDisturbanceCounts {
                action: action_index,
                expected: disturbance_count,
                observed: rules.len(),
            });
        }
        for (disturbance_index, rule) in rules.iter().enumerate() {
            if rule.coefficients_by_output.len() != problem.axis_sizes.len() {
                return Err(MonotoneGridError::WrongRuleOutputCount {
                    action: action_index,
                    disturbance: disturbance_index,
                    expected: problem.axis_sizes.len(),
                    observed: rule.coefficients_by_output.len(),
                });
            }
            if rule.offset_by_output.len() != problem.axis_sizes.len() {
                return Err(MonotoneGridError::WrongRuleOutputCount {
                    action: action_index,
                    disturbance: disturbance_index,
                    expected: problem.axis_sizes.len(),
                    observed: rule.offset_by_output.len(),
                });
            }
            for (output_axis, coefficients) in
                rule.coefficients_by_output.iter().enumerate()
            {
                if coefficients.len() != problem.axis_sizes.len() {
                    return Err(MonotoneGridError::WrongRuleInputCount {
                        action: action_index,
                        disturbance: disturbance_index,
                        output_axis,
                        expected: problem.axis_sizes.len(),
                        observed: coefficients.len(),
                    });
                }
            }
        }
    }

    let (state_universe, column_count) = checked_products(&problem.axis_sizes)?;
    if problem.initial_safe_thresholds.len() != column_count {
        return Err(MonotoneGridError::WrongThresholdCount {
            expected: column_count,
            observed: problem.initial_safe_thresholds.len(),
        });
    }

    let max_height = (problem.axis_sizes[problem.axis_sizes.len() - 1] - 1) as i64;
    for (column, threshold) in problem.initial_safe_thresholds.iter().enumerate() {
        if *threshold < -1 || *threshold > max_height {
            return Err(MonotoneGridError::ThresholdOutOfRange {
                column,
                value: *threshold,
                maximum: max_height,
            });
        }
    }

    validate_lower_closed_thresholds(
        &problem.initial_safe_thresholds,
        &problem.axis_sizes,
    )?;
    Ok((state_universe, column_count, disturbance_count))
}

/// Check lower-closedness by checking every cover edge in the projected
/// componentwise product order. Cover-edge checks imply the full order.
fn validate_lower_closed_thresholds(
    thresholds: &[i64],
    axis_sizes: &[usize],
) -> Result<(), MonotoneGridError> {
    let projection_sizes = &axis_sizes[..axis_sizes.len() - 1];
    for column in 0..thresholds.len() {
        let coordinates = decode_index(column, projection_sizes);
        for axis in 0..projection_sizes.len() {
            if coordinates[axis] + 1 >= projection_sizes[axis] {
                continue;
            }
            let mut greater_coordinates = coordinates.clone();
            greater_coordinates[axis] += 1;
            let greater_column = encode_index(&greater_coordinates, projection_sizes);
            if thresholds[column] < thresholds[greater_column] {
                return Err(MonotoneGridError::SafeSetNotLowerClosed {
                    column,
                    greater_column,
                    lower_threshold: thresholds[column],
                    greater_threshold: thresholds[greater_column],
                });
            }
        }
    }
    Ok(())
}


/// Stable non-cryptographic fingerprint for accidental stale-problem
/// detection when querying witness actions from a computed threshold report.
/// This is not an evidence attestation or security digest.
fn problem_fingerprint(problem: &MonotoneGridProblem) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    let mut feed = |value: u64| {
        for byte in value.to_le_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    };

    feed(problem.axis_sizes.len() as u64);
    for size in &problem.axis_sizes {
        feed(*size as u64);
    }
    feed(problem.rules_by_action.len() as u64);
    for action_rules in &problem.rules_by_action {
        feed(action_rules.len() as u64);
        for rule in action_rules {
            feed(rule.coefficients_by_output.len() as u64);
            for row in &rule.coefficients_by_output {
                feed(row.len() as u64);
                for coefficient in row {
                    feed(*coefficient);
                }
            }
            feed(rule.offset_by_output.len() as u64);
            for offset in &rule.offset_by_output {
                feed(*offset as u64);
            }
        }
    }
    feed(problem.initial_safe_thresholds.len() as u64);
    for threshold in &problem.initial_safe_thresholds {
        feed(*threshold as u64);
    }

    drop(feed);
    hash
}

fn apply_rule(
    rule: &MonotoneTransitionRule,
    coordinates: &[usize],
    axis_sizes: &[usize],
) -> Vec<usize> {
    let mut successor = Vec::with_capacity(axis_sizes.len());
    for output_axis in 0..axis_sizes.len() {
        let mut value = 0u128;
        for (input_axis, coefficient) in
            rule.coefficients_by_output[output_axis].iter().enumerate()
        {
            value = value.saturating_add(
                (*coefficient as u128).saturating_mul(coordinates[input_axis] as u128),
            );
        }

        let offset = rule.offset_by_output[output_axis];
        if offset >= 0 {
            value = value.saturating_add(offset as u128);
        } else {
            value = value.saturating_sub(offset.unsigned_abs() as u128);
        }

        let maximum = (axis_sizes[output_axis] - 1) as u128;
        successor.push(value.min(maximum) as usize);
    }
    successor
}

fn coordinate_is_in_kernel(
    coordinates: &[usize],
    thresholds: &[i64],
    axis_sizes: &[usize],
) -> bool {
    let height_axis = axis_sizes.len() - 1;
    let column = encode_index(&coordinates[..height_axis], &axis_sizes[..height_axis]);
    thresholds[column] >= coordinates[height_axis] as i64
}

fn has_robust_action(
    coordinates: &[usize],
    thresholds: &[i64],
    problem: &MonotoneGridProblem,
) -> bool {
    for rules in &problem.rules_by_action {
        let all_disturbances_safe = rules.iter().all(|rule| {
            let successor = apply_rule(rule, coordinates, &problem.axis_sizes);
            coordinate_is_in_kernel(&successor, thresholds, &problem.axis_sizes)
        });
        if all_disturbances_safe {
            return true;
        }
    }
    false
}

fn invariant_state_count(thresholds: &[i64]) -> u128 {
    thresholds.iter().fold(0u128, |total, threshold| {
        total.saturating_add(if *threshold < 0 {
            0
        } else {
            *threshold as u128 + 1
        })
    })
}

fn predecessor_query(
    column: usize,
    height: usize,
    thresholds: &[i64],
    problem: &MonotoneGridProblem,
) -> bool {
    let projection_sizes = &problem.axis_sizes[..problem.axis_sizes.len() - 1];
    let mut coordinates = decode_index(column, projection_sizes);
    coordinates.push(height);
    has_robust_action(&coordinates, thresholds, problem)
}

/// Compute the greatest robust controlled-invariant lower set.
///
/// Each fixed-point iteration computes the robust predecessor independently
/// per column (currently sequential in this implementation). For each column,
/// a binary search finds the greatest height whose
/// state has some action that keeps every disturbance successor inside the
/// previous threshold set. Monotonicity makes the predicate prefix-closed.
pub fn compute_monotone_invariant_kernel_with_limits(
    problem: &MonotoneGridProblem,
    limits: MonotoneGridLimits,
) -> Result<MonotoneGridExecution, MonotoneGridError> {
    let (state_universe_size, column_count, _disturbance_count) =
        validate_problem_shape(problem)?;

    let bounded = |resource: &str, observed: u128, limit: u128| {
        MonotoneGridExecution::Bounded {
            resource: resource.to_owned(),
            observed,
            limit,
        }
    };

    if state_universe_size > limits.max_state_universe {
        return Ok(bounded(
            "state_universe",
            state_universe_size,
            limits.max_state_universe,
        ));
    }
    if column_count > limits.max_columns {
        return Ok(bounded(
            "columns",
            column_count as u128,
            limits.max_columns as u128,
        ));
    }

    let max_height = (problem.axis_sizes[problem.axis_sizes.len() - 1] - 1) as i64;
    let mut thresholds = problem.initial_safe_thresholds.clone();
    let mut predecessor_queries = 0usize;
    let mut fixed_point_iterations = 0usize;

    loop {
        if fixed_point_iterations >= limits.max_fixed_point_iterations {
            return Ok(bounded(
                "fixed_point_iterations",
                (fixed_point_iterations + 1) as u128,
                limits.max_fixed_point_iterations as u128,
            ));
        }
        fixed_point_iterations += 1;

        let mut next_thresholds = vec![-1i64; column_count];

        for column in 0..column_count {
            let current_max = thresholds[column];
            if current_max < 0 {
                continue;
            }

            // i128 bounds allow the sentinel step max_height + 1 when the
            // stored i64 threshold is exactly i64::MAX.
            let mut low = 0i128;
            let mut high = current_max.min(max_height) as i128;
            let mut best = -1i64;

            // For a lower-closed set and monotone transitions, if height h
            // has a robust action then every lower height in this column
            // has the same action. Therefore the predicate is binary-searchable.
            while low <= high {
                if predecessor_queries >= limits.max_predecessor_queries {
                    return Ok(bounded(
                        "predecessor_queries",
                        (predecessor_queries + 1) as u128,
                        limits.max_predecessor_queries as u128,
                    ));
                }

                let middle = low + (high - low) / 2;
                predecessor_queries += 1;

                if predecessor_query(
                    column,
                    middle as usize,
                    &thresholds,
                    problem,
                ) {
                    best = middle as i64;
                    low = middle + 1;
                } else {
                    high = middle - 1;
                }
            }

            next_thresholds[column] = best;
        }

        validate_lower_closed_thresholds(&next_thresholds, &problem.axis_sizes)?;

        if next_thresholds == thresholds {
            return Ok(MonotoneGridExecution::Exact(
                MonotoneInvariantReport {
                    problem_fingerprint: problem_fingerprint(problem),
                    axis_sizes: problem.axis_sizes.clone(),
                    invariant_state_count: invariant_state_count(&thresholds),
                    thresholds,
                    column_count,
                    state_universe_size,
                    fixed_point_iterations,
                    predecessor_queries,
                    certificate: MonotonicityCertificate {
                        transition_family: "nonnegative_affine_saturating".to_owned(),
                        coefficients_nonnegative: true,
                        saturating_componentwise_clamp: true,
                        initial_safe_set_lower_closed: true,
                        height_axis: problem.axis_sizes.len() - 1,
                    },
                },
            ));
        }

        thresholds = next_thresholds;
    }
}

/// Compute using default resource caps.
pub fn compute_monotone_invariant_kernel(
    problem: &MonotoneGridProblem,
) -> Result<MonotoneGridExecution, MonotoneGridError> {
    compute_monotone_invariant_kernel_with_limits(problem, MonotoneGridLimits::default())
}

/// Return the index of the first caller-enumerated action that keeps every
/// disturbance successor inside a previously computed exact invariant
/// threshold set. Action identity is therefore tied to the original
/// rules_by_action ordering. Returns None for a state outside the invariant set.
pub fn witness_action_for_coordinate(
    problem: &MonotoneGridProblem,
    report: &MonotoneInvariantReport,
    coordinates: &[usize],
) -> Result<Option<usize>, MonotoneGridError> {
    if problem_fingerprint(problem) != report.problem_fingerprint
        || problem.axis_sizes != report.axis_sizes
    {
        return Err(MonotoneGridError::ProblemFingerprintMismatch);
    }
    if coordinates.len() != problem.axis_sizes.len() {
        return Err(MonotoneGridError::CoordinateDimensionMismatch {
            expected: problem.axis_sizes.len(),
            observed: coordinates.len(),
        });
    }
    for (axis, (value, size)) in coordinates.iter().zip(&problem.axis_sizes).enumerate() {
        if *value >= *size {
            return Err(MonotoneGridError::CoordinateOutOfRange {
                axis,
                value: *value,
                maximum: size - 1,
            });
        }
    }

    if !coordinate_is_in_kernel(
        coordinates,
        &report.thresholds,
        &problem.axis_sizes,
    ) {
        return Ok(None);
    }

    for (action_index, rules) in problem.rules_by_action.iter().enumerate() {
        if rules.iter().all(|rule| {
            let successor = apply_rule(rule, coordinates, &problem.axis_sizes);
            coordinate_is_in_kernel(
                &successor,
                &report.thresholds,
                &problem.axis_sizes,
            )
        }) {
            return Ok(Some(action_index));
        }
    }

    // A state in an exact greatest fixed point must have a witness.
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shift_x_up_rule() -> MonotoneTransitionRule {
        MonotoneTransitionRule {
            coefficients_by_output: vec![vec![1, 0], vec![0, 1]],
            offset_by_output: vec![1, 0],
        }
    }

    fn identity_rule_3d() -> MonotoneTransitionRule {
        MonotoneTransitionRule {
            coefficients_by_output: vec![
                vec![1, 0, 0],
                vec![0, 1, 0],
                vec![0, 0, 1],
            ],
            offset_by_output: vec![0, 0, 0],
        }
    }

    fn explicit_oracle(
        problem: &MonotoneGridProblem,
        thresholds: &[i64],
    ) -> Vec<bool> {
        let total_states = problem.axis_sizes.iter().product::<usize>();
        let mut kernel = (0..total_states)
            .map(|index| {
                let coordinates = decode_index(index, &problem.axis_sizes);
                coordinate_is_in_kernel(
                    &coordinates,
                    thresholds,
                    &problem.axis_sizes,
                )
            })
            .collect::<Vec<_>>();

        loop {
            let mut remove = Vec::new();
            for state_index in 0..total_states {
                if !kernel[state_index] {
                    continue;
                }
                let coordinates = decode_index(state_index, &problem.axis_sizes);
                let has_action = problem.rules_by_action.iter().any(|rules| {
                    rules.iter().all(|rule| {
                        let successor = apply_rule(
                            rule,
                            &coordinates,
                            &problem.axis_sizes,
                        );
                        let successor_index =
                            encode_index(&successor, &problem.axis_sizes);
                        kernel[successor_index]
                    })
                });
                if !has_action {
                    remove.push(state_index);
                }
            }

            if remove.is_empty() {
                break;
            }
            for state_index in remove {
                kernel[state_index] = false;
            }
        }
        kernel
    }

    fn thresholds_from_mask(
        mask: &[bool],
        axis_sizes: &[usize],
    ) -> Vec<i64> {
        let column_count = axis_sizes[..axis_sizes.len() - 1]
            .iter()
            .product::<usize>();
        let height_size = axis_sizes[axis_sizes.len() - 1];
        let mut thresholds = vec![-1i64; column_count];

        for column in 0..column_count {
            for height in 0..height_size {
                if mask[column * height_size + height] {
                    thresholds[column] = height as i64;
                }
            }
        }
        thresholds
    }

    #[test]
    fn threshold_kernel_matches_explicit_oracle_on_small_monotone_grid() {
        let problem = MonotoneGridProblem {
            axis_sizes: vec![2, 3],
            rules_by_action: vec![vec![shift_x_up_rule(), shift_x_up_rule()]],
            initial_safe_thresholds: vec![2, 1],
        };

        let execution =
            compute_monotone_invariant_kernel(&problem).unwrap();
        let MonotoneGridExecution::Exact(report) = execution else {
            panic!("small fixture must be exact");
        };

        let explicit = explicit_oracle(
            &problem,
            &problem.initial_safe_thresholds,
        );
        let explicit_thresholds =
            thresholds_from_mask(&explicit, &problem.axis_sizes);

        assert_eq!(report.thresholds, explicit_thresholds);
        assert_eq!(report.thresholds, vec![1, 1]);
        assert_eq!(report.invariant_state_count, 4);
        assert_eq!(
            witness_action_for_coordinate(&problem, &report, &[0, 0]).unwrap(),
            Some(0)
        );
        assert_eq!(
            witness_action_for_coordinate(&problem, &report, &[0, 2]).unwrap(),
            None
        );
    }



    #[test]
    fn maximum_i64_height_axis_is_handled_without_signed_cast_overflow() {
        let height_size = (i64::MAX as usize) + 1;
        let problem = MonotoneGridProblem {
            axis_sizes: vec![1, height_size],
            rules_by_action: vec![vec![MonotoneTransitionRule {
                coefficients_by_output: vec![vec![1, 0], vec![0, 1]],
                offset_by_output: vec![0, 0],
            }]],
            initial_safe_thresholds: vec![i64::MAX],
        };
        let limits = MonotoneGridLimits {
            max_state_universe: u128::MAX,
            max_columns: 1,
            max_fixed_point_iterations: 2,
            max_predecessor_queries: 128,
        };

        let execution =
            compute_monotone_invariant_kernel_with_limits(&problem, limits).unwrap();
        let MonotoneGridExecution::Exact(report) = execution else {
            panic!("identity fixture must be exact");
        };
        assert_eq!(report.thresholds, vec![i64::MAX]);
        assert_eq!(report.state_universe_size, (i64::MAX as u128) + 1);
    }

    #[test]
    fn witness_query_rejects_mismatched_transition_model() {
        let problem = MonotoneGridProblem {
            axis_sizes: vec![2, 3],
            rules_by_action: vec![vec![shift_x_up_rule()]],
            initial_safe_thresholds: vec![2, 1],
        };
        let execution = compute_monotone_invariant_kernel(&problem).unwrap();
        let MonotoneGridExecution::Exact(report) = execution else {
            panic!("small fixture must be exact");
        };

        let mut changed = problem.clone();
        changed.rules_by_action[0][0].offset_by_output[0] = 0;

        assert_eq!(
            witness_action_for_coordinate(&changed, &report, &[0, 0]),
            Err(MonotoneGridError::ProblemFingerprintMismatch)
        );
    }


    #[test]
    fn exhaustive_small_monotone_affine_maps_match_explicit_fixed_point_oracle() {
        let limits = MonotoneGridLimits {
            max_state_universe: 100,
            max_columns: 10,
            max_fixed_point_iterations: 100,
            max_predecessor_queries: 10_000,
        };

        // Every rule is componentwise monotone because all coefficients are
        // non-negative. Enumerate 16 cross-coupling/offset combinations and
        // every lower-closed threshold pair on the 2x3 grid.
        for x_from_height in 0_u64..=1 {
            for height_from_x in 0_u64..=1 {
                for x_offset in [-1_i64, 0] {
                    for height_offset in [-1_i64, 0] {
                        let rule = MonotoneTransitionRule {
                            coefficients_by_output: vec![
                                vec![1, x_from_height],
                                vec![height_from_x, 1],
                            ],
                            offset_by_output: vec![x_offset, height_offset],
                        };

                        for lower_column in -1_i64..=2 {
                            for upper_column in -1_i64..=lower_column {
                                let problem = MonotoneGridProblem {
                                    axis_sizes: vec![2, 3],
                                    rules_by_action: vec![vec![rule.clone()]],
                                    initial_safe_thresholds: vec![
                                        lower_column,
                                        upper_column,
                                    ],
                                };

                                let execution =
                                    compute_monotone_invariant_kernel_with_limits(
                                        &problem,
                                        limits,
                                    )
                                    .unwrap();
                                let MonotoneGridExecution::Exact(report) = execution else {
                                    panic!("small generated case must be exact");
                                };

                                let explicit =
                                    explicit_oracle(&problem, &problem.initial_safe_thresholds);
                                let expected =
                                    thresholds_from_mask(&explicit, &problem.axis_sizes);

                                assert_eq!(
                                    report.thresholds,
                                    expected,
                                    "threshold mismatch for coeffs ({x_from_height},{height_from_x}), offsets ({x_offset},{height_offset}), thresholds ({lower_column},{upper_column})"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn threshold_kernel_rejects_non_lower_closed_initial_safe_set() {
        let problem = MonotoneGridProblem {
            axis_sizes: vec![2, 3],
            rules_by_action: vec![vec![shift_x_up_rule()]],
            initial_safe_thresholds: vec![0, 1],
        };

        assert!(matches!(
            compute_monotone_invariant_kernel(&problem),
            Err(MonotoneGridError::SafeSetNotLowerClosed { .. })
        ));
    }

    #[test]
    fn threshold_kernel_rejects_wrong_affine_rule_shape() {
        let problem = MonotoneGridProblem {
            axis_sizes: vec![2, 3],
            rules_by_action: vec![vec![MonotoneTransitionRule {
                coefficients_by_output: vec![vec![1, 0]],
                offset_by_output: vec![0, 0],
            }]],
            initial_safe_thresholds: vec![2, 1],
        };

        assert!(matches!(
            compute_monotone_invariant_kernel(&problem),
            Err(MonotoneGridError::WrongRuleOutputCount { .. })
        ));
    }

    #[test]
    fn threshold_kernel_explicit_cap_returns_bounded_without_partial_thresholds() {
        let problem = MonotoneGridProblem {
            axis_sizes: vec![2, 3],
            rules_by_action: vec![vec![shift_x_up_rule()]],
            initial_safe_thresholds: vec![2, 1],
        };
        let limits = MonotoneGridLimits {
            max_state_universe: 100,
            max_columns: 1,
            max_fixed_point_iterations: 10,
            max_predecessor_queries: 100,
        };

        assert_eq!(
            compute_monotone_invariant_kernel_with_limits(&problem, limits).unwrap(),
            MonotoneGridExecution::Bounded {
                resource: "columns".to_owned(),
                observed: 2,
                limit: 1,
            }
        );
    }

    #[test]
    fn threshold_representation_scales_with_columns_not_full_grid_states() {
        let axes = vec![200, 200, 200];
        let columns = axes[0] * axes[1];
        let problem = MonotoneGridProblem {
            axis_sizes: axes,
            rules_by_action: vec![vec![identity_rule_3d()]],
            initial_safe_thresholds: vec![199; columns],
        };
        let limits = MonotoneGridLimits {
            max_state_universe: 10_000_000,
            max_columns: 50_000,
            max_fixed_point_iterations: 10,
            max_predecessor_queries: 1_000_000,
        };

        let execution =
            compute_monotone_invariant_kernel_with_limits(&problem, limits).unwrap();
        let MonotoneGridExecution::Exact(report) = execution else {
            panic!("structured 8-million-state fixture must complete within declared limits");
        };

        assert_eq!(report.state_universe_size, 8_000_000);
        assert_eq!(report.column_count, 40_000);
        assert_eq!(report.invariant_state_count, 8_000_000);
        assert_eq!(report.thresholds.len(), 40_000);
    }
}
