# Jung

A high-performance geospatial symbology and cartographic rendering engine written in Rust.

Jung transforms geospatial features + style definitions into rendered output (raster tiles, SVG, or raw pixels). Named after Carl Jung and his work on archetypal symbols.

## Features

- **Mapbox GL Style Compatible** — parse and render using the industry-standard Mapbox GL JSON style specification
- **Multiple Output Formats** — render to raw RGBA pixels, with SVG and vector tile output planned
- **WebAssembly Support** — run the full rendering engine in the browser via WASM
- **High Performance** — pure Rust with zero-copy geometry processing and SIMD-ready rendering paths
- **Extensible Symbology** — graduated symbols, proportional symbols, label placement, and military symbology (planned)

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  jung-style │────▶│  jung-core  │────▶│  jung-cli   │
│  (parsing)  │     │ (rendering) │     │   (CLI)     │
└─────────────┘     └─────────────┘     └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  jung-wasm  │
                    │  (browser)  │
                    └─────────────┘
```

### Crates

| Crate | Description |
|-------|-------------|
| `jung-core` | Core rendering engine — takes styled features, produces pixel output |
| `jung-style` | Style specification parser (Mapbox GL JSON compatible) |
| `jung-wasm` | WebAssembly bindings for browser-side rendering |
| `jung-cli` | Command-line tool for batch rendering |

## Quick Start

### CLI Usage

```bash
# Render a GeoJSON file with a style
jung --style style.json --input data.geojson --output tile.rgba --width 256 --height 256

# Specify a custom bounding box
jung --style style.json --input data.geojson --output tile.rgba --bbox "-180,-90,180,90"
```

### Library Usage

```rust
use jung_core::geometry::{Feature, Geometry, Point};
use jung_core::renderer::{BBox, Renderer};
use jung_style::parse_style;

let style_json = r#"{
    "name": "my-style",
    "layers": [{
        "id": "cities",
        "paint": { "circle-color": "#ff0000", "circle-radius": 5.0 }
    }]
}"#;

let style = parse_style(style_json).unwrap();
let renderer = Renderer::new(512, 512).unwrap();

let features = vec![Feature {
    geometry: Geometry::Point(Point { x: -73.9857, y: 40.7484 }),
    properties: Default::default(),
}];

let bbox = BBox { min_x: -74.1, min_y: 40.6, max_x: -73.8, max_y: 40.9 };
let pixels = renderer.render(&style, &features, &bbox).unwrap();
```

### WebAssembly

```javascript
import init, { render_to_pixels } from 'jung-wasm';

await init();

const pixels = render_to_pixels(
    256, 256,           // width, height
    styleJson,          // Mapbox GL style JSON string
    geojsonString,      // GeoJSON FeatureCollection
    -180, -90, 180, 90  // bounding box
);

// pixels is a Uint8Array of RGBA data
const imageData = new ImageData(new Uint8ClampedArray(pixels), 256, 256);
ctx.putImageData(imageData, 0, 0);
```

## Style Specification

Jung uses a Mapbox GL-compatible style format. Example:

```json
{
    "name": "urban-map",
    "layers": [
        {
            "id": "buildings",
            "source": "buildings-source",
            "paint": {
                "fill-color": "#d4d4d4",
                "line-color": "#888888",
                "line-width": 1.0
            }
        },
        {
            "id": "roads",
            "source": "roads-source",
            "paint": {
                "line-color": "#ffffff",
                "line-width": 2.5
            }
        },
        {
            "id": "labels",
            "source": "places-source",
            "paint": {
                "text-color": "#333333"
            },
            "layout": {
                "text-field": "{name}",
                "text-font": ["Noto Sans Regular"],
                "text-size": 14.0
            }
        }
    ]
}
```

### Supported Paint Properties

| Property | Type | Description |
|----------|------|-------------|
| `fill-color` | color | Polygon fill color |
| `line-color` | color | Line/stroke color |
| `line-width` | number | Line width in pixels |
| `circle-color` | color | Point circle color |
| `circle-radius` | number | Point circle radius in pixels |
| `text-color` | color | Label text color |

### Supported Layout Properties

| Property | Type | Description |
|----------|------|-------------|
| `text-field` | string | Feature property to use as label text |
| `text-font` | string[] | Font stack |
| `text-size` | number | Font size in pixels |

### Color Formats

- Hex: `#rgb`, `#rrggbb`, `#rrggbbaa`
- Named: `red`, `green`, `blue`, `white`, `black`, `yellow`, `cyan`, `magenta`, `transparent`
- Function: `rgb(r, g, b)`, `rgba(r, g, b, a)`

## Roadmap

### Phase 1 — Core Rendering (current)
- [x] Point rendering with circle symbolization
- [x] Mapbox GL style parsing
- [x] CLI tool for batch rendering
- [x] WebAssembly bindings
- [ ] Line rendering with width and dash patterns
- [ ] Polygon fill and stroke rendering

### Phase 2 — Advanced Symbology
- [ ] Graduated/classified symbology (color ramps based on attribute values)
- [ ] Proportional symbols (size driven by data)
- [ ] Icon/marker rendering from sprite sheets
- [ ] Heatmap rendering
- [ ] Dot density maps

### Phase 3 — Label Engine
- [ ] Text label placement
- [ ] Collision detection / decluttering
- [ ] Curved labels along lines
- [ ] Priority-based label selection
- [ ] Halo/buffer rendering

### Phase 4 — Military & Specialized
- [ ] MIL-STD-2525 military symbology
- [ ] Pattern fills (hatch, cross-hatch, custom)
- [ ] SVG output backend
- [ ] PDF output backend

### Phase 5 — Integration
- [ ] Vector tile rendering (MVT → pixels)
- [ ] TileTopia integration (tile generation pipeline)
- [ ] ViewTopia integration (WASM client-side rendering)
- [ ] Expression evaluation (data-driven styling)

## Building

```bash
# Build all crates
cargo build --all

# Run tests
cargo test --all

# Build WASM (requires wasm-pack)
cd crates/jung-wasm
wasm-pack build --target web
```

## Integration with TileTopia Ecosystem

Jung is part of the TileTopia geospatial platform:

- **[TileTopia](https://github.com/TileTopia-HQ/tiletopia)** — 3D tile generation pipeline
- **[ViewTopia](https://github.com/TileTopia-HQ/viewtopia)** — Geospatial viewer with agentic AI
- **[Ptolemy](https://github.com/TileTopia-HQ/ptolemy)** — Versioned geospatial database

Jung provides the symbology engine that TileTopia uses for raster tile rendering and ViewTopia uses (via WASM) for client-side styling.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contributing

Contributions are welcome! Please see our [contributing guide](CONTRIBUTING.md) for details.
