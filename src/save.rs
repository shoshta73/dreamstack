use std::io;

#[cfg(not(target_arch = "wasm32"))]
use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::player::SavedPlayer;

const SAVE_VERSION: u64 = 3;
const AUTOSAVE_PATH: &str = "autosave.json";
#[cfg(not(target_arch = "wasm32"))]
const AUTOSAVE_TMP_PATH: &str = "autosave.json.tmp";

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct SaveFile {
    meta: SaveMeta,
    pub(crate) state: SaveState,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct SaveMeta {
    version: u64,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct SaveState {
    pub(crate) player: SavedPlayer,
    pub(crate) active_tutorial: u8,
    pub(crate) tutorial: SavedTutorial,
    pub(crate) app: SavedApp,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct SavedTutorial {
    pub(crate) elapsed_seconds: u64,
    pub(crate) active_job: Option<SavedActiveJob>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct SavedApp {
    pub(crate) screen: SavedScreen,
    pub(crate) terminal_input: String,
    pub(crate) terminal_lines: Vec<String>,
    pub(crate) terminal_scanned: bool,
    pub(crate) terminal_server_scanned: bool,
    pub(crate) connected_server: Option<String>,
    pub(crate) root_server: Option<String>,
    pub(crate) backdoor_server: Option<String>,
    pub(crate) hack_execution: Option<SavedHackExecution>,
    pub(crate) editor_unlocked: bool,
    pub(crate) editor_text: String,
    pub(crate) editor_output: Vec<String>,
    pub(crate) company_reputation_rate_favor: f64,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SavedScreen {
    Intro,
    Working,
    TerminalPrompt,
    Terminal,
    Editor,
    Complete,
    Finished,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct SavedHackExecution {
    pub(crate) hostname: String,
    pub(crate) elapsed_seconds: u64,
    pub(crate) duration_seconds: u64,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct SavedActiveJob {
    employer_name: String,
    job_name: String,
}

impl SavedActiveJob {
    pub(crate) fn new(employer_name: &str, job_name: &str) -> Self {
        Self {
            employer_name: employer_name.to_string(),
            job_name: job_name.to_string(),
        }
    }
}

impl SaveFile {
    pub(crate) fn version(&self) -> u64 {
        self.meta.version
    }

    pub(crate) fn new(
        player: SavedPlayer,
        active_tutorial: u8,
        elapsed_seconds: u64,
        active_job: Option<SavedActiveJob>,
        app: SavedApp,
    ) -> Self {
        Self {
            meta: SaveMeta {
                version: SAVE_VERSION,
            },
            state: SaveState {
                player,
                active_tutorial,
                tutorial: SavedTutorial {
                    elapsed_seconds,
                    active_job,
                },
                app,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct VersionProbe {
    meta: SaveMeta,
}

#[derive(Debug, Deserialize)]
struct V1SaveFile {
    state: V1SaveState,
}

#[derive(Debug, Deserialize)]
struct V1SaveState {
    player: SavedPlayer,
    active_tutorial: u8,
    tutorial: SavedTutorial,
}

#[derive(Debug, Deserialize)]
struct V2SaveFile {
    state: V2SaveState,
}

#[derive(Debug, Deserialize)]
struct V2SaveState {
    player: SavedPlayer,
    active_tutorial: u8,
    tutorial: SavedTutorial,
    app: V2SavedApp,
}

#[derive(Debug, Deserialize)]
struct V2SavedApp {
    screen: SavedScreen,
    terminal_input: String,
    terminal_lines: Vec<String>,
    terminal_scanned: bool,
    connected_server: Option<String>,
    root_server: Option<String>,
    backdoor_server: Option<String>,
    hack_execution: Option<SavedHackExecution>,
    editor_unlocked: bool,
    editor_text: String,
    editor_output: Vec<String>,
    company_reputation_rate_favor: f64,
}

impl V1SaveFile {
    fn migrate(self) -> SaveFile {
        SaveFile {
            meta: SaveMeta { version: 1 },
            state: SaveState {
                player: self.state.player,
                active_tutorial: self.state.active_tutorial,
                tutorial: self.state.tutorial,
                app: SavedApp::v1_defaults(),
            },
        }
    }
}

impl SavedApp {
    fn v1_defaults() -> Self {
        Self {
            screen: SavedScreen::Intro,
            terminal_input: String::new(),
            terminal_lines: Vec::new(),
            terminal_scanned: false,
            terminal_server_scanned: false,
            connected_server: None,
            root_server: None,
            backdoor_server: None,
            hack_execution: None,
            editor_unlocked: false,
            editor_text: crate::default_editor_text(),
            editor_output: Vec::new(),
            company_reputation_rate_favor: 0.0,
        }
    }
}

impl V2SaveFile {
    fn migrate(self) -> SaveFile {
        let app = self.state.app;
        SaveFile {
            meta: SaveMeta { version: 2 },
            state: SaveState {
                player: self.state.player,
                active_tutorial: self.state.active_tutorial,
                tutorial: self.state.tutorial,
                app: SavedApp {
                    screen: app.screen,
                    terminal_input: app.terminal_input,
                    terminal_lines: app.terminal_lines,
                    terminal_scanned: app.terminal_scanned,
                    terminal_server_scanned: false,
                    connected_server: app.connected_server,
                    root_server: app.root_server,
                    backdoor_server: app.backdoor_server,
                    hack_execution: app.hack_execution,
                    editor_unlocked: app.editor_unlocked,
                    editor_text: app.editor_text,
                    editor_output: app.editor_output,
                    company_reputation_rate_favor: app.company_reputation_rate_favor,
                },
            },
        }
    }
}

pub(crate) fn write_autosave(save_file: &SaveFile) -> io::Result<()> {
    #[cfg(target_arch = "wasm32")]
    {
        write_browser_autosave(save_file)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        write_save_file(save_file, AUTOSAVE_PATH, AUTOSAVE_TMP_PATH)
    }
}

pub(crate) fn read_autosave() -> io::Result<Option<SaveFile>> {
    #[cfg(target_arch = "wasm32")]
    {
        read_browser_autosave()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        read_save_file(AUTOSAVE_PATH)
    }
}

pub(crate) fn parse_save_file(json: &str) -> io::Result<SaveFile> {
    let version = serde_json::from_str::<VersionProbe>(json)
        .map_err(io::Error::other)?
        .meta
        .version;

    match version {
        1 => serde_json::from_str::<V1SaveFile>(json)
            .map(V1SaveFile::migrate)
            .map_err(io::Error::other),
        2 => serde_json::from_str::<V2SaveFile>(json)
            .map(V2SaveFile::migrate)
            .map_err(io::Error::other),
        SAVE_VERSION => serde_json::from_str(json).map_err(io::Error::other),
        version => Err(io::Error::other(format!(
            "unsupported save version {version}"
        ))),
    }
}

#[cfg(target_arch = "wasm32")]
fn write_browser_autosave(save_file: &SaveFile) -> io::Result<()> {
    let json = serde_json::to_string_pretty(save_file).map_err(io::Error::other)?;
    let window = web_sys::window().ok_or_else(|| io::Error::other("window is unavailable"))?;
    let storage = window
        .local_storage()
        .map_err(|error| io::Error::other(format!("localStorage error: {error:?}")))?
        .ok_or_else(|| io::Error::other("localStorage is unavailable"))?;

    storage
        .set_item(AUTOSAVE_PATH, &json)
        .map_err(|error| io::Error::other(format!("localStorage error: {error:?}")))
}

#[cfg(target_arch = "wasm32")]
fn read_browser_autosave() -> io::Result<Option<SaveFile>> {
    let window = web_sys::window().ok_or_else(|| io::Error::other("window is unavailable"))?;
    let storage = window
        .local_storage()
        .map_err(|error| io::Error::other(format!("localStorage error: {error:?}")))?
        .ok_or_else(|| io::Error::other("localStorage is unavailable"))?;

    storage
        .get_item(AUTOSAVE_PATH)
        .map_err(|error| io::Error::other(format!("localStorage error: {error:?}")))?
        .map(|json| parse_save_file(&json))
        .transpose()
}

#[cfg(not(target_arch = "wasm32"))]
fn write_save_file(
    save_file: &SaveFile,
    path: impl AsRef<Path>,
    tmp_path: impl AsRef<Path>,
) -> io::Result<()> {
    let json = serde_json::to_string_pretty(save_file).map_err(io::Error::other)?;

    fs::write(&tmp_path, json)?;
    fs::rename(tmp_path, path)
}

#[cfg(not(target_arch = "wasm32"))]
fn read_save_file(path: impl AsRef<Path>) -> io::Result<Option<SaveFile>> {
    match fs::read_to_string(path) {
        Ok(json) => parse_save_file(&json).map(Some),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_arch = "wasm32"))]
    use std::fs;

    use crate::player::{SavedCompanyStanding, SavedPlayer};

    use super::{
        SaveFile, SavedActiveJob, SavedApp, SavedHackExecution, SavedScreen, parse_save_file,
        read_save_file,
    };

    #[test]
    fn serializes_expected_save_format() {
        let save_file = SaveFile::new(
            SavedPlayer {
                money: 880.0,
                charisma_experience: 576.0,
                charisma_skill: 1,
                hack_experience: 25.0,
                hack_skill: 1,
                exchange_marker: -1.0,
                company_standings: vec![SavedCompanyStanding {
                    company_name: "employer0".to_string(),
                    reputation: 28.8,
                }],
            },
            0,
            28_800,
            Some(SavedActiveJob::new("employer0", "employee")),
            SavedApp {
                screen: SavedScreen::Terminal,
                terminal_input: "scan".to_string(),
                terminal_lines: vec!["connected to home server: dreamstack".to_string()],
                terminal_scanned: true,
                terminal_server_scanned: true,
                connected_server: Some("server0".to_string()),
                root_server: Some("server0".to_string()),
                backdoor_server: Some("server0".to_string()),
                hack_execution: Some(SavedHackExecution {
                    hostname: "server0".to_string(),
                    elapsed_seconds: 60,
                    duration_seconds: 7_200,
                }),
                editor_unlocked: true,
                editor_text: "fn main(ds) {}".to_string(),
                editor_output: vec!["ok".to_string()],
                company_reputation_rate_favor: 0.05,
            },
        );

        insta::assert_snapshot!(serde_json::to_string_pretty(&save_file).unwrap(), @r#"
        {
          "meta": {
            "version": 3
          },
          "state": {
            "player": {
              "money": 880.0,
              "charisma_experience": 576.0,
              "charisma_skill": 1,
              "hack_experience": 25.0,
              "hack_skill": 1,
              "botanical_gardens": -1.0,
              "company_standings": [
                {
                  "company_name": "employer0",
                  "reputation": 28.8
                }
              ]
            },
            "active_tutorial": 0,
            "tutorial": {
              "elapsed_seconds": 28800,
              "active_job": {
                "employer_name": "employer0",
                "job_name": "employee"
              }
            },
            "app": {
              "screen": "terminal",
              "terminal_input": "scan",
              "terminal_lines": [
                "connected to home server: dreamstack"
              ],
              "terminal_scanned": true,
              "terminal_server_scanned": true,
              "connected_server": "server0",
              "root_server": "server0",
              "backdoor_server": "server0",
              "hack_execution": {
                "hostname": "server0",
                "elapsed_seconds": 60,
                "duration_seconds": 7200
              },
              "editor_unlocked": true,
              "editor_text": "fn main(ds) {}",
              "editor_output": [
                "ok"
              ],
              "company_reputation_rate_favor": 0.05
            }
          }
        }
        "#);
    }

    #[test]
    fn migrates_v1_save_format() {
        let save_file = parse_save_file(
            r#"
            {
              "meta": { "version": 1 },
              "state": {
                "player": {
                  "money": 880.0,
                  "charisma_experience": 576.0,
                  "charisma_skill": 1,
                  "hack_experience": 25.0,
                  "hack_skill": 1,
                  "botanical_gardens": -1.0,
                  "company_standings": []
                },
                "active_tutorial": 1,
                "tutorial": {
                  "elapsed_seconds": 28800,
                  "active_job": {
                    "employer_name": "employer0",
                    "job_name": "employee"
                  }
                }
              }
            }
            "#,
        )
        .unwrap();

        assert_eq!(save_file.state.active_tutorial, 1);
        assert_eq!(save_file.state.tutorial.elapsed_seconds, 28_800);
        assert_eq!(save_file.state.app.screen, SavedScreen::Intro);
        assert!(!save_file.state.app.terminal_server_scanned);
        assert_eq!(
            save_file.state.app.editor_text,
            crate::default_editor_text()
        );
    }

    #[test]
    fn migrates_v2_save_format() {
        let save_file = parse_save_file(
            r#"
            {
              "meta": { "version": 2 },
              "state": {
                "player": {
                  "money": 880.0,
                  "charisma_experience": 576.0,
                  "charisma_skill": 1,
                  "hack_experience": 25.0,
                  "hack_skill": 1,
                  "botanical_gardens": -1.0,
                  "company_standings": []
                },
                "active_tutorial": 1,
                "tutorial": {
                  "elapsed_seconds": 28800,
                  "active_job": {
                    "employer_name": "employer0",
                    "job_name": "employee"
                  }
                },
                "app": {
                  "screen": "terminal",
                  "terminal_input": "scan",
                  "terminal_lines": ["connected"],
                  "terminal_scanned": true,
                  "connected_server": "server0",
                  "root_server": null,
                  "backdoor_server": null,
                  "hack_execution": null,
                  "editor_unlocked": false,
                  "editor_text": "fn main(ds) {}",
                  "editor_output": [],
                  "company_reputation_rate_favor": 0.0
                }
              }
            }
            "#,
        )
        .unwrap();

        assert_eq!(save_file.state.active_tutorial, 1);
        assert_eq!(save_file.state.app.terminal_input, "scan");
        assert!(save_file.state.app.terminal_scanned);
        assert!(!save_file.state.app.terminal_server_scanned);
        assert_eq!(
            save_file.state.app.connected_server.as_deref(),
            Some("server0")
        );
    }

    #[test]
    fn rejects_unsupported_save_version() {
        let error = parse_save_file(r#"{"meta":{"version":99},"state":{}}"#).unwrap_err();

        assert!(error.to_string().contains("unsupported save version 99"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn missing_save_file_returns_none() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "dreamstack-missing-save-{}.json",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);

        assert!(read_save_file(&path).unwrap().is_none());
    }
}
