//! 路由调试处理器
//!
//! 用于查看路由决策过程和状态

use crate::{
    error::{ApiError, Result},
    extractors::AuthExtractor,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};
use keycompute_types::{ExecutionPlan, PricingSnapshot, RequestContext, UsageAccumulator};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// 路由调试请求
#[derive(Debug, Deserialize)]
pub struct RoutingDebugQuery {
    /// 模型名称
    pub model: String,
}

/// 路由目标信息
#[derive(Debug, Serialize)]
pub struct RoutingTargetInfo {
    /// Provider 名称
    pub provider: String,
    /// 账号 ID
    pub account_id: Uuid,
    /// 端点
    pub endpoint: String,
}

/// 路由调试响应
#[derive(Debug, Serialize)]
pub struct RoutingDebugResponse {
    /// 请求 ID
    pub request_id: Uuid,
    /// 主目标
    pub primary: RoutingTargetInfo,
    /// 备用链路
    pub fallback_chain: Vec<RoutingTargetInfo>,
    /// 使用的定价
    pub pricing: PricingInfo,
}

/// 定价信息
#[derive(Debug, Serialize)]
pub struct PricingInfo {
    /// 模型名称
    pub model_name: String,
    /// 货币
    pub currency: String,
    /// 输入价格（每 1K tokens）
    pub input_price_per_1k: String,
    /// 输出价格（每 1K tokens）
    pub output_price_per_1k: String,
}

/// 路由调试接口
///
/// 模拟一个请求，返回路由决策结果（不实际执行）
pub async fn debug_routing(
    State(state): State<AppState>,
    auth: AuthExtractor,
    Query(query): Query<RoutingDebugQuery>,
) -> Result<Json<RoutingDebugResponse>> {
    // 1. 构建 PricingSnapshot
    let pricing = state
        .pricing
        .create_snapshot(&query.model, &auth.tenant_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to create pricing snapshot: {}", e)))?;

    // 2. 构建模拟的 RequestContext
    let request_id = Uuid::new_v4();
    let ctx = RequestContext {
        request_id,
        user_id: auth.user_id,
        tenant_id: auth.tenant_id,
        api_key_id: auth.api_key_id,
        model: query.model.clone(),
        messages: vec![],
        stream: true,
        pricing_snapshot: pricing.clone(),
        usage: UsageAccumulator::default(),
        started_at: chrono::Utc::now(),
    };

    // 3. 执行路由（只读）
    let plan = state
        .routing
        .route(&ctx)
        .await
        .map_err(|e| ApiError::Internal(format!("Routing failed: {}", e)))?;

    // 4. 构建响应
    let response = RoutingDebugResponse {
        request_id,
        primary: RoutingTargetInfo {
            provider: plan.primary.provider.clone(),
            account_id: plan.primary.account_id,
            endpoint: plan.primary.endpoint.clone(),
        },
        fallback_chain: plan
            .fallback_chain
            .iter()
            .map(|t| RoutingTargetInfo {
                provider: t.provider.clone(),
                account_id: t.account_id,
                endpoint: t.endpoint.clone(),
            })
            .collect(),
        pricing: PricingInfo {
            model_name: pricing.model_name,
            currency: pricing.currency,
            input_price_per_1k: pricing.input_price_per_1k.to_string(),
            output_price_per_1k: pricing.output_price_per_1k.to_string(),
        },
    };

    tracing::info!(
        request_id = %request_id,
        primary_provider = %plan.primary.provider,
        fallback_count = plan.fallback_chain.len(),
        "Routing debug completed"
    );

    Ok(Json(response))
}

/// Provider 健康状态响应
#[derive(Debug, Serialize)]
pub struct ProviderHealthResponse {
    /// 可用 Provider 列表
    pub healthy_providers: Vec<String>,
    /// 账号状态存储中的账号数量
    pub account_count: usize,
}

/// 获取 Provider 健康状态
pub async fn get_provider_health(
    State(state): State<AppState>,
    _auth: AuthExtractor,
) -> Result<Json<ProviderHealthResponse>> {
    let providers = state.routing.healthy_providers().to_vec();

    Ok(Json(ProviderHealthResponse {
        healthy_providers: providers,
        account_count: 0, // TODO: 从 AccountStateStore 获取实际数量
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_debug_query_deserialize() {
        let json = r#"{"model": "gpt-4o"}"#;
        let query: RoutingDebugQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.model, "gpt-4o");
    }

    #[test]
    fn test_routing_target_info_serialize() {
        let info = RoutingTargetInfo {
            provider: "openai".to_string(),
            account_id: Uuid::new_v4(),
            endpoint: "https://api.openai.com/v1".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("openai"));
    }
}
