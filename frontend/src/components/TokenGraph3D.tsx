"use client";

import { useRef, useEffect, useCallback, useState } from "react";
import type { DailyContribution, Theme, TooltipPosition } from "@/lib/types";
import { getGradeColor } from "@/lib/themes";
import { groupByWeek, hexToNumber } from "@/lib/utils";
import {
  CUBE_SIZE,
  MAX_CUBE_HEIGHT,
  MIN_CUBE_HEIGHT,
  ISO_CANVAS_WIDTH,
  ISO_CANVAS_HEIGHT,
} from "@/lib/constants";

interface TokenGraph3DProps {
  contributions: DailyContribution[];
  theme: Theme;
  year: string;
  maxCost: number;
  onDayHover: (day: DailyContribution | null, position: TooltipPosition | null) => void;
  onDayClick: (day: DailyContribution | null) => void;
}

export function TokenGraph3D({
  contributions,
  theme,
  year,
  maxCost,
  onDayHover,
  onDayClick,
}: TokenGraph3DProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [obeliskLoaded, setObeliskLoaded] = useState(false);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const obeliskRef = useRef<any>(null);
  const weeksData = groupByWeek(contributions, year);

  // Load obelisk.js dynamically (client-side only)
  useEffect(() => {
    async function loadObelisk() {
      try {
        const obeliskModule = await import("obelisk.js");
        obeliskRef.current = obeliskModule.default || obeliskModule;
        setObeliskLoaded(true);
      } catch (err) {
        console.error("Failed to load obelisk.js:", err);
      }
    }
    loadObelisk();
  }, []);

  // Render the 3D graph
  useEffect(() => {
    if (!obeliskLoaded || !obeliskRef.current) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const obelisk = obeliskRef.current;

    // Set canvas size (don't use DPR scaling - obelisk handles its own rendering)
    canvas.width = ISO_CANVAS_WIDTH;
    canvas.height = ISO_CANVAS_HEIGHT;
    canvas.style.width = `${ISO_CANVAS_WIDTH}px`;
    canvas.style.height = `${ISO_CANVAS_HEIGHT}px`;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // Clear canvas with background color
    ctx.fillStyle = theme.background;
    ctx.fillRect(0, 0, ISO_CANVAS_WIDTH, ISO_CANVAS_HEIGHT);

    // Create pixel view with origin point (matching reference)
    const point = new obelisk.Point(130, 90);
    const pixelView = new obelisk.PixelView(canvas, point);

    // Render cubes for each day
    // Iterate FORWARD for proper z-ordering (back to front in isometric)
    // Reference uses GH_OFFSET=14 for spacing calculation
    const GH_OFFSET = 14;
    let transform = GH_OFFSET;

    for (let weekIndex = 0; weekIndex < weeksData.length; weekIndex++) {
      const week = weeksData[weekIndex];
      const x = transform / (GH_OFFSET + 1);
      transform += GH_OFFSET;

      let offsetY = 0;
      for (let dayIndex = 0; dayIndex < 7; dayIndex++) {
        const day = week.days[dayIndex];
        const y = offsetY / GH_OFFSET;
        offsetY += 13;

        // Calculate cube height based on cost
        let cubeHeight = MIN_CUBE_HEIGHT;
        if (day && maxCost > 0) {
          cubeHeight = MIN_CUBE_HEIGHT + Math.floor((MAX_CUBE_HEIGHT / maxCost) * day.totals.cost);
        }

        // Get color based on intensity
        const intensity = day?.intensity ?? 0;
        const colorHex = getGradeColor(theme, intensity);
        const colorNum = hexToNumber(colorHex);

        // Create cube
        const dimension = new obelisk.CubeDimension(CUBE_SIZE, CUBE_SIZE, Math.max(cubeHeight, MIN_CUBE_HEIGHT));
        const color = new obelisk.CubeColor().getByHorizontalColor(colorNum);
        const cube = new obelisk.Cube(dimension, color, false);

        // Position in grid (matching reference calculation)
        const p3d = new obelisk.Point3D(CUBE_SIZE * x, CUBE_SIZE * y, 0);

        // Render
        pixelView.renderObject(cube, p3d);
      }
    }
  }, [obeliskLoaded, contributions, theme, year, maxCost, weeksData]);

  // Hit testing is approximate for 3D - just use position estimate
  const getDayAtPosition = useCallback(
    (clientX: number, clientY: number): { day: DailyContribution | null; position: TooltipPosition } | null => {
      const canvas = canvasRef.current;
      if (!canvas) return null;

      const rect = canvas.getBoundingClientRect();
      const x = clientX - rect.left;
      const y = clientY - rect.top;

      // Approximate isometric to grid conversion
      // This is simplified - full implementation would need proper isometric math
      const isoX = (x - 130) / (CUBE_SIZE * 0.7);
      const isoY = (y - 90) / (CUBE_SIZE * 0.35) - isoX;

      const weekIndex = Math.floor(isoX);
      const dayIndex = Math.floor(isoY);

      if (weekIndex < 0 || weekIndex >= weeksData.length || dayIndex < 0 || dayIndex >= 7) {
        return null;
      }

      const day = weeksData[weekIndex]?.days[dayIndex] ?? null;
      return { day, position: { x: clientX, y: clientY } };
    },
    [weeksData]
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      const result = getDayAtPosition(e.clientX, e.clientY);
      if (result) {
        onDayHover(result.day, result.position);
      } else {
        onDayHover(null, null);
      }
    },
    [getDayAtPosition, onDayHover]
  );

  const handleMouseLeave = useCallback(() => {
    onDayHover(null, null);
  }, [onDayHover]);

  const handleClick = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      const result = getDayAtPosition(e.clientX, e.clientY);
      if (result && result.day) {
        onDayClick(result.day);
      }
    },
    [getDayAtPosition, onDayClick]
  );

  if (!obeliskLoaded) {
    return (
      <div
        className="flex items-center justify-center"
        style={{
          width: ISO_CANVAS_WIDTH,
          height: ISO_CANVAS_HEIGHT,
          backgroundColor: theme.background,
        }}
      >
        <div className="animate-pulse" style={{ color: theme.meta }}>
          Loading 3D view...
        </div>
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <canvas
        ref={canvasRef}
        onMouseMove={handleMouseMove}
        onMouseLeave={handleMouseLeave}
        onClick={handleClick}
        className="cursor-pointer"
        style={{
          minWidth: ISO_CANVAS_WIDTH,
        }}
      />
    </div>
  );
}
