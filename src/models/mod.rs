use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadLinks {
    pub version: String,
    pub content: HashMap<String, DownloadOption>,
    pub bases: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadOption {
    pub size: u64,
    pub version: String,
    pub deprecated: bool,
    #[serde(rename = "downloadLink")]
    pub download_link: String,
    #[serde(rename = "installerMd5")]
    pub installer_md5: String,
}
