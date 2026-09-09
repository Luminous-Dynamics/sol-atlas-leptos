// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Static JSON data loading — delegates parsing to sol-atlas-core.

use sol_atlas_core::{EvidenceBearingNaturalEvent, types::LoadedData};

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

const EARTHQUAKES_JSON: &str = include_str!("../../assets/data/usgs-earthquakes.json");
const FIRES_JSON: &str = include_str!("../../assets/data/nasa-firms.json");
const STORMS_JSON: &str = include_str!("../../assets/data/nasa-eonet.json");
const VOLCANOES_JSON: &str = include_str!("../../assets/data/volcanoes.json");
const CITIES_JSON: &str = include_str!("../../assets/data/major-cities-1m.json");
const CHOKEPOINTS_JSON: &str = include_str!("../../assets/data/chokepoints.json");
const CRITICAL_INFRA_JSON: &str = include_str!("../../assets/data/critical-infrastructure.json");

pub fn load_all() -> LoadedData {
    sol_atlas_core::data::load_all(
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

/// Evidence-bearing natural-event view over the same static source bytes used
/// by `load_all`. Keeping source inputs identical prevents the renderer and
/// dossier from drifting onto different snapshots.
pub fn load_natural_event_records() -> Vec<EvidenceBearingNaturalEvent> {
    sol_atlas_core::parse_natural_event_records(
        EARTHQUAKES_JSON,
        FIRES_JSON,
        STORMS_JSON,
        VOLCANOES_JSON,
    )
}
