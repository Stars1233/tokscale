import type { TUIData } from "../hooks/useData.js";
import type { ColorPaletteName } from "../config/themes.js";
import { getPalette, getGradeColor } from "../config/themes.js";

interface StatsViewProps {
  data: TUIData | null;
  height: number;
  colorPalette: ColorPaletteName;
}

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const DAYS = ["", "Mon", "", "Wed", "", "Fri", ""];

export function StatsView({ data, height: _height, colorPalette }: StatsViewProps) {
  if (!data) return null;

  const palette = getPalette(colorPalette);

  const formatTokens = (n: number) => {
    if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(1)}B`;
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
    return n.toString();
  };

  const grid = buildContributionGrid(data.contributions);

  return (
    <box flexDirection="column" gap={1}>
      <box gap={2}>
        <box backgroundColor="gray" paddingX={1}>
          <text fg="white" bold>Models</text>
        </box>
        <text dim>(tab to cycle)</text>
      </box>

      <box flexDirection="column">
        <box gap={1} marginLeft={4}>
          {MONTHS.map((month, i) => (
            <text key={`month-${i}`} dim>{month.padEnd(4)}</text>
          ))}
        </box>
        
        {DAYS.map((day, dayIndex) => (
          <box key={`day-${dayIndex}`}>
            <text dim>{day.padStart(3)} </text>
            {grid[dayIndex]?.map((cell, weekIndex) => (
              <text 
                key={`cell-${dayIndex}-${weekIndex}`} 
                fg={cell.level === 0 ? "gray" : undefined}
                backgroundColor={cell.level > 0 ? getGradeColor(palette, cell.level as 0 | 1 | 2 | 3 | 4) : undefined}
              >
                {cell.level === 0 ? "·" : "█"}
              </text>
            )) || null}
          </box>
        ))}
      </box>

      <box gap={2} marginTop={1}>
        <text dim>Less</text>
        <box gap={0}>
          {[0, 1, 2, 3, 4].map(level => (
            <text 
              key={level}
              fg={level === 0 ? "gray" : undefined}
              backgroundColor={level > 0 ? getGradeColor(palette, level as 0 | 1 | 2 | 3 | 4) : undefined}
            >
              {level === 0 ? "·" : "█"}
            </text>
          ))}
        </box>
        <text dim>More</text>
      </box>

      <box flexDirection="column" marginTop={1}>
        <box gap={4}>
          <box flexDirection="column">
            <box gap={1}>
              <text dim>Favorite model:</text>
              <text fg="cyan">{data.stats.favoriteModel}</text>
            </box>
            <box gap={1}>
              <text dim>Sessions:</text>
              <text fg="cyan">{data.stats.sessions.toLocaleString()}</text>
            </box>
            <box gap={1}>
              <text dim>Current streak:</text>
              <text fg="cyan">{`${data.stats.currentStreak} days`}</text>
            </box>
            <box gap={1}>
              <text dim>Active days:</text>
              <text fg="cyan">{`${data.stats.activeDays}/${data.stats.totalDays}`}</text>
            </box>
          </box>
          
          <box flexDirection="column">
            <box gap={1}>
              <text dim>Total tokens:</text>
              <text fg="cyan">{formatTokens(data.stats.totalTokens)}</text>
            </box>
            <box gap={1}>
              <text dim>Longest session:</text>
              <text fg="cyan">{data.stats.longestSession}</text>
            </box>
            <box gap={1}>
              <text dim>Longest streak:</text>
              <text fg="cyan">{`${data.stats.longestStreak} days`}</text>
            </box>
            <box gap={1}>
              <text dim>Peak hour:</text>
              <text fg="cyan">{data.stats.peakHour}</text>
            </box>
          </box>
        </box>
      </box>

      <box marginTop={1}>
        <text fg="yellow" italic>{`Your total spending is $${data.totalCost.toFixed(2)} on AI coding assistants!`}</text>
      </box>
    </box>
  );
}

interface GridCell {
  date: string | null;
  level: number;
}

function buildContributionGrid(contributions: TUIData["contributions"]): GridCell[][] {
  const grid: GridCell[][] = Array.from({ length: 7 }, () => []);
  
  const today = new Date();
  const startDate = new Date(today);
  startDate.setDate(startDate.getDate() - 364);
  
  const contributionMap = new Map(contributions.map(c => [c.date, c.level]));
  
  const currentDate = new Date(startDate);
  while (currentDate <= today) {
    const dateStr = currentDate.toISOString().split("T")[0];
    const dayOfWeek = currentDate.getDay();
    const level = contributionMap.get(dateStr) || 0;
    
    grid[dayOfWeek].push({ date: dateStr, level });
    currentDate.setDate(currentDate.getDate() + 1);
  }
  
  return grid;
}
