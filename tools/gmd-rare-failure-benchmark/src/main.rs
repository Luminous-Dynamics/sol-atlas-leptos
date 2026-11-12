use std::collections::BTreeSet;
use std::env;
use std::process::ExitCode;

const DIMENSIONS: usize = 4;
const REGION_COUNT: usize = 12;
const BUDGET: usize = 4096;
const INITIAL_GLOBAL: usize = 256;
const GLOBAL_PERIOD: usize = 4;
const SEEDS: [u64; 5] = [1, 7, 42, 99, 2026];
const HALTON_BASES: [u64; DIMENSIONS] = [2, 3, 5, 7];

#[derive(Clone, Copy, Debug)]
struct Region {
    id: &'static str,
    center: [f64; DIMENSIONS],
    radius: [f64; DIMENSIONS],
}

// Frozen synthetic challenge. Each region is a disconnected ellipsoid in a
// normalized four-axis domain: GIC pulse-width, fault phase, CT remanence,
// and IBR current-limit response. These are not physical thresholds.
const REGIONS: [Region; REGION_COUNT] = [
    Region {
        id: "pulse_ct_01",
        center: [0.12, 0.14, 0.17, 0.20],
        radius: [0.055, 0.045, 0.055, 0.060],
    },
    Region {
        id: "phase_ct_02",
        center: [0.31, 0.28, 0.44, 0.33],
        radius: [0.045, 0.050, 0.060, 0.050],
    },
    Region {
        id: "weak_grid_03",
        center: [0.55, 0.18, 0.63, 0.52],
        radius: [0.050, 0.050, 0.050, 0.055],
    },
    Region {
        id: "nbd_shift_04",
        center: [0.78, 0.31, 0.72, 0.25],
        radius: [0.040, 0.045, 0.050, 0.060],
    },
    Region {
        id: "pulse_lag_05",
        center: [0.18, 0.72, 0.50, 0.62],
        radius: [0.055, 0.050, 0.055, 0.050],
    },
    Region {
        id: "gfm_limit_06",
        center: [0.43, 0.60, 0.26, 0.77],
        radius: [0.050, 0.055, 0.045, 0.050],
    },
    Region {
        id: "scr_fault_07",
        center: [0.67, 0.77, 0.83, 0.18],
        radius: [0.045, 0.050, 0.050, 0.055],
    },
    Region {
        id: "ct_burden_08",
        center: [0.88, 0.85, 0.36, 0.54],
        radius: [0.040, 0.045, 0.050, 0.050],
    },
    Region {
        id: "backup_n1_09",
        center: [0.28, 0.46, 0.88, 0.88],
        radius: [0.050, 0.050, 0.045, 0.045],
    },
    Region {
        id: "phase_limit_10",
        center: [0.52, 0.92, 0.52, 0.34],
        radius: [0.045, 0.035, 0.050, 0.055],
    },
    Region {
        id: "clock_alias_11",
        center: [0.82, 0.55, 0.12, 0.91],
        radius: [0.045, 0.050, 0.050, 0.040],
    },
    Region {
        id: "interaction_12",
        center: [0.07, 0.95, 0.93, 0.67],
        radius: [0.035, 0.030, 0.035, 0.050],
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Method {
    UniformGrid,
    LatinHypercube,
    Halton,
    MarginGuided,
}

impl Method {
    fn name(self) -> &'static str {
        match self {
            Self::UniformGrid => "uniform_grid",
            Self::LatinHypercube => "latin_hypercube",
            Self::Halton => "halton_low_discrepancy",
            Self::MarginGuided => "margin_guided_local_refinement",
        }
    }

    fn all() -> [Self; 4] {
        [
            Self::UniformGrid,
            Self::LatinHypercube,
            Self::Halton,
            Self::MarginGuided,
        ]
    }
}

#[derive(Clone, Copy, Debug)]
struct Observation {
    point: [f64; DIMENSIONS],
    margin: f64,
    region_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResultRow {
    method: &'static str,
    seed: u64,
    calls: usize,
    regions_found: usize,
    region_recall_ppm: u32,
    failure_hits: usize,
    occupied_bins: usize,
    best_margin_milli: i64,
    certified_cells: usize,
    sample_fingerprint: u64,
}

#[derive(Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn unit(&mut self) -> f64 {
        const SCALE: f64 = 1.0 / ((1u64 << 53) as f64);
        ((self.next_u64() >> 11) as f64) * SCALE
    }

    fn signed(&mut self) -> f64 {
        self.unit() * 2.0 - 1.0
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        for index in (1..items.len()).rev() {
            let swap = (self.next_u64() as usize) % (index + 1);
            items.swap(index, swap);
        }
    }
}

fn radical_inverse(mut index: u64, base: u64) -> f64 {
    let inverse = 1.0 / base as f64;
    let mut factor = inverse;
    let mut value = 0.0;
    while index > 0 {
        value += (index % base) as f64 * factor;
        index /= base;
        factor *= inverse;
    }
    value
}

fn halton_point(index: usize) -> [f64; DIMENSIONS] {
    let n = (index as u64) + 1;
    std::array::from_fn(|dimension| radical_inverse(n, HALTON_BASES[dimension]))
}

fn oracle(point: [f64; DIMENSIONS]) -> Observation {
    let mut best_margin = f64::INFINITY;
    let mut best_region = 0;
    for (index, region) in REGIONS.iter().enumerate() {
        let mut normalized_squared_distance = 0.0;
        for dimension in 0..DIMENSIONS {
            let delta = (point[dimension] - region.center[dimension]) / region.radius[dimension];
            normalized_squared_distance += delta * delta;
        }
        let margin = normalized_squared_distance - 1.0;
        if margin < best_margin {
            best_margin = margin;
            best_region = index;
        }
    }
    Observation {
        point,
        margin: best_margin,
        region_index: best_region,
    }
}

fn in_domain(point: [f64; DIMENSIONS]) -> [f64; DIMENSIONS] {
    point.map(|value| value.clamp(0.0, 1.0))
}

fn sample_grid(index: usize) -> [f64; DIMENSIONS] {
    // 8^4 cells exactly match the 4096-evaluation budget.
    std::array::from_fn(|dimension| {
        let cell = (index >> (dimension * 3)) & 7;
        (cell as f64 + 0.5) / 8.0
    })
}

fn sample_latin_hypercube(seed: u64) -> Vec<[f64; DIMENSIONS]> {
    let mut rng = SplitMix64::new(seed);
    let mut permutations: [Vec<usize>; DIMENSIONS] = std::array::from_fn(|_| (0..BUDGET).collect());
    for permutation in &mut permutations {
        rng.shuffle(permutation);
    }
    (0..BUDGET)
        .map(|sample| {
            std::array::from_fn(|dimension| {
                (permutations[dimension][sample] as f64 + rng.unit()) / BUDGET as f64
            })
        })
        .collect()
}

fn sample_margin_guided(seed: u64) -> Vec<[f64; DIMENSIONS]> {
    let mut rng = SplitMix64::new(seed);
    let mut points = Vec::with_capacity(BUDGET);
    let mut observations = Vec::with_capacity(BUDGET);
    for index in 0..INITIAL_GLOBAL {
        let point = halton_point(index);
        points.push(point);
        observations.push(oracle(point));
    }

    for step in INITIAL_GLOBAL..BUDGET {
        if step % GLOBAL_PERIOD == 0 {
            // Keep global exploration alive so disconnected failure regions
            // are not abandoned after discovering a single promising basin.
            let index = INITIAL_GLOBAL + step / GLOBAL_PERIOD;
            let point = halton_point(index);
            points.push(point);
            observations.push(oracle(point));
            continue;
        }

        // Rotate through the best 48 observed margins instead of repeatedly
        // exploiting only the single best point.
        let mut ranked: Vec<usize> = (0..observations.len()).collect();
        ranked.sort_unstable_by(|left, right| {
            observations[*left]
                .margin
                .total_cmp(&observations[*right].margin)
        });
        let pool_size = ranked.len().min(48);
        let rank = ((step - INITIAL_GLOBAL) / 3) % pool_size;
        let base = observations[ranked[rank]].point;
        let round = (step - INITIAL_GLOBAL) / (pool_size * 3 + 1);
        let radius = (0.16 / (1.0 + round as f64 * 0.35)).max(0.012);
        let point = in_domain(std::array::from_fn(|dimension| {
            base[dimension] + rng.signed() * radius
        }));
        points.push(point);
        observations.push(oracle(point));
    }
    points
}

fn fingerprint(points: &[[f64; DIMENSIONS]]) -> u64 {
    // FNV-1a over IEEE-754 coordinate bytes: deterministic replay receipt,
    // not a cryptographic provenance digest.
    let mut hash = 0xcbf29ce484222325u64;
    for point in points {
        for coordinate in point {
            for byte in coordinate.to_bits().to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
    }
    hash
}

fn evaluate(method: Method, seed: u64) -> ResultRow {
    let points = match method {
        Method::UniformGrid => (0..BUDGET).map(sample_grid).collect(),
        Method::LatinHypercube => sample_latin_hypercube(seed),
        Method::Halton => (0..BUDGET).map(halton_point).collect(),
        Method::MarginGuided => sample_margin_guided(seed),
    };
    assert_eq!(points.len(), BUDGET);

    let mut found = [false; REGION_COUNT];
    let mut failure_hits = 0;
    let mut best_margin = f64::INFINITY;
    let mut occupied = BTreeSet::new();
    for point in &points {
        let observation = oracle(*point);
        if observation.margin <= 0.0 {
            found[observation.region_index] = true;
            failure_hits += 1;
        }
        best_margin = best_margin.min(observation.margin);

        let bin: [u8; DIMENSIONS] =
            std::array::from_fn(|dimension| (point[dimension] * 8.0).floor().clamp(0.0, 7.0) as u8);
        occupied.insert(bin);
    }
    let regions_found = found.iter().filter(|found| **found).count();
    ResultRow {
        method: method.name(),
        seed,
        calls: points.len(),
        regions_found,
        region_recall_ppm: ((regions_found as u64 * 1_000_000) / REGION_COUNT as u64) as u32,
        failure_hits,
        occupied_bins: occupied.len(),
        best_margin_milli: (best_margin * 1000.0).round() as i64,
        // This benchmark only discovers counterexamples. It has no
        // whole-cell enclosure mechanism and is forbidden to certify cells.
        certified_cells: 0,
        sample_fingerprint: fingerprint(&points),
    }
}

fn emit(rows: &[ResultRow]) {
    println!("{{");
    println!("  \"benchmark_id\": \"GMD-TRAJECTORY-FALSIFICATION-001\",");
    println!("  \"claim_ceiling\": \"E2\",");
    println!("  \"synthetic_only\": true,");
    println!("  \"dimensions\": {DIMENSIONS},");
    println!("  \"rare_failure_regions\": {REGION_COUNT},");
    println!("  \"matched_budget_per_method\": {BUDGET},");
    println!("  \"certification_authorized\": false,");
    println!("  \"results\": [");
    for (index, row) in rows.iter().enumerate() {
        let comma = if index + 1 == rows.len() { "" } else { "," };
        println!(
            "    {{\"method\":\"{}\",\"seed\":{},\"calls\":{},\"regions_found\":{},\"region_recall_ppm\":{},\"failure_hits\":{},\"occupied_bins\":{},\"best_margin_milli\":{},\"certified_cells\":{},\"sample_fingerprint\":\"{:016x}\"}}{}",
            row.method,
            row.seed,
            row.calls,
            row.regions_found,
            row.region_recall_ppm,
            row.failure_hits,
            row.occupied_bins,
            row.best_margin_milli,
            row.certified_cells,
            row.sample_fingerprint,
            comma
        );
    }
    println!("  ]");
    println!("}}");
}

fn main() -> ExitCode {
    let mut rows = Vec::new();
    for seed in SEEDS {
        for method in Method::all() {
            rows.push(evaluate(method, seed));
        }
    }
    emit(&rows);
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_frozen_centers_are_known_failure_points() {
        for region in REGIONS {
            assert!(oracle(region.center).margin < 0.0, "{}", region.id);
        }
    }

    #[test]
    fn every_method_receives_the_same_evaluation_budget() {
        for seed in SEEDS {
            for method in Method::all() {
                let result = evaluate(method, seed);
                assert_eq!(result.calls, BUDGET);
                assert_eq!(result.certified_cells, 0);
            }
        }
    }

    #[test]
    fn latin_hypercube_and_margin_guided_runs_are_reproducible() {
        for method in [Method::LatinHypercube, Method::MarginGuided] {
            assert_eq!(evaluate(method, 42), evaluate(method, 42));
        }
    }

    #[test]
    fn different_latin_hypercube_seeds_change_the_design() {
        assert_ne!(
            evaluate(Method::LatinHypercube, 1).sample_fingerprint,
            evaluate(Method::LatinHypercube, 7).sample_fingerprint
        );
    }

    #[test]
    fn discovery_receipts_never_claim_safety_certification() {
        for method in Method::all() {
            let result = evaluate(method, 2026);
            assert_eq!(result.certified_cells, 0);
            assert_eq!(
                result.region_recall_ppm,
                (result.regions_found as u32 * 1_000_000) / REGION_COUNT as u32
            );
        }
    }

    #[test]
    fn sample_methods_respect_normalized_domain() {
        for method in Method::all() {
            let row = evaluate(method, 99);
            assert_eq!(row.calls, BUDGET);
        }
    }
}
