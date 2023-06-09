use serde::{Deserialize, Serialize};

use crate::{
    extensions::{fixed_beacon, logger, smtp, twitter, ExtensionRegistry},
    flags::{flags, Flags},
};
#[macro_export]
macro_rules! switch {
    ($($rule:expr => $do:expr);+) => {
        $(if $rule { $do } )+
    };
}
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
        expression = r#"vec!["ta*","tb*","tc*","ym*"].iter().map(ToString::to_string).collect()"#
    ))]
    pub allowed_callsigns: Vec<String>,
    pub print_config_on_startup: bool,
    pub extension_server: ExtensionServerSettings,
    pub extensions: Extensions,
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
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Extensions {
    pub twitter: twitter::Config,
    pub logger: logger::Config,
    pub fixed_beacon: fixed_beacon::Config,
    pub smtp: smtp::Config,
}
static mut CONFIG: Option<Config> = None;
impl Config {
    pub fn get() -> &'static Config {
        unsafe { CONFIG.as_ref().expect("config not initialized") }
    }
    pub fn parse(flags: &Flags) -> Config {
        if unsafe { CONFIG.is_some() } {
            panic!("config already initialized");
        }
        let cpath = &flags.config;
        let config = match std::fs::read_to_string(cpath) {
            Ok(contents) => toml::from_str(&contents).expect("failed to parse config file"),
            Err(e) => {
                eprintln!("failed to read config file creating default config: {}", e);
                let config = Config::default();
                config.sync_file();
                config
            }
        };
        unsafe {
            CONFIG = Some(config.clone());
        }
        config
    }
    pub fn register_extensions(&self) {
        switch! {
            self.extensions.twitter.enabled => ExtensionRegistry::register(twitter::Twitter::new(&self.extensions.twitter));
            self.extensions.logger.enabled => ExtensionRegistry::register(logger::Logger);
            self.extensions.smtp.enabled => ExtensionRegistry::register(smtp::SmtpEmailer::new(&self.extensions.smtp));
            self.extensions.fixed_beacon.enabled => ExtensionRegistry::register(fixed_beacon::FixedBeacon::new(&self.extensions.fixed_beacon))
        }
    }
    pub fn sync_file(&self) {
        let cpath = &flags().config;
        let contents = toml::to_string_pretty(self).expect("failed to serialize config");
        std::fs::write(cpath, contents).expect("failed to write config file");
    }
}

pub fn write_default_config() {
    let cpath = &flags().config;
    let config = Config::default();
    let contents = toml::to_string_pretty(&config).expect("failed to serialize config");
    std::fs::write(cpath, contents).expect("failed to write config file");
}
