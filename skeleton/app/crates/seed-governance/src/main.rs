//! Executable architectural governance for the seed workspace.
//!
//! Seed is an application, so the crate as a whole performs I/O by design. What the gate holds is
//! the functional core inside it: `crate::domain` makes no I/O call and no inline clock read,
//! exposes no `async fn`, and never reaches back into the shell that drives it.

#![forbid(unsafe_code)]

use std::{env, process::ExitCode};

use tianheng::prelude::*;

const APP_REASON: &str = "seed is the application; it must never depend on its own governance gate, which judges the workspace from outside it.";
const GOVERNANCE_REASON: &str = "the governance gate must stay independent of the workspace graph it judges: its normal dependencies are Tianheng's composed adopter surface alone, never an individual governance instrument or a workspace crate under judgment.";
const DOMAIN_PURITY_REASON: &str = "crate::domain is the functional core: it makes no inline `std::time` `now` call and exposes no async function; time and asynchronous driving live in the shell. Coverage is partial by nature (a clock read through a method on a value, such as `Instant::elapsed`, is invisible to a source scan), so this tooth complements review rather than replacing it.";
const DOMAIN_NO_IO_REASON: &str = "the functional core performs no I/O: no code in crate::domain may call into std::io/fs/net/process; every effect lives in crate::shell. Coverage is partial by nature (macro-expanded I/O such as println! is invisible to a source scan), so this tooth complements review rather than replacing it.";
const DOMAIN_NO_SHELL_REASON: &str = "the functional core never reaches back into the shell that drives it: crate::domain must not import crate::shell.";

fn constitution() -> Constitution {
    let constitution = Constitution::new("seed")
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
        .sans_io_pure(
            SansIoPure::in_crate("seed")
                .module("crate::domain")
                .reading_clock_via("std::time", ["now"])
                .because(DOMAIN_PURITY_REASON),
        )
        .boundary(
            ModuleBoundary::in_crate("seed")
                .module("crate::domain")
                .must_not_import("crate::shell")
                .because(DOMAIN_NO_SHELL_REASON),
        );

    ["std::io", "std::fs", "std::net", "std::process"]
        .into_iter()
        .fold(constitution, |constitution, path| {
            constitution.boundary(
                ModuleBoundary::in_crate("seed")
                    .module("crate::domain")
                    .must_not_call_inline(path)
                    .because(DOMAIN_NO_IO_REASON),
            )
        })
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
    fn domain_io_calls_are_rejected() {
        for (path, call) in [
            ("std::fs", "let _ = std::fs::read(\"x\");"),
            ("std::io", "let _ = std::io::stdin();"),
            (
                "std::net",
                "let _ = std::net::TcpStream::connect(\"127.0.0.1:1\");",
            ),
            ("std::process", "let _ = std::process::Command::new(\"x\");"),
        ] {
            let report = violations(
                &format!("seed-governance-domain-{}", path.replace("::", "-")),
                &format!("pub fn leak() {{\n    {call}\n}}\n"),
                "",
            );
            assert!(
                report
                    .violations
                    .iter()
                    .any(|violation| violation.target() == path),
                "expected the domain no-I/O boundary for {path} to fire: {report:?}"
            );
        }
    }

    #[test]
    fn app_depending_on_its_gate_is_rejected() {
        let workspace = TempWorkspace::new("seed-governance-app-dependency", "", "");
        workspace.write_package(
            "seed",
            "[dependencies]\nseed-governance = { path = \"../seed-governance\" }\n",
            &[
                ("lib.rs", "pub mod domain;\npub mod shell;\n"),
                ("domain.rs", ""),
                ("shell.rs", ""),
            ],
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
        let workspace = TempWorkspace::new("seed-governance-gate-dependency", "", "");
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
    fn domain_ambient_clock_is_rejected() {
        let report = violations(
            "seed-governance-domain-clock",
            "pub fn leak() -> std::time::SystemTime {\n    std::time::SystemTime::now()\n}\n",
            "",
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.target() == "std::time"),
            "expected the domain ambient-clock boundary to fire: {report:?}"
        );
    }

    #[test]
    fn domain_async_exposure_is_rejected() {
        let report = violations(
            "seed-governance-domain-async",
            "pub async fn leak() {}\n",
            "",
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.rule == "must not expose async fn"),
            "expected the domain async-exposure boundary to fire: {report:?}"
        );
    }

    #[test]
    fn domain_importing_shell_is_rejected() {
        let report = violations(
            "seed-governance-domain-shell",
            "use crate::shell;\n",
            "pub fn run() {}\n",
        );
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.rule.contains("import")),
            "expected the domain-to-shell import boundary to fire: {report:?}"
        );
    }

    #[test]
    fn shell_io_with_pure_domain_stays_clean() {
        let workspace = TempWorkspace::new(
            "seed-governance-clean",
            "pub fn decide() {}\n",
            "pub fn run() {\n    let _ = std::fs::read(\"x\");\n}\n",
        );
        let outcome = workspace.outcome();
        assert!(
            matches!(outcome, Outcome::Clean(_)),
            "shell I/O over a pure domain must raise no violation: {outcome:?}"
        );
    }

    fn violations(name: &str, domain: &str, shell: &str) -> Report {
        match TempWorkspace::new(name, domain, shell).outcome() {
            Outcome::Violations(report) => report,
            other => panic!("expected violations, got {other:?}"),
        }
    }

    /// A scratch workspace holding the app crate (with the given `domain` and `shell` modules) and
    /// its governance crate, so every boundary has a real target.
    struct TempWorkspace {
        path: PathBuf,
    }

    impl TempWorkspace {
        fn new(name: &str, domain: &str, shell: &str) -> Self {
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
            workspace.write_package(
                "seed",
                "",
                &[
                    ("lib.rs", "pub mod domain;\npub mod shell;\n"),
                    ("domain.rs", domain),
                    ("shell.rs", shell),
                ],
            );
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
