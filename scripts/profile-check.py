#!/usr/bin/env python3
"""Check that the brick and application references share the family style byte for byte.

usage: profile-check.py

A brick is stamped from skeleton/brick/; an application is born from style/app/ overlaid with
kit/app/. What both profiles share (the AGENTS.md preamble, the sections both list as shared, the
opening paragraph of each prefix section, the base Definition of Done, the whole-file copies, and
the law projection preamble) must not drift between the references, or a family rule would differ
by profile without anyone deciding it. The birth kit must also host law the way the brick skeleton
does: the same Tianheng requirement, gate-independence reason, law projection preamble, workspace
root, and the three family governance tests. Reads the template only; exit 0 when the references
agree, 1 on drift.
"""

from __future__ import annotations

import importlib.util
import re
import sys
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    "family_check", Path(__file__).resolve().parent / "family-check.py"
)
fc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fc)


GOVERNANCE = "crates/seed-governance/src/main.rs"


def function_body(source: str, name: str) -> str | None:
    """The text of `fn <name>(...)` through its matching closing brace, or None."""
    match = re.search(rf"\bfn {name}\(", source)
    if not match:
        return None
    start = source.index("{", match.end())
    depth = 0
    for index in range(start, len(source)):
        depth += {"{": 1, "}": -1}.get(source[index], 0)
        if depth == 0:
            return source[match.start() : index + 1]
    return None


def governance_harness(reference: Path) -> dict[str, str | None]:
    source = fc.read(reference / GOVERNANCE) or ""
    manifest = fc.read(reference / "Cargo.toml") or ""
    harness = {
        "tianheng requirement": (re.search(r'^tianheng = ".*"$', manifest, re.M) or [None])[0],
        "GOVERNANCE_REASON": (
            re.search(r"^const GOVERNANCE_REASON: .*$", source, re.M) or [None]
        )[0],
        "LAW_PROJECTION_PREAMBLE": (
            re.search(r'const LAW_PROJECTION_PREAMBLE: &str = "\\\n.*?";', source, re.S) or [None]
        )[0],
    }
    for name in ["workspace_root", *fc.GOVERNANCE_TESTS]:
        harness[f"fn {name}"] = function_body(source, name)
    return harness


def main() -> int:
    brick, app = fc.REFERENCE["brick"], fc.REFERENCE["app"]
    drift = []
    brick_preamble, brick_sections, _ = fc.sections(fc.read(brick / "AGENTS.md") or "")
    app_preamble, app_sections, _ = fc.sections(fc.read(app / "AGENTS.md") or "")
    if fc.trim(brick_preamble) != fc.trim(app_preamble):
        drift.append("AGENTS.md: preamble differs between profiles")
    for heading in (h for h in fc.SHARED_SECTIONS["brick"] if h in fc.SHARED_SECTIONS["app"]):
        if fc.trim(brick_sections.get(heading, "")) != fc.trim(app_sections.get(heading, "")):
            drift.append(f"AGENTS.md: shared section `## {heading}` differs between profiles")
    for heading in fc.PREFIX_SECTIONS:
        if fc.first_paragraph(brick_sections.get(heading, "")) != fc.first_paragraph(
            app_sections.get(heading, "")
        ):
            drift.append(f"AGENTS.md: opening paragraph of `## {heading}` differs between profiles")
    if fc.bash_commands(brick_sections.get("Definition Of Done", "")) != fc.bash_commands(
        app_sections.get("Definition Of Done", "")
    ):
        drift.append("AGENTS.md: base Definition of Done differs between profiles")
    for relative in [*fc.EXACT_FILES, ".gitignore", "CHANGELOG.md", "deny.toml"]:
        if fc.read(brick / relative) != fc.read(app / relative):
            drift.append(f"{relative}: differs between profiles")
    law = "AGENTS.seed-law.md"
    preambles = [(fc.read(ref / law) or "").split("# Constitution:", 1)[0] for ref in (brick, app)]
    if preambles[0] != preambles[1]:
        drift.append(f"{law}: generated preamble differs between profiles")

    brick_harness = governance_harness(brick)
    kit_harness = governance_harness(fc.TEMPLATE / "kit" / "app")
    for part, value in brick_harness.items():
        if value is None or value != kit_harness.get(part):
            drift.append(f"kit/app: governance {part} differs from skeleton/brick")

    for line in drift:
        print(f"profile-check: {line}")
    if drift:
        count = len(drift)
        print(f"profile-check: {count} difference(s) between the brick and application references")
        return 1
    print("profile-check: skeleton/brick, style/app, and kit/app share the family style")
    return 0


if __name__ == "__main__":
    sys.exit(main())
