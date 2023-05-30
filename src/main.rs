mod config;
mod error;
mod extension_server;
mod flags;
mod utils;
pub use error::{Err, Result};
#[tokio::main]
async fn main() {
    let flags = flags::parse();
    if flags.write_default_config {
        config::write_default_config();
        eprintln!("default config written to {}", flags.config);
        return;
    }
    config::parse();
    println!("Hello, world!");
}
