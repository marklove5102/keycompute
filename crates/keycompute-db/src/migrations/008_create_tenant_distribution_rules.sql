-- tenant_distribution_rules: 租户分销规则
CREATE TABLE tenant_distribution_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    beneficiary_id UUID NOT NULL,
    share_ratio DECIMAL(5, 4) NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    effective_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    effective_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, beneficiary_id, effective_from)
);

CREATE INDEX idx_tenant_distribution_rules_tenant ON tenant_distribution_rules(tenant_id);
CREATE INDEX idx_tenant_distribution_rules_enabled ON tenant_distribution_rules(enabled) WHERE enabled = TRUE;
