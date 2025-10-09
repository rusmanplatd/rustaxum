-- Extend oauth_clients table to support OAuth 2.1 features

ALTER TABLE oauth_clients
ADD COLUMN IF NOT EXISTS public_key_pem TEXT,
ADD COLUMN IF NOT EXISTS metadata JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS jwks_uri VARCHAR(2048),
ADD COLUMN IF NOT EXISTS token_endpoint_auth_method VARCHAR(50) DEFAULT 'client_secret_basic' NOT NULL,
ADD COLUMN IF NOT EXISTS response_types TEXT[] DEFAULT ARRAY['code'],
ADD COLUMN IF NOT EXISTS grant_types TEXT[] DEFAULT ARRAY['authorization_code', 'refresh_token'],
ADD COLUMN IF NOT EXISTS scope VARCHAR(1000) DEFAULT 'openid' NOT NULL,
ADD COLUMN IF NOT EXISTS audience TEXT[],
ADD COLUMN IF NOT EXISTS require_auth_time BOOLEAN DEFAULT false NOT NULL,
ADD COLUMN IF NOT EXISTS default_max_age INTEGER,
ADD COLUMN IF NOT EXISTS require_pushed_authorization_requests BOOLEAN DEFAULT false NOT NULL,
ADD COLUMN IF NOT EXISTS certificate_bound_access_tokens BOOLEAN DEFAULT false NOT NULL;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_oauth_clients_token_endpoint_auth_method ON oauth_clients(token_endpoint_auth_method);
CREATE INDEX IF NOT EXISTS idx_oauth_clients_require_par ON oauth_clients(require_pushed_authorization_requests);
CREATE INDEX IF NOT EXISTS idx_oauth_clients_certificate_bound ON oauth_clients(certificate_bound_access_tokens);

-- Add comments for documentation
COMMENT ON COLUMN oauth_clients.public_key_pem IS 'Public key in PEM format for JWT client authentication';
COMMENT ON COLUMN oauth_clients.metadata IS 'Additional client metadata as JSON';
COMMENT ON COLUMN oauth_clients.jwks_uri IS 'URL of client JWKS endpoint for key discovery';
COMMENT ON COLUMN oauth_clients.token_endpoint_auth_method IS 'Client authentication method: client_secret_basic, client_secret_post, private_key_jwt, tls_client_auth, etc.';
COMMENT ON COLUMN oauth_clients.response_types IS 'OAuth response types supported by this client';
COMMENT ON COLUMN oauth_clients.grant_types IS 'OAuth grant types supported by this client';
COMMENT ON COLUMN oauth_clients.scope IS 'Default scopes for this client';
COMMENT ON COLUMN oauth_clients.audience IS 'Intended audience for tokens issued to this client';
COMMENT ON COLUMN oauth_clients.require_auth_time IS 'Require auth_time claim in ID tokens';
COMMENT ON COLUMN oauth_clients.default_max_age IS 'Default max authentication age in seconds';
COMMENT ON COLUMN oauth_clients.require_pushed_authorization_requests IS 'Require PAR (RFC 9126) for this client';
COMMENT ON COLUMN oauth_clients.certificate_bound_access_tokens IS 'Enable certificate-bound tokens (mTLS) for this client';
