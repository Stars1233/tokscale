/**
 * Gemini CLI data reader
 * Reads from ~/.gemini/tmp/{projectHash}/chats/session-*.json
 */

import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";

export interface GeminiUsageData {
  source: "gemini";
  model: string;
  messageCount: number;
  input: number;
  output: number;
  cached: number;
  thoughts: number;
  tool: number;
}

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

export function readGeminiSessions(): GeminiUsageData[] {
  const basePath = getGeminiBasePath();
  const tmpPath = path.join(basePath, "tmp");

  if (!fs.existsSync(tmpPath)) {
    return [];
  }

  const modelUsage = new Map<string, GeminiUsageData>();

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
          // Only process gemini messages with token data
          if (msg.type !== "gemini" || !msg.tokens || !msg.model) continue;

          const model = msg.model;
          let usage = modelUsage.get(model);
          if (!usage) {
            usage = {
              source: "gemini",
              model,
              messageCount: 0,
              input: 0,
              output: 0,
              cached: 0,
              thoughts: 0,
              tool: 0,
            };
            modelUsage.set(model, usage);
          }

          usage.messageCount++;
          usage.input += msg.tokens.input || 0;
          usage.output += msg.tokens.output || 0;
          usage.cached += msg.tokens.cached || 0;
          usage.thoughts += msg.tokens.thoughts || 0;
          usage.tool += msg.tokens.tool || 0;
        }
      } catch {
        // Skip malformed files
      }
    }
  }

  return Array.from(modelUsage.values());
}
