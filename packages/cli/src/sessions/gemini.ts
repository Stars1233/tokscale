/**
 * Gemini CLI session parser
 * Reads from ~/.gemini/tmp/{projectHash}/chats/session-*.json
 */

import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import { createUnifiedMessage, type UnifiedMessage, type TokenBreakdown } from "./types.js";

interface GeminiMessage {
  id: string;
  timestamp: string;
  type: "user" | "gemini";
  content: string;
  tokens?: {
    input: number;
    output: number;
    cached: number;
    thoughts: number;
    tool: number;
    total: number;
  };
  model?: string;
}

interface GeminiSession {
  sessionId: string;
  projectHash: string;
  startTime: string;
  lastUpdated: string;
  messages: GeminiMessage[];
}

export function getGeminiBasePath(): string {
  return path.join(os.homedir(), ".gemini");
}

export function parseGeminiMessages(): UnifiedMessage[] {
  const basePath = getGeminiBasePath();
  const tmpPath = path.join(basePath, "tmp");

  if (!fs.existsSync(tmpPath)) {
    return [];
  }

  const messages: UnifiedMessage[] = [];

  try {
    // Find all project directories
    const projectDirs = fs
      .readdirSync(tmpPath, { withFileTypes: true })
      .filter((d) => d.isDirectory())
      .map((d) => path.join(tmpPath, d.name));

    for (const projectDir of projectDirs) {
      const chatsDir = path.join(projectDir, "chats");
      if (!fs.existsSync(chatsDir)) continue;

      // Find all session JSON files
      const sessionFiles = fs
        .readdirSync(chatsDir)
        .filter((f) => f.startsWith("session-") && f.endsWith(".json"));

      for (const sessionFile of sessionFiles) {
        try {
          const content = fs.readFileSync(path.join(chatsDir, sessionFile), "utf-8");
          const session = JSON.parse(content) as GeminiSession;

          for (const msg of session.messages) {
            // Only process gemini messages with token data and timestamp
            if (msg.type !== "gemini" || !msg.tokens || !msg.model || !msg.timestamp) continue;

            const timestamp = new Date(msg.timestamp).getTime();

            // Skip invalid timestamps
            if (isNaN(timestamp)) continue;

            const tokens: TokenBreakdown = {
              input: msg.tokens.input || 0,
              output: msg.tokens.output || 0,
              cacheRead: msg.tokens.cached || 0,
              cacheWrite: 0, // Gemini doesn't track cache write
              reasoning: msg.tokens.thoughts || 0,
            };

            messages.push(
              createUnifiedMessage(
                "gemini",
                msg.model,
                "google",
                session.sessionId,
                timestamp,
                tokens
              )
            );
          }
        } catch {
          // Skip malformed files
        }
      }
    }
  } catch {
    // Skip inaccessible directories
  }

  return messages;
}
