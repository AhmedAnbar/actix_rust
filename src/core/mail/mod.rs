use lettre::{
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    Message, SmtpTransport, Transport,
};
use tokio::task;

use crate::config::CONFIG;

pub mod email_queue;

pub async fn send_email(
    to: String,
    subject: String,
    body: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let smtp_server = CONFIG.clone().smtp.server;
    let smtp_username = CONFIG.clone().smtp.username;
    let smtp_password = CONFIG.clone().smtp.password;
    let smtp_port = CONFIG.smtp.port.parse().unwrap_or(1025);
    let smtp_from = CONFIG.clone().smtp.from;
    let smtp_encryption = CONFIG.clone().smtp.encryption.to_lowercase();

    let email = Message::builder()
        .from(smtp_from.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .body(body)?;

    let creds = Credentials::new(smtp_username, smtp_password);

    let mailer = match smtp_encryption.as_str() {
        "starttls" => SmtpTransport::starttls_relay(&smtp_server)?
            .credentials(creds)
            .port(smtp_port)
            .build(),
        "tls" => {
            let tls_parameters = TlsParameters::new(smtp_server.clone())?;
            SmtpTransport::relay(&smtp_server)?
                .credentials(creds)
                .tls(Tls::Required(tls_parameters))
                .port(smtp_port)
                .build()
        }
        _ => SmtpTransport::builder_dangerous(&smtp_server)
            .credentials(creds)
            .port(smtp_port)
            .build(),
    };

    let _email_task = task::spawn_blocking(move || mailer.send(&email)).await??;

    Ok(())
}
