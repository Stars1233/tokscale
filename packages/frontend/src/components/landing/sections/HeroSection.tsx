"use client";

import Image from "next/image";
import styled from "styled-components";
import { useSquircleClip } from "../hooks";

interface HeroSectionProps {
  stargazersCount: number;
}

export function HeroSection({ stargazersCount }: HeroSectionProps) {
  const heroRight = useSquircleClip<HTMLDivElement>(32, 0.6, true);

  const starsText =
    stargazersCount > 0
      ? `${stargazersCount.toLocaleString()} stars`
      : "Star on GitHub";

  return (
    <>
      {/* SVG clip-path def for hero */}
      {heroRight.svgDef && (
        <svg
          width="0"
          height="0"
          style={{ position: "absolute", overflow: "hidden" }}
          aria-hidden="true"
          role="presentation"
        >
          <defs>
            <clipPath id={heroRight.svgDef.id}>
              <path
                d={heroRight.svgDef.path}
                transform={`translate(0, -${heroRight.svgDef.cornerRadius})`}
              />
            </clipPath>
          </defs>
        </svg>
      )}
      <HeroRow>
        <HeroLeft>
          <HeroBgStarfield
            src="/assets/landing/hero-bg-starfield.png"
            alt=""
            width={1076}
            height={536}
          />
          <HeroVideo
            src="/assets/landing/hero-video-transparent.webm"
            autoPlay
            loop
            muted
            playsInline
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
    </>
  );
}

/* ── Hero Styled Components ── */
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
  position: relative;
  flex: 0 0 600px;
  display: flex;
  flex-direction: column;
  align-items: center;
  align-self: stretch;
  justify-content: center;
  background: #000000;
  border-right: 1px solid #10233e;
  overflow: hidden;
  padding-bottom: 64px;

  @media (max-width: 900px) {
    flex: 0 0 auto;
    width: 100%;
    height: 400px;
    border-right: none;
    border-bottom: 1px solid #10233e;
    padding-bottom: 32px;
    padding-top: 60px;
  }
`;

const HeroBgStarfield = styled(Image)`
  position: absolute;
  top: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 1076px;
  height: 536px;
  object-fit: cover;
  pointer-events: none;

  @media (max-width: 900px) {
    width: 100%;
    height: 100%;
  }
`;

const HeroVideo = styled.video`
  position: relative;
  width: 552px;
  max-width: 552px;
  min-width: 552px;
  height: 552px;
  max-height: 552px;
  min-height: 552px;

  object-fit: contain;
  z-index: 1;
  margin-top: 120px;

  @media (max-width: 900px) {
    width: 70%;
    max-width: none;
    min-width: none;
    max-height: none;
    min-height: none;
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
  color: #8292b1;

  @media (max-width: 900px) {
    text-align: center;
  }
`;

const TrustedByLogos = styled.div`
  display: flex;

  @media (max-width: 900px) {
    justify-content: center;
  }
`;
