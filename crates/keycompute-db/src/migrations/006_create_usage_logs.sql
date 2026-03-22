-- usage_logs: 计费主账本，不可变
CREATE TABLE usage_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL UNIQUE,
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    api_key_id UUID NOT NULL,
    model_name VARCHAR(100) NOT NULL,
    provider_name VARCHAR(50) NOT NULL,
    account_id UUID NOT NULL,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    total_tokens INTEGER NOT NULL,
    input_unit_price_snapshot DECIMAL(20, 10) NOT NULL,
    output_unit_price_snapshot DECIMAL(20, 10) NOT NULL,
    user_amount DECIMAL(20, 10) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'CNY',
    usage_source VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    finished_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_usage_logs_tenant ON usage_logs(tenant_id);
CREATE INDEX idx_usage_logs_user ON usage_logs(user_id);
CREATE INDEX idx_usage_logs_api_key ON usage_logs(api_key_id);
CREATE INDEX idx_usage_logs_created ON usage_logs(created_at);
CREATE INDEX idx_usage_logs_request ON usage_logs(request_id);
