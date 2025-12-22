/**
 * Source-level merge helpers for submission API
 */

// Type matching the JSONB structure in dailyBreakdown.sourceBreakdown
export interface SourceBreakdownData {
  tokens: number;
  cost: number;
  modelId: string;
  input: number;
  output: number;
  cacheRead: number;
  cacheWrite: number;
  messages: number;
}

// Type for recalculated day totals
interface DayTotals {
  tokens: number;
  cost: number;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
}

export function recalculateDayTotals(
  sourceBreakdown: Record<string, SourceBreakdownData>
): DayTotals {
  let tokens = 0;
  let cost = 0;
  let inputTokens = 0;
  let outputTokens = 0;
  let cacheReadTokens = 0;
  let cacheWriteTokens = 0;

  for (const source of Object.values(sourceBreakdown)) {
    tokens += source.tokens;
    cost += source.cost;
    inputTokens += source.input;
    outputTokens += source.output;
    cacheReadTokens += source.cacheRead;
    cacheWriteTokens += source.cacheWrite;
  }

  return {
    tokens,
    cost,
    inputTokens,
    outputTokens,
    cacheReadTokens,
    cacheWriteTokens,
  };
}

export function mergeSourceBreakdowns(
  existing: Record<string, SourceBreakdownData> | null | undefined,
  incoming: Record<string, SourceBreakdownData>,
  incomingSources: Set<string>
): Record<string, SourceBreakdownData> {
  const merged: Record<string, SourceBreakdownData> = { ...(existing || {}) };

  for (const sourceName of incomingSources) {
    if (incoming[sourceName]) {
      merged[sourceName] = { ...incoming[sourceName] };
    }
  }

  return merged;
}

export function buildModelBreakdown(
  sourceBreakdown: Record<string, SourceBreakdownData>
): Record<string, number> {
  const modelBreakdown: Record<string, number> = {};

  for (const source of Object.values(sourceBreakdown)) {
    if (source.modelId) {
      modelBreakdown[source.modelId] =
        (modelBreakdown[source.modelId] || 0) + source.tokens;
    }
  }

  return modelBreakdown;
}

/**
 * Convert SourceContribution from CLI format to SourceBreakdownData for DB
 */
export function sourceContributionToBreakdownData(
  source: {
    tokens: { input: number; output: number; cacheRead: number; cacheWrite: number };
    cost: number;
    modelId: string;
    messages: number;
  }
): SourceBreakdownData {
  return {
    tokens: source.tokens.input + source.tokens.output,
    cost: source.cost,
    modelId: source.modelId,
    input: source.tokens.input,
    output: source.tokens.output,
    cacheRead: source.tokens.cacheRead,
    cacheWrite: source.tokens.cacheWrite,
    messages: source.messages,
  };
}
