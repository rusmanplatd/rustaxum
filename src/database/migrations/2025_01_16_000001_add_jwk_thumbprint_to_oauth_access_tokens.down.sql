-- Remove JWK thumbprint column from oauth_access_tokens table

-- Drop the index first
DROP INDEX IF EXISTS idx_oauth_access_tokens_jwk_thumbprint;

-- Drop the column
ALTER TABLE oauth_access_tokens
DROP COLUMN IF EXISTS jwk_thumbprint;