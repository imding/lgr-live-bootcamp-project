use {
    crate::domain::{email::Email, email_client::EmailClient},
    color_eyre::eyre::Result,
    tracing::info,
};

pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> Result<()> {
        info!("Sending email to {} with subject: {} and content: {}", recipient.as_ref(), subject, content);

        Ok(())
    }
}
