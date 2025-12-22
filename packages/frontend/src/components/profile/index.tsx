"use client";

import Image from "next/image";
import { GraphContainer } from "@/components/GraphContainer";
import type { TokenContributionData } from "@/lib/types";
import { formatNumber, formatCurrency } from "@/lib/utils";

export interface ProfileUser {
  username: string;
  displayName: string | null;
  avatarUrl: string | null;
  rank: number | null;
}

export interface ProfileStatsData {
  totalTokens: number;
  totalCost: number;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  activeDays: number;
  submissionCount?: number;
}

export interface ProfileHeaderProps {
  user: ProfileUser;
  stats: ProfileStatsData;
  lastUpdated?: string;
}

export function ProfileHeader({ user, stats, lastUpdated }: ProfileHeaderProps) {
  const avatarUrl = user.avatarUrl || `https://github.com/${user.username}.png`;

  return (
    <div
      className="flex flex-col gap-2 rounded-2xl border p-4 pb-[18px]"
      style={{ backgroundColor: "var(--color-bg-default)", borderColor: "var(--color-border-default)" }}
    >
      <div className="flex flex-col lg:flex-row gap-6 lg:gap-10">
        <div
          className="flex flex-row items-center gap-[19px] rounded-[20px] py-3 pl-3 pr-8 flex-1"
          style={{ backgroundColor: "var(--color-bg-darkest)" }}
        >
          <div
            className="relative w-[100px] h-[100px] rounded-[7px] overflow-hidden border-2 flex-shrink-0"
            style={{ borderColor: "var(--color-border-default)" }}
          >
            <Image
              src={avatarUrl}
              alt={user.username}
              fill
              className="object-cover"
            />
          </div>

          <div className="flex flex-col flex-1 min-w-0 justify-end gap-[6px] py-0 pb-1 h-[100px]">
            {user.rank && (
              <div
                className="w-8 h-8 rounded-lg flex items-center justify-center"
                style={{
                  background: "linear-gradient(135deg, var(--color-bg-darkest) 0%, color-mix(in srgb, var(--color-accent-blue) 20%, var(--color-bg-darkest)) 50%, color-mix(in srgb, var(--color-accent-blue) 35%, var(--color-bg-darkest)) 100%)",
                  border: "1px solid var(--color-border-default)",
                }}
              >
                <span
                  className="text-base font-medium"
                  style={{ color: "var(--color-accent-blue)" }}
                >
                  #{user.rank}
                </span>
              </div>
            )}

            <div className="flex flex-col gap-[6px] flex-1 justify-end min-w-0">
              <h1
                className="text-2xl font-bold truncate leading-[1.2]"
                style={{ color: "var(--color-fg-default)" }}
              >
                {user.displayName || user.username}
              </h1>
              <p
                className="text-sm font-bold leading-none"
                style={{ color: "var(--color-fg-muted)" }}
              >
                @{user.username}
              </p>
            </div>
          </div>
        </div>

        <div className="flex flex-row items-center gap-7 h-[124px] flex-1">
          <div className="flex flex-col gap-[8px] flex-1 min-w-[120px]">
            <span
              className="text-base font-semibold leading-none"
              style={{ color: "var(--color-accent-blue)" }}
            >
              Total Tokens
            </span>
            <span
              className="text-[27px] font-bold leading-none"
              style={{
                background: "linear-gradient(117deg, #169AFF 0%, #9FD4FB 26%, #B9DFF8 52%)",
                WebkitBackgroundClip: "text",
                WebkitTextFillColor: "transparent",
                backgroundClip: "text",
                textDecoration: "none",
              }}
              title={stats.totalTokens.toLocaleString()}
            >
              {formatNumber(stats.totalTokens)}
            </span>
          </div>

          <div className="flex flex-col gap-[8px] flex-1 min-w-[120px]">
            <span
              className="text-base font-semibold leading-none"
              style={{ color: "var(--color-fg-default)" }}
            >
              Total Cost
            </span>
            <span
              className="text-[27px] font-bold leading-none"
              style={{ color: "var(--color-fg-default)", textDecoration: "none" }}
              title={stats.totalCost.toLocaleString('en-US', { style: 'currency', currency: 'USD' })}
            >
              {formatCurrency(stats.totalCost)}
            </span>
          </div>
        </div>
      </div>

      <div className="w-full h-px" style={{ backgroundColor: "var(--color-border-default)" }} />

      <div className="flex flex-col sm:flex-row items-start sm:items-end justify-between gap-3">
        {lastUpdated && (
          <span
            className="text-sm leading-[1.21]"
            style={{ color: "var(--color-fg-muted)" }}
          >
            Last Updated: {new Date(lastUpdated).toLocaleString()}
          </span>
        )}

        <div className="flex flex-row items-center gap-[6px]">
          <button
            aria-label={`Share ${user.displayName || user.username}'s profile`}
            className="flex flex-row items-center justify-center gap-[6px] rounded-full border py-[9px] pl-[10px] pr-[11px] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2"
            style={{ backgroundColor: "var(--color-btn-bg)", borderColor: "var(--color-border-default)" }}
          >
            <Image src="/icons/icon-share.svg" alt="" width={20} height={20} aria-hidden="true" />
            <span
              className="text-sm leading-none"
              style={{ color: "var(--color-fg-default)" }}
            >
              Share
            </span>
          </button>

          <a
            href={`https://github.com/${user.username}`}
            target="_blank"
            rel="noopener noreferrer"
            aria-label={`View ${user.username}'s GitHub profile (opens in new tab)`}
            className="flex flex-row items-center justify-center gap-[6px] rounded-full border py-[9px] pl-[10px] pr-[11px] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2"
            style={{ backgroundColor: "var(--color-btn-bg)", borderColor: "var(--color-border-default)" }}
          >
            <Image src="/icons/icon-github.svg" alt="" width={20} height={20} aria-hidden="true" />
            <span
              className="text-sm leading-none"
              style={{ color: "var(--color-fg-default)" }}
            >
              GitHub
            </span>
          </a>
        </div>
      </div>
    </div>
  );
}

export type ProfileTab = "activity" | "breakdown" | "models";

export interface ProfileTabBarProps {
  activeTab: ProfileTab;
  onTabChange: (tab: ProfileTab) => void;
}

export function ProfileTabBar({ activeTab, onTabChange }: ProfileTabBarProps) {
  const tabs: { id: ProfileTab; label: string }[] = [
    { id: "activity", label: "Activity" },
    { id: "breakdown", label: "Token Breakdown" },
    { id: "models", label: "Models Used" },
  ];

  const handleKeyDown = (e: React.KeyboardEvent, currentIndex: number) => {
    if (e.key === "ArrowRight" || e.key === "ArrowDown") {
      e.preventDefault();
      const nextIndex = (currentIndex + 1) % tabs.length;
      onTabChange(tabs[nextIndex].id);
    } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
      e.preventDefault();
      const prevIndex = (currentIndex - 1 + tabs.length) % tabs.length;
      onTabChange(tabs[prevIndex].id);
    } else if (e.key === "Home") {
      e.preventDefault();
      onTabChange(tabs[0].id);
    } else if (e.key === "End") {
      e.preventDefault();
      onTabChange(tabs[tabs.length - 1].id);
    }
  };

  return (
    <div
      role="tablist"
      aria-label="Profile tabs"
      className="inline-flex flex-row items-center rounded-[25px] border p-[6px]"
      style={{
        width: "fit-content",
        backgroundColor: "var(--color-bg-elevated)",
        borderColor: "var(--color-border-default)",
      }}
    >
      {tabs.map((tab, index) => {
        const isActive = activeTab === tab.id;
        return (
          <button
            key={tab.id}
            role="tab"
            aria-selected={isActive}
            aria-controls={`tabpanel-${tab.id}`}
            tabIndex={isActive ? 0 : -1}
            onClick={() => onTabChange(tab.id)}
            onKeyDown={(e) => handleKeyDown(e, index)}
            className="flex items-center justify-center rounded-[25px] px-5 py-[10px] transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2"
            style={{
              backgroundColor: isActive ? "var(--color-bg-active)" : "transparent",
            }}
          >
            <span
              className="text-lg font-semibold leading-none whitespace-nowrap"
              style={{
                color: isActive ? "var(--color-fg-default)" : "color-mix(in srgb, var(--color-fg-default) 50%, transparent)",
              }}
            >
              {tab.label}
            </span>
          </button>
        );
      })}
    </div>
  );
}

export interface TokenBreakdownProps {
  stats: ProfileStatsData;
}

export function TokenBreakdown({ stats }: TokenBreakdownProps) {
  const { totalTokens, inputTokens, outputTokens, cacheReadTokens, cacheWriteTokens } = stats;

  const tokenTypes = [
    { label: "Input", value: inputTokens, color: "#006edb", percentage: totalTokens > 0 ? (inputTokens / totalTokens) * 100 : 0 },
    { label: "Output", value: outputTokens, color: "#894ceb", percentage: totalTokens > 0 ? (outputTokens / totalTokens) * 100 : 0 },
    { label: "Cache Read", value: cacheReadTokens, color: "#30a147", percentage: totalTokens > 0 ? (cacheReadTokens / totalTokens) * 100 : 0 },
    { label: "Cache Write", value: cacheWriteTokens, color: "#eb670f", percentage: totalTokens > 0 ? (cacheWriteTokens / totalTokens) * 100 : 0 },
  ];

  return (
    <div
      className="rounded-2xl border p-4 sm:p-6"
      style={{ backgroundColor: "var(--color-bg-default)", borderColor: "var(--color-border-default)" }}
    >
      {totalTokens > 0 && (
        <div className="mb-6">
          <div
            className="h-3 rounded-full overflow-hidden flex"
            style={{ backgroundColor: "var(--color-bg-subtle)" }}
          >
            {tokenTypes.map((type) => (
              <div
                key={type.label}
                style={{
                  width: `${type.percentage}%`,
                  backgroundColor: type.color,
                }}
                title={`${type.label}: ${formatNumber(type.value)}`}
              />
            ))}
          </div>
        </div>
      )}

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {tokenTypes.map((type) => (
          <div key={type.label} className="flex items-center gap-3">
            <div className="w-3 h-3 rounded-full flex-shrink-0" style={{ backgroundColor: type.color }} />
            <div className="min-w-0">
              <div className="flex items-center gap-2">
                <p className="text-xs" style={{ color: "var(--color-fg-muted)" }}>{type.label}</p>
                {type.percentage > 0 && (
                  <span className="text-xs" style={{ color: "var(--color-fg-subtle)" }}>
                    {type.percentage.toFixed(1)}%
                  </span>
                )}
              </div>
              <p
                className="text-base sm:text-lg font-semibold"
                style={{ color: "var(--color-fg-default)" }}
              >
                {formatNumber(type.value)}
              </p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

export interface ProfileStatsProps {
  stats: ProfileStatsData;
  currentStreak?: number;
  longestStreak?: number;
  favoriteModel?: string;
}

export function ProfileStats({ stats, currentStreak = 0, longestStreak = 0, favoriteModel }: ProfileStatsProps) {
  const statItems = [
    { label: "Active Days", value: stats.activeDays.toString(), color: "var(--color-primary)" },
    { label: "Current Streak", value: `${currentStreak} days`, color: "var(--color-primary)" },
    { label: "Longest Streak", value: `${longestStreak} days`, color: "var(--color-primary)" },
    { label: "Submits", value: (stats.submissionCount ?? 0).toString(), color: "var(--color-primary)" },
  ];

  return (
    <div
      className="rounded-2xl border p-4 sm:p-6"
      style={{ backgroundColor: "var(--color-bg-default)", borderColor: "var(--color-border-default)" }}
    >
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 sm:gap-6">
        {statItems.map((item) => (
          <div key={item.label} className="flex flex-col gap-1">
            <p className="text-xs sm:text-sm" style={{ color: "var(--color-fg-muted)" }}>{item.label}</p>
            <p
              className="text-lg sm:text-xl font-bold"
              style={{ color: item.color }}
            >
              {item.value}
            </p>
          </div>
        ))}
      </div>

      {favoriteModel && (
        <div className="mt-4 pt-4 border-t" style={{ borderColor: "var(--color-border-default)" }}>
          <div className="flex items-center gap-2">
            <span className="text-sm" style={{ color: "var(--color-fg-muted)" }}>Favorite Model:</span>
            <span
              className="px-2 py-1 rounded-md text-sm font-medium"
              style={{ backgroundColor: "var(--color-bg-subtle)", color: "var(--color-fg-default)" }}
            >
              {favoriteModel}
            </span>
          </div>
        </div>
      )}
    </div>
  );
}

const MODEL_COLORS: Record<string, string> = {
  "claude": "#D97706",
  "sonnet": "#D97706",
  "opus": "#DC2626",
  "haiku": "#059669",
  "gpt": "#10B981",
  "o1": "#6366F1",
  "o3": "#8B5CF6",
  "gemini": "#3B82F6",
  "deepseek": "#06B6D4",
  "codex": "#F59E0B",
};

function getModelColor(modelName: string): string {
  const lowerName = modelName.toLowerCase();
  for (const [key, color] of Object.entries(MODEL_COLORS)) {
    if (lowerName.includes(key)) return color;
  }
  return "#6B7280";
}

export interface ModelUsage {
  model: string;
  tokens: number;
  cost: number;
  percentage: number;
}

export interface ProfileModelsProps {
  models: string[];
  modelUsage?: ModelUsage[];
}

export function ProfileModels({ models, modelUsage }: ProfileModelsProps) {
  const filteredModels = models.filter((m) => m !== "<synthetic>");

  if (filteredModels.length === 0) return null;

  if (modelUsage && modelUsage.length > 0) {
    const sortedUsage = [...modelUsage].sort((a, b) => b.cost - a.cost);

    return (
      <div
        className="rounded-2xl border overflow-hidden"
        style={{ backgroundColor: "var(--color-bg-default)", borderColor: "var(--color-border-default)" }}
      >
        <div
          className="grid grid-cols-[1fr_auto_auto_auto] gap-4 px-4 sm:px-6 py-3 text-xs font-medium uppercase tracking-wider border-b"
          style={{ backgroundColor: "var(--color-bg-elevated)", borderColor: "var(--color-border-default)", color: "var(--color-fg-muted)" }}
        >
          <div>Model</div>
          <div className="text-right w-20 sm:w-24">Tokens</div>
          <div className="text-right w-16 sm:w-20">Cost</div>
          <div className="text-right w-12 sm:w-16">%</div>
        </div>

        <div>
          {sortedUsage.map((usage, index) => (
            <div
              key={usage.model}
              className="grid grid-cols-[1fr_auto_auto_auto] gap-4 px-4 sm:px-6 py-3 items-center"
              style={{
                backgroundColor: index % 2 === 1 ? "var(--color-bg-elevated)" : "transparent",
                borderTop: index > 0 ? "1px solid var(--color-border-default)" : undefined,
              }}
            >
              <div className="flex items-center gap-2 min-w-0">
                <div
                  className="w-2 h-2 rounded-full flex-shrink-0"
                  style={{ backgroundColor: getModelColor(usage.model) }}
                />
                <span
                  className="text-sm font-medium truncate"
                  style={{ color: "var(--color-fg-default)" }}
                >
                  {usage.model}
                </span>
              </div>
              <div className="text-right w-20 sm:w-24">
                <span className="text-sm" style={{ color: "var(--color-fg-default)" }}>
                  {formatNumber(usage.tokens)}
                </span>
              </div>
              <div className="text-right w-16 sm:w-20">
                <span className="text-sm font-medium" style={{ color: "var(--color-primary)" }}>
                  {formatCurrency(usage.cost)}
                </span>
              </div>
              <div className="text-right w-12 sm:w-16">
                <span className="text-sm" style={{ color: "var(--color-fg-muted)" }}>
                  {usage.percentage.toFixed(1)}%
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div
      className="rounded-2xl border p-4 sm:p-6"
      style={{ backgroundColor: "var(--color-bg-default)", borderColor: "var(--color-border-default)" }}
    >
      <div className="flex flex-wrap gap-2">
        {filteredModels.map((model) => (
          <span
            key={model}
            className="flex items-center gap-2 px-3 py-1.5 rounded-full text-sm font-medium"
            style={{ backgroundColor: "var(--color-bg-subtle)", color: "var(--color-fg-default)" }}
          >
            <span
              className="w-2 h-2 rounded-full"
              style={{ backgroundColor: getModelColor(model) }}
            />
            {model}
          </span>
        ))}
      </div>
    </div>
  );
}

export interface ProfileActivityProps {
  data: TokenContributionData;
}

export function ProfileActivity({ data }: ProfileActivityProps) {
  return (
    <div className="overflow-x-auto -mx-4 sm:mx-0 px-4 sm:px-0">
      <div className="min-w-[600px] sm:min-w-0">
        <GraphContainer data={data} />
      </div>
    </div>
  );
}

export function ProfileEmptyActivity() {
  return (
    <div
      className="rounded-2xl border p-6 sm:p-8 text-center"
      style={{ backgroundColor: "var(--color-bg-default)", borderColor: "var(--color-border-default)" }}
    >
      <p className="text-sm sm:text-base" style={{ color: "var(--color-fg-muted)" }}>
        No contribution data available yet.
      </p>
    </div>
  );
}


