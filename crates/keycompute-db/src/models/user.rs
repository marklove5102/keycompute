use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 用户模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建用户请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserRequest {
    pub tenant_id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: Option<String>,
}

/// 更新用户请求
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub role: Option<String>,
}

impl User {
    /// 创建新用户
    pub async fn create(pool: &sqlx::PgPool, req: &CreateUserRequest) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (tenant_id, email, name, role)
            VALUES ($1, $2, $3, COALESCE($4, 'user'))
            RETURNING *
            "#,
        )
        .bind(&req.tenant_id)
        .bind(&req.email)
        .bind(&req.name)
        .bind(&req.role)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// 根据 ID 查找用户
    pub async fn find_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    /// 根据邮箱查找用户
    pub async fn find_by_email(
        pool: &sqlx::PgPool,
        email: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    /// 查找租户下的所有用户
    pub async fn find_by_tenant(
        pool: &sqlx::PgPool,
        tenant_id: Uuid,
    ) -> Result<Vec<User>, sqlx::Error> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_all(pool)
            .await?;

        Ok(users)
    }

    /// 更新用户
    pub async fn update(
        &self,
        pool: &sqlx::PgPool,
        req: &UpdateUserRequest,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET name = COALESCE($1, name),
                role = COALESCE($2, role),
                updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(&req.role)
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// 删除用户
    pub async fn delete(&self, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(self.id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
