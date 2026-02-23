"use client";

import styled from "styled-components";
import {
  HeroSection,
  QuickstartSection,
  WorldwideSection,
  DescriptionSection,
  FollowSection,
  FooterSection,
} from "./sections";

interface LandingPageProps {
  stargazersCount?: number;
}

export function LandingPage({ stargazersCount = 0 }: LandingPageProps) {
  return (
    <PageWrapper>
      <PageInner>
        <HeroSection stargazersCount={stargazersCount} />
        <QuickstartSection />
        <WorldwideSection />
        <DescriptionSection />
        <FollowSection />
        <FooterSection />
      </PageInner>
    </PageWrapper>
  );
}

const PageWrapper = styled.div`
  min-height: 100vh;
  background: #000000;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0 16px;
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
