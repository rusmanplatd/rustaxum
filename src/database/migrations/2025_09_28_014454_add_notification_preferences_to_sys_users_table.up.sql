-- Add new consolidated notification preference columns with better naming
ALTER TABLE sys_users
ADD COLUMN push_notifications BOOLEAN DEFAULT true NOT NULL,
ADD COLUMN system_updates BOOLEAN DEFAULT true NOT NULL,
ADD COLUMN mention_notifications BOOLEAN DEFAULT true NOT NULL,
ADD COLUMN comment_notifications BOOLEAN DEFAULT true NOT NULL,
ADD COLUMN follow_notifications BOOLEAN DEFAULT true NOT NULL;

-- Add comments for documentation
COMMENT ON COLUMN sys_users.push_notifications IS 'User preference for receiving push notifications';
COMMENT ON COLUMN sys_users.system_updates IS 'User preference for receiving system update notifications';
COMMENT ON COLUMN sys_users.mention_notifications IS 'User preference for receiving mention notifications';
COMMENT ON COLUMN sys_users.comment_notifications IS 'User preference for receiving comment notifications';
COMMENT ON COLUMN sys_users.follow_notifications IS 'User preference for receiving follow notifications';