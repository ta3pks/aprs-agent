use serde::{Deserialize, Serialize};

use crate::flags::{flags, Flags};
use educe::Educe;

#[derive(Debug, Serialize, Deserialize, Educe, Clone)]
#[educe(Default)]
#[serde(default)]
pub struct Config {
    #[educe(Default = "euro.aprs2.net")]
    pub server: String,
    #[educe(Default = 14580)]
    pub port: u16,
    #[educe(Default = "N0CALL")]
    pub callsign: String,
    #[educe(Default(
        expression = r#"vec!["ta*","tb*","ym*"].iter().map(ToString::to_string).collect()"#
    ))]
    pub allowed_callsigns: Vec<String>,
    #[educe(Default = true)]
    pub print_config_on_startup: bool,
    pub extension_server: ExtensionServerSettings,
}
#[derive(Debug, Serialize, Deserialize, Educe, Clone)]
#[educe(Default)]
#[serde(default)]
pub struct ExtensionServerSettings {
    pub enabled: bool,
    #[educe(Default = "127.0.0.1")]
    pub host: String,
    #[educe(Default = 65080)]
    pub port: u16,
}

static mut CONFIG: Option<Config> = None;

pub fn parse(flags: &Flags) -> Config {
    let cpath = &flags.config;
    let contents = std::fs::read_to_string(cpath).expect("failed to read config file");
    let config: Config = toml::from_str(&contents).expect("failed to parse config file");
    unsafe {
        CONFIG = Some(config.clone());
    }
    config
}
pub fn write_default_config() {
    let cpath = &flags().config;
    let config = Config::default();
    let contents = toml::to_string_pretty(&config).expect("failed to serialize config");
    std::fs::write(cpath, contents).expect("failed to write config file");
}
