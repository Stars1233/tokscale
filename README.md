# Token Tracker

A high-performance CLI tool and visualization dashboard for tracking AI coding assistant token usage and costs across multiple platforms.

## Overview

**Token Tracker** helps you monitor and analyze your token consumption from:

| Platform | Data Location | Supported |
|----------|---------------|-----------|
| [OpenCode](https://github.com/sst/opencode) | `~/.local/share/opencode/storage/message/` | Yes |
| [Claude Code](https://docs.anthropic.com/en/docs/claude-code) | `~/.claude/projects/` | Yes |
| [Codex CLI](https://github.com/openai/codex) | `~/.codex/sessions/` | Yes |
| [Gemini CLI](https://github.com/google-gemini/gemini-cli) | `~/.gemini/tmp/*/chats/` | Yes |

Get real-time pricing calculations using [LiteLLM's pricing data](https://github.com/BerriAI/litellm), with support for tiered pricing models and cache token discounts.

## Features

- **Multi-platform support** - Track usage across OpenCode, Claude Code, Codex CLI, and Gemini CLI
- **Real-time pricing** - Fetches current pricing from LiteLLM for accurate cost calculations
- **Detailed breakdowns** - Input, output, cache read/write, and reasoning token tracking
- **Hybrid architecture** - TypeScript CLI with optional Rust native module for 10x faster processing
- **GitHub-style visualization** - Interactive contribution graph with 2D and 3D views
- **Flexible filtering** - Filter by platform, date range, or year
- **Export to JSON** - Generate data for external visualization tools

## Installation

### Prerequisites

- Node.js 18+
- (Optional) Rust toolchain for native module

### Quick Start

```bash
# Clone the repository
git clone https://github.com/wakeru-ai/token-usage-tracker.git
cd token-tracker

# Install dependencies
yarn install

# Run the CLI
yarn dev
```

### Building the Native Module (Optional)

The native Rust module provides ~10x faster processing through parallel file scanning and SIMD JSON parsing:

```bash
# Build the native core
yarn build:core

# Verify installation
yarn dev graph --benchmark
```

## Usage

### Basic Commands

```bash
# Show usage breakdown by model (default)
token-tracker

# Show usage breakdown by model
token-tracker models

# Show monthly usage report  
token-tracker monthly

# Export contribution graph data as JSON
token-tracker graph --output data.json
```

### Filtering by Platform

```bash
# Show only OpenCode usage
token-tracker --opencode

# Show only Claude Code usage
token-tracker --claude

# Show only Codex CLI usage
token-tracker --codex

# Show only Gemini CLI usage
token-tracker --gemini

# Combine filters
token-tracker --opencode --claude
```

### Graph Command Options

```bash
# Export graph data to file
token-tracker graph --output usage-data.json

# Filter by date range
token-tracker graph --since 2024-01-01 --until 2024-12-31

# Filter by year
token-tracker graph --year 2024

# Filter by platform
token-tracker graph --opencode --claude

# Force TypeScript implementation (skip native module)
token-tracker graph --no-native

# Show processing time benchmark
token-tracker graph --output data.json --benchmark
```

### Example Output

```
  Token Usage Report by Model

┌──────────────────────┬─────────────────┬───────────┬───────────┬─────────────┬────────────┬───────────┬──────────┐
│ Source/Model         │ Models          │     Input │    Output │ Cache Write │ Cache Read │     Total │     Cost │
├──────────────────────┼─────────────────┼───────────┼───────────┼─────────────┼────────────┼───────────┼──────────┤
│ Claude sonnet-4      │ - sonnet-4      │ 2,456,789 │   345,678 │      89,012 │  1,234,567 │ 4,126,046 │   $45.67 │
├──────────────────────┼─────────────────┼───────────┼───────────┼─────────────┼────────────┼───────────┼──────────┤
│ OpenCode opus-4-5    │ - opus-4-5      │   567,890 │   123,456 │      12,345 │    456,789 │ 1,160,480 │   $23.45 │
├──────────────────────┼─────────────────┼───────────┼───────────┼─────────────┼────────────┼───────────┼──────────┤
│ Total                │                 │ 3,024,679 │   469,134 │     101,357 │  1,691,356 │ 5,286,526 │   $69.12 │
└──────────────────────┴─────────────────┴───────────┴───────────┴─────────────┴────────────┴───────────┴──────────┘

  Total: 156 messages, 5,286,526 tokens, $69.12
```

## Architecture

```
token-tracker/
├── src/                    # TypeScript CLI
│   ├── cli.ts              # Commander.js entry point
│   ├── opencode.ts         # OpenCode session parser
│   ├── claudecode.ts       # Claude Code & Codex parser
│   ├── gemini.ts           # Gemini CLI parser
│   ├── graph.ts            # Graph data generation
│   ├── pricing.ts          # LiteLLM pricing fetcher
│   ├── table.ts            # Terminal table rendering
│   └── native.ts           # Native module loader
│
├── core/                   # Rust native module (napi-rs)
│   ├── src/
│   │   ├── lib.rs          # NAPI exports
│   │   ├── scanner.rs      # Parallel file discovery
│   │   ├── parser.rs       # SIMD JSON parsing
│   │   ├── aggregator.rs   # Parallel aggregation
│   │   └── sessions/       # Platform-specific parsers
│   │       ├── opencode.rs
│   │       ├── claudecode.rs
│   │       ├── codex.rs
│   │       └── gemini.rs
│   ├── Cargo.toml
│   └── package.json
│
├── frontend/               # Next.js visualization
│   └── src/
│       ├── app/            # Next.js app router
│       └── components/     # React components
│           ├── TokenGraph2D.tsx
│           ├── TokenGraph3D.tsx
│           ├── GraphControls.tsx
│           └── ...
│
└── benchmarks/             # Performance benchmarks
    ├── runner.ts           # Benchmark harness
    └── generate.ts         # Synthetic data generator
```

### Hybrid TypeScript + Rust Architecture

Token Tracker uses a hybrid architecture for optimal performance:

1. **TypeScript Layer**: CLI interface, command handling, output formatting
2. **Rust Native Core**: Parallel file scanning, SIMD JSON parsing, aggregation

The native module is loaded dynamically with automatic fallback to pure TypeScript:

```typescript
// Automatic fallback if native module unavailable
if (isNativeAvailable()) {
  return generateGraphNative(options);  // ~10x faster
} else {
  return generateGraphDataTS(options);   // Pure TypeScript
}
```

### Key Technologies

| Layer | Technology | Purpose |
|-------|------------|---------|
| CLI | [Commander.js](https://github.com/tj/commander.js) | Command-line parsing |
| Tables | [cli-table3](https://github.com/cli-table/cli-table3) | Terminal table rendering |
| Colors | [picocolors](https://github.com/alexeyraspopov/picocolors) | Terminal colors |
| Native | [napi-rs](https://napi.rs/) | Node.js bindings for Rust |
| Parallelism | [Rayon](https://github.com/rayon-rs/rayon) | Data parallelism in Rust |
| JSON | [simd-json](https://github.com/simd-lite/simd-json) | SIMD-accelerated parsing |
| Frontend | [Next.js 16](https://nextjs.org/) | React framework |
| 3D Viz | [obelisk.js](https://github.com/nicklockwood/obelisk.js) | Isometric 3D rendering |

## Performance

The native Rust module provides significant performance improvements:

| Operation | TypeScript | Rust Native | Speedup |
|-----------|------------|-------------|---------|
| File Discovery | ~500ms | ~50ms | **10x** |
| JSON Parsing | ~800ms | ~100ms | **8x** |
| Aggregation | ~200ms | ~25ms | **8x** |
| **Total** | **~1.5s** | **~175ms** | **~8.5x** |

*Benchmarks for ~1000 session files, 100k messages*

### Memory Optimization

The native module also provides ~45% memory reduction through:

- Streaming JSON parsing (no full file buffering)
- Zero-copy string handling
- Efficient parallel aggregation with map-reduce

### Running Benchmarks

```bash
# Run benchmarks with real data
yarn bench:ts                    # TypeScript implementation
yarn bench:rust                  # Rust implementation

# Run with synthetic data
yarn bench:generate              # Generate synthetic data first
yarn bench:ts:synthetic          # Benchmark with synthetic data
yarn bench:rust:synthetic
```

## Frontend Visualization

The frontend provides a GitHub-style contribution graph visualization:

### Features

- **2D View**: Classic GitHub contribution calendar
- **3D View**: Isometric 3D contribution graph with height based on cost
- **Multiple color palettes**: GitHub, GitLab, Halloween, Winter, and more
- **Interactive tooltips**: Hover for detailed daily breakdowns
- **Day breakdown panel**: Click to see per-source and per-model details
- **Year filtering**: Navigate between years
- **Source filtering**: Filter by platform (OpenCode, Claude, Codex, Gemini)
- **Stats panel**: Total cost, tokens, active days, streaks

### Running the Frontend

```bash
cd frontend
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) and load your JSON data exported from the CLI.

### Generating Data for Frontend

```bash
# Export data for visualization
token-tracker graph --output frontend/public/my-data.json
```

## Development

### Prerequisites

```bash
# Node.js 18+
node --version

# Rust (for native module)
rustc --version
cargo --version
```

### Setup

```bash
# Install dependencies
yarn install

# Build native module (optional but recommended)
cd core && yarn build && cd ..

# Run in development mode
yarn dev
```

### Project Scripts

| Script | Description |
|--------|-------------|
| `yarn dev` | Run CLI in development mode |
| `yarn build:core` | Build native Rust module (release) |
| `yarn build:core:debug` | Build native module (debug) |
| `yarn bench:generate` | Generate synthetic benchmark data |
| `yarn bench:ts` | Run TypeScript benchmarks |
| `yarn bench:rust` | Run Rust native benchmarks |

### Testing

```bash
# Test native module (Rust)
cd core
yarn test:rust      # Cargo tests
yarn test           # Node.js integration tests
yarn test:all       # Both
```

### Native Module Development

```bash
cd core

# Build in debug mode (faster compilation)
yarn build:debug

# Build in release mode (optimized)
yarn build

# Run Rust benchmarks
yarn bench
```

## Supported Platforms

### Native Module Targets

| Platform | Architecture | Status |
|----------|--------------|--------|
| macOS | x86_64 | Supported |
| macOS | aarch64 (Apple Silicon) | Supported |
| Linux | x86_64 (glibc) | Supported |
| Linux | aarch64 (glibc) | Supported |
| Linux | x86_64 (musl) | Supported |
| Linux | aarch64 (musl) | Supported |
| Windows | x86_64 | Supported |
| Windows | aarch64 | Supported |

## Data Sources

### OpenCode

Location: `~/.local/share/opencode/storage/message/{sessionId}/*.json`

Each message file contains:
```json
{
  "id": "msg_xxx",
  "role": "assistant",
  "modelID": "claude-sonnet-4-20250514",
  "providerID": "anthropic",
  "tokens": {
    "input": 1234,
    "output": 567,
    "reasoning": 0,
    "cache": { "read": 890, "write": 123 }
  },
  "time": { "created": 1699999999999 }
}
```

### Claude Code

Location: `~/.claude/projects/{projectPath}/*.jsonl`

JSONL format with assistant messages containing usage data:
```json
{"type": "assistant", "message": {"model": "claude-sonnet-4-20250514", "usage": {"input_tokens": 1234, "output_tokens": 567, "cache_read_input_tokens": 890}}, "timestamp": "2024-01-01T00:00:00Z"}
```

### Codex CLI

Location: `~/.codex/sessions/*.jsonl`

Event-based format with `token_count` events:
```json
{"type": "event_msg", "payload": {"type": "token_count", "info": {"last_token_usage": {"input_tokens": 1234, "output_tokens": 567}}}}
```

### Gemini CLI

Location: `~/.gemini/tmp/{projectHash}/chats/session-*.json`

Session files containing message arrays:
```json
{
  "sessionId": "xxx",
  "messages": [
    {"type": "gemini", "model": "gemini-2.5-pro", "tokens": {"input": 1234, "output": 567, "cached": 890, "thoughts": 123}}
  ]
}
```

## Pricing

Token Tracker fetches real-time pricing from [LiteLLM's pricing database](https://github.com/BerriAI/litellm/blob/main/model_prices_and_context_window.json).

Pricing includes:
- Input tokens
- Output tokens
- Cache read tokens (discounted)
- Cache write tokens
- Reasoning tokens (for models like o1)
- Tiered pricing (above 200k tokens)

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cd core && yarn test:all`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Development Guidelines

- Follow existing code style
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [LiteLLM](https://github.com/BerriAI/litellm) for pricing data
- [napi-rs](https://napi.rs/) for Rust/Node.js bindings
- [Isometric Contributions](https://github.com/jasonlong/isometric-contributions) for 3D graph inspiration
- [github-contributions-canvas](https://github.com/sallar/github-contributions-canvas) for 2D graph reference
