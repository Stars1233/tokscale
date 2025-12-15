/**
 * TypeScript fallback report generators
 *
 * Used when native Rust module is not available.
 */

import type { UnifiedMessage, TokenBreakdown, SourceType } from "./types.js";
import type { PricingEntry } from "../pricing.js";
import type { TokenContributionData } from "../graph-types.js";

export interface ModelUsage {
  source: string;
  model: string;
  provider: string;
  input: number;
  output: number;
  cacheRead: number;
  cacheWrite: number;
  reasoning: number;
  messageCount: number;
  cost: number;
}

export interface ModelReport {
  entries: ModelUsage[];
  totalInput: number;
  totalOutput: number;
  totalCacheRead: number;
  totalCacheWrite: number;
  totalMessages: number;
  totalCost: number;
  processingTimeMs: number;
}

export interface MonthlyUsage {
  month: string;
  models: string[];
  input: number;
  output: number;
  cacheRead: number;
  cacheWrite: number;
  messageCount: number;
  cost: number;
}

export interface MonthlyReport {
  entries: MonthlyUsage[];
  totalCost: number;
  processingTimeMs: number;
}

interface PricingInfo {
  inputCostPerToken: number;
  outputCostPerToken: number;
  cacheReadInputTokenCost?: number;
  cacheCreationInputTokenCost?: number;
}

/**
 * Build a pricing lookup map from pricing entries
 */
function buildPricingMap(pricing: PricingEntry[]): Map<string, PricingInfo> {
  const map = new Map<string, PricingInfo>();
  for (const entry of pricing) {
    map.set(entry.modelId, entry.pricing);
  }
  return map;
}

/**
 * Calculate cost for a message based on token counts and pricing
 */
function calculateCost(
  tokens: TokenBreakdown,
  pricing: PricingInfo | undefined
): number {
  if (!pricing) return 0;

  let cost = 0;
  cost += tokens.input * pricing.inputCostPerToken;
  cost += tokens.output * pricing.outputCostPerToken;
  cost += tokens.cacheRead * (pricing.cacheReadInputTokenCost || 0);
  cost += tokens.cacheWrite * (pricing.cacheCreationInputTokenCost || 0);
  // Note: reasoning tokens are typically charged at output rate
  cost += tokens.reasoning * pricing.outputCostPerToken;

  return cost;
}

/**
 * Generate model report from parsed messages
 */
export function generateModelReport(
  messages: UnifiedMessage[],
  pricing: PricingEntry[],
  startTime: number
): ModelReport {
  const pricingMap = buildPricingMap(pricing);

  // Aggregate by source + model
  const aggregated = new Map<string, ModelUsage>();

  for (const msg of messages) {
    const key = `${msg.source}:${msg.modelId}`;
    let usage = aggregated.get(key);

    if (!usage) {
      usage = {
        source: msg.source,
        model: msg.modelId,
        provider: msg.providerId,
        input: 0,
        output: 0,
        cacheRead: 0,
        cacheWrite: 0,
        reasoning: 0,
        messageCount: 0,
        cost: 0,
      };
      aggregated.set(key, usage);
    }

    usage.input += msg.tokens.input;
    usage.output += msg.tokens.output;
    usage.cacheRead += msg.tokens.cacheRead;
    usage.cacheWrite += msg.tokens.cacheWrite;
    usage.reasoning += msg.tokens.reasoning;
    usage.messageCount++;

    const calculatedCost = calculateCost(msg.tokens, pricingMap.get(msg.modelId));
    usage.cost += calculatedCost > 0 ? calculatedCost : msg.cost;
  }

  const entries = Array.from(aggregated.values()).sort((a, b) => b.cost - a.cost);

  const totals = entries.reduce(
    (acc, e) => ({
      input: acc.input + e.input,
      output: acc.output + e.output,
      cacheRead: acc.cacheRead + e.cacheRead,
      cacheWrite: acc.cacheWrite + e.cacheWrite,
      messages: acc.messages + e.messageCount,
      cost: acc.cost + e.cost,
    }),
    { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, messages: 0, cost: 0 }
  );

  return {
    entries,
    totalInput: totals.input,
    totalOutput: totals.output,
    totalCacheRead: totals.cacheRead,
    totalCacheWrite: totals.cacheWrite,
    totalMessages: totals.messages,
    totalCost: totals.cost,
    processingTimeMs: performance.now() - startTime,
  };
}

/**
 * Generate monthly report from parsed messages
 */
export function generateMonthlyReport(
  messages: UnifiedMessage[],
  pricing: PricingEntry[],
  startTime: number
): MonthlyReport {
  const pricingMap = buildPricingMap(pricing);

  // Aggregate by month
  const aggregated = new Map<
    string,
    {
      models: Set<string>;
      input: number;
      output: number;
      cacheRead: number;
      cacheWrite: number;
      messageCount: number;
      cost: number;
    }
  >();

  for (const msg of messages) {
    // Extract YYYY-MM from date
    const month = msg.date.slice(0, 7);
    let usage = aggregated.get(month);

    if (!usage) {
      usage = {
        models: new Set(),
        input: 0,
        output: 0,
        cacheRead: 0,
        cacheWrite: 0,
        messageCount: 0,
        cost: 0,
      };
      aggregated.set(month, usage);
    }

    usage.models.add(msg.modelId);
    usage.input += msg.tokens.input;
    usage.output += msg.tokens.output;
    usage.cacheRead += msg.tokens.cacheRead;
    usage.cacheWrite += msg.tokens.cacheWrite;
    usage.messageCount++;

    const calculatedCost = calculateCost(msg.tokens, pricingMap.get(msg.modelId));
    usage.cost += calculatedCost > 0 ? calculatedCost : msg.cost;
  }

  const entries: MonthlyUsage[] = Array.from(aggregated.entries())
    .map(([month, data]) => ({
      month,
      models: Array.from(data.models),
      input: data.input,
      output: data.output,
      cacheRead: data.cacheRead,
      cacheWrite: data.cacheWrite,
      messageCount: data.messageCount,
      cost: data.cost,
    }))
    .sort((a, b) => b.month.localeCompare(a.month)); // Most recent first

  const totalCost = entries.reduce((sum, e) => sum + e.cost, 0);

  return {
    entries,
    totalCost,
    processingTimeMs: performance.now() - startTime,
  };
}

/**
 * Generate graph data from parsed messages
 */
export function generateGraphData(
  messages: UnifiedMessage[],
  pricing: PricingEntry[],
  startTime: number
): TokenContributionData {
  const pricingMap = buildPricingMap(pricing);

  // Group messages by date
  const byDate = new Map<
    string,
    {
      tokens: TokenBreakdown;
      cost: number;
      messages: number;
      sources: Map<
        string,
        {
          modelId: string;
          providerId: string;
          tokens: TokenBreakdown;
          cost: number;
          messages: number;
        }
      >;
    }
  >();

  const allSources = new Set<string>();
  const allModels = new Set<string>();
  let totalTokens = 0;
  let totalCost = 0;
  let maxCostInSingleDay = 0;

  for (const msg of messages) {
    allSources.add(msg.source);
    allModels.add(msg.modelId);

    let dayData = byDate.get(msg.date);
    if (!dayData) {
      dayData = {
        tokens: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, reasoning: 0 },
        cost: 0,
        messages: 0,
        sources: new Map(),
      };
      byDate.set(msg.date, dayData);
    }

    const calculatedCost = calculateCost(msg.tokens, pricingMap.get(msg.modelId));
    const msgCost = calculatedCost > 0 ? calculatedCost : msg.cost;

    dayData.tokens.input += msg.tokens.input;
    dayData.tokens.output += msg.tokens.output;
    dayData.tokens.cacheRead += msg.tokens.cacheRead;
    dayData.tokens.cacheWrite += msg.tokens.cacheWrite;
    dayData.tokens.reasoning += msg.tokens.reasoning;
    dayData.cost += msgCost;
    dayData.messages++;

    // Source contribution
    const sourceKey = `${msg.source}:${msg.modelId}`;
    let sourceData = dayData.sources.get(sourceKey);
    if (!sourceData) {
      sourceData = {
        modelId: msg.modelId,
        providerId: msg.providerId,
        tokens: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, reasoning: 0 },
        cost: 0,
        messages: 0,
      };
      dayData.sources.set(sourceKey, sourceData);
    }
    sourceData.tokens.input += msg.tokens.input;
    sourceData.tokens.output += msg.tokens.output;
    sourceData.tokens.cacheRead += msg.tokens.cacheRead;
    sourceData.tokens.cacheWrite += msg.tokens.cacheWrite;
    sourceData.tokens.reasoning += msg.tokens.reasoning;
    sourceData.cost += msgCost;
    sourceData.messages++;

    const msgTokens =
      msg.tokens.input +
      msg.tokens.output +
      msg.tokens.cacheRead +
      msg.tokens.cacheWrite +
      msg.tokens.reasoning;
    totalTokens += msgTokens;
    totalCost += msgCost;
  }

  // Calculate max cost per day for intensity calculation
  for (const dayData of byDate.values()) {
    if (dayData.cost > maxCostInSingleDay) {
      maxCostInSingleDay = dayData.cost;
    }
  }

  // Build contributions array
  const contributions = Array.from(byDate.entries())
    .map(([date, data]) => {
      const dayTokens =
        data.tokens.input +
        data.tokens.output +
        data.tokens.cacheRead +
        data.tokens.cacheWrite +
        data.tokens.reasoning;

      // Calculate intensity (0-4 scale based on cost)
      let intensity: 0 | 1 | 2 | 3 | 4 = 0;
      if (maxCostInSingleDay > 0) {
        const ratio = data.cost / maxCostInSingleDay;
        if (ratio > 0.75) intensity = 4;
        else if (ratio > 0.5) intensity = 3;
        else if (ratio > 0.25) intensity = 2;
        else if (ratio > 0) intensity = 1;
      }

      return {
        date,
        totals: {
          tokens: dayTokens,
          cost: data.cost,
          messages: data.messages,
        },
        intensity,
        tokenBreakdown: data.tokens,
        sources: Array.from(data.sources.entries()).map(([key, src]) => ({
          source: key.split(":")[0] as SourceType,
          modelId: src.modelId,
          providerId: src.providerId,
          tokens: src.tokens,
          cost: src.cost,
          messages: src.messages,
        })),
      };
    })
    .sort((a, b) => a.date.localeCompare(b.date));

  // Determine date range
  const dates = Array.from(byDate.keys()).sort();
  const rangeStart = dates[0] || new Date().toISOString().split("T")[0];
  const rangeEnd = dates[dates.length - 1] || rangeStart;

  // Group by year
  const yearData = new Map<string, { tokens: number; cost: number; start: string; end: string }>();
  for (const [date, data] of byDate.entries()) {
    const year = date.slice(0, 4);
    let yd = yearData.get(year);
    if (!yd) {
      yd = { tokens: 0, cost: 0, start: date, end: date };
      yearData.set(year, yd);
    }
    const dayTokens =
      data.tokens.input +
      data.tokens.output +
      data.tokens.cacheRead +
      data.tokens.cacheWrite +
      data.tokens.reasoning;
    yd.tokens += dayTokens;
    yd.cost += data.cost;
    if (date < yd.start) yd.start = date;
    if (date > yd.end) yd.end = date;
  }

  const years = Array.from(yearData.entries())
    .map(([year, data]) => ({
      year,
      totalTokens: data.tokens,
      totalCost: data.cost,
      range: { start: data.start, end: data.end },
    }))
    .sort((a, b) => b.year.localeCompare(a.year));

  const activeDays = byDate.size;
  const totalDays =
    activeDays > 0
      ? Math.ceil((new Date(rangeEnd).getTime() - new Date(rangeStart).getTime()) / 86400000) + 1
      : 0;

  return {
    meta: {
      generatedAt: new Date().toISOString(),
      version: "1.0.0-ts-fallback",
      dateRange: { start: rangeStart, end: rangeEnd },
    },
    summary: {
      totalTokens,
      totalCost,
      totalDays,
      activeDays,
      averagePerDay: activeDays > 0 ? totalCost / activeDays : 0,
      maxCostInSingleDay,
      sources: Array.from(allSources) as SourceType[],
      models: Array.from(allModels),
    },
    years,
    contributions,
  };
}
