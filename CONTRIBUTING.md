# Contributing to Jung

Thank you for your interest in contributing to Jung!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone git@github.com:YOUR_USERNAME/jung.git`
3. Create a feature branch: `git checkout -b my-feature`
4. Make your changes
5. Run tests: `cargo test --all`
6. Run formatting: `cargo fmt --all`
7. Run clippy: `cargo clippy --all-targets --all-features -- -D warnings`
8. Commit and push your branch
9. Open a Pull Request

## Development

### Prerequisites

- Rust 1.85+ (edition 2024)
- For WASM development: `wasm-pack` (`cargo install wasm-pack`)

### Building

```bash
cargo build --all
```

### Testing

```bash
cargo test --all
```

### Code Quality

Before submitting a PR, ensure:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

## Code of Conduct

Be respectful. We're here to build great geospatial tools together.
