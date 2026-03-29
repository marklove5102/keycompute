use dioxus::prelude::*;

use crate::stores::user_store::UserStore;

/// 用户管理页面
///
/// - 普通用户：仅展示自己的信息和编辑入口
/// - Admin：展示全平台用户列表，支持搜索/状态筛选/批量操作
#[component]
pub fn Users() -> Element {
    let user_store = use_context::<UserStore>();
    let is_admin = user_store
        .info
        .read()
        .as_ref()
        .map(|u| u.is_admin())
        .unwrap_or(false);

    if is_admin {
        rsx! { AdminUsersView {} }
    } else {
        rsx! { UserSelfView {} }
    }
}

// ── Admin 视图 ────────────────────────────────────────────

#[component]
fn AdminUsersView() -> Element {
    let mut search = use_signal(String::new);

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "用户管理" }
            p { class: "page-description", "查看和管理平台所有注册用户" }
        }

        // 搜索/筛选工具栏
        div { class: "toolbar",
            div { class: "toolbar-left",
                div { class: "input-wrapper",
                    input {
                        class: "input-field",
                        r#type: "search",
                        placeholder: "搜索邮箱或用户名...",
                        value: "{search}",
                        oninput: move |e| *search.write() = e.value(),
                    }
                }
            }
            div { class: "toolbar-right",
                // 预留：导出按钮
                button { class: "btn btn-secondary btn-sm", r#type: "button",
                    "导出"
                }
            }
        }

        // 用户表格
        div { class: "card",
            div { class: "table-container",
                table { class: "table",
                    thead {
                        tr {
                            th { "用户" }
                            th { "角色" }
                            th { "租户" }
                            th { "注册时间" }
                            th { "状态" }
                            th { "操作" }
                        }
                    }
                    tbody {
                        tr {
                            td {
                                colspan: "6",
                                class: "table-empty",
                                "暂无用户数据"
                            }
                        }
                    }
                }
            }
        }

        // 分页
        div { class: "pagination",
            span { class: "pagination-info", "共 0 条" }
        }
    }
}

// ── 普通用户视图 ──────────────────────────────────────────

#[component]
fn UserSelfView() -> Element {
    let user_store = use_context::<UserStore>();
    let user_info = user_store.info.read();

    let display_name = user_info
        .as_ref()
        .map(|u| u.display_name().to_string())
        .unwrap_or_default();
    let email = user_info
        .as_ref()
        .map(|u| u.email.clone())
        .unwrap_or_default();
    let role = user_info
        .as_ref()
        .map(|u| u.role.clone())
        .unwrap_or_default();

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "我的账户" }
            p { class: "page-description", "查看和管理您的个人账户信息" }
        }

        div { class: "card",
            div { class: "card-header",
                h3 { class: "card-title", "账户信息" }
                a { class: "btn btn-secondary btn-sm", href: "/user/profile", "编辑资料" }
            }
            div { class: "card-body",
                div { class: "info-grid",
                    div { class: "info-item",
                        span { class: "info-label", "显示名称" }
                        span { class: "info-value", "{display_name}" }
                    }
                    div { class: "info-item",
                        span { class: "info-label", "邮箱" }
                        span { class: "info-value", "{email}" }
                    }
                    div { class: "info-item",
                        span { class: "info-label", "角色" }
                        span { class: "badge badge-info", "{role}" }
                    }
                }
            }
        }
    }
}
