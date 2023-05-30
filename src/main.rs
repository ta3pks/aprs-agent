mod aprs;
mod config;
mod error;
mod extension_server;
mod flags;
mod utils;

pub use config::Config;
pub use error::{Err, Result};
#[tokio::main]
async fn main() {
    let flags = flags::parse();
    if flags.write_default_config {
        config::write_default_config();
        eprintln!("default config written to {}", flags.config);
        return;
    }
    let config = config::parse(&flags);
    if flags.print_config {
        eprintln!("{:#?}", config);
        return;
    }
    if config.print_config_on_startup {
        eprintln!("{:#?}", config);
    }
    let _ext_con_store = if config.extension_server.enabled {
        extension_server::start(config.clone())
    } else {
        Default::default()
    };
    aprs::start_server(config).await;
}
