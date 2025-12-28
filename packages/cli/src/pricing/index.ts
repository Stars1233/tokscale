import { LiteLLMProvider, type LiteLLMModelPricing, type PricingDataset } from "./providers/litellm.js";
import { OpenRouterProvider, MODEL_MAPPING as OPENROUTER_MODEL_MAPPING } from "./providers/openrouter.js";
import { normalizeModelName, isWordBoundaryMatch } from "./utils.js";

export type { LiteLLMModelPricing, PricingDataset };
export { normalizeModelName, isWordBoundaryMatch };
export { OPENROUTER_MODEL_MAPPING };

export interface PricingEntry {
  modelId: string;
  pricing: {
    inputCostPerToken: number;
    outputCostPerToken: number;
    cacheReadInputTokenCost?: number;
    cacheCreationInputTokenCost?: number;
  };
}

export type PricingSource = "litellm" | "openrouter";

export interface PricingLookupResult {
  pricing: LiteLLMModelPricing;
  source: PricingSource;
  matchedKey: string;
}

const PROVIDER_PREFIXES = ["anthropic/", "openai/", "google/", "bedrock/", "openrouter/"];

export class PricingFetcher {
  private litellm = new LiteLLMProvider();
  private openrouter = new OpenRouterProvider();

  async fetchPricing(): Promise<PricingDataset> {
    const litellmData = this.litellm.loadCache();
    const openrouterData = this.openrouter.loadCache();

    if (litellmData && openrouterData) {
      return litellmData;
    }

    const promises: Promise<void>[] = [];

    if (!litellmData) {
      promises.push(this.litellm.fetch().then(() => {}));
    }

    if (!openrouterData) {
      promises.push(
        this.openrouter.fetch().then(() => {}).catch((err: Error) => {
          if (process.env.DEBUG) {
            console.warn("[OpenRouter] Fallback pricing fetch failed:", err.message || err);
          }
        })
      );
    }

    await Promise.all(promises);

    const data = this.litellm.getData();
    if (!data) {
      throw new Error("Failed to fetch LiteLLM pricing");
    }

    return data;
  }

  getPricingData(): PricingDataset | null {
    return this.litellm.getData();
  }

  toPricingEntries(): PricingEntry[] {
    const entries: PricingEntry[] = [];
    const litellmData = this.litellm.getData();
    const openrouterData = this.openrouter.getData();

    if (litellmData) {
      for (const [modelId, pricing] of Object.entries(litellmData)) {
        entries.push({
          modelId,
          pricing: {
            inputCostPerToken: pricing.input_cost_per_token ?? 0,
            outputCostPerToken: pricing.output_cost_per_token ?? 0,
            cacheReadInputTokenCost: pricing.cache_read_input_token_cost,
            cacheCreationInputTokenCost: pricing.cache_creation_input_token_cost,
          },
        });
      }
    }

    if (openrouterData) {
      for (const [localModelId, openRouterModelId] of Object.entries(OPENROUTER_MODEL_MAPPING)) {
        if (litellmData && litellmData[localModelId]) continue;

        const pricing = openrouterData[openRouterModelId];
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
    return this.getModelPricingWithSource(modelID)?.pricing ?? null;
  }

  getModelPricingWithSource(modelID: string): PricingLookupResult | null {
    const litellmResult = this.lookupLiteLLM(modelID);
    if (litellmResult) return litellmResult;

    return this.lookupOpenRouter(modelID);
  }

  getModelPricingFromProvider(modelID: string, provider: PricingSource): PricingLookupResult | null {
    if (provider === "litellm") {
      return this.lookupLiteLLM(modelID);
    }
    return this.lookupOpenRouter(modelID);
  }

  private lookupLiteLLM(modelID: string): PricingLookupResult | null {
    const data = this.litellm.getData();
    if (!data) return null;

    if (data[modelID]) {
      return { pricing: data[modelID], source: "litellm", matchedKey: modelID };
    }

    for (const prefix of PROVIDER_PREFIXES) {
      const key = prefix + modelID;
      if (data[key]) {
        return { pricing: data[key], source: "litellm", matchedKey: key };
      }
    }

    const normalized = normalizeModelName(modelID);
    if (normalized) {
      if (data[normalized]) {
        return { pricing: data[normalized], source: "litellm", matchedKey: normalized };
      }
      for (const prefix of PROVIDER_PREFIXES) {
        const key = prefix + normalized;
        if (data[key]) {
          return { pricing: data[key], source: "litellm", matchedKey: key };
        }
      }
    }

    const lowerModelID = modelID.toLowerCase();
    const lowerNormalized = normalized?.toLowerCase();
    const sortedKeys = this.litellm.getSortedKeys();

    for (const key of sortedKeys) {
      const lowerKey = key.toLowerCase();
      if (isWordBoundaryMatch(lowerKey, lowerModelID)) {
        return { pricing: data[key], source: "litellm", matchedKey: key };
      }
      if (lowerNormalized && isWordBoundaryMatch(lowerKey, lowerNormalized)) {
        return { pricing: data[key], source: "litellm", matchedKey: key };
      }
    }

    for (const key of sortedKeys) {
      const lowerKey = key.toLowerCase();
      if (isWordBoundaryMatch(lowerModelID, lowerKey)) {
        return { pricing: data[key], source: "litellm", matchedKey: key };
      }
      if (lowerNormalized && isWordBoundaryMatch(lowerNormalized, lowerKey)) {
        return { pricing: data[key], source: "litellm", matchedKey: key };
      }
    }

    return null;
  }

  private lookupOpenRouter(modelID: string): PricingLookupResult | null {
    const data = this.openrouter.getData();
    if (!data) return null;

    let lowerModelID = modelID.toLowerCase();

    for (const prefix of PROVIDER_PREFIXES) {
      if (lowerModelID.startsWith(prefix)) {
        lowerModelID = lowerModelID.slice(prefix.length);
        break;
      }
    }

    const openRouterID = OPENROUTER_MODEL_MAPPING[lowerModelID];
    if (openRouterID && data[openRouterID]) {
      return {
        pricing: data[openRouterID],
        source: "openrouter",
        matchedKey: openRouterID,
      };
    }

    return null;
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
    const outputCost = (tokens.output + (tokens.reasoning ?? 0)) * (pricing.output_cost_per_token ?? 0);
    const cacheWriteCost = tokens.cacheWrite * (pricing.cache_creation_input_token_cost ?? 0);
    const cacheReadCost = tokens.cacheRead * (pricing.cache_read_input_token_cost ?? 0);

    return inputCost + outputCost + cacheWriteCost + cacheReadCost;
  }
}

export function clearPricingCache(): void {
  new LiteLLMProvider().clearCache();
}

export function clearOpenRouterPricingCache(): void {
  new OpenRouterProvider().clearCache();
}
