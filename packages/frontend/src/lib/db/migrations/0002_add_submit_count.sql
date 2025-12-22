ALTER TABLE "submissions" DROP CONSTRAINT "submissions_submission_hash_unique";--> statement-breakpoint
ALTER TABLE "submissions" ADD COLUMN "submit_count" integer DEFAULT 1 NOT NULL;--> statement-breakpoint
ALTER TABLE "submissions" ADD CONSTRAINT "submissions_user_hash_unique" UNIQUE("user_id","submission_hash");