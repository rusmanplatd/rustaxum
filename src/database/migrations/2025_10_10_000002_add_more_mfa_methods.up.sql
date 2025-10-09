-- Add support for SMS OTP, Push Notifications, and Backup Email MFA methods

-- Create SMS OTP codes table
CREATE TABLE mfa_sms_codes (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    phone_number VARCHAR(20) NOT NULL,
    code VARCHAR(10) NOT NULL,
    code_hash TEXT NOT NULL, -- SHA256 hash of the code
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,
    is_used BOOLEAN NOT NULL DEFAULT false,
    send_attempts INTEGER NOT NULL DEFAULT 1,
    ip_address TEXT,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create Push Notification devices table
CREATE TABLE mfa_push_devices (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    device_token TEXT NOT NULL, -- FCM/APNS device token
    device_type VARCHAR(20) NOT NULL CHECK (device_type IN ('ios', 'android', 'web')),
    device_name VARCHAR(255),
    device_id TEXT, -- Unique device identifier
    platform_version VARCHAR(50), -- iOS version, Android version, etc.
    app_version VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Create Push Notification challenges table
CREATE TABLE mfa_push_challenges (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    device_id CHAR(26) REFERENCES mfa_push_devices(id) ON DELETE CASCADE,
    challenge TEXT NOT NULL, -- Random challenge string
    action_type VARCHAR(50) NOT NULL, -- login, transaction, sensitive_action
    action_details JSONB, -- Additional context about the action
    response VARCHAR(20), -- approved, denied, timeout
    responded_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    ip_address TEXT,
    location_data JSONB, -- Geolocation data
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create Backup Email table
CREATE TABLE mfa_backup_emails (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    backup_email VARCHAR(255) NOT NULL,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verification_token TEXT,
    verification_sent_at TIMESTAMPTZ,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(user_id, backup_email)
);

-- Create Backup Email codes table
CREATE TABLE mfa_backup_email_codes (
    id CHAR(26) PRIMARY KEY,
    backup_email_id CHAR(26) NOT NULL REFERENCES mfa_backup_emails(id) ON DELETE CASCADE,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    code VARCHAR(10) NOT NULL,
    code_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,
    is_used BOOLEAN NOT NULL DEFAULT false,
    ip_address TEXT,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create MFA recovery methods table (for account recovery)
CREATE TABLE mfa_recovery_methods (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    method_type VARCHAR(50) NOT NULL, -- security_questions, trusted_contacts, recovery_key
    method_data JSONB NOT NULL, -- Encrypted method-specific data
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create MFA sessions table (for "remember this device" functionality)
CREATE TABLE mfa_trusted_devices (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    device_fingerprint TEXT NOT NULL, -- Browser/device fingerprint
    device_name VARCHAR(255),
    ip_address TEXT,
    user_agent TEXT,
    trust_token TEXT NOT NULL UNIQUE, -- Secure token for device recognition
    expires_at TIMESTAMPTZ NOT NULL,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

-- Create MFA audit log table (comprehensive logging)
CREATE TABLE mfa_audit_log (
    id CHAR(26) PRIMARY KEY,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    method_type VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL, -- setup, verify, disable, challenge_sent, challenge_approved, etc.
    status VARCHAR(20) NOT NULL, -- success, failure, pending
    ip_address TEXT,
    user_agent TEXT,
    device_fingerprint TEXT,
    location_data JSONB,
    metadata JSONB, -- Additional context
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Update mfa_methods to support new method types
ALTER TABLE mfa_methods
DROP CONSTRAINT IF EXISTS mfa_methods_method_type_check;

ALTER TABLE mfa_methods
ADD CONSTRAINT mfa_methods_method_type_check
CHECK (method_type IN (
    'totp',
    'backup_codes',
    'email',
    'webauthn',
    'biometric',
    'sms',
    'push',
    'backup_email',
    'authenticator_app'
));

-- Add user preferences for MFA
ALTER TABLE sys_users
ADD COLUMN IF NOT EXISTS mfa_primary_method VARCHAR(50),
ADD COLUMN IF NOT EXISTS mfa_backup_method VARCHAR(50),
ADD COLUMN IF NOT EXISTS mfa_trust_device_enabled BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN IF NOT EXISTS mfa_trust_device_duration_days INTEGER DEFAULT 30;

-- Create indexes for performance
CREATE INDEX idx_mfa_sms_codes_user_id ON mfa_sms_codes(user_id);
CREATE INDEX idx_mfa_sms_codes_expires ON mfa_sms_codes(expires_at) WHERE is_used = false;
CREATE INDEX idx_mfa_sms_codes_phone ON mfa_sms_codes(phone_number);

CREATE INDEX idx_mfa_push_devices_user_id ON mfa_push_devices(user_id);
CREATE INDEX idx_mfa_push_devices_token ON mfa_push_devices(device_token);
CREATE INDEX idx_mfa_push_devices_active ON mfa_push_devices(user_id, is_active);

CREATE INDEX idx_mfa_push_challenges_user_id ON mfa_push_challenges(user_id);
CREATE INDEX idx_mfa_push_challenges_device_id ON mfa_push_challenges(device_id);
CREATE INDEX idx_mfa_push_challenges_expires ON mfa_push_challenges(expires_at);
CREATE INDEX idx_mfa_push_challenges_status ON mfa_push_challenges(user_id, response);

CREATE INDEX idx_mfa_backup_emails_user_id ON mfa_backup_emails(user_id);
CREATE INDEX idx_mfa_backup_emails_email ON mfa_backup_emails(backup_email);
CREATE INDEX idx_mfa_backup_emails_verified ON mfa_backup_emails(user_id, is_verified);

CREATE INDEX idx_mfa_backup_email_codes_user_id ON mfa_backup_email_codes(user_id);
CREATE INDEX idx_mfa_backup_email_codes_expires ON mfa_backup_email_codes(expires_at);

CREATE INDEX idx_mfa_recovery_methods_user_id ON mfa_recovery_methods(user_id);
CREATE INDEX idx_mfa_recovery_methods_type ON mfa_recovery_methods(user_id, method_type);

CREATE INDEX idx_mfa_trusted_devices_user_id ON mfa_trusted_devices(user_id);
CREATE INDEX idx_mfa_trusted_devices_token ON mfa_trusted_devices(trust_token);
CREATE INDEX idx_mfa_trusted_devices_fingerprint ON mfa_trusted_devices(device_fingerprint);
CREATE INDEX idx_mfa_trusted_devices_expires ON mfa_trusted_devices(expires_at);

CREATE INDEX idx_mfa_audit_log_user_id ON mfa_audit_log(user_id);
CREATE INDEX idx_mfa_audit_log_created_at ON mfa_audit_log(created_at);
CREATE INDEX idx_mfa_audit_log_method ON mfa_audit_log(method_type, action);

-- Create function to clean up expired data
CREATE OR REPLACE FUNCTION cleanup_expired_mfa_data_extended()
RETURNS void AS $$
BEGIN
    -- Delete expired SMS codes (older than 24 hours)
    DELETE FROM mfa_sms_codes
    WHERE expires_at < NOW() - INTERVAL '24 hours';

    -- Delete expired push challenges (older than 24 hours)
    DELETE FROM mfa_push_challenges
    WHERE expires_at < NOW() - INTERVAL '24 hours';

    -- Delete expired backup email codes (older than 24 hours)
    DELETE FROM mfa_backup_email_codes
    WHERE expires_at < NOW() - INTERVAL '24 hours';

    -- Delete expired trusted devices
    DELETE FROM mfa_trusted_devices
    WHERE expires_at < NOW() OR revoked_at IS NOT NULL;

    -- Archive old audit logs (older than 90 days) - optional
    -- DELETE FROM mfa_audit_log WHERE created_at < NOW() - INTERVAL '90 days';
END;
$$ LANGUAGE plpgsql;

-- Add comments for documentation
COMMENT ON TABLE mfa_sms_codes IS 'Stores temporary OTP codes sent via SMS';
COMMENT ON TABLE mfa_push_devices IS 'Stores mobile devices registered for push notification MFA';
COMMENT ON TABLE mfa_push_challenges IS 'Stores push notification authentication challenges';
COMMENT ON TABLE mfa_backup_emails IS 'Stores verified backup email addresses for account recovery';
COMMENT ON TABLE mfa_backup_email_codes IS 'Stores OTP codes sent to backup email addresses';
COMMENT ON TABLE mfa_recovery_methods IS 'Stores various account recovery methods';
COMMENT ON TABLE mfa_trusted_devices IS 'Stores trusted devices for "remember this device" functionality';
COMMENT ON TABLE mfa_audit_log IS 'Comprehensive audit log for all MFA activities';

COMMENT ON COLUMN mfa_push_challenges.action_type IS 'Type of action requiring approval: login, transaction, sensitive_action';
COMMENT ON COLUMN mfa_push_challenges.response IS 'User response: approved, denied, timeout';
COMMENT ON COLUMN mfa_trusted_devices.trust_token IS 'Secure token used to recognize trusted devices';
COMMENT ON COLUMN mfa_trusted_devices.device_fingerprint IS 'Browser/device fingerprint for device recognition';
