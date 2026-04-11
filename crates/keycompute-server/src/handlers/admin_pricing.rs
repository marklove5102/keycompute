//! 定价管理处理器
//!
//! 处理需要 Admin 权限的定价管理请求

use crate::{
    error::{ApiError, Result},
    extractors::AuthExtractor,
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
};
use bigdecimal::BigDecimal;
use keycompute_db::models::pricing_model::{
    CreatePricingRequest, PricingModel, UpdatePricingRequest,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 定价信息
#[derive(Debug, Serialize)]
pub struct PricingInfo {
    pub id: Uuid,
    pub model_name: String,
    pub provider: String,
    pub currency: String,
    pub input_price_per_1k: String,
    pub output_price_per_1k: String,
    pub is_default: bool,
    pub is_effective: bool,
    pub effective_from: String,
    pub effective_until: Option<String>,
    pub created_at: String,
}

/// 创建定价请求（管理员）
#[derive(Debug, Deserialize)]
pub struct CreatePricingAdminRequest {
    /// 模型名称
    pub model_name: String,
    /// Provider
    pub provider: String,
    /// 货币（默认 CNY）
    #[serde(default = "default_currency")]
    pub currency: String,
    /// 输入价格（每 1k tokens）
    pub input_price_per_1k: String,
    /// 输出价格（每 1k tokens）
    pub output_price_per_1k: String,
    /// 是否为默认定价
    #[serde(default)]
    pub is_default: bool,
    /// 生效时间（可选）
    pub effective_from: Option<String>,
    /// 失效时间（可选）
    pub effective_until: Option<String>,
}

fn default_currency() -> String {
    "CNY".to_string()
}

/// 更新定价请求（管理员）
#[derive(Debug, Deserialize)]
pub struct UpdatePricingAdminRequest {
    /// 输入价格（每 1k tokens）
    pub input_price_per_1k: Option<String>,
    /// 输出价格（每 1k tokens）
    pub output_price_per_1k: Option<String>,
    /// 失效时间
    pub effective_until: Option<String>,
}

/// 列出所有定价
///
/// GET /api/v1/pricing
pub async fn list_pricing(
    auth: AuthExtractor,
    State(state): State<AppState>,
) -> Result<Json<Vec<PricingInfo>>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Database not configured".to_string()))?;

    let pricing_models = PricingModel::find_by_tenant(pool, auth.tenant_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to query pricing: {}", e)))?;

    let pricing_list: Vec<PricingInfo> = pricing_models
        .into_iter()
        .map(|p| {
            let is_effective = p.is_effective();
            PricingInfo {
                id: p.id,
                model_name: p.model_name,
                provider: p.provider,
                currency: p.currency,
                input_price_per_1k: p.input_price_per_1k.to_string(),
                output_price_per_1k: p.output_price_per_1k.to_string(),
                is_default: p.is_default,
                is_effective,
                effective_from: p.effective_from.to_rfc3339(),
                effective_until: p.effective_until.map(|t| t.to_rfc3339()),
                created_at: p.created_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(Json(pricing_list))
}

/// 创建定价
///
/// POST /api/v1/pricing
pub async fn create_pricing(
    auth: AuthExtractor,
    State(state): State<AppState>,
    Json(req): Json<CreatePricingAdminRequest>,
) -> Result<Json<serde_json::Value>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Database not configured".to_string()))?;

    // 解析价格
    let input_price: BigDecimal = req
        .input_price_per_1k
        .parse()
        .map_err(|_| ApiError::BadRequest("Invalid input_price_per_1k".to_string()))?;

    let output_price: BigDecimal = req
        .output_price_per_1k
        .parse()
        .map_err(|_| ApiError::BadRequest("Invalid output_price_per_1k".to_string()))?;

    let db_req = CreatePricingRequest {
        tenant_id: if req.is_default {
            None
        } else {
            Some(auth.tenant_id)
        },
        model_name: req.model_name.clone(),
        provider: req.provider.clone(),
        currency: Some(req.currency.clone()),
        input_price_per_1k: input_price,
        output_price_per_1k: output_price,
        is_default: Some(req.is_default),
        effective_from: req.effective_from.as_ref().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .map(|d| d.with_timezone(&chrono::Utc))
                .ok()
        }),
        effective_until: req.effective_until.as_ref().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .map(|d| d.with_timezone(&chrono::Utc))
                .ok()
        }),
    };

    let pricing = PricingModel::create(pool, &db_req)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to create pricing: {}", e)))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Pricing created",
        "pricing_id": pricing.id,
        "model_name": pricing.model_name,
        "provider": pricing.provider,
        "input_price_per_1k": pricing.input_price_per_1k.to_string(),
        "output_price_per_1k": pricing.output_price_per_1k.to_string(),
        "is_default": pricing.is_default,
        "created_by": auth.user_id,
    })))
}

/// 更新定价
///
/// PUT /api/v1/pricing/{id}
pub async fn update_pricing(
    auth: AuthExtractor,
    Path(pricing_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<UpdatePricingAdminRequest>,
) -> Result<Json<serde_json::Value>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Database not configured".to_string()))?;

    // 查找现有定价
    let existing = PricingModel::find_by_id(pool, pricing_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to find pricing: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Pricing not found: {}", pricing_id)))?;

    // 解析价格
    let input_price = req.input_price_per_1k.as_ref().and_then(|s| s.parse().ok());

    let output_price = req
        .output_price_per_1k
        .as_ref()
        .and_then(|s| s.parse().ok());

    let effective_until = req.effective_until.as_ref().and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(s)
            .map(|d| d.with_timezone(&chrono::Utc))
            .ok()
    });

    let db_req = UpdatePricingRequest {
        input_price_per_1k: input_price,
        output_price_per_1k: output_price,
        effective_until,
    };

    let updated = existing
        .update(pool, &db_req)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to update pricing: {}", e)))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Pricing updated",
        "pricing_id": updated.id,
        "updated_fields": {
            "input_price_per_1k": req.input_price_per_1k,
            "output_price_per_1k": req.output_price_per_1k,
            "effective_until": req.effective_until.clone(),
        },
        "updated_by": auth.user_id,
    })))
}

/// 删除定价
///
/// DELETE /api/v1/pricing/{id}
pub async fn delete_pricing(
    auth: AuthExtractor,
    Path(pricing_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Database not configured".to_string()))?;

    // 查找并删除定价
    let existing = PricingModel::find_by_id(pool, pricing_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to find pricing: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Pricing not found: {}", pricing_id)))?;

    existing
        .delete(pool)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to delete pricing: {}", e)))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Pricing deleted",
        "pricing_id": pricing_id,
        "deleted_by": auth.user_id,
    })))
}

/// 批量设置默认定价
///
/// POST /api/v1/pricing/batch-defaults
///
/// 为常用模型设置默认定价
pub async fn set_default_pricing(
    auth: AuthExtractor,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Database not configured".to_string()))?;

    // 默认定价数据（参考 PricingService）
    let defaults = vec![
        ("gpt-4o", "openai", "0.5", "1.5"),
        ("gpt-4o-mini", "openai", "0.15", "0.6"),
        ("gpt-4-turbo", "openai", "1.0", "3.0"),
        ("gpt-3.5-turbo", "openai", "0.05", "0.15"),
        ("claude-3-5-sonnet-20241022", "anthropic", "0.3", "1.5"),
        ("claude-3-opus-20240229", "anthropic", "1.5", "7.5"),
        ("deepseek-chat", "deepseek", "0.01", "0.03"),
        ("deepseek-reasoner", "deepseek", "0.05", "0.15"),
    ];

    let mut created = 0;
    let mut skipped = 0;

    for (model_name, provider, input_price, output_price) in defaults {
        // 检查是否已存在
        let existing = PricingModel::find_by_model(pool, auth.tenant_id, model_name, provider)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to check existing pricing: {}", e)))?;

        if existing.is_some() {
            skipped += 1;
            continue;
        }

        let db_req = CreatePricingRequest {
            tenant_id: None,
            model_name: model_name.to_string(),
            provider: provider.to_string(),
            currency: Some("CNY".to_string()),
            input_price_per_1k: input_price.parse().unwrap(),
            output_price_per_1k: output_price.parse().unwrap(),
            is_default: Some(true),
            effective_from: None,
            effective_until: None,
        };

        match PricingModel::create(pool, &db_req).await {
            Ok(_) => created += 1,
            Err(e) => {
                tracing::warn!(model = model_name, error = %e, "Failed to create default pricing");
            }
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Default pricing set",
        "created": created,
        "skipped": skipped,
        "set_by": auth.user_id,
    })))
}
