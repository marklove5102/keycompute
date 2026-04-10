//! 邮件服务模块
//!
//! 提供 SMTP 邮件发送功能：
//! - 邮箱验证邮件
//! - 密码重置邮件
//! - 通用邮件发送
//!
//! # 配置
//!
//! 通过 `keycompute-config` 模块加载配置：
//! - 环境变量：`KC__EMAIL__SMTP_HOST`、`KC__EMAIL__SMTP_PORT` 等
//! - 配置文件：`config.toml` 中的 `[email]` 部分
//!
//! # 热更新支持
//!
//! 支持运行时配置更新：
//! ```rust,ignore
//! email_service.update_config(new_config).await;
//! ```

// 重新导出配置类型，方便调用方使用
pub use keycompute_config::EmailConfig;

use keycompute_types::KeyComputeError;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// 邮件发送错误
#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    /// 配置错误
    #[error("邮件服务未配置")]
    NotConfigured,

    /// 邮箱地址格式错误
    #[error("无效的邮箱地址: {0}")]
    InvalidAddress(String),

    /// 邮件构建错误
    #[error("邮件构建失败: {0}")]
    BuildError(String),

    /// SMTP 发送错误
    #[error("邮件发送失败: {0}")]
    SendError(String),
}

impl From<EmailError> for KeyComputeError {
    fn from(err: EmailError) -> Self {
        KeyComputeError::Internal(err.to_string())
    }
}

/// 邮件服务
#[derive(Clone)]
pub struct EmailService {
    config: Arc<RwLock<EmailConfig>>,
    transport: Arc<RwLock<Option<AsyncSmtpTransport<Tokio1Executor>>>>,
}

impl EmailService {
    /// 创建邮件服务实例
    pub fn new(config: EmailConfig) -> Self {
        let transport = Self::build_transport(&config);
        Self {
            config: Arc::new(RwLock::new(config)),
            transport: Arc::new(RwLock::new(transport)),
        }
    }

    /// 从 Arc<EmailConfig> 创建（克隆内部数据）
    ///
    /// 注意：此方法会克隆 EmailConfig 的内部数据，不会共享 Arc。
    /// 如需共享配置，请直接使用 new()。
    pub fn from_arc(config: Arc<EmailConfig>) -> Self {
        Self::new((*config).clone())
    }

    /// 构建 SMTP 传输
    fn build_transport(config: &EmailConfig) -> Option<AsyncSmtpTransport<Tokio1Executor>> {
        if !config.is_configured() {
            tracing::warn!("邮件服务未配置，邮件发送将被禁用");
            return None;
        }

        let creds = Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());

        // lettre 0.11 的 pool 配置在启用 pool feature 后自动生效
        // 使用默认连接池配置（最大 10 个连接）
        let transport = if config.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
                .ok()?
                .credentials(creds)
                .port(config.smtp_port)
                .timeout(Some(Duration::from_secs(config.timeout_secs)))
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_host)
                .credentials(creds)
                .port(config.smtp_port)
                .timeout(Some(Duration::from_secs(config.timeout_secs)))
                .build()
        };

        Some(transport)
    }

    /// 检查服务是否已配置
    pub async fn is_configured(&self) -> bool {
        self.transport.read().await.is_some()
    }

    /// 更新配置（支持热更新）
    ///
    /// 更新配置后会重新构建 SMTP 传输层。
    /// 先更新 transport，再更新 config，确保任何时刻至少有一个有效配置。
    pub async fn update_config(&self, config: EmailConfig) {
        let new_transport = Self::build_transport(&config);

        // 先更新 transport（使用新配置构建的 transport）
        let mut transport = self.transport.write().await;
        *transport = new_transport;
        drop(transport);

        // 再更新 config
        let mut cfg = self.config.write().await;
        *cfg = config;
        drop(cfg);

        tracing::info!("邮件服务配置已更新");
    }

    /// 获取当前配置的克隆
    pub async fn config(&self) -> EmailConfig {
        self.config.read().await.clone()
    }

    /// 发送邮箱验证邮件
    pub async fn send_verification_email(&self, to: &str, token: &str) -> Result<(), EmailError> {
        // 快速获取需要的配置字段，然后释放锁
        let verification_url = {
            let config = self.config.read().await;
            config.verification_url(token)
        };

        let subject = "请验证您的邮箱地址";
        let text_body = format!(
            r#"您好！

感谢您注册 KeyCompute。

请点击以下链接验证您的邮箱地址：
{}

此链接将在 24 小时后过期。

如果您没有注册 KeyCompute 账户，请忽略此邮件。

祝好，
KeyCompute 团队
"#,
            verification_url
        );

        let html_body = format!(
            r#"<html>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
<div style="max-width: 600px; margin: 0 auto; padding: 20px;">
<h2 style="color: #2c5282;">请验证您的邮箱地址</h2>
<p>您好！</p>
<p>感谢您注册 KeyCompute。</p>
<p>请点击以下按钮验证您的邮箱地址：</p>
<p>
<a href="{}" style="display: inline-block; padding: 12px 24px; background-color: #4299e1; color: white; text-decoration: none; border-radius: 4px;">
验证邮箱
</a>
</p>
<p>或复制以下链接到浏览器：<br><code style="word-break: break-all;">{}</code></p>
<p style="color: #718096; font-size: 14px;">此链接将在 24 小时后过期。</p>
<p style="color: #718096; font-size: 14px;">如果您没有注册 KeyCompute 账户，请忽略此邮件。</p>
<hr style="border: none; border-top: 1px solid #e2e8f0; margin: 20px 0;">
<p style="color: #718096; font-size: 12px;">KeyCompute 团队</p>
</div>
</body>
</html>"#,
            verification_url, verification_url
        );

        self.send_html_email(to, subject, &text_body, &html_body)
            .await
    }

    /// 发送密码重置邮件
    pub async fn send_password_reset_email(&self, to: &str, token: &str) -> Result<(), EmailError> {
        // 快速获取需要的配置字段，然后释放锁
        let reset_url = {
            let config = self.config.read().await;
            config.password_reset_url(token)
        };

        let subject = "重置您的密码";
        let text_body = format!(
            r#"您好！

我们收到了重置您密码的请求。

请点击以下链接重置密码：
{}

此链接将在 1 小时后过期。

如果您没有请求重置密码，请忽略此邮件，您的密码不会改变。

祝好，
KeyCompute 团队
"#,
            reset_url
        );

        let html_body = format!(
            r#"<html>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
<div style="max-width: 600px; margin: 0 auto; padding: 20px;">
<h2 style="color: #2c5282;">重置您的密码</h2>
<p>您好！</p>
<p>我们收到了重置您密码的请求。</p>
<p>请点击以下按钮重置密码：</p>
<p>
<a href="{}" style="display: inline-block; padding: 12px 24px; background-color: #e53e3e; color: white; text-decoration: none; border-radius: 4px;">
重置密码
</a>
</p>
<p>或复制以下链接到浏览器：<br><code style="word-break: break-all;">{}</code></p>
<p style="color: #718096; font-size: 14px;">此链接将在 1 小时后过期。</p>
<p style="color: #718096; font-size: 14px;">如果您没有请求重置密码，请忽略此邮件，您的密码不会改变。</p>
<hr style="border: none; border-top: 1px solid #e2e8f0; margin: 20px 0;">
<p style="color: #718096; font-size: 12px;">KeyCompute 团队</p>
</div>
</body>
</html>"#,
            reset_url, reset_url
        );

        self.send_html_email(to, subject, &text_body, &html_body)
            .await
    }

    /// 发送欢迎邮件（邮箱验证成功后）
    pub async fn send_welcome_email(&self, to: &str, name: Option<&str>) -> Result<(), EmailError> {
        // 先准备所有数据，不持有锁
        let greeting = name
            .map(|n| format!("{}！", n))
            .unwrap_or_else(|| "！".to_string());

        let subject = "欢迎加入 KeyCompute";
        let text_body = format!(
            r#"您好{}！

恭喜您成功验证了邮箱地址。

现在您可以开始使用 KeyCompute 的全部功能：
• 创建和管理 API Key
• 配置 LLM Provider
• 监控使用量和费用

如果您有任何问题，请随时联系我们的支持团队。

祝好，
KeyCompute 团队
"#,
            greeting
        );

        let html_body = format!(
            r#"<html>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
<div style="max-width: 600px; margin: 0 auto; padding: 20px;">
<h2 style="color: #2c5282;">欢迎加入 KeyCompute</h2>
<p>您好{}！</p>
<p>恭喜您成功验证了邮箱地址。</p>
<p>现在您可以开始使用 KeyCompute 的全部功能：</p>
<ul>
<li>创建和管理 API Key</li>
<li>配置 LLM Provider</li>
<li>监控使用量和费用</li>
</ul>
<p>如果您有任何问题，请随时联系我们的支持团队。</p>
<hr style="border: none; border-top: 1px solid #e2e8f0; margin: 20px 0;">
<p style="color: #718096; font-size: 12px;">KeyCompute 团队</p>
</div>
</body>
</html>"#,
            greeting
        );

        self.send_html_email(to, subject, &text_body, &html_body)
            .await
    }

    /// 发送纯文本邮件
    pub async fn send_text_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), EmailError> {
        // 先获取配置
        let config = self.config.read().await;
        let from_mailbox = Self::build_from_mailbox(&config)?;

        let to_mailbox: Mailbox = to
            .parse()
            .map_err(|_| EmailError::InvalidAddress(to.to_string()))?;

        // 构建邮件
        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .body(body.to_string())
            .map_err(|e| EmailError::BuildError(e.to_string()))?;

        drop(config);

        // 获取 transport 并发送
        let transport_guard = self.transport.read().await;
        let transport = transport_guard.as_ref().ok_or(EmailError::NotConfigured)?;

        transport
            .send(email)
            .await
            .map_err(|e| EmailError::SendError(e.to_string()))?;

        tracing::info!(
            to = %to,
            subject = %subject,
            "邮件发送成功"
        );

        Ok(())
    }

    /// 构建发件人邮箱地址
    fn build_from_mailbox(config: &EmailConfig) -> Result<Mailbox, EmailError> {
        let from_str = match &config.from_name {
            Some(name) => format!("{} <{}>", name, config.from_address),
            None => config.from_address.clone(),
        };

        from_str.parse().map_err(|_| {
            EmailError::BuildError(format!("Invalid from address: {}", config.from_address))
        })
    }

    /// 发送带 HTML 正文的多部分邮件
    pub async fn send_html_email(
        &self,
        to: &str,
        subject: &str,
        text_body: &str,
        html_body: &str,
    ) -> Result<(), EmailError> {
        // 先获取配置和检查 transport
        let config = self.config.read().await;
        let from_mailbox = Self::build_from_mailbox(&config)?;

        let to_mailbox: Mailbox = to
            .parse()
            .map_err(|_| EmailError::InvalidAddress(to.to_string()))?;

        // 构建邮件（不需要 transport）
        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text_body.to_string()),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_body.to_string()),
                    ),
            )
            .map_err(|e| EmailError::BuildError(e.to_string()))?;

        drop(config);

        // 获取 transport 并发送
        let transport_guard = self.transport.read().await;
        let transport = transport_guard.as_ref().ok_or(EmailError::NotConfigured)?;

        transport
            .send(email)
            .await
            .map_err(|e| EmailError::SendError(e.to_string()))?;

        tracing::info!(
            to = %to,
            subject = %subject,
            "HTML 邮件发送成功"
        );

        Ok(())
    }
}

impl std::fmt::Debug for EmailService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Debug 实现避免使用异步操作，防止阻塞和线程池问题
        // 只显示类型信息，不尝试获取锁
        f.debug_struct("EmailService")
            .field("type", &"EmailService")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> EmailConfig {
        EmailConfig {
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_username: "test@example.com".to_string(),
            smtp_password: "testpass".to_string(),
            from_address: "noreply@example.com".to_string(),
            from_name: Some("KeyCompute".to_string()),
            use_tls: true,
            verification_base_url: "https://api.example.com".to_string(),
            timeout_secs: 30,
        }
    }

    #[tokio::test]
    async fn test_email_service_creation() {
        let service = EmailService::new(test_config());
        assert!(service.is_configured().await);
    }

    #[tokio::test]
    async fn test_email_service_not_configured() {
        let service = EmailService::new(EmailConfig::default());
        assert!(!service.is_configured().await);
    }

    #[tokio::test]
    async fn test_invalid_email_address() {
        let service = EmailService::new(test_config());

        let result = service
            .send_text_email("invalid-email", "Test", "Body")
            .await;

        assert!(matches!(result, Err(EmailError::InvalidAddress(_))));
    }

    #[tokio::test]
    async fn test_send_without_config() {
        let service = EmailService::new(EmailConfig::default());

        let result = service
            .send_text_email("test@example.com", "Test", "Body")
            .await;

        assert!(matches!(result, Err(EmailError::NotConfigured)));
    }

    #[tokio::test]
    async fn test_config_update() {
        let service = EmailService::new(EmailConfig::default());
        assert!(!service.is_configured().await);

        // 更新配置
        let new_config = test_config();
        service.update_config(new_config).await;

        assert!(service.is_configured().await);
    }

    #[tokio::test]
    async fn test_from_name_usage() {
        let mut config = test_config();
        config.from_name = Some("Test Sender".to_string());

        let service = EmailService::new(config);
        let cfg = service.config().await;

        assert_eq!(cfg.from_name, Some("Test Sender".to_string()));
    }
}
