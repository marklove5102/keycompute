use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 执行计划：包含主目标和回退链
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub primary: ExecutionTarget,
    pub fallback_chain: Vec<ExecutionTarget>,
}

impl ExecutionPlan {
    pub fn new(primary: ExecutionTarget) -> Self {
        Self {
            primary,
            fallback_chain: Vec::new(),
        }
    }

    pub fn with_fallback(mut self, fallback: ExecutionTarget) -> Self {
        self.fallback_chain.push(fallback);
        self
    }

    pub fn with_fallbacks(mut self, fallbacks: Vec<ExecutionTarget>) -> Self {
        self.fallback_chain.extend(fallbacks);
        self
    }

    /// 获取所有执行目标（主目标 + 回退链）
    pub fn all_targets(&self) -> impl Iterator<Item = &ExecutionTarget> {
        std::iter::once(&self.primary).chain(self.fallback_chain.iter())
    }
}

/// 执行目标：指定具体的 provider 和账号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTarget {
    pub provider: String,
    pub account_id: Uuid,
    pub endpoint: String,
    pub api_key: String, // 已解密的上游 key
}

impl ExecutionTarget {
    pub fn new(
        provider: impl Into<String>,
        account_id: Uuid,
        endpoint: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            provider: provider.into(),
            account_id,
            endpoint: endpoint.into(),
            api_key: api_key.into(),
        }
    }
}
