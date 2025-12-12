import { getModelColor } from "../utils/colors.js";

interface LegendProps {
  models: string[];
}

export function Legend({ models }: LegendProps) {
  if (models.length === 0) return null;

  return (
    <box gap={1} flexWrap="wrap">
      {models.map((modelId, i) => (
        <box key={`${modelId}-${i}`} gap={0}>
          <text fg={getModelColor(modelId)}>●</text>
          <text>{` ${modelId}`}</text>
          {i < models.length - 1 && <text dim>  ·</text>}
        </box>
      ))}
    </box>
  );
}
