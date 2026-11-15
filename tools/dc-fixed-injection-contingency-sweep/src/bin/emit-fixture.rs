// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Emit a deterministic tiny contingency-sweep receipt for CI JSON validation.

use dc_fixed_injection_contingency_sweep::{
    SweepOptions, sweep_fixed_injection_single_branch_outages,
};
use dc_power_flow_reference::{DcBranch, DcBus, DcNetwork};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let network = DcNetwork {
        base_mva: 100.0,
        buses: vec![
            DcBus {
                id: 1,
                specified_injection_mw: None,
            },
            DcBus {
                id: 2,
                specified_injection_mw: Some(0.0),
            },
            DcBus {
                id: 3,
                specified_injection_mw: Some(-60.0),
            },
        ],
        branches: vec![
            DcBranch {
                from_bus: 1,
                to_bus: 2,
                reactance_pu: 0.1,
                tap_ratio: 0.0,
                phase_shift_degrees: 0.0,
                rate_a_mva: Some(45.0),
                in_service: true,
            },
            DcBranch {
                from_bus: 2,
                to_bus: 3,
                reactance_pu: 0.1,
                tap_ratio: 0.0,
                phase_shift_degrees: 0.0,
                rate_a_mva: Some(45.0),
                in_service: true,
            },
            DcBranch {
                from_bus: 1,
                to_bus: 3,
                reactance_pu: 0.2,
                tap_ratio: 0.0,
                phase_shift_degrees: 0.0,
                rate_a_mva: Some(50.0),
                in_service: true,
            },
        ],
    };

    let report = sweep_fixed_injection_single_branch_outages(
        "ci-fixture:triangle-v0.1",
        &network,
        SweepOptions::default(),
    )?;
    println!("{}", report.to_json());
    Ok(())
}
