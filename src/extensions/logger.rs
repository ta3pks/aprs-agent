use educe::Educe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Educe)]
#[educe(Default)]
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
        let _cfg = &crate::Config::get().extensions.logger;
        if line.starts_with('#') && _cfg.log_comments {
            self.log(line);
        }
        None
    }
}
