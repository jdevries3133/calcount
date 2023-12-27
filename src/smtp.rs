use anyhow::Result;

#[cfg(enable_smtp_email)]
/// Send a simple plain text email to a single recipient.
pub async fn send_email(to: &str, subject: &str, msg: &str) -> Result<()> {
    // Build a simple multipart message
    let message = MessageBuilder::new()
        .from(("Jack DeVries (beancount.bot)", "jdevries3133@gmail.com"))
        .to(to)
        .subject(subject)
        .html_body(format!("<html><head></head><body>{}</body></html>", msg))
        .text_body(msg);

    let username = env::var("SMTP_EMAIL_USERNAME")?;
    let password = env::var("SMTP_EMAIL_PASSWORD")?;

    SmtpClientBuilder::new("smtp.gmail.com", 587)
        .implicit_tls(false)
        .credentials((&username[..], &password[..]))
        .connect()
        .await?
        .send(message)
        .await?;

    Ok(())
}

#[cfg(not(enable_smtp_email))]
pub async fn send_email(to: &str, subject: &str, msg: &str) -> Result<()> {
    println!("Would send email:\n\tTo: {to}\n\tSubject: {subject}\n\tBody:\n{msg}\n===\n");

    Ok(())
}
