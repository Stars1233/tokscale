import { BarChart } from "./BarChart.js";
import { Legend } from "./Legend.js";
import { ModelListItem } from "./ModelListItem.js";
import type { TUIData } from "../hooks/useData.js";

interface OverviewViewProps {
  data: TUIData | null;
  selectedIndex: number;
  scrollOffset: number;
  height: number;
  width: number;
}

function formatCost(cost: number): string {
  if (cost >= 1000) return `$${(cost / 1000).toFixed(1)}K`;
  return `$${cost.toFixed(2)}`;
}

export function OverviewView({ data, selectedIndex, scrollOffset, height, width }: OverviewViewProps) {
  if (!data) return <text dim>No data</text>;

  const safeHeight = Math.max(height, 12);
  const chartHeight = Math.max(5, Math.floor(safeHeight * 0.35));
  const listHeight = Math.max(4, safeHeight - chartHeight - 4);
  const itemsPerPage = Math.max(1, Math.floor(listHeight / 2));
  
  const topModelsForLegend = data.topModels.slice(0, 5).map(m => m.modelId);
  
  const visibleModels = data.topModels.slice(scrollOffset, scrollOffset + itemsPerPage);
  const totalModels = data.topModels.length;
  const endIndex = Math.min(scrollOffset + visibleModels.length, totalModels);

  return (
    <box flexDirection="column" gap={1}>
      <box flexDirection="column">
        <BarChart data={data.chartData} width={width - 4} height={chartHeight} />
        <Legend models={topModelsForLegend} />
      </box>

      <box flexDirection="column">
        <box justifyContent="space-between" marginBottom={0}>
          <text bold>Models by Cost</text>
          <box>
            <text dim>Total: </text>
            <text fg="green">{formatCost(data.totalCost)}</text>
          </box>
        </box>
        
        <box flexDirection="column">
          {visibleModels.map((model, i) => (
            <ModelListItem
              key={model.modelId}
              modelId={model.modelId}
              percentage={model.percentage}
              inputTokens={model.inputTokens}
              outputTokens={model.outputTokens}
              isSelected={scrollOffset + i === selectedIndex}
            />
          ))}
        </box>
        
        {totalModels > visibleModels.length && (
          <text dim>{`↓ ${scrollOffset + 1}-${endIndex} of ${totalModels} models (↑↓ to scroll)`}</text>
        )}
      </box>
    </box>
  );
}
