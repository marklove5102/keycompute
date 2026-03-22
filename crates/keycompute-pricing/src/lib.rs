//! Pricing Module
//!
//! 定价模块，只读，生成 PricingSnapshot。
//! 架构约束：不写任何状态，不参与路由或执行。

use keycompute_types::{PricingSnapshot, Result};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 定价服务
///
/// 负责从数据库加载模型价格，生成 PricingSnapshot
#[derive(Debug, Clone)]
pub struct PricingService {
    /// 价格缓存
    cache: Arc<RwLock<HashMap<String, PricingSnapshot>>>,
}

impl Default for PricingService {
    fn default() -> Self {
        Self::new()
    }
}

impl PricingService {
    /// 创建新的定价服务
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建价格快照（固化到 RequestContext）
    ///
    /// 从数据库或缓存加载指定模型的价格
    pub async fn create_snapshot(
        &self,
        model_name: &str,
        _tenant_id: &Uuid,
    ) -> Result<PricingSnapshot> {
        // 先检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(snapshot) = cache.get(model_name) {
                return Ok(snapshot.clone());
            }
        }

        // TODO: 从数据库加载价格
        // 这里使用模拟数据
        let snapshot = self.get_default_pricing(model_name);

        // 写入缓存
        {
            let mut cache = self.cache.write().await;
            cache.insert(model_name.to_string(), snapshot.clone());
        }

        tracing::debug!(model = %model_name, "Created pricing snapshot");
        Ok(snapshot)
    }

    /// 获取默认定价
    fn get_default_pricing(&self, model_name: &str) -> PricingSnapshot {
        // 根据模型名称返回默认价格
        let (input_price, output_price) = match model_name {
            "gpt-4o" => (Decimal::from(500) / Decimal::from(1000), Decimal::from(1500) / Decimal::from(1000)),
            "gpt-4o-mini" => (Decimal::from(150) / Decimal::from(1000), Decimal::from(600) / Decimal::from(1000)),
            "gpt-4-turbo" => (Decimal::from(1000) / Decimal::from(1000), Decimal::from(3000) / Decimal::from(1000)),
            "gpt-3.5-turbo" => (Decimal::from(50) / Decimal::from(1000), Decimal::from(150) / Decimal::from(1000)),
            _ => (Decimal::from(100) / Decimal::from(1000), Decimal::from(300) / Decimal::from(1000)),
        };

        PricingSnapshot {
            model_name: model_name.to_string(),
            currency: "CNY".to_string(),
            input_price_per_1k: input_price,
            output_price_per_1k: output_price,
        }
    }

    /// 清除缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        tracing::info!("Pricing cache cleared");
    }

    /// 计算请求费用
    pub fn calculate_cost(
        &self,
        input_tokens: u32,
        output_tokens: u32,
        pricing: &PricingSnapshot,
    ) -> Decimal {
        let input_cost = Decimal::from(input_tokens) * pricing.input_price_per_1k / Decimal::from(1000);
        let output_cost = Decimal::from(output_tokens) * pricing.output_price_per_1k / Decimal::from(1000);
        input_cost + output_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pricing_service_new() {
        let service = PricingService::new();
        let snapshot = service.create_snapshot("gpt-4o", &Uuid::new_v4()).await.unwrap();

        assert_eq!(snapshot.model_name, "gpt-4o");
        assert_eq!(snapshot.currency, "CNY");
        assert!(snapshot.input_price_per_1k > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_create_snapshot_caching() {
        let service = PricingService::new();
        let tenant_id = Uuid::new_v4();

        // 第一次创建
        let snapshot1 = service.create_snapshot("gpt-4o", &tenant_id).await.unwrap();

        // 第二次应该从缓存读取
        let snapshot2 = service.create_snapshot("gpt-4o", &tenant_id).await.unwrap();

        assert_eq!(snapshot1.input_price_per_1k, snapshot2.input_price_per_1k);
    }

    #[test]
    fn test_calculate_cost() {
        let service = PricingService::new();
        let pricing = PricingSnapshot {
            model_name: "test".to_string(),
            currency: "CNY".to_string(),
            input_price_per_1k: Decimal::from(1),
            output_price_per_1k: Decimal::from(2),
        };

        let cost = service.calculate_cost(1000, 500, &pricing);
        assert_eq!(cost, Decimal::from(2)); // 1 * 1 + 2 * 0.5 = 2
    }
}
