# Token Tracker Social Platform - Design Document

## Overview

Transform token-tracker from a local CLI tool into a social platform where users can:
- Submit their AI token usage data from CLI
- View a global leaderboard of top users
- Browse individual profile pages with contribution graphs
- Compare usage across different AI providers

### Goals
- **Privacy-first**: Users explicitly choose to submit data
- **Anti-cheat**: Validate submissions to prevent fake data
- **Beautiful**: Showcase our 2D/3D contribution graphs on profile pages
- **Simple**: Minimal friction for CLI submission

### Non-Goals (v1)
- Real-time collaboration features
- Team/organization accounts
- Paid tiers or premium features

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (Next.js)                       │
├─────────────────────────────────────────────────────────────────┤
│  /                    → Global leaderboard                       │
│  /profile/[username]  → User profile + contribution graph        │
│  /settings            → User settings (auth required)            │
│  /admin               → Flagged submissions (admin only)         │
└─────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                      API Routes (Next.js)                        │
├─────────────────────────────────────────────────────────────────┤
│  POST /api/auth/github         → GitHub OAuth callback           │
│  GET  /api/auth/session        → Get current session             │
│  POST /api/auth/logout         → Clear session                   │
│  POST /api/submit              → Submit token data (CLI)         │
│  GET  /api/leaderboard         → Get ranked users                │
│  GET  /api/users/[username]    → Get user profile + data         │
│  POST /api/admin/review        → Approve/reject flagged (admin)  │
└─────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                     PostgreSQL Database                          │
├─────────────────────────────────────────────────────────────────┤
│  users              → User accounts (GitHub OAuth)               │
│  submissions        → Token usage submissions                    │
│  daily_breakdown    → Per-day token counts                       │
│  sessions           → Auth sessions (JWT alternative)            │
└─────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                          CLI Tool                                │
├─────────────────────────────────────────────────────────────────┤
│  token-tracker submit          → Submit data to platform         │
│  token-tracker login           → Authenticate via GitHub         │
│  token-tracker logout          → Clear local credentials         │
│  token-tracker whoami          → Show current user               │
└─────────────────────────────────────────────────────────────────┘
```

---

## Database Schema

### Provider: Vercel Postgres (Neon) or any PostgreSQL

Using **Drizzle ORM** for type-safe queries and migrations.

```sql
-- Users table (GitHub OAuth)
CREATE TABLE users (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  github_id     INTEGER UNIQUE NOT NULL,
  username      VARCHAR(39) UNIQUE NOT NULL,  -- GitHub username
  display_name  VARCHAR(255),
  avatar_url    TEXT,
  email         VARCHAR(255),
  is_admin      BOOLEAN DEFAULT FALSE,
  created_at    TIMESTAMP DEFAULT NOW(),
  updated_at    TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_github_id ON users(github_id);

-- Auth sessions (stateful sessions, not JWT)
CREATE TABLE sessions (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token         VARCHAR(64) UNIQUE NOT NULL,  -- Secure random token
  expires_at    TIMESTAMP NOT NULL,
  created_at    TIMESTAMP DEFAULT NOW(),
  
  -- For CLI vs web distinction
  source        VARCHAR(10) DEFAULT 'web',    -- 'web' | 'cli'
  user_agent    TEXT
);

CREATE INDEX idx_sessions_token ON sessions(token);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);

-- Submissions (aggregated token data)
CREATE TABLE submissions (
  id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id               UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  
  -- Aggregated totals
  total_tokens          BIGINT NOT NULL,
  total_cost            DECIMAL(12, 4) NOT NULL,
  input_tokens          BIGINT NOT NULL,
  output_tokens         BIGINT NOT NULL,
  cache_creation_tokens BIGINT DEFAULT 0,
  cache_read_tokens     BIGINT DEFAULT 0,
  
  -- Date range
  date_start            DATE NOT NULL,
  date_end              DATE NOT NULL,
  
  -- Metadata
  sources_used          TEXT[] NOT NULL,       -- ['opencode', 'claude', 'codex', 'gemini']
  models_used           TEXT[] NOT NULL,       -- ['claude-3-5-sonnet', 'gpt-4o', ...]
  
  -- Validation
  status                VARCHAR(20) DEFAULT 'pending',  -- 'pending' | 'verified' | 'flagged' | 'rejected'
  flagged_reason        TEXT,
  reviewed_by           UUID REFERENCES users(id),
  reviewed_at           TIMESTAMP,
  
  -- CLI metadata
  cli_version           VARCHAR(20),
  submission_hash       VARCHAR(64) UNIQUE,    -- SHA256 of raw data (dedup)
  
  created_at            TIMESTAMP DEFAULT NOW(),
  updated_at            TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_submissions_user_id ON submissions(user_id);
CREATE INDEX idx_submissions_status ON submissions(status);
CREATE INDEX idx_submissions_total_tokens ON submissions(total_tokens DESC);
CREATE INDEX idx_submissions_created_at ON submissions(created_at DESC);

-- Daily breakdown (for contribution graph)
CREATE TABLE daily_breakdown (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  submission_id   UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
  
  date            DATE NOT NULL,
  tokens          BIGINT NOT NULL,
  cost            DECIMAL(10, 4) NOT NULL,
  input_tokens    BIGINT NOT NULL,
  output_tokens   BIGINT NOT NULL,
  
  -- Per-day provider breakdown (JSONB for flexibility)
  provider_breakdown JSONB,  -- {"anthropic": 5000, "openai": 3000, ...}
  source_breakdown   JSONB,  -- {"opencode": 4000, "claude": 4000, ...}
  model_breakdown    JSONB,  -- {"claude-3-5-sonnet": 5000, "gpt-4o": 3000, ...}
  
  UNIQUE(submission_id, date)
);

CREATE INDEX idx_daily_breakdown_submission_id ON daily_breakdown(submission_id);
CREATE INDEX idx_daily_breakdown_date ON daily_breakdown(date);

-- API tokens for CLI authentication
CREATE TABLE api_tokens (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token         VARCHAR(64) UNIQUE NOT NULL,  -- tt_xxxxxxxxxxxx
  name          VARCHAR(100) NOT NULL,        -- e.g., "MacBook Pro"
  last_used_at  TIMESTAMP,
  expires_at    TIMESTAMP,                    -- NULL = never expires
  created_at    TIMESTAMP DEFAULT NOW(),
  
  UNIQUE(user_id, name)
);

CREATE INDEX idx_api_tokens_token ON api_tokens(token);
CREATE INDEX idx_api_tokens_user_id ON api_tokens(user_id);
```

### Drizzle Schema (TypeScript)

```typescript
// db/schema.ts
import { pgTable, uuid, varchar, text, boolean, timestamp, 
         bigint, decimal, date, jsonb, index, unique } from 'drizzle-orm/pg-core';

export const users = pgTable('users', {
  id: uuid('id').primaryKey().defaultRandom(),
  githubId: integer('github_id').unique().notNull(),
  username: varchar('username', { length: 39 }).unique().notNull(),
  displayName: varchar('display_name', { length: 255 }),
  avatarUrl: text('avatar_url'),
  email: varchar('email', { length: 255 }),
  isAdmin: boolean('is_admin').default(false),
  createdAt: timestamp('created_at').defaultNow(),
  updatedAt: timestamp('updated_at').defaultNow(),
}, (table) => ({
  usernameIdx: index('idx_users_username').on(table.username),
}));

export const sessions = pgTable('sessions', {
  id: uuid('id').primaryKey().defaultRandom(),
  userId: uuid('user_id').notNull().references(() => users.id, { onDelete: 'cascade' }),
  token: varchar('token', { length: 64 }).unique().notNull(),
  expiresAt: timestamp('expires_at').notNull(),
  createdAt: timestamp('created_at').defaultNow(),
  source: varchar('source', { length: 10 }).default('web'),
  userAgent: text('user_agent'),
}, (table) => ({
  tokenIdx: index('idx_sessions_token').on(table.token),
}));

export const submissions = pgTable('submissions', {
  id: uuid('id').primaryKey().defaultRandom(),
  userId: uuid('user_id').notNull().references(() => users.id, { onDelete: 'cascade' }),
  totalTokens: bigint('total_tokens', { mode: 'number' }).notNull(),
  totalCost: decimal('total_cost', { precision: 12, scale: 4 }).notNull(),
  inputTokens: bigint('input_tokens', { mode: 'number' }).notNull(),
  outputTokens: bigint('output_tokens', { mode: 'number' }).notNull(),
  cacheCreationTokens: bigint('cache_creation_tokens', { mode: 'number' }).default(0),
  cacheReadTokens: bigint('cache_read_tokens', { mode: 'number' }).default(0),
  dateStart: date('date_start').notNull(),
  dateEnd: date('date_end').notNull(),
  sourcesUsed: text('sources_used').array().notNull(),
  modelsUsed: text('models_used').array().notNull(),
  status: varchar('status', { length: 20 }).default('pending'),
  flaggedReason: text('flagged_reason'),
  reviewedBy: uuid('reviewed_by').references(() => users.id),
  reviewedAt: timestamp('reviewed_at'),
  cliVersion: varchar('cli_version', { length: 20 }),
  submissionHash: varchar('submission_hash', { length: 64 }).unique(),
  createdAt: timestamp('created_at').defaultNow(),
  updatedAt: timestamp('updated_at').defaultNow(),
}, (table) => ({
  userIdx: index('idx_submissions_user_id').on(table.userId),
  statusIdx: index('idx_submissions_status').on(table.status),
  tokensIdx: index('idx_submissions_total_tokens').on(table.totalTokens),
}));

export const dailyBreakdown = pgTable('daily_breakdown', {
  id: uuid('id').primaryKey().defaultRandom(),
  submissionId: uuid('submission_id').notNull().references(() => submissions.id, { onDelete: 'cascade' }),
  date: date('date').notNull(),
  tokens: bigint('tokens', { mode: 'number' }).notNull(),
  cost: decimal('cost', { precision: 10, scale: 4 }).notNull(),
  inputTokens: bigint('input_tokens', { mode: 'number' }).notNull(),
  outputTokens: bigint('output_tokens', { mode: 'number' }).notNull(),
  providerBreakdown: jsonb('provider_breakdown'),
  sourceBreakdown: jsonb('source_breakdown'),
  modelBreakdown: jsonb('model_breakdown'),
}, (table) => ({
  submissionIdx: index('idx_daily_breakdown_submission_id').on(table.submissionId),
  dateIdx: index('idx_daily_breakdown_date').on(table.date),
  uniqueDate: unique().on(table.submissionId, table.date),
}));

export const apiTokens = pgTable('api_tokens', {
  id: uuid('id').primaryKey().defaultRandom(),
  userId: uuid('user_id').notNull().references(() => users.id, { onDelete: 'cascade' }),
  token: varchar('token', { length: 64 }).unique().notNull(),
  name: varchar('name', { length: 100 }).notNull(),
  lastUsedAt: timestamp('last_used_at'),
  expiresAt: timestamp('expires_at'),
  createdAt: timestamp('created_at').defaultNow(),
}, (table) => ({
  tokenIdx: index('idx_api_tokens_token').on(table.token),
  userIdx: index('idx_api_tokens_user_id').on(table.userId),
  uniqueName: unique().on(table.userId, table.name),
}));
```

---

## Authentication Flow

### Design Principles
- **Stateful sessions** over JWT (simpler revocation, no token refresh complexity)
- **API tokens** for CLI (long-lived, can be named/revoked individually)
- **No passwords** - GitHub OAuth only

### Web Authentication

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  User    │     │  Next.js │     │  GitHub  │     │ Database │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │
     │ Click "Login"  │                │                │
     │───────────────>│                │                │
     │                │                │                │
     │                │ Redirect to GitHub OAuth        │
     │<───────────────│───────────────>│                │
     │                │                │                │
     │ Authorize app  │                │                │
     │───────────────────────────────>│                │
     │                │                │                │
     │                │ Callback with code              │
     │                │<───────────────│                │
     │                │                │                │
     │                │ Exchange code for access_token  │
     │                │───────────────>│                │
     │                │                │                │
     │                │ Get user info  │                │
     │                │───────────────>│                │
     │                │                │                │
     │                │ Upsert user    │                │
     │                │────────────────────────────────>│
     │                │                │                │
     │                │ Create session │                │
     │                │────────────────────────────────>│
     │                │                │                │
     │ Set cookie     │                │                │
     │<───────────────│                │                │
     │                │                │                │
```

### CLI Authentication

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│   CLI    │     │  Next.js │     │  Browser │     │ Database │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │
     │ token-tracker login             │                │
     │───────────────>│                │                │
     │                │                │                │
     │                │ Generate device code            │
     │                │────────────────────────────────>│
     │                │                │                │
     │ Display URL + code              │                │
     │<───────────────│                │                │
     │                │                │                │
     │ Open browser   │                │                │
     │───────────────────────────────>│                │
     │                │                │                │
     │                │ User completes OAuth            │
     │                │<───────────────│                │
     │                │                │                │
     │ Poll for token │                │                │
     │───────────────>│                │                │
     │                │                │                │
     │                │ Create API token               │
     │                │────────────────────────────────>│
     │                │                │                │
     │ Return API token                │                │
     │<───────────────│                │                │
     │                │                │                │
     │ Save to ~/.config/token-tracker/credentials.json │
     │                │                │                │
```

### Session/Token Format

```typescript
// Web session cookie
{
  name: 'tt_session',
  value: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',  // 32-byte random hex
  httpOnly: true,
  secure: true,
  sameSite: 'lax',
  maxAge: 60 * 60 * 24 * 30,  // 30 days
}

// CLI API token (stored locally)
{
  // ~/.config/token-tracker/credentials.json
  "token": "tt_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",  // Prefixed for easy identification
  "username": "junhoyeo",
  "expiresAt": null  // Never expires, but can be revoked
}
```

---

## API Endpoints

### Authentication

#### `GET /api/auth/github`
Initiates GitHub OAuth flow.

```typescript
// Redirects to GitHub with:
const params = new URLSearchParams({
  client_id: process.env.GITHUB_CLIENT_ID,
  redirect_uri: `${process.env.NEXT_PUBLIC_URL}/api/auth/github/callback`,
  scope: 'read:user user:email',
  state: generateSecureRandom(),  // CSRF protection
});
redirect(`https://github.com/login/oauth/authorize?${params}`);
```

#### `GET /api/auth/github/callback`
Handles OAuth callback, creates user and session.

```typescript
// Response: Redirects to / with session cookie set
```

#### `GET /api/auth/session`
Returns current user info.

```typescript
// Response
{
  "user": {
    "id": "uuid",
    "username": "junhoyeo",
    "displayName": "Junho Yeo",
    "avatarUrl": "https://avatars.githubusercontent.com/u/...",
    "isAdmin": false
  }
} | { "user": null }
```

#### `POST /api/auth/logout`
Clears session.

```typescript
// Response
{ "success": true }
```

### CLI Authentication

#### `POST /api/auth/device`
Initiates device flow for CLI login.

```typescript
// Request
{ "deviceName": "MacBook Pro" }

// Response
{
  "deviceCode": "XXXX-XXXX",
  "userCode": "ABCD-1234",  // User enters this in browser
  "verificationUrl": "https://token-tracker.dev/device",
  "expiresIn": 900,  // 15 minutes
  "interval": 5      // Poll every 5 seconds
}
```

#### `POST /api/auth/device/poll`
CLI polls this to check if user completed auth.

```typescript
// Request
{ "deviceCode": "XXXX-XXXX" }

// Response (pending)
{ "status": "pending" }

// Response (success)
{
  "status": "complete",
  "token": "tt_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  "user": {
    "username": "junhoyeo",
    "avatarUrl": "..."
  }
}

// Response (expired)
{ "status": "expired" }
```

#### `GET /device`
Web page where user enters code and authorizes CLI.

### Data Submission

#### `POST /api/submit`
Submit token usage data from CLI.

```typescript
// Request Headers
Authorization: Bearer tt_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

// Request Body
{
  "data": {
    "contributions": [...],  // Daily contribution data
    "summary": {
      "totalTokens": 1234567,
      "totalCost": 12.34,
      "inputTokens": 800000,
      "outputTokens": 434567,
      "cacheCreationTokens": 50000,
      "cacheReadTokens": 100000,
      "activeDays": 45,
      "avgTokensPerDay": 27434,
      "peakDay": { "date": "2024-11-15", "tokens": 150000 },
      "sources": ["opencode", "claude"],
      "providers": ["anthropic", "openai"],
      "models": ["claude-3-5-sonnet-20241022", "gpt-4o"]
    },
    "dateRange": {
      "start": "2024-10-01",
      "end": "2024-12-01"
    }
  },
  "cliVersion": "1.2.0"
}

// Response (success)
{
  "success": true,
  "submissionId": "uuid",
  "status": "verified",  // or "pending" if needs review
  "message": "Submission recorded successfully"
}

// Response (flagged)
{
  "success": true,
  "submissionId": "uuid",
  "status": "flagged",
  "message": "Submission flagged for review due to: unusually high token count"
}

// Response (error)
{
  "success": false,
  "error": "validation_failed",
  "message": "Token counts don't add up: input + output != total"
}
```

### Leaderboard

#### `GET /api/leaderboard`
Get ranked list of users by total tokens.

```typescript
// Query params
?period=all        // 'all' | 'month' | 'week'
&page=1
&limit=50
&sort=tokens       // 'tokens' | 'cost' | 'days'

// Response
{
  "users": [
    {
      "rank": 1,
      "username": "junhoyeo",
      "displayName": "Junho Yeo",
      "avatarUrl": "...",
      "totalTokens": 15000000,
      "totalCost": 150.00,
      "activeDays": 60,
      "topProvider": "anthropic",
      "topSource": "claude",
      "lastSubmission": "2024-12-01T12:00:00Z"
    },
    // ...
  ],
  "pagination": {
    "page": 1,
    "limit": 50,
    "total": 1234,
    "hasMore": true
  },
  "stats": {
    "totalUsers": 1234,
    "totalTokens": 5000000000,
    "totalCost": 50000.00
  }
}
```

### User Profiles

#### `GET /api/users/[username]`
Get user profile and contribution data.

```typescript
// Response
{
  "user": {
    "username": "junhoyeo",
    "displayName": "Junho Yeo",
    "avatarUrl": "...",
    "joinedAt": "2024-10-01T00:00:00Z",
    "rank": 1
  },
  "stats": {
    "totalTokens": 15000000,
    "totalCost": 150.00,
    "inputTokens": 10000000,
    "outputTokens": 5000000,
    "cacheReadTokens": 2000000,
    "activeDays": 60,
    "currentStreak": 15,
    "longestStreak": 30
  },
  "contributions": [
    // TokenContributionData format (same as local CLI output)
    {
      "date": "2024-11-15",
      "count": 150000,
      "intensity": 4,
      "providers": { "anthropic": 100000, "openai": 50000 },
      "sources": { "claude": 80000, "opencode": 70000 },
      "models": { "claude-3-5-sonnet": 100000, "gpt-4o": 50000 }
    },
    // ...
  ],
  "breakdown": {
    "byProvider": { "anthropic": 12000000, "openai": 3000000 },
    "bySource": { "claude": 8000000, "opencode": 7000000 },
    "byModel": { "claude-3-5-sonnet": 10000000, "gpt-4o": 3000000, "claude-3-haiku": 2000000 }
  }
}
```

### Admin

#### `GET /api/admin/flagged`
Get flagged submissions for review.

```typescript
// Response
{
  "submissions": [
    {
      "id": "uuid",
      "user": { "username": "...", "avatarUrl": "..." },
      "totalTokens": 999999999,
      "flaggedReason": "Daily limit exceeded: 500M tokens on 2024-11-15",
      "createdAt": "2024-12-01T12:00:00Z"
    }
  ]
}
```

#### `POST /api/admin/review`
Approve or reject a submission.

```typescript
// Request
{
  "submissionId": "uuid",
  "action": "approve" | "reject",
  "reason": "Verified with user"  // Optional
}

// Response
{ "success": true }
```

---

## Validation Rules (Anti-Cheat)

### Level 1: Basic Validation (Instant Reject)
```typescript
const basicValidation = {
  // Math must add up
  tokenSum: (d) => d.inputTokens + d.outputTokens === d.totalTokens,
  
  // No negative values
  noNegatives: (d) => Object.values(d).every(v => typeof v !== 'number' || v >= 0),
  
  // No future dates
  noFutureDates: (d) => new Date(d.dateEnd) <= new Date(),
  
  // Reasonable date range (max 1 year)
  reasonableDateRange: (d) => {
    const days = daysBetween(d.dateStart, d.dateEnd);
    return days > 0 && days <= 365;
  },
};
```

### Level 2: Suspicious Patterns (Flag for Review)
```typescript
const suspiciousPatterns = {
  // Daily limits
  dailyTokenLimit: 250_000_000,    // 250M tokens/day
  dailyCostLimit: 5000,            // $5,000/day
  
  // Anomaly detection
  suddenSpike: (contributions) => {
    // Flag if any day is >10x the user's average
    const avg = mean(contributions.map(c => c.tokens));
    return contributions.some(c => c.tokens > avg * 10);
  },
  
  // Impossible efficiency
  impossibleCacheRatio: (d) => {
    // Cache reads can't exceed 95% of total
    return d.cacheReadTokens / d.totalTokens > 0.95;
  },
  
  // Round number suspicion
  tooRound: (d) => {
    // Exactly 1,000,000 tokens is suspicious
    return d.totalTokens % 1000000 === 0 && d.totalTokens > 1000000;
  },
};
```

### Level 3: Duplicate Detection
```typescript
const deduplication = {
  // Hash the raw contribution data
  computeHash: (data) => sha256(JSON.stringify(sortKeys(data))),
  
  // Check for exact duplicates
  isDuplicate: async (hash, userId) => {
    return db.query.submissions.findFirst({
      where: and(
        eq(submissions.submissionHash, hash),
        eq(submissions.userId, userId)
      )
    });
  },
  
  // Check for overlapping date ranges (same user)
  hasOverlap: async (userId, dateStart, dateEnd) => {
    return db.query.submissions.findFirst({
      where: and(
        eq(submissions.userId, userId),
        or(
          between(submissions.dateStart, dateStart, dateEnd),
          between(submissions.dateEnd, dateStart, dateEnd)
        )
      )
    });
  },
};
```

---

## CLI Changes

### New Commands

```bash
# Authentication
token-tracker login           # Open browser for GitHub OAuth
token-tracker logout          # Clear local credentials
token-tracker whoami          # Show current authenticated user

# Submission
token-tracker submit          # Submit current data to platform
token-tracker submit --dry-run   # Preview what would be submitted
token-tracker submit --force     # Overwrite existing submission for date range
```

### Credentials Storage

```
~/.config/token-tracker/
├── credentials.json          # API token
└── config.json              # User preferences (optional)
```

```typescript
// credentials.json
{
  "token": "tt_abc123...",
  "username": "junhoyeo",
  "avatarUrl": "https://...",
  "createdAt": "2024-12-01T00:00:00Z"
}
```

### Submit Flow

```typescript
// src/submit.ts
export async function submitData(options: SubmitOptions) {
  // 1. Check authentication
  const credentials = loadCredentials();
  if (!credentials) {
    console.error('Not logged in. Run: token-tracker login');
    process.exit(1);
  }
  
  // 2. Generate graph data (reuse existing logic)
  const data = await generateGraphData();
  
  // 3. Show preview
  console.log(`\nSubmitting data for ${credentials.username}:`);
  console.log(`  Period: ${data.dateRange.start} to ${data.dateRange.end}`);
  console.log(`  Total tokens: ${formatNumber(data.summary.totalTokens)}`);
  console.log(`  Total cost: $${data.summary.totalCost.toFixed(2)}`);
  console.log(`  Active days: ${data.summary.activeDays}`);
  
  if (options.dryRun) {
    console.log('\n(Dry run - no data submitted)');
    return;
  }
  
  // 4. Confirm
  const confirm = await prompt('Submit this data? [y/N]');
  if (confirm.toLowerCase() !== 'y') {
    console.log('Cancelled.');
    return;
  }
  
  // 5. Submit
  const response = await fetch(`${API_URL}/api/submit`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${credentials.token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      data,
      cliVersion: VERSION,
    }),
  });
  
  // 6. Handle response
  const result = await response.json();
  if (result.success) {
    console.log(`\n✓ Submitted successfully!`);
    console.log(`  View at: https://token-tracker.dev/profile/${credentials.username}`);
    if (result.status === 'flagged') {
      console.log(`  ⚠ Note: Submission flagged for review`);
    }
  } else {
    console.error(`\n✗ Submission failed: ${result.message}`);
  }
}
```

---

## Frontend Pages

### `/` - Leaderboard (Home)

```tsx
// Components:
// - Hero stats (total users, total tokens, total cost)
// - Period selector (All time / This month / This week)
// - Leaderboard table with pagination
// - "Submit your data" CTA for non-logged-in users

<main>
  <HeroStats totalUsers={1234} totalTokens={5e9} totalCost={50000} />
  
  <div className="flex justify-between">
    <h2>Leaderboard</h2>
    <PeriodSelector value={period} onChange={setPeriod} />
  </div>
  
  <LeaderboardTable users={users} currentUser={session?.user} />
  
  <Pagination {...pagination} />
</main>
```

### `/profile/[username]` - User Profile

```tsx
// Components:
// - User header (avatar, name, rank, joined date)
// - Stats cards (total tokens, cost, active days, streaks)
// - Contribution graph (2D/3D toggle) ← REUSE EXISTING
// - Breakdown panel ← REUSE EXISTING
// - Provider/Source/Model breakdown charts

<main>
  <UserHeader user={user} rank={rank} />
  
  <StatsCards stats={stats} />
  
  <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <div className="lg:col-span-2">
      <GraphContainer
        data={contributions}
        renderMode={renderMode}
        // ... same props as current app
      />
    </div>
    <div>
      <BreakdownPanel
        data={contributions}
        selectedDate={selectedDate}
        // ... same props as current app
      />
    </div>
  </div>
  
  <BreakdownCharts breakdown={breakdown} />
</main>
```

### `/settings` - User Settings

```tsx
// Components:
// - API tokens management (create, revoke)
// - Submission history
// - Delete account

<main>
  <h1>Settings</h1>
  
  <section>
    <h2>API Tokens</h2>
    <p>Use these tokens to authenticate the CLI.</p>
    <ApiTokenList tokens={tokens} onRevoke={handleRevoke} />
    <CreateTokenButton onCreate={handleCreate} />
  </section>
  
  <section>
    <h2>Submissions</h2>
    <SubmissionHistory submissions={submissions} />
  </section>
  
  <section>
    <h2>Danger Zone</h2>
    <DeleteAccountButton />
  </section>
</main>
```

### `/device` - CLI Device Authorization

```tsx
// Simple page for CLI device flow
<main className="max-w-md mx-auto text-center">
  <h1>Authorize CLI</h1>
  
  {!session ? (
    <LoginButton />
  ) : (
    <>
      <p>Enter the code shown in your terminal:</p>
      <CodeInput value={code} onChange={setCode} />
      <Button onClick={handleAuthorize}>Authorize</Button>
    </>
  )}
</main>
```

---

## Deployment

### Environment Variables

```bash
# Database
DATABASE_URL=postgres://...

# GitHub OAuth
GITHUB_CLIENT_ID=...
GITHUB_CLIENT_SECRET=...

# App
NEXT_PUBLIC_URL=https://token-tracker.dev
SESSION_SECRET=...  # 32+ bytes random
```

### Database Hosting Options

| Provider | Free Tier | Notes |
|----------|-----------|-------|
| Vercel Postgres | 256MB storage | Native Vercel integration |
| Neon | 512MB storage | Generous free tier, serverless |
| Supabase | 500MB storage | If we change our mind on auth |
| Railway | $5 credit/month | Good for hobby projects |
| PlanetScale | 5GB storage | MySQL, not Postgres |

**Recommendation**: Vercel Postgres for simplest deployment, Neon if we need more storage.

### Caching Strategy

```typescript
// Leaderboard (revalidate every 5 minutes)
export const revalidate = 300;

// User profiles (revalidate every minute)
export const revalidate = 60;

// Use ISR for profile pages
export async function generateStaticParams() {
  // Pre-render top 100 users
  const topUsers = await getTopUsers(100);
  return topUsers.map(u => ({ username: u.username }));
}
```

---

## Implementation Order

### Phase 1: Core Infrastructure
1. Set up Drizzle + PostgreSQL schema
2. Implement GitHub OAuth (web)
3. Implement session management
4. Create basic API routes

### Phase 2: CLI Integration
5. Implement device flow auth
6. Add `login`, `logout`, `whoami` commands
7. Add `submit` command
8. Implement validation rules

### Phase 3: Frontend
9. Create leaderboard page
10. Create profile page (integrate existing graph components)
11. Create settings page
12. Add admin review page

### Phase 4: Polish
13. Add caching/ISR
14. Performance optimization
15. Error handling + edge cases
16. Rate limiting

---

## Open Questions

1. **Domain**: `token-tracker.dev`? `tokentracker.app`? Use Vercel subdomain initially?

2. **Submission granularity**: Allow multiple submissions for different date ranges, or only one "latest" submission per user?

3. **Historical data**: When user re-submits, keep history or replace?

4. **Privacy**: Show exact token counts or ranges? (e.g., "10M-50M tokens")

5. **Badges/Achievements**: Add gamification? (e.g., "100k tokens", "7-day streak")

---

## Appendix: Type Definitions

```typescript
// Shared types between CLI and frontend
// types/api.ts

export interface TokenSubmission {
  contributions: DailyContribution[];
  summary: TokenSummary;
  dateRange: DateRange;
}

export interface DailyContribution {
  date: string;  // YYYY-MM-DD
  count: number;
  intensity: number;  // 0-4
  providers: Record<string, number>;
  sources: Record<string, number>;
  models: Record<string, number>;
  cost: number;
  inputTokens: number;
  outputTokens: number;
}

export interface TokenSummary {
  totalTokens: number;
  totalCost: number;
  inputTokens: number;
  outputTokens: number;
  cacheCreationTokens: number;
  cacheReadTokens: number;
  activeDays: number;
  avgTokensPerDay: number;
  peakDay: { date: string; tokens: number };
  sources: string[];
  providers: string[];
  models: string[];
}

export interface DateRange {
  start: string;  // YYYY-MM-DD
  end: string;
}

export interface LeaderboardUser {
  rank: number;
  username: string;
  displayName: string | null;
  avatarUrl: string;
  totalTokens: number;
  totalCost: number;
  activeDays: number;
  topProvider: string;
  topSource: string;
  lastSubmission: string;
}

export interface UserProfile {
  user: {
    username: string;
    displayName: string | null;
    avatarUrl: string;
    joinedAt: string;
    rank: number;
  };
  stats: TokenSummary & {
    currentStreak: number;
    longestStreak: number;
  };
  contributions: DailyContribution[];
  breakdown: {
    byProvider: Record<string, number>;
    bySource: Record<string, number>;
    byModel: Record<string, number>;
  };
}
```
