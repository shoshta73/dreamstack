# Dreamstack

[![Build](https://github.com/shoshta73/dreamstack/actions/workflows/build.yml/badge.svg)](https://github.com/shoshta73/dreamstack/actions/workflows/build.yml)
[![CI](https://github.com/shoshta73/dreamstack/actions/workflows/ci.yml/badge.svg)](https://github.com/shoshta73/dreamstack/actions/workflows/ci.yml)
[![Dev Release](https://github.com/shoshta73/dreamstack/actions/workflows/dev-release.yml/badge.svg)](https://github.com/shoshta73/dreamstack/actions/workflows/dev-release.yml)

**Root your dreams. Escape the stack.**

Dreamstack is a native-performance, scriptable incremental hacking game about waking through nested dream layers, automating systems, and escaping the stack.

## Current State

Dreamstack currently has a playable Level 0 job loop and Level 1 hacking onboarding. Start with an 8-hour job, watch in-game time progress, earn money, company reputation, and charisma experience, then choose whether company reputation carries forward as favor. After the first work shift, Level 1 introduces hacking through an in-game terminal backed by embedded server data, hack skill requirements, security, money, and experience rewards.

The UI includes a collapsible left sidebar with hacking navigation, a terminal console pane, and a right player stats sidebar. The terminal supports `netscan`, `connect <hostname>`, `scan`, `nuke`, `npm i -g backdoor`, `hack`, and `home`. The game writes local autosaves during the run.

For release notes, see [CHANGELOG.md](CHANGELOG.md).

## Development

For detailed development notes and setup instructions, see [DEVELOPMENT.md](DEVELOPMENT.md).

## Windows

Dreamstack is supported as a native Windows desktop app through `eframe`/`egui`.

Requirements:

- Rust `1.96.0`, as pinned by `rust-toolchain.toml`.
- Microsoft C++ Build Tools or Visual Studio with the Desktop development with C++ workload.

Build and run:

```sh
cargo run
```

Autosaves are written to `autosave.json` in the current working directory. There are no known Windows-specific runtime limitations.

## Web

Dreamstack can be built for the browser with the `wasm32-unknown-unknown` target.

Requirements:

- Rust `1.96.0`, as pinned by `rust-toolchain.toml`.
- The `wasm32-unknown-unknown` Rust target.
- `wasm-bindgen-cli` for generating browser-loadable JavaScript bindings.

Build and run:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo build --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir web/pkg target/wasm32-unknown-unknown/debug/dreamstack.wasm
python3 -m http.server --directory web 8000
```

Then open `http://localhost:8000`.

Browser autosaves are stored in `localStorage` under the `autosave.json` key. Native autosaves remain file-based.

## macOS

Dreamstack is supported as a native macOS desktop app through `eframe`/`egui`.

Requirements:

- Rust `1.96.0`, as pinned by `rust-toolchain.toml`.
- Xcode Command Line Tools.

Build and run:

```sh
xcode-select --install
cargo run
```

Autosaves are written to `autosave.json` in the current working directory. Release artifacts are unsigned command-line app binaries, so macOS Gatekeeper may require explicit approval before first launch.

## License

Dreamstack is licensed under the [BSD-3-Clause](https://opensource.org/license/bsd-3-clause/) license.

For more information, see the [LICENSE](LICENSE) file.
