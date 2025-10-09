-- Add support for Email OTP, WebAuthn (Physical Keys), and Biometric MFA methods

-- Create Email OTP table for temporary codes
CREATE TABLE mfa_email_codes (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    code VARCHAR(10) NOT NULL,
    code_hash TEXT NOT NULL, -- SHA256 hash of the code
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,
    is_used BOOLEAN NOT NULL DEFAULT false,
    ip_address TEXT,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create WebAuthn credentials table (FIDO2/Physical Keys)
CREATE TABLE mfa_webauthn_credentials (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    credential_id TEXT NOT NULL UNIQUE, -- Base64 encoded credential ID
    public_key TEXT NOT NULL, -- COSE public key
    counter BIGINT NOT NULL DEFAULT 0, -- Signature counter for replay protection
    device_name VARCHAR(255), -- User-friendly name (e.g., "YubiKey 5")
    aaguid TEXT, -- Authenticator Attestation GUID
    transports TEXT[], -- Supported transports (usb, nfc, ble, internal)
    attestation_format VARCHAR(50), -- packed, fido-u2f, android-key, etc.
    is_backup_eligible BOOLEAN NOT NULL DEFAULT false,
    is_backup_state BOOLEAN NOT NULL DEFAULT false,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Create Biometric authentication table
CREATE TABLE mfa_biometric_credentials (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    device_id CHAR(26), -- Link to devices table if available
    biometric_type VARCHAR(50) NOT NULL CHECK (biometric_type IN ('fingerprint', 'face', 'iris', 'voice')),
    credential_id TEXT NOT NULL, -- Platform-specific credential identifier
    public_key TEXT NOT NULL, -- Public key for verification
    platform VARCHAR(50) NOT NULL, -- ios, android, windows, macos, linux
    device_name VARCHAR(255), -- User-friendly device name
    is_platform_authenticator BOOLEAN NOT NULL DEFAULT true,
    counter BIGINT NOT NULL DEFAULT 0,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Create WebAuthn challenges table (for registration and authentication)
CREATE TABLE mfa_webauthn_challenges (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    challenge TEXT NOT NULL, -- Base64 encoded challenge
    challenge_type VARCHAR(20) NOT NULL CHECK (challenge_type IN ('registration', 'authentication')),
    expires_at TIMESTAMPTZ NOT NULL,
    is_used BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Update mfa_methods to support new method types
ALTER TABLE mfa_methods
DROP CONSTRAINT IF EXISTS mfa_methods_method_type_check;

ALTER TABLE mfa_methods
ADD CONSTRAINT mfa_methods_method_type_check
CHECK (method_type IN ('totp', 'backup_codes', 'email', 'webauthn', 'biometric'));

-- Add metadata column for method-specific data
ALTER TABLE mfa_methods
ADD COLUMN IF NOT EXISTS metadata JSONB;

-- Create indexes for performance
CREATE INDEX idx_mfa_email_codes_user_id ON mfa_email_codes(user_id);
CREATE INDEX idx_mfa_email_codes_expires ON mfa_email_codes(expires_at) WHERE is_used = false;
CREATE INDEX idx_mfa_webauthn_credentials_user_id ON mfa_webauthn_credentials(user_id);
CREATE INDEX idx_mfa_webauthn_credentials_credential_id ON mfa_webauthn_credentials(credential_id);
CREATE INDEX idx_mfa_biometric_credentials_user_id ON mfa_biometric_credentials(user_id);
CREATE INDEX idx_mfa_biometric_credentials_device ON mfa_biometric_credentials(device_id);
CREATE INDEX idx_mfa_webauthn_challenges_user_id ON mfa_webauthn_challenges(user_id);
CREATE INDEX idx_mfa_webauthn_challenges_expires ON mfa_webauthn_challenges(expires_at) WHERE is_used = false;

-- Create cleanup function for expired codes and challenges
CREATE OR REPLACE FUNCTION cleanup_expired_mfa_data()
RETURNS void AS $$
BEGIN
    -- Delete expired email codes (older than 24 hours)
    DELETE FROM mfa_email_codes
    WHERE expires_at < NOW() - INTERVAL '24 hours';

    -- Delete expired WebAuthn challenges (older than 1 hour)
    DELETE FROM mfa_webauthn_challenges
    WHERE expires_at < NOW() - INTERVAL '1 hour';
END;
$$ LANGUAGE plpgsql;

-- Add comments for documentation
COMMENT ON TABLE mfa_email_codes IS 'Stores temporary OTP codes sent via email';
COMMENT ON TABLE mfa_webauthn_credentials IS 'Stores FIDO2/WebAuthn physical security keys and platform authenticators';
COMMENT ON TABLE mfa_biometric_credentials IS 'Stores biometric authentication credentials (fingerprint, face, etc)';
COMMENT ON TABLE mfa_webauthn_challenges IS 'Temporary storage for WebAuthn registration and authentication challenges';
COMMENT ON COLUMN mfa_webauthn_credentials.counter IS 'Signature counter prevents credential cloning attacks';
COMMENT ON COLUMN mfa_webauthn_credentials.transports IS 'How the authenticator communicates: usb, nfc, ble, internal';
COMMENT ON COLUMN mfa_biometric_credentials.is_platform_authenticator IS 'True if built into device (TouchID, FaceID), false if external';
