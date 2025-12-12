import type { SourceType, SortType, TabType } from "../App.js";
import type { ColorPaletteName } from "../config/themes.js";
import { getPalette } from "../config/themes.js";

interface FooterProps {
  enabledSources: Set<SourceType>;
  sortBy: SortType;
  totalCost: number;
  modelCount: number;
  activeTab: TabType;
  scrollStart?: number;
  scrollEnd?: number;
  totalItems?: number;
  colorPalette: ColorPaletteName;
}

export function Footer({ 
  enabledSources, 
  sortBy, 
  totalCost, 
  modelCount,
  activeTab,
  scrollStart,
  scrollEnd,
  totalItems,
  colorPalette,
}: FooterProps) {
  const formatCost = (cost: number) => {
    if (cost >= 1000) return `$${(cost / 1000).toFixed(1)}K`;
    return `$${cost.toFixed(2)}`;
  };

  const palette = getPalette(colorPalette);
  const showScrollInfo = activeTab === "overview" && totalItems && scrollStart !== undefined && scrollEnd !== undefined;

  return (
    <box flexDirection="column" paddingX={1}>
      <box justifyContent="space-between">
        <box gap={1}>
          <SourceBadge name="1:OC" enabled={enabledSources.has("opencode")} />
          <SourceBadge name="2:CC" enabled={enabledSources.has("claude")} />
          <SourceBadge name="3:CX" enabled={enabledSources.has("codex")} />
          <SourceBadge name="4:CR" enabled={enabledSources.has("cursor")} />
          <SourceBadge name="5:GM" enabled={enabledSources.has("gemini")} />
          <text dim>|</text>
          <text dim>Sort:</text>
          <text fg="white">{sortBy === "cost" ? "Cost" : sortBy === "name" ? "Name" : "Tokens"}</text>
          {showScrollInfo && (
            <>
              <text dim>|</text>
              <text dim>{`↓ ${scrollStart! + 1}-${scrollEnd} of ${totalItems} models`}</text>
            </>
          )}
        </box>
        <box gap={1}>
          <text dim>Total:</text>
          <text fg="green" bold>{formatCost(totalCost)}</text>
          <text dim>({modelCount})</text>
        </box>
      </box>
      <box>
        <text dim>
          ↑↓ scroll • tab/d view • c/n/t sort • 1-5 filter • p theme ({palette.name}) • r refresh • q quit
        </text>
      </box>
    </box>
  );
}

function SourceBadge({ name, enabled }: { name: string; enabled: boolean }) {
  return (
    <text fg={enabled ? "green" : "gray"}>
      {`[${enabled ? "●" : "○"}${name}]`}
    </text>
  );
}
