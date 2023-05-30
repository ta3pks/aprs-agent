use std::time::Duration;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::TcpStream,
    time::sleep,
};

use crate::extension_server::ConStore;

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
        let (r, _w) = con.split();
        let reader = tokio::io::BufReader::new(r);
        let mut lines = reader.lines();
        while let Ok(line) = lines.next_line().await {
            let line = if let Some(line) = line {
                line
            } else {
                //empty line reconnect
                break;
            };
            eprintln!("{}", line);
            tcp_ext_store.broadcast(line);
        }
        eprintln!("disconnected from server, reconnecting in 1s");
        sleep(Duration::from_secs(1)).await;
    }
}
