use async_trait::async_trait;
use tokio::io::{AsyncWrite, AsyncWriteExt};
pub mod fixed_beacon;
pub mod logger;
pub mod smtp;
pub mod twitter;

#[async_trait]
pub trait Extension {
    fn name(&self) -> &'static str;
    async fn handle(&self, line: &str) -> Option<Vec<u8>>;
    /// if an extension is spawnable it will be spawned in a new tokio task
    /// this is useful for extensions that for sure will not return something to the aprs server
    /// or that has an own writer
    fn is_spawnable(&self) -> bool {
        false
    }
    /// set own writer is used for extensions that need to write data back to the aprs server without getting a message first
    /// this is used for example by an extension that sends fixed position packets every x minutes
    fn set_own_writer(&self, _: tokio::sync::mpsc::Sender<Vec<u8>>) {}
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
    pub async fn broadcast(
        line: &str,
        mut w: impl AsyncWrite + Unpin,
    ) -> Result<(), std::io::Error> {
        unsafe {
            if let Some(ref exts) = EXTENSIONS {
                for ext in exts {
                    if ext.is_spawnable() {
                        let line = line.to_owned();
                        tokio::spawn(async move {
                            ext.handle(&line).await;
                        });
                    } else if let Some(mut res) = ext.handle(line).await {
                        eprintln!(
                            "extension {} writing to aprs server:\n{}\n-----",
                            ext.name(),
                            String::from_utf8_lossy(&res)
                        );
                        if res.is_empty() {
                            continue;
                        }
                        match res.last() {
                            Some(b'\n') => {}
                            Some(_) => {
                                res.push(b'\n');
                            }
                            None => {
                                continue;
                            }
                        }
                        if let Err(e) = w.write_all(&res).await {
                            eprintln!("failed to write to aprs server: {}", e);
                            return Err(e);
                        };
                    }
                }
            }
        }
        Ok(())
    }
    pub fn set_own_writers(w: tokio::sync::mpsc::Sender<Vec<u8>>) {
        unsafe {
            if let Some(ref exts) = EXTENSIONS {
                for ext in exts {
                    ext.set_own_writer(w.clone());
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
    async fn handle(&self, line: &str) -> Option<Vec<u8>> {
        (*self).handle(line).await
    }
}
