//! Google Gemini Provider Adapter
//!
//! Google Gemini API 的 Provider 适配器实现。
//! 支持 Gemini 1.5 和 Gemini 2.0 系列模型。
//!
//! ## 支持的模型
//! - gemini-2.0-flash-exp
//! - gemini-1.5-flash
//! - gemini-1.5-flash-8b
//! - gemini-1.5-pro
//! - gemini-1.0-pro
//! - gemini-pro
//!
//! ## API 端点
//! 默认: https://generativelanguage.googleapis.com/v1beta
//!
//! ## 认证方式
//! 使用 query parameter `key={api_key}` (而非 Bearer Token)

pub mod adapter;
pub mod protocol;
pub mod stream;

pub use adapter::GeminiProvider;
pub use protocol::{
    GeminiContent, GeminiPart, GeminiRequest, GeminiResponse, GeminiStreamResponse,
    GenerationConfig, UsageMetadata,
};
pub use stream::parse_gemini_stream;

#[cfg(test)]
mod tests {
    use super::*;
    use keycompute_provider_trait::ProviderAdapter;

    #[test]
    fn test_gemini_provider_exports() {
        let provider = adapter::GeminiProvider::new();
        assert_eq!(provider.name(), "gemini");
        assert!(!protocol::GEMINI_MODELS.is_empty());
    }
}
