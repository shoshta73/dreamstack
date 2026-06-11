# Developer guide

## Requirements

- [rustup](https://rustup.rs) — the toolchain (Rust **1.96.0**) is pinned in `rust-toolchain.toml` and will be installed automatically.

## Commands

```sh
cargo run              # debug
cargo build            # debug (no run)
cargo run --release    # release
cargo build --release  # release (no run)
cargo test             # unit tests, including inline insta snapshots
cargo insta review     # review pending snapshot changes
cargo insta accept     # accept pending snapshot changes
cargo fmt              # rustfmt
cargo clippy           # clippy lint
```

## Versioning

Package version defined in `Cargo.toml`, constrained to user facing features/changes
Development releases use the format YYwWW[weekly build], where YY are last two digits of the year and WW is week number. The weekly build follows this schema:
```
a - first build of the week
b - second build of the week
c - third build of the week
d - fourth build of the week
etc...
```

The weekly build letter is a base-26 ordinal (a=1, b=2, ..., z=26, aa=27, etc.)
