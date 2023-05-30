mod config;
mod error;
mod extension_server;
mod flags;
mod utils;
pub use error::{Err, Result};
#[tokio::main]
async fn main() {
    flags::parse();
    eprintln!("{:?}", flags::flags());
    println!("Hello, world!");
}
