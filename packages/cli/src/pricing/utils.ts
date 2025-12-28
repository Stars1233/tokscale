/**
 * Shared utilities for pricing providers
 */

import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";

// === Configuration ===

export const CACHE_TTL_MS = 60 * 60 * 1000; // 1 hour

const DEFAULT_FETCH_RETRIES = 2;
const DEFAULT_FETCH_TIMEOUT_MS = 15000;

function getFetchRetries(): number {
  const env = process.env.TOKSCALE_FETCH_RETRIES;
  if (env) {
    const parsed = parseInt(env, 10);
    if (Number.isFinite(parsed) && parsed >= 0) {
      return parsed;
    }
  }
  return DEFAULT_FETCH_RETRIES;
}

// === Model Name Normalization ===

export function normalizeModelName(modelId: string): string | null {
  const lower = modelId.toLowerCase();

  if (lower.includes("opus")) {
    if (lower.includes("4.5") || lower.includes("4-5")) {
      return "opus-4-5";
    } else if (lower.includes("4")) {
      return "opus-4";
    }
  }
  if (lower.includes("sonnet")) {
    if (lower.includes("4.5") || lower.includes("4-5")) {
      return "sonnet-4-5";
    } else if (lower.includes("4")) {
      return "sonnet-4";
    } else if (lower.includes("3.7") || lower.includes("3-7")) {
      return "sonnet-3-7";
    } else if (lower.includes("3.5") || lower.includes("3-5")) {
      return "sonnet-3-5";
    }
  }
  if (lower.includes("haiku") && (lower.includes("4.5") || lower.includes("4-5"))) {
    return "haiku-4-5";
  }

  if (lower === "o3") {
    return "o3";
  }
  if (lower.startsWith("gpt-4o") || lower === "gpt-4o") {
    return "gpt-4o";
  }
  if (lower.startsWith("gpt-4.1") || lower.includes("gpt-4.1")) {
    return "gpt-4.1";
  }

  if (lower.includes("gemini-2.5-pro")) {
    return "gemini-2.5-pro";
  }
  if (lower.includes("gemini-2.5-flash")) {
    return "gemini-2.5-flash";
  }

  if (lower === "big pickle" || lower === "big-pickle" || lower === "bigpickle") {
    return "glm-4.6";
  }

  return null;
}

export function isWordBoundaryMatch(haystack: string, needle: string): boolean {
  const pos = haystack.indexOf(needle);
  if (pos === -1) return false;

  const beforeOk = pos === 0 || !/[a-zA-Z0-9]/.test(haystack[pos - 1]);
  const afterOk =
    pos + needle.length === haystack.length ||
    !/[a-zA-Z0-9]/.test(haystack[pos + needle.length]);

  return beforeOk && afterOk;
}

// === Fetch with Retry ===

export interface FetchWithRetryOptions {
  headers?: Record<string, string>;
  timeoutMs?: number;
  retries?: number;
}

/** Respects Retry-After header (integer seconds only, no HTTP-date) */
export async function fetchWithRetry(
  url: string,
  options: FetchWithRetryOptions = {}
): Promise<Response> {
  const { headers, timeoutMs = DEFAULT_FETCH_TIMEOUT_MS, retries = getFetchRetries() } = options;
  let lastError: Error | null = null;

  for (let attempt = 0; attempt <= retries; attempt++) {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

    try {
      const response = await fetch(url, {
        signal: controller.signal,
        headers,
      });

      if (response.ok) {
        return response;
      }

      if ((response.status === 429 || response.status >= 500) && attempt < retries) {
        let delay = 1000 * (attempt + 1);

        const retryAfter = response.headers.get("Retry-After");
        if (retryAfter) {
          const parsed = parseInt(retryAfter, 10);
          if (Number.isFinite(parsed) && parsed > 0) {
            delay = Math.min(parsed * 1000, 5000);
          }
        }

        if (process.env.DEBUG) {
          console.warn(`[Fetch] HTTP ${response.status} for ${url}, retrying in ${delay}ms...`);
        }
        await new Promise((r) => setTimeout(r, delay));
        continue;
      }

      return response;
    } catch (err) {
      lastError = err as Error;
      if (attempt < retries) {
        const delay = 1000 * (attempt + 1);
        if (process.env.DEBUG) {
          console.warn(`[Fetch] Error for ${url}: ${lastError.message}, retrying in ${delay}ms...`);
        }
        await new Promise((r) => setTimeout(r, delay));
        continue;
      }
    } finally {
      clearTimeout(timeoutId);
    }
  }

  throw lastError || new Error(`Failed to fetch ${url} after ${retries + 1} attempts`);
}

// === Cache Directory Utilities ===

export function getCacheDir(): string {
  const cacheHome = process.env.XDG_CACHE_HOME || path.join(os.homedir(), ".cache");
  return path.join(cacheHome, "tokscale");
}

export function ensureCacheDir(): void {
  const cacheDir = getCacheDir();
  if (!fs.existsSync(cacheDir)) {
    fs.mkdirSync(cacheDir, { recursive: true });
  }
}

// === Price Parsing ===

export function parsePrice(value: unknown): number | null {
  if (value === undefined || value === null) return null;
  if (typeof value === "string") {
    const trimmed = value.trim();
    if (trimmed === "") return null;
    const num = Number(trimmed);
    if (!Number.isFinite(num) || num < 0) return null;
    return num;
  }
  if (typeof value === "number") {
    if (!Number.isFinite(value) || value < 0) return null;
    return value;
  }
  return null;
}
