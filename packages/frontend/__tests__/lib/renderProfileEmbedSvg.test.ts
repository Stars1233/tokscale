import { describe, expect, it } from "vitest";
import {
  renderProfileEmbedErrorSvg,
  renderProfileEmbedSvg,
} from "../../src/lib/embed/renderProfileEmbedSvg";
import type { UserEmbedStats } from "../../src/lib/embed/getUserEmbedStats";

const mockStats: UserEmbedStats = {
  user: {
    id: "user-id",
    username: "octocat",
    displayName: "The Octocat",
    avatarUrl: null,
  },
  stats: {
    totalTokens: 1234567,
    totalCost: 42.42,
    submissionCount: 7,
    rank: 3,
    updatedAt: "2026-02-24T00:00:00.000Z",
  },
};

describe("renderProfileEmbedSvg", () => {
  it("renders a complete SVG with metrics", () => {
    const svg = renderProfileEmbedSvg(mockStats);

    expect(svg).toContain("<svg");
    expect(svg).toContain("Tokscale Stats");
    expect(svg).toContain("@octocat");
    expect(svg).toContain("1,234,567");
    expect(svg).toContain("$42.42");
    expect(svg).toContain("#3");
    expect(svg).toContain("Submissions");
  });

  it("renders compact variant", () => {
    const svg = renderProfileEmbedSvg(mockStats, { compact: true, theme: "light" });

    expect(svg).toContain("width=\"460\"");
    expect(svg).toContain("height=\"162\"");
    expect(svg).toContain("@octocat");
    expect(svg).not.toContain("Submissions");
  });

  it("escapes XML in user-provided text", () => {
    const svg = renderProfileEmbedSvg({
      ...mockStats,
      user: {
        ...mockStats.user,
        displayName: "<script>alert('xss')</script>",
      },
    });

    expect(svg).toContain("&lt;script&gt;alert(&apos;xss&apos;)&lt;/script&gt;");
    expect(svg).not.toContain("<script>alert('xss')</script>");
  });
});

describe("renderProfileEmbedErrorSvg", () => {
  it("renders safe fallback SVG", () => {
    const svg = renderProfileEmbedErrorSvg("User <unknown>", { theme: "light" });

    expect(svg).toContain("Tokscale Stats");
    expect(svg).toContain("User &lt;unknown&gt;");
    expect(svg).not.toContain("User <unknown>");
  });
});
