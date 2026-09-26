#!/usr/bin/env python3
"""Check that the brick skeleton and the application style share the family style byte for byte.

usage: profile-check.py

A brick is stamped from skeleton/brick/; an application adopts only style/app/. What both
profiles share (the AGENTS.md preamble, the sections both list as shared, the opening paragraph of
each prefix section, the base Definition of Done, the whole-file copies, and the law projection
preamble) must not drift between the two references, or a family rule would differ by profile
without anyone deciding it. Reads the template only; exit 0 when the references agree, 1 on drift.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    "family_check", Path(__file__).resolve().parent / "family-check.py"
)
fc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fc)


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

    for line in drift:
        print(f"profile-check: {line}")
    if drift:
        print(f"profile-check: {len(drift)} difference(s) between skeleton/brick and style/app")
        return 1
    print("profile-check: skeleton/brick and style/app share the family style")
    return 0


if __name__ == "__main__":
    sys.exit(main())
