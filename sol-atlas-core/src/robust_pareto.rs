// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Conservative interval-aware Pareto analysis for site and design alternatives.
//! This module respects upstream gate dispositions; it does not establish site safety.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateDisposition {
    Eligible,
    NeedsEvidence,
    Excluded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveDirection {
    Maximize,
    Minimize,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeasurementInterval {
    pub lower: Option<f64>,
    pub upper: Option<f64>,
}

impl MeasurementInterval {
    pub fn exact(value: f64) -> Self {
        Self { lower: Some(value), upper: Some(value) }
    }

    fn complete(self) -> bool {
        self.lower.is_some() && self.upper.is_some()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParetoCandidate {
    pub candidate_id: String,
    pub gate: GateDisposition,
    #[serde(default)]
    pub objectives: BTreeMap<String, MeasurementInterval>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParetoPolicy {
    pub objectives: BTreeMap<String, ObjectiveDirection>,
    /// Optional per-objective absolute tolerances in that objective's raw units.
    /// An omitted tolerance defaults to zero; no shared cross-unit epsilon is used.
    #[serde(default)]
    pub tolerances: BTreeMap<String, f64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentKind {
    Frontier,
    RobustlyDominated,
    NeedsEvidence,
    Excluded,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CandidateDisposition {
    pub kind: AssessmentKind,
    #[serde(default)]
    pub dominated_by: Vec<String>,
    #[serde(default)]
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParetoResult {
    pub dispositions: BTreeMap<String, CandidateDisposition>,
    pub frontier: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParetoError {
    NoObjectives,
    EmptyObjectiveId,
    EmptyCandidateId,
    DuplicateCandidateId(String),
    UnknownObjective { candidate_id: String, objective_id: String },
    UnknownToleranceObjective(String),
    InvalidTolerance { objective_id: String, value: f64 },
    InvalidBound { candidate_id: String, objective_id: String, bound: f64 },
    InvertedInterval { candidate_id: String, objective_id: String, lower: f64, upper: f64 },
}

impl fmt::Display for ParetoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoObjectives => write!(f, "policy must define at least one objective"),
            Self::EmptyObjectiveId => write!(f, "objective IDs must not be empty"),
            Self::EmptyCandidateId => write!(f, "candidate IDs must not be empty"),
            Self::DuplicateCandidateId(id) => write!(f, "duplicate candidate ID: {id}"),
            Self::UnknownObjective { candidate_id, objective_id } =>
                write!(f, "candidate {candidate_id} has undeclared objective {objective_id}"),
            Self::UnknownToleranceObjective(id) => write!(f, "tolerance declared for unknown objective {id}"),
            Self::InvalidTolerance { objective_id, value } => write!(f, "tolerance for objective {objective_id} must be finite and non-negative, got {value}"),
            Self::InvalidBound { candidate_id, objective_id, bound } =>
                write!(f, "candidate {candidate_id} objective {objective_id} has invalid bound {bound}"),
            Self::InvertedInterval { candidate_id, objective_id, lower, upper } =>
                write!(f, "candidate {candidate_id} objective {objective_id} has inverted interval [{lower}, {upper}]"),
        }
    }
}
impl std::error::Error for ParetoError {}

#[derive(Clone, Copy)]
struct BenefitInterval {
    worst: f64,
    best: f64,
}

fn validate(policy: &ParetoPolicy, candidates: &[ParetoCandidate]) -> Result<(), ParetoError> {
    if policy.objectives.is_empty() { return Err(ParetoError::NoObjectives); }
    for (objective_id, tolerance) in &policy.tolerances {
        if !policy.objectives.contains_key(objective_id) {
            return Err(ParetoError::UnknownToleranceObjective(objective_id.clone()));
        }
        if !tolerance.is_finite() || *tolerance < 0.0 {
            return Err(ParetoError::InvalidTolerance {
                objective_id: objective_id.clone(),
                value: *tolerance,
            });
        }
    }
    if policy.objectives.keys().any(|id| id.trim().is_empty()) {
        return Err(ParetoError::EmptyObjectiveId);
    }
    let mut seen = BTreeSet::new();
    for candidate in candidates {
        if candidate.candidate_id.trim().is_empty() { return Err(ParetoError::EmptyCandidateId); }
        if !seen.insert(candidate.candidate_id.as_str()) {
            return Err(ParetoError::DuplicateCandidateId(candidate.candidate_id.clone()));
        }
        for (id, interval) in &candidate.objectives {
            if !policy.objectives.contains_key(id) {
                return Err(ParetoError::UnknownObjective {
                    candidate_id: candidate.candidate_id.clone(),
                    objective_id: id.clone(),
                });
            }
            for bound in [interval.lower, interval.upper].into_iter().flatten() {
                if !bound.is_finite() {
                    return Err(ParetoError::InvalidBound {
                        candidate_id: candidate.candidate_id.clone(),
                        objective_id: id.clone(),
                        bound,
                    });
                }
            }
            if let (Some(lower), Some(upper)) = (interval.lower, interval.upper) {
                if lower > upper {
                    return Err(ParetoError::InvertedInterval {
                        candidate_id: candidate.candidate_id.clone(),
                        objective_id: id.clone(),
                        lower,
                        upper,
                    });
                }
            }
        }
    }
    Ok(())
}

fn to_benefit(direction: ObjectiveDirection, interval: MeasurementInterval) -> Option<BenefitInterval> {
    let (lower, upper) = (interval.lower?, interval.upper?);
    Some(match direction {
        ObjectiveDirection::Maximize => BenefitInterval { worst: lower, best: upper },
        ObjectiveDirection::Minimize => BenefitInterval { worst: -upper, best: -lower },
    })
}

/// A dominates B only if A's worst plausible outcome is no worse than B's best
/// plausible outcome on every objective, and strictly better on at least one.
/// This avoids deleting candidates merely because point estimates happen to rank them.
fn robustly_dominates(a: &ParetoCandidate, b: &ParetoCandidate, policy: &ParetoPolicy) -> bool {
    let mut strict = false;
    for (id, direction) in &policy.objectives {
        let Some(ai) = a.objectives.get(id).copied().and_then(|v| to_benefit(*direction, v)) else { return false; };
        let Some(bi) = b.objectives.get(id).copied().and_then(|v| to_benefit(*direction, v)) else { return false; };
        let epsilon = policy.tolerances.get(id).copied().unwrap_or(0.0);
        if ai.worst + epsilon < bi.best { return false; }
        if ai.worst > bi.best + epsilon { strict = true; }
    }
    strict
}

fn complete(candidate: &ParetoCandidate, policy: &ParetoPolicy) -> bool {
    policy.objectives.keys().all(|id| {
        candidate.objectives.get(id).copied().map_or(false, MeasurementInterval::complete)
    })
}

/// Return a deterministic conservative frontier. Exclusions never re-enter the
/// frontier; unresolved gates and incomplete intervals are held for evidence.
pub fn analyze_pareto(candidates: &[ParetoCandidate], policy: &ParetoPolicy) -> Result<ParetoResult, ParetoError> {
    validate(policy, candidates)?;
    let mut eligible: Vec<&ParetoCandidate> = candidates.iter()
        .filter(|c| c.gate == GateDisposition::Eligible && complete(c, policy)).collect();
    eligible.sort_by(|a, b| a.candidate_id.cmp(&b.candidate_id));

    let mut dispositions = BTreeMap::new();
    let mut frontier = Vec::new();

    for candidate in candidates {
        if candidate.gate == GateDisposition::Excluded {
            dispositions.insert(candidate.candidate_id.clone(), CandidateDisposition {
                kind: AssessmentKind::Excluded,
                dominated_by: Vec::new(),
                reasons: vec!["upstream hard-constraint policy excluded this candidate".into()],
            });
            continue;
        }
        if candidate.gate == GateDisposition::NeedsEvidence {
            dispositions.insert(candidate.candidate_id.clone(), CandidateDisposition {
                kind: AssessmentKind::NeedsEvidence,
                dominated_by: Vec::new(),
                reasons: vec!["upstream gate disposition requires additional evidence".into()],
            });
            continue;
        }
        let missing: Vec<String> = policy.objectives.keys()
            .filter(|id| candidate.objectives.get(*id).map_or(true, |v| !v.complete()))
            .cloned().collect();
        if !missing.is_empty() {
            dispositions.insert(candidate.candidate_id.clone(), CandidateDisposition {
                kind: AssessmentKind::NeedsEvidence,
                dominated_by: Vec::new(),
                reasons: missing.into_iter().map(|id| format!("objective {id} lacks a complete uncertainty interval")).collect(),
            });
            continue;
        }
        let mut dominators: Vec<String> = eligible.iter()
            .filter(|other| other.candidate_id != candidate.candidate_id && robustly_dominates(other, candidate, policy))
            .map(|other| other.candidate_id.clone()).collect();
        dominators.sort();
        let kind = if dominators.is_empty() {
            frontier.push(candidate.candidate_id.clone());
            AssessmentKind::Frontier
        } else { AssessmentKind::RobustlyDominated };
        dispositions.insert(candidate.candidate_id.clone(), CandidateDisposition {
            kind, dominated_by: dominators, reasons: Vec::new(),
        });
    }
    frontier.sort();
    Ok(ParetoResult { dispositions, frontier })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    fn policy() -> ParetoPolicy {
        ParetoPolicy {
            objectives: [
                ("access".into(), ObjectiveDirection::Maximize),
                ("carbon".into(), ObjectiveDirection::Minimize),
            ].into_iter().collect(),
            tolerances: BTreeMap::new(),
        }
    }
    fn candidate(id: &str, access: f64, carbon: f64) -> ParetoCandidate {
        ParetoCandidate {
            candidate_id: id.into(),
            gate: GateDisposition::Eligible,
            objectives: [
                ("access".into(), MeasurementInterval::exact(access)),
                ("carbon".into(), MeasurementInterval::exact(carbon)),
            ].into_iter().collect(),
        }
    }

    #[test]
    fn clear_dominance_preserves_tradeoff_frontier() {
        let result = analyze_pareto(&[candidate("A",0.9,20.0),candidate("B",0.7,40.0),candidate("C",0.95,35.0)], &policy()).unwrap();
        assert_eq!(result.frontier, vec!["A","C"]);
        assert_eq!(result.dispositions["B"].dominated_by, vec!["A","C"]);
    }
    #[test]
    fn access_carbon_tradeoff_keeps_both() {
        let result = analyze_pareto(&[candidate("transit",0.95,55.0),candidate("low-carbon",0.6,18.0)], &policy()).unwrap();
        assert_eq!(result.frontier, vec!["low-carbon","transit"]);
    }
    #[test]
    fn excluded_candidate_never_reenters_frontier() {
        let mut bad = candidate("excluded-best",1.0,1.0); bad.gate = GateDisposition::Excluded;
        let result = analyze_pareto(&[bad,candidate("eligible",0.4,80.0)], &policy()).unwrap();
        assert_eq!(result.dispositions["excluded-best"].kind, AssessmentKind::Excluded);
        assert_eq!(result.frontier, vec!["eligible"]);
    }
    #[test]
    fn overlapping_intervals_do_not_create_false_dominance() {
        let mut a = candidate("A",0.0,10.0);
        a.objectives.insert("access".into(),MeasurementInterval{lower:Some(0.7),upper:Some(0.9)});
        a.objectives.insert("carbon".into(),MeasurementInterval{lower:Some(20.0),upper:Some(30.0)});
        let mut b = candidate("B",0.0,10.0);
        b.objectives.insert("access".into(),MeasurementInterval{lower:Some(0.8),upper:Some(0.95)});
        b.objectives.insert("carbon".into(),MeasurementInterval{lower:Some(25.0),upper:Some(35.0)});
        assert_eq!(analyze_pareto(&[a,b],&policy()).unwrap().frontier,vec!["A","B"]);
    }
    #[test]
    fn missing_interval_is_held_for_evidence() {
        let mut a = candidate("unknown-carbon",0.99,5.0);
        a.objectives.insert("carbon".into(),MeasurementInterval{lower:Some(5.0),upper:None});
        let r = analyze_pareto(&[a,candidate("measured",0.5,25.0)],&policy()).unwrap();
        assert_eq!(r.dispositions["unknown-carbon"].kind,AssessmentKind::NeedsEvidence);
        assert_eq!(r.frontier,vec!["measured"]);
    }
    #[test]
    fn minimization_direction_is_respected() {
        let r = analyze_pareto(&[candidate("better",0.8,10.0),candidate("worse",0.8,30.0)],&policy()).unwrap();
        assert_eq!(r.frontier,vec!["better"]);
        assert_eq!(r.dispositions["worse"].dominated_by,vec!["better"]);
    }
    #[test]
    fn input_order_does_not_change_result() {
        let a=candidate("A",0.9,20.0); let b=candidate("B",0.7,40.0); let c=candidate("C",0.95,35.0);
        assert_eq!(analyze_pareto(&[a.clone(),b.clone(),c.clone()],&policy()).unwrap(),analyze_pareto(&[c,a,b],&policy()).unwrap());
    }
    #[test]
    fn criterion_specific_tolerance_respects_objective_units() {
        let mut p = policy();
        p.tolerances.insert("access".into(), 0.1);
        let r = analyze_pareto(&[candidate("near-tie", 0.9, 20.0), candidate("other", 0.85, 20.0)], &p).unwrap();
        assert_eq!(r.frontier, vec!["near-tie", "other"]);
    }

    #[test]
    fn negative_tolerance_is_rejected() {
        let mut p = policy();
        p.tolerances.insert("access".into(), -0.01);
        assert!(matches!(analyze_pareto(&[candidate("A",0.9,20.0)], &p), Err(ParetoError::InvalidTolerance { .. })));
    }

    #[test]
    fn inverted_interval_is_rejected() {
        let mut bad=candidate("bad",0.9,10.0);
        bad.objectives.insert("carbon".into(),MeasurementInterval{lower:Some(30.0),upper:Some(10.0)});
        assert!(matches!(analyze_pareto(&[bad],&policy()),Err(ParetoError::InvertedInterval{..})));
    }

    #[derive(Deserialize)]
    struct BenchmarkCase {
        case_id: String,
        policy: ParetoPolicy,
        candidates: Vec<ParetoCandidate>,
        expected: BTreeMap<String, CandidateDisposition>,
        expected_frontier: Vec<String>,
    }
    #[test]
    fn synthetic_fixture_matches_expected_dispositions() {
        let fixture=include_str!("../../docs/research/fixtures/pareto-site-benchmark-v0.1.jsonl");
        let mut count=0;
        for line in fixture.lines().filter(|l|!l.trim().is_empty()) {
            let case:BenchmarkCase=serde_json::from_str(line).expect("valid JSONL benchmark");
            let result=analyze_pareto(&case.candidates,&case.policy).unwrap_or_else(|e|panic!("{}: {e}",case.case_id));
            assert_eq!(result.dispositions,case.expected,"{}",case.case_id);
            assert_eq!(result.frontier,case.expected_frontier,"{}",case.case_id);
            count+=1;
        }
        assert!(count>=6,"must contain at least six reference cases");
    }
}
