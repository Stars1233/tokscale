/**
 * Droid (Factory.ai) session parser
 * Reads from ~/.factory/sessions/
 */

import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import { createUnifiedMessage, type UnifiedMessage, type TokenBreakdown } from "./types.js";

interface DroidSettingsJson {
  model?: string;
  providerLock?: string;
  providerLockTimestamp?: string;
  tokenUsage?: {
    inputTokens?: number;
    outputTokens?: number;
    cacheCreationTokens?: number;
    cacheReadTokens?: number;
    thinkingTokens?: number;
  };
}

export function getDroidSessionsPath(): string {
  return path.join(os.homedir(), ".factory", "sessions");
}

function findSettingsFiles(dir: string): string[] {
  const files: string[] = [];

  function walk(currentDir: string) {
    try {
      const entries = fs.readdirSync(currentDir, { withFileTypes: true });
      for (const entry of entries) {
        const fullPath = path.join(currentDir, entry.name);
        if (entry.isDirectory()) {
          walk(fullPath);
        } else if (entry.isFile() && entry.name.endsWith(".settings.json")) {
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

/**
 * Extract model name from Droid's custom model format
 * e.g., "custom:Claude-Opus-4.5-Thinking-[Anthropic]-0" -> "claude-opus-4-5-thinking-0"
 * e.g., "gemini-2.5-pro" -> "gemini-2-5-pro"
 * e.g., "Claude-Sonnet-4-[Anthropic]" -> "claude-sonnet-4"
 */
function normalizeModelName(model: string): string {
  // Remove "custom:" prefix if present
  let normalized = model.replace(/^custom:/, "");
  
  // Handle bracket notation like "Claude-Opus-4.5-Thinking-[Anthropic]-0"
  normalized = normalized.replace(/\[.*?\]/g, "").replace(/-+$/, "");
  
  // Convert to lowercase and clean up
  normalized = normalized.toLowerCase().replace(/\./g, "-").replace(/-+/g, "-");
  
  return normalized;
}

function getProviderFromModel(model: string): string {
  const lowerModel = model.toLowerCase();
  
  if (lowerModel.includes("claude") || lowerModel.includes("anthropic") || 
      lowerModel.includes("opus") || lowerModel.includes("sonnet") || lowerModel.includes("haiku")) {
    return "anthropic";
  }
  if (lowerModel.includes("gpt") || lowerModel.includes("openai") || 
      lowerModel.includes("o1") || lowerModel.includes("o3")) {
    return "openai";
  }
  if (lowerModel.includes("gemini") || lowerModel.includes("google")) {
    return "google";
  }
  if (lowerModel.includes("grok")) {
    return "xai";
  }
  
  return "unknown";
}

/**
 * Get default model name based on provider when model field is missing
 */
function getDefaultModelFromProvider(provider: string): string {
  switch (provider.toLowerCase()) {
    case "anthropic":
      return "claude-unknown";
    case "openai":
      return "gpt-unknown";
    case "google":
      return "gemini-unknown";
    case "xai":
      return "grok-unknown";
    default:
      return `${provider}-unknown`;
  }
}

/**
 * Try to extract model name from JSONL file's system-reminder
 * Looks for pattern: "Model: Claude Opus 4.5 Thinking [Anthropic]"
 */
function extractModelFromJsonl(jsonlPath: string): string | null {
  try {
    const content = fs.readFileSync(jsonlPath, "utf-8");
    // Look for Model: pattern in system-reminder
    const match = content.match(/Model:\s*([^\\"\\n]+?)(?:\s*\[|\s*\\n)/);
    if (match && match[1]) {
      const modelName = match[1].trim();
      return normalizeModelName(modelName);
    }
  } catch {
    // Ignore read errors
  }
  return null;
}

export function parseDroidMessages(): UnifiedMessage[] {
  const sessionsPath = getDroidSessionsPath();

  if (!fs.existsSync(sessionsPath)) {
    return [];
  }

  const messages: UnifiedMessage[] = [];
  const files = findSettingsFiles(sessionsPath);

  for (const file of files) {
    try {
      const content = fs.readFileSync(file, "utf-8");
      const settings = JSON.parse(content) as DroidSettingsJson;

      // Skip if no token usage data
      if (!settings.tokenUsage) continue;

      const usage = settings.tokenUsage;
      
      // Skip if no tokens were used
      const totalTokens = 
        (usage.inputTokens || 0) + 
        (usage.outputTokens || 0) + 
        (usage.cacheCreationTokens || 0) + 
        (usage.cacheReadTokens || 0) +
        (usage.thinkingTokens || 0);
      
      if (totalTokens === 0) continue;

      // Extract session ID from filename (e.g., "uuid.settings.json" -> "uuid")
      const sessionId = path.basename(file).replace(/\.settings\.json$/, "");
      
      // Get model and provider
      const provider = settings.providerLock || getProviderFromModel(settings.model || "");
      let model: string;
      if (settings.model) {
        model = normalizeModelName(settings.model);
      } else {
        // Try to extract from JSONL file
        const jsonlPath = file.replace(/\.settings\.json$/, ".jsonl");
        const extractedModel = extractModelFromJsonl(jsonlPath);
        model = extractedModel || getDefaultModelFromProvider(provider);
      }

      // Get timestamp from providerLockTimestamp or file mtime
      let timestamp: number;
      if (settings.providerLockTimestamp) {
        timestamp = new Date(settings.providerLockTimestamp).getTime();
      } else {
        const stats = fs.statSync(file);
        timestamp = stats.mtime.getTime();
      }

      // Skip invalid timestamps
      if (isNaN(timestamp)) continue;

      const tokens: TokenBreakdown = {
        input: usage.inputTokens || 0,
        output: usage.outputTokens || 0,
        cacheRead: usage.cacheReadTokens || 0,
        cacheWrite: usage.cacheCreationTokens || 0,
        reasoning: usage.thinkingTokens || 0,
      };

      messages.push(
        createUnifiedMessage(
          "droid",
          model,
          provider,
          sessionId,
          timestamp,
          tokens
        )
      );
    } catch {
      // Skip unreadable files
    }
  }

  return messages;
}
