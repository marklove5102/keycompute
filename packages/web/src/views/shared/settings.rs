use dioxus::prelude::*;

use crate::stores::user_store::UserStore;

/// 系统设置页面
///
/// - 普通用户：**无此页面入口**（个人偏好通过导航栏按钮切换，存 localStorage）
/// - Admin：全局系统参数配置（调用 SettingsApi，需 Admin token）
#[component]
pub fn Settings() -> Element {
    let user_store = use_context::<UserStore>();
    let is_admin = user_store
        .info
        .read()
        .as_ref()
        .map(|u| u.is_admin())
        .unwrap_or(false);

    rsx! {
        div { class: "page-header",
            h1 { class: "page-title", "系统设置" }
            p { class: "page-description",
                if is_admin { "配置平台全局系统参数" }
                else { "查看系统运行配置（仅供参考）" }
            }
        }

        if !is_admin {
            div { class: "alert alert-info",
                span { class: "alert-icon", "ℹ" }
                div { class: "alert-content",
                    p { class: "alert-body",
                        "系统设置仅 Admin 可修改。个人语言/主题偏好请通过顶部导航栏右侧按钮切换。"
                    }
                }
            }
        }

        // 基础系统配置
        div { class: "card",
            div { class: "card-header",
                h3 { class: "card-title", "基础配置" }
                if is_admin {
                    button { class: "btn btn-primary btn-sm", r#type: "button", "保存" }
                }
            }
            div { class: "card-body",
                div { class: "settings-grid",
                    SettingItem {
                        label: "平台名称",
                        value: "KeyCompute",
                        editable: is_admin,
                    }
                    SettingItem {
                        label: "注册模式",
                        value: "开放注册",
                        editable: is_admin,
                    }
                    SettingItem {
                        label: "默认货币",
                        value: "CNY",
                        editable: is_admin,
                    }
                    SettingItem {
                        label: "最低充值金额",
                        value: "10.00",
                        editable: is_admin,
                    }
                }
            }
        }

        // 安全配置（仅 Admin 可见详情）
        div { class: "card",
            div { class: "card-header",
                h3 { class: "card-title", "安全配置" }
                if is_admin {
                    button { class: "btn btn-primary btn-sm", r#type: "button", "保存" }
                }
            }
            div { class: "card-body",
                div { class: "settings-grid",
                    SettingItem {
                        label: "JWT Token 有效期（小时）",
                        value: "24",
                        editable: is_admin,
                    }
                    SettingItem {
                        label: "邮箱验证",
                        value: "已启用",
                        editable: is_admin,
                    }
                }
            }
        }

        // 通知配置
        div { class: "card",
            div { class: "card-header",
                h3 { class: "card-title", "通知配置" }
                if is_admin {
                    button { class: "btn btn-primary btn-sm", r#type: "button", "保存" }
                }
            }
            div { class: "card-body",
                p { class: "text-secondary", "邮件服务器和通知模板配置（对接 SettingsApi 后实现）" }
            }
        }
    }
}

// ── 内部组件 ──────────────────────────────────────────────

#[component]
fn SettingItem(label: String, value: String, editable: bool) -> Element {
    let mut edit_val = use_signal(|| value.clone());

    rsx! {
        div { class: "setting-item",
            span { class: "setting-label", "{label}" }
            if editable {
                input {
                    class: "input-field",
                    value: "{edit_val}",
                    oninput: move |e| *edit_val.write() = e.value(),
                }
            } else {
                span { class: "setting-value", "{value}" }
            }
        }
    }
}
