import { useState } from "react";
import { useKeyboard, useTerminalDimensions } from "@opentui/react";
import { Header } from "./components/Header.js";
import { Footer } from "./components/Footer.js";
import { ModelView } from "./components/ModelView.js";
import { DailyView } from "./components/DailyView.js";
import { StatsView } from "./components/StatsView.js";
import { OverviewView } from "./components/OverviewView.js";
import { useData } from "./hooks/useData.js";
import type { ColorPaletteName } from "./config/themes.js";
import { DEFAULT_PALETTE, getPaletteNames } from "./config/themes.js";
import { loadSettings, saveSettings } from "./config/settings.js";

export type TabType = "overview" | "model" | "daily" | "stats";
export type SortType = "cost" | "name" | "tokens";
export type SourceType = "opencode" | "claude" | "codex" | "cursor" | "gemini";

export interface AppState {
  activeTab: TabType;
  enabledSources: Set<SourceType>;
  sortBy: SortType;
  sortDesc: boolean;
  selectedIndex: number;
  scrollOffset: number;
  colorPalette: ColorPaletteName;
}

export function App() {
  const { width: columns, height: rows } = useTerminalDimensions();
  
  const [state, setState] = useState<AppState>(() => {
    const settings = loadSettings();
    return {
      activeTab: "overview",
      enabledSources: new Set(["opencode", "claude", "codex", "cursor", "gemini"]),
      sortBy: "cost",
      sortDesc: true,
      selectedIndex: 0,
      scrollOffset: 0,
      colorPalette: (settings.colorPalette as ColorPaletteName) || DEFAULT_PALETTE,
    };
  });

  const { data, loading, error, refresh } = useData(state.enabledSources);

  const contentHeight = Math.max(rows - 6, 12);
  const overviewChartHeight = Math.max(5, Math.floor(contentHeight * 0.35));
  const overviewListHeight = Math.max(4, contentHeight - overviewChartHeight - 4);
  const overviewItemsPerPage = Math.max(1, Math.floor(overviewListHeight / 2));

  useKeyboard((key) => {
    if (key.name === "q") {
      process.exit(0);
    }

    if (key.name === "r") {
      refresh();
      return;
    }

    const cycleTab = (current: TabType): TabType => {
      const tabs: TabType[] = ["overview", "model", "daily", "stats"];
      const idx = tabs.indexOf(current);
      return tabs[(idx + 1) % tabs.length];
    };

    if (key.name === "tab" || key.name === "d") {
      setState((s) => ({
        ...s,
        activeTab: cycleTab(s.activeTab),
        selectedIndex: 0,
        scrollOffset: 0,
      }));
      return;
    }

    if (key.name === "c") {
      setState((s) => ({ ...s, sortBy: "cost", sortDesc: true }));
      return;
    }
    if (key.name === "n") {
      setState((s) => ({ ...s, sortBy: "name", sortDesc: false }));
      return;
    }
    if (key.name === "t") {
      setState((s) => ({ ...s, sortBy: "tokens", sortDesc: true }));
      return;
    }

    if (key.name === "p") {
      setState((s) => {
        const palettes = getPaletteNames();
        const currentIdx = palettes.indexOf(s.colorPalette);
        const nextIdx = (currentIdx + 1) % palettes.length;
        const newPalette = palettes[nextIdx];
        saveSettings({ colorPalette: newPalette });
        return { ...s, colorPalette: newPalette };
      });
      return;
    }

    if (key.name === "1") {
      setState((s) => {
        const newSources = new Set(s.enabledSources);
        if (newSources.has("opencode")) newSources.delete("opencode");
        else newSources.add("opencode");
        return { ...s, enabledSources: newSources };
      });
      return;
    }
    if (key.name === "2") {
      setState((s) => {
        const newSources = new Set(s.enabledSources);
        if (newSources.has("claude")) newSources.delete("claude");
        else newSources.add("claude");
        return { ...s, enabledSources: newSources };
      });
      return;
    }
    if (key.name === "3") {
      setState((s) => {
        const newSources = new Set(s.enabledSources);
        if (newSources.has("codex")) newSources.delete("codex");
        else newSources.add("codex");
        return { ...s, enabledSources: newSources };
      });
      return;
    }
    if (key.name === "4") {
      setState((s) => {
        const newSources = new Set(s.enabledSources);
        if (newSources.has("cursor")) newSources.delete("cursor");
        else newSources.add("cursor");
        return { ...s, enabledSources: newSources };
      });
      return;
    }
    if (key.name === "5") {
      setState((s) => {
        const newSources = new Set(s.enabledSources);
        if (newSources.has("gemini")) newSources.delete("gemini");
        else newSources.add("gemini");
        return { ...s, enabledSources: newSources };
      });
      return;
    }

    if (key.name === "up") {
      setState((s) => {
        if (s.activeTab === "overview" && s.scrollOffset > 0) {
          return { ...s, scrollOffset: s.scrollOffset - 1 };
        }
        return { ...s, selectedIndex: Math.max(0, s.selectedIndex - 1) };
      });
      return;
    }
    if (key.name === "down") {
      setState((s) => {
        if (s.activeTab === "overview") {
          const chartH = Math.max(5, Math.floor(contentHeight * 0.35));
          const listH = Math.max(4, contentHeight - chartH - 4);
          const perPage = Math.max(1, Math.floor(listH / 2));
          const maxOffset = Math.max(0, (data?.topModels.length ?? 0) - perPage);
          return { ...s, scrollOffset: Math.min(maxOffset, s.scrollOffset + 1) };
        }
        return { ...s, selectedIndex: s.selectedIndex + 1 };
      });
      return;
    }

    if (key.name === "e" && data) {
      import("node:fs").then((fs) => {
        const exportData = {
          exportedAt: new Date().toISOString(),
          totalCost: data.totalCost,
          modelCount: data.modelCount,
          models: data.modelEntries,
          daily: data.dailyEntries,
          stats: data.stats,
        };
        const filename = `token-usage-export-${new Date().toISOString().split("T")[0]}.json`;
        fs.writeFileSync(filename, JSON.stringify(exportData, null, 2));
      });
      return;
    }
  });

  return (
    <box flexDirection="column" width={columns} height={rows}>
      <Header activeTab={state.activeTab} />
      
      <box flexDirection="column" flexGrow={1} paddingX={1}>
        {loading ? (
          <box justifyContent="center" alignItems="center" flexGrow={1}>
            <text fg="cyan">Loading data...</text>
          </box>
        ) : error ? (
          <box justifyContent="center" alignItems="center" flexGrow={1}>
            <text fg="red">Error: {error}</text>
          </box>
        ) : (
          <>
            {state.activeTab === "overview" && (
              <OverviewView
                data={data}
                selectedIndex={state.selectedIndex}
                scrollOffset={state.scrollOffset}
                height={contentHeight}
                width={columns}
              />
            )}
            {state.activeTab === "model" && (
              <ModelView 
                data={data} 
                sortBy={state.sortBy} 
                sortDesc={state.sortDesc}
                selectedIndex={state.selectedIndex}
                height={contentHeight}
              />
            )}
            {state.activeTab === "daily" && (
              <DailyView 
                data={data} 
                sortBy={state.sortBy}
                sortDesc={state.sortDesc}
                selectedIndex={state.selectedIndex}
                height={contentHeight}
              />
            )}
            {state.activeTab === "stats" && (
              <StatsView 
                data={data} 
                height={contentHeight} 
                colorPalette={state.colorPalette}
              />
            )}
          </>
        )}
      </box>

      <Footer 
        enabledSources={state.enabledSources}
        sortBy={state.sortBy}
        totalCost={data?.totalCost ?? 0}
        modelCount={data?.modelCount ?? 0}
        activeTab={state.activeTab}
        scrollStart={state.scrollOffset}
        scrollEnd={Math.min(state.scrollOffset + overviewItemsPerPage, data?.topModels.length ?? 0)}
        totalItems={data?.topModels.length}
        colorPalette={state.colorPalette}
      />
    </box>
  );
}
