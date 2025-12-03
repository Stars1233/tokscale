# Token Tracker Social Platform - Design Document

> **Version**: 1.0.0  
> **Last Updated**: 2024-12-03  
> **Status**: Draft

## Table of Contents

1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [Project Structure](#project-structure)
4. [Database Design](#database-design)
5. [Authentication System](#authentication-system)
6. [API Specification](#api-specification)
7. [CLI Implementation](#cli-implementation)
8. [Frontend Implementation](#frontend-implementation)
9. [Data Validation](#data-validation)
10. [Security Considerations](#security-considerations)
11. [Deployment](#deployment)
12. [Implementation Phases](#implementation-phases)
13. [Type Definitions](#type-definitions)

---

## 1. Overview

### 1.1 Project Vision

Transform token-tracker from a local CLI tool into a social platform where developers can:

- **Submit** their AI token usage data from the command line
- **View** a global leaderboard ranking users by token consumption
- **Browse** individual profile pages featuring our signature 2D/3D contribution graphs
- **Compare** usage patterns across different AI providers and tools

### 1.2 Design Goals

| Goal | Description |
|------|-------------|
| **Privacy-first** | Users explicitly opt-in to share data. No automatic collection. |
| **Simplicity** | Minimal friction for CLI submission. One command to share. |
| **Beautiful** | Showcase our unique contribution graph visualizations. |
| **Transparent** | Open about what data is collected and how it's displayed. |

### 1.3 Non-Goals (v1)

- Real-time collaboration or social features (comments, likes)
- Team/organization accounts
- Paid tiers or premium features
- Mobile app
- Data export/import between users

### 1.4 Success Metrics

- Number of registered users
- Number of submissions per week
- Leaderboard page views
- Profile page views
- CLI command usage (`login`, `submit`)

---

## 2. System Architecture

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLIENTS                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────┐              ┌─────────────────┐                      │
│   │   Web Browser   │              │   CLI Tool      │                      │
│   │                 │              │                 │                      │
│   │  - Leaderboard  │              │  - login        │                      │
│   │  - Profiles     │              │  - logout       │                      │
│   │  - Settings     │              │  - whoami       │                      │
│   │  - Device Auth  │              │  - submit       │                      │
│   └────────┬────────┘              └────────┬────────┘                      │
│            │                                │                               │
│            │ HTTPS                          │ HTTPS                         │
│            │                                │                               │
└────────────┼────────────────────────────────┼───────────────────────────────┘
             │                                │
             ▼                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           NEXT.JS APPLICATION                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                        App Router (Pages)                            │   │
│   ├─────────────────────────────────────────────────────────────────────┤   │
│   │  /                    │ Leaderboard (SSR + ISR)                      │   │
│   │  /profile/[username]  │ User Profile (SSR + ISR)                     │   │
│   │  /settings            │ User Settings (Client, Protected)            │   │
│   │  /device              │ CLI Device Authorization (Client)            │   │
│   │  /admin               │ Admin Dashboard (Client, Protected)          │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                        API Routes (/api)                             │   │
│   ├─────────────────────────────────────────────────────────────────────┤   │
│   │  /api/auth/*          │ Authentication endpoints                     │   │
│   │  /api/submit          │ Data submission endpoint                     │   │
│   │  /api/leaderboard     │ Leaderboard data                            │   │
│   │  /api/users/*         │ User profile data                           │   │
│   │  /api/admin/*         │ Admin operations                            │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     Middleware (middleware.ts)                       │   │
│   ├─────────────────────────────────────────────────────────────────────┤   │
│   │  - Session validation                                                │   │
│   │  - Protected route enforcement                                       │   │
│   │  - Rate limiting headers                                            │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      │ Drizzle ORM
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                            POSTGRESQL DATABASE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│   │    users     │  │   sessions   │  │ submissions  │  │  api_tokens  │   │
│   ├──────────────┤  ├──────────────┤  ├──────────────┤  ├──────────────┤   │
│   │ id           │  │ id           │  │ id           │  │ id           │   │
│   │ github_id    │◄─┤ user_id      │  │ user_id      │──┤ user_id      │   │
│   │ username     │  │ token        │  │ total_tokens │  │ token        │   │
│   │ display_name │  │ expires_at   │  │ total_cost   │  │ name         │   │
│   │ avatar_url   │  │ source       │  │ date_start   │  │ expires_at   │   │
│   │ is_admin     │  │ user_agent   │  │ date_end     │  │ last_used_at │   │
│   │ created_at   │  │ created_at   │  │ status       │  │ created_at   │   │
│   └──────────────┘  └──────────────┘  │ ...          │  └──────────────┘   │
│                                       └──────┬───────┘                      │
│                                              │                              │
│                                              ▼                              │
│                                    ┌──────────────────┐                     │
│                                    │ daily_breakdown  │                     │
│                                    ├──────────────────┤                     │
│                                    │ id               │                     │
│                                    │ submission_id    │                     │
│                                    │ date             │                     │
│                                    │ tokens           │                     │
│                                    │ cost             │                     │
│                                    │ provider_breakdown│                    │
│                                    │ source_breakdown │                     │
│                                    │ model_breakdown  │                     │
│                                    └──────────────────┘                     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      │ GitHub API
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           EXTERNAL SERVICES                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│   GitHub OAuth (Authentication)                                              │
│   Vercel (Hosting)                                                          │
│   Neon/Vercel Postgres (Database)                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Data Flow

#### Submission Flow
```
┌───────┐    ┌─────────┐    ┌──────────┐    ┌────────┐    ┌──────────┐
│  CLI  │───▶│ /api/   │───▶│ Validate │───▶│ Store  │───▶│ Response │
│       │    │ submit  │    │ Data     │    │ in DB  │    │          │
└───────┘    └─────────┘    └──────────┘    └────────┘    └──────────┘
    │                            │
    │ Bearer Token               │ Check:
    │ (Authorization)            │ - Math validation
                                 │ - No negatives
                                 │ - No future dates
```

#### Authentication Flow (Device Code)
```
┌───────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  CLI  │───▶│ Request  │───▶│ Display  │    │ User     │    │ CLI      │
│       │    │ Device   │    │ Code +   │───▶│ Enters   │───▶│ Polls    │
│       │    │ Code     │    │ URL      │    │ Code in  │    │ for      │
│       │    │          │    │          │    │ Browser  │    │ Token    │
└───────┘    └──────────┘    └──────────┘    └──────────┘    └──────────┘
                                                                   │
                                                                   ▼
                                                            ┌──────────┐
                                                            │ Save     │
                                                            │ Token    │
                                                            │ Locally  │
                                                            └──────────┘
```

---

## 3. Project Structure

### 3.1 Directory Layout

```
frontend/
├── src/
│   ├── app/                          # Next.js App Router
│   │   ├── (auth)/                   # Auth-related pages (grouped)
│   │   │   ├── device/
│   │   │   │   └── page.tsx          # CLI device authorization
│   │   │   └── layout.tsx            # Minimal layout for auth pages
│   │   │
│   │   ├── (main)/                   # Main app pages (grouped)
│   │   │   ├── layout.tsx            # Main layout with nav
│   │   │   ├── page.tsx              # Leaderboard (home)
│   │   │   ├── profile/
│   │   │   │   └── [username]/
│   │   │   │       └── page.tsx      # User profile page
│   │   │   ├── settings/
│   │   │   │   └── page.tsx          # User settings (protected)
│   │   │   └── admin/
│   │   │       └── page.tsx          # Admin dashboard (protected)
│   │   │
│   │   ├── api/                      # API Routes
│   │   │   ├── auth/
│   │   │   │   ├── github/
│   │   │   │   │   ├── route.ts      # Initiate OAuth
│   │   │   │   │   └── callback/
│   │   │   │   │       └── route.ts  # OAuth callback
│   │   │   │   ├── device/
│   │   │   │   │   ├── route.ts      # Create device code
│   │   │   │   │   └── poll/
│   │   │   │   │       └── route.ts  # Poll for token
│   │   │   │   ├── session/
│   │   │   │   │   └── route.ts      # Get current session
│   │   │   │   └── logout/
│   │   │   │       └── route.ts      # Clear session
│   │   │   │
│   │   │   ├── submit/
│   │   │   │   └── route.ts          # Submit token data
│   │   │   │
│   │   │   ├── leaderboard/
│   │   │   │   └── route.ts          # Get leaderboard
│   │   │   │
│   │   │   ├── users/
│   │   │   │   └── [username]/
│   │   │   │       └── route.ts      # Get user profile
│   │   │   │
│   │   │   └── admin/
│   │   │       ├── flagged/
│   │   │       │   └── route.ts      # Get flagged submissions
│   │   │       └── review/
│   │   │           └── route.ts      # Review submission
│   │   │
│   │   ├── layout.tsx                # Root layout
│   │   ├── globals.css               # Global styles
│   │   └── not-found.tsx             # 404 page
│   │
│   ├── components/                   # React components
│   │   ├── ui/                       # Shadcn/UI components
│   │   │   ├── button.tsx
│   │   │   ├── input.tsx
│   │   │   ├── table.tsx
│   │   │   └── ...
│   │   │
│   │   ├── auth/                     # Auth-related components
│   │   │   ├── LoginButton.tsx
│   │   │   ├── UserMenu.tsx
│   │   │   └── DeviceCodeInput.tsx
│   │   │
│   │   ├── leaderboard/              # Leaderboard components
│   │   │   ├── LeaderboardTable.tsx
│   │   │   ├── LeaderboardRow.tsx
│   │   │   ├── PeriodSelector.tsx
│   │   │   └── HeroStats.tsx
│   │   │
│   │   ├── profile/                  # Profile page components
│   │   │   ├── UserHeader.tsx
│   │   │   ├── StatsCards.tsx
│   │   │   └── BreakdownCharts.tsx
│   │   │
│   │   ├── settings/                 # Settings components
│   │   │   ├── ApiTokenList.tsx
│   │   │   ├── SubmissionHistory.tsx
│   │   │   └── DangerZone.tsx
│   │   │
│   │   ├── graph/                    # Graph components (existing)
│   │   │   ├── GraphContainer.tsx
│   │   │   ├── ContributionGraph2D.tsx
│   │   │   ├── ContributionGraph3D.tsx
│   │   │   └── ...
│   │   │
│   │   └── shared/                   # Shared components
│   │       ├── BreakdownPanel.tsx    # (existing)
│   │       ├── ProviderLogo.tsx      # (existing)
│   │       ├── SourceLogo.tsx        # (existing)
│   │       ├── Navigation.tsx
│   │       └── Footer.tsx
│   │
│   ├── lib/                          # Utilities and libs
│   │   ├── db/                       # Database layer
│   │   │   ├── index.ts              # Drizzle client
│   │   │   ├── schema.ts             # Drizzle schema
│   │   │   └── migrations/           # SQL migrations
│   │   │
│   │   ├── auth/                     # Auth utilities
│   │   │   ├── session.ts            # Session management
│   │   │   ├── github.ts             # GitHub OAuth helpers
│   │   │   └── middleware.ts         # Auth middleware
│   │   │
│   │   ├── validation/               # Data validation
│   │   │   └── submission.ts         # Submission validators
│   │   │
│   │   ├── constants.ts              # (existing)
│   │   ├── themes.ts                 # (existing)
│   │   ├── types.ts                  # (existing)
│   │   └── utils.ts                  # (existing)
│   │
│   └── hooks/                        # React hooks
│       ├── useSession.ts             # Session hook
│       ├── useLeaderboard.ts         # Leaderboard data hook
│       └── useProfile.ts             # Profile data hook
│
├── drizzle.config.ts                 # Drizzle configuration
├── middleware.ts                     # Next.js middleware
└── ...

src/                                  # CLI source (existing)
├── cli.ts                            # Main CLI entry
├── auth.ts                           # NEW: Auth commands
├── submit.ts                         # NEW: Submit command
├── credentials.ts                    # NEW: Credential management
├── graph.ts                          # (existing)
├── native.ts                         # (existing)
└── ...
```

### 3.2 Package Dependencies

```json
{
  "dependencies": {
    // Existing
    "next": "^15.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    
    // Database
    "drizzle-orm": "^0.34.0",
    "@neondatabase/serverless": "^0.9.0",
    
    // Auth
    "oslo": "^1.2.0",  // Secure token generation
    
    // Validation
    "zod": "^3.23.0",
    
    // UI (if using shadcn)
    "@radix-ui/react-*": "*",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.5.0"
  },
  "devDependencies": {
    "drizzle-kit": "^0.25.0"
  }
}
```

---

## 4. Database Design

### 4.1 Entity Relationship Diagram

```
┌──────────────────────────────────────────────────────────────────────────┐
│                                                                          │
│  ┌─────────────────┐         ┌─────────────────┐                        │
│  │     users       │         │    sessions     │                        │
│  ├─────────────────┤         ├─────────────────┤                        │
│  │ id          PK  │◄────────│ user_id     FK  │                        │
│  │ github_id   UQ  │         │ id          PK  │                        │
│  │ username    UQ  │         │ token       UQ  │                        │
│  │ display_name    │         │ expires_at      │                        │
│  │ avatar_url      │         │ source          │                        │
│  │ email           │         │ user_agent      │                        │
│  │ is_admin        │         │ created_at      │                        │
│  │ created_at      │         └─────────────────┘                        │
│  │ updated_at      │                                                    │
│  └────────┬────────┘                                                    │
│           │                                                              │
│           │ 1:N                                                          │
│           │                                                              │
│           ▼                                                              │
│  ┌─────────────────┐         ┌─────────────────┐                        │
│  │   submissions   │         │   api_tokens    │                        │
│  ├─────────────────┤         ├─────────────────┤                        │
│  │ id          PK  │         │ id          PK  │                        │
│  │ user_id     FK  │◄────────│ user_id     FK  │                        │
│  │ total_tokens    │         │ token       UQ  │                        │
│  │ total_cost      │         │ name            │                        │
│  │ input_tokens    │         │ last_used_at    │                        │
│  │ output_tokens   │         │ expires_at      │                        │
│  │ cache_*_tokens  │         │ created_at      │                        │
│  │ date_start      │         └─────────────────┘                        │
│  │ date_end        │                                                    │
│  │ sources_used[]  │                                                    │
│  │ models_used[]   │                                                    │
│  │ status          │                                                    │
│  │ cli_version     │                                                    │
│  │ submission_hash │                                                    │
│  │ created_at      │                                                    │
│  └────────┬────────┘                                                    │
│           │                                                              │
│           │ 1:N                                                          │
│           │                                                              │
│           ▼                                                              │
│  ┌─────────────────┐                                                    │
│  │ daily_breakdown │                                                    │
│  ├─────────────────┤                                                    │
│  │ id          PK  │                                                    │
│  │ submission_id FK│                                                    │
│  │ date            │                                                    │
│  │ tokens          │                                                    │
│  │ cost            │                                                    │
│  │ input_tokens    │                                                    │
│  │ output_tokens   │                                                    │
│  │ provider_breakdown (JSONB)                                           │
│  │ source_breakdown   (JSONB)                                           │
│  │ model_breakdown    (JSONB)                                           │
│  └─────────────────┘                                                    │
│                                                                          │
│  ┌─────────────────┐                                                    │
│  │  device_codes   │  (Temporary, for CLI auth)                         │
│  ├─────────────────┤                                                    │
│  │ id          PK  │                                                    │
│  │ device_code UQ  │                                                    │
│  │ user_code   UQ  │                                                    │
│  │ user_id     FK  │  (NULL until authorized)                           │
│  │ expires_at      │                                                    │
│  │ created_at      │                                                    │
│  └─────────────────┘                                                    │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### 4.2 SQL Schema

```sql
-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- USERS TABLE
-- ============================================================================
-- Stores user accounts created via GitHub OAuth.
-- The username is the GitHub username (max 39 characters per GitHub rules).
-- ============================================================================
CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    github_id       INTEGER NOT NULL,
    username        VARCHAR(39) NOT NULL,
    display_name    VARCHAR(255),
    avatar_url      TEXT,
    email           VARCHAR(255),
    is_admin        BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT users_github_id_unique UNIQUE (github_id),
    CONSTRAINT users_username_unique UNIQUE (username)
);

CREATE INDEX idx_users_username ON users (username);
CREATE INDEX idx_users_github_id ON users (github_id);

-- ============================================================================
-- SESSIONS TABLE
-- ============================================================================
-- Stateful sessions for web authentication.
-- Using random tokens instead of JWTs for simpler revocation.
-- ============================================================================
CREATE TABLE sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL,
    token           VARCHAR(64) NOT NULL,
    expires_at      TIMESTAMP WITH TIME ZONE NOT NULL,
    source          VARCHAR(10) NOT NULL DEFAULT 'web',  -- 'web' | 'cli'
    user_agent      TEXT,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT sessions_token_unique UNIQUE (token),
    CONSTRAINT sessions_user_id_fk FOREIGN KEY (user_id) 
        REFERENCES users (id) ON DELETE CASCADE
);

CREATE INDEX idx_sessions_token ON sessions (token);
CREATE INDEX idx_sessions_user_id ON sessions (user_id);
CREATE INDEX idx_sessions_expires_at ON sessions (expires_at);

-- ============================================================================
-- API_TOKENS TABLE
-- ============================================================================
-- Long-lived tokens for CLI authentication.
-- Users can have multiple named tokens (e.g., "MacBook", "Work PC").
-- Tokens are prefixed with "tt_" for easy identification.
-- ============================================================================
CREATE TABLE api_tokens (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL,
    token           VARCHAR(64) NOT NULL,  -- Format: tt_<random>
    name            VARCHAR(100) NOT NULL,
    last_used_at    TIMESTAMP WITH TIME ZONE,
    expires_at      TIMESTAMP WITH TIME ZONE,  -- NULL = never expires
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT api_tokens_token_unique UNIQUE (token),
    CONSTRAINT api_tokens_user_name_unique UNIQUE (user_id, name),
    CONSTRAINT api_tokens_user_id_fk FOREIGN KEY (user_id) 
        REFERENCES users (id) ON DELETE CASCADE
);

CREATE INDEX idx_api_tokens_token ON api_tokens (token);
CREATE INDEX idx_api_tokens_user_id ON api_tokens (user_id);

-- ============================================================================
-- DEVICE_CODES TABLE
-- ============================================================================
-- Temporary codes for CLI device flow authentication.
-- Cleaned up by a scheduled job or on access.
-- ============================================================================
CREATE TABLE device_codes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_code     VARCHAR(32) NOT NULL,    -- Internal code (CLI uses this to poll)
    user_code       VARCHAR(9) NOT NULL,     -- Human-readable code (XXXX-XXXX)
    user_id         UUID,                    -- NULL until user authorizes
    device_name     VARCHAR(100),
    expires_at      TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT device_codes_device_code_unique UNIQUE (device_code),
    CONSTRAINT device_codes_user_code_unique UNIQUE (user_code),
    CONSTRAINT device_codes_user_id_fk FOREIGN KEY (user_id) 
        REFERENCES users (id) ON DELETE CASCADE
);

CREATE INDEX idx_device_codes_device_code ON device_codes (device_code);
CREATE INDEX idx_device_codes_user_code ON device_codes (user_code);
CREATE INDEX idx_device_codes_expires_at ON device_codes (expires_at);

-- ============================================================================
-- SUBMISSIONS TABLE
-- ============================================================================
-- Aggregated token usage data submitted by users.
-- Each submission covers a date range and includes daily breakdown.
-- ============================================================================
CREATE TABLE submissions (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                 UUID NOT NULL,
    
    -- Aggregated totals
    total_tokens            BIGINT NOT NULL,
    total_cost              DECIMAL(12, 4) NOT NULL,
    input_tokens            BIGINT NOT NULL,
    output_tokens           BIGINT NOT NULL,
    cache_creation_tokens   BIGINT NOT NULL DEFAULT 0,
    cache_read_tokens       BIGINT NOT NULL DEFAULT 0,
    
    -- Date range covered by this submission
    date_start              DATE NOT NULL,
    date_end                DATE NOT NULL,
    
    -- Metadata arrays (PostgreSQL native arrays)
    sources_used            TEXT[] NOT NULL,   -- ['opencode', 'claude', 'codex', 'gemini']
    models_used             TEXT[] NOT NULL,   -- ['claude-3-5-sonnet-20241022', 'gpt-4o', ...]
    
    -- Submission status
    status                  VARCHAR(20) NOT NULL DEFAULT 'verified',
    
    -- CLI metadata
    cli_version             VARCHAR(20),
    submission_hash         VARCHAR(64),       -- SHA256 hash for deduplication
    
    created_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT submissions_hash_unique UNIQUE (submission_hash),
    CONSTRAINT submissions_user_id_fk FOREIGN KEY (user_id) 
        REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT submissions_date_range_valid CHECK (date_start <= date_end)
);

CREATE INDEX idx_submissions_user_id ON submissions (user_id);
CREATE INDEX idx_submissions_status ON submissions (status);
CREATE INDEX idx_submissions_total_tokens ON submissions (total_tokens DESC);
CREATE INDEX idx_submissions_created_at ON submissions (created_at DESC);
CREATE INDEX idx_submissions_date_range ON submissions (date_start, date_end);

-- ============================================================================
-- DAILY_BREAKDOWN TABLE
-- ============================================================================
-- Per-day token counts for contribution graph visualization.
-- Linked to a submission; deleted when submission is deleted (CASCADE).
-- ============================================================================
CREATE TABLE daily_breakdown (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id       UUID NOT NULL,
    
    date                DATE NOT NULL,
    tokens              BIGINT NOT NULL,
    cost                DECIMAL(10, 4) NOT NULL,
    input_tokens        BIGINT NOT NULL,
    output_tokens       BIGINT NOT NULL,
    
    -- Flexible breakdown stored as JSONB
    -- Example: {"anthropic": 5000, "openai": 3000}
    provider_breakdown  JSONB,
    source_breakdown    JSONB,
    model_breakdown     JSONB,
    
    CONSTRAINT daily_breakdown_submission_date_unique 
        UNIQUE (submission_id, date),
    CONSTRAINT daily_breakdown_submission_id_fk FOREIGN KEY (submission_id) 
        REFERENCES submissions (id) ON DELETE CASCADE
);

CREATE INDEX idx_daily_breakdown_submission_id ON daily_breakdown (submission_id);
CREATE INDEX idx_daily_breakdown_date ON daily_breakdown (date);

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_submissions_updated_at
    BEFORE UPDATE ON submissions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

### 4.3 Drizzle Schema (TypeScript)

```typescript
// frontend/src/lib/db/schema.ts

import {
  pgTable,
  uuid,
  varchar,
  text,
  boolean,
  timestamp,
  bigint,
  decimal,
  date,
  jsonb,
  integer,
  index,
  unique,
  check,
} from 'drizzle-orm/pg-core';
import { relations } from 'drizzle-orm';

// ============================================================================
// USERS
// ============================================================================
export const users = pgTable('users', {
  id: uuid('id').primaryKey().defaultRandom(),
  githubId: integer('github_id').notNull().unique(),
  username: varchar('username', { length: 39 }).notNull().unique(),
  displayName: varchar('display_name', { length: 255 }),
  avatarUrl: text('avatar_url'),
  email: varchar('email', { length: 255 }),
  isAdmin: boolean('is_admin').notNull().default(false),
  createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow(),
  updatedAt: timestamp('updated_at', { withTimezone: true }).notNull().defaultNow(),
}, (table) => ({
  usernameIdx: index('idx_users_username').on(table.username),
  githubIdIdx: index('idx_users_github_id').on(table.githubId),
}));

export const usersRelations = relations(users, ({ many }) => ({
  sessions: many(sessions),
  apiTokens: many(apiTokens),
  submissions: many(submissions),
}));

// ============================================================================
// SESSIONS
// ============================================================================
export const sessions = pgTable('sessions', {
  id: uuid('id').primaryKey().defaultRandom(),
  userId: uuid('user_id').notNull().references(() => users.id, { onDelete: 'cascade' }),
  token: varchar('token', { length: 64 }).notNull().unique(),
  expiresAt: timestamp('expires_at', { withTimezone: true }).notNull(),
  source: varchar('source', { length: 10 }).notNull().default('web'),
  userAgent: text('user_agent'),
  createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow(),
}, (table) => ({
  tokenIdx: index('idx_sessions_token').on(table.token),
  userIdIdx: index('idx_sessions_user_id').on(table.userId),
  expiresAtIdx: index('idx_sessions_expires_at').on(table.expiresAt),
}));

export const sessionsRelations = relations(sessions, ({ one }) => ({
  user: one(users, {
    fields: [sessions.userId],
    references: [users.id],
  }),
}));

// ============================================================================
// API TOKENS
// ============================================================================
export const apiTokens = pgTable('api_tokens', {
  id: uuid('id').primaryKey().defaultRandom(),
  userId: uuid('user_id').notNull().references(() => users.id, { onDelete: 'cascade' }),
  token: varchar('token', { length: 64 }).notNull().unique(),
  name: varchar('name', { length: 100 }).notNull(),
  lastUsedAt: timestamp('last_used_at', { withTimezone: true }),
  expiresAt: timestamp('expires_at', { withTimezone: true }),
  createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow(),
}, (table) => ({
  tokenIdx: index('idx_api_tokens_token').on(table.token),
  userIdIdx: index('idx_api_tokens_user_id').on(table.userId),
  userNameUnique: unique('api_tokens_user_name_unique').on(table.userId, table.name),
}));

export const apiTokensRelations = relations(apiTokens, ({ one }) => ({
  user: one(users, {
    fields: [apiTokens.userId],
    references: [users.id],
  }),
}));

// ============================================================================
// DEVICE CODES
// ============================================================================
export const deviceCodes = pgTable('device_codes', {
  id: uuid('id').primaryKey().defaultRandom(),
  deviceCode: varchar('device_code', { length: 32 }).notNull().unique(),
  userCode: varchar('user_code', { length: 9 }).notNull().unique(),
  userId: uuid('user_id').references(() => users.id, { onDelete: 'cascade' }),
  deviceName: varchar('device_name', { length: 100 }),
  expiresAt: timestamp('expires_at', { withTimezone: true }).notNull(),
  createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow(),
}, (table) => ({
  deviceCodeIdx: index('idx_device_codes_device_code').on(table.deviceCode),
  userCodeIdx: index('idx_device_codes_user_code').on(table.userCode),
  expiresAtIdx: index('idx_device_codes_expires_at').on(table.expiresAt),
}));

// ============================================================================
// SUBMISSIONS
// ============================================================================
export const submissions = pgTable('submissions', {
  id: uuid('id').primaryKey().defaultRandom(),
  userId: uuid('user_id').notNull().references(() => users.id, { onDelete: 'cascade' }),
  
  totalTokens: bigint('total_tokens', { mode: 'number' }).notNull(),
  totalCost: decimal('total_cost', { precision: 12, scale: 4 }).notNull(),
  inputTokens: bigint('input_tokens', { mode: 'number' }).notNull(),
  outputTokens: bigint('output_tokens', { mode: 'number' }).notNull(),
  cacheCreationTokens: bigint('cache_creation_tokens', { mode: 'number' }).notNull().default(0),
  cacheReadTokens: bigint('cache_read_tokens', { mode: 'number' }).notNull().default(0),
  
  dateStart: date('date_start').notNull(),
  dateEnd: date('date_end').notNull(),
  
  sourcesUsed: text('sources_used').array().notNull(),
  modelsUsed: text('models_used').array().notNull(),
  
  status: varchar('status', { length: 20 }).notNull().default('verified'),
  
  cliVersion: varchar('cli_version', { length: 20 }),
  submissionHash: varchar('submission_hash', { length: 64 }).unique(),
  
  createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow(),
  updatedAt: timestamp('updated_at', { withTimezone: true }).notNull().defaultNow(),
}, (table) => ({
  userIdIdx: index('idx_submissions_user_id').on(table.userId),
  statusIdx: index('idx_submissions_status').on(table.status),
  totalTokensIdx: index('idx_submissions_total_tokens').on(table.totalTokens),
  createdAtIdx: index('idx_submissions_created_at').on(table.createdAt),
  dateRangeIdx: index('idx_submissions_date_range').on(table.dateStart, table.dateEnd),
}));

export const submissionsRelations = relations(submissions, ({ one, many }) => ({
  user: one(users, {
    fields: [submissions.userId],
    references: [users.id],
  }),
  dailyBreakdown: many(dailyBreakdown),
}));

// ============================================================================
// DAILY BREAKDOWN
// ============================================================================
export const dailyBreakdown = pgTable('daily_breakdown', {
  id: uuid('id').primaryKey().defaultRandom(),
  submissionId: uuid('submission_id').notNull().references(() => submissions.id, { onDelete: 'cascade' }),
  
  date: date('date').notNull(),
  tokens: bigint('tokens', { mode: 'number' }).notNull(),
  cost: decimal('cost', { precision: 10, scale: 4 }).notNull(),
  inputTokens: bigint('input_tokens', { mode: 'number' }).notNull(),
  outputTokens: bigint('output_tokens', { mode: 'number' }).notNull(),
  
  providerBreakdown: jsonb('provider_breakdown').$type<Record<string, number>>(),
  sourceBreakdown: jsonb('source_breakdown').$type<Record<string, number>>(),
  modelBreakdown: jsonb('model_breakdown').$type<Record<string, number>>(),
}, (table) => ({
  submissionIdIdx: index('idx_daily_breakdown_submission_id').on(table.submissionId),
  dateIdx: index('idx_daily_breakdown_date').on(table.date),
  submissionDateUnique: unique('daily_breakdown_submission_date_unique')
    .on(table.submissionId, table.date),
}));

export const dailyBreakdownRelations = relations(dailyBreakdown, ({ one }) => ({
  submission: one(submissions, {
    fields: [dailyBreakdown.submissionId],
    references: [submissions.id],
  }),
}));

// ============================================================================
// TYPE EXPORTS
// ============================================================================
export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;
export type Session = typeof sessions.$inferSelect;
export type NewSession = typeof sessions.$inferInsert;
export type ApiToken = typeof apiTokens.$inferSelect;
export type NewApiToken = typeof apiTokens.$inferInsert;
export type DeviceCode = typeof deviceCodes.$inferSelect;
export type NewDeviceCode = typeof deviceCodes.$inferInsert;
export type Submission = typeof submissions.$inferSelect;
export type NewSubmission = typeof submissions.$inferInsert;
export type DailyBreakdown = typeof dailyBreakdown.$inferSelect;
export type NewDailyBreakdown = typeof dailyBreakdown.$inferInsert;
```

### 4.4 Database Client

```typescript
// frontend/src/lib/db/index.ts

import { neon } from '@neondatabase/serverless';
import { drizzle } from 'drizzle-orm/neon-http';
import * as schema from './schema';

const sql = neon(process.env.DATABASE_URL!);

export const db = drizzle(sql, { schema });

export * from './schema';
```

---

## 5. Authentication System

### 5.1 Overview

The authentication system uses:
- **GitHub OAuth** for user identity
- **Stateful sessions** for web (stored in database, referenced by cookie)
- **API tokens** for CLI (long-lived, stored locally)
- **Device flow** for CLI login (user authorizes in browser)

### 5.2 Web Authentication Flow

```
User clicks "Sign in with GitHub"
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│ GET /api/auth/github                                        │
│                                                             │
│ 1. Generate random state (CSRF protection)                  │
│ 2. Store state in cookie (httpOnly, 10 min expiry)          │
│ 3. Redirect to GitHub OAuth authorize URL                   │
└─────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│ GitHub OAuth Consent Screen                                 │
│                                                             │
│ User grants permission to read:                             │
│ - Profile information (username, avatar)                    │
│ - Email address                                             │
└─────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│ GET /api/auth/github/callback?code=xxx&state=yyy            │
│                                                             │
│ 1. Validate state matches cookie (CSRF check)              │
│ 2. Exchange code for access_token with GitHub               │
│ 3. Fetch user profile from GitHub API                       │
│ 4. Upsert user in database                                  │
│ 5. Create session in database                               │
│ 6. Set session cookie (httpOnly, 30 day expiry)             │
│ 7. Redirect to homepage or returnTo URL                     │
└─────────────────────────────────────────────────────────────┘
```

### 5.3 CLI Device Flow

```
User runs `token-tracker login`
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│ CLI: POST /api/auth/device                                  │
│                                                             │
│ Request: { "deviceName": "MacBook Pro" }                    │
│                                                             │
│ Server:                                                     │
│ 1. Generate device_code (random, internal)                  │
│ 2. Generate user_code (XXXX-XXXX, human-readable)           │
│ 3. Store in device_codes table (15 min expiry)              │
│                                                             │
│ Response:                                                   │
│ {                                                           │
│   "deviceCode": "abc123...",                                │
│   "userCode": "ABCD-1234",                                  │
│   "verificationUrl": "https://token-tracker.dev/device",    │
│   "expiresIn": 900,                                         │
│   "interval": 5                                             │
│ }                                                           │
└─────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│ CLI displays:                                               │
│                                                             │
│   To authenticate, visit:                                   │
│   https://token-tracker.dev/device                          │
│                                                             │
│   And enter code: ABCD-1234                                 │
│                                                             │
│   Waiting for authorization...                              │
└─────────────────────────────────────────────────────────────┘
           │
           │ (CLI polls every 5 seconds)
           ▼
┌─────────────────────────────────────────────────────────────┐
│ User opens browser, navigates to /device                    │
│                                                             │
│ 1. User signs in with GitHub (if not already)               │
│ 2. User enters code: ABCD-1234                              │
│ 3. Server validates code, links to user                     │
│ 4. User sees "Device authorized!"                           │
└─────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│ CLI: POST /api/auth/device/poll                             │
│                                                             │
│ Request: { "deviceCode": "abc123..." }                      │
│                                                             │
│ Server:                                                     │
│ 1. Find device_code record                                  │
│ 2. Check if user_id is set (authorized)                     │
│ 3. If authorized:                                           │
│    - Create API token for user                              │
│    - Delete device_code record                              │
│    - Return token                                           │
│                                                             │
│ Response (success):                                         │
│ {                                                           │
│   "status": "complete",                                     │
│   "token": "tt_abc123...",                                  │
│   "user": { "username": "junhoyeo", "avatarUrl": "..." }    │
│ }                                                           │
└─────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│ CLI stores credentials locally:                             │
│                                                             │
│ ~/.config/token-tracker/credentials.json                    │
│ {                                                           │
│   "token": "tt_abc123...",                                  │
│   "username": "junhoyeo",                                   │
│   "avatarUrl": "https://...",                               │
│   "createdAt": "2024-12-03T00:00:00Z"                       │
│ }                                                           │
│                                                             │
│ CLI displays:                                               │
│   ✓ Logged in as junhoyeo                                   │
└─────────────────────────────────────────────────────────────┘
```

### 5.4 Session Management

```typescript
// frontend/src/lib/auth/session.ts

import { cookies } from 'next/headers';
import { db, sessions, users } from '@/lib/db';
import { eq, and, gt } from 'drizzle-orm';
import { generateRandomString } from './utils';

const SESSION_COOKIE_NAME = 'tt_session';
const SESSION_DURATION_MS = 30 * 24 * 60 * 60 * 1000; // 30 days

export interface SessionUser {
  id: string;
  username: string;
  displayName: string | null;
  avatarUrl: string | null;
  isAdmin: boolean;
}

/**
 * Get current session from request cookies.
 * Returns null if no valid session exists.
 */
export async function getSession(): Promise<SessionUser | null> {
  const cookieStore = await cookies();
  const sessionToken = cookieStore.get(SESSION_COOKIE_NAME)?.value;
  
  if (!sessionToken) {
    return null;
  }
  
  const result = await db
    .select({
      session: sessions,
      user: users,
    })
    .from(sessions)
    .innerJoin(users, eq(sessions.userId, users.id))
    .where(
      and(
        eq(sessions.token, sessionToken),
        gt(sessions.expiresAt, new Date())
      )
    )
    .limit(1);
  
  if (result.length === 0) {
    return null;
  }
  
  const { user } = result[0];
  
  return {
    id: user.id,
    username: user.username,
    displayName: user.displayName,
    avatarUrl: user.avatarUrl,
    isAdmin: user.isAdmin,
  };
}

/**
 * Create a new session for a user.
 */
export async function createSession(
  userId: string,
  options: { source?: 'web' | 'cli'; userAgent?: string } = {}
): Promise<string> {
  const token = generateRandomString(32);
  const expiresAt = new Date(Date.now() + SESSION_DURATION_MS);
  
  await db.insert(sessions).values({
    userId,
    token,
    expiresAt,
    source: options.source ?? 'web',
    userAgent: options.userAgent,
  });
  
  return token;
}

/**
 * Set session cookie.
 */
export async function setSessionCookie(token: string): Promise<void> {
  const cookieStore = await cookies();
  
  cookieStore.set(SESSION_COOKIE_NAME, token, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    maxAge: SESSION_DURATION_MS / 1000,
    path: '/',
  });
}

/**
 * Clear session cookie and delete session from database.
 */
export async function clearSession(): Promise<void> {
  const cookieStore = await cookies();
  const sessionToken = cookieStore.get(SESSION_COOKIE_NAME)?.value;
  
  if (sessionToken) {
    await db.delete(sessions).where(eq(sessions.token, sessionToken));
  }
  
  cookieStore.delete(SESSION_COOKIE_NAME);
}

/**
 * Validate API token for CLI requests.
 */
export async function validateApiToken(token: string): Promise<SessionUser | null> {
  if (!token.startsWith('tt_')) {
    return null;
  }
  
  const result = await db
    .select({
      apiToken: apiTokens,
      user: users,
    })
    .from(apiTokens)
    .innerJoin(users, eq(apiTokens.userId, users.id))
    .where(
      and(
        eq(apiTokens.token, token),
        or(
          isNull(apiTokens.expiresAt),
          gt(apiTokens.expiresAt, new Date())
        )
      )
    )
    .limit(1);
  
  if (result.length === 0) {
    return null;
  }
  
  // Update last_used_at
  await db
    .update(apiTokens)
    .set({ lastUsedAt: new Date() })
    .where(eq(apiTokens.token, token));
  
  const { user } = result[0];
  
  return {
    id: user.id,
    username: user.username,
    displayName: user.displayName,
    avatarUrl: user.avatarUrl,
    isAdmin: user.isAdmin,
  };
}
```

### 5.5 GitHub OAuth Helper

```typescript
// frontend/src/lib/auth/github.ts

const GITHUB_CLIENT_ID = process.env.GITHUB_CLIENT_ID!;
const GITHUB_CLIENT_SECRET = process.env.GITHUB_CLIENT_SECRET!;
const GITHUB_REDIRECT_URI = `${process.env.NEXT_PUBLIC_URL}/api/auth/github/callback`;

export interface GitHubUser {
  id: number;
  login: string;
  name: string | null;
  avatar_url: string;
  email: string | null;
}

/**
 * Get GitHub OAuth authorization URL.
 */
export function getAuthorizationUrl(state: string): string {
  const params = new URLSearchParams({
    client_id: GITHUB_CLIENT_ID,
    redirect_uri: GITHUB_REDIRECT_URI,
    scope: 'read:user user:email',
    state,
  });
  
  return `https://github.com/login/oauth/authorize?${params}`;
}

/**
 * Exchange authorization code for access token.
 */
export async function exchangeCodeForToken(code: string): Promise<string> {
  const response = await fetch('https://github.com/login/oauth/access_token', {
    method: 'POST',
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      client_id: GITHUB_CLIENT_ID,
      client_secret: GITHUB_CLIENT_SECRET,
      code,
      redirect_uri: GITHUB_REDIRECT_URI,
    }),
  });
  
  const data = await response.json();
  
  if (data.error) {
    throw new Error(`GitHub OAuth error: ${data.error_description || data.error}`);
  }
  
  return data.access_token;
}

/**
 * Fetch user profile from GitHub API.
 */
export async function getGitHubUser(accessToken: string): Promise<GitHubUser> {
  const response = await fetch('https://api.github.com/user', {
    headers: {
      'Authorization': `Bearer ${accessToken}`,
      'Accept': 'application/vnd.github.v3+json',
    },
  });
  
  if (!response.ok) {
    throw new Error(`Failed to fetch GitHub user: ${response.status}`);
  }
  
  return response.json();
}
```

---

## 6. API Specification

### 6.1 Authentication Endpoints

#### `GET /api/auth/github`

Initiates GitHub OAuth flow.

**Request**: None (browser redirect)

**Response**: Redirect to GitHub

**Implementation**:
```typescript
// frontend/src/app/api/auth/github/route.ts

import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { getAuthorizationUrl } from '@/lib/auth/github';
import { generateRandomString } from '@/lib/auth/utils';

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const returnTo = searchParams.get('returnTo') || '/';
  
  // Generate CSRF state
  const state = generateRandomString(32);
  
  // Store state and returnTo in cookie
  const cookieStore = await cookies();
  cookieStore.set('oauth_state', JSON.stringify({ state, returnTo }), {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    maxAge: 60 * 10, // 10 minutes
    path: '/',
  });
  
  // Redirect to GitHub
  return NextResponse.redirect(getAuthorizationUrl(state));
}
```

#### `GET /api/auth/github/callback`

Handles OAuth callback from GitHub.

**Request**: Query params `code`, `state`

**Response**: Redirect to homepage with session cookie set

**Error Responses**:
- `400` - Invalid state (CSRF)
- `400` - Missing code
- `500` - GitHub API error

#### `GET /api/auth/session`

Returns current user session.

**Request**: Cookie-based authentication

**Response**:
```typescript
// Authenticated
{
  "user": {
    "id": "uuid",
    "username": "junhoyeo",
    "displayName": "Junho Yeo",
    "avatarUrl": "https://avatars.githubusercontent.com/u/...",
    "isAdmin": false
  }
}

// Not authenticated
{
  "user": null
}
```

#### `POST /api/auth/logout`

Clears session.

**Request**: Cookie-based authentication

**Response**:
```typescript
{ "success": true }
```

#### `POST /api/auth/device`

Initiates device flow for CLI login.

**Request**:
```typescript
{
  "deviceName": "MacBook Pro"  // Optional
}
```

**Response**:
```typescript
{
  "deviceCode": "abc123def456...",      // Internal code (CLI uses to poll)
  "userCode": "ABCD-1234",              // Human-readable code
  "verificationUrl": "https://token-tracker.dev/device",
  "expiresIn": 900,                     // Seconds (15 minutes)
  "interval": 5                         // Poll interval in seconds
}
```

#### `POST /api/auth/device/poll`

CLI polls this endpoint to check authorization status.

**Request**:
```typescript
{
  "deviceCode": "abc123def456..."
}
```

**Response** (pending):
```typescript
{
  "status": "pending"
}
```

**Response** (authorized):
```typescript
{
  "status": "complete",
  "token": "tt_abc123...",
  "user": {
    "username": "junhoyeo",
    "avatarUrl": "https://..."
  }
}
```

**Response** (expired):
```typescript
{
  "status": "expired"
}
```

#### `POST /api/auth/device/authorize`

User authorizes device code (called from /device page).

**Request**:
```typescript
{
  "userCode": "ABCD-1234"
}
```

**Response**:
```typescript
{ "success": true }
```

**Error Responses**:
- `401` - Not authenticated
- `400` - Invalid or expired code

### 6.2 Data Submission Endpoint

#### `POST /api/submit`

Submit token usage data from CLI.

**Request Headers**:
```
Authorization: Bearer tt_abc123...
Content-Type: application/json
```

**Request Body**:
```typescript
{
  "data": {
    "contributions": [
      {
        "date": "2024-11-15",
        "count": 150000,
        "intensity": 4,
        "providers": { "anthropic": 100000, "openai": 50000 },
        "sources": { "claude": 80000, "opencode": 70000 },
        "models": { "claude-3-5-sonnet": 100000, "gpt-4o": 50000 },
        "cost": 1.50,
        "inputTokens": 100000,
        "outputTokens": 50000
      }
      // ... more days
    ],
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
```

**Response** (success):
```typescript
{
  "success": true,
  "submissionId": "uuid",
  "message": "Submission recorded successfully",
  "profileUrl": "https://token-tracker.dev/profile/junhoyeo"
}
```

**Error Responses**:
```typescript
// 401 - Unauthorized
{
  "success": false,
  "error": "unauthorized",
  "message": "Invalid or missing API token"
}

// 400 - Validation failed
{
  "success": false,
  "error": "validation_failed",
  "message": "Token counts don't add up: input + output != total",
  "details": {
    "expected": 1234567,
    "got": 1234000
  }
}

// 400 - Duplicate submission
{
  "success": false,
  "error": "duplicate",
  "message": "This data has already been submitted"
}
```

### 6.3 Leaderboard Endpoint

#### `GET /api/leaderboard`

Get ranked list of users.

**Query Parameters**:
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `period` | `all` \| `month` \| `week` | `all` | Time period filter |
| `page` | number | `1` | Page number (1-indexed) |
| `limit` | number | `50` | Items per page (max 100) |
| `sort` | `tokens` \| `cost` \| `days` | `tokens` | Sort field |

**Response**:
```typescript
{
  "users": [
    {
      "rank": 1,
      "username": "junhoyeo",
      "displayName": "Junho Yeo",
      "avatarUrl": "https://...",
      "totalTokens": 15000000,
      "totalCost": 150.00,
      "activeDays": 60,
      "topProvider": "anthropic",
      "topSource": "claude",
      "lastSubmission": "2024-12-01T12:00:00Z"
    }
    // ... more users
  ],
  "pagination": {
    "page": 1,
    "limit": 50,
    "total": 1234,
    "totalPages": 25,
    "hasNext": true,
    "hasPrev": false
  },
  "stats": {
    "totalUsers": 1234,
    "totalTokens": 5000000000,
    "totalCost": 50000.00
  }
}
```

### 6.4 User Profile Endpoint

#### `GET /api/users/[username]`

Get user profile and contribution data.

**Response**:
```typescript
{
  "user": {
    "username": "junhoyeo",
    "displayName": "Junho Yeo",
    "avatarUrl": "https://...",
    "joinedAt": "2024-10-01T00:00:00Z",
    "rank": 1
  },
  "stats": {
    "totalTokens": 15000000,
    "totalCost": 150.00,
    "inputTokens": 10000000,
    "outputTokens": 5000000,
    "cacheCreationTokens": 500000,
    "cacheReadTokens": 2000000,
    "activeDays": 60,
    "currentStreak": 15,
    "longestStreak": 30,
    "avgTokensPerDay": 250000,
    "peakDay": {
      "date": "2024-11-15",
      "tokens": 500000
    }
  },
  "contributions": [
    {
      "date": "2024-11-15",
      "count": 150000,
      "intensity": 4,
      "providers": { "anthropic": 100000, "openai": 50000 },
      "sources": { "claude": 80000, "opencode": 70000 },
      "models": { "claude-3-5-sonnet": 100000, "gpt-4o": 50000 },
      "cost": 1.50,
      "inputTokens": 100000,
      "outputTokens": 50000
    }
    // ... all days with data
  ],
  "breakdown": {
    "byProvider": {
      "anthropic": 12000000,
      "openai": 3000000
    },
    "bySource": {
      "claude": 8000000,
      "opencode": 5000000,
      "codex": 2000000
    },
    "byModel": {
      "claude-3-5-sonnet-20241022": 10000000,
      "gpt-4o": 3000000,
      "claude-3-haiku-20240307": 2000000
    }
  }
}
```

**Error Response**:
```typescript
// 404 - User not found
{
  "error": "not_found",
  "message": "User 'unknown' not found"
}
```

---

## 7. CLI Implementation

### 7.1 New Commands

```bash
# Authentication
token-tracker login           # Authenticate via GitHub device flow
token-tracker logout          # Clear local credentials
token-tracker whoami          # Show current authenticated user

# Submission
token-tracker submit          # Submit data to platform
token-tracker submit --dry-run    # Preview without submitting
```

### 7.2 Credentials Storage

**Location**: `~/.config/token-tracker/credentials.json`

```typescript
interface Credentials {
  token: string;          // API token (tt_xxx...)
  username: string;       // GitHub username
  avatarUrl: string;      // GitHub avatar URL
  createdAt: string;      // ISO timestamp
}
```

### 7.3 Implementation

```typescript
// src/credentials.ts

import fs from 'fs';
import path from 'path';
import os from 'os';

const CONFIG_DIR = path.join(os.homedir(), '.config', 'token-tracker');
const CREDENTIALS_FILE = path.join(CONFIG_DIR, 'credentials.json');

export interface Credentials {
  token: string;
  username: string;
  avatarUrl: string;
  createdAt: string;
}

export function loadCredentials(): Credentials | null {
  try {
    if (!fs.existsSync(CREDENTIALS_FILE)) {
      return null;
    }
    const content = fs.readFileSync(CREDENTIALS_FILE, 'utf-8');
    return JSON.parse(content);
  } catch {
    return null;
  }
}

export function saveCredentials(credentials: Credentials): void {
  fs.mkdirSync(CONFIG_DIR, { recursive: true });
  fs.writeFileSync(
    CREDENTIALS_FILE,
    JSON.stringify(credentials, null, 2),
    { mode: 0o600 }  // Owner read/write only
  );
}

export function clearCredentials(): void {
  if (fs.existsSync(CREDENTIALS_FILE)) {
    fs.unlinkSync(CREDENTIALS_FILE);
  }
}
```

```typescript
// src/auth.ts

import open from 'open';
import { loadCredentials, saveCredentials, clearCredentials } from './credentials';

const API_URL = process.env.TOKEN_TRACKER_API_URL || 'https://token-tracker.dev';

export async function login(): Promise<void> {
  // Check if already logged in
  const existing = loadCredentials();
  if (existing) {
    console.log(`Already logged in as ${existing.username}`);
    console.log('Run `token-tracker logout` to sign out first.');
    return;
  }
  
  // Request device code
  console.log('Requesting authorization...');
  
  const deviceResponse = await fetch(`${API_URL}/api/auth/device`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ deviceName: os.hostname() }),
  });
  
  if (!deviceResponse.ok) {
    console.error('Failed to initiate login');
    process.exit(1);
  }
  
  const { deviceCode, userCode, verificationUrl, interval } = await deviceResponse.json();
  
  // Display instructions
  console.log('\nTo authenticate, visit:');
  console.log(`  ${verificationUrl}`);
  console.log('\nAnd enter code:');
  console.log(`  ${userCode}`);
  console.log('\nWaiting for authorization...');
  
  // Open browser
  await open(verificationUrl);
  
  // Poll for completion
  let attempts = 0;
  const maxAttempts = 180; // 15 minutes at 5 second intervals
  
  while (attempts < maxAttempts) {
    await sleep(interval * 1000);
    attempts++;
    
    const pollResponse = await fetch(`${API_URL}/api/auth/device/poll`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ deviceCode }),
    });
    
    const result = await pollResponse.json();
    
    if (result.status === 'complete') {
      // Save credentials
      saveCredentials({
        token: result.token,
        username: result.user.username,
        avatarUrl: result.user.avatarUrl,
        createdAt: new Date().toISOString(),
      });
      
      console.log(`\n✓ Logged in as ${result.user.username}`);
      return;
    }
    
    if (result.status === 'expired') {
      console.error('\nAuthorization expired. Please try again.');
      process.exit(1);
    }
    
    // Still pending, continue polling
    process.stdout.write('.');
  }
  
  console.error('\nAuthorization timed out. Please try again.');
  process.exit(1);
}

export async function logout(): Promise<void> {
  const credentials = loadCredentials();
  
  if (!credentials) {
    console.log('Not logged in.');
    return;
  }
  
  clearCredentials();
  console.log(`Logged out from ${credentials.username}`);
}

export async function whoami(): Promise<void> {
  const credentials = loadCredentials();
  
  if (!credentials) {
    console.log('Not logged in.');
    console.log('Run `token-tracker login` to authenticate.');
    return;
  }
  
  console.log(`Logged in as: ${credentials.username}`);
  console.log(`Since: ${new Date(credentials.createdAt).toLocaleDateString()}`);
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}
```

```typescript
// src/submit.ts

import { loadCredentials } from './credentials';
import { generateGraphData } from './graph';
import { formatNumber } from './utils';
import readline from 'readline';

const API_URL = process.env.TOKEN_TRACKER_API_URL || 'https://token-tracker.dev';
const VERSION = require('../package.json').version;

interface SubmitOptions {
  dryRun?: boolean;
}

export async function submit(options: SubmitOptions = {}): Promise<void> {
  // 1. Check authentication
  const credentials = loadCredentials();
  if (!credentials) {
    console.error('Not logged in.');
    console.error('Run `token-tracker login` to authenticate.');
    process.exit(1);
  }
  
  // 2. Generate graph data
  console.log('Collecting token usage data...');
  const data = await generateGraphData();
  
  if (data.contributions.length === 0) {
    console.log('No token usage data found.');
    return;
  }
  
  // 3. Show preview
  console.log('\n┌─────────────────────────────────────────┐');
  console.log('│           Submission Preview            │');
  console.log('├─────────────────────────────────────────┤');
  console.log(`│ User:         ${credentials.username.padEnd(25)}│`);
  console.log(`│ Period:       ${data.dateRange.start} to ${data.dateRange.end} │`);
  console.log(`│ Total tokens: ${formatNumber(data.summary.totalTokens).padEnd(25)}│`);
  console.log(`│ Total cost:   $${data.summary.totalCost.toFixed(2).padEnd(24)}│`);
  console.log(`│ Active days:  ${String(data.summary.activeDays).padEnd(25)}│`);
  console.log(`│ Sources:      ${data.summary.sources.join(', ').padEnd(25)}│`);
  console.log('└─────────────────────────────────────────┘');
  
  if (options.dryRun) {
    console.log('\n(Dry run - no data submitted)');
    return;
  }
  
  // 4. Confirm
  const confirmed = await confirm('Submit this data?');
  if (!confirmed) {
    console.log('Cancelled.');
    return;
  }
  
  // 5. Submit
  console.log('\nSubmitting...');
  
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
  
  const result = await response.json();
  
  // 6. Handle response
  if (result.success) {
    console.log('\n✓ Submitted successfully!');
    console.log(`  View at: ${result.profileUrl}`);
  } else {
    console.error(`\n✗ Submission failed: ${result.message}`);
    process.exit(1);
  }
}

async function confirm(question: string): Promise<boolean> {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });
  
  return new Promise((resolve) => {
    rl.question(`${question} [y/N] `, (answer) => {
      rl.close();
      resolve(answer.toLowerCase() === 'y');
    });
  });
}
```

### 7.4 CLI Integration

```typescript
// src/cli.ts (additions)

import { login, logout, whoami } from './auth';
import { submit } from './submit';

// ... existing code ...

program
  .command('login')
  .description('Authenticate with token-tracker platform')
  .action(login);

program
  .command('logout')
  .description('Clear stored credentials')
  .action(logout);

program
  .command('whoami')
  .description('Show current authenticated user')
  .action(whoami);

program
  .command('submit')
  .description('Submit token usage data to platform')
  .option('--dry-run', 'Preview without submitting')
  .action((options) => submit(options));
```

---

## 8. Frontend Implementation

### 8.1 Leaderboard Page

```tsx
// frontend/src/app/(main)/page.tsx

import { Suspense } from 'react';
import { HeroStats } from '@/components/leaderboard/HeroStats';
import { LeaderboardTable } from '@/components/leaderboard/LeaderboardTable';
import { PeriodSelector } from '@/components/leaderboard/PeriodSelector';
import { Pagination } from '@/components/ui/pagination';

interface SearchParams {
  period?: 'all' | 'month' | 'week';
  page?: string;
  sort?: 'tokens' | 'cost' | 'days';
}

export default async function LeaderboardPage({
  searchParams,
}: {
  searchParams: SearchParams;
}) {
  const period = searchParams.period || 'all';
  const page = parseInt(searchParams.page || '1', 10);
  const sort = searchParams.sort || 'tokens';
  
  // Fetch data server-side
  const response = await fetch(
    `${process.env.NEXT_PUBLIC_URL}/api/leaderboard?period=${period}&page=${page}&sort=${sort}`,
    { next: { revalidate: 300 } }  // Cache for 5 minutes
  );
  const data = await response.json();
  
  return (
    <main className="container mx-auto px-4 py-8">
      {/* Hero Stats */}
      <HeroStats
        totalUsers={data.stats.totalUsers}
        totalTokens={data.stats.totalTokens}
        totalCost={data.stats.totalCost}
      />
      
      {/* Controls */}
      <div className="flex items-center justify-between my-8">
        <h2 className="text-2xl font-bold">Leaderboard</h2>
        <PeriodSelector value={period} />
      </div>
      
      {/* Table */}
      <Suspense fallback={<div>Loading...</div>}>
        <LeaderboardTable users={data.users} />
      </Suspense>
      
      {/* Pagination */}
      <Pagination
        currentPage={page}
        totalPages={data.pagination.totalPages}
        className="mt-8"
      />
    </main>
  );
}

// Enable ISR
export const revalidate = 300;
```

### 8.2 Profile Page

```tsx
// frontend/src/app/(main)/profile/[username]/page.tsx

import { notFound } from 'next/navigation';
import { UserHeader } from '@/components/profile/UserHeader';
import { StatsCards } from '@/components/profile/StatsCards';
import { GraphContainer } from '@/components/graph/GraphContainer';
import { BreakdownPanel } from '@/components/shared/BreakdownPanel';
import { BreakdownCharts } from '@/components/profile/BreakdownCharts';

interface Props {
  params: { username: string };
}

async function getUser(username: string) {
  const response = await fetch(
    `${process.env.NEXT_PUBLIC_URL}/api/users/${username}`,
    { next: { revalidate: 60 } }  // Cache for 1 minute
  );
  
  if (!response.ok) {
    return null;
  }
  
  return response.json();
}

export default async function ProfilePage({ params }: Props) {
  const data = await getUser(params.username);
  
  if (!data) {
    notFound();
  }
  
  return (
    <main className="container mx-auto px-4 py-8">
      {/* User Header */}
      <UserHeader
        username={data.user.username}
        displayName={data.user.displayName}
        avatarUrl={data.user.avatarUrl}
        joinedAt={data.user.joinedAt}
        rank={data.user.rank}
      />
      
      {/* Stats Cards */}
      <StatsCards stats={data.stats} className="my-8" />
      
      {/* Graph + Breakdown */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 my-8">
        <div className="lg:col-span-2">
          <GraphContainer
            data={data.contributions}
            // Reuse existing graph component props
          />
        </div>
        <div>
          <BreakdownPanel
            data={data.contributions}
            // Reuse existing breakdown component props
          />
        </div>
      </div>
      
      {/* Breakdown Charts */}
      <BreakdownCharts breakdown={data.breakdown} className="my-8" />
    </main>
  );
}

// Generate static params for top users
export async function generateStaticParams() {
  const response = await fetch(`${process.env.NEXT_PUBLIC_URL}/api/leaderboard?limit=100`);
  const data = await response.json();
  
  return data.users.map((user: { username: string }) => ({
    username: user.username,
  }));
}

export const revalidate = 60;
```

### 8.3 Device Authorization Page

```tsx
// frontend/src/app/(auth)/device/page.tsx

'use client';

import { useState } from 'react';
import { useSession } from '@/hooks/useSession';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';

export default function DevicePage() {
  const { user, isLoading } = useSession();
  const [code, setCode] = useState('');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [error, setError] = useState('');
  
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setStatus('loading');
    setError('');
    
    try {
      const response = await fetch('/api/auth/device/authorize', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ userCode: code.toUpperCase() }),
      });
      
      if (!response.ok) {
        const data = await response.json();
        throw new Error(data.message || 'Invalid code');
      }
      
      setStatus('success');
    } catch (err) {
      setStatus('error');
      setError(err instanceof Error ? err.message : 'Something went wrong');
    }
  };
  
  if (isLoading) {
    return <div className="flex justify-center py-20">Loading...</div>;
  }
  
  return (
    <main className="min-h-screen flex items-center justify-center">
      <div className="max-w-md w-full p-8 text-center">
        <h1 className="text-2xl font-bold mb-6">Authorize CLI</h1>
        
        {!user ? (
          <>
            <p className="text-muted-foreground mb-6">
              Sign in to authorize the token-tracker CLI.
            </p>
            <Button asChild size="lg">
              <a href="/api/auth/github?returnTo=/device">
                Sign in with GitHub
              </a>
            </Button>
          </>
        ) : status === 'success' ? (
          <>
            <div className="text-green-500 text-5xl mb-4">✓</div>
            <p className="text-lg">Device authorized!</p>
            <p className="text-muted-foreground mt-2">
              You can close this window and return to your terminal.
            </p>
          </>
        ) : (
          <form onSubmit={handleSubmit}>
            <p className="text-muted-foreground mb-6">
              Enter the code shown in your terminal:
            </p>
            
            <Input
              value={code}
              onChange={(e) => setCode(e.target.value.toUpperCase())}
              placeholder="XXXX-XXXX"
              className="text-center text-2xl tracking-widest font-mono"
              maxLength={9}
            />
            
            {error && (
              <p className="text-red-500 text-sm mt-2">{error}</p>
            )}
            
            <Button
              type="submit"
              size="lg"
              className="mt-6 w-full"
              disabled={code.length < 9 || status === 'loading'}
            >
              {status === 'loading' ? 'Authorizing...' : 'Authorize'}
            </Button>
          </form>
        )}
      </div>
    </main>
  );
}
```

---

## 9. Data Validation

### 9.1 Validation Rules

Basic validation is performed on all submissions. Invalid submissions are rejected immediately.

```typescript
// frontend/src/lib/validation/submission.ts

import { z } from 'zod';

// Schema for daily contribution
const dailyContributionSchema = z.object({
  date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
  count: z.number().int().nonnegative(),
  intensity: z.number().int().min(0).max(4),
  providers: z.record(z.string(), z.number().nonnegative()),
  sources: z.record(z.string(), z.number().nonnegative()),
  models: z.record(z.string(), z.number().nonnegative()),
  cost: z.number().nonnegative(),
  inputTokens: z.number().int().nonnegative(),
  outputTokens: z.number().int().nonnegative(),
});

// Schema for submission data
export const submissionSchema = z.object({
  data: z.object({
    contributions: z.array(dailyContributionSchema),
    summary: z.object({
      totalTokens: z.number().int().nonnegative(),
      totalCost: z.number().nonnegative(),
      inputTokens: z.number().int().nonnegative(),
      outputTokens: z.number().int().nonnegative(),
      cacheCreationTokens: z.number().int().nonnegative(),
      cacheReadTokens: z.number().int().nonnegative(),
      activeDays: z.number().int().nonnegative(),
      avgTokensPerDay: z.number().nonnegative(),
      peakDay: z.object({
        date: z.string(),
        tokens: z.number().int().nonnegative(),
      }),
      sources: z.array(z.string()),
      providers: z.array(z.string()),
      models: z.array(z.string()),
    }),
    dateRange: z.object({
      start: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
      end: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
    }),
  }),
  cliVersion: z.string().optional(),
});

export type SubmissionInput = z.infer<typeof submissionSchema>;

/**
 * Validation result with detailed error info.
 */
export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

export interface ValidationError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

/**
 * Validate submission data.
 * Returns validation errors if any rules are violated.
 */
export function validateSubmission(input: SubmissionInput): ValidationResult {
  const errors: ValidationError[] = [];
  const { data } = input;
  const { summary, contributions, dateRange } = data;
  
  // ─────────────────────────────────────────────────────────────────────────
  // Rule 1: Math validation - token counts must add up
  // ─────────────────────────────────────────────────────────────────────────
  const calculatedTotal = summary.inputTokens + summary.outputTokens;
  if (calculatedTotal !== summary.totalTokens) {
    errors.push({
      code: 'INVALID_TOKEN_SUM',
      message: 'Token counts do not add up: inputTokens + outputTokens != totalTokens',
      details: {
        inputTokens: summary.inputTokens,
        outputTokens: summary.outputTokens,
        expected: calculatedTotal,
        got: summary.totalTokens,
      },
    });
  }
  
  // Validate each daily contribution adds up correctly
  for (const day of contributions) {
    const dayTotal = day.inputTokens + day.outputTokens;
    if (dayTotal !== day.count) {
      errors.push({
        code: 'INVALID_DAILY_TOKEN_SUM',
        message: `Token counts do not add up for ${day.date}`,
        details: {
          date: day.date,
          inputTokens: day.inputTokens,
          outputTokens: day.outputTokens,
          expected: dayTotal,
          got: day.count,
        },
      });
    }
  }
  
  // ─────────────────────────────────────────────────────────────────────────
  // Rule 2: No negative values
  // ─────────────────────────────────────────────────────────────────────────
  const numericFields = [
    ['totalTokens', summary.totalTokens],
    ['totalCost', summary.totalCost],
    ['inputTokens', summary.inputTokens],
    ['outputTokens', summary.outputTokens],
    ['cacheCreationTokens', summary.cacheCreationTokens],
    ['cacheReadTokens', summary.cacheReadTokens],
  ] as const;
  
  for (const [field, value] of numericFields) {
    if (value < 0) {
      errors.push({
        code: 'NEGATIVE_VALUE',
        message: `Field '${field}' cannot be negative`,
        details: { field, value },
      });
    }
  }
  
  // ─────────────────────────────────────────────────────────────────────────
  // Rule 3: No future dates
  // ─────────────────────────────────────────────────────────────────────────
  const today = new Date();
  today.setHours(23, 59, 59, 999);
  
  const endDate = new Date(dateRange.end);
  if (endDate > today) {
    errors.push({
      code: 'FUTURE_DATE',
      message: 'Date range cannot include future dates',
      details: {
        endDate: dateRange.end,
        today: today.toISOString().split('T')[0],
      },
    });
  }
  
  // Check individual contributions for future dates
  for (const day of contributions) {
    const dayDate = new Date(day.date);
    if (dayDate > today) {
      errors.push({
        code: 'FUTURE_CONTRIBUTION_DATE',
        message: `Contribution date ${day.date} is in the future`,
        details: { date: day.date },
      });
    }
  }
  
  // ─────────────────────────────────────────────────────────────────────────
  // Rule 4: Valid date range
  // ─────────────────────────────────────────────────────────────────────────
  const startDate = new Date(dateRange.start);
  if (startDate > endDate) {
    errors.push({
      code: 'INVALID_DATE_RANGE',
      message: 'Start date must be before or equal to end date',
      details: {
        start: dateRange.start,
        end: dateRange.end,
      },
    });
  }
  
  // Maximum date range: 1 year
  const daysDiff = Math.floor((endDate.getTime() - startDate.getTime()) / (1000 * 60 * 60 * 24));
  if (daysDiff > 365) {
    errors.push({
      code: 'DATE_RANGE_TOO_LONG',
      message: 'Date range cannot exceed 1 year',
      details: {
        days: daysDiff,
        maxDays: 365,
      },
    });
  }
  
  return {
    valid: errors.length === 0,
    errors,
  };
}
```

---

## 10. Security Considerations

### 10.1 Authentication Security

| Measure | Implementation |
|---------|----------------|
| CSRF Protection | State parameter in OAuth flow, validated on callback |
| Session Tokens | Cryptographically random (32 bytes), stored hashed |
| Cookie Security | `httpOnly`, `secure` (production), `sameSite: 'lax'` |
| Token Prefix | API tokens prefixed with `tt_` for easy identification |
| Token Storage | Credentials file has `0600` permissions (owner only) |

### 10.2 API Security

| Measure | Implementation |
|---------|----------------|
| Input Validation | Zod schemas for all endpoints |
| Rate Limiting | Vercel Edge rate limiting (future) |
| SQL Injection | Drizzle ORM with parameterized queries |
| XSS Prevention | React's built-in escaping, no `dangerouslySetInnerHTML` |

### 10.3 Data Privacy

| Consideration | Approach |
|---------------|----------|
| Opt-in Only | Users must explicitly run `submit` command |
| Minimal Data | Only aggregate token counts, no content/prompts |
| No Tracking | No analytics on submission content |
| Delete on Request | Cascade delete removes all user data |

---

## 11. Deployment

### 11.1 Environment Variables

```bash
# Database (Neon or Vercel Postgres)
DATABASE_URL="postgres://user:pass@host/db?sslmode=require"

# GitHub OAuth
GITHUB_CLIENT_ID="Iv1.xxxxxxxxxx"
GITHUB_CLIENT_SECRET="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

# Application
NEXT_PUBLIC_URL="https://token-tracker.dev"
SESSION_SECRET="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"  # 32+ random bytes
```

### 11.2 Database Setup

```bash
# Install Drizzle CLI
pnpm add -D drizzle-kit

# Generate migrations
pnpm drizzle-kit generate:pg

# Apply migrations
pnpm drizzle-kit push:pg
```

### 11.3 Vercel Deployment

```json
// vercel.json
{
  "buildCommand": "pnpm run build",
  "outputDirectory": ".next",
  "framework": "nextjs",
  "regions": ["iad1"],  // US East for Neon proximity
  "env": {
    "DATABASE_URL": "@database-url",
    "GITHUB_CLIENT_ID": "@github-client-id",
    "GITHUB_CLIENT_SECRET": "@github-client-secret",
    "SESSION_SECRET": "@session-secret"
  }
}
```

---

## 12. Implementation Phases

### Phase 1: Core Infrastructure (Week 1)
- [ ] Set up Drizzle + PostgreSQL schema
- [ ] Implement database migrations
- [ ] Create GitHub OAuth endpoints
- [ ] Implement session management
- [ ] Create middleware for protected routes

### Phase 2: CLI Integration (Week 1-2)
- [ ] Implement device flow endpoints
- [ ] Add `login`, `logout`, `whoami` commands
- [ ] Add `submit` command with validation
- [ ] Create credential storage
- [ ] Implement submission API endpoint

### Phase 3: Frontend (Week 2-3)
- [ ] Create leaderboard page with pagination
- [ ] Create user profile page
- [ ] Integrate existing graph components
- [ ] Create settings page (API token management)
- [ ] Create device authorization page

### Phase 4: Polish (Week 3-4)
- [ ] Add ISR/caching strategy
- [ ] Error handling and edge cases
- [ ] Loading states and skeletons
- [ ] Mobile responsiveness
- [ ] Documentation

---

## 13. Type Definitions

### 13.1 Shared API Types

```typescript
// types/api.ts

// ─────────────────────────────────────────────────────────────────────────────
// Submission Types
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Leaderboard Types
// ─────────────────────────────────────────────────────────────────────────────

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

export interface LeaderboardResponse {
  users: LeaderboardUser[];
  pagination: Pagination;
  stats: GlobalStats;
}

export interface Pagination {
  page: number;
  limit: number;
  total: number;
  totalPages: number;
  hasNext: boolean;
  hasPrev: boolean;
}

export interface GlobalStats {
  totalUsers: number;
  totalTokens: number;
  totalCost: number;
}

// ─────────────────────────────────────────────────────────────────────────────
// Profile Types
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Auth Types
// ─────────────────────────────────────────────────────────────────────────────

export interface SessionUser {
  id: string;
  username: string;
  displayName: string | null;
  avatarUrl: string | null;
  isAdmin: boolean;
}

export interface DeviceCodeResponse {
  deviceCode: string;
  userCode: string;
  verificationUrl: string;
  expiresIn: number;
  interval: number;
}

export interface DevicePollResponse {
  status: 'pending' | 'complete' | 'expired';
  token?: string;
  user?: {
    username: string;
    avatarUrl: string;
  };
}

// ─────────────────────────────────────────────────────────────────────────────
// API Response Types
// ─────────────────────────────────────────────────────────────────────────────

export interface ApiSuccessResponse<T = unknown> {
  success: true;
  data?: T;
  message?: string;
}

export interface ApiErrorResponse {
  success: false;
  error: string;
  message: string;
  details?: Record<string, unknown>;
}

export type ApiResponse<T = unknown> = ApiSuccessResponse<T> | ApiErrorResponse;
```

---

## Appendix A: Drizzle Configuration

```typescript
// drizzle.config.ts

import type { Config } from 'drizzle-kit';

export default {
  schema: './src/lib/db/schema.ts',
  out: './src/lib/db/migrations',
  driver: 'pg',
  dbCredentials: {
    connectionString: process.env.DATABASE_URL!,
  },
  verbose: true,
  strict: true,
} satisfies Config;
```

---

## Appendix B: Middleware

```typescript
// frontend/middleware.ts

import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

// Routes that require authentication
const PROTECTED_ROUTES = ['/settings', '/admin'];

// Admin-only routes
const ADMIN_ROUTES = ['/admin'];

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  
  // Check if route is protected
  const isProtected = PROTECTED_ROUTES.some((route) =>
    pathname.startsWith(route)
  );
  
  if (!isProtected) {
    return NextResponse.next();
  }
  
  // Get session cookie
  const sessionToken = request.cookies.get('tt_session')?.value;
  
  if (!sessionToken) {
    // Redirect to login
    const loginUrl = new URL('/api/auth/github', request.url);
    loginUrl.searchParams.set('returnTo', pathname);
    return NextResponse.redirect(loginUrl);
  }
  
  // For admin routes, we'd need to verify the session and check isAdmin
  // This would require a database call, which is expensive in middleware
  // Instead, we handle admin checks in the page/API route itself
  
  return NextResponse.next();
}

export const config = {
  matcher: ['/settings/:path*', '/admin/:path*'],
};
```
