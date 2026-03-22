//! 处理器模块
//!
//! 处理各种 HTTP 请求

pub mod billing;
pub mod chat;
pub mod gateway;
pub mod health;
pub mod models;
pub mod pricing;
pub mod routing;

pub use billing::{
    calculate_cost, get_billing_stats, list_billing_records, trigger_billing,
};
pub use chat::chat_completions;
pub use gateway::{check_provider_health, get_execution_stats, get_gateway_status};
pub use health::health_check;
pub use models::list_models;
pub use pricing::{calculate_cost as get_pricing_cost, get_pricing};
pub use routing::{debug_routing, get_provider_health};
