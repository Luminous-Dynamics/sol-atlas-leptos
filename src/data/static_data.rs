// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Static JSON data loading — delegates parsing to terra-atlas-core.

use sol_atlas_core::data::LoadedDataReport;
use sol_atlas_core::types::LoadedData;

const SITES_JSON: &str = include_str!("../../assets/data/sites-clustered.json");
const MAGLEV_JSON: &str = include_str!("../../assets/data/maglev-network.json");
const VAULTS_JSON: &str = include_str!("../../assets/data/resontia-vaults.json");
const TERRA_LUMINA_JSON: &str = include_str!("../../assets/data/terra-lumina-sites.json");
const REGIONS_JSON: &str = include_str!("../../assets/data/earth-regions.json");
const SUPPLY_ROUTES_JSON: &str = include_str!("../../assets/data/supply-routes.json");
const CLIMATE_JSON: &str = include_str!("../../assets/data/climate-projects.json");
const INFRA_JSON: &str = include_str!("../../assets/data/infrastructure.json");
const FOSSIL_DEPOSITS_JSON: &str = include_str!("../../assets/data/fossil-deposits.json");
const NUCLEAR_SITES_JSON: &str = include_str!("../../assets/data/nuclear-sites.json");

// Restored 2026-07-11 (design Phase D): these 7 datasets were dropped from
// the WASM bundle on 2026-07-10 for having no render path. They now do —
// see DataState/update_renderer_data/layer_panel's REAL_LAYERS.
const EARTHQUAKES_JSON: &str = include_str!("../../assets/data/usgs-earthquakes.json");
const FIRES_JSON: &str = include_str!("../../assets/data/nasa-firms.json");
const STORMS_JSON: &str = include_str!("../../assets/data/nasa-eonet.json");
const VOLCANOES_JSON: &str = include_str!("../../assets/data/volcanoes.json");
const CITIES_JSON: &str = include_str!("../../assets/data/major-cities-1m.json");
const CHOKEPOINTS_JSON: &str = include_str!("../../assets/data/chokepoints.json");
const CRITICAL_INFRA_JSON: &str = include_str!("../../assets/data/critical-infrastructure.json");

pub fn load_all_with_health() -> LoadedDataReport {
    sol_atlas_core::data::load_all_with_health(
        SITES_JSON,
        MAGLEV_JSON,
        VAULTS_JSON,
        TERRA_LUMINA_JSON,
        REGIONS_JSON,
        SUPPLY_ROUTES_JSON,
        CLIMATE_JSON,
        INFRA_JSON,
        FOSSIL_DEPOSITS_JSON,
        NUCLEAR_SITES_JSON,
        EARTHQUAKES_JSON,
        FIRES_JSON,
        STORMS_JSON,
        VOLCANOES_JSON,
        CITIES_JSON,
        CHOKEPOINTS_JSON,
        CRITICAL_INFRA_JSON,
    )
}

/// Backwards-compatible data-only loader.
pub fn load_all() -> LoadedData {
    load_all_with_health().data
}
