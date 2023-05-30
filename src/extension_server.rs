use std::{
    collections::HashMap, fmt::Display, net::SocketAddr, str::FromStr, sync::Arc, time::Duration,
};

use parking_lot::RwLock;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::UnboundedSender,
};

use crate::{config::Config, utils::now_unix};

#[derive(Debug, Default, Clone)]
pub struct ConStore {
    store: Arc<RwLock<HashMap<String, UnboundedSender<String>>>>,
}
impl ConStore {
    fn add(&self, addr: SocketAddr, sock: UnboundedSender<String>) {
        let mut store = self.store.write();
        store.insert(addr.to_string(), sock);
    }
    fn remove(&self, addr: SocketAddr) {
        self.store.write().remove(&addr.to_string());
    }
    pub fn broadcast(&self, msg: String) {
        let store = self.store.read();
        for (_, sock) in store.iter() {
            sock.send(msg.clone()).ok();
        }
    }
}

pub fn start(cfg: Config) -> ConStore {
    let (host, port) = (cfg.extension_server.host, cfg.extension_server.port);
    let store = ConStore::default();
    eprintln!("Starting extension server on {host}:{port}");
    let _store = store.clone();
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind((host, port)).await.unwrap();
        loop {
            let (socket, addr) = listener.accept().await.unwrap();
            tokio::spawn(handler(socket, addr, _store.clone()));
        }
    });
    store
}

macro_rules! get_cmd {
    ($line:expr,$addr:expr) => {{
        let line = if let Ok(line) = $line {
            line
        } else {
            break;
        };
        let line = if let Some(line) = line {
            if line.is_empty() {
                eprintln!("empty line from {}", $addr);
                break;
            }
            line
        } else {
            eprintln!("no line from {}", $addr);
            break;
        };
        if let Ok(cmd) = line.parse::<ClientCmd>() {
            cmd
        } else {
            eprintln!("invalid command from {}: {}", $addr, line);
            break;
        }
    }};
}
async fn handler(mut sock: TcpStream, addr: SocketAddr, store: ConStore) {
    eprintln!("New connection from {addr} to devserver");
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    store.add(addr, tx);
    let (r, mut w) = sock.split();
    let buf_reader = tokio::io::BufReader::new(r);
    let mut lines = buf_reader.lines();
    loop {
        tokio::select! {
            line = lines.next_line() => {
                let cmd = get_cmd!(line, addr);
                //so far only ping is handled so this if is fine
                if cmd == ClientCmd::Ping && w.write_all(format!("{}\n", ServerCmd::Pong).as_bytes()).await.is_err() {
                    break;
                }
            },
            msg = rx.recv() => {
                if let Some(msg) = msg {
                    if w.write_all(format!("{}\n", ServerCmd::Data(msg)).as_bytes()).await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            },
            _ = tokio::time::sleep(Duration::from_secs(10)) =>{
                eprintln!("extension_server client {addr} timed out");
                break;
            }
        }
    }
    store.remove(addr);
    eprintln!("{addr} disconnected");
}

#[derive(Debug, PartialEq)]
pub enum ClientCmd {
    Ping,
}

impl FromStr for ClientCmd {
    type Err = crate::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ping" => Ok(ClientCmd::Ping),
            _ => Err(crate::error::ExtServerErrors::InvalidCmd(s.to_string()).into()),
        }
    }
}

enum ServerCmd {
    Pong,
    Data(String),
}
impl Display for ServerCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerCmd::Pong => write!(f, "pong {}", now_unix()),
            ServerCmd::Data(data) => write!(f, "data {}", data),
        }
    }
}
