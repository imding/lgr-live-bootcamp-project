use {
    crate::domain::{email::Email, email_client::EmailClient},
    color_eyre::eyre::Result,
    resend_rs::{Resend as ResendClient, types::CreateEmailBaseOptions},
    secrecy::{ExposeSecret, SecretBox},
    tracing::instrument,
};

pub struct Resend {
    client: ResendClient,
    sender: Email,
}

impl Resend {
    pub fn new(sender: Email, token: &SecretBox<String>) -> Self {
        Self { client: ResendClient::new(token.expose_secret()), sender }
    }
}

#[async_trait::async_trait]
impl EmailClient for Resend {
    #[instrument(name = "Send email", skip_all)]
    async fn send_email(&self, receipent: &Email, subject: &str, content: &str) -> Result<()> {
        let email = CreateEmailBaseOptions::new(
            self.sender.as_ref().expose_secret(),
            [receipent.as_ref().expose_secret()],
            subject,
        )
        .with_text(content);

        match self.client.emails.send(email).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
