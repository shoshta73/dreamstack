# AGENTS.md

## Project

- Single-crate Rust 2024 binary named `dreamstack`; `src/main.rs` starts the native `eframe`/`egui` app.
- Game data in `data/employers.json` and `data/servers.json` is embedded with `include_str!`; rebuild after changing either file.
- Autosaves write `autosave.json` via `autosave.json.tmp` in the repo root; both are ignored and should not be committed.

## Commands

```sh
cargo run              # debug build + run the native app
cargo build            # debug build
cargo run --release    # release build + run
cargo build --release  # release build
cargo test             # run unit tests, including inline insta snapshots
cargo test <name>      # run a focused test by name substring
cargo fmt --check      # CI formatting check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo insta review     # review newly generated/changed snapshots
cargo insta accept     # accept pending snapshot changes
```

## Testing

- Rust toolchain is pinned to `1.96.0` in `rust-toolchain.toml`; let `rustup` install/use it.
- CI runs `cargo fmt --check`, then clippy with `-D warnings`, then `cargo test --workspace`.
- Tests are inline in `src/*.rs` and use `test-case` plus inline `insta` snapshots.
- Run `cargo test` after snapshot edits before `cargo insta review` or `cargo insta accept`.

## Gotchas

- Debug logging reads `RUST_LOG`; release logging uses the fixed filter in `src/log.rs`.
- `.gitignore` ignores dotfiles with `**/.*`; only `.editorconfig` is unignored, and the intended `.gitignore` unignore is misspelled as `!.gitingore`.
- Development versioning notes live in `DEVELOPMENT.md`; the package version source of truth is `Cargo.toml`.
