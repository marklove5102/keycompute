//! Chat Completions 处理器
//!
//! 核心 API：POST /v1/chat/completions

use crate::{
    error::Result,
    extractors::{AuthExtractor, RequestId},
    state::AppState,
};
use axum::{
    extract::State,
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::Stream;
use keycompute_types::{Message, RequestContext, UsageAccumulator};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::convert::Infallible;

/// Chat 请求
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    /// 模型名称
    pub model: String,
    /// 消息列表
    pub messages: Vec<Message>,
    /// 是否流式输出
    #[serde(default = "default_stream")]
    pub stream: bool,
    /// 最大 token 数
    pub max_tokens: Option<u32>,
    /// 温度参数
    pub temperature: Option<f32>,
    /// top_p 参数
    pub top_p: Option<f32>,
}

fn default_stream() -> bool {
    true
}

/// Chat 响应（非流式）
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    /// 响应 ID
    pub id: String,
    /// 对象类型
    pub object: String,
    /// 创建时间戳
    pub created: i64,
    /// 模型名称
    pub model: String,
    /// 选择列表
    pub choices: Vec<Choice>,
    /// 用量信息
    pub usage: UsageInfo,
}

/// 选择项
#[derive(Debug, Serialize)]
pub struct Choice {
    /// 索引
    pub index: u32,
    /// 消息
    pub message: Message,
    /// 结束原因
    pub finish_reason: Option<String>,
}

/// 用量信息
#[derive(Debug, Serialize)]
pub struct UsageInfo {
    /// 输入 token 数
    pub prompt_tokens: u32,
    /// 输出 token 数
    pub completion_tokens: u32,
    /// 总 token 数
    pub total_tokens: u32,
}

/// SSE 流式响应
#[derive(Debug, Serialize)]
pub struct StreamChunk {
    /// 响应 ID
    pub id: String,
    /// 对象类型
    pub object: String,
    /// 创建时间戳
    pub created: i64,
    /// 模型名称
    pub model: String,
    /// 选择列表
    pub choices: Vec<StreamChoice>,
}

/// 流式选择项
#[derive(Debug, Serialize)]
pub struct StreamChoice {
    /// 索引
    pub index: u32,
    /// Delta 内容
    pub delta: DeltaContent,
    /// 结束原因
    pub finish_reason: Option<String>,
}

/// Delta 内容
#[derive(Debug, Serialize)]
pub struct DeltaContent {
    /// 角色（仅第一条）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// 内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Chat Completions 处理器
pub async fn chat_completions(
    State(state): State<AppState>,
    auth: AuthExtractor,
    request_id: RequestId,
    Json(request): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = std::result::Result<Event, Infallible>>>> {
    // 1. 构建 PricingSnapshot（请求开始时固化价格）
    let pricing = state
        .pricing
        .create_snapshot(&request.model, &auth.tenant_id)
        .await
        .map_err(|e| crate::error::ApiError::Internal(format!("Failed to create pricing snapshot: {}", e)))?;

    // 2. 构建 RequestContext
    let ctx = Arc::new(RequestContext {
        request_id: request_id.0,
        user_id: auth.user_id,
        tenant_id: auth.tenant_id,
        api_key_id: auth.api_key_id,
        model: request.model.clone(),
        messages: request.messages.clone(),
        stream: request.stream,
        pricing_snapshot: pricing,
        usage: UsageAccumulator::default(),
        started_at: chrono::Utc::now(),
    });

    // 3. 智能路由（只读）- 生成 ExecutionPlan
    let plan = state
        .routing
        .route(&ctx)
        .await
        .map_err(|e| crate::error::ApiError::Internal(format!("Routing failed: {}", e)))?;

    tracing::info!(
        request_id = %request_id.0,
        primary_provider = %plan.primary.provider,
        fallback_count = plan.fallback_chain.len(),
        "Routing decision made"
    );

    // TODO: 4. 执行（唯一执行层）
    // let (tx, rx) = tokio::sync::mpsc::channel(100);
    // let executor = state.gateway.clone();
    // ...

    // 5. 返回 SSE 流（简化实现，包含路由信息）
    let stream = create_mock_stream_with_routing(ctx, request.model, plan);

    Ok(Sse::new(stream))
}

/// 创建模拟流（用于测试）
fn create_mock_stream(
    _ctx: Arc<RequestContext>,
    model: String,
) -> impl Stream<Item = std::result::Result<Event, Infallible>> {
    use futures::stream::{self, StreamExt};
    use std::time::Duration;

    let chunks = vec![
        StreamChunk {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.clone(),
            choices: vec![StreamChoice {
                index: 0,
                delta: DeltaContent {
                    role: Some("assistant".to_string()),
                    content: None,
                },
                finish_reason: None,
            }],
        },
        StreamChunk {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.clone(),
            choices: vec![StreamChoice {
                index: 0,
                delta: DeltaContent {
                    role: None,
                    content: Some("Hello".to_string()),
                },
                finish_reason: None,
            }],
        },
        StreamChunk {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp(),
            model,
            choices: vec![StreamChoice {
                index: 0,
                delta: DeltaContent {
                    role: None,
                    content: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        },
    ];

    stream::iter(chunks)
        .then(|chunk| async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let data = serde_json::to_string(&chunk).unwrap();
            Ok(Event::default().data(data))
        })
        .chain(stream::once(async {
            Ok(Event::default().data("[DONE]"))
        }))
}

/// 创建包含路由信息的模拟流
fn create_mock_stream_with_routing(
    _ctx: Arc<RequestContext>,
    model: String,
    plan: keycompute_types::ExecutionPlan,
) -> impl Stream<Item = std::result::Result<Event, Infallible>> {
    use futures::stream::{self, StreamExt};
    use std::time::Duration;

    // 第一个 chunk 包含路由信息（通过自定义 header 或注释形式）
    let routing_info = format!(
        "[Routing] Primary: {}, Fallbacks: {:?}",
        plan.primary.provider,
        plan.fallback_chain.iter().map(|t| &t.provider).collect::<Vec<_>>()
    );

    let chunks = vec![
        StreamChunk {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.clone(),
            choices: vec![StreamChoice {
                index: 0,
                delta: DeltaContent {
                    role: Some("assistant".to_string()),
                    content: Some(routing_info),
                },
                finish_reason: None,
            }],
        },
        StreamChunk {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.clone(),
            choices: vec![StreamChoice {
                index: 0,
                delta: DeltaContent {
                    role: None,
                    content: Some("Hello from ".to_string() + &plan.primary.provider),
                },
                finish_reason: None,
            }],
        },
        StreamChunk {
            id: "chatcmpl-test".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp(),
            model,
            choices: vec![StreamChoice {
                index: 0,
                delta: DeltaContent {
                    role: None,
                    content: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        },
    ];

    stream::iter(chunks)
        .then(|chunk| async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let data = serde_json::to_string(&chunk).unwrap();
            Ok(Event::default().data(data))
        })
        .chain(stream::once(async {
            Ok(Event::default().data("[DONE]"))
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_deserialize() {
        let json = r#"{
            "model": "gpt-4o",
            "messages": [{"role": "user", "content": "Hello"}]
        }"#;
        let req: ChatRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.model, "gpt-4o");
        assert!(req.stream); // 默认值
    }

    #[test]
    fn test_stream_chunk_serialize() {
        let chunk = StreamChunk {
            id: "test".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: 1234567890,
            model: "gpt-4o".to_string(),
            choices: vec![],
        };
        let json = serde_json::to_string(&chunk).unwrap();
        assert!(json.contains("chat.completion.chunk"));
    }
}
