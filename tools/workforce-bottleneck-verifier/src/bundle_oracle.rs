//! Exact tiny-instance intervention-bundle oracle.
//! Separate from the diagnosis heuristics. Exhaustive subset enumeration is
//! used only as a reference oracle over this fixed synthetic action catalogue.
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Blocker { Skills, Spares, Authorization, Dispatch, JobQuality, DemandQuality }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Funding { Confirmed, Absent, Unknown }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status { ExecutableBundles, HoldFundingAbsent, HoldFundingUnknown, HoldEvidence, NoAdmissibleBundle }

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
struct Decision { status: Status, bundles: Vec<Vec<&'static str>>, subsets_examined: usize }
#[derive(Clone, Copy)]
struct Action { name: &'static str, clears: &'static [Blocker], eligible: fn(&Problem) -> bool }

const ACTIONS: [Action; 7] = [
    Action { name: "SUPERVISED_SKILL_DEVELOPMENT", clears: &[Blocker::Skills], eligible: |p| p.blockers.contains(&Blocker::Skills) && p.funding == Funding::Confirmed && p.evidence_complete && p.supervisor && !p.blockers.contains(&Blocker::Authorization) },
    Action { name: "QUALIFIED_REDEPLOYMENT", clears: &[Blocker::Skills], eligible: |p| p.blockers.contains(&Blocker::Skills) && p.funding == Funding::Confirmed && p.evidence_complete && p.redeployable && !p.blockers.contains(&Blocker::Authorization) },
    Action { name: "PROCURE_OR_QUALIFY_VALID_PART_SUBSTITUTE", clears: &[Blocker::Spares], eligible: |p| p.blockers.contains(&Blocker::Spares) && p.funding == Funding::Confirmed && p.evidence_complete && p.procurement_authority },
    Action { name: "OBTAIN_QUALIFIED_INSPECTION_OR_AUTHORIZATION", clears: &[Blocker::Authorization], eligible: |p| p.blockers.contains(&Blocker::Authorization) && p.funding == Funding::Confirmed && p.evidence_complete && p.inspector_route },
    Action { name: "REDESIGN_ROUTE_OR_SHIFT_WITH_WORKER_INPUT", clears: &[Blocker::Dispatch], eligible: |p| p.blockers.contains(&Blocker::Dispatch) && p.evidence_complete && p.worker_consultation },
    Action { name: "REMEDIATE_JOB_QUALITY_WITH_WORKER_CONSULTATION", clears: &[Blocker::JobQuality], eligible: |p| p.blockers.contains(&Blocker::JobQuality) && p.funding == Funding::Confirmed && p.evidence_complete && p.worker_consultation && p.employer_authority },
    Action { name: "DEDUPE_AND_VALIDATE_DEMAND", clears: &[Blocker::DemandQuality], eligible: |p| p.blockers.contains(&Blocker::DemandQuality) && p.evidence_complete },
];

fn solve(p: &Problem) -> Decision {
    let held = match p.funding {
        Funding::Absent => Some(Status::HoldFundingAbsent),
        Funding::Unknown => Some(Status::HoldFundingUnknown),
        Funding::Confirmed if !p.evidence_complete => Some(Status::HoldEvidence),
        Funding::Confirmed => None,
    };
    if let Some(status) = held {
        return Decision { status, bundles: vec![], subsets_examined: 0 };
    }

    let mut feasible = Vec::new();
    let limit = 1usize << ACTIONS.len();
    for mask in 0..limit {
        if mask == 0 && !p.blockers.is_empty() { continue; }
        let selected_are_eligible = ACTIONS.iter().enumerate().all(|(i, a)| mask & (1 << i) == 0 || (a.eligible)(p));
        if !selected_are_eligible { continue; }
        let covers_all = p.blockers.iter().all(|b| ACTIONS.iter().enumerate().any(|(i, a)| mask & (1 << i) != 0 && a.clears.contains(b)));
        if covers_all { feasible.push(mask); }
    }
    let min_masks: Vec<usize> = feasible.iter().copied().filter(|mask| !feasible.iter().copied().any(|other| other != *mask && (other & *mask) == other)).collect();
    let mut bundles: Vec<Vec<&'static str>> = min_masks.iter().map(|mask| ACTIONS.iter().enumerate().filter(|(i, _)| mask & (1 << i) != 0).map(|(_, a)| a.name).collect()).collect();
    bundles.sort();
    let status = if bundles.is_empty() { Status::NoAdmissibleBundle } else { Status::ExecutableBundles };
    Decision { status, bundles, subsets_examined: limit }
}

fn p(id: &'static str, blockers: &[Blocker], funding: Funding, evidence_complete: bool, supervisor: bool, redeployable: bool, procurement_authority: bool, inspector_route: bool, worker_consultation: bool, employer_authority: bool) -> Problem {
    Problem { id, blockers: blockers.to_vec(), funding, evidence_complete, supervisor, redeployable, procurement_authority, inspector_route, worker_consultation, employer_authority }
}
struct Reference { problem: Problem, status: Status, bundles: Vec<Vec<&'static str>> }

fn reference_cases() -> Vec<Reference> {
    use Blocker::{Authorization as A, DemandQuality as D, Dispatch as T, JobQuality as J, Skills as S, Spares as P};
    use Funding::{Absent as X, Confirmed as C, Unknown as U};
    use Status::{ExecutableBundles as E, HoldEvidence as H, HoldFundingAbsent as FA, HoldFundingUnknown as FU, NoAdmissibleBundle as N};
    vec![
        Reference { problem: p("BND-001", &[S], C, true, true, false, false, false, true, true), status: E, bundles: vec![vec!["SUPERVISED_SKILL_DEVELOPMENT"]] },
        Reference { problem: p("BND-002", &[S], C, true, true, true, false, false, true, true), status: E, bundles: vec![vec!["QUALIFIED_REDEPLOYMENT"], vec!["SUPERVISED_SKILL_DEVELOPMENT"]] },
        Reference { problem: p("BND-003", &[S, P], C, true, true, true, true, false, true, true), status: E, bundles: vec![vec!["QUALIFIED_REDEPLOYMENT", "PROCURE_OR_QUALIFY_VALID_PART_SUBSTITUTE"], vec!["SUPERVISED_SKILL_DEVELOPMENT", "PROCURE_OR_QUALIFY_VALID_PART_SUBSTITUTE"]] },
        Reference { problem: p("BND-004", &[A], C, true, false, false, false, true, false, false), status: E, bundles: vec![vec!["OBTAIN_QUALIFIED_INSPECTION_OR_AUTHORIZATION"]] },
        Reference { problem: p("BND-005", &[T], C, true, false, false, false, false, true, false), status: E, bundles: vec![vec!["REDESIGN_ROUTE_OR_SHIFT_WITH_WORKER_INPUT"]] },
        Reference { problem: p("BND-006", &[J], C, true, false, false, false, false, true, true), status: E, bundles: vec![vec!["REMEDIATE_JOB_QUALITY_WITH_WORKER_CONSULTATION"]] },
        Reference { problem: p("BND-007", &[D], C, true, false, false, false, false, false, false), status: E, bundles: vec![vec!["DEDUPE_AND_VALIDATE_DEMAND"]] },
        Reference { problem: p("BND-008", &[S], X, true, true, true, false, false, true, true), status: FA, bundles: vec![] },
        Reference { problem: p("BND-009", &[S], U, false, true, true, false, false, true, true), status: FU, bundles: vec![] },
        Reference { problem: p("BND-010", &[P], C, true, false, false, false, false, false, false), status: N, bundles: vec![] },
        Reference { problem: p("BND-011", &[S, P], C, true, false, false, true, false, true, true), status: N, bundles: vec![] },
        Reference { problem: p("BND-012", &[T, J], C, true, false, false, false, false, true, true), status: E, bundles: vec![vec!["REDESIGN_ROUTE_OR_SHIFT_WITH_WORKER_INPUT", "REMEDIATE_JOB_QUALITY_WITH_WORKER_CONSULTATION"]] },
        Reference { problem: p("BND-013", &[S], C, true, false, false, false, false, true, true), status: N, bundles: vec![] },
        Reference { problem: p("BND-014", &[A], C, true, false, false, false, false, true, true), status: N, bundles: vec![] },
        Reference { problem: p("BND-015", &[S], C, false, true, true, false, false, true, true), status: H, bundles: vec![] },
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
        let got = solve(&case.problem);
        if got.status != case.status || got.bundles != case.bundles {
            return Err(format!("{}: expected {:?} {:?}, got {:?} {:?}", case.problem.id, case.status, case.bundles, got.status, got.bundles));
        }
        if got.status == Status::ExecutableBundles { executable += 1; } else { held += 1; }
        subsets_examined += got.subsets_examined;
        bundle_count += got.bundles.len();
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
        let case = reference_cases().into_iter().find(|c| c.problem.id == "BND-003").unwrap();
        let decision = solve(&case.problem);
        assert_eq!(decision.status, Status::ExecutableBundles);
        assert_eq!(decision.bundles.len(), 2);
        for bundle in decision.bundles {
            assert!(bundle.iter().any(|a| a.contains("PROCURE_OR_QUALIFY")));
            assert!(bundle.iter().any(|a| *a == "SUPERVISED_SKILL_DEVELOPMENT" || *a == "QUALIFIED_REDEPLOYMENT"));
        }
    }

    #[test]
    fn unknown_funding_yields_no_executable_bundle() {
        let case = reference_cases().into_iter().find(|c| c.problem.id == "BND-009").unwrap();
        let decision = solve(&case.problem);
        assert_eq!(decision.status, Status::HoldFundingUnknown);
        assert!(decision.bundles.is_empty());
    }

    #[test]
    fn missing_supervisor_and_redeployable_capacity_cannot_be_papered_over() {
        let case = reference_cases().into_iter().find(|c| c.problem.id == "BND-011").unwrap();
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
                    let reduced: Vec<&str> = bundle.iter().enumerate().filter(|(i, _)| *i != omitted).map(|(_, name)| *name).collect();
                    let cleared: Vec<Blocker> = ACTIONS.iter().filter(|a| reduced.contains(&a.name)).flat_map(|a| a.clears.iter().copied()).collect();
                    assert!(!case.problem.blockers.iter().all(|b| cleared.contains(b)), "{} bundle {:?} is not minimal", case.problem.id, bundle);
                }
            }
        }
    }
}
