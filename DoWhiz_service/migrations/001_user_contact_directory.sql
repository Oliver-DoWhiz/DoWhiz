-- User Contact Directory for TPM Cross-Channel Messaging
--
-- Maps Notion task assignees to their Slack/Discord handles for proactive outreach.
-- Used by TPM agents to contact team members about task progress.
--
-- Run this in Supabase SQL Editor or via psql.

CREATE TABLE IF NOT EXISTS user_contact_directory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,

    -- Notion identity
    notion_user_id TEXT,               -- Notion person ID (from task assignee)
    notion_workspace_id TEXT,          -- Which Notion workspace

    -- Slack identity
    slack_user_id TEXT,                -- Slack member ID (e.g., U12345ABC)
    slack_workspace_id TEXT,           -- Slack team ID

    -- Discord identity
    discord_user_id TEXT,              -- Discord user ID (snowflake)
    discord_guild_id TEXT,             -- Discord server ID

    -- Contact preferences
    preferred_channel TEXT,            -- 'slack', 'discord', or 'email'
    contact_frequency_days INTEGER,    -- How often to check in (days)

    -- Tracking
    last_contacted_at TIMESTAMPTZ,     -- Last TPM follow-up time
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure unique mapping per account+workspace+notion_user
    CONSTRAINT unique_notion_user_per_workspace
        UNIQUE (account_id, notion_user_id, notion_workspace_id)
);

-- Index for fast lookup by Notion user
CREATE INDEX IF NOT EXISTS idx_user_contact_notion_lookup
    ON user_contact_directory (account_id, notion_workspace_id, notion_user_id);

-- Index for finding contacts due for follow-up
CREATE INDEX IF NOT EXISTS idx_user_contact_due_followup
    ON user_contact_directory (account_id, last_contacted_at, contact_frequency_days)
    WHERE contact_frequency_days IS NOT NULL;

-- Enable Row Level Security
ALTER TABLE user_contact_directory ENABLE ROW LEVEL SECURITY;

-- Policy: Users can only see their own contacts
CREATE POLICY "Users can view own contacts" ON user_contact_directory
    FOR SELECT USING (
        account_id IN (
            SELECT id FROM accounts WHERE auth_user_id = auth.uid()
        )
    );

-- Policy: Users can insert their own contacts
CREATE POLICY "Users can insert own contacts" ON user_contact_directory
    FOR INSERT WITH CHECK (
        account_id IN (
            SELECT id FROM accounts WHERE auth_user_id = auth.uid()
        )
    );

-- Policy: Users can update their own contacts
CREATE POLICY "Users can update own contacts" ON user_contact_directory
    FOR UPDATE USING (
        account_id IN (
            SELECT id FROM accounts WHERE auth_user_id = auth.uid()
        )
    );

-- Policy: Users can delete their own contacts
CREATE POLICY "Users can delete own contacts" ON user_contact_directory
    FOR DELETE USING (
        account_id IN (
            SELECT id FROM accounts WHERE auth_user_id = auth.uid()
        )
    );

-- Service role can do anything (for backend operations)
CREATE POLICY "Service role full access" ON user_contact_directory
    FOR ALL USING (auth.role() = 'service_role');

COMMENT ON TABLE user_contact_directory IS 'Maps Notion users to Slack/Discord handles for TPM follow-ups';
COMMENT ON COLUMN user_contact_directory.notion_user_id IS 'Notion person ID from task assignee field';
COMMENT ON COLUMN user_contact_directory.preferred_channel IS 'Contact channel preference: slack, discord, or email';
COMMENT ON COLUMN user_contact_directory.contact_frequency_days IS 'Days between automated check-ins (null = no automation)';
