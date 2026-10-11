// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root
use leptos::prelude::*;

use crate::data::types::*;
use sol_atlas_core::usgs::UsgsFeedStatus;

#[derive(Clone)]
pub struct DataState {
    pub sites: RwSignal<Vec<Site>>,
    pub geothermal_nodes: RwSignal<Vec<GeothermalNode>>,
    pub maglev_corridors: RwSignal<Vec<MaglevCorridor>>,
    pub resontia_vaults: RwSignal<Vec<ResontiaVault>>,
    pub terra_lumina_sites: RwSignal<Vec<TerraLuminaSite>>,
    pub earth_regions: RwSignal<Vec<EarthRegion>>,
    pub supply_routes: RwSignal<Vec<SupplyRoute>>,
    pub climate_projects: RwSignal<Vec<ClimateProject>>,
    pub emergency_shelters: RwSignal<Vec<EmergencyShelter>>,
    pub health_facilities: RwSignal<Vec<HealthFacility>>,
    pub robotics_dispatch: RwSignal<Vec<RoboticsDispatch>>,
    pub fossil_deposits: RwSignal<Vec<FossilDeposit>>,
    pub nuclear_sites: RwSignal<Vec<NuclearSite>>,
    pub natural_events: RwSignal<Vec<NaturalEvent>>,
    pub major_cities: RwSignal<Vec<MajorCity>>,
    pub chokepoints: RwSignal<Vec<Chokepoint>>,
    pub critical_infrastructure: RwSignal<Vec<CriticalInfrastructure>>,
    /// Current health/provenance for the live USGS earthquake layer.
    pub usgs_feed_status: RwSignal<UsgsFeedStatus>,
}

impl DataState {
    pub fn new() -> Self {
        Self {
            sites: RwSignal::new(Vec::new()),
            geothermal_nodes: RwSignal::new(Vec::new()),
            maglev_corridors: RwSignal::new(Vec::new()),
            resontia_vaults: RwSignal::new(Vec::new()),
            terra_lumina_sites: RwSignal::new(Vec::new()),
            earth_regions: RwSignal::new(Vec::new()),
            supply_routes: RwSignal::new(Vec::new()),
            climate_projects: RwSignal::new(Vec::new()),
            emergency_shelters: RwSignal::new(Vec::new()),
            health_facilities: RwSignal::new(Vec::new()),
            robotics_dispatch: RwSignal::new(Vec::new()),
            fossil_deposits: RwSignal::new(Vec::new()),
            nuclear_sites: RwSignal::new(Vec::new()),
            natural_events: RwSignal::new(Vec::new()),
            major_cities: RwSignal::new(Vec::new()),
            chokepoints: RwSignal::new(Vec::new()),
            critical_infrastructure: RwSignal::new(Vec::new()),
            usgs_feed_status: RwSignal::new(UsgsFeedStatus::default()),
        }
    }

    pub fn set_all(&self, data: LoadedData) {
        self.sites.set(data.sites);
        self.geothermal_nodes.set(data.geothermal_nodes);
        self.maglev_corridors.set(data.maglev_corridors);
        self.resontia_vaults.set(data.resontia_vaults);
        self.terra_lumina_sites.set(data.terra_lumina_sites);
        self.earth_regions.set(data.earth_regions);
        self.supply_routes.set(data.supply_routes);
        self.climate_projects.set(data.climate_projects);
        self.emergency_shelters.set(data.emergency_shelters);
        self.health_facilities.set(data.health_facilities);
        self.robotics_dispatch.set(data.robotics_dispatch);
        self.fossil_deposits.set(data.fossil_deposits);
        self.nuclear_sites.set(data.nuclear_sites);
        self.natural_events.set(data.natural_events);
        self.major_cities.set(data.major_cities);
        self.chokepoints.set(data.chokepoints);
        self.critical_infrastructure
            .set(data.critical_infrastructure);
    }

    /// Snapshot the current reactive state into a plain [`LoadedData`] —
    /// used by derived, non-signal computations (e.g. Confluence) that
    /// need a single owned view across all layers at once.
    pub fn snapshot(&self) -> LoadedData {
        LoadedData {
            sites: self.sites.get(),
            geothermal_nodes: self.geothermal_nodes.get(),
            maglev_corridors: self.maglev_corridors.get(),
            resontia_vaults: self.resontia_vaults.get(),
            terra_lumina_sites: self.terra_lumina_sites.get(),
            earth_regions: self.earth_regions.get(),
            supply_routes: self.supply_routes.get(),
            climate_projects: self.climate_projects.get(),
            emergency_shelters: self.emergency_shelters.get(),
            health_facilities: self.health_facilities.get(),
            robotics_dispatch: self.robotics_dispatch.get(),
            fossil_deposits: self.fossil_deposits.get(),
            nuclear_sites: self.nuclear_sites.get(),
            natural_events: self.natural_events.get(),
            major_cities: self.major_cities.get(),
            chokepoints: self.chokepoints.get(),
            critical_infrastructure: self.critical_infrastructure.get(),
        }
    }
}
