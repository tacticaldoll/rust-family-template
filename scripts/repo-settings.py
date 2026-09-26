#!/usr/bin/env python3
"""Compare, and optionally apply, a family repository's GitHub settings.

usage: repo-settings.py <owner/repo> <local-checkout> [--apply]

The required status checks are the display names of every job in the checkout's
.github/workflows/ci.yml (a `${{ matrix.<key> }}` in a name expands over that job's matrix list).
Every other value is the family standard stated in FAMILY.md, "Repository Settings". Without
`--apply` this only reads GitHub (`gh api`); with it, it writes the repository and `main`
protection settings. It also reports GitHub Release objects, which `--apply` never removes.
Exit 0 when the repository matches, 1 on drift, 2 on a usage or API error.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

GITHUB_ACTIONS_APP_ID = 15368

REPOSITORY = {
    "allow_squash_merge": True,
    "allow_merge_commit": False,
    "allow_rebase_merge": False,
    "squash_merge_commit_title": "PR_TITLE",
    "squash_merge_commit_message": "PR_BODY",
    "delete_branch_on_merge": True,
}


def required_checks(ci: str) -> list[str]:
    lines = ci.splitlines()
    start = next(i for i, line in enumerate(lines) if re.match(r"^jobs:\s*$", line))
    body = lines[start + 1 :]
    indent = next(len(l) - len(l.lstrip()) for l in body if l.strip() and not l.lstrip().startswith("#"))
    jobs: list[list[str]] = []
    for line in body:
        if line.strip() and not line.startswith(" "):
            break
        if re.match(rf"^ {{{indent}}}[A-Za-z0-9_-]+:\s*$", line):
            jobs.append([])
        elif jobs:
            jobs[-1].append(line)
    names = []
    for job in jobs:
        text = "\n".join(job)
        name = re.search(rf"^ {{{indent * 2}}}name:\s*(.+?)\s*$", text, re.M)
        if not name:
            raise ValueError("every CI job needs a display name; it is the required-check name")
        display = name.group(1).strip("'\"")
        placeholder = re.search(r"\$\{\{\s*matrix\.([A-Za-z0-9_-]+)\s*\}\}", display)
        if placeholder:
            values = re.search(rf"^\s+{placeholder.group(1)}:\s*\[(.*?)\]\s*$", text, re.M)
            if not values:
                raise ValueError(f"cannot expand matrix key in job name {display!r}")
            for value in (v.strip().strip("'\"") for v in values.group(1).split(",")):
                names.append(display.replace(placeholder.group(0), value))
        else:
            names.append(display)
    return names


def protection(checks: list[str]) -> dict:
    return {
        "required_status_checks": {
            "strict": True,
            "checks": [{"context": name, "app_id": GITHUB_ACTIONS_APP_ID} for name in checks],
        },
        "enforce_admins": True,
        "required_pull_request_reviews": {
            "dismiss_stale_reviews": False,
            "require_code_owner_reviews": False,
            "require_last_push_approval": False,
            "required_approving_review_count": 0,
        },
        "restrictions": None,
        "required_linear_history": True,
        "allow_force_pushes": False,
        "allow_deletions": False,
    }


def gh(*args: str, payload: dict | None = None) -> dict | None:
    command = ["gh", "api", *args]
    if payload is not None:
        command += ["--input", "-"]
    result = subprocess.run(command, input=json.dumps(payload) if payload is not None else None,
                            capture_output=True, text=True)
    if result.returncode != 0:
        if "Branch not protected" in result.stdout + result.stderr:
            return None
        raise RuntimeError(f"gh api {' '.join(args)} failed: {result.stderr.strip() or result.stdout.strip()}")
    return json.loads(result.stdout) if result.stdout.strip() else {}


def drift(repo: str, checks: list[str]) -> list[str]:
    found = []
    settings = gh(f"repos/{repo}")
    for key, value in REPOSITORY.items():
        if settings.get(key) != value:
            found.append(f"repository {key} is {settings.get(key)!r}, want {value!r}")
    releases: list[dict] = []
    for page in range(1, 100):
        batch = gh(f"repos/{repo}/releases?per_page=100&page={page}") or []
        releases += batch
        if len(batch) < 100:
            break
    if releases:
        tags = ", ".join(sorted(r["tag_name"] for r in releases))
        found.append(f"has {len(releases)} GitHub Release object(s) ({tags}); the family keeps "
                      "release notes in CHANGELOG.md only, so remove them by hand")
    current = gh(f"repos/{repo}/branches/main/protection")
    if current is None:
        return found + ["main is not protected"]
    status = current.get("required_status_checks") or {}
    have = sorted(c["context"] for c in status.get("checks", []))
    if have != sorted(checks):
        found.append(f"required checks are {have}, want {sorted(checks)}")
    if not status.get("strict"):
        found.append("required checks do not require an up-to-date branch")
    if not (current.get("enforce_admins") or {}).get("enabled"):
        found.append("protection is not enforced for administrators")
    reviews = current.get("required_pull_request_reviews")
    if reviews is None:
        found.append("pull requests are not required")
    elif reviews.get("required_approving_review_count", 0) != 0:
        found.append("required approvals must be 0")
    for key, want in [("required_linear_history", True), ("allow_force_pushes", False), ("allow_deletions", False)]:
        if (current.get(key) or {}).get("enabled") != want:
            found.append(f"{key} must be {str(want).lower()}")
    return found


def main(argv: list[str]) -> int:
    if len(argv) not in (3, 4) or (len(argv) == 4 and argv[3] != "--apply"):
        print("usage: repo-settings.py <owner/repo> <local-checkout> [--apply]", file=sys.stderr)
        return 2
    repo, checkout, apply = argv[1], Path(argv[2]), len(argv) == 4
    try:
        checks = required_checks((checkout / ".github/workflows/ci.yml").read_text())
        if apply:
            gh("-X", "PATCH", f"repos/{repo}", payload=REPOSITORY)
            gh("-X", "PUT", f"repos/{repo}/branches/main/protection", payload=protection(checks))
        found = drift(repo, checks)
    except (OSError, ValueError, RuntimeError, StopIteration) as error:
        print(f"repo-settings: {repo}: {error}", file=sys.stderr)
        return 2
    for line in found:
        print(f"repo-settings: {repo}: {line}")
    if found:
        print(f"repo-settings: {repo}: {len(found)} setting(s) differ from the family standard")
        return 1
    print(f"repo-settings: {repo}: matches the family standard (required checks: {', '.join(checks)})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
