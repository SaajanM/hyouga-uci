use std::collections::HashMap;

pub mod uci;
pub const ENGINE_NAME: &str = "Hyōga";
pub const ENGINE_AUTHOR: &str = "MythicalEngineer";

pub struct EngineSettings {
    pub debug: bool,
    pub options: HashMap<String, String>,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            debug: false,
            options: Default::default(),
        }
    }
}
