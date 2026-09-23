# Seed Tianheng Law Projection

This file is generated from `constitution()` in `crates/seed-governance/src/main.rs`.
The Rust declaration is authoritative; do not edit the projection by hand.
Regenerate it with `BLESS=1 cargo test -p seed-governance law_projection_is_fresh`.

# Constitution: seed

## Static boundaries

### `seed` (crate)

> seed is the application; it must never depend on its own governance gate, which judges the workspace from outside it.

- **rule**: forbid dependency on (crates: seed-governance)
- **kind**: crate · **severity**: enforce

### `seed-governance` (crate)

> the governance gate must stay independent of the workspace graph it judges: its normal dependencies are Tianheng's composed adopter surface alone, never an individual governance instrument or a workspace crate under judgment.

- **rule**: restrict dependencies to (only: tianheng)
- **kind**: crate · **severity**: enforce

### `seed::crate::domain` (module)

> crate::domain is the functional core: it makes no inline `std::time` `now` call and exposes no async function; time and asynchronous driving live in the shell. Coverage is partial by nature (a clock read through a method on a value, such as `Instant::elapsed`, is invisible to a source scan), so this tooth complements review rather than replacing it.

- **rule**: inline symbol path confined to module (confined_prefix: std::time; ending_with: now)
- **kind**: module · **severity**: enforce · **crate**: seed

### `seed::crate::domain` (module)

> the functional core never reaches back into the shell that drives it: crate::domain must not import crate::shell.

- **rule**: module must not import (forbidden: crate::shell)
- **kind**: module · **severity**: enforce · **crate**: seed

### `seed::crate::domain` (module)

> the functional core performs no I/O: no code in crate::domain may call into std::io/fs/net/process; every effect lives in crate::shell. Coverage is partial by nature (macro-expanded I/O such as println! is invisible to a source scan), so this tooth complements review rather than replacing it.

- **rule**: inline symbol path confined to module (confined_prefix: std::io)
- **kind**: module · **severity**: enforce · **crate**: seed

### `seed::crate::domain` (module)

> the functional core performs no I/O: no code in crate::domain may call into std::io/fs/net/process; every effect lives in crate::shell. Coverage is partial by nature (macro-expanded I/O such as println! is invisible to a source scan), so this tooth complements review rather than replacing it.

- **rule**: inline symbol path confined to module (confined_prefix: std::fs)
- **kind**: module · **severity**: enforce · **crate**: seed

### `seed::crate::domain` (module)

> the functional core performs no I/O: no code in crate::domain may call into std::io/fs/net/process; every effect lives in crate::shell. Coverage is partial by nature (macro-expanded I/O such as println! is invisible to a source scan), so this tooth complements review rather than replacing it.

- **rule**: inline symbol path confined to module (confined_prefix: std::net)
- **kind**: module · **severity**: enforce · **crate**: seed

### `seed::crate::domain` (module)

> the functional core performs no I/O: no code in crate::domain may call into std::io/fs/net/process; every effect lives in crate::shell. Coverage is partial by nature (macro-expanded I/O such as println! is invisible to a source scan), so this tooth complements review rather than replacing it.

- **rule**: inline symbol path confined to module (confined_prefix: std::process)
- **kind**: module · **severity**: enforce · **crate**: seed

## Async-exposure boundaries

### `seed::crate::domain` (semantic)

> crate::domain is the functional core: it makes no inline `std::time` `now` call and exposes no async function; time and asynchronous driving live in the shell. Coverage is partial by nature (a clock read through a method on a value, such as `Instant::elapsed`, is invisible to a source scan), so this tooth complements review rather than replacing it.

- **rule**: must not expose async fn (including_submodules: true; scan_depth: subtree)
- **kind**: semantic · **severity**: enforce · **crate**: seed
