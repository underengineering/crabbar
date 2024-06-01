use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub image_path: Option<PathBuf>,
    pub network_name: String,
    pub battery_name: Option<String>,
    pub layout_map: Option<HashMap<String, String>>,
}
