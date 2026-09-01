CREATE TABLE IF NOT EXISTS saga_log (
    saga_id VARCHAR(35) PRIMARY KEY,
    saga_type VARCHAR(50) NOT NULL,
    user_id VARCHAR(35) NOT NULL,
    status VARCHAR(20) NOT NULL,
    value BIGINT,
    compensation VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_saga_log_user_id ON saga_log (user_id);
CREATE INDEX IF NOT EXISTS idx_saga_log_status ON saga_log (status);