"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { Pagination, Avatar } from "@primer/react";
import { TabBar } from "@/components/TabBar";
import { Navigation } from "@/components/layout/Navigation";
import { Footer } from "@/components/layout/Footer";
import { LeaderboardSkeleton } from "@/components/Skeleton";
import { BlackholeHero } from "@/components/BlackholeHero";
import { formatCurrency, formatNumber } from "@/lib/utils";

type Period = "all" | "month" | "week";

interface LeaderboardUser {
  rank: number;
  userId: string;
  username: string;
  displayName: string | null;
  avatarUrl: string | null;
  totalTokens: number;
  totalCost: number;
  submissionCount: number;
  lastSubmission: string;
}

interface LeaderboardData {
  users: LeaderboardUser[];
  pagination: {
    page: number;
    limit: number;
    totalUsers: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
  stats: {
    totalTokens: number;
    totalCost: number;
    totalSubmissions: number;
    uniqueUsers: number;
  };
  period: Period;
}

export default function LeaderboardPage() {
  const [data, setData] = useState<LeaderboardData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [period, setPeriod] = useState<Period>("all");
  const [page, setPage] = useState(1);

  useEffect(() => {
    fetch(`/api/leaderboard?period=${period}&page=${page}&limit=50`)
      .then((res) => res.json())
      .then((result) => {
        setData(result);
        setIsLoading(false);
      })
      .catch(() => {
        setIsLoading(false);
      });
  }, [period, page]);

  return (
    <div className="min-h-screen flex flex-col" style={{ backgroundColor: "var(--color-bg-default)" }}>
      <Navigation />

      <main className="flex-1 max-w-7xl mx-auto px-6 pb-10 w-full">
        <BlackholeHero />

        <div className="mb-10">
          <h1 className="text-3xl font-bold mb-2" style={{ color: "var(--color-fg-default)" }}>
            Leaderboard
          </h1>
          <p className="mb-6" style={{ color: "var(--color-fg-muted)" }}>
            See who&apos;s using the most AI tokens
          </p>

          <div className="grid grid-cols-2 md:flex gap-3">
          <div
              className="flex-1 rounded-xl border p-3 sm:p-4"
              style={{ backgroundColor: "var(--color-bg-default)", borderColor: "var(--color-border-default)" }}
            >
              <p className="text-xs sm:text-sm" style={{ color: "var(--color-fg-muted)" }}>
                Users
              </p>
              <p className="text-xl sm:text-2xl font-bold" style={{ color: "var(--color-fg-default)" }}>
                {data ? data.stats.uniqueUsers : "-"}
              </p>
            </div>
            <div
              className="flex-1 rounded-xl border p-3 sm:p-4"
              style={{ backgroundColor: "var(--color-bg-default)", borderColor: "var(--color-border-default)" }}
            >
              <p className="text-xs sm:text-sm" style={{ color: "var(--color-fg-muted)" }}>
                Total Tokens
              </p>
              <p
                className="text-xl sm:text-2xl font-bold"
                style={{ color: "var(--color-primary)", textDecoration: "none" }}
                title={data ? data.stats.totalTokens.toLocaleString() : undefined}
              >
                {data ? formatNumber(data.stats.totalTokens) : "-"}
              </p>
            </div>
            <div
              className="flex-1 rounded-xl border p-3 sm:p-4"
              style={{ backgroundColor: "var(--color-bg-default)", borderColor: "var(--color-border-default)" }}
            >
              <p className="text-xs sm:text-sm" style={{ color: "var(--color-fg-muted)" }}>
                Total Cost
              </p>
              <p
                className="text-xl sm:text-2xl font-bold"
                style={{ color: "var(--color-fg-default)", textDecoration: "none" }}
                title={data ? data.stats.totalCost.toLocaleString('en-US', { style: 'currency', currency: 'USD', minimumFractionDigits: 2 }) : undefined}
              >
                {data ? formatCurrency(data.stats.totalCost) : "-"}
              </p>
            </div>
            {/* <div
              className="flex-1 rounded-xl border p-3 sm:p-4"
              style={{ backgroundColor: "var(--color-bg-default)", borderColor: "var(--color-border-default)" }}
            >
              <p className="text-xs sm:text-sm" style={{ color: "var(--color-fg-muted)" }}>
                Submissions
              </p>
              <p className="text-xl sm:text-2xl font-bold" style={{ color: "var(--color-fg-default)" }}>
                {data ? data.stats.totalSubmissions : "-"}
              </p>
            </div> */}
          </div>
        </div>

        <div className="mb-6">
          <TabBar
            tabs={[
              { id: "all" as Period, label: "All Time" },
              { id: "month" as Period, label: "This Month" },
              { id: "week" as Period, label: "This Week" },
            ]}
            activeTab={period}
            onTabChange={(tab) => {
              setPeriod(tab);
              setPage(1);
            }}
          />
        </div>

        {isLoading ? (
          <LeaderboardSkeleton />
        ) : (
          <div
            className="rounded-2xl border overflow-hidden"
            style={{ backgroundColor: "var(--color-bg-default)", borderColor: "var(--color-border-default)" }}
          >
            {!data || data.users.length === 0 ? (
              <div className="p-8 text-center">
                <p className="mb-4" style={{ color: "var(--color-fg-muted)" }}>
                  No submissions yet. Be the first!
                </p>
                <p className="text-sm" style={{ color: "var(--color-fg-subtle)" }}>
                  Run{" "}
                  <code
                    className="px-2 py-1 rounded"
                    style={{ backgroundColor: "var(--color-bg-subtle)" }}
                  >
                    tokscale login && tokscale submit
                  </code>
                </p>
              </div>
            ) : (
              <>
                <div className="overflow-x-auto">
                  <table className="w-full min-w-[500px]">
                    <thead
                      className="border-b"
                      style={{ backgroundColor: "var(--color-bg-elevated)", borderColor: "var(--color-border-default)" }}
                    >
                      <tr>
                        <th
                          className="px-3 sm:px-6 py-3 text-left text-xs font-medium uppercase tracking-wider"
                          style={{ color: "var(--color-fg-muted)" }}
                        >
                          Rank
                        </th>
                        <th
                          className="px-3 sm:px-6 py-3 text-left text-xs font-medium uppercase tracking-wider"
                          style={{ color: "var(--color-fg-muted)" }}
                        >
                          User
                        </th>
                        <th
                          className="px-3 sm:px-6 py-3 text-right text-xs font-medium uppercase tracking-wider"
                          style={{ color: "var(--color-fg-muted)" }}
                        >
                          Cost
                        </th>
                        <th
                          className="px-3 sm:px-6 py-3 text-right text-xs font-medium uppercase tracking-wider"
                          style={{ color: "var(--color-fg-muted)" }}
                        >
                          Tokens
                        </th>
                        <th
                          className="px-3 sm:px-6 py-3 text-right text-xs font-medium uppercase tracking-wider hidden md:table-cell w-24"
                          style={{ color: "var(--color-fg-muted)" }}
                        >
                          Submissions
                        </th>
                      </tr>
                    </thead>
                    <tbody>
                      {data.users.map((user, index) => (
                        <tr
                          key={user.userId}
                          className="transition-colors hover:opacity-80"
                          style={{
                            borderBottom: index < data.users.length - 1 ? "1px solid var(--color-border-default)" : "none",
                          }}
                        >
                          <td className="px-3 sm:px-6 py-3 sm:py-4 whitespace-nowrap">
                            <span
                              className="text-base sm:text-lg font-bold"
                              style={{
                                color:
                                  user.rank === 1
                                    ? "#EAB308"
                                    : user.rank === 2
                                    ? "#9CA3AF"
                                    : user.rank === 3
                                    ? "#D97706"
                                    : "var(--color-fg-muted)",
                              }}
                            >
                              #{user.rank}
                            </span>
                          </td>
                          <td className="px-3 sm:px-6 py-3 sm:py-4 whitespace-nowrap">
                            <Link
                              href={`/u/${user.username}`}
                              className="flex items-center gap-2 sm:gap-3 group"
                            >
                              <Avatar
                                src={user.avatarUrl || `https://github.com/${user.username}.png`}
                                alt={user.username}
                                size={40}
                              />
                              <div className="min-w-0">
                                <p
                                  className="font-medium text-sm sm:text-base truncate max-w-[120px] sm:max-w-none group-hover:opacity-80 transition-opacity"
                                  style={{ color: "var(--color-fg-default)" }}
                                >
                                  {user.displayName || user.username}
                                </p>
                                <p
                                  className="text-xs sm:text-sm truncate max-w-[120px] sm:max-w-none"
                                  style={{ color: "var(--color-fg-muted)" }}
                                >
                                  @{user.username}
                                </p>
                              </div>
                            </Link>
                          </td>
                          <td className="px-3 sm:px-6 py-3 sm:py-4 whitespace-nowrap text-right">
                            <span
                              className="font-medium text-sm sm:text-base"
                              style={{ color: "var(--color-fg-default)", textDecoration: "none" }}
                              title={user.totalCost.toLocaleString('en-US', { style: 'currency', currency: 'USD', minimumFractionDigits: 2 })}
                            >
                              {formatCurrency(user.totalCost)}
                            </span>
                          </td>
                          <td className="px-3 sm:px-6 py-3 sm:py-4 whitespace-nowrap text-right">
                            <span
                              className="font-medium text-sm sm:text-base"
                              style={{ color: "var(--color-primary)", textDecoration: "none" }}
                              title={user.totalTokens.toString()}
                            >
                              {user.totalTokens.toLocaleString()}
                            </span>
                          </td>
                          <td className="px-3 sm:px-6 py-3 sm:py-4 whitespace-nowrap text-right hidden md:table-cell w-24">
                            <span style={{ color: "var(--color-fg-muted)" }}>{user.submissionCount}</span>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>

                {data.pagination.totalPages > 1 && (
                  <div
                    className="px-3 sm:px-6 py-3 sm:py-4 border-t flex flex-col sm:flex-row items-center justify-between gap-3"
                    style={{ borderColor: "var(--color-border-default)" }}
                  >
                    <p className="text-xs sm:text-sm text-center sm:text-left" style={{ color: "var(--color-fg-muted)" }}>
                      Showing {(data.pagination.page - 1) * data.pagination.limit + 1}-
                      {Math.min(data.pagination.page * data.pagination.limit, data.pagination.totalUsers)} of{" "}
                      {data.pagination.totalUsers}
                    </p>
                    <Pagination
                      pageCount={data.pagination.totalPages}
                      currentPage={data.pagination.page}
                      onPageChange={(e, pageNum) => setPage(pageNum)}
                      showPages={{ narrow: false, regular: true, wide: true }}
                    />
                  </div>
                )}
              </>
            )}
          </div>
        )}

        <div
          className="mt-8 p-6 rounded-2xl border"
          style={{ backgroundColor: "var(--color-bg-default)", borderColor: "var(--color-border-default)" }}
        >
          <h2 className="text-lg font-semibold mb-3" style={{ color: "var(--color-fg-default)" }}>
            Join the Leaderboard
          </h2>
          <p className="mb-4" style={{ color: "var(--color-fg-muted)" }}>
            Install Tokscale CLI and submit your usage data:
          </p>
          <div className="space-y-2 font-mono text-sm">
            <div className="p-3 rounded-lg" style={{ backgroundColor: "var(--color-bg-elevated)", color: "var(--color-fg-muted)" }}>
              <span style={{ color: "var(--color-primary)" }}>$</span> bunx tokscale login
            </div>
            <div className="p-3 rounded-lg" style={{ backgroundColor: "var(--color-bg-elevated)", color: "var(--color-fg-muted)" }}>
              <span style={{ color: "var(--color-primary)" }}>$</span> bunx tokscale submit
            </div>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
