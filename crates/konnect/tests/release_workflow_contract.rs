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

fn workflow(name: &str) -> String {
    std::fs::read_to_string(repository_root().join(".github/workflows").join(name))
        .unwrap_or_else(|error| panic!("failed to read {name}: {error}"))
}

#[test]
fn release_publication_requires_real_kicad_acceptance() {
    let release = workflow("release.yml");

    assert!(
        release.contains("real-kicad-acceptance:")
            && release.contains("uses: ./.github/workflows/e2e-kicad.yml"),
        "release.yml must call the real-KiCad acceptance workflow"
    );
    assert!(
        release.contains("needs: [real-kicad-acceptance, build, pcm-package]"),
        "the release publication job must require real-KiCad acceptance and both artifact jobs"
    );
}

#[test]
fn real_kicad_workflow_remains_reusable_and_opt_in_for_pull_requests() {
    let e2e = workflow("e2e-kicad.yml");

    assert!(
        e2e.contains("workflow_call:"),
        "the release workflow needs a reusable real-KiCad workflow"
    );
    assert!(
        e2e.contains("pull_request:")
            && e2e.contains("types: [labeled]")
            && e2e.contains("github.event.label.name == 'run:e2e-kicad'"),
        "pull requests must run real-KiCad acceptance only through the run:e2e-kicad label"
    );
    assert!(
        e2e.contains("schedule:") && e2e.contains("workflow_dispatch:"),
        "weekly and manual real-KiCad entry points must remain available"
    );
}
