use educe::Educe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Educe)]
#[educe(Default)]
pub struct Config {
    pub enabled: bool,
    pub api_key: String,
    pub api_secret: String,
    pub access_token_key: String,
    pub access_token_secret: String,
    pub add_hash_tag: bool,
    #[educe(Default(
        expression = r#"vec!["twsend","TWSEND"].into_iter().map(Into::into).collect()"#
    ))]
    pub allowed_recepients: Vec<String>,
}
