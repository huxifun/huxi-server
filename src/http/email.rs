//! 发送电子邮件

use crate::config::WebConfig;
use lettre::{
    message, transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport,
    Message, Tokio1Executor,
};

pub async fn send_email(
    config: &WebConfig,
    to: &str,
    subject: &str,
    body: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let smtp_credentials = Credentials::new(
        config.email.stmp_user.clone(),
        config.email.stmp_password.clone(),
    );

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(config.email.stmp_server.as_str())?
        .credentials(smtp_credentials)
        .build();

    let from = config.email.stmp_from.clone();

    send_email_smtp(&mailer, from.as_str(), to, subject, body).await
}

async fn send_email_smtp(
    mailer: &AsyncSmtpTransport<Tokio1Executor>,
    from: &str,
    to: &str,
    subject: &str,
    body: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let email = Message::builder()
        .from(from.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .header(message::header::ContentType::TEXT_HTML)
        .body(body.to_string())?;

    mailer.send(email).await?;

    Ok(())
}
