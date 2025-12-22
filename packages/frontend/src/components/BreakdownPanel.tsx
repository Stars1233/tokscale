"use client";

import type { DailyContribution, GraphColorPalette, SourceType } from "@/lib/types";
import { formatDateFull, formatCurrency, formatTokenCount, groupSourcesByType, sortSourcesByCost } from "@/lib/utils";
import { SOURCE_DISPLAY_NAMES, SOURCE_COLORS } from "@/lib/constants";
import { SourceLogo } from "./SourceLogo";

interface BreakdownPanelProps {
  day: DailyContribution | null;
  onClose: () => void;
  palette: GraphColorPalette;
}

export function BreakdownPanel({ day, onClose, palette }: BreakdownPanelProps) {
  if (!day) return null;

  const groupedSources = groupSourcesByType(day.sources);
  const sortedSourceTypes = Array.from(groupedSources.keys()).sort();

  return (
    <div
      role="region"
      aria-label="Day breakdown"
      className="mt-8 rounded-2xl border overflow-hidden shadow-sm transition-shadow hover:shadow-md"
      style={{ backgroundColor: "var(--color-card-bg)", borderColor: "var(--color-border-default)" }}
    >
      <div className="flex items-center justify-between px-6 py-4 border-b" style={{ borderColor: "var(--color-border-default)" }}>
        <h3 className="font-bold text-lg" style={{ color: "var(--color-fg-default)" }}>
          {formatDateFull(day.date)} - Detailed Breakdown
        </h3>
        <button
          onClick={onClose}
          aria-label="Close breakdown panel"
          className="p-2 rounded-full hover:bg-[var(--color-btn-hover-bg)] transition-all duration-200 hover:scale-110 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2"
          style={{ color: "var(--color-fg-muted)" }}
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>

      <div className="p-6">
        {day.sources.length === 0 ? (
          <p className="text-center py-8 text-sm font-medium" style={{ color: "var(--color-fg-muted)" }}>
            No activity on this day
          </p>
        ) : (
          <div className="space-y-6">
            {sortedSourceTypes.map((sourceType) => {
              const sources = sortSourcesByCost(groupedSources.get(sourceType) || []);
              const sourceTotalCost = sources.reduce((sum, s) => sum + s.cost, 0);
              return (
                <SourceSection key={sourceType} sourceType={sourceType} sources={sources} totalCost={sourceTotalCost} palette={palette} />
              );
            })}
          </div>
        )}

        {day.sources.length > 0 && (
          <div className="mt-6 pt-6 border-t flex flex-wrap gap-6 text-sm" style={{ borderColor: "var(--color-border-default)" }}>
            <div className="font-medium" style={{ color: "var(--color-fg-muted)" }}>
              Total: <span className="font-bold text-base" style={{ color: "var(--color-fg-default)" }}>{formatCurrency(day.totals.cost)}</span>
            </div>
            <div className="font-medium" style={{ color: "var(--color-fg-muted)" }}>
              across <span className="font-semibold" style={{ color: "var(--color-fg-default)" }}>{sortedSourceTypes.length} source{sortedSourceTypes.length !== 1 ? "s" : ""}</span>
            </div>
            <div className="font-medium" style={{ color: "var(--color-fg-muted)" }}>
              <span className="font-semibold" style={{ color: "var(--color-fg-default)" }}>
                {(() => {
                  const allModels = new Set<string>();
                  for (const s of day.sources) {
                    if (s.models) {
                      for (const modelId of Object.keys(s.models)) {
                        allModels.add(modelId);
                      }
                    } else if (s.modelId) {
                      allModels.add(s.modelId);
                    }
                  }
                  const count = allModels.size;
                  return `${count} model${count !== 1 ? "s" : ""}`;
                })()}
              </span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

interface SourceSectionProps {
  sourceType: SourceType;
  sources: DailyContribution["sources"];
  totalCost: number;
  palette: GraphColorPalette;
}

function SourceSection({ sourceType, sources, totalCost, palette }: SourceSectionProps) {
  const sourceColor = SOURCE_COLORS[sourceType] || palette.grade3;

  const modelEntries: Array<{ modelId: string; cost: number; messages: number; tokens: { input: number; output: number; cacheRead: number; cacheWrite: number; reasoning: number } }> = [];
  for (const source of sources) {
    if (source.models && Object.keys(source.models).length > 0) {
      for (const [modelId, data] of Object.entries(source.models)) {
        modelEntries.push({
          modelId,
          cost: data.cost || 0,
          messages: data.messages || 0,
          tokens: {
            input: data.input || 0,
            output: data.output || 0,
            cacheRead: data.cacheRead || 0,
            cacheWrite: data.cacheWrite || 0,
            reasoning: data.reasoning || 0,
          },
        });
      }
    } else if (source.modelId) {
      modelEntries.push({
        modelId: source.modelId,
        cost: source.cost,
        messages: source.messages,
        tokens: source.tokens,
      });
    }
  }

  const sortedModels = modelEntries.sort((a, b) => b.cost - a.cost);

  return (
    <div>
      <div className="flex items-center gap-3 mb-3">
        <span
          className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-semibold transition-transform hover:scale-105"
          style={{ backgroundColor: `${sourceColor}20`, color: sourceColor }}
        >
          <SourceLogo sourceId={sourceType} height={14} />
          {SOURCE_DISPLAY_NAMES[sourceType] || sourceType}
        </span>
        <span className="text-sm font-bold" style={{ color: "var(--color-fg-default)" }}>{formatCurrency(totalCost)}</span>
      </div>

      <div className="ml-5 space-y-3">
        {sortedModels.map((model, index) => (
          <ModelRow key={`${model.modelId}-${index}`} model={model} isLast={index === sortedModels.length - 1} palette={palette} />
        ))}
      </div>
    </div>
  );
}

interface ModelRowProps {
  model: {
    modelId: string;
    cost: number;
    messages: number;
    tokens: { input: number; output: number; cacheRead: number; cacheWrite: number; reasoning: number };
  };
  isLast: boolean;
  palette: GraphColorPalette;
}

function ModelRow({ model, isLast, palette }: ModelRowProps) {
  const { modelId, tokens, cost, messages } = model;

  return (
    <div className="relative">
      <div className="absolute left-0 top-0 w-4 h-full" style={{ color: "var(--color-fg-muted)" }}>
        <span className="absolute left-0 top-3 w-3 border-t" style={{ borderColor: "var(--color-border-default)" }} />
        {!isLast && <span className="absolute left-0 top-0 h-full border-l" style={{ borderColor: "var(--color-border-default)" }} />}
      </div>

      <div className="ml-6">
        <div className="flex items-center gap-3 flex-wrap">
          <span className="font-mono text-sm font-semibold" style={{ color: "var(--color-fg-default)" }}>{modelId}</span>
          <span className="font-bold text-sm" style={{ color: palette.grade1 }}>{formatCurrency(cost)}</span>
        </div>

        <div className="mt-2 grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-x-4 gap-y-2 text-xs">
          {tokens.input > 0 && <TokenBadge label="Input" value={tokens.input} />}
          {tokens.output > 0 && <TokenBadge label="Output" value={tokens.output} />}
          {tokens.cacheRead > 0 && <TokenBadge label="Cache Read" value={tokens.cacheRead} />}
          {tokens.cacheWrite > 0 && <TokenBadge label="Cache Write" value={tokens.cacheWrite} />}
          {tokens.reasoning > 0 && <TokenBadge label="Reasoning" value={tokens.reasoning} />}
        </div>

        <div className="mt-2 text-xs font-medium" style={{ color: "var(--color-fg-muted)" }}>
          {messages.toLocaleString()} message{messages !== 1 ? "s" : ""}
        </div>
      </div>
    </div>
  );
}

function TokenBadge({ label, value }: { label: string; value: number }) {
  return (
    <div className="flex items-center gap-1.5">
      <span className="font-medium" style={{ color: "var(--color-fg-muted)" }}>{label}:</span>
      <span className="font-mono font-semibold" style={{ color: "var(--color-fg-default)" }}>{formatTokenCount(value)}</span>
    </div>
  );
}
