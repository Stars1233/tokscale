"use client";

import type { SquircleBorderDef } from "../hooks";

interface SquircleBorderProps {
  def: SquircleBorderDef | null;
  color?: string;
}

export function SquircleBorder({
  def,
  color = "#10233E",
}: SquircleBorderProps) {
  if (!def) return null;
  const {
    outerClipId, innerClipId, maskId,
    outerPath, innerPath,
    width, height, cornerRadius, borderWidth, bottomOnly,
  } = def;

  return (
    <svg
      style={{
        position: "absolute",
        inset: 0,
        width: "100%",
        height: "100%",
        pointerEvents: "none",
        zIndex: 1,
      }}
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="none"
      aria-hidden="true"
    >
      <defs>
        <clipPath id={outerClipId}>
          <path
            d={outerPath}
            transform={bottomOnly ? `translate(0, -${cornerRadius})` : undefined}
          />
        </clipPath>
        <clipPath id={innerClipId}>
          <path
            d={innerPath}
            transform={
              bottomOnly
                ? `translate(${borderWidth}, ${borderWidth - cornerRadius})`
                : `translate(${borderWidth}, ${borderWidth})`
            }
          />
        </clipPath>
        <mask id={maskId}>
          <rect
            width={width}
            height={height}
            fill="white"
            clipPath={`url(#${outerClipId})`}
          />
          <rect
            width={width - borderWidth * 2}
            height={height - borderWidth * 2}
            x={borderWidth}
            y={borderWidth}
            fill="black"
            clipPath={`url(#${innerClipId})`}
          />
        </mask>
      </defs>
      <rect
        width={width}
        height={height}
        fill={color}
        mask={`url(#${maskId})`}
      />
    </svg>
  );
}
