import { homedir } from "os";
import { join } from "path";
import { readFileSync, existsSync } from "fs";

const CONFIG_DIR = join(homedir(), ".config", "tokscale");
const CONFIG_FILE = join(CONFIG_DIR, "settings.json");

export interface TokscaleSettings {
  colorPalette: string;
  autoRefreshEnabled?: boolean;
  autoRefreshMs?: number;
  includeUnusedModels?: boolean;
  nativeTimeoutMs?: number;
}

const DEFAULT_SETTINGS: TokscaleSettings = {
  colorPalette: "blue",
  autoRefreshEnabled: false,
  autoRefreshMs: 60000,
  includeUnusedModels: false,
  nativeTimeoutMs: 300000,
};

export function loadSettings(): TokscaleSettings {
  try {
    if (existsSync(CONFIG_FILE)) {
      const raw = JSON.parse(readFileSync(CONFIG_FILE, "utf-8"));
      return { ...DEFAULT_SETTINGS, ...raw };
    }
  } catch {}
  return DEFAULT_SETTINGS;
}
