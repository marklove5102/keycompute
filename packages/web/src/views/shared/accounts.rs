use dioxus::prelude::*;

use crate::stores::user_store::UserStore;

/// 账号管理页面（LLM 渠道配置）
///
/// - 普通用户：无权限提示
/// - Admin：管理 LLM Provider 渠道，支持测试连接、刷新状态
#[component]
pub fn Accounts() -> Element {
    let user_store = use_context::<UserStore>();
    let is_admin = user_store
        .info
        .read()
        .as_ref()
        .map(|u| u.is_admin())
        .unwrap_or(false);

    if is_admin {
        rsx! { AdminAccountsView {} }
    } else {
        rsx! { NoPermissionView { resource: "账号管理" } }
    }
}

// ── Admin 视图 ────────────────────────────────────────────

#[component]
fn AdminAccountsView() -> Element {
    let mut show_create = use_signal(|| false);

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "账号管理" }
            p { class: "page-description", "管理 LLM Provider 渠道，配置 API Key 和模型映射" }
        }

        div { class: "toolbar",
            div { class: "toolbar-right",
                button {
                    class: "btn btn-primary btn-sm",
                    r#type: "button",
                    onclick: move |_| *show_create.write() = true,
                    "+ 新增渠道"
                }
            }
        }

        div { class: "card",
            div { class: "table-container",
                table { class: "table",
                    thead {
                        tr {
                            th { "渠道名称" }
                            th { "Provider" }
                            th { "基础 URL" }
                            th { "模型数" }
                            th { "状态" }
                            th { "操作" }
                        }
                    }
                    tbody {
                        tr {
                            td {
                                colspan: "6",
                                class: "table-empty",
                                "暂无渠道配置，请点击「新增渠道」添加"
                            }
                        }
                    }
                }
            }
        }

        // 新增渠道弹窗（骨架）
        if show_create() {
            div { class: "modal-backdrop",
                onclick: move |_| *show_create.write() = false,
                div {
                    class: "modal",
                    onclick: move |e| e.stop_propagation(),
                    div { class: "modal-header",
                        h2 { class: "modal-title", "新增 LLM 渠道" }
                        button {
                            class: "btn btn-ghost btn-sm",
                            r#type: "button",
                            onclick: move |_| *show_create.write() = false,
                            "✕"
                        }
                    }
                    div { class: "modal-body",
                        p { class: "text-secondary", "渠道配置表单（对接 AccountApi 后实现）" }
                    }
                    div { class: "modal-footer",
                        button {
                            class: "btn btn-ghost",
                            r#type: "button",
                            onclick: move |_| *show_create.write() = false,
                            "取消"
                        }
                        button { class: "btn btn-primary", r#type: "submit", "保存" }
                    }
                }
            }
        }
    }
}

// ── 无权限视图（共用组件）────────────────────────────────

#[component]
pub fn NoPermissionView(resource: String) -> Element {
    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "{resource}" }
        }
        div { class: "empty-state",
            div { class: "empty-icon", "🔒" }
            h3 { class: "empty-title", "暂无访问权限" }
            p { class: "empty-description",
                "您没有访问「{resource}」的权限，请联系管理员"
            }
        }
    }
}
