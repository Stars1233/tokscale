"use client";

import { useState, useEffect, useMemo } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { Navigation } from "@/components/layout/Navigation";
import { Footer } from "@/components/layout/Footer";
import { GraphContainer } from "@/components/GraphContainer";
import { ProfileSkeleton } from "@/components/Skeleton";
import type { TokenContributionData, DailyContribution, SourceType } from "@/lib/types";

interface ProfileData {
  user: {
    id: string;
    username: string;
    displayName: string | null;
    avatarUrl: string | null;
    createdAt: string;
    rank: number | null;
  };
  stats: {
    totalTokens: number;
    totalCost: number;
    inputTokens: number;
    outputTokens: number;
    cacheReadTokens: number;
    cacheCreationTokens: number;
    submissionCount: number;
    activeDays: number;
  };
  dateRange: {
    start: string | null;
    end: string | null;
  };
  sources: string[];
  models: string[];
  contributions: DailyContribution[];
}

function formatNumber(num: number): string {
  if (num >= 1_000_000_000) return `${(num / 1_000_000_000).toFixed(1)}B`;
  if (num >= 1_000_000) return `${(num / 1_000_000).toFixed(1)}M`;
  if (num >= 1_000) return `${(num / 1_000).toFixed(1)}K`;
  return num.toLocaleString();
}

function formatCurrency(amount: number): string {
  return `$${amount.toFixed(2)}`;
}

export default function ProfilePage() {
  const params = useParams();
  const username = params.username as string;
  const [data, setData] = useState<ProfileData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setIsLoading(true);
    setError(null);
    fetch(`/api/users/${username}`)
      .then((res) => {
        if (!res.ok) throw new Error("User not found");
        return res.json();
      })
      .then((result) => {
        setData(result);
        setIsLoading(false);
      })
      .catch((err) => {
        setError(err.message);
        setIsLoading(false);
      });
  }, [username]);

  // Convert profile data to TokenContributionData format for GraphContainer
  const graphData: TokenContributionData | null = useMemo(() => {
    if (!data || data.contributions.length === 0) return null;

    const contributions = data.contributions;
    const totalCost = data.stats.totalCost;
    const totalTokens = data.stats.totalTokens;
    const maxCost = Math.max(...contributions.map((c) => c.totals.cost), 0);

    // Group by year
    const yearMap = new Map<string, { totalTokens: number; totalCost: number; start: string; end: string }>();
    for (const day of contributions) {
      const year = day.date.split("-")[0];
      const existing = yearMap.get(year);
      if (existing) {
        existing.totalTokens += day.totals.tokens;
        existing.totalCost += day.totals.cost;
        if (day.date < existing.start) existing.start = day.date;
        if (day.date > existing.end) existing.end = day.date;
      } else {
        yearMap.set(year, {
          totalTokens: day.totals.tokens,
          totalCost: day.totals.cost,
          start: day.date,
          end: day.date,
        });
      }
    }

    const years = Array.from(yearMap.entries())
      .sort((a, b) => a[0].localeCompare(b[0]))
      .map(([year, stats]) => ({
        year,
        totalTokens: stats.totalTokens,
        totalCost: stats.totalCost,
        range: { start: stats.start, end: stats.end },
      }));

    return {
      meta: {
        generatedAt: new Date().toISOString(),
        version: "1.0.0",
        dateRange: {
          start: data.dateRange.start || contributions[0]?.date || "",
          end: data.dateRange.end || contributions[contributions.length - 1]?.date || "",
        },
      },
      summary: {
        totalTokens,
        totalCost,
        totalDays: contributions.length,
        activeDays: data.stats.activeDays,
        averagePerDay: data.stats.activeDays > 0 ? totalCost / data.stats.activeDays : 0,
        maxCostInSingleDay: maxCost,
        sources: data.sources as SourceType[],
        models: data.models,
      },
      years,
      contributions: contributions as DailyContribution[],
    };
  }, [data]);

  if (isLoading) {
    return (
      <div className="min-h-screen flex flex-col bg-gray-50 dark:bg-gray-950">
        <Navigation />
        <main className="flex-1 max-w-7xl mx-auto px-4 sm:px-6 py-6 sm:py-10 w-full">
          <ProfileSkeleton />
        </main>
        <Footer />
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="min-h-screen flex flex-col bg-gray-50 dark:bg-gray-950">
        <Navigation />
        <main className="flex-1 max-w-7xl mx-auto px-6 py-10 w-full">
          <div className="text-center py-20">
            <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
              User Not Found
            </h1>
            <p className="text-gray-500 dark:text-gray-400 mb-6">
              The user @{username} doesn't exist or hasn't submitted any data yet.
            </p>
            <Link
              href="/"
              className="inline-flex items-center px-4 py-2 bg-green-600 text-white font-medium rounded-lg hover:bg-green-700 transition-colors"
            >
              Back to Leaderboard
            </Link>
          </div>
        </main>
        <Footer />
      </div>
    );
  }

  return (
    <div className="min-h-screen flex flex-col bg-gray-50 dark:bg-gray-950 transition-colors duration-300">
      <Navigation />

      <main className="flex-1 max-w-7xl mx-auto px-4 sm:px-6 py-6 sm:py-10 w-full">
        {/* User Header */}
        <div className="mb-6 sm:mb-8">
          <div className="flex items-start gap-4 sm:gap-6 mb-6">
            {data.user.avatarUrl ? (
              <img
                src={data.user.avatarUrl}
                alt={data.user.username}
                className="w-16 h-16 sm:w-24 sm:h-24 rounded-xl sm:rounded-2xl ring-2 sm:ring-4 ring-gray-200 dark:ring-gray-700"
              />
            ) : (
              <div className="w-16 h-16 sm:w-24 sm:h-24 rounded-xl sm:rounded-2xl bg-gradient-to-br from-green-400 to-emerald-600 flex items-center justify-center text-white text-xl sm:text-3xl font-bold">
                {data.user.username[0].toUpperCase()}
              </div>
            )}
            <div className="flex-1 min-w-0">
              <div className="flex flex-wrap items-center gap-2 sm:gap-3 mb-1">
                <h1 className="text-xl sm:text-2xl font-bold text-gray-900 dark:text-white truncate">
                  {data.user.displayName || data.user.username}
                </h1>
                {data.user.rank && (
                  <span className="px-2 py-0.5 sm:py-1 text-xs sm:text-sm font-medium bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 rounded-lg shrink-0">
                    #{data.user.rank}
                  </span>
                )}
              </div>
              <p className="text-sm sm:text-base text-gray-500 dark:text-gray-400 mb-2 sm:mb-3">
                @{data.user.username}
              </p>
              <div className="flex flex-wrap gap-1.5 sm:gap-2">
                {data.sources.map((source) => (
                  <span
                    key={source}
                    className="px-2 py-0.5 sm:py-1 text-xs font-medium bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-300 rounded-lg"
                  >
                    {source}
                  </span>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 sm:gap-4 mb-6 sm:mb-8">
          <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-3 sm:p-4">
            <p className="text-xs sm:text-sm text-gray-500 dark:text-gray-400">Total Tokens</p>
            <p className="text-lg sm:text-2xl font-bold text-gray-900 dark:text-white">
              {formatNumber(data.stats.totalTokens)}
            </p>
          </div>
          <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-3 sm:p-4">
            <p className="text-xs sm:text-sm text-gray-500 dark:text-gray-400">Total Cost</p>
            <p className="text-lg sm:text-2xl font-bold text-green-600 dark:text-green-400">
              {formatCurrency(data.stats.totalCost)}
            </p>
          </div>
          <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-3 sm:p-4">
            <p className="text-xs sm:text-sm text-gray-500 dark:text-gray-400">Active Days</p>
            <p className="text-lg sm:text-2xl font-bold text-gray-900 dark:text-white">
              {data.stats.activeDays}
            </p>
          </div>
          <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-3 sm:p-4">
            <p className="text-xs sm:text-sm text-gray-500 dark:text-gray-400">Submissions</p>
            <p className="text-lg sm:text-2xl font-bold text-gray-900 dark:text-white">
              {data.stats.submissionCount}
            </p>
          </div>
        </div>

        {/* Token Breakdown */}
        <div className="bg-white dark:bg-gray-900 rounded-2xl border border-gray-200 dark:border-gray-800 p-4 sm:p-6 mb-6 sm:mb-8">
          <h2 className="text-base sm:text-lg font-semibold text-gray-900 dark:text-white mb-3 sm:mb-4">
            Token Breakdown
          </h2>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3 sm:gap-4">
            <div>
              <p className="text-xs sm:text-sm text-gray-500 dark:text-gray-400">Input</p>
              <p className="text-base sm:text-lg font-semibold text-gray-900 dark:text-white">
                {formatNumber(data.stats.inputTokens)}
              </p>
            </div>
            <div>
              <p className="text-xs sm:text-sm text-gray-500 dark:text-gray-400">Output</p>
              <p className="text-base sm:text-lg font-semibold text-gray-900 dark:text-white">
                {formatNumber(data.stats.outputTokens)}
              </p>
            </div>
            <div>
              <p className="text-xs sm:text-sm text-gray-500 dark:text-gray-400">Cache Read</p>
              <p className="text-base sm:text-lg font-semibold text-gray-900 dark:text-white">
                {formatNumber(data.stats.cacheReadTokens)}
              </p>
            </div>
            <div>
              <p className="text-xs sm:text-sm text-gray-500 dark:text-gray-400">Cache Write</p>
              <p className="text-base sm:text-lg font-semibold text-gray-900 dark:text-white">
                {formatNumber(data.stats.cacheCreationTokens)}
              </p>
            </div>
          </div>
        </div>

        {/* Contribution Graph */}
        {graphData ? (
          <div className="mb-6 sm:mb-8">
            <h2 className="text-base sm:text-lg font-semibold text-gray-900 dark:text-white mb-3 sm:mb-4">
              Activity
            </h2>
            <div className="overflow-x-auto -mx-4 sm:mx-0 px-4 sm:px-0">
              <div className="min-w-[600px] sm:min-w-0">
                <GraphContainer data={graphData} />
              </div>
            </div>
          </div>
        ) : (
          <div className="bg-white dark:bg-gray-900 rounded-2xl border border-gray-200 dark:border-gray-800 p-6 sm:p-8 text-center">
            <p className="text-sm sm:text-base text-gray-500 dark:text-gray-400">
              No contribution data available yet.
            </p>
          </div>
        )}
      </main>

      <Footer />
    </div>
  );
}
