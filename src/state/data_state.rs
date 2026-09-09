// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root
use leptos::prelude::*;
use sol_atlas_core::{
    evidence_confluence::{
        compute_evidence_aware, EventAdmissionPolicy, EvidenceConfluenceResult,
    },
    EvidenceBearingNaturalEvent,
};

use crate::data::types::*;

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
    /// Evidence-bearing view over natural events. Kept separate from the
    /// legacy renderer vector until selection/render types migrate fully.
    pub natural_event_records: RwSignal<Vec<EvidenceBearingNaturalEvent>>,
    pub major_cities: RwSignal<Vec<MajorCity>>,
    pub chokepoints: RwSignal<Vec<Chokepoint>>,
    pub critical_infrastructure: RwSignal<Vec<CriticalInfrastructure>>,
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
            natural_event_records: RwSignal::new(Vec::new()),
            major_cities: RwSignal::new(Vec::new()),
            chokepoints: RwSignal::new(Vec::new()),
            critical_infrastructure: RwSignal::new(Vec::new()),
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

    pub fn set_natural_event_records(&self, records: Vec<EvidenceBearingNaturalEvent>) {
        self.natural_event_records.set(records);
    }

    /// Resolve the evidence record corresponding to a renderer-facing event.
    ///
    /// The legacy selection shape currently has no stable event ID, so the
    /// bridge matches the exact projected fields generated from the same
    /// source parse. A later selection migration should put the evidence-record
    /// ID directly in the selection model and remove this compatibility lookup.
    pub fn record_for_natural_event(
        &self,
        event: &NaturalEvent,
    ) -> Option<EvidenceBearingNaturalEvent> {
        self.natural_event_records
            .read()
            .iter()
            .find(|record| same_event(&record.event, event))
            .cloned()
    }

    /// Compute Confluence from one coherent state snapshot while evaluating
    /// natural events from their canonical record-level evidence classes.
    ///
    /// Keeping this join inside `DataState` prevents renderer components from
    /// accidentally pairing one `LoadedData` snapshot with a different
    /// natural-event evidence snapshot. The returned audit preserves the exact
    /// admission/exclusion accounting for the computation.
    pub fn compute_evidence_confluence(
        &self,
        min_layers: usize,
        policy: EventAdmissionPolicy,
    ) -> EvidenceConfluenceResult {
        let snapshot = self.snapshot();
        let event_records = self.natural_event_records.read();
        compute_evidence_aware(&snapshot, &event_records, min_layers, policy)
    }

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

fn same_event(left: &NaturalEvent, right: &NaturalEvent) -> bool {
    left.lat == right.lat
        && left.lon == right.lon
        && left.event_type == right.event_type
        && left.magnitude == right.magnitude
        && left.name == right.name
}
