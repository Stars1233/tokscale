/**
 * Tokscale CLI Submit Command
 * Submits local token usage data to the social platform
 */

import pc from "picocolors";
import { loadCredentials, getApiBaseUrl } from "./credentials.js";
import { parseLocalSourcesAsync, finalizeGraphAsync, finalizeReportAsync, type ParsedMessages } from "./native.js";
import { syncCursorCache, loadCursorCredentials } from "./cursor.js";
import type { TokenContributionData } from "./graph-types.js";
import { formatCurrency } from "./table.js";

interface SubmitOptions {
  opencode?: boolean;
  claude?: boolean;
  codex?: boolean;
  gemini?: boolean;
  cursor?: boolean;
  amp?: boolean;
  droid?: boolean;
  since?: string;
  until?: string;
  year?: string;
  dryRun?: boolean;
}

interface SubmitResponse {
  success: boolean;
  submissionId?: string;
  username?: string;
  metrics?: {
    totalTokens: number;
    totalCost: number;
    dateRange: {
      start: string;
      end: string;
    };
    activeDays: number;
    sources: string[];
  };
  warnings?: string[];
  error?: string;
  details?: string[];
}

type SourceType = "opencode" | "claude" | "codex" | "gemini" | "cursor" | "amp" | "droid";

/**
 * Submit command - sends usage data to the platform
 */
export async function submit(options: SubmitOptions = {}): Promise<void> {
  // Step 1: Check if logged in
  const credentials = loadCredentials();
  if (!credentials) {
    console.log(pc.yellow("\n  Not logged in."));
    console.log(pc.gray("  Run 'tokscale login' first.\n"));
    process.exit(1);
  }

  console.log(pc.cyan("\n  Tokscale - Submit Usage Data\n"));

  console.log(pc.gray("  Scanning local session data..."));

  const hasFilter = options.opencode || options.claude || options.codex || options.gemini || options.cursor || options.amp || options.droid;
  let sources: SourceType[] | undefined;
  let includeCursor = true;
  if (hasFilter) {
    sources = [];
    if (options.opencode) sources.push("opencode");
    if (options.claude) sources.push("claude");
    if (options.codex) sources.push("codex");
    if (options.gemini) sources.push("gemini");
    if (options.cursor) sources.push("cursor");
    if (options.amp) sources.push("amp");
    if (options.droid) sources.push("droid");
    includeCursor = sources.includes("cursor");
  }

  // Filter out cursor from local sources (it's handled separately via sync)
  const localSources = sources?.filter((s): s is Exclude<SourceType, "cursor"> => s !== "cursor");

  let data: TokenContributionData;
  try {
    // Two-phase processing (same as TUI) for consistency:
    // Phase 1: Parse local sources + sync cursor in parallel
    const [localMessages, cursorSync] = await Promise.all([
      parseLocalSourcesAsync({
        sources: localSources,
        since: options.since,
        until: options.until,
        year: options.year,
      }),
      includeCursor && loadCursorCredentials()
        ? syncCursorCache()
        : Promise.resolve({ synced: false, rows: 0 }),
    ]);

    // Phase 2: Finalize with pricing (combines local + cursor)
    // Call both report and graph in parallel to ensure consistent pricing
    const finalizeOptions = {
      localMessages,
      includeCursor: includeCursor && cursorSync.synced,
      since: options.since,
      until: options.until,
      year: options.year,
    };
    const [report, graph] = await Promise.all([
      finalizeReportAsync(finalizeOptions),
      finalizeGraphAsync(finalizeOptions),
    ]);

    // Use graph structure for submission, but report's cost for consistency with TUI
    data = graph;
    data.summary.totalCost = report.totalCost;
  } catch (error) {
    console.error(pc.red(`\n  Error generating data: ${(error as Error).message}\n`));
    process.exit(1);
  }

  // Step 4: Show summary
  console.log(pc.white("  Data to submit:"));
  console.log(pc.gray(`    Date range: ${data.meta.dateRange.start} to ${data.meta.dateRange.end}`));
  console.log(pc.gray(`    Active days: ${data.summary.activeDays}`));
  console.log(pc.gray(`    Total tokens: ${data.summary.totalTokens.toLocaleString()}`));
  console.log(pc.gray(`    Total cost: ${formatCurrency(data.summary.totalCost)}`));
  console.log(pc.gray(`    Sources: ${data.summary.sources.join(", ")}`));
  console.log(pc.gray(`    Models: ${data.summary.models.length} models`));
  console.log();

  if (data.summary.totalTokens === 0) {
    console.log(pc.yellow("  No usage data found to submit.\n"));
    return;
  }

  // Step 5: Dry run check
  if (options.dryRun) {
    console.log(pc.yellow("  Dry run - not submitting data.\n"));
    return;
  }

  // Step 6: Submit to server
  console.log(pc.gray("  Submitting to server..."));

  const baseUrl = getApiBaseUrl();

  try {
    const response = await fetch(`${baseUrl}/api/submit`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${credentials.token}`,
      },
      body: JSON.stringify(data),
    });

    const result: SubmitResponse = await response.json();

    if (!response.ok) {
      console.error(pc.red(`\n  Error: ${result.error || "Submission failed"}`));
      if (result.details) {
        for (const detail of result.details) {
          console.error(pc.gray(`    - ${detail}`));
        }
      }
      console.log();
      process.exit(1);
    }

    // Success!
    console.log(pc.green("\n  Successfully submitted!"));
    console.log();
    console.log(pc.white("  Summary:"));
    console.log(pc.gray(`    Submission ID: ${result.submissionId}`));
    console.log(pc.gray(`    Total tokens: ${result.metrics?.totalTokens?.toLocaleString()}`));
    console.log(pc.gray(`    Total cost: ${formatCurrency(result.metrics?.totalCost || 0)}`));
    console.log(pc.gray(`    Active days: ${result.metrics?.activeDays}`));
    console.log();
    console.log(pc.cyan(`  View your profile: ${baseUrl}/u/${credentials.username}`));
    console.log();

    if (result.warnings && result.warnings.length > 0) {
      console.log(pc.yellow("  Warnings:"));
      for (const warning of result.warnings) {
        console.log(pc.gray(`    - ${warning}`));
      }
      console.log();
    }
  } catch (error) {
    console.error(pc.red(`\n  Error: Failed to connect to server.`));
    console.error(pc.gray(`  ${(error as Error).message}\n`));
    process.exit(1);
  }
}
