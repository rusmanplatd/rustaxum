-- Add performance optimization indexes for E2EE chat system
-- These indexes optimize common query patterns and improve system performance

-- Optimize message queries with partial indexes
CREATE INDEX CONCURRENTLY idx_messages_conversation_active ON messages (conversation_id, sent_at DESC) WHERE is_deleted = false;
CREATE INDEX CONCURRENTLY idx_messages_conversation_type_active ON messages (conversation_id, message_type, sent_at DESC) WHERE is_deleted = false;
CREATE INDEX CONCURRENTLY idx_messages_expiring ON messages (expires_at) WHERE expires_at IS NOT NULL AND is_deleted = false;

-- Optimize device queries
CREATE INDEX CONCURRENTLY idx_devices_user_active_last_seen ON devices (user_id, is_active, last_seen_at DESC) WHERE is_active = true;
CREATE INDEX CONCURRENTLY idx_devices_algorithms ON devices USING GIN (supported_algorithms);

-- Optimize conversation participant queries
CREATE INDEX CONCURRENTLY idx_conversation_participants_user_active_joined ON conversation_participants (user_id, is_active, joined_at DESC) WHERE is_active = true;
CREATE INDEX CONCURRENTLY idx_conversation_participants_role_active ON conversation_participants (conversation_id, role) WHERE is_active = true;

-- Optimize prekey bundle queries for unused keys
CREATE INDEX CONCURRENTLY idx_prekey_bundles_device_unused_created ON prekey_bundles (device_id, created_at) WHERE is_used = false;

-- Optimize signal session queries
CREATE INDEX CONCURRENTLY idx_signal_sessions_active_last_used ON signal_sessions (is_active, last_used_at DESC) WHERE is_active = true;
CREATE INDEX CONCURRENTLY idx_signal_sessions_conversation_active ON signal_sessions (conversation_id, is_active) WHERE is_active = true;

-- Optimize message delivery tracking
CREATE INDEX CONCURRENTLY idx_message_delivery_status_pending_retry ON message_delivery_status (next_retry_at) WHERE status = 'failed' AND retry_count < max_retries;
CREATE INDEX CONCURRENTLY idx_message_delivery_status_device_unread ON message_delivery_status (recipient_device_id, status) WHERE status IN ('delivered', 'sent');

-- Optimize typing indicators (short-lived data)
CREATE INDEX CONCURRENTLY idx_typing_indicators_active_expires ON typing_indicators (conversation_id, expires_at) WHERE is_typing = true;

-- Optimize presence queries
CREATE INDEX CONCURRENTLY idx_device_presence_online_last_seen ON device_presence (status, last_seen_at DESC) WHERE status IN ('online', 'away', 'busy');

-- Optimize security incident queries
CREATE INDEX CONCURRENTLY idx_security_incidents_recent_unresolved ON security_incidents (incident_type, severity, created_at DESC) WHERE is_resolved = false;
CREATE INDEX CONCURRENTLY idx_security_incidents_device_recent ON security_incidents (device_id, created_at DESC) WHERE created_at > NOW() - INTERVAL '30 days';

-- Optimize backup queries
CREATE INDEX CONCURRENTLY idx_encrypted_backup_keys_user_recent ON encrypted_backup_keys (user_id, backup_type, created_at DESC) WHERE expires_at > NOW();

-- Optimize conversation queries
CREATE INDEX CONCURRENTLY idx_conversations_encrypted_type ON conversations (is_encrypted, conversation_type, created_at DESC);
CREATE INDEX CONCURRENTLY idx_conversations_creator_public ON conversations (creator_id, is_public, created_at DESC) WHERE is_public = true;

-- Optimize poll queries
CREATE INDEX CONCURRENTLY idx_polls_conversation_active ON polls (conversation_id, is_closed, created_at DESC) WHERE is_closed = false;
CREATE INDEX CONCURRENTLY idx_poll_votes_poll_created ON poll_votes (poll_id, created_at);

-- Optimize pinned message queries
CREATE INDEX CONCURRENTLY idx_pinned_messages_conversation_active_pinned ON pinned_messages (conversation_id, pinned_at DESC) WHERE is_active = true;

-- Optimize mention queries
CREATE INDEX CONCURRENTLY idx_message_mentions_user_recent ON message_mentions (mentioned_user_id, created_at DESC);

-- Optimize forward history queries
CREATE INDEX CONCURRENTLY idx_forward_history_original_depth ON forward_history (original_message_id, forward_depth);

-- Optimize reaction queries
CREATE INDEX CONCURRENTLY idx_message_reactions_message_created ON message_reactions (message_id, created_at DESC);

-- Add composite indexes for common join patterns
CREATE INDEX CONCURRENTLY idx_messages_sender_conversation ON messages (sender_user_id, conversation_id, sent_at DESC) WHERE is_deleted = false;
CREATE INDEX CONCURRENTLY idx_devices_user_registration ON devices (user_id, registration_id) WHERE is_active = true;

-- Add partial indexes for algorithm negotiation
CREATE INDEX CONCURRENTLY idx_conversation_algorithm_negotiations_incomplete ON conversation_algorithm_negotiations (conversation_id, negotiation_started_at) WHERE is_negotiation_complete = false;