use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub network_name: String,
    pub battery_name: String,
    pub layout_map: HashMap<String, String>,
}
