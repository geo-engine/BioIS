use crate::config::{SmtpConfig, SmtpTls};
use anyhow::Context;
use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    transport::smtp::{authentication::Credentials, extension::ClientId},
};

/// Sends emails. Abstracts over the concrete transport to allow testing without an SMTP server.
#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send(&self, message: Message) -> anyhow::Result<()>;
}

#[async_trait]
impl<T> Mailer for T
where
    T: AsyncTransport + Send + Sync,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    async fn send(&self, message: Message) -> anyhow::Result<()> {
        AsyncTransport::send(self, message)
            .await
            .context("failed to send email")?;
        Ok(())
    }
}

/// Creates an SMTP transport from the config. Connections are established lazily on first send.
pub fn smtp_transport(config: &SmtpConfig) -> anyhow::Result<AsyncSmtpTransport<Tokio1Executor>> {
    let mut builder = match config.tls {
        SmtpTls::Starttls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
            .context("failed to configure SMTP STARTTLS relay")?,
        SmtpTls::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
            .context("failed to configure SMTP TLS relay")?,
        SmtpTls::None => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host),
    }
    .port(config.port);

    if let Some(helo_name) = &config.helo_name {
        builder = builder.hello_name(ClientId::Domain(helo_name.clone()));
    }

    match (&config.username, &config.password) {
        (Some(username), Some(password)) => {
            builder = builder.credentials(Credentials::new(
                username.clone(),
                password.expose().to_string(),
            ));
        }
        (None, None) => {}
        _ => anyhow::bail!("SMTP username and password must be set together"),
    }

    Ok(builder.build())
}
