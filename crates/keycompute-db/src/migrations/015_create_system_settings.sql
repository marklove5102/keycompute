-- 系统设置表
-- 存储全局系统配置，支持运行时修改

CREATE TABLE IF NOT EXISTS system_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- 设置键名（唯一）
    key VARCHAR(100) UNIQUE NOT NULL,
    -- 设置值（以字符串形式存储）
    value TEXT NOT NULL,
    -- 值类型：string, bool, int, decimal, json
    value_type VARCHAR(20) NOT NULL DEFAULT 'string',
    -- 设置描述
    description VARCHAR(255),
    -- 是否为敏感设置（敏感设置不在日志中显示）
    is_sensitive BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_system_settings_key ON system_settings(key);

-- 插入默认系统设置
INSERT INTO system_settings (key, value, value_type, description) VALUES
    -- 站点设置
    ('site_name', 'KeyCompute', 'string', '站点名称'),
    ('site_description', 'AI Gateway Platform', 'string', '站点描述'),
    ('site_logo_url', '', 'string', '站点 Logo URL'),
    ('site_favicon_url', '', 'string', '站点 Favicon URL'),
    
    -- 注册设置
    ('allow_registration', 'true', 'bool', '是否允许新用户注册'),
    ('email_verification_required', 'true', 'bool', '注册是否需要邮箱验证'),
    ('default_user_quota', '10.00', 'decimal', '新用户默认配额（元）'),
    ('default_user_role', 'user', 'string', '新用户默认角色'),
    
    -- 限流设置
    ('default_rpm_limit', '60', 'int', '默认 RPM 限制'),
    ('default_tpm_limit', '100000', 'int', '默认 TPM 限制'),
    
    -- 系统状态
    ('maintenance_mode', 'false', 'bool', '维护模式（开启后禁止所有 API 访问）'),
    ('maintenance_message', '', 'string', '维护模式提示信息'),
    
    -- 分销设置
    ('distribution_enabled', 'false', 'bool', '是否启用分销系统'),
    ('distribution_level1_default_ratio', '0.03', 'decimal', '一级分销默认分成比例'),
    ('distribution_level2_default_ratio', '0.01', 'decimal', '二级分销默认分成比例'),
    ('distribution_min_withdraw', '10.00', 'decimal', '最低提现金额'),
    
    -- 支付设置
    ('alipay_enabled', 'false', 'bool', '是否启用支付宝支付'),
    ('wechatpay_enabled', 'false', 'bool', '是否启用微信支付'),
    ('min_recharge_amount', '1.00', 'decimal', '最小充值金额'),
    ('max_recharge_amount', '100000.00', 'decimal', '最大充值金额'),
    
    -- 安全设置
    ('login_failed_limit', '5', 'int', '登录失败次数限制'),
    ('login_lockout_minutes', '30', 'int', '登录锁定时长（分钟）'),
    -- 密码策略使用硬编码，参见 keycompute-auth/src/password/validator.rs
    -- ('password_min_length', '8', 'int', '密码最小长度'),
    -- ('password_require_uppercase', 'true', 'bool', '密码是否需要大写字母'),
    -- ('password_require_lowercase', 'true', 'bool', '密码是否需要小写字母'),
    -- ('password_require_number', 'true', 'bool', '密码是否需要数字'),
    -- ('password_require_special', 'false', 'bool', '密码是否需要特殊字符'),
    
    -- 公告设置
    ('system_notice', '', 'string', '系统公告内容'),
    ('system_notice_enabled', 'false', 'bool', '是否显示系统公告'),
    
    -- 其他设置
    ('footer_content', '', 'string', '页脚自定义内容'),
    ('about_content', '', 'string', '关于页面内容'),
    ('terms_of_service_url', '', 'string', '服务条款 URL'),
    ('privacy_policy_url', '', 'string', '隐私政策 URL')
ON CONFLICT (key) DO NOTHING;

-- 创建更新时间触发器
CREATE OR REPLACE FUNCTION update_system_settings_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_system_settings_updated_at
    BEFORE UPDATE ON system_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_system_settings_updated_at();
