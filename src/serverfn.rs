use crate::server;
use dioxus::prelude::*;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone)]
pub struct File {
    pub name: String,
}

#[server]
pub async fn get_files() -> Result<Vec<File>, ServerFnError> {
    Ok(std::fs::read_dir("testfiles")?
        .filter_map(|res| res.ok())
        .map(|entry| File {
            name: entry.file_name().to_string_lossy().to_string(),
        })
        .collect())
}
