# Changelog

All notable changes to Dreamstack will be documented in this file.

## Unreleased

### UI

- Made the player stats sidebar collapsible and removed the level field from it.

### Internal

- Added Windows and macOS build coverage, release artifacts, and platform setup notes.
- Added web build support with a browser entry point, localStorage autosaves, build coverage, and setup notes.
- Added dev-release web deployment under `/dev`.

## 26w25a [2026-06-15]

### UI

- Restyled the terminal as a console pane with improved visual hierarchy, prompt styling, and command history presentation.

### Internal

- Added GitHub issue and pull request templates.
- Updated the README to describe the current Level 0 job loop, Level 1 hacking onboarding, terminal commands, and sidebars.
- Documented development release workflow and issue task labels.
- Scoped Rust CI and build workflows to source, data, workflow, and project configuration changes.

## 26w24a [2026-06-12]

### Gameplay

- Initial native `eframe`/`egui` incremental game prototype.
- Level 0 job flow with an employer, job offer, pay rate, and 8-hour in-game shift.
- Level 1 hacking onboarding after the first work shift.
- Embedded server data with hack skill requirements, security, max money, and hack experience rewards.
- Player hack experience, hack skill, charisma skill, and starting money stats.
- Live work progress with accelerated in-game time and visible player stats.
- Job rewards for money, company reputation, and charisma experience over worked time.
- Level skip action that applies the remaining shift rewards before completing the level.
- End-of-level choice to convert company reputation into future favor or clear company standings.
- Company favor now increases future company reputation gain rate by 0.5% per favor.
- Company favor applies only to subsequent levels, not the level where it is earned.
- Money, skill experience, and company reputation reset when moving between levels.
- New players start with `1024.0` money.

### UI

- Collapsible left sidebar with a Hacking group and Terminal entry.
- Right player stats sidebar.

### Terminal

- Home server terminal flow with `netscan`, `connect <hostname>`, `scan`, `nuke`, `npm i -g backdoor`, `hack`, and `home` commands.
- Terminal hacking flow that rewards hack experience after a successful `hack` command.

### Persistence

- Autosave support.

### Internal

- CI, build, and dev-release automation.
