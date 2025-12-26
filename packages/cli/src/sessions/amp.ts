/**
 * Amp (Sourcegraph) session parser
 * Reads from ~/.local/share/amp/threads/
 */

import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import { createUnifiedMessage, type UnifiedMessage, type TokenBreakdown } from "./types.js";

interface AmpUsageEvent {
  timestamp: string;
  model: string;
  credits: number;
  tokens: {
    input: number;
    output: number;
    cacheReadInputTokens?: number;
    cacheCreationInputTokens?: number;
  };
  operationType: string;
}

interface AmpMessageUsage {
  model: string;
  inputTokens: number;
  outputTokens: number;
  cacheCreationInputTokens?: number;
  cacheReadInputTokens?: number;
  totalInputTokens?: number;
  credits: number;
}

interface AmpMessage {
  role: string;
  messageId: number;
  usage?: AmpMessageUsage;
}

interface AmpThread {
  id: string;
  created: number;
  messages: AmpMessage[];
  usageLedger?: {
    events: AmpUsageEvent[];
  };
}

export function getAmpThreadsPath(): string {
  return path.join(os.homedir(), ".local", "share", "amp", "threads");
}

function findThreadFiles(dir: string): string[] {
  const files: string[] = [];

  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isFile() && entry.name.endsWith(".json") && entry.name.startsWith("T-")) {
        files.push(path.join(dir, entry.name));
      }
    }
  } catch {
    // Skip inaccessible directories
  }

  return files;
}

function getProviderFromModel(model: string): string {
  const modelLower = model.toLowerCase();
  if (modelLower.includes("claude") || modelLower.includes("opus") || modelLower.includes("sonnet") || modelLower.includes("haiku")) {
    return "anthropic";
  }
  if (modelLower.includes("gpt") || modelLower.includes("o1") || modelLower.includes("o3")) {
    return "openai";
  }
  if (modelLower.includes("gemini")) {
    return "google";
  }
  if (modelLower.includes("grok")) {
    return "xai";
  }
  return "anthropic"; // Default for Amp
}

export function parseAmpMessages(): UnifiedMessage[] {
  const threadsPath = getAmpThreadsPath();

  if (!fs.existsSync(threadsPath)) {
    return [];
  }

  const messages: UnifiedMessage[] = [];
  const files = findThreadFiles(threadsPath);

  for (const file of files) {
    try {
      const content = fs.readFileSync(file, "utf-8");
      const thread = JSON.parse(content) as AmpThread;
      const threadId = thread.id || path.basename(file, ".json");

      // Parse from usageLedger.events (preferred - cleaner aggregated data)
      const beforeCount = messages.length;
      if (thread.usageLedger?.events) {
        for (const event of thread.usageLedger.events) {
          if (!event.timestamp || !event.model) continue;

          const timestamp = new Date(event.timestamp).getTime();
          if (isNaN(timestamp)) continue;

          const tokens: TokenBreakdown = {
            input: event.tokens?.input || 0,
            output: event.tokens?.output || 0,
            cacheRead: event.tokens?.cacheReadInputTokens || 0,
            cacheWrite: event.tokens?.cacheCreationInputTokens || 0,
            reasoning: 0,
          };

          messages.push(
            createUnifiedMessage(
              "amp",
              event.model,
              getProviderFromModel(event.model),
              threadId,
              timestamp,
              tokens,
              event.credits || 0
            )
          );
        }
      }

      // Fallback: If no messages added from ledger, parse from individual message usage
      if (messages.length === beforeCount) {
        for (const msg of thread.messages || []) {
          if (msg.role !== "assistant" || !msg.usage) continue;

          const usage = msg.usage;
          // Use thread created timestamp + messageId offset as approximation
          const timestamp = thread.created + (msg.messageId * 1000);

          const tokens: TokenBreakdown = {
            input: usage.inputTokens || 0,
            output: usage.outputTokens || 0,
            cacheRead: usage.cacheReadInputTokens || 0,
            cacheWrite: usage.cacheCreationInputTokens || 0,
            reasoning: 0,
          };

          messages.push(
            createUnifiedMessage(
              "amp",
              usage.model || "unknown",
              getProviderFromModel(usage.model || ""),
              threadId,
              timestamp,
              tokens,
              usage.credits || 0
            )
          );
        }
      }
    } catch {
      // Skip unreadable files
    }
  }

  return messages;
}
