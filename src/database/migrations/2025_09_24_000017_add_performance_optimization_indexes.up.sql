-- Add performance optimization indexes for E2EE chat system
-- These indexes optimize common query patterns and improve system performance
-- Uses CONCURRENTLY to avoid blocking production traffic during index creation
-- Focus on partial indexes to optimize high-volume tables with frequent filtering

-- Optimize message queries with partial indexes for active (non-deleted) messages
CREATE INDEX idx_messages_conversation_active ON messages (conversation_id, sent_at DESC) WHERE is_deleted = false;
COMMENT ON INDEX idx_messages_conversation_active IS 'Primary message timeline query optimization - retrieves active messages in chronological order for conversation display';

CREATE INDEX idx_messages_conversation_type_active ON messages (conversation_id, message_type, sent_at DESC) WHERE is_deleted = false;
COMMENT ON INDEX idx_messages_conversation_type_active IS 'Optimizes filtered message queries by type (text, image, file, etc.) within conversations for specialized views';

CREATE INDEX idx_messages_expiring ON messages (expires_at) WHERE expires_at IS NOT NULL AND is_deleted = false;
COMMENT ON INDEX idx_messages_expiring IS 'Efficiently processes disappearing messages for automatic deletion - partial index on expiring messages only';

-- Optimize device queries for multi-device E2EE operations
CREATE INDEX idx_devices_user_active_last_seen ON devices (user_id, is_active, last_seen_at DESC) WHERE is_active = true;
COMMENT ON INDEX idx_devices_user_active_last_seen IS 'Optimizes active device lookup with presence ordering - critical for E2EE message delivery routing';

CREATE INDEX idx_devices_algorithms ON devices USING GIN (supported_algorithms);
COMMENT ON INDEX idx_devices_algorithms IS 'GIN index for efficient algorithm capability matching during E2EE negotiation and device compatibility checks';

-- Optimize conversation participant queries for membership and role management
CREATE INDEX idx_conversation_participants_user_active_joined ON conversation_participants (user_id, is_active, joined_at DESC) WHERE is_active = true;
COMMENT ON INDEX idx_conversation_participants_user_active_joined IS 'Optimizes user conversation listing with chronological ordering for dashboard and navigation';

CREATE INDEX idx_conversation_participants_role_active ON conversation_participants (conversation_id, role) WHERE is_active = true;
COMMENT ON INDEX idx_conversation_participants_role_active IS 'Optimizes role-based permission checks for E2EE conversation management and moderation';

-- Optimize prekey bundle queries for Signal Protocol key exchange
CREATE INDEX idx_prekey_bundles_device_unused_created ON prekey_bundles (device_id, created_at) WHERE is_used = false;
COMMENT ON INDEX idx_prekey_bundles_device_unused_created IS 'Optimizes unused prekey selection for X3DH key agreement - ordered by creation for proper key rotation';

-- Optimize Signal Protocol session queries for E2EE message encryption
CREATE INDEX idx_signal_sessions_active_last_used ON signal_sessions (is_active, last_used_at DESC) WHERE is_active = true;
COMMENT ON INDEX idx_signal_sessions_active_last_used IS 'Optimizes active session lookup with recency ordering for Double Ratchet message encryption';

CREATE INDEX idx_signal_sessions_conversation_active ON signal_sessions (conversation_id, is_active) WHERE is_active = true;
COMMENT ON INDEX idx_signal_sessions_conversation_active IS 'Optimizes conversation-specific session lookup for multi-device E2EE message routing';

-- Optimize message delivery tracking for multi-device E2EE delivery
CREATE INDEX idx_message_delivery_status_pending_retry ON message_delivery_status (next_retry_at) WHERE status = 'failed' AND retry_count < max_retries;
COMMENT ON INDEX idx_message_delivery_status_pending_retry IS 'Optimizes retry processing for failed E2EE message deliveries - partial index on retryable failures only';

CREATE INDEX idx_message_delivery_status_device_unread ON message_delivery_status (recipient_device_id, status) WHERE status IN ('delivered', 'sent');
COMMENT ON INDEX idx_message_delivery_status_device_unread IS 'Optimizes unread message counting per device for multi-device E2EE read receipt synchronization';

-- Optimize real-time features (short-lived data with frequent updates)
CREATE INDEX idx_typing_indicators_active_expires ON typing_indicators (conversation_id, expires_at) WHERE is_typing = true;
COMMENT ON INDEX idx_typing_indicators_active_expires IS 'Optimizes active typing indicator cleanup and expiration processing - partial index on active indicators only';

-- Optimize presence and availability queries for user experience
CREATE INDEX idx_device_presence_online_last_seen ON device_presence (status, last_seen_at DESC) WHERE status IN ('online', 'away', 'busy');
COMMENT ON INDEX idx_device_presence_online_last_seen IS 'Optimizes online presence queries for contact lists and availability display - partial index on visible states';

-- Optimize security monitoring and incident response
CREATE INDEX idx_security_incidents_recent_unresolved ON security_incidents (incident_type, severity, created_at DESC) WHERE is_resolved = false;
COMMENT ON INDEX idx_security_incidents_recent_unresolved IS 'Optimizes active security incident monitoring and alerting - partial index on unresolved incidents';

CREATE INDEX idx_security_incidents_device_recent ON security_incidents (device_id, created_at DESC);
COMMENT ON INDEX idx_security_incidents_device_recent IS 'Optimizes device-specific security monitoring - ordered by recency';

-- Optimize encrypted backup and key recovery queries
CREATE INDEX idx_encrypted_backup_keys_user_recent ON encrypted_backup_keys (user_id, backup_type, created_at DESC);
COMMENT ON INDEX idx_encrypted_backup_keys_user_recent IS 'Optimizes user backup retrieval with type filtering and recency ordering';

-- Optimize conversation discovery and management
CREATE INDEX idx_conversations_encrypted_type ON conversations (is_encrypted, conversation_type, created_at DESC);
COMMENT ON INDEX idx_conversations_encrypted_type IS 'Optimizes conversation listing by encryption status and type with chronological ordering';

CREATE INDEX idx_conversations_creator_public ON conversations (creator_id, is_public, created_at DESC) WHERE is_public = true;
COMMENT ON INDEX idx_conversations_creator_public IS 'Optimizes public channel discovery by creator - partial index on public channels only';

-- Optimize interactive features and engagement
CREATE INDEX idx_polls_conversation_active ON polls (conversation_id, is_closed, created_at DESC) WHERE is_closed = false;
COMMENT ON INDEX idx_polls_conversation_active IS 'Optimizes active poll display within conversations - partial index on open polls only';

CREATE INDEX idx_poll_votes_poll_created ON poll_votes (poll_id, created_at);
COMMENT ON INDEX idx_poll_votes_poll_created IS 'Optimizes poll vote counting and chronological vote display for results tallying';

CREATE INDEX idx_pinned_messages_conversation_active_pinned ON pinned_messages (conversation_id, pinned_at DESC) WHERE is_active = true;
COMMENT ON INDEX idx_pinned_messages_conversation_active_pinned IS 'Optimizes pinned message display with recency ordering - partial index on active pins only';

CREATE INDEX idx_message_mentions_user_recent ON message_mentions (mentioned_user_id, created_at DESC);
COMMENT ON INDEX idx_message_mentions_user_recent IS 'Optimizes user mention notifications and history with chronological ordering';

CREATE INDEX idx_forward_history_original_depth ON forward_history (original_message_id, forward_depth);
COMMENT ON INDEX idx_forward_history_original_depth IS 'Optimizes forward chain tracking and depth limit enforcement for message forwarding';

CREATE INDEX idx_message_reactions_message_created ON message_reactions (message_id, created_at DESC);
COMMENT ON INDEX idx_message_reactions_message_created IS 'Optimizes reaction display ordering and reaction history for individual messages';

-- Add composite indexes for common join patterns and complex queries
CREATE INDEX idx_messages_sender_conversation ON messages (sender_user_id, conversation_id, sent_at DESC) WHERE is_deleted = false;
COMMENT ON INDEX idx_messages_sender_conversation IS 'Optimizes user message history across conversations with chronological ordering - excludes deleted messages';

-- NOTE: idx_devices_user_registration already exists as UNIQUE index in devices table migration

-- Add specialized indexes for E2EE protocol operations
CREATE INDEX idx_conversation_algorithm_negotiations_incomplete ON conversation_algorithm_negotiations (conversation_id, negotiation_started_at) WHERE is_negotiation_complete = false;
COMMENT ON INDEX idx_conversation_algorithm_negotiations_incomplete IS 'Optimizes processing of incomplete algorithm negotiations - partial index for efficiency';