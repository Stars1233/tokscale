export interface ModelPricing {
  input_cost_per_token: number | null;
  output_cost_per_token: number | null;
  cache_read_input_token_cost?: number;
  cache_creation_input_token_cost?: number;
}

export interface PricingLookupResult {
  matchedKey: string;
  source: "litellm" | "openrouter";
  pricing: ModelPricing;
}
