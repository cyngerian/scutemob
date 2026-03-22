# Runtime Integrity: Watchdog, Recovery, and Bug Reporting

> **Purpose**: Ensure that when a rules bug occurs during a live game, it is caught
> immediately, the game state is recoverable, and the bug is diagnosable. This is a
> pre-alpha requirement — a game that silently corrupts and can't recover is a showstopper.
>
> **Created**: 2026-03-21
> **Status**: PROPOSAL — not yet scheduled

---

## Problem

The engine is a complex system modeling a Turing-complete game. Complete correctness
is provably impossible. Bugs will exist at alpha and beyond. The question is not
whether bugs will occur during a live game, but what happens when they do.

Currently:
- 83 property invariant tests run during `cargo test` but **not at runtime**
- M10 plans rewind/pause but only for player-initiated rollback, not error recovery
- M11 plans rewind UI but no mechanism to detect *when* a rewind is needed
- No runtime validation that game rules are being followed
- No bug reporting mechanism to capture reproduction cases from live games

If a bug corrupts game state, every subsequent state builds on the corruption.
Rewind doesn't help because the engine's history faithfully preserves the incorrect
state. The longer corruption goes undetected, the less recoverable the game becomes.

| Detection timing | State recoverable? | Game recoverable? |
|-----------------|-------------------|-------------------|
| Same command (immediate) | Yes | Yes — roll back one step, fix, continue |
| Within a few steps | Yes | Mostly — replay changes some outcomes |
| Many turns later | Mechanically yes | Practically no — too much history diverges |
| Never detected | No | No |

**The investment must go into immediate detection.**

---

## Proposal: Three-Layer Integrity System

### Layer 1: Runtime Invariant Checker (engine crate)

Extract the existing 83 property invariant assertions into a callable function:

```rust
/// Validates structural and logical invariants on the game state.
/// Returns Ok(()) if all invariants hold, or Err with a description
/// of every violation found.
pub fn validate_invariants(state: &GameState) -> Result<(), Vec<InvariantViolation>>;
```

**What it checks** (from existing invariants.rs):
- Zone integrity: every object in exactly one zone, no dangling references
- Player validity: active_player and priority_holder within bounds
- Stack consistency: LIFO order, stack zone length matches stack_objects
- Object conservation: no objects created or destroyed outside of explicit effects
- Mana pool: non-negative components
- Commander: tax never negative, damage_received valid
- Continuous effects: unique effect IDs
- Timestamps: monotonically increasing
- Attachment validity: no `attached_to` pointing at objects in wrong zones

**What it adds** (new, rule-aware invariants):
- SBA completeness: after an SBA pass, no SBA conditions remain
  (creatures with 0 toughness on battlefield, auras with invalid targets, etc.)
- Trigger completeness: after an event, all permanents with matching trigger
  conditions have queued triggers (catches "trigger didn't fire" bugs)
- Layer correctness: for a sample of permanents, verify that calculated
  characteristics match a fresh layer recalculation (catches stale layer cache)
- Zone-change identity: no ObjectId appears in both a zone and the graveyard/exile
  (catches the CR 400.7 "new object" violation)

**Performance budget**: This runs after every `process_command`. Must be < 1ms for
a typical 4-player board state. The structural checks are O(N) where N is total
objects. The rule-aware checks can be sampled (check 10 random permanents per call
instead of all) to stay within budget.

**Integration point**: The server (M10) calls `validate_invariants` after every
`process_command`. If it fails, the server pauses the game before broadcasting the
new state. The corrupt state is never sent to clients.

### Layer 2: State Recovery (server crate, extends M10)

M10 already plans a state history ring buffer and rewind. This proposal extends it:

**Automatic recovery on invariant failure:**
1. Invariant checker fires after command N
2. Server does NOT broadcast the new state
3. Server rolls back to state before command N (the last known-good state)
4. Server broadcasts to all clients: `ServerMessage::IntegrityError { command_index: N, violations: [...], rolled_back_to: N-1 }`
5. Game is paused automatically
6. Players see: "A rules inconsistency was detected. The game has been rolled back
   to before the last action. The error has been logged for investigation."
7. Players can choose: retry the action (if engine is patched), skip the action
   (manual adjustment), or save and quit

**Known-good state tagging:** After each successful `validate_invariants`, tag the
state as "verified." The rewind system preferentially targets verified states.
If a bug slips past the checker (checker has a gap), manual rewind still targets
the most recent verified state, which limits the blast radius.

**State diff on failure:** When an invariant fails, compute and log the diff between
the last known-good state and the corrupt state. This narrows the bug to exactly
what changed in the failing command.

### Layer 3: Bug Reporter (client + server)

When an integrity error occurs OR when a player manually reports something wrong:

**Automatic capture (on integrity error):**
- Full event log from game start through the failing command
- The last known-good state (serialized)
- The corrupt state (serialized)
- The command that caused the failure
- All invariant violations
- Game configuration (players, decks, format)

**Manual capture (player clicks "Report Bug"):**
- Everything above, plus:
- Player's description of what looked wrong
- Screenshot or board state snapshot
- The last 10 commands and their resulting states

**Output format:** A self-contained JSON file that can be loaded into the replay
harness for reproduction. This is essentially an extended game script — the existing
script format plus the error context.

**Privacy consideration:** The full event log includes all players' hidden information
(hands, library order). Bug reports from multiplayer games should strip or encrypt
private information from other players before submission. The reporting player's own
hidden info is included (they consented by reporting).

---

## Where This Fits in the Milestone Plan

### Decided: M9.7 → M10 → M11

**Layer 1** is a standalone mini-milestone **M9.7** between card authoring and M10.
Pure engine crate, no dependencies on server or UI. 5-8 sessions. See
`docs/mtg-engine-roadmap.md` M9.7 section.

**Layer 2** (auto-rollback, recovery) is part of **M10** deliverables. The server
calls `validate_invariants()` after every `process_command`, rolls back on failure,
serializes bug reports.

**Layer 3** (UI) is part of **M11** deliverables:
- Integrity error display with rollback prompt
- "Report Bug" button
- Manual state adjustment mode (already planned) as fallback for unrecoverable states

---

## What This Doesn't Solve

- **Bugs the checker can't detect**: If the engine consistently applies a wrong rule
  (e.g., always resolves replacement effects in the wrong order), the checker won't
  catch it because the state is internally consistent. These require human testing
  and CR audits.

- **Performance-critical games**: The checker adds latency per command. For casual
  play this is invisible. For tournament-speed play or bots-only simulation, it may
  need to be toggleable.

- **Three-way+ interactions**: The checker validates state after each step, which
  catches the *result* of any interaction (two-way, three-way, N-way) that produces
  an illegal state. But it can't catch interactions that produce legal-but-wrong
  states (e.g., the wrong player gains life). Those still require rule-specific tests.

---

## Appendix: Existing Infrastructure

| Component | Location | Relevant to |
|-----------|----------|-------------|
| 83 property invariant tests | `crates/engine/tests/invariants.rs` (72), `state_invariants.rs` (6), `turn_invariants.rs` (5) | Layer 1 extraction source |
| `process_command` signature | `crates/engine/src/rules/engine.rs:43` — `(GameState, Command) -> Result<(GameState, Vec<GameEvent>), GameStateError>` | Layer 1 integration point |
| State history ring buffer | M10 deliverable (planned) | Layer 2 |
| Rewind command | M10 deliverable (planned) | Layer 2 |
| Pause command | M10 deliverable (planned) | Layer 2 |
| Rewind UI | M11 deliverable (planned) | Layer 3 |
| Manual state adjustment | M11 deliverable (planned) | Layer 3 fallback |
| Game script format | `test-data/generated-scripts/` | Layer 3 bug report format |
| Replay harness | `crates/engine/src/testing/replay_harness.rs` | Layer 1 test integration |
| `im-rs` immutable state | Architecture invariant | Enables rollback to any prior state |
| `GameEvent::reveals_hidden_info()` | M9 | Layer 2 safe checkpoint identification |
