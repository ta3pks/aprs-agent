use std::fmt::{self, Formatter};

use aprs_parser::AprsData;
use educe::Educe;
use serde::{Deserialize, Serialize};
use tap::Pipe;
fn fmt_pass(v: &String, f: &mut Formatter) -> fmt::Result {
    let fst = v.chars().take(3).collect::<String>();
    let lst = if v.len() > 3 {
        v.chars().skip(v.len() - 3).collect::<String>()
    } else {
        String::new()
    };
    f.write_str(&format!("{fst}xxxx{lst}"))
}

#[derive(Serialize, Deserialize, Clone, Educe)]
#[educe(Default, Debug)]
pub struct Config {
    pub enabled: bool,
    #[educe(Debug(method = "fmt_pass"))]
    pub api_key: String,
    #[educe(Debug(method = "fmt_pass"))]
    pub api_secret: String,
    #[educe(Debug(method = "fmt_pass"))]
    pub access_token_key: String,
    #[educe(Debug(method = "fmt_pass"))]
    pub access_token_secret: String,
    #[educe(Default = true)]
    pub add_hash_tag: bool,
    #[educe(Default(
        expression = r#"vec!["twsend","TWSEND"].into_iter().map(Into::into).collect()"#
    ))]
    pub allowed_recepients: Vec<String>,
    #[educe(Default(expression = r#"vec!["TA3PKS"].into_iter().map(Into::into).collect()"#))]
    pub allowed_senders: Vec<String>,
}
pub struct Twitter;

#[async_trait::async_trait]
impl super::Extension for Twitter {
    fn name(&self) -> &'static str {
        "twitter"
    }
    async fn handle(&self, line: &str) -> Option<String> {
        let cfg = &crate::Config::get().extensions.twitter;
        if !cfg.enabled {
            return None;
        }
        let package = aprs_parser::AprsPacket::decode_textual(line.as_bytes()).ok()?;
        if package.data.data_type_identifier() != b':' {
            return None;
        }
        if cfg.allowed_recepients.is_empty() || cfg.allowed_senders.is_empty() {
            panic!("Twitter extension enabled but no allowed recepients or senders specified");
        }

        let ssid = &package.from;
        let sender_callsign = ssid.call().to_string();
        if !cfg
            .allowed_senders
            .iter()
            .any(|x| x.to_uppercase() == sender_callsign.to_uppercase())
        {
            return None;
        }
        let path = package
            .via
            .iter()
            .map(|x| match x {
                aprs_parser::Via::Callsign(c, _) => c.to_string(),
                aprs_parser::Via::QConstruct(q) => q.as_textual().to_string(),
            })
            .collect::<Vec<_>>()
            .join(",");
        let to = package.to().map(ToString::to_string).unwrap_or_default();
        let msg = if let AprsData::Message(m) = package.data {
            m
        } else {
            return None;
        };
        let recepient = msg
            .addressee
            .as_slice()
            .pipe(String::from_utf8_lossy)
            .to_string();
        dbg!(&line);
        dbg!(&path, &to, &ssid, &recepient);
        if !cfg.allowed_recepients.contains(&recepient) {
            return None;
        }
        let msg_id = msg.id.unwrap_or_default().pipe(|id| {
            if id.is_empty() {
                String::new()
            } else {
                String::from_utf8_lossy(&id).to_string()
            }
        });
        self.error(&msg_id);
        let msg = String::from_utf8_lossy(&msg.text);
        let _tweet = format!("{msg}\nfrom {ssid}>{to},{path}");
        if msg.is_empty() {
            None
        } else {
            Some(format!("{recepient}>{ssid},{path}::{ssid:<9}:ack{msg_id}"))
        }
    }
}
