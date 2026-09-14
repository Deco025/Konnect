//! Release safety depends on a small set of GitHub Actions edges that Rust's
//! compiler cannot see. Keep those edges executable so a workflow cleanup
//! cannot silently make real-KiCad acceptance advisory again (#572).

use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("konnect crate must live below the repository root")
        .to_path_buf()
}

fn workflow(name: &str) -> serde_json::Value {
    let source = std::fs::read_to_string(repository_root().join(".github/workflows").join(name))
        .unwrap_or_else(|error| panic!("failed to read {name}: {error}"));
    serde_yaml_ng::from_str(&source)
        .unwrap_or_else(|error| panic!("failed to parse {name} as YAML: {error}"))
}

#[test]
fn release_publication_requires_real_kicad_acceptance() {
    let release = workflow("release.yml");

    assert_eq!(
        release["jobs"]["real-kicad-acceptance"]["uses"], "./.github/workflows/e2e-kicad.yml",
        "release.yml must call the real-KiCad acceptance workflow"
    );
    assert_eq!(
        release["jobs"]["release"]["needs"],
        serde_json::json!(["real-kicad-acceptance", "build", "pcm-package"]),
        "the release publication job must require real-KiCad acceptance and both artifact jobs"
    );
}

#[test]
fn real_kicad_workflow_remains_reusable_and_opt_in_for_pull_requests() {
    let e2e = workflow("e2e-kicad.yml");

    let triggers = &e2e["on"];
    assert!(
        triggers.get("workflow_call").is_some(),
        "the release workflow needs a reusable real-KiCad workflow"
    );
    assert_eq!(
        triggers["pull_request"]["types"],
        serde_json::json!(["labeled"]),
        "the pull-request entry point must react only to label events"
    );
    assert_eq!(
        e2e["jobs"]["e2e"]["if"],
        "github.event_name != 'pull_request' || github.event.label.name == 'run:e2e-kicad'",
        "pull requests must run real-KiCad acceptance only through the run:e2e-kicad label"
    );
    assert!(
        triggers.get("schedule").is_some() && triggers.get("workflow_dispatch").is_some(),
        "weekly and manual real-KiCad entry points must remain available"
    );
}
