use twilio::{Client, OutboundMessage};

use crate::config::CONFIG;

pub mod sms_queue;

pub async fn send_sms(to: &str, body: &str) -> Result<String, String> {
    if CONFIG.sms.enable {
        let client = Client::new(&CONFIG.sms.account, &CONFIG.sms.token);
        let message = OutboundMessage::new(&CONFIG.sms.from, to, body);

        match client.send_message(message).await {
            Ok(_) => {
                log::info!("SMS sent to {}", to);
                return Ok("SMS message sent successfully".to_string());
            }
            Err(err) => {
                log::error!("Failed to send SMS to {}: {}", to, err);
                return Err("Failed to send SMS message: ".to_string() + &err.to_string());
            }
        }
    }

    Ok("No SMS data found".to_string())
}
