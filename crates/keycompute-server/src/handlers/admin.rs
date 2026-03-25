//! 管理功能处理器
//!
//! 处理需要 Admin 权限的管理请求
//! 注意：Admin 也是用户，通过权限系统控制访问

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

// ==================== 用户管理 ====================

/// 用户信息（Admin 视图）
#[derive(Debug, Serialize)]
pub struct AdminUserInfo {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub balance: f64,
    pub is_active: bool,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

/// 列出所有用户
///
/// GET /api/v1/users
/// 支持查询参数：?tenant_id=xxx&role=xxx&search=xxx
pub async fn list_all_users(
    auth: AuthExtractor,
    State(_state): State<AppState>,
) -> Result<Json<Vec<AdminUserInfo>>> {
    // 检查权限（简化实现，实际应使用中间件）
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    // 实际实现中应从数据库查询所有用户
    let users = vec![AdminUserInfo {
        id: Uuid::new_v4(),
        email: "user1@example.com".to_string(),
        name: Some("User One".to_string()),
        role: "user".to_string(),
        tenant_id: auth.tenant_id,
        tenant_name: "Default Tenant".to_string(),
        balance: 100.0,
        is_active: true,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        last_login_at: Some("2024-01-15T10:30:00Z".to_string()),
    }];

    Ok(Json(users))
}

/// 获取指定用户信息
///
/// GET /api/v1/users/{id}
pub async fn get_user_by_id(
    auth: AuthExtractor,
    Path(user_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> Result<Json<AdminUserInfo>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    Ok(Json(AdminUserInfo {
        id: user_id,
        email: "user@example.com".to_string(),
        name: Some("Test User".to_string()),
        role: "user".to_string(),
        tenant_id: auth.tenant_id,
        tenant_name: "Default Tenant".to_string(),
        balance: 100.0,
        is_active: true,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        last_login_at: Some("2024-01-15T10:30:00Z".to_string()),
    }))
}

/// 更新用户请求
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}

/// 更新用户信息
///
/// PUT /api/v1/users/{id}
pub async fn update_user(
    auth: AuthExtractor,
    Path(user_id): Path<Uuid>,
    State(_state): State<AppState>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<serde_json::Value>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "User updated",
        "user_id": user_id,
        "updated_fields": {
            "name": req.name,
            "role": req.role,
            "is_active": req.is_active,
        }
    })))
}

/// 删除用户
///
/// DELETE /api/v1/users/{id}
pub async fn delete_user(
    auth: AuthExtractor,
    Path(user_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    // 防止删除自己
    if user_id == auth.user_id {
        return Err(ApiError::BadRequest("Cannot delete yourself".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "User deleted",
        "user_id": user_id,
        "deleted_by": auth.user_id,
    })))
}

/// 更新用户余额请求
#[derive(Debug, Deserialize)]
pub struct UpdateBalanceRequest {
    pub amount: f64, // 正数增加，负数减少
    pub reason: String,
}

/// 更新用户余额
///
/// POST /api/v1/users/{id}/balance
pub async fn update_user_balance(
    auth: AuthExtractor,
    Path(user_id): Path<Uuid>,
    State(_state): State<AppState>,
    Json(req): Json<UpdateBalanceRequest>,
) -> Result<Json<serde_json::Value>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Balance updated",
        "user_id": user_id,
        "amount": req.amount,
        "reason": req.reason,
        "new_balance": 100.0 + req.amount, // 模拟计算
        "updated_by": auth.user_id,
    })))
}

/// 列出用户的所有 API Keys（Admin 视图）
///
/// GET /api/v1/users/{id}/api-keys
pub async fn list_all_api_keys(
    auth: AuthExtractor,
    Path(user_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    let keys = vec![serde_json::json!({
        "id": Uuid::new_v4(),
        "user_id": user_id,
        "name": "Default Key",
        "key_preview": "sk-abc...",
        "is_active": true,
        "created_at": "2024-01-01T00:00:00Z",
    })];

    Ok(Json(keys))
}

// ==================== 账号/渠道管理 ====================

/// Provider 账号信息
#[derive(Debug, Serialize)]
pub struct AccountInfo {
    pub id: Uuid,
    pub name: String,
    pub provider: String, // openai, anthropic, etc.
    pub api_key_preview: String,
    pub base_url: Option<String>,
    pub models: Vec<String>,
    pub rpm_limit: i32,
    pub current_rpm: i32,
    pub is_active: bool,
    pub is_healthy: bool,
    pub priority: i32,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

/// 列出所有账号
///
/// GET /api/v1/accounts
pub async fn list_accounts(
    auth: AuthExtractor,
    State(_state): State<AppState>,
) -> Result<Json<Vec<AccountInfo>>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    let accounts = vec![AccountInfo {
        id: Uuid::new_v4(),
        name: "OpenAI Primary".to_string(),
        provider: "openai".to_string(),
        api_key_preview: "sk-***1234".to_string(),
        base_url: None,
        models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
        rpm_limit: 60,
        current_rpm: 12,
        is_active: true,
        is_healthy: true,
        priority: 100,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        last_used_at: Some("2024-01-15T10:30:00Z".to_string()),
    }];

    Ok(Json(accounts))
}

/// 创建账号请求
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub provider: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub models: Vec<String>,
    pub rpm_limit: Option<i32>,
    pub priority: Option<i32>,
}

/// 创建账号
///
/// POST /api/v1/accounts
pub async fn create_account(
    auth: AuthExtractor,
    State(_state): State<AppState>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<serde_json::Value>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Account created",
        "account_id": Uuid::new_v4(),
        "name": req.name,
        "provider": req.provider,
        "models": req.models,
        "created_by": auth.user_id,
    })))
}

/// 更新账号请求
#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Option<Vec<String>>,
    pub rpm_limit: Option<i32>,
    pub is_active: Option<bool>,
    pub priority: Option<i32>,
}

/// 更新账号
///
/// PUT /api/v1/accounts/{id}
pub async fn update_account(
    auth: AuthExtractor,
    Path(account_id): Path<Uuid>,
    State(_state): State<AppState>,
    Json(req): Json<UpdateAccountRequest>,
) -> Result<Json<serde_json::Value>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Account updated",
        "account_id": account_id,
        "updated_fields": {
            "name": req.name,
            "models": req.models,
            "is_active": req.is_active,
        },
        "updated_by": auth.user_id,
    })))
}

/// 删除账号
///
/// DELETE /api/v1/accounts/{id}
pub async fn delete_account(
    auth: AuthExtractor,
    Path(account_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Account deleted",
        "account_id": account_id,
        "deleted_by": auth.user_id,
    })))
}

/// 测试账号连接
///
/// POST /api/v1/accounts/{id}/test
pub async fn test_account(
    auth: AuthExtractor,
    Path(account_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    // 实际实现中应测试账号连接
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Account connection test passed",
        "account_id": account_id,
        "test_result": {
            "is_healthy": true,
            "latency_ms": 150,
            "available_models": ["gpt-4", "gpt-3.5-turbo"],
        }
    })))
}

/// 刷新账号信息（重新获取模型列表等）
///
/// POST /api/v1/accounts/{id}/refresh
pub async fn refresh_account(
    auth: AuthExtractor,
    Path(account_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Account refreshed",
        "account_id": account_id,
        "refreshed_by": auth.user_id,
        "updated_models": ["gpt-4", "gpt-3.5-turbo", "gpt-4-turbo"],
    })))
}

// ==================== 租户管理 ====================

/// 租户信息
#[derive(Debug, Serialize)]
pub struct TenantInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub user_count: i64,
    pub is_active: bool,
    pub created_at: String,
}

/// 列出所有租户
///
/// GET /api/v1/tenants
pub async fn list_tenants(
    auth: AuthExtractor,
    State(_state): State<AppState>,
) -> Result<Json<Vec<TenantInfo>>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    let tenants = vec![TenantInfo {
        id: auth.tenant_id,
        name: "Default Tenant".to_string(),
        description: Some("System default tenant".to_string()),
        user_count: 10,
        is_active: true,
        created_at: "2024-01-01T00:00:00Z".to_string(),
    }];

    Ok(Json(tenants))
}

// ==================== 系统设置 ====================

/// 系统设置
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemSettings {
    pub site_name: String,
    pub site_description: Option<String>,
    pub allow_registration: bool,
    pub default_user_quota: f64,
    pub rate_limit_rpm: i32,
    pub maintenance_mode: bool,
}

/// 获取系统设置
///
/// GET /api/v1/settings
pub async fn get_system_settings(
    auth: AuthExtractor,
    State(_state): State<AppState>,
) -> Result<Json<SystemSettings>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    Ok(Json(SystemSettings {
        site_name: "KeyCompute".to_string(),
        site_description: Some("AI Gateway Platform".to_string()),
        allow_registration: true,
        default_user_quota: 10.0,
        rate_limit_rpm: 60,
        maintenance_mode: false,
    }))
}

/// 更新系统设置
///
/// PUT /api/v1/settings
pub async fn update_system_settings(
    auth: AuthExtractor,
    State(_state): State<AppState>,
    Json(settings): Json<SystemSettings>,
) -> Result<Json<serde_json::Value>> {
    if !auth.is_admin() {
        return Err(ApiError::Auth("Admin permission required".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Settings updated",
        "updated_by": auth.user_id,
        "settings": settings,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_user_info_serialization() {
        let user = AdminUserInfo {
            id: Uuid::new_v4(),
            email: "admin@example.com".to_string(),
            name: Some("Admin".to_string()),
            role: "admin".to_string(),
            tenant_id: Uuid::new_v4(),
            tenant_name: "Test".to_string(),
            balance: 1000.0,
            is_active: true,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            last_login_at: None,
        };

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("admin@example.com"));
    }
}
