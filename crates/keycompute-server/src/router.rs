//! 路由配置
//!
//! Axum Router 配置，挂载所有路由

use crate::{
    handlers::{
        calculate_cost, chat_completions, check_provider_health, debug_routing, get_billing_stats,
        get_execution_stats, get_gateway_status, get_pricing, get_pricing_cost, get_provider_health,
        health_check, list_billing_records, list_models, trigger_billing,
    },
    middleware::{
        cors_layer, rate_limit_middleware, request_logger, trace_id_middleware,
    },
    state::AppState,
};
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

/// 创建路由器
pub fn create_router(state: AppState) -> Router {
    // OpenAI 兼容 API 路由（需要限流）
    let api_routes = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        // API 路由添加限流中间件
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware));

    // 定价管理路由（需要限流）
    let pricing_routes = Router::new()
        .route("/v1/pricing", get(get_pricing))
        .route("/v1/pricing/calculate", post(get_pricing_cost))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware));

    // Billing 管理路由（需要限流）
    let billing_routes = Router::new()
        .route("/v1/billing/records", get(list_billing_records))
        .route("/v1/billing/stats", get(get_billing_stats))
        .route("/v1/billing/trigger", post(trigger_billing))
        .route("/v1/billing/calculate", post(calculate_cost))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware));

    // 路由调试接口（需要限流）
    let routing_debug_routes = Router::new()
        .route("/debug/routing", get(debug_routing))
        .route("/debug/providers", get(get_provider_health))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware));

    // Gateway 调试接口（需要限流）
    let gateway_debug_routes = Router::new()
        .route("/debug/gateway/status", get(get_gateway_status))
        .route("/debug/gateway/stats", get(get_execution_stats))
        .route("/debug/gateway/health", post(check_provider_health))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware));

    // 健康检查路由（不需要限流）
    let health_routes = Router::new().route("/health", get(health_check));

    // 合并所有路由
    Router::new()
        .merge(api_routes)
        .merge(pricing_routes)
        .merge(billing_routes)
        .merge(routing_debug_routes)
        .merge(gateway_debug_routes)
        .merge(health_routes)
        .layer(axum::middleware::from_fn(request_logger))
        .layer(axum::middleware::from_fn(trace_id_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_router() {
        let state = AppState::new();
        let router = create_router(state);
        // 确保可以创建路由器
        let _ = router;
    }
}
