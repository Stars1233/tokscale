"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import type { ColorPaletteName } from "./themes";
import { DEFAULT_PALETTE } from "./themes";

export interface Settings {
  paletteName: ColorPaletteName;
}

const DEFAULT_SETTINGS: Settings = {
  paletteName: DEFAULT_PALETTE,
};

const STORAGE_KEY = "tokscale-settings";

function getStoredSettings(): Settings {
  if (typeof window === "undefined") return DEFAULT_SETTINGS;

  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return {
        paletteName: parsed.paletteName || DEFAULT_SETTINGS.paletteName,
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
  const [settings, setSettings] = useState<Settings>(() => {
    if (typeof window === "undefined") return DEFAULT_SETTINGS;
    return getStoredSettings();
  });

  const mountedRef = useRef(false);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    applyDarkModeToDocument();
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

  return {
    paletteName: settings.paletteName,
    setPalette,
    mounted,
  };
}
