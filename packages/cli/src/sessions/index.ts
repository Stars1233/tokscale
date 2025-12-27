/**
 * Session parsers for different AI coding assistant formats
 *
 * This module provides TypeScript fallback parsers that match the Rust
 * implementations in packages/core/src/sessions/. Used when native module
 * is not available.
 */

export * from "./types.js";
export { parseOpenCodeMessages, getOpenCodeStoragePath } from "./opencode.js";
export { parseClaudeCodeMessages, getClaudeCodeProjectsPath } from "./claudecode.js";
export { parseCodexMessages, getCodexSessionsPath } from "./codex.js";
export { parseGeminiMessages, getGeminiBasePath } from "./gemini.js";
export { parseAmpMessages, getAmpThreadsPath } from "./amp.js";
export { parseDroidMessages, getDroidSessionsPath } from "./droid.js";

import type { UnifiedMessage, SourceType } from "./types.js";
import { parseOpenCodeMessages } from "./opencode.js";
import { parseClaudeCodeMessages } from "./claudecode.js";
import { parseCodexMessages } from "./codex.js";
import { parseGeminiMessages } from "./gemini.js";
import { parseAmpMessages } from "./amp.js";
import { parseDroidMessages } from "./droid.js";

export interface ParsedMessages {
  messages: UnifiedMessage[];
  opencodeCount: number;
  claudeCount: number;
  codexCount: number;
  geminiCount: number;
  ampCount: number;
  droidCount: number;
  processingTimeMs: number;
}

export interface ParseOptions {
  sources?: SourceType[];
  since?: string;
  until?: string;
  year?: string;
}

/**
 * Check if a message date is within the filter range
 */
function isDateInRange(date: string, since?: string, until?: string, year?: string): boolean {
  // Year filter
  if (year && !date.startsWith(year)) {
    return false;
  }

  // Since filter
  if (since && date < since) {
    return false;
  }

  // Until filter
  if (until && date > until) {
    return false;
  }

  return true;
}

/**
 * Parse all local session sources using TypeScript fallback parsers
 */
export function parseLocalSources(options: ParseOptions = {}): ParsedMessages {
  const startTime = performance.now();
  const sources = options.sources;

  const allMessages: UnifiedMessage[] = [];
  let opencodeCount = 0;
  let claudeCount = 0;
  let codexCount = 0;
  let geminiCount = 0;
  let ampCount = 0;
  let droidCount = 0;

  // Parse OpenCode
  if (!sources || sources.includes("opencode")) {
    const messages = parseOpenCodeMessages();
    for (const msg of messages) {
      if (isDateInRange(msg.date, options.since, options.until, options.year)) {
        allMessages.push(msg);
        opencodeCount++;
      }
    }
  }

  // Parse Claude Code
  if (!sources || sources.includes("claude")) {
    const messages = parseClaudeCodeMessages();
    for (const msg of messages) {
      if (isDateInRange(msg.date, options.since, options.until, options.year)) {
        allMessages.push(msg);
        claudeCount++;
      }
    }
  }

  // Parse Codex
  if (!sources || sources.includes("codex")) {
    const messages = parseCodexMessages();
    for (const msg of messages) {
      if (isDateInRange(msg.date, options.since, options.until, options.year)) {
        allMessages.push(msg);
        codexCount++;
      }
    }
  }

  // Parse Gemini
  if (!sources || sources.includes("gemini")) {
    const messages = parseGeminiMessages();
    for (const msg of messages) {
      if (isDateInRange(msg.date, options.since, options.until, options.year)) {
        allMessages.push(msg);
        geminiCount++;
      }
    }
  }

  // Parse Amp
  if (!sources || sources.includes("amp")) {
    const messages = parseAmpMessages();
    for (const msg of messages) {
      if (isDateInRange(msg.date, options.since, options.until, options.year)) {
        allMessages.push(msg);
        ampCount++;
      }
    }
  }

  // Parse Droid
  if (!sources || sources.includes("droid")) {
    const messages = parseDroidMessages();
    for (const msg of messages) {
      if (isDateInRange(msg.date, options.since, options.until, options.year)) {
        allMessages.push(msg);
        droidCount++;
      }
    }
  }

  const processingTimeMs = performance.now() - startTime;

  return {
    messages: allMessages,
    opencodeCount,
    claudeCount,
    codexCount,
    geminiCount,
    ampCount,
    droidCount,
    processingTimeMs,
  };
}
