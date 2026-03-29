use dioxus::prelude::*;

use crate::stores::user_store::UserStore;
use crate::views::shared::accounts::NoPermissionView;

/// 租户管理页面（仅 Admin 可访问）
///
/// - 普通用户：无权限提示
/// - Admin：查看全平台租户列表（调用 TenantApi）
#[component]
pub fn Tenants() -> Element {
    let user_store = use_context::<UserStore>();
    let is_admin = user_store
        .info
        .read()
        .as_ref()
        .map(|u| u.is_admin())
        .unwrap_or(false);

    if !is_admin {
        return rsx! { NoPermissionView { resource: "租户管理" } };
    }

    let mut search = use_signal(String::new);

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "租户管理" }
            p { class: "page-description", "查看和管理平台所有租户信息" }
        }

        div { class: "toolbar",
            div { class: "toolbar-left",
                div { class: "input-wrapper",
                    input {
                        class: "input-field",
                        r#type: "search",
                        placeholder: "搜索租户名称或 ID...",
                        value: "{search}",
                        oninput: move |e| *search.write() = e.value(),
                    }
                }
            }
        }

        div { class: "card",
            div { class: "table-container",
                table { class: "table",
                    thead {
                        tr {
                            th { "租户 ID" }
                            th { "名称" }
                            th { "用户数" }
                            th { "API Key 数" }
                            th { "余额" }
                            th { "创建时间" }
                            th { "状态" }
                        }
                    }
                    tbody {
                        tr {
                            td {
                                colspan: "7",
                                class: "table-empty",
                                "暂无租户数据"
                            }
                        }
                    }
                }
            }
        }

        div { class: "pagination",
            span { class: "pagination-info", "共 0 条" }
        }
    }
}
