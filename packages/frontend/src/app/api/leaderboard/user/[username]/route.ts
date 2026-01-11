import { NextRequest, NextResponse } from "next/server";
import { db, users, submissions } from "@/lib/db";
import { eq, and, gte, lte, sql } from "drizzle-orm";

type Period = "all" | "month" | "week";

const VALID_PERIODS: Period[] = ["all", "month", "week"];

function getDateFilter(period: Period) {
  const now = new Date();

  if (period === "week") {
    const weekAgo = new Date(now);
    weekAgo.setDate(weekAgo.getDate() - 7);
    return and(
      gte(submissions.createdAt, weekAgo),
      lte(submissions.createdAt, now)
    );
  }
  
  if (period === "month") {
    const monthAgo = new Date(now);
    monthAgo.setMonth(monthAgo.getMonth() - 1);
    return and(
      gte(submissions.createdAt, monthAgo),
      lte(submissions.createdAt, now)
    );
  }
  
  return undefined;
}

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ username: string }> }
) {
  try {
    const { username } = await params;
    
    if (!username || !/^[a-zA-Z0-9-]{1,39}$/.test(username)) {
      return NextResponse.json(
        { error: "Invalid username format" },
        { status: 400 }
      );
    }

    const { searchParams } = new URL(request.url);
    const periodParam = searchParams.get("period") || "all";
    const period: Period = VALID_PERIODS.includes(periodParam as Period)
      ? (periodParam as Period)
      : "all";
    
    const dateFilter = getDateFilter(period);

    const userResult = await db
      .select({ id: users.id, username: users.username, displayName: users.displayName, avatarUrl: users.avatarUrl })
      .from(users)
      .where(eq(users.username, username))
      .limit(1);

    if (userResult.length === 0) {
      return NextResponse.json({ error: "User not found" }, { status: 404 });
    }

    const user = userResult[0];

    const userStatsResult = await db
      .select({
        totalTokens: sql<number>`SUM(${submissions.totalTokens})`.as("total_tokens"),
        totalCost: sql<number>`SUM(CAST(${submissions.totalCost} AS DECIMAL(12,4)))`.as("total_cost"),
        submissionCount: sql<number>`COALESCE(SUM(${submissions.submitCount}), 0)`.as("submission_count"),
        lastSubmission: sql<string>`MAX(${submissions.createdAt})`.as("last_submission"),
      })
      .from(submissions)
      .where(and(eq(submissions.userId, user.id), dateFilter));

    if (!userStatsResult[0] || userStatsResult[0].totalTokens == null) {
      return NextResponse.json({ error: "User has no submissions" }, { status: 404 });
    }

    const userStats = userStatsResult[0];
    const userTotalTokens = Number(userStats.totalTokens);

    const higherRankedResult = await db
      .select({
        count: sql<number>`COUNT(DISTINCT ${submissions.userId})`.as("count"),
      })
      .from(submissions)
      .where(
        and(
          dateFilter,
          sql`${submissions.userId} IN (
            SELECT user_id FROM submissions
            ${dateFilter ? sql`WHERE ${dateFilter}` : sql``}
            GROUP BY user_id
            HAVING SUM(total_tokens) > ${userTotalTokens}
          )`
        )
      );

    const rank = Number(higherRankedResult[0]?.count || 0) + 1;

    return NextResponse.json({
      rank,
      userId: user.id,
      username: user.username,
      displayName: user.displayName,
      avatarUrl: user.avatarUrl,
      totalTokens: userTotalTokens,
      totalCost: userStats.totalCost != null ? Number(userStats.totalCost) : 0,
      submissionCount: userStats.submissionCount != null ? Number(userStats.submissionCount) : 0,
      lastSubmission: userStats.lastSubmission,
    });
  } catch (error) {
    console.error("Error fetching user rank:", error);
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 }
    );
  }
}
