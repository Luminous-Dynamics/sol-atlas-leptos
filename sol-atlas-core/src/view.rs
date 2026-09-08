// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Composable Sol Atlas view context.
//!
//! A user-facing "mode" is represented as orthogonal dimensions instead of a
//! growing enum of special cases. Personal, municipal, planetary, and
//! interplanetary experiences therefore share one interaction model.

use serde::{Deserialize, Serialize};

use crate::claims::{Confidence, EvidenceClass};

/// Whose/which world is being viewed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum AtlasScope {
    Personal,
    Household(String),
    Community(String),
    Organization(String),
    Municipality(String),
    Region(String),
    Nation(String),
    Humanity,
    Earth,
    CelestialBody(String),
    SolarSystem,
}

/// Top-level intent. Sub-workflows can refine these activities without
/// fragmenting the primary interaction contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasActivity {
    Observe,
    Simulate,
    Coordinate,
}

/// Open lens vocabulary. New domains should generally be data/configuration,
/// not a new core enum variant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AtlasLens(pub String);

impl AtlasLens {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl From<&str> for AtlasLens {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AtlasLens {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Temporal projection of the world model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AtlasTime {
    Now,
    At { unix_ms: i64 },
    Interval { start_unix_ms: i64, end_unix_ms: i64 },
    Forecast { as_of_unix_ms: i64, horizon_s: u64 },
    Scenario { scenario_id: String },
}

/// Which evidence classes may participate in a view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustFilter {
    pub allowed_classes: Vec<EvidenceClass>,
    pub minimum_confidence: Option<Confidence>,
}

impl TrustFilter {
    /// Evidence-grounded default for ordinary observation. Predictions and
    /// planning scenarios require an explicit opt-in.
    pub fn grounded() -> Self {
        Self {
            allowed_classes: vec![
                EvidenceClass::Observed,
                EvidenceClass::Verified,
                EvidenceClass::Curated,
                EvidenceClass::Estimated,
                EvidenceClass::Contested,
            ],
            minimum_confidence: None,
        }
    }

    pub fn allows(&self, class: EvidenceClass, confidence: Confidence) -> bool {
        if !self.allowed_classes.contains(&class) {
            return false;
        }
        match self.minimum_confidence {
            Some(minimum) => confidence.value() >= minimum.value(),
            None => true,
        }
    }
}

/// Complete composable context for a Sol Atlas projection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtlasViewContext {
    pub scope: AtlasScope,
    pub activity: AtlasActivity,
    pub lenses: Vec<AtlasLens>,
    pub time: AtlasTime,
    pub trust: TrustFilter,
}

impl AtlasViewContext {
    /// Current planetary behavior expressed through the new contract.
    pub fn earth_observe() -> Self {
        Self {
            scope: AtlasScope::Earth,
            activity: AtlasActivity::Observe,
            lenses: Vec::new(),
            time: AtlasTime::Now,
            trust: TrustFilter::grounded(),
        }
    }

    /// Personal Atlas deliberately changes scope only; Observe/Simulate/
    /// Coordinate and every domain lens remain reusable.
    pub fn personal(activity: AtlasActivity) -> Self {
        Self {
            scope: AtlasScope::Personal,
            activity,
            lenses: Vec::new(),
            time: AtlasTime::Now,
            trust: TrustFilter::grounded(),
        }
    }

    pub fn add_lens(&mut self, lens: impl Into<AtlasLens>) {
        let lens = lens.into();
        if !self.lenses.contains(&lens) {
            self.lenses.push(lens);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn personal_is_a_scope_not_a_parallel_activity_model() {
        let observe = AtlasViewContext::personal(AtlasActivity::Observe);
        let simulate = AtlasViewContext::personal(AtlasActivity::Simulate);
        let coordinate = AtlasViewContext::personal(AtlasActivity::Coordinate);

        assert_eq!(observe.scope, AtlasScope::Personal);
        assert_eq!(simulate.scope, AtlasScope::Personal);
        assert_eq!(coordinate.scope, AtlasScope::Personal);
        assert_ne!(observe.activity, simulate.activity);
    }

    #[test]
    fn lenses_are_open_and_deduplicated() {
        let mut view = AtlasViewContext::earth_observe();
        view.add_lens("water");
        view.add_lens("water");
        view.add_lens("accessibility");
        assert_eq!(view.lenses.len(), 2);
    }

    #[test]
    fn grounded_view_requires_explicit_scenario_opt_in() {
        let trust = TrustFilter::grounded();
        let confidence = Confidence::certain();
        assert!(trust.allows(EvidenceClass::Observed, confidence));
        assert!(trust.allows(EvidenceClass::Contested, confidence));
        assert!(!trust.allows(EvidenceClass::Predicted, confidence));
        assert!(!trust.allows(EvidenceClass::Scenario, confidence));
    }

    #[test]
    fn minimum_confidence_is_enforced() {
        let mut trust = TrustFilter::grounded();
        trust.minimum_confidence = Some(Confidence::new(0.8).unwrap());
        assert!(trust.allows(
            EvidenceClass::Observed,
            Confidence::new(0.9).unwrap()
        ));
        assert!(!trust.allows(
            EvidenceClass::Observed,
            Confidence::new(0.7).unwrap()
        ));
    }
}
