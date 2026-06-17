use std::io;

#[cfg(not(target_arch = "wasm32"))]
use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::player::SavedPlayer;

const SAVE_VERSION: u64 = 1;
const AUTOSAVE_PATH: &str = "autosave.json";
#[cfg(not(target_arch = "wasm32"))]
const AUTOSAVE_TMP_PATH: &str = "autosave.json.tmp";

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct SaveFile {
    meta: SaveMeta,
    state: SaveState,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct SaveMeta {
    version: u64,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct SaveState {
    player: SavedPlayer,
    active_level: u8,
    level: SavedLevel,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct SavedLevel {
    elapsed_seconds: u64,
    active_job: Option<SavedActiveJob>,
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
    pub(crate) fn new(
        player: SavedPlayer,
        active_level: u8,
        elapsed_seconds: u64,
        active_job: Option<SavedActiveJob>,
    ) -> Self {
        Self {
            meta: SaveMeta {
                version: SAVE_VERSION,
            },
            state: SaveState {
                player,
                active_level,
                level: SavedLevel {
                    elapsed_seconds,
                    active_job,
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

#[cfg(test)]
mod tests {
    use crate::player::{SavedCompanyStanding, SavedPlayer};

    use super::{SaveFile, SavedActiveJob};

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
        );

        insta::assert_snapshot!(serde_json::to_string_pretty(&save_file).unwrap(), @r#"
        {
          "meta": {
            "version": 1
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
            "active_level": 0,
            "level": {
              "elapsed_seconds": 28800,
              "active_job": {
                "employer_name": "employer0",
                "job_name": "employee"
              }
            }
          }
        }
        "#);
    }
}
