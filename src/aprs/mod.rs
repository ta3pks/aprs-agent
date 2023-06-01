use std::time::Duration;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::TcpStream,
    time::sleep,
};

use crate::{extension_server::ConStore, extensions};

pub async fn start_server(config: crate::Config, tcp_ext_store: ConStore) {
    loop {
        let mut con = TcpStream::connect(format!("{}:{}", config.server, config.port))
            .await
            .expect("failed to connect to aprs server");
        let passcode: i64 = callpass::Callpass::from(config.callsign.as_str()).into();
        con.write_all(
            format!(
                "user {} pass {} vers APRS-AGENT 0.1 filter b/{}\n",
                config.callsign,
                passcode,
                config.allowed_callsigns.join("/")
            )
            .as_bytes(),
        )
        .await
        .expect("failed to write to aprs server");
        let (r, mut w) = con.split();
        let reader = tokio::io::BufReader::new(r);
        let mut lines = reader.lines();
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(1);
        extensions::ExtensionRegistry::set_own_writers(tx);
        loop {
            tokio::select! {
                line = lines.next_line() => {
                    let line = line.unwrap_or_default().unwrap_or_default();
                    if line.is_empty(){
                        break;
                    }
                if extensions::ExtensionRegistry::broadcast(&line, &mut w).await.is_err(){
                    break;
                }
                tcp_ext_store.broadcast(line);
                }
                line = rx.recv() => {
                    if let Some(mut line) = line {
                        if line.is_empty(){
                            continue;
                        }
                        if line.last() != Some(&b'\n'){
                            line.push(b'\n');
                        }
                        eprintln!("--> {}", String::from_utf8_lossy(&line));
                        if let Err(e) = w.write_all(&line).await {
                            eprintln!("failed to write to aprs server: {}", e);
                            break
                        };
                    }
                }
            }
        }
        eprintln!("disconnected from server, reconnecting in 1s");
        sleep(Duration::from_secs(1)).await;
    }
}
