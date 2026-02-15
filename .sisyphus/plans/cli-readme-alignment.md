# CLI README Alignment

## TL;DR

> **Quick Summary**: Align Rust CLI behavior with README documentation - add missing root flags, fix models/monthly to launch TUI tabs, and fix keyboard shortcuts.
> 
> **Deliverables**:
> - Root command with all documented flags (--json, --light, source/date filters)
> - `models`/`monthly` launch TUI with correct tab selected
> - TUI keyboard shortcuts match README (c/n/t, 1-4 for views)
> - `--light` mode outputs table instead of TUI
> 
> **Estimated Effort**: Medium
> **Parallel Execution**: NO - sequential changes to same files
> **Critical Path**: Task 1 → Task 2 → Task 3 → Task 4 → Task 5

---

## Context

### Original Request
Align Rust CLI (crates/tokscale-cli) with README.md documentation. Multiple discrepancies found by validation agents.

### Research Findings
From 5 parallel validation agents:
1. **CLI Commands**: Root command missing --json, --light, source/date filters
2. **models/monthly behavior**: Should launch TUI tabs, currently outputs reports
3. **TUI shortcuts**: README says c/n/t but code does c/t/d; README says 1-4 for views but code only has 1-8 for sources
4. **Env var**: TOKSCALE_API_URL undocumented (minor)

---

## Work Objectives

### Core Objective
Make Rust CLI behavior 100% match README.md documentation.

### Concrete Deliverables
- `crates/tokscale-cli/src/main.rs` - Updated CLI flags and command routing
- `crates/tokscale-cli/src/tui/app.rs` - Fixed keyboard shortcuts
- `crates/tokscale-cli/src/tui/mod.rs` - Add initial_tab parameter

### Definition of Done
- [ ] `tokscale --json` outputs JSON report
- [ ] `tokscale --light` outputs table (no TUI)
- [ ] `tokscale --opencode --claude` filters sources
- [ ] `tokscale models` launches TUI on Models tab
- [ ] `tokscale monthly` launches TUI on Daily tab
- [ ] README keyboard docs match actual behavior (c/t/d, Tab/arrows)
- [ ] `cargo check` passes
- [ ] `cargo build --release` succeeds

### Must NOT Have (Guardrails)
- Breaking changes to existing subcommand behavior
- Removing any existing functionality
- Changes to tokscale-core crate

---

## TODOs

- [ ] 1. Add root command flags to Cli struct

  **What to do**:
  - Add to `Cli` struct in `main.rs`:
    - `json: bool` with `#[arg(long)]`
    - `light: bool` with `#[arg(long)]`
    - `opencode: bool`, `claude: bool`, `codex: bool`, `gemini: bool`, `cursor: bool`, `amp: bool`, `droid: bool`, `openclaw: bool`
    - `today: bool`, `week: bool`, `month: bool`
    - `since: Option<String>`, `until: Option<String>`, `year: Option<String>`
    - `benchmark: bool`

  **Must NOT do**:
  - Don't remove existing theme, refresh, debug, test_data flags

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 2

  **References**:
  - `crates/tokscale-cli/src/main.rs:10-28` - Current Cli struct
  - `crates/tokscale-cli/src/main.rs:33-69` - Models command flags (copy same pattern)

  **Acceptance Criteria**:
  - [ ] `cargo check` passes
  - [ ] `tokscale --help` shows all new flags

  **Commit**: NO (group with Task 2)

---

- [ ] 2. Implement root command behavior with new flags

  **What to do**:
  - In `main.rs` match arm for `None` (default command), handle:
    - If `--json` flag: call `run_models_report(json=true, sources, since, until, year)`
    - If `--light` flag: call `run_models_report(json=false, sources, since, until, year)` (table output)
    - Otherwise: call `tui::run()` with source/date filters applied
  - Build source filter from cli.opencode, cli.claude, etc.
  - Build date filter from cli.today, cli.week, cli.month, cli.since, cli.until

  **Must NOT do**:
  - Don't break subcommand routing

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 1
  - **Blocks**: Task 3

  **References**:
  - `crates/tokscale-cli/src/main.rs:492-510` - Current None match arm
  - `crates/tokscale-cli/src/main.rs:342-347` - build_source_filter usage example
  - `crates/tokscale-cli/src/main.rs:561-612` - run_models_report function

  **Acceptance Criteria**:
  - [ ] `tokscale --json` outputs JSON
  - [ ] `tokscale --light` outputs table
  - [ ] `tokscale --opencode` filters to OpenCode only
  - [ ] `tokscale --today` filters to today only
  - [ ] `tokscale` (no flags) launches TUI

  **Commit**: YES
  - Message: `feat(cli): add root command flags matching README`
  - Files: `crates/tokscale-cli/src/main.rs`

---

- [ ] 3. Fix models/monthly to launch TUI with specific tab

  **What to do**:
  - Add `initial_tab: Option<Tab>` parameter to `tui::run()`
  - In `tui/mod.rs`, pass initial_tab to App::new()
  - In `tui/app.rs`, set `current_tab` from initial_tab if provided
  - Change `Commands::Models` handler:
    - If `--json` or `--light`: run report (current behavior)
    - Otherwise: call `tui::run()` with `initial_tab = Tab::Models`
  - Change `Commands::Monthly` handler:
    - If `--json` or `--light`: run report (current behavior)
    - Otherwise: call `tui::run()` with `initial_tab = Tab::Daily`

  **Must NOT do**:
  - Don't break --json flag behavior

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 2
  - **Blocks**: Task 4

  **References**:
  - `crates/tokscale-cli/src/tui/mod.rs:24-87` - tui::run function
  - `crates/tokscale-cli/src/tui/app.rs:24-30` - Tab enum
  - `crates/tokscale-cli/src/tui/app.rs:132-204` - App::new()
  - `crates/tokscale-cli/src/main.rs:321-347` - Commands::Models handler
  - `crates/tokscale-cli/src/main.rs:348-373` - Commands::Monthly handler

  **Acceptance Criteria**:
  - [ ] `tokscale models` launches TUI on Models tab
  - [ ] `tokscale models --json` outputs JSON (unchanged)
  - [ ] `tokscale monthly` launches TUI on Daily tab
  - [ ] `tokscale monthly --light` outputs table (unchanged)

  **Commit**: YES
  - Message: `feat(cli): models/monthly launch TUI with correct tab`
  - Files: `crates/tokscale-cli/src/main.rs`, `crates/tokscale-cli/src/tui/mod.rs`, `crates/tokscale-cli/src/tui/app.rs`

---

- [ ] 4. Fix README keyboard shortcuts + verify Rust TUI matches TS CLI

  **What to do**:
  - Update `README.md` keyboard navigation section:
    - Change `1-4 or ←/→/Tab: Switch views` → `←/→/Tab: Switch views`
    - Change `c/n/t: Sort by cost/name/tokens` → `c/t/d: Sort by cost/tokens/date`
  - Verify Rust TUI already uses `c/t/d` (it does per validation agent)
  - Verify Rust TUI uses `1-8` for source toggle only (it does)

  **Why this approach**:
  - `1-4` conflicts with `1-8` source toggles
  - TS CLI uses `c/t/d` not `c/n/t`
  - README was wrong, not the implementation

  **Must NOT do**:
  - Don't change TUI behavior - it's correct
  - Don't add conflicting key bindings

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 3
  - **Blocks**: Task 5

  **References**:
  - `README.md:218-227` - TUI Features keyboard navigation section
  - `packages/cli/src/tui/App.tsx:222-276` - TS CLI actual key handling
  - `crates/tokscale-cli/src/tui/app.rs:246-315` - Rust TUI key handling

  **Acceptance Criteria**:
  - [ ] README says `←/→/Tab: Switch views` (no 1-4)
  - [ ] README says `c/t/d: Sort by cost/tokens/date`
  - [ ] Rust TUI behavior unchanged (already correct)

  **Commit**: YES
  - Message: `docs: fix keyboard shortcuts to match actual TUI behavior`
  - Files: `README.md`

---

- [ ] 5. Final verification and cleanup

  **What to do**:
  - Run `cargo check` in crates/tokscale-cli
  - Run `cargo build --release` in crates/tokscale-cli
  - Run `cargo clippy` to check for warnings
  - Test each command manually if possible:
    - `./target/release/tokscale --help`
    - `./target/release/tokscale --json`
    - `./target/release/tokscale --light`
    - `./target/release/tokscale models`
  - Push all changes to origin

  **Must NOT do**:
  - Don't skip verification

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 4

  **References**:
  - `crates/tokscale-cli/Cargo.toml` - Build configuration

  **Acceptance Criteria**:
  - [ ] `cargo check` passes with no errors
  - [ ] `cargo build --release` succeeds
  - [ ] All changes pushed to origin

  **Commit**: NO (verification only)

---

## Success Criteria

### Verification Commands
```bash
cd crates/tokscale-cli
cargo check
cargo build --release
./target/release/tokscale --help  # Should show all new flags
./target/release/tokscale --json  # Should output JSON
./target/release/tokscale models  # Should launch TUI on Models tab
```

### Final Checklist
- [ ] All root flags from README implemented
- [ ] models/monthly launch TUI with correct tab
- [ ] README keyboard docs fixed (c/t/d, no 1-4)
- [ ] Build passes
- [ ] Changes pushed to origin
