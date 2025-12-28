/**
 * Pricing data fetcher using LiteLLM as source
 * Features disk caching with 1-hour TTL
 */

import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";

function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

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

const LITELLM_PRICING_URL =
  "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";

const OPENROUTER_API_URL = "https://openrouter.ai/api/v1/models";
const OPENROUTER_CACHE_FILENAME = "openrouter-pricing.json";

const CACHE_TTL_MS = 60 * 60 * 1000; // 1 hour

// OpenRouter API response types
export interface OpenRouterModel {
  id: string; // e.g., "z-ai/glm-4.7"
  pricing: {
    prompt: string; // Cost per token as string
    completion: string;
    input_cache_read?: string;
    input_cache_write?: string;
  };
}

export interface OpenRouterResponse {
  data: OpenRouterModel[];
}

/**
 * Response from /api/v1/models/{author}/{slug}/endpoints
 * Contains all available providers for a specific model
 */
export interface OpenRouterEndpoint {
  name: string;                 // e.g., "Z.AI | z-ai/glm-4.7-20251222"
  model_name: string;           // e.g., "Z.AI: GLM 4.7"
  provider_name: string;         // e.g., "Z.AI", "DeepInfra", "Chutes"
  pricing: {
    prompt: string;              // Cost per input token (string format)
    completion: string;          // Cost per output token (string format)
    input_cache_read?: string;   // Cache read cost (optional)
    input_cache_write?: string;  // Cache write cost (optional)
  };
}

export interface OpenRouterEndpointsResponse {
  data: {
    id: string;                  // e.g., "z-ai/glm-4.7"
    name: string;                // e.g., "Z.AI: GLM 4.7"
    endpoints: OpenRouterEndpoint[];
  };
}

// Manual mapping from model IDs to OpenRouter model IDs
// Format: { "local-model-id": "openrouter-provider/model-id" }
export const OPENROUTER_MODEL_MAPPING: Record<string, string> = {
  // GLM models - Z-AI is the primary/author provider
  "glm-4.7": "z-ai/glm-4.7",
  "glm-4-7": "z-ai/glm-4.7",
};

export interface LiteLLMModelPricing {
  input_cost_per_token?: number;
  output_cost_per_token?: number;
  cache_creation_input_token_cost?: number;
  cache_read_input_token_cost?: number;
  input_cost_per_token_above_200k_tokens?: number;
  output_cost_per_token_above_200k_tokens?: number;
  cache_creation_input_token_cost_above_200k_tokens?: number;
  cache_read_input_token_cost_above_200k_tokens?: number;
}

export type PricingDataset = Record<string, LiteLLMModelPricing>;

interface CachedPricing {
  timestamp: number;
  data: PricingDataset;
}

interface CachedOpenRouterPricing {
  timestamp: number;
  data: Record<string, LiteLLMModelPricing>;
}

/**
 * Format for passing pricing to Rust native module
 * Note: napi-rs expects undefined (not null) for Rust Option<T> fields
 */
export interface PricingEntry {
  modelId: string;
  pricing: {
    inputCostPerToken: number;
    outputCostPerToken: number;
    cacheReadInputTokenCost?: number;
    cacheCreationInputTokenCost?: number;
  };
}

function getCacheDir(): string {
  const cacheHome = process.env.XDG_CACHE_HOME || path.join(os.homedir(), ".cache");
  return path.join(cacheHome, "tokscale");
}

function getCachePath(): string {
  return path.join(getCacheDir(), "pricing.json");
}

function loadCachedPricing(): CachedPricing | null {
  try {
    const cachePath = getCachePath();
    if (!fs.existsSync(cachePath)) {
      return null;
    }

    const content = fs.readFileSync(cachePath, "utf-8");
    const cached = JSON.parse(content) as CachedPricing;

    // Check TTL
    const age = Date.now() - cached.timestamp;
    if (age > CACHE_TTL_MS) {
      return null; // Cache expired
    }

    return cached;
  } catch {
    return null;
  }
}

function saveCachedPricing(data: PricingDataset): void {
  try {
    const cacheDir = getCacheDir();
    if (!fs.existsSync(cacheDir)) {
      fs.mkdirSync(cacheDir, { recursive: true });
    }

    const cached: CachedPricing = {
      timestamp: Date.now(),
      data,
    };

    fs.writeFileSync(getCachePath(), JSON.stringify(cached), "utf-8");
  } catch {
    // Ignore cache write errors
  }
}

// OpenRouter cache functions
function getOpenRouterCachePath(): string {
  return path.join(getCacheDir(), OPENROUTER_CACHE_FILENAME);
}

function loadCachedOpenRouterPricing(): CachedOpenRouterPricing | null {
  try {
    const cachePath = getOpenRouterCachePath();
    if (!fs.existsSync(cachePath)) {
      return null;
    }

    const content = fs.readFileSync(cachePath, "utf-8");
    const cached = JSON.parse(content) as CachedOpenRouterPricing;

    // Check TTL (same as LiteLLM)
    const age = Date.now() - cached.timestamp;
    if (age > CACHE_TTL_MS) {
      return null; // Cache expired
    }

    return cached;
  } catch {
    return null;
  }
}

function saveOpenRouterCachedPricing(data: Record<string, LiteLLMModelPricing>): void {
  try {
    const cacheDir = getCacheDir();
    if (!fs.existsSync(cacheDir)) {
      fs.mkdirSync(cacheDir, { recursive: true });
    }

    const cached: CachedOpenRouterPricing = {
      timestamp: Date.now(),
      data,
    };

    fs.writeFileSync(getOpenRouterCachePath(), JSON.stringify(cached), "utf-8");
  } catch {
    // Ignore cache write errors
  }
}

/**
 * Convert OpenRouter pricing (string values) to LiteLLM format (number values)
 */
function convertOpenRouterToLiteLLM(model: OpenRouterModel): LiteLLMModelPricing {
  return {
    input_cost_per_token: parseFloat(model.pricing.prompt) || 0,
    output_cost_per_token: parseFloat(model.pricing.completion) || 0,
    cache_read_input_token_cost: model.pricing.input_cache_read
      ? parseFloat(model.pricing.input_cache_read)
      : undefined,
    cache_creation_input_token_cost: model.pricing.input_cache_write
      ? parseFloat(model.pricing.input_cache_write)
      : undefined,
  };
}

/**
 * Fetch endpoints for a specific OpenRouter model and extract author provider pricing
 * @param author - Provider/author from model ID (e.g., "z-ai")
 * @param slug - Model name (e.g., "glm-4.7")
 * @returns Pricing from author provider, or null if not found
 */
function fetchModelEndpoints(
  author: string,
  slug: string
): Promise<LiteLLMModelPricing | null> {
  return new Promise((resolve) => {
    const url = `https://openrouter.ai/api/v1/models/${author}/${slug}/endpoints`;

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 5000); // 5s timeout

    fetch(url, { signal: controller.signal })
      .then((response) => {
        clearTimeout(timeoutId);

        if (!response.ok) {
          console.warn(`Failed to fetch endpoints for ${author}/${slug}: ${response.status}`);
          return null;
        }

        return response.json() as Promise<OpenRouterEndpointsResponse>;
      })
      .then((apiResponse) => {
        if (!apiResponse) {
          resolve(null);
          return;
        }

        // Mapping for author names that don't match provider names exactly
        const providerNameMapping: Record<string, string> = {
          "z-ai": "Z.AI",
        };

        const expectedProvider = providerNameMapping[author.toLowerCase()] || author;

        // Find endpoint from author provider (case-insensitive match)
        const authorEndpoint = apiResponse.data.endpoints.find(
          endpoint => endpoint.provider_name.toLowerCase() === expectedProvider.toLowerCase()
        );

        if (!authorEndpoint) {
          console.warn(`No endpoint found for provider "${author}" in model ${author}/${slug}`);
          resolve(null);
          return;
        }

        // Convert to LiteLLM format
        resolve({
          input_cost_per_token: parseFloat(authorEndpoint.pricing.prompt) || 0,
          output_cost_per_token: parseFloat(authorEndpoint.pricing.completion) || 0,
          cache_read_input_token_cost: authorEndpoint.pricing.input_cache_read
            ? parseFloat(authorEndpoint.pricing.input_cache_read)
            : undefined,
          cache_creation_input_token_cost: authorEndpoint.pricing.input_cache_write
            ? parseFloat(authorEndpoint.pricing.input_cache_write)
            : undefined,
        });
      })
      .catch(() => {
        resolve(null);
      })
      .finally(() => {
        clearTimeout(timeoutId);
      });
  });
}

export class PricingFetcher {
  private pricingData: PricingDataset | null = null;
  private openRouterData: Record<string, LiteLLMModelPricing> | null = null;

  /**
   * Fetch pricing data (with disk cache, 1-hour TTL)
   * Also fetches OpenRouter pricing in parallel for fallback
   */
  async fetchPricing(): Promise<PricingDataset> {
    if (this.pricingData && this.openRouterData) return this.pricingData;

    // Try to load from cache first
    const cachedLiteLLM = loadCachedPricing();
    const cachedOpenRouter = loadCachedOpenRouterPricing();

    // Load LiteLLM from cache or fetch
    if (cachedLiteLLM) {
      this.pricingData = cachedLiteLLM.data;
    }

    // Load OpenRouter from cache or fetch
    if (cachedOpenRouter) {
      this.openRouterData = cachedOpenRouter.data;
    }

    // If both caches are loaded, we're done
    if (this.pricingData && this.openRouterData) {
      return this.pricingData;
    }

    // Fetch what's missing in parallel
    const promises: Promise<unknown>[] = [];

    if (!this.pricingData) {
      promises.push(this.fetchLiteLLMPricing());
    }

    if (!this.openRouterData) {
      promises.push(
        this.fetchOpenRouterPricing().catch(() => {
          // Ignore OpenRouter fetch errors - it's a fallback
        })
      );
    }

    await Promise.all(promises);

    // Ensure pricingData is set (should always be true at this point)
    if (!this.pricingData) {
      throw new Error("Failed to fetch LiteLLM pricing");
    }

    return this.pricingData;
  }

  /**
   * Fetch LiteLLM pricing from network
   */
  private async fetchLiteLLMPricing(): Promise<PricingDataset> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 15000);

    let response: Response;
    try {
      response = await fetch(LITELLM_PRICING_URL, { signal: controller.signal });
    } finally {
      clearTimeout(timeoutId);
    }

    if (!response.ok) {
      throw new Error(`Failed to fetch pricing: ${response.status}`);
    }

    this.pricingData = (await response.json()) as PricingDataset;

    // Save to cache
    saveCachedPricing(this.pricingData);

    return this.pricingData;
  }

  /**
   * Get raw pricing dataset
   */
  getPricingData(): PricingDataset | null {
    return this.pricingData;
  }

  /**
   * Convert pricing data to format expected by Rust native module
   * Includes both LiteLLM and OpenRouter fallback pricing
   */
  toPricingEntries(): PricingEntry[] {
    const entries: PricingEntry[] = [];

    // Add LiteLLM pricing
    if (this.pricingData) {
      for (const [modelId, pricing] of Object.entries(this.pricingData)) {
        entries.push({
          modelId,
          pricing: {
            inputCostPerToken: pricing.input_cost_per_token ?? 0,
            outputCostPerToken: pricing.output_cost_per_token ?? 0,
            // napi-rs expects undefined (not null) for Option<T> fields
            cacheReadInputTokenCost: pricing.cache_read_input_token_cost,
            cacheCreationInputTokenCost: pricing.cache_creation_input_token_cost,
          },
        });
      }
    }

    // Add OpenRouter fallback pricing for mapped models (if not already in LiteLLM)
    if (this.openRouterData) {
      for (const [localModelId, openRouterModelId] of Object.entries(OPENROUTER_MODEL_MAPPING)) {
        // Skip if already exists in LiteLLM
        if (this.pricingData && this.pricingData[localModelId]) continue;

        const pricing = this.openRouterData[openRouterModelId];
        if (pricing) {
          entries.push({
            modelId: localModelId,
            pricing: {
              inputCostPerToken: pricing.input_cost_per_token ?? 0,
              outputCostPerToken: pricing.output_cost_per_token ?? 0,
              cacheReadInputTokenCost: pricing.cache_read_input_token_cost,
              cacheCreationInputTokenCost: pricing.cache_creation_input_token_cost,
            },
          });
        }
      }
    }

    return entries;
  }

  getModelPricing(modelID: string): LiteLLMModelPricing | null {
    if (!this.pricingData) return null;

    // Direct lookup
    if (this.pricingData[modelID]) {
      return this.pricingData[modelID];
    }

    // Try with provider prefix
    const prefixes = ["anthropic/", "openai/", "google/", "bedrock/"];
    for (const prefix of prefixes) {
      if (this.pricingData[prefix + modelID]) {
        return this.pricingData[prefix + modelID];
      }
    }

    const normalized = normalizeModelName(modelID);
    if (normalized) {
      if (this.pricingData[normalized]) {
        return this.pricingData[normalized];
      }
      for (const prefix of prefixes) {
        if (this.pricingData[prefix + normalized]) {
          return this.pricingData[prefix + normalized];
        }
      }
    }

    const lowerModelID = modelID.toLowerCase();
    const lowerNormalized = normalized?.toLowerCase();
    const sortedKeys = Object.keys(this.pricingData).sort();

    for (const key of sortedKeys) {
      const lowerKey = key.toLowerCase();
      if (isWordBoundaryMatch(lowerKey, lowerModelID)) {
        return this.pricingData[key];
      }
      if (lowerNormalized && isWordBoundaryMatch(lowerKey, lowerNormalized)) {
        return this.pricingData[key];
      }
    }

    for (const key of sortedKeys) {
      const lowerKey = key.toLowerCase();
      if (isWordBoundaryMatch(lowerModelID, lowerKey)) {
        return this.pricingData[key];
      }
      if (lowerNormalized && isWordBoundaryMatch(lowerNormalized, lowerKey)) {
        return this.pricingData[key];
      }
    }

    // Fallback to OpenRouter for mapped models
    return this.getOpenRouterPricing(modelID);
  }

  calculateCost(
    tokens: {
      input: number;
      output: number;
      reasoning?: number;
      cacheRead: number;
      cacheWrite: number;
    },
    pricing: LiteLLMModelPricing
  ): number {
    const inputCost = tokens.input * (pricing.input_cost_per_token ?? 0);
    const outputCost =
      (tokens.output + (tokens.reasoning ?? 0)) * (pricing.output_cost_per_token ?? 0);
    const cacheWriteCost =
      tokens.cacheWrite * (pricing.cache_creation_input_token_cost ?? 0);
    const cacheReadCost =
      tokens.cacheRead * (pricing.cache_read_input_token_cost ?? 0);

    return inputCost + outputCost + cacheWriteCost + cacheReadCost;
  }

  /**
   * Fetch OpenRouter pricing data (lazy loading with disk cache)
   *
   * When called without modelID: loads from cache only (no network)
   * When called with modelID: fetches only that model's endpoints
   *
   * @param modelID - Optional OpenRouter model ID to fetch (e.g., "z-ai/glm-4.7")
   */
  private async fetchOpenRouterPricing(
    modelID?: string
  ): Promise<Record<string, LiteLLMModelPricing>> {
    if (this.openRouterData) return this.openRouterData;

    const cached = loadCachedOpenRouterPricing();
    if (cached) {
      this.openRouterData = cached.data;
      return this.openRouterData;
    }

    const normalizedData: Record<string, LiteLLMModelPricing> = {};

    if (!modelID) {
      const uniqueOpenRouterIds = [...new Set(Object.values(OPENROUTER_MODEL_MAPPING))];

      await Promise.all(
        uniqueOpenRouterIds.map(async (openRouterModelId) => {
          const [author, slug] = openRouterModelId.split('/');
          if (!author || !slug) return;

          const pricing = await fetchModelEndpoints(author, slug);
          if (pricing) {
            normalizedData[openRouterModelId] = pricing;
          }
        })
      );
    } else {
      const [author, slug] = modelID.split('/');
      if (author && slug) {
        const pricing = await fetchModelEndpoints(author, slug);
        if (pricing) {
          normalizedData[modelID] = pricing;
        }
      }
    }

    this.openRouterData = normalizedData;
    saveOpenRouterCachedPricing(this.openRouterData);

    return this.openRouterData;
  }

  /**
   * Look up pricing in OpenRouter using manual mapping
   */
  private getOpenRouterPricing(modelID: string): LiteLLMModelPricing | null {
    if (!this.openRouterData) return null;

    // Check manual mapping
    const lowerModelID = modelID.toLowerCase();
    const openRouterID = OPENROUTER_MODEL_MAPPING[lowerModelID];

    if (openRouterID && this.openRouterData[openRouterID]) {
      return this.openRouterData[openRouterID];
    }

    return null;
  }

  /**
   * Get model pricing with OpenRouter fallback
   * First tries LiteLLM, then falls back to OpenRouter for mapped models
   */
  async getModelPricingWithFallback(modelID: string): Promise<LiteLLMModelPricing | null> {
    // First try LiteLLM (existing logic)
    const liteLLMPricing = this.getModelPricing(modelID);
    if (liteLLMPricing) {
      return liteLLMPricing;
    }

    // Fallback to OpenRouter for mapped models only
    const lowerModelID = modelID.toLowerCase();
    const openRouterID = OPENROUTER_MODEL_MAPPING[lowerModelID];

    if (!openRouterID) {
      // Model not in manual mapping, no fallback available
      return null;
    }

    // Fetch OpenRouter data (lazily, only when needed)
    try {
      await this.fetchOpenRouterPricing(openRouterID);
      return this.getOpenRouterPricing(modelID);
    } catch {
      // OpenRouter fetch failed, return null
      return null;
    }
  }
}

/**
 * Clear pricing cache (for testing or forced refresh)
 */
export function clearPricingCache(): void {
  try {
    const cachePath = getCachePath();
    if (fs.existsSync(cachePath)) {
      fs.unlinkSync(cachePath);
    }
  } catch {
    // Ignore errors
  }
}

/**
 * Clear OpenRouter pricing cache (for testing or forced refresh)
 */
export function clearOpenRouterPricingCache(): void {
  try {
    const cachePath = getOpenRouterCachePath();
    if (fs.existsSync(cachePath)) {
      fs.unlinkSync(cachePath);
    }
  } catch {
    // Ignore errors
  }
}
