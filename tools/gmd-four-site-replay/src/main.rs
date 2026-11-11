use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DATASET_COMMIT: &str = "089617756920bcc9df621f7f2299ba0644320cc3";
const MANIFEST: &str =
    include_str!("../../../docs/research/results/gmd_may2024_four_site_replay_v0_1.json");
const EXPECTED_PAIRS: usize = 2_341;

#[derive(Clone, Copy)]
struct Pin {
    path: &'static str,
    blob_sha1: &'static str,
    size: usize,
    role: &'static str,
    site: &'static str,
    date: Option<&'static str>,
    substation: Option<&'static str>,
}

const PINS: [Pin; 20] = [
    Pin {
        path: "data_original/tva/gic/GIC-measured/gic-bullrun_20240510.csv",
        blob_sha1: "edfcc1e6614a3b7477661f00e9e0f08cac98ae1a",
        size: 6_599_421,
        role: "measured_1s",
        site: "Bull Run",
        date: None,
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-measured/gic-montgomery_20240510.csv",
        blob_sha1: "d88cff5d1c3a9fc75bca17a0c1dbd6c1e35b5a1e",
        size: 6_615_823,
        role: "measured_1s",
        site: "Montgomery",
        date: None,
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-measured/gic-union_20240510.csv",
        blob_sha1: "2dfd89020012c3d834b2da9f89a772f2cb08387a",
        size: 6_614_809,
        role: "measured_1s",
        site: "Union",
        date: None,
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-measured/gic-widowscreek2_20240510.csv",
        blob_sha1: "da7f8c0ac7fe99421d1e82acee0afccfee6001dc",
        size: 6_604_529,
        role: "measured_1s",
        site: "Widows Creek",
        date: None,
        substation: None,
    },
    Pin {
        path: "data_original/gmu/tva/site_108137138.csv",
        blob_sha1: "c08d4d359838eb197276b61f414e5b28124665da",
        size: 516_458,
        role: "reference_model",
        site: "Bull Run",
        date: None,
        substation: Some("108137138"),
    },
    Pin {
        path: "data_original/gmu/tva/site_708931360.csv",
        blob_sha1: "ad8b0d090ae6dddcad7b05326501cb3a81cb3d86",
        size: 522_237,
        role: "reference_model",
        site: "Montgomery",
        date: None,
        substation: Some("708931360"),
    },
    Pin {
        path: "data_original/gmu/tva/site_98699542.csv",
        blob_sha1: "17fd6bcd6dda7bf0aef4239c69f0d0fc2d9a447e",
        size: 497_404,
        role: "reference_model",
        site: "Union",
        date: None,
        substation: Some("98699542"),
    },
    Pin {
        path: "data_original/gmu/tva/site_14133615.csv",
        blob_sha1: "e28d2125031a7eaeb2467e102e4554c658bb6ff5",
        size: 530_310,
        role: "reference_model",
        site: "Widows Creek",
        date: None,
        substation: Some("14133615"),
    },
    Pin {
        path: "data_original/tva/gic/GIC-calculated/20240510_BullRunXfrmGIC.dat",
        blob_sha1: "562f690f3e2849fa24e85b2bcf733c3e69832e30",
        size: 21_743,
        role: "tva_operator",
        site: "Bull Run",
        date: Some("20240510"),
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-calculated/20240510_MontgomeryGIC.dat",
        blob_sha1: "84690f81b28978d61d6afd8b5b97060c3869aaa3",
        size: 21_798,
        role: "tva_operator",
        site: "Montgomery",
        date: Some("20240510"),
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-calculated/20240510_UnionGIC.dat",
        blob_sha1: "01bcc3035eb9805749922f6fd9120af54b79e409",
        size: 22_286,
        role: "tva_operator",
        site: "Union",
        date: Some("20240510"),
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-calculated/20240510_WidowsCreek2GIC.dat",
        blob_sha1: "6e2ef9e1d0d6aaf56f866e8fbfb778cd578b3902",
        size: 21_747,
        role: "tva_operator",
        site: "Widows Creek",
        date: Some("20240510"),
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-calculated/20240511_BullRunXfrmGIC.dat",
        blob_sha1: "74d88e8e0ae4827c2664e853441a4ee342934c0c",
        size: 22_008,
        role: "tva_operator",
        site: "Bull Run",
        date: Some("20240511"),
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-calculated/20240511_MontgomeryGIC.dat",
        blob_sha1: "75db46ce3635afe5916a2bdd3a1302b09c996190",
        size: 21_959,
        role: "tva_operator",
        site: "Montgomery",
        date: Some("20240511"),
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-calculated/20240511_UnionGIC.dat",
        blob_sha1: "4e129587c1d677182804e6a4c6a54eb838f1460d",
        size: 21_967,
        role: "tva_operator",
        site: "Union",
        date: Some("20240511"),
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-calculated/20240511_WidowsCreek2GIC.dat",
        blob_sha1: "05a671a0923ad9ee75d8e6eb98de16938ee15bca",
        size: 22_000,
        role: "tva_operator",
        site: "Widows Creek",
        date: Some("20240511"),
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-calculated/20240512_BullRunXfrmGIC.dat",
        blob_sha1: "73a8cca21d844d42c944c3e366eb920c79457d92",
        size: 22_093,
        role: "tva_operator",
        site: "Bull Run",
        date: Some("20240512"),
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-calculated/20240512_MontgomeryGIC.dat",
        blob_sha1: "3d20085f7945eafbc428b3495329de6ffc0596d4",
        size: 22_075,
        role: "tva_operator",
        site: "Montgomery",
        date: Some("20240512"),
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-calculated/20240512_UnionGIC.dat",
        blob_sha1: "6650d1cb6987da7526045fb9acfc57936dc0d2bd",
        size: 21_898,
        role: "tva_operator",
        site: "Union",
        date: Some("20240512"),
        substation: None,
    },
    Pin {
        path: "data_original/tva/gic/GIC-calculated/20240512_WidowsCreek2GIC.dat",
        blob_sha1: "c861343ae8cf02d5a534dea7aabfe5fac1cff597",
        size: 22_152,
        role: "tva_operator",
        site: "Widows Creek",
        date: Some("20240512"),
        substation: None,
    },
];

const SITES: [&str; 4] = ["Bull Run", "Montgomery", "Union", "Widows Creek"];

#[derive(Clone, Copy)]
struct KnownAnswer {
    site: &'static str,
    model: &'static str,
    r2: f64,
    pe: f64,
}

const KNOWN_ANSWERS: [KnownAnswer; 8] = [
    KnownAnswer {
        site: "Bull Run",
        model: "TVA",
        r2: 0.7993592783300696,
        pe: 0.49794711058427144,
    },
    KnownAnswer {
        site: "Montgomery",
        model: "TVA",
        r2: 0.8637185602798242,
        pe: 0.42977033697187983,
    },
    KnownAnswer {
        site: "Union",
        model: "TVA",
        r2: 0.6688164049618603,
        pe: 0.651510319908838,
    },
    KnownAnswer {
        site: "Widows Creek",
        model: "TVA",
        r2: 0.6708368563169974,
        pe: 0.6314735353459772,
    },
    KnownAnswer {
        site: "Bull Run",
        model: "GMU",
        r2: 0.4802353334557028,
        pe: 0.47525322259760483,
    },
    KnownAnswer {
        site: "Montgomery",
        model: "GMU",
        r2: 0.1537256123621483,
        pe: 0.15202585354717357,
    },
    KnownAnswer {
        site: "Union",
        model: "GMU",
        r2: 0.5691151888165665,
        pe: 0.4980835146131537,
    },
    KnownAnswer {
        site: "Widows Creek",
        model: "GMU",
        r2: 0.2790051141346092,
        pe: 0.22716776534074634,
    },
];

#[derive(Debug)]
struct Metrics {
    raw_r: f64,
    r2: f64,
    pe: f64,
    rmse: f64,
    mae: f64,
    observed_sigma: f64,
    predicted_sigma: f64,
    polarity_correction: bool,
}

#[derive(Debug)]
struct Comparison {
    site: &'static str,
    model: &'static str,
    metrics: Metrics,
    known_r2: f64,
    known_pe: f64,
    pass_r2: bool,
    pass_pe: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("gmd-four-site-replay: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let Some(root_arg) = args.next() else {
        return Err("usage: gmd-four-site-replay <path-to-pinned-SWERVE-checkout>".to_owned());
    };
    if args.next().is_some() {
        return Err("exactly one dataset checkout path is required".to_owned());
    }
    let source_root = PathBuf::from(root_arg);
    if !source_root.is_dir() {
        return Err(format!(
            "dataset root is not a directory: {}",
            source_root.display()
        ));
    }

    verify_manifest_constants()?;
    let source_head = git_output(&source_root, &["rev-parse", "HEAD"])?;
    if source_head != DATASET_COMMIT {
        return Err(format!(
            "dataset HEAD mismatch: expected {DATASET_COMMIT}, found {source_head}; checkout the pinned commit, not a mutable branch"
        ));
    }

    let tool_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| {
            "could not locate Sol-Atlas repository root from Cargo manifest".to_owned()
        })?;
    let tool_commit = git_output(tool_root, &["rev-parse", "HEAD"])?;
    let clean = Command::new("git")
        .arg("-C")
        .arg(tool_root)
        .args([
            "diff",
            "--quiet",
            "HEAD",
            "--",
            "tools/gmd-four-site-replay",
        ])
        .status()
        .map_err(|error| format!("cannot inspect tool source identity with git: {error}"))?;
    if !clean.success() {
        return Err("tool source tree is modified relative to HEAD; commit the verifier before producing a qualified receipt".to_owned());
    }

    let source_files = verify_source_files(&source_root)?;
    let mut comparisons = Vec::with_capacity(8);

    for site in SITES {
        let measured_pin = find_pin("measured_1s", site, None)?;
        let measured_bytes = source_files
            .get(measured_pin.path)
            .ok_or_else(|| format!("verified source missing from memory: {}", measured_pin.path))?;
        let measured_seconds = parse_measured_csv(measured_bytes, measured_pin.path)?;
        let observed = aggregate_measured(&measured_seconds)?;

        let mut tva_minutes = BTreeMap::new();
        for pin in PINS
            .iter()
            .filter(|pin| pin.role == "tva_operator" && pin.site == site)
        {
            let bytes = source_files
                .get(pin.path)
                .ok_or_else(|| format!("verified source missing from memory: {}", pin.path))?;
            let daily = parse_tva_daily(bytes, pin)?;
            for (minute, value) in daily {
                if tva_minutes.insert(minute, value).is_some() {
                    return Err(format!(
                        "duplicate TVA prediction minute for {site}: {minute}"
                    ));
                }
            }
        }

        let reference_pin = find_pin("reference_model", site, None)?;
        let reference_bytes = source_files.get(reference_pin.path).ok_or_else(|| {
            format!(
                "verified source missing from memory: {}",
                reference_pin.path
            )
        })?;
        let reference_minutes = parse_reference_csv(reference_bytes, reference_pin)?;

        comparisons.push(compare(site, "TVA", &observed, &tva_minutes)?);
        comparisons.push(compare(site, "GMU", &observed, &reference_minutes)?);
    }

    let all_pass = comparisons.iter().all(|item| item.pass_r2 && item.pass_pe);
    let receipt = render_receipt(
        &source_head,
        &tool_commit,
        &source_files,
        &comparisons,
        all_pass,
    );
    print!("{receipt}");
    if !all_pass {
        return Err("one or more publisher-known-answer checks failed; receipt was emitted for diagnosis but is not a PASS".to_owned());
    }
    Ok(())
}

fn find_pin(role: &str, site: &str, date: Option<&str>) -> Result<&'static Pin, String> {
    PINS.iter()
        .find(|pin| pin.role == role && pin.site == site && pin.date == date)
        .ok_or_else(|| format!("no pinned source for role={role}, site={site}, date={date:?}"))
}

fn verify_manifest_constants() -> Result<(), String> {
    if !MANIFEST.contains(DATASET_COMMIT) {
        return Err(
            "embedded benchmark manifest does not contain the pinned dataset commit".to_owned(),
        );
    }
    for pin in &PINS {
        let needle = format!("\"path\": \"{}\"", pin.path);
        let start = MANIFEST
            .find(&needle)
            .ok_or_else(|| format!("source path is absent from embedded manifest: {}", pin.path))?;
        let end = MANIFEST[start..]
            .find('}')
            .map(|offset| start + offset)
            .ok_or_else(|| format!("unterminated source object for {}", pin.path))?;
        let object = &MANIFEST[start..=end];
        if !object.contains(pin.blob_sha1) || !object.contains(&pin.size.to_string()) {
            return Err(format!(
                "embedded source pin disagrees with benchmark manifest: {}",
                pin.path
            ));
        }
    }
    Ok(())
}

fn verify_source_files(root: &Path) -> Result<BTreeMap<&'static str, Vec<u8>>, String> {
    let mut verified = BTreeMap::new();
    for pin in &PINS {
        let path = root.join(pin.path);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("cannot stat pinned input {}: {error}", pin.path))?;
        if !metadata.file_type().is_file() {
            return Err(format!(
                "pinned input is not a regular file (symlink/directory rejected): {}",
                pin.path
            ));
        }
        let bytes = fs::read(&path)
            .map_err(|error| format!("cannot read pinned input {}: {error}", pin.path))?;
        if bytes.len() != pin.size {
            return Err(format!(
                "size mismatch for {}: expected {} bytes, found {}",
                pin.path,
                pin.size,
                bytes.len()
            ));
        }
        let actual = git_blob_sha1(&bytes);
        if actual != pin.blob_sha1 {
            return Err(format!(
                "Git blob SHA-1 mismatch for {}: expected {}, found {}",
                pin.path, pin.blob_sha1, actual
            ));
        }
        verified.insert(pin.path, bytes);
    }
    Ok(verified)
}

fn parse_measured_csv(bytes: &[u8], label: &str) -> Result<BTreeMap<i64, f64>, String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| format!("{label}: input is not UTF-8: {error}"))?;
    let mut samples = BTreeMap::new();
    let mut skipped_header = false;
    let mut previous_timestamp: Option<i64> = None;

    for (line_number, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_csv_record(line)
            .map_err(|error| format!("{label}: line {}: {error}", line_number + 1))?;
        let single_field_timestamps: Vec<(usize, i64)> = fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| {
                parse_timestamp(field)
                    .ok()
                    .map(|timestamp| (index, timestamp))
            })
            .collect();

        if single_field_timestamps.is_empty() && !skipped_header {
            skipped_header = true;
            continue;
        }

        let (timestamp_indices, timestamp) = if single_field_timestamps.len() == 1 {
            let (index, timestamp) = single_field_timestamps[0];
            (vec![index], timestamp)
        } else if single_field_timestamps.is_empty() {
            let mut split_candidates = Vec::new();
            for index in 0..fields.len().saturating_sub(1) {
                let combined = format!("{} {}", fields[index], fields[index + 1]);
                if let Ok(timestamp) = parse_timestamp(&combined) {
                    split_candidates.push((vec![index, index + 1], timestamp));
                }
            }
            if split_candidates.len() != 1 {
                return Err(format!(
                    "{label}: line {} must contain exactly one parseable UTC timestamp (single field or adjacent date/time fields)",
                    line_number + 1
                ));
            }
            split_candidates.remove(0)
        } else {
            return Err(format!(
                "{label}: line {} contains multiple timestamp fields",
                line_number + 1
            ));
        };

        let numeric: Vec<(usize, f64)> = fields
            .iter()
            .enumerate()
            .filter(|(index, _)| !timestamp_indices.contains(index))
            .filter_map(|(index, field)| parse_finite(field).ok().map(|value| (index, value)))
            .collect();
        if numeric.len() != 1 {
            return Err(format!(
                "{label}: line {} must contain exactly one current value beside its timestamp",
                line_number + 1
            ));
        }

        if let Some(previous) = previous_timestamp {
            if timestamp == previous {
                return Err(format!(
                    "{label}: duplicate one-second timestamp at line {}",
                    line_number + 1
                ));
            }
            if timestamp < previous {
                return Err(format!(
                    "{label}: timestamps are not strictly increasing at line {}",
                    line_number + 1
                ));
            }
        }
        previous_timestamp = Some(timestamp);
        samples.insert(timestamp, numeric[0].1);
    }

    if samples.is_empty() {
        return Err(format!("{label}: no measured samples were parsed"));
    }
    Ok(samples)
}

fn parse_tva_daily(bytes: &[u8], pin: &Pin) -> Result<BTreeMap<i64, f64>, String> {
    let date = pin
        .date
        .ok_or_else(|| format!("TVA pin missing date: {}", pin.path))?;
    let midnight = parse_compact_date(date)?;
    let text =
        std::str::from_utf8(bytes).map_err(|error| format!("{}: not UTF-8: {error}", pin.path))?;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header_line = lines
        .next()
        .ok_or_else(|| format!("{}: empty TVA daily file", pin.path))?;
    let headers = parse_csv_record(header_line)?;
    let amp_index = unique_header_index(&headers, "GICXFNeutralAmps", pin.path)?;
    let time_index = unique_header_index(&headers, "time", pin.path)?;
    let mut values = BTreeMap::new();
    let mut previous_minute: Option<i64> = None;

    for (offset, line) in lines.enumerate() {
        let fields = parse_csv_record(line)
            .map_err(|error| format!("{}: data row {}: {error}", pin.path, offset + 2))?;
        if fields.len() != headers.len() {
            return Err(format!(
                "{}: data row {} has {} fields, expected {}",
                pin.path,
                offset + 2,
                fields.len(),
                headers.len()
            ));
        }
        let amps = parse_finite(&fields[amp_index])
            .map_err(|error| format!("{}: data row {}: {error}", pin.path, offset + 2))?;
        let seconds_float = parse_finite(&fields[time_index])
            .map_err(|error| format!("{}: data row {}: {error}", pin.path, offset + 2))?;
        let seconds_rounded = seconds_float.round();
        if (seconds_float - seconds_rounded).abs() > 1e-9 {
            return Err(format!(
                "{}: elapsed seconds must be an integer at row {}",
                pin.path,
                offset + 2
            ));
        }
        let seconds = seconds_rounded as i64;
        if !(0..86_400).contains(&seconds) || seconds % 60 != 0 {
            return Err(format!(
                "{}: elapsed seconds must be a minute boundary in [0,86400) at row {}",
                pin.path,
                offset + 2
            ));
        }
        let minute = midnight + seconds;
        if let Some(previous) = previous_minute {
            if minute == previous {
                return Err(format!(
                    "{}: duplicate minute at row {}",
                    pin.path,
                    offset + 2
                ));
            }
            if minute < previous {
                return Err(format!(
                    "{}: minutes are not strictly increasing at row {}",
                    pin.path,
                    offset + 2
                ));
            }
        }
        previous_minute = Some(minute);
        values.insert(minute, amps);
    }

    if values.is_empty() {
        return Err(format!("{}: no TVA predictions were parsed", pin.path));
    }
    Ok(values)
}

fn parse_reference_csv(bytes: &[u8], pin: &Pin) -> Result<BTreeMap<i64, f64>, String> {
    let text =
        std::str::from_utf8(bytes).map_err(|error| format!("{}: not UTF-8: {error}", pin.path))?;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header_line = lines
        .next()
        .ok_or_else(|| format!("{}: empty reference-model file", pin.path))?;
    let headers = parse_csv_record(header_line)?;
    let timestamp_index = unique_header_index(&headers, "timestamp", pin.path)?;
    let substation_index = unique_header_index(&headers, "substation", pin.path)?;
    let gic_index = unique_header_index(&headers, "Sim GIC (Median)", pin.path)?;
    let expected_substation = pin
        .substation
        .ok_or_else(|| format!("{}: missing expected substation ID", pin.path))?;
    let mut values = BTreeMap::new();
    let mut previous_timestamp: Option<i64> = None;

    for (offset, line) in lines.enumerate() {
        let fields = parse_csv_record(line)
            .map_err(|error| format!("{}: data row {}: {error}", pin.path, offset + 2))?;
        if fields.len() != headers.len() {
            return Err(format!(
                "{}: data row {} has {} fields, expected {}",
                pin.path,
                offset + 2,
                fields.len(),
                headers.len()
            ));
        }
        if fields[substation_index].trim() != expected_substation {
            return Err(format!(
                "{}: substation identity mismatch at row {}: expected {}, found {}",
                pin.path,
                offset + 2,
                expected_substation,
                fields[substation_index]
            ));
        }
        let timestamp = parse_timestamp(&fields[timestamp_index])
            .map_err(|error| format!("{}: timestamp at row {}: {error}", pin.path, offset + 2))?;
        if timestamp.rem_euclid(60) != 0 {
            return Err(format!(
                "{}: reference prediction is not aligned to a UTC minute at row {}",
                pin.path,
                offset + 2
            ));
        }
        let amps = parse_finite(&fields[gic_index])
            .map_err(|error| format!("{}: GIC at row {}: {error}", pin.path, offset + 2))?;
        if let Some(previous) = previous_timestamp {
            if timestamp == previous {
                return Err(format!(
                    "{}: duplicate reference-model minute at row {}",
                    pin.path,
                    offset + 2
                ));
            }
            if timestamp < previous {
                return Err(format!(
                    "{}: reference-model timestamps are not strictly increasing at row {}",
                    pin.path,
                    offset + 2
                ));
            }
        }
        previous_timestamp = Some(timestamp);
        values.insert(timestamp, amps);
    }

    if values.is_empty() {
        return Err(format!(
            "{}: no reference predictions were parsed",
            pin.path
        ));
    }
    Ok(values)
}

fn unique_header_index(headers: &[String], expected: &str, label: &str) -> Result<usize, String> {
    let matches: Vec<usize> = headers
        .iter()
        .enumerate()
        .filter(|(_, header)| header.trim().eq_ignore_ascii_case(expected))
        .map(|(index, _)| index)
        .collect();
    if matches.len() != 1 {
        return Err(format!(
            "{label}: expected exactly one header named {expected:?}, found {}",
            matches.len()
        ));
    }
    Ok(matches[0])
}

fn parse_csv_record(line: &str) -> Result<Vec<String>, String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut chars = line.trim_end_matches('\r').chars().peekable();
    let mut quoted = false;
    let mut after_quote = false;

    while let Some(ch) = chars.next() {
        if quoted {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                    after_quote = true;
                }
            } else {
                field.push(ch);
            }
            continue;
        }

        if after_quote {
            if ch == ',' {
                fields.push(field.trim().to_owned());
                field.clear();
                after_quote = false;
            } else if !ch.is_whitespace() {
                return Err("unexpected character after closing CSV quote".to_owned());
            }
            continue;
        }

        match ch {
            ',' => {
                fields.push(field.trim().to_owned());
                field.clear();
            }
            '"' if field.trim().is_empty() => {
                field.clear();
                quoted = true;
            }
            '"' => return Err("quote inside an unquoted CSV field".to_owned()),
            _ => field.push(ch),
        }
    }

    if quoted {
        return Err("unterminated CSV quoted field".to_owned());
    }
    fields.push(field.trim().to_owned());
    Ok(fields)
}

fn parse_finite(value: &str) -> Result<f64, String> {
    let parsed = value
        .trim()
        .parse::<f64>()
        .map_err(|error| format!("invalid numeric value {value:?}: {error}"))?;
    if !parsed.is_finite() {
        return Err(format!("non-finite numeric value is prohibited: {value:?}"));
    }
    Ok(parsed)
}

fn parse_timestamp(value: &str) -> Result<i64, String> {
    let mut raw = value.trim().trim_matches('"').to_owned();
    if raw.ends_with(" UTC") {
        raw.truncate(raw.len() - 4);
        raw = raw.trim_end().to_owned();
    }

    if raw.ends_with('Z') || raw.ends_with('z') {
        raw.pop();
    } else if let Some(position) = raw.rfind(|ch| ch == '+' || ch == '-') {
        if position > 10 {
            let offset = &raw[position..];
            if !matches!(
                offset,
                "+00:00" | "-00:00" | "+0000" | "-0000" | "+00" | "-00"
            ) {
                return Err(format!("non-UTC timezone offset is not allowed: {offset}"));
            }
            raw.truncate(position);
        }
    }

    let (date_part, time_part) = if let Some((date, time)) = raw.split_once('T') {
        (date, time)
    } else if let Some((date, time)) = raw.split_once(' ') {
        (date, time)
    } else {
        return Err(format!("timestamp must contain a date and time: {value:?}"));
    };

    let (year, month, day) = parse_date(date_part)?;
    let (hour, minute, second) = parse_clock(time_part)?;
    Ok(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn parse_date(value: &str) -> Result<(i64, i64, i64), String> {
    let parts: Vec<&str> = if value.contains('-') {
        value.split('-').collect()
    } else if value.contains('/') {
        value.split('/').collect()
    } else {
        return Err(format!("unsupported date form: {value:?}"));
    };
    if parts.len() != 3 {
        return Err(format!("invalid date fields: {value:?}"));
    }

    let (year, month, day) = if parts[0].len() == 4 {
        (
            parse_i64(parts[0])?,
            parse_i64(parts[1])?,
            parse_i64(parts[2])?,
        )
    } else if value.contains('/') && parts[2].len() == 4 {
        (
            parse_i64(parts[2])?,
            parse_i64(parts[0])?,
            parse_i64(parts[1])?,
        )
    } else {
        return Err(format!("date must be YYYY-MM-DD or MM/DD/YYYY: {value:?}"));
    };

    if !(1..=12).contains(&month) || year < 1 {
        return Err(format!("invalid calendar date: {value:?}"));
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => unreachable!(),
    };
    if !(1..=days_in_month).contains(&day) {
        return Err(format!("invalid day in calendar date: {value:?}"));
    }
    Ok((year, month, day))
}

fn parse_clock(value: &str) -> Result<(i64, i64, i64), String> {
    let mut clock = value.trim().to_ascii_uppercase();
    let am = clock.ends_with(" AM");
    let pm = clock.ends_with(" PM");
    if am || pm {
        clock.truncate(clock.len() - 3);
        clock = clock.trim_end().to_owned();
    }

    let parts: Vec<&str> = clock.split(':').collect();
    if !(2..=3).contains(&parts.len()) {
        return Err(format!("invalid clock time: {value:?}"));
    }
    let hour = parse_i64(parts[0])?;
    let minute = parse_i64(parts[1])?;
    let seconds_text = if parts.len() == 3 { parts[2] } else { "0" };
    let whole_seconds = seconds_text.split('.').next().unwrap_or(seconds_text);
    let fraction = seconds_text.split_once('.').map(|(_, fraction)| fraction);
    if fraction.is_some_and(|digits| digits.is_empty() || digits.chars().any(|digit| digit != '0'))
    {
        return Err(format!(
            "sub-second timestamps are not supported: {value:?}"
        ));
    }
    let second = parse_i64(whole_seconds)?;

    let hour = if am || pm {
        if !(1..=12).contains(&hour) {
            return Err(format!("invalid 12-hour clock value: {value:?}"));
        }
        (hour % 12) + if pm { 12 } else { 0 }
    } else {
        hour
    };

    if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) || !(0..=59).contains(&second) {
        return Err(format!("clock component out of range: {value:?}"));
    }
    Ok((hour, minute, second))
}

fn parse_i64(value: &str) -> Result<i64, String> {
    value
        .trim()
        .parse::<i64>()
        .map_err(|error| format!("invalid integer {value:?}: {error}"))
}

fn parse_compact_date(value: &str) -> Result<i64, String> {
    if value.len() != 8 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("invalid YYYYMMDD date in source pin: {value}"));
    }
    let year = parse_i64(&value[0..4])?;
    let month = parse_i64(&value[4..6])?;
    let day = parse_i64(&value[6..8])?;
    let iso = format!("{year:04}-{month:02}-{day:02}");
    let (year, month, day) = parse_date(&iso)?;
    Ok(days_from_civil(year, month, day) * 86_400)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let adjusted_year = if month <= 2 { year - 1 } else { year };
    let era = if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    } / 400;
    let year_of_era = adjusted_year - era * 400;
    let adjusted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn aggregate_measured(samples: &BTreeMap<i64, f64>) -> Result<Vec<(i64, f64)>, String> {
    let start = days_from_civil(2024, 5, 10) * 86_400 + 15 * 3_600;
    let end = days_from_civil(2024, 5, 12) * 86_400 + 6 * 3_600;
    let mut bins: BTreeMap<i64, Vec<f64>> = BTreeMap::new();

    for (&timestamp, &value) in samples {
        if (start..end + 60).contains(&timestamp) {
            let minute = timestamp.div_euclid(60) * 60;
            bins.entry(minute).or_default().push(value);
        }
    }

    let mut result = Vec::with_capacity(EXPECTED_PAIRS);
    let mut minute = start;
    while minute <= end {
        let values = bins
            .get(&minute)
            .ok_or_else(|| format!("missing measured minute bin at UTC epoch {minute}"))?;
        if values.len() != 60 {
            return Err(format!(
                "minute {minute} contains {} measured samples; exactly 60 are required",
                values.len()
            ));
        }
        result.push((minute, values.iter().sum::<f64>() / 60.0));
        minute += 60;
    }

    if result.len() != EXPECTED_PAIRS {
        return Err(format!(
            "internal evaluation-window error: expected {EXPECTED_PAIRS} bins, built {}",
            result.len()
        ));
    }
    Ok(result)
}

fn compare(
    site: &'static str,
    model: &'static str,
    observed: &[(i64, f64)],
    predicted: &BTreeMap<i64, f64>,
) -> Result<Comparison, String> {
    let mut obs = Vec::with_capacity(EXPECTED_PAIRS);
    let mut pred = Vec::with_capacity(EXPECTED_PAIRS);
    for &(minute, measured) in observed {
        let value = predicted.get(&minute).ok_or_else(|| {
            format!("{model} prediction missing exact UTC minute {minute} at {site}")
        })?;
        obs.push(measured);
        pred.push(*value);
    }
    if obs.len() != EXPECTED_PAIRS {
        return Err(format!(
            "{site}/{model}: expected {EXPECTED_PAIRS} exact pairs, found {}",
            obs.len()
        ));
    }

    let metrics = calculate_metrics(&obs, &pred)?;
    let known = KNOWN_ANSWERS
        .iter()
        .find(|answer| answer.site == site && answer.model == model)
        .ok_or_else(|| format!("no known-answer row for {site}/{model}"))?;
    let pass_r2 = (metrics.r2 - known.r2).abs() <= 0.005_000_1;
    let pass_pe = (metrics.pe - known.pe).abs() <= 0.005_000_1;

    Ok(Comparison {
        site,
        model,
        metrics,
        known_r2: known.r2,
        known_pe: known.pe,
        pass_r2,
        pass_pe,
    })
}

fn calculate_metrics(observed: &[f64], predicted: &[f64]) -> Result<Metrics, String> {
    if observed.len() != predicted.len() || observed.len() < 2 {
        return Err("metric inputs must have equal length and at least two pairs".to_owned());
    }
    let n = observed.len() as f64;
    let mean_observed = observed.iter().sum::<f64>() / n;
    let mean_predicted = predicted.iter().sum::<f64>() / n;
    let mut covariance = 0.0;
    let mut observed_ss = 0.0;
    let mut predicted_ss = 0.0;

    for (&actual, &forecast) in observed.iter().zip(predicted) {
        let x = actual - mean_observed;
        let y = forecast - mean_predicted;
        covariance += x * y;
        observed_ss += x * x;
        predicted_ss += y * y;
    }
    if observed_ss <= f64::EPSILON || predicted_ss <= f64::EPSILON {
        return Err("Pearson correlation is undefined for a constant series".to_owned());
    }

    let raw_r = covariance / (observed_ss * predicted_ss).sqrt();
    let polarity_correction = raw_r < 0.0;
    let orientation = if polarity_correction { -1.0 } else { 1.0 };
    let mut squared_error = 0.0;
    let mut absolute_error = 0.0;

    for (&actual, &forecast) in observed.iter().zip(predicted) {
        let error = actual - orientation * forecast;
        squared_error += error * error;
        absolute_error += error.abs();
    }

    let pe = 1.0 - squared_error / observed_ss;
    Ok(Metrics {
        raw_r,
        r2: raw_r * raw_r,
        pe,
        rmse: (squared_error / n).sqrt(),
        mae: absolute_error / n,
        observed_sigma: (observed_ss / n).sqrt(),
        predicted_sigma: (predicted_ss / n).sqrt(),
        polarity_correction,
    })
}

fn render_receipt(
    source_head: &str,
    tool_commit: &str,
    source_files: &BTreeMap<&'static str, Vec<u8>>,
    comparisons: &[Comparison],
    all_pass: bool,
) -> String {
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str("  \"schema\": \"gmd-four-site-replay-receipt-v0.1\",\n");
    output.push_str("  \"benchmark_id\": \"GMD-MAY2024-FOUR-SITE-REPLAY-V0_1\",\n");
    output.push_str("  \"evidence_tier\": \"E1_DATA_PIPELINE_KNOWN_ANSWER\",\n");
    output.push_str(&format!(
        "  \"dataset_commit\": {},\n",
        json_string(source_head)
    ));
    output.push_str(&format!(
        "  \"tool_source_commit\": {},\n",
        json_string(tool_commit)
    ));
    output.push_str("  \"runtime_network_access\": false,\n");
    output.push_str("  \"evaluation\": {\n");
    output.push_str("    \"start_utc\": \"2024-05-10T15:00:00Z\",\n");
    output.push_str("    \"end_utc_inclusive\": \"2024-05-12T06:00:00Z\",\n");
    output.push_str("    \"expected_pairs_per_site_model\": 2341,\n");
    output.push_str(
        "    \"measured_aggregation\": \"exact_mean_of_60_one_second_samples_per_utc_minute\",\n",
    );
    output.push_str("    \"join\": \"exact_utc_minute_only_no_interpolation_no_forward_fill\",\n");
    output.push_str("    \"timezone_policy\": \"naive source timestamps are interpreted as UTC; explicit non-UTC offsets are rejected\",\n");
    output.push_str("    \"polarity\": \"preserve_raw_pearson_r_and_invert_prediction_only_for_residual_metrics_when_r_is_negative\"\n");
    output.push_str("  },\n");
    output.push_str("  \"verified_source_files\": [\n");
    for (index, pin) in PINS.iter().enumerate() {
        let comma = if index + 1 == PINS.len() { "" } else { "," };
        let actual_size = source_files.get(pin.path).map_or(0, Vec::len);
        output.push_str(&format!(
            "    {{\"path\":{},\"git_blob_sha1\":{},\"size_bytes\":{}}}{}\n",
            json_string(pin.path),
            json_string(pin.blob_sha1),
            actual_size,
            comma
        ));
    }
    output.push_str("  ],\n");
    output.push_str("  \"comparisons\": [\n");
    for (index, item) in comparisons.iter().enumerate() {
        let comma = if index + 1 == comparisons.len() {
            ""
        } else {
            ","
        };
        let m = &item.metrics;
        output.push_str(&format!(
            concat!(
                "    {{\"site\":{},\"model\":{},\"matched_minutes\":2341,",
                "\"raw_pearson_r\":{},\"r_squared\":{},\"prediction_efficiency\":{},",
                "\"rmse_A\":{},\"mae_A\":{},\"observed_sigma_A\":{},\"predicted_sigma_A\":{},",
                "\"polarity_correction_applied\":{},\"known_answer_r_squared\":{},",
                "\"known_answer_prediction_efficiency\":{},\"r_squared_pass\":{},",
                "\"prediction_efficiency_pass\":{},\"pass\":{}}}{}\n"
            ),
            json_string(item.site),
            json_string(item.model),
            number(m.raw_r),
            number(m.r2),
            number(m.pe),
            number(m.rmse),
            number(m.mae),
            number(m.observed_sigma),
            number(m.predicted_sigma),
            m.polarity_correction,
            number(item.known_r2),
            number(item.known_pe),
            item.pass_r2,
            item.pass_pe,
            item.pass_r2 && item.pass_pe,
            comma
        ));
    }
    output.push_str("  ],\n");
    output.push_str(&format!("  \"all_known_answers_pass\": {all_pass},\n"));
    output.push_str("  \"claim_ceiling\": \"reproduces a published data pipeline and metric table only; does not validate any physical model, transformer response, grid security, operational resilience, compliance, or safety\"\n");
    output.push_str("}\n");
    output
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn number(value: f64) -> String {
    if value.is_finite() {
        value.to_string()
    } else {
        "null".to_owned()
    }
}

fn git_output(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| format!("could not run git in {}: {error}", root.display()))?;
    if !output.status.success() {
        return Err(format!(
            "git {:?} failed in {}: {}",
            args,
            root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| format!("git output is not UTF-8: {error}"))
}

fn git_blob_sha1(bytes: &[u8]) -> String {
    let header = format!("blob {}\0", bytes.len());
    let mut git_object = Vec::with_capacity(header.len() + bytes.len());
    git_object.extend_from_slice(header.as_bytes());
    git_object.extend_from_slice(bytes);
    let digest = sha1(&git_object);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sha1(input: &[u8]) -> [u8; 20] {
    let mut message = input.to_vec();
    let bit_length = (message.len() as u64).wrapping_mul(8);
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_length.to_be_bytes());

    let mut h0 = 0x6745_2301_u32;
    let mut h1 = 0xEFCD_AB89_u32;
    let mut h2 = 0x98BA_DCFE_u32;
    let mut h3 = 0x1032_5476_u32;
    let mut h4 = 0xC3D2_E1F0_u32;

    for chunk in message.chunks_exact(64) {
        let mut words = [0_u32; 80];
        for (index, word) in words.iter_mut().take(16).enumerate() {
            let start = index * 4;
            *word = u32::from_be_bytes([
                chunk[start],
                chunk[start + 1],
                chunk[start + 2],
                chunk[start + 3],
            ]);
        }
        for index in 16..80 {
            words[index] =
                (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16])
                    .rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for (index, word) in words.iter().enumerate() {
            let (function, constant) = match index {
                0..=19 => ((b & c) | ((!b) & d), 0x5A82_7999),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(function)
                .wrapping_add(e)
                .wrapping_add(constant)
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

    let mut digest = [0_u8; 20];
    for (index, word) in [h0, h1, h2, h3, h4].iter().enumerate() {
        digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

fn average_exact_minute(samples: &BTreeMap<i64, f64>, minute: i64) -> Result<f64, String> {
    let mut sum = 0.0;
    for offset in 0..60 {
        let timestamp = minute + offset;
        let value = samples
            .get(&timestamp)
            .ok_or_else(|| format!("missing second {timestamp} in UTC minute {minute}"))?;
        sum += value;
    }
    Ok(sum / 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_blob_sha1_matches_known_git_objects() {
        assert_eq!(
            git_blob_sha1(b""),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
        assert_eq!(
            git_blob_sha1(b"hello\n"),
            "ce013625030ba8dba906f756967f9e9ca394464a"
        );
    }

    #[test]
    fn parses_utc_iso_and_slash_dates() {
        assert_eq!(
            parse_timestamp("2024-05-10 15:00:00").unwrap(),
            parse_timestamp("2024-05-10T15:00:00Z").unwrap()
        );
        assert_eq!(
            parse_timestamp("05/10/2024 11:00:00 AM").unwrap(),
            parse_timestamp("2024-05-10T11:00:00+00:00").unwrap()
        );
    }

    #[test]
    fn parses_split_date_and_time_columns_in_measurement_rows() {
        let csv = b"date,time,current\n2024-05-10,15:00:00,1.25\n";
        let samples = parse_measured_csv(csv, "split.csv").unwrap();
        let expected = days_from_civil(2024, 5, 10) * 86_400 + 15 * 3_600;
        assert_eq!(samples.get(&expected), Some(&1.25));
    }

    #[test]
    fn rejects_ambiguous_or_extra_numeric_measurement_columns() {
        let ambiguous = b"timestamp,current\n2024-05-10 15:00:00,1.0\n2024-05-10 15:00:01,2.0\n";
        assert!(parse_measured_csv(ambiguous, "ambiguous.csv").is_ok());

        let extra = b"timestamp,current,quality_value\n2024-05-10 15:00:00,1.0,2.0\n";
        let error = parse_measured_csv(extra, "extra.csv").unwrap_err();
        assert!(error.contains("exactly one current value"));
    }

    #[test]
    fn validates_leap_dates_and_rejects_bad_calendar_values() {
        assert!(parse_timestamp("2000-02-29 00:00:00").is_ok());
        assert!(parse_timestamp("1900-02-29 00:00:00").is_err());
        assert!(parse_timestamp("2024-02-30 00:00:00").is_err());
        assert!(parse_timestamp("2023-02-29 00:00:00").is_err());
    }

    #[test]
    fn rejects_non_utc_offsets_and_fractional_samples() {
        assert!(parse_timestamp("2024-05-10T15:00:00+02:00").is_err());
        assert!(parse_timestamp("2024-05-10T15:00:00.250Z").is_err());
    }

    #[test]
    fn parses_quoted_csv_fields_and_escaped_quotes() {
        let fields = parse_csv_record("\"timestamp\",\"substation\",\"Sim GIC (Median)\"").unwrap();
        assert_eq!(fields, vec!["timestamp", "substation", "Sim GIC (Median)"]);
        let fields = parse_csv_record("\"a\"\"b\",2").unwrap();
        assert_eq!(fields, vec!["a\"b", "2"]);
    }

    #[test]
    fn one_complete_minute_averages_all_sixty_samples() {
        let start = days_from_civil(2024, 5, 10) * 86_400 + 15 * 3_600;
        let mut samples = BTreeMap::new();
        for second in 0..60 {
            samples.insert(start + second, 2.0);
        }
        assert!((average_exact_minute(&samples, start).unwrap() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn a_partial_minute_is_rejected() {
        let start = days_from_civil(2024, 5, 10) * 86_400 + 15 * 3_600;
        let mut samples = BTreeMap::new();
        for second in 0..59 {
            samples.insert(start + second, 1.0);
        }
        assert!(average_exact_minute(&samples, start).is_err());
    }

    #[test]
    fn duplicate_measured_timestamps_are_rejected() {
        let csv = b"timestamp,current\n2024-05-10 15:00:00,1.0\n2024-05-10 15:00:00,1.1\n";
        let error = parse_measured_csv(csv, "fixture.csv").unwrap_err();
        assert!(error.contains("duplicate one-second timestamp"));
    }

    #[test]
    fn measured_rows_must_be_chronologically_ordered() {
        let csv = b"timestamp,current\n2024-05-10 15:00:01,1.0\n2024-05-10 15:00:00,1.1\n";
        let error = parse_measured_csv(csv, "fixture.csv").unwrap_err();
        assert!(error.contains("not strictly increasing"));
    }

    #[test]
    fn negative_raw_correlation_requires_explicit_orientation_adjustment() {
        let observed = [1.0, 2.0, 3.0, 4.0];
        let predicted = [-1.0, -2.0, -3.0, -4.0];
        let metrics = calculate_metrics(&observed, &predicted).unwrap();
        assert!(metrics.raw_r < 0.0);
        assert!(metrics.polarity_correction);
        assert!((metrics.r2 - 1.0).abs() < 1e-12);
        assert!((metrics.pe - 1.0).abs() < 1e-12);
    }

    #[test]
    fn canonical_known_answer_rows_have_expected_published_rounding() {
        let expected_r2 = [0.80, 0.86, 0.67, 0.67, 0.48, 0.15, 0.57, 0.28];
        let expected_pe = [0.50, 0.43, 0.65, 0.63, 0.48, 0.15, 0.50, 0.23];
        for (index, answer) in KNOWN_ANSWERS.iter().enumerate() {
            assert_eq!(round2(answer.r2), expected_r2[index]);
            assert_eq!(round2(answer.pe), expected_pe[index]);
        }
        assert_eq!(KNOWN_ANSWERS.len(), 8);
        assert_eq!(PINS.len(), 20);
    }

    #[test]
    fn compiled_source_pins_match_the_machine_readable_manifest() {
        verify_manifest_constants().unwrap();
    }

    fn round2(value: f64) -> f64 {
        (value * 100.0).round() / 100.0
    }
}
