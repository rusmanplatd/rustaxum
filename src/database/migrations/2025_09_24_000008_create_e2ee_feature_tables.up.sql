-- Create polls table for encrypted polling feature
CREATE TABLE polls (
    id CHAR(26) PRIMARY KEY,
    message_id CHAR(26) NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    conversation_id CHAR(26) NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,

    -- Encrypted poll data (question and options encrypted client-side)
    encrypted_question TEXT NOT NULL,
    encrypted_options TEXT NOT NULL, -- JSON array of options, encrypted

    -- Poll settings
    allows_multiple_votes BOOLEAN NOT NULL DEFAULT false,
    is_anonymous BOOLEAN NOT NULL DEFAULT false,
    expires_at TIMESTAMPTZ,
    is_closed BOOLEAN NOT NULL DEFAULT false,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table comment for polls
COMMENT ON TABLE polls IS 'Stores encrypted polls within E2EE conversations. Poll questions and options are encrypted client-side to maintain privacy. Supports various poll types including anonymous voting and multiple choice, while preserving voting privacy through encryption.';

-- Column comments for polls table
COMMENT ON COLUMN polls.id IS 'ULID primary key uniquely identifying this encrypted poll';
COMMENT ON COLUMN polls.message_id IS 'Foreign key to message containing this poll - polls are delivered as special message types in E2EE conversations';
COMMENT ON COLUMN polls.conversation_id IS 'Foreign key to conversation - enables efficient poll queries and access control validation';
COMMENT ON COLUMN polls.encrypted_question IS 'Poll question encrypted with conversation key - only participants with access to conversation keys can decrypt and view the question';
COMMENT ON COLUMN polls.encrypted_options IS 'JSON array of poll options encrypted with conversation key - maintains option privacy until participants decrypt locally';
COMMENT ON COLUMN polls.allows_multiple_votes IS 'Whether participants can vote for multiple options - setting is public metadata not requiring encryption';
COMMENT ON COLUMN polls.is_anonymous IS 'Whether votes are anonymous even to poll creator - when true, voting data includes additional privacy measures';
COMMENT ON COLUMN polls.expires_at IS 'Optional poll expiration timestamp - polls can have time limits for voting participation';
COMMENT ON COLUMN polls.is_closed IS 'Whether poll is closed to new votes - can be manually closed or automatically closed on expiration';
COMMENT ON COLUMN polls.created_at IS 'Poll creation timestamp for audit and ordering';
COMMENT ON COLUMN polls.updated_at IS 'Last modification timestamp for poll settings updates';

-- Create poll votes table for encrypted vote storage
CREATE TABLE poll_votes (
    id CHAR(26) PRIMARY KEY,
    poll_id CHAR(26) NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Encrypted vote data (which option(s) were selected)
    encrypted_vote_data TEXT NOT NULL,
    vote_algorithm VARCHAR NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table comment for poll_votes
COMMENT ON TABLE poll_votes IS 'Stores encrypted votes for polls in E2EE conversations. Vote selections are encrypted to preserve voting privacy, supporting both identified and anonymous voting modes. Each device can vote independently for multi-device user scenarios.';

-- Column comments for poll_votes table
COMMENT ON COLUMN poll_votes.id IS 'ULID primary key uniquely identifying this encrypted vote';
COMMENT ON COLUMN poll_votes.poll_id IS 'Foreign key to the poll being voted on - enables vote aggregation and poll result calculation';
COMMENT ON COLUMN poll_votes.user_id IS 'Foreign key to voting user - used for vote validation and duplicate prevention (may be anonymized for anonymous polls)';
COMMENT ON COLUMN poll_votes.device_id IS 'Foreign key to voting device - enables multi-device voting scenarios where each device can vote independently';
COMMENT ON COLUMN poll_votes.encrypted_vote_data IS 'Encrypted selection data indicating which poll option(s) were chosen - decryptable only by conversation participants';
COMMENT ON COLUMN poll_votes.vote_algorithm IS 'Algorithm used to encrypt the vote data - enables proper decryption and vote tallying by authorized parties';
COMMENT ON COLUMN poll_votes.created_at IS 'Vote submission timestamp for audit and chronological ordering';
COMMENT ON COLUMN poll_votes.updated_at IS 'Last vote modification timestamp for vote changes (if allowed by poll settings)';

-- Create pinned messages table
CREATE TABLE pinned_messages (
    id CHAR(26) PRIMARY KEY,
    conversation_id CHAR(26) NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    message_id CHAR(26) NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    pinned_by_user_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    pinned_by_device_id CHAR(26) NOT NULL REFERENCES devices(id),

    pinned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    unpinned_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true
);

-- Table comment for pinned_messages
COMMENT ON TABLE pinned_messages IS 'Manages pinned messages in E2EE conversations. Pinned messages remain accessible and highlighted for conversation participants. The encrypted message content remains protected while pinning metadata provides conversation organization features.';

-- Column comments for pinned_messages table
COMMENT ON COLUMN pinned_messages.id IS 'ULID primary key uniquely identifying this message pin action';
COMMENT ON COLUMN pinned_messages.conversation_id IS 'Foreign key to conversation containing the pinned message - enables efficient pinned message queries per conversation';
COMMENT ON COLUMN pinned_messages.message_id IS 'Foreign key to the encrypted message being pinned - message content remains encrypted while pin status is metadata';
COMMENT ON COLUMN pinned_messages.pinned_by_user_id IS 'User who pinned the message - requires appropriate conversation permissions to pin messages';
COMMENT ON COLUMN pinned_messages.pinned_by_device_id IS 'Device from which pinning action was performed - provides audit trail for pin operations';
COMMENT ON COLUMN pinned_messages.pinned_at IS 'Timestamp when message was pinned - used for pin chronological ordering and audit';
COMMENT ON COLUMN pinned_messages.unpinned_at IS 'Timestamp when message was unpinned - null for currently pinned messages';
COMMENT ON COLUMN pinned_messages.is_active IS 'Whether message is currently pinned - enables efficient queries for active pins while preserving pin history';

-- Create message mentions table
CREATE TABLE message_mentions (
    id CHAR(26) PRIMARY KEY,
    message_id CHAR(26) NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    mentioned_user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,

    -- Mention type: 'user', 'everyone', 'here'
    mention_type VARCHAR NOT NULL DEFAULT 'user' CHECK (mention_type IN ('user', 'everyone', 'here')),

    -- Position in message for highlighting (encrypted content positions)
    mention_start_pos INTEGER,
    mention_length INTEGER,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table comment for message_mentions
COMMENT ON TABLE message_mentions IS 'Tracks user mentions within encrypted messages. While message content is encrypted, mention metadata enables notification delivery and UI highlighting. Position data helps clients highlight mentions after local decryption.';

-- Column comments for message_mentions table
COMMENT ON COLUMN message_mentions.id IS 'ULID primary key uniquely identifying this mention';
COMMENT ON COLUMN message_mentions.message_id IS 'Foreign key to encrypted message containing the mention - enables mention-based notifications';
COMMENT ON COLUMN message_mentions.mentioned_user_id IS 'User being mentioned - enables targeted notification delivery without exposing message content';
COMMENT ON COLUMN message_mentions.mention_type IS 'Type of mention: user (specific user), everyone (all participants), here (active participants) - affects notification scope';
COMMENT ON COLUMN message_mentions.mention_start_pos IS 'Character position where mention starts in decrypted message content - enables client-side highlighting after decryption';
COMMENT ON COLUMN message_mentions.mention_length IS 'Length of mention text in decrypted content - used with start_pos for accurate highlighting';
COMMENT ON COLUMN message_mentions.created_at IS 'Mention creation timestamp for notification ordering';

-- Create forward history table to track message forwarding chains
CREATE TABLE forward_history (
    id CHAR(26) PRIMARY KEY,
    message_id CHAR(26) NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    original_message_id CHAR(26) NOT NULL REFERENCES messages(id),
    forwarded_by_user_id CHAR(26) NOT NULL REFERENCES sys_users(id),
    forwarded_by_device_id CHAR(26) NOT NULL REFERENCES devices(id),

    -- Forward chain depth (to limit forwarding)
    forward_depth INTEGER NOT NULL DEFAULT 1,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table comment for forward_history
COMMENT ON TABLE forward_history IS 'Tracks message forwarding chains in E2EE conversations. Records forwarding relationships while preserving encryption. Helps prevent excessive forwarding and provides forwarding attribution without exposing message content.';

-- Column comments for forward_history table
COMMENT ON COLUMN forward_history.id IS 'ULID primary key uniquely identifying this forward action';
COMMENT ON COLUMN forward_history.message_id IS 'Foreign key to the new forwarded message - the copy created in the destination conversation';
COMMENT ON COLUMN forward_history.original_message_id IS 'Foreign key to the original message being forwarded - maintains forwarding lineage';
COMMENT ON COLUMN forward_history.forwarded_by_user_id IS 'User who performed the forwarding action - provides attribution for forwarding behavior';
COMMENT ON COLUMN forward_history.forwarded_by_device_id IS 'Device from which forwarding was performed - enables device-specific forwarding audit';
COMMENT ON COLUMN forward_history.forward_depth IS 'Depth in forwarding chain - prevents excessive forwarding by limiting chain length';
COMMENT ON COLUMN forward_history.created_at IS 'Forward action timestamp for audit and chronological tracking';

-- Create message reactions table for encrypted reactions
CREATE TABLE message_reactions (
    id CHAR(26) PRIMARY KEY,
    message_id CHAR(26) NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id CHAR(26) NOT NULL REFERENCES sys_users(id) ON DELETE CASCADE,
    device_id CHAR(26) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    -- Encrypted reaction data (emoji/reaction encrypted)
    encrypted_reaction TEXT NOT NULL,
    reaction_algorithm VARCHAR NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table comment for message_reactions
COMMENT ON TABLE message_reactions IS 'Stores encrypted reactions to messages in E2EE conversations. Reaction content (emojis/text) is encrypted to preserve privacy while enabling reaction aggregation and display. Supports multi-device scenarios.';

-- Column comments for message_reactions table
COMMENT ON COLUMN message_reactions.id IS 'ULID primary key uniquely identifying this encrypted reaction';
COMMENT ON COLUMN message_reactions.message_id IS 'Foreign key to message being reacted to - enables reaction aggregation per message';
COMMENT ON COLUMN message_reactions.user_id IS 'User who added the reaction - enables reaction attribution and duplicate prevention';
COMMENT ON COLUMN message_reactions.device_id IS 'Device from which reaction was added - supports multi-device reaction scenarios';
COMMENT ON COLUMN message_reactions.encrypted_reaction IS 'Encrypted reaction content (emoji or text) - only decryptable by conversation participants';
COMMENT ON COLUMN message_reactions.reaction_algorithm IS 'Algorithm used to encrypt the reaction data - enables proper decryption by authorized parties';
COMMENT ON COLUMN message_reactions.created_at IS 'Reaction creation timestamp for chronological ordering';
COMMENT ON COLUMN message_reactions.updated_at IS 'Last reaction update timestamp for reaction modifications';

-- Indexes for polls
CREATE INDEX idx_polls_message ON polls (message_id);
COMMENT ON INDEX idx_polls_message IS 'Optimizes queries linking polls to their containing messages - enables efficient poll content retrieval';

CREATE INDEX idx_polls_conversation ON polls (conversation_id);
COMMENT ON INDEX idx_polls_conversation IS 'Optimizes conversation-level poll queries - supports poll listing and conversation-wide poll management';

CREATE INDEX idx_polls_expires ON polls (expires_at) WHERE expires_at IS NOT NULL;
COMMENT ON INDEX idx_polls_expires IS 'Partial index for polls with expiration - optimizes cleanup jobs and expiration processing';

-- Indexes for poll votes
CREATE UNIQUE INDEX idx_poll_votes_user_poll ON poll_votes (poll_id, user_id, device_id);
COMMENT ON INDEX idx_poll_votes_user_poll IS 'Enforces one vote per poll per user per device - prevents duplicate voting while supporting multi-device scenarios';

CREATE INDEX idx_poll_votes_poll ON poll_votes (poll_id);
COMMENT ON INDEX idx_poll_votes_poll IS 'Optimizes vote aggregation queries per poll - enables efficient vote counting and result calculation';

CREATE INDEX idx_poll_votes_user ON poll_votes (user_id);
COMMENT ON INDEX idx_poll_votes_user IS 'Optimizes user-centric vote queries - supports user voting history and participation tracking';

-- Indexes for pinned messages
CREATE UNIQUE INDEX idx_pinned_messages_unique ON pinned_messages (conversation_id, message_id) WHERE is_active = true;
COMMENT ON INDEX idx_pinned_messages_unique IS 'Partial unique index preventing duplicate active pins per message per conversation - maintains pin consistency';

CREATE INDEX idx_pinned_messages_conversation ON pinned_messages (conversation_id, is_active);
COMMENT ON INDEX idx_pinned_messages_conversation IS 'Optimizes queries for active pinned messages in conversations - supports pinned message display';

CREATE INDEX idx_pinned_messages_message ON pinned_messages (message_id);
COMMENT ON INDEX idx_pinned_messages_message IS 'Optimizes message-centric pin queries - enables pin status checking and pin history';

-- Indexes for mentions
CREATE INDEX idx_message_mentions_message ON message_mentions (message_id);
COMMENT ON INDEX idx_message_mentions_message IS 'Optimizes mention queries per message - supports mention highlighting and notification processing';

CREATE INDEX idx_message_mentions_user ON message_mentions (mentioned_user_id);
COMMENT ON INDEX idx_message_mentions_user IS 'Optimizes user mention queries - enables efficient mention notification delivery and mention history';

CREATE INDEX idx_message_mentions_type ON message_mentions (mention_type);
COMMENT ON INDEX idx_message_mentions_type IS 'Optimizes mention type queries - supports different notification handling for user vs everyone mentions';

-- Indexes for forward history
CREATE INDEX idx_forward_history_message ON forward_history (message_id);
COMMENT ON INDEX idx_forward_history_message IS 'Optimizes forward tracking per message - enables forward attribution and lineage tracking';

CREATE INDEX idx_forward_history_original ON forward_history (original_message_id);
COMMENT ON INDEX idx_forward_history_original IS 'Optimizes queries for forwarding chains from original messages - supports forward impact analysis';

CREATE INDEX idx_forward_history_depth ON forward_history (forward_depth);
COMMENT ON INDEX idx_forward_history_depth IS 'Optimizes forward depth queries - enables enforcement of forwarding limits and chain analysis';

-- Indexes for reactions
CREATE UNIQUE INDEX idx_message_reactions_unique ON message_reactions (message_id, user_id, encrypted_reaction);
COMMENT ON INDEX idx_message_reactions_unique IS 'Enforces unique reactions per message per user - prevents duplicate reactions while allowing reaction changes';

CREATE INDEX idx_message_reactions_message ON message_reactions (message_id);
COMMENT ON INDEX idx_message_reactions_message IS 'Optimizes reaction queries per message - enables efficient reaction aggregation and display';

CREATE INDEX idx_message_reactions_user ON message_reactions (user_id);
COMMENT ON INDEX idx_message_reactions_user IS 'Optimizes user reaction queries - supports user reaction history and activity tracking';