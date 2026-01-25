//! Email service for sending verification and password reset emails.

use lettre::{
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tracing::info;

/// Email configuration from environment variables
#[derive(Clone)]
pub struct EmailConfig {
    pub enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_email: String,
    pub base_url: String,
}

impl EmailConfig {
    /// Load email configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            enabled: std::env::var("EMAIL_ENABLED")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            smtp_host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            smtp_port: std::env::var("SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(587),
            smtp_user: std::env::var("SMTP_USER").unwrap_or_default(),
            smtp_pass: std::env::var("SMTP_PASS").unwrap_or_default(),
            from_email: std::env::var("FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@holidaywheel.com".to_string()),
            base_url: std::env::var("BASE_URL")
                .unwrap_or_else(|_| "http://localhost:5000".to_string()),
        }
    }
}

/// Email service for sending emails
#[derive(Clone)]
pub struct EmailService {
    config: EmailConfig,
}

impl EmailService {
    /// Create a new email service
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }

    /// Create from environment variables
    pub fn from_env() -> Self {
        Self::new(EmailConfig::from_env())
    }

    /// Check if email service is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Send an email
    async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), String> {
        if !self.config.enabled {
            // Dev mode - just log the email
            info!("[DEV EMAIL] To: {}", to_email);
            info!("[DEV EMAIL] Subject: {}", subject);
            info!("[DEV EMAIL] Body: {}", text_body);
            return Ok(());
        }

        let email = Message::builder()
            .from(
                self.config
                    .from_email
                    .parse()
                    .map_err(|e| format!("Invalid from email: {}", e))?,
            )
            .to(to_email
                .parse()
                .map_err(|e| format!("Invalid to email: {}", e))?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text_body.to_string()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_body.to_string()),
                    ),
            )
            .map_err(|e| format!("Failed to build email: {}", e))?;

        let creds = Credentials::new(self.config.smtp_user.clone(), self.config.smtp_pass.clone());

        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.config.smtp_host)
                .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
                .credentials(creds)
                .port(self.config.smtp_port)
                .build();

        mailer
            .send(email)
            .await
            .map_err(|e| format!("Failed to send email: {}", e))?;

        info!("Email sent to {}", to_email);
        Ok(())
    }

    /// Send email verification link
    pub async fn send_verification_email(&self, email: &str, token: &str) -> Result<(), String> {
        let verify_url = format!("{}/auth/verify/{}", self.config.base_url, token);

        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: Arial, sans-serif; background: #f5f5f5; padding: 20px; }}
        .container {{ max-width: 600px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; }}
        h1 {{ color: #2b5cff; }}
        .button {{ display: inline-block; background: #2b5cff; color: white; padding: 12px 24px;
                   text-decoration: none; border-radius: 6px; margin: 20px 0; }}
        .url {{ word-break: break-all; color: #666; font-size: 12px; }}
        .footer {{ margin-top: 30px; color: #999; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Welcome to Holiday Wheel of Fortune!</h1>
        <p>Please verify your email address to complete your registration.</p>
        <p><a href="{verify_url}" class="button">Verify Email</a></p>
        <p class="url">Or copy this URL: {verify_url}</p>
        <p class="footer">This link expires in 24 hours.<br>
        If you didn't create an account, you can ignore this email.</p>
    </div>
</body>
</html>"#,
            verify_url = verify_url
        );

        let text = format!(
            r#"Welcome to Holiday Wheel of Fortune!

Please verify your email by visiting:
{verify_url}

This link expires in 24 hours.
If you didn't create an account, you can ignore this email."#,
            verify_url = verify_url
        );

        self.send_email(email, "Verify your Holiday Wheel account", &html, &text)
            .await
    }

    /// Send password reset link
    pub async fn send_password_reset_email(&self, email: &str, token: &str) -> Result<(), String> {
        let reset_url = format!("{}/auth/reset-password/{}", self.config.base_url, token);

        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: Arial, sans-serif; background: #f5f5f5; padding: 20px; }}
        .container {{ max-width: 600px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; }}
        h1 {{ color: #2b5cff; }}
        .button {{ display: inline-block; background: #2b5cff; color: white; padding: 12px 24px;
                   text-decoration: none; border-radius: 6px; margin: 20px 0; }}
        .url {{ word-break: break-all; color: #666; font-size: 12px; }}
        .footer {{ margin-top: 30px; color: #999; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Password Reset Request</h1>
        <p>Click the button below to reset your password:</p>
        <p><a href="{reset_url}" class="button">Reset Password</a></p>
        <p class="url">Or copy this URL: {reset_url}</p>
        <p class="footer">This link expires in 1 hour.<br>
        If you didn't request a password reset, you can ignore this email.</p>
    </div>
</body>
</html>"#,
            reset_url = reset_url
        );

        let text = format!(
            r#"Password Reset Request

Reset your password by visiting:
{reset_url}

This link expires in 1 hour.
If you didn't request a password reset, you can ignore this email."#,
            reset_url = reset_url
        );

        self.send_email(email, "Reset your Holiday Wheel password", &html, &text)
            .await
    }
}
