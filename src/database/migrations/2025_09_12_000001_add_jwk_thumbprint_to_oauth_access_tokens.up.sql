-- Add JWK thumbprint column to oauth_access_tokens table for DPoP support (RFC 9449)
-- This enables binding access tokens to a specific public key for proof of possession

ALTER TABLE oauth_access_tokens
ADD COLUMN jwk_thumbprint VARCHAR(255) NULL;

-- Add index on jwk_thumbprint for efficient lookups during DPoP validation
CREATE INDEX idx_oauth_access_tokens_jwk_thumbprint
ON oauth_access_tokens(jwk_thumbprint)
WHERE jwk_thumbprint IS NOT NULL;

-- Add comment to document DPoP binding support
COMMENT ON COLUMN oauth_access_tokens.jwk_thumbprint
IS 'JWK thumbprint for DPoP (RFC 9449) proof of possession binding. NULL for Bearer tokens.';