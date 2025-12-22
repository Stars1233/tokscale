# Source-Level Merge Implementation Plan v2

## Problem Statement

When Claude Code cleans up local session files after 30 days, users who re-submit lose ALL historical data because the current implementation uses "full replacement mode":

```typescript
// packages/frontend/src/app/api/submit/route.ts (Line 88-89)
await db.delete(submissions).where(eq(submissions.userId, tokenRecord.userId));
```

## Solution

Implement source-level merge: only update sources present in submission, preserve other sources.

---

## Task 1: Create Helper Module

**File**: `packages/frontend/src/lib/db/helpers.ts` (NEW FILE)

**Purpose**: Centralized helper functions for source-level merge operations.

### Code

```typescript
/**
 * Source-level merge helpers for submission API
 */

// Type matching the JSONB structure in dailyBreakdown.sourceBreakdown
export interface SourceBreakdownData {
  tokens: number;
  cost: number;
  modelId: string;
  input: number;
  output: number;
  cacheRead: number;
  cacheWrite: number;
  messages: number;
}

// Type for recalculated day totals
export interface DayTotals {
  tokens: number;
  cost: number;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheCreationTokens: number;
}

/**
 * Recalculate day totals from merged sourceBreakdown
 * Sums all numeric fields across all sources
 */
export function recalculateDayTotals(
  sourceBreakdown: Record<string, SourceBreakdownData>
): DayTotals {
  let tokens = 0;
  let cost = 0;
  let inputTokens = 0;
  let outputTokens = 0;
  let cacheReadTokens = 0;
  let cacheCreationTokens = 0;

  for (const source of Object.values(sourceBreakdown)) {
    tokens += source.tokens;
    cost += source.cost;
    inputTokens += source.input;
    outputTokens += source.output;
    cacheReadTokens += source.cacheRead;
    cacheCreationTokens += source.cacheWrite;
  }

  return {
    tokens,
    cost,
    inputTokens,
    outputTokens,
    cacheReadTokens,
    cacheCreationTokens,
  };
}

/**
 * Merge sourceBreakdown objects with source-level granularity
 * 
 * CRITICAL: Only updates sources present in incomingSources set.
 * Sources NOT in incomingSources are PRESERVED from existing data.
 * 
 * @param existing - Current sourceBreakdown from database (may be null)
 * @param incoming - New sourceBreakdown from submission
 * @param incomingSources - Set of source names being submitted (e.g., ["claude", "cursor"])
 * @returns Merged sourceBreakdown object
 */
export function mergeSourceBreakdowns(
  existing: Record<string, SourceBreakdownData> | null | undefined,
  incoming: Record<string, SourceBreakdownData>,
  incomingSources: Set<string>
): Record<string, SourceBreakdownData> {
  // Start with existing data (or empty object if none)
  const merged: Record<string, SourceBreakdownData> = { ...(existing || {}) };

  // Only update sources that are in the incoming submission
  for (const sourceName of incomingSources) {
    if (incoming[sourceName]) {
      // Replace this source's data with new data
      merged[sourceName] = { ...incoming[sourceName] };
    }
  }

  return merged;
}

/**
 * Build modelBreakdown from sourceBreakdown
 * Aggregates tokens by modelId across all sources
 */
export function buildModelBreakdown(
  sourceBreakdown: Record<string, SourceBreakdownData>
): Record<string, number> {
  const modelBreakdown: Record<string, number> = {};

  for (const source of Object.values(sourceBreakdown)) {
    if (source.modelId) {
      modelBreakdown[source.modelId] =
        (modelBreakdown[source.modelId] || 0) + source.tokens;
    }
  }

  return modelBreakdown;
}

/**
 * Convert SourceContribution from CLI format to SourceBreakdownData for DB
 */
export function sourceContributionToBreakdownData(
  source: {
    tokens: { input: number; output: number; cacheRead: number; cacheWrite: number };
    cost: number;
    modelId: string;
    messages: number;
  }
): SourceBreakdownData {
  return {
    tokens: source.tokens.input + source.tokens.output,
    cost: source.cost,
    modelId: source.modelId,
    input: source.tokens.input,
    output: source.tokens.output,
    cacheRead: source.tokens.cacheRead,
    cacheWrite: source.tokens.cacheWrite,
    messages: source.messages,
  };
}
```

### Verification Criteria

- [ ] File compiles without TypeScript errors
- [ ] `recalculateDayTotals` returns all 6 fields (tokens, cost, inputTokens, outputTokens, cacheReadTokens, cacheCreationTokens)
- [ ] `mergeSourceBreakdowns` preserves sources NOT in `incomingSources`
- [ ] `mergeSourceBreakdowns` replaces sources IN `incomingSources`

---

## Task 2: Update Validation Hash Function

**File**: `packages/frontend/src/lib/validation/submission.ts`

**Change**: Update `generateSubmissionHash()` (lines 237-257) to be source-aware and synchronous.

### Current Code (lines 237-257)

```typescript
export function generateSubmissionHash(data: SubmissionData): string {
  const content = JSON.stringify({
    dateRange: data.meta.dateRange,
    totalTokens: data.summary.totalTokens,
    totalCost: data.summary.totalCost,
    activeDays: data.summary.activeDays,
    firstDay: data.contributions[0]?.date,
    lastDay: data.contributions[data.contributions.length - 1]?.date,
  });

  let hash = 0;
  for (let i = 0; i < content.length; i++) {
    const char = content.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash;
  }

  return Math.abs(hash).toString(16).padStart(16, "0");
}
```

### New Code

```typescript
/**
 * Generate a hash for the submission data (for deduplication)
 * 
 * CHANGED for source-level merge:
 * - Hash is now based on sources + date range (not totals)
 * - Totals change after merge, so they can't be in the hash
 * - This hash identifies "what sources and dates are being submitted"
 */
export function generateSubmissionHash(data: SubmissionData): string {
  const content = JSON.stringify({
    // What sources are being submitted
    sources: data.summary.sources.slice().sort(),
    // Date range of this submission
    dateRange: data.meta.dateRange,
    // Number of days with data (for basic fingerprinting)
    daysCount: data.contributions.length,
    // First and last dates (for ordering detection)
    firstDay: data.contributions[0]?.date,
    lastDay: data.contributions[data.contributions.length - 1]?.date,
  });

  // Simple synchronous hash (djb2 algorithm)
  let hash = 5381;
  for (let i = 0; i < content.length; i++) {
    const char = content.charCodeAt(i);
    hash = ((hash << 5) + hash) + char; // hash * 33 + char
    hash = hash & hash; // Convert to 32-bit integer
  }

  return Math.abs(hash).toString(16).padStart(16, "0");
}
```

### Verification Criteria

- [ ] Function is synchronous (no async/await)
- [ ] Hash does NOT include `totalTokens` or `totalCost`
- [ ] Hash DOES include `sources` array
- [ ] Same submission data produces same hash
- [ ] Different sources produce different hash

---

## Task 3: Refactor Submit API Route

**File**: `packages/frontend/src/app/api/submit/route.ts`

**Change**: Replace lines 88-142 (DELETE + INSERT) with source-level merge logic.

### New Implementation

```typescript
import { NextResponse } from "next/server";
import { db, apiTokens, users, submissions, dailyBreakdown } from "@/lib/db";
import { eq, sql } from "drizzle-orm";
import {
  validateSubmission,
  generateSubmissionHash,
  type SubmissionData,
} from "@/lib/validation/submission";
import {
  mergeSourceBreakdowns,
  recalculateDayTotals,
  buildModelBreakdown,
  sourceContributionToBreakdownData,
  type SourceBreakdownData,
} from "@/lib/db/helpers";

export async function POST(request: Request) {
  try {
    // ========================================
    // STEP 1: Authentication (UNCHANGED)
    // ========================================
    const authHeader = request.headers.get("Authorization");
    if (!authHeader?.startsWith("Bearer ")) {
      return NextResponse.json(
        { error: "Missing or invalid Authorization header" },
        { status: 401 }
      );
    }

    const token = authHeader.slice(7);

    const [tokenRecord] = await db
      .select({
        tokenId: apiTokens.id,
        userId: apiTokens.userId,
        username: users.username,
        expiresAt: apiTokens.expiresAt,
      })
      .from(apiTokens)
      .innerJoin(users, eq(apiTokens.userId, users.id))
      .where(eq(apiTokens.token, token))
      .limit(1);

    if (!tokenRecord) {
      return NextResponse.json({ error: "Invalid API token" }, { status: 401 });
    }

    if (tokenRecord.expiresAt && tokenRecord.expiresAt < new Date()) {
      return NextResponse.json({ error: "API token has expired" }, { status: 401 });
    }

    // ========================================
    // STEP 2: Parse and Validate (UNCHANGED)
    // ========================================
    let data: SubmissionData;
    try {
      data = await request.json();
    } catch {
      return NextResponse.json({ error: "Invalid JSON body" }, { status: 400 });
    }

    const validation = validateSubmission(data);
    if (!validation.valid) {
      return NextResponse.json(
        { error: "Validation failed", details: validation.errors },
        { status: 400 }
      );
    }

    // Reject empty submissions
    if (data.contributions.length === 0) {
      return NextResponse.json(
        { error: "No contribution data to submit" },
        { status: 400 }
      );
    }

    // Track which sources are in this submission
    const submittedSources = new Set(data.summary.sources);

    // ========================================
    // STEP 3: DATABASE OPERATIONS IN TRANSACTION
    // ========================================
    const result = await db.transaction(async (tx) => {
      // Update token last used timestamp
      await tx
        .update(apiTokens)
        .set({ lastUsedAt: new Date() })
        .where(eq(apiTokens.id, tokenRecord.tokenId));

      // ------------------------------------------
      // STEP 3a: Get or create user's submission
      // NOTE: .for('update') locks the row to prevent race conditions
      // ------------------------------------------
      let [existingSubmission] = await tx
        .select({
          id: submissions.id,
        })
        .from(submissions)
        .where(eq(submissions.userId, tokenRecord.userId))
        .for('update')  // CRITICAL: Prevents concurrent submission race condition
        .limit(1);

      let submissionId: string;
      let isNewSubmission = false;

      if (existingSubmission) {
        submissionId = existingSubmission.id;
      } else {
        // First submission - create placeholder (totals will be calculated later)
        isNewSubmission = true;
        const [newSubmission] = await tx
          .insert(submissions)
          .values({
            userId: tokenRecord.userId,
            totalTokens: 0,
            totalCost: "0",
            inputTokens: 0,
            outputTokens: 0,
            cacheCreationTokens: 0,
            cacheReadTokens: 0,
            dateStart: data.meta.dateRange.start,
            dateEnd: data.meta.dateRange.end,
            sourcesUsed: [],
            modelsUsed: [],
            status: "verified",
            cliVersion: data.meta.version,
            submissionHash: generateSubmissionHash(data),
          })
          .returning({ id: submissions.id });

        submissionId = newSubmission.id;
      }

      // ------------------------------------------
      // STEP 3b: Fetch existing daily breakdown for merge
      // NOTE: .for('update') locks rows to prevent concurrent modification
      // ------------------------------------------
      const existingDays = await tx
        .select({
          id: dailyBreakdown.id,
          date: dailyBreakdown.date,
          sourceBreakdown: dailyBreakdown.sourceBreakdown,
        })
        .from(dailyBreakdown)
        .where(eq(dailyBreakdown.submissionId, submissionId))
        .for('update');  // CRITICAL: Locks rows being modified

      // Build lookup map: date -> existing record
      const existingDaysMap = new Map(
        existingDays.map((d) => [d.date, d])
      );

      // ------------------------------------------
      // STEP 3c: Process each incoming day
      // ------------------------------------------
      for (const incomingDay of data.contributions) {
        // Build incoming sourceBreakdown from CLI data
        const incomingSourceBreakdown: Record<string, SourceBreakdownData> = {};
        for (const source of incomingDay.sources) {
          incomingSourceBreakdown[source.source] = sourceContributionToBreakdownData(source);
        }

        const existingDay = existingDaysMap.get(incomingDay.date);

        if (existingDay) {
          // ---- MERGE: Day exists, merge sources ----
          const existingSourceBreakdown = (existingDay.sourceBreakdown || {}) as Record<string, SourceBreakdownData>;

          // Merge: preserve existing sources not in submission, update submitted sources
          const mergedSourceBreakdown = mergeSourceBreakdowns(
            existingSourceBreakdown,
            incomingSourceBreakdown,
            submittedSources
          );

          // Recalculate day totals from merged data
          const dayTotals = recalculateDayTotals(mergedSourceBreakdown);

          // Build modelBreakdown from merged sources
          const modelBreakdown = buildModelBreakdown(mergedSourceBreakdown);

          // UPDATE existing daily breakdown
          await tx
            .update(dailyBreakdown)
            .set({
              tokens: dayTotals.tokens,
              cost: dayTotals.cost.toFixed(4),
              inputTokens: dayTotals.inputTokens,
              outputTokens: dayTotals.outputTokens,
              sourceBreakdown: mergedSourceBreakdown,
              modelBreakdown: modelBreakdown,
            })
            .where(eq(dailyBreakdown.id, existingDay.id));
        } else {
          // ---- INSERT: New day ----
          const dayTotals = recalculateDayTotals(incomingSourceBreakdown);
          const modelBreakdown = buildModelBreakdown(incomingSourceBreakdown);

          await tx.insert(dailyBreakdown).values({
            submissionId: submissionId,
            date: incomingDay.date,
            tokens: dayTotals.tokens,
            cost: dayTotals.cost.toFixed(4),
            inputTokens: dayTotals.inputTokens,
            outputTokens: dayTotals.outputTokens,
            sourceBreakdown: incomingSourceBreakdown,
            modelBreakdown: modelBreakdown,
          });
        }
      }

      // ------------------------------------------
      // STEP 3d: Recalculate submission totals from ALL daily breakdown
      // ------------------------------------------
      const [aggregates] = await tx
        .select({
          totalTokens: sql<number>`COALESCE(SUM(${dailyBreakdown.tokens}), 0)::int`,
          totalCost: sql<string>`COALESCE(SUM(CAST(${dailyBreakdown.cost} AS DECIMAL(12,4))), 0)::text`,
          inputTokens: sql<number>`COALESCE(SUM(${dailyBreakdown.inputTokens}), 0)::int`,
          outputTokens: sql<number>`COALESCE(SUM(${dailyBreakdown.outputTokens}), 0)::int`,
          dateStart: sql<string>`MIN(${dailyBreakdown.date})`,
          dateEnd: sql<string>`MAX(${dailyBreakdown.date})`,
          activeDays: sql<number>`COUNT(CASE WHEN ${dailyBreakdown.tokens} > 0 THEN 1 END)::int`,
        })
        .from(dailyBreakdown)
        .where(eq(dailyBreakdown.submissionId, submissionId));

      // Collect all unique sources and models from ALL daily breakdown records
      const allDays = await tx
        .select({
          sourceBreakdown: dailyBreakdown.sourceBreakdown,
        })
        .from(dailyBreakdown)
        .where(eq(dailyBreakdown.submissionId, submissionId));

      const allSources = new Set<string>();
      const allModels = new Set<string>();
      let totalCacheRead = 0;
      let totalCacheCreation = 0;

      for (const day of allDays) {
        if (day.sourceBreakdown) {
          for (const [sourceName, sourceData] of Object.entries(day.sourceBreakdown)) {
            allSources.add(sourceName);
            const data = sourceData as SourceBreakdownData;
            if (data.modelId) {
              allModels.add(data.modelId);
            }
            totalCacheRead += data.cacheRead || 0;
            totalCacheCreation += data.cacheWrite || 0;
          }
        }
      }

      // ------------------------------------------
      // STEP 3e: Update submission record
      // ------------------------------------------
      await tx
        .update(submissions)
        .set({
          totalTokens: aggregates.totalTokens,
          totalCost: aggregates.totalCost,
          inputTokens: aggregates.inputTokens,
          outputTokens: aggregates.outputTokens,
          cacheReadTokens: totalCacheRead,
          cacheCreationTokens: totalCacheCreation,
          dateStart: aggregates.dateStart,
          dateEnd: aggregates.dateEnd,
          sourcesUsed: Array.from(allSources),
          modelsUsed: Array.from(allModels),
          cliVersion: data.meta.version,
          submissionHash: generateSubmissionHash(data),
          updatedAt: new Date(),
        })
        .where(eq(submissions.id, submissionId));

      return {
        submissionId,
        isNewSubmission,
        metrics: {
          totalTokens: aggregates.totalTokens,
          totalCost: parseFloat(aggregates.totalCost),
          dateRange: {
            start: aggregates.dateStart,
            end: aggregates.dateEnd,
          },
          activeDays: aggregates.activeDays,
          sources: Array.from(allSources),
        },
      };
    });

    // ========================================
    // STEP 4: Return success response
    // ========================================
    return NextResponse.json({
      success: true,
      submissionId: result.submissionId,
      username: tokenRecord.username,
      metrics: result.metrics,
      mode: result.isNewSubmission ? "create" : "merge",
      warnings: validation.warnings.length > 0 ? validation.warnings : undefined,
    });
  } catch (error) {
    console.error("Submit error:", error);
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 }
    );
  }
}
```

### Verification Criteria

- [ ] No DELETE statement in the code
- [ ] Transaction wraps all database operations
- [ ] `.for('update')` used on submission SELECT (prevents race condition)
- [ ] `.for('update')` used on dailyBreakdown SELECT (prevents race condition)
- [ ] Existing sources NOT in submission are preserved
- [ ] Submitted sources are replaced/added
- [ ] All totals (including cache) are recalculated from dailyBreakdown
- [ ] `activeDays` is recalculated
- [ ] Response includes `mode: "create" | "merge"`

---

## Task 4: Add Unique Constraint on submissions.userId

**File**: `packages/frontend/src/lib/db/schema.ts`

**Problem**: Without a unique constraint, concurrent first submissions can create duplicate submission records for the same user. The `SELECT ... FOR UPDATE` returns empty rows for new users, so nothing is locked.

**Change**: Add unique constraint to the submissions table definition.

### Current Code (around line 186)

```typescript
  (table) => [
    index("idx_submissions_user_id").on(table.userId),
    index("idx_submissions_status").on(table.status),
    // ... other indexes
  ]
```

### New Code

First, ensure `unique` is imported at the top of the file:
```typescript
import { pgTable, uuid, bigint, decimal, date, varchar, text, timestamp, index, unique, jsonb } from "drizzle-orm/pg-core";
```

Then update the table definition:
```typescript
  (table) => [
    index("idx_submissions_user_id").on(table.userId),
    index("idx_submissions_status").on(table.status),
    // ... other indexes
    unique("submissions_user_id_unique").on(table.userId),  // ADD THIS - prevents duplicate submissions per user
  ]
```

### Migration Required

After changing the schema, run:
```bash
cd packages/frontend
bunx drizzle-kit push
```

Or create explicit migration:
```sql
ALTER TABLE submissions ADD CONSTRAINT submissions_user_id_unique UNIQUE (user_id);
```

### Verification Criteria

- [ ] Unique constraint added to schema
- [ ] Migration applied successfully
- [ ] Concurrent first submissions are prevented (only one succeeds, other gets conflict error)

---

## Task 5: Verify db/index.ts Exports

**File**: `packages/frontend/src/lib/db/index.ts`

**Check**: Ensure `dailyBreakdown` is exported. If not, add:

```typescript
export { dailyBreakdown } from "./schema";
```

### Verification Criteria

- [ ] Import `dailyBreakdown` from `@/lib/db` works in submit route

---

## Task 6: Add Test Suite

**File**: `packages/frontend/__tests__/api/submit.test.ts` (NEW FILE)

```typescript
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { db, submissions, dailyBreakdown, users, apiTokens } from '@/lib/db';
import { eq } from 'drizzle-orm';

describe('POST /api/submit - Source-Level Merge', () => {
  let testUserId: string;
  let testToken: string;

  beforeEach(async () => {
    // Setup test user and token
  });

  afterEach(async () => {
    // Cleanup test data
  });

  describe('First Submission (Create Mode)', () => {
    it('should create new submission with all sources', async () => {
      // Submit claude + cursor data
      // Assert: submission created, dailyBreakdown has both sources
    });

    it('should create dailyBreakdown for each day', async () => {
      // Submit 3 days of data
      // Assert: 3 dailyBreakdown records exist
    });
  });

  describe('Source-Level Merge', () => {
    it('should preserve existing sources when submitting partial data', async () => {
      // Setup: Submit claude + cursor for Jan 1-15
      // Action: Submit only claude for Jan 1-15 (cursor cleaned up)
      // Assert: cursor data still present for Jan 1-15
    });

    it('should update submitted source data', async () => {
      // Setup: Submit claude data with cost $10
      // Action: Submit claude data with cost $15
      // Assert: claude cost is now $15, not $25
    });

    it('should merge new source into existing day', async () => {
      // Setup: Submit claude for Jan 1
      // Action: Submit cursor for Jan 1
      // Assert: Both sources present for Jan 1
    });

    it('should add new dates without affecting existing', async () => {
      // Setup: Submit Jan 1-15
      // Action: Submit Jan 16-31
      // Assert: All 31 days present
    });
  });

  describe('Totals Recalculation', () => {
    it('should recalculate totalTokens from dailyBreakdown', async () => {
      // Submit, then submit more
      // Assert: totalTokens = sum of all dailyBreakdown.tokens
    });

    it('should recalculate cache tokens', async () => {
      // Submit data with cacheRead and cacheWrite
      // Assert: cacheReadTokens and cacheCreationTokens match
    });

    it('should recalculate activeDays', async () => {
      // Submit 5 days with tokens, 2 days with 0 tokens
      // Assert: activeDays = 5
    });

    it('should update sourcesUsed to include all sources', async () => {
      // Submit claude, then cursor
      // Assert: sourcesUsed = ["claude", "cursor"]
    });
  });

  describe('Edge Cases', () => {
    it('should reject empty submissions', async () => {
      // Submit with contributions: []
      // Assert: 400 error
    });

    it('should handle day with no data for submitted source', async () => {
      // Submit --claude but day only has opencode
      // Assert: opencode preserved, no claude added
    });

    it('should handle concurrent submissions without data loss', async () => {
      // Submit claude and cursor simultaneously for existing user
      // Assert: Both sources present, no lost updates
      // Note: Row-level locking (.for('update')) ensures this
    });

    it('should prevent duplicate submissions for new user', async () => {
      // Attempt two concurrent first submissions
      // Assert: Only one submission record exists
      // Note: Unique constraint on userId ensures this
    });
  });

  describe('Response Format', () => {
    it('should return mode: "create" for first submission', async () => {});
    it('should return mode: "merge" for subsequent submissions', async () => {});
    it('should include recalculated metrics', async () => {});
  });
});
```

### Verification Criteria

- [ ] All test cases pass
- [ ] Coverage for: create, merge, preserve, recalculate, edge cases

---

## Edge Cases Matrix

| Scenario | Expected Behavior | Test Coverage |
|----------|-------------------|---------------|
| First submission | Create new submission + dailyBreakdown | `First Submission` suite |
| Same day, same source | Replace source data | `should update submitted source data` |
| Same day, different source | Add source to existing | `should merge new source into existing day` |
| Different day | Insert new dailyBreakdown | `should add new dates without affecting existing` |
| Source not in submission | Preserve existing data | `should preserve existing sources` |
| Empty submission | Reject with 400 | `should reject empty submissions` |
| Day has no data for submitted source | Skip that day for source | `should handle day with no data` |
| Concurrent submissions (existing user) | No lost updates (row locking) | `should handle concurrent submissions` |
| Concurrent first submissions (new user) | Only one succeeds (unique constraint) | `should prevent duplicate submissions` |

---

## File Dependency Order

```
1. packages/frontend/src/lib/db/helpers.ts (NEW - no dependencies)
2. packages/frontend/src/lib/validation/submission.ts (MODIFY - no new dependencies)
3. packages/frontend/src/app/api/submit/route.ts (MODIFY - depends on 1, 2)
4. packages/frontend/src/lib/db/schema.ts (MODIFY - add unique constraint)
5. packages/frontend/src/lib/db/index.ts (VERIFY - ensure exports)
6. packages/frontend/__tests__/api/submit.test.ts (NEW - depends on 3, 4)
```

**Note**: Task 4 (schema change) requires running `bunx drizzle-kit push` or creating a migration after the code change.

---

## Verification Checklist

### Functional Requirements

- [ ] Submitting partial data preserves other sources
- [ ] Submitting same source replaces (not adds) data
- [ ] New dates are added without affecting existing
- [ ] All totals are recalculated from dailyBreakdown
- [ ] `activeDays` is recalculated correctly
- [ ] `sourcesUsed` includes all sources from all days
- [ ] `modelsUsed` includes all models from all sources

### Non-Functional Requirements

- [ ] No DELETE statement in happy path
- [ ] All operations in single transaction
- [ ] Row-level locking prevents concurrent submission race conditions
- [ ] TypeScript compiles without errors
- [ ] Response format backward compatible
- [ ] Schema migration applied successfully (unique constraint on userId)

### Backward Compatibility

- [ ] First submission (no existing data) works as before
- [ ] Profile page shows correct data after merge
- [ ] Leaderboard calculations unchanged
- [ ] CLI doesn't need any changes

---

## Definition of Done

1. All 6 tasks completed and verified
2. All verification criteria checked
3. Database migration applied (unique constraint on userId)
4. TypeScript compiles without errors
5. All tests pass
6. Manual testing confirms source-level merge works
7. Concurrent submissions do not create duplicate records
