# cargo-typoguard

A supply chain security tool that checks `Cargo.toml` dependencies for potential typosquatting attacks.

## What it does

- Parses `[dependencies]`, `[dev-dependencies]`, and `[build-dependencies]` from your `Cargo.toml`
- Compares each dependency name against a built-in list of the top ~200 most-downloaded crates using Levenshtein distance
- Flags dependencies that are suspiciously similar to popular crates but not exact matches
- Queries the crates.io API for flagged crates to check download counts and creation dates
- Provides colored terminal output with severity levels

## Installation

```bash
cargo install cargo-typoguard
```

## Usage

```bash
# Check the current directory's Cargo.toml
cargo typoguard check

# Check a specific Cargo.toml
cargo typoguard check --path /path/to/Cargo.toml

# Adjust the similarity threshold (default: 0.8)
cargo typoguard check --threshold 0.9
```

## Output

- **Green**: Dependency is clean (no similar popular crate found)
- **Yellow**: Similar to a popular crate but has decent download counts
- **Red**: Similar to a popular crate AND has low downloads, is very new, or doesn't exist on crates.io

## Exit codes

- `0` - No red warnings (safe for CI)
- `1` - One or more red warnings detected

## CI Integration

```yaml
# GitHub Actions example
- name: Check for typosquatting
  run: cargo typoguard check --path ./Cargo.toml
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
