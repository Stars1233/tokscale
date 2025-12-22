-- Add reasoning_tokens column to submissions table
ALTER TABLE "submissions" ADD COLUMN IF NOT EXISTS "reasoning_tokens" bigint NOT NULL DEFAULT 0;
