"use client";

import Image from "next/image";
import styled from "styled-components";

export function FollowSection() {
  return (
    <FollowSectionWrapper>
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
    </FollowSectionWrapper>
  );
}

/* ── Follow Section Styled Components ── */
const FollowSectionWrapper = styled.div`
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
