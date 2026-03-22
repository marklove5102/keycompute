//! Billing 管理接口
//!
//! 用于查询计费记录和手动触发计费结算

use crate::{error::Result, extractors::AuthExtractor, state::AppState};
use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 计费记录查询请求
#[derive(Debug, Deserialize)]
pub struct ListBillingQuery {
    /// 分页偏移
    #[serde(default)]
    pub offset: Option<i64>,
    /// 分页限制
    #[serde(default)]
    pub limit: Option<i64>,
    /// 开始时间
    #[serde(default)]
    pub start_time: Option<DateTime<Utc>>,
    /// 结束时间
    #[serde(default)]
    pub end_time: Option<DateTime<Utc>>,
}

/// 计费记录响应
#[derive(Debug, Serialize)]
pub struct BillingListResponse {
    /// 记录列表
    pub records: Vec<BillingRecord>,
    /// 总记录数
    pub total: i64,
}

/// 计费记录
#[derive(Debug, Serialize)]
pub struct BillingRecord {
    /// 记录 ID
    pub id: Uuid,
    /// 请求 ID
    pub request_id: Uuid,
    /// 模型名称
    pub model_name: String,
    /// Provider 名称
    pub provider_name: String,
    /// 输入 token 数
    pub input_tokens: i32,
    /// 输出 token 数
    pub output_tokens: i32,
    /// 总金额
    pub user_amount: Decimal,
    /// 货币
    pub currency: String,
    /// 状态
    pub status: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 列出计费记录
pub async fn list_billing_records(
    State(_state): State<AppState>,
    _auth: AuthExtractor,
    Query(query): Query<ListBillingQuery>,
) -> Result<Json<BillingListResponse>> {
    // TODO: 从数据库查询实际的计费记录
    // 目前返回模拟数据
    let records = vec![
        BillingRecord {
            id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            model_name: "gpt-4o".to_string(),
            provider_name: "openai".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            user_amount: Decimal::from(2),
            currency: "CNY".to_string(),
            status: "success".to_string(),
            created_at: Utc::now(),
        },
    ];

    Ok(Json(BillingListResponse {
        records,
        total: 1,
    }))
}

/// 计费统计查询请求
#[derive(Debug, Deserialize)]
pub struct BillingStatsQuery {
    /// 开始时间
    #[serde(default)]
    pub start_time: Option<DateTime<Utc>>,
    /// 结束时间
    #[serde(default)]
    pub end_time: Option<DateTime<Utc>>,
    /// 按模型分组
    #[serde(default)]
    pub group_by_model: Option<bool>,
}

/// 计费统计响应
#[derive(Debug, Serialize)]
pub struct BillingStatsResponse {
    /// 总请求数
    pub total_requests: i64,
    /// 总输入 tokens
    pub total_input_tokens: i64,
    /// 总输出 tokens
    pub total_output_tokens: i64,
    /// 总金额
    pub total_amount: Decimal,
    /// 货币
    pub currency: String,
    /// 按模型统计
    pub by_model: Vec<ModelStats>,
}

/// 模型统计
#[derive(Debug, Serialize)]
pub struct ModelStats {
    /// 模型名称
    pub model_name: String,
    /// 请求数
    pub request_count: i64,
    /// 输入 tokens
    pub input_tokens: i64,
    /// 输出 tokens
    pub output_tokens: i64,
    /// 金额
    pub amount: Decimal,
}

/// 获取计费统计
pub async fn get_billing_stats(
    State(_state): State<AppState>,
    _auth: AuthExtractor,
    Query(query): Query<BillingStatsQuery>,
) -> Result<Json<BillingStatsResponse>> {
    // TODO: 从数据库查询实际的统计数据
    // 目前返回模拟数据
    let by_model = vec![
        ModelStats {
            model_name: "gpt-4o".to_string(),
            request_count: 100,
            input_tokens: 100000,
            output_tokens: 50000,
            amount: Decimal::from(200),
        },
        ModelStats {
            model_name: "gpt-3.5-turbo".to_string(),
            request_count: 200,
            input_tokens: 150000,
            output_tokens: 80000,
            amount: Decimal::from(150),
        },
    ];

    Ok(Json(BillingStatsResponse {
        total_requests: 300,
        total_input_tokens: 250000,
        total_output_tokens: 130000,
        total_amount: Decimal::from(350),
        currency: "CNY".to_string(),
        by_model,
    }))
}

/// 手动触发计费请求
#[derive(Debug, Deserialize)]
pub struct TriggerBillingRequest {
    /// 请求 ID
    pub request_id: Uuid,
    /// Provider 名称
    pub provider_name: String,
    /// 账号 ID
    pub account_id: Uuid,
    /// 状态
    pub status: String,
}

/// 手动触发计费响应
#[derive(Debug, Serialize)]
pub struct TriggerBillingResponse {
    /// 是否成功
    pub success: bool,
    /// 记录 ID
    pub record_id: Option<Uuid>,
    /// 消息
    pub message: String,
}

/// 手动触发计费（调试用）
pub async fn trigger_billing(
    State(state): State<AppState>,
    _auth: AuthExtractor,
    Json(request): Json<TriggerBillingRequest>,
) -> Result<Json<TriggerBillingResponse>> {
    // TODO: 从存储中获取对应的 RequestContext 并触发计费
    // 目前返回模拟响应
    tracing::info!(
        request_id = %request.request_id,
        provider = %request.provider_name,
        "Manual billing triggered"
    );

    Ok(Json(TriggerBillingResponse {
        success: true,
        record_id: Some(Uuid::new_v4()),
        message: "Billing triggered successfully".to_string(),
    }))
}

/// 费用计算请求
#[derive(Debug, Deserialize)]
pub struct CalculateCostRequest {
    /// 模型名称
    pub model: String,
    /// 输入 token 数
    pub input_tokens: u32,
    /// 输出 token 数
    pub output_tokens: u32,
}

/// 费用计算响应
#[derive(Debug, Serialize)]
pub struct CalculateCostResponse {
    /// 模型名称
    pub model: String,
    /// 输入 token 数
    pub input_tokens: u32,
    /// 输出 token 数
    pub output_tokens: u32,
    /// 输入费用
    pub input_cost: Decimal,
    /// 输出费用
    pub output_cost: Decimal,
    /// 总费用
    pub total_cost: Decimal,
    /// 货币
    pub currency: String,
}

/// 计算费用（基于 PricingSnapshot）
pub async fn calculate_cost(
    State(state): State<AppState>,
    _auth: AuthExtractor,
    Json(request): Json<CalculateCostRequest>,
) -> Result<Json<CalculateCostResponse>> {
    // 使用默认租户 ID 创建价格快照
    let tenant_id = Uuid::nil();
    let pricing = state
        .pricing
        .create_snapshot(&request.model, &tenant_id)
        .await
        .map_err(|e| crate::error::ApiError::Internal(format!("Failed to get pricing: {}", e)))?;

    // 计算费用
    let input_cost = Decimal::from(request.input_tokens) / Decimal::from(1000)
        * pricing.input_price_per_1k;
    let output_cost = Decimal::from(request.output_tokens) / Decimal::from(1000)
        * pricing.output_price_per_1k;
    let total_cost = input_cost + output_cost;

    Ok(Json(CalculateCostResponse {
        model: request.model,
        input_tokens: request.input_tokens,
        output_tokens: request.output_tokens,
        input_cost,
        output_cost,
        total_cost,
        currency: pricing.currency,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_billing_query_deserialize() {
        let json = r#"{"offset": 0, "limit": 10}"#;
        let query: ListBillingQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.offset, Some(0));
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_calculate_cost_request_deserialize() {
        let json = r#"{"model": "gpt-4o", "input_tokens": 1000, "output_tokens": 500}"#;
        let req: CalculateCostRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.model, "gpt-4o");
        assert_eq!(req.input_tokens, 1000);
        assert_eq!(req.output_tokens, 500);
    }

    #[test]
    fn test_trigger_billing_request_deserialize() {
        let request_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();
        let json = format!(
            r#"{{"request_id": "{}", "provider_name": "openai", "account_id": "{}", "status": "success"}}"#,
            request_id, account_id
        );
        let req: TriggerBillingRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.request_id, request_id);
        assert_eq!(req.provider_name, "openai");
        assert_eq!(req.status, "success");
    }
}
