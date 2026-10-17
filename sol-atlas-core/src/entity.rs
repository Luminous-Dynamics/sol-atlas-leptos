// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Renderer-agnostic identity and space-time primitives for Sol Atlas.
//!
//! The existing layer-specific types remain the rendering compatibility
//! surface. These primitives form the additive substrate that lets multiple
//! datasets describe the same real-world entity without forcing every domain
//! to invent a new top-level state vector.

use serde::{Deserialize, Serialize};

/// Stable Sol Atlas identity for a thing represented in the world model.
///
/// The identifier is intentionally opaque. It may be backed by a local ID,
/// Mycelix identity, Overture GERS ID, registry key, or another durable naming
/// scheme without coupling the core crate to any one authority.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AtlasEntityId(pub String);

impl AtlasEntityId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for AtlasEntityId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AtlasEntityId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Identifier assigned to the same entity by an external system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalId {
    /// Stable namespace, e.g. `overture-gers`, `osm-node`, `wikidata`.
    pub namespace: String,
    pub value: String,
}

impl ExternalId {
    pub fn new(namespace: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            value: value.into(),
        }
    }
}

/// Coordinate frame used by a spatial extent.
///
/// Earth WGS84 is explicit rather than implicit so Sol Atlas can grow into
/// lunar, planetary, orbital, indoor, and local engineering frames without
/// rewriting the entity contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReferenceFrame {
    EarthWgs84,
    CelestialBody { body: String, frame: String },
    Local { name: String },
}

/// Invalid latitude/longitude supplied to a WGS84 point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateError {
    LatitudeOutOfRange,
    LongitudeOutOfRange,
}

/// A position with an explicit coordinate frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoPoint {
    pub lat: f64,
    pub lon: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub altitude_m: Option<f64>,
    pub reference_frame: ReferenceFrame,
}

impl GeoPoint {
    /// Construct a validated WGS84 position.
    pub fn wgs84(
        lat: f64,
        lon: f64,
        altitude_m: Option<f64>,
    ) -> Result<Self, CoordinateError> {
        if !(-90.0..=90.0).contains(&lat) {
            return Err(CoordinateError::LatitudeOutOfRange);
        }
        if !(-180.0..=180.0).contains(&lon) {
            return Err(CoordinateError::LongitudeOutOfRange);
        }
        Ok(Self {
            lat,
            lon,
            altitude_m,
            reference_frame: ReferenceFrame::EarthWgs84,
        })
    }

    /// Construct a point in a non-WGS84 frame.
    ///
    /// The two numeric axes retain the lightweight `lat`/`lon` storage shape
    /// used by current renderers; frame-specific adapters own interpretation.
    pub fn in_frame(lat: f64, lon: f64, altitude_m: Option<f64>, frame: ReferenceFrame) -> Self {
        Self {
            lat,
            lon,
            altitude_m,
            reference_frame: frame,
        }
    }
}

/// Spatial footprint of an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum SpatialExtent {
    Point(GeoPoint),
    Polyline(Vec<GeoPoint>),
    /// Polygon rings; the first ring is the exterior and subsequent rings are holes.
    Polygon(Vec<Vec<GeoPoint>>),
    /// Geometry delegated to another Atlas entity, useful for jurisdictions,
    /// buildings, parcels, and externally managed boundary datasets.
    EntityRef(AtlasEntityId),
}

/// Interval in which an entity representation is considered applicable.
///
/// `None` on both ends means timeless/unknown. Unix milliseconds are used at
/// this boundary because the current web stack already exchanges integer time;
/// higher precision or non-Earth time scales can be introduced in adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TemporalExtent {
    pub valid_from_unix_ms: Option<i64>,
    pub valid_until_unix_ms: Option<i64>,
}

impl TemporalExtent {
    pub const fn timeless() -> Self {
        Self {
            valid_from_unix_ms: None,
            valid_until_unix_ms: None,
        }
    }
}

/// A typed edge from one entity to another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    /// Open predicate vocabulary, e.g. `located_in`, `feeds`, `operated_by`.
    pub predicate: String,
    pub target: AtlasEntityId,
}

impl Relationship {
    pub fn new(predicate: impl Into<String>, target: impl Into<AtlasEntityId>) -> Self {
        Self {
            predicate: predicate.into(),
            target: target.into(),
        }
    }
}

/// Audience authorized to receive an entity projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Audience {
    Private,
    Capability(String),
    TrustedGroup(String),
    Community(String),
    Public,
}

/// Visibility is part of the entity contract rather than a UI afterthought.
///
/// Precision fields allow callers to reveal an approximate place/time without
/// disclosing the source record's full precision. Enforcement belongs to the
/// data-access boundary; this type makes the requested policy explicit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisibilityPolicy {
    pub audience: Audience,
    pub spatial_precision_m: Option<u32>,
    pub temporal_precision_s: Option<u64>,
    pub expires_at_unix_ms: Option<i64>,
    pub redisclosure_allowed: bool,
}

impl VisibilityPolicy {
    pub const fn private() -> Self {
        Self {
            audience: Audience::Private,
            spatial_precision_m: None,
            temporal_precision_s: None,
            expires_at_unix_ms: None,
            redisclosure_allowed: false,
        }
    }

    pub const fn public() -> Self {
        Self {
            audience: Audience::Public,
            spatial_precision_m: None,
            temporal_precision_s: None,
            expires_at_unix_ms: None,
            redisclosure_allowed: true,
        }
    }
}

/// Universal identity shell for a thing in Sol Atlas.
///
/// Domain observations intentionally do not live here yet. SAT-2 adds
/// provenance-bearing claims so conflicting datasets can coexist without one
/// silently overwriting another.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtlasEntity {
    pub id: AtlasEntityId,
    pub display_name: Option<String>,
    /// Open, additive ontology terms such as `hospital`, `bridge`, `river`.
    pub kinds: Vec<String>,
    pub external_ids: Vec<ExternalId>,
    pub spatial: Option<SpatialExtent>,
    pub temporal: TemporalExtent,
    pub relationships: Vec<Relationship>,
    pub visibility: VisibilityPolicy,
}

impl AtlasEntity {
    /// Visibility is mandatory: callers must choose the disclosure boundary
    /// rather than receiving an accidental public default.
    pub fn new(id: impl Into<AtlasEntityId>, visibility: VisibilityPolicy) -> Self {
        Self {
            id: id.into(),
            display_name: None,
            kinds: Vec::new(),
            external_ids: Vec::new(),
            spatial: None,
            temporal: TemporalExtent::timeless(),
            relationships: Vec::new(),
            visibility,
        }
    }

    pub fn add_kind(&mut self, kind: impl Into<String>) {
        let kind = kind.into();
        if !self.kinds.contains(&kind) {
            self.kinds.push(kind);
        }
    }

    pub fn add_external_id(&mut self, external_id: ExternalId) {
        if !self.external_ids.contains(&external_id) {
            self.external_ids.push(external_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wgs84_rejects_impossible_coordinates() {
        assert_eq!(
            GeoPoint::wgs84(90.1, 0.0, None),
            Err(CoordinateError::LatitudeOutOfRange)
        );
        assert_eq!(
            GeoPoint::wgs84(0.0, 180.1, None),
            Err(CoordinateError::LongitudeOutOfRange)
        );
    }

    #[test]
    fn entity_identity_can_bridge_external_namespaces() {
        let mut entity = AtlasEntity::new("atlas:hoover-dam", VisibilityPolicy::public());
        entity.display_name = Some("Hoover Dam".into());
        entity.add_kind("dam");
        entity.add_kind("power_plant");
        entity.add_external_id(ExternalId::new("wikidata", "Q12516"));
        entity.relationships.push(Relationship::new(
            "located_in",
            AtlasEntityId::new("atlas:nevada"),
        ));
        entity.spatial = Some(SpatialExtent::Point(
            GeoPoint::wgs84(36.0156, -114.7378, None).unwrap(),
        ));

        let encoded = serde_json::to_string(&entity).unwrap();
        let decoded: AtlasEntity = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, entity);
        assert_eq!(decoded.external_ids[0].namespace, "wikidata");
    }

    #[test]
    fn kinds_and_external_ids_are_deduplicated() {
        let mut entity = AtlasEntity::new("atlas:test", VisibilityPolicy::private());
        entity.add_kind("hospital");
        entity.add_kind("hospital");
        entity.add_external_id(ExternalId::new("gers", "123"));
        entity.add_external_id(ExternalId::new("gers", "123"));

        assert_eq!(entity.kinds, vec!["hospital"]);
        assert_eq!(entity.external_ids.len(), 1);
        assert_eq!(entity.visibility.audience, Audience::Private);
    }
}
