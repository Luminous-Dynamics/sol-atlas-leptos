use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs, process,
};

const FIXTURE_PATH: &str =
    "tools/shared-amenity-spatial-eval/fixtures/SHARED_AMENITY_SPATIAL_FIXTURES_V0.1.json";
const EXPECTED_SCHEMA: &str = "luminous.shared-amenity-spatial-evaluation-fixtures";
const EXPECTED_VERSION: &str = "0.1.0";
const EXPECTED_MATURITY: &str = "SYNTHETIC_ORACLE_FIXTURES_ONLY";
const EXPECTED_CLAIM_CEILING: &str = "NO_LIVE_JOHANNESBURG_GEOGRAPHY_RESIDENTS_PROVIDER_EVIDENCE_MEASURED_PERFORMANCE_OR_ALLOCATION_CLAIM";
const EXPECTED_ARMS: &[&str] = &[
    "PRIVATE_DUPLICATION",
    "CENTRAL_SHARED",
    "DISTRIBUTED_SHARED",
    "HYBRID_SHARED_PRIVATE",
    "EXTERNAL_COMMERCIAL",
];
const EXPECTED_SCENARIOS: &[&str] = &[
    "COMPACT_HIGH_DENSITY",
    "ACCESSIBLE_ROUTE_DISCONTINUITY",
    "EXTERNAL_PROVIDER_DISTANCE_OR_CLOSURE",
    "UNEQUAL_POPULATION_DISTRIBUTION",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureSet {
    schema: String,
    schema_version: String,
    maturity: String,
    claim_ceiling: String,
    owner_bindings: OwnerBindings,
    fixture_policy: FixturePolicy,
    scenarios: Vec<ScenarioFixture>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerBindings {
    common_scope: String,
    effective_access: String,
    amenity_service: String,
    capacity_feasibility: String,
    household_affordability: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixturePolicy {
    fixture_data_state: String,
    registered_50_case_benchmark_state: String,
    routing_computation: String,
    allocation: String,
    queueing: String,
    capacity_accounting: String,
    aggregate_metrics: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioFixture {
    scenario_id: String,
    scope: EvaluationScope,
    population_groups: Vec<PopulationGroup>,
    demand_units: u64,
    arms: Vec<ProvisionArm>,
    expected: Vec<ArmResult>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvaluationScope {
    scope_digest: String,
    population_digest: String,
    service_id: String,
    functional_unit_id: String,
    period: String,
    service_quality: String,
    maximum_generalized_cost_seconds: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PopulationGroup {
    id: String,
    population: u64,
    demand_units: u64,
    mandatory_access_required: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProvisionArm {
    comparison_arm: String,
    scope_digest: String,
    population_digest: String,
    service_id: String,
    functional_unit_id: String,
    period: String,
    service_quality: String,
    facilities: Vec<Facility>,
    access_records: Vec<AccessRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Facility {
    id: String,
    failure_domain_id: String,
    service_state: String,
    capacity_state: String,
    capacity_units: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AccessRecord {
    group_id: String,
    facility_id: String,
    access_state: String,
    generalized_cost_seconds: Option<u64>,
    mandatory_access_state: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
struct ArmResult {
    comparison_arm: String,
    total_population: u64,
    effective_access_population: u64,
    no_access_population: u64,
    unknown_access_population: u64,
    outside_tolerance_population: u64,
    service_unavailable_population: u64,
    effective_access_coverage_basis_points: u64,
    p50_generalized_cost_seconds: Option<u64>,
    p90_generalized_cost_seconds: Option<u64>,
    worst_decile_mean_generalized_cost_seconds: Option<u64>,
    mandatory_access_gate: String,
    aggregate_capacity_units: Option<u64>,
    demand_units: u64,
    capacity_shortfall_units: Option<u64>,
    capacity_surplus_units: Option<u64>,
    capacity_state: String,
    minimum_independent_failure_domains: Option<u64>,
}

#[derive(Debug, Serialize)]
struct ScenarioEvaluation {
    scenario_id: String,
    scope_digest: String,
    arms: Vec<ArmResult>,
}

#[derive(Debug, Serialize)]
struct EvaluationOutput {
    schema: String,
    schema_version: String,
    maturity: String,
    claim_ceiling: String,
    fixture_sha256: String,
    scenario_count: usize,
    arm_result_count: usize,
    frozen_oracle_match_count: usize,
    evaluations: Vec<ScenarioEvaluation>,
}

// serde_json's normal map deserialization can discard an earlier occurrence of a
// duplicate key. Parse recursively into Value while rejecting duplicates first.
struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueValueVisitor)
    }
}

struct UniqueValueVisitor;

impl<'de> Visitor<'de> for UniqueValueVisitor {
    type Value = UniqueValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value with unique object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::from(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let number = serde_json::Number::from_f64(value)
            .ok_or_else(|| E::custom("non-finite JSON number"))?;
        Ok(UniqueValue(Value::Number(number)))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::String(value)))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        UniqueValue::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(UniqueValue(value)) = sequence.next_element::<UniqueValue>()? {
            values.push(value);
        }
        Ok(UniqueValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!(
                    "duplicate JSON object key: {key}"
                )));
            }
            let UniqueValue(value) = map.next_value::<UniqueValue>()?;
            values.insert(key, value);
        }
        Ok(UniqueValue(Value::Object(values)))
    }
}

fn parse_fixture(bytes: &[u8]) -> Result<FixtureSet, String> {
    let UniqueValue(value) = serde_json::from_slice(bytes)
        .map_err(|error| format!("invalid or ambiguous JSON: {error}"))?;
    serde_json::from_value(value).map_err(|error| format!("fixture schema error: {error}"))
}

fn validate_fixture(fixtures: &FixtureSet) -> Result<(), String> {
    if fixtures.schema != EXPECTED_SCHEMA
        || fixtures.schema_version != EXPECTED_VERSION
        || fixtures.maturity != EXPECTED_MATURITY
        || fixtures.claim_ceiling != EXPECTED_CLAIM_CEILING
    {
        return Err("fixture identity, maturity, version or claim ceiling changed".to_owned());
    }

    let owners = &fixtures.owner_bindings;
    if owners.common_scope != "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/623"
        || owners.effective_access
            != "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/555"
        || owners.amenity_service != "https://github.com/Luminous-Dynamics/mycelix/issues/6328"
        || owners.capacity_feasibility
            != "https://github.com/Luminous-Dynamics/symtropy/issues/3768"
        || owners.household_affordability
            != "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/816"
    {
        return Err("canonical owner bindings changed".to_owned());
    }

    let policy = &fixtures.fixture_policy;
    if policy.fixture_data_state != "SYNTHETIC_ONLY"
        || policy.registered_50_case_benchmark_state != "NOT_EXECUTED"
        || policy.routing_computation != "OWNER_OUTPUT_CONSUMED_ONLY"
        || policy.allocation != "NOT_PERFORMED"
        || policy.queueing != "NOT_PERFORMED"
        || policy.capacity_accounting != "COUNT_EACH_FACILITY_ONCE_PER_ARM"
        || policy.aggregate_metrics != "NO_WEIGHTED_COMPOSITE_SCORE"
    {
        return Err("fixture policy violates the declared scope boundary".to_owned());
    }

    if fixtures.scenarios.len() != EXPECTED_SCENARIOS.len() {
        return Err(format!(
            "expected {} frozen scenarios, found {}",
            EXPECTED_SCENARIOS.len(),
            fixtures.scenarios.len()
        ));
    }

    let expected_scenarios = EXPECTED_SCENARIOS
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<BTreeSet<_>>();
    let mut seen_scenarios = BTreeSet::new();

    for (scenario_index, scenario) in fixtures.scenarios.iter().enumerate() {
        if !seen_scenarios.insert(scenario.scenario_id.clone()) {
            return Err(format!(
                "duplicate scenario identifier at scenarios[{scenario_index}]"
            ));
        }

        if scenario.scope.scope_digest.is_empty()
            || !scenario.scope.scope_digest.starts_with("synthetic-scope:")
            || scenario.scope.population_digest.is_empty()
            || !scenario
                .scope
                .population_digest
                .starts_with("synthetic-population:")
            || scenario.scope.service_id != "COMMUNITY_KITCHEN"
            || scenario.scope.functional_unit_id != "ACCESSIBLE_SHARED_KITCHEN_HOUR"
            || scenario.scope.period != "SYNTHETIC_WEEK"
            || scenario.scope.service_quality != "RESIDENT_USE_ACCESS"
            || scenario.scope.maximum_generalized_cost_seconds == 0
        {
            return Err(format!(
                "scenario {} has an invalid or non-synthetic common scope",
                scenario.scenario_id
            ));
        }

        if scenario.population_groups.is_empty() {
            return Err(format!(
                "scenario {} has no population groups",
                scenario.scenario_id
            ));
        }

        let mut groups = BTreeMap::new();
        let mut total_population = 0_u64;
        let mut total_demand = 0_u64;
        for group in &scenario.population_groups {
            if group.id.trim().is_empty() || group.population == 0 || group.demand_units == 0 {
                return Err(format!(
                    "scenario {} has an empty group identity or zero population/demand",
                    scenario.scenario_id
                ));
            }
            if groups.insert(group.id.as_str(), group).is_some() {
                return Err(format!(
                    "scenario {} has duplicate population group {}",
                    scenario.scenario_id, group.id
                ));
            }
            total_population = total_population
                .checked_add(group.population)
                .ok_or_else(|| "population total overflow".to_owned())?;
            total_demand = total_demand
                .checked_add(group.demand_units)
                .ok_or_else(|| "demand total overflow".to_owned())?;
        }
        if total_population == 0 || total_demand != scenario.demand_units {
            return Err(format!(
                "scenario {} demand does not equal the sum of its resident-group demand",
                scenario.scenario_id
            ));
        }

        if scenario.arms.len() != EXPECTED_ARMS.len() {
            return Err(format!(
                "scenario {} must contain exactly five provision arms",
                scenario.scenario_id
            ));
        }
        let mut seen_arms = BTreeSet::new();

        for arm in &scenario.arms {
            if !seen_arms.insert(arm.comparison_arm.as_str()) {
                return Err(format!(
                    "scenario {} contains duplicate arm {}",
                    scenario.scenario_id, arm.comparison_arm
                ));
            }
            if arm.scope_digest != scenario.scope.scope_digest
                || arm.population_digest != scenario.scope.population_digest
                || arm.service_id != scenario.scope.service_id
                || arm.functional_unit_id != scenario.scope.functional_unit_id
                || arm.period != scenario.scope.period
                || arm.service_quality != scenario.scope.service_quality
            {
                return Err(format!(
                    "scenario {} arm {} is not bound to the common comparison scope",
                    scenario.scenario_id, arm.comparison_arm
                ));
            }

            if arm.facilities.is_empty() {
                return Err(format!(
                    "scenario {} arm {} has no explicit facility/service records",
                    scenario.scenario_id, arm.comparison_arm
                ));
            }
            let mut facilities = BTreeMap::new();
            for facility in &arm.facilities {
                if facility.id.trim().is_empty() || facility.failure_domain_id.trim().is_empty() {
                    return Err(format!(
                        "scenario {} arm {} has an empty facility or failure-domain identity",
                        scenario.scenario_id, arm.comparison_arm
                    ));
                }
                match (
                    facility.service_state.as_str(),
                    facility.capacity_state.as_str(),
                    facility.capacity_units,
                ) {
                    ("AVAILABLE", "KNOWN", Some(_))
                    | ("AVAILABLE", "UNKNOWN", None)
                    | ("UNAVAILABLE", "KNOWN", Some(0))
                    | ("UNKNOWN", "UNKNOWN", None) => {}
                    _ => {
                        return Err(format!(
                            "facility {} has inconsistent service-availability and deliverable-capacity states",
                            facility.id
                        ));
                    }
                }
                if facilities.insert(facility.id.as_str(), facility).is_some() {
                    return Err(format!(
                        "scenario {} arm {} duplicates facility {}",
                        scenario.scenario_id, arm.comparison_arm, facility.id
                    ));
                }
            }

            let mut access_pairs = BTreeSet::new();
            let mut groups_with_records = BTreeSet::new();
            for record in &arm.access_records {
                let group = groups.get(record.group_id.as_str()).ok_or_else(|| {
                    format!(
                        "scenario {} arm {} has access for unknown group {}",
                        scenario.scenario_id, arm.comparison_arm, record.group_id
                    )
                })?;
                if !facilities.contains_key(record.facility_id.as_str()) {
                    return Err(format!(
                        "scenario {} arm {} references unknown facility {}",
                        scenario.scenario_id, arm.comparison_arm, record.facility_id
                    ));
                }
                if !access_pairs.insert((record.group_id.as_str(), record.facility_id.as_str())) {
                    return Err(format!(
                        "scenario {} arm {} duplicates group/facility access pair",
                        scenario.scenario_id, arm.comparison_arm
                    ));
                }
                groups_with_records.insert(record.group_id.as_str());

                match (
                    record.access_state.as_str(),
                    record.generalized_cost_seconds,
                ) {
                    ("QUALIFIED", Some(_)) => {}
                    ("UNREACHABLE" | "UNKNOWN" | "BLOCKED", None) => {}
                    _ => {
                        return Err(format!(
                            "access record for group {} and facility {} has inconsistent state/time",
                            record.group_id, record.facility_id
                        ));
                    }
                }

                match record.mandatory_access_state.as_str() {
                    "PASS" | "FAIL" | "UNKNOWN" | "NOT_REQUIRED" => {}
                    _ => return Err("unknown mandatory-access state".to_owned()),
                }
                if group.mandatory_access_required
                    && record.mandatory_access_state == "NOT_REQUIRED"
                {
                    return Err(format!(
                        "mandatory-access group {} has a NOT_REQUIRED access record",
                        group.id
                    ));
                }
                if !group.mandatory_access_required
                    && record.mandatory_access_state != "NOT_REQUIRED"
                {
                    return Err(format!(
                        "non-mandatory group {} must use NOT_REQUIRED",
                        group.id
                    ));
                }
            }

            if groups_with_records.len() != groups.len() {
                return Err(format!(
                    "scenario {} arm {} omits an explicit access state for one or more groups",
                    scenario.scenario_id, arm.comparison_arm
                ));
            }
            if !EXPECTED_ARMS.contains(&arm.comparison_arm.as_str()) {
                return Err(format!(
                    "scenario {} has an unregistered provision arm {}",
                    scenario.scenario_id, arm.comparison_arm
                ));
            }
        }

        let actual_arms = seen_arms
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<BTreeSet<_>>();
        let expected_arms = EXPECTED_ARMS
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<BTreeSet<_>>();
        if actual_arms != expected_arms {
            return Err(format!(
                "scenario {} does not contain the exact frozen five-arm set",
                scenario.scenario_id
            ));
        }

        if scenario.expected.len() != scenario.arms.len() {
            return Err(format!(
                "scenario {} golden oracle does not cover every provision arm",
                scenario.scenario_id
            ));
        }
    }

    if seen_scenarios != expected_scenarios {
        return Err("scenario identifiers differ from the frozen synthetic corpus".to_owned());
    }
    Ok(())
}

fn weighted_quantile(
    samples: &[(u64, u64)],
    numerator: u64,
    denominator: u64,
) -> Result<Option<u64>, String> {
    if samples.is_empty() {
        return Ok(None);
    }
    if denominator == 0 {
        return Err("weighted quantile denominator must be non-zero".to_owned());
    }
    let total_weight = samples.iter().try_fold(0_u64, |total, sample| {
        total
            .checked_add(sample.1)
            .ok_or_else(|| "weighted population overflow".to_owned())
    })?;
    let scaled = u128::from(total_weight)
        .checked_mul(u128::from(numerator))
        .ok_or_else(|| "weighted quantile overflow".to_owned())?;
    let target = scaled
        .checked_add(u128::from(denominator - 1))
        .ok_or_else(|| "weighted quantile overflow".to_owned())?
        / u128::from(denominator);
    let target = u64::try_from(target).map_err(|_| "weighted quantile target overflow")?;

    let mut ordered = samples.to_vec();
    ordered.sort_by_key(|sample| sample.0);
    let mut cumulative = 0_u64;
    for (seconds, population) in ordered {
        cumulative = cumulative
            .checked_add(population)
            .ok_or_else(|| "weighted population overflow".to_owned())?;
        if cumulative >= target {
            return Ok(Some(seconds));
        }
    }
    Err("weighted quantile could not reach its population target".to_owned())
}

fn worst_decile_mean(samples: &[(u64, u64)]) -> Result<Option<u64>, String> {
    if samples.is_empty() {
        return Ok(None);
    }
    let total_population = samples.iter().try_fold(0_u64, |total, sample| {
        total
            .checked_add(sample.1)
            .ok_or_else(|| "worst-decile population overflow".to_owned())
    })?;
    let target = total_population.div_ceil(10);
    let mut remaining = target;
    let mut weighted_seconds = 0_u128;
    let mut ordered = samples.to_vec();
    ordered.sort_by(|left, right| right.0.cmp(&left.0));

    for (seconds, population) in ordered {
        let included = remaining.min(population);
        let contribution = u128::from(seconds)
            .checked_mul(u128::from(included))
            .ok_or_else(|| "worst-decile weighted time overflow".to_owned())?;
        weighted_seconds = weighted_seconds
            .checked_add(contribution)
            .ok_or_else(|| "worst-decile weighted time overflow".to_owned())?;
        remaining -= included;
        if remaining == 0 {
            break;
        }
    }
    if remaining != 0 {
        return Err("worst-decile sample is incomplete".to_owned());
    }
    let mean = weighted_seconds / u128::from(target);
    Ok(Some(
        u64::try_from(mean).map_err(|_| "worst-decile mean overflow")?,
    ))
}

fn evaluate_arm(scenario: &ScenarioFixture, arm: &ProvisionArm) -> Result<ArmResult, String> {
    let facilities = arm
        .facilities
        .iter()
        .map(|facility| (facility.id.as_str(), facility))
        .collect::<BTreeMap<_, _>>();

    let mut aggregate_capacity = Some(0_u64);
    for facility in &arm.facilities {
        if facility.service_state == "UNKNOWN" || facility.capacity_state == "UNKNOWN" {
            aggregate_capacity = None;
            break;
        }
        // A facility known to be unavailable in the declared period contributes
        // zero deliverable service capacity, even if the physical asset exists.
        if facility.service_state == "UNAVAILABLE" {
            continue;
        }
        if let Some(current) = aggregate_capacity {
            aggregate_capacity = Some(
                current
                    .checked_add(facility.capacity_units.unwrap_or(0))
                    .ok_or_else(|| "aggregate facility capacity overflow".to_owned())?,
            );
        }
    }

    let mut total_population = 0_u64;
    let mut effective_access_population = 0_u64;
    let mut no_access_population = 0_u64;
    let mut unknown_access_population = 0_u64;
    let mut outside_tolerance_population = 0_u64;
    let mut service_unavailable_population = 0_u64;
    let mut samples = Vec::new();
    let mut minimum_failure_domains: Option<usize> = None;
    let mut mandatory_group_count = 0_usize;
    let mut mandatory_failure = false;
    let mut mandatory_unknown = false;

    for group in &scenario.population_groups {
        total_population = total_population
            .checked_add(group.population)
            .ok_or_else(|| "population total overflow".to_owned())?;
        let records = arm
            .access_records
            .iter()
            .filter(|record| record.group_id == group.id)
            .collect::<Vec<_>>();

        let valid_routes = records
            .iter()
            .filter(|record| {
                let service_available = facilities
                    .get(record.facility_id.as_str())
                    .is_some_and(|facility| facility.service_state == "AVAILABLE");
                record.access_state == "QUALIFIED"
                    && service_available
                    && record.generalized_cost_seconds.is_some_and(|seconds| {
                        seconds <= scenario.scope.maximum_generalized_cost_seconds
                    })
                    && (!group.mandatory_access_required || record.mandatory_access_state == "PASS")
            })
            .copied()
            .collect::<Vec<_>>();

        if !valid_routes.is_empty() {
            effective_access_population = effective_access_population
                .checked_add(group.population)
                .ok_or_else(|| "effective-access population overflow".to_owned())?;
            let best_seconds = valid_routes
                .iter()
                .filter_map(|record| record.generalized_cost_seconds)
                .min()
                .ok_or_else(|| "qualified route has no generalized cost".to_owned())?;
            samples.push((best_seconds, group.population));

            let domains = valid_routes
                .iter()
                .map(|record| {
                    facilities
                        .get(record.facility_id.as_str())
                        .map(|facility| facility.failure_domain_id.as_str())
                        .ok_or_else(|| "qualified access references missing facility".to_owned())
                })
                .collect::<Result<BTreeSet<_>, _>>()?;
            let domain_count = domains.len();
            minimum_failure_domains = Some(
                minimum_failure_domains.map_or(domain_count, |current| current.min(domain_count)),
            );
        } else {
            let has_unknown = records.iter().any(|record| {
                let facility = facilities
                    .get(record.facility_id.as_str())
                    .expect("access references were validated");
                let within_tolerance = record.generalized_cost_seconds.is_some_and(|seconds| {
                    seconds <= scenario.scope.maximum_generalized_cost_seconds
                });
                let service_not_known_unavailable = facility.service_state != "UNAVAILABLE";
                let unresolved_route_candidate = record.access_state == "UNKNOWN"
                    && record.mandatory_access_state != "FAIL"
                    && service_not_known_unavailable;
                let unresolved_mandatory_candidate = record.access_state == "QUALIFIED"
                    && within_tolerance
                    && record.mandatory_access_state == "UNKNOWN"
                    && service_not_known_unavailable;
                let unresolved_service_candidate = record.access_state == "QUALIFIED"
                    && within_tolerance
                    && (!group.mandatory_access_required || record.mandatory_access_state == "PASS")
                    && facility.service_state == "UNKNOWN";
                unresolved_route_candidate
                    || unresolved_mandatory_candidate
                    || unresolved_service_candidate
            });
            let has_service_unavailable = records.iter().any(|record| {
                record.access_state == "QUALIFIED"
                    && facilities
                        .get(record.facility_id.as_str())
                        .is_some_and(|facility| facility.service_state == "UNAVAILABLE")
                    && (!group.mandatory_access_required || record.mandatory_access_state == "PASS")
            });
            let known_route_outside_tolerance = records.iter().any(|record| {
                record.access_state == "QUALIFIED"
                    && facilities
                        .get(record.facility_id.as_str())
                        .is_some_and(|facility| facility.service_state == "AVAILABLE")
                    && record
                        .generalized_cost_seconds
                        .is_some_and(|seconds| {
                            seconds > scenario.scope.maximum_generalized_cost_seconds
                        })
                    && (!group.mandatory_access_required || record.mandatory_access_state == "PASS")
            });

            if has_unknown {
                unknown_access_population = unknown_access_population
                    .checked_add(group.population)
                    .ok_or_else(|| "unknown-access population overflow".to_owned())?;
            } else if has_service_unavailable {
                service_unavailable_population = service_unavailable_population
                    .checked_add(group.population)
                    .ok_or_else(|| "service-unavailable population overflow".to_owned())?;
            } else if known_route_outside_tolerance {
                outside_tolerance_population = outside_tolerance_population
                    .checked_add(group.population)
                    .ok_or_else(|| "outside-tolerance population overflow".to_owned())?;
            } else {
                no_access_population = no_access_population
                    .checked_add(group.population)
                    .ok_or_else(|| "no-access population overflow".to_owned())?;
            }
        }

        if group.mandatory_access_required {
            mandatory_group_count += 1;
            if valid_routes.is_empty() {
                let has_unknown = records.iter().any(|record| {
                    let facility = facilities
                        .get(record.facility_id.as_str())
                        .expect("access references were validated");
                    let within_tolerance = record.generalized_cost_seconds.is_some_and(|seconds| {
                        seconds <= scenario.scope.maximum_generalized_cost_seconds
                    });
                    let service_not_known_unavailable = facility.service_state != "UNAVAILABLE";
                    let unresolved_route_candidate = record.access_state == "UNKNOWN"
                        && record.mandatory_access_state != "FAIL"
                        && service_not_known_unavailable;
                    let unresolved_mandatory_candidate = record.access_state == "QUALIFIED"
                        && within_tolerance
                        && record.mandatory_access_state == "UNKNOWN"
                        && service_not_known_unavailable;
                    let unresolved_service_candidate = record.access_state == "QUALIFIED"
                        && within_tolerance
                        && record.mandatory_access_state == "PASS"
                        && facility.service_state == "UNKNOWN";
                    unresolved_route_candidate
                        || unresolved_mandatory_candidate
                        || unresolved_service_candidate
                });
                if has_unknown {
                    // An unresolved alternative could satisfy the resident's
                    // mandatory condition; do not turn partial evidence into FAIL.
                    mandatory_unknown = true;
                } else {
                    mandatory_failure = true;
                }
            }
        }
    }

    let mandatory_access_gate = if mandatory_group_count == 0 {
        "NOT_APPLICABLE"
    } else if mandatory_failure {
        "FAIL"
    } else if mandatory_unknown {
        "UNKNOWN"
    } else {
        "PASS"
    };

    let coverage_basis_points = u64::try_from(
        u128::from(effective_access_population)
            .checked_mul(10_000)
            .ok_or_else(|| "access coverage overflow".to_owned())?
            / u128::from(total_population),
    )
    .map_err(|_| "access coverage conversion overflow".to_owned())?;

    let (capacity_shortfall, capacity_surplus, capacity_state) = match aggregate_capacity {
        None => (None, None, "UNKNOWN"),
        Some(capacity) if capacity < scenario.demand_units => {
            (Some(scenario.demand_units - capacity), Some(0), "SHORTFALL")
        }
        Some(capacity) if capacity > scenario.demand_units => {
            (Some(0), Some(capacity - scenario.demand_units), "SURPLUS")
        }
        Some(_) => (Some(0), Some(0), "BALANCED"),
    };

    Ok(ArmResult {
        comparison_arm: arm.comparison_arm.clone(),
        total_population,
        effective_access_population,
        no_access_population,
        unknown_access_population,
        outside_tolerance_population,
        service_unavailable_population,
        effective_access_coverage_basis_points: coverage_basis_points,
        p50_generalized_cost_seconds: weighted_quantile(&samples, 50, 100)?,
        p90_generalized_cost_seconds: weighted_quantile(&samples, 90, 100)?,
        worst_decile_mean_generalized_cost_seconds: worst_decile_mean(&samples)?,
        mandatory_access_gate: mandatory_access_gate.to_owned(),
        aggregate_capacity_units: aggregate_capacity,
        demand_units: scenario.demand_units,
        capacity_shortfall_units: capacity_shortfall,
        capacity_surplus_units: capacity_surplus,
        capacity_state: capacity_state.to_owned(),
        minimum_independent_failure_domains: minimum_failure_domains.map(|value| value as u64),
    })
}

fn evaluate_fixture(bytes: &[u8]) -> Result<EvaluationOutput, String> {
    let fixtures = parse_fixture(bytes)?;
    validate_fixture(&fixtures)?;

    let mut evaluations = Vec::new();
    let mut result_count = 0_usize;
    let mut oracle_matches = 0_usize;

    for scenario in &fixtures.scenarios {
        let mut results = Vec::new();
        for arm in &scenario.arms {
            let result = evaluate_arm(scenario, arm)?;
            let expected = scenario
                .expected
                .iter()
                .find(|candidate| candidate.comparison_arm == arm.comparison_arm)
                .ok_or_else(|| {
                    format!(
                        "scenario {} has no golden oracle for arm {}",
                        scenario.scenario_id, arm.comparison_arm
                    )
                })?;
            if &result != expected {
                return Err(format!(
                    "frozen oracle mismatch for {} / {}: actual={result:?}, expected={expected:?}",
                    scenario.scenario_id, arm.comparison_arm
                ));
            }
            oracle_matches += 1;
            result_count += 1;
            results.push(result);
        }
        evaluations.push(ScenarioEvaluation {
            scenario_id: scenario.scenario_id.clone(),
            scope_digest: scenario.scope.scope_digest.clone(),
            arms: results,
        });
    }

    let digest = Sha256::digest(bytes);
    let fixture_sha256 = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    Ok(EvaluationOutput {
        schema: "luminous.shared-amenity-spatial-evaluation".to_owned(),
        schema_version: EXPECTED_VERSION.to_owned(),
        maturity: EXPECTED_MATURITY.to_owned(),
        claim_ceiling: EXPECTED_CLAIM_CEILING.to_owned(),
        fixture_sha256,
        scenario_count: evaluations.len(),
        arm_result_count: result_count,
        frozen_oracle_match_count: oracle_matches,
        evaluations,
    })
}

fn main() {
    let mut args = env::args().skip(1);
    let path = match args.next() {
        None => FIXTURE_PATH.to_owned(),
        Some(arg) if arg == "--help" || arg == "-h" => {
            println!("Usage: shared-amenity-spatial-eval [FIXTURES.json]");
            println!("Defaults to {FIXTURE_PATH}");
            return;
        }
        Some(arg) => arg,
    };

    if args.next().is_some() {
        eprintln!("expected at most one fixture path");
        process::exit(2);
    }

    let bytes = fs::read(&path).unwrap_or_else(|error| {
        eprintln!("cannot read {path}: {error}");
        process::exit(2);
    });

    match evaluate_fixture(&bytes) {
        Ok(output) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&output).expect("evaluation output serialization")
            );
        }
        Err(error) => {
            eprintln!("evaluation blocked: {error}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../fixtures/SHARED_AMENITY_SPATIAL_FIXTURES_V0.1.json");

    fn fixture_value() -> Value {
        serde_json::from_str(FIXTURE).expect("fixture JSON must parse for mutation tests")
    }

    fn evaluate_value(value: Value) -> Result<EvaluationOutput, String> {
        let bytes = serde_json::to_vec(&value).expect("test fixture serialization");
        evaluate_fixture(&bytes)
    }

    fn find_arm<'a>(
        output: &'a EvaluationOutput,
        scenario_id: &str,
        arm_id: &str,
    ) -> &'a ArmResult {
        output
            .evaluations
            .iter()
            .find(|scenario| scenario.scenario_id == scenario_id)
            .expect("scenario exists")
            .arms
            .iter()
            .find(|arm| arm.comparison_arm == arm_id)
            .expect("arm exists")
    }

    #[test]
    fn frozen_fixture_corpus_matches_every_golden_result() {
        let output =
            evaluate_fixture(FIXTURE.as_bytes()).expect("synthetic fixture corpus should pass");
        assert_eq!(output.scenario_count, 4);
        assert_eq!(output.arm_result_count, 20);
        assert_eq!(output.frozen_oracle_match_count, 20);
        assert_eq!(output.maturity, "SYNTHETIC_ORACLE_FIXTURES_ONLY");
        assert_eq!(output.fixture_sha256.len(), 64);
    }

    #[test]
    fn shared_facility_capacity_is_counted_once_not_once_per_reachable_group() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture should pass");
        let distributed = find_arm(&output, "COMPACT_HIGH_DENSITY", "DISTRIBUTED_SHARED");
        assert_eq!(distributed.aggregate_capacity_units, Some(100));
        assert_eq!(distributed.demand_units, 100);
        assert_eq!(distributed.capacity_state, "BALANCED");
        assert_eq!(distributed.minimum_independent_failure_domains, Some(2));
    }

    #[test]
    fn mandatory_access_failure_is_not_hidden_by_other_groups_access() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture should pass");
        let central = find_arm(&output, "ACCESSIBLE_ROUTE_DISCONTINUITY", "CENTRAL_SHARED");
        assert_eq!(central.effective_access_coverage_basis_points, 8_000);
        assert_eq!(central.no_access_population, 20);
        assert_eq!(central.mandatory_access_gate, "FAIL");
    }

    #[test]
    fn unknown_access_and_capacity_remain_unknown_not_zero_or_pass() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture should pass");
        let external = find_arm(
            &output,
            "ACCESSIBLE_ROUTE_DISCONTINUITY",
            "EXTERNAL_COMMERCIAL",
        );
        assert_eq!(external.unknown_access_population, 20);
        assert_eq!(external.mandatory_access_gate, "UNKNOWN");
        assert_eq!(external.aggregate_capacity_units, None);
        assert_eq!(external.capacity_shortfall_units, None);
        assert_eq!(external.capacity_state, "UNKNOWN");
    }

    #[test]
    fn colocated_nodes_with_one_common_failure_domain_are_not_independent() {
        let mut value = fixture_value();
        {
            let arms = value["scenarios"][0]["arms"]
                .as_array_mut()
                .expect("provision arm array");
            let distributed = arms
                .iter_mut()
                .find(|arm| arm["comparison_arm"] == "DISTRIBUTED_SHARED")
                .expect("distributed arm");
            let facilities = distributed["facilities"]
                .as_array_mut()
                .expect("facility array");
            let second_node = facilities
                .iter_mut()
                .find(|facility| facility["id"] == "D2")
                .expect("second distributed node");
            second_node["failure_domain_id"] = Value::String("NODE_EAST".to_owned());
        }
        {
            let expected = value["scenarios"][0]["expected"]
                .as_array_mut()
                .expect("golden result array");
            let distributed = expected
                .iter_mut()
                .find(|result| result["comparison_arm"] == "DISTRIBUTED_SHARED")
                .expect("distributed golden result");
            distributed["minimum_independent_failure_domains"] = Value::from(1_u64);
        }

        let output = evaluate_value(value).expect("common-cause dependency is representable");
        let distributed = find_arm(&output, "COMPACT_HIGH_DENSITY", "DISTRIBUTED_SHARED");
        assert_eq!(distributed.minimum_independent_failure_domains, Some(1));
    }

    #[test]
    fn route_outside_resident_tolerance_does_not_count_as_effective_access() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture should pass");
        let external = find_arm(
            &output,
            "UNEQUAL_POPULATION_DISTRIBUTION",
            "EXTERNAL_COMMERCIAL",
        );
        assert_eq!(external.effective_access_population, 20);
        assert_eq!(external.outside_tolerance_population, 80);
        assert_eq!(external.effective_access_coverage_basis_points, 2_000);
        assert_eq!(external.mandatory_access_gate, "FAIL");
    }

    #[test]
    fn common_scope_mismatch_blocks_cross_arm_comparison() {
        let mut value = fixture_value();
        value["scenarios"][0]["arms"][0]["scope_digest"] =
            Value::String("synthetic-scope:wrong".to_owned());
        let error = evaluate_value(value).expect_err("mixed-scope arm must be blocked");
        assert!(error.contains("common comparison scope"));
    }

    #[test]
    fn duplicated_facility_capacity_identity_is_rejected() {
        let mut value = fixture_value();
        let duplicate = value["scenarios"][0]["arms"][0]["facilities"][0].clone();
        value["scenarios"][0]["arms"][0]["facilities"]
            .as_array_mut()
            .expect("facility array")
            .push(duplicate);
        let error = evaluate_value(value).expect_err("duplicate facility must be rejected");
        assert!(error.contains("duplicates facility"));
    }

    #[test]
    fn duplicate_group_facility_access_pair_is_rejected() {
        let mut value = fixture_value();
        let duplicate = value["scenarios"][0]["arms"][0]["access_records"][0].clone();
        value["scenarios"][0]["arms"][0]["access_records"]
            .as_array_mut()
            .expect("access array")
            .push(duplicate);
        let error = evaluate_value(value).expect_err("duplicate access pair must be rejected");
        assert!(error.contains("duplicates group/facility access pair"));
    }

    #[test]
    fn missing_group_access_record_is_rejected_instead_of_assumed_reachable() {
        let mut value = fixture_value();
        value["scenarios"][0]["arms"][0]["access_records"]
            .as_array_mut()
            .expect("access array")
            .retain(|record| record["group_id"] != "EDGE_STEP_FREE");
        let error = evaluate_value(value).expect_err("missing group access must be rejected");
        assert!(error.contains("omits an explicit access state"));
    }

    #[test]
    fn duplicate_json_keys_are_rejected_recursively() {
        let duplicate = br#"{"schema":"luminous.shared-amenity-spatial-evaluation-fixtures","schema":"luminous.shared-amenity-spatial-evaluation-fixtures"}"#;
        let error = evaluate_fixture(duplicate).expect_err("duplicate JSON key must fail closed");
        assert!(error.contains("duplicate JSON object key"));
    }

    #[test]
    fn negative_population_cannot_be_deserialized_as_a_valid_count() {
        let mut value = fixture_value();
        value["scenarios"][0]["population_groups"][0]["population"] = Value::from(-1);
        let error = evaluate_value(value).expect_err("negative population must be rejected");
        assert!(error.contains("fixture schema error"));
    }

    #[test]
    fn reachable_but_closed_service_is_not_an_effective_opportunity() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture should pass");
        let external = find_arm(
            &output,
            "EXTERNAL_PROVIDER_DISTANCE_OR_CLOSURE",
            "EXTERNAL_COMMERCIAL",
        );
        assert_eq!(external.effective_access_population, 0);
        assert_eq!(external.service_unavailable_population, 100);
        assert_eq!(external.no_access_population, 0);
        assert_eq!(external.mandatory_access_gate, "FAIL");
        assert_eq!(external.aggregate_capacity_units, Some(0));
        assert_eq!(external.capacity_shortfall_units, Some(100));
    }

    #[test]
    fn unknown_service_availability_is_not_assumed_open() {
        let mut value = fixture_value();
        let facility = &mut value["scenarios"][0]["arms"][1]["facilities"][0];
        facility["service_state"] = Value::String("UNKNOWN".to_owned());
        facility["capacity_state"] = Value::String("UNKNOWN".to_owned());
        facility["capacity_units"] = Value::Null;
        let expected = value["scenarios"][0]["expected"]
            .as_array_mut()
            .expect("golden result array");
        let expected_central = expected
            .iter_mut()
            .find(|result| result["comparison_arm"] == "CENTRAL_SHARED")
            .expect("central golden result");
        expected_central["total_population"] = Value::from(100_u64);
        expected_central["effective_access_population"] = Value::from(0_u64);
        expected_central["no_access_population"] = Value::from(0_u64);
        expected_central["unknown_access_population"] = Value::from(100_u64);
        expected_central["outside_tolerance_population"] = Value::from(0_u64);
        expected_central["service_unavailable_population"] = Value::from(0_u64);
        expected_central["effective_access_coverage_basis_points"] = Value::from(0_u64);
        expected_central["p50_generalized_cost_seconds"] = Value::Null;
        expected_central["p90_generalized_cost_seconds"] = Value::Null;
        expected_central["worst_decile_mean_generalized_cost_seconds"] = Value::Null;
        expected_central["mandatory_access_gate"] = Value::String("UNKNOWN".to_owned());
        expected_central["aggregate_capacity_units"] = Value::Null;
        expected_central["demand_units"] = Value::from(100_u64);
        expected_central["capacity_shortfall_units"] = Value::Null;
        expected_central["capacity_surplus_units"] = Value::Null;
        expected_central["capacity_state"] = Value::String("UNKNOWN".to_owned());
        expected_central["minimum_independent_failure_domains"] = Value::Null;
        let output = evaluate_value(value).expect("unknown service state is representable");
        let central = find_arm(&output, "COMPACT_HIGH_DENSITY", "CENTRAL_SHARED");
        assert_eq!(central.effective_access_population, 0);
        assert_eq!(central.unknown_access_population, 100);
        assert_eq!(central.mandatory_access_gate, "UNKNOWN");
        assert_eq!(central.capacity_state, "UNKNOWN");
    }

    #[test]
    fn unknown_service_on_a_blocked_mandatory_route_does_not_erase_known_failure() {
        let mut value = fixture_value();
        let central = value["scenarios"][1]["arms"]
            .as_array_mut()
            .expect("arm array")
            .iter_mut()
            .find(|arm| arm["comparison_arm"] == "CENTRAL_SHARED")
            .expect("central arm");
        let facility = &mut central["facilities"][0];
        facility["service_state"] = Value::String("UNKNOWN".to_owned());
        facility["capacity_state"] = Value::String("UNKNOWN".to_owned());
        facility["capacity_units"] = Value::Null;

        let expected = value["scenarios"][1]["expected"]
            .as_array_mut()
            .expect("golden result array");
        let expected_central = expected
            .iter_mut()
            .find(|result| result["comparison_arm"] == "CENTRAL_SHARED")
            .expect("central golden result");
        expected_central["effective_access_population"] = Value::from(0_u64);
        expected_central["no_access_population"] = Value::from(20_u64);
        expected_central["unknown_access_population"] = Value::from(80_u64);
        expected_central["service_unavailable_population"] = Value::from(0_u64);
        expected_central["effective_access_coverage_basis_points"] = Value::from(0_u64);
        expected_central["p50_generalized_cost_seconds"] = Value::Null;
        expected_central["p90_generalized_cost_seconds"] = Value::Null;
        expected_central["worst_decile_mean_generalized_cost_seconds"] = Value::Null;
        expected_central["mandatory_access_gate"] = Value::String("FAIL".to_owned());
        expected_central["aggregate_capacity_units"] = Value::Null;
        expected_central["capacity_shortfall_units"] = Value::Null;
        expected_central["capacity_surplus_units"] = Value::Null;
        expected_central["capacity_state"] = Value::String("UNKNOWN".to_owned());
        expected_central["minimum_independent_failure_domains"] = Value::Null;

        let output = evaluate_value(value).expect("blocked route remains a known failure");
        let result = find_arm(&output, "ACCESSIBLE_ROUTE_DISCONTINUITY", "CENTRAL_SHARED");
        assert_eq!(result.unknown_access_population, 80);
        assert_eq!(result.no_access_population, 20);
        assert_eq!(result.mandatory_access_gate, "FAIL");
    }

    #[test]
    fn aggregate_capacity_overflow_fails_closed() {
        let mut value = fixture_value();
        value["scenarios"][0]["arms"][0]["facilities"][0]["capacity_units"] = Value::from(u64::MAX);
        let error = evaluate_value(value).expect_err("capacity overflow must fail closed");
        assert!(error.contains("aggregate facility capacity overflow"));
    }

    #[test]
    fn no_weighted_composite_score_or_allocation_is_emitted() {
        let output = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture should pass");
        let rendered = serde_json::to_string(&output).expect("output JSON");
        assert!(!rendered.contains("composite_score"));
        assert!(!rendered.contains("allocation_decision"));
        assert!(!rendered.contains("assigned_capacity"));
    }
}
