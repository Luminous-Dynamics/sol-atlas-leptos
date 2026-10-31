//! Independent parser and semantic validator for energy-dispatch LP artifacts.
//!
//! This crate intentionally does not depend on, import, or call the LP emitter.
//! It derives the expected integer model from the source request, parses a
//! restricted LP grammar, and compares the two semantic structures exactly.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const VALIDATOR_ID: &str = "energy-dispatch-lp-validator";
pub const VALIDATOR_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MODEL_SCHEMA_VERSION: &str = "energy-dispatch-independent-mip-v0.1";
pub const MAX_REQUEST_BYTES: usize = 1_048_576;
pub const MAX_LP_BYTES: usize = 16 * 1_048_576;
pub const MAX_INTERVALS: usize = 128;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    schema_version: String,
    case_id: String,
    synthetic: bool,
    evidence_class: String,
    evidence_ref: Option<String>,
    evidence_as_of: Option<String>,
    energy_quantum_kwh: f64,
    max_candidate_expansions: u64,
    battery: Battery,
    intervals: Vec<Interval>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Battery {
    capacity_quanta: u16,
    initial_soc_quanta: u16,
    charge_input_limit_quanta: u16,
    discharge_withdrawal_limit_quanta: u16,
    charge_efficiency_numerator: u16,
    charge_efficiency_denominator: u16,
    discharge_efficiency_numerator: u16,
    discharge_efficiency_denominator: u16,
    terminal_reserve_quanta: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Interval {
    interval_id: String,
    start_unix_seconds: i64,
    duration_seconds: u64,
    grid_outage: bool,
    pv_available_quanta: u16,
    grid_import_limit_quanta: u16,
    essential_demand_quanta: u16,
    flexible_demand_quanta: u16,
    grid_cost_units_per_quantum: u32,
    grid_carbon_grams_per_quantum: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveId {
    EssentialUnservedQuanta,
    FlexibleUnservedQuanta,
    OperatingCostUnits,
    GridCarbonGrams,
    CurtailedPvQuanta,
    TerminalSocQuanta,
}

impl ObjectiveId {
    pub const ALL: [Self; 6] = [
        Self::EssentialUnservedQuanta,
        Self::FlexibleUnservedQuanta,
        Self::OperatingCostUnits,
        Self::GridCarbonGrams,
        Self::CurtailedPvQuanta,
        Self::TerminalSocQuanta,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::EssentialUnservedQuanta => "essential_unserved_quanta",
            Self::FlexibleUnservedQuanta => "flexible_unserved_quanta",
            Self::OperatingCostUnits => "operating_cost_units",
            Self::GridCarbonGrams => "grid_carbon_grams",
            Self::CurtailedPvQuanta => "curtailed_pv_quanta",
            Self::TerminalSocQuanta => "terminal_soc_quanta",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|candidate| candidate.as_str() == value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Min,
    Max,
}

impl Direction {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "min" => Some(Self::Min),
            "max" => Some(Self::Max),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableDomain {
    Integer,
    Binary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variable {
    pub domain: VariableDomain,
    pub lower: i64,
    pub upper: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinearRow {
    pub coefficients: BTreeMap<String, i64>,
    pub relation: Relation,
    pub rhs: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    Eq,
    Le,
    Ge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticModel {
    pub schema_version: String,
    pub objective_id: ObjectiveId,
    pub direction: Direction,
    pub variables: BTreeMap<String, Variable>,
    pub objective: BTreeMap<String, i64>,
    pub objective_constant: i64,
    pub constraints: BTreeMap<String, LinearRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReceipt {
    pub receipt_schema: String,
    pub validator_id: String,
    pub validator_version: String,
    pub request_sha256: String,
    pub lp_sha256: String,
    pub model_schema_version: String,
    pub objective_id: ObjectiveId,
    pub direction: Direction,
    pub reference_model_sha256: String,
    pub parsed_model_sha256: String,
    pub equivalent: bool,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(pub String);

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Metadata {
    source_sha256: Option<String>,
    model_schema: Option<String>,
    objective: Option<String>,
    direction: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedLp {
    direction: Direction,
    objective_name: String,
    objective: BTreeMap<String, i64>,
    objective_constant: i64,
    constraints: BTreeMap<String, LinearRow>,
    bounds: BTreeMap<String, (i64, i64)>,
    binaries: BTreeSet<String>,
    generals: BTreeSet<String>,
    metadata: Metadata,
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn error(message: impl Into<String>) -> ValidationError {
    ValidationError(message.into())
}

fn parse_metadata(lines: &[&str]) -> Result<Metadata, ValidationError> {
    fn value(lines: &[&str], key: &str) -> Result<String, ValidationError> {
        let matches = lines
            .iter()
            .filter_map(|line| {
                let (_, tail) = line.split_once(key)?;
                let candidate = tail
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .trim_matches('"');
                (!candidate.is_empty()).then(|| candidate.to_owned())
            })
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [single] => Ok(single.clone()),
            [] => Err(error(format!("required LP metadata key is missing: {key}"))),
            _ => Err(error(format!("duplicate LP metadata key: {key}"))),
        }
    }

    Ok(Metadata {
        source_sha256: Some(value(lines, "source_sha256=")?),
        model_schema: Some(value(lines, "model_schema=")?),
        objective: Some(value(lines, "objective=")?),
        direction: Some(value(lines, "direction=")?),
    })
}

fn parse_expression(
    tokens: &[&str],
    allow_zero_coefficients: bool,
) -> Result<(BTreeMap<String, i64>, i64), ValidationError> {
    if tokens.is_empty() {
        return Err(error("empty linear expression"));
    }

    let mut coefficients = BTreeMap::<String, i64>::new();
    let mut seen_variables = BTreeSet::<String>::new();
    let mut position = 0usize;
    let mut first_term = true;
    while position < tokens.len() {
        let sign = if first_term {
            if tokens[position] == "-" {
                position += 1;
                -1i64
            } else if tokens[position] == "+" {
                return Err(error("leading '+' is not emitted by this LP profile"));
            } else {
                1i64
            }
        } else {
            match tokens[position] {
                "+" => {
                    position += 1;
                    1i64
                }
                "-" => {
                    position += 1;
                    -1i64
                }
                token => return Err(error(format!("missing sign before linear term: {token}"))),
            }
        };

        if position + 1 >= tokens.len() {
            return Err(error("linear term must contain coefficient and variable"));
        }
        let magnitude = tokens[position]
            .parse::<i64>()
            .map_err(|_| error(format!("invalid integer coefficient: {}", tokens[position])))?;
        if magnitude < 0 || (magnitude == 0 && !allow_zero_coefficients) {
            return Err(error("coefficient magnitude is not valid for this LP row"));
        }
        let coefficient = magnitude
            .checked_mul(sign)
            .ok_or_else(|| error("coefficient sign overflow"))?;
        let variable = tokens[position + 1];
        if !valid_identifier(variable) {
            return Err(error(format!("invalid variable identifier: {variable}")));
        }
        if !seen_variables.insert(variable.to_owned()) {
            return Err(error(format!("duplicate variable term: {variable}")));
        }
        if coefficient != 0 {
            coefficients.insert(variable.to_owned(), coefficient);
        }
        position += 2;
        first_term = false;
    }

    Ok((coefficients, 0))
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        && value.as_bytes()[0].is_ascii_alphabetic()
}

fn parse_row(line: &str) -> Result<(String, LinearRow), ValidationError> {
    let (name, body) = line
        .split_once(':')
        .ok_or_else(|| error("constraint row is missing ':'"))?;
    if !valid_identifier(name) {
        return Err(error(format!("invalid constraint name: {name}")));
    }

    let tokens = body.split_whitespace().collect::<Vec<_>>();
    let relation_index = tokens
        .iter()
        .position(|token| matches!(*token, "=" | "<=" | ">="))
        .ok_or_else(|| error(format!("constraint {name} has no relation")))?;
    if relation_index == 0 || relation_index + 2 != tokens.len() {
        return Err(error(format!("malformed constraint row: {name}")));
    }
    // The emitter can produce explicit zero coefficient terms when a
    // request-side capacity/limit is zero. They are semantically neutral and
    // are removed from the canonical coefficient map, but duplicate variables
    // remain rejected by the parser.
    let (coefficients, constant) = parse_expression(&tokens[..relation_index], true)?;
    if constant != 0 {
        return Err(error("unexpected expression constant"));
    }
    if coefficients.is_empty() {
        return Err(error(format!("constraint {name} has no non-zero coefficients")));
    }
    let rhs = tokens[relation_index + 1]
        .parse::<i64>()
        .map_err(|_| error(format!("invalid RHS in constraint {name}")))?;
    let relation = match tokens[relation_index] {
        "=" => Relation::Eq,
        "<=" => Relation::Le,
        ">=" => Relation::Ge,
        _ => unreachable!(),
    };
    Ok((
        name.to_owned(),
        LinearRow {
            coefficients,
            relation,
            rhs,
        },
    ))
}

fn parse_bound(line: &str) -> Result<(String, (i64, i64)), ValidationError> {
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    if tokens.len() != 5 || tokens[0] != "0" || tokens[1] != "<=" || tokens[3] != "<=" {
        return Err(error(format!("unsupported bound syntax: {line}")));
    }
    if !valid_identifier(tokens[2]) {
        return Err(error(format!("invalid bound variable: {}", tokens[2])));
    }
    let upper = tokens[4]
        .parse::<i64>()
        .map_err(|_| error(format!("invalid upper bound: {}", tokens[4])))?;
    if upper < 0 {
        return Err(error("negative variable upper bound"));
    }
    Ok((tokens[2].to_owned(), (0, upper)))
}

fn parse_lp(lp_bytes: &[u8]) -> Result<ParsedLp, ValidationError> {
    if lp_bytes.len() > MAX_LP_BYTES {
        return Err(error(format!("LP exceeds {MAX_LP_BYTES} byte limit")));
    }
    let source = std::str::from_utf8(lp_bytes).map_err(|_| error("LP is not UTF-8"))?;
    let all_lines = source.lines().collect::<Vec<_>>();
    let metadata = parse_metadata(
        &all_lines
            .iter()
            .copied()
            .filter(|line| line.trim_start().starts_with('\\'))
            .collect::<Vec<_>>(),
    )?;
    let lines = all_lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('\\'))
        .collect::<Vec<_>>();

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Section {
        Constraints,
        Bounds,
        Binaries,
        Generals,
        Ended,
    }

    if lines.len() < 7 {
        return Err(error("LP is truncated"));
    }
    let direction = match lines[0] {
        "Minimize" => Direction::Min,
        "Maximize" => Direction::Max,
        _ => return Err(error("LP must begin with Minimize or Maximize")),
    };

    if lines.get(2) != Some(&"Subject To") {
        return Err(error("LP must contain one objective row followed by Subject To"));
    }

    let objective_line = lines[1];
    let (objective_name, objective_body) = objective_line
        .split_once(':')
        .ok_or_else(|| error("objective is missing ':'"))?;
    if objective_name != "obj" {
        return Err(error("objective row must be named 'obj'"));
    }
    let (objective, objective_constant) =
        parse_expression(&objective_body.split_whitespace().collect::<Vec<_>>(), true)?;

    let mut section = Section::Constraints;
    let mut constraints = BTreeMap::new();
    let mut bounds = BTreeMap::new();
    let mut binaries = BTreeSet::new();
    let mut generals = BTreeSet::new();
    let mut seen_bounds = false;
    let mut seen_binaries = false;
    let mut seen_generals = false;
    let mut seen_end = false;

    for line in lines.iter().skip(3) {
        match *line {
            "Bounds" => {
                if section != Section::Constraints || seen_bounds {
                    return Err(error("misplaced or duplicate Bounds section"));
                }
                seen_bounds = true;
                section = Section::Bounds;
                continue;
            }
            "Binaries" => {
                if section != Section::Bounds || seen_binaries {
                    return Err(error("misplaced or duplicate Binaries section"));
                }
                seen_binaries = true;
                section = Section::Binaries;
                continue;
            }
            "Generals" => {
                if section != Section::Binaries || seen_generals {
                    return Err(error("misplaced or duplicate Generals section"));
                }
                seen_generals = true;
                section = Section::Generals;
                continue;
            }
            "End" => {
                if section != Section::Generals || seen_end {
                    return Err(error("misplaced or duplicate End marker"));
                }
                seen_end = true;
                section = Section::Ended;
                continue;
            }
            _ => {}
        }

        match section {
            Section::Constraints => {
                let (name, row) = parse_row(line)?;
                if constraints.insert(name.clone(), row).is_some() {
                    return Err(error(format!("duplicate constraint name: {name}")));
                }
            }
            Section::Bounds => {
                let (name, bound) = parse_bound(line)?;
                if bounds.insert(name.clone(), bound).is_some() {
                    return Err(error(format!("duplicate bound: {name}")));
                }
            }
            Section::Binaries => {
                if !valid_identifier(line) || !binaries.insert((*line).to_owned()) {
                    return Err(error(format!("invalid or duplicate binary declaration: {line}")));
                }
            }
            Section::Generals => {
                if !valid_identifier(line) || !generals.insert((*line).to_owned()) {
                    return Err(error(format!("invalid or duplicate general declaration: {line}")));
                }
            }
            Section::Ended => {
                return Err(error("content after End or outside a valid LP section"));
            }
        }
    }

    if !seen_bounds || !seen_binaries || !seen_generals || !seen_end || section != Section::Ended {
        return Err(error("LP is missing a required section or End marker"));
    }

    Ok(ParsedLp {
        direction,
        objective_name: objective_name.to_owned(),
        objective,
        objective_constant,
        constraints,
        bounds,
        binaries,
        generals,
        metadata,
    })
}

fn add_variable(
    model: &mut SemanticModel,
    name: String,
    domain: VariableDomain,
    upper: u64,
) -> Result<(), ValidationError> {
    let upper = i64::try_from(upper).map_err(|_| error("variable upper bound exceeds i64"))?;
    if model
        .variables
        .insert(
            name.clone(),
            Variable {
                domain,
                lower: 0,
                upper,
            },
        )
        .is_some()
    {
        return Err(error(format!("duplicate reference variable: {name}")));
    }
    Ok(())
}

fn add_row(
    model: &mut SemanticModel,
    name: String,
    coefficients: Vec<(i64, String)>,
    relation: Relation,
    rhs: i64,
) -> Result<(), ValidationError> {
    let mut canonical = BTreeMap::new();
    let mut seen_variables = BTreeSet::new();
    for (coefficient, variable) in coefficients {
        if !seen_variables.insert(variable.clone()) {
            return Err(error(format!("duplicate variable in reference row {name}")));
        }
        if coefficient != 0 {
            canonical.insert(variable, coefficient);
        }
    }
    if canonical.is_empty() {
        return Err(error(format!("reference row {name} has no non-zero coefficients")));
    }
    if model
        .constraints
        .insert(
            name.clone(),
            LinearRow {
                coefficients: canonical,
                relation,
                rhs,
            },
        )
        .is_some()
    {
        return Err(error(format!("duplicate reference row: {name}")));
    }
    Ok(())
}

fn indexed(prefix: &str, index: usize) -> String {
    format!("{prefix}_{index:03}")
}

/// Independently derive the integer LP semantics from the request DTO.
/// This function does not share model-generation helpers with the LP emitter.
pub fn derive_reference_model(
    request_bytes: &[u8],
    objective_id: ObjectiveId,
    direction: Direction,
) -> Result<SemanticModel, ValidationError> {
    if request_bytes.len() > MAX_REQUEST_BYTES {
        return Err(error(format!("request exceeds {MAX_REQUEST_BYTES} byte limit")));
    }
    let request: Request =
        serde_json::from_slice(request_bytes).map_err(|parse_error| error(format!("request parse failed: {parse_error}")))?;
    validate_request(&request)?;

    let mut model = SemanticModel {
        schema_version: MODEL_SCHEMA_VERSION.to_owned(),
        objective_id,
        direction,
        variables: BTreeMap::new(),
        objective: BTreeMap::new(),
        objective_constant: 0,
        constraints: BTreeMap::new(),
    };

    for (index, interval) in request.intervals.iter().enumerate() {
        let discharge_limit = request
            .battery
            .discharge_withdrawal_limit_quanta
            .min(request.battery.capacity_quanta);
        let charge_gain_limit = u64::from(request.battery.charge_input_limit_quanta)
            .checked_mul(u64::from(request.battery.charge_efficiency_numerator))
            .ok_or_else(|| error("charge bound overflow"))?
            / u64::from(request.battery.charge_efficiency_denominator);
        let delivered_limit = u64::from(discharge_limit)
            .checked_mul(u64::from(request.battery.discharge_efficiency_numerator))
            .ok_or_else(|| error("discharge bound overflow"))?
            / u64::from(request.battery.discharge_efficiency_denominator);

        let variables = [
            ("grid_import", if interval.grid_outage { 0 } else { u64::from(interval.grid_import_limit_quanta) }),
            ("charge_input", u64::from(request.battery.charge_input_limit_quanta)),
            ("soc_withdrawal", u64::from(discharge_limit)),
            ("charge_gain", charge_gain_limit),
            ("discharge_delivered", delivered_limit),
            ("soc_after", u64::from(request.battery.capacity_quanta)),
            ("essential_served", u64::from(interval.essential_demand_quanta)),
            ("essential_unserved", u64::from(interval.essential_demand_quanta)),
            ("flexible_served", u64::from(interval.flexible_demand_quanta)),
            ("flexible_unserved", u64::from(interval.flexible_demand_quanta)),
            ("pv_used", u64::from(interval.pv_available_quanta)),
            ("pv_curtailed", u64::from(interval.pv_available_quanta)),
        ];
        for (prefix, upper) in variables {
            add_variable(
                &mut model,
                indexed(prefix, index),
                VariableDomain::Integer,
                upper,
            )?;
        }
        add_variable(
            &mut model,
            indexed("charge_mode", index),
            VariableDomain::Binary,
            1,
        )?;

        let soc = indexed("soc_after", index);
        let withdrawal = indexed("soc_withdrawal", index);
        let charge_gain = indexed("charge_gain", index);
        let charge_input = indexed("charge_input", index);
        let mode = indexed("charge_mode", index);
        let prev_soc = if index == 0 {
            None
        } else {
            Some(indexed("soc_after", index - 1))
        };

        let mut transition = vec![(1, soc.clone()), (1, withdrawal.clone()), (-1, charge_gain.clone())];
        if let Some(previous) = &prev_soc {
            transition.push((-1, previous.clone()));
        }
        add_row(
            &mut model,
            indexed("soc_transition", index),
            transition,
            Relation::Eq,
            if index == 0 { i64::from(request.battery.initial_soc_quanta) } else { 0 },
        )?;

        let mut withdrawal_terms = vec![(1, withdrawal.clone())];
        if let Some(previous) = &prev_soc {
            withdrawal_terms.push((-1, previous.clone()));
        }
        add_row(
            &mut model,
            indexed("withdrawal_prev_soc", index),
            withdrawal_terms,
            Relation::Le,
            if index == 0 { i64::from(request.battery.initial_soc_quanta) } else { 0 },
        )?;

        add_row(
            &mut model,
            indexed("charge_mode_limit", index),
            vec![
                (1, charge_input.clone()),
                (-i64::from(request.battery.charge_input_limit_quanta), mode.clone()),
            ],
            Relation::Le,
            0,
        )?;

        add_row(
            &mut model,
            indexed("discharge_mode_limit", index),
            vec![
                (1, withdrawal.clone()),
                (i64::from(discharge_limit), mode),
            ],
            Relation::Le,
            i64::from(discharge_limit),
        )?;

        let cn = i64::from(request.battery.charge_efficiency_numerator);
        let cd = i64::from(request.battery.charge_efficiency_denominator);
        add_row(
            &mut model,
            indexed("charge_floor_lower", index),
            vec![(cd, charge_gain.clone()), (-cn, charge_input.clone())],
            Relation::Le,
            0,
        )?;
        add_row(
            &mut model,
            indexed("charge_floor_upper", index),
            vec![(cn, charge_input), (-cd, charge_gain)],
            Relation::Le,
            cd - 1,
        )?;

        let discharge_num = i64::from(request.battery.discharge_efficiency_numerator);
        let discharge_den = i64::from(request.battery.discharge_efficiency_denominator);
        let delivered = indexed("discharge_delivered", index);
        add_row(
            &mut model,
            indexed("discharge_floor_lower", index),
            vec![(discharge_den, delivered.clone()), (-discharge_num, withdrawal.clone())],
            Relation::Le,
            0,
        )?;
        add_row(
            &mut model,
            indexed("discharge_floor_upper", index),
            vec![(discharge_num, withdrawal), (-discharge_den, delivered)],
            Relation::Le,
            discharge_den - 1,
        )?;

        let essential_served = indexed("essential_served", index);
        let essential_unserved = indexed("essential_unserved", index);
        let flexible_served = indexed("flexible_served", index);
        let flexible_unserved = indexed("flexible_unserved", index);
        let pv_used = indexed("pv_used", index);
        let pv_curtailed = indexed("pv_curtailed", index);
        let grid_import = indexed("grid_import", index);

        add_row(
            &mut model,
            indexed("essential_partition", index),
            vec![(1, essential_served.clone()), (1, essential_unserved.clone())],
            Relation::Eq,
            i64::from(interval.essential_demand_quanta),
        )?;
        add_row(
            &mut model,
            indexed("flexible_partition", index),
            vec![(1, flexible_served.clone()), (1, flexible_unserved.clone())],
            Relation::Eq,
            i64::from(interval.flexible_demand_quanta),
        )?;
        add_row(
            &mut model,
            indexed("pv_partition", index),
            vec![(1, pv_used.clone()), (1, pv_curtailed.clone())],
            Relation::Eq,
            i64::from(interval.pv_available_quanta),
        )?;
        add_row(
            &mut model,
            indexed("bus_balance", index),
            vec![
                (1, pv_used),
                (1, grid_import.clone()),
                (1, delivered),
                (-1, essential_served),
                (-1, flexible_served),
                (-1, charge_input),
            ],
            Relation::Eq,
            0,
        )?;

        let objective_terms = match objective_id {
            ObjectiveId::EssentialUnservedQuanta => {
                vec![(1, essential_unserved)]
            }
            ObjectiveId::FlexibleUnservedQuanta => {
                vec![(1, flexible_unserved)]
            }
            ObjectiveId::OperatingCostUnits => {
                vec![(i64::from(interval.grid_cost_units_per_quantum), grid_import)]
            }
            ObjectiveId::GridCarbonGrams => {
                vec![(i64::from(interval.grid_carbon_grams_per_quantum), grid_import)]
            }
            ObjectiveId::CurtailedPvQuanta => {
                vec![(1, pv_curtailed)]
            }
            ObjectiveId::TerminalSocQuanta if index + 1 == request.intervals.len() => {
                vec![(1, soc)]
            }
            ObjectiveId::TerminalSocQuanta => Vec::new(),
        };
        for (coefficient, variable) in objective_terms {
            if coefficient != 0 {
                if model.objective.insert(variable.clone(), coefficient).is_some() {
                    return Err(error(format!("duplicate objective term: {variable}")));
                }
            }
        }
    }

    let terminal = indexed("soc_after", request.intervals.len() - 1);
    add_row(
        &mut model,
        "terminal_reserve".to_owned(),
        vec![(1, terminal)],
        Relation::Ge,
        i64::from(request.battery.terminal_reserve_quanta),
    )?;

    // An objective with all-zero coefficients is valid: it represents
    // a constant-zero objective, which the emitter writes as explicit zero
    // terms. It must not be confused with a missing objective row.
    Ok(model)
}

fn validate_request(request: &Request) -> Result<(), ValidationError> {
    if request.schema_version != "energy-dispatch-v0.1" {
        return Err(error("unsupported dispatch request schema"));
    }
    if request.case_id.trim().is_empty() {
        return Err(error("case_id must not be empty"));
    }
    if !request.energy_quantum_kwh.is_finite() || request.energy_quantum_kwh <= 0.0 {
        return Err(error("energy_quantum_kwh must be finite and positive"));
    }
    if request.max_candidate_expansions == 0 || request.max_candidate_expansions > 1_000_000 {
        return Err(error("max_candidate_expansions is outside [1, 1000000]"));
    }
    if request.intervals.is_empty() || request.intervals.len() > MAX_INTERVALS {
        return Err(error(format!("interval count must be in [1, {MAX_INTERVALS}]")));
    }
    if request.evidence_class != "analyst_assumption"
        && request.evidence_ref.as_deref().is_none_or(|value| value.trim().is_empty())
    {
        return Err(error("non-assumption evidence requires a non-empty evidence_ref"));
    }
    let _evidence_context = (request.synthetic, request.evidence_as_of.as_deref());
    let battery = &request.battery;
    if battery.initial_soc_quanta > battery.capacity_quanta
        || battery.terminal_reserve_quanta > battery.capacity_quanta
    {
        return Err(error("initial SOC and terminal reserve must not exceed capacity"));
    }
    for (name, numerator, denominator) in [
        ("charge", battery.charge_efficiency_numerator, battery.charge_efficiency_denominator),
        ("discharge", battery.discharge_efficiency_numerator, battery.discharge_efficiency_denominator),
    ] {
        if numerator == 0 || denominator == 0 || numerator > denominator {
            return Err(error(format!("{name} efficiency ratio is invalid")));
        }
    }

    let mut ids = BTreeSet::new();
    let mut expected_start = None::<i64>;
    for interval in &request.intervals {
        if interval.interval_id.trim().is_empty() || !ids.insert(interval.interval_id.as_str()) {
            return Err(error("interval IDs must be non-empty and unique"));
        }
        if interval.duration_seconds == 0 {
            return Err(error(format!("interval {} has zero duration", interval.interval_id)));
        }
        if let Some(expected) = expected_start {
            if expected != interval.start_unix_seconds {
                return Err(error(format!("interval {} breaks chronology", interval.interval_id)));
            }
        }
        let duration = i64::try_from(interval.duration_seconds)
            .map_err(|_| error("interval duration exceeds signed chronology range"))?;
        expected_start = Some(
            interval
                .start_unix_seconds
                .checked_add(duration)
                .ok_or_else(|| error("interval chronology arithmetic overflow"))?,
        );
    }
    Ok(())
}

fn build_parsed_model(parsed: ParsedLp, objective_id: ObjectiveId) -> Result<SemanticModel, ValidationError> {
    if parsed.objective_name != "obj" {
        return Err(error("unexpected objective row name"));
    }
    let mut variables = BTreeMap::new();
    for name in &parsed.generals {
        if parsed.binaries.contains(name) {
            return Err(error(format!("variable declared as both binary and general: {name}")));
        }
        let (lower, upper) = parsed
            .bounds
            .get(name)
            .copied()
            .ok_or_else(|| error(format!("general integer variable has no explicit bound: {name}")))?;
        variables.insert(
            name.clone(),
            Variable {
                domain: VariableDomain::Integer,
                lower,
                upper,
            },
        );
    }
    for name in &parsed.binaries {
        if parsed.bounds.contains_key(name) {
            return Err(error(format!("binary variable must not also have an explicit bound: {name}")));
        }
        variables.insert(
            name.clone(),
            Variable {
                domain: VariableDomain::Binary,
                lower: 0,
                upper: 1,
            },
        );
    }
    if variables.len() != parsed.generals.len() + parsed.binaries.len() {
        return Err(error("variable declarations are not unique"));
    }
    if parsed.bounds.keys().any(|name| !parsed.generals.contains(name)) {
        return Err(error("bound exists for an undeclared general integer variable"));
    }

    for row in parsed.constraints.values() {
        for variable in row.coefficients.keys() {
            if !variables.contains_key(variable) {
                return Err(error(format!("constraint references undeclared variable: {variable}")));
            }
        }
    }
    for variable in parsed.objective.keys() {
        if !variables.contains_key(variable) {
            return Err(error(format!("objective references undeclared variable: {variable}")));
        }
    }

    Ok(SemanticModel {
        schema_version: MODEL_SCHEMA_VERSION.to_owned(),
        objective_id,
        direction: parsed.direction,
        variables,
        objective: parsed.objective,
        objective_constant: parsed.objective_constant,
        constraints: parsed.constraints,
    })
}

fn identify_artifact(file_name: &str) -> Result<(ObjectiveId, Direction), ValidationError> {
    let stem = std::path::Path::new(file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| error("LP artifact file name is not UTF-8"))?;
    let (objective, direction) = stem
        .rsplit_once('_')
        .ok_or_else(|| error("LP artifact name must be <objective>_<min|max>.lp"))?;
    let objective = ObjectiveId::parse(objective)
        .ok_or_else(|| error(format!("unknown objective identifier: {objective}")))?;
    let direction =
        Direction::parse(direction).ok_or_else(|| error("LP direction suffix must be min or max"))?;
    Ok((objective, direction))
}

pub fn validate_lp(
    request_bytes: &[u8],
    lp_bytes: &[u8],
    file_name: &str,
) -> Result<ValidationReceipt, ValidationError> {
    let (objective_id, direction) = identify_artifact(file_name)?;
    let request_sha = sha256(request_bytes);
    let lp_sha = sha256(lp_bytes);
    let expected = derive_reference_model(request_bytes, objective_id, direction)?;
    let parsed = parse_lp(lp_bytes)?;

    if parsed.direction != direction {
        return Err(error("LP objective direction disagrees with artifact identity"));
    }
    if let Some(source_sha) = &parsed.metadata.source_sha256 {
        if source_sha != &request_sha {
            return Err(error("LP source_sha256 comment does not match request bytes"));
        }
    } else {
        return Err(error("LP source_sha256 metadata is missing"));
    }
    if let Some(schema) = &parsed.metadata.model_schema {
        if schema != MODEL_SCHEMA_VERSION {
            return Err(error(format!("unsupported LP model schema: {schema}")));
        }
    } else {
        return Err(error("LP model_schema metadata is missing"));
    }
    if parsed.metadata.objective.as_deref() != Some(objective_id.as_str()) {
        return Err(error("LP objective metadata disagrees with artifact identity"));
    }
    let expected_direction = match direction {
        Direction::Min => "min",
        Direction::Max => "max",
    };
    if parsed.metadata.direction.as_deref() != Some(expected_direction) {
        return Err(error("LP direction metadata disagrees with artifact identity"));
    }

    let actual = build_parsed_model(parsed, objective_id)?;
    if actual != expected {
        return Err(model_diff(&expected, &actual));
    }

    let reference_model_sha = sha256(
        &serde_json::to_vec(&expected).map_err(|serialize_error| error(format!("serialize reference model: {serialize_error}")))?,
    );
    let parsed_model_sha = sha256(
        &serde_json::to_vec(&actual).map_err(|serialize_error| error(format!("serialize parsed model: {serialize_error}")))?,
    );

    Ok(ValidationReceipt {
        receipt_schema: "energy-dispatch-translation-validation-receipt-v0.1".to_owned(),
        validator_id: VALIDATOR_ID.to_owned(),
        validator_version: VALIDATOR_VERSION.to_owned(),
        request_sha256: request_sha,
        lp_sha256: lp_sha,
        model_schema_version: MODEL_SCHEMA_VERSION.to_owned(),
        objective_id,
        direction,
        reference_model_sha256: reference_model_sha,
        parsed_model_sha256: parsed_model_sha,
        equivalent: true,
        claim_ceiling: concat!(
            "the parsed LP is semantically identical to the independently derived integer model ",
            "for the exact request and objective; does not prove solver optimality, physical validity, ",
            "evidence authenticity, operational safety, or authorization"
        ).to_owned(),
    })
}

fn model_diff(expected: &SemanticModel, actual: &SemanticModel) -> ValidationError {
    if expected.variables != actual.variables {
        return error("semantic model mismatch: variable inventory, domain, or bound differs");
    }
    if expected.objective != actual.objective
        || expected.objective_constant != actual.objective_constant
        || expected.direction != actual.direction
        || expected.objective_id != actual.objective_id
    {
        return error("semantic model mismatch: objective identity, direction, coefficient, or constant differs");
    }
    if expected.constraints.keys().collect::<Vec<_>>() != actual.constraints.keys().collect::<Vec<_>>() {
        return error("semantic model mismatch: constraint inventory differs");
    }
    for (name, expected_row) in &expected.constraints {
        let actual_row = actual
            .constraints
            .get(name)
            .ok_or_else(|| error(format!("semantic model mismatch: missing constraint {name}")))?;
        if expected_row != actual_row {
            return error(format!(
                "semantic model mismatch: constraint {name} has a different coefficient, relation, or RHS"
            ));
        }
    }
    error("semantic model mismatch: model identities differ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const REQUEST: &[u8] = br#"{
      "schema_version":"energy-dispatch-v0.1",
      "case_id":"small-fixture",
      "synthetic":true,
      "evidence_class":"analyst_assumption",
      "evidence_ref":null,
      "evidence_as_of":null,
      "energy_quantum_kwh":1.0,
      "max_candidate_expansions":100000,
      "battery":{
        "capacity_quanta":2,
        "initial_soc_quanta":0,
        "charge_input_limit_quanta":2,
        "discharge_withdrawal_limit_quanta":2,
        "charge_efficiency_numerator":1,
        "charge_efficiency_denominator":1,
        "discharge_efficiency_numerator":1,
        "discharge_efficiency_denominator":1,
        "terminal_reserve_quanta":0
      },
      "intervals":[{
        "interval_id":"only",
        "start_unix_seconds":1800000000,
        "duration_seconds":3600,
        "grid_outage":false,
        "pv_available_quanta":2,
        "grid_import_limit_quanta":2,
        "essential_demand_quanta":1,
        "flexible_demand_quanta":0,
        "grid_cost_units_per_quantum":3,
        "grid_carbon_grams_per_quantum":7
      }]
    }"#;

    fn sample_lp() -> String {
        // Golden parser fixture authored independently of the production emitter.
        // Values follow the one-interval request above.
        [
            "\\ Energy Dispatch independent MILP reference v0.1",
            "\\ source_sha256=PLACEHOLDER",
            "\\\\ model_schema=energy-dispatch-independent-mip-v0.1 objective=essential_unserved_quanta direction=min",
            "Minimize",
            " obj: 1 essential_unserved_000",
            "Subject To",
            " soc_transition_000: 1 soc_after_000 + 1 soc_withdrawal_000 - 1 charge_gain_000 = 0",
            " withdrawal_prev_soc_000: 1 soc_withdrawal_000 <= 0",
            " charge_mode_limit_000: 1 charge_input_000 - 2 charge_mode_000 <= 0",
            " discharge_mode_limit_000: 1 soc_withdrawal_000 + 2 charge_mode_000 <= 2",
            " charge_floor_lower_000: 1 charge_gain_000 - 1 charge_input_000 <= 0",
            " charge_floor_upper_000: 1 charge_input_000 - 1 charge_gain_000 <= 0",
            " discharge_floor_lower_000: 1 discharge_delivered_000 - 1 soc_withdrawal_000 <= 0",
            " discharge_floor_upper_000: 1 soc_withdrawal_000 - 1 discharge_delivered_000 <= 0",
            " essential_partition_000: 1 essential_served_000 + 1 essential_unserved_000 = 1",
            " flexible_partition_000: 1 flexible_served_000 + 1 flexible_unserved_000 = 0",
            " pv_partition_000: 1 pv_used_000 + 1 pv_curtailed_000 = 2",
            " bus_balance_000: 1 pv_used_000 + 1 grid_import_000 + 1 discharge_delivered_000 - 1 essential_served_000 - 1 flexible_served_000 - 1 charge_input_000 = 0",
            " terminal_reserve: 1 soc_after_000 >= 0",
            "Bounds",
            " 0 <= grid_import_000 <= 2",
            " 0 <= charge_input_000 <= 2",
            " 0 <= soc_withdrawal_000 <= 2",
            " 0 <= charge_gain_000 <= 2",
            " 0 <= discharge_delivered_000 <= 2",
            " 0 <= soc_after_000 <= 2",
            " 0 <= essential_served_000 <= 1",
            " 0 <= essential_unserved_000 <= 1",
            " 0 <= flexible_served_000 <= 0",
            " 0 <= flexible_unserved_000 <= 0",
            " 0 <= pv_used_000 <= 2",
            " 0 <= pv_curtailed_000 <= 2",
            "Binaries",
            " charge_mode_000",
            "Generals",
            " grid_import_000",
            " charge_input_000",
            " soc_withdrawal_000",
            " charge_gain_000",
            " discharge_delivered_000",
            " soc_after_000",
            " essential_served_000",
            " essential_unserved_000",
            " flexible_served_000",
            " flexible_unserved_000",
            " pv_used_000",
            " pv_curtailed_000",
            "End",
            "",
        ]
        .join("\n")
    }

    fn with_source_digest(mut lp: String) -> String {
        let digest = sha256(REQUEST);
        lp = lp.replace("PLACEHOLDER", &digest);
        lp
    }

    fn integer_at(assignment: &BTreeMap<String, i64>, name: &str) -> i64 {
        *assignment.get(name).unwrap_or_else(|| panic!("missing assignment for {name}"))
    }

    fn model_accepts_assignment(
        model: &SemanticModel,
        assignment: &BTreeMap<String, i64>,
    ) -> bool {
        for (name, variable) in &model.variables {
            let Some(value) = assignment.get(name) else {
                return false;
            };
            if *value < variable.lower || *value > variable.upper {
                return false;
            }
            if variable.domain == VariableDomain::Binary && !matches!(*value, 0 | 1) {
                return false;
            }
        }

        for row in model.constraints.values() {
            let mut lhs = 0i64;
            for (name, coefficient) in &row.coefficients {
                let Some(value) = assignment.get(name) else {
                    return false;
                };
                let Some(term) = coefficient.checked_mul(*value) else {
                    return false;
                };
                let Some(next) = lhs.checked_add(term) else {
                    return false;
                };
                lhs = next;
            }
            let satisfied = match row.relation {
                Relation::Eq => lhs == row.rhs,
                Relation::Le => lhs <= row.rhs,
                Relation::Ge => lhs >= row.rhs,
            };
            if !satisfied {
                return false;
            }
        }
        true
    }

    /// Separate schedule-level specification: compute floor-rounded battery
    /// transitions directly from physical request fields, then check service
    /// partitions, reserve, mode exclusivity and bus energy balance.
    fn schedule_spec_accepts_assignment(
        request: &Request,
        assignment: &BTreeMap<String, i64>,
    ) -> bool {
        if request.intervals.is_empty() {
            return false;
        }
        let battery = &request.battery;
        let mut previous_soc = i64::from(battery.initial_soc_quanta);

        for (index, interval) in request.intervals.iter().enumerate() {
            let value = |prefix: &str| {
                integer_at(assignment, &format!("{prefix}_{index:03}"))
            };

            let grid = value("grid_import");
            let charge_input = value("charge_input");
            let withdrawal = value("soc_withdrawal");
            let charge_gain = value("charge_gain");
            let delivered = value("discharge_delivered");
            let soc_after = value("soc_after");
            let essential_served = value("essential_served");
            let essential_unserved = value("essential_unserved");
            let flexible_served = value("flexible_served");
            let flexible_unserved = value("flexible_unserved");
            let pv_used = value("pv_used");
            let pv_curtailed = value("pv_curtailed");
            let charge_mode = value("charge_mode");

            let grid_limit = if interval.grid_outage {
                0
            } else {
                i64::from(interval.grid_import_limit_quanta)
            };
            let withdrawal_limit = i64::from(
                battery
                    .discharge_withdrawal_limit_quanta
                    .min(battery.capacity_quanta),
            );
            if grid < 0
                || grid > grid_limit
                || charge_input < 0
                || charge_input > i64::from(battery.charge_input_limit_quanta)
                || withdrawal < 0
                || withdrawal > withdrawal_limit
                || withdrawal > previous_soc
                || charge_mode != 0 && charge_mode != 1
            {
                return false;
            }
            if (charge_mode == 0 && charge_input != 0)
                || (charge_mode == 1 && withdrawal != 0)
            {
                return false;
            }

            let expected_gain = charge_input
                .checked_mul(i64::from(battery.charge_efficiency_numerator))
                .map(|product| product / i64::from(battery.charge_efficiency_denominator));
            let expected_delivery = withdrawal
                .checked_mul(i64::from(battery.discharge_efficiency_numerator))
                .map(|product| product / i64::from(battery.discharge_efficiency_denominator));
            if expected_gain != Some(charge_gain) || expected_delivery != Some(delivered) {
                return false;
            }

            let expected_soc = previous_soc
                .checked_add(charge_gain)
                .and_then(|stored| stored.checked_sub(withdrawal));
            if expected_soc != Some(soc_after)
                || soc_after < 0
                || soc_after > i64::from(battery.capacity_quanta)
            {
                return false;
            }

            let essential_total = essential_served.checked_add(essential_unserved);
            let flexible_total = flexible_served.checked_add(flexible_unserved);
            let pv_total = pv_used.checked_add(pv_curtailed);
            if essential_served < 0
                || essential_unserved < 0
                || essential_total != Some(i64::from(interval.essential_demand_quanta))
                || flexible_served < 0
                || flexible_unserved < 0
                || flexible_total != Some(i64::from(interval.flexible_demand_quanta))
                || pv_used < 0
                || pv_curtailed < 0
                || pv_total != Some(i64::from(interval.pv_available_quanta))
            {
                return false;
            }

            let supply = pv_used
                .checked_add(grid)
                .and_then(|value| value.checked_add(delivered));
            let demand = essential_served
                .checked_add(flexible_served)
                .and_then(|value| value.checked_add(charge_input));
            if supply.is_none() || supply != demand {
                return false;
            }

            previous_soc = soc_after;
        }

        previous_soc >= i64::from(battery.terminal_reserve_quanta)
    }

    fn model_objective_value(
        model: &SemanticModel,
        assignment: &BTreeMap<String, i64>,
    ) -> Option<i64> {
        model.objective.iter().try_fold(
            model.objective_constant,
            |total, (name, coefficient)| {
                let value = integer_at(assignment, name);
                coefficient
                    .checked_mul(value)
                    .and_then(|term| total.checked_add(term))
            },
        )
    }

    fn schedule_objective_value(
        request: &Request,
        assignment: &BTreeMap<String, i64>,
    ) -> Option<i64> {
        request
            .intervals
            .iter()
            .enumerate()
            .try_fold(0_i64, |total, (index, _)| {
                total.checked_add(integer_at(
                    assignment,
                    &format!("essential_unserved_{index:03}"),
                ))
            })
    }

    fn sampled_compare_model_to_schedule_spec(
        request_bytes: &[u8],
        samples: usize,
        initial_seed: u64,
    ) -> (usize, usize) {
        let request: Request = serde_json::from_slice(request_bytes).unwrap();
        let model = derive_reference_model(
            request_bytes,
            ObjectiveId::EssentialUnservedQuanta,
            Direction::Min,
        )
        .unwrap();
        let variables = model
            .variables
            .iter()
            .map(|(name, variable)| (name.clone(), variable.lower, variable.upper))
            .collect::<Vec<_>>();
        let mut state = initial_seed;
        let mut assignment = BTreeMap::new();
        let mut feasible = 0usize;

        for _ in 0..samples {
            assignment.clear();
            for (name, lower, upper) in &variables {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let width = u64::try_from(upper - lower + 1).unwrap();
                let value = lower + i64::try_from(state % width).unwrap();
                assignment.insert(name.clone(), value);
            }
            let model_accepts = model_accepts_assignment(&model, &assignment);
            let schedule_accepts = schedule_spec_accepts_assignment(&request, &assignment);
            assert_eq!(
                model_accepts,
                schedule_accepts,
                "sampled model/schedule disagreement for seed {initial_seed}: {assignment:?}"
            );
            assert_eq!(
                model_objective_value(&model, &assignment),
                schedule_objective_value(&request, &assignment),
                "sampled objective mismatch for seed {initial_seed}: {assignment:?}"
            );
            if model_accepts {
                feasible += 1;
            }
        }
        (samples, feasible)
    }

    fn exhaustively_compare_model_to_schedule_spec(request_bytes: &[u8]) -> (usize, usize) {
        let request: Request = serde_json::from_slice(request_bytes).unwrap();
        let model = derive_reference_model(
            request_bytes,
            ObjectiveId::EssentialUnservedQuanta,
            Direction::Min,
        ).unwrap();
        let variables = model.variables.iter()
            .map(|(name, variable)| (name.clone(), variable.lower, variable.upper))
            .collect::<Vec<_>>();
        let mut assignment = BTreeMap::new();
        let mut visited = 0usize;
        let mut feasible = 0usize;

        fn visit(
            index: usize,
            variables: &[(String, i64, i64)],
            model: &SemanticModel,
            request: &Request,
            assignment: &mut BTreeMap<String, i64>,
            visited: &mut usize,
            feasible: &mut usize,
        ) {
            if index == variables.len() {
                *visited += 1;
                let model_accepts = model_accepts_assignment(model, assignment);
                let schedule_accepts = schedule_spec_accepts_assignment(request, assignment);
                assert_eq!(
                    model_accepts,
                    schedule_accepts,
                    "model/schedule disagreement for assignment {assignment:?}"
                );
                assert_eq!(
                    model_objective_value(model, assignment),
                    schedule_objective_value(request, assignment),
                    "LP objective/source-request objective mismatch for assignment {assignment:?}"
                );
                if model_accepts {
                    *feasible += 1;
                }
                return;
            }

            let (name, lower, upper) = &variables[index];
            for value in *lower..=*upper {
                assignment.insert(name.clone(), value);
                visit(
                    index + 1,
                    variables,
                    model,
                    request,
                    assignment,
                    visited,
                    feasible,
                );
            }
            assignment.remove(name);
        }

        visit(
            0,
            &variables,
            &model,
            &request,
            &mut assignment,
            &mut visited,
            &mut feasible,
        );
        (visited, feasible)
    }

    fn two_interval_charge_then_outage_request() -> Vec<u8> {
        let mut request: serde_json::Value = serde_json::from_slice(REQUEST).unwrap();
        request["battery"]["capacity_quanta"] = serde_json::json!(1);
        request["battery"]["charge_input_limit_quanta"] = serde_json::json!(1);
        request["battery"]["discharge_withdrawal_limit_quanta"] = serde_json::json!(1);
        request["intervals"][0]["interval_id"] = serde_json::json!("charge-from-pv");
        request["intervals"][0]["grid_import_limit_quanta"] = serde_json::json!(0);
        request["intervals"][0]["pv_available_quanta"] = serde_json::json!(1);
        request["intervals"][0]["essential_demand_quanta"] = serde_json::json!(0);
        request["intervals"][0]["flexible_demand_quanta"] = serde_json::json!(0);
        request["intervals"][0]["grid_outage"] = serde_json::json!(false);
        request["intervals"].as_array_mut().unwrap().push(serde_json::json!({
            "interval_id": "serve-during-outage",
            "start_unix_seconds": 1_800_003_600_i64,
            "duration_seconds": 3600,
            "grid_outage": true,
            "pv_available_quanta": 0,
            "grid_import_limit_quanta": 0,
            "essential_demand_quanta": 1,
            "flexible_demand_quanta": 0,
            "grid_cost_units_per_quantum": 3,
            "grid_carbon_grams_per_quantum": 7
        }));
        serde_json::to_vec(&request).unwrap()
    }

    #[test]
    fn exhaustively_matches_two_interval_soc_carry_and_outage_schedule_semantics() {
        let request = two_interval_charge_then_outage_request();
        let (visited, feasible) = exhaustively_compare_model_to_schedule_spec(&request);
        assert_eq!(visited, 65_536);
        assert!(feasible > 0);
    }

    fn three_interval_charge_outage_recovery_request() -> Vec<u8> {
        let mut request: serde_json::Value =
            serde_json::from_slice(&two_interval_charge_then_outage_request()).unwrap();
        request["intervals"].as_array_mut().unwrap().push(serde_json::json!({
            "interval_id": "post-outage-idle",
            "start_unix_seconds": 1_800_007_200_i64,
            "duration_seconds": 3600,
            "grid_outage": false,
            "pv_available_quanta": 0,
            "grid_import_limit_quanta": 0,
            "essential_demand_quanta": 0,
            "flexible_demand_quanta": 0,
            "grid_cost_units_per_quantum": 3,
            "grid_carbon_grams_per_quantum": 7
        }));
        serde_json::to_vec(&request).unwrap()
    }

    #[test]
    fn three_interval_model_matches_independent_schedule_and_objective_samples() {
        let request = three_interval_charge_outage_recovery_request();
        let (visited, feasible) =
            sampled_compare_model_to_schedule_spec(&request, 50_000, 0x534f_435f_4833);
        assert_eq!(visited, 50_000);
        assert!(feasible <= visited);
    }

    #[test]
    fn independent_schedule_oracle_rejects_broken_soc_carry_forward() {
        let request_bytes = two_interval_charge_then_outage_request();
        let request: Request = serde_json::from_slice(&request_bytes).unwrap();
        let mut assignment = BTreeMap::from([
            ("grid_import_000".to_owned(), 0),
            ("charge_input_000".to_owned(), 1),
            ("soc_withdrawal_000".to_owned(), 0),
            ("charge_gain_000".to_owned(), 1),
            ("discharge_delivered_000".to_owned(), 0),
            ("soc_after_000".to_owned(), 1),
            ("essential_served_000".to_owned(), 0),
            ("essential_unserved_000".to_owned(), 0),
            ("flexible_served_000".to_owned(), 0),
            ("flexible_unserved_000".to_owned(), 0),
            ("pv_used_000".to_owned(), 1),
            ("pv_curtailed_000".to_owned(), 0),
            ("charge_mode_000".to_owned(), 1),
            ("grid_import_001".to_owned(), 0),
            ("charge_input_001".to_owned(), 0),
            ("soc_withdrawal_001".to_owned(), 1),
            ("charge_gain_001".to_owned(), 0),
            ("discharge_delivered_001".to_owned(), 1),
            ("soc_after_001".to_owned(), 0),
            ("essential_served_001".to_owned(), 1),
            ("essential_unserved_001".to_owned(), 0),
            ("flexible_served_001".to_owned(), 0),
            ("flexible_unserved_001".to_owned(), 0),
            ("pv_used_001".to_owned(), 0),
            ("pv_curtailed_001".to_owned(), 0),
            ("charge_mode_001".to_owned(), 0),
        ]);

        assert!(schedule_spec_accepts_assignment(&request, &assignment));
        assignment.insert("soc_after_000".to_owned(), 0);
        assert!(!schedule_spec_accepts_assignment(&request, &assignment));
    }

    #[test]
    fn exhaustively_matches_the_schedule_spec_for_unity_efficiency_fixture() {
        let (visited, feasible) = exhaustively_compare_model_to_schedule_spec(REQUEST);
        assert!(visited >= 10_000);
        assert!(feasible > 0);
    }

    #[test]
    fn exhaustively_matches_non_integral_integer_floor_efficiency() {
        let request = String::from_utf8(REQUEST.to_vec())
            .unwrap()
            .replace(
                "\"charge_efficiency_numerator\":1",
                "\"charge_efficiency_numerator\":2",
            )
            .replace(
                "\"charge_efficiency_denominator\":1",
                "\"charge_efficiency_denominator\":3",
            )
            .replace(
                "\"discharge_efficiency_numerator\":1",
                "\"discharge_efficiency_numerator\":2",
            )
            .replace(
                "\"discharge_efficiency_denominator\":1",
                "\"discharge_efficiency_denominator\":3",
            );
        let (visited, feasible) = exhaustively_compare_model_to_schedule_spec(request.as_bytes());
        assert!(visited >= 10_000);
        assert!(feasible > 0);
    }

    #[test]
    fn derives_expected_variable_and_constraint_counts() {
        let model = derive_reference_model(
            REQUEST,
            ObjectiveId::EssentialUnservedQuanta,
            Direction::Min,
        )
        .expect("valid reference request");
        assert_eq!(model.variables.len(), 13);
        assert_eq!(model.constraints.len(), 13);
        assert_eq!(model.objective.get("essential_unserved_000"), Some(&1));
    }

    #[test]
    fn validates_independently_authored_known_answer_lp() {
        let lp = with_source_digest(sample_lp());
        let receipt = validate_lp(
            REQUEST,
            lp.as_bytes(),
            "essential_unserved_quanta_min.lp",
        )
        .expect("LP must match independently derived model");
        assert!(receipt.equivalent);
        assert_eq!(receipt.request_sha256, sha256(REQUEST));
        assert_eq!(receipt.lp_sha256, sha256(lp.as_bytes()));
        assert_eq!(receipt.reference_model_sha256, receipt.parsed_model_sha256);
    }

    #[test]
    fn rejects_changed_constraint_coefficient() {
        let lp = with_source_digest(sample_lp()).replace(
            "1 discharge_delivered_000 - 1 soc_withdrawal_000",
            "2 discharge_delivered_000 - 1 soc_withdrawal_000",
        );
        assert!(validate_lp(
            REQUEST,
            lp.as_bytes(),
            "essential_unserved_quanta_min.lp"
        )
        .is_err());
    }

    #[test]
    fn rejects_changed_bound() {
        let lp = with_source_digest(sample_lp()).replace(
            "0 <= grid_import_000 <= 2",
            "0 <= grid_import_000 <= 3",
        );
        assert!(validate_lp(
            REQUEST,
            lp.as_bytes(),
            "essential_unserved_quanta_min.lp"
        )
        .is_err());
    }

    #[test]
    fn rejects_changed_integer_domain() {
        let lp = with_source_digest(sample_lp()).replace(
            " grid_import_000\n",
            "",
        );
        assert!(validate_lp(
            REQUEST,
            lp.as_bytes(),
            "essential_unserved_quanta_min.lp"
        )
        .is_err());
    }

    #[test]
    fn rejects_missing_constraint_and_unknown_syntax() {
        let lp = with_source_digest(sample_lp()).replace(
            " terminal_reserve: 1 soc_after_000 >= 0\n",
            "",
        );
        assert!(validate_lp(
            REQUEST,
            lp.as_bytes(),
            "essential_unserved_quanta_min.lp"
        )
        .is_err());

        let lp = with_source_digest(sample_lp()).replace("Bounds\n", "Semi-Continuous\n");
        assert!(validate_lp(
            REQUEST,
            lp.as_bytes(),
            "essential_unserved_quanta_min.lp"
        )
        .is_err());
    }

    #[test]
    fn accepts_emitter_zero_coefficients_but_discards_them_semantically() {
        let parsed = parse_expression(
            &["1", "x", "+", "0", "y", "-", "2", "z"],
            true,
        ).expect("zero terms are valid in the emitted LP profile");
        assert_eq!(parsed.0, BTreeMap::from([
            ("x".to_owned(), 1),
            ("z".to_owned(), -2),
        ]));
        assert!(parse_expression(&["1", "x", "+", "0", "y"], false).is_err());
        assert!(parse_expression(&["1", "x", "+", "0", "x"], true).is_err());
    }

    #[test]
    fn rejects_trailing_content_and_duplicate_sections() {
        let mut lp = with_source_digest(sample_lp());
        lp.push_str("unexpected\n");
        assert!(validate_lp(
            REQUEST,
            lp.as_bytes(),
            "essential_unserved_quanta_min.lp"
        )
        .is_err());

        let lp = with_source_digest(sample_lp()).replace("Bounds\n", "Bounds\nBounds\n");
        assert!(validate_lp(
            REQUEST,
            lp.as_bytes(),
            "essential_unserved_quanta_min.lp"
        )
        .is_err());
    }

    #[test]
    fn rejects_source_and_metadata_substitution() {
        let lp = with_source_digest(sample_lp()).replace(
            &sha256(REQUEST),
            &"0".repeat(64),
        );
        assert!(validate_lp(
            REQUEST,
            lp.as_bytes(),
            "essential_unserved_quanta_min.lp"
        )
        .is_err());

        let lp = with_source_digest(sample_lp()).replace(
            "direction=min",
            "direction=max",
        );
        assert!(validate_lp(
            REQUEST,
            lp.as_bytes(),
            "essential_unserved_quanta_min.lp"
        )
        .is_err());
    }

    #[test]
    fn rejects_unknown_artifact_identity() {
        assert!(identify_artifact("made_up_min.lp").is_err());
        assert!(identify_artifact("essential_unserved_quanta_unknown.lp").is_err());
    }

    #[test]
    fn reference_model_accepts_zero_charge_and_discharge_limits() {
        let request = String::from_utf8(REQUEST.to_vec())
            .unwrap()
            .replace("\"charge_input_limit_quanta\":2", "\"charge_input_limit_quanta\":0")
            .replace("\"discharge_withdrawal_limit_quanta\":2", "\"discharge_withdrawal_limit_quanta\":0");
        let model = derive_reference_model(
            request.as_bytes(),
            ObjectiveId::EssentialUnservedQuanta,
            Direction::Min,
        )
        .expect("zero charge/discharge limits are valid integer constraints");
        assert_eq!(model.constraints["charge_mode_limit_000"].coefficients.get("charge_mode_000"), None);
        assert_eq!(model.constraints["discharge_mode_limit_000"].coefficients.get("charge_mode_000"), None);
    }

    #[test]
    fn reference_model_accepts_an_identically_zero_cost_objective() {
        let request = String::from_utf8(REQUEST.to_vec())
            .unwrap()
            .replace("\"grid_cost_units_per_quantum\":3", "\"grid_cost_units_per_quantum\":0");
        let model = derive_reference_model(
            request.as_bytes(),
            ObjectiveId::OperatingCostUnits,
            Direction::Min,
        )
        .expect("zero-coefficient objective is a valid constant-zero objective");
        assert!(model.objective.is_empty());
        assert_eq!(model.objective_constant, 0);
    }

    #[test]
    fn rejects_one_thousand_deterministic_semantic_mutations() {
        let baseline = with_source_digest(sample_lp());
        let cases = [
            (
                "soc_transition_000: 1 soc_after_000 + 1 soc_withdrawal_000 - 1 charge_gain_000 = 0",
                "soc_transition_000: {value} soc_after_000 + 1 soc_withdrawal_000 - 1 charge_gain_000 = 0",
            ),
            (
                "0 <= grid_import_000 <= 2",
                "0 <= grid_import_000 <= {value}",
            ),
            (
                "1 discharge_delivered_000 - 1 soc_withdrawal_000",
                "{value} discharge_delivered_000 - 1 soc_withdrawal_000",
            ),
            (
                "1 essential_unserved_000",
                "{value} essential_unserved_000",
            ),
        ];

        let mut rejected = 0usize;
        for seed in 0..1_000usize {
            let value = (seed + 2).to_string();
            let (needle, replacement_template) = cases[seed % cases.len()];
            let replacement = replacement_template.replace("{value}", &value);
            let mutated = baseline.replacen(needle, &replacement, 1);
            assert_ne!(mutated, baseline, "mutation seed {seed} did not change the LP");
            assert!(
                validate_lp(
                    REQUEST,
                    mutated.as_bytes(),
                    "essential_unserved_quanta_min.lp"
                )
                .is_err(),
                "semantic mutation seed {seed} was accepted"
            );
            rejected += 1;
        }
        assert_eq!(rejected, 1_000);
    }

    #[test]
    fn parser_rejects_missing_term_separators_and_repeated_signs() {
        assert!(parse_expression(&["1", "x", "2", "y"], false).is_err());
        assert!(parse_expression(&["1", "x", "+", "-", "2", "y"], false).is_err());
        assert!(parse_expression(&["1", "x", "+", "0", "y"], false).is_err());
    }


    #[test]
    fn rejects_invalid_and_oversized_inputs() {
        assert!(derive_reference_model(
            b"{",
            ObjectiveId::EssentialUnservedQuanta,
            Direction::Min
        )
        .is_err());
        assert!(parse_lp(&vec![b'x'; MAX_LP_BYTES + 1]).is_err());
    }
}
