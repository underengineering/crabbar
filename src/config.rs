use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub network_name: String,
    pub battery_name: Option<String>,
    pub layout_map: Option<HashMap<String, String>>,
}
