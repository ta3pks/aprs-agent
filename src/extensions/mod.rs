use async_trait::async_trait;
use tokio::io::{AsyncWrite, AsyncWriteExt};
pub mod logger;
pub mod twitter;

#[async_trait]
pub trait Extension {
    fn name(&self) -> &'static str;
    async fn handle(&self, line: &str) -> Option<String>;
    fn is_spawnable(&self) -> bool {
        false
    }
    fn log(&self, msg: &str) {
        eprintln!("\x1B[32m{}:\x1B[0m {}", self.name(), msg);
    }
    fn error(&self, msg: &str) {
        eprintln!("\x1B[31m{}:\x1B[0m {}", self.name(), msg);
    }
    fn warn(&self, msg: &str) {
        eprintln!("\x1B[33m{}:\x1B[0m {}", self.name(), msg);
    }
}

static mut EXTENSIONS: Option<Vec<Box<dyn Extension + Send + Sync>>> = None;
pub struct ExtensionRegistry;
impl ExtensionRegistry {
    pub fn register(ext: impl Extension + 'static + Send + Sync) {
        unsafe {
            if let Some(ref mut exts) = EXTENSIONS {
                exts.push(Box::new(ext));
            } else {
                EXTENSIONS = Some(vec![Box::new(ext)]);
            }
        }
    }
    //later extensions might write data back to the aprs server
    pub async fn handle(line: &str, mut w: impl AsyncWrite + Unpin) {
        unsafe {
            if let Some(ref exts) = EXTENSIONS {
                for ext in exts {
                    if ext.is_spawnable() {
                        let line = line.to_owned();
                        tokio::spawn(async move {
                            ext.handle(&line).await;
                        });
                    } else if let Some(res) = ext.handle(line).await {
                        eprintln!(
                            "extension {} writing to aprs server:\n{}\n-----",
                            ext.name(),
                            res
                        );
                        if let Err(e) = w.write_all(format!("{}\n", res).as_bytes()).await {
                            eprintln!("failed to write to aprs server: {}", e);
                        };
                    }
                }
            }
        }
    }
}

#[async_trait]
impl<T: Extension + std::marker::Sync> Extension for &T {
    fn name(&self) -> &'static str {
        (*self).name()
    }
    async fn handle(&self, line: &str) -> Option<String> {
        (*self).handle(line).await
    }
}
