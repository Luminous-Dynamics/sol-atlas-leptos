// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Small deterministic fixture for the Humanity/AI Bootstrap Atlas.
//!
//! This is intentionally synthetic. It exercises graph semantics without
//! pretending that the fixture is field evidence or a civilization recipe.

use crate::capability::*;

pub const SYNTHETIC_FIXTURE_REFERENCE: &str = "sol-atlas:bootstrap-fixture:v1";

fn capability(
    id: &str,
    name: &str,
    dependencies: Vec<CapabilityDependency>,
) -> Capability {
    Capability {
        id: CapabilityId(id.into()),
        name: name.into(),
        description: format!("Synthetic bootstrap fixture capability: {name}."),
        state: CapabilityState::Demonstrated,
        dependencies,
        evidence: vec![CapabilityEvidence {
            kind: EvidenceKind::Scenario,
            reference: SYNTHETIC_FIXTURE_REFERENCE.into(),
            claim_ceiling: "Synthetic fixture only; not field validation or qualification."
                .into(),
        }],
        provenance: vec![CapabilityProvenance {
            source: SYNTHETIC_FIXTURE_REFERENCE.into(),
            snapshot: Some("2026-10-01".into()),
            notes: "Deterministic test fixture; intentionally non-empirical.".into(),
        }],
        locations: vec![],
        qualification: None,
        contribution: HumanAiContribution {
            human: "Define goals, operate, maintain, judge outcomes".into(),
            ai: "Optional analysis, planning, and dependency exploration".into(),
        },
    }
}

fn required(id: &str, relation: DependencyKind) -> CapabilityDependency {
    CapabilityDependency {
        capability: CapabilityId(id.into()),
        relation,
        substitutes: vec![],
    }
}

fn required_with_substitutes(
    id: &str,
    relation: DependencyKind,
    substitutes: &[&str],
) -> CapabilityDependency {
    CapabilityDependency {
        capability: CapabilityId(id.into()),
        relation,
        substitutes: substitutes
            .iter()
            .map(|candidate| CapabilityId((*candidate).into()))
            .collect(),
    }
}

/// A minimal graph for exercising the bootstrap-path interaction model.
///
/// The graph is deliberately not presented as a universal historical sequence.
pub fn water_purification_fixture() -> CapabilityGraph {
    CapabilityGraph {
        capabilities: vec![
            capability(
                "water.purification",
                "Water purification",
                vec![
                    required_with_substitutes(
                        "energy.electricity",
                        DependencyKind::Energy,
                        &["energy.mechanical"],
                    ),
                    required("materials.filter_media", DependencyKind::Material),
                    required("knowledge.water_treatment", DependencyKind::Knowledge),
                    required("maintenance.pump", DependencyKind::Maintenance),
                ],
            ),
            capability(
                "energy.electricity",
                "Electricity supply",
                vec![required("manufacturing.generator", DependencyKind::Required)],
            ),
            capability(
                "materials.filter_media",
                "Filter-media production",
                vec![required("manufacturing.workshop", DependencyKind::Material)],
            ),
            capability(
                "manufacturing.generator",
                "Generator manufacturing",
                vec![required("manufacturing.workshop", DependencyKind::Required)],
            ),
            capability(
                "manufacturing.workshop",
                "General manufacturing workshop",
                vec![
                    required("materials.metals", DependencyKind::Material),
                    required("knowledge.fabrication", DependencyKind::Knowledge),
                ],
            ),
            capability(
                "maintenance.pump",
                "Pump maintenance",
                vec![required("manufacturing.workshop", DependencyKind::Maintenance)],
            ),
            capability(
                "knowledge.water_treatment",
                "Water-treatment knowledge",
                vec![required("knowledge.preservation", DependencyKind::Knowledge)],
            ),
            capability(
                "knowledge.fabrication",
                "Fabrication knowledge",
                vec![required("knowledge.preservation", DependencyKind::Knowledge)],
            ),
            capability(
                "materials.metals",
                "Metal feedstock",
                vec![required("manufacturing.workshop", DependencyKind::Material)],
            ),
            capability(
                "knowledge.preservation",
                "Knowledge preservation",
                vec![],
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_is_explicitly_synthetic() {
        let graph = water_purification_fixture();
        assert!(graph
            .capabilities
            .iter()
            .all(|c| c.evidence.iter().all(|e| e.kind == EvidenceKind::Scenario)));
        assert!(graph.capabilities.iter().all(|c| !c.is_qualified()));
    }

    #[test]
    fn water_purification_has_a_deterministic_required_closure() {
        let graph = water_purification_fixture();
        let closure = graph
            .required_closure(&CapabilityId("water.purification".into()))
            .unwrap();

        assert!(closure.contains(&CapabilityId("water.purification".into())));
        assert!(closure.contains(&CapabilityId("energy.electricity".into())));
        assert!(closure.contains(&CapabilityId("manufacturing.workshop".into())));
        assert!(closure.contains(&CapabilityId("knowledge.preservation".into())));
    }

    #[test]
    #[test]
    fn fixture_declares_alternative_candidates_without_selecting_them() {
        let graph = water_purification_fixture();
        let root = graph
            .capabilities
            .iter()
            .find(|capability| capability.id == CapabilityId("water.purification".into()))
            .unwrap();
        let alternatives = root.alternative_paths();

        assert_eq!(alternatives.len(), 1);
        assert_eq!(alternatives[0].for_dependency, CapabilityId("energy.electricity".into()));
        assert_eq!(alternatives[0].candidate, CapabilityId("energy.mechanical".into()));
        assert!(alternatives[0].evidence.is_empty());
        assert!(alternatives[0].claim_ceiling.contains("not established"));

        let closure = graph
            .required_closure(&CapabilityId("water.purification".into()))
            .unwrap();
        assert!(!closure.contains(&CapabilityId("energy.mechanical".into())));
    }

    #[test]
    fn fixture_cycles_are_handled_without_nontermination() {
        let graph = water_purification_fixture();
        let closure = graph
            .required_closure(&CapabilityId("water.purification".into()))
            .unwrap();

        let workshop_count = closure
            .iter()
            .filter(|id| id.0 == "manufacturing.workshop")
            .count();

        assert_eq!(workshop_count, 1);
    }
}
