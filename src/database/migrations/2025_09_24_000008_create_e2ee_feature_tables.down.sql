-- Drop E2EE feature tables in reverse dependency order
DROP TABLE IF EXISTS message_reactions;
DROP TABLE IF EXISTS forward_history;
DROP TABLE IF EXISTS message_mentions;
DROP TABLE IF EXISTS pinned_messages;
DROP TABLE IF EXISTS poll_votes;
DROP TABLE IF EXISTS polls;