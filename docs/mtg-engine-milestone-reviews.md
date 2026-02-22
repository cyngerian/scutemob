# MTG Engine — Milestone Code Reviews

> **Purpose**: Per-milestone code review tracking. Records files introduced, CR sections
> implemented, findings (bugs, enforcement gaps, code quality, test gaps), and deferred
> issues. Updated as milestones complete and issues are discovered or resolved.
>
> **Convention**: Every completed milestone MUST have its new/changed files reviewed and
> findings added to this document before the milestone is considered done. This is a
> required step in the Milestone Completion Checklist (see CLAUDE.md).
>
> **IMPORTANT: Review only ONE milestone per session.** Do not batch reviews. Each
> milestone review requires careful reading of every source file — rushing through
> multiple milestones in one session leads to shallow reviews and missed issues.
> Finish one, commit, then start a new session for the next.
>
> **Last Updated**: 2026-02-22

---

## Severity Key

| Level | Meaning | Examples |
|-------|---------|---------|
| **CRITICAL** | Wrong game outcome, crash, data loss | Panic in engine library code, incorrect damage calculation |
| **HIGH** | Allows illegal game states | Missing validation, unchecked state transitions |
| **MEDIUM** | Code quality, edge cases, fragile logic | Unsafe casts, fragile parsing, missing error context |
| **LOW** | Performance, style, minor test gaps | Redundant checks, naming inconsistencies |
| **INFO** | Documentation, contracts, design notes | Missing CR citations, architectural observations |

## Issue ID Format

`MR-M{milestone}-{sequence}` — e.g., `MR-M6-03`

---

## Table of Contents

- [M0: Project Scaffold & Data Foundation](#m0-project-scaffold--data-foundation)
- [M1: Game State & Object Model](#m1-game-state--object-model)
- [M2: Turn Structure & Priority](#m2-turn-structure--priority)
- [M3: Stack, Spells & Abilities](#m3-stack-spells--abilities)
- [M4: State-Based Actions](#m4-state-based-actions)
- [M5: The Layer System](#m5-the-layer-system)
- [M6: Combat](#m6-combat)
- [M7: Card Definition Framework & First Cards](#m7-card-definition-framework--first-cards)
- [M8: Replacement & Prevention Effects](#m8-replacement--prevention-effects)
- [M9: Commander Rules Integration](#m9-commander-rules-integration)
- [Cross-Milestone Issue Index](#cross-milestone-issue-index)

---

## M0: Project Scaffold & Data Foundation

**Review Status**: REVIEWED (2026-02-22)

### Files Introduced

| File | Lines | Purpose |
|------|-------|---------|
| `crates/card-db/src/lib.rs` | 36 | Database open/init, error types (thiserror) |
| `crates/card-db/src/schema.rs` | 105 | SQLite schema: cards, card_faces, rulings, card_definitions |
| `tools/scryfall-import/src/main.rs` | 282 | Bulk Scryfall JSON → SQLite importer |
| `tools/mcp-server/src/main.rs` | 467 | MCP server: 4 tools (search_rules, get_rule, lookup_card, search_rulings) |
| `tools/mcp-server/src/rules_db.rs` | 386 | CR text parser, FTS5 index builder |
| `crates/card-db/Cargo.toml` | — | rusqlite (bundled), thiserror |
| `tools/scryfall-import/Cargo.toml` | — | ureq, anyhow, serde_json |
| `tools/mcp-server/Cargo.toml` | — | rmcp, tokio, schemars |

**Source total**: ~1,276 lines | **Tests**: (inline in rules_db.rs only)

### CR Sections Implemented

None directly — M0 is infrastructure. CR text is parsed and indexed for lookup.

### Findings

| ID | Severity | File | Description | Status |
|----|----------|------|-------------|--------|
| MR-M0-01 | **HIGH** | scryfall-import/main.rs | **Delete-then-import pattern risks data loss.** Lines ~145: `DELETE FROM card_faces; DELETE FROM cards;` clears all data before reimport. If import fails midway (network, OOM, disk full), database is left empty or partial. Should use temp table + atomic swap, or wrap in explicit transaction with rollback. | OPEN |
| MR-M0-02 | **HIGH** | mcp-server/main.rs | **FTS5 MATCH operator injection.** User queries passed directly to `WHERE rules_fts MATCH ?1`. FTS5 interprets operators (`AND`, `OR`, `NOT`, quotes, parentheses). Malformed queries can cause FTS5 parse errors. Parameterized queries protect against SQL injection but not FTS syntax injection. Fix: wrap input in double-quotes to force literal matching. | OPEN |
| MR-M0-03 | **MEDIUM** | mcp-server/rules_db.rs | **Multi-line CR rules not fully captured.** Parser treats each line independently. CR rules spanning 2+ lines in the text file have only the first line captured as `rule_text`. Affects completeness of FTS index. | OPEN |
| MR-M0-04 | **MEDIUM** | mcp-server/rules_db.rs | **CR format assumptions fragile.** Parsing relies on "Glossary" and "Credits" as exact case-sensitive stop markers, position-based detection (after `seen_rules`). No CR version metadata captured. If Wizards changes the format or casing, import silently produces fewer rules with no validation. | OPEN |
| MR-M0-05 | **MEDIUM** | mcp-server/main.rs | **FTS index probe is fragile.** Lines ~441-454: probes for the word "the" to detect if FTS index is populated. Not guaranteed to exist in all future CR revisions. Should query table count or metadata directly. | OPEN |
| MR-M0-06 | **MEDIUM** | scryfall-import/main.rs | **JSON parse errors lose context.** `serde_json::from_reader()` on a ~200MB file gives no indication of which card or line caused the failure. Debugging reimport failures is painful. | OPEN |
| MR-M0-07 | **MEDIUM** | scryfall-import/main.rs | **No download integrity check.** Bulk downloads have no timeout, resumption, or checksum validation. Corrupted download produces silent parse failures. | OPEN |
| MR-M0-08 | **LOW** | card-db/schema.rs | **No ON DELETE CASCADE** for `card_faces.card_id` FK. Orphaned card_faces possible if cards are deleted without cascading. Not a practical issue with current delete-all pattern, but schema doesn't enforce it. | OPEN |
| MR-M0-09 | **LOW** | card-db/schema.rs | **JSON columns stored as TEXT.** `colors`, `color_identity`, `keywords`, `legalities` are TEXT not JSON type. Requires callers to always serialize correctly. Risk of accidental string matching instead of JSON queries. | OPEN |
| MR-M0-10 | **LOW** | mcp-server/main.rs | **Partial card name matching too broad.** `name LIKE '%' || ?1 || '%'` matches substrings ("Sol" matches "Sollen's Zendikon"). Single-letter queries return hundreds of results before LIMIT. | OPEN |
| MR-M0-11 | **INFO** | card-db/lib.rs | Clean error types, WAL mode, foreign keys enabled. No issues. | — |
| MR-M0-12 | **INFO** | mcp-server/rules_db.rs | Good test coverage: section headers, rule lines, parent computation, TOC edge cases. | — |

### Notes

- M0 files are **tools/binaries**, not core engine. `unwrap()`/`expect()` and `anyhow` are
  acceptable per project conventions. Findings focus on data integrity and parsing fragility.
- `card-db` is a library crate using `thiserror` — correct pattern.
- The MCP server is consumed by Claude Code only (trusted input), so the FTS injection risk
  is low-probability but should be fixed for defense-in-depth.
- Scryfall importer is run manually and infrequently. The delete-then-import pattern is
  tolerable for dev use but unacceptable for any automated pipeline (M12).

---

## M1: Game State & Object Model

**Review Status**: REVIEWED (2026-02-22)

### Files Introduced

**Source files:**

| File | Lines | Purpose |
|------|-------|---------|
| `crates/engine/src/state/types.rs` | 105 | Color, ManaColor, SuperType, CardType, CounterType, KeywordAbility enums |
| `crates/engine/src/state/player.rs` | 83 | PlayerId, CardId, ManaPool, PlayerState (Commander fields) |
| `crates/engine/src/state/game_object.rs` | 221 | ObjectId, Characteristics, ManaAbility, GameObject |
| `crates/engine/src/state/zone.rs` | 185 | ZoneId, ZoneType, Zone (Ordered/Unordered), operations |
| `crates/engine/src/state/turn.rs` | 121 | Phase, Step, TurnState |
| `crates/engine/src/state/error.rs` | 75 | GameStateError enum (thiserror) |
| `crates/engine/src/state/builder.rs` | 676 | GameStateBuilder + ObjectSpec + PlayerBuilder |
| `crates/engine/src/state/stubs.rs` | 43 | Placeholder types (PendingTrigger, etc.) |
| `crates/engine/src/state/mod.rs` | 268 | GameState struct, add_object, move_object_to_zone |
| `crates/engine/src/lib.rs` | 30 | Module declarations, re-exports |

**Source total**: 1,807 lines

**Test files:**

| File | Lines | Tests | Focus |
|------|-------|-------|-------|
| `tests/structural_sharing.rs` | 200 | 4 | im-rs clone independence (mock types) |
| `tests/state_foundation.rs` | 170 | 12 | GameState construction, field defaults, accessors |
| `tests/zone_integrity.rs` | 266 | 11 | Zone invariants, add/remove/move, shuffle |
| `tests/object_identity.rs` | 276 | 10 | CR 400.7 zone-change identity |
| `tests/builder_tests.rs` | 381 | 24 | Fluent builder API coverage |
| `tests/state_invariants.rs` | 176 | 5 | Property-based (proptest): zone integrity, unique IDs |
| `tests/snapshot_perf.rs` | 221 | 5 | Structural sharing with real types, performance |

**Test total**: 1,690 lines, 71 tests

### CR Sections Implemented

| CR Section | Implementation |
|------------|----------------|
| CR 400.7 | Zone-change identity: new ObjectId, reset status/counters/attachments (`state/mod.rs:move_object_to_zone`) |
| CR 109 (colors) | Color, ManaColor enums (`state/types.rs`) |
| CR 205 (types) | CardType, SuperType enums (`state/types.rs`) |

### Findings

| ID | Severity | File | Description | Status |
|----|----------|------|-------------|--------|
| MR-M1-01 | **HIGH** | state/mod.rs:159 | **`.unwrap()` in `add_object()`.** `self.zones.get_mut(&zone_id).unwrap()` after a `contains_key` guard. Provably safe in isolation (im-rs prevents concurrent mutation between check and access), but violates the architectural constraint: "engine crate uses typed errors — never `unwrap()` or `expect()` in engine logic." If a future refactor removes the guard or reorders code, this panics the engine. Should use `.ok_or(GameStateError::ZoneNotFound(zone_id))?`. | OPEN |
| MR-M1-02 | **HIGH** | state/mod.rs:228 | **`.unwrap()` in `move_object_to_zone()`.** Same pattern as MR-M1-01: `self.zones.get_mut(&to).unwrap()` after earlier validation. Same fix needed. | OPEN |
| MR-M1-03 | **MEDIUM** | state/builder.rs:318 | **`.expect()` in `build()`.** `state.add_object(object, zone).expect("failed to add object in builder")`. Builder is documented as a test utility that panics on invalid configuration, so the convention is arguably acceptable. However, `build()` is public API and could be used outside tests. Consider returning `Result` or documenting the panic contract. | OPEN |
| MR-M1-04 | **MEDIUM** | state/mod.rs | **Check-then-access pattern.** Both `add_object` and `move_object_to_zone` use `contains_key()` + `get_mut().unwrap()` instead of the idiomatic `get_mut().ok_or()?` pattern. Creates maintenance risk — the guard and the access can drift apart during refactoring. | OPEN |
| MR-M1-05 | **MEDIUM** | state/builder.rs:181 | **Panics on 0 players.** `build()` panics if `self.players.is_empty()`. Could return `Result` for consistency with engine error handling philosophy. Currently has `#[should_panic]` test in builder_tests.rs so it's tested, but violates typed-error convention. | OPEN |
| MR-M1-06 | **LOW** | structural_sharing.rs | **Uses mock types, not real GameState.** Tests im-rs principle with stand-in structs. Real structural sharing validated in `snapshot_perf.rs`, so this is redundant — not wrong, just low-value. | OPEN |
| MR-M1-07 | **LOW** | state_foundation.rs | **ManaPool tests thin.** Only 1 test (`test_mana_pool_operations`) covering basic add/total/empty. No colored mana allocation, insufficient mana, or complex scenarios. Adequate for M1 (pool used properly starting M3). | OPEN |
| MR-M1-08 | **INFO** | object_identity.rs | **Exemplary CR citation.** All 10 tests directly reference CR 400.7 with specific sub-behaviors. Model for other test files. | — |
| MR-M1-09 | **INFO** | state_invariants.rs | **Good property-based foundation.** 5 proptest tests covering zone integrity, unique IDs, move semantics. Could expand with state determinism properties in M3+. | — |
| MR-M1-10 | **INFO** | — | **Commander format compliance verified.** `PlayerState` defaults: life=40, commander_tax tracking, commander_damage_received matrix, poison_counters. All correct for Commander. | — |
| MR-M1-11 | **INFO** | — | **Type safety is strong.** PlayerId, ObjectId, CardId are distinct types. ZoneId enum prevents invalid zone references. No accidental ID confusion possible. | — |

### Test Coverage Assessment

| M1 Behavior | Coverage | Notes |
|-------------|----------|-------|
| GameState construction | Good (12+24 tests) | Defaults, accessors, builder |
| Zone operations | Good (11 tests) | Insert, remove, move, shuffle, ordering |
| Object identity (CR 400.7) | Excellent (10 tests) | Status reset, controller reset, card_id persistence, timestamps |
| Player state defaults | Good (3 tests) | Life, mana, poison, land plays, commander fields |
| ManaPool operations | Thin (1 test) | Basic add/total/empty only |
| Builder fluent API | Good (24 tests) | All type/zone/modifier combinations |
| Structural sharing | Good (4+5 tests) | Mock + real types, performance |
| State invariants | Good (5 proptests) | Zone integrity, unique IDs, move preservation |
| Error handling | Thin (2 tests) | Only panic and move-nonexistent |
| Zone queries | Limited (1 test) | Only `objects_in_zone()` |

### Notes

- `state/stubs.rs` (43 lines) contains placeholder types (`PendingTrigger`) that are later
  filled in by M3-E. Not a finding — intentional forward declaration.
- `builder.rs` at 676 lines is the largest M1 file. The fluent API is well-designed
  but the `expect()` in `build()` should be addressed.
- `im::OrdMap` used consistently for deterministic iteration — correct per CLAUDE.md.

---

## M2: Turn Structure & Priority

**Review Status**: REVIEWED (2026-02-22)

### Files Introduced

**Source files (M2 contributions — these files grow in later milestones):**

| File | Lines (M7) | M2 Contribution | Purpose |
|------|------------|------------------|---------|
| `rules/command.rs` | 113 | `PassPriority`, `Concede` variants (M3+ adds CastSpell, etc.) | Command enum |
| `rules/engine.rs` | 395 | `process_command`, `handle_pass_priority`, `handle_all_passed`, `enter_step`, `handle_concede`, `check_game_over`, `is_game_over`, `start_game`, validation helpers | Engine entry point and game loop |
| `rules/events.rs` | 398 | First 15 variants (TurnStarted through GameOver), `LossReason` | GameEvent enum |
| `rules/priority.rs` | 105 | All code (unchanged since M2) | APNAP ordering, pass/grant priority |
| `rules/turn_structure.rs` | 133 | Core logic; M6 added FirstStrikeDamage insertion | Step ordering, turn advancement |
| `rules/turn_actions.rs` | 274 | `untap`, `draw`, `cleanup`, `empty_mana_pools`, `reset_turn_state` (M4+ adds clear_damage, M6 adds combat) | Turn-based actions |
| `rules/mod.rs` | 24 | Module declarations (grows as modules added) | Module exports |

**M2 source contribution**: ~1,200 lines (of 1,442 total in these files at M7)

**Test files:**

| File | Lines | Tests | Focus |
|------|-------|-------|-------|
| `tests/turn_structure.rs` | 237 | 6 | Full step order, phase transitions, 4-player rotation, 10-cycle stress, turn number, wraparound |
| `tests/priority.rs` | 151 | 7 | Active player first, APNAP order, all pass → advance, wrong player error, eliminated skip, no priority in Untap/Cleanup |
| `tests/turn_actions.rs` | 248 | 7 | Untap (active only), draw, first-draw skip, cleanup discard, cleanup clear damage, mana pool empty |
| `tests/extra_turns.rs` | 116 | 4 | LIFO, designated player, normal resumption, multiple stack |
| `tests/concede.rs` | 134 | 5 | Priority skip, turn skip, game continues, last player wins, eliminated can't act |
| `tests/turn_invariants.rs` | 104 | 4 | Proptest: state validity, holder validity, turn monotonicity, eliminated never gets priority |

**Test total**: 990 lines, 33 tests

### CR Sections Implemented

| CR Section | Implementation |
|------------|----------------|
| CR 103.8 | First player skips first draw (`turn_actions.rs:81`) |
| CR 104.3a | Concession (`engine.rs:269-327`) |
| CR 104.3b | Empty library → player loses (`turn_actions.rs:99-108`) |
| CR 116.3a | Active player gets priority first (`priority.rs:100-103`) |
| CR 116.3d | All pass → resolve stack or advance step (`engine.rs:143-180`) |
| CR 302.6 | Summoning sickness cleared at untap (`turn_actions.rs:52`) |
| CR 500.4 | Mana pools empty at step transitions (`turn_actions.rs:186-204`) |
| CR 502.2 | Untap active player's permanents (`turn_actions.rs:37-69`) |
| CR 504.1 | Draw step draws a card (`turn_actions.rs:77-86`) |
| CR 514.1 | Cleanup: discard to hand size (`turn_actions.rs:133-168`) |
| CR 514.2 | Cleanup: clear damage, end "until end of turn" effects (`turn_actions.rs:170-175`) |

### Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| MR-M2-01 | **HIGH** | priority.rs:54 | **`.expect()` in engine library code.** `next_priority_player(state, player).expect(...)` in `pass_priority`. Logically unreachable (the `all_passed` check on line 46 guarantees at least one player hasn't passed), but violates the "no unwrap/expect in engine" convention. If state becomes inconsistent (e.g., a bug in `active_players()` vs `players_passed`), this panics the engine instead of returning an error. **Fix:** `.ok_or(GameStateError::NoActivePlayers)?` | OPEN |
| MR-M2-02 | **HIGH** | turn_structure.rs:78 | **`.expect()` in engine library code.** `next_player_in_turn_order(state, turn.last_regular_active).expect("no active players remaining")` in `advance_turn`. Called when there are no extra turns. If all players are eliminated, this panics. Currently unreachable because `is_game_over` checks in `enter_step` prevent reaching `advance_turn` when ≤1 players remain, but the panic is a landmine for future refactors. **Fix:** change `advance_turn` to return `Result<...>` and propagate error. | OPEN |
| MR-M2-03 | **HIGH** | engine.rs:292-323 | **Concede while active player: step-advance then turn-advance.** When the conceding player is both the active player AND the priority holder, and all other players have already passed, the code calls `handle_all_passed` (line 305) which advances the step and executes turn-based actions, then ALSO calls `advance_turn` (line 316) because `active_player == conceding player`. This executes one step's turn-based actions for a player who has already conceded (e.g., `draw_for_turn` draws a card for the conceded player if we advance through Draw — see MR-M2-04). Should skip `handle_all_passed` when the conceding player is the active player and go straight to `advance_turn`. | OPEN |
| MR-M2-04 | **MEDIUM** | turn_actions.rs:90 | **`draw_card` has no concession/elimination guard.** `draw_card(state, player)` draws for any player regardless of `has_conceded` or `has_lost` status. Reachable via MR-M2-03: conceded active player's turn advances through the Draw step. The drawn card is pointless (player is eliminated) but modifies dead state. **Fix:** add `if p.has_lost \|\| p.has_conceded { return Ok(vec![]); }` guard. | OPEN |
| MR-M2-05 | **HIGH** | engine.rs:269-327 | **Concede doesn't clean up owned objects (CR 800.4a).** When a player leaves a multiplayer game, "all objects owned by that player leave the game and any effects which give that player control of any objects or players end." The concede handler marks `has_conceded = true` but doesn't: (1) exile/remove the player's permanents from the battlefield, (2) remove the player's spells from the stack, (3) end control-change effects. **Note:** may be intentionally deferred — this is a complex interaction that needs the full effect/replacement system. Mark as deferred to M9 (Commander rules integration). | DEFERRED → M9 |
| MR-M2-06 | **MEDIUM** | turn_actions.rs:158 | **`DiscardedToHandSize` event uses wrong ObjectId.** `object_id: new_id` where `new_id` is the NEW graveyard ObjectId (from `move_object_to_zone`). The event also has `zone_from: hand_zone`, but the object at `new_id` was never in the hand zone — it's the post-zone-change identity (CR 400.7). Should include both old hand ID and new graveyard ID, or at minimum use the old ID for `object_id`. | OPEN |
| MR-M2-07 | **LOW** | turn_invariants.rs | **Proptest lacks library cards.** `run_pass_sequence` builds a 4-player state with no library cards. Players hit empty-library loss within 1-2 turns, limiting the turn-cycle coverage of the proptest. Adding 10+ library cards per player would exercise more turn cycles. | OPEN |
| MR-M2-08 | **LOW** | concede.rs | **No test for concede when active player + all others passed.** The complex code path at `engine.rs:302-307` (active player concedes, no next priority player → `handle_all_passed` + `advance_turn`) has zero test coverage. This is the path where MR-M2-03 manifests. | OPEN |
| MR-M2-09 | **LOW** | turn_actions.rs:142 | **`unwrap_or(7)` for max_hand_size lookup.** If `state.players.get(&active)` returns None (active player missing from map), silently defaults to 7. Should be unreachable but a `.ok_or()` would be more defensive. Minor since the scenario requires a corrupted state. | OPEN |
| MR-M2-10 | **INFO** | engine.rs | **Loop-based step advancement (not recursion).** `enter_step` correctly uses a loop for no-priority steps (Untap, Cleanup), avoiding stack overflow on long chains of auto-advancing steps. Good design. | — |
| MR-M2-11 | **INFO** | priority.rs | **`pass_priority` doesn't mutate state.** The function builds a local `passed` set and returns `PriorityResult`; the caller (`handle_pass_priority`) applies the state change. Clean separation of query vs mutation. | — |
| MR-M2-12 | **INFO** | turn_structure.rs | **Extra turns correctly use LIFO with `pop_back`.** `advance_turn` pops from the back of `extra_turns` (most recently added goes first per CR 614.10), and `last_regular_active` correctly tracks normal order resumption. 4 tests verify this behavior. | — |
| MR-M2-13 | **INFO** | turn_actions.rs:52 | **Summoning sickness cleared at untap.** CR 302.6 implementation: `has_summoning_sickness = false` for all active player's permanents during untap. Correct. | — |

### Test Coverage Assessment

| M2 Behavior | Coverage | Notes |
|-------------|----------|-------|
| Step ordering (full turn) | Good (6 tests) | Full step order, phase mapping, wraparound, 10-cycle stress |
| Priority APNAP | Good (7 tests) | Active first, order rotation, wrong player error, eliminated skip |
| Untap step | Good (2 tests) | Active player only, doesn't affect other players |
| Draw step | Good (2 tests) | Normal draw, first-draw skip (CR 103.8) |
| Cleanup | Good (2 tests) | Discard to hand size, clear damage |
| Mana pool empty | Good (1 test) | Verifies emptying between steps |
| Extra turns | Good (4 tests) | LIFO, designated player, resumption, multi-stack |
| Concession | Adequate (5 tests) | Priority skip, turn skip, game continues, last wins, can't re-act |
| Concede while active + all passed | **Missing** | MR-M2-08: the complex code path has no coverage |
| Empty library loss | Thin (indirect) | Proptest may trigger it but no dedicated test |
| Proptest invariants | Good (4 tests) | State validity, holder validity, monotonicity, eliminated check |

### Notes

- M2 files are the backbone of the engine — `process_command`, `enter_step`, and the turn FSM
  are called by every subsequent milestone. The two `.expect()` calls (MR-M2-01, MR-M2-02) are
  the most important fixes since any future state inconsistency would crash the engine.
- MR-M2-05 (CR 800.4a cleanup on concede) is a significant gap for multiplayer correctness
  but requires M8/M9 infrastructure (replacement effects, zone-change cleanup). Tracked as
  deferred to M9.
- The concede handler (MR-M2-03) is the most complex code path in M2 and the least tested
  (MR-M2-08). The overlapping step-advance + turn-advance logic should be simplified.
- `draw_card` (MR-M2-04) should guard against eliminated players, not just for correctness
  but to prevent confusing events in the history log.

---

## M3: Stack, Spells & Abilities

**Review Status**: REVIEWED (2026-02-22)

### Files Introduced

**Source files — state:**

| File | Lines | Purpose |
|------|-------|---------|
| `state/stack.rs` | 64 | StackObject, StackObjectKind (Spell, ActivatedAbility, TriggeredAbility) |
| `state/hash.rs` | 1,223 | HashInto trait, blake3 field-by-field hashing, `public_state_hash`/`private_state_hash` |
| `state/targeting.rs` | 36 | Target (Player/Object), SpellTarget (with zone snapshot at cast) |

**Source files — rules:**

| File | Lines | Purpose |
|------|-------|---------|
| `rules/mana.rs` | 112 | CR 605 mana ability handler (tap-activated only) |
| `rules/lands.rs` | 107 | CR 305.1 land play handler (sorcery speed, one per turn) |
| `rules/casting.rs` | 302 | CR 601 spell casting, target validation, mana cost payment (`can_pay_cost`/`pay_cost`) |
| `rules/resolution.rs` | 355 | CR 608 stack resolution (LIFO), fizzle rule, `counter_stack_object` |
| `rules/abilities.rs` | 448 | CR 602-603 activated/triggered abilities, APNAP ordering, intervening-if |

**Source files — testing:**

| File | Lines | Purpose |
|------|-------|---------|
| `testing/script_schema.rs` | 325 | GameScript JSON schema types (GameScript, ScriptAction with 9 variants) |
| `testing/mod.rs` | 9 | Module exports |

**Source total**: 2,981 lines

**Test files:**

| File | Lines | Tests | Focus |
|------|-------|-------|-------|
| `tests/mana_and_lands.rs` | 660 | 19 | PlayLand, TapForMana, summoning sickness, land play limits, error cases |
| `tests/casting.rs` | 550 | 12 | CastSpell, sorcery/instant speed, Flash, LIFO, priority reset |
| `tests/resolution.rs` | 626 | 10 | Resolve to graveyard/battlefield, LIFO, countering, Flash creature ETB |
| `tests/targeting.rs` | 742 | 13 | Target validation, fizzle (all/partial), mana cost payment, hexproof/shroud |
| `tests/abilities.rs` | 852 | 15 | Activated abilities, triggered (ETB/SpellCast/Tap), APNAP, intervening-if |
| `tests/state_hashing.rs` | 477 | 19 | Determinism, sensitivity (7), public/private partition (4), dual-instance proptest (3) |
| `tests/script_schema.rs` | 128 | 3 | JSON round-trip, type tags, enum serialization |

**Test total**: 4,035 lines, 91 tests

### CR Sections Implemented

| CR Section | Implementation |
|------------|----------------|
| CR 305.1 | Land plays: sorcery speed, one per turn, validates card is land and in hand (`lands.rs:25-107`) |
| CR 400.7 | Zone-change identity for stack objects — CastSpell creates new ObjectId (`casting.rs:123`) |
| CR 601 | Spell casting: validates casting speed, moves to Stack zone, pushes StackObject (`casting.rs:41-152`) |
| CR 601.2c | Target selection at cast time: validates existence, snapshots zones (`casting.rs:162-228`) |
| CR 601.2f-h | Mana cost payment: colored strict, colorless strict (CR 106.1), generic from any remainder (`casting.rs:235-302`) |
| CR 601.2i | After casting, active player receives priority — not necessarily the caster (`casting.rs:139-140`) |
| CR 602 | Activated abilities: validates priority, battlefield, controller, pays tap/mana cost (`abilities.rs:46-217`) |
| CR 603.2 | Trigger checking: scans battlefield permanents per event (`abilities.rs:233-351`) |
| CR 603.3 | Trigger flushing: APNAP-sorted push to stack before priority grants (`abilities.rs:372-422`) |
| CR 603.4 | Intervening-if: checked at trigger time AND resolution time (`abilities.rs:440-448`, `resolution.rs:200-225`) |
| CR 605 | Mana abilities: special action, does not use stack, player retains priority (`mana.rs:25-112`) |
| CR 605.5 | Mana abilities do not reset `players_passed` (`mana.rs:108-109`) |
| CR 608.1 | Stack resolution: LIFO, top of stack resolves when all players pass (`resolution.rs:35-278`) |
| CR 608.2b | Fizzle rule: all targets illegal → SpellFizzled, card to graveyard (`resolution.rs:50-78`) |
| CR 608.2n | Instant/sorcery → owner's graveyard after resolution (`resolution.rs:157-166`) |
| CR 608.3a | Permanent spell → battlefield under controller's control (`resolution.rs:135-156`) |
| CR 701.5 | Countering a spell: remove from stack, card to graveyard (`resolution.rs:312-355`) |
| CR 702.11a | Hexproof: can't be targeted by opponents (`casting.rs:196-216`, `abilities.rs:146-168`) |
| CR 702.18a | Shroud: can't be targeted by any spell or ability (`casting.rs:196-216`, `abilities.rs:146-168`) |

### Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| MR-M3-01 | **HIGH** | resolution.rs:79-80,122-129 | **Partial fizzle: targets not filtered.** When some (but not all) targets are illegal at resolution time, `resolve_top_of_stack` passes ALL original targets to `EffectContext` via `stack_obj.targets.clone()` (line 126), including the illegal ones. Effects execute against illegal targets instead of skipping them. Comment on line 80 says "Illegal targets will be unaffected when effects are implemented (M7+)" but M7 is now complete and this filtering was not added. **Fix:** filter `stack_obj.targets` to only legal targets before passing to `EffectContext`. | OPEN |
| MR-M3-03 | **HIGH** | hash.rs:349-365, game_object.rs:220 | **GameObject hash omits `has_summoning_sickness`.** The `HashInto` impl for `GameObject` (hash.rs:349-365) hashes 14 fields but skips `has_summoning_sickness: bool` (game_object.rs:220). Two game states differing only in summoning sickness produce identical public hashes, breaking the distributed verification model. **Fix:** add `self.has_summoning_sickness.hash_into(hasher);` to the GameObject impl. | OPEN |
| MR-M3-04 | **HIGH** | abilities.rs:148 | **Non-existent object target silently accepted in ability activation.** In `handle_activate_ability`, hexproof/shroud target validation (lines 146-171) uses `if let Some(obj) = state.objects.get(id)` which silently skips non-existent objects. A `Target::Object` with a bogus ObjectId passes validation — no existence check, no zone snapshot. Compare with `casting.rs:187-192` which explicitly returns `ObjectNotFound` for missing targets. **Fix:** return `GameStateError::ObjectNotFound(*id)` when object doesn't exist. | OPEN |
| MR-M3-02 | **MEDIUM** | casting.rs:108 | **ManaCostPaid not emitted for {0} cost spells.** The `if cost.mana_value() > 0` guard (line 108) skips the entire payment block for zero-cost spells, including the `ManaCostPaid` event. Zero-cost spells like Ornithopter ({0}) and Memnite ({0}) never emit `ManaCostPaid`. Matters if triggers key on `ManaCostPaid` (e.g., "whenever a player casts a spell" that checks mana paid). **Fix:** emit `ManaCostPaid` even when cost is {0} (no pool deduction needed, just the event). | OPEN |
| MR-M3-05 | **MEDIUM** | hash.rs:585-589, game_object.rs:89 | **ActivatedAbility hash omits `effect` field.** The `HashInto` impl (hash.rs:585-589) only hashes `cost` and `description`, but `ActivatedAbility` also has `effect: Option<Effect>` (game_object.rs:89, added in M7). In the distributed verification model, if one peer loaded a different card definition with a different effect, the hash wouldn't detect the mismatch. **Cross-milestone:** M3 wrote the hash impl; M7 added the field without updating the hash. **Fix:** add `self.effect.hash_into(hasher);` (requires `HashInto` impl for `Effect`). | OPEN |
| MR-M3-06 | **MEDIUM** | hash.rs:658-663, game_object.rs:140 | **TriggeredAbilityDef hash omits `effect` field.** Same pattern as MR-M3-05: `HashInto` for `TriggeredAbilityDef` hashes `trigger_on`, `intervening_if`, `description` but not `effect: Option<Effect>` (game_object.rs:140). Same cross-milestone gap as MR-M3-05. **Fix:** add `self.effect.hash_into(hasher);`. | OPEN |
| MR-M3-07 | **MEDIUM** | abilities.rs:146-171, casting.rs:196-216 | **Hexproof/shroud target validation duplicated.** `handle_activate_ability` (abilities.rs:146-171) and `validate_targets` (casting.rs:196-216) both implement hexproof/shroud checks with nearly identical code. The abilities version is weaker (silently skips non-existent objects per MR-M3-04). **Fix:** extract a shared `validate_target_protection(state, target, controller)` helper used by both paths. | OPEN |
| MR-M3-08 | **MEDIUM** | targeting.rs:591-594 | **`matches!` used as bare statement — silent no-op assertion.** In `test_601_insufficient_mana_fails`, line 591-594: `matches!(result.unwrap_err(), mtg_engine::GameStateError::InsufficientMana);` is a bare expression, not wrapped in `assert!()`. The `matches!` macro returns a `bool` that is silently discarded — the test passes regardless of the error variant. **Fix:** wrap in `assert!(matches!(...));`. | OPEN |
| MR-M3-09 | **LOW** | hash.rs:955-965 | **LegendaryRuleApplied event hash missing length prefix.** The `for (old_id, new_id) in put_to_graveyard` loop (lines 961-964) hashes pairs without prefixing the vec length. The generic `Vec<T>` HashInto impl uses a length prefix for unambiguous framing. Inconsistent pattern — low collision risk in practice since the discriminant byte and subsequent events provide implicit framing. **Fix:** add `(put_to_graveyard.len() as u64).hash_into(hasher);` before the loop. | OPEN |
| MR-M3-10 | **LOW** | targeting.rs:224-269 | **Incomplete test discards results.** `test_608_2b_fizzle_player_target_concedes` constructs a fizzle scenario but the test body ends with `let _ = (final_state, events);` (line 268) and a comment "will redo below." The test has no assertions. The replacement test (`test_608_2b_fizzle_all_targets_illegal` at line 275) covers the fizzle case properly, making this test dead code. **Fix:** delete the incomplete test or complete it for the concede-specific path. | OPEN |
| MR-M3-11 | **LOW** | abilities.rs:435 | **`apnap_order` silently defaults position with `unwrap_or(0)`.** If the active player is not found in `turn_order`, the function starts from index 0 instead of returning an error. Requires a corrupted state to trigger — `active_player` is always in `turn_order` under normal operation. Minor defensiveness gap. | OPEN |
| MR-M3-12 | **LOW** | lands.rs:83-87 | **`NotController` error used for ownership check.** Line 83 checks `card_obj.owner != player` (ownership) but returns `NotController` error. Cards in hand are always owned, not "controlled" in the MTG sense. Misleading error name for debugging. **Fix:** add a `NotOwner` error variant or use `InvalidCommand("card is not owned by player")`. | OPEN |
| MR-M3-13 | **INFO** | casting.rs:235-302 | **Mana payment design correct.** Colored mana strict (W/U/B/R/G exact match), colorless `{C}` strict (CR 106.1 — must use pool.colorless), generic `{N}` from any remaining. Payment order for generic (colorless→green→red→black→blue→white) is arbitrary but deterministic. Well-documented. | — |
| MR-M3-14 | **INFO** | stack.rs | **Clean stack module design.** `StackObject` is well-typed with `StackObjectKind` covering Spell, ActivatedAbility, and TriggeredAbility. Correct use of `im::Vector` for LIFO. | — |
| MR-M3-15 | **INFO** | hash.rs | **State hashing framework solid.** blake3 for speed, explicit field-by-field hashing (no derive magic), clear public/private separation, cross-platform deterministic. 19 tests including dual-instance proptest. Good foundation for distributed verification (M10). | — |
| MR-M3-16 | **INFO** | — | **Well-structured test suites with CR citations.** All 7 test files consistently cite CR sections in test doc-comments. Good use of GameStateBuilder for test state construction. One assertion focus per test. | — |
| MR-M3-17 | **INFO** | state_hashing.rs | **Dual-instance proptest strong.** Three proptest tests (state_hashing.rs) run identical command sequences on cloned states and verify hash equality. Good coverage of the determinism invariant. | — |
| MR-M3-18 | **INFO** | script_schema.rs | **Script schema well-designed and extensible.** `GameScript` with `ScriptAction` (9 variants using `#[serde(tag = "type", rename_all = "snake_case")]`) supports future expansion. 3 round-trip tests confirm serialization fidelity. | — |

### Test Coverage Assessment

| M3 Behavior | Coverage | Notes |
|-------------|----------|-------|
| Land plays (CR 305.1) | Excellent (7 tests) | Priority, active player, main phase, stack, land count, card in hand, card is land |
| Mana abilities (CR 605) | Good (12 tests) | Tap for mana, summoning sickness, priority, ability index, already tapped, not controlled |
| Spell casting (CR 601) | Good (12 tests) | Sorcery/instant speed, Flash, LIFO, priority reset, card in hand, not a land |
| Target validation (CR 601.2c) | Good (7 tests) | Hexproof, shroud, zone snapshot, player target, existence |
| Mana cost payment (CR 601.2f-h) | Good (6 tests) | Colored, generic, insufficient, exact, colorless {C}, zero cost |
| Stack resolution (CR 608) | Good (10 tests) | Graveyard destination, battlefield ETB, LIFO, controller set, countering |
| Fizzle rule (CR 608.2b) | Adequate (3 tests) | All-targets fizzle, partial fizzle, but MR-M3-10 incomplete concede test |
| Activated abilities (CR 602) | Good (5 tests) | Tap cost, mana cost, priority, error cases |
| Triggered abilities (CR 603) | Good (7 tests) | ETB, SpellCast, SelfBecomesTapped, APNAP ordering |
| Intervening-if (CR 603.4) | Good (3 tests) | Trigger-time check, resolution-time check, false-at-resolution |
| State hashing | Excellent (19 tests) | Determinism, sensitivity, partitioning, dual-instance proptest |
| Script schema | Adequate (3 tests) | Round-trip only; no invalid-input tests |
| Mana abilities don't reset priority (CR 605.5) | Good (1 test) | Explicit test that `players_passed` unchanged |
| CastSpell resets priority to active (CR 601.2i) | Good (1 test) | Non-active player casts, priority goes to active |

### Notes

- M3 is the largest milestone by file count (10 source files, 7 test files) and introduces the
  most CR sections (19). The stack, casting, and resolution modules form the backbone of all
  future gameplay.
- The hash framework (1,223 lines) is the largest single file. It was written in M3 but accumulates
  fields from later milestones. Two cross-milestone gaps (MR-M3-05, MR-M3-06) arose when M7 added
  `effect` fields to `ActivatedAbility` and `TriggeredAbilityDef` without updating the hash impls.
  The `has_summoning_sickness` omission (MR-M3-03) is a pure M3 gap.
- The partial fizzle issue (MR-M3-01) is the most impactful finding: effects currently execute
  against illegal targets. This was intentionally deferred ("M7+") when M3 was written, but M7
  shipped without the fix. Should be addressed early in M8 or as a hotfix.
- The duplicated hexproof/shroud validation (MR-M3-07) between `casting.rs` and `abilities.rs`
  creates maintenance risk — the abilities version is already weaker (MR-M3-04). Extract a shared
  helper.
- The silent no-op assertion (MR-M3-08) in `test_601_insufficient_mana_fails` means that test
  doesn't actually verify the error variant. Easy fix.
- `mana.rs` and `stack.rs` are exemplary: clean, focused, well-documented, no issues found.

---

## M4: State-Based Actions

**Review Status**: REVIEWED (2026-02-22)

### Files Introduced

| File | Lines | Purpose |
|------|-------|---------|
| `rules/sba.rs` | 587 | `check_and_apply_sbas` fixed-point loop, all CR 704.5 checks |

**Source total**: 587 lines

**Test files:**

| File | Lines | Tests | Focus |
|------|-------|-------|-------|
| `tests/sba.rs` | 756 | 28 | All SBA checks, batch behavior, convergence, event ordering |
| `tests/keywords.rs` (M7, partial) | 677 | 2 (relevant) | Indestructible + SBA interaction (704.5f vs 704.5g) |

**Test total**: 28 dedicated + 2 cross-milestone = 30 SBA-relevant tests

### CR Sections Implemented

| CR Section | Implementation |
|------------|----------------|
| CR 704.3 | Fixed-point SBA loop: check all, apply simultaneously, repeat until none fire (`sba.rs:44-54`) |
| CR 704.5a | Player at 0 or less life loses (`sba.rs:95-103`) |
| CR 704.5c | Player with 10+ poison counters loses (`sba.rs:107-116`) |
| CR 704.5d | Token in non-battlefield zone ceases to exist (`sba.rs:147-171`) |
| CR 704.5f | Creature with toughness ≤ 0 → owner's graveyard; indestructible does NOT prevent (`sba.rs:211-213`) |
| CR 704.5g | Creature with lethal damage (damage ≥ toughness > 0) destroyed; indestructible prevents (`sba.rs:217-220`) |
| CR 704.5h | Creature dealt deathtouch damage destroyed; indestructible prevents (`sba.rs:224-226`) |
| CR 704.5i | Planeswalker with 0 loyalty → owner's graveyard (`sba.rs:260-301`) |
| CR 704.5j | Legendary rule: 2+ legendaries same name/controller → keep newest ObjectId (`sba.rs:311-368`) |
| CR 704.5m | Aura attached to illegal/non-existent object → owner's graveyard (`sba.rs:378-425`) |
| CR 704.5n | Equipment/Fortification illegally attached → unattach, stays on battlefield (`sba.rs:436-499`) |
| CR 704.5q | +1/+1 and -1/-1 counter pair annihilation (`sba.rs:506-573`) |
| CR 704.5u | Commander: 21+ combat damage from one commander → player loses (`sba.rs:118-133`) |

### Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| MR-M4-01 | **HIGH** | sba.rs:120 | **`.unwrap()` in `check_player_sbas`.** `state.players.get(&id).unwrap()` for the commander damage check. The player IS guaranteed to exist (iterated from `state.players.keys()` on line 85, checked with `state.players.get(&id)` on line 87), but violates the "no unwrap/expect in engine" convention. If a future refactor between lines 87-120 invalidates the guarantee, this panics. **Fix:** `if let Some(p) = state.players.get(&id) { ... }` with early-continue on None. | OPEN |
| MR-M4-02 | **HIGH** | sba.rs:189-226 | **SBAs don't use `calculate_characteristics` for P/T or keyword checks.** `check_creature_sbas` reads raw `obj.characteristics.toughness` and `obj.characteristics.keywords` instead of the layer-calculated values. A creature with base 2/2 under a continuous -2/-2 effect has `raw_toughness == 2` but `effective_toughness == 0` — the SBA would miss it. Same for indestructible: Humility removes keywords from the raw characteristics level, but the SBA reads from raw. The layer system (M5) provides `calculate_characteristics()` which returns the correct values. **Cross-milestone:** M4 code, M5+ fix. | DEFERRED → M5 review |
| MR-M4-03 | **HIGH** | sba.rs:340 | **`.unwrap()` in `check_legendary_rule`.** `ids.last().unwrap()` — safe because `ids.len() < 2` guard on line 334 guarantees ≥2 elements, but violates convention. **Fix:** `let Some(&kept) = ids.last() else { continue; };` | OPEN |
| MR-M4-04 | **MEDIUM** | sba.rs:217 | **`u32 as i32` cast in lethal damage comparison.** `obj.damage_marked as i32 >= toughness` — `damage_marked` is `u32`. If damage exceeds `i32::MAX` (2,147,483,647), the cast wraps to negative, making the comparison incorrect. Practically unreachable in normal gameplay but formally unsound. **Fix:** convert toughness to u32 instead: `toughness > 0 && obj.damage_marked >= toughness as u32`. | OPEN |
| MR-M4-05 | **MEDIUM** | sba.rs:280 | **`u32 as i32` cast in planeswalker loyalty.** `loyalty_counter.map(|c| c as i32)` — same overflow pattern as MR-M4-04. If a planeswalker somehow had >2B loyalty counters, the cast wraps. **Fix:** compare in u32 space: if counter > 0, alive; if counter == 0, dead. | OPEN |
| MR-M4-06 | **MEDIUM** | sba.rs (missing) | **CR 704.5b not implemented as SBA.** "If a player attempted to draw from an empty library since the last SBA check, that player loses." Currently handled as immediate loss in `turn_actions.rs:99-108` (CR 104.3b). The timing difference matters if replacement effects (M8) could replace the draw — the current approach loses the player before replacements can apply. Acceptable for M4; should be revisited when replacement effects are implemented in M8. | DEFERRED → M8 |
| MR-M4-07 | **LOW** | sba.rs (missing) | **CR 704.5e (spell/card copies) not implemented.** "If a copy of a spell is in a zone other than the stack, it ceases to exist." No `is_copy` field on `GameObject`. Copy effects are M8+ territory — expected omission. | DEFERRED → M8+ |
| MR-M4-08 | **LOW** | sba.rs (missing) | **CR 704.5k (world rule) not implemented.** "If two or more permanents have the supertype world..." World is an extremely rare supertype (handful of old cards, none Commander staples). Reasonable to defer indefinitely. | DEFERRED |
| MR-M4-09 | **LOW** | sba.rs:329 | **`String::clone()` allocation in legendary rule hot path.** `obj.characteristics.name.clone()` for every legendary permanent on every SBA pass. Creates many small allocations if many legends are on the battlefield. Minor performance concern — could use `&str` references in the grouping map. | OPEN |
| MR-M4-10 | **LOW** | sba.rs:391,449,453 | **`SubType("...".to_string())` allocates on every comparison.** Aura/Equipment/Fortification checks create new `String` allocations on every object iteration. Should use a static or pre-allocated `SubType`. Same pattern in `check_aura_sbas` and `check_equipment_sbas`. | OPEN |
| MR-M4-11 | **LOW** | sba.rs:281 | **`unwrap_or(1)` default for missing planeswalker loyalty.** If a planeswalker has no Loyalty counter AND no `characteristics.loyalty`, effective loyalty defaults to 1 (survives). Correct for well-constructed states (planeswalkers always have `characteristics.loyalty`), but silently hides construction bugs. A `unwrap_or(0)` or logging would catch incorrectly built test states. | OPEN |
| MR-M4-12 | **LOW** | tests/sba.rs | **Test gap: no test for planeswalker with Loyalty counters (vs characteristics.loyalty).** All planeswalker tests use `ObjectSpec::planeswalker(p, name, loyalty)` which sets `characteristics.loyalty`. No test verifies the `CounterType::Loyalty` counter path (sba.rs:278-279) which is the runtime path for planeswalkers that have used loyalty abilities. | OPEN |
| MR-M4-13 | **LOW** | tests/sba.rs | **Test gap: no test for aura whose target left the battlefield.** The only aura test (704.5m) tests an unattached aura (`attached_to == None`). No test for the `target.zone != Battlefield` branch (sba.rs:400-404) where an aura is attached to an object that moved zones. | OPEN |
| MR-M4-14 | **LOW** | tests/sba.rs | **Test gap: no test for 3+ legendary copies.** Only tests 2 copies of a legendary. No test verifying that with 3+ copies, all but the newest are removed simultaneously. The grouping logic (sba.rs:333-340) should handle it, but it's unverified. | OPEN |
| MR-M4-15 | **INFO** | sba.rs:204-226 | **Correct indestructible handling for 704.5f vs 704.5g/h.** Indestructible correctly does NOT prevent 704.5f (zero toughness = "put into graveyard", not "destroy") but DOES prevent 704.5g (lethal damage) and 704.5h (deathtouch damage). Matches CR 702.12a precisely. Verified by 2 tests in keywords.rs. | — |
| MR-M4-16 | **INFO** | sba.rs:44-54 | **Fixed-point loop correct and convergent.** Each pass removes dying objects from state, so subsequent passes find fewer objects. Convergence is guaranteed because SBAs only remove/modify — they never create new SBA-triggering conditions. Two tests verify this (convergence + no-infinite-loop). | — |
| MR-M4-17 | **INFO** | engine.rs:202-219 | **SBA integration with engine correct.** `enter_step` calls `check_and_apply_sbas` before granting priority (CR 704.3), then checks triggers, flushes them to stack, then grants priority. `resolution.rs` also calls SBAs after resolving spells (lines 68, 260, 345). Correct sequence per CR 704.3. | — |
| MR-M4-18 | **INFO** | sba.rs:436-499 | **Equipment unattach vs aura destroy: correct distinction.** Equipment/Fortification illegally attached → unattach and stay on battlefield (CR 704.5n). Aura illegally attached → owner's graveyard (CR 704.5m). Both correctly implemented with different event types and state mutations. | — |
| MR-M4-19 | **INFO** | sba.rs:82-136 | **Single loss event per player per pass: correct.** `check_player_sbas` uses `continue` after emitting a loss event (lines 103, 115), ensuring only one `PlayerLost` event per player per SBA pass. A player at 0 life with 10 poison gets `LifeTotal` reason (first in CR order). This is correct — the player can only lose once. | — |

### Test Coverage Assessment

| M4 Behavior | Coverage | Notes |
|-------------|----------|-------|
| Life total ≤ 0 (CR 704.5a) | Good (4 tests) | Zero, negative, 1 survives, multiple simultaneous |
| Poison counters (CR 704.5c) | Good (2 tests) | 10 loses, 9 survives |
| Token in wrong zone (CR 704.5d) | Good (2 tests) | Graveyard ceases, battlefield stays |
| Toughness ≤ 0 (CR 704.5f) | Good (3 tests) | Zero, negative, 1 survives |
| Lethal damage (CR 704.5g) | Good (2 tests) | Lethal destroys, sub-lethal survives |
| Deathtouch damage (CR 704.5h) | Adequate (1 test) | Deathtouch + 1 damage destroys |
| Planeswalker loyalty (CR 704.5i) | Good (2 tests) | 0 dies, 3 survives |
| Legendary rule (CR 704.5j) | Good (3 tests) | Duplicate, different names, different controllers |
| Aura illegal (CR 704.5m) | Thin (1 test) | Unattached only; no "target left battlefield" test (MR-M4-13) |
| Equipment illegal (CR 704.5n) | Adequate (1 test) | On non-creature unattaches |
| Counter annihilation (CR 704.5q) | Good (2 tests) | Equal pairs, unequal partial |
| Commander damage (CR 704.5u) | Good (2 tests) | 21 loses, 20 survives |
| Indestructible + SBA | Good (2 tests in keywords.rs) | Survives lethal, dies to 0 toughness |
| Fixed-point convergence | Good (2 tests) | Only applicable fire, no infinite loop |
| SBA fires before priority | Good (1 test) | CreatureDied before PriorityGiven |
| Planeswalker with Loyalty counters | **Missing** (MR-M4-12) | Only tests characteristics.loyalty path |
| Aura target left battlefield | **Missing** (MR-M4-13) | Only tests unattached aura |
| 3+ legendary copies | **Missing** (MR-M4-14) | Only tests 2 copies |

### Notes

- M4 is a focused, well-structured milestone — 587 source lines implementing 12 CR 704.5 sub-rules
  with a clean fixed-point loop. The code quality is high: each SBA is a separate function, events
  are well-typed, and the integration with the engine's priority system is correct.
- The two `.unwrap()` calls (MR-M4-01, MR-M4-03) are provably safe in current code but violate the
  project's typed-error convention. Both are easy fixes.
- MR-M4-02 (raw characteristics instead of layer-calculated) is the most significant finding. It
  means any continuous effect modifying P/T (e.g., -X/-X from Tragic Slip) or granting/removing
  indestructible (e.g., Humility) will not be reflected in SBA checks. This is a cross-milestone
  issue requiring the M5 layer system.
- MR-M4-06 (704.5b as immediate loss vs SBA) is a subtle rules deviation that becomes relevant
  when replacement effects (M8) can replace draws. For now, the immediate-loss approach is adequate
  and matches most other MTG engines.
- The `u32 as i32` casts (MR-M4-04, MR-M4-05) are the same pattern seen in combat damage
  (MR-M6-05 stub). Should be fixed systematically across the codebase.
- CR 704.5b (empty library draw), 704.5e (copies), 704.5k (world rule) are intentionally omitted.
  704.5b is handled differently (immediate loss); 704.5e and 704.5k are deferred to later milestones
  or indefinitely (world rule is irrelevant for Commander).

---

## M5: The Layer System

**Review Status**: REVIEWED (2026-02-22)

### Files Introduced

**Source files:**

| File | Lines | Purpose |
|------|-------|---------|
| `state/continuous_effect.rs` | 208 | EffectId, EffectLayer (10 sublayers), EffectDuration (3), EffectFilter (10), LayerModification (21), ContinuousEffect |
| `rules/layers.rs` | 498 | `calculate_characteristics`, `is_effect_active`, `effect_applies_to`, `apply_layer_modification`, `resolve_layer_order`, `toposort_with_timestamp_fallback`, `depends_on`, `expire_end_of_turn_effects` |

**Additions to existing files:**

| File | Lines Added | Purpose |
|------|-------------|---------|
| `state/hash.rs` | ~136 (lines 419-554) | `HashInto` impls for all 6 M5 types |
| `state/mod.rs` | 1 | `continuous_effects: Vector<ContinuousEffect>` field on GameState |
| `state/builder.rs` | ~8 | `add_continuous_effect()` builder method |
| `rules/mod.rs` | 1 | `pub use layers::calculate_characteristics;` |
| `lib.rs` | 1 | Re-export `calculate_characteristics` |

**Source total**: ~706 new lines + ~147 additions to existing files

**Test files:**

| File | Lines | Tests | Focus |
|------|-------|-------|-------|
| `tests/layers.rs` | 1,393 | 28 | Layer ordering, timestamps, dependencies, CDAs, counters, duration tracking, Humility+Opalescence, Blood Moon+Urborg |

### CR Sections Implemented

| CR Section | Implementation |
|------------|----------------|
| CR 613.1 | Layer order: Copy(1) → Control(2) → Text(3) → Type(4) → Color(5) → Ability(6) → P/T 7a-d (`layers.rs:48-59`) |
| CR 613.1d | Type-changing: SetTypeLine, AddCardTypes, AddSubtypes, LoseAllSubtypes (`layers.rs:238-262`) |
| CR 613.1e | Color-changing: SetColors, AddColors, BecomeColorless (`layers.rs:265-277`) |
| CR 613.1f | Ability add/remove: AddKeyword(s), RemoveAllAbilities, RemoveKeyword (`layers.rs:280-303`) |
| CR 613.3 | CDAs applied before non-CDAs within each layer (`layers.rs:367`) |
| CR 613.4a | CDA P/T: SetPtViaCda, SetPtToManaValue (`layers.rs:306-315`) |
| CR 613.4b | P/T-setting: SetPowerToughness (`layers.rs:318-321`) |
| CR 613.4c | P/T-modifying: ModifyPower/Toughness/Both, +1/+1 and -1/-1 counters (`layers.rs:93-115, 324-343`) |
| CR 613.4d | P/T-switching: SwitchPowerToughness (`layers.rs:346-351`) |
| CR 613.7 | Timestamp ordering within layers (`layers.rs:391`) |
| CR 613.8 | Dependency ordering — SetTypeLine depends on AddSubtypes/AddCardTypes (`layers.rs:453-484`) |
| CR 613.8b | Circular dependencies fall back to timestamp order (`layers.rs:432-439`) |
| CR 611.2b | WhileSourceOnBattlefield duration (`layers.rs:129-137`) |
| CR 514.2 | UntilEndOfTurn expiry at cleanup (`layers.rs:489-497`) |

**Placeholder layers (deferred):**

| CR Section | Status |
|------------|--------|
| CR 613.1a / CR 707 | Layer 1 (Copy): `CopyOf` variant defined, TODO in `apply_layer_modification` → deferred to M7 |
| CR 613.1b | Layer 2 (Control): `SetController` variant defined, controller lives on `GameObject` not `Characteristics` → handled outside `calculate_characteristics` |
| CR 613.1c | Layer 3 (Text): `EffectLayer::Text` defined, no `LayerModification` variant → no text-changing effects in card pool |

### Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| MR-M5-01 | **HIGH** | layers.rs:95 | **`.expect()` in engine library code.** `state.objects.get(&object_id).expect("object exists")` in the counter P/T block (layer 7c). Provably safe — the object was retrieved at line 36 and state is `&` (immutable), so it cannot have been removed. However, violates the "no unwrap/expect in engine" convention. **Fix:** wrap counter logic in `if let Some(obj_ref) = state.objects.get(&object_id) { ... }`. | OPEN |
| MR-M5-02 | **HIGH** | sba.rs:193,200,204-207,469,472,585 | **SBAs use raw characteristics, not `calculate_characteristics()`.** Forwarded from MR-M4-02, now validated. Seven call sites in `sba.rs` read `obj.characteristics.{toughness,card_types,keywords}` directly. Impact: (1) creature under -X/-X continuous effect: raw toughness unmodified, SBA won't kill it; (2) Humility removes Indestructible keyword: raw still has it, SBA incorrectly skips lethal-damage destroy; (3) Opalescence makes enchantment a creature: raw card_types lacks Creature, equipment attachment SBA misses it. **Fix:** call `calculate_characteristics` for each battlefield object at the start of each SBA pass. | OPEN |
| MR-M5-03 | **MEDIUM** | layers.rs:435 | **`ptr::eq` for effect identity in cycle fallback.** Uses `std::ptr::eq(*e, *effect)` to check whether an effect was already emitted during Kahn's cycle recovery. Correct in current usage — all references point into the same `im::Vector`, so each element has a unique address. Fragile: if refactored to use cloned values or indices, `ptr::eq` silently breaks. **Fix:** compare by `EffectId`: `result.iter().any(\|e\| e.id == effect.id)`. | OPEN |
| MR-M5-04 | **MEDIUM** | continuous_effect.rs:46-57 | **`EffectDuration` missing general condition-based durations.** Only `WhileSourceOnBattlefield`, `UntilEndOfTurn`, `Indefinite`. Missing: (1) `UntilEndOfNextTurn`/`UntilYourNextTurn` — common in Commander ("until your next turn"); (2) general `AsLongAs(Condition)` — "as long as you control a creature." Roadmap M5 deliverable says "as long as" is covered; only the most common case (`WhileSourceOnBattlefield`) is implemented. | DEFERRED → M8+ |
| MR-M5-05 | **MEDIUM** | layers.rs:184-192 | **`AllPermanents` filter over-checks card types.** Checks `card_types.contains(Creature\|Artifact\|Enchantment\|Land\|Planeswalker\|Battle)` instead of just `obj_zone == Battlefield`. Per CR 110.4, any card on the battlefield is a permanent. If layer 4 removes all card types, `AllPermanents` incorrectly excludes the object. Extremely rare but technically wrong. **Fix:** `obj_zone == ZoneId::Battlefield`. | OPEN |
| MR-M5-06 | **LOW** | layers.rs:417 | **`ready.remove(0)` is O(n) in Kahn's algorithm.** `Vec::remove(0)` shifts all elements on every iteration → O(n²) total. Should use `VecDeque::pop_front()` for O(1). Negligible: n ≤ 20 effects per layer. | OPEN |
| MR-M5-07 | **LOW** | continuous_effect.rs | **Missing `AddSupertypes`/`RemoveSupertypes` layer 4 variants.** `SetTypeLine` can set supertypes, but no way to add individual supertypes (e.g., "becomes legendary" in addition to existing types). Uncommon but exists in Commander card pool. | DEFERRED → M8+ |
| MR-M5-08 | **LOW** | tests/layers.rs:410-457 | **CDA priority test doesn't exercise same-layer partition.** `test_613_layer7a_cda_applies_before_static_pt` puts CDA in PtCda (7a) and non-CDA in PtSet (7b) — different sublayers. CDA applies first because 7a < 7b, not because of the `is_cda` partition in `resolve_layer_order` (line 367). That partition logic is untested. **Fix:** add test with two effects in the SAME sublayer, one CDA and one not, verifying CDA applies first. | OPEN |
| MR-M5-09 | **INFO** | tests/layers.rs:461 | **Test name mismatch.** `test_613_layer7a_set_pt_to_mana_value` but effect uses `EffectLayer::PtSet` (layer 7b). Name implies 7a; should be `test_613_layer7b_set_pt_to_mana_value`. | — |
| MR-M5-10 | **INFO** | layers.rs:77-78 | **Comment inaccuracy.** "The mana value comes from the base mana cost (printed on the card)" but `chars.mana_cost` at layer 7+ has been through layers 1-6. No current layer modifies `mana_cost`, so the value is effectively base. No correctness impact. | — |
| MR-M5-11 | **INFO** | — | **No test scripts for layers.** `test-data/generated-scripts/layers/` doesn't exist. All 28 tests are unit tests. Unit tests are the correct approach for layer system isolated computation; script-based testing is more appropriate for full-game scenarios. | — |
| MR-M5-12 | **INFO** | layers.rs:432-439 | **Cycle fallback code is dead code.** `depends_on()` only produces `SetTypeLine → AddSubtypes/AddCardTypes` edges. No combination creates a cycle (Add* never depends on SetTypeLine). Lines 432-439 are unreachable. Defensively correct for future dependency rules. | — |
| MR-M5-13 | **INFO** | hash.rs:419-554 | **Complete hash coverage for all M5 types.** All 6 types have correct `HashInto` implementations. 10 `EffectFilter` variants, 21 `LayerModification` variants, and all 8 `ContinuousEffect` fields are hashed with unique discriminant bytes. No gaps. | — |

### Test Coverage Assessment

| M5 Behavior | Coverage | Notes |
|-------------|----------|-------|
| Layer ordering (1→7d) | Excellent (3 tests) | Type before ability, type before P/T, full 10-layer sequence |
| Layer 4 (type-changing) | Good (7 tests) | AddCardTypes, SetTypeLine, filter re-evaluation, Blood Moon, Urborg |
| Layer 5 (color-changing) | Thin (1 test) | SetColors only; no AddColors or BecomeColorless tests |
| Layer 6 (ability add/remove) | Good (3 tests) | RemoveAllAbilities, AddKeyword, layer 4 enables layer 6 filter |
| Layer 7a (CDA P/T) | Adequate (2 tests) | SetPtViaCda, SetPtToManaValue — but see MR-M5-08 (same-layer CDA partition untested) |
| Layer 7b (P/T-setting) | Good (5 tests) | SetPowerToughness, timestamp wins, Humility override |
| Layer 7c (P/T-modifying) | Good (4 tests) | ModifyBoth, ModifyPower, ModifyToughness stacking, counter integration |
| Layer 7d (P/T-switching) | Good (1 test) | Switch after set+modify chain |
| Timestamp ordering | Good (1 test) | Later timestamp overrides earlier |
| Dependencies (CR 613.8) | Excellent (4 tests) | Blood Moon+Urborg both directions, dependency chain, independent fallback |
| CDA partition (CR 613.3) | **Missing** (MR-M5-08) | No same-sublayer CDA vs non-CDA test |
| Duration: WhileSourceOnBattlefield | Good (1 test) | Source dies → effect inactive |
| Duration: UntilEndOfTurn | Good (1 test) | Expires at cleanup, Indefinite persists |
| Counter P/T at layer 7c | Good (3 tests) | +1/+1, -1/-1, ordering with 7b |
| Humility + Opalescence | Excellent (1 test, comprehensive) | Both cards verified: creature type, no abilities, 1/1 |
| Blood Moon + Urborg | Excellent (2 tests) | Both timestamp orderings; dependency wins regardless |
| Filter exclusion | Good (2 tests) | Non-matching objects, layer 4 enables later filter |
| Edge cases | Good (2 tests) | No effects = base chars, nonexistent object = None |
| AddColors, BecomeColorless | **Missing** | Defined but untested |
| RemoveKeyword | **Missing** | Defined but untested |
| LoseAllSubtypes | **Missing** | Defined but untested |
| Layer 1 (Copy) | **N/A** | Deferred to M7 |
| Layer 2 (Control) | **N/A** | Deferred (controller on GameObject) |
| Layer 3 (Text) | **N/A** | Deferred (no cards need it) |

### Notes

- M5 is the highest-risk milestone in the roadmap, and the implementation is solid. The core
  algorithm — 10-layer sequential application with per-layer CDA partition, dependency-aware
  topological sort, and timestamp fallback — is correct for all implemented layers (4-7d).
- The dependency model implements only one rule (`SetTypeLine → AddSubtypes/AddCardTypes`) which
  correctly handles the Blood Moon + Urborg interaction regardless of timestamp order. This is the
  most important dependency in Commander; additional dependency rules can be added as the card pool
  expands.
- The cycle fallback code (MR-M5-12) is dead code today but will become reachable when future
  dependency rules are added. The Kahn's algorithm implementation is correct.
- MR-M5-02 (SBAs using raw characteristics) is the most impactful finding — it's been tracked
  since M4 (MR-M4-02) and now that the layer system exists, the fix is straightforward: call
  `calculate_characteristics` in `check_creature_sbas` and related functions. This should be
  addressed before M8.
- MR-M5-01 (`.expect()`) continues the pattern seen across every milestone — provably safe but
  violating convention. The engine now has at least 8 such violations across M1-M5.
- Hash coverage (MR-M5-13) is complete — all M5 types are properly hashed with unique discriminants.
  No repeat of the M3 cross-milestone gap (where M7 added fields without updating hashes).
- Missing tests for `AddColors`, `BecomeColorless`, `RemoveKeyword`, and `LoseAllSubtypes` are
  LOW risk — these are simple set operations with the same pattern as tested variants. The CDA
  partition test gap (MR-M5-08) is more significant since it's the only untested branch in the
  core sorting logic.

---

## M6: Combat

**Review Status**: STUB — to be reviewed

### Files Introduced

| File | Lines | Purpose |
|------|-------|---------|
| `state/combat.rs` | 105 | AttackTarget, CombatState |
| `rules/combat.rs` | 789 | Declare attackers/blockers, order blockers, apply_combat_damage, first strike detection |

**Source total**: 894 lines

**Test files:**

| File | Lines | Tests | Focus |
|------|-------|-------|-------|
| `tests/combat.rs` | 937 | ~11 | Unblocked, blocked, mutual death, first/double strike, trample, deathtouch+trample, multiple blockers, triggers, commander damage, multiplayer |

### CR Sections Implemented

| CR Section | Implementation |
|------------|----------------|
| CR 506 | Begin combat step |
| CR 508 | Declare attackers (tap non-vigilance) |
| CR 509 | Declare blockers |
| CR 509.2 | Damage assignment order |
| CR 510 | Combat damage (simultaneous, two-phase collect+apply) |
| CR 510.1c | Last blocker gets all remaining power (no trample) |
| CR 702.2 | First strike |
| CR 702.4 | Double strike |
| CR 702.19 | Trample (excess to player/planeswalker) |
| CR 702.2+702.19 | Deathtouch + trample (1 to each blocker, rest to player) |
| CR 903.10a | Commander damage tracking |
| CR 511 | End of combat step |

### Known Issues (to be validated during review)

| ID | Severity | File | Description | Status |
|----|----------|------|-------------|--------|
| MR-M6-01 | **HIGH** | combat.rs | **Attack target not validated.** `DeclareAttackers` accepts any `AttackTarget` (Player/Planeswalker) without validating the target exists or is a legal attack target (e.g., attacking your own planeswalker). | STUB |
| MR-M6-02 | **HIGH** | combat.rs | **Double-blocking not prevented.** Same creature can be declared as blocker for multiple attackers. CR 509.1a says each creature blocks at most one attacking creature. | STUB |
| MR-M6-03 | **HIGH** | combat.rs | **Partial ordering accepted.** `OrderBlockers` command accepts incomplete ordering (not all blockers listed). Should require all blockers for the specified attacker. | STUB |
| MR-M6-04 | **MEDIUM** | combat.rs | **`is_blocked` contract/invariant risk.** If `blocker_map` is pruned (e.g., blocker removed by SBA), an attacker with no remaining blockers might still be treated as blocked (CR 509.1h: remains blocked even if all blockers removed). Need to verify this is correctly handled. | STUB |
| MR-M6-05 | **MEDIUM** | combat.rs | **Unsafe `i32→u32` cast in damage calculation.** Power is `i32` but damage is `u32`. Negative power (from layer effects) cast to u32 wraps to very large number. Should clamp to 0. | STUB |
| MR-M6-06 | **LOW** | combat.rs | **Performance: `apply_combat_damage` could extract helpers.** Large function with repeated patterns. Not a correctness issue. | STUB |
| MR-M6-07 | **LOW** | combat.rs | **Test gap: no test for creature that can't attack (Defender keyword).** Defender enforcement exists but may not be tested in combat.rs (may be in keywords.rs). | STUB |
| MR-M6-08 | **LOW** | combat.rs | **Test gap: no game script for combat.** All combat tests are Rust unit tests; no JSON script exercises combat through the replay harness. | STUB |

---

## M7: Card Definition Framework & First Cards

**Review Status**: STUB — to be reviewed

### Files Introduced

**Source files — cards:**

| File | Lines | Purpose |
|------|-------|---------|
| `cards/card_definition.rs` | 574 | CardDefinition, AbilityDefinition, Effect (recursive, 30+ variants), EffectAmount, EffectTarget, etc. |
| `cards/definitions.rs` | 1,230 | 50 hand-authored Commander staple definitions |
| `cards/registry.rs` | 52 | CardRegistry with Arc<Self>, lookup by CardId |
| `cards/mod.rs` | 21 | Module exports |

**Source files — effects:**

| File | Lines | Purpose |
|------|-------|---------|
| `effects/mod.rs` | 1,209 | `execute_effect` engine, EffectContext, all effect implementations |

**Source total**: 3,086 lines

**Test files:**

| File | Lines | Tests | Focus |
|------|-------|-------|-------|
| `tests/effects.rs` | 578 | ~15 | Direct effect execution (DealDamage, Exile, GainLife, DrawCards, Sequence, Conditional, ForEach) |
| `tests/keywords.rs` | 677 | — | Keyword enforcement (Hexproof, Shroud, Indestructible, Lifelink, Menace, Defender, Haste, Vigilance, Flash) |
| `tests/run_all_scripts.rs` | 155 | — | Auto-discovery of approved game scripts |
| `tests/script_replay.rs` | 1,247 | — | Script replay harness (build_initial_state, translate_player_action, check_assertions, enrich_spec_from_def) |

**Test data:**

| File | Category |
|------|----------|
| `test-data/generated-scripts/baseline/001_priority_pass_empty_stack.json` | Baseline |
| `test-data/generated-scripts/baseline/002_play_basic_land.json` | Baseline |
| `test-data/generated-scripts/baseline/003_tap_land_for_mana.json` | Baseline |
| `test-data/generated-scripts/stack/001_lightning_bolt_resolves.json` | Stack |
| `test-data/generated-scripts/stack/002_counterspell_counters_spell.json` | Stack |
| `test-data/generated-scripts/stack/003_sol_ring_enters_battlefield.json` | Stack |
| `test-data/generated-scripts/stack/004_swords_to_plowshares_exiles_creature.json` | Stack |

**7 approved game scripts** (3 baseline + 4 stack)

### CR Sections Implemented

| CR Section | Implementation |
|------------|----------------|
| CR 702.11 | Hexproof (targeting check) |
| CR 702.18 | Shroud (targeting check) |
| CR 702.12 | Indestructible (destroy replacement) |
| CR 702.15 | Lifelink (damage → gain life) |
| CR 702.121 | Menace (≥2 blockers required) |
| CR 702.3 | Defender (can't attack) |
| CR 702.10 | Flash (instant-speed casting) |
| CR 702.10 | Haste (no summoning sickness) |
| CR 702.20 | Vigilance (no tap on attack) |

### Known Issues (to be validated during review)

| ID | Severity | File | Description | Status |
|----|----------|------|-------------|--------|
| MR-M7-01 | **HIGH** | effects/mod.rs | **MoveToZone effect doesn't emit correct zone-change event.** `execute_effect` for zone moves may not fire the expected `move_object_to_zone` path that generates proper CR 400.7 events. | STUB |
| MR-M7-02 | **HIGH** | effects/mod.rs | **Doom Blade filter: "non-black" not enforced.** `DestroyPermanent` effect may not check target color filter. Doom Blade should not destroy black creatures. | STUB |
| MR-M7-03 | **HIGH** | effects/mod.rs | **Owner vs controller confusion in effects.** Some effects use `controller` where `owner` is correct (e.g., "return to its owner's hand") or vice versa. | STUB |
| MR-M7-04 | **MEDIUM** | effects/mod.rs | **Lifelink only works for combat damage, not spell damage.** Lifelink applies to ALL damage (CR 702.15), not just combat. If a creature with lifelink deals damage via a fight effect or similar, lifelink should trigger. | STUB |
| MR-M7-05 | **MEDIUM** | effects/mod.rs | **Controller filter not checked in some target resolution.** `resolve_target` may not filter by controller when the effect specifies "target creature you control." | STUB |
| MR-M7-06 | **MEDIUM** | effects/mod.rs | **ForEach on "each player" may not iterate in APNAP order.** CR requires player-affecting actions in APNAP order. | STUB |
| MR-M7-07 | **MEDIUM** | effects/mod.rs | **Unsafe `i32→u32` casts.** Power/toughness are i32 but some effect amounts use u32. Negative values wrap. | STUB |
| MR-M7-08 | **MEDIUM** | effects/mod.rs | **Unsafe cast in another location.** Second instance of i32→u32 issue. | STUB |
| MR-M7-09 | **MEDIUM** | abilities.rs | **`unwrap_or(0)` in effect resolution paths.** Used by M7 effect system when resolving EffectAmount::PowerOf. If power is None, returns 0 silently. Should this warn or log? | STUB |
| MR-M7-10 | **MEDIUM** | abilities.rs | **Second `unwrap_or` in abilities.** Similar pattern, different location. | STUB |
| MR-M7-11 | **LOW** | casting.rs | **Casting helper could be extracted.** `handle_cast_spell` is large; common validation patterns repeated. | STUB |
| MR-M7-12 | **LOW** | lands.rs | **Redundant check in land play handler.** Validates a condition already guaranteed by earlier check. | STUB |
| MR-M7-13 | **LOW** | — | **Test gap: no SBA cascade test.** No test verifying that SBAs triggered by spell resolution correctly chain (e.g., Lightning Bolt kills creature → CreatureDied event → triggers checked). | STUB |
| MR-M7-14 | **LOW** | — | **Test gap: no layer + SBA interaction test.** No test verifying that a continuous effect reducing toughness to 0 triggers SBA correctly via `calculate_characteristics`. | STUB |
| MR-M7-15 | **LOW** | — | **Test gap: no combat game script.** All 7 scripts are baseline/stack; no script exercises the combat system through the replay harness. | STUB |

---

## M8: Replacement & Prevention Effects

**Review Status**: NOT STARTED (current milestone)

### Files Expected

Per roadmap deliverables:
- Replacement effect framework in `effects/` or `rules/`
- Self-replacement effects (CR 614.15)
- Player choice for multiple replacement effects
- Loop prevention (CR 614.5)
- Prevention effects (prevent N damage, prevent all damage)
- Prevention/replacement interaction (CR 616)
- "If ~ would die" replacement effects
- "If a player would draw" replacement effects
- "Enters the battlefield" replacement effects (e.g., "enters tapped")

### CR Sections to Implement

| CR Section | Description |
|------------|-------------|
| CR 614 | Replacement effects |
| CR 614.5 | Loop prevention |
| CR 614.15 | Self-replacement effects priority |
| CR 615 | Prevention effects |
| CR 616 | Interaction between replacement and prevention |

### Findings

(None yet — milestone in progress)

---

## M9: Commander Rules Integration

**Review Status**: NOT STARTED

### Files Expected

Per roadmap deliverables:
- Commander format enforcement (deck validation, color identity, banned list)
- Command zone mechanics (casting, commander tax)
- Commander replacement effects (zone-change choice)
- Commander damage SBA integration
- Partner mechanics
- Mulligan (Commander-specific)
- `GameEvent::reveals_hidden_info()` method

### CR Sections to Implement

| CR Section | Description |
|------------|-------------|
| CR 903 | Commander format rules |
| CR 903.3 | Color identity |
| CR 903.5a | 100-card singleton |
| CR 903.6 | Command zone |
| CR 903.8 | Commander tax |
| CR 903.10a | Commander damage |
| CR 903.9 | Commander zone-change replacement |
| CR 903.13 | Partner |
| CR 103.5 | Mulligan (Commander variant) |

### Findings

(None yet — milestone not started)

---

## Cross-Milestone Issue Index

All findings across all milestones, sorted by severity then milestone.

### CRITICAL

(None identified — all engine panics classified as HIGH per current assessment)

### HIGH

| ID | Milestone | Summary | Status |
|----|-----------|---------|--------|
| MR-M0-01 | M0 | Delete-then-import data loss risk in scryfall-import | OPEN |
| MR-M0-02 | M0 | FTS5 MATCH operator injection in MCP server | OPEN |
| MR-M1-01 | M1 | `.unwrap()` in `add_object()` (state/mod.rs:159) | OPEN |
| MR-M1-02 | M1 | `.unwrap()` in `move_object_to_zone()` (state/mod.rs:228) | OPEN |
| MR-M2-01 | M2 | `.expect()` in priority.rs:54 — `next_priority_player` | OPEN |
| MR-M2-02 | M2 | `.expect()` in turn_structure.rs:78 — `next_player_in_turn_order` | OPEN |
| MR-M2-03 | M2 | Concede while active: step-advance then turn-advance overlap | OPEN |
| MR-M2-05 | M2→M9 | Concede doesn't clean up owned objects (CR 800.4a) | DEFERRED → M9 |
| MR-M3-01 | M3 | Partial fizzle: targets not filtered — effects execute against illegal targets | OPEN |
| MR-M3-03 | M3 | GameObject hash omits `has_summoning_sickness` — breaks distributed verification | OPEN |
| MR-M3-04 | M3 | Non-existent object target silently accepted in ability activation | OPEN |
| MR-M4-01 | M4 | `.unwrap()` in `check_player_sbas` (sba.rs:120) — commander damage path | OPEN |
| MR-M4-02 | M4→M5 | SBAs don't use `calculate_characteristics` for P/T or keyword checks | OPEN — see MR-M5-02 |
| MR-M4-03 | M4 | `.unwrap()` in `check_legendary_rule` (sba.rs:340) — `ids.last()` | OPEN |
| MR-M5-01 | M5 | `.expect()` in layers.rs:95 — counter P/T block | OPEN |
| MR-M5-02 | M5 | SBAs use raw characteristics, not `calculate_characteristics` (7 call sites) | OPEN |
| MR-M6-01 | M6 | Attack target not validated | STUB |
| MR-M6-02 | M6 | Double-blocking not prevented | STUB |
| MR-M6-03 | M6 | Partial blocker ordering accepted | STUB |
| MR-M7-01 | M7 | MoveToZone effect doesn't emit correct zone-change event | STUB |
| MR-M7-02 | M7 | Doom Blade filter not enforced | STUB |
| MR-M7-03 | M7 | Owner vs controller confusion in effects | STUB |

### MEDIUM

| ID | Milestone | Summary | Status |
|----|-----------|---------|--------|
| MR-M0-03 | M0 | Multi-line CR rules not captured | OPEN |
| MR-M0-04 | M0 | CR format assumptions fragile | OPEN |
| MR-M0-05 | M0 | FTS index probe fragile | OPEN |
| MR-M0-06 | M0 | JSON parse errors lose context | OPEN |
| MR-M0-07 | M0 | No download integrity check | OPEN |
| MR-M1-03 | M1 | `.expect()` in builder.rs:318 | OPEN |
| MR-M1-04 | M1 | Check-then-access pattern in state/mod.rs | OPEN |
| MR-M1-05 | M1 | Panics on 0 players instead of Result | OPEN |
| MR-M2-04 | M2 | `draw_card` has no concession/elimination guard | OPEN |
| MR-M2-06 | M2 | `DiscardedToHandSize` event uses wrong ObjectId (new graveyard ID instead of old hand ID) | OPEN |
| MR-M3-02 | M3 | ManaCostPaid not emitted for {0} cost spells | OPEN |
| MR-M3-05 | M3 (cross: M7) | ActivatedAbility hash omits `effect` field — M7 added field, hash not updated | OPEN |
| MR-M3-06 | M3 (cross: M7) | TriggeredAbilityDef hash omits `effect` field — same gap as MR-M3-05 | OPEN |
| MR-M3-07 | M3 | Hexproof/shroud target validation duplicated between casting.rs and abilities.rs | OPEN |
| MR-M3-08 | M3 | `matches!` bare statement in test — silent no-op assertion | OPEN |
| MR-M4-04 | M4 | `u32 as i32` cast in lethal damage comparison (sba.rs:217) — wraps on overflow | OPEN |
| MR-M4-05 | M4 | `u32 as i32` cast in planeswalker loyalty counter (sba.rs:280) | OPEN |
| MR-M4-06 | M4→M8 | CR 704.5b not implemented as SBA — empty library loss is immediate, not SBA-checked | DEFERRED → M8 |
| MR-M5-03 | M5 | `ptr::eq` for effect identity in cycle fallback — correct but fragile | OPEN |
| MR-M5-04 | M5→M8+ | `EffectDuration` missing `UntilEndOfNextTurn` and general `AsLongAs(Condition)` | DEFERRED → M8+ |
| MR-M5-05 | M5 | `AllPermanents` filter over-checks card types instead of just checking Battlefield zone | OPEN |
| MR-M6-04 | M6 | `is_blocked` contract/invariant risk | STUB |
| MR-M6-05 | M6 | Unsafe i32→u32 cast in combat damage | STUB |
| MR-M7-04 | M7 | Lifelink only works for combat damage | STUB |
| MR-M7-05 | M7 | Controller filter not checked in target resolution | STUB |
| MR-M7-06 | M7 | ForEach players not APNAP ordered | STUB |
| MR-M7-07 | M7 | Unsafe i32→u32 cast in effects (first) | STUB |
| MR-M7-08 | M7 | Unsafe i32→u32 cast in effects (second) | STUB |
| MR-M7-09 | M7 | `unwrap_or(0)` for PowerOf resolution | STUB |
| MR-M7-10 | M7 | Second `unwrap_or` in abilities | STUB |

### LOW

| ID | Milestone | Summary | Status |
|----|-----------|---------|--------|
| MR-M0-08 | M0 | No ON DELETE CASCADE for card_faces FK | OPEN |
| MR-M0-09 | M0 | JSON columns stored as TEXT | OPEN |
| MR-M0-10 | M0 | Partial card name matching too broad | OPEN |
| MR-M1-06 | M1 | structural_sharing.rs uses mock types | OPEN |
| MR-M1-07 | M1 | ManaPool tests thin (1 test) | OPEN |
| MR-M2-07 | M2 | Proptest lacks library cards — limited turn coverage | OPEN |
| MR-M2-08 | M2 | Test gap: concede while active + all others passed | OPEN |
| MR-M2-09 | M2 | `unwrap_or(7)` for max_hand_size in cleanup | OPEN |
| MR-M3-09 | M3 | LegendaryRuleApplied event hash missing length prefix for put_to_graveyard | OPEN |
| MR-M3-10 | M3 | Incomplete test discards results (test_608_2b_fizzle_player_target_concedes) | OPEN |
| MR-M3-11 | M3 | `apnap_order` silently defaults position with `unwrap_or(0)` | OPEN |
| MR-M3-12 | M3 | `NotController` error used for ownership check in lands.rs — misleading | OPEN |
| MR-M4-07 | M4 | CR 704.5e (spell/card copies) not implemented — no `is_copy` field | DEFERRED → M8+ |
| MR-M4-08 | M4 | CR 704.5k (world rule) not implemented — irrelevant for Commander | DEFERRED |
| MR-M4-09 | M4 | `String::clone()` allocation in legendary rule hot path (sba.rs:329) | OPEN |
| MR-M4-10 | M4 | `SubType("...".to_string())` allocates on every SBA comparison (sba.rs:391,449,453) | OPEN |
| MR-M4-11 | M4 | `unwrap_or(1)` default for missing planeswalker loyalty — hides construction bugs | OPEN |
| MR-M4-12 | M4 | Test gap: no planeswalker with Loyalty counters (only characteristics.loyalty) | OPEN |
| MR-M4-13 | M4 | Test gap: no aura whose target left battlefield (only tests unattached) | OPEN |
| MR-M4-14 | M4 | Test gap: no 3+ legendary copies test | OPEN |
| MR-M5-06 | M5 | `ready.remove(0)` is O(n) in Kahn's algorithm — use VecDeque | OPEN |
| MR-M5-07 | M5→M8+ | Missing `AddSupertypes`/`RemoveSupertypes` layer 4 variants | DEFERRED → M8+ |
| MR-M5-08 | M5 | CDA partition test uses different sublayers — same-layer CDA priority untested | OPEN |
| MR-M6-06 | M6 | Combat damage function large, needs helpers | STUB |
| MR-M6-07 | M6 | Test gap: Defender keyword in combat | STUB |
| MR-M6-08 | M6 | Test gap: no combat game script | STUB |
| MR-M7-11 | M7 | Casting helper extraction | STUB |
| MR-M7-12 | M7 | Redundant check in lands.rs | STUB |
| MR-M7-13 | M7 | Test gap: SBA cascade after spell resolution | STUB |
| MR-M7-14 | M7 | Test gap: layer + SBA interaction | STUB |
| MR-M7-15 | M7 | Test gap: combat game script | STUB |

### INFO

| ID | Milestone | Summary | Status |
|----|-----------|---------|--------|
| MR-M0-11 | M0 | card-db/lib.rs clean | — |
| MR-M0-12 | M0 | rules_db.rs good test coverage | — |
| MR-M1-08 | M1 | object_identity.rs exemplary CR citation | — |
| MR-M1-09 | M1 | state_invariants.rs good property-based foundation | — |
| MR-M1-10 | M1 | Commander format compliance verified | — |
| MR-M1-11 | M1 | Type safety is strong | — |
| MR-M2-10 | M2 | Loop-based step advancement (good design) | — |
| MR-M2-11 | M2 | `pass_priority` query/mutation separation (good design) | — |
| MR-M2-12 | M2 | Extra turns LIFO with correct normal-order resumption | — |
| MR-M2-13 | M2 | Summoning sickness cleared at untap (CR 302.6) | — |
| MR-M3-13 | M3 | Mana payment design correct (colored/colorless strict, generic any) | — |
| MR-M3-14 | M3 | Clean stack module — well-typed StackObject/StackObjectKind | — |
| MR-M3-15 | M3 | State hashing framework solid (blake3, explicit, cross-platform) | — |
| MR-M3-16 | M3 | Well-structured test suites with CR citations across all 7 test files | — |
| MR-M3-17 | M3 | Dual-instance proptest strong for determinism validation | — |
| MR-M3-18 | M3 | Script schema well-designed and extensible | — |
| MR-M4-15 | M4 | Correct indestructible handling: 704.5f (no prevent) vs 704.5g/h (prevent) | — |
| MR-M4-16 | M4 | Fixed-point loop correct and convergent — each pass removes objects | — |
| MR-M4-17 | M4 | SBA integration with engine correct — fires before priority, after resolution | — |
| MR-M4-18 | M4 | Equipment unattach vs aura destroy — correct CR 704.5m/n distinction | — |
| MR-M4-19 | M4 | Single loss event per player per SBA pass — correct CR ordering | — |
| MR-M5-09 | M5 | Test name mismatch: `test_613_layer7a_*` but effect is in PtSet (layer 7b) | — |
| MR-M5-10 | M5 | Comment says "base mana cost" but `chars.mana_cost` is post-layer-1-6 modified | — |
| MR-M5-11 | M5 | No test scripts for layers — unit tests are the right approach for isolated computation | — |
| MR-M5-12 | M5 | Cycle fallback code is dead code — current `depends_on()` cannot produce cycles | — |
| MR-M5-13 | M5 | Complete hash coverage for all 6 M5 types — no gaps | — |

---

## Statistics

| Metric | Value |
|--------|-------|
| Total unique issue IDs | 109 (MR-M5-02 cross-refs MR-M4-02; MR-M3-05/06 cross-ref M7) |
| CRITICAL | 0 |
| HIGH (OPEN) | 14 |
| HIGH (DEFERRED) | 2 |
| HIGH (STUB) | 6 |
| MEDIUM (OPEN) | 19 |
| MEDIUM (DEFERRED) | 2 |
| MEDIUM (STUB) | 9 |
| LOW (OPEN) | 20 |
| LOW (DEFERRED) | 3 |
| LOW (STUB) | 8 |
| INFO | 26 |
| Milestones fully reviewed | 6 (M0, M1, M2, M3, M4, M5) |
| Milestones with stubs | 2 (M6, M7) |
| Milestones not started | 2 (M8, M9) |

**Engine source LOC (M0-M7)**: ~12,500 lines
**Engine test LOC (M1-M7)**: ~14,600 lines
**Total test count**: 303 (all passing)
