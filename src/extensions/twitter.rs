use std::fmt::{self, Formatter};

use aprs_parser::AprsData;
use educe::Educe;
use serde::{Deserialize, Serialize};
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
        if cfg.allowed_recepients.is_empty() {
            panic!("Twitter extension enabled but no allowed recepients specified");
        }
        if !cfg
            .allowed_recepients
            .contains(&package.to().map(ToString::to_string).unwrap_or_default())
        {
            return None;
        }
        self.log(&format!("Sending tweet: {}", line));
        let ssid = package.from.to_string();
        let path = package
            .via
            .iter()
            .map(|x| match x {
                aprs_parser::Via::Callsign(c, _) => c.to_string(),
                aprs_parser::Via::QConstruct(q) => q.as_textual().to_string(),
            })
            .collect::<Vec<_>>()
            .join(",");
        let recepient = package.to().map(ToString::to_string).unwrap_or_default();
        let msg = if let AprsData::Message(m) = package.data {
            m
        } else {
            return None;
        };
        let msg = String::from_utf8_lossy(&msg.text);
        let tweet = format!("{msg}\nfrom {ssid}>{path}");
        self.warn(&format!("Sending tweet: {tweet} {recepient}"));
        None
    }
}
