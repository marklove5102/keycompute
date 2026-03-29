use dioxus::prelude::*;

use crate::stores::user_store::UserStore;

/// 支付订单页面
///
/// - 普通用户：仅查看自己的订单
/// - Admin：查看所有订单，支持审核和退款
#[component]
pub fn PaymentOrders() -> Element {
    let user_store = use_context::<UserStore>();
    let is_admin = user_store
        .info
        .read()
        .as_ref()
        .map(|u| u.is_admin())
        .unwrap_or(false);

    let mut status_filter = use_signal(|| "all".to_string());

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "支付订单" }
            p { class: "page-description",
                if is_admin { "查看和管理平台所有支付订单" }
                else { "查看您的充值和支付记录" }
            }
        }

        // 状态筛选
        div { class: "toolbar",
            div { class: "toolbar-left",
                div { class: "filter-tabs",
                    for (val, label) in [("all", "全部"), ("pending", "待支付"), ("paid", "已支付"), ("failed", "已失败"), ("refunded", "已退款")] {
                        button {
                            class: if status_filter() == val { "filter-tab active" } else { "filter-tab" },
                            r#type: "button",
                            onclick: {
                                let val = val.to_string();
                                move |_| *status_filter.write() = val.clone()
                            },
                            "{label}"
                        }
                    }
                }
            }
        }

        div { class: "card",
            div { class: "table-container",
                table { class: "table",
                    thead {
                        tr {
                            th { "订单号" }
                            th { "金额" }
                            th { "支付方式" }
                            th { "状态" }
                            th { "创建时间" }
                            if is_admin {
                                th { "用户" }
                                th { "操作" }
                            }
                        }
                    }
                    tbody {
                        tr {
                            td {
                                colspan: if is_admin { "7" } else { "5" },
                                class: "table-empty",
                                "暂无订单记录"
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
