//! KeyCompute 数据库访问层
//!
//! 提供 PostgreSQL 数据库连接池、ORM 模型和迁移支持

pub mod models;
pub mod schema;

use once_cell::sync::OnceCell;
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use std::time::Duration;

pub use models::*;
pub use schema::*;

/// 数据库连接池全局实例
static DB_POOL: OnceCell<PgPool> = OnceCell::new();

/// 数据库错误类型
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database connection failed: {0}")]
    ConnectionError(String),
    #[error("migration failed: {0}")]
    MigrationError(String),
    #[error("pool not initialized")]
    PoolNotInitialized,
}

/// 数据库配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// 数据库连接 URL
    pub url: String,
    /// 最大连接数
    pub max_connections: u32,
    /// 最小连接数
    pub min_connections: u32,
    /// 连接超时时间（秒）
    pub connect_timeout: u64,
    /// 连接空闲超时时间（秒）
    pub idle_timeout: u64,
    /// 连接最大生命周期（秒）
    pub max_lifetime: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/keycompute".to_string()),
            max_connections: 10,
            min_connections: 2,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 1800,
        }
    }
}

/// 初始化数据库连接池
///
/// # Examples
///
/// ```rust,no_run
/// use keycompute_db::{init_pool, DatabaseConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = DatabaseConfig::default();
///     let pool = init_pool(&config).await?;
///     Ok(())
/// }
/// ```
pub async fn init_pool(config: &DatabaseConfig) -> Result<PgPool, DbError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout))
        .idle_timeout(Duration::from_secs(config.idle_timeout))
        .max_lifetime(Duration::from_secs(config.max_lifetime))
        .connect(&config.url)
        .await
        .map_err(|e| DbError::ConnectionError(e.to_string()))?;

    // 存储到全局
    DB_POOL
        .set(pool.clone())
        .map_err(|_| DbError::PoolNotInitialized)?;

    tracing::info!("Database pool initialized successfully");

    Ok(pool)
}

/// 获取数据库连接池
///
/// 必须先调用 `init_pool` 初始化
pub fn get_pool() -> Result<PgPool, DbError> {
    DB_POOL.get().cloned().ok_or(DbError::PoolNotInitialized)
}

/// 运行数据库迁移
///
/// 使用 sqlx 的嵌入式迁移
pub async fn run_migrations(pool: &PgPool) -> Result<(), DbError> {
    // 嵌入式迁移文件
    static MIGRATOR: Migrator = sqlx::migrate!("src/migrations");

    MIGRATOR
        .run(pool)
        .await
        .map_err(|e| DbError::MigrationError(e.to_string()))?;

    tracing::info!("Database migrations completed successfully");

    Ok(())
}

/// 数据库连接管理器
#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    /// 创建新的数据库管理器
    pub async fn new(config: &DatabaseConfig) -> Result<Self, DbError> {
        let pool = init_pool(config).await?;
        Ok(Self { pool })
    }

    /// 从环境变量创建数据库管理器
    pub async fn from_env() -> Result<Self, DbError> {
        let config = DatabaseConfig::default();
        Self::new(&config).await
    }

    /// 获取连接池
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 运行迁移
    pub async fn migrate(&self) -> Result<(), DbError> {
        run_migrations(&self.pool).await
    }

    /// 测试连接
    pub async fn test_connection(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }
}

/// 重新导出 sqlx 类型
pub use sqlx::{PgPool as SqlxPgPool, Row, Transaction};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
    }
}
