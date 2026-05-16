//! # jung-style
//!
//! Style specification parser for geospatial symbology.
//!
//! Parses a Mapbox GL-compatible JSON style spec into typed Rust structs
//! that the rendering engine (`jung-core`) consumes.

mod parse;

pub use parse::{Color, Layer, Style, StyleError, parse_style};
