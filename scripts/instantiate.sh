#!/usr/bin/env bash
# Instantiate a new family repository.
#
# usage: scripts/instantiate.sh <brick|app> <name> <dest-dir>
#
# A brick is stamped from skeleton/brick, which carries the shared brick architecture. An
# application is born from style/app overlaid with kit/app: the shared style plus the thinnest
# workspace that passes its own gates (an empty library, its governance crate holding only the
# gate-independence law, and PROJECT.md with the intent left to write). Its architecture and the
# rest of its law are its own; see FAMILY.md, "Profiles".
#
# Copies the profile's sources to <dest-dir> and renames the placeholder product `seed` / `Seed` to
# <name> / <Name> in paths and file contents, including the lockfile, so the new workspace resolves
# the same dependency versions its sources were verified with, then runs `cargo fmt --all`. The
# result is a standalone workspace: it neither references nor depends on this template afterwards.
set -euo pipefail

usage() {
  echo "usage: $0 <brick|app> <name> <dest-dir>" >&2
  exit 2
}

[ "$#" -eq 3 ] || usage
profile=$1
name=$2
dest=$3
[ "$profile" = brick ] || [ "$profile" = app ] || usage

if ! printf '%s' "$name" | grep -qE '^[a-z][a-z0-9]*$'; then
  echo "instantiate: <name> must be lowercase ASCII letters and digits" >&2
  exit 2
fi
# Rust keywords and names that collide with the toolchain or the governance dependency cannot name
# a crate that builds.
reserved=" as async await box break const continue crate dyn else enum extern false final fn for gen
  if impl in let loop macro match mod move mut override priv pub ref return self static struct super
  trait true try type typeof unsafe unsized use virtual where while yield abstract become do
  alloc core proc std test tianheng "
for word in $reserved; do
  if [ "$word" = "$name" ]; then
    echo "instantiate: '$name' is a Rust keyword or reserved crate name" >&2
    exit 2
  fi
done
if [ -e "$dest" ] && [ -n "$(ls -A "$dest" 2>/dev/null)" ]; then
  echo "instantiate: $dest exists and is not empty" >&2
  exit 2
fi

title="$(printf '%s' "${name:0:1}" | tr '[:lower:]' '[:upper:]')${name:1}"
template="$(cd "$(dirname "$0")/.." && pwd)"
if [ "$profile" = brick ]; then
  sources=("$template/skeleton/brick")
else
  sources=("$template/style/app" "$template/kit/app")
fi

created=""
cleanup() {
  if [ -n "$created" ]; then
    rm -rf "$created"
  fi
}
trap cleanup EXIT

mkdir -p "$dest"
created="$(cd "$dest" && pwd)"
# Later sources overlay earlier ones: an application's kit completes its style.
for source in "${sources[@]}"; do
  (cd "$source" && tar --exclude=./target -cf - .) | (cd "$created" && tar -xf -)
done

cd "$created"
find . -depth -name '*seed*' | while IFS= read -r path; do
  mv "$path" "$(dirname "$path")/$(basename "$path" | sed "s/seed/$name/g")"
done
grep -rlI -e seed -e Seed . | while IFS= read -r file; do
  NAME="$name" TITLE="$title" perl -pi -e 's/seed/$ENV{NAME}/g; s/Seed/$ENV{TITLE}/g' "$file"
done

# Renaming changes line lengths, so the result is reformatted to stay rustfmt-clean.
cargo fmt --all
# Resolving once normalises the renamed lockfile, so the first build leaves the tree unchanged.
# Offline first, from the local registry cache; a host without that cache resolves online.
cargo metadata --format-version 1 --offline >/dev/null 2>&1 ||
  cargo metadata --format-version 1 >/dev/null
created=""
echo "instantiate: created $profile repository '$name' in $dest"
