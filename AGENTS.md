# AGENTS.md

## Project

- Single-crate Rust binary named `dreamstack`; entrypoint is `src/main.rs`.
- Tests currently live inline with the code and use `test-case` plus `insta` snapshots.

## Commands

```sh
cargo run              # debug build + run
cargo build            # debug build only
cargo run --release    # release build + run
cargo build --release  # release build only
cargo test             # run unit tests, including inline insta snapshots
cargo insta review     # review newly generated/changed snapshots
cargo insta accept     # accept all pending snapshot changes
cargo fmt              # rustfmt
cargo clippy           # clippy lint
```

## Repo Facts

- Rust toolchain is pinned to `1.96.0` in `rust-toolchain.toml`; let `rustup` install/use it.
- Rust edition is `2024` in `Cargo.toml`.
- `.editorconfig` sets 4-space indent for `*.rs`, 2-space indent elsewhere, LF endings, UTF-8, and max line length 120.
- `.gitignore` ignores dotfiles with `**/.*`; currently only `.editorconfig` is explicitly unignored (`!.gitingore` is misspelled). If adding a tracked dotfile or dot-directory such as `.github/`, fix/unignore it deliberately.
- Development versioning notes live in `DEVELOPMENT.md`; package version is the `Cargo.toml` version.
