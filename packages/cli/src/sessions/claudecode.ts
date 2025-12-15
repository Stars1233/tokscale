/**
 * Claude Code (Anthropic official) session parser
 * Reads from ~/.claude/projects/
 */

import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import { createUnifiedMessage, type UnifiedMessage, type TokenBreakdown } from "./types.js";

interface ClaudeCodeEntry {
  type: string;
  timestamp?: string;
  message?: {
    model?: string;
    usage?: {
      input_tokens?: number;
      output_tokens?: number;
      cache_read_input_tokens?: number;
      cache_creation_input_tokens?: number;
    };
  };
}

export function getClaudeCodeProjectsPath(): string {
  return path.join(os.homedir(), ".claude", "projects");
}

function findJsonlFiles(dir: string): string[] {
  const files: string[] = [];

  function walk(currentDir: string) {
    try {
      const entries = fs.readdirSync(currentDir, { withFileTypes: true });
      for (const entry of entries) {
        const fullPath = path.join(currentDir, entry.name);
        if (entry.isDirectory()) {
          walk(fullPath);
        } else if (entry.isFile() && entry.name.endsWith(".jsonl")) {
          files.push(fullPath);
        }
      }
    } catch {
      // Skip inaccessible directories
    }
  }

  walk(dir);
  return files;
}

export function parseClaudeCodeMessages(): UnifiedMessage[] {
  const projectsPath = getClaudeCodeProjectsPath();

  if (!fs.existsSync(projectsPath)) {
    return [];
  }

  const messages: UnifiedMessage[] = [];
  const files = findJsonlFiles(projectsPath);

  for (const file of files) {
    // Use file path as session ID
    const sessionId = path.relative(projectsPath, file).replace(/\.jsonl$/, "");

    try {
      const content = fs.readFileSync(file, "utf-8");
      const lines = content.split(/\r?\n/);

      for (const line of lines) {
        const trimmed = line.trim();
        if (!trimmed) continue;

        try {
          const entry = JSON.parse(trimmed) as ClaudeCodeEntry;

          // Process assistant messages with usage data and timestamp
          if (
            entry.type === "assistant" &&
            entry.message?.usage &&
            entry.timestamp
          ) {
            const model = entry.message.model || "unknown";
            const usage = entry.message.usage;
            const timestamp = new Date(entry.timestamp).getTime();

            // Skip invalid timestamps
            if (isNaN(timestamp)) continue;

            const tokens: TokenBreakdown = {
              input: usage.input_tokens || 0,
              output: usage.output_tokens || 0,
              cacheRead: usage.cache_read_input_tokens || 0,
              cacheWrite: usage.cache_creation_input_tokens || 0,
              reasoning: 0,
            };

            messages.push(
              createUnifiedMessage(
                "claude",
                model,
                "anthropic",
                sessionId,
                timestamp,
                tokens
              )
            );
          }
        } catch {
          // Skip malformed lines
        }
      }
    } catch {
      // Skip unreadable files
    }
  }

  return messages;
}
