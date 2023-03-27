use hyouga_chess::{
    uci::{UciArguments, UciService},
    EngineSettings,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let engine_settings = Arc::new(RwLock::new(EngineSettings::default()));

    let uci_args = UciArguments {
        engine_settings: engine_settings.clone(),
    };

    let mut _uci_thread = UciService::new(uci_args);
    //Main Loop
    loop {}
}
