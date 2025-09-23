use {super::email::Email, color_eyre::eyre::Result};

#[async_trait::async_trait]
pub trait EmailClient: Send + Sync {
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> Result<()>;
}
