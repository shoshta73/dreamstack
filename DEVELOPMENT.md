# Developer guide

## Requirements

- [rustup](https://rustup.rs) — the toolchain (Rust **1.96.0**) is pinned in `rust-toolchain.toml` and will be installed automatically.

## Commands

```sh
cargo run              # debug
cargo build            # debug (no run)
cargo run --release    # release
cargo build --release  # release (no run)
cargo build --workspace
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo insta review     # review pending snapshot changes
cargo insta accept     # accept pending snapshot changes
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

## Dev Release Tags

Dev release tags use one of these formats:

```text
YYwWW[build-id]
vX.Y.Z-YYwWW[build-id]
```

The versioned form is used when a `vX.Y.Z` tag exists. Otherwise, dev releases use the unversioned weekly form.

Before pushing a version, commit the changelog update with this exact message:

```sh
git commit -m "chore: update changelog"
```

Then create an empty commit whose message exactly matches the tag that will be pushed:

```sh
git commit --allow-empty -m "<tag>"
```

```sh
scripts/next-dev-tag         # print the next dev-release tag
scripts/next-dev-tag create  # create the next dev-release tag locally
scripts/next-dev-tag push    # create and push the next dev-release tag
```

## Release Notes

The dev-release workflow uses the first `##` section in `CHANGELOG.md` as the release notes body.

## Game Data

`data/employers.json` is embedded into the binary at compile time. Rebuild the app after changing it.

Autosaves write `autosave.json` through `autosave.json.tmp` in the repo root. Both files are ignored by git.

## Commit Style

Prefer small atomic commits that each capture one focused change.

## Contributions

Use the GitHub issue and pull request templates when reporting bugs, proposing features, or opening changes. Blank issues are disabled so reports include the context needed to reproduce or review changes.

Use the development task template for internal engineering work such as documentation, tooling, CI, platform support, refactors, and maintenance. Prefer labels such as `documentation`, `tooling`, `platform`, and `development` to classify the task.
