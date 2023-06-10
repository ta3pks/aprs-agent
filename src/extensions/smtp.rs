use educe::Educe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Educe)]
#[educe(Default)]
pub struct Config {
    pub enabled: bool,
    #[educe(Default = "smtp.example.com:25")]
    pub smtp_server: String,
    #[educe(Default = "smtp@example.com")]
    pub smtp_username: String,
    #[educe(Default = "smtp_password")]
    pub smtp_password: String,
}
