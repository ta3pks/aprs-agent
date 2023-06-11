use aprs_parser::AprsData;
use educe::Educe;
use lettre::{transport::smtp::authentication::Credentials, Transport};
use serde::{Deserialize, Serialize};
use tap::{Pipe, TapOptional};

use super::Extension;

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
    #[educe(Default(expression = r#"vec!["N0CALL"].iter().map(ToString::to_string).collect()"#))]
    pub allowed_senders: Vec<String>,
    #[educe(Default(expression = r#"vec!["EMAIL"].iter().map(ToString::to_string).collect()"#))]
    pub allowed_recipients: Vec<String>,
    pub allowed_receiver_emails: Vec<String>,
    #[educe(Default = "https://github.com/ta3pks/aprs-agent <aprs@nodomain.com>")]
    pub from_email: String,
}

pub struct SmtpEmailer(());
impl SmtpEmailer {
    pub fn new(cfg: &Config) -> Self {
        if cfg.allowed_senders.is_empty() || cfg.allowed_recipients.is_empty() {
            panic!("smtp extension requires at least one allowed sender and recipient");
        }
        Self(())
    }
}
#[async_trait::async_trait]
impl Extension for SmtpEmailer {
    fn name(&self) -> &'static str {
        "smtp"
    }

    async fn handle(&self, line: &str) -> Option<Vec<u8>> {
        let package = aprs_parser::AprsPacket::decode_textual(line.as_bytes()).ok()?;
        if package.data.data_type_identifier() != b':' {
            return None;
        }
        let to = package.to()?.clone();
        let sender = package.from;
        let cfg = &crate::Config::get().extensions.smtp;
        if !cfg
            .allowed_senders
            .iter()
            .any(|s| s.to_uppercase() == sender.call().to_string().to_uppercase())
        {
            return None;
        }
        let msg = if let AprsData::Message(d) = package.data {
            d
        } else {
            return None;
        };
        let receiver = msg
            .addressee
            .pipe(|d| String::from_utf8_lossy(&d).to_uppercase());
        if !cfg
            .allowed_recipients
            .iter()
            .any(|s| s.to_uppercase() == receiver)
        {
            return None;
        }
        if !cfg
            .allowed_recipients
            .iter()
            .any(|s| s.to_uppercase() == receiver)
        {
            return None;
        }
        let body = String::from_utf8_lossy(&msg.text);
        let (receiver_email, content) = body.split_once(' ')?;
        if !cfg.allowed_receiver_emails.is_empty()
            && !cfg
                .allowed_receiver_emails
                .iter()
                .any(|s| s.to_uppercase() == receiver_email.to_uppercase())
        {
            self.error(&format!("receiver email {receiver_email} not allowed"));
            return None;
        }
        let message = lettre::Message::builder()
            .from(
                cfg.from_email
                    .parse()
                    .map_err(|e| {
                        self.error(&format!("invalid from email: {}", e));
                    })
                    .ok()?,
            )
            .to(receiver_email
                .parse()
                .map_err(|e| {
                    self.error(&format!("invalid receiver email: {}", e));
                })
                .ok()?)
            .date_now()
            .subject("This email was sent using https://github.com/ta3pks/aprs-agent")
            .body(content.to_string())
            .map_err(|e| {
                self.error(&format!("failed to build email: {}", e));
            })
            .ok()?;
        let credentials = Credentials::new(cfg.smtp_username.clone(), cfg.smtp_password.clone());
        let (host, port) = cfg.smtp_server.split_once(':').tap_none(|| {
            self.error(&format!(
                "invalid smtp server: {} format must be host:port",
                cfg.smtp_server
            ));
        })?;
        let mailer = lettre::SmtpTransport::relay(host)
            .map_err(|e| {
                self.error(&format!("failed to connect to smtp server: {}", e));
            })
            .ok()?
            .port(
                port.parse()
                    .map_err(|e| {
                        self.error(&format!("invalid smtp port: {}", e));
                    })
                    .ok()?,
            )
            .credentials(credentials)
            .build();
        mailer
            .send(&message)
            .map_err(|e| {
                self.error(&format!("failed to send email: {}", e));
            })
            .ok()?;
        let msg_id = String::from_utf8_lossy(&msg.id?).to_string();
        let path = package
            .via
            .iter()
            .map(|x| match x {
                aprs_parser::Via::Callsign(c, _) => c.to_string(),
                aprs_parser::Via::QConstruct(q) => q.as_textual().to_string(),
            })
            .collect::<Vec<_>>()
            .join(",");
        let sender = sender.to_string();
        Some(format!("{to}>{sender},{path}::{sender: <9}:ack{msg_id}\n",).into_bytes())
    }
}
