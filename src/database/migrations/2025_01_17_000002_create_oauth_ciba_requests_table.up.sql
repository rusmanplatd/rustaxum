-- Create OAuth CIBA (Client Initiated Backchannel Authentication) tables (RFC 8955)
-- These tables support decoupled authentication scenarios

CREATE TABLE oauth_ciba_requests (
    id CHAR(26) PRIMARY KEY,
    auth_req_id VARCHAR(255) NOT NULL UNIQUE,
    client_id CHAR(26) NOT NULL,
    user_id CHAR(26),
    scope VARCHAR(255),
    binding_message VARCHAR(255),
    user_code VARCHAR(255),
    login_hint VARCHAR(255),
    login_hint_token TEXT,
    id_token_hint TEXT,
    requested_expiry INTEGER,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, authorized, denied, expired
    notification_endpoint VARCHAR(255),
    notification_token TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    interval_seconds INTEGER NOT NULL DEFAULT 5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    authorized_at TIMESTAMPTZ,
    denied_at TIMESTAMPTZ
);

-- Create CIBA authorization codes table (separate from regular auth codes)
CREATE TABLE oauth_ciba_auth_codes (
    id CHAR(26) PRIMARY KEY,
    ciba_request_id CHAR(26) NOT NULL,
    code VARCHAR(255) NOT NULL UNIQUE,
    client_id CHAR(26) NOT NULL,
    user_id CHAR(26) NOT NULL,
    scopes TEXT,
    redirect_uri VARCHAR(2048),
    code_challenge VARCHAR(255),
    code_challenge_method VARCHAR(10),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX oauth_ciba_requests_auth_req_id_idx ON oauth_ciba_requests(auth_req_id);
CREATE INDEX oauth_ciba_requests_client_id_idx ON oauth_ciba_requests(client_id);
CREATE INDEX oauth_ciba_requests_user_id_idx ON oauth_ciba_requests(user_id);
CREATE INDEX oauth_ciba_requests_status_idx ON oauth_ciba_requests(status);
CREATE INDEX oauth_ciba_requests_expires_at_idx ON oauth_ciba_requests(expires_at);

CREATE INDEX oauth_ciba_auth_codes_code_idx ON oauth_ciba_auth_codes(code);
CREATE INDEX oauth_ciba_auth_codes_ciba_request_id_idx ON oauth_ciba_auth_codes(ciba_request_id);
CREATE INDEX oauth_ciba_auth_codes_client_id_idx ON oauth_ciba_auth_codes(client_id);
CREATE INDEX oauth_ciba_auth_codes_user_id_idx ON oauth_ciba_auth_codes(user_id);
CREATE INDEX oauth_ciba_auth_codes_expires_at_idx ON oauth_ciba_auth_codes(expires_at);

-- Add foreign key constraints
ALTER TABLE oauth_ciba_requests
ADD CONSTRAINT oauth_ciba_requests_client_id_foreign
FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE;

ALTER TABLE oauth_ciba_requests
ADD CONSTRAINT oauth_ciba_requests_user_id_foreign
FOREIGN KEY (user_id) REFERENCES sys_users(id) ON DELETE CASCADE;

ALTER TABLE oauth_ciba_auth_codes
ADD CONSTRAINT oauth_ciba_auth_codes_ciba_request_id_foreign
FOREIGN KEY (ciba_request_id) REFERENCES oauth_ciba_requests(id) ON DELETE CASCADE;

ALTER TABLE oauth_ciba_auth_codes
ADD CONSTRAINT oauth_ciba_auth_codes_client_id_foreign
FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE;

ALTER TABLE oauth_ciba_auth_codes
ADD CONSTRAINT oauth_ciba_auth_codes_user_id_foreign
FOREIGN KEY (user_id) REFERENCES sys_users(id) ON DELETE CASCADE;