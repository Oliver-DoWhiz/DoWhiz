-- Scheduled Jobs for TPM and other recurring tasks
--
-- Enables cron-based autonomous task execution without inbound triggers.
-- Used by job_runner to poll and execute due jobs.
--
-- Run this in Supabase SQL Editor or via psql.

CREATE TABLE IF NOT EXISTS scheduled_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,

    -- Job configuration
    employee_id TEXT NOT NULL,         -- Which employee runs this job
    job_type TEXT NOT NULL,            -- 'tpm_checkin', 'tpm_report', etc.
    cron_expression TEXT NOT NULL,     -- Standard 5-field cron: 'min hour day month weekday'
    timezone TEXT DEFAULT 'UTC',       -- Timezone for cron interpretation

    -- Job-specific configuration (JSON)
    config JSONB NOT NULL DEFAULT '{}',

    -- Execution tracking
    last_run_at TIMESTAMPTZ,           -- Last successful execution
    next_run_at TIMESTAMPTZ,           -- Pre-computed next run time
    last_error TEXT,                   -- Error from last failed run

    -- Status
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT valid_cron_expression CHECK (
        cron_expression ~ '^(\*|[0-9,\-\/]+)\s+(\*|[0-9,\-\/]+)\s+(\*|[0-9,\-\/]+)\s+(\*|[0-9,\-\/]+)\s+(\*|[0-9,\-\/]+)$'
    )
);

-- Index for job_runner polling
CREATE INDEX IF NOT EXISTS idx_scheduled_jobs_due
    ON scheduled_jobs (next_run_at, enabled)
    WHERE enabled = true;

-- Index for listing jobs by account
CREATE INDEX IF NOT EXISTS idx_scheduled_jobs_account
    ON scheduled_jobs (account_id, job_type);

-- Trigger to update updated_at
CREATE OR REPLACE FUNCTION update_scheduled_jobs_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_scheduled_jobs_updated_at
    BEFORE UPDATE ON scheduled_jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_scheduled_jobs_updated_at();

-- Enable Row Level Security
ALTER TABLE scheduled_jobs ENABLE ROW LEVEL SECURITY;

-- Policy: Users can view own jobs
CREATE POLICY "Users can view own jobs" ON scheduled_jobs
    FOR SELECT USING (
        account_id IN (
            SELECT id FROM accounts WHERE auth_user_id = auth.uid()
        )
    );

-- Policy: Users can create own jobs
CREATE POLICY "Users can create own jobs" ON scheduled_jobs
    FOR INSERT WITH CHECK (
        account_id IN (
            SELECT id FROM accounts WHERE auth_user_id = auth.uid()
        )
    );

-- Policy: Users can update own jobs
CREATE POLICY "Users can update own jobs" ON scheduled_jobs
    FOR UPDATE USING (
        account_id IN (
            SELECT id FROM accounts WHERE auth_user_id = auth.uid()
        )
    );

-- Policy: Users can delete own jobs
CREATE POLICY "Users can delete own jobs" ON scheduled_jobs
    FOR DELETE USING (
        account_id IN (
            SELECT id FROM accounts WHERE auth_user_id = auth.uid()
        )
    );

-- Service role can do anything
CREATE POLICY "Service role full access" ON scheduled_jobs
    FOR ALL USING (auth.role() = 'service_role');

COMMENT ON TABLE scheduled_jobs IS 'Cron-based scheduled jobs for TPM and other recurring tasks';
COMMENT ON COLUMN scheduled_jobs.cron_expression IS 'Standard 5-field cron: minute hour day month weekday';
COMMENT ON COLUMN scheduled_jobs.config IS 'Job-specific config, e.g., {"database_id": "...", "report_recipients": [...]}';
COMMENT ON COLUMN scheduled_jobs.next_run_at IS 'Pre-computed next execution time for efficient polling';

-- Example job types and their config schemas:
--
-- tpm_checkin:
--   config: {
--     "notion_database_id": "abc123",     -- Task board to monitor
--     "status_filter": ["In Progress"],   -- Only check tasks with these statuses
--     "assignee_property": "Assignee"     -- Property name for task owner
--   }
--
-- tpm_report:
--   config: {
--     "notion_database_id": "abc123",
--     "report_recipients": ["slack:U123", "discord:456"],
--     "report_type": "weekly_summary"
--   }
