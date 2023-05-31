use educe::Educe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Educe)]
#[educe(Default)]
#[serde(default)]
pub struct Config {
    #[educe(Default = true)]
    pub enabled: bool,
    #[educe(Default = true)]
    pub log_comments: bool,
    #[educe(Default(
        expression = r#"vec!['!','/','\\','@','~','`','^','&','*','(',')','_','-','=','+','[',']','{','}','|',';',':','"','<','>','?','.']"#
    ))]
    pub filter_by_message_type: Vec<char>,
    pub exclude_by_message_type: Vec<char>,
    pub keyword_filter: Vec<String>,
}

pub struct Logger;

#[async_trait::async_trait]
impl super::Extension for Logger {
    fn name(&self) -> &'static str {
        "logger"
    }
    fn is_spawnable(&self) -> bool {
        true
    }
    async fn handle(&self, line: &str) -> Option<String> {
        let cfg = &crate::Config::get().extensions.logger;
        if line.starts_with('#') && cfg.log_comments {
            self.log(line);
            return None;
        }
        let msg = match aprs_parser::AprsPacket::decode_textual(line.as_bytes()) {
            Ok(msg) => msg,
            Err(e) => {
                self.error(&format!("failed to parse aprs packet: {e}\n{line}"));
                return None;
            }
        };
        if !cfg.keyword_filter.is_empty() {
            if cfg
                .keyword_filter
                .iter()
                .any(|k| line.to_lowercase().contains(&k.to_lowercase()))
            {
                self.log(line);
            }
            return None;
        }
        if cfg.filter_by_message_type.is_empty()
            || cfg
                .filter_by_message_type
                .contains(&(msg.data.data_type_identifier() as char))
        {
            if cfg
                .exclude_by_message_type
                .contains(&(msg.data.data_type_identifier() as char))
            {
                if !cfg.filter_by_message_type.is_empty() {
                    self.warn(
                        &format!(
                            "both exclude filter and filter_by_message_type config parameters contain the same char '{}'",
                            msg.data.data_type_identifier() as char)
                    )
                }
                return None;
            }
            self.log(line);
        }

        None
    }
}
