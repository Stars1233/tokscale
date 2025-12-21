import { createCanvas } from "@napi-rs/canvas";
import * as fs from "node:fs";
import * as path from "node:path";
import {
  parseLocalSourcesAsync,
  finalizeReportAsync,
  finalizeGraphAsync,
  type ParsedMessages,
} from "./native.js";
import { PricingFetcher } from "./pricing.js";
import { syncCursorCache, loadCursorCredentials } from "./cursor.js";
import type { SourceType } from "./graph-types.js";

interface WrappedData {
  year: string;
  firstDay: string;
  totalDays: number;
  activeDays: number;
  totalTokens: number;
  totalCost: number;
  currentStreak: number;
  longestStreak: number;
  topModels: Array<{ name: string; cost: number; tokens: number }>;
  topClients: Array<{ name: string; cost: number; tokens: number }>;
  contributions: Array<{ date: string; level: 0 | 1 | 2 | 3 | 4 }>;
  totalMessages: number;
}

export interface WrappedOptions {
  output?: string;
  year?: string;
  sources?: SourceType[];
}

const IMAGE_WIDTH = 1200;
const IMAGE_HEIGHT = 1200;
const PADDING = 60;

const COLORS = {
  background: "#1a1a1a",
  textPrimary: "#ffffff",
  textSecondary: "#888888",
  textMuted: "#555555",
  accent: "#ff6b35",
  grade0: "#2d2d2d",
  grade1: "#ff6b3544",
  grade2: "#ff6b3588",
  grade3: "#ff6b35cc",
  grade4: "#ff6b35",
};

const SOURCE_DISPLAY_NAMES: Record<string, string> = {
  opencode: "OpenCode",
  claude: "Claude Code",
  codex: "Codex CLI",
  gemini: "Gemini CLI",
  cursor: "Cursor IDE",
};

async function loadWrappedData(options: WrappedOptions): Promise<WrappedData> {
  const year = options.year || new Date().getFullYear().toString();
  const sources = options.sources || ["opencode", "claude", "codex", "gemini", "cursor"];
  const localSources = sources.filter(s => s !== "cursor") as ("opencode" | "claude" | "codex" | "gemini")[];
  const includeCursor = sources.includes("cursor");

  const since = `${year}-01-01`;
  const until = `${year}-12-31`;

  const pricingFetcher = new PricingFetcher();
  
  const phase1Results = await Promise.allSettled([
    pricingFetcher.fetchPricing(),
    includeCursor && loadCursorCredentials() ? syncCursorCache() : Promise.resolve({ synced: false, rows: 0 }),
    localSources.length > 0
      ? parseLocalSourcesAsync({ sources: localSources, since, until, year })
      : Promise.resolve({ messages: [], opencodeCount: 0, claudeCount: 0, codexCount: 0, geminiCount: 0, processingTimeMs: 0 } as ParsedMessages),
  ]);

  const cursorSync = phase1Results[1].status === "fulfilled" 
    ? phase1Results[1].value 
    : { synced: false, rows: 0 };
  const localMessages = phase1Results[2].status === "fulfilled" 
    ? phase1Results[2].value 
    : null;

  const emptyMessages: ParsedMessages = {
    messages: [],
    opencodeCount: 0,
    claudeCount: 0,
    codexCount: 0,
    geminiCount: 0,
    processingTimeMs: 0,
  };

  const [reportResult, graphResult] = await Promise.allSettled([
    finalizeReportAsync({
      localMessages: localMessages || emptyMessages,
      pricing: pricingFetcher.toPricingEntries(),
      includeCursor: includeCursor && cursorSync.synced,
      since,
      until,
      year,
    }),
    finalizeGraphAsync({
      localMessages: localMessages || emptyMessages,
      pricing: pricingFetcher.toPricingEntries(),
      includeCursor: includeCursor && cursorSync.synced,
      since,
      until,
      year,
    }),
  ]);

  if (reportResult.status === "rejected") {
    throw new Error(`Failed to generate report: ${reportResult.reason}`);
  }
  if (graphResult.status === "rejected") {
    throw new Error(`Failed to generate graph: ${graphResult.reason}`);
  }

  const report = reportResult.value;
  const graph = graphResult.value;

  const modelMap = new Map<string, { cost: number; tokens: number }>();
  for (const entry of report.entries) {
    const existing = modelMap.get(entry.model) || { cost: 0, tokens: 0 };
    modelMap.set(entry.model, {
      cost: existing.cost + entry.cost,
      tokens: existing.tokens + entry.input + entry.output + entry.cacheRead + entry.cacheWrite,
    });
  }
  const topModels = Array.from(modelMap.entries())
    .map(([name, data]) => ({ name, ...data }))
    .sort((a, b) => b.cost - a.cost)
    .slice(0, 3);

  const clientMap = new Map<string, { cost: number; tokens: number }>();
  for (const entry of report.entries) {
    const displayName = SOURCE_DISPLAY_NAMES[entry.source] || entry.source;
    const existing = clientMap.get(displayName) || { cost: 0, tokens: 0 };
    clientMap.set(displayName, {
      cost: existing.cost + entry.cost,
      tokens: existing.tokens + entry.input + entry.output + entry.cacheRead + entry.cacheWrite,
    });
  }
  const topClients = Array.from(clientMap.entries())
    .map(([name, data]) => ({ name, ...data }))
    .sort((a, b) => b.cost - a.cost)
    .slice(0, 3);

  const maxCost = Math.max(...graph.contributions.map(c => c.totals.cost), 1);
  const contributions = graph.contributions.map(c => ({
    date: c.date,
    level: calculateIntensity(c.totals.cost, maxCost),
  }));

  const sortedDates = contributions.map(c => c.date).filter(d => d.startsWith(year)).sort();
  const { currentStreak, longestStreak } = calculateStreaks(sortedDates);

  const firstDay = sortedDates.length > 0 ? sortedDates[0] : `${year}-01-01`;

  return {
    year,
    firstDay,
    totalDays: graph.summary.totalDays,
    activeDays: graph.summary.activeDays,
    totalTokens: graph.summary.totalTokens,
    totalCost: graph.summary.totalCost,
    currentStreak,
    longestStreak,
    topModels,
    topClients,
    contributions,
    totalMessages: report.totalMessages,
  };
}

function calculateIntensity(cost: number, maxCost: number): 0 | 1 | 2 | 3 | 4 {
  if (cost === 0 || maxCost === 0) return 0;
  const ratio = cost / maxCost;
  if (ratio >= 0.75) return 4;
  if (ratio >= 0.5) return 3;
  if (ratio >= 0.25) return 2;
  return 1;
}

function calculateStreaks(sortedDates: string[]): { currentStreak: number; longestStreak: number } {
  if (sortedDates.length === 0) return { currentStreak: 0, longestStreak: 0 };

  const todayStr = new Date().toISOString().split("T")[0];
  let currentStreak = 0;
  let longestStreak = 0;
  let streak = 1;

  for (let i = sortedDates.length - 1; i >= 0; i--) {
    if (i === sortedDates.length - 1) {
      const daysDiff = dateDiffDays(sortedDates[i], todayStr);
      if (daysDiff <= 1) {
        currentStreak = 1;
      } else {
        break;
      }
    } else {
      const daysDiff = dateDiffDays(sortedDates[i], sortedDates[i + 1]);
      if (daysDiff === 1) {
        currentStreak++;
      } else {
        break;
      }
    }
  }

  for (let i = 1; i < sortedDates.length; i++) {
    const daysDiff = dateDiffDays(sortedDates[i - 1], sortedDates[i]);
    if (daysDiff === 1) {
      streak++;
    } else {
      longestStreak = Math.max(longestStreak, streak);
      streak = 1;
    }
  }
  longestStreak = Math.max(longestStreak, streak);

  return { currentStreak, longestStreak };
}

function dateDiffDays(date1: string, date2: string): number {
  const d1 = new Date(date1 + "T00:00:00Z");
  const d2 = new Date(date2 + "T00:00:00Z");
  return Math.abs(Math.round((d2.getTime() - d1.getTime()) / (1000 * 60 * 60 * 24)));
}

function formatTokens(tokens: number): string {
  if (tokens >= 1_000_000_000) return `${(tokens / 1_000_000_000).toFixed(2)}B`;
  if (tokens >= 1_000_000) return `${(tokens / 1_000_000).toFixed(2)}M`;
  if (tokens >= 1_000) return `${(tokens / 1_000).toFixed(1)}K`;
  return tokens.toString();
}

function formatCost(cost: number): string {
  if (cost >= 1000) return `$${(cost / 1000).toFixed(2)}K`;
  return `$${cost.toFixed(2)}`;
}

const MODEL_DISPLAY_NAMES: Record<string, string> = {
  "claude-sonnet-4-20250514": "Claude Sonnet 4",
  "claude-3-5-sonnet-20241022": "Claude 3.5 Sonnet",
  "claude-3-5-sonnet-20240620": "Claude 3.5 Sonnet",
  "claude-3-opus-20240229": "Claude 3 Opus",
  "claude-3-haiku-20240307": "Claude 3 Haiku",
  "gpt-4o": "GPT-4o",
  "gpt-4o-mini": "GPT-4o Mini",
  "gpt-4-turbo": "GPT-4 Turbo",
  "o1": "o1",
  "o1-mini": "o1 Mini",
  "o1-preview": "o1 Preview",
  "o3-mini": "o3 Mini",
  "gemini-2.5-pro": "Gemini 2.5 Pro",
  "gemini-2.5-flash": "Gemini 2.5 Flash",
  "gemini-2.0-flash": "Gemini 2.0 Flash",
  "gemini-1.5-pro": "Gemini 1.5 Pro",
  "gemini-1.5-flash": "Gemini 1.5 Flash",
  "grok-3": "Grok 3",
  "grok-3-mini": "Grok 3 Mini",
};

function formatModelName(model: string): string {
  if (MODEL_DISPLAY_NAMES[model]) return MODEL_DISPLAY_NAMES[model];
  
  const suffixMatch = model.match(/[-_](high|medium|low)$/i);
  const suffix = suffixMatch ? ` ${suffixMatch[1].charAt(0).toUpperCase()}${suffixMatch[1].slice(1).toLowerCase()}` : "";
  
  const cleaned = model
    .replace(/-20\d{6,8}(-\d+)?$/, "")
    .replace(/-\d{8}$/, "")
    .replace(/:[-\w]+$/, "")
    .replace(/[-_](high|medium|low)$/i, "")
    .replace(/[-_]thinking$/i, "");

  if (/claude[-_]?opus[-_]?4[-_.]?5/i.test(cleaned)) return `Claude Opus 4.5${suffix}`;
  if (/claude[-_]?4[-_]?opus/i.test(cleaned)) return `Claude 4 Opus${suffix}`;
  if (/claude[-_]?opus[-_]?4/i.test(cleaned)) return `Claude Opus 4${suffix}`;
  if (/claude[-_]?sonnet[-_]?4[-_.]?5/i.test(cleaned)) return `Claude Sonnet 4.5${suffix}`;
  if (/claude[-_]?4[-_]?sonnet/i.test(cleaned)) return `Claude 4 Sonnet${suffix}`;
  if (/claude[-_]?sonnet[-_]?4/i.test(cleaned)) return `Claude Sonnet 4${suffix}`;
  if (/claude[-_]?haiku[-_]?4[-_.]?5/i.test(cleaned)) return `Claude Haiku 4.5${suffix}`;
  if (/claude[-_]?4[-_]?haiku/i.test(cleaned)) return `Claude 4 Haiku${suffix}`;
  if (/claude[-_]?haiku[-_]?4/i.test(cleaned)) return `Claude Haiku 4${suffix}`;
  if (/claude[-_]?3[-_.]?7[-_]?sonnet/i.test(cleaned)) return `Claude 3.7 Sonnet${suffix}`;
  if (/claude[-_]?3[-_.]?5[-_]?sonnet/i.test(cleaned)) return `Claude 3.5 Sonnet${suffix}`;
  if (/claude[-_]?3[-_.]?5[-_]?haiku/i.test(cleaned)) return `Claude 3.5 Haiku${suffix}`;
  if (/claude[-_]?3[-_]?opus/i.test(cleaned)) return `Claude 3 Opus${suffix}`;
  if (/claude[-_]?3[-_]?sonnet/i.test(cleaned)) return `Claude 3 Sonnet${suffix}`;
  if (/claude[-_]?3[-_]?haiku/i.test(cleaned)) return `Claude 3 Haiku${suffix}`;
  if (/gpt[-_]?5[-_.]?1/i.test(cleaned)) return `GPT-5.1${suffix}`;
  if (/gpt[-_]?5/i.test(cleaned)) return `GPT-5${suffix}`;
  if (/gpt[-_]?4[-_]?o[-_]?mini/i.test(cleaned)) return `GPT-4o Mini${suffix}`;
  if (/gpt[-_]?4[-_]?o/i.test(cleaned)) return `GPT-4o${suffix}`;
  if (/gpt[-_]?4[-_]?turbo/i.test(cleaned)) return `GPT-4 Turbo${suffix}`;
  if (/gpt[-_]?4/i.test(cleaned)) return `GPT-4${suffix}`;
  if (/^o1[-_]?mini/i.test(cleaned)) return `o1 Mini${suffix}`;
  if (/^o1[-_]?preview/i.test(cleaned)) return `o1 Preview${suffix}`;
  if (/^o3[-_]?mini/i.test(cleaned)) return `o3 Mini${suffix}`;
  if (/^o1$/i.test(cleaned)) return `o1${suffix}`;
  if (/^o3$/i.test(cleaned)) return `o3${suffix}`;
  if (/gemini[-_]?3[-_]?pro/i.test(cleaned)) return `Gemini 3 Pro${suffix}`;
  if (/gemini[-_]?3[-_]?flash/i.test(cleaned)) return `Gemini 3 Flash${suffix}`;
  if (/gemini[-_]?2[-_.]?5[-_]?pro/i.test(cleaned)) return `Gemini 2.5 Pro${suffix}`;
  if (/gemini[-_]?2[-_.]?5[-_]?flash/i.test(cleaned)) return `Gemini 2.5 Flash${suffix}`;
  if (/gemini[-_]?2[-_.]?0[-_]?flash/i.test(cleaned)) return `Gemini 2.0 Flash${suffix}`;
  if (/gemini[-_]?1[-_.]?5[-_]?pro/i.test(cleaned)) return `Gemini 1.5 Pro${suffix}`;
  if (/gemini[-_]?1[-_.]?5[-_]?flash/i.test(cleaned)) return `Gemini 1.5 Flash${suffix}`;
  if (/grok[-_]?3[-_]?mini/i.test(cleaned)) return `Grok Code 3 Mini${suffix}`;
  if (/grok[-_]?3/i.test(cleaned)) return `Grok Code 3${suffix}`;
  if (/grok/i.test(cleaned)) return `Grok Code${suffix}`;
  if (/deepseek[-_]?v3/i.test(cleaned)) return `DeepSeek V3${suffix}`;
  if (/deepseek[-_]?r1/i.test(cleaned)) return `DeepSeek R1${suffix}`;
  if (/deepseek/i.test(cleaned)) return `DeepSeek${suffix}`;

  const baseName = cleaned
    .replace(/^claude[-_]/i, "Claude ")
    .replace(/^gpt[-_]/i, "GPT-")
    .replace(/^gemini[-_]/i, "Gemini ")
    .replace(/^grok[-_]/i, "Grok Code ")
    .split(/[-_]/)
    .filter(Boolean)
    .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join(" ")
    .trim();
  
  return `${baseName}${suffix}`;
}

function drawRoundedRect(
  ctx: ReturnType<ReturnType<typeof createCanvas>["getContext"]>,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number
) {
  ctx.beginPath();
  ctx.moveTo(x + radius, y);
  ctx.lineTo(x + width - radius, y);
  ctx.quadraticCurveTo(x + width, y, x + width, y + radius);
  ctx.lineTo(x + width, y + height - radius);
  ctx.quadraticCurveTo(x + width, y + height, x + width - radius, y + height);
  ctx.lineTo(x + radius, y + height);
  ctx.quadraticCurveTo(x, y + height, x, y + height - radius);
  ctx.lineTo(x, y + radius);
  ctx.quadraticCurveTo(x, y, x + radius, y);
  ctx.closePath();
}

function drawContributionGraph(
  ctx: ReturnType<ReturnType<typeof createCanvas>["getContext"]>,
  data: WrappedData,
  x: number,
  y: number,
  width: number,
  height: number
) {
  const year = parseInt(data.year);
  const startDate = new Date(year, 0, 1);
  while (startDate.getDay() !== 0) {
    startDate.setDate(startDate.getDate() - 1);
  }

  const endDate = new Date(year, 11, 31);
  while (endDate.getDay() !== 6) {
    endDate.setDate(endDate.getDate() + 1);
  }

  const contribMap = new Map(data.contributions.map(c => [c.date, c.level]));

  const totalDays = Math.ceil((endDate.getTime() - startDate.getTime()) / (1000 * 60 * 60 * 24)) + 1;
  const totalWeeks = Math.ceil(totalDays / 7);
  
  const cellSize = Math.min(
    Math.floor((height - 20) / totalWeeks),
    Math.floor((width - 20) / 7),
    16
  );
  const dotRadius = (cellSize - 4) / 2;

  const graphWidth = 7 * cellSize;
  const graphHeight = totalWeeks * cellSize;
  const offsetX = x + (width - graphWidth) / 2;
  const offsetY = y + (height - graphHeight) / 2;

  const currentDate = new Date(startDate);
  let weekIndex = 0;

  const gradeColors = [COLORS.grade0, COLORS.grade1, COLORS.grade2, COLORS.grade3, COLORS.grade4];

  while (currentDate <= endDate) {
    const dayOfWeek = currentDate.getDay();
    const dateStr = currentDate.toISOString().split("T")[0];
    const level = contribMap.get(dateStr) || 0;

    const centerX = offsetX + dayOfWeek * cellSize + cellSize / 2;
    const centerY = offsetY + weekIndex * cellSize + cellSize / 2;

    ctx.beginPath();
    ctx.arc(centerX, centerY, dotRadius, 0, Math.PI * 2);
    ctx.fillStyle = gradeColors[level];
    ctx.fill();

    currentDate.setDate(currentDate.getDate() + 1);
    if (currentDate.getDay() === 0) {
      weekIndex++;
    }
  }
}

function drawStat(
  ctx: ReturnType<ReturnType<typeof createCanvas>["getContext"]>,
  x: number,
  y: number,
  label: string,
  value: string
) {
  ctx.fillStyle = COLORS.textSecondary;
  ctx.font = "18px sans-serif";
  ctx.fillText(label, x, y);
  
  ctx.fillStyle = COLORS.textPrimary;
  ctx.font = "bold 36px sans-serif";
  ctx.fillText(value, x, y + 40);
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr + "T00:00:00");
  return date.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
}

async function generateWrappedImage(data: WrappedData): Promise<Buffer> {
  const canvas = createCanvas(IMAGE_WIDTH, IMAGE_HEIGHT);
  const ctx = canvas.getContext("2d");

  ctx.fillStyle = COLORS.background;
  ctx.fillRect(0, 0, IMAGE_WIDTH, IMAGE_HEIGHT);

  const leftWidth = IMAGE_WIDTH * 0.45;
  const rightWidth = IMAGE_WIDTH * 0.55;
  const rightX = leftWidth;

  let yPos = PADDING;

  ctx.fillStyle = COLORS.textSecondary;
  ctx.font = "24px sans-serif";
  ctx.fillText(`Tracking since ${formatDate(data.firstDay)}`, PADDING, yPos);
  yPos += 60;

  ctx.fillStyle = COLORS.textSecondary;
  ctx.font = "20px sans-serif";
  ctx.fillText("Total Cost", PADDING, yPos);
  yPos += 10;
  
  ctx.fillStyle = COLORS.textPrimary;
  ctx.font = "bold 56px sans-serif";
  ctx.fillText(formatCost(data.totalCost), PADDING, yPos + 50);
  yPos += 100;

  ctx.fillStyle = COLORS.textSecondary;
  ctx.font = "20px sans-serif";
  ctx.fillText("Top Models", PADDING, yPos);
  yPos += 40;

  for (let i = 0; i < data.topModels.length; i++) {
    const model = data.topModels[i];
    ctx.fillStyle = COLORS.textPrimary;
    ctx.font = "bold 32px sans-serif";
    ctx.fillText(`${i + 1}`, PADDING, yPos);
    
    ctx.font = "32px sans-serif";
    ctx.fillText(formatModelName(model.name), PADDING + 40, yPos);
    yPos += 50;
  }
  yPos += 30;

  ctx.fillStyle = COLORS.textSecondary;
  ctx.font = "20px sans-serif";
  ctx.fillText("Top Clients", PADDING, yPos);
  yPos += 40;

  for (let i = 0; i < data.topClients.length; i++) {
    const client = data.topClients[i];
    ctx.fillStyle = COLORS.textPrimary;
    ctx.font = "bold 32px sans-serif";
    ctx.fillText(`${i + 1}`, PADDING, yPos);
    
    ctx.font = "32px sans-serif";
    ctx.fillText(client.name, PADDING + 40, yPos);
    yPos += 50;
  }
  yPos += 40;

  const statsStartY = yPos;
  const statWidth = (leftWidth - PADDING * 2) / 2;

  drawStat(ctx, PADDING, statsStartY, "Messages", data.totalMessages.toLocaleString());
  drawStat(ctx, PADDING + statWidth, statsStartY, "Active Days", `${data.activeDays}`);

  drawStat(ctx, PADDING, statsStartY + 100, "Tokens", formatTokens(data.totalTokens));
  drawStat(ctx, PADDING + statWidth, statsStartY + 100, "Streak", `${data.longestStreak}d`);

  drawContributionGraph(
    ctx,
    data,
    rightX + 20,
    PADDING,
    rightWidth - 40,
    IMAGE_HEIGHT - PADDING * 2 - 100
  );

  const footerY = IMAGE_HEIGHT - PADDING - 30;
  
  ctx.fillStyle = COLORS.accent;
  ctx.font = "bold 28px sans-serif";
  ctx.fillText("◆", PADDING, footerY);
  
  ctx.fillStyle = COLORS.textSecondary;
  ctx.font = "24px sans-serif";
  ctx.fillText(`tokscale.dev/${data.year}`, PADDING + 40, footerY);

  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "bold 120px sans-serif";
  ctx.textAlign = "right";
  ctx.fillText(data.year, IMAGE_WIDTH - PADDING, footerY);
  ctx.textAlign = "left";

  return canvas.toBuffer("image/png");
}

export async function generateWrapped(options: WrappedOptions): Promise<string> {
  const data = await loadWrappedData(options);
  const imageBuffer = await generateWrappedImage(data);
  
  const outputPath = options.output || `tokscale-${data.year}-wrapped.png`;
  const absolutePath = path.resolve(outputPath);
  
  fs.writeFileSync(absolutePath, imageBuffer);

  return absolutePath;
}

export { type WrappedData };
