# Seed Tianheng Law Projection

This file is generated from `constitution()` in `crates/seed-governance/src/main.rs`.
The Rust declaration is authoritative; do not edit the projection by hand.
Regenerate it with `BLESS=1 cargo test -p seed-governance law_projection_is_fresh`.

# Constitution: seed

## Static boundaries

### `seed-contract` (crate)

> seed-contract is the isolated core. It depends on nothing, and must never depend on another workspace crate or a runtime framework: its mechanism is pure.

- **rule**: restrict dependencies to (only: )
- **kind**: crate · **severity**: enforce

### `seed-governance` (crate)

> the governance gate must stay independent of the workspace graph it judges: its normal dependencies are Tianheng's composed adopter surface alone, never an individual governance instrument or a workspace crate under judgment.

- **rule**: restrict dependencies to (only: tianheng)
- **kind**: crate · **severity**: enforce

### `seed` (crate)

> seed is the curated published entrypoint. It may depend only on seed-contract, never on a backend, runtime, or external framework.

- **rule**: restrict dependencies to (only: seed-contract)
- **kind**: crate · **severity**: enforce

### `seed-contract::crate` (module)

> seed-contract makes no inline `std::time` `now` call and exposes no async function: time and asynchronous driving live at the runtime edge. Coverage is partial by nature (a clock read through a method on a value, such as `Instant::elapsed`, is invisible to a source scan), so this tooth complements review rather than replacing it.

- **rule**: inline symbol path confined to module (confined_prefix: std::time; ending_with: now)
- **kind**: module · **severity**: enforce · **crate**: seed-contract

### `seed-contract::crate` (module)

> the sans-I/O core performs no I/O: no code in seed-contract may call into std::io/fs/net/process; I/O lives in a runtime outside the core. Coverage is partial by nature (macro-expanded I/O such as println! is invisible to a source scan), so this tooth complements review rather than replacing it.

- **rule**: inline symbol path confined to module (confined_prefix: std::io)
- **kind**: module · **severity**: enforce · **crate**: seed-contract

### `seed-contract::crate` (module)

> the sans-I/O core performs no I/O: no code in seed-contract may call into std::io/fs/net/process; I/O lives in a runtime outside the core. Coverage is partial by nature (macro-expanded I/O such as println! is invisible to a source scan), so this tooth complements review rather than replacing it.

- **rule**: inline symbol path confined to module (confined_prefix: std::fs)
- **kind**: module · **severity**: enforce · **crate**: seed-contract

### `seed-contract::crate` (module)

> the sans-I/O core performs no I/O: no code in seed-contract may call into std::io/fs/net/process; I/O lives in a runtime outside the core. Coverage is partial by nature (macro-expanded I/O such as println! is invisible to a source scan), so this tooth complements review rather than replacing it.

- **rule**: inline symbol path confined to module (confined_prefix: std::net)
- **kind**: module · **severity**: enforce · **crate**: seed-contract

### `seed-contract::crate` (module)

> the sans-I/O core performs no I/O: no code in seed-contract may call into std::io/fs/net/process; I/O lives in a runtime outside the core. Coverage is partial by nature (macro-expanded I/O such as println! is invisible to a source scan), so this tooth complements review rather than replacing it.

- **rule**: inline symbol path confined to module (confined_prefix: std::process)
- **kind**: module · **severity**: enforce · **crate**: seed-contract

## Async-exposure boundaries

### `seed-contract::crate` (semantic)

> seed-contract makes no inline `std::time` `now` call and exposes no async function: time and asynchronous driving live at the runtime edge. Coverage is partial by nature (a clock read through a method on a value, such as `Instant::elapsed`, is invisible to a source scan), so this tooth complements review rather than replacing it.

- **rule**: must not expose async fn (including_submodules: true; scan_depth: subtree)
- **kind**: semantic · **severity**: enforce · **crate**: seed-contract
