//! User/Tenant 加载
//!
//! 用户和租户信息的加载与管理。

use keycompute_types::Result;
use uuid::Uuid;

/// 用户信息
#[derive(Debug, Clone)]
pub struct UserInfo {
    /// 用户 ID
    pub id: Uuid,
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 邮箱
    pub email: String,
    /// 名称
    pub name: String,
    /// 角色
    pub role: String,
    /// 是否激活
    pub active: bool,
}

impl UserInfo {
    /// 创建新的用户信息
    pub fn new(
        id: Uuid,
        tenant_id: Uuid,
        email: impl Into<String>,
        name: impl Into<String>,
        role: impl Into<String>,
    ) -> Self {
        Self {
            id,
            tenant_id,
            email: email.into(),
            name: name.into(),
            role: role.into(),
            active: true,
        }
    }

    /// 检查是否是管理员
    pub fn is_admin(&self) -> bool {
        self.role == "admin" || self.role == "tenant_admin"
    }

    /// 检查是否是系统管理员
    pub fn is_system_admin(&self) -> bool {
        self.role == "system"
    }
}

/// 租户信息
#[derive(Debug, Clone)]
pub struct TenantInfo {
    /// 租户 ID
    pub id: Uuid,
    /// 租户名称
    pub name: String,
    /// 租户 slug（唯一标识）
    pub slug: String,
    /// 是否激活
    pub active: bool,
    /// 配置
    pub config: TenantConfig,
}

/// 租户配置
#[derive(Debug, Clone, Default)]
pub struct TenantConfig {
    /// 默认 RPM 限制
    pub default_rpm_limit: u32,
    /// 默认 TPM 限制
    pub default_tpm_limit: u32,
    /// 是否启用分销
    pub distribution_enabled: bool,
}

impl TenantInfo {
    /// 创建新的租户信息
    pub fn new(id: Uuid, name: impl Into<String>, slug: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            slug: slug.into(),
            active: true,
            config: TenantConfig::default(),
        }
    }

    /// 设置配置
    pub fn with_config(mut self, config: TenantConfig) -> Self {
        self.config = config;
        self
    }
}

/// 用户服务
#[derive(Debug, Clone)]
pub struct UserService {
    // TODO: 添加数据库连接
}

impl UserService {
    /// 创建新的用户服务
    pub fn new() -> Self {
        Self {}
    }

    /// 根据 ID 加载用户
    pub async fn load_user(&self, user_id: Uuid) -> Result<UserInfo> {
        // TODO: 从数据库加载用户
        tracing::debug!(user_id = %user_id, "Loading user");

        // 模拟返回
        Ok(UserInfo::new(
            user_id,
            Uuid::new_v4(),
            "user@example.com",
            "Test User",
            "user",
        ))
    }

    /// 根据 ID 加载租户
    pub async fn load_tenant(&self, tenant_id: Uuid) -> Result<TenantInfo> {
        // TODO: 从数据库加载租户
        tracing::debug!(tenant_id = %tenant_id, "Loading tenant");

        // 模拟返回
        Ok(TenantInfo::new(tenant_id, "Test Tenant", "test-tenant"))
    }

    /// 根据 API Key ID 加载用户信息
    pub async fn load_by_api_key(&self, api_key_id: Uuid) -> Result<UserInfo> {
        // TODO: 从数据库查询
        tracing::debug!(api_key_id = %api_key_id, "Loading user by API key");

        Ok(UserInfo::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "api@example.com",
            "API User",
            "user",
        ))
    }
}

impl Default for UserService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_info() {
        let user = UserInfo::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "test@example.com",
            "Test User",
            "user",
        );

        assert_eq!(user.email, "test@example.com");
        assert!(!user.is_admin());
        assert!(user.active);
    }

    #[test]
    fn test_user_info_admin() {
        let user = UserInfo::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "admin@example.com",
            "Admin User",
            "admin",
        );

        assert!(user.is_admin());
    }

    #[test]
    fn test_tenant_info() {
        let tenant = TenantInfo::new(Uuid::new_v4(), "Test", "test");
        assert_eq!(tenant.name, "Test");
        assert_eq!(tenant.slug, "test");
        assert!(tenant.active);
    }

    #[tokio::test]
    async fn test_user_service() {
        let service = UserService::new();
        let user_id = Uuid::new_v4();

        let user = service.load_user(user_id).await;
        assert!(user.is_ok());
        assert_eq!(user.unwrap().id, user_id);
    }
}
