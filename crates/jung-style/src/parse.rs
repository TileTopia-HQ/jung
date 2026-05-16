use serde::Deserialize;
use thiserror::Error;

/// Errors from style parsing.
#[derive(Debug, Error)]
pub enum StyleError {
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("missing required field: {0}")]
    MissingField(String),
}

/// An RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

/// A parsed style definition.
#[derive(Debug, Clone)]
pub struct Style {
    pub name: String,
    pub layers: Vec<Layer>,
}

/// A single symbology layer.
#[derive(Debug, Clone)]
pub struct Layer {
    pub id: String,
    pub source: Option<String>,
    pub fill_color: Option<Color>,
    pub stroke_color: Option<Color>,
    pub stroke_width: Option<f32>,
    pub point_radius: Option<f32>,
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub text_field: Option<String>,
    pub text_color: Option<Color>,
}

/// Intermediate JSON representation for deserialization.
#[derive(Deserialize)]
struct StyleJson {
    name: Option<String>,
    layers: Vec<LayerJson>,
}

#[derive(Deserialize)]
struct LayerJson {
    id: String,
    source: Option<String>,
    #[serde(default)]
    paint: PaintJson,
    #[serde(default)]
    layout: LayoutJson,
}

#[derive(Deserialize, Default)]
struct PaintJson {
    #[serde(rename = "fill-color")]
    fill_color: Option<String>,
    #[serde(rename = "line-color")]
    line_color: Option<String>,
    #[serde(rename = "line-width")]
    line_width: Option<f32>,
    #[serde(rename = "circle-radius")]
    circle_radius: Option<f32>,
    #[serde(rename = "circle-color")]
    circle_color: Option<String>,
    #[serde(rename = "text-color")]
    text_color: Option<String>,
}

#[derive(Deserialize, Default)]
struct LayoutJson {
    #[serde(rename = "text-field")]
    text_field: Option<String>,
    #[serde(rename = "text-font")]
    text_font: Option<Vec<String>>,
    #[serde(rename = "text-size")]
    text_size: Option<f32>,
}

/// Parse a JSON style string into a `Style`.
pub fn parse_style(json: &str) -> Result<Style, StyleError> {
    let raw: StyleJson = serde_json::from_str(json)?;

    let layers = raw
        .layers
        .into_iter()
        .map(|l| Layer {
            id: l.id,
            source: l.source,
            fill_color: l
                .paint
                .fill_color
                .as_deref()
                .and_then(parse_css_color)
                .or_else(|| l.paint.circle_color.as_deref().and_then(parse_css_color)),
            stroke_color: l.paint.line_color.as_deref().and_then(parse_css_color),
            stroke_width: l.paint.line_width,
            point_radius: l.paint.circle_radius,
            font_family: l.layout.text_font.as_ref().and_then(|f| f.first().cloned()),
            font_size: l.layout.text_size,
            text_field: l.layout.text_field,
            text_color: l.paint.text_color.as_deref().and_then(parse_css_color),
        })
        .collect();

    Ok(Style {
        name: raw.name.unwrap_or_else(|| "untitled".to_string()),
        layers,
    })
}

/// Parse a CSS color string (#rgb, #rrggbb, #rrggbbaa, or named colors).
fn parse_css_color(s: &str) -> Option<Color> {
    let s = s.trim();

    // Named colors
    match s.to_lowercase().as_str() {
        "red" => return Some(Color::rgb(255, 0, 0)),
        "green" => return Some(Color::rgb(0, 128, 0)),
        "blue" => return Some(Color::rgb(0, 0, 255)),
        "white" => return Some(Color::rgb(255, 255, 255)),
        "black" => return Some(Color::rgb(0, 0, 0)),
        "yellow" => return Some(Color::rgb(255, 255, 0)),
        "cyan" => return Some(Color::rgb(0, 255, 255)),
        "magenta" => return Some(Color::rgb(255, 0, 255)),
        "transparent" => return Some(Color::rgba(0, 0, 0, 0)),
        _ => {}
    }

    // Hex colors
    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex_color(hex);
    }

    // rgba(r, g, b, a) and rgb(r, g, b)
    if let Some(inner) = s.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 4 {
            let r = parts[0].trim().parse::<u8>().ok()?;
            let g = parts[1].trim().parse::<u8>().ok()?;
            let b = parts[2].trim().parse::<u8>().ok()?;
            let a = (parts[3].trim().parse::<f32>().ok()? * 255.0) as u8;
            return Some(Color::rgba(r, g, b, a));
        }
    }
    if let Some(inner) = s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0].trim().parse::<u8>().ok()?;
            let g = parts[1].trim().parse::<u8>().ok()?;
            let b = parts[2].trim().parse::<u8>().ok()?;
            return Some(Color::rgb(r, g, b));
        }
    }

    None
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some(Color::rgb(r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::rgb(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color::rgba(r, g, b, a))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_colors() {
        assert_eq!(parse_css_color("#ff0000"), Some(Color::rgb(255, 0, 0)));
        assert_eq!(
            parse_css_color("#00ff0080"),
            Some(Color::rgba(0, 255, 0, 128))
        );
        assert_eq!(parse_css_color("#fff"), Some(Color::rgb(255, 255, 255)));
    }

    #[test]
    fn parse_named_colors() {
        assert_eq!(parse_css_color("red"), Some(Color::rgb(255, 0, 0)));
        assert_eq!(
            parse_css_color("transparent"),
            Some(Color::rgba(0, 0, 0, 0))
        );
    }

    #[test]
    fn parse_rgb_rgba() {
        assert_eq!(
            parse_css_color("rgb(128, 64, 32)"),
            Some(Color::rgb(128, 64, 32))
        );
        assert_eq!(
            parse_css_color("rgba(255, 0, 0, 0.5)"),
            Some(Color::rgba(255, 0, 0, 127))
        );
    }

    #[test]
    fn parse_minimal_style() {
        let json = r##"{
            "name": "test-style",
            "layers": [
                {
                    "id": "points",
                    "source": "my-source",
                    "paint": {
                        "circle-color": "#ff0000",
                        "circle-radius": 5.0
                    },
                    "layout": {}
                }
            ]
        }"##;
        let style = parse_style(json).unwrap();
        assert_eq!(style.name, "test-style");
        assert_eq!(style.layers.len(), 1);
        assert_eq!(style.layers[0].id, "points");
        assert_eq!(style.layers[0].fill_color, Some(Color::rgb(255, 0, 0)));
        assert_eq!(style.layers[0].point_radius, Some(5.0));
    }

    #[test]
    fn parse_multi_layer_style() {
        let json = r##"{
            "layers": [
                {
                    "id": "fill-layer",
                    "paint": { "fill-color": "blue" }
                },
                {
                    "id": "line-layer",
                    "paint": { "line-color": "#00ff00", "line-width": 2.0 }
                },
                {
                    "id": "label-layer",
                    "paint": { "text-color": "white" },
                    "layout": { "text-field": "{name}", "text-size": 14.0 }
                }
            ]
        }"##;
        let style = parse_style(json).unwrap();
        assert_eq!(style.name, "untitled");
        assert_eq!(style.layers.len(), 3);
        assert_eq!(style.layers[1].stroke_color, Some(Color::rgb(0, 255, 0)));
        assert_eq!(style.layers[1].stroke_width, Some(2.0));
        assert_eq!(style.layers[2].text_field, Some("{name}".to_string()));
        assert_eq!(style.layers[2].font_size, Some(14.0));
    }

    #[test]
    fn invalid_json() {
        assert!(parse_style("not json").is_err());
    }
}
