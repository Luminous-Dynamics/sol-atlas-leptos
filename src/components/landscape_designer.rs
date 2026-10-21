// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! First vertical slice of the multi-scale landscape design workbench.
//!
//! This is a schematic editor, not a survey/GIS source or an engineering solver.
//! All canvas coordinates are arbitrary project-local drawing units, not metres or map coordinates.
//! Georeferencing and standards-based exchange must be explicit future operations.

use crate::landscape::{
    DESIGN_INTENT_STATUS, FRAME_KIND, GEOMETRY_REPRESENTATION, GEOMETRY_STATUS, SNAPSHOT_SCHEMA,
    SNAPSHOT_SCHEMA_URI, USER_AUTHORED_SOURCE, validate_snapshot,
};
use leptos::prelude::*;
use leptos_router::components::A;
use serde_json::{Value, json};
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FeatureKind {
    Tree,
    PlantingBed,
    Water,
    Path,
    BuiltElement,
}

impl FeatureKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Tree => "Canopy tree",
            Self::PlantingBed => "Planting bed",
            Self::Water => "Water feature",
            Self::Path => "Path",
            Self::BuiltElement => "Built element",
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Tree => "tree",
            Self::PlantingBed => "planting-bed",
            Self::Water => "water",
            Self::Path => "path",
            Self::BuiltElement => "built-element",
        }
    }

    fn from_key(value: &str) -> Option<Self> {
        match value {
            "tree" => Some(Self::Tree),
            "planting-bed" => Some(Self::PlantingBed),
            "water" => Some(Self::Water),
            "path" => Some(Self::Path),
            "built-element" => Some(Self::BuiltElement),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DesignFeature {
    id: u32,
    x_du: u16,
    y_du: u16,
    kind: FeatureKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LandscapeScale {
    Site,
    District,
    Watershed,
    Bioregion,
}

impl LandscapeScale {
    const fn label(self) -> &'static str {
        match self {
            Self::Site => "Site / parcel",
            Self::District => "District",
            Self::Watershed => "Watershed",
            Self::Bioregion => "Bioregion",
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Site => "site-parcel",
            Self::District => "district",
            Self::Watershed => "watershed",
            Self::Bioregion => "bioregion",
        }
    }

    fn from_key(value: &str) -> Option<Self> {
        match value {
            "site-parcel" => Some(Self::Site),
            "district" => Some(Self::District),
            "watershed" => Some(Self::Watershed),
            "bioregion" => Some(Self::Bioregion),
            _ => None,
        }
    }
}

fn starter_features() -> Vec<DesignFeature> {
    vec![
        DesignFeature {
            id: 1,
            x_du: 180,
            y_du: 160,
            kind: FeatureKind::Tree,
        },
        DesignFeature {
            id: 2,
            x_du: 245,
            y_du: 195,
            kind: FeatureKind::Tree,
        },
        DesignFeature {
            id: 3,
            x_du: 365,
            y_du: 250,
            kind: FeatureKind::PlantingBed,
        },
        DesignFeature {
            id: 4,
            x_du: 430,
            y_du: 285,
            kind: FeatureKind::Water,
        },
        DesignFeature {
            id: 5,
            x_du: 540,
            y_du: 360,
            kind: FeatureKind::Path,
        },
        DesignFeature {
            id: 6,
            x_du: 740,
            y_du: 190,
            kind: FeatureKind::BuiltElement,
        },
        DesignFeature {
            id: 7,
            x_du: 785,
            y_du: 420,
            kind: FeatureKind::Tree,
        },
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditCommand {
    Added {
        feature: DesignFeature,
    },
    Deleted {
        feature: DesignFeature,
        index: usize,
    },
}

/// Remove one feature while preserving enough information to undo the edit.
fn remove_feature(features: &mut Vec<DesignFeature>, id: u32) -> Option<(usize, DesignFeature)> {
    let index = features.iter().position(|feature| feature.id == id)?;
    Some((index, features.remove(index)))
}

/// Reapply one command removed from the undo stack. Conflicting/missing
/// identities fail closed instead of silently corrupting edit history.
fn redo_last_edit(features: &mut Vec<DesignFeature>, command: EditCommand) -> Result<u32, String> {
    match command {
        EditCommand::Added { feature } => {
            if features.iter().any(|existing| existing.id == feature.id) {
                return Err(format!(
                    "cannot redo add: feature ID {} already exists",
                    feature.id
                ));
            }
            features.push(feature);
            Ok(feature.id)
        }
        EditCommand::Deleted { feature, .. } => {
            let Some(index) = features
                .iter()
                .position(|existing| existing.id == feature.id)
            else {
                return Err(format!(
                    "cannot redo delete: feature ID {} is missing",
                    feature.id
                ));
            };
            if features[index] != feature {
                return Err(format!(
                    "cannot redo delete: feature ID {} no longer matches the recorded command",
                    feature.id
                ));
            }
            features.remove(index);
            Ok(feature.id)
        }
    }
}

/// Reverse one edit command. Deletion undo restores the element to its original
/// position; addition undo removes only the feature created by that command.
fn undo_last_edit(features: &mut Vec<DesignFeature>, command: EditCommand) -> u32 {
    match command {
        EditCommand::Added { feature } => {
            let _ = remove_feature(features, feature.id);
            feature.id
        }
        EditCommand::Deleted { feature, index } => {
            let insert_at = index.min(features.len());
            features.insert(insert_at, feature);
            feature.id
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ImportedSnapshot {
    project_name: String,
    scale: LandscapeScale,
    features: Vec<DesignFeature>,
    next_feature_id: u32,
}

/// Parse only user-authored native v0 projects. Synthetic benchmark snapshots
/// are valid benchmark inputs but must not silently become design-intent data.
fn parse_import_snapshot(value: &Value) -> Result<ImportedSnapshot, String> {
    validate_snapshot(value)
        .map_err(|errors| format!("Invalid snapshot: {}", errors.join("; ")))?;

    if value["geometry_status"].as_str() != Some(GEOMETRY_STATUS) {
        return Err(
            "Synthetic benchmark snapshots cannot be imported as user-authored projects"
                .to_string(),
        );
    }

    let project_name = value["project_name"]
        .as_str()
        .ok_or_else(|| "Project name is missing".to_string())
        .map(normalized_project_name)?;
    let scale = LandscapeScale::from_key(
        value["scope"]
            .as_str()
            .ok_or_else(|| "Project scope is missing".to_string())?,
    )
    .ok_or_else(|| "Project scope is unsupported".to_string())?;
    let values = value["features"]
        .as_array()
        .ok_or_else(|| "Project features must be an array".to_string())?;

    let mut features = Vec::with_capacity(values.len());
    for (index, feature) in values.iter().enumerate() {
        let id_text = feature["id"]
            .as_str()
            .ok_or_else(|| format!("Feature {index} has no ID"))?;
        let id = id_text
            .strip_prefix("feature-")
            .and_then(|digits| digits.parse::<u32>().ok())
            .filter(|id| *id < u32::MAX)
            .ok_or_else(|| format!("Feature {index} ID is outside the v0 editor identity range"))?;
        let kind_key = feature["kind"]
            .as_str()
            .ok_or_else(|| format!("Feature {index} has no kind"))?;
        let kind = FeatureKind::from_key(kind_key)
            .ok_or_else(|| format!("Feature {index} has an unsupported kind"))?;
        let x_du = feature["x_du"]
            .as_u64()
            .and_then(|number| u16::try_from(number).ok())
            .ok_or_else(|| format!("Feature {index} has an invalid x coordinate"))?;
        let y_du = feature["y_du"]
            .as_u64()
            .and_then(|number| u16::try_from(number).ok())
            .ok_or_else(|| format!("Feature {index} has an invalid y coordinate"))?;
        features.push(DesignFeature {
            id,
            x_du,
            y_du,
            kind,
        });
    }

    let next_feature_id = features
        .iter()
        .map(|feature| feature.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .filter(|id| *id < u32::MAX)
        .ok_or_else(|| "No safe feature ID remains for a subsequent edit".to_string())?;

    Ok(ImportedSnapshot {
        project_name,
        scale,
        features,
        next_feature_id,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LandscapeMetricCard {
    id: String,
    family: String,
    label: String,
    unit: String,
    status: String,
    target_policy: String,
    required_inputs: Vec<String>,
}

/// Read only metric definitions. This deliberately fails closed if a future
/// registry adds values or loses the v0 schema/status contract.
fn landscape_metric_cards() -> Result<Vec<LandscapeMetricCard>, String> {
    let registry: Value = serde_json::from_str(include_str!(
        "../../metrics/landscape/registry-v0.json"
    ))
    .map_err(|error| format!("Performance registry is invalid JSON: {error}"))?;

    if registry["schema"] != "luminous.landscape-performance-registry.v0"
        || registry["schema_uri"]
            != "https://luminousdynamics.io/schemas/landscape-performance-registry-v0.schema.json"
        || registry["status"] != "specification-only-no-field-values"
    {
        return Err(
            "Performance registry does not match the versioned definition-only contract"
                .to_string(),
        );
    }

    let entries = registry["metrics"]
        .as_array()
        .ok_or_else(|| "Performance registry has no metrics array".to_string())?;
    if entries.is_empty() || entries.len() > 100 {
        return Err("Performance registry metric count is outside the supported v0 bounds".to_string());
    }

    let mut cards = Vec::with_capacity(entries.len());
    let mut ids = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        if entry.get("value").is_some() || entry.get("measured_value").is_some() {
            return Err(format!(
                "Performance metric at index {index} embeds a value; v0 accepts definitions only"
            ));
        }

        let read_string = |key: &str| -> Result<String, String> {
            entry
                .get(key)
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string)
                .ok_or_else(|| format!("Performance metric {index} has no valid {key}"))
        };

        let id = read_string("id")?;
        if ids.contains(&id) {
            return Err(format!("Performance registry contains duplicate metric ID {id}"));
        }
        ids.push(id.clone());

        let required_values = entry["required_inputs"]
            .as_array()
            .filter(|values| !values.is_empty())
            .ok_or_else(|| format!("Performance metric {id} has no required_inputs"))?;
        let mut required_inputs = Vec::with_capacity(required_values.len());
        for (input_index, input) in required_values.iter().enumerate() {
            let value = input.as_str().filter(|text| !text.trim().is_empty()).ok_or_else(|| {
                format!("Performance metric {id} has invalid required input at index {input_index}")
            })?;
            required_inputs.push(value.to_string());
        }

        let status = read_string("v0_status")?;
        if !matches!(
            status.as_str(),
            "not-computable" | "requires-myclix-stewardship-contract"
        ) {
            return Err(format!(
                "Performance metric {id} has unsupported v0 status {status:?}"
            ));
        }

        cards.push(LandscapeMetricCard {
            id,
            family: read_string("family")?,
            label: read_string("label")?,
            unit: read_string("unit")?,
            status,
            target_policy: read_string("target_policy")?,
            required_inputs,
        });
    }

    Ok(cards)
}

fn landscape_source_status() -> Result<(usize, usize), String> {
    let catalog: Value = serde_json::from_str(include_str!(
        "../../data-sources/landscape/south-africa-johannesburg-v0.json"
    ))
    .map_err(|error| format!("Source catalog is invalid JSON: {error}"))?;
    let records = catalog["records"]
        .as_array()
        .ok_or_else(|| "Source catalog has no records array".to_string())?;
    let ingested = records
        .iter()
        .filter(|record| record["ingestion_state"].as_str() == Some("ingested"))
        .count();
    Ok((records.len(), ingested))
}

fn normalized_project_name(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "Untitled landscape".to_string()
    } else {
        trimmed.to_string()
    }
}

fn snapshot_json(project_name: &str, scale: LandscapeScale, features: &[DesignFeature]) -> Value {
    json!({
        "schema": SNAPSHOT_SCHEMA,
        "schema_uri": SNAPSHOT_SCHEMA_URI,
        "project_name": project_name,
        "scope": scale.key(),
        "coordinate_frame": {
            "kind": FRAME_KIND,
            "origin": "canvas top-left",
            "x_axis": "right",
            "y_axis": "down",
            "georeferenced": false
        },
        "geometry_status": GEOMETRY_STATUS,
        "features": features.iter().map(|feature| json!({
            "id": format!("feature-{:04}", feature.id),
            "kind": feature.kind.key(),
            "label": feature.kind.label(),
            "x_du": feature.x_du,
            "y_du": feature.y_du,
            "status": DESIGN_INTENT_STATUS,
            "source": USER_AUTHORED_SOURCE,
            "geometry_representation": GEOMETRY_REPRESENTATION
        })).collect::<Vec<_>>()
    })
}

const LOCAL_DRAFT_STORAGE_KEY: &str = "luminous.landscape-design.native-v0";
const MAX_LOCAL_DRAFT_BYTES: usize = 2 * 1024 * 1024;

fn validate_local_draft_size(content: &str) -> Result<(), String> {
    if content.len() > MAX_LOCAL_DRAFT_BYTES {
        return Err(format!(
            "draft is {} bytes, above the v0 browser-storage limit of {} bytes; download the JSON file instead",
            content.len(),
            MAX_LOCAL_DRAFT_BYTES
        ));
    }
    Ok(())
}

/// Explicit, same-browser persistence. localStorage is not encrypted and may be
/// cleared by the user/browser; users should download a JSON backup for portability.
fn save_local_draft(content: &str) -> Result<(), String> {
    validate_local_draft_size(content)?;
    let window = web_sys::window().ok_or_else(|| "Browser window is unavailable".to_string())?;
    let storage = window
        .local_storage()
        .map_err(|error| format!("Could not access browser storage: {error:?}"))?
        .ok_or_else(|| "This browser does not expose local storage".to_string())?;
    storage
        .set_item(LOCAL_DRAFT_STORAGE_KEY, content)
        .map_err(|error| {
            format!("Could not save local draft (storage may be full or blocked): {error:?}")
        })
}

fn load_local_draft() -> Result<Option<String>, String> {
    let window = web_sys::window().ok_or_else(|| "Browser window is unavailable".to_string())?;
    let storage = window
        .local_storage()
        .map_err(|error| format!("Could not access browser storage: {error:?}"))?
        .ok_or_else(|| "This browser does not expose local storage".to_string())?;
    let content = storage
        .get_item(LOCAL_DRAFT_STORAGE_KEY)
        .map_err(|error| format!("Could not read local draft: {error:?}"))?;
    if let Some(content) = content.as_ref() {
        validate_local_draft_size(content).map_err(|error| {
            format!("saved {error}; download a JSON backup and use file import")
        })?;
    }
    Ok(content)
}

/// Download a native JSON snapshot using a browser Blob (not a data URI).
/// Blob URLs avoid the practical URL-length limits of embedding the whole
/// project in an anchor's href.
fn download_snapshot(content: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "Browser window is unavailable".to_string())?;
    let document = window
        .document()
        .ok_or_else(|| "Browser document is unavailable".to_string())?;
    let body = document
        .body()
        .ok_or_else(|| "Browser document has no body".to_string())?;

    let parts = js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(content));
    let blob = web_sys::Blob::new_with_str_sequence(&parts)
        .map_err(|error| format!("Could not create project Blob: {error:?}"))?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|error| format!("Could not create download URL: {error:?}"))?;

    let activation: Result<(), String> = (|| {
        let element = document
            .create_element("a")
            .map_err(|error| format!("Could not create download link: {error:?}"))?;
        let anchor = element
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .map_err(|_| "Download element was not an anchor".to_string())?;
        anchor.set_href(&url);
        anchor.set_download("landscape-design.json");
        body.append_child(&anchor)
            .map_err(|error| format!("Could not attach download link: {error:?}"))?;
        anchor.click();
        anchor.remove();
        Ok(())
    })();

    let cleanup = web_sys::Url::revoke_object_url(&url)
        .map_err(|error| format!("Could not release download URL: {error:?}"));
    activation?;
    cleanup?;
    Ok(())
}

#[component]
pub fn LandscapeDesigner() -> impl IntoView {
    let metric_cards = landscape_metric_cards();
    let (features, set_features) = signal(starter_features());
    let (edit_history, set_edit_history) = signal(Vec::<EditCommand>::new());
    let (redo_history, set_redo_history) = signal(Vec::<EditCommand>::new());
    let (selected_feature_id, set_selected_feature_id) = signal(None::<u32>);
    let (project_name, set_project_name) = signal("Untitled landscape".to_string());
    // Never reuse a feature ID after undo: downstream provenance will rely on
    // IDs continuing to identify one authored object within an edit session.
    let (next_feature_id, set_next_feature_id) = signal(8_u32);
    let (active_tool, set_active_tool) = signal(FeatureKind::Tree);
    let (active_scale, set_active_scale) = signal(LandscapeScale::Site);
    let (show_grid, set_show_grid) = signal(true);
    let (show_context, set_show_context) = signal(true);
    let (export_text, set_export_text) = signal(String::new());
    let (download_status, set_download_status) = signal(String::new());

    let place_feature = move |event: web_sys::MouseEvent| {
        // Use the SVG element's measured bounds so pointer placement remains
        // correct when the responsive canvas is scaled by CSS.
        if let Some(target) = event.current_target() {
            let element: web_sys::Element = target.unchecked_into();
            let rect = element.get_bounding_client_rect();
            if rect.width() > 0.0 && rect.height() > 0.0 {
                let x = ((f64::from(event.client_x()) - rect.left()) * 1000.0 / rect.width())
                    .clamp(0.0, 1000.0)
                    .round() as u16;
                let y = ((f64::from(event.client_y()) - rect.top()) * 650.0 / rect.height())
                    .clamp(0.0, 650.0)
                    .round() as u16;
                let next_id = next_feature_id.get_untracked();
                if next_id == u32::MAX {
                    set_download_status.set("No safe feature IDs remain; export this project and start a new v0 project.".to_string());
                    return;
                }
                set_next_feature_id.update(|id| *id = id.saturating_add(1));
                let feature = DesignFeature {
                    id: next_id,
                    x_du: x,
                    y_du: y,
                    kind: active_tool.get_untracked(),
                };
                set_features.update(|items| items.push(feature));
                set_edit_history.update(|history| history.push(EditCommand::Added { feature }));
                set_redo_history.set(Vec::new());
                set_selected_feature_id.set(Some(next_id));
                set_export_text.set(String::new());
                set_download_status.set(String::new());
            }
        }
    };

    let open_import = move |_| {
        let input = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("landscape-import-file"))
            .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok());
        match input {
            Some(input) => input.click(),
            None => set_download_status.set("Could not open the local file picker.".to_string()),
        }
    };

    let import_project = move |event: web_sys::Event| {
        let input = leptos::prelude::event_target::<web_sys::HtmlInputElement>(&event);
        let Some(file) = input.files().and_then(|files| files.get(0)) else {
            return;
        };
        const MAX_IMPORT_BYTES: f64 = 64.0 * 1024.0 * 1024.0;
        if file.size() > MAX_IMPORT_BYTES {
            input.set_value("");
            set_download_status
                .set("Import blocked: file exceeds the 64 MiB v0 limit.".to_string());
            return;
        }

        let file_name = file.name();
        let promise = file.text();
        input.set_value("");
        set_download_status.set(format!("Reading and validating {file_name}…"));

        wasm_bindgen_futures::spawn_local(async move {
            let parsed = match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(value) => value
                    .as_string()
                    .ok_or_else(|| "Browser returned non-text file content".to_string())
                    .and_then(|text| {
                        serde_json::from_str::<Value>(&text)
                            .map_err(|error| format!("Invalid JSON: {error}"))
                    })
                    .and_then(|value| parse_import_snapshot(&value)),
                Err(error) => Err(format!("Could not read local file: {error:?}")),
            };

            match parsed {
                Ok(imported) => {
                    let feature_count = imported.features.len();
                    set_features.set(imported.features);
                    set_project_name.set(imported.project_name);
                    set_active_scale.set(imported.scale);
                    set_next_feature_id.set(imported.next_feature_id);
                    set_selected_feature_id.set(None);
                    set_edit_history.set(Vec::new());
                    set_redo_history.set(Vec::new());
                    set_export_text.set(String::new());
                    set_download_status.set(format!(
                        "Imported and validated {feature_count} schematic elements. No geospatial or physical validation is implied."
                    ));
                }
                Err(error) => {
                    set_download_status.set(format!(
                        "Import blocked; current project preserved: {error}"
                    ));
                }
            }
        });
    };

    let export_project = move |_| {
        let name = normalized_project_name(&project_name.get_untracked());
        let value = snapshot_json(
            &name,
            active_scale.get_untracked(),
            &features.get_untracked(),
        );
        match validate_snapshot(&value) {
            Ok(()) => {
                set_export_text.set(
                    serde_json::to_string_pretty(&value)
                        .unwrap_or_else(|error| format!("Snapshot serialization failed: {error}")),
                );
                set_download_status.set(String::new());
            }
            Err(errors) => {
                set_export_text.set(String::new());
                set_download_status.set(format!(
                    "Snapshot validation failed; download blocked: {}",
                    errors.join("; ")
                ));
            }
        }
    };

    let download_project = move |_| {
        let content = export_text.get_untracked();
        if content.is_empty() {
            set_download_status.set("Generate a snapshot before downloading.".to_string());
            return;
        }
        set_download_status.set(match download_snapshot(&content) {
            Ok(()) => "Download requested: landscape-design.json".to_string(),
            Err(error) => format!("Download failed: {error}"),
        });
    };

    let save_browser_project = move |_| {
        let name = normalized_project_name(&project_name.get_untracked());
        let value = snapshot_json(
            &name,
            active_scale.get_untracked(),
            &features.get_untracked(),
        );
        match validate_snapshot(&value) {
            Err(errors) => set_download_status.set(format!(
                "Local save blocked by snapshot validation: {}",
                errors.join("; ")
            )),
            Ok(()) => match serde_json::to_string_pretty(&value) {
                Err(error) => {
                    set_download_status.set(format!("Could not serialize local draft: {error}"))
                }
                Ok(content) => match save_local_draft(&content) {
                    Ok(()) => {
                        set_export_text.set(content);
                        set_download_status.set(
                            "Saved in this browser only. Storage is not encrypted; download a JSON backup for portability."
                                .to_string(),
                        );
                    }
                    Err(error) => set_download_status.set(format!("Local save failed: {error}")),
                },
            },
        }
    };

    let load_browser_project = move |_| match load_local_draft() {
        Err(error) => set_download_status.set(format!("Local load failed: {error}")),
        Ok(None) => set_download_status
            .set("No browser-local landscape draft has been saved yet.".to_string()),
        Ok(Some(content)) => {
            let result = serde_json::from_str::<Value>(&content)
                .map_err(|error| format!("Saved draft is invalid JSON: {error}"))
                .and_then(|value| parse_import_snapshot(&value));
            match result {
                Err(error) => set_download_status.set(format!(
                    "Local load blocked; current project preserved: {error}"
                )),
                Ok(imported) => {
                    let feature_count = imported.features.len();
                    set_features.set(imported.features);
                    set_project_name.set(imported.project_name);
                    set_active_scale.set(imported.scale);
                    set_next_feature_id.set(imported.next_feature_id);
                    set_selected_feature_id.set(None);
                    set_edit_history.set(Vec::new());
                    set_redo_history.set(Vec::new());
                    set_export_text.set(content);
                    set_download_status.set(format!(
                        "Loaded and validated {feature_count} schematic elements from this browser. No geospatial or physical validation is implied."
                    ));
                }
            }
        }
    };

    let undo = move |_| {
        let Some(command) = edit_history.get_untracked().last().copied() else {
            return;
        };
        set_edit_history.update(|history| {
            history.pop();
        });
        let mut current_features = features.get_untracked();
        let affected_id = undo_last_edit(&mut current_features, command);
        set_features.set(current_features);
        set_redo_history.update(|history| history.push(command));
        match command {
            EditCommand::Added { .. } => {
                if selected_feature_id.get_untracked() == Some(affected_id) {
                    set_selected_feature_id.set(None);
                }
            }
            EditCommand::Deleted { .. } => set_selected_feature_id.set(Some(affected_id)),
        }
        set_export_text.set(String::new());
        set_download_status.set(String::new());
    };

    let redo = move |_| {
        let Some(command) = redo_history.get_untracked().last().copied() else {
            return;
        };
        let mut current_features = features.get_untracked();
        match redo_last_edit(&mut current_features, command) {
            Ok(affected_id) => {
                set_redo_history.update(|history| {
                    history.pop();
                });
                set_edit_history.update(|history| history.push(command));
                set_features.set(current_features);
                match command {
                    EditCommand::Added { .. } => set_selected_feature_id.set(Some(affected_id)),
                    EditCommand::Deleted { .. } => {
                        if selected_feature_id.get_untracked() == Some(affected_id) {
                            set_selected_feature_id.set(None);
                        }
                    }
                }
                set_export_text.set(String::new());
                set_download_status.set(String::new());
            }
            Err(error) => {
                set_download_status.set(format!("Redo blocked: {error}"));
            }
        }
    };

    let delete_selected = move |_| {
        if let Some(id) = selected_feature_id.get_untracked() {
            let mut removed = None;
            set_features.update(|items| {
                removed = remove_feature(items, id);
            });
            if let Some((index, feature)) = removed {
                set_edit_history.update(|history| {
                    history.push(EditCommand::Deleted { feature, index });
                });
                set_redo_history.set(Vec::new());
            }
            set_selected_feature_id.set(None);
            set_export_text.set(String::new());
            set_download_status.set(String::new());
        }
    };

    let selected_description = move || {
        let id = selected_feature_id.get();
        let items = features.get();
        match id.and_then(|selected_id| items.iter().find(|feature| feature.id == selected_id)) {
            Some(feature) => format!(
                "{} · anchor ({}, {})",
                feature.kind.label(),
                feature.x_du,
                feature.y_du
            ),
            None => "No element selected".to_string(),
        }
    };

    let reset = move |_| {
        set_features.set(starter_features());
        set_edit_history.set(Vec::new());
        set_redo_history.set(Vec::new());
        set_next_feature_id.set(8);
        set_selected_feature_id.set(None);
        set_project_name.set("Untitled landscape".to_string());
        set_active_tool.set(FeatureKind::Tree);
        set_active_scale.set(LandscapeScale::Site);
        set_show_grid.set(true);
        set_show_context.set(true);
        set_export_text.set(String::new());
        set_download_status.set(String::new());
    };

    view! {
        <main class="landscape-workbench" on:keydown=move |event: web_sys::KeyboardEvent| {
            if let Some(target) = event.target() {
                let element: web_sys::Element = target.unchecked_into();
                let tag = element.tag_name();
                if tag == "INPUT" || tag == "TEXTAREA" { return; }
            }
            match event.key().to_lowercase().as_str() {
                "t" => set_active_tool.set(FeatureKind::Tree),
                "p" => set_active_tool.set(FeatureKind::PlantingBed),
                "w" => set_active_tool.set(FeatureKind::Water),
                "l" => set_active_tool.set(FeatureKind::Path),
                "b" => set_active_tool.set(FeatureKind::BuiltElement),
                _ => return,
            }
            event.prevent_default();
        }>
            <header class="landscape-topbar">
                <div class="landscape-brand">
                    <A href="/" attr:class="landscape-back">"← Sol Atlas"</A>
                    <div>
                        <p class="landscape-kicker">"LUMINOUS DYNAMICS / LAND SYSTEMS"</p>
                        <h1>"Landscape Workbench"</h1>
                    </div>
                </div>
                <div class="landscape-top-actions">
                    <span class="landscape-draft-mark"><span></span>" SCHEMATIC DRAFT"</span>
                    <button class="landscape-button quiet" type="button" disabled=move || edit_history.get().is_empty() on:click=undo>"Undo"</button>
                    <button class="landscape-button quiet" type="button" disabled=move || redo_history.get().is_empty() on:click=redo>"Redo"</button>
                    <button class="landscape-button quiet" type="button" on:click=load_browser_project>"Load browser draft"</button>
                    <button class="landscape-button quiet" type="button" on:click=save_browser_project>"Save in browser"</button>
                    <button class="landscape-button quiet" type="button" on:click=open_import>"Import JSON"</button>
                    <button class="landscape-button primary" type="button" on:click=export_project>"Export snapshot"</button>
                </div>
            </header>
            <input
                id="landscape-import-file"
                type="file"
                accept=".json,application/json"
                style="display:none"
                aria-label="Import a saved landscape project JSON"
                on:change=import_project
            />

            <div class="landscape-layout">
                <aside class="landscape-sidebar" aria-label="Design tools and project scope">
                    <section class="landscape-section">
                        <p class="landscape-section-label">"01 / DESIGN SCALE"</p>
                        <h2>"Think in systems"</h2>
                        <label class="landscape-project-field">
                            <span>"PROJECT NAME"</span>
                            <input type="text" maxlength="200" prop:value=move || project_name.get() on:input=move |event| {
                                set_project_name.set(event_target_value(&event));
                                set_export_text.set(String::new());
                                set_download_status.set(String::new());
                            }/>
                        </label>
                        <p class="landscape-muted">"Switch planning context from a single property to its wider living landscape. This changes the declared scope, not the coordinate frame."</p>
                        <div class="landscape-scale-list">
                            <button type="button" class:chosen=move || active_scale.get() == LandscapeScale::Site on:click=move |_| set_active_scale.set(LandscapeScale::Site)><span>"01"</span>"Site / parcel"</button>
                            <button type="button" class:chosen=move || active_scale.get() == LandscapeScale::District on:click=move |_| set_active_scale.set(LandscapeScale::District)><span>"02"</span>"District"</button>
                            <button type="button" class:chosen=move || active_scale.get() == LandscapeScale::Watershed on:click=move |_| set_active_scale.set(LandscapeScale::Watershed)><span>"03"</span>"Watershed"</button>
                            <button type="button" class:chosen=move || active_scale.get() == LandscapeScale::Bioregion on:click=move |_| set_active_scale.set(LandscapeScale::Bioregion)><span>"04"</span>"Bioregion"</button>
                        </div>
                    </section>

                    <section class="landscape-section">
                        <p class="landscape-section-label">"02 / PLACE ELEMENTS"</p>
                        <h2>"Design palette"</h2>
                        <p class="landscape-muted">"Choose an element, then click the plan to place it."</p>
                        <div class="landscape-tool-list">
                            <button type="button" class:chosen=move || active_tool.get() == FeatureKind::Tree on:click=move |_| set_active_tool.set(FeatureKind::Tree) aria-pressed=move || (active_tool.get() == FeatureKind::Tree).to_string()><span class="tool-swatch tree-swatch"></span><span>"Canopy tree"</span><kbd>"T"</kbd></button>
                            <button type="button" class:chosen=move || active_tool.get() == FeatureKind::PlantingBed on:click=move |_| set_active_tool.set(FeatureKind::PlantingBed) aria-pressed=move || (active_tool.get() == FeatureKind::PlantingBed).to_string()><span class="tool-swatch bed-swatch"></span><span>"Planting bed"</span><kbd>"P"</kbd></button>
                            <button type="button" class:chosen=move || active_tool.get() == FeatureKind::Water on:click=move |_| set_active_tool.set(FeatureKind::Water) aria-pressed=move || (active_tool.get() == FeatureKind::Water).to_string()><span class="tool-swatch water-swatch"></span><span>"Water feature"</span><kbd>"W"</kbd></button>
                            <button type="button" class:chosen=move || active_tool.get() == FeatureKind::Path on:click=move |_| set_active_tool.set(FeatureKind::Path) aria-pressed=move || (active_tool.get() == FeatureKind::Path).to_string()><span class="tool-swatch path-swatch"></span><span>"Path / connection"</span><kbd>"L"</kbd></button>
                            <button type="button" class:chosen=move || active_tool.get() == FeatureKind::BuiltElement on:click=move |_| set_active_tool.set(FeatureKind::BuiltElement) aria-pressed=move || (active_tool.get() == FeatureKind::BuiltElement).to_string()><span class="tool-swatch built-swatch"></span><span>"Built element"</span><kbd>"B"</kbd></button>
                        </div>
                    </section>

                    <section class="landscape-section">
                        <p class="landscape-section-label">"03 / DRAWING LAYERS"</p>
                        <h2>"Reading the site"</h2>
                        <label class="landscape-toggle"><input type="checkbox" prop:checked=move || show_grid.get() on:change=move |_| set_show_grid.update(|visible| *visible = !*visible)/><span>"Measurement grid"</span></label>
                        <label class="landscape-toggle"><input type="checkbox" prop:checked=move || show_context.get() on:change=move |_| set_show_context.update(|visible| *visible = !*visible)/><span>"Illustrative contours"</span></label>
                        <p class="landscape-note">"Context lines are decorative teaching geometry, not a terrain survey or hydrology result."</p>
                    </section>

                    <section class="landscape-section">
                        <p class="landscape-section-label">"04 / EVIDENCE READINESS"</p>
                        <h2>"What do we know?"</h2>
                        {move || match landscape_source_status() {
                            Ok((catalogued, ingested)) => view! {
                                <div class="landscape-evidence-counts">
                                    <div><strong>{catalogued.to_string()}</strong><span>"candidate sources"</span></div>
                                    <div><strong>{ingested.to_string()}</strong><span>"layers ingested"</span></div>
                                </div>
                                <p class="landscape-evidence-blocked">
                                    {if ingested == 0 {
                                        "SCREENING BLOCKED · no source layers are ingested".to_string()
                                    } else {
                                        "SOURCE LAYERS PRESENT · per-model qualification still required".to_string()
                                    }}
                                </p>
                            }.into_any(),
                            Err(error) => view! {
                                <p class="landscape-evidence-blocked">"SOURCE CATALOG INVALID · " {error}</p>
                            }.into_any()
                        }}
                        <p class="landscape-muted">"A public URL is only a candidate source. Before a layer can support analysis, verify license/attribution, date and resolution, CRS/units, missing-data behavior, and an independent reference."</p>
                        <p class="landscape-note">"The canvas is design intent. No runoff, shade, biodiversity, carbon, cost, or accessibility result is calculated here."</p>
                    </section>

                    <section class="landscape-section">
                        <p class="landscape-section-label">"05 / INSPECT SELECTION"</p>
                        <h2>"Selected element"</h2>
                        <p class="landscape-muted">"Select a symbol on the canvas to inspect its current anchor and remove it from this schematic draft."</p>
                        <p class="landscape-selected-value">{move || selected_description()}</p>
                        <button class="landscape-button quiet full-width" type="button" disabled=move || selected_feature_id.get().is_none() on:click=delete_selected>"Delete selected"</button>
                    </section>
                    <section class="landscape-section landscape-summary">
                        <p class="landscape-section-label">"CURRENT PLAN"</p>
                        <div class="landscape-stat"><strong>{move || features.get().len().to_string()}</strong><span>"placed elements"</span></div>
                        <div class="landscape-stat"><strong>{move || active_scale.get().label()}</strong><span>"declared scale"</span></div>
                        <button class="landscape-button quiet full-width" type="button" on:click=reset>"Reset starter plan"</button>
                    </section>
                </aside>

                <section class="landscape-stage" aria-label="Landscape plan editor">
                    <div class="landscape-stage-head">
                        <div>
                            <p class="landscape-kicker">{move || format!("{} / CONCEPT 01", project_name.get().to_uppercase())}</p>
                            <h2>"Living systems plan"</h2>
                        </div>
                        <div class="landscape-stage-meta"><span>"LOCAL GRID"</span><span>"1000 × 650 schematic units"</span></div>
                    </div>
                    <div class="landscape-canvas-frame">
                        <svg class="landscape-canvas" viewBox="0 0 1000 650" tabindex="0" role="application" aria-label="Schematic landscape canvas. Click to place the selected design element; press Enter or Space to place at the center." on:click=place_feature on:keydown=move |event: web_sys::KeyboardEvent| {
                            if event.key() == "Enter" || event.key() == " " {
                                event.prevent_default();
                                let next_id = next_feature_id.get_untracked();
                                if next_id == u32::MAX {
                                    set_download_status.set("No safe feature IDs remain; export this project and start a new v0 project.".to_string());
                                    return;
                                }
                                set_next_feature_id.update(|id| *id = id.saturating_add(1));
                                let feature = DesignFeature {
                                    id: next_id,
                                    x_du: 500,
                                    y_du: 325,
                                    kind: active_tool.get_untracked(),
                                };
                                set_features.update(|items| items.push(feature));
                                set_edit_history.update(|history| history.push(EditCommand::Added { feature }));
                                set_redo_history.set(Vec::new());
                                set_selected_feature_id.set(Some(next_id));
                                set_export_text.set(String::new());
                                set_download_status.set(String::new());
                            }
                        }>
                            <defs>
                                <pattern id="landscape-grid" width="50" height="50" patternUnits="userSpaceOnUse">
                                    <path d="M 50 0 L 0 0 0 50" fill="none" stroke="#7f9384" stroke-width="1" stroke-opacity="0.22"/>
                                </pattern>
                            </defs>
                            <rect x="0" y="0" width="1000" height="650" fill="#e7e7d7"/>
                            <rect x="0" y="0" width="1000" height="650" fill="url(#landscape-grid)" style:opacity=move || if show_grid.get() { "1" } else { "0" }/>
                            <g style:opacity=move || if show_context.get() { "0.62" } else { "0" }>
                                <path d="M -20 130 C 180 55, 260 185, 430 125 S 720 65, 1020 145" fill="none" stroke="#a7b49c" stroke-width="2"/>
                                <path d="M -20 166 C 180 91, 260 221, 430 161 S 720 101, 1020 181" fill="none" stroke="#a7b49c" stroke-width="2"/>
                                <path d="M -20 202 C 180 127, 260 257, 430 197 S 720 137, 1020 217" fill="none" stroke="#a7b49c" stroke-width="2"/>
                                <path d="M 80 670 C 120 500, 230 470, 330 360 S 520 280, 650 340 S 820 540, 1020 440" fill="none" stroke="#9da995" stroke-width="2"/>
                                <path d="M 105 670 C 145 525, 245 495, 350 380 S 525 305, 665 365 S 835 565, 1020 465" fill="none" stroke="#9da995" stroke-width="2"/>
                            </g>
                            <path d="M 45 45 L 935 45 L 955 595 L 65 610 Z" fill="#a6b59b" fill-opacity="0.12" stroke="#6f8a76" stroke-width="3" stroke-dasharray="12 8"/>
                            <text x="70" y="80" fill="#566a5b" font-size="17" letter-spacing="3">"CONCEPTUAL SITE BOUNDARY"</text>
                            <For each=move || features.get() key=|feature| feature.id children=move |feature: DesignFeature| {
                                let x = feature.x_du.to_string();
                                let y = feature.y_du.to_string();
                                let left = feature.x_du.saturating_sub(17).to_string();
                                let top = feature.y_du.saturating_sub(12).to_string();
                                let title = feature.kind.label();
                                match feature.kind {
                                    FeatureKind::Tree => view! {
                                        <g
                                            class:landscape-feature-selected=move || selected_feature_id.get() == Some(feature.id)
                                            tabindex="0"
                                            role="button"
                                            aria-label=move || format!("Select {} at drawing coordinates ({}, {})", title, feature.x_du, feature.y_du)
                                            aria-pressed=move || selected_feature_id.get() == Some(feature.id)
                                            on:keydown=move |event: web_sys::KeyboardEvent| {
                                                if event.key() == "Enter" || event.key() == " " {
                                                    event.prevent_default();
                                                    event.stop_propagation();
                                                    set_selected_feature_id.set(Some(feature.id));
                                                }
                                            }
                                            on:click=move |event: web_sys::MouseEvent| {
                                                event.stop_propagation();
                                                set_selected_feature_id.set(Some(feature.id));
                                            }
                                        >
                                            <circle cx=x.clone() cy=y.clone() r="17" fill="#438b66" fill-opacity="0.78" stroke="#315d47" stroke-width="2"/>
                                            <circle cx=x.clone() cy=y.clone() r="7" fill="#b8d3a8" fill-opacity="0.75"/>
                                            <title>{title}</title>
                                        </g>
                                    }.into_any(),
                                    FeatureKind::PlantingBed => view! {
                                        <g
                                            class:landscape-feature-selected=move || selected_feature_id.get() == Some(feature.id)
                                            tabindex="0"
                                            role="button"
                                            aria-label=move || format!("Select {} at drawing coordinates ({}, {})", title, feature.x_du, feature.y_du)
                                            aria-pressed=move || selected_feature_id.get() == Some(feature.id)
                                            on:keydown=move |event: web_sys::KeyboardEvent| {
                                                if event.key() == "Enter" || event.key() == " " {
                                                    event.prevent_default();
                                                    event.stop_propagation();
                                                    set_selected_feature_id.set(Some(feature.id));
                                                }
                                            }
                                            on:click=move |event: web_sys::MouseEvent| {
                                                event.stop_propagation();
                                                set_selected_feature_id.set(Some(feature.id));
                                            }
                                        >
                                            <rect x=left.clone() y=top.clone() width="34" height="24" rx="5" fill="#86a96f" stroke="#4f744d" stroke-width="2"/>
                                            <path d=format!("M {} {} l 10 16 m 5 -16 l 10 16", feature.x_du.saturating_sub(12), feature.y_du.saturating_sub(8)) fill="none" stroke="#e6efd9" stroke-width="2"/>
                                            <title>{title}</title>
                                        </g>
                                    }.into_any(),
                                    FeatureKind::Water => view! {
                                        <g
                                            class:landscape-feature-selected=move || selected_feature_id.get() == Some(feature.id)
                                            tabindex="0"
                                            role="button"
                                            aria-label=move || format!("Select {} at drawing coordinates ({}, {})", title, feature.x_du, feature.y_du)
                                            aria-pressed=move || selected_feature_id.get() == Some(feature.id)
                                            on:keydown=move |event: web_sys::KeyboardEvent| {
                                                if event.key() == "Enter" || event.key() == " " {
                                                    event.prevent_default();
                                                    event.stop_propagation();
                                                    set_selected_feature_id.set(Some(feature.id));
                                                }
                                            }
                                            on:click=move |event: web_sys::MouseEvent| {
                                                event.stop_propagation();
                                                set_selected_feature_id.set(Some(feature.id));
                                            }
                                        >
                                            <path d=format!("M {} {} C {} {}, {} {}, {} {} C {} {}, {} {}, {} {}", feature.x_du.saturating_sub(18), feature.y_du, feature.x_du.saturating_sub(10), feature.y_du.saturating_sub(19), feature.x_du.saturating_add(8), feature.y_du.saturating_sub(16), feature.x_du.saturating_add(18), feature.y_du, feature.x_du.saturating_add(25), feature.y_du.saturating_add(15), feature.x_du.saturating_sub(12), feature.y_du.saturating_add(17), feature.x_du.saturating_sub(18), feature.y_du) fill="#5caab4" stroke="#347780" stroke-width="2"/>
                                            <title>{title}</title>
                                        </g>
                                    }.into_any(),
                                    FeatureKind::Path => view! {
                                        <g
                                            class:landscape-feature-selected=move || selected_feature_id.get() == Some(feature.id)
                                            tabindex="0"
                                            role="button"
                                            aria-label=move || format!("Select {} at drawing coordinates ({}, {})", title, feature.x_du, feature.y_du)
                                            aria-pressed=move || selected_feature_id.get() == Some(feature.id)
                                            on:keydown=move |event: web_sys::KeyboardEvent| {
                                                if event.key() == "Enter" || event.key() == " " {
                                                    event.prevent_default();
                                                    event.stop_propagation();
                                                    set_selected_feature_id.set(Some(feature.id));
                                                }
                                            }
                                            on:click=move |event: web_sys::MouseEvent| {
                                                event.stop_propagation();
                                                set_selected_feature_id.set(Some(feature.id));
                                            }
                                        >
                                            <path d=format!("M {} {} l 35 18", feature.x_du.saturating_sub(16), feature.y_du.saturating_sub(8)) fill="none" stroke="#c8b785" stroke-width="12" stroke-linecap="round"/>
                                            <path d=format!("M {} {} l 35 18", feature.x_du.saturating_sub(16), feature.y_du.saturating_sub(8)) fill="none" stroke="#f1e7c6" stroke-width="2" stroke-linecap="round"/>
                                            <title>{title}</title>
                                        </g>
                                    }.into_any(),
                                    FeatureKind::BuiltElement => view! {
                                        <g
                                            class:landscape-feature-selected=move || selected_feature_id.get() == Some(feature.id)
                                            tabindex="0"
                                            role="button"
                                            aria-label=move || format!("Select {} at drawing coordinates ({}, {})", title, feature.x_du, feature.y_du)
                                            aria-pressed=move || selected_feature_id.get() == Some(feature.id)
                                            on:keydown=move |event: web_sys::KeyboardEvent| {
                                                if event.key() == "Enter" || event.key() == " " {
                                                    event.prevent_default();
                                                    event.stop_propagation();
                                                    set_selected_feature_id.set(Some(feature.id));
                                                }
                                            }
                                            on:click=move |event: web_sys::MouseEvent| {
                                                event.stop_propagation();
                                                set_selected_feature_id.set(Some(feature.id));
                                            }
                                        >
                                            <rect x=left.clone() y=top.clone() width="34" height="24" rx="2" fill="#b4aaa0" stroke="#716961" stroke-width="2"/>
                                            <path d=format!("M {} {} l 17 -11 l 17 11", feature.x_du.saturating_sub(17), feature.y_du.saturating_sub(12)) fill="none" stroke="#716961" stroke-width="2"/>
                                            <title>{title}</title>
                                        </g>
                                    }.into_any(),
                                }
                            }/>
                            <g transform="translate(900 75)">
                                <path d="M 0 22 L 0 -12 M -7 -2 L 0 -14 L 7 -2" fill="none" stroke="#4d6356" stroke-width="3"/>
                                <text x="0" y="39" text-anchor="middle" font-size="14" fill="#4d6356">"N"</text>
                            </g>
                        </svg>
                        <div class="landscape-canvas-legend"><span><i class="legend-tree"></i>"Canopy"</span><span><i class="legend-bed"></i>"Planting"</span><span><i class="legend-water"></i>"Water"</span><span><i class="legend-path"></i>"Circulation"</span><span><i class="legend-built"></i>"Built"</span></div>
                    </div>
                    <footer class="landscape-stage-footer">
                        <span><i class="status-dot"></i>"Editor ready"</span>
                        <span>"Selected tool: " {move || active_tool.get().label()}</span>
                        <span>"Local drawing units · not georeferenced"</span>
                        <span>"Focus plan + Enter/Space places at center · T/P/W/L/B switch tools"</span>
                    </footer>
                    <section class="landscape-export-panel">
                        <div class="landscape-export-header"><div><p class="landscape-section-label">"PORTABLE PROJECT SNAPSHOT"</p><h3>"Draft data"</h3></div><div class="landscape-export-actions"><p>"Native JSON · schema v0"</p><button class="landscape-button quiet" type="button" on:click=download_project>"Download JSON"</button></div></div>
                        {move || if export_text.get().is_empty() {
                            view! { <p class="landscape-muted">"Export a portable snapshot or explicitly save one draft in this browser. Browser storage is not encrypted and may be cleared; do not save sensitive land or participant details on a shared device."</p> }.into_any()
                        } else {
                            view! { <pre class="landscape-json">{move || export_text.get()}</pre> }.into_any()
                        }}
                        <p class="landscape-download-status" role="status" aria-live="polite">{move || download_status.get()}</p>
                    </section>

                    <section class="landscape-metric-panel" aria-label="Landscape performance metric registry">
                        <div class="landscape-metric-panel-head">
                            <div>
                                <p class="landscape-section-label">"OUTCOME METRICS / EVIDENCE CONTRACT"</p>
                                <h3>"What would success mean?"</h3>
                            </div>
                            <span class="landscape-metric-status-badge">"DEFINITION ONLY"</span>
                        </div>
                        <p class="landscape-muted">"No environmental or social outcome is calculated for this schematic plan. These versioned definitions show which evidence would be needed before any metric could be reported."</p>
                        {match metric_cards.clone() {
                            Ok(metrics) => {
                                let definition_count = metrics.len().to_string();
                                view! {
                                    <>
                                        <div class="landscape-metric-summary">
                                            <strong>{definition_count}</strong>
                                            <span>"metric definitions"</span>
                                            <span class="landscape-metric-zero">"0 values computed by v0"</span>
                                        </div>
                                        <div class="landscape-metric-grid">
                                            <For
                                                each=move || metrics.clone()
                                                key=|metric| metric.id.clone()
                                                children=move |metric: LandscapeMetricCard| {
                                                    let family = metric.family.clone();
                                                    let label = metric.label.clone();
                                                    let unit = metric.unit.clone();
                                                    let status = match metric.status.as_str() {
                                                        "requires-myclix-stewardship-contract" => "Needs stewardship contract",
                                                        _ => "Not computed",
                                                    };
                                                    let policy = metric.target_policy.clone();
                                                    let required_inputs = metric.required_inputs.clone();
                                                    let input_count = required_inputs.len().to_string();
                                                    view! {
                                                        <article class="landscape-metric-card">
                                                            <div class="landscape-metric-card-meta">
                                                                <span class="landscape-metric-family">{family}</span>
                                                                <span class="landscape-metric-card-status">{status}</span>
                                                            </div>
                                                            <h4>{label}</h4>
                                                            <p class="landscape-metric-unit">"Unit: " {unit}</p>
                                                            <details>
                                                                <summary>"Required inputs (" {input_count} ")"</summary>
                                                                <ul>
                                                                    <For
                                                                        each=move || required_inputs.clone()
                                                                        key=|input| input.clone()
                                                                        children=move |input: String| view! { <li>{input}</li> }
                                                                    />
                                                                </ul>
                                                            </details>
                                                            <details>
                                                                <summary>"Target / decision policy"</summary>
                                                                <p class="landscape-metric-policy">{policy}</p>
                                                            </details>
                                                        </article>
                                                    }
                                                }
                                            />
                                        </div>
                                    </>
                                }.into_any()
                            }
                            Err(error) => view! {
                                <p class="landscape-evidence-blocked" role="status">
                                    "METRIC REGISTRY INVALID · no outcome definitions shown. " {error}
                                </p>
                            }.into_any()
                        }}
                    </section>
                </section>
            </div>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_registry_ui_contract_is_definition_only() {
        let cards = landscape_metric_cards().unwrap();
        assert_eq!(cards.len(), 8);
        assert_eq!(cards.iter().map(|card| card.id.as_str()).collect::<HashSet<_>>().len(), 8);
        assert_eq!(cards.iter().map(|card| card.family.as_str()).collect::<HashSet<_>>().len(), 7);
        assert!(cards.iter().all(|card| !card.required_inputs.is_empty()));
        assert!(cards.iter().all(|card| !card.target_policy.is_empty()));
    }

    #[test]
    fn evidence_readiness_reports_catalogued_candidates_not_ingested_layers() {
        assert_eq!(landscape_source_status().unwrap(), (6, 0));
    }

    #[test]
    fn browser_storage_draft_limit_is_explicitly_bounded() {
        assert_eq!(MAX_LOCAL_DRAFT_BYTES, 2 * 1024 * 1024);
        assert!(validate_local_draft_size("").is_ok());
        let at_limit = "x".repeat(MAX_LOCAL_DRAFT_BYTES);
        assert!(validate_local_draft_size(&at_limit).is_ok());
        let over_limit = "x".repeat(MAX_LOCAL_DRAFT_BYTES + 1);
        let error = validate_local_draft_size(&over_limit).unwrap_err();
        assert!(error.contains("above the v0 browser-storage limit"));
        assert!(error.contains("download the JSON file instead"));
    }

    #[test]
    fn native_import_restores_project_metadata_and_stable_feature_ids() {
        let value: Value = serde_json::from_str(include_str!(
            "../../fixtures/landscape/seed-landscape-v0.json"
        ))
        .unwrap();
        let imported = parse_import_snapshot(&value).unwrap();

        assert_eq!(imported.project_name, "Starter landscape (schematic)");
        assert_eq!(imported.scale, LandscapeScale::Site);
        assert_eq!(imported.features, starter_features());
        assert_eq!(imported.next_feature_id, 8);
    }

    #[test]
    fn native_import_rejects_synthetic_benchmark_provenance() {
        let value = crate::landscape::synthetic_benchmark_snapshot(3, 0x4c41_4e44).unwrap();
        let error = parse_import_snapshot(&value).unwrap_err();

        assert!(error.contains("Synthetic benchmark snapshots"));
    }

    #[test]
    fn native_import_rejects_ids_outside_editor_identity_range() {
        let mut value: Value = serde_json::from_str(include_str!(
            "../../fixtures/landscape/seed-landscape-v0.json"
        ))
        .unwrap();
        value["features"][0]["id"] = json!("feature-999999999999");

        let error = parse_import_snapshot(&value).unwrap_err();
        assert!(error.contains("Invalid snapshot"));
    }

    #[test]
    fn blank_project_names_fall_back_to_a_valid_default() {
        assert_eq!(normalized_project_name(""), "Untitled landscape");
        assert_eq!(normalized_project_name("   "), "Untitled landscape");
        assert_eq!(
            normalized_project_name("  Meadow Commons  "),
            "Meadow Commons"
        );
    }

    #[test]
    fn snapshot_declares_local_frame_and_never_claims_georeferencing() {
        let features = starter_features();
        let value = snapshot_json("fixture", LandscapeScale::Watershed, &features);
        assert_eq!(value["schema"], "luminous.landscape-design.v0");
        assert_eq!(value["project_name"], "fixture");
        assert_eq!(
            value["schema_uri"],
            "https://luminousdynamics.io/schemas/landscape-design-v0.schema.json"
        );
        assert_eq!(value["scope"], "watershed");
        assert_eq!(
            value["coordinate_frame"]["kind"],
            "LOCAL_CARTESIAN_DRAWING_UNITS"
        );
        assert_eq!(value["coordinate_frame"]["georeferenced"], false);
        assert_eq!(value["features"].as_array().unwrap().len(), 7);
    }

    #[test]
    fn snapshot_round_trips_as_json_and_preserves_feature_ids() {
        let features = starter_features();
        let value = snapshot_json("fixture", LandscapeScale::Site, &features);
        let serialized = serde_json::to_string(&value).unwrap();
        let parsed: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(parsed, value);
        assert_eq!(parsed["features"][0]["id"], "feature-0001");
        assert_eq!(
            parsed["features"][0]["geometry_representation"],
            "schematic-symbol-anchor-v0"
        );
    }

    #[test]
    fn feature_ids_in_starter_plan_are_unique_and_increasing() {
        let features = starter_features();
        let ids: Vec<u32> = features.iter().map(|feature| feature.id).collect();
        let unique: std::collections::HashSet<u32> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len());
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn committed_starter_fixture_matches_exporter() {
        let committed: Value = serde_json::from_str(include_str!(
            "../../fixtures/landscape/seed-landscape-v0.json"
        ))
        .unwrap();
        let generated = snapshot_json(
            "Starter landscape (schematic)",
            LandscapeScale::Site,
            &starter_features(),
        );
        assert_eq!(committed, generated);
    }

    #[test]
    fn removing_selected_feature_preserves_unselected_features() {
        let original = starter_features();
        let mut features = original.clone();
        let removed_id = features.last().unwrap().id;

        let (index, removed) = remove_feature(&mut features, removed_id).unwrap();
        assert_eq!(removed.id, removed_id);
        assert_eq!(features.len(), 6);
        assert!(!features.iter().any(|feature| feature.id == removed_id));

        let before = features.clone();
        assert!(remove_feature(&mut features, u32::MAX).is_none());
        assert_eq!(features, before);

        features.insert(index, removed);
        assert_eq!(features, original);
    }

    #[test]
    fn undoing_a_deletion_restores_exact_order() {
        let original = starter_features();
        let mut features = original.clone();
        let target = features[2];
        let (index, removed) = remove_feature(&mut features, target.id).unwrap();
        assert_eq!(removed, target);

        let affected_id = undo_last_edit(
            &mut features,
            EditCommand::Deleted {
                feature: removed,
                index,
            },
        );
        assert_eq!(affected_id, target.id);
        assert_eq!(features, original);
    }

    #[test]
    fn undoing_an_addition_removes_only_that_added_feature() {
        let mut features = starter_features();
        let original = features.clone();
        let added = DesignFeature {
            id: 8,
            x_du: 500,
            y_du: 325,
            kind: FeatureKind::Water,
        };
        features.push(added);
        let affected_id = undo_last_edit(&mut features, EditCommand::Added { feature: added });

        assert_eq!(affected_id, 8);
        assert_eq!(features, original);
    }

    #[test]
    fn redo_after_undo_restores_an_added_feature_exactly() {
        let original = starter_features();
        let added = DesignFeature {
            id: 8,
            x_du: 500,
            y_du: 325,
            kind: FeatureKind::Water,
        };
        let mut features = original.clone();
        features.push(added);

        assert_eq!(
            undo_last_edit(&mut features, EditCommand::Added { feature: added }),
            added.id
        );
        assert_eq!(features, original);
        assert_eq!(
            redo_last_edit(&mut features, EditCommand::Added { feature: added }),
            Ok(added.id)
        );
        assert_eq!(features, [original, vec![added]].concat());
    }

    #[test]
    fn redo_after_undo_restores_a_deletion_exactly() {
        let original = starter_features();
        let mut after_delete = original.clone();
        let target = after_delete[2];
        let (index, removed) = remove_feature(&mut after_delete, target.id).unwrap();
        let expected_after_delete = after_delete.clone();

        assert_eq!(
            undo_last_edit(
                &mut after_delete,
                EditCommand::Deleted {
                    feature: removed,
                    index,
                }
            ),
            target.id
        );
        assert_eq!(after_delete, original);
        assert_eq!(
            redo_last_edit(
                &mut after_delete,
                EditCommand::Deleted {
                    feature: removed,
                    index,
                }
            ),
            Ok(target.id)
        );
        assert_eq!(after_delete, expected_after_delete);
    }

    #[test]
    fn redo_fails_closed_on_feature_identity_conflict() {
        let mut features = starter_features();
        let original = features.clone();
        let existing = features[0];

        let result = redo_last_edit(&mut features, EditCommand::Added { feature: existing });

        assert!(result.is_err());
        assert_eq!(features, original);
    }

    #[test]
    fn every_design_tool_has_a_stable_machine_key() {
        let kinds = [
            FeatureKind::Tree,
            FeatureKind::PlantingBed,
            FeatureKind::Water,
            FeatureKind::Path,
            FeatureKind::BuiltElement,
        ];
        assert_eq!(
            kinds.map(FeatureKind::key),
            ["tree", "planting-bed", "water", "path", "built-element"]
        );
    }
}
