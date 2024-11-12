use tokio::sync::mpsc::{self, Receiver, Sender};

use super::send_email;

#[derive(Debug)]
pub struct EmailJob {
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct EmailQueue {
    pub sender: Sender<EmailJob>,
}

impl EmailQueue {
    pub fn new() -> (Self, Receiver<EmailJob>) {
        let (sender, receiver) = mpsc::channel(100); // Buffer size of 100
        (Self { sender }, receiver)
    }

    pub async fn process_queue(mut receiver: Receiver<EmailJob>) {
        while let Some(job) = receiver.recv().await {
            if let Err(e) = send_email(job.to, job.subject, job.body).await {
                eprintln!("Failed to send email: {:?}", e);
            }
        }
    }
}
