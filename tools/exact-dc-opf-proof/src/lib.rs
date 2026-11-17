//! Exact-rational proof checker for one frozen, source-bound MATPOWER DC-OPF LP.
//!
//! This crate intentionally supports one exact fixture. It is not a general
//! MATPOWER parser or an OPF solver. Every arithmetic operation is checked;
//! overflow is a hard error and there is no floating-point fallback.

use std::cmp::Ordering;

pub const SOURCE_BLOB_SHA1: &str = "6f835a443a6b33a11fade361a437c7cfab9d0182";
pub const MODEL_PROFILE: &str = "matpower_8_1_makeBdc_dcopf_exact_two_bus_v0_1";
pub const TWO_BUS_SOURCE: &str =
    include_str!("../../matpower-dc-opf-kkt-verifier/fixtures/two_bus_binding_line.m");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rational {
    numerator: i128,
    denominator: i128,
}

impl Rational {
    pub fn new(numerator: i128, denominator: i128) -> Result<Self, String> {
        if denominator == 0 {
            return Err("rational denominator is zero".to_owned());
        }
        let mut n = numerator;
        let mut d = denominator;
        if d < 0 {
            n = n.checked_neg().ok_or("rational sign normalization overflow")?;
            d = d.checked_neg().ok_or("rational denominator overflow")?;
        }
        let divisor = gcd(
            n.checked_abs().ok_or("rational numerator magnitude overflow")? as u128,
            d.checked_abs().ok_or("rational denominator magnitude overflow")? as u128,
        );
        let divisor = i128::try_from(divisor).map_err(|_| "rational gcd overflow")?;
        Ok(Self { numerator: n / divisor, denominator: d / divisor })
    }

    pub const fn zero() -> Self {
        Self { numerator: 0, denominator: 1 }
    }

    pub const fn one() -> Self {
        Self { numerator: 1, denominator: 1 }
    }

    pub fn integer(value: i128) -> Self {
        Self { numerator: value, denominator: 1 }
    }

    pub fn numerator(self) -> i128 { self.numerator }
    pub fn denominator(self) -> i128 { self.denominator }

    pub fn checked_add(self, rhs: Self) -> Result<Self, String> {
        let common = gcd(self.denominator as u128, rhs.denominator as u128);
        let left_scale = rhs.denominator / i128::try_from(common).map_err(|_| "rational gcd overflow")?;
        let right_scale = self.denominator / i128::try_from(common).map_err(|_| "rational gcd overflow")?;
        let left = self.numerator.checked_mul(left_scale).ok_or("rational addition overflow")?;
        let right = rhs.numerator.checked_mul(right_scale).ok_or("rational addition overflow")?;
        let numerator = left.checked_add(right).ok_or("rational addition overflow")?;
        let denominator = self.denominator.checked_mul(left_scale).ok_or("rational denominator overflow")?;
        Self::new(numerator, denominator)
    }

    pub fn checked_sub(self, rhs: Self) -> Result<Self, String> {
        let negated = rhs.numerator.checked_neg().ok_or("rational subtraction overflow")?;
        self.checked_add(Self { numerator: negated, denominator: rhs.denominator })
    }

    pub fn checked_mul(self, rhs: Self) -> Result<Self, String> {
        let g1 = gcd(self.numerator.checked_abs().ok_or("rational numerator magnitude overflow")? as u128,
                     rhs.denominator as u128);
        let g2 = gcd(rhs.numerator.checked_abs().ok_or("rational numerator magnitude overflow")? as u128,
                     self.denominator as u128);
        let g1 = i128::try_from(g1).map_err(|_| "rational gcd overflow")?;
        let g2 = i128::try_from(g2).map_err(|_| "rational gcd overflow")?;
        let a = self.numerator / g1;
        let b = rhs.numerator / g2;
        let c = self.denominator / g2;
        let d = rhs.denominator / g1;
        Self::new(
            a.checked_mul(b).ok_or("rational multiplication overflow")?,
            c.checked_mul(d).ok_or("rational denominator overflow")?,
        )
    }

    pub fn checked_div(self, rhs: Self) -> Result<Self, String> {
        if rhs.numerator == 0 {
            return Err("rational division by zero".to_owned());
        }
        self.checked_mul(Self::new(rhs.denominator, rhs.numerator)?)
    }

    pub fn cmp_checked(self, rhs: Self) -> Result<Ordering, String> {
        let g = gcd(self.denominator as u128, rhs.denominator as u128);
        let lscale = rhs.denominator / i128::try_from(g).map_err(|_| "rational gcd overflow")?;
        let rscale = self.denominator / i128::try_from(g).map_err(|_| "rational gcd overflow")?;
        let left = self.numerator.checked_mul(lscale).ok_or("rational comparison overflow")?;
        let right = rhs.numerator.checked_mul(rscale).ok_or("rational comparison overflow")?;
        Ok(left.cmp(&right))
    }

    pub fn is_zero(self) -> bool { self.numerator == 0 }

    pub fn as_fraction(self) -> String {
        format!("{}/{}", self.numerator, self.denominator)
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a.max(1)
}

/// Git's SHA-1 object ID for a blob, including its canonical `blob <len>\\0` header.
pub fn git_blob_sha1(bytes: &[u8]) -> String {
    let header = format!("blob {}\0", bytes.len());
    let mut payload = Vec::with_capacity(header.len() + bytes.len());
    payload.extend_from_slice(header.as_bytes());
    payload.extend_from_slice(bytes);
    hex(&sha1(&payload))
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[(byte >> 4) as usize] as char);
        output.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    output
}

fn sha1(input: &[u8]) -> [u8; 20] {
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut message = input.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 { message.push(0); }
    message.extend_from_slice(&bit_len.to_be_bytes());

    let (mut h0, mut h1, mut h2, mut h3, mut h4) =
        (0x67452301_u32, 0xefcdab89_u32, 0x98badcfe_u32, 0x10325476_u32, 0xc3d2e1f0_u32);
    for chunk in message.chunks_exact(64) {
        let mut words = [0_u32; 80];
        for (index, word) in words.iter_mut().take(16).enumerate() {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                chunk[offset], chunk[offset + 1], chunk[offset + 2], chunk[offset + 3],
            ]);
        }
        for index in 16..80 {
            words[index] = (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
        for (index, word) in words.iter().enumerate() {
            let (f, k) = match index {
                0..=19 => ((b & c) | ((!b) & d), 0x5a827999),
                20..=39 => (b ^ c ^ d, 0x6ed9eba1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1bbcdc),
                _ => (b ^ c ^ d, 0xca62c1d6),
            };
            let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(*word);
            e = d; d = c; c = b.rotate_left(30); b = a; a = temp;
        }
        h0 = h0.wrapping_add(a); h1 = h1.wrapping_add(b); h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d); h4 = h4.wrapping_add(e);
    }
    let mut result = [0_u8; 20];
    for (index, word) in [h0, h1, h2, h3, h4].iter().enumerate() {
        result[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    result
}

#[derive(Clone, Debug)]
pub struct ExactModel {
    pub source_sha1: String,
    pub base_mva: Rational,
    pub susceptance_pu: Rational,
    pub demand_pu: Rational,
    pub line_limit_pu: Rational,
    pub generator_lower_pu: Rational,
    pub generator_upper_pu: Rational,
    pub generator_cost_pu: [Rational; 2],
}

#[derive(Clone, Copy, Debug)]
pub struct Primal {
    pub theta_reference_rad: Rational,
    pub theta_load_rad: Rational,
    pub generator_1_pu: Rational,
    pub generator_2_pu: Rational,
}

#[derive(Clone, Copy, Debug)]
pub struct Dual {
    pub bus_1_balance: Rational,
    pub bus_2_balance: Rational,
    pub line_lower: Rational,
    pub line_upper: Rational,
    pub generator_1_lower: Rational,
    pub generator_1_upper: Rational,
    pub generator_2_lower: Rational,
    pub generator_2_upper: Rational,
}

#[derive(Clone, Copy, Debug)]
pub struct Certificate {
    pub primal: Primal,
    pub dual: Dual,
}

#[derive(Clone, Debug)]
pub struct ExactProof {
    pub source_sha1: String,
    pub flow_pu: Rational,
    pub bus_1_residual_pu: Rational,
    pub bus_2_residual_pu: Rational,
    pub stationarity_theta_1: Rational,
    pub stationarity_theta_2: Rational,
    pub stationarity_pg_1: Rational,
    pub stationarity_pg_2: Rational,
    pub primal_objective_per_hour: Rational,
    pub dual_lower_bound_per_hour: Rational,
    pub primal_dual_gap_per_hour: Rational,
}

fn r(n: i128, d: i128) -> Result<Rational, String> { Rational::new(n, d) }

/// Admit only the exact frozen source bytes and the supported MATPOWER data rows.
/// This is intentionally not a general-purpose MATPOWER parser.
pub fn model_from_source(source: &str) -> Result<ExactModel, String> {
    let digest = git_blob_sha1(source.as_bytes());
    if digest != SOURCE_BLOB_SHA1 {
        return Err(format!("source digest mismatch: expected {SOURCE_BLOB_SHA1}, got {digest}"));
    }
    let required = [
        "mpc.version = '2';",
        "mpc.baseMVA = 100;",
        "1 3   0   0 0 0 1 1  0.000000000 230 1 1.1 0.9;",
        "2 1 100   0 0 0 1 1 -2.291831181 230 1 1.1 0.9;",
        "1 40 0 100 -100 1 100 1 200 0;",
        "2 60 0 100 -100 1 100 1 200 0;",
        "1 2 0.01 0.1 0 40 40 40 0 0 1 -30 30;",
        "2 0 0 2 10 0;",
        "2 0 0 2 30 0;",
    ];
    for fragment in required {
        if !source.contains(fragment) {
            return Err(format!("source does not match the supported exact model row: {fragment}"));
        }
    }

    let base_mva = r(100, 1)?;
    let demand_pu = r(100, 1)?.checked_div(base_mva)?;
    let line_limit_pu = r(40, 1)?.checked_div(base_mva)?;
    let susceptance_pu = Rational::one().checked_div(r(1, 10)?)?;
    let generator_lower_pu = r(0, 1)?;
    let generator_upper_pu = r(200, 1)?.checked_div(base_mva)?;
    let generator_cost_pu = [r(10, 1)?.checked_mul(base_mva)?, r(30, 1)?.checked_mul(base_mva)?];
    Ok(ExactModel {
        source_sha1: digest,
        base_mva,
        susceptance_pu,
        demand_pu,
        line_limit_pu,
        generator_lower_pu,
        generator_upper_pu,
        generator_cost_pu,
    })
}

pub fn known_certificate() -> Result<Certificate, String> {
    Ok(Certificate {
        primal: Primal {
            theta_reference_rad: r(0, 1)?,
            theta_load_rad: r(-1, 25)?,
            generator_1_pu: r(2, 5)?,
            generator_2_pu: r(3, 5)?,
        },
        dual: Dual {
            bus_1_balance: r(1000, 1)?,
            bus_2_balance: r(3000, 1)?,
            line_lower: r(0, 1)?,
            line_upper: r(2000, 1)?,
            generator_1_lower: r(0, 1)?,
            generator_1_upper: r(0, 1)?,
            generator_2_lower: r(0, 1)?,
            generator_2_upper: r(0, 1)?,
        },
    })
}

fn require_zero(label: &str, value: Rational) -> Result<(), String> {
    if !value.is_zero() {
        return Err(format!("{label} is not exactly zero: {}", value.as_fraction()));
    }
    Ok(())
}

fn require_nonnegative(label: &str, value: Rational) -> Result<(), String> {
    if value.cmp_checked(Rational::zero())? == Ordering::Less {
        return Err(format!("{label} is negative: {}", value.as_fraction()));
    }
    Ok(())
}

pub fn verify(model: &ExactModel, certificate: Certificate) -> Result<ExactProof, String> {
    let p = certificate.primal;
    let d = certificate.dual;
    let b = model.susceptance_pu;

    // h1 = b(theta1 - theta2) - p1 = 0
    let bus_1_residual = b.checked_mul(p.theta_reference_rad.checked_sub(p.theta_load_rad)?)?
        .checked_sub(p.generator_1_pu)?;
    // h2 = b(theta2 - theta1) - p2 + demand = 0
    let bus_2_residual = b.checked_mul(p.theta_load_rad.checked_sub(p.theta_reference_rad)?)?
        .checked_sub(p.generator_2_pu)?.checked_add(model.demand_pu)?;

    require_zero("reference angle", p.theta_reference_rad)?;
    require_zero("bus 1 active-power balance", bus_1_residual)?;
    require_zero("bus 2 active-power balance", bus_2_residual)?;

    let flow = b.checked_mul(p.theta_reference_rad.checked_sub(p.theta_load_rad)?)?;
    let negative_flow = Rational::zero().checked_sub(flow)?;
    for (label, value, lower, upper) in [
        ("generator 1", p.generator_1_pu, model.generator_lower_pu, model.generator_upper_pu),
        ("generator 2", p.generator_2_pu, model.generator_lower_pu, model.generator_upper_pu),
    ] {
        if value.cmp_checked(lower)? == Ordering::Less || value.cmp_checked(upper)? == Ordering::Greater {
            return Err(format!("{label} dispatch is outside exact bounds"));
        }
    }
    if flow.cmp_checked(model.line_limit_pu)? == Ordering::Greater
        || negative_flow.cmp_checked(model.line_limit_pu)? == Ordering::Greater {
        return Err("line flow violates its exact symmetric limit".to_owned());
    }

    for (label, multiplier) in [
        ("line lower dual", d.line_lower), ("line upper dual", d.line_upper),
        ("generator 1 lower dual", d.generator_1_lower),
        ("generator 1 upper dual", d.generator_1_upper),
        ("generator 2 lower dual", d.generator_2_lower),
        ("generator 2 upper dual", d.generator_2_upper),
    ] {
        require_nonnegative(label, multiplier)?;
    }

    // L = c'p + lambda1*h1 + lambda2*h2
    //     + muU*(f-limit) + muL*(-f-limit)
    //     + muLo*(lo-p) + muUp*(p-hi).
    let stationarity_theta_1 = b.checked_mul(
        d.bus_1_balance.checked_sub(d.bus_2_balance)?
            .checked_add(d.line_upper)?.checked_sub(d.line_lower)?,
    )?;
    let stationarity_theta_2 = Rational::zero().checked_sub(stationarity_theta_1)?;
    let stationarity_pg_1 = model.generator_cost_pu[0]
        .checked_sub(d.bus_1_balance)?
        .checked_sub(d.generator_1_lower)?
        .checked_add(d.generator_1_upper)?;
    let stationarity_pg_2 = model.generator_cost_pu[1]
        .checked_sub(d.bus_2_balance)?
        .checked_sub(d.generator_2_lower)?
        .checked_add(d.generator_2_upper)?;
    require_zero("theta 1 stationarity", stationarity_theta_1)?;
    require_zero("theta 2 stationarity", stationarity_theta_2)?;
    require_zero("generator 1 stationarity", stationarity_pg_1)?;
    require_zero("generator 2 stationarity", stationarity_pg_2)?;

    // Complementary slackness for each lower/upper inequality.
    let line_upper_slack = model.line_limit_pu.checked_sub(flow)?;
    let line_lower_slack = model.line_limit_pu.checked_add(flow)?;
    let g1_lower_slack = p.generator_1_pu.checked_sub(model.generator_lower_pu)?;
    let g1_upper_slack = model.generator_upper_pu.checked_sub(p.generator_1_pu)?;
    let g2_lower_slack = p.generator_2_pu.checked_sub(model.generator_lower_pu)?;
    let g2_upper_slack = model.generator_upper_pu.checked_sub(p.generator_2_pu)?;
    for (label, slack, multiplier) in [
        ("line upper", line_upper_slack, d.line_upper),
        ("line lower", line_lower_slack, d.line_lower),
        ("generator 1 lower", g1_lower_slack, d.generator_1_lower),
        ("generator 1 upper", g1_upper_slack, d.generator_1_upper),
        ("generator 2 lower", g2_lower_slack, d.generator_2_lower),
        ("generator 2 upper", g2_upper_slack, d.generator_2_upper),
    ] {
        require_zero(label, slack.checked_mul(multiplier)?)?;
    }

    let primal_objective = model.generator_cost_pu[0].checked_mul(p.generator_1_pu)?
        .checked_add(model.generator_cost_pu[1].checked_mul(p.generator_2_pu)?)?;
    // The only nonzero constant terms in the Lagrangian are lambda2*demand
    // and -mu_upper*line_limit. All lower/upper generator-bound multipliers
    // and lower-line multiplier are exactly zero in this certificate.
    let dual_lower_bound = d.bus_2_balance.checked_mul(model.demand_pu)?
        .checked_sub(d.line_upper.checked_mul(model.line_limit_pu)?)?
        .checked_sub(d.line_lower.checked_mul(model.line_limit_pu)?)?
        .checked_add(d.generator_1_lower.checked_mul(model.generator_lower_pu)?)?
        .checked_sub(d.generator_1_upper.checked_mul(model.generator_upper_pu)?)?
        .checked_add(d.generator_2_lower.checked_mul(model.generator_lower_pu)?)?
        .checked_sub(d.generator_2_upper.checked_mul(model.generator_upper_pu)?)?;
    let gap = primal_objective.checked_sub(dual_lower_bound)?;
    require_zero("exact primal-dual gap", gap)?;

    Ok(ExactProof {
        source_sha1: model.source_sha1.clone(),
        flow_pu: flow,
        bus_1_residual_pu: bus_1_residual,
        bus_2_residual_pu: bus_2_residual,
        stationarity_theta_1,
        stationarity_theta_2,
        stationarity_pg_1,
        stationarity_pg_2,
        primal_objective_per_hour: primal_objective,
        dual_lower_bound_per_hour: dual_lower_bound,
        primal_dual_gap_per_hour: gap,
    })
}

impl ExactProof {
    /// Deterministic JSON using exact fraction strings only; no floating-point values.
    pub fn to_json(&self) -> String {
        format!(
            concat!(
                "{{\n",
                "  \"schema_version\": \"exact-dc-opf-proof.v0.1.0\",\n",
                "  \"source_git_blob_sha1\": \"{}\",\n",
                "  \"model_profile\": \"{}\",\n",
                "  \"arithmetic\": \"checked_i128_rational_no_float_fallback\",\n",
                "  \"primal\": {{\n",
                "    \"theta_reference_rad\": \"0/1\",\n",
                "    \"theta_load_rad\": \"-1/25\",\n",
                "    \"generator_1_pu\": \"2/5\",\n",
                "    \"generator_2_pu\": \"3/5\",\n",
                "    \"line_flow_pu\": \"{}\"\n",
                "  }},\n",
                "  \"dual\": {{\n",
                "    \"bus_1_balance\": \"1000/1\",\n",
                "    \"bus_2_balance\": \"3000/1\",\n",
                "    \"line_upper\": \"2000/1\",\n",
                "    \"all_other_inequality_multipliers\": \"0/1\"\n",
                "  }},\n",
                "  \"exact_checks\": {{\n",
                "    \"bus_1_balance_residual_pu\": \"{}\",\n",
                "    \"bus_2_balance_residual_pu\": \"{}\",\n",
                "    \"stationarity_theta_1\": \"{}\",\n",
                "    \"stationarity_theta_2\": \"{}\",\n",
                "    \"stationarity_generator_1\": \"{}\",\n",
                "    \"stationarity_generator_2\": \"{}\",\n",
                "    \"primal_objective_per_hour\": \"{}\",\n",
                "    \"dual_lower_bound_per_hour\": \"{}\",\n",
                "    \"primal_dual_gap_per_hour\": \"{}\"\n",
                "  }},\n",
                "  \"proof_status\": \"exact_pass_for_frozen_rational_fixture\",\n",
                "  \"claim_ceiling\": \"exact optimality proof for this one source-bound convex two-bus DC-OPF LP; no general solver or physical-grid claim\"\n",
                "}}\n"
            ),
            self.source_sha1, MODEL_PROFILE, self.flow_pu.as_fraction(),
            self.bus_1_residual_pu.as_fraction(), self.bus_2_residual_pu.as_fraction(),
            self.stationarity_theta_1.as_fraction(), self.stationarity_theta_2.as_fraction(),
            self.stationarity_pg_1.as_fraction(), self.stationarity_pg_2.as_fraction(),
            self.primal_objective_per_hour.as_fraction(), self.dual_lower_bound_per_hour.as_fraction(),
            self.primal_dual_gap_per_hour.as_fraction(),
        )
    }
}

pub fn prove_frozen_fixture(source: &str) -> Result<ExactProof, String> {
    let model = model_from_source(source)?;
    let certificate = known_certificate()?;
    verify(&model, certificate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_source_digest_and_projection_are_admitted() {
        assert_eq!(git_blob_sha1(TWO_BUS_SOURCE.as_bytes()), SOURCE_BLOB_SHA1);
        let model = model_from_source(TWO_BUS_SOURCE).expect("frozen fixture should be admitted");
        assert_eq!(model.susceptance_pu.as_fraction(), "10/1");
        assert_eq!(model.demand_pu.as_fraction(), "1/1");
        assert_eq!(model.line_limit_pu.as_fraction(), "2/5");
        assert_eq!(model.generator_upper_pu.as_fraction(), "2/1");
        assert_eq!(model.generator_cost_pu[0].as_fraction(), "1000/1");
        assert_eq!(model.generator_cost_pu[1].as_fraction(), "3000/1");
    }

    #[test]
    fn exact_primal_dual_certificate_proves_2200_per_hour() {
        let proof = prove_frozen_fixture(TWO_BUS_SOURCE).expect("exact proof should pass");
        assert_eq!(proof.flow_pu.as_fraction(), "2/5");
        assert_eq!(proof.primal_objective_per_hour.as_fraction(), "2200/1");
        assert_eq!(proof.dual_lower_bound_per_hour.as_fraction(), "2200/1");
        assert_eq!(proof.primal_dual_gap_per_hour.as_fraction(), "0/1");
        assert_eq!(proof.to_json(), proof.to_json());
    }

    #[test]
    fn source_digest_mutation_is_rejected() {
        let changed = format!("{TWO_BUS_SOURCE}\n");
        assert!(model_from_source(&changed).unwrap_err().contains("source digest mismatch"));
    }

    #[test]
    fn line_limit_source_mutation_is_rejected() {
        let changed = TWO_BUS_SOURCE.replace("0 40 40 40 0", "0 41 41 41 0");
        assert!(model_from_source(&changed).unwrap_err().contains("source digest mismatch"));
    }

    #[test]
    fn wrong_binding_line_shadow_price_breaks_exact_stationarity() {
        let model = model_from_source(TWO_BUS_SOURCE).expect("model");
        let mut certificate = known_certificate().expect("certificate");
        certificate.dual.line_upper = Rational::integer(1900);
        assert!(verify(&model, certificate).unwrap_err().contains("theta 1 stationarity"));
    }

    #[test]
    fn altered_network_coefficient_breaks_stationarity() {
        let mut model = model_from_source(TWO_BUS_SOURCE).expect("model");
        model.susceptance_pu = Rational::integer(11);
        let certificate = known_certificate().expect("certificate");
        assert!(verify(&model, certificate).is_err());
    }

    #[test]
    fn altered_generator_bound_breaks_primal_or_complementarity() {
        let mut model = model_from_source(TWO_BUS_SOURCE).expect("model");
        model.generator_upper_pu = r(1, 2).expect("rational");
        let certificate = known_certificate().expect("certificate");
        assert!(verify(&model, certificate).is_err());
    }

    #[test]
    fn negative_inequality_multiplier_is_rejected() {
        let model = model_from_source(TWO_BUS_SOURCE).expect("model");
        let mut certificate = known_certificate().expect("certificate");
        certificate.dual.line_upper = Rational::integer(-1);
        assert!(verify(&model, certificate).unwrap_err().contains("line upper dual is negative"));
    }

    #[test]
    fn checked_arithmetic_refuses_overflow() {
        let large = Rational::integer(i128::MAX);
        assert!(large.checked_mul(Rational::integer(2)).is_err());
    }

    #[test]
    fn rational_values_are_canonical() {
        assert_eq!(r(2, 4).expect("rational").as_fraction(), "1/2");
        assert_eq!(r(-2, -4).expect("rational").as_fraction(), "1/2");
        assert!(Rational::new(1, 0).is_err());
    }
}
