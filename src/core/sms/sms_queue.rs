use crate::core::sms::send_sms;
use log::{error, info};
use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct SmsJob {
    pub to: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct SmsQueue {
    pub sender: Sender<SmsJob>,
}

impl SmsQueue {
    pub fn new() -> (Self, Receiver<SmsJob>) {
        let (sender, receiver) = mpsc::channel(100); // Buffer size of 100
        (Self { sender }, receiver)
    }

    pub async fn process_queue(mut receiver: Receiver<SmsJob>) {
        info!("Starting SMS queue processing");

        while let Some(job) = receiver.recv().await {
            info!("Processing SMS job for: {}", job.to);

            match send_sms(&job.to, &job.body).await {
                Ok(_) => info!("SMS message sent successfully to: {}", job.to),
                Err(err) => error!("Failed to send SMS message to {}: {}", job.to, err),
            }
        }

        info!("SMS queue processing stopped");
    }
}
