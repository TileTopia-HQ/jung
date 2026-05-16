use crate::geometry::{Feature, Geometry, Point};
use jung_style::{Color, Layer, Style};
use thiserror::Error;

/// Errors that can occur during rendering.
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("canvas dimensions must be positive: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },

    #[error("no layers to render")]
    NoLayers,
}

/// Bounding box in map coordinates.
#[derive(Debug, Clone, Copy)]
pub struct BBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

/// An RGBA pixel buffer (row-major, 4 bytes per pixel).
#[derive(Debug, Clone)]
pub struct PixelBuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl PixelBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height * 4) as usize],
        }
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        self.data[idx] = color.r;
        self.data[idx + 1] = color.g;
        self.data[idx + 2] = color.b;
        self.data[idx + 3] = color.a;
    }
}

/// The main renderer. Takes a style and features, produces pixel output.
pub struct Renderer {
    pub width: u32,
    pub height: u32,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Result<Self, RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::InvalidDimensions { width, height });
        }
        Ok(Self { width, height })
    }

    /// Render features according to the given style within the bounding box.
    pub fn render(
        &self,
        style: &Style,
        features: &[Feature],
        bbox: &BBox,
    ) -> Result<PixelBuffer, RenderError> {
        if style.layers.is_empty() {
            return Err(RenderError::NoLayers);
        }

        let mut buffer = PixelBuffer::new(self.width, self.height);

        for layer in &style.layers {
            for feature in features {
                self.render_feature(&mut buffer, layer, feature, bbox);
            }
        }

        Ok(buffer)
    }

    fn render_feature(
        &self,
        buffer: &mut PixelBuffer,
        layer: &Layer,
        feature: &Feature,
        bbox: &BBox,
    ) {
        match &feature.geometry {
            Geometry::Point(pt) => self.render_point(buffer, layer, pt, bbox),
            Geometry::MultiPoint(pts) => {
                for pt in pts {
                    self.render_point(buffer, layer, pt, bbox);
                }
            }
            // Line and polygon rendering will be implemented in later phases
            _ => {}
        }
    }

    fn render_point(&self, buffer: &mut PixelBuffer, layer: &Layer, pt: &Point, bbox: &BBox) {
        let px = self.map_x(pt.x, bbox);
        let py = self.map_y(pt.y, bbox);

        let radius = layer.point_radius.unwrap_or(4.0) as i32;
        let color = layer.fill_color.unwrap_or(Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        });

        // Simple circle rasterization
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let x = px as i32 + dx;
                    let y = py as i32 + dy;
                    if x >= 0 && y >= 0 {
                        buffer.set_pixel(x as u32, y as u32, color);
                    }
                }
            }
        }
    }

    fn map_x(&self, x: f64, bbox: &BBox) -> f64 {
        (x - bbox.min_x) / (bbox.max_x - bbox.min_x) * self.width as f64
    }

    fn map_y(&self, y: f64, bbox: &BBox) -> f64 {
        (bbox.max_y - y) / (bbox.max_y - bbox.min_y) * self.height as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_style() -> Style {
        Style {
            name: "test".to_string(),
            layers: vec![Layer {
                id: "points".to_string(),
                source: None,
                fill_color: Some(Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }),
                stroke_color: None,
                stroke_width: None,
                point_radius: Some(3.0),
                font_family: None,
                font_size: None,
                text_field: None,
                text_color: None,
            }],
        }
    }

    #[test]
    fn render_empty_features() {
        let renderer = Renderer::new(256, 256).unwrap();
        let style = test_style();
        let bbox = BBox {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        };
        let result = renderer.render(&style, &[], &bbox).unwrap();
        assert_eq!(result.width, 256);
        assert_eq!(result.height, 256);
        // All pixels should be transparent black
        assert!(result.data.iter().all(|&b| b == 0));
    }

    #[test]
    fn render_single_point() {
        let renderer = Renderer::new(256, 256).unwrap();
        let style = test_style();
        let bbox = BBox {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        };
        let features = vec![Feature {
            geometry: Geometry::Point(Point { x: 0.5, y: 0.5 }),
            properties: HashMap::new(),
        }];
        let result = renderer.render(&style, &features, &bbox).unwrap();
        // Center pixel should be red
        let cx = 128u32;
        let cy = 128u32;
        let idx = ((cy * 256 + cx) * 4) as usize;
        assert_eq!(result.data[idx], 255); // R
        assert_eq!(result.data[idx + 1], 0); // G
        assert_eq!(result.data[idx + 2], 0); // B
        assert_eq!(result.data[idx + 3], 255); // A
    }

    #[test]
    fn invalid_dimensions() {
        assert!(Renderer::new(0, 256).is_err());
        assert!(Renderer::new(256, 0).is_err());
    }

    #[test]
    fn no_layers_error() {
        let renderer = Renderer::new(256, 256).unwrap();
        let style = Style {
            name: "empty".to_string(),
            layers: vec![],
        };
        let bbox = BBox {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        };
        assert!(renderer.render(&style, &[], &bbox).is_err());
    }
}
