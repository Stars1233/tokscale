#!/usr/bin/env node
/**
 * Token Tracker CLI
 * Display OpenCode and Claude Code usage with dynamic width tables
 */

import { Command } from "commander";
import pc from "picocolors";
import { PricingFetcher } from "./pricing.js";
import {
  createUsageTable,
  formatUsageRow,
  formatTotalsRow,
  formatNumber,
  formatCurrency,
  formatModelName,
} from "./table.js";
import { readOpenCodeMessages, aggregateOpenCodeByModel } from "./opencode.js";
import { readClaudeCodeSessions } from "./claudecode.js";

interface UsageSummary {
  source: string;
  model: string;
  input: number;
  output: number;
  cacheRead: number;
  cacheWrite: number;
  reasoning: number;
  messageCount: number;
  cost: number;
}

async function main() {
  const program = new Command();

  program
    .name("token-tracker")
    .description("Calculate token prices from OpenCode and Claude Code sessions")
    .version("1.0.0");

  program
    .command("monthly")
    .description("Show monthly usage report")
    .option("--opencode", "Show only OpenCode usage")
    .option("--claudecode", "Show only Claude Code usage")
    .action(async (options) => {
      await showMonthlyReport(options);
    });

  program
    .command("models")
    .description("Show usage breakdown by model")
    .option("--opencode", "Show only OpenCode usage")
    .option("--claudecode", "Show only Claude Code usage")
    .action(async (options) => {
      await showModelReport(options);
    });

  // Default command with options
  program
    .option("--opencode", "Show only OpenCode usage")
    .option("--claudecode", "Show only Claude Code usage")
    .action(async (options) => {
      await showModelReport(options);
    });

  await program.parseAsync();
}

async function showModelReport(options: { opencode?: boolean; claudecode?: boolean }) {
  console.log(pc.cyan("\n  Token Usage Report by Model\n"));

  const fetcher = new PricingFetcher();
  console.log(pc.gray("  Fetching pricing data from LiteLLM..."));
  await fetcher.fetchPricing();

  const summaries: UsageSummary[] = [];

  // OpenCode data
  if (!options.claudecode) {
    const openCodeMessages = readOpenCodeMessages();
    if (openCodeMessages.length > 0) {
      const openCodeData = aggregateOpenCodeByModel(openCodeMessages);

      for (const data of openCodeData.values()) {
        const pricing = fetcher.getModelPricing(data.modelID);
        const cost = pricing
          ? fetcher.calculateCost(
              {
                input: data.input,
                output: data.output,
                reasoning: data.reasoning,
                cacheRead: data.cacheRead,
                cacheWrite: data.cacheWrite,
              },
              pricing
            )
          : 0;

        summaries.push({
          source: "OpenCode",
          model: data.modelID,
          input: data.input,
          output: data.output,
          cacheRead: data.cacheRead,
          cacheWrite: data.cacheWrite,
          reasoning: data.reasoning,
          messageCount: data.messageCount,
          cost,
        });
      }
    }
  }

  // Claude Code data
  if (!options.opencode) {
    const claudeCodeData = readClaudeCodeSessions();
    if (claudeCodeData.length > 0) {
      for (const data of claudeCodeData) {
        const pricing = fetcher.getModelPricing(data.model);
        const cost = pricing
          ? fetcher.calculateCost(
              {
                input: data.input,
                output: data.output,
                reasoning: data.reasoning,
                cacheRead: data.cachedInput,
                cacheWrite: 0,
              },
              pricing
            )
          : 0;

        summaries.push({
          source: "Claude Code",
          model: data.model,
          input: data.input,
          output: data.output,
          cacheRead: data.cachedInput,
          cacheWrite: 0,
          reasoning: data.reasoning,
          messageCount: data.messageCount,
          cost,
        });
      }
    }
  }

  if (summaries.length === 0) {
    console.log(pc.yellow("  No usage data found.\n"));
    return;
  }

  // Sort by cost descending
  summaries.sort((a, b) => b.cost - a.cost);

  // Create table
  const table = createUsageTable("Source/Model");

  let totalInput = 0;
  let totalOutput = 0;
  let totalCacheRead = 0;
  let totalCacheWrite = 0;
  let totalCost = 0;

  for (const summary of summaries) {
    const modelDisplay = `${pc.dim(summary.source)} ${formatModelName(summary.model)}`;
    table.push(
      formatUsageRow(
        modelDisplay,
        [summary.model],
        summary.input,
        summary.output,
        summary.cacheWrite,
        summary.cacheRead,
        summary.cost
      )
    );

    totalInput += summary.input;
    totalOutput += summary.output;
    totalCacheRead += summary.cacheRead;
    totalCacheWrite += summary.cacheWrite;
    totalCost += summary.cost;
  }

  // Add totals row
  table.push(formatTotalsRow(totalInput, totalOutput, totalCacheWrite, totalCacheRead, totalCost));

  console.log(table.toString());

  // Summary stats
  console.log(
    pc.gray(
      `\n  Total: ${formatNumber(summaries.reduce((sum, s) => sum + s.messageCount, 0))} messages, ` +
        `${formatNumber(totalInput + totalOutput + totalCacheRead + totalCacheWrite)} tokens, ` +
        `${pc.green(formatCurrency(totalCost))}\n`
    )
  );
}

async function showMonthlyReport(options: { opencode?: boolean; claudecode?: boolean }) {
  console.log(pc.cyan("\n  Monthly Token Usage Report\n"));

  const fetcher = new PricingFetcher();
  console.log(pc.gray("  Fetching pricing data from LiteLLM..."));
  await fetcher.fetchPricing();

  // Group by month
  const monthlyData = new Map<
    string,
    {
      models: Set<string>;
      input: number;
      output: number;
      cacheRead: number;
      cacheWrite: number;
      cost: number;
    }
  >();

  // OpenCode data
  if (!options.claudecode) {
    const openCodeMessages = readOpenCodeMessages();
    for (const msg of openCodeMessages) {
      if (!msg.tokens || !msg.modelID) continue;

      const date = new Date(msg.time.created);
      const monthKey = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}`;

      let monthly = monthlyData.get(monthKey);
      if (!monthly) {
        monthly = {
          models: new Set(),
          input: 0,
          output: 0,
          cacheRead: 0,
          cacheWrite: 0,
          cost: 0,
        };
        monthlyData.set(monthKey, monthly);
      }

      monthly.models.add(msg.modelID);
      monthly.input += msg.tokens.input;
      monthly.output += msg.tokens.output;
      monthly.cacheRead += msg.tokens.cache.read;
      monthly.cacheWrite += msg.tokens.cache.write;

      const pricing = fetcher.getModelPricing(msg.modelID);
      if (pricing) {
        monthly.cost += fetcher.calculateCost(
          {
            input: msg.tokens.input,
            output: msg.tokens.output,
            reasoning: msg.tokens.reasoning,
            cacheRead: msg.tokens.cache.read,
            cacheWrite: msg.tokens.cache.write,
          },
          pricing
        );
      }
    }
  }

  if (monthlyData.size === 0) {
    console.log(pc.yellow("  No usage data found.\n"));
    return;
  }

  // Sort by month
  const sortedMonths = Array.from(monthlyData.entries()).sort((a, b) => a[0].localeCompare(b[0]));

  const table = createUsageTable("Month");

  let totalInput = 0;
  let totalOutput = 0;
  let totalCacheRead = 0;
  let totalCacheWrite = 0;
  let totalCost = 0;

  for (const [month, data] of sortedMonths) {
    table.push(
      formatUsageRow(
        month,
        Array.from(data.models),
        data.input,
        data.output,
        data.cacheWrite,
        data.cacheRead,
        data.cost
      )
    );

    totalInput += data.input;
    totalOutput += data.output;
    totalCacheRead += data.cacheRead;
    totalCacheWrite += data.cacheWrite;
    totalCost += data.cost;
  }

  table.push(formatTotalsRow(totalInput, totalOutput, totalCacheWrite, totalCacheRead, totalCost));

  console.log(table.toString());
  console.log(pc.gray(`\n  Total Cost: ${pc.green(formatCurrency(totalCost))}\n`));
}

main().catch(console.error);
