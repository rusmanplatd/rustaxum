-- Add session state recovery and backup mechanisms to Signal Protocol tables
-- Enables device synchronization and session recovery for multi-device E2EE
-- Implements secure backup and restore of Signal Protocol session states

-- Add session backup and recovery fields to signal_sessions table
ALTER TABLE signal_sessions ADD COLUMN backup_encrypted_state TEXT;
COMMENT ON COLUMN signal_sessions.backup_encrypted_state IS 'Encrypted backup of session state for device synchronization - encrypted with user master key';

ALTER TABLE signal_sessions ADD COLUMN recovery_key_hash TEXT;
COMMENT ON COLUMN signal_sessions.recovery_key_hash IS 'Hash of recovery key used to encrypt backup state - enables secure session restoration';

ALTER TABLE signal_sessions ADD COLUMN backup_created_at TIMESTAMPTZ;
COMMENT ON COLUMN signal_sessions.backup_created_at IS 'When backup state was last created - used for backup staleness detection';

ALTER TABLE signal_sessions ADD COLUMN backup_device_id CHAR(26) REFERENCES devices(id);
COMMENT ON COLUMN signal_sessions.backup_device_id IS 'Device that created this session backup - for multi-device sync coordination';

ALTER TABLE signal_sessions ADD COLUMN is_recoverable BOOLEAN NOT NULL DEFAULT true;
COMMENT ON COLUMN signal_sessions.is_recoverable IS 'Whether session can be recovered by other devices - some sessions may be device-specific';

-- Create session recovery log table
-- Tracks session recovery attempts for security monitoring and debugging
CREATE TABLE session_recovery_log (
    id CHAR(26) PRIMARY KEY,
    session_id CHAR(26) NOT NULL REFERENCES signal_sessions(id) ON DELETE CASCADE,

    -- Recovery attempt details
    requesting_device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    recovery_method VARCHAR NOT NULL CHECK (recovery_method IN ('backup_restore', 'key_exchange', 'master_key', 'device_sync')),

    -- Recovery status
    recovery_status VARCHAR NOT NULL DEFAULT 'initiated' CHECK (recovery_status IN ('initiated', 'in_progress', 'completed', 'failed', 'cancelled')),

    -- Timing information
    initiated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    failure_reason TEXT,

    -- Security verification
    recovery_key_verified BOOLEAN NOT NULL DEFAULT false,
    device_authorized BOOLEAN NOT NULL DEFAULT false,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table and column comments for session recovery log
COMMENT ON TABLE session_recovery_log IS 'Audit log for Signal Protocol session recovery attempts - tracks multi-device sync and recovery for security monitoring';
COMMENT ON COLUMN session_recovery_log.id IS 'Unique recovery attempt identifier (ULID format)';
COMMENT ON COLUMN session_recovery_log.session_id IS 'Session being recovered - cascade deletes with session';
COMMENT ON COLUMN session_recovery_log.requesting_device_id IS 'Device requesting session recovery - must be authorized for user';
COMMENT ON COLUMN session_recovery_log.recovery_method IS 'Method used for recovery: backup_restore (encrypted backup), key_exchange (new X3DH), master_key (user master key), device_sync (cross-device sync)';
COMMENT ON COLUMN session_recovery_log.recovery_status IS 'Current status of recovery attempt - enables monitoring and retry logic';
COMMENT ON COLUMN session_recovery_log.initiated_at IS 'When recovery was requested';
COMMENT ON COLUMN session_recovery_log.completed_at IS 'When recovery completed successfully';
COMMENT ON COLUMN session_recovery_log.failed_at IS 'When recovery failed permanently';
COMMENT ON COLUMN session_recovery_log.failure_reason IS 'Detailed failure reason for debugging';
COMMENT ON COLUMN session_recovery_log.recovery_key_verified IS 'Whether recovery key was successfully verified';
COMMENT ON COLUMN session_recovery_log.device_authorized IS 'Whether requesting device is authorized for recovery';

-- Create device session backups table
-- Stores encrypted session backups for efficient multi-device synchronization
CREATE TABLE device_session_backups (
    id CHAR(26) PRIMARY KEY,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,

    -- Backup metadata
    backup_name VARCHAR NOT NULL,
    backup_type VARCHAR NOT NULL CHECK (backup_type IN ('full_sync', 'incremental', 'emergency', 'scheduled')),
    backup_version INTEGER NOT NULL DEFAULT 1,

    -- Encrypted backup data
    encrypted_sessions_data TEXT NOT NULL,
    backup_algorithm VARCHAR NOT NULL,
    backup_key_hash TEXT NOT NULL,

    -- Session statistics in backup
    sessions_count INTEGER NOT NULL DEFAULT 0,
    conversations_count INTEGER NOT NULL DEFAULT 0,

    -- Backup lifecycle
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '90 days',
    last_accessed_at TIMESTAMPTZ,

    -- Verification and integrity
    backup_checksum TEXT NOT NULL,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verification_failed_at TIMESTAMPTZ,

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table and column comments for device session backups
COMMENT ON TABLE device_session_backups IS 'Stores encrypted backups of Signal Protocol sessions for multi-device synchronization and recovery';
COMMENT ON COLUMN device_session_backups.id IS 'Unique backup identifier (ULID format)';
COMMENT ON COLUMN device_session_backups.device_id IS 'Device that created this backup';
COMMENT ON COLUMN device_session_backups.user_id IS 'User who owns these session backups';
COMMENT ON COLUMN device_session_backups.backup_name IS 'Human-readable backup name for user management';
COMMENT ON COLUMN device_session_backups.backup_type IS 'Type of backup: full_sync (complete), incremental (changes only), emergency (immediate), scheduled (automated)';
COMMENT ON COLUMN device_session_backups.backup_version IS 'Backup format version for backward compatibility';
COMMENT ON COLUMN device_session_backups.encrypted_sessions_data IS 'Encrypted session states and metadata - encrypted with user backup key';
COMMENT ON COLUMN device_session_backups.backup_algorithm IS 'Algorithm used to encrypt backup data';
COMMENT ON COLUMN device_session_backups.backup_key_hash IS 'Hash of key used to encrypt backup - for verification';
COMMENT ON COLUMN device_session_backups.sessions_count IS 'Number of sessions included in backup';
COMMENT ON COLUMN device_session_backups.conversations_count IS 'Number of conversations covered by backup';
COMMENT ON COLUMN device_session_backups.expires_at IS 'When backup expires and should be cleaned up';
COMMENT ON COLUMN device_session_backups.last_accessed_at IS 'Last time backup was accessed for recovery';
COMMENT ON COLUMN device_session_backups.backup_checksum IS 'Checksum for backup integrity verification';
COMMENT ON COLUMN device_session_backups.is_verified IS 'Whether backup integrity has been verified';
COMMENT ON COLUMN device_session_backups.verification_failed_at IS 'When backup verification failed';

-- Indexes for session recovery and backup queries
CREATE INDEX idx_signal_sessions_backup ON signal_sessions (backup_created_at, is_recoverable) WHERE backup_encrypted_state IS NOT NULL;
COMMENT ON INDEX idx_signal_sessions_backup IS 'Find sessions with available backups for recovery';

CREATE INDEX idx_signal_sessions_recovery_device ON signal_sessions (backup_device_id, backup_created_at);
COMMENT ON INDEX idx_signal_sessions_recovery_device IS 'Find sessions backed up by specific device';

CREATE INDEX idx_session_recovery_log_device ON session_recovery_log (requesting_device_id, recovery_status);
COMMENT ON INDEX idx_session_recovery_log_device IS 'Track recovery attempts by device';

CREATE INDEX idx_session_recovery_log_session ON session_recovery_log (session_id, initiated_at);
COMMENT ON INDEX idx_session_recovery_log_session IS 'Find recovery attempts for specific session';

CREATE INDEX idx_session_recovery_log_pending ON session_recovery_log (recovery_status, initiated_at) WHERE recovery_status IN ('initiated', 'in_progress');
COMMENT ON INDEX idx_session_recovery_log_pending IS 'Find pending recovery attempts that need processing';

CREATE INDEX idx_device_session_backups_device ON device_session_backups (device_id, backup_type);
COMMENT ON INDEX idx_device_session_backups_device IS 'Find backups by device and type';

CREATE INDEX idx_device_session_backups_user ON device_session_backups (user_id, created_at DESC);
COMMENT ON INDEX idx_device_session_backups_user IS 'Find user backups ordered by recency';

CREATE INDEX idx_device_session_backups_expired ON device_session_backups (expires_at) WHERE is_verified = true;
COMMENT ON INDEX idx_device_session_backups_expired IS 'Find verified backups ready for cleanup';

CREATE UNIQUE INDEX idx_device_session_backups_name ON device_session_backups (device_id, backup_name);
COMMENT ON INDEX idx_device_session_backups_name IS 'Ensure unique backup names per device';

-- Add indexes for session recovery performance
CREATE INDEX idx_signal_sessions_recoverable ON signal_sessions (is_recoverable, last_used_at) WHERE is_recoverable = true;
COMMENT ON INDEX idx_signal_sessions_recoverable IS 'Find recoverable sessions ordered by usage recency';