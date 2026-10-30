//! Independent LP model emitter for the bounded energy-dispatch request.
//!
//! This crate deliberately duplicates the request DTOs and the mathematical
//! constraints. It does not import the production oracle, its schedule verifier,
//! objective evaluator, or frontier code. Emitted LP files are solver inputs;
//! model generation alone is not a solver result or proof.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt::Write as _;

pub const MODEL_SCHEMA_VERSION: &str = "energy-dispatch-independent-mip-v0.1";
pub const MAX_INTERVALS: usize = 128;
pub const MAX_REQUEST_BYTES: usize = 1_048_576;

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Minimize,
    Maximize,
}

impl Direction {
    pub const ALL: [Self; 2] = [Self::Minimize, Self::Maximize];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Minimize => "min",
            Self::Maximize => "max",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelArtifact {
    pub objective: ObjectiveId,
    pub direction: Direction,
    pub file_name: String,
    pub sha256: String,
    pub byte_length: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelManifest {
    pub model_schema_version: String,
    pub case_id: String,
    pub source_sha256: String,
    pub model_count: usize,
    pub models: Vec<ModelArtifact>,
    pub model_claim_ceiling: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelError(pub String);

impl std::fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ModelError {}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn validate_request(request: &Request) -> Result<(), ModelError> {
    if request.schema_version != "energy-dispatch-v0.1" {
        return Err(ModelError("unsupported request schema version".to_owned()));
    }
    if request.case_id.trim().is_empty() {
        return Err(ModelError("case_id must be non-empty".to_owned()));
    }
    if !request.energy_quantum_kwh.is_finite() || request.energy_quantum_kwh <= 0.0 {
        return Err(ModelError("energy_quantum_kwh must be finite and positive".to_owned()));
    }
    if request.max_candidate_expansions == 0 || request.max_candidate_expansions > 1_000_000 {
        return Err(ModelError("max_candidate_expansions must be within [1, 1000000]".to_owned()));
    }
    if request.intervals.is_empty() || request.intervals.len() > MAX_INTERVALS {
        return Err(ModelError(format!(
            "interval count must be within [1, {MAX_INTERVALS}]"
        )));
    }
    if request.evidence_class != "analyst_assumption"
        && request.evidence_ref.as_deref().is_none_or(|value| value.trim().is_empty())
    {
        return Err(ModelError("non-assumption evidence requires evidence_ref".to_owned()));
    }
    let battery = &request.battery;
    if battery.initial_soc_quanta > battery.capacity_quanta
        || battery.terminal_reserve_quanta > battery.capacity_quanta
    {
        return Err(ModelError("initial SOC and terminal reserve must not exceed capacity".to_owned()));
    }
    for (name, numerator, denominator) in [
        ("charge", battery.charge_efficiency_numerator, battery.charge_efficiency_denominator),
        ("discharge", battery.discharge_efficiency_numerator, battery.discharge_efficiency_denominator),
    ] {
        if numerator == 0 || denominator == 0 || numerator > denominator {
            return Err(ModelError(format!("{name} efficiency ratio is invalid")));
        }
    }
    let mut ids = BTreeSet::new();
    let mut expected_start = None::<i64>;
    for interval in &request.intervals {
        if interval.interval_id.trim().is_empty() || !ids.insert(interval.interval_id.as_str()) {
            return Err(ModelError("interval IDs must be non-empty and unique".to_owned()));
        }
        if interval.duration_seconds == 0 {
            return Err(ModelError(format!("interval {} has zero duration", interval.interval_id)));
        }
        if let Some(expected) = expected_start {
            if expected != interval.start_unix_seconds {
                return Err(ModelError(format!("interval {} breaks chronology", interval.interval_id)));
            }
        }
        expected_start = Some(
            interval.start_unix_seconds.checked_add(
                i64::try_from(interval.duration_seconds)
                    .map_err(|_| ModelError("interval duration does not fit signed chronology".to_owned()))?
            ).ok_or_else(|| ModelError("chronology arithmetic overflow".to_owned()))?
        );
    }
    Ok(())
}

fn variable(name: &str, index: usize) -> String {
    format!("{name}_{index:03}")
}

fn write_line(buffer: &mut String, line: &str) {
    buffer.push_str(line);
    buffer.push('\n');
}

fn objective_terms(request: &Request, objective: ObjectiveId) -> Vec<(u64, String)> {
    let mut terms = Vec::with_capacity(request.intervals.len());
    for (index, input) in request.intervals.iter().enumerate() {
        let (coefficient, name) = match objective {
            ObjectiveId::EssentialUnservedQuanta => (
                1,
                variable("essential_unserved", index),
            ),
            ObjectiveId::FlexibleUnservedQuanta => (
                1,
                variable("flexible_unserved", index),
            ),
            ObjectiveId::OperatingCostUnits => (
                u64::from(input.grid_cost_units_per_quantum),
                variable("grid_import", index),
            ),
            ObjectiveId::GridCarbonGrams => (
                u64::from(input.grid_carbon_grams_per_quantum),
                variable("grid_import", index),
            ),
            ObjectiveId::CurtailedPvQuanta => (
                1,
                variable("pv_curtailed", index),
            ),
            ObjectiveId::TerminalSocQuanta => {
                if index + 1 == request.intervals.len() {
                    (1, variable("soc_after", index))
                } else {
                    continue;
                }
            }
        };
        terms.push((coefficient, name));
    }
    terms
}

fn append_constraint(
    output: &mut String,
    name: &str,
    terms: &[(i64, String)],
    relation: &str,
    rhs: i64,
) {
    let _ = write!(output, " {name}: ");
    for (index, (coefficient, variable)) in terms.iter().enumerate() {
        if index > 0 {
            if *coefficient >= 0 {
                output.push_str(" + ");
            } else {
                output.push_str(" - ");
            }
        } else if *coefficient < 0 {
            output.push('-');
        }
        let _ = write!(output, "{} {}", coefficient.unsigned_abs(), variable);
    }
    let _ = writeln!(output, " {relation} {rhs}");
}

fn append_bound(output: &mut String, name: &str, upper: u64) {
    let _ = writeln!(output, " 0 <= {name} <= {upper}");
}

/// Emit a standalone integer linear model for one objective and direction.
/// The rational-efficiency floor is exact because all decision variables are
/// integer and the quotient is bounded by a pair of integer inequalities.
pub fn emit_lp(
    request: &Request,
    objective: ObjectiveId,
    direction: Direction,
    source_sha256: &str,
) -> Result<String, ModelError> {
    validate_request(request)?;
    if source_sha256.len() != 64
        || !source_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(ModelError("source_sha256 must be a 64-character hexadecimal digest".to_owned()));
    }
    let mut output = String::new();
    let _ = writeln!(output, "\\ Energy dispatch independent MILP reference v0.1");
    let _ = writeln!(output, "\\\\ case_id={} source_sha256={}", request.case_id, source_sha256);
    let _ = writeln!(output, "\\\\ model_schema={} objective={} direction={}", MODEL_SCHEMA_VERSION, objective.as_str(), direction.as_str());
    write_line(&mut output, if direction == Direction::Minimize { "Minimize" } else { "Maximize" });
    let terms = objective_terms(request, objective);
    if terms.is_empty() {
        return Err(ModelError("objective expression unexpectedly has no terms".to_owned()));
    }
    output.push_str(" obj: ");
    for (index, (coefficient, name)) in terms.iter().enumerate() {
        if index > 0 { output.push_str(" + "); }
        let _ = write!(output, "{coefficient} {name}");
    }
    output.push('\n');
    write_line(&mut output, "Subject To");

    for (index, input) in request.intervals.iter().enumerate() {
        let g=variable("grid_import",index);
        let c=variable("charge_input",index);
        let w=variable("soc_withdrawal",index);
        let q=variable("charge_gain",index);
        let d=variable("discharge_delivered",index);
        let soc=variable("soc_after",index);
        let e=variable("essential_served",index);
        let eu=variable("essential_unserved",index);
        let f=variable("flexible_served",index);
        let fu=variable("flexible_unserved",index);
        let pv=variable("pv_used",index);
        let curtailed=variable("pv_curtailed",index);
        let mode=variable("charge_mode",index);

        append_constraint(
            &mut output,
            &format!("soc_transition_{index:03}"),
            &[
                (1,soc.clone()),
                (1,w.clone()),
                (-1,q.clone()),
            ].into_iter().chain(
                if index == 0 { Vec::new() } else { vec![(-1,variable("soc_after",index-1))] }
            ).collect::<Vec<_>>().as_slice(),
            "=",
            if index == 0 { i64::from(request.battery.initial_soc_quanta) } else { 0 },
        );

        let discharge_ceiling = request.battery.discharge_withdrawal_limit_quanta.min(request.battery.capacity_quanta);
        append_constraint(
            &mut output,
            &format!("withdrawal_prev_soc_{index:03}"),
            &[(1,w.clone())].into_iter().chain(
                if index == 0 { Vec::new() } else { vec![(-1,variable("soc_after",index-1))] }
            ).collect::<Vec<_>>().as_slice(),
            "<=",
            if index == 0 { i64::from(request.battery.initial_soc_quanta) } else { 0 },
        );
        append_constraint(
            &mut output,
            &format!("charge_mode_limit_{index:03}"),
            &[(1,c.clone()),(-i64::from(request.battery.charge_input_limit_quanta),mode.clone())],
            "<=",
            0,
        );
        append_constraint(
            &mut output,
            &format!("discharge_mode_limit_{index:03}"),
            &[(1,w.clone()),(i64::from(discharge_ceiling),mode.clone())],
            "<=",
            i64::from(discharge_ceiling),
        );

        let cn=i64::from(request.battery.charge_efficiency_numerator);
        let cd=i64::from(request.battery.charge_efficiency_denominator);
        append_constraint(
            &mut output,
            &format!("charge_floor_lower_{index:03}"),
            &[(cd,q.clone()),(-cn,c.clone())],
            "<=",
            0,
        );
        append_constraint(
            &mut output,
            &format!("charge_floor_upper_{index:03}"),
            &[(cn,c.clone()),(-cd,q.clone())],
            "<=",
            cd-1,
        );
        let dn=i64::from(request.battery.discharge_efficiency_numerator);
        let dd=i64::from(request.battery.discharge_efficiency_denominator);
        append_constraint(
            &mut output,
            &format!("discharge_floor_lower_{index:03}"),
            &[(dd,d.clone()),(-dn,w.clone())],
            "<=",
            0,
        );
        append_constraint(
            &mut output,
            &format!("discharge_floor_upper_{index:03}"),
            &[(dn,w.clone()),(-dd,d.clone())],
            "<=",
            dd-1,
        );

        append_constraint(
            &mut output,
            &format!("essential_partition_{index:03}"),
            &[(1,e.clone()),(1,eu.clone())],
            "=",
            i64::from(input.essential_demand_quanta),
        );
        append_constraint(
            &mut output,
            &format!("flexible_partition_{index:03}"),
            &[(1,f.clone()),(1,fu.clone())],
            "=",
            i64::from(input.flexible_demand_quanta),
        );
        append_constraint(
            &mut output,
            &format!("pv_partition_{index:03}"),
            &[(1,pv.clone()),(1,curtailed.clone())],
            "=",
            i64::from(input.pv_available_quanta),
        );
        append_constraint(
            &mut output,
            &format!("bus_balance_{index:03}"),
            &[(1,pv.clone()),(1,g.clone()),(1,d.clone()),(-1,e.clone()),(-1,f.clone()),(-1,c.clone())],
            "=",
            0,
        );

        let soc_max = u64::from(request.battery.capacity_quanta);
        let charge_max = u64::from(request.battery.charge_input_limit_quanta);
        let charge_gain_max = (u64::from(request.battery.charge_input_limit_quanta)
            * u64::from(request.battery.charge_efficiency_numerator)
            / u64::from(request.battery.charge_efficiency_denominator));
        let discharge_max = u64::from(discharge_ceiling);
        let delivered_max = discharge_max
            * u64::from(request.battery.discharge_efficiency_numerator)
            / u64::from(request.battery.discharge_efficiency_denominator);
        let grid_max = if input.grid_outage { 0 } else { u64::from(input.grid_import_limit_quanta) };

        for (name, max) in [
            (&g,grid_max),(&c,charge_max),(&w,discharge_max),(&q,charge_gain_max),
            (&d,delivered_max),(&soc,soc_max),
            (&e,u64::from(input.essential_demand_quanta)),
            (&eu,u64::from(input.essential_demand_quanta)),
            (&f,u64::from(input.flexible_demand_quanta)),
            (&fu,u64::from(input.flexible_demand_quanta)),
            (&pv,u64::from(input.pv_available_quanta)),
            (&curtailed,u64::from(input.pv_available_quanta)),
        ] {
            append_bound(&mut output,name,max);
        }
    }

    let last = request.intervals.len()-1;
    append_constraint(
        &mut output,
        "terminal_reserve",
        &[(1,variable("soc_after",last))],
        ">=",
        i64::from(request.battery.terminal_reserve_quanta),
    );

    write_line(&mut output, "Binaries");
    for index in 0..request.intervals.len() {
        let _=writeln!(output," {}",variable("charge_mode",index));
    }
    write_line(&mut output, "Generals");
    for index in 0..request.intervals.len() {
        for name in [
            "grid_import","charge_input","soc_withdrawal","charge_gain",
            "discharge_delivered","soc_after","essential_served","essential_unserved",
            "flexible_served","flexible_unserved","pv_used","pv_curtailed",
        ] {
            let _=writeln!(output," {}",variable(name,index));
        }
    }
    write_line(&mut output, "End");
    Ok(output)
}

pub fn build_model_set(source_bytes: &[u8]) -> Result<(ModelManifest, Vec<(String, String)>), ModelError> {
    if source_bytes.len() > MAX_REQUEST_BYTES {
        return Err(ModelError("request exceeds the 1 MiB byte cap".to_owned()));
    }
    let request: Request = serde_json::from_slice(source_bytes)
        .map_err(|error| ModelError(format!("strict request parse failed: {error}")))?;
    validate_request(&request)?;
    let source_sha256 = sha256(source_bytes);
    let mut models = Vec::with_capacity(ObjectiveId::ALL.len()*Direction::ALL.len());
    let mut files = Vec::with_capacity(ObjectiveId::ALL.len()*Direction::ALL.len());
    for objective in ObjectiveId::ALL {
        for direction in Direction::ALL {
            let file_name=format!("{}_{}.lp",objective.as_str(),direction.as_str());
            let lp=emit_lp(&request,objective,direction,&source_sha256)?;
            let artifact=ModelArtifact {
                objective,
                direction,
                file_name:file_name.clone(),
                sha256:sha256(lp.as_bytes()),
                byte_length:lp.len(),
            };
            models.push(artifact);
            files.push((file_name,lp));
        }
    }
    let manifest=ModelManifest {
        model_schema_version:MODEL_SCHEMA_VERSION.to_owned(),
        case_id:request.case_id,
        source_sha256,
        model_count:models.len(),
        models,
        model_claim_ceiling:concat!(
            "independent integer LP formulation of the declared finite request only; ",
            "generation is not solving, optimality certification, physical validation, evidence authentication or authorization"
        ).to_owned(),
    };
    Ok((manifest,files))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE:&[u8]=include_bytes!("../../../schemas/examples/energy-dispatch-pv-outage.synthetic.json");

    #[test]
    fn emits_twelve_models_for_six_objectives_in_two_directions() {
        let (manifest,files)=build_model_set(FIXTURE).expect("valid fixture");
        assert_eq!(manifest.model_count,12);
        assert_eq!(files.len(),12);
        assert!(files.iter().any(|(name,_)|name=="essential_unserved_quanta_min.lp"));
        assert!(files.iter().any(|(name,_)|name=="terminal_soc_quanta_max.lp"));
        assert!(files.iter().all(|(_,lp)|lp.contains("Binaries\n")&&lp.contains("Generals\n")&&lp.ends_with("End\n")));
    }

    #[test]
    fn preserves_floor_efficiency_with_exact_integer_inequalities() {
        let (manifest,files)=build_model_set(FIXTURE).expect("valid fixture");
        let lp=&files[0].1;
        assert!(lp.contains("charge_floor_lower_000"));
        assert!(lp.contains("charge_floor_upper_000"));
        assert!(lp.contains("discharge_floor_lower_001"));
        assert!(lp.contains("discharge_floor_upper_001"));
        assert!(lp.contains("charge_mode_limit_000"));
        assert!(lp.contains("discharge_mode_limit_000"));
    }

    #[test]
    fn outage_import_is_zero_and_terminal_reserve_is_hard() {
        let (_,files)=build_model_set(FIXTURE).expect("valid fixture");
        let lp=&files[0].1;
        assert!(lp.contains("0 <= grid_import_001 <= 0"));
        assert!(lp.contains("terminal_reserve: 1 soc_after_001 >= 0"));
    }

    #[test]
    fn non_unit_efficiency_uses_exact_floor_quotient_bounds() {
        let original=std::str::from_utf8(FIXTURE).unwrap();
        let changed=original
            .replace(r#""charge_efficiency_numerator": 1"#, r#""charge_efficiency_numerator": 3"#)
            .replace(r#""charge_efficiency_denominator": 1"#, r#""charge_efficiency_denominator": 4"#)
            .replace(r#""discharge_efficiency_numerator": 1"#, r#""discharge_efficiency_numerator": 2"#)
            .replace(r#""discharge_efficiency_denominator": 1"#, r#""discharge_efficiency_denominator": 3"#);
        let (_,files)=build_model_set(changed.as_bytes()).expect("valid rational efficiencies");
        let lp=&files[0].1;
        assert!(lp.contains("4 charge_gain_000 - 3 charge_input_000 <= 0"));
        assert!(lp.contains("3 charge_input_000 - 4 charge_gain_000 <= 3"));
        assert!(lp.contains("3 discharge_delivered_001 - 2 soc_withdrawal_001 <= 0"));
        assert!(lp.contains("2 soc_withdrawal_001 - 3 discharge_delivered_001 <= 2"));
    }

    #[test]
    fn deterministic_model_set_preserves_exact_request_identity() {
        let first=build_model_set(FIXTURE).expect("first build");
        let second=build_model_set(FIXTURE).expect("second build");
        assert_eq!(first.0,second.0);
        assert_eq!(first.1,second.1);
    }

    #[test]
    fn invalid_ratio_chronology_and_unknown_fields_fail_closed() {
        let original=std::str::from_utf8(FIXTURE).unwrap();
        let bad_ratio=original.replace(
            r#""charge_efficiency_denominator": 1"#,
            r#""charge_efficiency_denominator": 0"#
        );
        assert!(build_model_set(bad_ratio.as_bytes()).is_err());

        let trimmed=original.trim_end();
        let close=trimmed.rfind('}').expect("fixture JSON has closing object");
        let bad_unknown=format!("{}{}", &trimmed[..close], r#", "unknown":true}"#);
        assert!(build_model_set(bad_unknown.as_bytes()).is_err());
    }
}
