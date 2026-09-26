//! Executable architectural governance for the seed workspace.
//!
//! At birth the gate holds only how law is hosted: the application never depends on its own gate,
//! and the gate depends on Tianheng alone. The application's architecture, and the boundaries that
//! hold it, are its own and grow here with it.

#![forbid(unsafe_code)]

use std::{env, process::ExitCode};

use tianheng::prelude::*;

const APP_REASON: &str = "seed is the application; it must never take a normal dependency on its own governance gate, which judges the workspace from outside it.";
const GOVERNANCE_REASON: &str = "the governance gate must stay independent of the workspace graph it judges: its normal dependencies are Tianheng's composed adopter surface alone, never an individual governance instrument or a workspace crate under judgment.";

fn constitution() -> Constitution {
    Constitution::new("seed")
        .boundary(
            CrateBoundary::crate_("seed")
                .forbid_dependency_on(["seed-governance"])
                .because(APP_REASON),
        )
        .boundary(
            CrateBoundary::crate_("seed-governance")
                .restrict_dependencies_to(["tianheng"])
                .because(GOVERNANCE_REASON),
        )
}

fn main() -> ExitCode {
    tianheng::run(&constitution(), env::args().collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use super::*;

    const LAW_PROJECTION_PREAMBLE: &str = "\
# Seed Tianheng Law Projection

This file is generated from `constitution()` in `crates/seed-governance/src/main.rs`.
The Rust declaration is authoritative; do not edit the projection by hand.
Regenerate it with `BLESS=1 cargo test -p seed-governance law_projection_is_fresh`.

";

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn current_workspace_satisfies_constitution() {
        GovernanceTest::for_constitution(constitution())
            .with_manifest_dir(workspace_root())
            .assert_clean();
    }

    #[test]
    fn every_workspace_crate_is_covered() {
        GovernanceTest::for_constitution(constitution())
            .with_manifest_dir(workspace_root())
            .assert_all_workspace_members_covered();
    }

    #[test]
    fn law_projection_is_fresh() {
        GovernanceTest::for_constitution(constitution())
            .with_manifest_dir(workspace_root())
            .assert_projection_fresh_with_preamble("AGENTS.seed-law.md", LAW_PROJECTION_PREAMBLE);
    }

    #[test]
    fn app_depending_on_its_gate_is_rejected() {
        let workspace = TempWorkspace::new("seed-governance-app-dependency");
        workspace.write_package(
            "seed",
            "[dependencies]\nseed-governance = { path = \"../seed-governance\" }\n",
            &[("lib.rs", "")],
        );
        let report = match workspace.outcome() {
            Outcome::Violations(report) => report,
            other => panic!("expected violations, got {other:?}"),
        };
        assert!(
            report.violations.iter().any(|violation| {
                violation.target() == "seed" && violation.finding == "seed-governance"
            }),
            "expected the application dependency boundary to fire: {report:?}"
        );
    }

    #[test]
    fn governance_dependency_beyond_tianheng_is_rejected() {
        let workspace = TempWorkspace::new("seed-governance-gate-dependency");
        workspace.write_package("tokio", "", &[("lib.rs", "")]);
        workspace.write_package(
            "seed-governance",
            "[dependencies]\ntianheng = { path = \"../tianheng\" }\ntokio = { path = \"../tokio\" }\n",
            &[("lib.rs", "")],
        );
        fs::write(
            workspace.path.join("Cargo.toml"),
            "[workspace]\nresolver = \"2\"\nmembers = [\"tianheng\", \"tokio\", \"seed-governance\", \"seed\"]\n",
        )
        .expect("workspace manifest should be writable");
        let report = match workspace.outcome() {
            Outcome::Violations(report) => report,
            other => panic!("expected violations, got {other:?}"),
        };
        assert!(
            report.violations.iter().any(|violation| {
                violation.target() == "seed-governance" && violation.finding == "tokio"
            }),
            "expected the governance dependency boundary to fire: {report:?}"
        );
    }

    #[test]
    fn a_newborn_workspace_stays_clean() {
        let outcome = TempWorkspace::new("seed-governance-newborn").outcome();
        assert!(
            matches!(outcome, Outcome::Clean(_)),
            "an empty application beside its gate must raise no violation: {outcome:?}"
        );
    }

    /// A scratch workspace holding the application crate and its governance crate, so every
    /// boundary has a real target.
    struct TempWorkspace {
        path: PathBuf,
    }

    impl TempWorkspace {
        fn new(name: &str) -> Self {
            let path = env::temp_dir().join(format!("{name}-{}", std::process::id()));
            if path.exists() {
                fs::remove_dir_all(&path).expect("stale temporary workspace should be removable");
            }
            let workspace = Self { path };
            workspace.write_package("tianheng", "", &[("lib.rs", "")]);
            workspace.write_package(
                "seed-governance",
                "[dependencies]\ntianheng = { path = \"../tianheng\" }\n",
                &[("lib.rs", "")],
            );
            workspace.write_package("seed", "", &[("lib.rs", "")]);
            fs::write(
                workspace.path.join("Cargo.toml"),
                "[workspace]\nresolver = \"2\"\nmembers = [\"tianheng\", \"seed-governance\", \"seed\"]\n",
            )
            .expect("workspace manifest should be writable");
            workspace
        }

        fn outcome(&self) -> Outcome {
            tianheng::check_constitution(&constitution(), &self.path.join("Cargo.toml"))
        }

        fn write_package(&self, name: &str, dependencies: &str, sources: &[(&str, &str)]) {
            let package = self.path.join(name);
            let _ = fs::remove_dir_all(&package);
            fs::create_dir_all(package.join("src")).expect("package source dir should be writable");
            fs::write(
                package.join("Cargo.toml"),
                format!(
                    "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n{dependencies}"
                ),
            )
            .expect("package manifest should be writable");
            for (file, source) in sources {
                fs::write(package.join("src").join(file), source)
                    .expect("package source should be writable");
            }
        }
    }

    impl Drop for TempWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
