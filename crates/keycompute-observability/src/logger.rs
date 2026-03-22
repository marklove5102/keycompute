use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// 初始化日志系统
///
/// 使用 tracing-subscriber 配置结构化日志输出，支持 JSON 格式
/// 环境变量 KEYCOMPUTE_LOG 控制日志级别，默认为 info
///
/// # Examples
///
/// ```
/// use keycompute_observability::init_logger;
/// init_logger();
/// ```
pub fn init_logger() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info")
            .add_directive("keycompute=info".parse().unwrap())
            .add_directive("tower_http=info".parse().unwrap())
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

/// 初始化开发环境日志（人类可读格式）
///
/// 适用于本地开发，输出格式更易读
pub fn init_dev_logger() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("debug").add_directive("keycompute=debug".parse().unwrap())
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().compact())
        .init();
}

/// 初始化测试环境日志
///
/// 仅在测试时启用，避免污染测试输出
#[cfg(test)]
pub fn init_test_logger() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("error")
        .try_init();
}
