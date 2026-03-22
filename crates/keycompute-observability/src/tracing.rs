use tracing::Span;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

/// 创建请求追踪 Span
///
/// 用于跟踪单个请求在系统中的流转
pub fn create_request_span(request_id: &str, user_id: &str, model: &str) -> Span {
    tracing::info_span!(
        "request",
        request_id = %request_id,
        user_id = %user_id,
        model = %model,
    )
}

/// 创建 Provider 调用追踪 Span
pub fn create_provider_span(provider: &str, model: &str, account_id: &str) -> Span {
    tracing::info_span!(
        "provider_call",
        provider = %provider,
        model = %model,
        account_id = %account_id,
    )
}

/// 创建流处理追踪 Span
pub fn create_stream_span(request_id: &str) -> Span {
    tracing::info_span!(
        "stream_processing",
        request_id = %request_id,
    )
}

/// 记录关键业务事件
pub mod events {
    use tracing::{error, info, warn};

    /// 记录路由决策
    pub fn routing_decision(request_id: &str, provider: &str, account_id: &str, score: f64) {
        info!(
            request_id = %request_id,
            provider = %provider,
            account_id = %account_id,
            score = %score,
            "Routing decision made"
        );
    }

    /// 记录 Fallback 事件
    pub fn fallback_triggered(
        request_id: &str,
        from_provider: &str,
        to_provider: &str,
        reason: &str,
    ) {
        warn!(
            request_id = %request_id,
            from_provider = %from_provider,
            to_provider = %to_provider,
            reason = %reason,
            "Fallback triggered"
        );
    }

    /// 记录计费事件
    pub fn billing_recorded(request_id: &str, amount: f64, currency: &str, tokens: u32) {
        info!(
            request_id = %request_id,
            amount = %amount,
            currency = %currency,
            tokens = %tokens,
            "Billing recorded"
        );
    }

    /// 记录限流事件
    pub fn rate_limit_hit(key_id: &str, limit_type: &str) {
        warn!(
            key_id = %key_id,
            limit_type = %limit_type,
            "Rate limit exceeded"
        );
    }

    /// 记录 Provider 错误
    pub fn provider_error(provider: &str, error: &str, status_code: Option<u16>) {
        error!(
            provider = %provider,
            error = %error,
            status_code = ?status_code,
            "Provider error occurred"
        );
    }
}

/// 自定义追踪层配置
#[derive(Debug, Default)]
pub struct CustomLayer;

impl<S> Layer<S> for CustomLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, _event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        // 可以在这里添加自定义的事件处理逻辑
        // 例如：发送到外部追踪系统、过滤特定事件等
    }
}

/// 初始化分布式追踪（预留接口）
///
/// MVP 阶段可以仅使用本地 tracing，后续可接入 OpenTelemetry
pub fn init_distributed_tracing(_service_name: &str) {
    // TODO: 接入 OpenTelemetry Collector
    // 当需要分布式追踪时，可以在这里初始化 opentelemetry-otlp
    tracing::info!("Distributed tracing initialized (local mode)");
}
