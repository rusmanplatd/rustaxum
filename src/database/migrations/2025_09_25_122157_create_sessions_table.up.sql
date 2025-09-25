-- Create sessions table
CREATE TABLE sessions (
    id VARCHAR(40) PRIMARY KEY,
    user_id CHAR(26) REFERENCES sys_users(id),
    ip_address VARCHAR(45),
    user_agent TEXT,
    payload TEXT NOT NULL,
    last_activity INTEGER NOT NULL
);

-- Add indexes for performance
CREATE INDEX idx_sessions_user_id ON sessions (user_id);
CREATE INDEX idx_sessions_last_activity ON sessions (last_activity);
