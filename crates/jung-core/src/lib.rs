//! # jung-core
//!
//! Core rendering engine for geospatial symbology.
//!
//! Takes styled geospatial features and produces rendered output
//! (raw pixels, SVG, or vector draw commands).

pub mod geometry;
pub mod renderer;

pub use renderer::Renderer;
