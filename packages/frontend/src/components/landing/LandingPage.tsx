"use client";

import { useState, useEffect, useMemo, useCallback, useRef } from "react";
import Image from "next/image";
import Link from "next/link";
import styled from "styled-components";
import { getSvgPath } from "figma-squircle";

function useCopy(text: string) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };
  return { copied, copy };
}

function useSquircleClip<T extends HTMLElement = HTMLElement>(
  cornerRadius: number,
  cornerSmoothing: number = 0.6,
  bottomOnly: boolean = false
) {
  const ref = useRef<T | null>(null);
  const clipIdRef = useRef(
    `squircle-clip-${Math.random().toString(36).slice(2, 9)}`
  );
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 });

  const clipResult = useMemo(() => {
    const { width, height } = dimensions;
    if (width === 0 || height === 0)
      return { clipPath: "", svgDef: null };

    if (bottomOnly) {
      const effectiveHeight = height + cornerRadius;
      const path = getSvgPath({
        width,
        height: effectiveHeight,
        cornerRadius,
        cornerSmoothing,
      });
      const clipId = clipIdRef.current;

      return {
        clipPath: `url(#${clipId})`,
        svgDef: { id: clipId, path, width, effectiveHeight, cornerRadius },
      };
    }

    const path = getSvgPath({ width, height, cornerRadius, cornerSmoothing });
    return { clipPath: `path('${path}')`, svgDef: null };
  }, [dimensions, cornerRadius, cornerSmoothing, bottomOnly]);

  useEffect(() => {
    if (!ref.current) return;
    const el = ref.current;

    const update = () => {
      const { width, height } = el.getBoundingClientRect();
      setDimensions({ width: Math.round(width), height: Math.round(height) });
    };

    update();
    const observer = new ResizeObserver(update);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const setRef = useCallback((node: T | null) => {
    ref.current = node;
    if (node) {
      const { width, height } = node.getBoundingClientRect();
      if (width > 0 || height > 0) {
        setDimensions({ width: Math.round(width), height: Math.round(height) });
      }
    }
  }, []);

  return {
    ref: setRef,
    clipPath: clipResult.clipPath,
    svgDef: clipResult.svgDef,
  };
}

interface LandingPageProps {
  stargazersCount?: number;
}

export function LandingPage({ stargazersCount = 0 }: LandingPageProps) {
  const tui = useCopy("bunx tokscale@latest");
  const submit = useCopy("bunx tokscale@latest submit");

  const heroRight = useSquircleClip<HTMLDivElement>(32, 0.6, true);
  const cardsRow = useSquircleClip<HTMLDivElement>(32, 0.6, true);
  const globeSection = useSquircleClip<HTMLDivElement>(32, 0.6, true);

  const starsText =
    stargazersCount > 0
      ? `${stargazersCount.toLocaleString()} stars`
      : "Star on GitHub";

  return (
    <PageWrapper>
      {/* SVG clip-path defs for bottom-only squircles */}
      <svg
        width="0"
        height="0"
        style={{ position: "absolute", overflow: "hidden" }}
        aria-hidden="true"
        role="presentation"
      >
        <defs>
          {[heroRight.svgDef, cardsRow.svgDef, globeSection.svgDef]
            .filter(Boolean)
            .map((def) => (
              <clipPath key={def!.id} id={def!.id}>
                <path
                  d={def!.path}
                  transform={`translate(0, -${def!.cornerRadius})`}
                />
              </clipPath>
            ))}
        </defs>
      </svg>
      <PageInner>
        {/* ── Section 1: Hero ── */}
        <HeroRow>
          <HeroLeft>
            <HeroSVG
              src="/assets/landing/hero-main.svg"
              alt="Tokscale hero illustration"
              width={600}
              height={536}
              priority
            />
          </HeroLeft>

          <HeroRight
            ref={heroRight.ref}
            style={{
              clipPath: heroRight.clipPath || undefined,
            }}
          >
            {/* Top part with BG image */}
            <HeroTopSection>
              <HeroContent>
                <HeroTitle>
                  The Kardashev
                  <br />
                  Scale for AI Devs
                </HeroTitle>

                <HeroButtonsRow>
                  <StarButton
                    href="https://github.com/junhoyeo/tokscale"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    <StarGlow />
                    <StarButtonText>Star us on GitHub</StarButtonText>
                  </StarButton>
                </HeroButtonsRow>
              </HeroContent>

              <StarBadge
                href="https://github.com/junhoyeo/tokscale/stargazers"
                target="_blank"
                rel="noopener noreferrer"
              >
                <Image
                  src="/assets/landing/star-link-icon.svg"
                  alt="Star"
                  width={18}
                  height={18}
                />
                <StarBadgeText>{starsText}</StarBadgeText>
              </StarBadge>
            </HeroTopSection>

            {/* Bottom part: Trusted By */}
            <TrustedBySection>
              <TrustedByLabel>Trusted by professionals at</TrustedByLabel>
              <TrustedByLogos>
                <Image
                  src="/assets/landing/trusted-by-logos.svg"
                  alt="Trusted by companies"
                  width={408}
                  height={73}
                  style={{ width: "100%", maxWidth: 408, height: "auto" }}
                />
              </TrustedByLogos>
            </TrustedBySection>
          </HeroRight>
        </HeroRow>

        {/* ── Section 2: Separator Bar ── */}
        <SeparatorBar>
          <SeparatorIcon>
            <Image
              src="/assets/landing/separator-pattern.svg"
              alt=""
              width={16}
              height={16}
            />
          </SeparatorIcon>
        </SeparatorBar>

        {/* ── Section 3: Quickstart Label ── */}
        <QuickstartLabel>
          <QuickstartText>Quickstart</QuickstartText>
        </QuickstartLabel>

        {/* ── Section 4: Quickstart Cards ── */}
        <QuickstartCardsWrapper>
          <QuickstartCardsRow
              ref={cardsRow.ref}
              style={{
                clipPath: cardsRow.clipPath || undefined,
              }}
            >
            {/* Left Card */}
            <QuickstartCard $position="left">
              <CardPatternOverlay $position="left" />
              <CardScreenshot>
                <Image
                  src="/assets/landing/screenshot-tui-4d3240.png"
                  alt="TUI Screenshot"
                  width={171}
                  height={168}
                  style={{ width: 171.25, height: 168, objectFit: "cover", borderRadius: 8 }}
                />
              </CardScreenshot>
              <CardContent>
                <CardTitle>
                  View your
                  <br />
                  Usage Stats
                </CardTitle>
                <CommandBox>
                  <CommandInputArea>
                    <CommandText>bunx tokscale@latest</CommandText>
                    <GradientAccent />
                  </CommandInputArea>
                  <CopyBtn onClick={tui.copy}>
                    <CopyBtnText>{tui.copied ? "Copied" : "Copy"}</CopyBtnText>
                  </CopyBtn>
                </CommandBox>
              </CardContent>
            </QuickstartCard>

            {/* Right Card */}
            <QuickstartCard $position="right">
              <CardPatternOverlay $position="right" />
              <CardScreenshot>
                <Image
                  src="/assets/landing/screenshot-leaderboard-75a76a.png"
                  alt="Leaderboard Screenshot"
                  width={152}
                  height={180}
                  style={{ width: 152.02, height: 180, objectFit: "cover", borderRadius: 8 }}
                />
              </CardScreenshot>
              <CardContent>
                <CardTitle>
                  Submit DATA
                  <br />
                  to the Global Leaderboard
                </CardTitle>
                <CommandBox>
                  <CommandInputArea>
                    <CommandText>bunx tokscale@latest submit</CommandText>
                    <GradientAccent />
                  </CommandInputArea>
                  <CopyBtn onClick={submit.copy}>
                    <CopyBtnText>{submit.copied ? "Copied" : "Copy"}</CopyBtnText>
                  </CopyBtn>
                </CommandBox>
              </CardContent>
            </QuickstartCard>
          </QuickstartCardsRow>
        </QuickstartCardsWrapper>

        {/* ── Separator before Globe ── */}
        <GlobeSeparatorBar>
          <SeparatorIcon>
            <Image
              src="/assets/landing/separator-pattern.svg"
              alt=""
              width={16}
              height={16}
            />
          </SeparatorIcon>
        </GlobeSeparatorBar>

        {/* ── Section 5: Globe + Largest Group ── */}
        <GlobeSection
          ref={globeSection.ref}
          style={{
            clipPath: globeSection.clipPath || undefined,
          }}
        >
          <GlobeImage>
            <Image
              src="/assets/landing/globe-illustration.svg"
              alt="Globe illustration"
              width={847}
              height={348}
              style={{ width: "100%", maxWidth: 847, height: "auto" }}
              priority
            />
          </GlobeImage>

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
        </GlobeSection>

        {/* ── Section 6: Description + Client Logos + GitHub ── */}
        <DescriptionSection>
          <DescriptionText>
            A high-performance CLI tool
            <br />
            and visualization dashboard
            <br />
            for tracking token usage and costs
            <br />
            across multiple AI coding agents.
          </DescriptionText>

          <ClientLogosContainer>
            <ClientLogosFadeLeft />
            <Image
              src="/assets/landing/client-logos-grid.svg"
              alt="Supported AI coding clients"
              width={965}
              height={100}
              style={{ width: "100%", maxWidth: 965, height: "auto" }}
            />
            <ClientLogosFadeRight />
          </ClientLogosContainer>

          <GitHubBtn
            href="https://github.com/junhoyeo/tokscale"
            target="_blank"
            rel="noopener noreferrer"
          >
            <GitHubBtnText>GitHub</GitHubBtnText>
          </GitHubBtn>
        </DescriptionSection>

        {/* ── Section 7: Follow Section ── */}
        <FollowSection>
          <FollowAvatar>
            <Image
              src="/assets/landing/follow-github-avatar.png"
              alt="@junhoyeo"
              width={288}
              height={288}
              style={{ width: 288, height: 288, borderRadius: 16 }}
            />
          </FollowAvatar>
          <FollowText>
            Follow{" "}
            <FollowLink
              href="https://github.com/junhoyeo"
              target="_blank"
              rel="noopener noreferrer"
            >
              @junhoyeo
            </FollowLink>{" "}
            on GitHub
          </FollowText>
        </FollowSection>

        {/* ── Section 8: Footer ── */}
        <LandingFooter>
          <FooterInner>
            <FooterCopyright>© 2026 STROKE</FooterCopyright>
          </FooterInner>
        </LandingFooter>
      </PageInner>
    </PageWrapper>
  );
}

/* ═══════════════════════════════════════════════════════════════
   Styled Components
   ═══════════════════════════════════════════════════════════════ */

const PageWrapper = styled.div`
  min-height: 100vh;
  background: #000000;
  display: flex;
  flex-direction: column;
  align-items: center;
`;

const PageInner = styled.div`
  width: 1200px;
  display: flex;
  flex-direction: column;
  align-items: center;

  @media (max-width: 1200px) {
    width: 100%;
  }
`;

/* ── Hero ── */
const HeroRow = styled.div`
  width: 100%;
  display: flex;
  flex-direction: row;
  height: 536px;
  border: 1px solid #10233e;
  overflow: hidden;

  @media (max-width: 900px) {
    flex-direction: column;
    height: auto;
  }
`;

const HeroLeft = styled.div`
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  align-self: stretch;
  justify-content: center;
  border-right: 1px solid #10233e;
  overflow: hidden;
  padding-bottom: 64px;

  @media (max-width: 900px) {
    border-right: none;
    border-bottom: 1px solid #10233e;
    padding-bottom: 32px;
    padding-top: 60px;
  }
`;

const HeroSVG = styled(Image)`
  width: auto;
  height: 100%;
  max-height: 536px;
  object-fit: contain;

  @media (max-width: 900px) {
    width: 100%;
    height: auto;
    max-height: 320px;
  }
`;

const HeroRight = styled.div`
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-self: stretch;
  overflow: hidden;
`;

const HeroTopSection = styled.div`
  position: relative;
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  gap: 17px;
  padding: 97px 0px 33px;
  background-image: url("/assets/landing/hero-trusted-bg.png");
  background-size: cover;
  background-position: center;
  border-bottom: 1px solid #10233e;

  @media (max-width: 900px) {
    padding: 60px 0px 24px;
  }
`;

const HeroContent = styled.div`
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 33px 40px 0px;

  @media (max-width: 900px) {
    padding: 20px 24px 0px;
  }
`;

const HeroTitle = styled.h1`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 48px;
  line-height: 0.94em;
  letter-spacing: -0.05em;
  color: #ffffff;

  @media (max-width: 900px) {
    font-size: 36px;
  }

  @media (max-width: 480px) {
    font-size: 28px;
  }
`;

const HeroButtonsRow = styled.div`
  display: flex;
  flex-direction: row;
  gap: 20px;
`;

const StarButton = styled.a`
  position: relative;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 6px;
  width: 198px;
  min-width: 198px;
  height: 48px;
  padding: 0 28px;
  background: #000000;
  border-radius: 16px;
  border: none;
  box-shadow: 0px 4px 48.3px 0px rgba(0, 115, 255, 0.14);
  text-decoration: none;
  overflow: hidden;
  transition: opacity 0.2s;
  flex-shrink: 0;

  &::before {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: 16px;
    padding: 1px;
    background: linear-gradient(207deg, rgba(70, 107, 159, 1) 0%, rgba(0, 115, 255, 1) 100%);
    -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
    -webkit-mask-composite: xor;
    mask-composite: exclude;
    pointer-events: none;
  }

  &:hover {
    opacity: 0.9;
  }

  @media (max-width: 480px) {
    width: 170px;
    height: 44px;
    padding: 0 20px;
  }
`;

const StarGlow = styled.div`
  position: absolute;
  left: -36px;
  top: 16px;
  width: 89px;
  height: 70px;
  border-radius: 50%;
  background: #0073ff;
  opacity: 0.54;
  filter: blur(39.2px);
  pointer-events: none;
`;

const StarButtonText = styled.span`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 800;
  font-size: 18px;
  line-height: 1.33em;
  letter-spacing: -0.0174em;
  text-align: center;
  color: #ffffff;
  white-space: nowrap;
  z-index: 1;

  @media (max-width: 480px) {
    font-size: 16px;
  }
`;

const StarBadge = styled.a`
  position: absolute;
  left: 136.38px;
  top: 310px;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 3.5px;
  padding: 6px 8px;
  background: rgba(0, 115, 255, 0.08);
  border: 1px solid rgba(0, 115, 255, 0.26);
  backdrop-filter: blur(4px);
  border-radius: 12px;
  text-decoration: none;
  transition: opacity 0.2s;

  &:hover {
    opacity: 0.8;
  }

  @media (max-width: 900px) {
    left: 24px;
    top: auto;
    bottom: 12px;
  }
`;

const StarBadgeText = styled.span`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 16px;
  line-height: 1em;
  letter-spacing: -0.0114em;
  text-align: center;
  color: #87f0f2;
`;

const TrustedBySection = styled.div`
  display: flex;
  flex-direction: column;
  gap: 28px;
  padding: 28px 32px 36px;
  background: #01070f;

  @media (max-width: 900px) {
    padding: 20px 20px 28px;
  }
`;

const TrustedByLabel = styled.p`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 16px;
  line-height: 1.25em;
  text-transform: uppercase;
  text-align: center;
  color: #8292b1;
`;

const TrustedByLogos = styled.div`
  display: flex;
  justify-content: center;
`;

/* ── Separator Bar ── */
const SeparatorBar = styled.div`
  width: 100%;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px 32px;
  box-sizing: content-box;
  background-image: url("/assets/landing/separator-bar-fill.png");
  background-size: 24px 24px;
  background-repeat: repeat;
  border-left: 1px solid #10233e;
  border-right: 1px solid #10233e;
  border-bottom: 1px solid #10233e;
`;

const SeparatorIcon = styled.div`
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
`;

/* ── Quickstart Label ── */
const QuickstartLabel = styled.div`
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px 32px;
  background: #0073ff;
  border-left: 1px solid #10233e;
  border-right: 1px solid #10233e;
`;

const QuickstartText = styled.span`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 20px;
  line-height: 1em;
  text-transform: uppercase;
  text-align: center;
  color: #ffffff;
`;

/* ── Quickstart Cards ── */
const QuickstartCardsWrapper = styled.div`
  width: 100%;
  padding-bottom: 64px;
`;

const QuickstartCardsRow = styled.div`
  width: 100%;
  display: flex;
  flex-direction: row;
  background: #01070f;
  border: 1px solid #10233e;
  overflow: hidden;

  @media (max-width: 768px) {
    flex-direction: column;
  }
`;

const QuickstartCard = styled.div<{ $position: "left" | "right" }>`
  position: relative;
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  padding: ${({ $position }) =>
    $position === "left" ? "32px" : "21px 32px 32px"};
  min-height: ${({ $position }) =>
    $position === "left" ? "320px" : "320px"};
  ${({ $position }) =>
    $position === "left" ? "border-right: 1px solid #10233e;" : ""}

  @media (max-width: 768px) {
    ${({ $position }) =>
      $position === "left"
        ? "border-right: none; border-bottom: 1px solid #10233e;"
        : ""}
  }
`;

const CardPatternOverlay = styled.div<{ $position: "left" | "right" }>`
  position: absolute;
  left: 0;
  top: ${({ $position }) => ($position === "left" ? "142px" : "121px")};
  width: 100%;
  max-width: 600px;
  height: 20px;
  background-image: url("/assets/landing/separator-bar-fill.png");
  background-size: 20px 20px;
  background-repeat: repeat;
  padding: 16.67px 26.67px;
  pointer-events: none;
`;

const CardScreenshot = styled.div`
  position: absolute;
  top: 32px;
  right: 32px;
`;

const CardContent = styled.div`
  display: flex;
  flex-direction: column;
  align-self: stretch;
  gap: 20px;
  margin-top: auto;
`;

const CardTitle = styled.h3`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 20px;
  line-height: 1em;
  text-transform: uppercase;
  color: #ffffff;
`;

const CommandBox = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  align-self: stretch;
  gap: 6px;
  padding: 8px;
  background: #010a15;
  border: 1px solid #10233e;
  border-radius: 12px;
`;

const CommandInputArea = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  flex: 1;
  gap: 10px;
  padding: 0 12px;
  background: #111b2c;
  border-radius: 8px;
  height: 36px;
  overflow: hidden;
`;

const CommandText = styled.span`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 18px;
  line-height: 0.94em;
  letter-spacing: -0.05em;
  text-align: center;
  color: #ffffff;
  white-space: nowrap;

  @media (max-width: 480px) {
    font-size: 14px;
  }
`;

const GradientAccent = styled.div`
  flex-shrink: 0;
  width: 25px;
  height: 36px;
  background: linear-gradient(
    270deg,
    rgba(26, 27, 28, 0) 0%,
    rgba(1, 127, 255, 0.14) 50%,
    rgba(26, 27, 28, 0) 100%
  );
`;

const CopyBtn = styled.button`
  display: flex;
  justify-content: center;
  align-items: center;
  width: 86px;
  height: 36px;
  background: #0073ff;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  flex-shrink: 0;
  transition: opacity 0.15s;

  &:hover {
    opacity: 0.9;
  }
  &:active {
    transform: scale(0.97);
  }
`;

const CopyBtnText = styled.span`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 18px;
  line-height: 0.94em;
  letter-spacing: -0.05em;
  text-align: center;
  color: #ffffff;
`;

const GlobeSeparatorBar = styled.div`
  width: 100%;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px 32px;
  box-sizing: content-box;
  background-image: url("/assets/landing/separator-bar-fill.png");
  background-size: 24px 24px;
  background-repeat: repeat;
  border-top: 1px solid #10233e;
  border-left: 1px solid #10233e;
  border-right: 1px solid #10233e;
  border-bottom: none;
`;

const GlobeSection = styled.div`
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  background: #010a15;
  border: 1px solid #10233e;
  overflow: hidden;
`;

const GlobeImage = styled.div`
  display: flex;
  justify-content: center;
  width: 100%;
`;

const GlobeContentStack = styled.div`
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
  flex-direction: column;
  gap: 16px;
  padding: 0 32px;
  background: #020f1e;

  @media (max-width: 768px) {
    min-height: 80px;
  }
`;

/* ── Description Section ── */
const DescriptionSection = styled.div`
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 48px;
  padding: 60px 32px 100px;

  @media (max-width: 768px) {
    padding: 40px 20px 60px;
    gap: 32px;
  }
`;

const DescriptionText = styled.p`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 40px;
  line-height: 1.2em;
  letter-spacing: -0.03em;
  text-align: center;
  color: #b6c0d4;

  @media (max-width: 768px) {
    font-size: 28px;
  }

  @media (max-width: 480px) {
    font-size: 22px;
  }
`;

const ClientLogosContainer = styled.div`
  position: relative;
  width: 100%;
  max-width: 965px;
  display: flex;
  justify-content: center;
  overflow: hidden;
`;

const ClientLogosFadeLeft = styled.div`
  position: absolute;
  left: 0;
  top: 0;
  width: 324px;
  height: 100%;
  background: linear-gradient(90deg, rgba(1, 10, 21, 1) 0%, rgba(1, 10, 21, 0) 100%);
  z-index: 1;
  pointer-events: none;

  @media (max-width: 768px) {
    width: 120px;
  }
`;

const ClientLogosFadeRight = styled.div`
  position: absolute;
  right: 0;
  top: 0;
  width: 325px;
  height: 100%;
  background: linear-gradient(270deg, rgba(1, 10, 21, 1) 0%, rgba(1, 10, 21, 0) 100%);
  z-index: 1;
  pointer-events: none;

  @media (max-width: 768px) {
    width: 120px;
  }
`;

const GitHubBtn = styled.a`
  display: inline-flex;
  justify-content: center;
  align-items: center;
  gap: 4px;
  padding: 9px 28px;
  background: #ffffff;
  border-radius: 32px;
  text-decoration: none;
  transition: opacity 0.15s;

  &:hover {
    opacity: 0.9;
  }
`;

const GitHubBtnText = styled.span`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 23px;
  line-height: 1.2em;
  color: #000000;
`;

/* ── Follow Section ── */
const FollowSection = styled.div`
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  background: #01070f;
`;

const FollowAvatar = styled.div`
  width: 288px;
  height: 288px;
  flex-shrink: 0;
  overflow: hidden;
  border-radius: 16px;

  @media (max-width: 480px) {
    width: 200px;
    height: 200px;
  }
`;

const FollowText = styled.p`
  font-family: var(--font-figtree), "Figtree", sans-serif;
  font-weight: 700;
  font-size: 40px;
  line-height: 1.2em;
  letter-spacing: -0.03em;
  text-align: center;
  color: #b6c0d4;

  @media (max-width: 768px) {
    font-size: 28px;
  }

  @media (max-width: 480px) {
    font-size: 22px;
  }
`;

const FollowLink = styled.a`
  color: #b6c0d4;
  text-decoration: underline;
  text-underline-offset: 4px;
  transition: color 0.15s;

  &:hover {
    color: #ffffff;
  }
`;

/* ── Footer ── */
const LandingFooter = styled.div`
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 48px;
  padding: 0 0 100px;

  @media (max-width: 768px) {
    padding: 0 0 60px;
  }
`;

const FooterInner = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  padding-top: 16px;
  border-top: 0.625px solid rgba(255, 255, 255, 0.1);
`;

const FooterCopyright = styled.p`
  font-family: "Wanted Sans", system-ui, -apple-system, sans-serif;
  font-weight: 600;
  font-size: 16px;
  line-height: 1.5em;
  letter-spacing: -0.0195em;
  text-transform: uppercase;
  color: #99a1af;
`;
