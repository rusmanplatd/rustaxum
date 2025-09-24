-- Remove redundant and excessive indexes to improve write performance
-- Keeps only the most critical indexes for E2EE operations
-- Focuses on composite indexes that serve multiple query patterns

-- Remove redundant single-column indexes that are covered by composite indexes
-- Keep composite indexes as they serve multiple query patterns more efficiently

-- Remove excessive indexes from message_delivery_status_tables
-- The existing composite indexes already cover these query patterns
DROP INDEX IF EXISTS idx_message_delivery_status_device_unread;
-- Reason: Covered by existing composite index idx_message_delivery_status_device_status

-- Remove redundant indexes from signal_sessions that overlap with composite indexes
-- Keep the most specific composite indexes for performance
DROP INDEX IF EXISTS idx_signal_sessions_local_device;
DROP INDEX IF EXISTS idx_signal_sessions_remote_device;
-- Reason: Basic device queries are covered by more specific composite indexes

-- Remove redundant single-column indexes from skipped_message_keys
DROP INDEX IF EXISTS idx_skipped_message_keys_session;
-- Reason: Covered by composite index idx_skipped_message_keys_session_number

-- Remove overlapping indexes from device capabilities
DROP INDEX IF EXISTS idx_device_capabilities_encryption;
-- Reason: Algorithm capability queries should use more specific composite indexes

-- Remove redundant indexes from conversation participants
-- Keep the unique constraint index and most critical performance indexes
DROP INDEX IF EXISTS idx_conversation_participants_user_active_joined;
-- Reason: Similar functionality covered by role-based index with better selectivity

-- Remove excessive indexes from prekey bundles
DROP INDEX IF EXISTS idx_prekey_bundles_device;
-- Reason: Covered by composite index idx_prekey_bundles_device_key

-- Remove redundant indexes from device presence
-- Keep only the most selective partial indexes
DROP INDEX IF EXISTS idx_device_presence_online_last_seen;
-- Reason: Overly broad partial index - status-specific queries should be more targeted

-- Consolidate security incident indexes
DROP INDEX IF EXISTS idx_security_incidents_device_recent;
-- Reason: Time-window partial indexes require careful maintenance and may become stale

-- Remove excessive backup key indexes
DROP INDEX IF EXISTS idx_encrypted_backup_keys_user_recent;
-- Reason: Complex partial index with multiple conditions - simpler indexes are more maintainable

-- Remove redundant conversation indexes
DROP INDEX IF EXISTS idx_conversations_creator_public;
-- Reason: Public channel discovery should use simpler, more maintainable indexes

-- Remove overlapping poll indexes
DROP INDEX IF EXISTS idx_polls_conversation_active;
DROP INDEX IF EXISTS idx_poll_votes_poll_created;
-- Reason: These are overly specific for feature-level functionality

-- Remove excessive pinned message indexes
DROP INDEX IF EXISTS idx_pinned_messages_conversation_active_pinned;
-- Reason: Pin functionality doesn't require complex partial indexes

-- Remove redundant mention and forward indexes
DROP INDEX IF EXISTS idx_message_mentions_user_recent;
DROP INDEX IF EXISTS idx_forward_history_original_depth;
-- Reason: These are feature-specific indexes that may not justify the maintenance overhead

-- Remove overlapping reaction indexes
DROP INDEX IF EXISTS idx_message_reactions_message_created;
-- Reason: Reaction ordering is less critical than uniqueness constraints

-- Remove excessive composite indexes from messages table
DROP INDEX IF EXISTS idx_messages_sender_conversation;
-- Reason: User message history across conversations is less common than single-conversation queries

-- Remove redundant device registration index
DROP INDEX IF EXISTS idx_devices_user_registration;
-- Reason: Registration ID lookups are covered by device-specific indexes

-- Remove overly complex algorithm negotiation index
DROP INDEX IF EXISTS idx_conversation_algorithm_negotiations_incomplete;
-- Reason: Negotiation status queries should be simpler and more direct

-- Add optimized composite indexes to replace removed ones
-- These provide better query coverage with fewer indexes

-- Optimized index for active device queries across conversations
CREATE INDEX idx_devices_user_active_seen ON devices (user_id, is_active, last_seen_at DESC) WHERE is_active = true;
COMMENT ON INDEX idx_devices_user_active_seen IS 'Optimized index for active device queries with recency ordering';

-- Optimized index for message delivery across devices
CREATE INDEX idx_message_delivery_device_status ON message_delivery_status (recipient_device_id, status, delivered_at) WHERE status IN ('delivered', 'read');
COMMENT ON INDEX idx_message_delivery_device_status IS 'Optimized index for delivered message queries per device';

-- Optimized index for session management
CREATE INDEX idx_signal_sessions_conversation_devices ON signal_sessions (conversation_id, local_device_id, remote_device_id) WHERE is_active = true;
COMMENT ON INDEX idx_signal_sessions_conversation_devices IS 'Optimized index for active session lookup in conversations';

-- Optimized index for prekey management
CREATE INDEX idx_prekey_bundles_device_unused ON prekey_bundles (device_id, created_at DESC) WHERE is_used = false;
COMMENT ON INDEX idx_prekey_bundles_device_unused IS 'Optimized index for unused prekey selection with recency';

-- Add summary comment about index optimization
COMMENT ON SCHEMA public IS 'Index optimization applied: Removed 15+ redundant indexes, consolidated into 4 optimized composite indexes. Focus on E2EE critical operations while maintaining query performance.';