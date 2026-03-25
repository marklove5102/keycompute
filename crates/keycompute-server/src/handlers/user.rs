//! 用户自服务处理器
//!
//! 处理用户管理自己资源的请求
//! Admin 也可以访问这些端点，但会根据权限返回不同范围的数据

use crate::{
    error::{ApiError, Result},
    extractors::AuthExtractor,
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 当前用户信息响应
#[derive(Debug, Serialize)]
pub struct CurrentUserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub tenant_id: Uuid,
    pub created_at: String,
}

/// 获取当前用户信息
///
/// GET /api/v1/me
pub async fn get_current_user(auth: AuthExtractor) -> Result<Json<CurrentUserResponse>> {
    // 实际实现中应从数据库查询用户信息
    // 这里返回基于认证信息的基本数据
    Ok(Json(CurrentUserResponse {
        id: auth.user_id,
        email: "user@example.com".to_string(), // 实际应从数据库获取
        name: None,
        role: auth.role,
        tenant_id: auth.tenant_id,
        created_at: "2024-01-01T00:00:00Z".to_string(),
    }))
}

/// 更新个人资料请求
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// 更新个人资料
///
/// PUT /api/v1/me/profile
pub async fn update_profile(
    auth: AuthExtractor,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<serde_json::Value>> {
    // 实际实现中应更新数据库
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Profile updated",
        "user_id": auth.user_id,
        "updated_fields": {
            "name": req.name,
            "email": req.email,
        }
    })))
}

/// 修改密码请求
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// 修改密码
///
/// PUT /api/v1/me/password
pub async fn change_password(
    auth: AuthExtractor,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    // 实际实现中应验证当前密码并更新
    // 注意：这里应该调用 auth 服务进行密码修改
    if req.new_password.len() < 8 {
        return Err(ApiError::BadRequest(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Password changed successfully",
        "user_id": auth.user_id,
    })))
}

/// API Key 信息
#[derive(Debug, Serialize)]
pub struct ApiKeyInfo {
    pub id: Uuid,
    pub name: String,
    pub key_preview: String, // 只显示前几位
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub is_active: bool,
}

/// 列出我的 API Keys
///
/// GET /api/v1/keys
/// - 普通用户：只返回自己的 Keys
/// - Admin：可以返回所有 Keys（通过查询参数控制）
pub async fn list_my_api_keys(
    _auth: AuthExtractor,
    State(_state): State<AppState>,
) -> Result<Json<Vec<ApiKeyInfo>>> {
    // 实际实现中应从数据库查询
    // 这里返回模拟数据
    let keys = vec![ApiKeyInfo {
        id: Uuid::new_v4(),
        name: "Default Key".to_string(),
        key_preview: "sk-abc...".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        last_used_at: Some("2024-01-15T10:30:00Z".to_string()),
        is_active: true,
    }];

    Ok(Json(keys))
}

/// 创建 API Key 请求
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

/// 创建 API Key
///
/// POST /api/v1/keys
pub async fn create_api_key(
    _auth: AuthExtractor,
    State(_state): State<AppState>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<serde_json::Value>> {
    // 实际实现中应生成新的 API Key 并存储
    let new_key = format!(
        "sk-{}-{}-{}-{}-{}{}",
        uuid::Uuid::new_v4().to_string()[..8].to_string(),
        uuid::Uuid::new_v4().to_string()[..4].to_string(),
        uuid::Uuid::new_v4().to_string()[..4].to_string(),
        uuid::Uuid::new_v4().to_string()[..4].to_string(),
        uuid::Uuid::new_v4().to_string()[..12].to_string(),
        uuid::Uuid::new_v4().to_string()[..8].to_string(),
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "API Key created",
        "key": new_key, // 注意：这是唯一一次返回完整 key
        "key_id": Uuid::new_v4(),
        "name": req.name,
        "created_at": "2024-01-15T10:30:00Z",
    })))
}

/// 删除 API Key
///
/// DELETE /api/v1/keys/{id}
pub async fn delete_api_key(
    _auth: AuthExtractor,
    Path(key_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    // 实际实现中应验证所有权并删除
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "API Key deleted",
        "key_id": key_id,
    })))
}

/// 用量记录
#[derive(Debug, Serialize)]
pub struct UsageRecord {
    pub id: Uuid,
    pub request_id: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub cost: f64,
    pub status: String,
    pub created_at: String,
}

/// 获取我的用量记录
///
/// GET /api/v1/usage
/// - 普通用户：只返回自己的用量
/// - Admin：可以返回所有用量（通过查询参数控制）
pub async fn get_my_usage(
    _auth: AuthExtractor,
    State(_state): State<AppState>,
) -> Result<Json<Vec<UsageRecord>>> {
    // 实际实现中应从 usage_logs 表查询
    let usage = vec![UsageRecord {
        id: Uuid::new_v4(),
        request_id: "req_123".to_string(),
        model: "gpt-4".to_string(),
        input_tokens: 100,
        output_tokens: 200,
        total_tokens: 300,
        cost: 0.015,
        status: "success".to_string(),
        created_at: "2024-01-15T10:30:00Z".to_string(),
    }];

    Ok(Json(usage))
}

/// 用量统计响应
#[derive(Debug, Serialize)]
pub struct UsageStatsResponse {
    pub total_requests: i64,
    pub total_tokens: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost: f64,
    pub period: String,
}

/// 获取我的用量统计
///
/// GET /api/v1/usage/stats
pub async fn get_my_usage_stats(
    _auth: AuthExtractor,
    State(_state): State<AppState>,
) -> Result<Json<UsageStatsResponse>> {
    // 实际实现中应聚合 usage_logs 数据
    Ok(Json(UsageStatsResponse {
        total_requests: 100,
        total_tokens: 50000,
        total_input_tokens: 20000,
        total_output_tokens: 30000,
        total_cost: 2.5,
        period: "last_30_days".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_user_response_serialization() {
        let user = CurrentUserResponse {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            role: "user".to_string(),
            tenant_id: Uuid::new_v4(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("test@example.com"));
    }
}
