"use client";

/* eslint-disable @next/next/no-img-element */

interface SourceLogoProps {
  sourceId: string;
  height?: number;
  className?: string;
}

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
    <img
      src={src}
      alt={sourceId}
      height={height}
      className={`rounded-sm object-contain ${className}`}
      style={{
        height: height,
        width: "auto",
        minWidth: height,
        maxWidth: height,
        minHeight: height,
        maxHeight: height,
      }}
    />
  );
}
