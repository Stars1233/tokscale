import { getModelColor } from "../utils/colors.js";

interface ModelListItemProps {
  modelId: string;
  percentage: number;
  inputTokens: number;
  outputTokens: number;
  isSelected: boolean;
}

function formatTokens(n: number): string {
  if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(1)}B`;
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toString();
}

export function ModelListItem({
  modelId,
  percentage,
  inputTokens,
  outputTokens,
  isSelected,
}: ModelListItemProps) {
  const color = getModelColor(modelId);
  const bgColor = isSelected ? "blue" : undefined;

  return (
    <box flexDirection="column">
      <box backgroundColor={bgColor}>
        <text fg={color}>●</text>
        <text fg={isSelected ? "white" : undefined}>{` ${modelId} `}</text>
        <text dim>{`(${percentage.toFixed(1)}%)`}</text>
      </box>
      <text dim>{`  In: ${formatTokens(inputTokens)} · Out: ${formatTokens(outputTokens)}`}</text>
    </box>
  );
}
