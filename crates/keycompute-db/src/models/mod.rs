//! 数据库模型模块
//!
//! 包含所有表的 ORM 模型和 CRUD 操作

pub mod account;
pub mod api_key;
pub mod distribution_record;
pub mod pricing_model;
pub mod tenant;
pub mod tenant_distribution_rule;
pub mod usage_log;
pub mod user;

// 重新导出常用模型
pub use account::{Account, CreateAccountRequest, UpdateAccountRequest};
pub use api_key::{ApiKey, ApiKeyResponse, CreateApiKeyRequest};
pub use distribution_record::{
    CreateDistributionRecordRequest, DistributionRecord, DistributionStats,
};
pub use pricing_model::{CreatePricingRequest, PricingModel, UpdatePricingRequest};
pub use tenant::{CreateTenantRequest, Tenant, UpdateTenantRequest};
pub use tenant_distribution_rule::{
    CreateDistributionRuleRequest, TenantDistributionRule, UpdateDistributionRuleRequest,
};
pub use usage_log::{CreateUsageLogRequest, UsageLog, UsageStats};
pub use user::{CreateUserRequest, UpdateUserRequest, User};
