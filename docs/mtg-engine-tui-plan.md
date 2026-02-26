# MTG Engine — Ratatui TUI Applications Plan

> **Purpose**: Plan document for all TUI applications in the MTG rules engine project.
> Built with [Ratatui](https://ratatui.rs/) (v0.30) + crossterm backend.
>
> **Created**: 2026-02-26
> **Status**: Phase 1 complete (Sessions 1-3) — dashboard binary runs, all 5 tabs working

---

## Architecture Overview

### Crate Structure

A single binary crate at `tools/tui/` with subcommands:

```
mtg-tui dashboard    # Project progress dashboard
mtg-tui stepper      # Game state stepper (replay debugger)
mtg-tui cards        # Interactive card browser
mtg-tui rules        # Comprehensive Rules explorer
```

This shares dependencies (ratatui, crossterm, engine crate) and avoids binary proliferation.

```
tools/tui/
├── Cargo.toml
├── src/
│   ├── main.rs                  # CLI dispatch (clap subcommands)
│   ├── event.rs                 # Shared event loop + key handling
│   ├── theme.rs                 # MTG-themed color palette
│   ├── widgets/                 # Shared custom widgets
│   │   ├── mod.rs
│   │   ├── progress_bar.rs      # Colored progress bar with label
│   │   └── status_badge.rs      # Colored status pill (validated/partial/none/gap)
│   ├── dashboard/               # `mtg-tui dashboard`
│   │   ├── mod.rs
│   │   ├── app.rs               # App state, data loading, file watching
│   │   ├── render.rs            # Main render dispatch
│   │   ├── parser.rs            # Markdown file parsers
│   │   ├── tabs/
│   │   │   ├── overview.rs      # Summary gauges + sparklines
│   │   │   ├── milestones.rs    # Milestone table with deliverable drill-down
│   │   │   ├── abilities.rs     # Ability coverage grid
│   │   │   ├── corner_cases.rs  # Corner case audit table
│   │   │   ├── reviews.rs       # Code review findings browser
│   │   │   └── session.rs       # Live session tracker (hook-driven)
│   │   └── data.rs              # Parsed data models
│   ├── stepper/                 # `mtg-tui stepper`
│   │   ├── mod.rs
│   │   ├── app.rs
│   │   ├── render.rs
│   │   └── panels/
│   │       ├── players.rs       # Player panels (life, zones)
│   │       ├── stack.rs         # Stack display
│   │       ├── events.rs        # Event timeline
│   │       ├── assertions.rs    # Assertion badges
│   │       └── card_detail.rs   # Card popup overlay
│   ├── cards/                   # `mtg-tui cards`
│   │   ├── mod.rs
│   │   ├── app.rs
│   │   └── render.rs
│   └── rules/                   # `mtg-tui rules`
│       ├── mod.rs
│       ├── app.rs
│       └── render.rs
```

### Workspace Integration

Add to `Cargo.toml` workspace members:

```toml
members = [
    # ... existing ...
    "tools/tui",
]
```

### Dependencies

```toml
[package]
name = "mtg-tui"
version = "0.1.0"
edition.workspace = true

[dependencies]
# Core TUI
ratatui = "0.30"
crossterm = "0.29"

# CLI
clap = { version = "4", features = ["derive"] }

# Async runtime + file watching
tokio = { version = "1", features = ["full"] }
notify = "6"

# Error handling
anyhow = "1"
color-eyre = "0.6"

# Serialization (for hook state file)
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Engine crate (for stepper)
mtg-engine = { path = "../../crates/engine" }

# Card DB (for card browser)
card-db = { path = "../../crates/card-db" }
```

### Shared Color Theme

MTG-inspired palette used consistently across all subcommands:

```
White    → Color::White       (plains, text)
Blue     → Color::Cyan        (island, info)
Black    → Color::DarkGray    (swamp, backgrounds)
Red      → Color::Red         (mountain, errors, HIGH)
Green    → Color::Green       (forest, success, validated)
Gold     → Color::Yellow      (multicolor, warnings, MEDIUM)
Artifact → Color::Gray        (borders, LOW)
```

Status badge colors:
- `validated` → Green
- `complete` → Cyan
- `partial` → Yellow
- `none` → Red
- `gap` → Red (blinking)
- `deferred` → Gray
- `n/a` → DarkGray

---

## Application 1: Progress Dashboard (FIRST IMPLEMENTATION)

### Overview

A real-time project progress dashboard that parses the existing markdown tracking
documents and presents them as an interactive, color-coded TUI. Optionally integrates
with Claude Code hooks for live session tracking.

**Launch**: `mtg-tui dashboard` or `mtg-tui dashboard --watch` (auto-refresh on file changes)

### Data Sources & Parsing

The dashboard reads these files, all with well-structured markdown table formats:

| Source File | Data Extracted | Parsing Strategy |
|-------------|---------------|-----------------|
| `docs/mtg-engine-ability-coverage.md` | Summary table (P1-P4 counts), per-section ability rows | Regex: `\| P[1-4]\s+\|` for summary; table header detection + row parsing per section |
| `docs/mtg-engine-milestone-reviews.md` | Statistics section, cross-milestone issue index, per-milestone findings | Regex: `## Statistics` section, then parse `\| Metric \| Value \|` table; `### HIGH/MEDIUM/LOW` tables |
| `docs/mtg-engine-corner-case-audit.md` | Summary table, per-case status rows | Regex: `## Summary` table, then `## Corner Case Coverage Table` rows |
| `docs/mtg-engine-roadmap.md` | Milestone overview block, per-milestone `[x]`/`[ ]` deliverables | Regex: fenced code block for overview; `- [x]`/`- [ ]` checkboxes per `### M<N>:` section |
| `CLAUDE.md` | Current State section (active milestone, test counts, last updated) | Regex: `## Current State` section through next `---` |
| `test-data/generated-scripts/` | Script counts per subdirectory | `fs::read_dir` + count `.json` files |
| `tools/tui/state.json` (optional) | Live session data from Claude Code hooks | JSON deserialization |

**Parser module** (`dashboard/parser.rs`): Each source has a dedicated parse function
returning a typed struct. Parsers are lenient — if a section is missing or format changes,
they return `None` for that field rather than crashing.

```rust
// Core parsed data types
pub struct DashboardData {
    pub current_state: CurrentState,
    pub abilities: AbilityCoverage,
    pub milestones: Vec<MilestoneStatus>,
    pub corner_cases: CornerCaseAudit,
    pub reviews: ReviewStatistics,
    pub scripts: ScriptCounts,
    pub session: Option<SessionState>,  // From hooks
}

pub struct CurrentState {
    pub active_milestone: String,
    pub test_count: u32,
    pub script_count: u32,
    pub last_updated: String,
}

pub struct AbilityCoverage {
    pub summary: Vec<PrioritySummary>,     // P1-P4 rows
    pub sections: Vec<AbilitySection>,     // Per-section details
}

pub struct PrioritySummary {
    pub priority: String,     // "P1", "P2", etc.
    pub total: u32,
    pub validated: u32,
    pub complete: u32,
    pub partial: u32,
    pub none: u32,
    pub na: u32,
}

pub struct AbilityRow {
    pub name: String,
    pub cr: String,
    pub priority: String,
    pub status: String,       // "validated", "complete", "partial", "none", "n/a"
    pub engine_files: String,
    pub card_def: String,
    pub script: String,
    pub notes: String,
}

pub struct MilestoneStatus {
    pub id: String,           // "M0", "M1", ..., "M9.5"
    pub name: String,
    pub total_deliverables: u32,
    pub completed_deliverables: u32,
    pub is_active: bool,
    pub review_status: String, // "REVIEWED", "RE-REVIEWED", ""
}

pub struct CornerCaseAudit {
    pub covered: u32,
    pub partial: u32,
    pub gap: u32,
    pub deferred: u32,
    pub cases: Vec<CornerCase>,
}

pub struct CornerCase {
    pub number: u32,
    pub name: String,
    pub status: String,       // "COVERED", "GAP", "DEFERRED", "PARTIAL"
    pub milestone: String,
}

pub struct ReviewStatistics {
    pub total_issues: u32,
    pub high_open: u32,
    pub high_closed: u32,
    pub medium_open: u32,
    pub medium_closed: u32,
    pub low_open: u32,
    pub low_closed: u32,
    pub info: u32,
    pub milestones_reviewed: u32,
    pub engine_loc: u32,
    pub test_loc: u32,
}

pub struct ScriptCounts {
    pub total: u32,
    pub by_directory: Vec<(String, u32)>,
}
```

### Tab Layout

Six tabs, navigated with `Tab`/`Shift+Tab` or number keys `1`-`6`:

#### Tab 1: Overview (default)

The landing page. At-a-glance project health.

```
┌─[1:Overview] 2:Milestones 3:Abilities 4:Corner Cases 5:Reviews 6:Session──┐
│                                                                             │
│  MTG Commander Rules Engine          Active: M10  Tests: 729  Scripts: 71   │
│  Last Updated: 2026-02-26                                                   │
│                                                                             │
│ ┌─Milestone Progress────────────────────────────────────────────────────┐   │
│ │ ████████████████████████████████████████████░░░░░░░░░  10/16 (63%)   │   │
│ │ M0✓ M1✓ M2✓ M3✓ M4✓ M5✓ M6✓ M7✓ M8✓ M9✓ M9.5✓ [M10] M11 M12...  │   │
│ └───────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│ ┌─Ability Coverage──────────────┐  ┌─Corner Cases──────────────────────┐   │
│ │ P1  ██████████████████████ 42 │  │ ████████████████████░░░░  29/36   │   │
│ │     ████████████████████ 40 ✓ │  │ Covered:  29  (81%)              │   │
│ │ P2  █████████████████ 17      │  │ Gap:       4  (11%)              │   │
│ │     ███ 6 ✓                   │  │ Deferred:  3   (8%)              │   │
│ │ P3  ████████████████████ 40   │  └──────────────────────────────────┘   │
│ │     (none validated)          │                                          │
│ │ P4  ██████████████████████ 88 │  ┌─Code Reviews───────────────────-─┐   │
│ │     (none validated)          │  │ HIGH:   0 open  33 closed         │   │
│ └───────────────────────────────┘  │ MEDIUM: 2 open  49 closed         │   │
│                                     │ LOW:   68 open   6 closed         │   │
│ ┌─Scripts by Category───────────┐  │ 12 milestones reviewed            │   │
│ │ baseline ███████ 14           │  └──────────────────────────────────┘   │
│ │ combat   ████████ 16          │                                          │
│ │ stack    █████████████ 25     │  ┌─Engine Size─────────────────────-─┐   │
│ │ ...                           │  │ Source: ~17,800 LOC                │   │
│ └───────────────────────────────┘  │ Tests:  ~25,400 LOC               │   │
│                                     │ Viewer: ~4,600 LOC                │   │
│                                     └──────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────────────┤
│ q:quit  Tab:next  1-6:jump  r:refresh  ?:help                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

Widgets used: `Gauge` (milestone progress), `BarChart` (script counts, ability bars),
`Paragraph` (text blocks), `Block` (panels), `Sparkline` (if we add historical data later).

#### Tab 2: Milestones

Scrollable table of all milestones with completion status.

```
┌─Milestones────────────────────────────────────────────────────────────────-─┐
│                                                                              │
│  #    Name                               Deliverables  Review    Status      │
│ ─────────────────────────────────────────────────────────────────────────    │
│  M0   Project Scaffold & Data Foundation    10/10      RE-REVIEWED  ✓       │
│  M1   Game State & Object Model              8/8      RE-REVIEWED  ✓       │
│  M2   Turn Structure & Priority              7/7      RE-REVIEWED  ✓       │
│  M3   Stack, Spells & Abilities              6/6      REVIEWED     ✓       │
│  M4   State-Based Actions                    5/5      REVIEWED     ✓       │
│  M5   The Layer System                       8/8      REVIEWED     ✓       │
│  M6   Combat                                 7/7      REVIEWED     ✓       │
│  M7   Card Definition Framework              9/9      REVIEWED     ✓       │
│  M8   Replacement & Prevention Effects       6/6      REVIEWED     ✓       │
│  M9   Commander Rules Integration            8/8      REVIEWED     ✓       │
│  M9.5 Game State Stepper                     5/5      REVIEWED     ✓       │
│ →M10  Networking Layer                       0/7      —            ACTIVE   │
│  M11  Tauri App Shell & Basic UI             0/6      —                    │
│  ...                                                                        │
│                                                                              │
│ ──────────── M10 Deliverables ──────────────────────────────────────────    │
│  [ ] Server binary with game room lifecycle                                 │
│  [ ] Client connection, authentication, reconnection                        │
│  [ ] Event broadcasting with hidden-information filtering                   │
│  [ ] Command validation and forwarding                                      │
│  [ ] Spectator mode                                                         │
│  [ ] Integration tests with multi-client scenarios                          │
│  [ ] Latency benchmarks                                                     │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ j/k:scroll  Enter:expand  q:quit  Tab:next tab                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

Widgets: `Table` (stateful, scrollable), `List` (deliverables), `Paragraph` (detail view).

#### Tab 3: Abilities

The ability coverage grid — the most data-dense tab.

```
┌─Abilities──────────────────────────────────────────────────────────────────-┐
│                                                                              │
│ Filter: [All ▼]  Section: [All ▼]  Search: [_______________]                │
│                                                                              │
│  ┌─Summary─────────────────────────────────────────────────────────────┐    │
│  │ P1: ██████████████████████████████████████████ 40/42 validated      │    │
│  │ P2: ██████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  6/17 validated      │    │
│  │ P3: ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  0/40 validated      │    │
│  │ P4: ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  0/88 validated      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  § Evergreen Keywords (P1: 16/16 validated)                                 │
│  ──────────────────────────────────────────────────────────────             │
│  Deathtouch    P1 ✓validated  │ Defender      P1 ✓validated                │
│  Double Strike P1 ✓validated  │ First Strike  P1 ✓validated                │
│  Flash         P1 ✓validated  │ Flying        P1 ✓validated                │
│  Haste         P1 ✓validated  │ Hexproof      P1 ✓validated                │
│  Indestructible P1 ✓validated │ Lifelink      P1 ✓validated                │
│  Menace        P1 ✓validated  │ Protection    P1 ✓validated                │
│  Reach         P1 ✓validated  │ Shroud        P1 ✓validated                │
│  Trample       P1 ✓validated  │ Vigilance     P1 ✓validated                │
│                                                                              │
│  § Cost Modification (P2: 3/3 validated, P3: 0/3)                          │
│  ──────────────────────────────────────────────────────────────             │
│  Convoke       P2 ✓validated  │ Delve         P2 ✓validated                │
│  Kicker        P2 ✓validated  │ Improvise     P3 ○none                     │
│  Affinity      P3 ○none       │ Undaunted     P3 ○none                     │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ j/k:scroll  /:search  f:filter  Enter:detail  q:quit                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

Widgets: `Table`/`List` (ability rows), `Gauge` (summary bars), custom `StatusBadge` widget.

Enter on an ability row shows a popup with:
- CR reference
- Engine file(s)
- Card definitions using it
- Script reference
- Notes

#### Tab 4: Corner Cases

The 36 corner cases with status, drill-down, and gap highlighting.

```
┌─Corner Cases────────────────────────────────────────────────────────────-───┐
│                                                                              │
│  Covered: 29 (81%)  Gap: 4 (11%)  Deferred: 3 (8%)                         │
│                                                                              │
│  #  Name                                CR          Status     Milestone     │
│  ── ──────────────────────────────────  ──────────  ─────────  ──────────   │
│   1 Humility + Opalescence             613.8       ✓COVERED   M4            │
│   2 Blood Moon + Urborg (BM newer)     613.8       ✓COVERED   M4            │
│  ...                                                                         │
│  25 Phasing and Auras/Equipment        702.26      ◌DEFERRED  —             │
│  ...                                                                         │
│  36 Blood Moon + Urza's Saga           714.4       ✗GAP       —             │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ j/k:scroll  Enter:detail  g:show gaps only  q:quit                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

#### Tab 5: Reviews

Code review findings browser with severity filtering.

```
┌─Code Reviews───────────────────────────────────────────────────────────-────┐
│                                                                              │
│ Filter: [All ▼]  Severity: [All ▼]  Status: [OPEN ▼]                       │
│                                                                              │
│  Total: 232 issues │ 0 HIGH open │ 2 MEDIUM open │ 68 LOW open             │
│                                                                              │
│  ID          Sev     Milestone  Summary                          Status      │
│  ─────────── ─────── ────────── ────────────────────────────── ──────────   │
│  MR-M7-09   MEDIUM  M7         Search with no valid targets     OPEN        │
│  MR-M7-12   MEDIUM  M7         Goad duration tracking            OPEN        │
│  MR-M0-08   LOW     M0         No ON DELETE CASCADE              OPEN        │
│  MR-M0-09   LOW     M0         JSON columns stored as TEXT       OPEN        │
│  MR-M0-10   LOW     M0         Partial card name matching        OPEN        │
│  ...                                                                         │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ j/k:scroll  Enter:detail  s:severity filter  o:open only  q:quit             │
└──────────────────────────────────────────────────────────────────────────────┘
```

#### Tab 6: Session (Hook-Driven, Phase 2)

Live Claude Code session tracking. Requires hook integration (see below).

```
┌─Live Session────────────────────────────────────────────────────────────-───┐
│                                                                              │
│  Agent: ability-impl-runner         Ability: Ward                            │
│  Phase: implement                   Step: 3/4 (triggers)                     │
│  Started: 14:32                     Duration: 8m 23s                         │
│                                                                              │
│  ┌─Progress──────────────────────────────────────────────────────────┐      │
│  │ [✓] 1. Add KeywordAbility::Ward enum                              │      │
│  │ [✓] 2. Enforce Ward in targeting validation                       │      │
│  │ [→] 3. Wire Ward trigger in abilities.rs                          │      │
│  │ [ ] 4. Write unit tests                                           │      │
│  └───────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│  ┌─Recent Events─────────────────────────────────────────────────────┐      │
│  │ 14:40:15  Edit state/types.rs — added Ward(u32) variant           │      │
│  │ 14:40:28  Edit rules/casting.rs — Ward check in target validation │      │
│  │ 14:40:45  cargo test -- ward — 3/7 passing                        │      │
│  │ 14:41:02  Edit rules/abilities.rs — trigger wiring                │      │
│  └───────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│  (No active session)  — waiting for hook data...                             │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ q:quit  Tab:next tab  r:refresh                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### File Watching Strategy

Two modes:

1. **Manual refresh** (`r` key): Re-parse all source files, update display.
2. **Watch mode** (`--watch` flag): Use the `notify` crate to watch source files.
   On any change detected, debounce (250ms) then re-parse the changed file(s) only.

Watched paths:
- `docs/mtg-engine-ability-coverage.md`
- `docs/mtg-engine-milestone-reviews.md`
- `docs/mtg-engine-corner-case-audit.md`
- `docs/mtg-engine-roadmap.md`
- `CLAUDE.md`
- `test-data/generated-scripts/`
- `tools/tui/state.json` (if exists)

### Claude Code Hook Integration (Phase 2)

Claude Code hooks fire shell commands on events. The TUI reads a JSON state file
that hooks write to.

**State file**: `tools/tui/state.json`

```json
{
  "session": {
    "agent": "ability-impl-runner",
    "ability": "Ward",
    "phase": "implement",
    "step": 3,
    "total_steps": 4,
    "started_at": "2026-02-26T14:32:00Z",
    "events": [
      {
        "timestamp": "2026-02-26T14:40:15Z",
        "type": "file_edit",
        "detail": "Edit state/types.rs — added Ward(u32) variant"
      }
    ]
  },
  "last_test_run": {
    "timestamp": "2026-02-26T14:40:45Z",
    "passed": 3,
    "failed": 4,
    "total": 7,
    "filter": "ward"
  }
}
```

**Hook configuration** (`.claude/settings.json` additions):

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit",
        "command": "tools/tui/hooks/on-edit.sh \"$FILE_PATH\""
      },
      {
        "matcher": "Bash",
        "command": "tools/tui/hooks/on-bash.sh \"$COMMAND\""
      }
    ]
  }
}
```

Hook scripts parse tool output and append to `state.json`. The TUI picks up
changes via file watch. This is Phase 2 — the dashboard works without hooks,
just with markdown parsing.

### Implementation Plan (Dashboard)

**Phase 1: Core Dashboard (Sessions 1-3)**

Session 1 — Scaffold + parsers:
- [x] Create `tools/tui/` crate with Cargo.toml, add to workspace
- [x] Implement `main.rs` with clap subcommand routing
- [x] Implement `event.rs` shared event loop (crossterm poll + key dispatch)
- [x] Implement `theme.rs` color constants
- [x] Implement `dashboard/parser.rs`: parse `ability-coverage.md` summary table
- [x] Implement `dashboard/parser.rs`: parse `corner-case-audit.md` summary + rows
- [x] Implement `dashboard/data.rs` typed data models

Session 2 — First four tabs:
- [x] Implement `dashboard/app.rs` with tab state + data loading
- [x] Implement `dashboard/render.rs` main render dispatch
- [x] Implement `tabs/overview.rs` — summary gauges, headline stats
- [x] Implement `tabs/abilities.rs` — ability grid with status badges
- [x] Implement `tabs/corner_cases.rs` — corner case table
- [x] Implement `dashboard/parser.rs`: parse `milestone-reviews.md` statistics section
- [x] Implement `dashboard/parser.rs`: parse `roadmap.md` milestone deliverables

Session 3 — Remaining tabs + polish:
- [x] Implement `tabs/milestones.rs` — milestone table with deliverable drill-down
- [x] Implement `tabs/reviews.rs` — findings browser with severity filter
- [x] Implement `widgets/status_badge.rs` custom widget
- [x] Implement `widgets/progress_bar.rs` custom widget
- [x] Script counting from `test-data/generated-scripts/`
- [x] Parse `CLAUDE.md` Current State section
- [x] Manual refresh (`r` key) support

**Phase 2: File Watching + Session Tab (Session 4)**

Session 4 — Live updates:
- [ ] Implement file watching with `notify` crate
- [ ] Debounced re-parse on file change (250ms)
- [ ] Implement `tabs/session.rs` — reads `state.json`
- [ ] Write hook shell scripts (`on-edit.sh`, `on-bash.sh`)
- [ ] Document hook configuration

---

## Application 2: Game State Stepper TUI

### Motivation

The existing replay viewer (axum + Svelte) requires building a web frontend and
running an HTTP server. This causes OOM kills in agent contexts (SIGKILL exit 137).
A TUI stepper links the engine directly as a Rust library — zero serialization
overhead, instant startup, works over SSH.

### Design

**Launch**: `mtg-tui stepper <script.json>` or `mtg-tui stepper --dir test-data/generated-scripts/`

**Layout** (4-player game):

```
┌─Phase: Main 1 ── Turn 3 (Player 1)────────────────────────────────────-────┐
│ ┌─P1 (Active) Life:40──┐ ┌─P2 Life:38─────────┐ ┌─P3/P4──────────────┐   │
│ │ Hand: 6 cards         │ │ Hand: 7 cards       │ │ P3 Life:40 Hand:7  │   │
│ │ Battlefield:          │ │ Battlefield:         │ │ P4 Life:40 Hand:7  │   │
│ │  Forest               │ │  Mountain            │ │                    │   │
│ │  Llanowar Elves (2/2) │ │  Sol Ring             │ │                    │   │
│ │  [more...]            │ │                      │ │                    │   │
│ └───────────────────────┘ └──────────────────────┘ └────────────────────┘   │
├─Stack──────────────────────────────────────────────────────────────────-────┤
│ (empty)                                                                     │
├─Event Timeline─────────────────────────────────────────────────────────-────┤
│ → Step 12: CastSpell(Lightning Bolt) targeting Llanowar Elves  [P2]         │
│   Step 11: PassPriority [P1]                                                │
│   Step 10: Land Drop: Forest [P1]                                           │
│   Step  9: PassPriority [P4]                                                │
├─Assertions─────────────────────────────────────────────────────────────-────┤
│ ✓ P2.life == 38  ✓ Bolt in graveyard  ○ Elves in graveyard (pending)       │
├─────────────────────────────────────────────────────────────────────────────┤
│ ←/→:step  j/k:scroll events  Enter:card detail  p:pick script  q:quit      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Key Features

- **Step forward/backward** through game states (←/→ keys)
- **Card detail popup** — Enter on any card shows full oracle text, types, abilities
- **Assertion tracking** — green check / red X / gray circle for each assertion at each step
- **Script picker** — `p` opens a fuzzy-searchable list of available scripts
- **Debug mode** — `d` toggles showing hidden information (all hands, library order)
- **Zone browser** — Tab cycles through zones; scrollable within zones
- **Search** — `/` to search for card names across all zones

### Engine Integration

```rust
use mtg_engine::testing::replay_harness::ReplayHarness;
use mtg_engine::testing::script_schema::GameScript;

// Load and execute script
let script: GameScript = serde_json::from_str(&fs::read_to_string(path)?)?;
let harness = ReplayHarness::new();
let states: Vec<GameState> = harness.execute_script(&script)?;

// Navigate states
app.current_step = 0;
app.states = states;
app.script = script;
```

No HTTP server, no serialization boundary, no web stack.

### Implementation Plan (Stepper)

**Phase 1: Basic Stepper (Sessions 1-2)**

Session 1 — Core stepping:
- [ ] Implement `stepper/app.rs` — load script, execute, store states
- [ ] Implement `stepper/render.rs` — main layout
- [ ] Implement `panels/players.rs` — player panels with life, zone summaries
- [ ] Implement `panels/events.rs` — event timeline
- [ ] Forward/backward stepping

Session 2 — Full feature set:
- [ ] Implement `panels/stack.rs` — stack display
- [ ] Implement `panels/assertions.rs` — assertion badges
- [ ] Implement `panels/card_detail.rs` — popup overlay
- [ ] Script picker (fuzzy search)
- [ ] Debug mode toggle
- [ ] Zone browser with scroll

---

## Application 3: Card Browser

### Design

**Launch**: `mtg-tui cards` or `mtg-tui cards --search "Lightning"`

Browse all CardDefinitions in the engine. Two-panel layout: list on left, detail on right.

```
┌─Card Browser───────────────────────────────────────────────────────────-────┐
│ Search: [lightning___________]  Filter: [All Types ▼]                       │
│                                                                              │
│ ┌─Cards (56)─────────────┐  ┌─Detail──────────────────────────────────-─┐  │
│ │ Avacyn, Angel of Hope   │  │ Lightning Bolt                            │  │
│ │ Beast Within             │  │ {R} — Instant                             │  │
│ │ Bladetusk Boar           │  │                                           │  │
│ │ Burst Lightning          │  │ Oracle: Lightning Bolt deals 3 damage     │  │
│ │ Command Tower            │  │ to any target.                            │  │
│ │ Counterspell             │  │                                           │  │
│ │ Darksteel Plate          │  │ ── Engine Implementation ──               │  │
│ │ ...                      │  │                                           │  │
│ │→Lightning Bolt           │  │ Effects:                                  │  │
│ │ Lightning Greaves        │  │   DealDamage { amount: 3, target: ... }   │  │
│ │ Llanowar Elves           │  │                                           │  │
│ │ ...                      │  │ Keywords: (none)                          │  │
│ │                          │  │ Types: Instant                            │  │
│ │                          │  │ Colors: Red                               │  │
│ └──────────────────────────┘  └───────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────┤
│ j/k:scroll  /:search  Enter:expand  q:quit                                  │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Implementation Plan (Card Browser)

Single session:
- [ ] Implement `cards/app.rs` — load CardDefinitions from engine
- [ ] Implement `cards/render.rs` — two-panel layout
- [ ] Search/filter functionality
- [ ] Detail panel with oracle text + engine Effect DSL side-by-side

---

## Application 4: Rules Explorer

### Design

**Launch**: `mtg-tui rules` or `mtg-tui rules --search "replacement effects"`

Browse Comprehensive Rules from the SQLite database (same data the MCP server uses).

```
┌─Rules Explorer─────────────────────────────────────────────────────────-────┐
│ Search: [replacement effect__]                                               │
│                                                                              │
│ ┌─Sections──────────────────┐  ┌─Rule Detail──────────────────────────-─┐  │
│ │ 1. Game Concepts           │  │ 614. Replacement Effects               │  │
│ │ 2. Parts of a Card         │  │                                        │  │
│ │ 3. Card Types              │  │ 614.1 Some continuous effects are      │  │
│ │ 4. Zones                   │  │ replacement effects. Like prevention   │  │
│ │ 5. Turn Structure          │  │ effects (see rule 615), replacement    │  │
│ │ 6. Spells, Abilities...    │  │ effects apply continuously as events   │  │
│ │ ▶ 6.1 Replacement Effects  │  │ happen—they aren't locked in ahead    │  │
│ │   6.1.4 Rule 614           │  │ of time...                             │  │
│ │   6.1.5 Rule 614.1         │  │                                        │  │
│ │   6.1.5a                   │  │ Children:                              │  │
│ │   6.1.5b                   │  │  614.1a Effects that use "instead"     │  │
│ │ ...                        │  │  614.1b Effects that use "skip"        │  │
│ │                            │  │  ...                                    │  │
│ │                            │  │                                        │  │
│ │                            │  │ ── Test Coverage ──                    │  │
│ │                            │  │ 8 tests cite this rule                 │  │
│ │                            │  │ 2 scripts cite this rule               │  │
│ └────────────────────────────┘  └────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────┤
│ j/k:scroll  /:search  Enter:expand  t:show tests  q:quit                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Implementation Plan (Rules Explorer)

Single session:
- [ ] Implement `rules/app.rs` — load rules from SQLite (reuse `card-db` crate)
- [ ] Implement `rules/render.rs` — two-panel layout
- [ ] Section tree navigation
- [ ] Full-text search
- [ ] Cross-reference with test files (grep for CR citations)

---

## Implementation Priority & Effort Estimates

| Application | Priority | Sessions | Dependencies |
|-------------|----------|----------|-------------|
| Progress Dashboard (Phase 1) | 1 — FIRST | 3 | None (markdown parsing only) |
| Progress Dashboard (Phase 2) | 2 | 1 | Claude Code hooks |
| Game State Stepper | 3 | 2 | Engine crate (replay harness) |
| Card Browser | 4 | 1 | Engine crate (CardDefinition) |
| Rules Explorer | 5 | 1 | card-db crate (SQLite) |

Total: ~8 implementation sessions.

The dashboard can be built entirely independently of the engine — it only parses
markdown files. The stepper, card browser, and rules explorer depend on engine
and card-db crates.

---

## Open Questions

1. **Async or sync event loop?** The dashboard with file watching benefits from
   tokio. The stepper doesn't need async. Options:
   - Use tokio everywhere (consistent, adds ~200KB binary size)
   - Use tokio only for dashboard, sync loop for others
   - Recommendation: tokio everywhere — the binary already links the engine (~2MB),
     200KB is negligible

2. **Historical data for sparklines?** The dashboard could track metric snapshots
   over time (test count, ability count, etc.) in a small JSON file. This enables
   sparkline trends. Worth adding in Phase 2.

3. **Stepper: share code with replay-viewer?** The `StateViewModel` in
   `replay_harness.rs` is already public and shared. The TUI stepper would use the
   same `ReplayHarness` API. No duplication needed.

4. **Color theme customization?** A `--theme` flag or config file for color-blind
   accessibility. Could define 2-3 preset themes (default, high-contrast, monochrome).
   Defer to after initial implementation.
