//! 处理器模块
//!
//! 处理各种 HTTP 请求

pub mod chat;
pub mod health;
pub mod models;
pub mod pricing;
pub mod routing;

pub use chat::chat_completions;
pub use health::health_check;
pub use models::list_models;
pub use pricing::{calculate_cost, get_pricing};
pub use routing::{debug_routing, get_provider_health};
