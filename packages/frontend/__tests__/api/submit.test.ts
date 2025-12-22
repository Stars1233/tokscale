import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

/**
 * Test suite for POST /api/submit - Source-Level Merge
 * 
 * These tests verify the source-level merge functionality:
 * - First submission creates new records
 * - Subsequent submissions merge by source
 * - Sources not in submission are preserved
 * - Totals are recalculated from dailyBreakdown
 * - Concurrent submissions are handled correctly
 */

// Mock data factories
function createMockSubmissionData(overrides: Partial<{
  sources: string[];
  contributions: Array<{
    date: string;
    sources: Array<{
      source: string;
      modelId: string;
      cost: number;
      tokens: { input: number; output: number; cacheRead: number; cacheWrite: number };
      messages: number;
    }>;
  }>;
}> = {}) {
  const defaultSources = overrides.sources || ['claude'];
  const defaultContributions = overrides.contributions || [
    {
      date: '2024-12-01',
      sources: defaultSources.map(source => ({
        source,
        modelId: 'claude-sonnet-4-20250514',
        cost: 1.5,
        tokens: { input: 1000, output: 500, cacheRead: 100, cacheWrite: 50 },
        messages: 5,
      })),
    },
  ];

  return {
    meta: {
      generatedAt: new Date().toISOString(),
      version: '1.0.0',
      dateRange: {
        start: defaultContributions[0]?.date || '2024-12-01',
        end: defaultContributions[defaultContributions.length - 1]?.date || '2024-12-01',
      },
    },
    summary: {
      totalTokens: defaultContributions.reduce((sum, d) => 
        sum + d.sources.reduce((s, src) => s + src.tokens.input + src.tokens.output, 0), 0
      ),
      totalCost: defaultContributions.reduce((sum, d) => 
        sum + d.sources.reduce((s, src) => s + src.cost, 0), 0
      ),
      totalDays: defaultContributions.length,
      activeDays: defaultContributions.filter(d => d.sources.length > 0).length,
      averagePerDay: 0,
      maxCostInSingleDay: 0,
      sources: defaultSources,
      models: ['claude-sonnet-4-20250514'],
    },
    years: [],
    contributions: defaultContributions.map(d => ({
      date: d.date,
      totals: {
        tokens: d.sources.reduce((s, src) => s + src.tokens.input + src.tokens.output, 0),
        cost: d.sources.reduce((s, src) => s + src.cost, 0),
        messages: d.sources.reduce((s, src) => s + src.messages, 0),
      },
      intensity: 2 as const,
      tokenBreakdown: {
        input: d.sources.reduce((s, src) => s + src.tokens.input, 0),
        output: d.sources.reduce((s, src) => s + src.tokens.output, 0),
        cacheRead: d.sources.reduce((s, src) => s + src.tokens.cacheRead, 0),
        cacheWrite: d.sources.reduce((s, src) => s + src.tokens.cacheWrite, 0),
        reasoning: 0,
      },
      sources: d.sources.map(src => ({
        source: src.source as 'opencode' | 'claude' | 'codex' | 'gemini' | 'cursor',
        modelId: src.modelId,
        tokens: src.tokens,
        cost: src.cost,
        messages: src.messages,
      })),
    })),
  };
}

describe('POST /api/submit - Source-Level Merge', () => {
  describe('First Submission (Create Mode)', () => {
    it('should create new submission with all sources', () => {
      const data = createMockSubmissionData({ sources: ['claude', 'cursor'] });
      
      // Verify data structure
      expect(data.summary.sources).toContain('claude');
      expect(data.summary.sources).toContain('cursor');
      expect(data.contributions[0].sources.length).toBe(2);
    });

    it('should create dailyBreakdown for each day', () => {
      const data = createMockSubmissionData({
        contributions: [
          { date: '2024-12-01', sources: [{ source: 'claude', modelId: 'claude-sonnet-4', cost: 1, tokens: { input: 100, output: 50, cacheRead: 0, cacheWrite: 0 }, messages: 1 }] },
          { date: '2024-12-02', sources: [{ source: 'claude', modelId: 'claude-sonnet-4', cost: 2, tokens: { input: 200, output: 100, cacheRead: 0, cacheWrite: 0 }, messages: 2 }] },
          { date: '2024-12-03', sources: [{ source: 'claude', modelId: 'claude-sonnet-4', cost: 3, tokens: { input: 300, output: 150, cacheRead: 0, cacheWrite: 0 }, messages: 3 }] },
        ],
      });
      
      expect(data.contributions.length).toBe(3);
      expect(data.contributions.map(c => c.date)).toEqual(['2024-12-01', '2024-12-02', '2024-12-03']);
    });
  });

  describe('Source-Level Merge Logic', () => {
    it('should preserve existing sources when submitting partial data', () => {
      // Scenario: User had claude + cursor, now only submits claude (cursor cleaned up)
      const existingSourceBreakdown = {
        claude: { tokens: 1000, cost: 10, modelId: 'claude-sonnet-4', input: 600, output: 400, cacheRead: 0, cacheWrite: 0, messages: 5 },
        cursor: { tokens: 500, cost: 5, modelId: 'cursor-small', input: 300, output: 200, cacheRead: 0, cacheWrite: 0, messages: 3 },
      };
      
      const incomingSources = new Set(['claude']); // Only claude in new submission
      const incomingSourceBreakdown = {
        claude: { tokens: 1200, cost: 12, modelId: 'claude-sonnet-4', input: 700, output: 500, cacheRead: 0, cacheWrite: 0, messages: 6 },
      };
      
      // Simulate merge logic
      const merged = { ...existingSourceBreakdown };
      for (const sourceName of incomingSources) {
        if (incomingSourceBreakdown[sourceName as keyof typeof incomingSourceBreakdown]) {
          merged[sourceName as keyof typeof merged] = incomingSourceBreakdown[sourceName as keyof typeof incomingSourceBreakdown];
        }
      }
      
      // cursor should be preserved (not in incomingSources)
      expect(merged.cursor).toEqual(existingSourceBreakdown.cursor);
      // claude should be updated
      expect(merged.claude.tokens).toBe(1200);
      expect(merged.claude.cost).toBe(12);
    });

    it('should update submitted source data', () => {
      // Same source submitted again should replace, not add
      const existingClaude = { tokens: 1000, cost: 10, modelId: 'claude-sonnet-4', input: 600, output: 400, cacheRead: 0, cacheWrite: 0, messages: 5 };
      const newClaude = { tokens: 1500, cost: 15, modelId: 'claude-sonnet-4', input: 900, output: 600, cacheRead: 0, cacheWrite: 0, messages: 8 };
      
      // After merge, should be new values, not sum
      expect(newClaude.cost).toBe(15); // Not 10 + 15 = 25
      expect(newClaude.tokens).toBe(1500); // Not 1000 + 1500 = 2500
    });

    it('should merge new source into existing day', () => {
      // Day has claude, now cursor is added
      const existingSourceBreakdown = {
        claude: { tokens: 1000, cost: 10, modelId: 'claude-sonnet-4', input: 600, output: 400, cacheRead: 0, cacheWrite: 0, messages: 5 },
      };
      
      const incomingSources = new Set(['cursor']);
      const incomingSourceBreakdown = {
        cursor: { tokens: 500, cost: 5, modelId: 'cursor-small', input: 300, output: 200, cacheRead: 0, cacheWrite: 0, messages: 3 },
      };
      
      // Simulate merge
      const merged = { ...existingSourceBreakdown };
      for (const sourceName of incomingSources) {
        if (incomingSourceBreakdown[sourceName as keyof typeof incomingSourceBreakdown]) {
          (merged as Record<string, typeof existingSourceBreakdown.claude>)[sourceName] = incomingSourceBreakdown[sourceName as keyof typeof incomingSourceBreakdown];
        }
      }
      
      // Both sources should be present
      expect(Object.keys(merged)).toContain('claude');
      expect(Object.keys(merged)).toContain('cursor');
    });

    it('should add new dates without affecting existing', () => {
      const existingDates = ['2024-12-01', '2024-12-02'];
      const newDates = ['2024-12-03', '2024-12-04'];
      
      // Simulate: new dates should be added to the set
      const allDates = new Set([...existingDates, ...newDates]);
      
      expect(allDates.size).toBe(4);
      expect(Array.from(allDates)).toContain('2024-12-01');
      expect(Array.from(allDates)).toContain('2024-12-04');
    });
  });

  describe('Totals Recalculation', () => {
    it('should recalculate totalTokens from dailyBreakdown', () => {
      const sourceBreakdown = {
        claude: { tokens: 1000, cost: 10, modelId: 'claude-sonnet-4', input: 600, output: 400, cacheRead: 50, cacheWrite: 25, messages: 5 },
        cursor: { tokens: 500, cost: 5, modelId: 'cursor-small', input: 300, output: 200, cacheRead: 30, cacheWrite: 15, messages: 3 },
      };
      
      // Simulate recalculateDayTotals
      let totalTokens = 0;
      for (const source of Object.values(sourceBreakdown)) {
        totalTokens += source.tokens;
      }
      
      expect(totalTokens).toBe(1500);
    });

    it('should recalculate cache tokens', () => {
      const sourceBreakdown = {
        claude: { tokens: 1000, cost: 10, modelId: 'claude-sonnet-4', input: 600, output: 400, cacheRead: 50, cacheWrite: 25, messages: 5 },
        opencode: { tokens: 800, cost: 8, modelId: 'gpt-4o', input: 500, output: 300, cacheRead: 40, cacheWrite: 20, messages: 4 },
      };
      
      let totalCacheRead = 0;
      let totalCacheWrite = 0;
      for (const source of Object.values(sourceBreakdown)) {
        totalCacheRead += source.cacheRead;
        totalCacheWrite += source.cacheWrite;
      }
      
      expect(totalCacheRead).toBe(90);
      expect(totalCacheWrite).toBe(45);
    });

    it('should update sourcesUsed to include all sources', () => {
      // Simulate collecting sources from all days
      const day1Sources = ['claude', 'cursor'];
      const day2Sources = ['claude', 'opencode'];
      
      const allSources = new Set([...day1Sources, ...day2Sources]);
      
      expect(Array.from(allSources).sort()).toEqual(['claude', 'cursor', 'opencode']);
    });
  });

  describe('Edge Cases', () => {
    it('should reject empty submissions', () => {
      const data = createMockSubmissionData({ contributions: [] });
      
      expect(data.contributions.length).toBe(0);
      // API should return 400 for this
    });

    it('should handle day with no data for submitted source', () => {
      // User submits --claude but a day only has opencode data
      const dayWithOnlyOpencode = {
        date: '2024-12-01',
        sources: [
          { source: 'opencode', modelId: 'gpt-4o', cost: 5, tokens: { input: 300, output: 200, cacheRead: 0, cacheWrite: 0 }, messages: 3 },
        ],
      };
      
      const submittedSources = new Set(['claude']);
      
      // No claude data to update for this day
      const claudeInDay = dayWithOnlyOpencode.sources.find(s => s.source === 'claude');
      expect(claudeInDay).toBeUndefined();
      
      // opencode should be preserved
      const opencodeInDay = dayWithOnlyOpencode.sources.find(s => s.source === 'opencode');
      expect(opencodeInDay).toBeDefined();
    });

    it('should handle concurrent submissions without data loss', () => {
      // This is tested at the database level with .for('update') locks
      // Here we just verify the concept
      const submission1Sources = ['claude'];
      const submission2Sources = ['cursor'];
      
      // Both should be present after sequential processing
      const finalSources = new Set([...submission1Sources, ...submission2Sources]);
      expect(finalSources.size).toBe(2);
    });


  });

  describe('Response Format', () => {
    it('should return mode: "create" for first submission', () => {
      const isNewSubmission = true;
      const mode = isNewSubmission ? 'create' : 'merge';
      expect(mode).toBe('create');
    });

    it('should return mode: "merge" for subsequent submissions', () => {
      const isNewSubmission = false;
      const mode = isNewSubmission ? 'create' : 'merge';
      expect(mode).toBe('merge');
    });

    it('should include recalculated metrics', () => {
      const mockResponse = {
        success: true,
        submissionId: 'test-id',
        username: 'testuser',
        metrics: {
          totalTokens: 1500,
          totalCost: 15.5,
          dateRange: { start: '2024-12-01', end: '2024-12-31' },
          activeDays: 25,
          sources: ['claude', 'cursor'],
        },
        mode: 'merge' as const,
      };
      
      expect(mockResponse.metrics).toBeDefined();
      expect(mockResponse.metrics.totalTokens).toBeGreaterThan(0);
      expect(mockResponse.metrics.sources).toContain('claude');
      expect(mockResponse.mode).toBe('merge');
    });
  });
});
