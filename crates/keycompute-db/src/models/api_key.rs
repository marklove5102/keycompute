use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// API 密钥模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub key_preview: String,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建 API 密钥请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiKeyRequest {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub key_preview: String,
    pub expires_at: Option<DateTime<Utc>>,
}

/// API 密钥响应（不包含敏感信息）
#[derive(Debug, Clone, Serialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_preview: String,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(key: ApiKey) -> Self {
        Self {
            id: key.id,
            tenant_id: key.tenant_id,
            user_id: key.user_id,
            name: key.name,
            key_preview: key.key_preview,
            revoked: key.revoked,
            revoked_at: key.revoked_at,
            expires_at: key.expires_at,
            last_used_at: key.last_used_at,
            created_at: key.created_at,
        }
    }
}

impl ApiKey {
    /// 创建新 API 密钥
    pub async fn create(
        pool: &sqlx::PgPool,
        req: &CreateApiKeyRequest,
    ) -> Result<ApiKey, sqlx::Error> {
        let key = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (tenant_id, user_id, name, key_hash, key_preview, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(&req.tenant_id)
        .bind(&req.user_id)
        .bind(&req.name)
        .bind(&req.key_hash)
        .bind(&req.key_preview)
        .bind(&req.expires_at)
        .fetch_one(pool)
        .await?;

        Ok(key)
    }

    /// 根据 ID 查找 API 密钥
    pub async fn find_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<ApiKey>, sqlx::Error> {
        let key = sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(key)
    }

    /// 根据 key_hash 查找 API 密钥
    pub async fn find_by_hash(
        pool: &sqlx::PgPool,
        key_hash: &str,
    ) -> Result<Option<ApiKey>, sqlx::Error> {
        let key = sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys WHERE key_hash = $1")
            .bind(key_hash)
            .fetch_optional(pool)
            .await?;

        Ok(key)
    }

    /// 查找用户的所有 API 密钥
    pub async fn find_by_user(
        pool: &sqlx::PgPool,
        user_id: Uuid,
    ) -> Result<Vec<ApiKey>, sqlx::Error> {
        let keys = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(keys)
    }

    /// 查找租户的所有 API 密钥
    pub async fn find_by_tenant(
        pool: &sqlx::PgPool,
        tenant_id: Uuid,
    ) -> Result<Vec<ApiKey>, sqlx::Error> {
        let keys = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE tenant_id = $1 ORDER BY created_at DESC",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await?;

        Ok(keys)
    }

    /// 撤销 API 密钥
    pub async fn revoke(&self, pool: &sqlx::PgPool) -> Result<ApiKey, sqlx::Error> {
        let key = sqlx::query_as::<_, ApiKey>(
            r#"
            UPDATE api_keys
            SET revoked = TRUE,
                revoked_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(key)
    }

    /// 更新最后使用时间
    pub async fn update_last_used(&self, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE api_keys
            SET last_used_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 检查密钥是否有效（未撤销且未过期）
    pub fn is_valid(&self) -> bool {
        if self.revoked {
            return false;
        }

        if let Some(expires_at) = self.expires_at {
            if expires_at < Utc::now() {
                return false;
            }
        }

        true
    }
}
