use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::process;

const FIXTURE_PATH: &str = "docs/research/service-opportunity-resilience/SERVICE_OPPORTUNITY_RESILIENCE_REFERENCE_ORACLE_V0_1.json";
const OUTPUT_SCHEMA: &str = "luminous.service-opportunity-resilience-evaluation";
const STABILITY_OWNER_REFERENCE: &str = "https://github.com/Luminous-Dynamics/symtropy/issues/1848";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema: String,
    schema_version: String,
    maturity: String,
    claim_ceiling: String,
    time_basis: TimeBasis,
    decision_policy: DecisionPolicy,
    cases: Vec<Case>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TimeBasis {
    tick_seconds: u64,
    origin: String,
    tick_semantics: String,
    interval_semantics: String,
    horizon_start_tick: i64,
    horizon_end_tick: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct DecisionPolicy {
    predicate_names: Vec<String>,
    predicate_states: Vec<String>,
    alternative_rule: String,
    opportunity_rule: String,
    capacity_rule: String,
    freshness_rule: String,
    dependency_rule: String,
    stable_recovery_rule: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Scope {
    scope_digest: String,
    population_frame_id: String,
    service_id: String,
    functional_unit_id: String,
    cohort_id: String,
    time_basis_id: String,
    horizon_start_tick: i64,
    horizon_end_tick: i64,
}

impl Scope {
    fn binding(&self) -> ScopeBinding {
        ScopeBinding {
            scope_digest: self.scope_digest.clone(),
            population_frame_id: self.population_frame_id.clone(),
            service_id: self.service_id.clone(),
            functional_unit_id: self.functional_unit_id.clone(),
            cohort_id: self.cohort_id.clone(),
            time_basis_id: self.time_basis_id.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ScopeBinding {
    scope_digest: String,
    population_frame_id: String,
    service_id: String,
    functional_unit_id: String,
    cohort_id: String,
    time_basis_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Demand {
    unit: String,
    required_units: u64,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum PredicateState {
    QualifiedAvailable,
    QualifiedUnavailable,
    Unknown,
    Stale,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Predicates {
    route: PredicateState,
    service: PredicateState,
    capacity: PredicateState,
    suitability: PredicateState,
}

impl Predicates {
    fn values(self) -> [PredicateState; 4] {
        [self.route, self.service, self.capacity, self.suitability]
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum OpportunityState {
    Available,
    Unavailable,
    Unknown,
    NotComparable,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SourceRevisions {
    access: String,
    service: String,
    capacity: String,
    suitability: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AlternativeSlice {
    start_tick: i64,
    end_tick: i64,
    predicates: Predicates,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Alternative {
    alternative_id: String,
    scope_binding: ScopeBinding,
    owner_revisions: SourceRevisions,
    failure_domains: Vec<String>,
    slices: Vec<AlternativeSlice>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum EvidenceClass {
    SyntheticControlled,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DisruptionEvent {
    event_id: String,
    failure_domain: String,
    start_tick: i64,
    end_tick: i64,
    evidence_class: EvidenceClass,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RecoveryState {
    Verified,
    NotVerified,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct StabilityEvidence {
    owner: String,
    owner_reference: String,
    owner_revision: String,
    state: RecoveryState,
    effective_tick: Option<i64>,
    reason_code: String,
    scope_binding: ScopeBinding,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum StableOutputState {
    Verified,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct StableOutput {
    state: StableOutputState,
    tick: Option<i64>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AlternativeCounts {
    available: usize,
    unavailable: usize,
    unknown: usize,
    not_comparable: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct OpportunitySlice {
    start_tick: i64,
    end_tick: i64,
    state: OpportunityState,
    alternative_counts: AlternativeCounts,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct OpportunityInterval {
    start_tick: i64,
    end_tick: i64,
    state: OpportunityState,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    opportunity_slices: Vec<OpportunitySlice>,
    coalesced_opportunity_intervals: Vec<OpportunityInterval>,
    first_available_tick: Option<i64>,
    first_restored_after_known_loss_tick: Option<i64>,
    stable_opportunity_restoration: StableOutput,
    dependency_map_state: DependencyMapState,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum DependencyMapState {
    Complete,
    Incomplete,
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    case_id: String,
    title: String,
    scenario_class: String,
    scope: Scope,
    demand: Demand,
    dependency_map_state: DependencyMapState,
    disruption_events: Vec<DisruptionEvent>,
    alternatives: Vec<Alternative>,
    stability_evidence: StabilityEvidence,
    expected: Expected,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct CaseEvaluation {
    case_id: String,
    scenario_class: String,
    scope: Scope,
    demand: Demand,
    input_alternatives: Vec<Alternative>,
    disruption_events: Vec<DisruptionEvent>,
    stability_evidence: StabilityEvidence,
    opportunity_slices: Vec<OpportunitySlice>,
    coalesced_opportunity_intervals: Vec<OpportunityInterval>,
    first_available_tick: Option<i64>,
    first_restored_after_known_loss_tick: Option<i64>,
    stable_opportunity_restoration: StableOutput,
    dependency_map_state: DependencyMapState,
}

#[derive(Debug, Clone, Serialize)]
struct EvaluationOutput {
    schema: &'static str,
    schema_version: &'static str,
    maturity: &'static str,
    claim_ceiling: String,
    fixture_sha256: String,
    case_count: usize,
    opportunity_interval_count: usize,
    frozen_oracle_match_count: usize,
    mismatch_count: usize,
    results: Vec<CaseEvaluation>,
}

fn ensure_non_empty(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field} must not be empty"))
    } else {
        Ok(())
    }
}

fn validate_fixture(fixture: &Fixture) -> Result<(), String> {
    if fixture.schema != "luminous.service-opportunity-resilience-reference-oracle"
        || fixture.schema_version != "0.1.0"
        || fixture.maturity != "SYNTHETIC_REFERENCE_ORACLE_ONLY"
    {
        return Err("unsupported fixture schema, version or maturity".to_owned());
    }
    ensure_non_empty(&fixture.claim_ceiling, "claim_ceiling")?;

    let expected_names = vec![
        "route".to_owned(),
        "service".to_owned(),
        "capacity".to_owned(),
        "suitability".to_owned(),
    ];
    let expected_states = vec![
        "QUALIFIED_AVAILABLE".to_owned(),
        "QUALIFIED_UNAVAILABLE".to_owned(),
        "UNKNOWN".to_owned(),
        "STALE".to_owned(),
    ];
    let policy = &fixture.decision_policy;
    if policy.predicate_names != expected_names
        || policy.predicate_states != expected_states
        || !policy.alternative_rule.contains("QUALIFIED_UNAVAILABLE")
        || !policy.opportunity_rule.contains("NOT_COMPARABLE")
        || !policy.capacity_rule.contains("does not calculate capacity")
        || !policy.freshness_rule.contains("STALE")
        || !policy.dependency_rule.contains("not probabilities")
        || !policy
            .stable_recovery_rule
            .contains("Do not infer stability")
    {
        return Err("decision policy differs from the frozen v0.1 contract".to_owned());
    }

    let basis = &fixture.time_basis;
    if basis.tick_seconds != 900
        || basis.origin != "SYNTHETIC_HORIZON_START"
        || basis.tick_semantics != "integer 15-minute ticks"
        || basis.interval_semantics != "HALF_OPEN_START_INCLUSIVE_END_EXCLUSIVE"
        || basis.horizon_start_tick != 0
        || basis.horizon_end_tick != 8
    {
        return Err("time basis differs from the frozen v0.1 synthetic profile".to_owned());
    }
    if fixture.cases.len() < 18 {
        return Err("reference fixture must contain at least 18 cases".to_owned());
    }

    let mut case_ids = HashSet::new();
    for case in &fixture.cases {
        if !case_ids.insert(case.case_id.as_str()) {
            return Err(format!("duplicate case_id {}", case.case_id));
        }
        validate_case(case, basis)?;
    }
    Ok(())
}

fn validate_case(case: &Case, basis: &TimeBasis) -> Result<(), String> {
    ensure_non_empty(&case.case_id, "case_id")?;
    ensure_non_empty(&case.title, "case title")?;
    ensure_non_empty(&case.scenario_class, "scenario_class")?;
    ensure_non_empty(&case.scope.scope_digest, "scope_digest")?;
    ensure_non_empty(&case.scope.population_frame_id, "population_frame_id")?;
    ensure_non_empty(&case.scope.service_id, "service_id")?;
    ensure_non_empty(&case.scope.functional_unit_id, "functional_unit_id")?;
    ensure_non_empty(&case.scope.cohort_id, "cohort_id")?;
    ensure_non_empty(&case.scope.time_basis_id, "time_basis_id")?;
    ensure_non_empty(&case.demand.unit, "demand unit")?;

    if case.demand.required_units == 0
        || case.scope.horizon_start_tick != basis.horizon_start_tick
        || case.scope.horizon_end_tick != basis.horizon_end_tick
        || case.scope.horizon_end_tick <= case.scope.horizon_start_tick
    {
        return Err(format!("{}: invalid demand or evaluation horizon", case.case_id));
    }
    if case.alternatives.is_empty() {
        return Err(format!("{}: at least one alternative is required", case.case_id));
    }

    let mut alternative_ids = HashSet::new();
    let mut canonical_grid: Option<Vec<(i64, i64)>> = None;
    for alternative in &case.alternatives {
        ensure_non_empty(&alternative.alternative_id, "alternative_id")?;
        if !alternative_ids.insert(alternative.alternative_id.as_str()) {
            return Err(format!("{}: duplicate alternative_id", case.case_id));
        }
        for (field, value) in [
            ("access revision", alternative.owner_revisions.access.as_str()),
            ("service revision", alternative.owner_revisions.service.as_str()),
            ("capacity revision", alternative.owner_revisions.capacity.as_str()),
            ("suitability revision", alternative.owner_revisions.suitability.as_str()),
        ] {
            ensure_non_empty(value, field)?;
        }

        let mut domains = HashSet::new();
        for domain in &alternative.failure_domains {
            ensure_non_empty(domain, "failure_domain")?;
            if !domains.insert(domain.as_str()) {
                return Err(format!("{}: duplicate failure-domain identifier", case.case_id));
            }
        }

        let binding = &alternative.scope_binding;
        if binding.scope_digest.trim().is_empty()
            || binding.population_frame_id.trim().is_empty()
            || binding.service_id.trim().is_empty()
            || binding.functional_unit_id.trim().is_empty()
            || binding.cohort_id.trim().is_empty()
            || binding.time_basis_id.trim().is_empty()
        {
            return Err(format!("{}: alternative scope binding is incomplete", case.case_id));
        }
        if alternative.slices.len() != 4 {
            return Err(format!("{}: expected exactly four v0.1 time slices", case.case_id));
        }

        let mut cursor = case.scope.horizon_start_tick;
        let mut grid = Vec::with_capacity(alternative.slices.len());
        for slice in &alternative.slices {
            if slice.start_tick != cursor
                || slice.end_tick <= slice.start_tick
                || slice.end_tick > case.scope.horizon_end_tick
            {
                return Err(format!(
                    "{}: slice gap, overlap, ordering error or invalid half-open interval",
                    case.case_id
                ));
            }
            cursor = slice.end_tick;
            grid.push((slice.start_tick, slice.end_tick));
        }
        if cursor != case.scope.horizon_end_tick {
            return Err(format!("{}: slices do not cover the full horizon", case.case_id));
        }
        if let Some(expected_grid) = &canonical_grid {
            if expected_grid != &grid {
                return Err(format!("{}: alternatives use different time partitions", case.case_id));
            }
        } else {
            canonical_grid = Some(grid);
        }
    }

    let mut event_ids = HashSet::new();
    for event in &case.disruption_events {
        ensure_non_empty(&event.event_id, "event_id")?;
        ensure_non_empty(&event.failure_domain, "event failure_domain")?;
        if !event_ids.insert(event.event_id.as_str()) {
            return Err(format!("{}: duplicate disruption event identity", case.case_id));
        }
        if event.start_tick < case.scope.horizon_start_tick
            || event.end_tick <= event.start_tick
            || event.end_tick > case.scope.horizon_end_tick
        {
            return Err(format!("{}: invalid disruption interval", case.case_id));
        }
        match event.evidence_class {
            EvidenceClass::SyntheticControlled => {}
        }
    }

    ensure_non_empty(&case.stability_evidence.owner, "stability evidence owner")?;
    ensure_non_empty(
        &case.stability_evidence.owner_reference,
        "stability evidence owner_reference",
    )?;
    ensure_non_empty(
        &case.stability_evidence.owner_revision,
        "stability evidence owner_revision",
    )?;
    ensure_non_empty(
        &case.stability_evidence.reason_code,
        "stability evidence reason_code",
    )?;
    if case.stability_evidence.state == RecoveryState::Verified {
        match case.stability_evidence.effective_tick {
            Some(tick)
                if case.scope.horizon_start_tick <= tick
                    && tick < case.scope.horizon_end_tick => {}
            _ => {
                return Err(format!(
                    "{}: VERIFIED stability evidence needs an in-horizon effective_tick",
                    case.case_id
                ));
            }
        }
    }
    Ok(())
}

fn classify_predicates(predicates: Predicates) -> OpportunityState {
    let values = predicates.values();
    if values.contains(&PredicateState::QualifiedUnavailable) {
        OpportunityState::Unavailable
    } else if values.contains(&PredicateState::Unknown)
        || values.contains(&PredicateState::Stale)
    {
        OpportunityState::Unknown
    } else {
        OpportunityState::Available
    }
}

fn alternative_state(
    scope: &Scope,
    alternative: &Alternative,
    predicates: Predicates,
) -> OpportunityState {
    if alternative.scope_binding != scope.binding() {
        OpportunityState::NotComparable
    } else {
        classify_predicates(predicates)
    }
}

fn aggregate_alternative_states(states: &[OpportunityState]) -> OpportunityState {
    if states.contains(&OpportunityState::Available) {
        OpportunityState::Available
    } else if states.contains(&OpportunityState::Unknown) {
        OpportunityState::Unknown
    } else if states.contains(&OpportunityState::NotComparable) {
        OpportunityState::NotComparable
    } else {
        OpportunityState::Unavailable
    }
}

fn coalesce(slices: &[OpportunitySlice]) -> Vec<OpportunityInterval> {
    let mut intervals: Vec<OpportunityInterval> = Vec::new();
    for slice in slices {
        if let Some(previous) = intervals.last_mut() {
            if previous.state == slice.state && previous.end_tick == slice.start_tick {
                previous.end_tick = slice.end_tick;
                continue;
            }
        }
        intervals.push(OpportunityInterval {
            start_tick: slice.start_tick,
            end_tick: slice.end_tick,
            state: slice.state,
        });
    }
    intervals
}

fn stable_opportunity_state(case: &Case, slices: &[OpportunitySlice]) -> StableOutput {
    let evidence = &case.stability_evidence;
    if evidence.state == RecoveryState::NotApplicable {
        return StableOutput {
            state: StableOutputState::NotApplicable,
            tick: None,
        };
    }
    if evidence.state != RecoveryState::Verified
        || evidence.owner != "Symtropy"
        || evidence.owner_reference != STABILITY_OWNER_REFERENCE
        || evidence.owner_revision.trim().is_empty()
        || evidence.scope_binding != case.scope.binding()
    {
        return StableOutput {
            state: StableOutputState::Unknown,
            tick: None,
        };
    }

    let Some(tick) = evidence.effective_tick else {
        return StableOutput {
            state: StableOutputState::Unknown,
            tick: None,
        };
    };
    let available_at_tick = slices.iter().any(|slice| {
        slice.state == OpportunityState::Available
            && slice.start_tick <= tick
            && tick < slice.end_tick
    });
    if available_at_tick {
        StableOutput {
            state: StableOutputState::Verified,
            tick: Some(tick),
        }
    } else {
        StableOutput {
            state: StableOutputState::Unknown,
            tick: None,
        }
    }
}

fn evaluate_case(case: &Case) -> CaseEvaluation {
    let mut opportunity_slices = Vec::with_capacity(4);
    for index in 0..4 {
        let alternative_states = case
            .alternatives
            .iter()
            .map(|alternative| {
                alternative_state(
                    &case.scope,
                    alternative,
                    alternative.slices[index].predicates,
                )
            })
            .collect::<Vec<_>>();

        let counts = AlternativeCounts {
            available: alternative_states
                .iter()
                .filter(|state| **state == OpportunityState::Available)
                .count(),
            unavailable: alternative_states
                .iter()
                .filter(|state| **state == OpportunityState::Unavailable)
                .count(),
            unknown: alternative_states
                .iter()
                .filter(|state| **state == OpportunityState::Unknown)
                .count(),
            not_comparable: alternative_states
                .iter()
                .filter(|state| **state == OpportunityState::NotComparable)
                .count(),
        };
        let state = aggregate_alternative_states(&alternative_states);
        let time_slice = &case.alternatives[0].slices[index];

        opportunity_slices.push(OpportunitySlice {
            start_tick: time_slice.start_tick,
            end_tick: time_slice.end_tick,
            state,
            alternative_counts: counts,
        });
    }

    let first_available_tick = opportunity_slices
        .iter()
        .find(|slice| slice.state == OpportunityState::Available)
        .map(|slice| slice.start_tick);

    let mut known_loss_seen = false;
    let mut first_restored_after_known_loss_tick = None;
    for slice in &opportunity_slices {
        if slice.state == OpportunityState::Unavailable {
            known_loss_seen = true;
        } else if known_loss_seen && slice.state == OpportunityState::Available {
            first_restored_after_known_loss_tick = Some(slice.start_tick);
            break;
        }
    }

    let stable_opportunity_restoration =
        stable_opportunity_state(case, &opportunity_slices);
    let coalesced_opportunity_intervals = coalesce(&opportunity_slices);

    // Canonicalize unordered provenance collections so semantically irrelevant
    // input permutations do not change the normalized evaluation payload.
    let mut input_alternatives = case.alternatives.clone();
    for alternative in &mut input_alternatives {
        alternative.failure_domains.sort();
    }
    input_alternatives.sort_by(|left, right| left.alternative_id.cmp(&right.alternative_id));

    let mut disruption_events = case.disruption_events.clone();
    disruption_events.sort_by(|left, right| left.event_id.cmp(&right.event_id));

    CaseEvaluation {
        case_id: case.case_id.clone(),
        scenario_class: case.scenario_class.clone(),
        scope: case.scope.clone(),
        demand: case.demand.clone(),
        input_alternatives,
        disruption_events,
        stability_evidence: case.stability_evidence.clone(),
        opportunity_slices,
        coalesced_opportunity_intervals,
        first_available_tick,
        first_restored_after_known_loss_tick,
        stable_opportunity_restoration,
        dependency_map_state: case.dependency_map_state,
    }
}

fn verify_case(case: &Case, actual: &CaseEvaluation) -> Result<(), String> {
    let expected = &case.expected;
    if actual.opportunity_slices != expected.opportunity_slices {
        return Err(format!("{}: opportunity slice/oracle mismatch", case.case_id));
    }
    if actual.coalesced_opportunity_intervals != expected.coalesced_opportunity_intervals {
        return Err(format!("{}: coalesced interval/oracle mismatch", case.case_id));
    }
    if actual.first_available_tick != expected.first_available_tick {
        return Err(format!("{}: first_available_tick/oracle mismatch", case.case_id));
    }
    if actual.first_restored_after_known_loss_tick
        != expected.first_restored_after_known_loss_tick
    {
        return Err(format!(
            "{}: first_restored_after_known_loss_tick/oracle mismatch",
            case.case_id
        ));
    }
    if actual.stable_opportunity_restoration != expected.stable_opportunity_restoration {
        return Err(format!("{}: stable recovery/oracle mismatch", case.case_id));
    }
    if actual.dependency_map_state != expected.dependency_map_state {
        return Err(format!("{}: dependency map state/oracle mismatch", case.case_id));
    }
    Ok(())
}

fn evaluate_fixture(bytes: &[u8]) -> Result<EvaluationOutput, String> {
    let fixture: Fixture = serde_json::from_slice(bytes)
        .map_err(|error| format!("cannot parse reference fixture: {error}"))?;
    validate_fixture(&fixture)?;

    let mut cases = fixture.cases.clone();
    cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));

    let mut results = Vec::with_capacity(cases.len());
    for case in &cases {
        let result = evaluate_case(case);
        verify_case(case, &result)?;
        results.push(result);
    }

    let opportunity_interval_count = results
        .iter()
        .map(|result| result.opportunity_slices.len())
        .sum();
    let case_count = results.len();
    let digest = Sha256::digest(bytes);
    let fixture_sha256 = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    Ok(EvaluationOutput {
        schema: OUTPUT_SCHEMA,
        schema_version: "0.1.0",
        maturity: "SYNTHETIC_REFERENCE_ORACLE_EXECUTION_ONLY",
        claim_ceiling: fixture.claim_ceiling,
        fixture_sha256,
        case_count,
        opportunity_interval_count,
        frozen_oracle_match_count: case_count,
        mismatch_count: 0,
        results,
    })
}

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| FIXTURE_PATH.to_owned());
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read fixture {path}: {error}");
            process::exit(2);
        }
    };

    match evaluate_fixture(&bytes) {
        Ok(output) => match serde_json::to_string_pretty(&output) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("cannot serialize evaluation output: {error}");
                process::exit(3);
            }
        },
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!(
        "../../../../docs/research/service-opportunity-resilience/SERVICE_OPPORTUNITY_RESILIENCE_REFERENCE_ORACLE_V0_1.json"
    );

    fn fixture() -> Fixture {
        serde_json::from_str(FIXTURE).expect("reference fixture must parse")
    }

    fn by_id<'a>(results: &'a [CaseEvaluation], id: &str) -> &'a CaseEvaluation {
        results
            .iter()
            .find(|result| result.case_id == id)
            .expect("case result exists")
    }

    #[test]
    fn frozen_reference_oracle_matches_all_seventeen_cases() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("all frozen cases must match");
        assert_eq!(output.case_count, 18);
        assert_eq!(output.opportunity_interval_count, 72);
        assert_eq!(output.frozen_oracle_match_count, 18);
        assert_eq!(output.mismatch_count, 0);
        assert_eq!(output.fixture_sha256.len(), 64);
    }

    #[test]
    fn case_input_order_does_not_change_canonical_results() {
        let original = fixture();
        let mut permuted = original.clone();
        permuted.cases.reverse();

        let mut left = original.cases.iter().map(evaluate_case).collect::<Vec<_>>();
        let mut right = permuted.cases.iter().map(evaluate_case).collect::<Vec<_>>();
        left.sort_by(|a, b| a.case_id.cmp(&b.case_id));
        right.sort_by(|a, b| a.case_id.cmp(&b.case_id));
        assert_eq!(left, right);
    }

    #[test]
    fn alternative_order_does_not_change_opportunity_results() {
        let mut case = fixture()
            .cases
            .into_iter()
            .find(|case| case.case_id == "INDEPENDENT_BACKUP_SURVIVES_PRIMARY_FAILURE")
            .expect("backup case exists");
        let before = evaluate_case(&case);
        case.alternatives.reverse();
        for alternative in &mut case.alternatives {
            alternative.failure_domains.reverse();
        }
        case.disruption_events.reverse();
        let after = evaluate_case(&case);
        assert_eq!(before, after);
    }

    #[test]
    fn evaluation_preserves_exact_scope_revisions_events_and_failure_domains() {
        let case = fixture()
            .cases
            .into_iter()
            .find(|case| case.case_id == "SHARED_OPERATOR_COMMON_CAUSE")
            .expect("shared operator case exists");
        let evaluated = evaluate_case(&case);
        assert_eq!(evaluated.scope, case.scope);
        assert_eq!(evaluated.demand.unit, case.demand.unit);
        assert_eq!(evaluated.demand.required_units, case.demand.required_units);
        assert_eq!(evaluated.input_alternatives, case.alternatives);
        assert_eq!(evaluated.disruption_events, case.disruption_events);
        assert_eq!(evaluated.stability_evidence, case.stability_evidence);
        assert!(evaluated.input_alternatives.iter().all(|alternative| {
            alternative
                .failure_domains
                .iter()
                .any(|domain| domain == "SHARED_OPERATOR_X")
        }));
        assert!(evaluated.disruption_events.iter().any(|event| {
            event.failure_domain == "SHARED_OPERATOR_X"
        }));
    }

    #[test]
    fn known_unavailability_dominates_stale_evidence_for_one_alternative() {
        let predicates = Predicates {
            route: PredicateState::QualifiedAvailable,
            service: PredicateState::QualifiedUnavailable,
            capacity: PredicateState::Stale,
            suitability: PredicateState::QualifiedAvailable,
        };
        assert_eq!(
            classify_predicates(predicates),
            OpportunityState::Unavailable
        );
    }

    #[test]
    fn available_alternative_dominates_unknown_alternative() {
        assert_eq!(
            aggregate_alternative_states(&[
                OpportunityState::Unknown,
                OpportunityState::Available,
            ]),
            OpportunityState::Available
        );
    }

    #[test]
    fn unknown_dominates_non_comparable_when_none_is_available() {
        assert_eq!(
            aggregate_alternative_states(&[
                OpportunityState::NotComparable,
                OpportunityState::Unknown,
            ]),
            OpportunityState::Unknown
        );
    }

    #[test]
    fn backup_preserves_opportunity_when_primary_fails() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture matches");
        let result = by_id(
            &output.results,
            "INDEPENDENT_BACKUP_SURVIVES_PRIMARY_FAILURE",
        );
        assert_eq!(result.opportunity_slices[1].alternative_counts.available, 1);
        assert_eq!(
            result.opportunity_slices[1].state,
            OpportunityState::Available
        );
    }

    #[test]
    fn shared_operator_failure_removes_both_nominal_nodes() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture matches");
        let result = by_id(&output.results, "SHARED_OPERATOR_COMMON_CAUSE");
        assert_eq!(result.opportunity_slices[1].alternative_counts.unavailable, 2);
        assert_eq!(
            result.opportunity_slices[1].state,
            OpportunityState::Unavailable
        );
    }

    #[test]
    fn qualified_return_after_an_unknown_interval_is_still_reported() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture matches");
        let result = by_id(
            &output.results,
            "UNRESOLVED_INTERVAL_BEFORE_QUALIFIED_RETURN",
        );
        assert_eq!(result.opportunity_slices[1].state, OpportunityState::Unavailable);
        assert_eq!(result.opportunity_slices[2].state, OpportunityState::Unknown);
        assert_eq!(result.opportunity_slices[3].state, OpportunityState::Available);
        assert_eq!(result.first_restored_after_known_loss_tick, Some(6));
    }

    #[test]
    fn unknown_dependency_map_does_not_erase_current_opportunity() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture matches");
        let result = by_id(
            &output.results,
            "UNKNOWN_COMMON_CAUSE_MAP_BLOCKS_INDEPENDENCE_CLAIM",
        );
        assert!(
            result
                .opportunity_slices
                .iter()
                .all(|slice| slice.state == OpportunityState::Available)
        );
        assert_eq!(result.dependency_map_state, DependencyMapState::Unknown);
    }

    #[test]
    fn wrong_cohort_stable_recovery_evidence_remains_unknown() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture matches");
        let result = by_id(
            &output.results,
            "STABLE_RECOVERY_WRONG_SCOPE_NOT_ACCEPTED",
        );
        assert_eq!(
            result.stable_opportunity_restoration,
            StableOutput {
                state: StableOutputState::Unknown,
                tick: None,
            }
        );
    }

    #[test]
    fn transient_return_before_recurrence_is_not_stable_recovery() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture matches");
        let result = by_id(
            &output.results,
            "RECURRENT_FAILURE_BLOCKS_STABLE_RECOVERY",
        );
        assert_eq!(result.first_restored_after_known_loss_tick, Some(4));
        assert_eq!(
            result.stable_opportunity_restoration.state,
            StableOutputState::Unknown
        );
    }

    #[test]
    fn contiguous_half_open_slices_coalesce_without_a_gap() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture matches");
        let result = by_id(&output.results, "HALF_OPEN_ADJACENCY_HAS_NO_GAP");
        assert_eq!(
            result.coalesced_opportunity_intervals,
            vec![OpportunityInterval {
                start_tick: 0,
                end_tick: 8,
                state: OpportunityState::Available,
            }]
        );
    }

    #[test]
    fn incomplete_calendar_gap_is_unknown_and_complete_gap_is_unavailable() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture matches");
        assert_eq!(
            by_id(
                &output.results,
                "INCOMPLETE_CALENDAR_GAP_REMAINS_UNKNOWN"
            )
            .opportunity_slices[1]
                .state,
            OpportunityState::Unknown
        );
        assert_eq!(
            by_id(&output.results, "COMPLETE_CALENDAR_GAP_PROVES_CLOSURE")
                .opportunity_slices[1]
                .state,
            OpportunityState::Unavailable
        );
    }

    #[test]
    fn restored_route_and_restored_service_are_both_required() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture matches");
        for id in [
            "ROUTE_RESTORED_AFTER_SERVICE",
            "SERVICE_RESTORED_AFTER_ROUTE",
        ] {
            let result = by_id(&output.results, id);
            assert_eq!(
                result.opportunity_slices[2].state,
                OpportunityState::Unavailable
            );
            assert_eq!(result.first_restored_after_known_loss_tick, Some(6));
        }
    }

    #[test]
    fn stale_service_evidence_is_unknown() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture matches");
        assert_eq!(
            by_id(
                &output.results,
                "STALE_SERVICE_EVIDENCE_DURING_OPEN_WINDOW"
            )
            .opportunity_slices[2]
                .state,
            OpportunityState::Unknown
        );
    }

    #[test]
    fn unsuitable_service_is_not_rescued_by_route_proximity() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture matches");
        let result = by_id(
            &output.results,
            "COHORT_SPECIFIC_SUITABILITY_FAILURE",
        );
        assert!(
            result
                .opportunity_slices
                .iter()
                .all(|slice| slice.state == OpportunityState::Unavailable)
        );
    }

    #[test]
    fn scope_mismatched_positive_candidate_is_not_comparable() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture matches");
        let result = by_id(&output.results, "SCOPE_MISMATCH_BLOCKS_COMPARISON");
        assert!(
            result
                .opportunity_slices
                .iter()
                .all(|slice| slice.state == OpportunityState::NotComparable)
        );
    }

    #[test]
    fn slice_gaps_are_rejected() {
        let mut case = fixture().cases.remove(0);
        case.alternatives[0].slices[1].start_tick = 3;
        assert!(validate_case(&case, &fixture().time_basis).is_err());
    }

    #[test]
    fn duplicate_alternative_and_event_ids_are_rejected() {
        let mut case = fixture().cases.remove(0);
        case.alternatives[1].alternative_id =
            case.alternatives[0].alternative_id.clone();
        let basis = fixture().time_basis;
        assert!(validate_case(&case, &basis).is_err());

        let mut compound = fixture()
            .cases
            .into_iter()
            .find(|candidate| candidate.case_id == "OVERLAPPING_COMPOUND_FAILURES")
            .expect("compound case exists");
        compound.disruption_events[1].event_id =
            compound.disruption_events[0].event_id.clone();
        assert!(validate_case(&compound, &basis).is_err());
    }
}
