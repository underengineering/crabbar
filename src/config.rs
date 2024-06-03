use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Margins {
    pub left: Option<i32>,
    pub right: Option<i32>,
    pub top: Option<i32>,
    pub bottom: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub margins: Option<Margins>,
    pub image_path: Option<PathBuf>,
    pub network_name: String,
    pub battery_name: Option<String>,
    pub layout_map: Option<HashMap<String, String>>,
}
