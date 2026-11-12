use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

const SCHEMA_VERSION: &str = "0.1.0";
const BENCHMARK_ID: &str = "GMD-TRAJECTORY-CLOCK-001";
const NS_PER_SECOND: i128 = 1_000_000_000;
const PPB_SCALE: i128 = 1_000_000_000;
const PHASE_SCALE: i128 = 1_000_000_000_000;
const MILLIDEGREES_PER_CYCLE: i128 = 360_000;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Experiment {
    schema_version: String,
    benchmark_id: String,
    fixture_notice: String,
    synthetic_only: bool,
    claim_ceiling: String,
    region_profile: RegionProfile,
    reference_epoch_tai_ns: i64,
    phase_reference: PhaseReference,
    clocks: Vec<Clock>,
    channels: Vec<Channel>,
    transformations: Vec<Transformation>,
    events: Vec<Event>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionProfile {
    region_id: String,
    nominal_frequency_millihz: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseReference {
    zero_time_tai_ns: i64,
    zero_phase_millideg: u32,
    nominal_frequency_millihz: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Clock {
    clock_id: String,
    anchor_local_ns: u64,
    offset_bounds_ns: [i64; 2],
    observed_offset_ns: i64,
    drift_bounds_ppb: [i64; 2],
    observed_drift_ppb: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SignalKind {
    Gic,
    CtSecondary,
    RelayInput,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Channel {
    channel_id: String,
    signal_kind: SignalKind,
    clock_id: String,
    unit: String,
    polarity: String,
    time_column: String,
    sample_period_ns: u64,
    sample_rate_numerator_hz: u64,
    sample_rate_denominator: u64,
    sample_timestamps_ns: Vec<u64>,
    nominal_frequency_millihz: Option<u32>,
    source_artifact: ArtifactRef,
    effective_artifact: ArtifactRef,
    transformation_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct ArtifactRef {
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum TransformationKind {
    Resample,
    Filter,
    PolarityInversion,
    UnitConversion,
    ClockCorrection,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Transformation {
    transformation_id: String,
    operation: TransformationKind,
    input_artifact: ArtifactRef,
    output_artifact: ArtifactRef,
    method: String,
    implementation_revision: String,
    parameters_sha256: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum EventKind {
    GicOnset,
    GicPeak,
    TransformerResponse,
    FaultInception,
    FaultClearing,
    IbrCurrentLimit,
    RelayPickup,
    RelayTrip,
    BreakerOpen,
    ServiceFloorBreach,
    ServiceRecovered,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Event {
    event_id: String,
    kind: EventKind,
    clock_id: String,
    local_lower_ns: u64,
    local_upper_ns: u64,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum OrderKind {
    Before,
    After,
    Unresolved,
}

#[derive(Debug, Serialize)]
struct EventReceipt {
    event_id: String,
    kind: EventKind,
    mapped_interval_tai_ns: [i128; 2],
    phase: Option<PhaseInterval>,
}

#[derive(Debug, Serialize)]
struct PhaseInterval {
    lower_millideg: u32,
    upper_millideg: u32,
    wraps_zero: bool,
    full_cycle_uncertainty: bool,
    uncertainty_width_millideg: u32,
}

#[derive(Debug, Serialize)]
struct EventOrderReceipt {
    first_event_id: String,
    second_event_id: String,
    relation: OrderKind,
}

#[derive(Debug, Serialize)]
struct ChannelReceipt {
    channel_id: String,
    signal_kind: SignalKind,
    nominal_frequency_millihz: Option<u32>,
    sample_rate_numerator_hz: u64,
    sample_rate_denominator: u64,
    sample_count: usize,
    sample_gap_count: usize,
    transformations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Report {
    status: &'static str,
    benchmark_id: String,
    schema_version: String,
    input_sha256: String,
    synthetic_only: bool,
    claim_ceiling: String,
    region_id: String,
    nominal_frequency_millihz: u32,
    channels: Vec<ChannelReceipt>,
    events: Vec<EventReceipt>,
    event_order: Vec<EventOrderReceipt>,
    limitations: Vec<&'static str>,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn safe_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

fn validate_pair(label: &str, pair: [i64; 2]) -> Result<(), String> {
    if pair[0] > pair[1] {
        return Err(format!("{label}: lower bound exceeds upper bound"));
    }
    Ok(())
}

fn floor_div(numerator: i128, denominator: i128) -> i128 {
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    if remainder != 0 && ((remainder < 0) != (denominator < 0)) {
        quotient - 1
    } else {
        quotient
    }
}

fn ceil_div(numerator: i128, denominator: i128) -> i128 {
    -floor_div(-numerator, denominator)
}

fn verify_artifact(reference: &ArtifactRef, root: &Path) -> Result<Vec<u8>, String> {
    if !valid_sha256(&reference.sha256) {
        return Err(format!("malformed SHA-256 for {}", reference.path));
    }
    if !safe_relative_path(&reference.path) {
        return Err(format!("unsafe artifact path {}", reference.path));
    }
    let path = root.join(&reference.path);
    let bytes = fs::read(&path)
        .map_err(|error| format!("cannot read artifact {}: {error}", path.display()))?;
    let actual = sha256(&bytes);
    if actual != reference.sha256 {
        return Err(format!(
            "artifact digest mismatch for {}: expected {}, got {}",
            reference.path, reference.sha256, actual
        ));
    }
    Ok(bytes)
}

fn verify_time_column(
    channel: &Channel,
    artifact_bytes: &[u8],
) -> Result<(), String> {
    let content = std::str::from_utf8(artifact_bytes)
        .map_err(|_| format!("{}: effective artifact is not UTF-8 CSV", channel.channel_id))?;
    let mut lines = content.lines();
    let header = lines
        .next()
        .ok_or_else(|| format!("{}: empty CSV artifact", channel.channel_id))?;
    let columns: Vec<&str> = header.split(',').collect();
    let time_index = columns
        .iter()
        .position(|column| *column == channel.time_column)
        .ok_or_else(|| format!("{}: time column not present in CSV", channel.channel_id))?;
    let mut actual_times = Vec::new();
    for (line_number, line) in lines.enumerate() {
        let fields: Vec<&str> = line.split(',').collect();
        let value = fields
            .get(time_index)
            .ok_or_else(|| format!("{}: malformed CSV row {}", channel.channel_id, line_number + 2))?;
        let timestamp = value
            .parse::<u64>()
            .map_err(|_| format!("{}: invalid time value on CSV row {}", channel.channel_id, line_number + 2))?;
        actual_times.push(timestamp);
    }
    if actual_times != channel.sample_timestamps_ns {
        return Err(format!(
            "{}: manifest sample timestamps do not match effective artifact time column",
            channel.channel_id
        ));
    }
    Ok(())
}

fn verify_transformation_chain(
    channel: &Channel,
    transformations: &BTreeMap<&str, &Transformation>,
    root: &Path,
) -> Result<(), String> {
    verify_artifact(&channel.source_artifact, root)?;
    let effective_bytes = verify_artifact(&channel.effective_artifact, root)?;
    let mut current_digest = channel.source_artifact.sha256.as_str();
    let mut seen = BTreeSet::new();

    for transformation_id in &channel.transformation_ids {
        if !seen.insert(transformation_id.as_str()) {
            return Err(format!(
                "{}: duplicate transformation reference {}",
                channel.channel_id, transformation_id
            ));
        }
        let transformation = transformations
            .get(transformation_id.as_str())
            .ok_or_else(|| format!(
                "{}: undeclared transformation {}",
                channel.channel_id, transformation_id
            ))?;
        if transformation.input_artifact.sha256 != current_digest {
            return Err(format!(
                "{}: transformation {} input digest does not continue the artifact chain",
                channel.channel_id, transformation_id
            ));
        }
        if transformation.input_artifact.sha256 == transformation.output_artifact.sha256 {
            return Err(format!(
                "{}: transformation {} must produce a distinct artifact digest",
                channel.channel_id, transformation_id
            ));
        }
        match transformation.operation {
            TransformationKind::Resample
            | TransformationKind::Filter
            | TransformationKind::PolarityInversion
            | TransformationKind::UnitConversion
            | TransformationKind::ClockCorrection => {}
        }
        if transformation.method.trim().is_empty()
            || transformation.implementation_revision.trim().is_empty()
            || !valid_sha256(&transformation.parameters_sha256)
        {
            return Err(format!(
                "{}: transformation {} lacks method, implementation revision or parameter digest",
                channel.channel_id, transformation_id
            ));
        }
        verify_artifact(&transformation.input_artifact, root)?;
        verify_artifact(&transformation.output_artifact, root)?;
        current_digest = transformation.output_artifact.sha256.as_str();
    }

    if channel.transformation_ids.is_empty()
        && channel.source_artifact.sha256 != channel.effective_artifact.sha256
    {
        return Err(format!(
            "{}: source/effective artifact differ without declared transformations",
            channel.channel_id
        ));
    }
    if current_digest != channel.effective_artifact.sha256 {
        return Err(format!(
            "{}: effective artifact digest is not the end of its declared transformation chain",
            channel.channel_id
        ));
    }
    verify_time_column(channel, &effective_bytes)?;
    Ok(())
}

fn validate_experiment(experiment: &Experiment) -> Result<(), String> {
    if experiment.schema_version != SCHEMA_VERSION {
        return Err(format!("schema_version must be {SCHEMA_VERSION}"));
    }
    if experiment.benchmark_id != BENCHMARK_ID {
        return Err(format!("benchmark_id must be {BENCHMARK_ID}"));
    }
    if !experiment.synthetic_only
        || !experiment
            .fixture_notice
            .to_ascii_uppercase()
            .contains("ALL VALUES ARE SYNTHETIC")
        || experiment.claim_ceiling != "E2"
    {
        return Err("v0.1 accepts synthetic E2 fixtures only".to_string());
    }
    if !matches!(
        experiment.region_profile.nominal_frequency_millihz,
        50_000 | 60_000
    ) {
        return Err("nominal frequency must be explicitly declared as 50 Hz or 60 Hz in v0.1".to_string());
    }
    if experiment.region_profile.region_id == "south_africa"
        && experiment.region_profile.nominal_frequency_millihz != 50_000
    {
        return Err("South African profile must declare nominal 50-Hz context".to_string());
    }
    if experiment.phase_reference.nominal_frequency_millihz
        != experiment.region_profile.nominal_frequency_millihz
        || experiment.phase_reference.zero_phase_millideg >= 360_000
    {
        return Err("phase reference frequency or zero phase is inconsistent".to_string());
    }
    if experiment.clocks.is_empty() || experiment.channels.is_empty() || experiment.events.is_empty() {
        return Err("clocks, channels and events must be non-empty".to_string());
    }

    let mut clock_ids = BTreeSet::new();
    for clock in &experiment.clocks {
        if clock.clock_id.trim().is_empty() || !clock_ids.insert(clock.clock_id.as_str()) {
            return Err(format!("empty or duplicate clock ID {}", clock.clock_id));
        }
        validate_pair("clock offset bounds", clock.offset_bounds_ns)?;
        validate_pair("clock drift bounds", clock.drift_bounds_ppb)?;
        if clock.observed_offset_ns < clock.offset_bounds_ns[0]
            || clock.observed_offset_ns > clock.offset_bounds_ns[1]
        {
            return Err(format!("{}: observed clock offset is outside declared bounds", clock.clock_id));
        }
        if clock.observed_drift_ppb < clock.drift_bounds_ppb[0]
            || clock.observed_drift_ppb > clock.drift_bounds_ppb[1]
        {
            return Err(format!("{}: observed clock drift is outside declared bounds", clock.clock_id));
        }
        if clock.drift_bounds_ppb[0] <= -1_000_000_000 {
            return Err(format!("{}: clock drift bound would make the time mapping non-monotonic", clock.clock_id));
        }
    }

    let mut transformation_map = BTreeMap::new();
    for transformation in &experiment.transformations {
        if transformation.transformation_id.trim().is_empty()
            || transformation_map
                .insert(transformation.transformation_id.as_str(), transformation)
                .is_some()
        {
            return Err(format!(
                "empty or duplicate transformation ID {}",
                transformation.transformation_id
            ));
        }
    }

    let mut channel_ids = BTreeSet::new();
    let mut referenced_transformations = BTreeSet::new();
    for channel in &experiment.channels {
        if channel.channel_id.trim().is_empty() || !channel_ids.insert(channel.channel_id.as_str()) {
            return Err(format!("empty or duplicate channel ID {}", channel.channel_id));
        }
        if !clock_ids.contains(channel.clock_id.as_str()) {
            return Err(format!("{}: unknown clock {}", channel.channel_id, channel.clock_id));
        }
        if channel.unit.trim().is_empty() || channel.polarity.trim().is_empty() {
            return Err(format!("{}: unit and polarity must be explicit", channel.channel_id));
        }
        if channel.time_column.trim().is_empty() {
            return Err(format!("{}: time column must be explicit", channel.channel_id));
        }
        if channel.sample_period_ns == 0
            || channel.sample_rate_numerator_hz == 0
            || channel.sample_rate_denominator == 0
        {
            return Err(format!("{}: sample period and rational sample rate must be positive", channel.channel_id));
        }
        let left = u128::from(channel.sample_period_ns)
            .checked_mul(u128::from(channel.sample_rate_numerator_hz))
            .ok_or_else(|| format!("{}: sample-rate arithmetic overflow", channel.channel_id))?;
        let right = 1_000_000_000_u128
            .checked_mul(u128::from(channel.sample_rate_denominator))
            .ok_or_else(|| format!("{}: sample-rate arithmetic overflow", channel.channel_id))?;
        if left != right {
            return Err(format!("{}: sample period and rational sample rate disagree", channel.channel_id));
        }
        if channel.sample_timestamps_ns.len() < 2 {
            return Err(format!("{}: at least two samples are required", channel.channel_id));
        }
        let channel_clock = experiment
            .clocks
            .iter()
            .find(|clock| clock.clock_id == channel.clock_id)
            .expect("channel clock ID was checked");
        if channel.sample_timestamps_ns[0] < channel_clock.anchor_local_ns {
            return Err(format!("{}: sample sequence precedes the clock mapping anchor", channel.channel_id));
        }
        for pair in channel.sample_timestamps_ns.windows(2) {
            if pair[1] == pair[0] {
                return Err(format!("{}: duplicate sample timestamp", channel.channel_id));
            }
            if pair[1] < pair[0] {
                return Err(format!("{}: sample timestamps are not monotonic", channel.channel_id));
            }
            if pair[1] - pair[0] != channel.sample_period_ns {
                return Err(format!("{}: sample gap or unexpected sample interval", channel.channel_id));
            }
        }
        match channel.signal_kind {
            SignalKind::Gic => {
                if channel.nominal_frequency_millihz.is_some() {
                    return Err(format!("{}: GIC observation cadence must not be represented as AC frequency", channel.channel_id));
                }
            }
            SignalKind::CtSecondary | SignalKind::RelayInput => {
                if channel.nominal_frequency_millihz
                    != Some(experiment.region_profile.nominal_frequency_millihz)
                {
                    return Err(format!("{}: AC channel frequency does not match regional profile", channel.channel_id));
                }
            }
        }
        for id in &channel.transformation_ids {
            if !referenced_transformations.insert(id.as_str()) {
                return Err(format!("transformation {} is referenced more than once", id));
            }
        }
    }
    if referenced_transformations.len() != transformation_map.len() {
        return Err("one or more transformation records are unreferenced".to_string());
    }

    let mut event_ids = BTreeSet::new();
    for event in &experiment.events {
        if event.event_id.trim().is_empty() || !event_ids.insert(event.event_id.as_str()) {
            return Err(format!("empty or duplicate event ID {}", event.event_id));
        }
        if !clock_ids.contains(event.clock_id.as_str()) {
            return Err(format!("{}: event references unknown clock {}", event.event_id, event.clock_id));
        }
        if event.local_lower_ns > event.local_upper_ns {
            return Err(format!("{}: event-time interval is inverted", event.event_id));
        }
        let clock = experiment
            .clocks
            .iter()
            .find(|clock| clock.clock_id == event.clock_id)
            .expect("clock ID was checked");
        if event.local_lower_ns < clock.anchor_local_ns {
            return Err(format!("{}: event precedes the clock mapping anchor", event.event_id));
        }
    }
    Ok(())
}

fn map_interval(
    event: &Event,
    clock: &Clock,
    reference_epoch_tai_ns: i64,
) -> Result<[i128; 2], String> {
    let local_values = [event.local_lower_ns, event.local_upper_ns];
    let offsets = clock.offset_bounds_ns;
    let drifts = clock.drift_bounds_ppb;
    let mut lower = i128::MAX;
    let mut upper = i128::MIN;
    for local in local_values {
        let delta = i128::from(local - clock.anchor_local_ns);
        for offset in offsets {
            for drift in drifts {
                let product = delta
                    .checked_mul(i128::from(drift))
                    .ok_or_else(|| format!("{}: clock drift arithmetic overflow", event.event_id))?;
                let correction_low = floor_div(product, PPB_SCALE);
                let correction_high = ceil_div(product, PPB_SCALE);
                let base = i128::from(reference_epoch_tai_ns)
                    .checked_add(delta)
                    .and_then(|value| value.checked_add(i128::from(offset)))
                    .ok_or_else(|| format!("{}: timestamp arithmetic overflow", event.event_id))?;
                let candidate_lower = base
                    .checked_add(correction_low)
                    .ok_or_else(|| format!("{}: timestamp lower-bound overflow", event.event_id))?;
                let candidate_upper = base
                    .checked_add(correction_high)
                    .ok_or_else(|| format!("{}: timestamp upper-bound overflow", event.event_id))?;
                lower = lower.min(candidate_lower);
                upper = upper.max(candidate_upper);
            }
        }
    }
    Ok([lower, upper])
}

fn phase_interval(
    mapped: [i128; 2],
    phase_reference: &PhaseReference,
) -> Result<PhaseInterval, String> {
    let zero = i128::from(phase_reference.zero_time_tai_ns);
    if mapped[0] < zero {
        return Err("fault phase interval precedes declared phase reference".to_string());
    }
    let frequency = i128::from(phase_reference.nominal_frequency_millihz);
    let lower_delta = mapped[0] - zero;
    let upper_delta = mapped[1] - zero;
    let lower_product = lower_delta
        .checked_mul(frequency)
        .and_then(|value| value.checked_mul(MILLIDEGREES_PER_CYCLE))
        .ok_or_else(|| "phase arithmetic overflow".to_string())?;
    let upper_product = upper_delta
        .checked_mul(frequency)
        .and_then(|value| value.checked_mul(MILLIDEGREES_PER_CYCLE))
        .ok_or_else(|| "phase arithmetic overflow".to_string())?;
    let base_phase = i128::from(phase_reference.zero_phase_millideg);
    let lower_total = base_phase + floor_div(lower_product, PHASE_SCALE);
    let upper_total = base_phase + ceil_div(upper_product, PHASE_SCALE);
    let width = upper_total.saturating_sub(lower_total).max(0);
    let full = width >= MILLIDEGREES_PER_CYCLE;
    let lower_phase = lower_total.rem_euclid(MILLIDEGREES_PER_CYCLE) as u32;
    let upper_phase = upper_total.rem_euclid(MILLIDEGREES_PER_CYCLE) as u32;
    let wraps = full || lower_total.div_euclid(MILLIDEGREES_PER_CYCLE)
        != upper_total.div_euclid(MILLIDEGREES_PER_CYCLE);
    Ok(PhaseInterval {
        lower_millideg: lower_phase,
        upper_millideg: upper_phase,
        wraps_zero: wraps,
        full_cycle_uncertainty: full,
        uncertainty_width_millideg: width.min(MILLIDEGREES_PER_CYCLE) as u32,
    })
}

fn classify_order(first: [i128; 2], second: [i128; 2]) -> OrderKind {
    if first[1] < second[0] {
        OrderKind::Before
    } else if second[1] < first[0] {
        OrderKind::After
    } else {
        OrderKind::Unresolved
    }
}

fn verify_experiment(
    experiment: &Experiment,
    input_bytes: &[u8],
    artifact_root: &Path,
) -> Result<Report, String> {
    validate_experiment(experiment)?;
    let transformation_map: BTreeMap<&str, &Transformation> = experiment
        .transformations
        .iter()
        .map(|item| (item.transformation_id.as_str(), item))
        .collect();

    let mut channel_receipts = Vec::new();
    for channel in &experiment.channels {
        verify_transformation_chain(channel, &transformation_map, artifact_root)?;
        channel_receipts.push(ChannelReceipt {
            channel_id: channel.channel_id.clone(),
            signal_kind: channel.signal_kind,
            nominal_frequency_millihz: channel.nominal_frequency_millihz,
            sample_rate_numerator_hz: channel.sample_rate_numerator_hz,
            sample_rate_denominator: channel.sample_rate_denominator,
            sample_count: channel.sample_timestamps_ns.len(),
            sample_gap_count: 0,
            transformations: channel.transformation_ids.clone(),
        });
    }

    let mut mapped_events: Vec<([i128; 2], &Event)> = Vec::new();
    let mut event_receipts = Vec::new();
    for event in &experiment.events {
        let clock = experiment
            .clocks
            .iter()
            .find(|clock| clock.clock_id == event.clock_id)
            .expect("clock ID was validated");
        let mapped = map_interval(event, clock, experiment.reference_epoch_tai_ns)?;
        let phase = if event.kind == EventKind::FaultInception {
            Some(phase_interval(mapped, &experiment.phase_reference)?)
        } else {
            None
        };
        mapped_events.push((mapped, event));
        event_receipts.push(EventReceipt {
            event_id: event.event_id.clone(),
            kind: event.kind,
            mapped_interval_tai_ns: mapped,
            phase,
        });
    }

    let mut event_order = Vec::new();
    for first_index in 0..mapped_events.len() {
        for second_index in (first_index + 1)..mapped_events.len() {
            let (first_interval, first_event) = mapped_events[first_index];
            let (second_interval, second_event) = mapped_events[second_index];
            event_order.push(EventOrderReceipt {
                first_event_id: first_event.event_id.clone(),
                second_event_id: second_event.event_id.clone(),
                relation: classify_order(first_interval, second_interval),
            });
        }
    }

    Ok(Report {
        status: "valid_synthetic_contract",
        benchmark_id: experiment.benchmark_id.clone(),
        schema_version: experiment.schema_version.clone(),
        input_sha256: sha256(input_bytes),
        synthetic_only: true,
        claim_ceiling: experiment.claim_ceiling.clone(),
        region_id: experiment.region_profile.region_id.clone(),
        nominal_frequency_millihz: experiment.region_profile.nominal_frequency_millihz,
        channels: channel_receipts,
        events: event_receipts,
        event_order,
        limitations: vec![
            "E2 synthetic contract verification only; no physical clocks, relay, CT or grid are qualified.",
            "Clock bounds and phase references are declared inputs; this tool does not calibrate instruments or establish UTC/TAI traceability.",
            "The verifier checks artifact digests and CSV time-column consistency; it does not parse or validate all COMTRADE fields.",
            "Declared transformations are provenance records; their signal-processing correctness is not independently proved.",
            "Event ordering is partial: overlapping mapped intervals remain unresolved.",
            "No protection, switching, dispatch or operational authority is created.",
        ],
    })
}

fn run(path: &Path) -> Result<Report, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot read experiment {}: {error}", path.display()))?;
    let experiment: Experiment = serde_json::from_slice(&bytes)
        .map_err(|error| format!("experiment JSON does not match strict typed contract: {error}"))?;
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    verify_experiment(&experiment, &bytes, root)
}

fn main() -> ExitCode {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_default();
    let Some(path) = args.next() else {
        eprintln!(
            "usage: {} <synthetic-experiment.json>",
            PathBuf::from(program).display()
        );
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        eprintln!("accepts exactly one experiment path");
        return ExitCode::from(2);
    }
    match run(Path::new(&path)).and_then(|report| {
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("cannot encode report: {error}"))
    }) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("GMD trajectory clock verification failed: {error}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/research/gmd-ibr-protection/fixtures/gmd_trajectory_clock_experiment_v0_1.json")
    }

    fn fixture() -> (Experiment, Vec<u8>, PathBuf) {
        let path = fixture_path();
        let bytes = fs::read(&path).expect("fixture must be readable");
        let experiment: Experiment = serde_json::from_slice(&bytes).expect("fixture must parse");
        let root = path.parent().expect("fixture parent");
        (experiment, bytes, root.to_path_buf())
    }

    #[test]
    fn frozen_fixture_preserves_50hz_and_three_distinct_sample_rates() {
        let (experiment, bytes, root) = fixture();
        let report = verify_experiment(&experiment, &bytes, &root).expect("valid fixture");
        assert_eq!(report.nominal_frequency_millihz, 50_000);
        assert_eq!(report.channels.len(), 3);
        assert_eq!(report.channels[0].sample_rate_numerator_hz, 1);
        assert_eq!(report.channels[1].sample_rate_numerator_hz, 16_000);
        assert_eq!(report.channels[2].sample_rate_numerator_hz, 16_000);
        assert_eq!(report.channels[2].transformations, vec!["relay-resample-001"]);
    }

    #[test]
    fn generic_60hz_profile_is_valid_when_all_ac_channels_match() {
        let (mut experiment, bytes, root) = fixture();
        experiment.region_profile.region_id = "generic_60hz".to_string();
        experiment.region_profile.nominal_frequency_millihz = 60_000;
        experiment.phase_reference.nominal_frequency_millihz = 60_000;
        for channel in &mut experiment.channels {
            if channel.signal_kind != SignalKind::Gic {
                channel.nominal_frequency_millihz = Some(60_000);
            }
        }
        let report = verify_experiment(&experiment, &bytes, &root).expect("valid 60-Hz profile");
        assert_eq!(report.nominal_frequency_millihz, 60_000);
    }

    #[test]
    fn south_africa_rejects_60hz_even_if_all_channels_agree() {
        let (mut experiment, _bytes, _root) = fixture();
        experiment.region_profile.nominal_frequency_millihz = 60_000;
        experiment.phase_reference.nominal_frequency_millihz = 60_000;
        for channel in &mut experiment.channels {
            if channel.signal_kind != SignalKind::Gic {
                channel.nominal_frequency_millihz = Some(60_000);
            }
        }
        let error = validate_experiment(&experiment).unwrap_err();
        assert!(error.contains("South African profile"));
    }

    #[test]
    fn observed_clock_offset_must_remain_inside_calibration_bounds() {
        let (mut experiment, _bytes, _root) = fixture();
        experiment.clocks[0].observed_offset_ns = 9_000;
        let error = validate_experiment(&experiment).unwrap_err();
        assert!(error.contains("observed clock offset"));
    }

    #[test]
    fn observed_clock_drift_must_remain_inside_calibration_bounds() {
        let (mut experiment, _bytes, _root) = fixture();
        experiment.clocks[1].observed_drift_ppb = 900_000;
        let error = validate_experiment(&experiment).unwrap_err();
        assert!(error.contains("observed clock drift"));
    }

    #[test]
    fn clock_drift_mapping_uses_outward_rounded_bounds() {
        let (experiment, _bytes, _root) = fixture();
        let event = &experiment.events[0];
        let clock = experiment.clocks.iter().find(|c| c.clock_id == event.clock_id).unwrap();
        let mapped = map_interval(event, clock, experiment.reference_epoch_tai_ns).unwrap();
        assert!(mapped[0] <= mapped[1]);
        assert!(mapped[1] - mapped[0] > 0);
    }

    #[test]
    fn overlapping_event_intervals_remain_unresolved() {
        let (mut experiment, bytes, root) = fixture();
        experiment.events[1].local_lower_ns = experiment.events[0].local_lower_ns;
        experiment.events[1].local_upper_ns = experiment.events[0].local_upper_ns;
        experiment.events[1].clock_id = experiment.events[0].clock_id.clone();
        let report = verify_experiment(&experiment, &bytes, &root).expect("overlap is valid but unresolved");
        let relation = report.event_order.iter().find(|r|
            r.first_event_id == experiment.events[0].event_id
                && r.second_event_id == experiment.events[1].event_id
        ).unwrap();
        assert_eq!(relation.relation, OrderKind::Unresolved);
    }

    #[test]
    fn known_event_order_is_derived_from_non_overlapping_intervals() {
        let (experiment, bytes, root) = fixture();
        let report = verify_experiment(&experiment, &bytes, &root).expect("valid fixture");
        let relation = report.event_order.iter().find(|r|
            r.first_event_id == "gic-onset"
                && r.second_event_id == "fault-inception"
        ).unwrap();
        assert_eq!(relation.relation, OrderKind::Before);
    }

    #[test]
    fn sample_gap_is_rejected() {
        let (mut experiment, _bytes, _root) = fixture();
        experiment.channels[1].sample_timestamps_ns[2] += 62_500;
        let error = validate_experiment(&experiment).unwrap_err();
        assert!(error.contains("sample gap"));
    }

    #[test]
    fn duplicate_sample_timestamp_is_rejected() {
        let (mut experiment, _bytes, _root) = fixture();
        experiment.channels[1].sample_timestamps_ns[2] = experiment.channels[1].sample_timestamps_ns[1];
        let error = validate_experiment(&experiment).unwrap_err();
        assert!(error.contains("duplicate sample timestamp"));
    }

    #[test]
    fn sample_manifest_must_match_effective_csv_time_column() {
        let (mut experiment, bytes, root) = fixture();
        experiment.channels[1].sample_timestamps_ns[1] += 1;
        let error = verify_experiment(&experiment, &bytes, &root).unwrap_err();
        assert!(error.contains("sample gap") || error.contains("do not match"));
    }

    #[test]
    fn untransformed_different_artifact_is_rejected() {
        let (mut experiment, _bytes, root) = fixture();
        experiment.channels[2].transformation_ids.clear();
        let error = verify_experiment(&experiment, b"mutated", &root).unwrap_err();
        assert!(error.contains("source/effective artifact differ"));
    }

    #[test]
    fn declared_resampling_requires_complete_provenance_chain() {
        let (mut experiment, _bytes, root) = fixture();
        experiment.transformations[0].method.clear();
        let transformation_map: BTreeMap<&str, &Transformation> = experiment.transformations
            .iter().map(|item| (item.transformation_id.as_str(), item)).collect();
        let error = verify_transformation_chain(
            &experiment.channels[2], &transformation_map, &root
        ).unwrap_err();
        assert!(error.contains("method"));
    }

    #[test]
    fn south_african_50hz_fault_phase_is_reported_as_an_uncertainty_interval() {
        let (experiment, bytes, root) = fixture();
        let report = verify_experiment(&experiment, &bytes, &root).expect("valid fixture");
        let fault = report.events.iter().find(|e| e.kind == EventKind::FaultInception).unwrap();
        let phase = fault.phase.as_ref().unwrap();
        assert!(phase.uncertainty_width_millideg > 0);
        assert!(!phase.full_cycle_uncertainty);
    }

    #[test]
    fn fault_phase_is_marked_fully_ambiguous_when_clock_interval_spans_a_cycle() {
        let (mut experiment, bytes, root) = fixture();
        let fault = experiment.events.iter_mut()
            .find(|event| event.kind == EventKind::FaultInception)
            .unwrap();
        fault.local_upper_ns = fault.local_lower_ns + 20_000_000;
        let report = verify_experiment(&experiment, &bytes, &root).expect("valid wide phase interval");
        let receipt = report.events.iter()
            .find(|event| event.kind == EventKind::FaultInception)
            .unwrap();
        assert!(receipt.phase.as_ref().unwrap().full_cycle_uncertainty);
    }

    #[test]
    fn digest_mismatch_fails_closed() {
        let (mut experiment, bytes, root) = fixture();
        experiment.channels[0].effective_artifact.sha256 = "0".repeat(64);
        let error = verify_experiment(&experiment, &bytes, &root).unwrap_err();
        assert!(error.contains("artifact digest mismatch"));
    }
}
