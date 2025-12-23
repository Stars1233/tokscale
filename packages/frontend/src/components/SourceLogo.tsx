"use client";

import styled from "styled-components";

/* eslint-disable @next/next/no-img-element */

interface SourceLogoProps {
  sourceId: string;
  height?: number;
  className?: string;
}

const StyledImg = styled.img<{ $height: number }>`
  border-radius: 2px;
  object-fit: contain;
  height: ${props => props.$height}px;
  width: auto;
  min-width: ${props => props.$height}px;
  max-width: ${props => props.$height}px;
  min-height: ${props => props.$height}px;
  max-height: ${props => props.$height}px;
`;

export function SourceLogo({ sourceId, height = 14, className = "" }: SourceLogoProps) {
  const normalizedId = sourceId.toLowerCase();

  const getLogoSrc = (id: string) => {
    switch (id) {
      case "opencode":
        return "/assets/client-opencode.png";
      case "claude":
        return "/assets/client-claude.jpg";
      case "codex":
        return "/assets/client-openai.jpg";
      case "gemini":
        return "/assets/client-gemini.png";
      case "cursor":
        return "/assets/client-cursor.jpg";
      default:
        return null;
    }
  };

  const src = getLogoSrc(normalizedId);

  if (!src) {
    return <span className={className}>{sourceId}</span>;
  }

  return (
    <StyledImg
      src={src}
      alt={sourceId}
      $height={height}
      className={className}
    />
  );
}
