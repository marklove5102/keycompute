//! 认证处理器
//!
//! 处理用户注册、登录、邮箱验证、密码重置等认证相关的 HTTP 请求

use crate::{
    error::{ApiError, Result},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use keycompute_auth::{
    LoginRequest, PasswordResetService, RegisterRequest, RegistrationService,
    RequestPasswordResetRequest, ResetPasswordRequest,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ============================================================================
// 请求/响应类型
// ============================================================================

/// 注册请求
#[derive(Debug, Deserialize)]
pub struct RegisterRequestJson {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
    pub tenant_slug: Option<String>,
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequestJson {
    pub email: String,
    pub password: String,
}

/// 忘记密码请求
#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequestJson {
    pub email: String,
}

/// 重置密码请求
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequestJson {
    pub token: String,
    pub new_password: String,
}

/// 通用消息响应
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

/// 验证令牌响应
#[derive(Debug, Serialize)]
pub struct VerifyTokenResponse {
    pub valid: bool,
    pub user_id: Option<String>,
}

// ============================================================================
// 处理器函数
// ============================================================================

/// 用户注册
///
/// POST /auth/register
pub async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequestJson>,
) -> Result<impl IntoResponse> {
    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Database not configured".into()))?;

    let register_req = RegisterRequest {
        email: req.email,
        password: req.password,
        name: req.name,
        tenant_slug: req.tenant_slug,
    };

    let service = RegistrationService::new(Arc::clone(pool))
        .with_email_service((*state.email_service).clone());
    let response = service
        .register(&register_req)
        .await
        .map_err(|e| ApiError::Auth(format!("Registration failed: {}", e)))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "user_id": response.user_id.to_string(),
            "tenant_id": response.tenant_id.to_string(),
            "email": response.email,
            "message": response.message
        })),
    ))
}

/// 用户登录
///
/// POST /auth/login
pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequestJson>,
) -> Result<impl IntoResponse> {
    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Database not configured".into()))?;

    let jwt_validator = state
        .auth
        .get_jwt_validator()
        .ok_or_else(|| ApiError::Internal("JWT not configured".into()))?
        .clone();

    let login_req = LoginRequest {
        email: req.email,
        password: req.password,
        client_ip: None, // TODO: 从请求中提取
    };

    let service = keycompute_auth::LoginService::new(Arc::clone(pool), jwt_validator);
    let response = service.login(&login_req).await.map_err(|e| match e {
        keycompute_types::KeyComputeError::AuthError(msg) => ApiError::Auth(msg),
        _ => ApiError::Internal(format!("Login failed: {}", e)),
    })?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "user_id": response.user_id.to_string(),
            "tenant_id": response.tenant_id.to_string(),
            "email": response.email,
            "role": response.role,
            "access_token": response.jwt_token,
            "token_type": "Bearer",
            "expires_in": response.expires_in
        })),
    ))
}

/// 邮箱验证
///
/// GET /auth/verify-email/{token}
pub async fn verify_email_handler(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse> {
    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Database not configured".into()))?;

    let service = RegistrationService::new(Arc::clone(pool))
        .with_email_service((*state.email_service).clone());
    let user_id = service
        .verify_email(&token)
        .await
        .map_err(|e| ApiError::Auth(format!("Email verification failed: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Email verified successfully",
            "user_id": user_id.to_string()
        })),
    ))
}

/// 忘记密码
///
/// POST /auth/forgot-password
pub async fn forgot_password_handler(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequestJson>,
) -> Result<impl IntoResponse> {
    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Database not configured".into()))?;

    let service = PasswordResetService::new(Arc::clone(pool))
        .with_email_service((*state.email_service).clone());

    // 无论邮箱是否存在都返回成功（防止邮箱枚举攻击）
    service
        .request_reset(&RequestPasswordResetRequest {
            email: req.email,
            client_ip: None, // TODO: 从请求中提取
        })
        .await
        .map_err(|e| ApiError::Internal(format!("Password reset request failed: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "If the email exists, a reset link has been sent.".to_string(),
        }),
    ))
}

/// 重置密码
///
/// POST /auth/reset-password
pub async fn reset_password_handler(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequestJson>,
) -> Result<impl IntoResponse> {
    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Database not configured".into()))?;

    let service = PasswordResetService::new(Arc::clone(pool));

    let reset_req = ResetPasswordRequest {
        token: req.token,
        new_password: req.new_password,
    };

    let user_id = service
        .reset_password(&reset_req)
        .await
        .map_err(|e| match e {
            keycompute_types::KeyComputeError::AuthError(msg) => ApiError::Auth(msg),
            keycompute_types::KeyComputeError::ValidationError(msg) => ApiError::BadRequest(msg),
            _ => ApiError::Internal(format!("Password reset failed: {}", e)),
        })?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Password reset successfully",
            "user_id": user_id.to_string()
        })),
    ))
}

/// 验证重置令牌
///
/// GET /auth/verify-reset-token/:token
pub async fn verify_reset_token_handler(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse> {
    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Database not configured".into()))?;

    let service = PasswordResetService::new(Arc::clone(pool));
    let valid = service
        .verify_token(&token)
        .await
        .map_err(|e| ApiError::Internal(format!("Token verification failed: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(VerifyTokenResponse {
            valid,
            user_id: None,
        }),
    ))
}

/// 刷新 Token
///
/// POST /auth/refresh-token
pub async fn refresh_token_handler(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequestJson>,
) -> Result<impl IntoResponse> {
    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Database not configured".into()))?;

    let jwt_validator = state
        .auth
        .get_jwt_validator()
        .ok_or_else(|| ApiError::Internal("JWT not configured".into()))?
        .clone();

    let service = keycompute_auth::LoginService::new(Arc::clone(pool), jwt_validator);
    let response = service
        .refresh_token(&req.token)
        .await
        .map_err(|e| ApiError::Auth(format!("Token refresh failed: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "user_id": response.user_id.to_string(),
            "tenant_id": response.tenant_id.to_string(),
            "email": response.email,
            "role": response.role,
            "access_token": response.jwt_token,
            "token_type": "Bearer",
            "expires_in": response.expires_in
        })),
    ))
}

/// 刷新 Token 请求
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequestJson {
    pub token: String,
}

/// 重新发送验证邮件
///
/// POST /auth/resend-verification
pub async fn resend_verification_handler(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequestJson>,
) -> Result<impl IntoResponse> {
    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Database not configured".into()))?;

    let service = RegistrationService::new(Arc::clone(pool))
        .with_email_service((*state.email_service).clone());
    service
        .resend_verification(&req.email)
        .await
        .map_err(|e| match e {
            keycompute_types::KeyComputeError::AuthError(msg) => ApiError::Auth(msg),
            _ => ApiError::Internal(format!("Resend verification failed: {}", e)),
        })?;

    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message:
                "If the email exists and is not verified, a new verification email has been sent."
                    .to_string(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_json() {
        let json = RegisterRequestJson {
            email: "test@example.com".to_string(),
            password: "SecurePass123!".to_string(),
            name: Some("Test User".to_string()),
            tenant_slug: None,
        };

        assert_eq!(json.email, "test@example.com");
    }

    #[test]
    fn test_login_request_json() {
        let json = LoginRequestJson {
            email: "test@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };

        assert_eq!(json.email, "test@example.com");
    }

    #[test]
    fn test_message_response() {
        let resp = MessageResponse {
            message: "Success".to_string(),
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("Success"));
    }
}
