-- Revert the extension of oauth_clients table

DROP INDEX IF EXISTS idx_oauth_clients_certificate_bound;
DROP INDEX IF EXISTS idx_oauth_clients_require_par;
DROP INDEX IF EXISTS idx_oauth_clients_token_endpoint_auth_method;

ALTER TABLE oauth_clients
DROP COLUMN IF EXISTS certificate_bound_access_tokens,
DROP COLUMN IF EXISTS require_pushed_authorization_requests,
DROP COLUMN IF EXISTS default_max_age,
DROP COLUMN IF EXISTS require_auth_time,
DROP COLUMN IF EXISTS audience,
DROP COLUMN IF EXISTS scope,
DROP COLUMN IF EXISTS grant_types,
DROP COLUMN IF EXISTS response_types,
DROP COLUMN IF EXISTS token_endpoint_auth_method,
DROP COLUMN IF EXISTS jwks_uri,
DROP COLUMN IF EXISTS metadata,
DROP COLUMN IF EXISTS public_key_pem;
