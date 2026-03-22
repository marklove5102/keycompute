use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 价格快照：请求开始时固化，不可变
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingSnapshot {
    pub model_name: String,
    pub currency: String,
    pub input_price_per_1k: Decimal,
    pub output_price_per_1k: Decimal,
}

impl PricingSnapshot {
    pub fn new(
        model_name: impl Into<String>,
        currency: impl Into<String>,
        input_price_per_1k: Decimal,
        output_price_per_1k: Decimal,
    ) -> Self {
        Self {
            model_name: model_name.into(),
            currency: currency.into(),
            input_price_per_1k,
            output_price_per_1k,
        }
    }
}

impl Default for PricingSnapshot {
    fn default() -> Self {
        Self {
            model_name: String::new(),
            currency: "CNY".to_string(),
            input_price_per_1k: Decimal::ZERO,
            output_price_per_1k: Decimal::ZERO,
        }
    }
}
