import { For, Show, createMemo, createSignal, type Setter } from "solid-js";
import type { TUIData } from "../hooks/useData.js";
import type { ColorPaletteName } from "../config/themes.js";
import type { DailyModelBreakdown } from "../types/index.js";
import { getPalette, getGradeColor } from "../config/themes.js";
import { getModelColor } from "../utils/colors.js";
import { formatTokens, formatCost } from "../utils/format.js";
import { isNarrow } from "../utils/responsive.js";

interface StatsViewProps {
  data: TUIData;
  height: number;
  colorPalette: ColorPaletteName;
  width?: number;
  selectedDate?: string | null;
  onDateSelect?: Setter<string | null>;
}

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const MONTHS_SHORT = ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"];
const DAYS = ["", "Mon", "", "Wed", "", "Fri", ""];
const SOURCE_COLORS: Record<string, string> = {
  opencode: "#22c55e",
  claude: "#f97316",
  codex: "#3b82f6",
  cursor: "#a855f7",
  gemini: "#06b6d4",
};

interface MonthLabel {
  month: string;
  weekIndex: number;
}

function formatDateDisplay(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString("en-US", { weekday: "short", month: "short", day: "numeric", year: "numeric" });
}

export function StatsView(props: StatsViewProps) {
  const palette = () => getPalette(props.colorPalette);
  const isNarrowTerminal = () => isNarrow(props.width);
  const grid = () => props.data.contributionGrid;
  const cellWidth = 2;
  
  const [clickedCell, setClickedCell] = createSignal<string | null>(null);
  // DEBUG: keep this for debugging mouse click coordinate mapping
  const [debugInfo, setDebugInfo] = createSignal<string>("No click yet");
  
  const selectedBreakdown = createMemo(() => {
    if (!props.selectedDate) return null;
    if (!props.data.dailyBreakdowns) return null;
    if (!(props.data.dailyBreakdowns instanceof Map)) return null;
    return props.data.dailyBreakdowns.get(props.selectedDate) || null;
  });
  
  const monthPositions = createMemo(() => {
    const sundayRow = grid()[0] || [];
    if (sundayRow.length === 0) return [];
    
    const positions: MonthLabel[] = [];
    let lastMonth = -1;
    const monthNames = isNarrowTerminal() ? MONTHS_SHORT : MONTHS;
    
    for (let weekIdx = 0; weekIdx < sundayRow.length; weekIdx++) {
      const cell = sundayRow[weekIdx];
      if (!cell.date) continue;
      const month = new Date(cell.date).getMonth();
      if (month !== lastMonth) {
        positions.push({ month: monthNames[month], weekIndex: weekIdx });
        lastMonth = month;
      }
    }
    return positions;
  });

  const totalWeeks = createMemo(() => (grid()[0] || []).length);

  const monthLabelRow = createMemo(() => {
    const weeks = totalWeeks();
    const positions = monthPositions();
    const chars: string[] = new Array(weeks * cellWidth).fill(" ");
    
    for (const pos of positions) {
      const startIdx = pos.weekIndex * cellWidth;
      const monthChars = pos.month.split("");
      for (let i = 0; i < monthChars.length && startIdx + i < chars.length; i++) {
        chars[startIdx + i] = monthChars[i];
      }
    }
    
    return chars.join("");
  });

  const dayLabelWidth = () => isNarrowTerminal() ? 2 : 4;

  const getCellStyle = (cellDate: string | null, level: number) => {
    const isSelected = cellDate && (clickedCell() === cellDate || props.selectedDate === cellDate);
    const baseColor = level === 0 ? "#666666" : getGradeColor(palette(), level as 0 | 1 | 2 | 3 | 4);
    
    if (isSelected) {
      return { char: "▓▓", color: "#ffffff", bg: baseColor };
    }
    return { char: level === 0 ? "· " : "██", color: baseColor, bg: undefined };
  };



  return (
    <box flexDirection="column" gap={1}>
      <box flexDirection="column">
        <box flexDirection="row">
          <text dim>{" ".repeat(dayLabelWidth())}</text>
          <text dim>{monthLabelRow()}</text>
        </box>

        {/* DEBUG: onMouseDown handler for grid click - keep for coordinate debugging */}
        <box onMouseDown={(e: { x: number; y: number }) => {
          const labelW = dayLabelWidth();
          const col = Math.floor((e.x - labelW) / cellWidth);
          const row = e.y - 2;
          const gridRows = grid().length;
          
          setDebugInfo(`y=${e.y} row=${row} (y-2) col=${col}`);
          
          if (row < 0 || row >= gridRows) {
            setDebugInfo(`y=${e.y} row=${row} OUT_OF_BOUNDS (0-6)`);
            return;
          }
          if (col < 0) {
            setDebugInfo(`x=${e.x} col=${col} OUT_OF_BOUNDS (label area)`);
            return;
          }
          
          const rowData = grid()[row];
          if (!rowData || col >= rowData.length) {
            setDebugInfo(`y=${e.y} row=${row} col=${col} NO_CELL`);
            return;
          }
          
          const cell = rowData[col];
          if (!cell?.date) {
            setDebugInfo(`y=${e.y} row=${row} col=${col} EMPTY_CELL`);
            return;
          }
          
          setClickedCell(cell.date);
          setDebugInfo(`y=${e.y} row=${row} col=${col} → ${cell.date}`);
        }}>
          <For each={DAYS}>
            {(day, dayIndex) => (
              <box flexDirection="row">
                <text dim>{isNarrowTerminal() ? "  " : day.padStart(3) + " "}</text>
                <For each={grid()[dayIndex()] || []}>
                  {(cell) => {
                    const style = getCellStyle(cell.date, cell.level);
                    return (
                      <text fg={style.color} bg={style.bg}>{style.char}</text>
                    );
                  }}
                </For>
              </box>
            )}
          </For>
        </box>
      </box>

      {/* DEBUG: display click coordinates - keep for debugging */}
      <box flexDirection="row" gap={2}>
        <text fg="yellow">{`DEBUG: ${debugInfo()} | selected: ${clickedCell() || 'none'}`}</text>
      </box>

      <box flexDirection="row" gap={2} marginTop={1}>
        <text dim>Less</text>
        <box flexDirection="row" gap={0}>
          <For each={[0, 1, 2, 3, 4]}>
            {(level) => (
              <text
                fg={level === 0 ? "#666666" : getGradeColor(palette(), level as 0 | 1 | 2 | 3 | 4)}
              >
                {level === 0 ? "· " : "██"}
              </text>
            )}
          </For>
        </box>
        <text dim>More</text>
        <Show when={!isNarrowTerminal()}>
          <text dim>|</text>
          <text dim>Click on a day to see breakdown</text>
        </Show>
      </box>

      <Show when={selectedBreakdown()}>
        <DateBreakdownPanel breakdown={selectedBreakdown()!} isNarrow={isNarrowTerminal()} palette={palette()} />
      </Show>

      <Show when={!selectedBreakdown()}>
        <box flexDirection="column" marginTop={1}>
          <box flexDirection={isNarrowTerminal() ? "column" : "row"} gap={isNarrowTerminal() ? 0 : 4}>
            <box flexDirection="column">
              <box flexDirection="row" gap={1}>
                <text dim>{isNarrowTerminal() ? "Model:" : "Favorite model:"}</text>
                <text fg={getModelColor(props.data.stats.favoriteModel)}>{props.data.stats.favoriteModel}</text>
              </box>
              <box flexDirection="row" gap={1}>
                <text dim>Sessions:</text>
                <text fg="cyan">{props.data.stats.sessions.toLocaleString()}</text>
              </box>
              <box flexDirection="row" gap={1}>
                <text dim>{isNarrowTerminal() ? "Streak:" : "Current streak:"}</text>
                <text fg="cyan">{`${props.data.stats.currentStreak} days`}</text>
              </box>
              <box flexDirection="row" gap={1}>
                <text dim>{isNarrowTerminal() ? "Active:" : "Active days:"}</text>
                <text fg="cyan">{`${props.data.stats.activeDays}/${props.data.stats.totalDays}`}</text>
              </box>
            </box>

            <box flexDirection="column">
              <box flexDirection="row" gap={1}>
                <text dim>{isNarrowTerminal() ? "Tokens:" : "Total tokens:"}</text>
                <text fg="cyan">{formatTokens(props.data.stats.totalTokens)}</text>
              </box>
              <box flexDirection="row" gap={1}>
                <text dim>{isNarrowTerminal() ? "Session:" : "Longest session:"}</text>
                <text fg="cyan">{props.data.stats.longestSession}</text>
              </box>
              <box flexDirection="row" gap={1}>
                <text dim>{isNarrowTerminal() ? "Max streak:" : "Longest streak:"}</text>
                <text fg="cyan">{`${props.data.stats.longestStreak} days`}</text>
              </box>
              <box flexDirection="row" gap={1}>
                <text dim>{isNarrowTerminal() ? "Peak:" : "Peak hour:"}</text>
                <text fg="cyan">{props.data.stats.peakHour}</text>
              </box>
            </box>
          </box>
        </box>

        <Show when={!isNarrowTerminal()}>
          <box marginTop={1}>
            <text fg="yellow" italic>{`Your total spending is $${props.data.totalCost.toFixed(2)} on AI coding assistants!`}</text>
          </box>
        </Show>
        <Show when={isNarrowTerminal()}>
          <box marginTop={1}>
            <text fg="yellow" italic>{`Total: $${props.data.totalCost.toFixed(2)}`}</text>
          </box>
        </Show>
      </Show>
    </box>
  );
}

interface DateBreakdownPanelProps {
  breakdown: DailyModelBreakdown;
  isNarrow: boolean;
  palette: ReturnType<typeof getPalette>;
}

function DateBreakdownPanel(props: DateBreakdownPanelProps) {
  const groupedBySource = createMemo(() => {
    if (!props.breakdown?.models) return new Map();
    const groups = new Map<string, typeof props.breakdown.models>();
    for (const model of props.breakdown.models) {
      const existing = groups.get(model.source) || [];
      existing.push(model);
      groups.set(model.source, existing);
    }
    return groups;
  });

  return (
    <box flexDirection="column" marginTop={1} borderStyle="round" borderColor="#444444" paddingX={1}>
      <box flexDirection="row" justifyContent="space-between">
        <text bold fg="white">{formatDateDisplay(props.breakdown.date)}</text>
        <text fg="green" bold>{formatCost(props.breakdown.cost)}</text>
      </box>
      
      <box flexDirection="column" marginTop={1}>
        <For each={Array.from(groupedBySource().entries())}>
          {([source, models]) => (
            <box flexDirection="column">
              <box flexDirection="row" gap={1}>
                <text fg={SOURCE_COLORS[source] || "#888888"} bold>{`● ${source.toUpperCase()}`}</text>
                <text dim>{`(${models.length} model${models.length > 1 ? "s" : ""})`}</text>
              </box>
              <For each={models}>
                {(model) => (
                  <box flexDirection={props.isNarrow ? "column" : "row"} marginLeft={2} gap={props.isNarrow ? 0 : 2}>
                    <box flexDirection="row" gap={1}>
                      <text fg={getModelColor(model.modelId)}>{model.modelId}</text>
                      <text fg="green">{formatCost(model.cost)}</text>
                    </box>
                    <box flexDirection="row" gap={1}>
                      <Show when={model.tokens.input > 0}>
                        <text dim>In:</text>
                        <text fg="cyan">{formatTokens(model.tokens.input)}</text>
                      </Show>
                      <Show when={model.tokens.output > 0}>
                        <text dim>Out:</text>
                        <text fg="cyan">{formatTokens(model.tokens.output)}</text>
                      </Show>
                      <Show when={model.tokens.cacheRead > 0}>
                        <text dim>Cache:</text>
                        <text fg="cyan">{formatTokens(model.tokens.cacheRead)}</text>
                      </Show>
                    </box>
                  </box>
                )}
              </For>
            </box>
          )}
        </For>
      </box>
      
      <box flexDirection="row" marginTop={1} gap={2}>
        <text dim>Total:</text>
        <text fg="cyan">{formatTokens(props.breakdown.totalTokens)} tokens</text>
        <text dim>|</text>
        <text dim>Click another day or same day to close</text>
      </box>
    </box>
  );
}
