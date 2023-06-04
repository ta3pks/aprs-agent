use std::{error::Error, sync::Arc, time::Duration};

use educe::Educe;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use super::Extension;

#[derive(Debug, Serialize, Deserialize, Educe, Clone)]
#[educe(Default)]
#[serde(default)]
pub struct Config {
    pub enabled: bool,
    #[educe(Default = "N0CALL-10")]
    pub ssid: String,
    #[educe(Default = "3800.00N")]
    pub lat: String,
    #[educe(Default = "02700.00E")]
    pub lon: String,
    #[educe(Default = '/')]
    pub symbol_table: char,
    #[educe(Default = '-')]
    pub symbol: char,
    #[educe(Default = "https://github.com/ta3pks/aprs-agent")]
    pub comment: String,
    #[educe(Default = 15)]
    pub beacon_interval_mins: u64,
}

#[derive(Clone)]
pub struct FixedBeacon(Arc<Mutex<FixedBeaconInner>>);
impl FixedBeacon {
    pub fn new(cfg: &Config) -> Self {
        if !cfg.enabled {
            panic!("fixed beacon is not enabled but tried to be created");
        }
        if cfg.ssid.len() > 9 {
            panic!("ssid cannot be longer than 9 characters");
        }
        if cfg.lat.len() > 8 {
            panic!("lat cannot be longer than 8 characters");
        }
        if cfg.lon.len() > 9 {
            panic!("lon cannot be longer than 9 characters");
        }
        if !&['N', 'S'].contains(&cfg.lat.chars().last().unwrap_or(0x00 as char)) {
            panic!("lat must end with N or S");
        }
        if !&['E', 'W'].contains(&cfg.lon.chars().last().unwrap_or(0x00 as char)) {
            panic!("lon must end with E or W");
        }
        let inner = FixedBeaconInner {
            own_writer: None,
            is_worker_running: false,
        };
        Self(Arc::new(Mutex::new(inner)))
    }
    fn start(&self) {
        let cfg = &crate::Config::get().extensions.fixed_beacon;
        let ext = self.clone();
        tokio::spawn(async move {
            ext.0.lock().is_worker_running = true;
            loop {
                if let Err(e) = ext.send().await {
                    ext.error(&format!("failed to send beacon: {}", e));
                }
                tokio::time::sleep(Duration::from_secs(60 * cfg.beacon_interval_mins)).await;
            }
        });
    }
    async fn send(&self) -> Result<(), Box<dyn Error>> {
        let writer = {
            if let Some(writer) = self.0.lock().own_writer.clone() {
                writer
            } else {
                return Ok(());
            }
        };
        //TA3PKS-7>APAT81-15,WIDE1-1,WIDE2-2,qAR,TA3ML-1:!3800.00N/02700.00E>/A=000000ta3pks@mugsoft.io +905418622094 433.500 QRV
        let cfg = &crate::Config::get().extensions.fixed_beacon;
        let package = format!(
            "{ssid}>AP4GNT,TCPID*,qAC,APRSAGENT:!{lat}{symbol_table}{lon}{symbol}{comment}\n",
            ssid = cfg.ssid.to_uppercase(),
            lat = cfg.lat,
            symbol_table = cfg.symbol_table,
            lon = cfg.lon,
            symbol = cfg.symbol,
            comment = cfg.comment
        );
        writer.send(package.into_bytes()).await?;
        Ok(())
    }
}

#[derive(Default)]
struct FixedBeaconInner {
    own_writer: Option<tokio::sync::mpsc::Sender<Vec<u8>>>,
    is_worker_running: bool,
}

#[async_trait::async_trait]
impl Extension for FixedBeacon {
    fn name(&self) -> &'static str {
        "fixed_beacon"
    }
    async fn handle(&self, _: &str) -> Option<Vec<u8>> {
        None
    }
    fn set_own_writer(&self, w: tokio::sync::mpsc::Sender<Vec<u8>>) {
        let mut inner = self.0.lock();
        inner.own_writer = Some(w);
        if !inner.is_worker_running {
            self.start();
        }
    }
}
