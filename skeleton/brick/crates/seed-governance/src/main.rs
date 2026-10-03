//! Executable architectural governance for the seed workspace.
//!
//! It enforces dependency boundaries, the core's sans-I/O purity (no I/O call, no inline clock
//! read, no exposed `async fn`), the facade's re-exports-only shape, and workspace coverage. The
//! axiom that the core makes no semantic judgment has no syntactic marker: it stays
//! review-governed.

#![forbid(unsafe_code)]

use std::{env, process::ExitCode};

use tianheng::prelude::*;

const CONTRACT_REASON: &str = "seed-contract is the isolated core. It depends on nothing, and must never depend on another workspace crate or a runtime framework: its mechanism is pure.";
const GOVERNANCE_REASON: &str = "the governance gate must stay independent of the workspace graph it judges: its normal dependencies are Tianheng's composed adopter surface alone, never an individual governance instrument or a workspace crate under judgment.";
const FACADE_REASON: &str = "seed is the curated published entrypoint. It may depend only on seed-contract, never on a backend, runtime, or external framework.";
const CORE_PURITY_REASON: &str = "seed-contract makes no inline `std::time` `now` call and exposes no async function: time and asynchronous driving live at the runtime edge. Coverage is partial by nature (a clock read through a method on a value, such as `Instant::elapsed`, is invisible to a source scan), so this tooth complements review rather than replacing it.";
const CORE_NO_IO_REASON: &str = "the sans-I/O core performs no I/O: no code in seed-contract may call into std::io/fs/net/process; I/O lives in a runtime outside the core. Coverage is partial by nature (macro-expanded I/O such as println! is invisible to a source scan), so this tooth complements review rather than replacing it.";
const FACADE_REEXPORT_REASON: &str =
    "the seed facade must stay a pure re-export entrypoint and hold no logic of its own";

fn constitution() -> Constitution {
    let constitution = Constitution::new("seed")
        .boundary(
            CrateBoundary::crate_("seed-contract")
                .restrict_dependencies_to(Vec::<&str>::new())
                .because(CONTRACT_REASON),
        )
        .boundary(
            CrateBoundary::crate_("seed-governance")
                .restrict_dependencies_to(["tianheng"])
                .because(GOVERNANCE_REASON),
        )
        .boundary(
            CrateBoundary::crate_("seed")
                .restrict_dependencies_to(["seed-contract"])
                .because(FACADE_REASON),
        )
        .reexport_only_boundary(
            ReexportOnlyBoundary::in_crate("seed")
                .module("crate")
                .must_declare_only_reexports()
                .because(FACADE_REEXPORT_REASON),
        )
        .sans_io_pure(
            SansIoPure::in_crate("seed-contract")
                .module("crate")
                .reading_clock_via("std::time", ["now"])
                .because(CORE_PURITY_REASON),
        );

    ["std::io", "std::fs", "std::net", "std::process"]
        .into_iter()
        .fold(constitution, |constitution, path| {
            constitution.boundary(
                ModuleBoundary::in_crate("seed-contract")
                    .module("crate")
                    .must_not_call_inline(path)
                    .because(CORE_NO_IO_REASON),
            )
        })
}

fn main() -> ExitCode {
    tianheng::run(&constitution(), env::args())
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
    fn unapproved_core_dependency_is_rejected() {
        let workspace = TempWorkspace::new("seed-governance-core-dependency");
        workspace.write_package("tokio", "", "");
        workspace.write_package(
            "seed-contract",
            "[dependencies]\ntokio = { path = \"../tokio\" }\n",
            "",
        );

        let report = workspace.violations(&["seed-contract", "tokio"]);
        assert!(
            report.violations.iter().any(|violation| {
                violation.target() == "seed-contract"
                    && violation.rule == "restrict dependencies to"
                    && violation.finding == "tokio"
            }),
            "expected the core dependency boundary to fire: {report:?}"
        );
    }

    #[test]
    fn core_io_calls_are_rejected() {
        for (path, call) in [
            ("std::fs", "let _ = std::fs::read(\"x\");"),
            ("std::io", "let _ = std::io::stdin();"),
            (
                "std::net",
                "let _ = std::net::TcpStream::connect(\"127.0.0.1:1\");",
            ),
            ("std::process", "let _ = std::process::Command::new(\"x\");"),
        ] {
            let workspace =
                TempWorkspace::new(&format!("seed-governance-core-{}", path.replace("::", "-")));
            workspace.write_package(
                "seed-contract",
                "",
                &format!("pub fn leak() {{\n    {call}\n}}\n"),
            );

            let report = workspace.violations(&["seed-contract"]);
            assert!(
                report
                    .violations
                    .iter()
                    .any(|violation| violation.target() == path),
                "expected the core no-I/O boundary for {path} to fire: {report:?}"
            );
        }
    }

    #[test]
    fn unapproved_facade_dependency_is_rejected() {
        let workspace = TempWorkspace::new("seed-governance-facade-dependency");
        workspace.write_package("seed-contract", "", "");
        workspace.write_package("tokio", "", "");
        workspace.write_package(
            "seed",
            "[dependencies]\nseed-contract = { path = \"../seed-contract\" }\ntokio = { path = \"../tokio\" }\n",
            "",
        );

        let report = workspace.violations(&["seed-contract", "tokio"]);
        assert!(
            report
                .violations
                .iter()
                .any(|violation| { violation.target() == "seed" && violation.finding == "tokio" }),
            "expected the facade dependency boundary to fire: {report:?}"
        );
    }

    #[test]
    fn governance_dependency_beyond_tianheng_is_rejected() {
        let workspace = TempWorkspace::new("seed-governance-gate-dependency");
        workspace.write_package("seed-contract", "", "");
        workspace.write_package("tokio", "", "");
        workspace.write_package(
            "seed-governance",
            "[dependencies]\ntianheng = { path = \"../tianheng\" }\ntokio = { path = \"../tokio\" }\n",
            "",
        );

        let report = workspace.violations(&["seed-contract", "tokio"]);
        assert!(
            report.violations.iter().any(|violation| {
                violation.target() == "seed-governance" && violation.finding == "tokio"
            }),
            "expected the governance dependency boundary to fire: {report:?}"
        );
    }

    #[test]
    fn core_ambient_clock_is_rejected() {
        let workspace = TempWorkspace::new("seed-governance-core-clock");
        workspace.write_package(
            "seed-contract",
            "",
            "pub fn leak() -> std::time::SystemTime {\n    std::time::SystemTime::now()\n}\n",
        );

        let report = workspace.violations(&["seed-contract"]);
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.target() == "std::time"),
            "expected the core ambient-clock boundary to fire: {report:?}"
        );
    }

    #[test]
    fn core_async_exposure_is_rejected() {
        let workspace = TempWorkspace::new("seed-governance-core-async");
        workspace.write_package(
            "seed-contract",
            "",
            "pub mod inner {\n    pub async fn leak() {}\n}\n",
        );

        let report = workspace.violations(&["seed-contract"]);
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.rule == "must not expose async fn"),
            "expected the core async-exposure boundary to fire: {report:?}"
        );
    }

    #[test]
    fn pure_core_stays_clean() {
        let workspace = TempWorkspace::new("seed-governance-core-clean");
        workspace.write_package("seed-contract", "", "pub fn decide() {}\n");

        let outcome = workspace.outcome(&["seed-contract"]);
        assert!(
            matches!(outcome, Outcome::Clean(_)),
            "a pure core must raise no violation: {outcome:?}"
        );
    }

    #[test]
    fn facade_logic_item_is_rejected() {
        let workspace = TempWorkspace::new("seed-governance-facade-logic");
        workspace.write_package("seed-contract", "", "pub struct Verdict;\n");
        workspace.write_package(
            "seed",
            "[dependencies]\nseed-contract = { path = \"../seed-contract\" }\n",
            "pub use seed_contract::Verdict;\n\npub fn helper() {}\n",
        );

        let report = workspace.violations(&["seed-contract"]);
        assert!(
            report.violations.iter().any(|violation| {
                violation.rule == "must declare only re-exports" && violation.finding == "fn helper"
            }),
            "expected the facade re-export-only boundary to fire: {report:?}"
        );
    }

    #[test]
    fn facade_child_module_is_rejected() {
        let workspace = TempWorkspace::new("seed-governance-facade-module");
        workspace.write_package("seed-contract", "", "pub struct Verdict;\n");
        workspace.write_package(
            "seed",
            "[dependencies]\nseed-contract = { path = \"../seed-contract\" }\n",
            "pub mod prelude {\n    pub use seed_contract::Verdict;\n}\n",
        );

        let report = workspace.violations(&["seed-contract"]);
        assert!(
            report.violations.iter().any(|violation| {
                violation.rule == "must declare only re-exports"
                    && violation.finding.contains("prelude")
            }),
            "expected a facade child module to fire the re-export-only boundary: {report:?}"
        );
    }

    #[test]
    fn facade_reexports_stay_clean() {
        let workspace = TempWorkspace::new("seed-governance-facade-clean");
        workspace.write_package(
            "seed-contract",
            "",
            "pub struct Decision;\npub struct Verdict;\n",
        );
        workspace.write_package(
            "seed",
            "[dependencies]\nseed-contract = { path = \"../seed-contract\" }\n",
            "//! Facade docs.\n#![no_std]\n\nuse seed_contract::Decision;\npub use seed_contract::{Decision as Choice, Verdict};\n",
        );

        let outcome = workspace.outcome(&["seed-contract"]);
        assert!(
            matches!(outcome, Outcome::Clean(_)),
            "a facade of re-exports must raise no violation: {outcome:?}"
        );
    }

    /// A scratch workspace that always carries the governance and facade crates, so every
    /// crate-level boundary has a real target and a fixture differs from a clean one only in what
    /// the test writes.
    struct TempWorkspace {
        path: PathBuf,
    }

    impl TempWorkspace {
        fn new(name: &str) -> Self {
            let path = env::temp_dir().join(format!("{name}-{}", std::process::id()));
            if path.exists() {
                fs::remove_dir_all(&path).expect("stale temporary workspace should be removable");
            }
            fs::create_dir_all(&path).expect("temporary workspace should be creatable");
            let workspace = Self { path };
            workspace.write_package("tianheng", "", "");
            workspace.write_package(
                "seed-governance",
                "[dependencies]\ntianheng = { path = \"../tianheng\" }\n",
                "",
            );
            workspace.write_package(
                "seed",
                "[dependencies]\nseed-contract = { path = \"../seed-contract\" }\n",
                "",
            );
            workspace
        }

        fn outcome(&self, members: &[&str]) -> Outcome {
            let entries = ["tianheng", "seed-governance", "seed"]
                .iter()
                .chain(members)
                .map(|member| format!("    \"{member}\","))
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(
                self.path.join("Cargo.toml"),
                format!("[workspace]\nresolver = \"2\"\nmembers = [\n{entries}\n]\n"),
            )
            .expect("workspace manifest should be writable");

            tianheng::check_constitution(&constitution(), &self.path.join("Cargo.toml"))
        }

        fn violations(&self, members: &[&str]) -> Report {
            match self.outcome(members) {
                Outcome::Violations(report) => report,
                other => panic!("expected violations, got {other:?}"),
            }
        }

        fn write_package(&self, name: &str, dependencies: &str, source: &str) {
            let package = self.path.join(name);
            fs::create_dir_all(package.join("src")).expect("package source dir should be writable");
            fs::write(
                package.join("Cargo.toml"),
                format!(
                    "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n{dependencies}"
                ),
            )
            .expect("package manifest should be writable");
            fs::write(package.join("src/lib.rs"), source)
                .expect("package source should be writable");
        }
    }

    impl Drop for TempWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
