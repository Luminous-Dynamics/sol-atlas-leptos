// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use sol_atlas_core::energy_pathway::EvidenceClass;
use sol_atlas_core::energy_storage::{
    EnergyStorageRequest, StorageCarrier, StorageInterval, assess_energy_storage,
};
use sol_atlas_core::heat_pump::{
    HeatPumpRequest, HeatPumpTemperatures, assess_heat_pump,
};

const TOLERANCE: f64 = 1.0e-9;

/// Coupled reference path: recover low-grade heat, add compressor electricity,
/// store a capacity-limited amount of useful heat, then serve a later interval.
#[test]
fn heat_pump_to_thermal_store_to_later_load_closes_each_boundary() {
    let heat_pump = HeatPumpRequest {
        schema_version: "heat-pump-v0.1".to_owned(),
        scenario_id: "synthetic-coupled-campus-heat".to_owned(),
        synthetic: true,
        temperatures: HeatPumpTemperatures {
            source_c: 20.0,
            delivery_c: 80.0,
        },
        source_heat_available_kwh: 30.0,
        max_source_heat_kwh: 30.0,
        compressor_electricity_available_kwh: 10.0,
        max_compressor_electricity_kwh: 10.0,
        auxiliary_electricity_kwh: 1.0,
        distribution_loss_fraction: 0.1,
        evidence_class: EvidenceClass::AnalystAssumption,
        evidence_ref: None,
        evidence_as_of: None,
    };
    let heat = assess_heat_pump(&heat_pump).expect("synthetic heat-pump case should be valid");

    assert!((heat.gross_heat_output_kwh - 40.0).abs() < TOLERANCE);
    assert!((heat.useful_heat_delivered_kwh - 36.0).abs() < TOLERANCE);
    assert!((heat.first_law_balance_residual_kwh).abs() < TOLERANCE);

    let store = EnergyStorageRequest {
        schema_version: "energy-storage-v0.1".to_owned(),
        storage_id: "synthetic-thermal-store".to_owned(),
        synthetic: true,
        carrier: StorageCarrier::ThermalHeat,
        capacity_kwh: 20.0,
        opening_energy_kwh: 0.0,
        max_charge_power_kw: 50.0,
        max_discharge_power_kw: 10.0,
        charge_efficiency: 0.9,
        discharge_efficiency: 0.8,
        standing_loss_fraction_per_hour: 0.0,
        terminal_reserve_kwh: 7.0,
        evidence_class: EvidenceClass::AnalystAssumption,
        evidence_ref: None,
        evidence_as_of: None,
        intervals: vec![
            StorageInterval {
                interval_id: "heat-pump-charge".to_owned(),
                duration_hours: 1.0,
                charge_available_kwh: heat.useful_heat_delivered_kwh,
                discharge_requested_kwh: 0.0,
            },
            StorageInterval {
                interval_id: "later-heat-demand".to_owned(),
                duration_hours: 1.0,
                charge_available_kwh: 0.0,
                discharge_requested_kwh: 10.0,
            },
        ],
    };
    let storage =
        assess_energy_storage(&store).expect("synthetic thermal-store case should be valid");

    assert!(storage.intervals[0].closing_energy_kwh <= storage.capacity_kwh);
    assert!((storage.total_discharge_delivered_kwh - 10.0).abs() < TOLERANCE);
    assert!((storage.final_energy_kwh - 7.5).abs() < TOLERANCE);
    assert!(storage.terminal_reserve_met);
    assert!((storage.trajectory_balance_residual_kwh).abs() < TOLERANCE);

    // The heat pump's 36 kWh service stream divides into charge accepted and
    // excess/curtailed source heat; the store then accounts for all accepted input.
    assert!(
        (heat.useful_heat_delivered_kwh
            - storage.total_charge_accepted_kwh
            - storage.total_charge_curtailed_kwh)
            .abs()
            < TOLERANCE
    );
    assert!(
        (storage.total_charge_accepted_kwh
            - storage.final_energy_kwh
            - storage.total_discharge_delivered_kwh
            - storage.total_loss_kwh)
            .abs()
            < TOLERANCE
    );
}
