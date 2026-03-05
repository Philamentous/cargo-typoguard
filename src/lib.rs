use colored::Colorize;
use serde::Deserialize;
use std::thread;
use std::time::Duration;

/// Hardcoded list of top ~200 most-downloaded crates on crates.io.
pub const TOP_CRATES: &[&str] = &[
    "serde",
    "tokio",
    "clap",
    "reqwest",
    "rand",
    "regex",
    "log",
    "syn",
    "quote",
    "proc-macro2",
    "anyhow",
    "thiserror",
    "chrono",
    "futures",
    "bytes",
    "hyper",
    "axum",
    "tracing",
    "once_cell",
    "lazy_static",
    "libc",
    "cc",
    "bitflags",
    "serde_json",
    "serde_derive",
    "toml",
    "walkdir",
    "tempfile",
    "rayon",
    "crossbeam",
    "parking_lot",
    "smallvec",
    "indexmap",
    "hashbrown",
    "memchr",
    "aho-corasick",
    "pest",
    "nom",
    "winnow",
    "url",
    "http",
    "tower",
    "tower-service",
    "tower-layer",
    "tower-http",
    "pin-project",
    "pin-project-lite",
    "pin-utils",
    "futures-core",
    "futures-util",
    "futures-io",
    "futures-sink",
    "futures-channel",
    "futures-executor",
    "futures-macro",
    "tokio-util",
    "tokio-stream",
    "tokio-macros",
    "mio",
    "socket2",
    "num-traits",
    "num-integer",
    "num-bigint",
    "num-rational",
    "num-complex",
    "num-derive",
    "num",
    "itertools",
    "either",
    "cfg-if",
    "autocfg",
    "version_check",
    "unicode-ident",
    "unicode-segmentation",
    "unicode-normalization",
    "unicode-bidi",
    "unicode-width",
    "percent-encoding",
    "form_urlencoded",
    "idna",
    "tinyvec",
    "tinyvec_macros",
    "getrandom",
    "ppv-lite86",
    "rand_core",
    "rand_chacha",
    "semver",
    "rustc_version",
    "pkg-config",
    "cmake",
    "bindgen",
    "errno",
    "rustix",
    "linux-raw-sys",
    "windows-sys",
    "windows-targets",
    "windows_x86_64_msvc",
    "windows_x86_64_gnu",
    "windows_aarch64_msvc",
    "windows_i686_msvc",
    "windows_i686_gnu",
    "tracing-core",
    "tracing-attributes",
    "tracing-subscriber",
    "tracing-log",
    "tracing-futures",
    "env_logger",
    "termcolor",
    "atty",
    "strsim",
    "textwrap",
    "os_str_bytes",
    "clap_derive",
    "clap_lex",
    "clap_builder",
    "slab",
    "lock_api",
    "scopeguard",
    "crossbeam-utils",
    "crossbeam-epoch",
    "crossbeam-deque",
    "crossbeam-channel",
    "crossbeam-queue",
    "memoffset",
    "signal-hook-registry",
    "signal-hook",
    "ctrlc",
    "nix",
    "glob",
    "globset",
    "ignore",
    "which",
    "dirs",
    "dirs-sys",
    "home",
    "shellexpand",
    "dunce",
    "same-file",
    "filetime",
    "notify",
    "sha2",
    "sha1",
    "md-5",
    "digest",
    "crypto-common",
    "block-buffer",
    "generic-array",
    "typenum",
    "base64",
    "hex",
    "data-encoding",
    "uuid",
    "time",
    "humantime",
    "colored",
    "console",
    "indicatif",
    "dialoguer",
    "prettytable-rs",
    "tabled",
    "comfy-table",
    "termion",
    "crossterm",
    "ratatui",
    "rustyline",
    "assert_cmd",
    "predicates",
    "assert_fs",
    "insta",
    "proptest",
    "quickcheck",
    "criterion",
    "test-case",
    "mockall",
    "wiremock",
    "serial_test",
    "env_test_util",
    "ureq",
    "actix-web",
    "actix-rt",
    "warp",
    "rocket",
    "tide",
    "poem",
    "salvo",
    "diesel",
    "sqlx",
    "rusqlite",
    "sea-orm",
    "mongodb",
    "redis",
    "aws-sdk-s3",
    "config",
    "dotenv",
    "dotenvy",
    "structopt",
    "derive_more",
    "derive_builder",
    "strum",
    "strum_macros",
    "paste",
    "darling",
    "proc-macro-crate",
    "proc-macro-error",
    "inventory",
    "ctor",
    "linkme",
    "dashmap",
    "flume",
    "kanal",
    "async-trait",
    "async-std",
    "smol",
];

/// Normalize a crate name by replacing hyphens with underscores,
/// since crates.io treats them as equivalent.
pub fn normalize_name(name: &str) -> String {
    name.to_lowercase().replace('-', "_")
}

/// Compute the normalized similarity between two crate names.
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
/// Uses Levenshtein distance on the normalized forms.
pub fn similarity(a: &str, b: &str) -> f64 {
    let a_norm = normalize_name(a);
    let b_norm = normalize_name(b);
    let max_len = a_norm.len().max(b_norm.len());
    if max_len == 0 {
        return 1.0;
    }
    let dist = edit_distance::edit_distance(&a_norm, &b_norm);
    1.0 - (dist as f64 / max_len as f64)
}

/// A dependency extracted from Cargo.toml.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub section: String,
}

/// Parse dependencies from a Cargo.toml string.
/// Extracts crate names from `[dependencies]`, `[dev-dependencies]`,
/// and `[build-dependencies]` sections.
pub fn parse_dependencies(cargo_toml: &str) -> Result<Vec<Dependency>, String> {
    let parsed: toml::Value =
        toml::from_str(cargo_toml).map_err(|e| format!("Failed to parse Cargo.toml: {e}"))?;

    let table = parsed
        .as_table()
        .ok_or("Cargo.toml root is not a table")?;

    let sections = [
        ("dependencies", "dependencies"),
        ("dev-dependencies", "dev-dependencies"),
        ("build-dependencies", "build-dependencies"),
    ];

    let mut deps = Vec::new();

    for (key, section_name) in &sections {
        if let Some(section) = table.get(*key) {
            if let Some(dep_table) = section.as_table() {
                for dep_name in dep_table.keys() {
                    deps.push(Dependency {
                        name: dep_name.clone(),
                        section: section_name.to_string(),
                    });
                }
            }
        }
    }

    Ok(deps)
}

/// A match result when comparing a dependency against the top crates list.
#[derive(Debug, Clone)]
pub struct SimilarityMatch {
    pub dependency: String,
    pub similar_to: String,
    pub score: f64,
}

/// Find similar crates from the top crates list for a given dependency name.
/// Returns matches above the threshold that are NOT exact matches (after normalization).
pub fn find_similar_crates(dep_name: &str, threshold: f64) -> Vec<SimilarityMatch> {
    let dep_normalized = normalize_name(dep_name);
    let mut matches = Vec::new();

    for &top_crate in TOP_CRATES {
        let top_normalized = normalize_name(top_crate);

        // Skip exact matches (after normalization)
        if dep_normalized == top_normalized {
            continue;
        }

        let score = similarity(dep_name, top_crate);
        if score >= threshold {
            matches.push(SimilarityMatch {
                dependency: dep_name.to_string(),
                similar_to: top_crate.to_string(),
                score,
            });
        }
    }

    // Sort by score descending
    matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    matches
}

/// Response from the crates.io API for a single crate.
#[derive(Debug, Deserialize)]
pub struct CratesIoResponse {
    #[serde(rename = "crate")]
    pub krate: CrateInfo,
}

#[derive(Debug, Deserialize)]
pub struct CrateInfo {
    pub downloads: u64,
    pub created_at: String,
}

/// Severity level for a flagged dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    /// No similar popular crate found
    Clean,
    /// Similar to popular crate but has decent downloads
    Warning,
    /// Similar to popular crate AND (low downloads OR very new OR doesn't exist)
    Danger,
}

/// Result of checking a single dependency.
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub dependency: String,
    pub section: String,
    pub severity: Severity,
    pub similar_to: Option<String>,
    pub score: Option<f64>,
    pub downloads: Option<u64>,
    pub created_at: Option<String>,
    pub exists_on_crates_io: Option<bool>,
    pub message: String,
}

/// Query the crates.io API for information about a crate.
pub fn query_crates_io(crate_name: &str) -> Result<Option<CratesIoResponse>, String> {
    let url = format!("https://crates.io/api/v1/crates/{crate_name}");

    let response = ureq::get(&url)
        .set("User-Agent", "cargo-typoguard/0.1.0")
        .call();

    match response {
        Ok(resp) => {
            let body: CratesIoResponse = resp
                .into_json()
                .map_err(|e| format!("Failed to parse crates.io response: {e}"))?;
            Ok(Some(body))
        }
        Err(ureq::Error::Status(404, _)) => Ok(None),
        Err(e) => Err(format!("Failed to query crates.io: {e}")),
    }
}

/// Check if a creation date string is less than 30 days old.
/// Expects ISO 8601 format like "2024-01-15T10:30:00Z" or similar.
fn is_recently_created(created_at: &str) -> bool {
    // Parse just the date portion (first 10 chars: YYYY-MM-DD)
    if created_at.len() < 10 {
        return false;
    }
    let date_str = &created_at[..10];
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return false;
    }

    let Ok(year) = parts[0].parse::<i64>() else {
        return false;
    };
    let Ok(month) = parts[1].parse::<i64>() else {
        return false;
    };
    let Ok(day) = parts[2].parse::<i64>() else {
        return false;
    };

    // Simple days-since-epoch approximation for comparison
    let crate_days = year * 365 + month * 30 + day;

    // Get current date from system time
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let now_days = now.as_secs() as i64 / 86400;

    // Convert epoch days to comparable format
    // epoch is 1970-01-01, so we approximate
    let epoch_base = 1970 * 365 + 30 + 1;
    let crate_epoch_days = crate_days - epoch_base;

    (now_days - crate_epoch_days).abs() < 30
}

const LOW_DOWNLOAD_THRESHOLD: u64 = 1000;

/// Check all dependencies against the top crates list and optionally query crates.io.
/// If `skip_api` is true, no API calls are made (useful for testing).
pub fn check_dependencies(
    deps: &[Dependency],
    threshold: f64,
    skip_api: bool,
) -> Vec<CheckResult> {
    let mut results = Vec::new();

    for dep in deps {
        let dep_normalized = normalize_name(&dep.name);

        // Check if this IS a top crate (exact match after normalization)
        let is_top_crate = TOP_CRATES
            .iter()
            .any(|c| normalize_name(c) == dep_normalized);

        if is_top_crate {
            results.push(CheckResult {
                dependency: dep.name.clone(),
                section: dep.section.clone(),
                severity: Severity::Clean,
                similar_to: None,
                score: None,
                downloads: None,
                created_at: None,
                exists_on_crates_io: None,
                message: format!("{} is a known popular crate", dep.name),
            });
            continue;
        }

        let similar = find_similar_crates(&dep.name, threshold);

        if similar.is_empty() {
            results.push(CheckResult {
                dependency: dep.name.clone(),
                section: dep.section.clone(),
                severity: Severity::Clean,
                similar_to: None,
                score: None,
                downloads: None,
                created_at: None,
                exists_on_crates_io: None,
                message: format!("{} - no similar popular crates found", dep.name),
            });
            continue;
        }

        let best_match = &similar[0];

        if skip_api {
            // Without API data, treat all flagged crates as warnings
            results.push(CheckResult {
                dependency: dep.name.clone(),
                section: dep.section.clone(),
                severity: Severity::Warning,
                similar_to: Some(best_match.similar_to.clone()),
                score: Some(best_match.score),
                downloads: None,
                created_at: None,
                exists_on_crates_io: None,
                message: format!(
                    "{} is similar to {} (score: {:.1}%)",
                    dep.name,
                    best_match.similar_to,
                    best_match.score * 100.0
                ),
            });
            continue;
        }

        // Query crates.io for more information
        // Small delay to respect rate limits
        thread::sleep(Duration::from_millis(100));

        match query_crates_io(&dep.name) {
            Ok(Some(info)) => {
                let downloads = info.krate.downloads;
                let created = info.krate.created_at.clone();
                let is_new = is_recently_created(&created);
                let is_low_downloads = downloads < LOW_DOWNLOAD_THRESHOLD;

                let severity = if is_low_downloads || is_new {
                    Severity::Danger
                } else {
                    Severity::Warning
                };

                let mut msg = format!(
                    "{} is similar to {} (score: {:.1}%, downloads: {}, created: {})",
                    dep.name,
                    best_match.similar_to,
                    best_match.score * 100.0,
                    downloads,
                    &created[..10.min(created.len())]
                );

                if is_new {
                    msg.push_str(" [RECENTLY CREATED]");
                }
                if is_low_downloads {
                    msg.push_str(" [LOW DOWNLOADS]");
                }

                results.push(CheckResult {
                    dependency: dep.name.clone(),
                    section: dep.section.clone(),
                    severity,
                    similar_to: Some(best_match.similar_to.clone()),
                    score: Some(best_match.score),
                    downloads: Some(downloads),
                    created_at: Some(created),
                    exists_on_crates_io: Some(true),
                    message: msg,
                });
            }
            Ok(None) => {
                // Crate doesn't exist on crates.io - very suspicious
                results.push(CheckResult {
                    dependency: dep.name.clone(),
                    section: dep.section.clone(),
                    severity: Severity::Danger,
                    similar_to: Some(best_match.similar_to.clone()),
                    score: Some(best_match.score),
                    downloads: None,
                    created_at: None,
                    exists_on_crates_io: Some(false),
                    message: format!(
                        "{} is similar to {} (score: {:.1}%) and DOES NOT EXIST on crates.io!",
                        dep.name,
                        best_match.similar_to,
                        best_match.score * 100.0
                    ),
                });
            }
            Err(e) => {
                // API error - treat as warning
                results.push(CheckResult {
                    dependency: dep.name.clone(),
                    section: dep.section.clone(),
                    severity: Severity::Warning,
                    similar_to: Some(best_match.similar_to.clone()),
                    score: Some(best_match.score),
                    downloads: None,
                    created_at: None,
                    exists_on_crates_io: None,
                    message: format!(
                        "{} is similar to {} (score: {:.1}%) [API error: {}]",
                        dep.name,
                        best_match.similar_to,
                        best_match.score * 100.0,
                        e
                    ),
                });
            }
        }
    }

    results
}

/// Print check results with colored output.
pub fn print_results(results: &[CheckResult]) {
    println!(
        "\n{}",
        "=== cargo-typoguard dependency check ===".bold()
    );
    println!();

    for result in results {
        let prefix = match result.severity {
            Severity::Clean => "[CLEAN]".green().bold(),
            Severity::Warning => "[WARN] ".yellow().bold(),
            Severity::Danger => "[DANGER]".red().bold(),
        };

        let section_tag = format!("({})", result.section).dimmed();

        println!("  {} {} {}", prefix, result.message, section_tag);
    }

    let danger_count = results.iter().filter(|r| r.severity == Severity::Danger).count();
    let warn_count = results.iter().filter(|r| r.severity == Severity::Warning).count();
    let clean_count = results.iter().filter(|r| r.severity == Severity::Clean).count();

    println!();
    println!(
        "  {} clean, {} warnings, {} dangerous",
        clean_count.to_string().green(),
        warn_count.to_string().yellow(),
        danger_count.to_string().red()
    );
    println!();

    if danger_count > 0 {
        println!(
            "  {}",
            "Potential typosquatting detected! Review flagged dependencies carefully."
                .red()
                .bold()
        );
        println!();
    }
}

/// Returns true if any result has Danger severity.
pub fn has_danger(results: &[CheckResult]) -> bool {
    results.iter().any(|r| r.severity == Severity::Danger)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_identical() {
        assert!((similarity("serde", "serde") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_similarity_known_pairs() {
        // serde vs serdee - very similar
        let score = similarity("serde", "serdee");
        assert!(score > 0.8, "serde/serdee score was {score}");

        // tokio vs toko - similar
        let score = similarity("tokio", "toko");
        assert!(score > 0.7, "tokio/toko score was {score}");

        // clap vs clapp - very similar
        let score = similarity("clap", "clapp");
        assert!(score > 0.7, "clap/clapp score was {score}");
    }

    #[test]
    fn test_similarity_completely_different() {
        let score = similarity("serde", "zzzzzzzzz");
        assert!(score < 0.3, "serde/zzzzzzzzz score was {score}");
    }

    #[test]
    fn test_normalize_name_hyphen_underscore() {
        assert_eq!(normalize_name("my-crate"), normalize_name("my_crate"));
        assert_eq!(normalize_name("aho-corasick"), "aho_corasick");
        assert_eq!(normalize_name("AHO-Corasick"), "aho_corasick");
    }

    #[test]
    fn test_similarity_with_hyphen_underscore_equivalence() {
        // These should be identical after normalization
        let score = similarity("serde-json", "serde_json");
        assert!(
            (score - 1.0).abs() < f64::EPSILON,
            "serde-json/serde_json should be 1.0, got {score}"
        );
    }

    #[test]
    fn test_parse_dependencies_basic() {
        let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
insta = "1.0"

[build-dependencies]
cc = "1.0"
"#;
        let deps = parse_dependencies(cargo_toml).unwrap();
        assert_eq!(deps.len(), 4);

        let dep_names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(dep_names.contains(&"serde"));
        assert!(dep_names.contains(&"tokio"));
        assert!(dep_names.contains(&"insta"));
        assert!(dep_names.contains(&"cc"));

        let serde_dep = deps.iter().find(|d| d.name == "serde").unwrap();
        assert_eq!(serde_dep.section, "dependencies");

        let insta_dep = deps.iter().find(|d| d.name == "insta").unwrap();
        assert_eq!(insta_dep.section, "dev-dependencies");
    }

    #[test]
    fn test_parse_dependencies_empty() {
        let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
"#;
        let deps = parse_dependencies(cargo_toml).unwrap();
        assert!(deps.is_empty());
    }

    #[test]
    fn test_parse_dependencies_invalid() {
        let result = parse_dependencies("not valid toml {{{{");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_similar_crates_exact_match_excluded() {
        // Exact match should not appear in results
        let matches = find_similar_crates("serde", 0.5);
        assert!(
            !matches.iter().any(|m| m.similar_to == "serde"),
            "Exact match should be excluded"
        );
    }

    #[test]
    fn test_find_similar_crates_typo() {
        let matches = find_similar_crates("serdee", 0.8);
        assert!(
            !matches.is_empty(),
            "serdee should match against serde"
        );
        assert_eq!(matches[0].similar_to, "serde");
    }

    #[test]
    fn test_find_similar_crates_no_match() {
        let matches = find_similar_crates("zzzznotacrate", 0.8);
        assert!(
            matches.is_empty(),
            "zzzznotacrate should not match any top crate"
        );
    }

    #[test]
    fn test_threshold_filtering() {
        // With a high threshold, fewer matches
        let matches_high = find_similar_crates("serdee", 0.95);
        let matches_low = find_similar_crates("serdee", 0.5);
        assert!(
            matches_low.len() >= matches_high.len(),
            "Lower threshold should yield >= matches"
        );
    }

    #[test]
    fn test_check_dependencies_clean() {
        let deps = vec![Dependency {
            name: "serde".to_string(),
            section: "dependencies".to_string(),
        }];

        let results = check_dependencies(&deps, 0.8, true);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].severity, Severity::Clean);
    }

    #[test]
    fn test_check_dependencies_suspicious() {
        let deps = vec![Dependency {
            name: "serdee".to_string(),
            section: "dependencies".to_string(),
        }];

        let results = check_dependencies(&deps, 0.8, true);
        assert_eq!(results.len(), 1);
        // Without API, flagged crates become warnings
        assert_eq!(results[0].severity, Severity::Warning);
        assert_eq!(results[0].similar_to, Some("serde".to_string()));
    }

    #[test]
    fn test_check_dependencies_no_match() {
        let deps = vec![Dependency {
            name: "zzzznotacrate".to_string(),
            section: "dependencies".to_string(),
        }];

        let results = check_dependencies(&deps, 0.8, true);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].severity, Severity::Clean);
    }

    #[test]
    fn test_has_danger() {
        let results = vec![
            CheckResult {
                dependency: "foo".to_string(),
                section: "dependencies".to_string(),
                severity: Severity::Clean,
                similar_to: None,
                score: None,
                downloads: None,
                created_at: None,
                exists_on_crates_io: None,
                message: "ok".to_string(),
            },
            CheckResult {
                dependency: "bar".to_string(),
                section: "dependencies".to_string(),
                severity: Severity::Danger,
                similar_to: Some("baz".to_string()),
                score: Some(0.9),
                downloads: None,
                created_at: None,
                exists_on_crates_io: Some(false),
                message: "bad".to_string(),
            },
        ];

        assert!(has_danger(&results));

        let clean_only = vec![results[0].clone()];
        assert!(!has_danger(&clean_only));
    }

    #[test]
    fn test_normalize_handles_mixed_case_and_separators() {
        assert_eq!(normalize_name("My-Crate-Name"), "my_crate_name");
        assert_eq!(normalize_name("my_crate_name"), "my_crate_name");
        assert_eq!(normalize_name("MY-CRATE-NAME"), "my_crate_name");
    }

    #[test]
    fn test_top_crates_list_has_enough() {
        assert!(
            TOP_CRATES.len() >= 100,
            "Top crates list should have at least 100 entries, got {}",
            TOP_CRATES.len()
        );
    }
}
