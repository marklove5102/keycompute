//! Pricing Module
//!
//! 定价模块，只读，生成 PricingSnapshot。
//! 架构约束：不写任何状态，不参与路由或执行。

use keycompute_db::PricingModel;
use keycompute_types::{KeyComputeError, PricingSnapshot, Result};
use lru::LruCache;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 标记价格来源，用于优化缓存策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PricingSource {
    /// 租户特定定价
    TenantSpecific,
    /// 数据库默认定价
    DatabaseDefault,
    /// 硬编码默认定价（兜底）
    HardcodedDefault,
}

/// 带来源标记的价格快照
#[derive(Debug, Clone)]
struct SnapshotWithSource {
    snapshot: PricingSnapshot,
    source: PricingSource,
    /// 实际匹配到的 provider（用于缓存键）
    /// 当回退匹配时，可能与请求的 provider 不同
    matched_provider: String,
}

/// 默认缓存 TTL（5 分钟）
const DEFAULT_CACHE_TTL_SECS: u64 = 300;

/// 默认缓存容量
const DEFAULT_CACHE_CAPACITY: usize = 10000;

/// 缓存条目
#[derive(Clone)]
struct CacheEntry {
    /// 价格快照
    snapshot: PricingSnapshot,
    /// 创建时间（用于 TTL 检查）
    created_at: Instant,
}

impl CacheEntry {
    fn new(snapshot: PricingSnapshot) -> Self {
        Self {
            snapshot,
            created_at: Instant::now(),
        }
    }

    /// 检查是否过期
    fn is_expired(&self, ttl_secs: u64) -> bool {
        // TTL 为 0 表示立即过期
        if ttl_secs == 0 {
            return true;
        }
        self.created_at.elapsed().as_secs() > ttl_secs
    }
}

/// 定价服务
///
/// 负责从数据库加载模型价格，生成 PricingSnapshot
#[derive(Clone)]
pub struct PricingService {
    /// 数据库连接池（可选，用于测试时可以不提供）
    pool: Option<Arc<PgPool>>,
    /// 价格缓存：key = "tenant_id:model_name:provider"，使用 LRU 淘汰策略
    cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    /// 缓存 TTL（秒）
    cache_ttl_secs: u64,
    /// 缓存容量
    cache_capacity: usize,
}

impl std::fmt::Debug for PricingService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PricingService")
            .field("pool", &self.pool.as_ref().map(|_| "PgPool"))
            .field("cache", &"LruCache")
            .field("cache_ttl_secs", &self.cache_ttl_secs)
            .field("cache_capacity", &self.cache_capacity)
            .finish()
    }
}

impl Default for PricingService {
    fn default() -> Self {
        Self::new()
    }
}

impl PricingService {
    /// 创建新的定价服务（无数据库连接，使用默认价格）
    pub fn new() -> Self {
        Self {
            pool: None,
            cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(DEFAULT_CACHE_CAPACITY).unwrap(),
            ))),
            cache_ttl_secs: DEFAULT_CACHE_TTL_SECS,
            cache_capacity: DEFAULT_CACHE_CAPACITY,
        }
    }

    /// 创建带数据库连接的定价服务
    pub fn with_pool(pool: Arc<PgPool>) -> Self {
        Self {
            pool: Some(pool),
            cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(DEFAULT_CACHE_CAPACITY).unwrap(),
            ))),
            cache_ttl_secs: DEFAULT_CACHE_TTL_SECS,
            cache_capacity: DEFAULT_CACHE_CAPACITY,
        }
    }

    /// 设置缓存 TTL
    pub fn with_cache_ttl(mut self, ttl_secs: u64) -> Self {
        self.cache_ttl_secs = ttl_secs;
        self
    }

    /// 设置缓存容量
    pub fn with_cache_capacity(mut self, capacity: usize) -> Self {
        if capacity > 0 {
            self.cache_capacity = capacity;
            self.cache = Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap(),
            )));
        }
        self
    }

    /// 生成缓存 key
    fn cache_key(tenant_id: &Uuid, model_name: &str, provider: &str) -> String {
        format!("{}:{}:{}", tenant_id, model_name, provider)
    }

    /// 创建价格快照（固化到 RequestContext）
    ///
    /// 从数据库或缓存加载指定模型的价格
    ///
    /// # 参数
    /// - `model_name`: 模型名称
    /// - `tenant_id`: 租户 ID
    /// - `provider`: Provider 名称（可选，默认 "openai"）
    ///
    /// # 缓存策略
    /// 采用多级缓存 Key 查找策略：
    /// 1. `tenant_id:model:provider` - 租户特定定价
    /// 2. `nil:model:provider` - 系统默认定价（按 provider）
    /// 3. 兜底到硬编码默认价格
    pub async fn create_snapshot(
        &self,
        model_name: &str,
        tenant_id: &Uuid,
        provider: Option<&str>,
    ) -> Result<PricingSnapshot> {
        let provider = provider.unwrap_or("openai");
        let nil_tenant = Uuid::nil();

        // 构建多级缓存 key（优先级从高到低）
        let cache_keys = [
            Self::cache_key(tenant_id, model_name, provider),
            Self::cache_key(&nil_tenant, model_name, provider),
        ];

        // 按优先级检查缓存（使用写锁，因为 LruCache::get 需要更新访问顺序）
        {
            let mut cache = self.cache.write().await;
            for key in &cache_keys {
                if let Some(entry) = cache.get(key)
                    && !entry.is_expired(self.cache_ttl_secs)
                {
                    tracing::debug!(
                        model = %model_name,
                        tenant_id = %tenant_id,
                        provider = %provider,
                        cache_key = %key,
                        "Pricing snapshot from cache"
                    );
                    return Ok(entry.snapshot.clone());
                }
            }
        }

        // 尝试从数据库加载
        let snapshot_with_source = if let Some(pool) = &self.pool {
            self.load_from_database_with_source(pool, model_name, tenant_id, provider)
                .await?
        } else {
            // 无数据库连接时使用默认价格
            SnapshotWithSource {
                snapshot: self.get_default_pricing(model_name),
                source: PricingSource::HardcodedDefault,
                matched_provider: provider.to_string(),
            }
        };

        // 智能缓存策略：根据来源决定缓存方式
        {
            let mut cache = self.cache.write().await;
            let snapshot = snapshot_with_source.snapshot.clone();

            // 使用实际匹配到的 provider 构建缓存键
            let matched_cache_key = Self::cache_key(
                &nil_tenant,
                model_name,
                &snapshot_with_source.matched_provider,
            );

            // 请求的 provider 的缓存键（可能与 matched_provider 不同）
            let requested_cache_key = cache_keys[1].clone();

            match snapshot_with_source.source {
                PricingSource::TenantSpecific => {
                    // 租户特定定价：缓存到租户 key 和默认 key
                    let primary_key = cache_keys[0].clone();
                    cache.put(primary_key, CacheEntry::new(snapshot.clone()));
                    // 同时缓存到默认 key，避免重复查询
                    cache.put(matched_cache_key, CacheEntry::new(snapshot));
                }
                PricingSource::DatabaseDefault => {
                    // 数据库默认定价：缓存到匹配到的 provider key
                    cache.put(matched_cache_key, CacheEntry::new(snapshot.clone()));
                    // 如果回退匹配（matched_provider != 请求的 provider），
                    // 还需要缓存到请求的 provider key，避免后续相同请求重复查询
                    if snapshot_with_source.matched_provider != provider {
                        cache.put(requested_cache_key, CacheEntry::new(snapshot));
                    }
                }
                PricingSource::HardcodedDefault => {
                    // 硬编码默认定价：缓存到请求的 provider key
                    cache.put(requested_cache_key, CacheEntry::new(snapshot));
                }
            }
        }

        tracing::debug!(
            model = %model_name,
            tenant_id = %tenant_id,
            provider = %provider,
            source = ?snapshot_with_source.source,
            price = ?snapshot_with_source.snapshot,
            "Created pricing snapshot"
        );
        Ok(snapshot_with_source.snapshot)
    }

    /// 更新 RequestContext 的定价快照（路由后调用）
    ///
    /// 当路由确定的 provider 与初始 provider 不同时，
    /// 重新获取定价并更新 RequestContext
    ///
    /// # 参数
    /// - `ctx`: 可变引用的 RequestContext
    /// - `actual_provider`: 路由确定的实际 provider
    ///
    /// # 返回
    /// - `true`: 定价已更新
    /// - `false`: 定价未变化（provider 相同或获取失败）
    pub async fn update_context_pricing(
        &self,
        ctx: &mut keycompute_types::RequestContext,
        actual_provider: &str,
    ) -> bool {
        let current_provider = ctx.provider.as_deref().unwrap_or("openai");

        // 如果 provider 相同，只需设置 provider 字段（如果尚未设置）
        if current_provider == actual_provider {
            if ctx.provider.is_none() {
                ctx.set_provider(actual_provider);
            }
            return false;
        }

        // 获取新 provider 的定价
        match self
            .create_snapshot(&ctx.model, &ctx.tenant_id, Some(actual_provider))
            .await
        {
            Ok(new_pricing) => {
                tracing::debug!(
                    request_id = %ctx.request_id,
                    model = %ctx.model,
                    old_provider = %current_provider,
                    new_provider = %actual_provider,
                    "Updated pricing for different provider"
                );
                ctx.set_provider(actual_provider);
                ctx.update_pricing(new_pricing);
                true
            }
            Err(e) => {
                tracing::warn!(
                    request_id = %ctx.request_id,
                    model = %ctx.model,
                    provider = %actual_provider,
                    error = %e,
                    "Failed to update pricing for provider, keeping original"
                );
                false
            }
        }
    }

    /// 从数据库加载价格（带来源标记）
    async fn load_from_database_with_source(
        &self,
        pool: &PgPool,
        model_name: &str,
        tenant_id: &Uuid,
        provider: &str,
    ) -> Result<SnapshotWithSource> {
        // 尝试按租户+模型名+provider查找
        let pricing = PricingModel::find_by_model(pool, *tenant_id, model_name, provider)
            .await
            .map_err(|e| {
                KeyComputeError::DatabaseError(format!("Failed to load pricing: {}", e))
            })?;

        if let Some(p) = pricing {
            return Ok(SnapshotWithSource {
                snapshot: PricingSnapshot {
                    model_name: p.model_name,
                    currency: p.currency,
                    input_price_per_1k: bigdecimal_to_decimal(&p.input_price_per_1k)?,
                    output_price_per_1k: bigdecimal_to_decimal(&p.output_price_per_1k)?,
                },
                source: PricingSource::TenantSpecific,
                matched_provider: provider.to_string(),
            });
        }

        // 尝试查找默认定价（优先匹配 model_name + provider）
        let defaults = PricingModel::find_defaults(pool).await.map_err(|e| {
            KeyComputeError::DatabaseError(format!("Failed to load default pricing: {}", e))
        })?;

        // 先尝试精确匹配 model_name + provider
        for p in &defaults {
            if p.model_name == model_name && p.provider == provider {
                return Ok(SnapshotWithSource {
                    snapshot: PricingSnapshot {
                        model_name: p.model_name.clone(),
                        currency: p.currency.clone(),
                        input_price_per_1k: bigdecimal_to_decimal(&p.input_price_per_1k)?,
                        output_price_per_1k: bigdecimal_to_decimal(&p.output_price_per_1k)?,
                    },
                    source: PricingSource::DatabaseDefault,
                    matched_provider: provider.to_string(),
                });
            }
        }

        // 如果找不到匹配的 provider，尝试只匹配 model_name（任意 provider）
        for p in defaults {
            if p.model_name == model_name {
                tracing::debug!(
                    model = %model_name,
                    requested_provider = %provider,
                    matched_provider = %p.provider,
                    "Using default pricing from different provider"
                );
                return Ok(SnapshotWithSource {
                    snapshot: PricingSnapshot {
                        model_name: p.model_name.clone(),
                        currency: p.currency.clone(),
                        input_price_per_1k: bigdecimal_to_decimal(&p.input_price_per_1k)?,
                        output_price_per_1k: bigdecimal_to_decimal(&p.output_price_per_1k)?,
                    },
                    source: PricingSource::DatabaseDefault,
                    // 使用匹配到的 provider 作为缓存键
                    matched_provider: p.provider.clone(),
                });
            }
        }

        // 未找到，使用硬编码默认价格
        tracing::warn!(
            model = %model_name,
            tenant_id = %tenant_id,
            provider = %provider,
            "No pricing found in database, using hardcoded default"
        );
        Ok(SnapshotWithSource {
            snapshot: self.get_default_pricing(model_name),
            source: PricingSource::HardcodedDefault,
            // 硬编码默认价格使用请求的 provider
            matched_provider: provider.to_string(),
        })
    }

    /// 获取默认定价
    ///
    /// 统一使用默认价格，不再区分模型
    fn get_default_pricing(&self, model_name: &str) -> PricingSnapshot {
        PricingSnapshot {
            model_name: model_name.to_string(),
            currency: "CNY".to_string(),
            // 统一默认价格：输入 0.1 元/1k tokens，输出 0.3 元/1k tokens
            input_price_per_1k: Decimal::from(100) / Decimal::from(1000),
            output_price_per_1k: Decimal::from(300) / Decimal::from(1000),
        }
    }

    /// 清除缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        tracing::info!("Pricing cache cleared");
    }

    /// 清除过期缓存条目
    ///
    /// 由于 LruCache 不支持 retain，需要手动收集过期 key 后删除
    pub async fn clear_expired(&self) {
        let mut cache = self.cache.write().await;
        let before_len = cache.len();

        // 收集过期的 key
        let expired_keys: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| entry.is_expired(self.cache_ttl_secs))
            .map(|(key, _)| key.clone())
            .collect();

        // 删除过期条目
        for key in expired_keys {
            cache.pop(&key);
        }

        let after_len = cache.len();
        if before_len != after_len {
            tracing::info!(
                removed = before_len - after_len,
                remaining = after_len,
                "Expired cache entries cleared"
            );
        }
    }

    /// 预热缓存（从数据库加载所有默认定价）
    ///
    /// 使用 nil UUID 作为租户 ID，适用于默认定价场景
    pub async fn warmup_cache(&self) -> Result<()> {
        let Some(pool) = &self.pool else {
            return Ok(());
        };

        let defaults = PricingModel::find_defaults(pool).await.map_err(|e| {
            KeyComputeError::DatabaseError(format!("Failed to load default pricing: {}", e))
        })?;

        let nil_tenant = Uuid::nil();
        let mut cache = self.cache.write().await;
        for p in defaults {
            let snapshot = PricingSnapshot {
                model_name: p.model_name.clone(),
                currency: p.currency.clone(),
                input_price_per_1k: bigdecimal_to_decimal(&p.input_price_per_1k)?,
                output_price_per_1k: bigdecimal_to_decimal(&p.output_price_per_1k)?,
            };
            // 使用 nil tenant_id 和 provider 作为缓存 key
            let key = Self::cache_key(&nil_tenant, &p.model_name, &p.provider);
            cache.put(key, CacheEntry::new(snapshot));
        }

        tracing::info!(count = cache.len(), "Pricing cache warmed up");
        Ok(())
    }

    /// 计算请求费用
    pub fn calculate_cost(
        &self,
        input_tokens: u32,
        output_tokens: u32,
        pricing: &PricingSnapshot,
    ) -> Decimal {
        let input_cost =
            Decimal::from(input_tokens) * pricing.input_price_per_1k / Decimal::from(1000);
        let output_cost =
            Decimal::from(output_tokens) * pricing.output_price_per_1k / Decimal::from(1000);
        input_cost + output_cost
    }

    /// 检查是否已配置数据库连接
    ///
    /// 用于启动时验证配置
    pub fn has_pool(&self) -> bool {
        self.pool.is_some()
    }
}

/// 将 BigDecimal 转换为 Decimal（精确转换）
///
/// 对于价格数据，使用字符串中间格式足够精确，因为：
/// 1. 价格通常只有 2-6 位小数
/// 2. BigDecimal 和 Decimal 都支持任意精度
/// 3. 避免手动处理 BigInt 导致的溢出问题
///
/// # 返回
/// - `Ok(Decimal)`: 转换成功
/// - `Err(KeyComputeError)`: 转换失败，返回错误信息
fn bigdecimal_to_decimal(value: &bigdecimal::BigDecimal) -> Result<Decimal> {
    let s = value.to_string();
    s.parse::<Decimal>().map_err(|e| {
        tracing::error!(value = %s, error = %e, "Failed to convert BigDecimal to Decimal");
        KeyComputeError::Internal(format!(
            "Failed to convert BigDecimal '{}' to Decimal: {}",
            s, e
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    /// 测试缓存 key 生成
    #[test]
    fn test_cache_key_generation() {
        let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let key = PricingService::cache_key(&tenant_id, "gpt-4o", "openai");

        assert!(key.contains("gpt-4o"));
        assert!(key.contains("openai"));
        assert!(key.contains("00000000-0000-0000-0000-000000000001"));
    }

    /// 测试 nil tenant 的缓存 key
    #[test]
    fn test_cache_key_nil_tenant() {
        let nil_tenant = Uuid::nil();
        let key = PricingService::cache_key(&nil_tenant, "gpt-4o", "openai");

        assert!(key.starts_with("00000000-0000-0000-0000-000000000000"));
    }

    /// 测试成本计算
    #[test]
    fn test_calculate_cost() {
        let service = PricingService::new();
        let snapshot = PricingSnapshot {
            model_name: "test-model".to_string(),
            currency: "CNY".to_string(),
            input_price_per_1k: Decimal::from(100), // 0.1 元/token
            output_price_per_1k: Decimal::from(200), // 0.2 元/token
        };

        // 1000 input tokens + 500 output tokens
        // input_cost = 1000 * 100 / 1000 = 100
        // output_cost = 500 * 200 / 1000 = 100
        // total = 200
        let cost = service.calculate_cost(1000, 500, &snapshot);
        assert_eq!(cost, Decimal::from(200));
    }

    /// 测试成本计算 - 边界情况
    #[test]
    fn test_calculate_cost_zero_tokens() {
        let service = PricingService::new();
        let snapshot = PricingSnapshot {
            model_name: "test-model".to_string(),
            currency: "CNY".to_string(),
            input_price_per_1k: Decimal::from(100),
            output_price_per_1k: Decimal::from(200),
        };

        let cost = service.calculate_cost(0, 0, &snapshot);
        assert_eq!(cost, Decimal::ZERO);
    }

    /// 测试默认定价获取 - 统一使用相同默认价格
    #[test]
    fn test_get_default_pricing() {
        let service = PricingService::new();

        // 测试不同模型都返回相同的默认价格
        let models = [
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "gpt-3.5-turbo",
            "unknown-model",
        ];

        for model in models {
            let snapshot = service.get_default_pricing(model);

            assert_eq!(snapshot.model_name, model);
            assert_eq!(snapshot.currency, "CNY");
            // 统一默认价格: input = 0.1, output = 0.3
            assert_eq!(
                snapshot.input_price_per_1k,
                Decimal::from_str("0.1").unwrap()
            );
            assert_eq!(
                snapshot.output_price_per_1k,
                Decimal::from_str("0.3").unwrap()
            );
        }
    }

    /// 测试 BigDecimal 到 Decimal 转换 - 简单小数
    #[test]
    fn test_bigdecimal_to_decimal_simple() {
        let bd = BigDecimal::from_str("0.5").unwrap();
        let d = bigdecimal_to_decimal(&bd).unwrap();
        assert_eq!(d, Decimal::from_str("0.5").unwrap());
    }

    /// 测试 BigDecimal 到 Decimal 转换 - 整数
    #[test]
    fn test_bigdecimal_to_decimal_integer() {
        let bd = BigDecimal::from_str("100").unwrap();
        let d = bigdecimal_to_decimal(&bd).unwrap();
        assert_eq!(d, Decimal::from(100));
    }

    /// 测试 BigDecimal 到 Decimal 转换 - 多位小数
    #[test]
    fn test_bigdecimal_to_decimal_precision() {
        let bd = BigDecimal::from_str("0.123456789").unwrap();
        let d = bigdecimal_to_decimal(&bd).unwrap();
        assert_eq!(d, Decimal::from_str("0.123456789").unwrap());
    }

    /// 测试 BigDecimal 到 Decimal 转换 - 零
    #[test]
    fn test_bigdecimal_to_decimal_zero() {
        let bd = BigDecimal::from_str("0").unwrap();
        let d = bigdecimal_to_decimal(&bd).unwrap();
        assert_eq!(d, Decimal::ZERO);
    }

    /// 测试 BigDecimal 到 Decimal 转换 - 大数
    #[test]
    fn test_bigdecimal_to_decimal_large() {
        let bd = BigDecimal::from_str("12345.67").unwrap();
        let d = bigdecimal_to_decimal(&bd).unwrap();
        assert_eq!(d, Decimal::from_str("12345.67").unwrap());
    }

    /// 测试缓存条目过期检查
    #[test]
    fn test_cache_entry_expiry() {
        let snapshot = PricingSnapshot {
            model_name: "test".to_string(),
            currency: "CNY".to_string(),
            input_price_per_1k: Decimal::ONE,
            output_price_per_1k: Decimal::ONE,
        };
        let entry = CacheEntry::new(snapshot);

        // 新创建的条目不应过期
        assert!(!entry.is_expired(300));

        // TTL 为 0 时应立即过期
        assert!(entry.is_expired(0));
    }

    /// 测试 PricingService 创建
    #[test]
    fn test_pricing_service_new() {
        let service = PricingService::new();
        assert!(!service.has_pool());
    }

    /// 测试 PricingService 配置链式调用
    #[test]
    fn test_pricing_service_with_cache_ttl() {
        let service = PricingService::new().with_cache_ttl(600);
        assert_eq!(service.cache_ttl_secs, 600);
    }

    /// 测试无数据库时创建快照
    #[tokio::test]
    async fn test_create_snapshot_without_db() {
        let service = PricingService::new();
        let tenant_id = Uuid::new_v4();

        let snapshot = service
            .create_snapshot("gpt-4o", &tenant_id, None)
            .await
            .unwrap();

        assert_eq!(snapshot.model_name, "gpt-4o");
        assert_eq!(snapshot.currency, "CNY");
        assert!(snapshot.input_price_per_1k > Decimal::ZERO);
        assert!(snapshot.output_price_per_1k > Decimal::ZERO);
    }

    /// 测试缓存命中 - 相同模型不同租户应命中默认缓存
    #[tokio::test]
    async fn test_cache_hit_default_pricing() {
        let service = PricingService::new();
        let tenant1 = Uuid::new_v4();
        let tenant2 = Uuid::new_v4();

        // 第一次请求，缓存未命中，使用默认价格
        let snapshot1 = service
            .create_snapshot("gpt-4o", &tenant1, None)
            .await
            .unwrap();

        // 第二次请求，不同租户，应命中 nil_tenant 缓存
        let snapshot2 = service
            .create_snapshot("gpt-4o", &tenant2, None)
            .await
            .unwrap();

        // 两个快照应该相同
        assert_eq!(snapshot1.input_price_per_1k, snapshot2.input_price_per_1k);
        assert_eq!(snapshot1.output_price_per_1k, snapshot2.output_price_per_1k);

        // 验证缓存中有 nil_tenant 的条目
        let cache = service.cache.write().await;
        let nil_tenant = Uuid::nil();
        let default_key = PricingService::cache_key(&nil_tenant, "gpt-4o", "openai");
        assert!(
            cache.contains(&default_key),
            "缓存应包含 nil_tenant 的默认定价"
        );
    }

    /// 测试清除缓存
    #[tokio::test]
    async fn test_clear_cache() {
        let service = PricingService::new();
        let tenant_id = Uuid::new_v4();

        // 创建快照，填充缓存
        let _ = service
            .create_snapshot("gpt-4o", &tenant_id, None)
            .await
            .unwrap();

        // 清除缓存
        service.clear_cache().await;

        // 验证缓存已清空
        let cache = service.cache.write().await;
        assert!(cache.is_empty());
    }

    /// 测试缓存键使用匹配到的 provider（回退匹配场景）
    /// 当请求的 provider 没有定价时，会回退到任意 provider
    /// 此时缓存键应使用匹配到的 provider，而非请求的 provider
    #[tokio::test]
    async fn test_cache_key_uses_matched_provider() {
        let service = PricingService::new();
        let tenant1 = Uuid::new_v4();
        let tenant2 = Uuid::new_v4();

        // 第一次请求：使用 provider="claude"（无数据库，使用默认价格）
        // 由于没有数据库，会使用硬编码默认价格
        let snapshot1 = service
            .create_snapshot("gpt-4o", &tenant1, Some("claude"))
            .await
            .unwrap();

        // 验证缓存使用的是 "claude"（因为无数据库时使用请求的 provider）
        {
            let cache = service.cache.write().await;
            let nil_tenant = Uuid::nil();
            let claude_key = PricingService::cache_key(&nil_tenant, "gpt-4o", "claude");
            assert!(
                cache.contains(&claude_key),
                "缓存应包含 nil:gpt-4o:claude 的默认定价"
            );
        }

        // 第二次请求：使用 provider="openai"，不同租户
        // 应该缓存到不同的 key（nil:gpt-4o:openai）
        let snapshot2 = service
            .create_snapshot("gpt-4o", &tenant2, Some("openai"))
            .await
            .unwrap();

        // 验证两个请求都返回相同的默认价格（硬编码）
        assert_eq!(snapshot1.input_price_per_1k, snapshot2.input_price_per_1k);
        assert_eq!(snapshot1.output_price_per_1k, snapshot2.output_price_per_1k);

        // 验证缓存中有两个不同的 key
        {
            let cache = service.cache.write().await;
            let nil_tenant = Uuid::nil();
            let claude_key = PricingService::cache_key(&nil_tenant, "gpt-4o", "claude");
            let openai_key = PricingService::cache_key(&nil_tenant, "gpt-4o", "openai");
            assert!(cache.contains(&claude_key));
            assert!(cache.contains(&openai_key));
            // 应该有 2 个缓存条目（不同 provider）
            assert_eq!(cache.len(), 2, "应该有 2 个不同 provider 的缓存条目");
        }
    }

    /// 测试相同 provider 请求的缓存命中
    /// 验证第一次请求后，后续相同 provider 的请求能命中缓存
    #[tokio::test]
    async fn test_cache_hit_same_provider_request() {
        let service = PricingService::new();
        let tenant1 = Uuid::new_v4();
        let tenant2 = Uuid::new_v4();

        // 第一次请求 provider="claude"
        let _snapshot1 = service
            .create_snapshot("gpt-4o", &tenant1, Some("claude"))
            .await
            .unwrap();

        // 清除跟踪变量，准备验证第二次请求命中缓存
        // 第二次请求相同 provider="claude"，不同租户
        // 由于硬编码默认定价缓存到了 nil:gpt-4o:claude，应该命中缓存
        let _snapshot2 = service
            .create_snapshot("gpt-4o", &tenant2, Some("claude"))
            .await
            .unwrap();

        // 验证缓存只有一个条目（nil:gpt-4o:claude）
        // 因为两个请求都应该使用同一个缓存
        {
            let cache = service.cache.write().await;
            // 应该只有 1 个缓存条目，证明第二次请求命中了缓存
            assert_eq!(cache.len(), 1, "第二次请求应命中缓存，不应新增缓存条目");
        }
    }

    /// 测试 LRU 缓存容量限制
    #[tokio::test]
    async fn test_lru_cache_capacity() {
        // 创建一个容量为 3 的服务
        let service = PricingService::new().with_cache_capacity(3);
        let tenant = Uuid::new_v4();

        // 插入 4 个不同的模型
        for i in 0..4 {
            let model_name = format!("model-{}", i);
            let _ = service
                .create_snapshot(&model_name, &tenant, None)
                .await
                .unwrap();
        }

        // 验证缓存只有 3 个条目（容量限制）
        {
            let cache = service.cache.write().await;
            assert_eq!(cache.len(), 3, "缓存应受容量限制");
        }

        // 验证 model-0 被淘汰（最旧的条目）
        {
            let cache = service.cache.write().await;
            let nil_tenant = Uuid::nil();
            let key0 = PricingService::cache_key(&nil_tenant, "model-0", "openai");
            assert!(!cache.contains(&key0), "model-0 应该被 LRU 淘汰");

            // 验证最新的 3 个条目存在
            for i in 1..4 {
                let key = PricingService::cache_key(&nil_tenant, &format!("model-{}", i), "openai");
                assert!(cache.contains(&key), "model-{} 应该在缓存中", i);
            }
        }
    }

    /// 测试 PricingService 配置链式调用 - 缓存容量
    #[test]
    fn test_pricing_service_with_cache_capacity() {
        let service = PricingService::new().with_cache_capacity(500);
        assert_eq!(service.cache_capacity, 500);
    }
}
