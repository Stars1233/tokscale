/**
 * OpenCode session parser
 * Reads from ~/.local/share/opencode/storage/message/
 */

import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import { createUnifiedMessage, type UnifiedMessage, type TokenBreakdown } from "./types.js";

interface OpenCodeMessageFile {
  id: string;
  sessionID: string;
  role: "assistant" | "user";
  modelID?: string;
  providerID?: string;
  cost: number;
  tokens?: {
    input: number;
    output: number;
    reasoning: number;
    cache: {
      read: number;
      write: number;
    };
  };
  time: {
    created: number;
    completed?: number;
  };
  agent?: string;
  mode?: string;
}

export function normalizeAgentName(agent: string): string {
  const agentLower = agent.toLowerCase();

  if (agentLower.includes("plan")) {
    if (agentLower.includes("omo") || agentLower.includes("sisyphus")) {
      return "Planner-Sisyphus";
    }
    return agent;
  }

  if (agentLower === "omo" || agentLower === "sisyphus") {
    return "Sisyphus";
  }

  return agent;
}

export function getOpenCodeStoragePath(): string {
  const dataHome = process.env.XDG_DATA_HOME || path.join(os.homedir(), ".local", "share");
  return path.join(dataHome, "opencode", "storage");
}

export function parseOpenCodeMessages(): UnifiedMessage[] {
  const storagePath = getOpenCodeStoragePath();
  const messagePath = path.join(storagePath, "message");

  if (!fs.existsSync(messagePath)) {
    return [];
  }

  const messages: UnifiedMessage[] = [];
  
  try {
    const sessionDirs = fs
      .readdirSync(messagePath)
      .map((dir) => path.join(messagePath, dir))
      .filter((dir) => fs.statSync(dir).isDirectory());

    for (const sessionDir of sessionDirs) {
      const sessionId = path.basename(sessionDir);
      const files = fs.readdirSync(sessionDir).filter((f) => f.endsWith(".json"));

      for (const file of files) {
        try {
          const content = fs.readFileSync(path.join(sessionDir, file), "utf-8");
          const msg = JSON.parse(content) as OpenCodeMessageFile;

          if (msg.role === "assistant" && msg.tokens && msg.modelID) {
            const tokens: TokenBreakdown = {
              input: msg.tokens.input || 0,
              output: msg.tokens.output || 0,
              cacheRead: msg.tokens.cache?.read || 0,
              cacheWrite: msg.tokens.cache?.write || 0,
              reasoning: msg.tokens.reasoning || 0,
            };

            const agentOrMode = msg.mode || msg.agent;
            const agent = agentOrMode ? normalizeAgentName(agentOrMode) : undefined;

            messages.push(
              createUnifiedMessage(
                "opencode",
                msg.modelID,
                msg.providerID || "unknown",
                sessionId,
                msg.time.created,
                tokens,
                msg.cost || 0,
                agent
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
