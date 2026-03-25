//! 路由配置
//!
//! Axum Router 配置，挂载所有路由
//!
//! 路由设计原则：
//! - 统一使用 /api/v1/* 前缀（除 OpenAI 兼容 API 外）
//! - 权限控制通过中间件实现，而非路径前缀
//! - Admin 和普通用户共用前端，通过权限控制展示不同模块

use crate::{
    handlers::{
        calculate_cost,
        change_password,
        // OpenAI 兼容 API
        chat_completions,
        check_provider_health,
        create_account,
        create_api_key,
        // 调试接口
        debug_routing,
        delete_account,
        delete_api_key,
        delete_user,
        // 认证相关
        forgot_password_handler,
        get_billing_stats,
        // 用户自服务
        get_current_user,
        get_execution_stats,
        get_gateway_status,
        get_my_usage,
        get_my_usage_stats,
        // 定价和账单
        get_pricing,
        get_provider_health,
        get_system_settings,
        get_user_by_id,
        // 健康检查
        health_check,
        list_accounts,
        list_all_api_keys,
        // 管理功能（Admin 权限）
        list_all_users,
        list_billing_records,
        list_models,
        list_my_api_keys,
        list_tenants,
        login_handler,
        refresh_account,
        refresh_token_handler,
        register_handler,
        resend_verification_handler,
        reset_password_handler,
        test_account,
        update_account,
        update_profile,
        update_system_settings,
        update_user,
        update_user_balance,
        verify_email_handler,
        verify_reset_token_handler,
    },
    middleware::{cors_layer, rate_limit_middleware, request_logger, trace_id_middleware},
    state::AppState,
};
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
};
use tower_http::trace::TraceLayer;

/// 创建路由器
pub fn create_router(state: AppState) -> Router {
    // ==================== 1. 认证路由（不需要限流） ====================
    let auth_routes = Router::new()
        .route("/auth/register", post(register_handler))
        .route("/auth/login", post(login_handler))
        .route("/auth/verify-email/{token}", get(verify_email_handler))
        .route("/auth/forgot-password", post(forgot_password_handler))
        .route("/auth/reset-password", post(reset_password_handler))
        .route(
            "/auth/verify-reset-token/{token}",
            get(verify_reset_token_handler),
        )
        .route("/auth/refresh-token", post(refresh_token_handler))
        .route(
            "/auth/resend-verification",
            post(resend_verification_handler),
        );

    // ==================== 2. OpenAI 兼容 API（需要限流） ====================
    // 这些端点使用 API Key 认证，路径保持与 OpenAI 一致
    let openai_routes = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware));

    // ==================== 3. 用户自服务 API（需要认证 + 限流） ====================
    // 用户管理自己的资源，Admin 也可以访问（根据业务逻辑返回不同范围的数据）
    let user_routes = Router::new()
        // 当前用户信息
        .route("/api/v1/me", get(get_current_user))
        .route("/api/v1/me/profile", put(update_profile))
        .route("/api/v1/me/password", put(change_password))
        // API Keys 管理
        .route("/api/v1/keys", get(list_my_api_keys).post(create_api_key))
        .route("/api/v1/keys/{id}", delete(delete_api_key))
        // 用量统计
        .route("/api/v1/usage", get(get_my_usage))
        .route("/api/v1/usage/stats", get(get_my_usage_stats))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware));

    // ==================== 4. 管理功能 API（需要 Admin 权限） ====================
    // 用户管理（Admin 可以管理所有用户，普通用户只能看自己）
    let admin_user_routes = Router::new()
        .route("/api/v1/users", get(list_all_users))
        .route(
            "/api/v1/users/{id}",
            get(get_user_by_id).put(update_user).delete(delete_user),
        )
        .route("/api/v1/users/{id}/balance", post(update_user_balance))
        .route("/api/v1/users/{id}/api-keys", get(list_all_api_keys));

    // 账号/渠道管理（仅 Admin）
    let admin_account_routes = Router::new()
        .route("/api/v1/accounts", get(list_accounts).post(create_account))
        .route(
            "/api/v1/accounts/{id}",
            put(update_account).delete(delete_account),
        )
        .route("/api/v1/accounts/{id}/test", post(test_account))
        .route("/api/v1/accounts/{id}/refresh", post(refresh_account));

    // 租户管理（仅 Admin）
    let admin_tenant_routes = Router::new().route("/api/v1/tenants", get(list_tenants));

    // 系统设置（仅 Admin）
    let admin_settings_routes = Router::new().route(
        "/api/v1/settings",
        get(get_system_settings).put(update_system_settings),
    );

    // 合并管理路由并添加限流
    let admin_routes = admin_user_routes
        .merge(admin_account_routes)
        .merge(admin_tenant_routes)
        .merge(admin_settings_routes)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware));

    // ==================== 5. 定价和账单 API ====================
    let billing_routes = Router::new()
        // 定价查询（公开或限流）
        .route("/api/v1/pricing", get(get_pricing))
        .route("/api/v1/pricing/calculate", post(calculate_cost))
        // 账单记录（用户看自己的，Admin 看所有）
        .route("/api/v1/billing/records", get(list_billing_records))
        .route("/api/v1/billing/stats", get(get_billing_stats))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware));

    // ==================== 6. 调试接口（仅开发/Admin 使用） ====================
    let debug_routes = Router::new()
        .route("/debug/routing", get(debug_routing))
        .route("/debug/providers", get(get_provider_health))
        .route("/debug/gateway/status", get(get_gateway_status))
        .route("/debug/gateway/stats", get(get_execution_stats))
        .route("/debug/gateway/health", post(check_provider_health))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware));

    // ==================== 7. 健康检查（公开） ====================
    let health_routes = Router::new().route("/health", get(health_check));

    // ==================== 合并所有路由 ====================
    Router::new()
        .merge(auth_routes)
        .merge(openai_routes)
        .merge(user_routes)
        .merge(admin_routes)
        .merge(billing_routes)
        .merge(debug_routes)
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
