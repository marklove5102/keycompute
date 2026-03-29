use dioxus::prelude::*;

use crate::stores::user_store::UserStore;

/// 分销记录页面
///
/// - 普通用户：仅查看自己的分销收益记录
/// - Admin：查看全平台分销记录，配置分销规则
#[component]
pub fn DistributionRecords() -> Element {
    let user_store = use_context::<UserStore>();
    let is_admin = user_store
        .info
        .read()
        .as_ref()
        .map(|u| u.is_admin())
        .unwrap_or(false);

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "分销记录" }
            p { class: "page-description",
                if is_admin { "查看全平台分销收益记录，管理分销规则" }
                else { "查看您通过邀请获得的分销收益明细" }
            }
        }

        // 收益统计卡片
        div { class: "stats-grid",
            div { class: "stat-card card",
                div { class: "card-body",
                    p { class: "stat-label", "总收益" }
                    p { class: "stat-value", "¥ 0.00" }
                }
            }
            div { class: "stat-card card",
                div { class: "card-body",
                    p { class: "stat-label", "可用余额" }
                    p { class: "stat-value", "¥ 0.00" }
                }
            }
            div { class: "stat-card card",
                div { class: "card-body",
                    p { class: "stat-label", "待结算" }
                    p { class: "stat-value", "¥ 0.00" }
                }
            }
        }

        if is_admin {
            // Admin 工具栏：规则配置入口
            div { class: "toolbar",
                div { class: "toolbar-right",
                    button { class: "btn btn-secondary btn-sm", r#type: "button",
                        "分销规则配置"
                    }
                    button { class: "btn btn-secondary btn-sm", r#type: "button",
                        "导出记录"
                    }
                }
            }
        }

        div { class: "card",
            div { class: "table-container",
                table { class: "table",
                    thead {
                        tr {
                            th { "记录编号" }
                            th { "来源用户" }
                            th { "消费金额" }
                            th { "分销比例" }
                            th { "分销金额" }
                            th { "状态" }
                            th { "创建时间" }
                            if is_admin {
                                th { "推荐人" }
                            }
                        }
                    }
                    tbody {
                        tr {
                            td {
                                colspan: if is_admin { "8" } else { "7" },
                                class: "table-empty",
                                "暂无分销记录"
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
