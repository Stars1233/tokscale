"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import type { ColorPaletteName } from "./themes";
import { DEFAULT_PALETTE } from "./themes";

export type LeaderboardSortBy = 'tokens' | 'cost';

export interface Settings {
  paletteName: ColorPaletteName;
  leaderboardSortBy: LeaderboardSortBy;
}

const DEFAULT_SETTINGS: Settings = {
  paletteName: DEFAULT_PALETTE,
  leaderboardSortBy: 'tokens',
};

const STORAGE_KEY = "tokscale-settings";

const VALID_SORT_BY: LeaderboardSortBy[] = ['tokens', 'cost'];

function isValidSortBy(value: unknown): value is LeaderboardSortBy {
  return typeof value === 'string' && VALID_SORT_BY.includes(value as LeaderboardSortBy);
}

function getStoredSettings(): Settings {
  if (typeof window === "undefined") return DEFAULT_SETTINGS;

  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return {
        paletteName: parsed.paletteName || DEFAULT_SETTINGS.paletteName,
        leaderboardSortBy: isValidSortBy(parsed.leaderboardSortBy) 
          ? parsed.leaderboardSortBy 
          : DEFAULT_SETTINGS.leaderboardSortBy,
      };
    }
  } catch {
    // Invalid JSON or localStorage error
  }

  return DEFAULT_SETTINGS;
}

function saveSettings(settings: Settings): void {
  if (typeof window === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch {
    // localStorage might be full or disabled
  }
}

function applyDarkModeToDocument(): void {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  root.classList.remove("light");
  root.classList.add("dark");
}

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const mountedRef = useRef(false);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    applyDarkModeToDocument();
    const stored = getStoredSettings();
    setSettings(stored);
    mountedRef.current = true;
    setMounted(true);
  }, []);

  const setPalette = useCallback((paletteName: ColorPaletteName) => {
    setSettings((prev) => {
      const newSettings = { ...prev, paletteName };
      saveSettings(newSettings);
      return newSettings;
    });
  }, []);

  const setLeaderboardSort = useCallback((sortBy: LeaderboardSortBy) => {
    setSettings((prev) => {
      const newSettings = { ...prev, leaderboardSortBy: sortBy };
      saveSettings(newSettings);
      return newSettings;
    });
  }, []);

  return {
    paletteName: settings.paletteName,
    setPalette,
    leaderboardSortBy: settings.leaderboardSortBy,
    setLeaderboardSort,
    mounted,
  };
}
