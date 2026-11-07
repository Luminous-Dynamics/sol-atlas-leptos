use city_morph_safe_oracle::{
    CORPUS_REVISION, Disposition, EXPECTED_CASE_COUNT, PROFILE_REVISION, SCENARIOS, evaluate_batch,
    fixture_fingerprint,
};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut blocked = 0usize;
    let mut unknown = 0usize;
    let mut review_eligible = 0usize;
    let mut mismatches = 0usize;

    let results = evaluate_batch(&SCENARIOS);
    for (case, actual) in SCENARIOS.iter().zip(results) {
        match actual {
            Disposition::Blocked => blocked += 1,
            Disposition::Unknown => unknown += 1,
            Disposition::EligibleForProfessionalReview => review_eligible += 1,
        }

        if actual != case.expected {
            mismatches += 1;
            eprintln!(
                "MISMATCH {}: expected={}, actual={}",
                case.id,
                case.expected.as_str(),
                actual.as_str()
            );
        }
    }

    println!("suite=synthetic_e1_only");
    println!("corpus_revision={CORPUS_REVISION}");
    println!("profile_revision={PROFILE_REVISION}");
    println!("scenario_count={}", SCENARIOS.len());
    println!("expected_scenario_count={EXPECTED_CASE_COUNT}");
    println!("blocked={blocked}");
    println!("unknown={unknown}");
    println!("eligible_for_professional_review={review_eligible}");
    println!("mismatches={mismatches}");
    println!(
        "fixture_fingerprint_fnv1a_non_cryptographic={:016x}",
        fixture_fingerprint()
    );
    println!("claim_ceiling=professional_review_eligibility_only_not_safety_approval");

    if SCENARIOS.len() != EXPECTED_CASE_COUNT || mismatches != 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
