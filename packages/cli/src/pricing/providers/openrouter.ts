import * as fs from "node:fs";
import * as path from "node:path";
import { CACHE_TTL_MS, fetchWithRetry, getCacheDir, ensureCacheDir, parsePrice } from "../utils.js";
import type { LiteLLMModelPricing } from "./litellm.js";

const PROVIDER_NAME = "openrouter";
const CACHE_FILENAME = `pricing-${PROVIDER_NAME}.json`;

export const MODEL_MAPPING: Record<string, string> = {
  "glm-4.7": "z-ai/glm-4.7",
  "glm-4.7-free": "z-ai/glm-4.7",
};

const PROVIDER_NAME_MAPPING: Record<string, string> = {
  "z-ai": "Z.AI",
};

export interface OpenRouterEndpoint {
  name: string;
  model_name: string;
  provider_name: string;
  pricing: {
    prompt: string;
    completion: string;
    input_cache_read?: string;
    input_cache_write?: string;
  };
}

export interface OpenRouterEndpointsResponse {
  data: {
    id: string;
    name: string;
    endpoints: OpenRouterEndpoint[];
  };
}

interface CachedData {
  timestamp: number;
  data: Record<string, LiteLLMModelPricing>;
}

export class OpenRouterProvider {
  private data: Record<string, LiteLLMModelPricing> | null = null;

  private getCachePath(): string {
    return path.join(getCacheDir(), CACHE_FILENAME);
  }

  loadCache(): Record<string, LiteLLMModelPricing> | null {
    if (this.data) return this.data;

    const cachePath = this.getCachePath();
    try {
      if (!fs.existsSync(cachePath)) return null;

      const content = fs.readFileSync(cachePath, "utf-8");
      const cached = JSON.parse(content) as CachedData;

      if (!Number.isFinite(cached?.timestamp) || typeof cached?.data !== "object" || cached.data === null || Array.isArray(cached.data)) {
        fs.unlinkSync(cachePath);
        return null;
      }

      if (Object.keys(cached.data).length === 0) {
        fs.unlinkSync(cachePath);
        return null;
      }

      const age = Date.now() - cached.timestamp;
      if (age > CACHE_TTL_MS) return null;

      this.data = cached.data;
      return this.data;
    } catch {
      try { fs.unlinkSync(cachePath); } catch {}
      return null;
    }
  }

  saveCache(data: Record<string, LiteLLMModelPricing>): void {
    try {
      ensureCacheDir();
      const cached: CachedData = { timestamp: Date.now(), data };
      fs.writeFileSync(this.getCachePath(), JSON.stringify(cached), "utf-8");
    } catch {
      // Ignore cache write errors
    }
  }

  clearCache(): void {
    try {
      const cachePath = this.getCachePath();
      if (fs.existsSync(cachePath)) {
        fs.unlinkSync(cachePath);
      }
    } catch {
      // Ignore errors
    }
  }

  private async fetchModelEndpoints(author: string, slug: string): Promise<LiteLLMModelPricing | null> {
    const url = `https://openrouter.ai/api/v1/models/${author}/${slug}/endpoints`;

    const headers: Record<string, string> = { "Content-Type": "application/json" };
    const apiKey = process.env.OPENROUTER_API_KEY;
    if (apiKey) {
      headers["Authorization"] = `Bearer ${apiKey}`;
    }

    let response: Response;
    try {
      response = await fetchWithRetry(url, { headers, timeoutMs: 10000 });
    } catch (err) {
      if (process.env.DEBUG) {
        console.warn(`[OpenRouter] Fetch failed for ${author}/${slug}:`, (err as Error).message || err);
      }
      return null;
    }

    if (!response.ok) {
      if (process.env.DEBUG) {
        console.warn(`[OpenRouter] HTTP ${response.status} for ${author}/${slug}`);
      }
      return null;
    }

    const apiResponse = await response.json() as OpenRouterEndpointsResponse;

    if (!apiResponse?.data?.endpoints || !Array.isArray(apiResponse.data.endpoints)) {
      if (process.env.DEBUG) {
        console.warn(`[OpenRouter] Invalid response shape for ${author}/${slug}`);
      }
      return null;
    }

    const expectedProvider = PROVIDER_NAME_MAPPING[author.toLowerCase()] || author;
    const authorEndpoint = apiResponse.data.endpoints.find(
      endpoint => endpoint.provider_name?.toLowerCase() === expectedProvider.toLowerCase()
    );

    if (!authorEndpoint) {
      if (process.env.DEBUG) {
        console.warn(`[OpenRouter] Author provider "${expectedProvider}" not found for ${author}/${slug}`);
      }
      return null;
    }

    const inputCost = parsePrice(authorEndpoint.pricing?.prompt);
    const outputCost = parsePrice(authorEndpoint.pricing?.completion);

    if (inputCost === null || outputCost === null) {
      if (process.env.DEBUG) {
        const reason = String(authorEndpoint.pricing?.prompt) === "-1" || String(authorEndpoint.pricing?.completion) === "-1"
          ? "pricing unavailable (TBD)"
          : "invalid pricing values";
        console.warn(`[OpenRouter] ${reason} for ${author}/${slug}`);
      }
      return null;
    }

    return {
      input_cost_per_token: inputCost,
      output_cost_per_token: outputCost,
      cache_read_input_token_cost: parsePrice(authorEndpoint.pricing?.input_cache_read) ?? undefined,
      cache_creation_input_token_cost: parsePrice(authorEndpoint.pricing?.input_cache_write) ?? undefined,
    };
  }

  async fetch(modelID?: string): Promise<Record<string, LiteLLMModelPricing>> {
    const cached = this.loadCache();
    if (cached) return cached;

    const result: Record<string, LiteLLMModelPricing> = {};

    if (!modelID) {
      const uniqueIds = [...new Set(Object.values(MODEL_MAPPING))];
      await Promise.all(
        uniqueIds.map(async (id) => {
          const [author, slug] = id.split('/');
          if (!author || !slug) return;
          const pricing = await this.fetchModelEndpoints(author, slug);
          if (pricing) result[id] = pricing;
        })
      );
    } else {
      const [author, slug] = modelID.split('/');
      if (author && slug) {
        const pricing = await this.fetchModelEndpoints(author, slug);
        if (pricing) result[modelID] = pricing;
      }
    }

    this.data = result;
    if (Object.keys(result).length > 0) {
      this.saveCache(result);
    }

    return result;
  }

  getData(): Record<string, LiteLLMModelPricing> | null {
    return this.data;
  }
}
