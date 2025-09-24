-- Create OAuth Pushed Authorization Requests table (RFC 9126)
-- This table stores pushed authorization requests for PAR flow

CREATE TABLE oauth_pushed_requests (
    id CHAR(26) PRIMARY KEY,
    request_uri VARCHAR(255) NOT NULL UNIQUE,
    client_id CHAR(26) NOT NULL,
    request_data TEXT NOT NULL, -- JSON serialized authorization request
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX oauth_pushed_requests_request_uri_idx ON oauth_pushed_requests(request_uri);
CREATE INDEX oauth_pushed_requests_client_id_idx ON oauth_pushed_requests(client_id);
CREATE INDEX oauth_pushed_requests_expires_at_idx ON oauth_pushed_requests(expires_at);
CREATE INDEX oauth_pushed_requests_used_idx ON oauth_pushed_requests(used);

-- Add foreign key constraint to oauth_clients
ALTER TABLE oauth_pushed_requests
ADD CONSTRAINT oauth_pushed_requests_client_id_foreign
FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE;