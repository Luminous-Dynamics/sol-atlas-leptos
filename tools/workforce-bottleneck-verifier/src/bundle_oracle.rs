//! Exact tiny-instance intervention-bundle oracle.
//!
//! This module is separate from the diagnosis heuristics. Exhaustive subset
//! enumeration is used only as a reference oracle over a fixed synthetic action
//! catalogue, not as a production workforce allocator.

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Blocker {
    Skills,
    Spares,
    Authorization,
    Dispatch,
    JobQuality,
    DemandQuality,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Funding {
    Confirmed,
    Absent,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    ExecutableBundles,
    HoldFundingAbsent,
    HoldFundingUnknown,
    HoldEvidence,
    NoAdmissibleBundle,
}

#[derive(Clone, Debug)]
struct Problem {
    id: &'static str,
    blockers: Vec<Blocker>,
    funding: Funding,
    evidence_complete: bool,
    supervisor: bool,
    redeployable: bool,
    procurement_authority: bool,
    inspector_route: bool,
    worker_consultation: bool,
    employer_authority: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Decision {
    status: Status,
    bundles: Vec<Vec<&'static str>>,
    subsets_examined: usize,
}

#[derive(Clone, Copy)]
struct Action {
    name: &'static str,
    clears: &'static [Blocker],
    eligible: fn(&Problem) -> bool,
}

fn can_train(p: &Problem) -> bool {
    p.blockers.contains(&Blocker::Skills)
        && p.funding == Funding::Confirmed
        && p.evidence_complete
        && p.supervisor
        // Training does not bypass an unresolved live-work authorization gate.
        && !p.blockers.contains(&Blocker::Authorization)
}

fn can_redeploy(p: &Problem) -> bool {
    p.blockers.contains(&Blocker::Skills)
        && p.funding == Funding::Confirmed
        && p.evidence_complete
        && p.redeployable
        && !p.blockers.contains(&Blocker::Authorization)
}

fn can_procure_spare(p: &Problem) -> bool {
    p.blockers.contains(&Blocker::Spares)
        && p.funding == Funding::Confirmed
        && p.evidence_complete
        && p.procurement_authority
}

fn can_obtain_inspection(p: &Problem) -> bool {
    p.blockers.contains(&Blocker::Authorization)
        && p.funding == Funding::Confirmed
        && p.evidence_complete
        && p.inspector_route
}

fn can_redesign_route(p: &Problem) -> bool {
    p.blockers.contains(&Blocker::Dispatch)
        && p.evidence_complete
        && p.worker_consultation
}

fn can_remediate_job_quality(p: &Problem) -> bool {
    p.blockers.contains(&Blocker::JobQuality)
        && p.funding == Funding::Confirmed
        && p.evidence_complete
        && p.worker_consultation
        && p.employer_authority
}

fn can_dedupe_demand(p: &Problem) -> bool {
    p.blockers.contains(&Blocker::DemandQuality) && p.evidence_complete
}

const ACTIONS: [Action; 7] = [
    Action {
        name: "SUPERVISED_SKILL_DEVELOPMENT",
        clears: &[Blocker::Skills],
        eligible: can_train,
    },
    Action {
        name: "QUALIFIED_REDEPLOYMENT",
        clears: &[Blocker::Skills],
        eligible: can_redeploy,
    },
    Action {
        name: "PROCURE_OR_QUALIFY_VALID_PART_SUBSTITUTE",
        clears: &[Blocker::Spares],
        eligible: can_procure_spare,
    },
    Action {
        name: "OBTAIN_QUALIFIED_INSPECTION_OR_AUTHORIZATION",
        clears: &[Blocker::Authorization],
        eligible: can_obtain_inspection,
    },
    Action {
        name: "REDESIGN_ROUTE_OR_SHIFT_WITH_WORKER_INPUT",
        clears: &[Blocker::Dispatch],
        eligible: can_redesign_route,
    },
    Action {
        name: "REMEDIATE_JOB_QUALITY_WITH_WORKER_CONSULTATION",
        clears: &[Blocker::JobQuality],
        eligible: can_remediate_job_quality,
    },
    Action {
        name: "DEDUPE_AND_VALIDATE_DEMAND",
        clears: &[Blocker::DemandQuality],
        eligible: can_dedupe_demand,
    },
];

fn solve(p: &Problem) -> Decision {
    // External gates are not optimization variables. A solver cannot make a
    // payer appear or turn unknown information into a confirmed fact.
    let held = match p.funding {
        Funding::Absent => Some(Status::HoldFundingAbsent),
        Funding::Unknown => Some(Status::HoldFundingUnknown),
        Funding::Confirmed if !p.evidence_complete => Some(Status::HoldEvidence),
        Funding::Confirmed => None,
    };
    if let Some(status) = held {
        return Decision {
            status,
            bundles: vec![],
            subsets_examined: 0,
        };
    }

    let mut feasible = Vec::new();
    let subset_count = 1usize << ACTIONS.len();
    for mask in 0..subset_count {
        if mask == 0 && !p.blockers.is_empty() {
            continue;
        }

        let selected_are_eligible = ACTIONS.iter().enumerate().all(|(i, action)| {
            mask & (1usize << i) == 0 || (action.eligible)(p)
        });
        if !selected_are_eligible {
            continue;
        }

        let clears_every_blocker = p.blockers.iter().all(|blocker| {
            ACTIONS.iter().enumerate().any(|(i, action)| {
                mask & (1usize << i) != 0 && action.clears.contains(blocker)
            })
        });
        if clears_every_blocker {
            feasible.push(mask);
        }
    }

    // A feasible bundle is inclusion-minimal only if no strict feasible subset
    // already clears every confirmed blocker.
    let minimal_masks: Vec<usize> = feasible
        .iter()
        .copied()
        .filter(|mask| {
            !feasible.iter().copied().any(|other| {
                other != *mask && (other & *mask) == other
            })
        })
        .collect();

    let mut bundles: Vec<Vec<&'static str>> = minimal_masks
        .iter()
        .map(|mask| {
            ACTIONS
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1usize << i) != 0)
                .map(|(_, action)| action.name)
                .collect()
        })
        .collect();
    bundles.sort();

    let status = if bundles.is_empty() {
        Status::NoAdmissibleBundle
    } else {
        Status::ExecutableBundles
    };
    Decision {
        status,
        bundles,
        subsets_examined: subset_count,
    }
}

fn problem(
    id: &'static str,
    blockers: &[Blocker],
    funding: Funding,
    evidence_complete: bool,
    supervisor: bool,
    redeployable: bool,
    procurement_authority: bool,
    inspector_route: bool,
    worker_consultation: bool,
    employer_authority: bool,
) -> Problem {
    Problem {
        id,
        blockers: blockers.to_vec(),
        funding,
        evidence_complete,
        supervisor,
        redeployable,
        procurement_authority,
        inspector_route,
        worker_consultation,
        employer_authority,
    }
}

struct Reference {
    problem: Problem,
    status: Status,
    bundles: Vec<Vec<&'static str>>,
}

fn reference_cases() -> Vec<Reference> {
    use Blocker::{
        Authorization as A, DemandQuality as D, Dispatch as T, JobQuality as J, Skills as S,
        Spares as P,
    };
    use Funding::{Absent as X, Confirmed as C, Unknown as U};
    use Status::{
        ExecutableBundles as E, HoldEvidence as H, HoldFundingAbsent as FA,
        HoldFundingUnknown as FU, NoAdmissibleBundle as N,
    };

    vec![
        Reference {
            problem: problem("BND-001", &[S], C, true, true, false, false, false, true, true),
            status: E,
            bundles: vec![vec!["SUPERVISED_SKILL_DEVELOPMENT"]],
        },
        Reference {
            problem: problem("BND-002", &[S], C, true, true, true, false, false, true, true),
            status: E,
            bundles: vec![
                vec!["QUALIFIED_REDEPLOYMENT"],
                vec!["SUPERVISED_SKILL_DEVELOPMENT"],
            ],
        },
        Reference {
            problem: problem("BND-003", &[S, P], C, true, true, true, true, false, true, true),
            status: E,
            bundles: vec![
                vec![
                    "QUALIFIED_REDEPLOYMENT",
                    "PROCURE_OR_QUALIFY_VALID_PART_SUBSTITUTE",
                ],
                vec![
                    "SUPERVISED_SKILL_DEVELOPMENT",
                    "PROCURE_OR_QUALIFY_VALID_PART_SUBSTITUTE",
                ],
            ],
        },
        Reference {
            problem: problem("BND-004", &[A], C, true, false, false, false, true, false, false),
            status: E,
            bundles: vec![vec!["OBTAIN_QUALIFIED_INSPECTION_OR_AUTHORIZATION"]],
        },
        Reference {
            problem: problem("BND-005", &[T], C, true, false, false, false, false, true, false),
            status: E,
            bundles: vec![vec!["REDESIGN_ROUTE_OR_SHIFT_WITH_WORKER_INPUT"]],
        },
        Reference {
            problem: problem("BND-006", &[J], C, true, false, false, false, false, true, true),
            status: E,
            bundles: vec![vec!["REMEDIATE_JOB_QUALITY_WITH_WORKER_CONSULTATION"]],
        },
        Reference {
            problem: problem("BND-007", &[D], C, true, false, false, false, false, false, false),
            status: E,
            bundles: vec![vec!["DEDUPE_AND_VALIDATE_DEMAND"]],
        },
        Reference {
            problem: problem("BND-008", &[S], X, true, true, true, false, false, true, true),
            status: FA,
            bundles: vec![],
        },
        Reference {
            problem: problem("BND-009", &[S], U, false, true, true, false, false, true, true),
            status: FU,
            bundles: vec![],
        },
        Reference {
            problem: problem("BND-010", &[P], C, true, false, false, false, false, false, false),
            status: N,
            bundles: vec![],
        },
        Reference {
            problem: problem("BND-011", &[S, P], C, true, false, false, true, false, true, true),
            status: N,
            bundles: vec![],
        },
        Reference {
            problem: problem("BND-012", &[T, J], C, true, false, false, false, false, true, true),
            status: E,
            bundles: vec![vec![
                "REDESIGN_ROUTE_OR_SHIFT_WITH_WORKER_INPUT",
                "REMEDIATE_JOB_QUALITY_WITH_WORKER_CONSULTATION",
            ]],
        },
        Reference {
            problem: problem("BND-013", &[S], C, true, false, false, false, false, true, true),
            status: N,
            bundles: vec![],
        },
        Reference {
            problem: problem("BND-014", &[A], C, true, false, false, false, false, true, true),
            status: N,
            bundles: vec![],
        },
        Reference {
            problem: problem("BND-015", &[S], C, false, true, true, false, false, true, true),
            status: H,
            bundles: vec![],
        },
    ]
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct BundleOracleSummary {
    pub status: &'static str,
    pub scenarios: usize,
    pub action_count: usize,
    pub subsets_per_ready_scenario: usize,
    pub subsets_examined: usize,
    pub executable_scenarios: usize,
    pub held_or_infeasible_scenarios: usize,
    pub minimal_bundles_found: usize,
    pub exact_hand_oracle_matches: usize,
    pub claim_ceiling: &'static str,
}

pub fn verify_reference_suite() -> Result<BundleOracleSummary, String> {
    let cases = reference_cases();
    let mut subsets_examined = 0;
    let mut executable = 0;
    let mut held = 0;
    let mut bundle_count = 0;

    for case in &cases {
        let actual = solve(&case.problem);
        if actual.status != case.status || actual.bundles != case.bundles {
            return Err(format!(
                "{}: expected {:?} {:?}, got {:?} {:?}",
                case.problem.id, case.status, case.bundles, actual.status, actual.bundles
            ));
        }
        if actual.status == Status::ExecutableBundles {
            executable += 1;
        } else {
            held += 1;
        }
        subsets_examined += actual.subsets_examined;
        bundle_count += actual.bundles.len();
    }

    Ok(BundleOracleSummary {
        status: "PASS_SYNTHETIC_EXACT_BUNDLE_ORACLE",
        scenarios: cases.len(),
        action_count: ACTIONS.len(),
        subsets_per_ready_scenario: 1usize << ACTIONS.len(),
        subsets_examined,
        executable_scenarios: executable,
        held_or_infeasible_scenarios: held,
        minimal_bundles_found: bundle_count,
        exact_hand_oracle_matches: cases.len(),
        claim_ceiling: "TINY SYNTHETIC BUNDLE CONTRACT ONLY; no field performance or intervention efficacy claim",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhaustive_solver_matches_all_hand_authored_expected_bundles() {
        let summary = verify_reference_suite().expect("bundle reference suite should match");
        assert_eq!(summary.scenarios, 15);
        assert_eq!(summary.exact_hand_oracle_matches, 15);
        assert_eq!(summary.minimal_bundles_found, 10);
    }

    #[test]
    fn skill_and_spare_case_requires_both_intervention_families() {
        let case = reference_cases()
            .into_iter()
            .find(|case| case.problem.id == "BND-003")
            .expect("BND-003 exists");
        let decision = solve(&case.problem);
        assert_eq!(decision.status, Status::ExecutableBundles);
        assert_eq!(decision.bundles.len(), 2);
        for bundle in decision.bundles {
            assert!(bundle
                .iter()
                .any(|action| action.contains("PROCURE_OR_QUALIFY")));
            assert!(bundle.iter().any(|action| {
                *action == "SUPERVISED_SKILL_DEVELOPMENT"
                    || *action == "QUALIFIED_REDEPLOYMENT"
            }));
        }
    }

    #[test]
    fn unknown_funding_yields_no_executable_bundle() {
        let case = reference_cases()
            .into_iter()
            .find(|case| case.problem.id == "BND-009")
            .expect("BND-009 exists");
        let decision = solve(&case.problem);
        assert_eq!(decision.status, Status::HoldFundingUnknown);
        assert!(decision.bundles.is_empty());
    }

    #[test]
    fn missing_supervisor_and_redeployable_capacity_cannot_be_papered_over() {
        let case = reference_cases()
            .into_iter()
            .find(|case| case.problem.id == "BND-011")
            .expect("BND-011 exists");
        let decision = solve(&case.problem);
        assert_eq!(decision.status, Status::NoAdmissibleBundle);
        assert!(decision.bundles.is_empty());
    }

    #[test]
    fn every_reported_bundle_is_inclusion_minimal() {
        for case in reference_cases() {
            let result = solve(&case.problem);
            for bundle in &result.bundles {
                for omitted in 0..bundle.len() {
                    let reduced: Vec<&str> = bundle
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| *index != omitted)
                        .map(|(_, name)| *name)
                        .collect();
                    let cleared: Vec<Blocker> = ACTIONS
                        .iter()
                        .filter(|action| reduced.contains(&action.name))
                        .flat_map(|action| action.clears.iter().copied())
                        .collect();
                    assert!(
                        !case.problem.blockers.iter().all(|blocker| cleared.contains(blocker)),
                        "{} bundle {:?} is not minimal",
                        case.problem.id,
                        bundle
                    );
                }
            }
        }
    }
}
