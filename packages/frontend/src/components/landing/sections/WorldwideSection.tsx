"use client";

import Link from "next/link";
import styled from "styled-components";
import { useSquircleClip } from "../hooks";
import { SquircleBorder } from "../components";

export function WorldwideSection() {
  const worldwideSection = useSquircleClip<HTMLDivElement>(32, 0.6, true, 1);

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

      {/* Globe + Largest Group */}
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
            <GlobeRightCol />
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
  align-items: center;
  justify-content: center;
  padding: 0 32px;
  background: #020f1e;
  overflow: hidden;

  @media (max-width: 768px) {
    min-height: 80px;
  }
`;

const TrophyVideo = styled.video`
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 33%;
  height: auto;
  object-fit: contain;
  pointer-events: none;
  z-index: 2;
`;
