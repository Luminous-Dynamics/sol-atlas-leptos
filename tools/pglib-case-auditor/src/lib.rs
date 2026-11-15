// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Strict, offline structural auditor for the classic MATPOWER v2 case profile.
//!
//! This crate checks source structure and basic network-reference integrity. It does
//! not solve power flow or OPF, certify an electrical model, or qualify operation.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

const BUS_COLUMNS: usize = 13;
const GEN_COLUMNS: usize = 10;
const BRANCH_COLUMNS: usize = 13;

#[derive(Debug, Clone, PartialEq)]
pub struct AuditSummary {
    pub source_git_blob_sha1: String,
    pub function_name: String,
    pub matpower_version: String,
    pub base_mva: f64,
    pub bus_count: usize,
    pub generator_count: usize,
    pub branch_count: usize,
    pub cost_row_count: usize,
    pub in_service_generator_count: usize,
    pub in_service_branch_count: usize,
    pub energized_island_count: usize,
    pub reference_buses: Vec<i64>,
    pub claim_ceiling: &'static str,
}

impl AuditSummary {
    /// Stable field order and number formatting for a machine-readable receipt.
    pub fn to_json(&self) -> String {
        let refs = self
            .reference_buses
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            concat!(
                "{{\n",
                "  \"schema_version\": \"pglib-case-audit.v0.1.0\",\n",
                "  \"source_git_blob_sha1\": \"{}\",\n",
                "  \"function_name\": \"{}\",\n",
                "  \"matpower_version\": \"{}\",\n",
                "  \"base_mva\": {},\n",
                "  \"bus_count\": {},\n",
                "  \"generator_count\": {},\n",
                "  \"branch_count\": {},\n",
                "  \"cost_row_count\": {},\n",
                "  \"in_service_generator_count\": {},\n",
                "  \"in_service_branch_count\": {},\n",
                "  \"energized_island_count\": {},\n",
                "  \"reference_buses\": [{}],\n",
                "  \"claim_ceiling\": \"{}\"\n",
                "}}"
            ),
            self.source_git_blob_sha1,
            self.function_name,
            self.matpower_version,
            format_number(self.base_mva),
            self.bus_count,
            self.generator_count,
            self.branch_count,
            self.cost_row_count,
            self.in_service_generator_count,
            self.in_service_branch_count,
            self.energized_island_count,
            refs,
            self.claim_ceiling
        )
    }
}

fn format_number(value: f64) -> String {
    let rendered = format!("{value:.12}");
    rendered
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

/// Compute Git's SHA-1 object ID for a blob, including its canonical header.
pub fn git_blob_sha1(bytes: &[u8]) -> String {
    let header = format!("blob {}\0", bytes.len());
    let mut payload = Vec::with_capacity(header.len() + bytes.len());
    payload.extend_from_slice(header.as_bytes());
    payload.extend_from_slice(bytes);
    hex(&sha1(&payload))
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(DIGITS[(byte >> 4) as usize] as char);
        out.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    out
}

fn sha1(input: &[u8]) -> [u8; 20] {
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut message = input.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    let mut h0 = 0x67452301_u32;
    let mut h1 = 0xefcdab89_u32;
    let mut h2 = 0x98badcfe_u32;
    let mut h3 = 0x10325476_u32;
    let mut h4 = 0xc3d2e1f0_u32;

    for chunk in message.chunks_exact(64) {
        let mut w = [0_u32; 80];
        for (index, word) in w.iter_mut().take(16).enumerate() {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for index in 16..80 {
            w[index] = (w[index - 3] ^ w[index - 8] ^ w[index - 14] ^ w[index - 16]).rotate_left(1);
        }

        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
        for (index, word) in w.iter().enumerate() {
            let (f, k) = match index {
                0..=19 => ((b & c) | ((!b) & d), 0x5a827999),
                20..=39 => (b ^ c ^ d, 0x6ed9eba1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1bbcdc),
                _ => (b ^ c ^ d, 0xca62c1d6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut result = [0_u8; 20];
    for (index, word) in [h0, h1, h2, h3, h4].iter().enumerate() {
        result[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    result
}

#[derive(Default)]
struct CaseTables {
    function_name: Option<String>,
    version: Option<String>,
    base_mva: Option<f64>,
    matrices: BTreeMap<String, Vec<Vec<f64>>>,
    saw_end: bool,
}

enum ParseState {
    Ready,
    Matrix { name: String, content: String },
}

fn strip_comment(line: &str) -> String {
    let mut quoted = false;
    for (index, ch) in line.char_indices() {
        if ch == '\'' {
            quoted = !quoted;
        } else if ch == '%' && !quoted {
            return line[..index].to_owned();
        }
    }
    line.to_owned()
}

fn parse_assignment_number(statement: &str, key: &str) -> Result<f64, String> {
    let prefix = format!("mpc.{key}");
    let rest = statement
        .strip_prefix(&prefix)
        .ok_or_else(|| format!("expected assignment to {key}"))?
        .trim();
    let rest = rest
        .strip_prefix('=')
        .ok_or_else(|| format!("expected '=' in {key} assignment"))?
        .trim()
        .trim_end_matches(';')
        .trim();
    parse_number(rest, &format!("{key} assignment"))
}

fn parse_number(token: &str, context: &str) -> Result<f64, String> {
    let value = token
        .parse::<f64>()
        .map_err(|_| format!("invalid numeric value '{token}' in {context}"))?;
    if !value.is_finite() {
        return Err(format!("non-finite numeric value in {context}"));
    }
    Ok(value)
}

fn parse_integer(value: f64, context: &str) -> Result<i64, String> {
    if value.fract() != 0.0 || value < i64::MIN as f64 || value > i64::MAX as f64 {
        return Err(format!("expected integer in {context}"));
    }
    Ok(value as i64)
}

fn parse_matrix(name: &str, content: &str) -> Result<Vec<Vec<f64>>, String> {
    let mut rows: Vec<Vec<f64>> = Vec::new();
    for raw_row in content.split(';') {
        let row = raw_row.trim();
        if row.is_empty() {
            continue;
        }
        let values = row
            .split(|ch: char| ch.is_whitespace() || ch == ',')
            .filter(|token| !token.is_empty())
            .map(|token| parse_number(token, name))
            .collect::<Result<Vec<_>, _>>()?;
        if values.is_empty() {
            continue;
        }
        if let Some(expected) = rows.first().map(Vec::len) {
            if values.len() != expected {
                return Err(format!(
                    "inconsistent row width in mpc.{name}: expected {expected}, got {}",
                    values.len()
                ));
            }
        }
        rows.push(values);
    }
    if rows.is_empty() {
        return Err(format!("mpc.{name} matrix is empty"));
    }
    Ok(rows)
}

fn parse_case_source(source: &str) -> Result<CaseTables, String> {
    let mut parsed = CaseTables::default();
    let mut collecting: Option<(String, String)> = None;

    for (line_index, raw_line) in source.lines().enumerate() {
        let line_number = line_index + 1;
        let clean = strip_comment(raw_line);
        let line = clean.trim();
        if line.is_empty() {
            continue;
        }
        if parsed.saw_end {
            return Err(format!(
                "unexpected statement after function end at line {line_number}"
            ));
        }

        if let Some((name, mut content)) = collecting.take() {
            if let Some(close) = line.find("];") {
                content.push_str(&line[..close]);
                if !line[close + 2..].trim().is_empty() {
                    return Err(format!(
                        "unexpected text after mpc.{name} matrix at line {line_number}"
                    ));
                }
                let matrix = parse_matrix(&name, &content)?;
                if parsed.matrices.insert(name.clone(), matrix).is_some() {
                    return Err(format!("duplicate mpc.{name} matrix"));
                }
            } else {
                if line.contains(']') {
                    return Err(format!("malformed matrix terminator at line {line_number}"));
                }
                content.push_str(line);
                content.push('\n');
                collecting = Some((name, content));
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("function mpc = ") {
            if parsed.function_name.is_some() {
                return Err(format!(
                    "duplicate function declaration at line {line_number}"
                ));
            }
            let name = rest.trim();
            if name.is_empty()
                || !name
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                return Err(format!(
                    "unsupported function declaration at line {line_number}"
                ));
            }
            parsed.function_name = Some(name.to_owned());
        } else if line.starts_with("mpc.version") {
            if parsed.version.is_some() {
                return Err(format!(
                    "duplicate mpc.version assignment at line {line_number}"
                ));
            }
            let rest = line
                .strip_prefix("mpc.version")
                .unwrap()
                .trim()
                .strip_prefix('=')
                .ok_or_else(|| format!("invalid version assignment at line {line_number}"))?
                .trim()
                .trim_end_matches(';')
                .trim();
            let version = rest
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
                .ok_or_else(|| format!("version must be a quoted string at line {line_number}"))?;
            if version != "2" {
                return Err(format!(
                    "only MATPOWER case format version 2 is supported, got '{version}'"
                ));
            }
            parsed.version = Some(version.to_owned());
        } else if line.starts_with("mpc.baseMVA") {
            if parsed.base_mva.is_some() {
                return Err(format!(
                    "duplicate mpc.baseMVA assignment at line {line_number}"
                ));
            }
            parsed.base_mva = Some(parse_assignment_number(line, "baseMVA")?);
        } else if ["gencost", "branch", "bus", "gen"]
            .iter()
            .any(|name| line.starts_with(&format!("mpc.{name}")))
        {
            let name = ["gencost", "branch", "bus", "gen"]
                .into_iter()
                .find(|name| line.starts_with(&format!("mpc.{name}")))
                .unwrap();
            let prefix = format!("mpc.{name}");
            let rest = line
                .strip_prefix(&prefix)
                .unwrap()
                .trim()
                .strip_prefix('=')
                .ok_or_else(|| format!("invalid mpc.{name} assignment at line {line_number}"))?
                .trim();
            let rest = rest.strip_prefix('[').ok_or_else(|| {
                format!("mpc.{name} must be a literal matrix at line {line_number}")
            })?;
            if parsed.matrices.contains_key(name) {
                return Err(format!("duplicate mpc.{name} matrix"));
            }
            if let Some(close) = rest.find("];") {
                if !rest[close + 2..].trim().is_empty() {
                    return Err(format!(
                        "unexpected text after mpc.{name} matrix at line {line_number}"
                    ));
                }
                let matrix = parse_matrix(name, &rest[..close])?;
                parsed.matrices.insert(name.to_owned(), matrix);
            } else {
                if rest.contains(']') {
                    return Err(format!("malformed matrix terminator at line {line_number}"));
                }
                collecting = Some((name.to_owned(), format!("{rest}\n")));
            }
        } else if line == "end" {
            parsed.saw_end = true;
        } else {
            return Err(format!(
                "unsupported statement at line {line_number}: {line}"
            ));
        }
    }

    if let Some((name, _)) = collecting {
        return Err(format!("unterminated mpc.{name} matrix"));
    }
    Ok(parsed)
}

fn validate_width(matrix: &[Vec<f64>], width: usize, name: &str) -> Result<(), String> {
    for (index, row) in matrix.iter().enumerate() {
        if row.len() != width {
            return Err(format!(
                "unsupported mpc.{name} row width at row {}: expected {width}, got {}",
                index + 1,
                row.len()
            ));
        }
    }
    Ok(())
}

fn validate_binary_status(value: f64, context: &str) -> Result<bool, String> {
    match parse_integer(value, context)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(format!("status must be 0 or 1 in {context}")),
    }
}

fn connected_components(
    active_buses: &BTreeSet<i64>,
    adjacency: &BTreeMap<i64, Vec<i64>>,
    reference_bus_ids: &BTreeSet<i64>,
) -> Result<usize, String> {
    let mut visited = BTreeSet::new();
    let mut components = 0;

    for start in active_buses {
        if visited.contains(start) {
            continue;
        }
        components += 1;
        let mut queue = VecDeque::from([*start]);
        let mut references_in_component = 0usize;

        while let Some(bus) = queue.pop_front() {
            if !visited.insert(bus) {
                continue;
            }
            if reference_bus_ids.contains(&bus) {
                references_in_component += 1;
            }
            if let Some(neighbors) = adjacency.get(&bus) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        queue.push_back(*neighbor);
                    }
                }
            }
        }

        if references_in_component != 1 {
            return Err(format!(
                "energized island {components} has {references_in_component} reference buses; exactly one is required by this import profile"
            ));
        }
    }
    Ok(components)
}

/// Audit a literal classic MATPOWER v2 case file; no MATLAB code is executed.
pub fn audit_matpower_case(source: &str) -> Result<AuditSummary, String> {
    let parsed = parse_case_source(source)?;
    let function_name = parsed
        .function_name
        .ok_or_else(|| "missing function declaration".to_owned())?;
    let version = parsed
        .version
        .ok_or_else(|| "missing mpc.version".to_owned())?;
    let base_mva = parsed
        .base_mva
        .ok_or_else(|| "missing mpc.baseMVA".to_owned())?;
    if base_mva <= 0.0 {
        return Err("mpc.baseMVA must be greater than zero".to_owned());
    }

    let buses = parsed.matrices.get("bus").ok_or("missing mpc.bus")?;
    let generators = parsed.matrices.get("gen").ok_or("missing mpc.gen")?;
    let branches = parsed.matrices.get("branch").ok_or("missing mpc.branch")?;
    let costs = parsed
        .matrices
        .get("gencost")
        .ok_or("missing mpc.gencost")?;
    validate_width(buses, BUS_COLUMNS, "bus")?;
    validate_width(generators, GEN_COLUMNS, "gen")?;
    validate_width(branches, BRANCH_COLUMNS, "branch")?;
    if costs.len() != generators.len() {
        return Err(format!(
            "gencost row count {} does not match generator row count {}",
            costs.len(),
            generators.len()
        ));
    }

    let mut bus_types = BTreeMap::new();
    let mut active_buses = BTreeSet::new();
    let mut reference_bus_ids = BTreeSet::new();

    for (index, bus) in buses.iter().enumerate() {
        let id = parse_integer(bus[0], "bus identifier")?;
        let bus_type = parse_integer(bus[1], "bus type")?;
        if id <= 0 {
            return Err(format!(
                "bus identifier must be positive at row {}",
                index + 1
            ));
        }
        if !matches!(bus_type, 1..=4) {
            return Err(format!(
                "unsupported MATPOWER bus type {bus_type} at bus {id}"
            ));
        }
        if bus[11] < bus[12] {
            return Err(format!("Vmax is below Vmin at bus {id}"));
        }
        if bus_types.insert(id, bus_type).is_some() {
            return Err(format!("duplicate bus identifier {id}"));
        }
        if bus_type != 4 {
            active_buses.insert(id);
        }
        if bus_type == 3 {
            reference_bus_ids.insert(id);
        }
    }

    if active_buses.is_empty() {
        return Err("case has no energized buses".to_owned());
    }

    let mut in_service_generator_count = 0usize;
    for (index, generator) in generators.iter().enumerate() {
        let bus_id = parse_integer(generator[0], "generator bus identifier")?;
        let status = validate_binary_status(generator[7], "generator status")?;
        let p_max = generator[8];
        let p_min = generator[9];
        let q_max = generator[3];
        let q_min = generator[4];

        if !bus_types.contains_key(&bus_id) {
            return Err(format!(
                "generator row {} references unknown bus {bus_id}",
                index + 1
            ));
        }
        if p_max < p_min || q_max < q_min {
            return Err(format!(
                "generator bounds are inverted at row {}",
                index + 1
            ));
        }
        if status {
            if bus_types[&bus_id] == 4 {
                return Err(format!(
                    "in-service generator references isolated bus {bus_id}"
                ));
            }
            in_service_generator_count += 1;
        }
    }

    for (index, cost) in costs.iter().enumerate() {
        let model = parse_integer(cost[0], "gencost model")?;
        let points = parse_integer(cost[3], "gencost point/coefficient count")?;
        if points <= 0 {
            return Err(format!("gencost row {} has non-positive count", index + 1));
        }
        let expected = match model {
            1 => 4 + (points as usize) * 2,
            2 => 4 + points as usize,
            _ => {
                return Err(format!(
                    "unsupported gencost model {model} at row {}",
                    index + 1
                ));
            }
        };
        if cost.len() != expected {
            return Err(format!(
                "gencost row {} has {} columns; model {model} with count {points} requires {expected}",
                index + 1,
                cost.len()
            ));
        }
    }

    let mut adjacency: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    let mut in_service_branch_count = 0usize;
    for (index, branch) in branches.iter().enumerate() {
        let from = parse_integer(branch[0], "branch from-bus")?;
        let to = parse_integer(branch[1], "branch to-bus")?;
        let status = validate_binary_status(branch[10], "branch status")?;
        if !bus_types.contains_key(&from) || !bus_types.contains_key(&to) {
            return Err(format!(
                "branch row {} references an unknown bus ({from}, {to})",
                index + 1
            ));
        }
        if from == to {
            return Err(format!("branch row {} connects a bus to itself", index + 1));
        }
        if branch[8] < 0.0 {
            return Err(format!(
                "negative transformer tap ratio at branch row {}",
                index + 1
            ));
        }
        if status {
            if bus_types[&from] == 4 || bus_types[&to] == 4 {
                return Err(format!(
                    "in-service branch row {} touches an isolated bus",
                    index + 1
                ));
            }
            adjacency.entry(from).or_default().push(to);
            adjacency.entry(to).or_default().push(from);
            in_service_branch_count += 1;
        }
    }

    for bus_id in &reference_bus_ids {
        if !active_buses.contains(bus_id) {
            return Err(format!("reference bus {bus_id} is marked isolated"));
        }
    }
    let energized_island_count =
        connected_components(&active_buses, &adjacency, &reference_bus_ids)?;

    Ok(AuditSummary {
        source_git_blob_sha1: String::new(),
        function_name,
        matpower_version: version,
        base_mva,
        bus_count: buses.len(),
        generator_count: generators.len(),
        branch_count: branches.len(),
        cost_row_count: costs.len(),
        in_service_generator_count,
        in_service_branch_count,
        energized_island_count,
        reference_buses: reference_bus_ids.into_iter().collect(),
        claim_ceiling: "structural_import_only",
    })
}

/// Audit source bytes and bind the result to their Git blob identity.
pub fn audit_matpower_bytes(bytes: &[u8]) -> Result<AuditSummary, String> {
    let source = std::str::from_utf8(bytes).map_err(|_| "source is not UTF-8".to_owned())?;
    let mut summary = audit_matpower_case(source)?;
    summary.source_git_blob_sha1 = git_blob_sha1(bytes);
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PGLIB_CASE14: &str = include_str!("../fixtures/pglib_opf_case14_ieee.m");
    const SYNTHETIC_CASE: &str = r#"
function mpc = tiny_case
mpc.version = '2';
mpc.baseMVA = 100;
mpc.bus = [
  1 3 0 0 0 0 1 1 0 110 1 1.1 0.9;
  2 1 50 10 0 0 1 1 0 110 1 1.1 0.9;
];
mpc.gen = [
  1 50 0 10 -10 1 100 1 100 0;
];
mpc.gencost = [
  2 0 0 3 0.01 2 0;
];
mpc.branch = [
  1 2 0.01 0.1 0 100 100 100 0 0 1 -30 30;
];
end
"#;

    #[test]
    fn pglib_case14_is_structurally_admitted() {
        let result = audit_matpower_bytes(PGLIB_CASE14.as_bytes()).expect("case14 should parse");
        assert_eq!(
            result.source_git_blob_sha1,
            "2c8260fff0311c902c3d36dcabcbe861576fc97c"
        );
        assert_eq!(result.function_name, "pglib_opf_case14_ieee");
        assert_eq!(result.matpower_version, "2");
        assert_eq!(result.base_mva, 100.0);
        assert_eq!(result.bus_count, 14);
        assert_eq!(result.generator_count, 5);
        assert_eq!(result.branch_count, 20);
        assert_eq!(result.cost_row_count, 5);
        assert_eq!(result.in_service_generator_count, 5);
        assert_eq!(result.in_service_branch_count, 20);
        assert_eq!(result.energized_island_count, 1);
        assert_eq!(result.reference_buses, vec![1]);
        assert_eq!(result.claim_ceiling, "structural_import_only");
    }

    #[test]
    fn git_blob_sha1_matches_known_vector() {
        assert_eq!(
            git_blob_sha1(b"abc"),
            "f2ba8f84ab5c1bce84a7b441cb1959cfc7093b7f"
        );
    }

    #[test]
    fn small_synthetic_case_is_admitted() {
        let result = audit_matpower_case(SYNTHETIC_CASE).expect("synthetic case should parse");
        assert_eq!(result.bus_count, 2);
        assert_eq!(result.energized_island_count, 1);
        assert_eq!(result.reference_buses, vec![1]);
    }

    #[test]
    fn unknown_statement_is_rejected() {
        let source =
            SYNTHETIC_CASE.replace("mpc.baseMVA = 100;", "mpc.baseMVA = 100;\nmpc.mystery = 7;");
        assert!(
            audit_matpower_case(&source)
                .unwrap_err()
                .contains("unsupported statement")
        );
    }

    #[test]
    fn duplicate_bus_identifier_is_rejected() {
        let source = SYNTHETIC_CASE.replace(
            "2 1 50 10 0 0 1 1 0 110 1 1.1 0.9;",
            "1 1 50 10 0 0 1 1 0 110 1 1.1 0.9;",
        );
        assert!(
            audit_matpower_case(&source)
                .unwrap_err()
                .contains("duplicate bus")
        );
    }

    #[test]
    fn unknown_branch_bus_is_rejected() {
        let source = SYNTHETIC_CASE.replace(
            "1 2 0.01 0.1 0 100 100 100 0 0 1 -30 30;",
            "1 99 0.01 0.1 0 100 100 100 0 0 1 -30 30;",
        );
        assert!(
            audit_matpower_case(&source)
                .unwrap_err()
                .contains("unknown bus")
        );
    }

    #[test]
    fn connected_component_with_two_reference_buses_is_rejected() {
        let source = SYNTHETIC_CASE.replace(
            "2 1 50 10 0 0 1 1 0 110 1 1.1 0.9;",
            "2 3 50 10 0 0 1 1 0 110 1 1.1 0.9;",
        );
        assert!(
            audit_matpower_case(&source)
                .unwrap_err()
                .contains("exactly one is required")
        );
    }

    #[test]
    fn island_without_reference_bus_is_rejected() {
        let source = SYNTHETIC_CASE.replace(
            "  1 2 0.01 0.1 0 100 100 100 0 0 1 -30 30;",
            "  1 2 0.01 0.1 0 100 100 100 0 0 0 -30 30;",
        );
        assert!(
            audit_matpower_case(&source)
                .unwrap_err()
                .contains("exactly one is required")
        );
    }

    #[test]
    fn non_finite_values_are_rejected() {
        let source = SYNTHETIC_CASE.replace("mpc.baseMVA = 100;", "mpc.baseMVA = NaN;");
        assert!(
            audit_matpower_case(&source)
                .unwrap_err()
                .contains("non-finite numeric value")
        );
    }

    #[test]
    fn non_v2_case_is_rejected() {
        let source = SYNTHETIC_CASE.replace("mpc.version = '2';", "mpc.version = '3';");
        assert!(
            audit_matpower_case(&source)
                .unwrap_err()
                .contains("only MATPOWER case format version 2")
        );
    }

    #[test]
    fn malformed_matrix_width_is_rejected() {
        let source = SYNTHETIC_CASE.replace(
            "2 1 50 10 0 0 1 1 0 110 1 1.1 0.9;",
            "2 1 50 10 0 0 1 1 0 110 1 1.1;",
        );
        assert!(
            audit_matpower_case(&source)
                .unwrap_err()
                .contains("inconsistent row width")
        );
    }

    #[test]
    fn output_is_deterministic() {
        let result = audit_matpower_case(SYNTHETIC_CASE).expect("synthetic case should parse");
        assert_eq!(result.to_json(), result.to_json());
        assert!(
            result
                .to_json()
                .contains("\"claim_ceiling\": \"structural_import_only\"")
        );
    }
}
