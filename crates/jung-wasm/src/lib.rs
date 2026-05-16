//! # jung-wasm
//!
//! WebAssembly bindings for the Jung symbology engine.
//! Allows browser-side rendering of styled geospatial features.

use jung_core::geometry::{Feature, Geometry, Point};
use jung_core::renderer::{BBox, Renderer};
use jung_style::parse_style;
use wasm_bindgen::prelude::*;

/// Render features from a GeoJSON string using a Mapbox GL style JSON.
/// Returns raw RGBA pixel data.
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn render_to_pixels(
    width: u32,
    height: u32,
    style_json: &str,
    geojson: &str,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) -> Result<Vec<u8>, JsValue> {
    let style =
        parse_style(style_json).map_err(|e| JsValue::from_str(&format!("Style error: {e}")))?;

    let features = parse_geojson_points(geojson)
        .map_err(|e| JsValue::from_str(&format!("GeoJSON error: {e}")))?;

    let renderer = Renderer::new(width, height)
        .map_err(|e| JsValue::from_str(&format!("Renderer error: {e}")))?;

    let bbox = BBox {
        min_x,
        min_y,
        max_x,
        max_y,
    };

    let buffer = renderer
        .render(&style, &features, &bbox)
        .map_err(|e| JsValue::from_str(&format!("Render error: {e}")))?;

    Ok(buffer.data)
}

/// Minimal GeoJSON point parser (FeatureCollection with Point geometries).
fn parse_geojson_points(geojson: &str) -> Result<Vec<Feature>, String> {
    let value: serde_json::Value =
        serde_json::from_str(geojson).map_err(|e| format!("JSON parse: {e}"))?;

    let features_array = value
        .get("features")
        .and_then(|f| f.as_array())
        .ok_or("missing 'features' array")?;

    let mut features = Vec::new();
    for feat_val in features_array {
        let geom = feat_val.get("geometry").ok_or("feature missing geometry")?;

        let geom_type = geom
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or("geometry missing type")?;

        if geom_type != "Point" {
            continue;
        }

        let coords = geom
            .get("coordinates")
            .and_then(|c| c.as_array())
            .ok_or("Point missing coordinates")?;

        if coords.len() < 2 {
            continue;
        }

        let x = coords[0].as_f64().ok_or("invalid x coordinate")?;
        let y = coords[1].as_f64().ok_or("invalid y coordinate")?;

        features.push(Feature {
            geometry: Geometry::Point(Point { x, y }),
            properties: std::collections::HashMap::new(),
        });
    }

    Ok(features)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_geojson() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": { "type": "Point", "coordinates": [1.0, 2.0] },
                    "properties": {}
                }
            ]
        }"#;
        let features = parse_geojson_points(geojson).unwrap();
        assert_eq!(features.len(), 1);
        assert_eq!(
            features[0].geometry,
            Geometry::Point(Point { x: 1.0, y: 2.0 })
        );
    }
}
