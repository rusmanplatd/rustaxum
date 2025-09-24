-- Drop performance optimization indexes

-- Drop composite indexes for common join patterns
DROP INDEX CONCURRENTLY IF EXISTS idx_conversation_algorithm_negotiations_incomplete;
DROP INDEX CONCURRENTLY IF EXISTS idx_devices_user_registration;
DROP INDEX CONCURRENTLY IF EXISTS idx_messages_sender_conversation;

-- Drop reaction optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_message_reactions_message_created;

-- Drop forward history optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_forward_history_original_depth;

-- Drop mention optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_message_mentions_user_recent;

-- Drop pinned message optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_pinned_messages_conversation_active_pinned;

-- Drop poll optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_poll_votes_poll_created;
DROP INDEX CONCURRENTLY IF EXISTS idx_polls_conversation_active;

-- Drop conversation optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_conversations_creator_public;
DROP INDEX CONCURRENTLY IF EXISTS idx_conversations_encrypted_type;

-- Drop backup optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_encrypted_backup_keys_user_recent;

-- Drop security incident optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_security_incidents_device_recent;
DROP INDEX CONCURRENTLY IF EXISTS idx_security_incidents_recent_unresolved;

-- Drop presence optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_device_presence_online_last_seen;

-- Drop typing indicator optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_typing_indicators_active_expires;

-- Drop message delivery optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_message_delivery_status_device_unread;
DROP INDEX CONCURRENTLY IF EXISTS idx_message_delivery_status_pending_retry;

-- Drop signal session optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_signal_sessions_conversation_active;
DROP INDEX CONCURRENTLY IF EXISTS idx_signal_sessions_active_last_used;

-- Drop prekey bundle optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_prekey_bundles_device_unused_created;

-- Drop conversation participant optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_conversation_participants_role_active;
DROP INDEX CONCURRENTLY IF EXISTS idx_conversation_participants_user_active_joined;

-- Drop device optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_devices_algorithms;
DROP INDEX CONCURRENTLY IF EXISTS idx_devices_user_active_last_seen;

-- Drop message optimization indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_messages_expiring;
DROP INDEX CONCURRENTLY IF EXISTS idx_messages_conversation_type_active;
DROP INDEX CONCURRENTLY IF EXISTS idx_messages_conversation_active;