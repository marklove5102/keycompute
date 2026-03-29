use dioxus::prelude::*;

use crate::stores::user_store::UserStore;

/// 定价管理页面
///
/// - 普通用户：只读查看定价策略列表
/// - Admin：完整 CRUD（创建/编辑/删除/设置默认）
#[component]
pub fn Pricing() -> Element {
    let user_store = use_context::<UserStore>();
    let is_admin = user_store
        .info
        .read()
        .as_ref()
        .map(|u| u.is_admin())
        .unwrap_or(false);

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "定价管理" }
            p { class: "page-description",
                if is_admin { "管理平台定价策略，设置模型调用费率" }
                else { "查看当前平台可用的定价策略" }
            }
        }

        if is_admin {
            div { class: "toolbar",
                div { class: "toolbar-right",
                    button { class: "btn btn-primary btn-sm", r#type: "button",
                        "+ 新建定价"
                    }
                }
            }
        }

        div { class: "card",
            div { class: "table-container",
                table { class: "table",
                    thead {
                        tr {
                            th { "名称" }
                            th { "描述" }
                            th { "输入单价（/1K tokens）" }
                            th { "输出单价（/1K tokens）" }
                            th { "默认" }
                            if is_admin {
                                th { "操作" }
                            }
                        }
                    }
                    tbody {
                        tr {
                            td {
                                colspan: if is_admin { "6" } else { "5" },
                                class: "table-empty",
                                "暂无定价策略"
                            }
                        }
                    }
                }
            }
        }
    }
}
