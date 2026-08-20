-- 0002_create_audit_logs.sql
-- Stores an audit entry for each CRUD operation

CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    actor_name TEXT,
    actor_role TEXT,
    action_ts TIMESTAMP WITH TIME ZONE DEFAULT now(),
    action_type TEXT,
    target_table TEXT,
    description TEXT
);

CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_logs(actor_name);
CREATE INDEX IF NOT EXISTS idx_audit_ts ON audit_logs(action_ts);
