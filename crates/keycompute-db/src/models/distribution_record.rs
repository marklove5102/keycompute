use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 分销记录模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DistributionRecord {
    pub id: Uuid,
    pub usage_log_id: Uuid,
    pub tenant_id: Uuid,
    pub beneficiary_id: Uuid,
    pub share_amount: BigDecimal,
    pub share_ratio: BigDecimal,
    pub status: String,
    pub settled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 创建分销记录请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDistributionRecordRequest {
    pub usage_log_id: Uuid,
    pub tenant_id: Uuid,
    pub beneficiary_id: Uuid,
    pub share_amount: BigDecimal,
    pub share_ratio: BigDecimal,
}

/// 分销统计
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DistributionStats {
    pub total_records: i64,
    pub total_amount: BigDecimal,
    pub settled_amount: BigDecimal,
    pub pending_amount: BigDecimal,
}

impl DistributionRecord {
    /// 创建分销记录
    pub async fn create(
        pool: &sqlx::PgPool,
        req: &CreateDistributionRecordRequest,
    ) -> Result<DistributionRecord, sqlx::Error> {
        let record = sqlx::query_as::<_, DistributionRecord>(
            r#"
            INSERT INTO distribution_records (
                usage_log_id, tenant_id, beneficiary_id,
                share_amount, share_ratio, status
            )
            VALUES ($1, $2, $3, $4, $5, 'pending')
            RETURNING *
            "#,
        )
        .bind(&req.usage_log_id)
        .bind(&req.tenant_id)
        .bind(&req.beneficiary_id)
        .bind(&req.share_amount)
        .bind(&req.share_ratio)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// 批量创建分销记录
    pub async fn create_many(
        pool: &sqlx::PgPool,
        requests: &[CreateDistributionRecordRequest],
    ) -> Result<Vec<DistributionRecord>, sqlx::Error> {
        let mut records = Vec::with_capacity(requests.len());

        for req in requests {
            let record = Self::create(pool, req).await?;
            records.push(record);
        }

        Ok(records)
    }

    /// 根据 ID 查找分销记录
    pub async fn find_by_id(
        pool: &sqlx::PgPool,
        id: Uuid,
    ) -> Result<Option<DistributionRecord>, sqlx::Error> {
        let record = sqlx::query_as::<_, DistributionRecord>(
            "SELECT * FROM distribution_records WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// 查找用量日志的所有分销记录
    pub async fn find_by_usage_log(
        pool: &sqlx::PgPool,
        usage_log_id: Uuid,
    ) -> Result<Vec<DistributionRecord>, sqlx::Error> {
        let records = sqlx::query_as::<_, DistributionRecord>(
            "SELECT * FROM distribution_records WHERE usage_log_id = $1",
        )
        .bind(usage_log_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// 查找租户的分销记录
    pub async fn find_by_tenant(
        pool: &sqlx::PgPool,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DistributionRecord>, sqlx::Error> {
        let records = sqlx::query_as::<_, DistributionRecord>(
            r#"
            SELECT * FROM distribution_records
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// 查找受益人的分销记录
    pub async fn find_by_beneficiary(
        pool: &sqlx::PgPool,
        beneficiary_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DistributionRecord>, sqlx::Error> {
        let records = sqlx::query_as::<_, DistributionRecord>(
            r#"
            SELECT * FROM distribution_records
            WHERE beneficiary_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(beneficiary_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// 结算分销记录
    pub async fn settle(&self, pool: &sqlx::PgPool) -> Result<DistributionRecord, sqlx::Error> {
        let record = sqlx::query_as::<_, DistributionRecord>(
            r#"
            UPDATE distribution_records
            SET status = 'settled',
                settled_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// 获取受益人统计
    pub async fn get_stats_by_beneficiary(
        pool: &sqlx::PgPool,
        beneficiary_id: Uuid,
    ) -> Result<DistributionStats, sqlx::Error> {
        let stats = sqlx::query_as::<_, DistributionStats>(
            r#"
            SELECT
                COUNT(*) as total_records,
                COALESCE(SUM(share_amount), 0) as total_amount,
                COALESCE(SUM(CASE WHEN status = 'settled' THEN share_amount ELSE 0 END), 0) as settled_amount,
                COALESCE(SUM(CASE WHEN status = 'pending' THEN share_amount ELSE 0 END), 0) as pending_amount
            FROM distribution_records
            WHERE beneficiary_id = $1
            "#,
        )
        .bind(beneficiary_id)
        .fetch_one(pool)
        .await?;

        Ok(stats)
    }
}
