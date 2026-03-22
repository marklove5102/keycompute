//! JWT Token 解析与校验
//!
//! 处理 JWT Token 的生成、验证和解析。

use keycompute_types::Result;
use uuid::Uuid;

use crate::{AuthContext, Permission};

/// JWT Claims
#[derive(Debug, Clone)]
pub struct JwtClaims {
    /// 用户 ID
    pub user_id: Uuid,
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 角色
    pub role: String,
    /// 过期时间（Unix 时间戳）
    pub exp: i64,
    /// 签发时间（Unix 时间戳）
    pub iat: i64,
}

impl JwtClaims {
    /// 创建新的 Claims
    pub fn new(user_id: Uuid, tenant_id: Uuid, role: impl Into<String>, exp: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            user_id,
            tenant_id,
            role: role.into(),
            exp,
            iat: now,
        }
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.exp < now
    }
}

/// JWT 验证器
#[derive(Debug, Clone)]
pub struct JwtValidator {
    /// JWT 密钥
    secret: String,
    /// 签发者
    issuer: String,
}

impl JwtValidator {
    /// 创建新的 JWT 验证器
    pub fn new(secret: impl Into<String>, issuer: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            issuer: issuer.into(),
        }
    }

    /// 验证 JWT Token
    pub fn validate(&self, _token: &str) -> Result<AuthContext> {
        // TODO: 实现实际的 JWT 验证
        // 这里简化处理，仅解析模拟数据
        tracing::debug!("Validating JWT token");

        // 模拟验证成功
        // 实际应该使用 jsonwebtoken crate 验证签名和 claims
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let api_key_id = Uuid::new_v4();

        Ok(AuthContext {
            user_id,
            tenant_id,
            api_key_id,
            role: "user".to_string(),
            permissions: vec![Permission::UseApi, Permission::ViewUsage],
        })
    }

    /// 生成 JWT Token（简化实现）
    pub fn generate_token(&self, _claims: &JwtClaims) -> Result<String> {
        // TODO: 实现实际的 JWT 生成
        Ok("mock-jwt-token".to_string())
    }
}

impl Default for JwtValidator {
    fn default() -> Self {
        Self::new("default-secret", "keycompute")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_claims_new() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let claims = JwtClaims::new(user_id, tenant_id, "user", 1893456000);

        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.tenant_id, tenant_id);
        assert_eq!(claims.role, "user");
    }

    #[test]
    fn test_jwt_claims_expired() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        // 过期时间设为过去
        let claims = JwtClaims::new(user_id, tenant_id, "user", 0);

        assert!(claims.is_expired());
    }

    #[test]
    fn test_jwt_validator_validate() {
        let validator = JwtValidator::new("secret", "keycompute");
        let result = validator.validate("some-token");
        assert!(result.is_ok());

        let ctx = result.unwrap();
        assert!(ctx.has_permission(&Permission::UseApi));
        assert!(ctx.has_permission(&Permission::ViewUsage));
    }
}
