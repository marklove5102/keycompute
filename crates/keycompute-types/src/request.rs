use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{PricingSnapshot, UsageAccumulator};

/// 请求上下文：贯穿全链路的唯一状态载体
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub api_key_id: Uuid,
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    pub pricing_snapshot: PricingSnapshot, // 请求开始时固化
    pub usage: UsageAccumulator,           // streaming 中累积
    pub started_at: DateTime<Utc>,
}

impl RequestContext {
    pub fn new(
        user_id: Uuid,
        tenant_id: Uuid,
        api_key_id: Uuid,
        model: impl Into<String>,
        messages: Vec<Message>,
        stream: bool,
        pricing_snapshot: PricingSnapshot,
    ) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            user_id,
            tenant_id,
            api_key_id,
            model: model.into(),
            messages,
            stream,
            pricing_snapshot,
            usage: UsageAccumulator::new(),
            started_at: Utc::now(),
        }
    }

    /// 获取请求持续时间
    pub fn duration(&self) -> chrono::Duration {
        Utc::now() - self.started_at
    }

    /// 获取当前用量快照
    pub fn usage_snapshot(&self) -> (u32, u32) {
        self.usage.snapshot()
    }
}

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new("system", content)
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new("user", content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new("assistant", content)
    }
}

/// OpenAI 兼容的请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

impl ChatCompletionRequest {
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            stream: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stop: None,
        }
    }
}
