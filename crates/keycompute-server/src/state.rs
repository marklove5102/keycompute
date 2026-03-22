//! 应用状态
//!
//! AppState 定义（DB Pool, Redis, 各模块 Handle）

use keycompute_auth::{ApiKeyValidator, AuthService};
use keycompute_runtime::AccountStateStore;
use keycompute_routing::RoutingEngine;
use std::sync::Arc;

/// 应用状态
#[derive(Debug, Clone)]
pub struct AppState {
    /// 认证服务
    pub auth: Arc<AuthService>,
    /// 限流服务
    pub rate_limiter: Arc<keycompute_ratelimit::RateLimitService>,
    /// 定价服务
    pub pricing: Arc<keycompute_pricing::PricingService>,
    /// 运行时状态存储
    pub account_states: Arc<AccountStateStore>,
    /// 路由引擎
    pub routing: Arc<RoutingEngine>,
    // TODO: 添加其他模块服务
    // pub gateway: Arc<llm_gateway::GatewayExecutor>,
    // pub billing: Arc<keycompute_billing::BillingService>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new() -> Self {
        // 创建 API Key 验证器
        let api_key_validator = ApiKeyValidator::new("default-secret");
        let auth_service = AuthService::new(api_key_validator);

        // 创建定价服务
        let pricing_service = keycompute_pricing::PricingService::new();

        // 创建运行时状态存储
        let account_states = Arc::new(AccountStateStore::new());

        // 创建路由引擎
        let routing_engine = Arc::new(RoutingEngine::new(Arc::clone(&account_states)));

        Self {
            auth: Arc::new(auth_service),
            rate_limiter: Arc::new(keycompute_ratelimit::RateLimitService::default_memory()),
            pricing: Arc::new(pricing_service),
            account_states,
            routing: routing_engine,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        // 基础测试，确保可以创建
        let _ = state;
    }
}
