-- Drop E2EE feature tables in reverse dependency order
DROP TABLE message_reactions;
DROP TABLE forward_history;
DROP TABLE message_mentions;
DROP TABLE pinned_messages;
DROP TABLE poll_votes;
DROP TABLE polls;