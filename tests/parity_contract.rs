use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct BenchmarkPair {
    scenario: String,
    v1_benchmark: String,
    v2_benchmark: String,
    parameter: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TestPair {
    scenario: String,
    v1_test_fn: String,
    v2_test_fn: String,
}

#[derive(Debug, Default, Deserialize)]
struct ParityManifest {
    benchmark_pairs: Vec<BenchmarkPair>,
    test_pairs: Vec<TestPair>,
}

fn load_manifest() -> ParityManifest {
    let text = fs::read_to_string("parity_manifest.toml")
        .expect("failed to read parity_manifest.toml from repository root");
    toml::from_str::<ParityManifest>(&text)
        .expect("failed to parse parity_manifest.toml as ParityManifest")
}

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root).expect("failed to read benchmark directory");
    for entry in entries {
        let path = entry.expect("failed to read benchmark entry").path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn load_bench_sources() -> String {
    let mut files = Vec::new();
    collect_rs_files(Path::new("benches"), &mut files);
    files.sort();

    files
        .into_iter()
        .map(|path| {
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("failed to read {}", path.display()))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn parity_manifest_contract_is_satisfied() {
    let manifest = load_manifest();
    assert!(
        !manifest.benchmark_pairs.is_empty(),
        "parity manifest must define at least one benchmark pair"
    );
    assert!(
        !manifest.test_pairs.is_empty(),
        "parity manifest must define at least one test pair"
    );

    let benches_src = load_bench_sources();
    let v1_tests_src = fs::read_to_string("src/dlt/v1/framer.rs")
        .expect("failed to read src/dlt/v1/framer.rs");
    let v2_tests_src = fs::read_to_string("src/dlt/v2/framer.rs")
        .expect("failed to read src/dlt/v2/framer.rs");

    let mut seen_scenarios = HashSet::new();
    let mut failures: Vec<String> = Vec::new();

    for pair in &manifest.benchmark_pairs {
        if !seen_scenarios.insert(format!("bench:{}", pair.scenario)) {
            failures.push(format!(
                "duplicate benchmark scenario '{}' in parity manifest",
                pair.scenario
            ));
        }

        let v1_expected = format!(
            "BenchmarkId::new(\"{}\", \"{}\")",
            pair.v1_benchmark, pair.parameter
        );
        let v2_expected = format!(
            "BenchmarkId::new(\"{}\", \"{}\")",
            pair.v2_benchmark, pair.parameter
        );

        let v1_modular_expected = format!("BenchmarkId::new(\"{}\", spec.name)", pair.v1_benchmark);
        let v2_modular_expected = format!("BenchmarkId::new(\"{}\", spec.name)", pair.v2_benchmark);
        let scenario_declared = format!("name: \"{}\"", pair.parameter);

        if !benches_src.contains(&v1_expected)
            && !(benches_src.contains(&v1_modular_expected) && benches_src.contains(&scenario_declared))
        {
            failures.push(format!(
                "benchmark parity missing v1 side for scenario '{}' (expected {})",
                pair.scenario, v1_expected
            ));
        }
        if !benches_src.contains(&v2_expected)
            && !(benches_src.contains(&v2_modular_expected) && benches_src.contains(&scenario_declared))
        {
            failures.push(format!(
                "benchmark parity missing v2 side for scenario '{}' (expected {})",
                pair.scenario, v2_expected
            ));
        }
    }

    for pair in &manifest.test_pairs {
        if !seen_scenarios.insert(format!("test:{}", pair.scenario)) {
            failures.push(format!(
                "duplicate test scenario '{}' in parity manifest",
                pair.scenario
            ));
        }

        let v1_expected = format!("fn {}(", pair.v1_test_fn);
        let v2_expected = format!("fn {}(", pair.v2_test_fn);

        if !v1_tests_src.contains(&v1_expected) {
            failures.push(format!(
                "test parity missing v1 side for scenario '{}' (expected {})",
                pair.scenario, pair.v1_test_fn
            ));
        }
        if !v2_tests_src.contains(&v2_expected) {
            failures.push(format!(
                "test parity missing v2 side for scenario '{}' (expected {})",
                pair.scenario, pair.v2_test_fn
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "parity contract violations:\n{}",
        failures.join("\n")
    );
}