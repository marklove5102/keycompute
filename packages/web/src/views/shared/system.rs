use dioxus::prelude::*;

use crate::stores::user_store::UserStore;
use crate::views::shared::accounts::NoPermissionView;

/// 系统诊断页面（仅 Admin 可访问）
///
/// 展示 Provider 健康状态、网关运行统计、路由调试信息（调用 DebugApi）
#[component]
pub fn System() -> Element {
    let user_store = use_context::<UserStore>();
    let is_admin = user_store
        .info
        .read()
        .as_ref()
        .map(|u| u.is_admin())
        .unwrap_or(false);

    if !is_admin {
        return rsx! { NoPermissionView { resource: "系统诊断" } };
    }

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "系统诊断" }
            p { class: "page-description", "查看 Provider 健康状态、网关运行统计和路由调试信息" }
        }

        // Provider 健康状态
        div { class: "section",
            h2 { class: "section-title", "Provider 健康状态" }
            div { class: "card",
                div { class: "card-body",
                    div { class: "health-grid",
                        HealthItem { name: "OpenAI", status: "unknown" }
                        HealthItem { name: "Azure OpenAI", status: "unknown" }
                        HealthItem { name: "Anthropic", status: "unknown" }
                        HealthItem { name: "Google Gemini", status: "unknown" }
                    }
                }
            }
        }

        // 网关运行统计
        div { class: "section",
            h2 { class: "section-title", "网关运行统计" }
            div { class: "stats-grid",
                div { class: "stat-card card",
                    div { class: "card-body",
                        p { class: "stat-label", "总请求数" }
                        p { class: "stat-value", "—" }
                    }
                }
                div { class: "stat-card card",
                    div { class: "card-body",
                        p { class: "stat-label", "成功率" }
                        p { class: "stat-value", "—" }
                    }
                }
                div { class: "stat-card card",
                    div { class: "card-body",
                        p { class: "stat-label", "平均响应时间" }
                        p { class: "stat-value", "—" }
                    }
                }
                div { class: "stat-card card",
                    div { class: "card-body",
                        p { class: "stat-label", "活跃路由数" }
                        p { class: "stat-value", "—" }
                    }
                }
            }
        }

        // 路由调试
        div { class: "section",
            h2 { class: "section-title", "路由调试" }
            div { class: "card",
                div { class: "card-header",
                    h3 { class: "card-title", "路由规则列表" }
                    button { class: "btn btn-secondary btn-sm", r#type: "button",
                        "刷新"
                    }
                }
                div { class: "card-body",
                    p { class: "text-secondary", "路由规则数据（对接 DebugApi 后展示）" }
                }
            }
        }
    }
}

// ── 内部组件 ──────────────────────────────────────────────

#[component]
fn HealthItem(name: String, status: String) -> Element {
    let (status_class, status_text) = match status.as_str() {
        "healthy" => ("badge-success", "健康"),
        "degraded" => ("badge-warning", "降级"),
        "unhealthy" => ("badge-error", "异常"),
        _ => ("badge-neutral", "未知"),
    };

    rsx! {
        div { class: "health-item",
            div { class: "health-name", "{name}" }
            span { class: "badge {status_class}", "{status_text}" }
        }
    }
}
