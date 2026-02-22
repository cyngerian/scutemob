# Game State Stepper: Developer Replay Viewer

## Purpose

A visual developer tool for stepping through game scripts and watching the MTG engine
process each action. Load any game script JSON, see the full game state at every step,
expand into individual events, and validate correctness with human eyes.

This is a **developer tool** — not shipped with the main client. It lives in
`tools/replay-viewer/` and runs as a local web app.

**Key benefit**: Catches issues that automated assertions miss — wrong zone, unexpected
priority holder, missing trigger, incorrect damage assignment — by making the full game
state visible at every point in a test scenario.

**Secondary benefit**: Svelte components built here are directly reusable in the Tauri
app at M11, giving the main client a head start on UI development.

---

## System Architecture

```
┌─────────────────────────────────────────────────┐
│  Browser (Svelte 5 SPA)                         │
│  ┌─────────────┐ ┌──────────┐ ┌──────────────┐ │
│  │ StepControls │ │ StateView│ │ EventTimeline│ │
│  └─────────────┘ └──────────┘ └──────────────┘ │
│         ↕ JSON fetch                            │
├─────────────────────────────────────────────────┤
│  Rust HTTP Server (axum)           :3030        │
│  ┌───────────┐  ┌──────────────┐                │
│  │ /api/*    │  │ Static files │                │
│  │ endpoints │  │ (Svelte dist)│                │
│  └─────┬─────┘  └──────────────┘                │
│        ↓                                        │
│  Replay Engine (in-process)                     │
│  ┌──────────────────────────────┐               │
│  │ Load GameScript JSON         │               │
│  │ → build_initial_state()      │               │
│  │ → process_command() per step │               │
│  │ → collect StepSnapshot[]     │               │
│  └──────────────────────────────┘               │
└─────────────────────────────────────────────────┘
```

### Data Flow

1. **Startup**: Server loads a game script (CLI arg or API request)
2. **Pre-compute**: Replay harness runs all commands, storing a `StepSnapshot` after each
3. **Serve**: Frontend fetches snapshots on demand as the user steps through
4. **No live engine**: All computation happens at load time; stepping is pure data lookup

### Why Pre-compute?

Game scripts are small (7-200 steps). im-rs structural sharing means storing all snapshots
costs O(total changed nodes), not O(full state x N). A 100-step script with 200 objects
might use ~2MB total. Pre-computing gives instant stepping with zero latency.

---

## Backend Design

### Tech Stack

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `axum` | 0.7 | HTTP server (already planned for M10 networking) |
| `tower-http` | 0.6 | Static file serving for Svelte dist |
| `serde_json` | 1 | JSON serialization (already in engine) |
| `clap` | 4 | CLI argument parsing |
| `mtg-engine` | path | Engine crate (path dependency) |

### CLI Interface

```
replay-viewer [OPTIONS]

Options:
  --script <PATH>        Load a single game script
  --scripts-dir <DIR>    Serve a directory of scripts (default: test-data/generated-scripts/)
  --port <PORT>          HTTP port (default: 3030)
```

### Core Data Structures

```rust
/// A complete replay session for one game script
struct ReplaySession {
    script: GameScript,
    initial_state: GameState,
    steps: Vec<StepSnapshot>,
}

/// One step = one process_command() call + results
struct StepSnapshot {
    index: usize,
    /// The script action that produced this step (for display context)
    script_action: ScriptAction,
    /// The engine command that was sent
    command: Command,
    /// All events emitted by this command
    events: Vec<GameEvent>,
    /// Game state after this command resolved
    state_after: GameState,
    /// Assertion results if this step had an assert_state action
    assertions: Option<Vec<AssertionResult>>,
}

/// Result of checking one assertion
struct AssertionResult {
    path: String,           // e.g., "players.p2.life"
    expected: serde_json::Value,
    actual: serde_json::Value,
    passed: bool,
}
```

### API Endpoints

#### `GET /api/scripts`
List all available scripts (when using `--scripts-dir`).

```json
{
  "scripts": [
    {
      "path": "baseline/001_priority_pass_empty_stack.json",
      "name": "Priority pass with empty stack",
      "tags": ["baseline", "priority"],
      "review_status": "approved"
    }
  ]
}
```

#### `GET /api/session`
Current loaded script metadata and step count.

```json
{
  "script_id": "script_stack_001",
  "name": "Lightning Bolt resolves targeting player",
  "description": "p1 casts Lightning Bolt targeting p2...",
  "total_steps": 6,
  "cr_sections_tested": ["601.2", "608.2", "120.3"],
  "tags": ["stack", "instant", "damage"]
}
```

#### `GET /api/step/:n`
Full data for step N. This is the primary endpoint the frontend uses.

```json
{
  "index": 3,
  "script_action": {
    "type": "priority_round",
    "players": ["p1", "p2"],
    "result": "all_pass"
  },
  "command": { "PassPriority": { "player": "p2" } },
  "events": [
    { "PriorityPassed": { "player": "p2" } },
    { "AllPlayersPassed": {} },
    { "SpellResolved": { "player": "p1", "stack_object_id": 99 } },
    { "DamageDealt": { "source": 99, "target": { "Player": "p2" }, "amount": 3 } }
  ],
  "state": { /* view model — see below */ },
  "assertions": null
}
```

#### `GET /api/step/:n/state`
Just the view model for step N (lighter payload for rapid stepping).

#### `POST /api/load`
Load a different script.

```json
{ "path": "stack/004_lightning_bolt_resolves.json" }
```

### View Model

The state endpoint returns a **view model** shaped for the UI, not raw `GameState`.
This decouples the frontend from engine internals.

```json
{
  "turn": {
    "number": 1,
    "active_player": "p1",
    "phase": "PreCombatMain",
    "step": "PreCombatMain",
    "priority": "p1"
  },
  "players": {
    "p1": {
      "life": 40,
      "poison": 0,
      "mana_pool": { "R": 1, "G": 0, "W": 0, "U": 0, "B": 0, "C": 0 },
      "hand_size": 6,
      "library_size": 93,
      "graveyard_size": 1,
      "commander_damage_received": {},
      "land_plays_remaining": 1
    },
    "p2": {
      "life": 37,
      "poison": 0,
      "mana_pool": {},
      "hand_size": 7,
      "library_size": 94,
      "graveyard_size": 0,
      "commander_damage_received": {},
      "land_plays_remaining": 1
    }
  },
  "zones": {
    "battlefield": {
      "p1": [
        {
          "object_id": 42,
          "name": "Forest",
          "card_types": ["Land"],
          "subtypes": ["Forest"],
          "tapped": true,
          "summoning_sick": false,
          "power": null,
          "toughness": null,
          "counters": {},
          "damage_marked": 0,
          "attached_to": null,
          "attachments": [],
          "is_commander": false
        }
      ],
      "p2": []
    },
    "hand": {
      "p1": [{ "name": "Mountain", "card_types": ["Land"] }],
      "p2": [{ "name": "(hidden)", "card_types": [] }]
    },
    "graveyard": {
      "p1": [{ "name": "Lightning Bolt", "card_types": ["Instant"] }],
      "p2": []
    },
    "exile": [],
    "command_zone": {
      "p1": [],
      "p2": []
    },
    "stack": []
  },
  "combat": null,
  "continuous_effects": []
}
```

**Note on hidden information**: This is a developer tool, so all information is shown
(including opponents' hands). A `show_hidden: bool` query parameter could optionally
hide this for testing the player-facing view.

---

## Frontend Design

### Directory Structure

```
tools/replay-viewer/
├── Cargo.toml                      # Rust binary
├── src/
│   ├── main.rs                     # axum server, CLI, replay engine bridge
│   ├── replay.rs                   # ReplaySession, StepSnapshot, pre-compute logic
│   ├── api.rs                      # Route handlers
│   └── view_model.rs               # GameState → view model conversion
├── frontend/
│   ├── package.json                # Svelte 5 + Vite 6
│   ├── vite.config.js
│   ├── svelte.config.js
│   ├── index.html
│   └── src/
│       ├── main.js                 # Svelte mount point
│       ├── App.svelte              # Root layout
│       └── lib/
│           ├── api.js              # Fetch wrapper for /api/*
│           ├── stores.js           # Svelte stores: currentStep, session, etc.
│           ├── StepControls.svelte
│           ├── StateView.svelte
│           ├── EventTimeline.svelte
│           ├── PlayerPanel.svelte
│           ├── ZoneBattlefield.svelte
│           ├── ZoneStack.svelte
│           ├── ZoneHand.svelte
│           ├── ZoneGraveyard.svelte
│           ├── ZoneExile.svelte
│           ├── CardDisplay.svelte
│           ├── CombatView.svelte
│           ├── PhaseIndicator.svelte
│           └── ScriptPicker.svelte
└── dist/                           # Built frontend (generated, gitignored)
```

### Component Hierarchy

```
App.svelte
├── ScriptPicker          (top bar — select which script to view)
├── PhaseIndicator        (phase/step bar with current position highlighted)
├── StepControls          (⏮ ◀ ▶ ⏭ + step counter + keyboard nav)
├── StateView             (main content area)
│   ├── PlayerPanel × N   (one per player: life, mana, poison, cmdr damage)
│   ├── ZoneBattlefield   (permanent grid per player)
│   ├── ZoneStack         (ordered stack items)
│   ├── ZoneHand × N      (card list per player)
│   ├── ZoneGraveyard × N (card list per player)
│   ├── ZoneExile         (shared exile pile)
│   └── CombatView        (shown during combat steps only)
└── EventTimeline         (right sidebar — scrollable event list)
    └── EventGroup × N    (one per command; expandable to show individual events)
```

### Component Contracts (Props-Based)

All components receive data via props. They do **not** fetch data internally.
This is critical for reuse in the Tauri app.

```svelte
<!-- ZoneBattlefield.svelte -->
<script>
  export let permanents = [];  // Array of permanent view-model objects
  export let playerName = "";
</script>
```

```svelte
<!-- PlayerPanel.svelte -->
<script>
  export let player = {};      // { life, poison, mana_pool, ... }
  export let playerName = "";
  export let isActive = false;
  export let hasPriority = false;
</script>
```

The data-fetching layer (`api.js` + `stores.js`) is the only part that differs
between the stepper and the Tauri app.

### Stepping Model

**Per-command (default)**:
Each "step" = one `process_command()` call. The `EventTimeline` shows all events
from that command as a collapsed group with a summary line (e.g., "PassPriority(p2) — 4 events").

**Per-event (drill-down)**:
Click a command group in the timeline to expand it. Individual events are listed
with descriptions. Since events are applied atomically (no intermediate states),
the drill-down shows what changed semantically, not intermediate states.

**Navigation**:
| Key | Action |
|-----|--------|
| `→` | Next step |
| `←` | Previous step |
| `Shift+→` | Next phase transition |
| `Shift+←` | Previous phase transition |
| `Home` | First step (initial state) |
| `End` | Last step |
| `Space` | Auto-play (1 step/second) |

### State Diff Highlighting (Phase 3)

When stepping forward, changed fields are highlighted:
- Life total changed → flash yellow
- New permanent on battlefield → flash green border
- Permanent left battlefield → flash red, fade out
- Stack grew → new item highlighted
- Mana pool changed → numbers flash

Implementation: compare current step's view model with previous step's view model,
mark changed fields in the Svelte store, components read diff flags.

---

## Shared Component Strategy for Tauri (M11)

### The Problem

The stepper and the Tauri app both need to render the same game state elements
(battlefield, stack, player panels, etc.). Building these twice wastes effort.

### The Solution

Components are **data-driven** — they accept props and render. The data source
is external. Two thin adapter layers exist:

**Stepper adapter** (`tools/replay-viewer/frontend/src/lib/api.js`):
```js
export async function fetchStep(n) {
  const res = await fetch(`/api/step/${n}`);
  return res.json();
}
```

**Tauri adapter** (future, `tauri-app/src/lib/api.js`):
```js
import { invoke } from '@tauri-apps/api/core';

export async function fetchStep(n) {
  return invoke('get_step', { step: n });
}
```

Both return the same view model shape. Components don't care where the data comes from.

### Import Mechanism

At M11, the Tauri app imports stepper components via one of:

1. **npm workspace** (if both share a monorepo root `package.json`)
2. **`file:` dependency** in `tauri-app/package.json`: `"mtg-components": "file:../tools/replay-viewer/frontend"`
3. **Symlink** `tauri-app/src/lib/components → tools/replay-viewer/frontend/src/lib`

Option 2 (file dependency) is the cleanest — it works with Vite's module resolution
and doesn't require restructuring.

---

## Implementation Phases

### Phase 1: Backend + Minimal UI (~1 week)

Get the data flowing. A developer can load a script and step through it with basic
text rendering.

- Rust HTTP server with axum
- Replay pre-computation from GameScript → Vec<StepSnapshot>
- View model serialization
- API endpoints (session, step, scripts)
- Svelte shell with StepControls + raw JSON state display
- Keyboard navigation

### Phase 2: Rich Visualization (~1 week)

Make the state actually readable at a glance.

- PlayerPanel, ZoneBattlefield, ZoneStack, ZoneHand, ZoneGraveyard, ZoneExile
- PhaseIndicator
- EventTimeline with per-event expansion
- Basic CSS styling for readability

### Phase 3: Polish & Script Browser (~0.5-1 week)

Make it pleasant to use across many scripts.

- ScriptPicker with directory tree
- CombatView
- CardDisplay with full details
- State diff highlighting
- Assertion result badges

---

## Open Questions (Decide at Implementation Time)

1. **CSS framework**: Plain CSS vs. Tailwind? Plain CSS is simpler for a dev tool;
   Tailwind gives faster iteration. Decide based on preference at build time.

2. **Hidden information toggle**: Show all info by default (dev tool). Add a toggle
   to hide opponents' hands for testing the player-facing view? Nice to have, not
   required for Phase 1-3.

3. **Hot reload during development**: Vite's dev server proxies API requests to the
   Rust server. Standard setup: `vite.config.js` proxy `{ '/api': 'http://localhost:3030' }`.
   Or serve the dist statically and rebuild on change.

4. **Script editing**: Should the stepper allow editing a script and re-running it?
   Out of scope for M9.5 — scripts are authored externally. But the API supports
   `POST /api/load` for switching between scripts.
