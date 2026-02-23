"use client";
import { useState, useEffect, useCallback } from "react";
import Link from "next/link";
import Image from "next/image";
import styled from "styled-components";
import { useSquircleClip } from "../hooks";
import { SquircleBorder } from "../components";

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

function formatCompactNumber(n: number): string {
  if (n >= 1_000_000_000)
    return (n / 1_000_000_000).toFixed(1).replace(/\.0$/, "") + "B";
  if (n >= 1_000_000)
    return (n / 1_000_000).toFixed(1).replace(/\.0$/, "") + "M";
  if (n >= 1_000) return (n / 1_000).toFixed(1).replace(/\.0$/, "") + "K";
  return n.toString();
}

function formatCompactCurrency(n: number): string {
  if (n >= 1_000)
    return "$" + (n / 1_000).toFixed(1).replace(/\.0$/, "") + "K";
  if (n >= 1) return "$" + n.toFixed(2);
  return "$" + n.toFixed(4);
}
export function WorldwideSection() {
  const worldwideSection = useSquircleClip<HTMLDivElement>(32, 0.6, true, 1);
  const [activeTab, setActiveTab] = useState<"tokens" | "cost">("tokens");
  const [users, setUsers] = useState<LeaderboardUser[]>([]);

  const fetchLeaderboard = useCallback(async (sortBy: "tokens" | "cost") => {
    try {
      const res = await fetch(
        `/api/leaderboard?period=all&page=1&limit=3&sortBy=${sortBy}`
      );
      if (!res.ok) return;
      const data = await res.json();
      setUsers(data.users ?? []);
    } catch {
      setUsers([]);
    }
  }, []);

  useEffect(() => {
    fetchLeaderboard(activeTab);
  }, [activeTab, fetchLeaderboard]);
  return (
    <>
      {/* SVG clip-path def for globe section */}
      {worldwideSection.svgDef && (
        <svg
          width="0"
          height="0"
          style={{ position: "absolute", overflow: "hidden" }}
          aria-hidden="true"
          role="presentation"
        >
          <defs>
            <clipPath id={worldwideSection.svgDef.id}>
              <path
                d={worldwideSection.svgDef.path}
                transform={`translate(0, -${worldwideSection.svgDef.cornerRadius})`}
              />
            </clipPath>
          </defs>
        </svg>
      )}
      {/* Separator before Globe */}
      <GlobeSeparatorBar />
      <GlobeSectionWrapper
        ref={worldwideSection.ref}
        style={{
          clipPath: worldwideSection.clipPath || undefined,
        }}
      >
        <SquircleBorder def={worldwideSection.borderDef} />
        <GlobeImageWrapper>
          <GlobeBackground />
          <GlobeFadeTop />
          <GlobeFadeBottom />
          <TrophyVideo
            autoPlay
            loop
            muted
            playsInline
            src="/assets/landing/trophy-cup-transparent.webm"
          />
        </GlobeImageWrapper>
        <GlobeContentStack>
          <GlobeBlueHeader>
            <GlobeHeaderText>
              THE LARGEST GROUP
              <br />
              OF TOKEN CONSUMERS
            </GlobeHeaderText>
          </GlobeBlueHeader>
          <GlobeTwoCol>
            <GlobeLeftCol>
              <GlobeLeftTitle>
                Tracking 800B+
                <br />
                Tokens Worldwide
              </GlobeLeftTitle>
              <LeaderboardBtn href="/leaderboard">
                <LeaderboardBtnText>Leaderboard</LeaderboardBtnText>
              </LeaderboardBtn>
            </GlobeLeftCol>
            <GlobeRightCol>
              <LeaderboardWidget>
                <WidgetHeader>
                  <WidgetTitle>Top Users</WidgetTitle>
                  <TabSwitcher>
                    <Tab
                      $active={activeTab === "tokens"}
                      onClick={() => setActiveTab("tokens")}
                    >
                      Tokens
                    </Tab>
                    <Tab
                      $active={activeTab === "cost"}
                      onClick={() => setActiveTab("cost")}
                    >
                      Cost
                    </Tab>
                  </TabSwitcher>
                </WidgetHeader>
                <UserList>
                  {users.map((user) => (
                    <UserRow key={user.userId} href={`/u/${user.username}`}>
                      <RankBadge data-rank={user.rank}>
                        {user.rank === 1
                          ? "🥇"
                          : user.rank === 2
                            ? "🥈"
                            : "🥉"}
                      </RankBadge>
                      <UserAvatar
                        src={
                          user.avatarUrl ||
                          `https://github.com/${user.username}.png`
                        }
                        alt={user.displayName || user.username}
                        width={32}
                        height={32}
                        unoptimized
                      />
                      <UserInfo>
                        <UserName>
                          {user.displayName || user.username}
                        </UserName>
                        <UserHandle>@{user.username}</UserHandle>
                      </UserInfo>
                      <UserValue>
                        {activeTab === "tokens"
                          ? formatCompactNumber(user.totalTokens)
                          : formatCompactCurrency(user.totalCost)}
                      </UserValue>
                    </UserRow>
                  ))}
                </UserList>
                <ViewMoreLink href="/leaderboard">
                  View Full Leaderboard
                  <ViewMoreArrow>→</ViewMoreArrow>
                </ViewMoreLink>
              </LeaderboardWidget>
            </GlobeRightCol>
          </GlobeTwoCol>
        </GlobeContentStack>
      </GlobeSectionWrapper>
    </>
  );
}

const GlobeSeparatorBar = styled.div`
  width: 100%;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-image: url("/assets/landing/separator-pattern-slash.svg");
  background-size: 24px 24px;
  background-repeat: repeat;
  border-top: 1px solid #10233E;
  border-left: 1px solid #10233E;
  border-right: 1px solid #10233E;
  border-bottom: none;
`;

const GlobeSectionWrapper = styled.div`
  position: relative;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  background: #010a15;
  overflow: hidden;
`;

const GlobeImageWrapper = styled.div`
  position: relative;
  width: 100%;
  height: 348px;
  overflow: hidden;
`;

const GlobeBackground = styled.div`
  position: absolute;
  inset: 0;
  background-color: #010101;
  background-image: url("/assets/landing/worldwide-section-bg.svg");
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;

  @media (max-width: 900px) {
    background-image: url("/assets/landing/worldwide-section-bg@mobile.svg");
  }
`;

const GlobeFadeTop = styled.div`
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 70px;
  background: linear-gradient(180deg, rgba(1, 1, 1, 1) 0%, rgba(1, 1, 1, 0) 100%);
  pointer-events: none;
  z-index: 1;
`;

const GlobeFadeBottom = styled.div`
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 70px;
  background: linear-gradient(0deg, rgba(1, 1, 1, 1) 0%, rgba(1, 1, 1, 0) 100%);
  pointer-events: none;
  z-index: 1;
`;

const GlobeContentStack = styled.div`
  position: relative;
  z-index: 1;
  width: 100%;
  display: flex;
  flex-direction: column;
  margin-top: -24px;
`;

const GlobeBlueHeader = styled.div`
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px 32px;
  background: #0073ff;
  border-left: 1px solid #10233e;
  border-right: 1px solid #10233e;
`;

const GlobeHeaderText = styled.span`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 20px;
  line-height: 1em;
  text-transform: uppercase;
  text-align: center;
  color: #ffffff;
`;

const GlobeTwoCol = styled.div`
  width: 100%;
  display: flex;
  flex-direction: row;
  align-items: stretch;
  background: #01070f;

  @media (max-width: 768px) {
    flex-direction: column;
  }
`;

const GlobeLeftCol = styled.div`
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 32px 32px 56px;
  border-right: 1px solid #10233e;

  @media (max-width: 768px) {
    border-right: none;
    border-bottom: 1px solid #10233e;
  }
`;

const GlobeLeftTitle = styled.h2`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 40px;
  line-height: 1.2em;
  letter-spacing: -0.03em;
  color: #ffffff;

  @media (max-width: 768px) {
    font-size: 32px;
  }

  @media (max-width: 480px) {
    font-size: 26px;
  }
`;

const LeaderboardBtn = styled(Link)`
  display: inline-flex;
  justify-content: center;
  align-items: center;
  gap: 4px;
  padding: 9px 28px;
  background: #ffffff;
  border-radius: 32px;
  text-decoration: none;
  width: fit-content;
  transition: opacity 0.15s;

  &:hover {
    opacity: 0.9;
  }
`;

const LeaderboardBtnText = styled.span`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 23px;
  line-height: 1.2em;
  color: #000000;
`;

const GlobeRightCol = styled.div`
  flex: 1;
  align-self: stretch;
  display: flex;
  align-items: stretch;
  justify-content: flex-start;
  padding: 0 32px;
  background: #020f1e;
  overflow: hidden;
  @media (max-width: 768px) {
    padding: 0 24px;
  }
`;

const LeaderboardWidget = styled.div`
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 24px 0;
`;

const WidgetHeader = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
`;

const WidgetTitle = styled.span`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 18px;
  color: #ffffff;
`;

const TabSwitcher = styled.div`
  display: flex;
  gap: 2px;
  background: #0a1929;
  border-radius: 8px;
  padding: 2px;
`;

const Tab = styled.button<{ $active: boolean }>`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 600;
  font-size: 13px;
  padding: 5px 14px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
  color: ${(p) => (p.$active ? "#ffffff" : "#6b7a90")};
  background: ${(p) => (p.$active ? "#0073FF" : "transparent")};

  &:hover {
    color: #ffffff;
  }
`;

const UserList = styled.div`
  display: flex;
  flex-direction: column;
  gap: 4px;
`;

const UserRow = styled(Link)`
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 10px;
  text-decoration: none;
  transition: background 0.15s;

  &:hover {
    background: rgba(255, 255, 255, 0.04);
  }
`;

const RankBadge = styled.span`
  font-size: 18px;
  width: 28px;
  text-align: center;
  flex-shrink: 0;
`;

const UserAvatar = styled(Image)`
  border-radius: 50%;
  flex-shrink: 0;
`;

const UserInfo = styled.div`
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
`;

const UserName = styled.span`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 600;
  font-size: 14px;
  color: #ffffff;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
`;

const UserHandle = styled.span`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 400;
  font-size: 12px;
  color: #6b7a90;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
`;

const UserValue = styled.span`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 15px;
  color: #0073FF;
  flex-shrink: 0;
  margin-left: auto;
`;

const ViewMoreLink = styled(Link)`
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px;
  border-top: 1px solid #10233e;
  text-decoration: none;
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 600;
  font-size: 14px;
  color: #6b7a90;
  transition: color 0.15s;

  &:hover {
    color: #0073ff;
  }
`;

const ViewMoreArrow = styled.span`
  font-size: 16px;
  transition: transform 0.15s;

  ${ViewMoreLink}:hover & {
    transform: translateX(3px);
  }
`;

const TrophyVideo = styled.video`
  position: absolute;

  width: 396px;
  height: 396px;
  min-width: 396px;
  min-height: 396px;
  max-width: 396px;
  max-height: 396px;

  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  object-fit: contain;
  pointer-events: none;
  z-index: 2;
`;
