import * as fs from "node:fs";
import * as path from "node:path";
import { CACHE_TTL_MS, fetchWithRetry, getCacheDir, ensureCacheDir } from "../utils.js";

const PROVIDER_NAME = "litellm";
const CACHE_FILENAME = `pricing-${PROVIDER_NAME}.json`;
const PRICING_URL =
  "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";

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

interface CachedData {
  timestamp: number;
  data: PricingDataset;
}

export class LiteLLMProvider {
  private data: PricingDataset | null = null;
  private sortedKeys: string[] | null = null;

  private getCachePath(): string {
    return path.join(getCacheDir(), CACHE_FILENAME);
  }

  loadCache(): PricingDataset | null {
    if (this.data) return this.data;

    try {
      const cachePath = this.getCachePath();
      if (!fs.existsSync(cachePath)) return null;

      const content = fs.readFileSync(cachePath, "utf-8");
      const cached = JSON.parse(content) as CachedData;

      const age = Date.now() - cached.timestamp;
      if (age > CACHE_TTL_MS) return null;

      this.data = cached.data;
      this.sortedKeys = Object.keys(cached.data).sort();
      return this.data;
    } catch {
      return null;
    }
  }

  saveCache(data: PricingDataset): void {
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

  async fetch(): Promise<PricingDataset> {
    const cached = this.loadCache();
    if (cached) return cached;

    const response = await fetchWithRetry(PRICING_URL);
    if (!response.ok) {
      throw new Error(`Failed to fetch LiteLLM pricing: ${response.status}`);
    }

    this.data = (await response.json()) as PricingDataset;
    this.sortedKeys = Object.keys(this.data).sort();
    this.saveCache(this.data);
    return this.data;
  }

  getData(): PricingDataset | null {
    return this.data;
  }

  getSortedKeys(): string[] {
    return this.sortedKeys || (this.data ? Object.keys(this.data).sort() : []);
  }
}
