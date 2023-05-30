use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server: String,
    pub port: u16,
    pub callsign: String,
    pub passcode: String,
    pub allowed_callsigns: Vec<String>,
    pub extension_server: ExtensionServerSettings,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtensionServerSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}
