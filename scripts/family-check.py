#!/usr/bin/env python3
"""Check a family repository against a family template profile.

usage: family-check.py <brick|app> <name> <repo-dir>

The skeleton's placeholder product `seed` / `Seed` is substituted with `<name>` / `<Name>` before
comparison; the skeleton uses the placeholder for nothing else, so the substitution is plain.
Every rule checked here is stated in FAMILY.md; this script is its mechanical projection. It reads
the repository (and runs `cargo metadata --no-deps` in it) and never writes to it.

Exit 0 when the repository conforms, 1 on drift, 2 on a usage or internal error.
"""

from __future__ import annotations

import json
import re
import shlex
import subprocess
import sys
import tomllib
from pathlib import Path

TEMPLATE = Path(__file__).resolve().parent.parent

# AGENTS.md sections that must match the skeleton, per profile.
SHARED_SECTIONS = {
    "brick": [
        "Lineage",
        "Document Authority",
        "Governance and Conformance",
        "OpenSpec Workflow",
        "Language",
        "Commit And Integration Governance",
    ],
    "app": [
        "Composition",
        "Document Authority",
        "Governance and Conformance",
        "OpenSpec Workflow",
        "Language",
        "Commit And Integration Governance",
    ],
}

# AGENTS.md sections whose first paragraph is shared and whose remainder is repository-owned.
PREFIX_SECTIONS = ["Architectural Axioms", "Adversarial Review Stance", "Definition Of Done"]

# Canonical AGENTS.md order after `<Name> In One Sentence`, where the profile has the section.
AGENTS_ORDER = [
    "Architectural Axioms",
    "Lineage",
    "Composition",
    "Document Authority",
    "Adversarial Review Stance",
    "Disposition Discipline",
    "Governance and Conformance",
    "OpenSpec Workflow",
    "Language",
    "Commit And Integration Governance",
    "Definition Of Done",
]

# Flags a repository may add to a base Definition of Done gate. Anything else changes the gate
# rather than extending it.
ALLOWED_GATE_FLAGS = {"--all-features", "--no-default-features", "--locked", "--all-targets"}

EXACT_FILES = [
    "LICENSE-APACHE",
    "LICENSE-MIT",
    "docs/development-flow.md",
    "openspec/config.yaml",
    "scripts/changelog-guard.sh",
]

REQUIRED_HEADINGS = {
    "PROJECT.md": ["Vision", "Product Positioning", "Core Contract", "Non-Goals", "References"],
    "README.md": ["Scope", "Architecture", "Contributing", "License"],
}

REQUIRED_PATHS = ["docs/domain-language.md", "openspec/specs", "BACKLOG.md"]

PROFILE_CRATES = {
    "brick": ["{name}-contract", "{name}", "{name}-governance"],
    "app": ["{name}", "{name}-governance"],
}

GOVERNANCE_TESTS = [
    "current_workspace_satisfies_constitution",
    "every_workspace_crate_is_covered",
    "law_projection_is_fresh",
]

CI_JOBS = {
    "dod": "Definition of Done",
    "msrv": "MSRV (1.88)",
    "supply-chain": "Supply chain (cargo-deny)",
    "governance": "Tianheng reaction",
}

WORKSPACE_PACKAGE = {
    "edition": "2024",
    "rust-version": "1.88",
    "license": "MIT OR Apache-2.0",
}

NAME = re.compile(r"^[a-z][a-z0-9]*$")


class Report:
    def __init__(self) -> None:
        self.drift: list[str] = []

    def add(self, where: str, what: str) -> None:
        self.drift.append(f"{where}: {what}")


def title_of(name: str) -> str:
    return name[:1].upper() + name[1:]


def substitute(text: str, name: str) -> str:
    return text.replace("seed", name).replace("Seed", title_of(name))


def read(path: Path) -> str | None:
    try:
        with path.open(encoding="utf-8", newline="") as handle:
            return handle.read()
    except (OSError, UnicodeDecodeError):
        return None


def skeleton_text(skeleton: Path, relative: str, name: str) -> str:
    text = read(skeleton / relative)
    if text is None:
        raise RuntimeError(f"template skeleton is missing {relative}")
    return substitute(text, name)


def sections(markdown: str) -> tuple[str, dict[str, str], list[str]]:
    """Split Markdown into (preamble, {level-2 heading: body}, heading order).

    Fenced code blocks (``` or ~~~, closed by a fence of the same character at least as long) and
    HTML comments are not scanned for headings.
    """
    preamble: list[str] = []
    bodies: dict[str, list[str]] = {}
    order: list[str] = []
    current = preamble
    fence: str | None = None
    in_comment = False
    for line in markdown.splitlines(keepends=True):
        stripped = line.strip()
        if fence is not None:
            if stripped.startswith(fence) and set(stripped) == {fence[0]}:
                fence = None
        elif in_comment:
            if "-->" in line:
                in_comment = False
        else:
            opening = re.match(r"^ {0,3}(`{3,}|~{3,})", line)
            if opening:
                fence = opening.group(1)
            elif stripped.startswith("<!--") and "-->" not in stripped:
                in_comment = True
            elif line.startswith("## "):
                title = re.sub(r"\s+#+\s*$", "", line[3:]).strip()
                order.append(title)
                current = bodies.setdefault(title, [])
                continue
        current.append(line)
    return "".join(preamble), {k: "".join(v) for k, v in bodies.items()}, order


def trim(text: str) -> str:
    """Drop the blank lines separating a section from its neighbours; keep everything else."""
    return text.strip("\n")


def first_paragraph(body: str) -> str:
    return trim(body).split("\n\n", 1)[0]


def bash_commands(body: str) -> list[list[str]]:
    match = re.search(r"```bash\n(.*?)```", body, re.S)
    commands = []
    for line in (match.group(1) if match else "").splitlines():
        code = line.split(" #", 1)[0].strip()
        if code:
            commands.append(shlex.split(code))
    return commands


def extends(command: list[str], base: list[str]) -> bool:
    """True when `command` is `base` with only allowed flags inserted."""
    remaining = list(base)
    for token in command:
        if remaining and token == remaining[0]:
            remaining.pop(0)
        elif token not in ALLOWED_GATE_FLAGS:
            return False
    return not remaining


def check_agents(profile: str, name: str, repo: Path, skeleton: Path, report: Report) -> None:
    actual = read(repo / "AGENTS.md")
    if actual is None:
        report.add("AGENTS.md", "missing")
        return
    exp_pre, exp, _ = sections(skeleton_text(skeleton, "AGENTS.md", name))
    act_pre, act, order = sections(actual)

    if trim(exp_pre) != trim(act_pre):
        report.add("AGENTS.md", "preamble differs from the skeleton")

    if not order or order[0] != f"{title_of(name)} In One Sentence":
        report.add("AGENTS.md", f"first section must be `## {title_of(name)} In One Sentence`")

    for heading in SHARED_SECTIONS[profile]:
        if heading not in act:
            report.add("AGENTS.md", f"shared section `## {heading}` is missing")
        elif trim(act[heading]) != trim(exp[heading]):
            report.add("AGENTS.md", f"shared section `## {heading}` differs from the skeleton")

    for heading in PREFIX_SECTIONS:
        if heading not in act:
            report.add("AGENTS.md", f"section `## {heading}` is missing")
        elif first_paragraph(act[heading]) != first_paragraph(exp[heading]):
            report.add("AGENTS.md", f"section `## {heading}` must open with the skeleton's paragraph")

    canonical = [h for h in AGENTS_ORDER if h in exp]
    present = [h for h in order if h in canonical]
    if present != [h for h in canonical if h in present]:
        report.add("AGENTS.md", f"sections out of canonical order: {present}")

    if "Definition Of Done" in act:
        commands = bash_commands(act["Definition Of Done"])
        for base in bash_commands(exp["Definition Of Done"]):
            if not any(extends(command, base) for command in commands):
                report.add("AGENTS.md", f"Definition Of Done lacks base gate `{shlex.join(base)}`")


def check_exact(name: str, repo: Path, skeleton: Path, report: Report) -> None:
    for relative in EXACT_FILES:
        actual = read(repo / relative)
        if actual is None:
            report.add(relative, "missing")
        elif actual != skeleton_text(skeleton, relative, name):
            report.add(relative, "differs from the skeleton")


def check_gitignore(name: str, repo: Path, skeleton: Path, report: Report) -> None:
    actual = read(repo / ".gitignore")
    if actual is None:
        report.add(".gitignore", "missing")
    elif not actual.startswith(skeleton_text(skeleton, ".gitignore", name)):
        report.add(".gitignore", "must open with the skeleton's entries; repository entries follow them")


def check_changelog(name: str, repo: Path, skeleton: Path, report: Report) -> None:
    actual = read(repo / "CHANGELOG.md")
    if actual is None:
        report.add("CHANGELOG.md", "missing")
        return
    expected = skeleton_text(skeleton, "CHANGELOG.md", name)
    if trim(re.split(r"^## ", actual, maxsplit=1, flags=re.M)[0]) != trim(expected):
        report.add("CHANGELOG.md", "preamble differs from the skeleton")
    if re.search(r"^## \[?unreleased\]?", actual, re.M | re.I):
        report.add("CHANGELOG.md", "carries an Unreleased section")
    for version in re.findall(r"^## \[(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)\]", actual, re.M):
        link = rf"^\[{re.escape(version)}\]: https://github\.com/[^/ ]+/[^/ ]+/releases/tag/v{re.escape(version)}$"
        if not re.search(link, actual, re.M):
            report.add("CHANGELOG.md", f"version {version} has no footer link to releases/tag/v{version}")


def load_toml(path: Path, report: Report, where: str) -> dict | None:
    text = read(path)
    if text is None:
        report.add(where, "missing")
        return None
    try:
        return tomllib.loads(text)
    except tomllib.TOMLDecodeError as error:
        report.add(where, f"is not valid TOML: {error}")
        return None


def check_deny(repo: Path, skeleton: Path, report: Report) -> None:
    actual = load_toml(repo / "deny.toml", report, "deny.toml")
    if actual is None:
        return
    expected = tomllib.loads(skeleton_text(skeleton, "deny.toml", "seed"))
    base = set(expected["licenses"].pop("allow"))
    allowed = actual.get("licenses", {}).pop("allow", [])
    if not base <= set(allowed):
        report.add("deny.toml", f"license allow-list must include the family base {sorted(base)}")
    if actual != expected:
        report.add("deny.toml", "differs from the skeleton outside the license allow-list")


def cargo_metadata(repo: Path, report: Report) -> dict | None:
    try:
        output = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1", "--offline"],
            cwd=repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    except subprocess.CalledProcessError as error:
        report.add("Cargo.toml", f"cargo metadata failed: {error.stderr.strip()}")
        return None
    return json.loads(output)


def check_workspace(profile: str, name: str, repo: Path, report: Report) -> None:
    manifest = load_toml(repo / "Cargo.toml", report, "Cargo.toml")
    if manifest is not None:
        package = manifest.get("workspace", {}).get("package", {})
        for key, value in WORKSPACE_PACKAGE.items():
            if package.get(key) != value:
                report.add("Cargo.toml", f"[workspace.package] {key} must be {value!r}")
        if "publish" in package:
            report.add("Cargo.toml", "[workspace.package] must not set publish; each crate states it")

    metadata = cargo_metadata(repo, report)
    if metadata is None:
        return
    members = set(metadata["workspace_members"])
    packages = {p["name"]: p for p in metadata["packages"] if p["id"] in members}
    for crate in (pattern.format(name=name) for pattern in PROFILE_CRATES[profile]):
        if crate not in packages:
            report.add("Cargo.toml", f"workspace lacks the profile crate `{crate}`")
    for crate, package in sorted(packages.items()):
        member = load_toml(Path(package["manifest_path"]), report, f"{crate}/Cargo.toml") or {}
        if not isinstance(member.get("package", {}).get("publish"), bool):
            report.add(f"{crate}/Cargo.toml", "must state publish = true or publish = false")

    governance = packages.get(f"{name}-governance")
    if governance is not None:
        if governance["publish"] != []:
            report.add(f"{name}-governance/Cargo.toml", "must be publish = false")
        deps = sorted({d["name"] for d in governance["dependencies"] if d["kind"] is None})
        if deps != ["tianheng"]:
            report.add(f"{name}-governance/Cargo.toml", f"must depend only on tianheng, found {deps}")


def job_blocks(ci: str) -> dict[str, str]:
    """Map each GitHub Actions job id to the non-comment lines of its block."""
    lines = ci.splitlines()
    start = next((i for i, line in enumerate(lines) if re.match(r"^jobs:\s*$", line)), None)
    if start is None:
        return {}
    body = lines[start + 1 :]
    indent = next(
        (len(l) - len(l.lstrip()) for l in body if l.strip() and not l.lstrip().startswith("#")),
        2,
    )
    blocks: dict[str, list[str]] = {}
    current: list[str] | None = None
    for line in body:
        if line.strip() and not line.startswith(" "):
            break
        match = re.match(rf"^ {{{indent}}}([A-Za-z0-9_-]+):\s*$", line)
        if match:
            current = blocks.setdefault(match.group(1), [])
        elif current is not None and not line.lstrip().startswith("#"):
            current.append(re.sub(r"\s+#.*$", "", line))
    return {job: "\n".join(block) for job, block in blocks.items()}


def check_governance(name: str, repo: Path, skeleton: Path, report: Report) -> None:
    law = read(repo / f"AGENTS.{name}-law.md")
    preamble = skeleton_text(skeleton, "AGENTS.seed-law.md", name).split("# Constitution:", 1)[0]
    if law is None:
        report.add(f"AGENTS.{name}-law.md", "missing")
    elif not law.startswith(preamble):
        report.add(f"AGENTS.{name}-law.md", "does not open with the skeleton's generated preamble")

    source = "".join(
        read(path) or "" for path in sorted((repo / f"crates/{name}-governance").rglob("*.rs"))
    )
    for test in GOVERNANCE_TESTS:
        if not re.search(rf"#\[test\]\s*\n\s*fn {test}\(\)", source):
            report.add(f"crates/{name}-governance", f"lacks the test `{test}`")

    jobs = job_blocks(read(repo / ".github/workflows/ci.yml") or "")
    for job, display in CI_JOBS.items():
        if job not in jobs:
            report.add(".github/workflows/ci.yml", f"lacks job `{job}`")
        elif not re.search(rf"^\s+name: {re.escape(display)}\s*$", jobs[job], re.M):
            report.add(".github/workflows/ci.yml", f"job `{job}` must be named `{display}`")
    if f"cargo run -p {name}-governance -- check" not in jobs.get("governance", ""):
        report.add(".github/workflows/ci.yml", f"governance job does not run `{name}-governance check`")
    if "./scripts/changelog-guard.sh" not in "".join(jobs.values()):
        report.add(".github/workflows/ci.yml", "no job runs ./scripts/changelog-guard.sh")


def check_headings(repo: Path, report: Report) -> None:
    for relative, required in REQUIRED_HEADINGS.items():
        text = read(repo / relative)
        if text is None:
            report.add(relative, "missing")
            continue
        _, _, order = sections(text)
        if [h for h in order if h in required] != required:
            report.add(relative, f"level-2 headings must include, in order: {required}")
    for relative in REQUIRED_PATHS:
        if not (repo / relative).exists():
            report.add(relative, "missing")


def run(profile: str, name: str, repo: Path) -> int:
    skeleton = TEMPLATE / "skeleton" / profile
    report = Report()
    check_agents(profile, name, repo, skeleton, report)
    check_exact(name, repo, skeleton, report)
    check_gitignore(name, repo, skeleton, report)
    check_changelog(name, repo, skeleton, report)
    check_deny(repo, skeleton, report)
    check_workspace(profile, name, repo, report)
    check_governance(name, repo, skeleton, report)
    check_headings(repo, report)

    for line in report.drift:
        print(f"family-check: {name}: {line}")
    if report.drift:
        print(f"family-check: {name}: {len(report.drift)} drift finding(s) against the {profile} profile")
        return 1
    print(f"family-check: {name}: conforms to the {profile} profile")
    return 0


def main(argv: list[str]) -> int:
    if len(argv) != 4 or argv[1] not in SHARED_SECTIONS or not NAME.match(argv[2]):
        print("usage: family-check.py <brick|app> <name> <repo-dir>", file=sys.stderr)
        print("  <name> is lowercase ASCII letters and digits", file=sys.stderr)
        return 2
    repo = Path(argv[3])
    if not repo.is_dir():
        print(f"family-check: {repo} is not a directory", file=sys.stderr)
        return 2
    try:
        return run(argv[1], argv[2], repo)
    except Exception as error:  # an internal failure must not read as drift
        print(f"family-check: internal error: {error!r}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
