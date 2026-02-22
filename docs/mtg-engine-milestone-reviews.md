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

**Review Status**: REVIEWED (2026-02-22)

### Files Introduced

| File | Lines | Purpose |
|------|-------|---------|
| `state/combat.rs` | 105 | `AttackTarget` enum, `CombatState` struct (attackers, blockers, damage_assignment_order, defenders_declared) |
| `rules/combat.rs` | 789 | `handle_declare_attackers`, `handle_declare_blockers`, `handle_order_blockers`, `apply_combat_damage`, `should_have_first_strike_step`, helpers (`get_effective_power/toughness`, `has_keyword`, `deals_damage_in_step`, `push_player_or_pw_damage`) |

**Additions to existing files:**

| File | Lines Added | Purpose |
|------|-------------|---------|
| `rules/turn_actions.rs` | ~33 (lines 233-273) | `begin_combat`, `first_strike_damage_step`, `combat_damage_step`, `end_combat` turn-based actions + dispatch in `execute_turn_based_actions` |
| `rules/turn_structure.rs` | ~7 (lines 37-44) | Conditional `FirstStrikeDamage` step insertion between DeclareBlockers and CombatDamage |
| `rules/events.rs` | — | `AttackersDeclared`, `BlockersDeclared`, `CombatDamageDealt`, `CombatEnded` events; `CombatDamageAssignment`, `CombatDamageTarget` types |
| `rules/command.rs` | — | `DeclareAttackers`, `DeclareBlockers`, `OrderBlockers` command variants |
| `state/mod.rs` | 1 | `combat: Option<CombatState>` field on GameState |
| `state/hash.rs` | ~8 (lines 724-733) | `HashInto` impls for `CombatState`, `AttackTarget` |

**Source total**: ~894 new lines + ~49 additions to existing files

**Test files:**

| File | Lines | Tests | Focus |
|------|-------|-------|-------|
| `tests/combat.rs` | 938 | 11 | Unblocked, blocked, mutual death, first/double strike, trample, deathtouch+trample, multiple blockers, triggers, commander damage, multiplayer |

### CR Sections Implemented

| CR Section | Implementation |
|------------|----------------|
| CR 506 | Begin combat step — `begin_combat` initializes `CombatState` (`turn_actions.rs:241-247`) |
| CR 508.1 | Declare attackers — validation (creature, not tapped, no summoning sickness, no defender), tap non-vigilance (`combat.rs:34-161`) |
| CR 508.1f | Non-vigilance attackers tapped on declaration (`combat.rs:119-131`) |
| CR 509.1 | Declare blockers — validation (creature, untapped, controlled by declaring player, flying/reach) (`combat.rs:172-305`) |
| CR 509.1h | Blocked status persists even if all blockers removed — `is_blocked()` queries declaration map (`combat.rs:86-93, 440-465`) |
| CR 509.2 | Damage assignment order — `handle_order_blockers` command (`combat.rs:316-370`) |
| CR 510.1c | Lethal damage to each blocker in order before excess flows to next; last blocker absorbs all remaining (`combat.rs:467-530`) |
| CR 510.2 | Simultaneous damage — two-phase collect+apply pattern (`combat.rs:386-697`) |
| CR 510.4 | First-strike damage step conditionally inserted (`turn_structure.rs:37-44`) |
| CR 702.7 | First strike — `deals_damage_in_step` routes FS creatures to first-strike step only (`combat.rs:755-764`) |
| CR 702.4 | Double strike — deals damage in both steps (`combat.rs:755-764`) |
| CR 702.19b | Trample — excess to player/planeswalker after lethal to each blocker (`combat.rs:491-510`) |
| CR 702.2c+702.19b | Deathtouch + trample — 1 damage is lethal for assignment (`combat.rs:479-480`) |
| CR 702.15a | Lifelink — controller gains life equal to combat damage dealt (`combat.rs:674-694`) |
| CR 702.110a | Menace — requires ≥2 blockers (`combat.rs:247-277`) |
| CR 903.10a | Commander damage tracking — per-player per-commander `OrdMap` (`combat.rs:636-662`) |
| CR 511.1 | End of combat — clear `CombatState` (`turn_actions.rs:270-273`) |

### Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| MR-M6-01 | **HIGH** | combat.rs:67 | **Attack target not validated.** `handle_declare_attackers` validates each attacker (battlefield, controller, creature type, defender, tapped, summoning sickness) but completely ignores the `AttackTarget`. The `_target` variable on line 67 is never inspected. Accepts: (1) attacking yourself `AttackTarget::Player(self)`, (2) attacking a non-existent player, (3) attacking your own planeswalker, (4) attacking a planeswalker not on the battlefield. **Fix:** validate that `Player(pid)` is an opponent and exists; validate that `Planeswalker(pw_id)` is on the battlefield and controlled by an opponent. | OPEN |
| MR-M6-02 | **HIGH** | combat.rs:282-284 | **Same creature can block multiple attackers.** `combat.blockers.insert(*blocker_id, *attacker_id)` uses OrdMap insert which silently overwrites. If the `blockers` Vec parameter contains the same `blocker_id` twice with different attacker_ids (e.g., `[(B, A1), (B, A2)]`), the last entry wins. CR 509.1a: "For each of the chosen creatures, the defending player chooses one creature for it to block." A creature blocks exactly one attacker. **Fix:** before inserting, check `combat.blockers.contains_key(blocker_id)` or validate no duplicate blocker_ids in the input Vec. | OPEN |
| MR-M6-03 | **HIGH** | combat.rs:356-363 | **Partial blocker ordering accepted.** `handle_order_blockers` validates that every blocker in `order` is blocking the attacker (line 356-362) but does NOT validate that every blocker OF the attacker is in `order`. Submitting a partial order (e.g., ordering 1 of 3 blockers) silently accepts; the other 2 blockers are excluded from `damage_assignment_order` and receive no attacker damage. CR 509.2: the attacking player orders ALL blocking creatures. **Fix:** add `if order.len() != blocking_this.len() { return Err(...) }` after line 363. | OPEN |
| MR-M6-09 | **HIGH** | combat.rs:196-230 | **Blocker can block attacker targeting a different player.** `handle_declare_blockers` validates that the attacker is a declared attacker (line 225) but never checks that the attacker is targeting the declaring player or their planeswalker. In multiplayer, p2 could declare their creature blocks an attacker that's attacking p3. CR 509.1a: "the defending player chooses one creature for it to block that's attacking that player or a planeswalker that player controls." **Fix:** after line 230, resolve the attacker's `AttackTarget` and verify it matches the declaring player. | OPEN |
| MR-M6-10 | **HIGH** | combat.rs:172-305 | **`defenders_declared` tracked but never enforced.** The `CombatState::defenders_declared` set is populated on line 286 but never checked as a guard at the top of `handle_declare_blockers`. The same player can call `DeclareBlockers` multiple times, overwriting earlier blocker assignments (OrdMap insert). This allows: (1) changing which attacker a creature blocks mid-step, (2) adding new blockers after already finishing declaration. **Fix:** add guard: `if combat.defenders_declared.contains(&player) { return Err(...) }` after line 193. | OPEN |
| MR-M6-11 | **MEDIUM** | combat.rs:249-276 | **Menace check counts Vec entries, not unique creatures.** The menace check sums `blocker_count_for_attacker` from both existing combat blockers and the new `blockers` Vec. If MR-M6-02 allows the same blocker_id to appear twice for the same attacker, the count would be 2 (satisfying menace) even though only 1 unique creature is blocking. **Dependency:** fixing MR-M6-02 (duplicate blocker prevention) also fixes this. | OPEN — depends on MR-M6-02 |
| MR-M6-12 | **MEDIUM** | combat.rs:98,121 | **Redundant `calculate_characteristics` for vigilance.** Vigilance is checked twice in `handle_declare_attackers`: first via `chars.keywords.contains(&Vigilance)` (line 98, from the full `calculate_characteristics` call at line 81), then again via the `has_keyword` helper (line 121, which calls `calculate_characteristics` a second time). The second call is unnecessary — `has_vigilance` from line 98 should be reused. Not a correctness bug (both calls see the same immutable state) but wasteful and fragile. | OPEN |
| MR-M6-04 | **INFO** | combat.rs:86-93,440-465 | **RESOLVED — `is_blocked` correctly implements CR 509.1h.** The `blockers` OrdMap is never pruned during combat; entries persist until `end_combat` clears the entire `CombatState`. When `ordered_blockers` is empty (all blockers left battlefield), `is_blocked()` still returns true because the declaration entries remain. The damage code at lines 440-465 correctly distinguishes: (1) never blocked → damage to player, (2) was blocked + blockers gone + trample → damage to player, (3) was blocked + blockers gone + no trample → no damage. Stub concern was unfounded. | RESOLVED |
| MR-M6-05 | **INFO** | combat.rs:404-407,554-557 | **RESOLVED — i32→u32 cast is safe.** `get_effective_power` returns `i32`; the guard `if power <= 0 { continue; }` (lines 405, 555) ensures only positive values reach `power as u32`. All intermediate values (`to_blocker`, `trample_amount`, `remaining`) are provably non-negative: `remaining` starts positive and decreases by non-negative amounts; `to_blocker = remaining.min(lethal)` where `lethal >= 0`; `trample_amount = remaining - to_blocker >= 0`. Stub concern was unfounded. | RESOLVED |
| MR-M6-06 | **LOW** | combat.rs:386-697 | **`apply_combat_damage` is 312 lines.** Attacker damage (lines 398-531) and blocker damage (lines 533-564) could be extracted into helper functions. The two-phase collect+apply pattern (lines 570-694) could also be a helper. Not a correctness issue. | OPEN |
| MR-M6-07 | **INFO** | — | **RESOLVED — Defender tested in keywords.rs.** `test_702_3_defender_cannot_attack` in `tests/keywords.rs` verifies that `DeclareAttackers` returns an error for creatures with Defender. Coverage exists; no need to duplicate in `combat.rs`. | RESOLVED |
| MR-M6-08 | **LOW** | — | **Test gap: no combat game script.** All 11 combat tests are Rust unit tests. The `test-data/generated-scripts/combat/` directory is empty. No JSON script exercises combat through the replay harness. Combat is complex enough to warrant at least one basic script (declare attackers → declare blockers → damage). | OPEN |
| MR-M6-13 | **LOW** | — | **Test gap: no test for blocker-removed-before-damage (CR 509.1h).** The `is_blocked` behavior is correct (MR-M6-04) but untested for the specific scenario where all blockers leave the battlefield before combat damage. `test_509_blocked_attacker_no_player_damage` has the blocker present during damage. A test where the blocker is killed (e.g., by first-strike damage from a different attacker, or by a spell during DeclareBlockers priority) would exercise the "was blocked, blockers gone, no trample → no player damage" branch at line 465. | OPEN |
| MR-M6-14 | **LOW** | combat.rs:74-83 | **`blockers_for()` rebuilds list on every call.** Iterates the entire `blockers` OrdMap filtering by attacker. Called in `apply_combat_damage` (line 413 path) and `handle_order_blockers` (line 349). For n total blockers, each call is O(n). Typical combat has ≤10 blockers so impact is negligible. Could cache in `CombatState` if it becomes a bottleneck. | OPEN |
| MR-M6-15 | **INFO** | combat.rs:86-93 | **Blocked status persistence is correct per CR 509.1h.** "An attacking creature with one or more creatures declared as blockers for it becomes a blocked creature [...] A creature remains blocked even if all the creatures blocking it are removed from combat." The `blockers` OrdMap serves as both the blocker assignment and the blocked-status record. Clean design. | — |
| MR-M6-16 | **INFO** | combat.rs:570-612 | **Two-phase collect+apply prevents use-after-free.** Pre-extracting deathtouch, lifelink, controller, and commander info (lines 570-612) before mutating state (lines 618-679) ensures consistent reads. Simultaneous damage per CR 510.2. Sound design. | — |
| MR-M6-17 | **INFO** | turn_actions.rs:241-273 | **Combat state lifecycle correct.** Initialized at BeginningOfCombat (`begin_combat`), cleared at EndOfCombat (`end_combat` sets `state.combat = None`). FirstStrikeDamage conditionally inserted per `should_have_first_strike_step`. All combat turn-based actions correctly dispatched via `execute_turn_based_actions`. | — |

### Test Coverage Assessment

| M6 Behavior | Coverage | Notes |
|-------------|----------|-------|
| Declare attackers (active player, tap, summoning sickness) | Good (implicit in all 11 tests) | Every test declares attackers; errors tested in keywords.rs (Defender, summoning sickness, haste) |
| Declare blockers (defending player, flying/reach, menace) | Good (tested) | Blocking validated in blocked/mutual/first-strike/trample tests; flying/reach/menace in keywords.rs |
| Unblocked attacker → player damage | Excellent (test 1) | 2/2 deals 2 to p2, life 40→38 |
| Blocked attacker → no player damage | Good (test 2) | 5/5 blocked by 1/1, p2 life unchanged |
| Mutual combat death (simultaneous damage) | Good (test 3) | 3/3 vs 3/3, both die, 2 CreatureDied events |
| First strike kills before regular damage | Good (test 4) | 2/1 FS kills 2/2 blocker; attacker survives |
| Double strike (both damage steps) | Good (test 5) | 2/2 DS deals 2+2=4 to player |
| Trample excess to player | Good (test 6) | 5/5 trample vs 2/2: 2 to blocker, 3 to player |
| Deathtouch + trample (1 lethal) | Excellent (test 7) | 4/4 DT+T vs 3/3: 1 to blocker, 3 to player |
| Multiple blockers + damage order | Good (test 8) | 5/5 vs [2/2, 2/2]: first gets 2, second gets 3 |
| SelfAttacks trigger | Good (test 9) | AbilityTriggered event + stack entry verified |
| Commander damage tracking | Good (test 10) | `commander_damage_received[p1][card_id] == 5` |
| Multiplayer simultaneous attacks | Good (test 11) | Attack p2 and p3; both take damage correctly |
| Vigilance (no tap on attack) | Tested in keywords.rs | Not in combat.rs; `test_702_20_vigilance_no_tap_on_attack` |
| Defender (can't attack) | Tested in keywords.rs | `test_702_3_defender_cannot_attack` |
| Lifelink (gain life on combat damage) | Tested in keywords.rs | `test_702_15_lifelink_grants_life_on_combat_damage` |
| Attack target validation | **MISSING** | No test for illegal targets (MR-M6-01) |
| Double-blocking prevention | **MISSING** | No test that same creature can't block two attackers (MR-M6-02) |
| Cross-player blocking prevention | **MISSING** | No test that p2 can't block attackers targeting p3 (MR-M6-09) |
| Re-declaration prevention | **MISSING** | No test that same player can't call DeclareBlockers twice (MR-M6-10) |
| Complete ordering requirement | **MISSING** | No test that OrderBlockers rejects partial orders (MR-M6-03) |
| Blocker removed before damage | **MISSING** | CR 509.1h behavior untested for blocker-gone scenario (MR-M6-13) |
| Combat game scripts | **MISSING** | No JSON scripts in `test-data/generated-scripts/combat/` (MR-M6-08) |

### Notes

- **Architecture is solid.** The core combat damage calculation — two-phase collect+apply,
  simultaneous damage, deathtouch+trample interaction, first/double strike step routing,
  and damage assignment order — is correct and well-tested. The 11 tests cover the main
  mechanical interactions comprehensively.
- **Blocked status (CR 509.1h) is correctly implemented.** The `blockers` OrdMap naturally
  preserves blocked status because entries are never removed during combat. `is_blocked()`
  queries declaration history, not current battlefield state. The damage code at lines
  440-465 correctly uses this to determine whether an attacker deals player damage. Stub
  concern MR-M6-04 was unfounded.
- **i32→u32 casts are safe.** The `power <= 0` guard prevents negative values from reaching
  any `as u32` cast. All intermediate values (`remaining`, `to_blocker`, `trample_amount`)
  are provably non-negative. Stub concern MR-M6-05 was unfounded.
- **The 5 HIGH findings are all validation gaps**, not algorithmic bugs. The damage calculation,
  step routing, keyword handling, and state lifecycle are all correct. The missing checks are:
  (1) attack target legality, (2) one-creature-one-attacker, (3) complete blocker ordering,
  (4) defenders can only block their own attackers, (5) no re-declaration. All fixes are
  localized to `handle_declare_attackers` and `handle_declare_blockers`.
- **MR-M6-09 and MR-M6-10 are multiplayer-specific.** In 1v1 games, there's only one
  defending player and one attack target, so these issues are benign. In 4-player Commander
  (the target format), they allow illegal cross-player blocking and blocker re-assignment.
- The `defenders_declared` field was clearly intended to prevent re-declaration but the guard
  check was never added. This is likely an oversight during implementation.

---

## M7: Card Definition Framework & First Cards

**Review Status**: REVIEWED (2026-02-22)

### Files Introduced

**Source files — cards:**

| File | Lines | Purpose |
|------|-------|---------|
| `cards/card_definition.rs` | 574 | CardDefinition, AbilityDefinition, Effect (recursive enum, 30+ variants), EffectAmount, EffectTarget, PlayerTarget, TargetRequirement, TargetFilter, Cost, TokenSpec, TypeLine, ZoneTarget, TriggerCondition, Condition, ContinuousEffectDef, ModeSelection, ForEachTarget, LibraryPosition, TimingRestriction |
| `cards/definitions.rs` | 1,230 | 50 hand-authored Commander staple definitions (8 mana rocks, 10 lands, 6 targeted removal, 3 mass removal, 4 counterspells, 7 card draw, 4 ramp spells, 2 equipment, 7 utility creatures) |
| `cards/registry.rs` | 52 | CardRegistry with `Arc<Self>` construction, `get()` by CardId, `empty()` for tests |
| `cards/mod.rs` | 21 | Module declarations and re-exports |

**Source files — effects:**

| File | Lines | Purpose |
|------|-------|---------|
| `effects/mod.rs` | 1,209 | `execute_effect` engine: EffectContext (controller, source, targets, target_remaps), all effect implementations (DealDamage, GainLife, LoseLife, DrawCards, DiscardCards, MillCards, CreateToken, DestroyPermanent, ExileObject, CounterSpell, TapPermanent, UntapPermanent, AddMana, AddManaAnyColor, AddManaChoice, AddCounter, RemoveCounter, MoveZone, SearchLibrary, Shuffle, ApplyContinuousEffect, Conditional, ForEach, Choose, MayPayOrElse, Sequence, Nothing), resolve helpers, token creation, filter matching, condition checking |

**Source files — updates to existing files:**

| File | M7 Changes | Purpose |
|------|------------|---------|
| `rules/resolution.rs` | ~50 lines | Updated `resolve_top_of_stack` to look up `CardDefinition` from `CardRegistry` and call `execute_effect` for Spell/ActivatedAbility/TriggeredAbility kinds |

**Source total**: 3,086 lines new + ~50 lines modified

**Test files:**

| File | Lines | Tests | Focus |
|------|-------|-------|-------|
| `tests/effects.rs` | 578 | 8 | Direct effect execution: DealDamage to player, DealDamage to creature, ExileObject+GainLife (STP), DrawCards, Nothing, Sequence, Conditional true/false |
| `tests/keywords.rs` | 677 | 14 | Keyword enforcement: Defender, Summoning Sickness (3), Flying/Reach (3), Hexproof, Shroud, Indestructible (2), Menace (2), Lifelink |
| `tests/run_all_scripts.rs` | 155 | 1 | Auto-discovery and replay of all approved game scripts |
| `tests/script_replay.rs` | 1,247 | 4 | Replay harness module: `replay_script`, `build_initial_state`, `translate_player_action`, `check_assertions`, `enrich_spec_from_def`, `card_name_to_id` |

**Test total**: 2,657 lines, 27 tests (8 effects + 14 keywords + 1 script runner + 4 replay tests)

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
| CR 106 | Mana production effects: AddMana, AddManaAnyColor, AddManaChoice (`effects/mod.rs:404-445`) |
| CR 111 | Token creation: `make_token` from TokenSpec (`effects/mod.rs:940-997`) |
| CR 118.4 | Life gain/loss effects (`effects/mod.rs:178-210`) |
| CR 119 | Damage to players: DealDamage reduces life, emits DamageDealt+LifeLost (`effects/mod.rs:111-176`) |
| CR 120.3b | Damage to creatures: marks `damage_marked` on creature (`effects/mod.rs:153-159`) |
| CR 120.3c | Damage to planeswalkers: removes loyalty counters (`effects/mod.rs:140-151`) |
| CR 121.1 | Card draw effect: `draw_one_card` helper (`effects/mod.rs:1001-1028`) |
| CR 122 | Counter manipulation: AddCounter, RemoveCounter (`effects/mod.rs:448-497`) |
| CR 608.2 | Spell effect execution: resolution.rs looks up CardDefinition, finds Spell ability, calls execute_effect (`resolution.rs:106-133`) |
| CR 608.3b | Activated/Triggered ability effect execution: resolution.rs looks up ability effect from characteristics (`resolution.rs:168-250`) |
| CR 701.5 | CounterSpell effect: removes from stack, card to graveyard (`effects/mod.rs:329-365`) |
| CR 701.6 | CreateToken effect: builds GameObject from TokenSpec (`effects/mod.rs:241-255`) |
| CR 701.7 | DestroyPermanent effect: checks Indestructible, moves to graveyard (`effects/mod.rs:257-301`) |
| CR 701.7 | DiscardCards effect: deterministic first-by-ObjectId (`effects/mod.rs:1030-1051`) |
| CR 701.13 | MillCards effect: top-of-library to graveyard (`effects/mod.rs:1053-1064`) |
| CR 701.19 | SearchLibrary: deterministic fallback (first matching by ObjectId) (`effects/mod.rs:520-561`) |
| CR 701.20 | Shuffle effect (`effects/mod.rs:563-572`) |
| CR 702.3 | Defender: can't attack (`tests/keywords.rs:52-80`) |
| CR 702.9 | Flying: can't be blocked by ground creatures (`tests/keywords.rs:236-272`) |
| CR 702.10 | Haste: bypasses summoning sickness (`tests/keywords.rs:130-163`) |
| CR 702.11 | Hexproof: opponents can't target (`tests/keywords.rs:416-468`) |
| CR 702.12 | Indestructible: survives lethal damage, not zero toughness (`tests/keywords.rs:472-529`) |
| CR 702.15 | Lifelink: controller gains life equal to combat damage (`tests/keywords.rs:622-677`) |
| CR 702.17 | Reach: can block flying (`tests/keywords.rs:276-314`) |
| CR 702.18 | Shroud: can't be targeted by any player (`tests/keywords.rs:358-412`) |
| CR 702.110 | Menace: must be blocked by 2+ creatures (`tests/keywords.rs:535-616`) |

### Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| MR-M7-01 | **HIGH** | effects/mod.rs:509 | **`MoveZone` effect always emits `ObjectExiled` regardless of destination.** Line 509: `events.push(GameEvent::ObjectExiled { ... })` fires for every MoveZone, even when the destination is Battlefield, Hand, Graveyard, or Library. A creature bounced to hand would emit `ObjectExiled`. **Fix:** match on destination zone and emit the appropriate event (`PermanentEnteredBattlefield` for battlefield, `CardDrawn` or a generic `ObjectMoved` for hand, etc.). | OPEN |
| MR-M7-02 | **HIGH** | casting.rs:162 / card_definition.rs:358 | **`TargetRequirement` filters not validated at cast time.** `validate_targets` (casting.rs:162) only checks existence, zone snapshot, and hexproof/shroud — it does NOT check `TargetRequirement` type restrictions. Comment on line 160 says "Full type-restriction validation deferred to M7" but M7 is now complete and this was not implemented. Doom Blade (`TargetCreatureWithFilter`) can target non-creatures. Negate (`TargetSpell`) can counter creature spells. Any `TargetRequirement` variant beyond basic existence is unenforced. **Fix:** validate each target against its `TargetRequirement` (creature type, permanent type, color filter, etc.) during `validate_targets`. | OPEN |
| MR-M7-03 | **HIGH** | card_definition.rs:639-641 / effects/mod.rs:1090 | **Doom Blade's "non-black" filter semantically inverted.** `TargetFilter { colors: Some([Color::Black]) }` combined with `matches_filter` (line 1090) means "object MUST be black." Doom Blade says "destroy target NON-black creature" — the filter selects the exact opposite set. Additionally, `TargetFilter` has no negation field for colors (unlike `non_land: bool`). **Fix:** add a `non_colors: Option<OrdSet<Color>>` field to `TargetFilter`, or a `negate_colors: bool` flag, and use it for Doom Blade. Even once MR-M7-02 is fixed, this filter would admit only black creatures. | OPEN |
| MR-M7-04 | **HIGH** | effects/mod.rs:914-927 | **`resolve_zone_target` ignores `owner` field — always uses spell controller.** Lines 917-923: all three owner-bearing `ZoneTarget` variants (Graveyard, Hand, Library) discard the `owner: PlayerTarget` field and use the passed `controller` instead. Comment on line 918 says "For simplicity, use controller." For effects like "return target permanent to its owner's hand" (e.g., Unsummon), this puts the card in the CASTER's hand, not the card owner's. Currently no card definition in the 50 uses `MoveZone` for cross-player returns, but this will break when added. **Fix:** resolve the `PlayerTarget` in the `owner` field using `resolve_player_target_list`, or accept a resolved `PlayerId` directly. | OPEN |
| MR-M7-05 | **HIGH** | effects/mod.rs:112 | **`i32` to `u32` cast wraps negative values in DealDamage.** `resolve_amount` returns `i32`; line 112 casts `as u32`. If `EffectAmount::PowerOf` resolves to a negative value (creature with negative power from layer effects), the cast wraps to ~4 billion, dealing massive damage. The `if dmg == 0 { return; }` guard on line 113 doesn't catch wrapping. Same pattern in GainLife (line 179), LoseLife (line 196), DrawCards (line 214: `as usize`). **Fix:** clamp to 0 before casting: `let dmg = resolve_amount(...).max(0) as u32;`. | OPEN |
| MR-M7-06 | **MEDIUM** | effects/mod.rs:1206-1207 | **`ForEachTarget::EachPlayer` and `EachOpponent` return empty vec.** `collect_for_each` returns `Vec<ObjectId>` — player-based ForEach targets return `vec![]` with a comment "players aren't ObjectIds." Effects like "each player draws a card" via ForEach can't work. The `ForEach` combinator only handles object iteration, not player iteration. **Fix:** either refactor `collect_for_each` to return an enum of ObjectId/PlayerId collections, or handle player ForEach variants separately in `execute_effect_inner`. | OPEN |
| MR-M7-07 | **MEDIUM** | effects/mod.rs:900-907 | **`EffectAmount::CardCount` always returns 0.** The match arm has a comment "M7+: implement card counting if needed. Defaults to 0." Any card definition using CardCount for damage/draw amounts produces zero effect. No current definition uses it, but the variant exists and silently fails. **Fix:** implement the zone-card-count logic or remove the variant until needed. | OPEN |
| MR-M7-08 | **MEDIUM** | definitions.rs:697, 738, 788 | **Supreme Verdict "can't be countered" not modeled.** Definition encodes only the destroy-all-creatures effect; the uncounterable restriction is in oracle text but not in abilities. Similarly, Negate's "noncreature spell" restriction uses `TargetSpell` without a filter (no `TargetSpellWithFilter` variant exists). Arcane Denial's delayed draw triggers simplified to immediate draw. These are known simplifications but may confuse script authors expecting correct behavior. | OPEN |
| MR-M7-09 | **MEDIUM** | effects/mod.rs:432-445 | **`AddManaAnyColor` and `AddManaChoice` default to colorless.** Both variants add 1 colorless instead of letting the player choose a color. Affects 4 definitions: Arcane Signet, Command Tower, Birds of Paradise, Darksteel Ingot. Documented as "M9+: interactive mana color choice" but means these cards produce wrong mana (colorless instead of any-color). In most games this would produce incorrect mana pool states. | OPEN → M9 |
| MR-M7-10 | **MEDIUM** | effects/mod.rs:930-936, 542-547 | **`dest_tapped` helper ignores `ZoneTarget::Battlefield { tapped: true }` flag.** `dest_tapped()` takes a `ZoneId` (not `ZoneTarget`) and always returns `Some(false)` for Battlefield. The `tapped: true` flag from `ZoneTarget::Battlefield { tapped: true }` is only captured in the SearchLibrary handler via a separate check (lines 542-548 manually apply `tapped`). If other effects use `dest_tapped` expecting it to detect "enters tapped," they'll get the wrong answer. The function is misleading. **Fix:** pass `ZoneTarget` instead of `ZoneId`, or remove the function and inline the tapped check. | OPEN |
| MR-M7-11 | **MEDIUM** | definitions.rs:946-961 | **Brainstorm only draws 3 — does not put 2 cards back.** Oracle text says "Draw three cards, then put two cards from your hand on top of your library in any order." The effect is `DrawCards { count: 3 }` only — the put-back is missing entirely. Comment says "simplified as draw without scry" (confusing Brainstorm with a scry card). This makes Brainstorm strictly better than intended — pure card advantage instead of card selection. | OPEN |
| MR-M7-12 | **MEDIUM** | definitions.rs:509-539 | **Path to Exile's search is unconditional but should be "may".** Oracle says the exiled creature's controller "may search their library." Currently encoded as an unconditional `SearchLibrary`, which always fetches a land. Requires interactive choice (`MayPayOrElse` pattern) for correctness. Note: `MayPayOrElse` already exists in the Effect enum and defaults to "don't pay → apply or_else" — could model as `MayPayOrElse { cost: ..., payer: ControllerOf, or_else: Nothing }` with SearchLibrary in the "may" branch once M9 interactive choice is implemented. | OPEN → M9 |
| MR-M7-13 | **LOW** | definitions.rs:1083-1108 | **Equipment definitions have empty ability lists.** Lightning Greaves and Swiftfoot Boots have `abilities: vec![]` — they are blank permanents on the battlefield. The equip cost, keyword grants (haste+shroud / haste+hexproof) are described only in oracle text. These cards do nothing until the equipment/attach system is implemented. Expected for M7; flagged for completeness. | OPEN → M8+ |
| MR-M7-14 | **LOW** | definitions.rs:160-177, 362-396 | **"No maximum hand size" ability not modeled.** Thought Vessel, Reliquary Tower, and Rogue's Passage second ability ({4},{T}: unblockable) are described in oracle text but not encoded in abilities. Only the tap-for-mana ability exists on each. | OPEN → M8+ |
| MR-M7-15 | **LOW** | tests/effects.rs | **No test for CreateToken or CounterSpell effects.** The 8 effect tests cover DealDamage (2), ExileObject+GainLife (1), DrawCards (1), Nothing (1), Sequence (1), Conditional (2). Token creation and spell countering are exercised only by game scripts, not by direct effect unit tests. | OPEN |
| MR-M7-16 | **LOW** | tests/ | **No combat game script.** All 7 approved scripts cover baseline and stack interactions. No script exercises the combat system (declare attackers/blockers, damage step) through the replay harness. Combat is tested via unit tests in `tests/combat.rs` and `tests/keywords.rs`, but not via the engine-independent script format. | OPEN |
| MR-M7-17 | **LOW** | effects/mod.rs:567 | **Shuffle uses `from_entropy()` — non-deterministic in effect execution.** `SearchLibrary` and `Shuffle` effects call `rand::rngs::StdRng::from_entropy()` (line 567), making the shuffle non-deterministic across replays. Tests use seeded RNG at the `GameState` level, but effect-level shuffles bypass this. For replay determinism, the shuffle should use the state's seeded RNG. | OPEN |
| MR-M7-18 | **LOW** | script_replay.rs:700-718 | **`try_as_tap_mana_ability` only converts single-mana single-color abilities.** Sol Ring ({T}: Add {C}{C}) is NOT converted to a ManaAbility by `enrich_spec_from_def` because it produces 2 colorless. Scripts must use ActivateAbility instead of TapForMana for Sol Ring. Documented in CLAUDE.md but creates an asymmetry. | OPEN |
| MR-M7-19 | **INFO** | card_definition.rs | **Effect DSL is well-designed and extensible.** The recursive `Effect` enum with `Box` for Conditional/ForEach/MayPayOrElse avoids stack overflow. `EffectTarget` and `PlayerTarget` properly separate object and player targeting. `EffectContext.target_remaps` elegantly solves the STP "exile then reference power" pattern. `Condition` enum covers the main intervening-if patterns. | — |
| MR-M7-20 | **INFO** | effects/mod.rs | **No `unwrap()`/`expect()` in effect execution code.** All object lookups use `if let Some(obj)` or `.get()` with graceful fallback. Contrast with earlier milestones' `.unwrap()` violations. Clean engine-library code. | — |
| MR-M7-21 | **INFO** | script_replay.rs | **Script replay harness well-structured.** Deterministic player ordering (alphabetical sort → sequential PlayerId). Comprehensive assertion paths (life, poison, zone counts, includes/excludes, permanent status, counters). `enrich_spec_from_def` correctly bridges `ObjectSpec::card()` to engine expectations. Unknown actions skipped gracefully (future-proof). | — |
| MR-M7-22 | **INFO** | definitions.rs | **Card definitions cover a good cross-section of Commander staples.** 50 cards across 9 categories. All creatures have printed P/T. Mana costs are correct. `..Default::default()` used consistently for non-creature cards. Oracle text matches official Scryfall text. Token specs for Beast Within, Generous Gift, Swan Song manually verified correct. | — |
| MR-M7-23 | **INFO** | registry.rs | **CardRegistry is clean and minimal.** `new()` returns `Arc<Self>` (avoids double-wrapping documented in CLAUDE.md). `HashMap<CardId, CardDefinition>` is the right choice for O(1) lookup (not shared across state snapshots since it's static). `empty()` for tests that don't need card effects. | — |

### Test Coverage Assessment

| M7 Behavior | Coverage | Notes |
|-------------|----------|-------|
| DealDamage to player | Good (1 test) | Lightning Bolt → player life decreases, DamageDealt event |
| DealDamage to creature | Good (1 test) | Lightning Bolt → damage marked, SBA kills creature |
| ExileObject + GainLife | Good (1 test) | STP → exile + power-based life gain with target_remaps |
| DrawCards | Good (1 test) | Divination → draws 2, hand size changes |
| Nothing effect | Good (1 test) | No events, no state change |
| Sequence combinator | Good (1 test) | GainLife+LoseLife → net correct |
| Conditional combinator | Good (2 tests) | True/false branches both verified |
| CreateToken | **Missing** (0 tests) | Only exercised via game scripts |
| CounterSpell | **Missing** (0 tests) | Only exercised via game scripts |
| DestroyPermanent | **Missing** (0 tests) | Exercised indirectly via SBA tests |
| ForEach combinator | **Indirect** | Used by Wrath of God definition but no direct unit test |
| Keyword: Defender | Good (1 test) | Can't attack |
| Keyword: Summoning Sickness | Good (3 tests) | Prevents attack, cleared after untap, haste bypass |
| Keyword: Flying/Reach | Good (3 tests) | Ground can't block, reach can, flying can |
| Keyword: Hexproof/Shroud | Good (2 tests) | Opponent can't target, nobody can target |
| Keyword: Indestructible | Good (2 tests) | Survives lethal, dies to zero toughness |
| Keyword: Menace | Good (2 tests) | Requires 2 blockers, allows 2 blockers |
| Keyword: Lifelink | Good (1 test) | Life gained on combat damage |
| Script replay | Good (7 scripts) | Baseline + stack scenarios all approved |
| Script auto-discovery | Good (1 test) | Runs all approved scripts, vacuous pass if none |
| Card definitions | Adequate | 50 cards defined; correctness verified by script replay |

### Notes

- **Effect execution is clean.** No unwrap/expect violations, graceful fallback for missing objects, proper event emission for most effects. The `EffectContext.target_remaps` design is particularly good.
- **MR-M7-02 is the most impactful finding** — `TargetRequirement` filter validation was explicitly deferred to M7 in the casting.rs comments, but M7 completed without implementing it. This means ALL type restrictions (TargetCreature, TargetPermanent, TargetCreatureWithFilter, etc.) are unenforced at cast time. Players can target anything with any spell.
- **MR-M7-05 (i32→u32 wrapping) is a correctness bug** that will surface as soon as a creature has negative power from layer effects and is referenced by a DealDamage/GainLife/LoseLife effect.
- **Simplification decisions are well-documented** — AddManaAnyColor → colorless, Choose → first option, MayPayOrElse → don't pay, SearchLibrary → first match. These are all tagged with "M9+" comments.
- **The 50 card definitions are a good foundation** — they cover the most common Commander staples across multiple effect categories. The incomplete definitions (equipment, some abilities) are clearly documented.

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
| MR-M6-01 | M6 | Attack target not validated — accepts self, non-existent, own planeswalker | OPEN |
| MR-M6-02 | M6 | Double-blocking not prevented — same creature can block multiple attackers | OPEN |
| MR-M6-03 | M6 | Partial blocker ordering accepted — OrderBlockers doesn't require completeness | OPEN |
| MR-M6-09 | M6 | Cross-player blocking — blocker can block attacker targeting a different player | OPEN |
| MR-M6-10 | M6 | `defenders_declared` tracked but never enforced — same player can re-declare | OPEN |
| MR-M7-01 | M7 | `MoveZone` always emits `ObjectExiled` regardless of destination zone | OPEN |
| MR-M7-02 | M7 | `TargetRequirement` filters not validated at cast time — all type restrictions unenforced | OPEN |
| MR-M7-03 | M7 | Doom Blade "non-black" filter semantically inverted — `colors` field is inclusion-only | OPEN |
| MR-M7-04 | M7 | `resolve_zone_target` ignores `owner` field — always uses spell controller | OPEN |
| MR-M7-05 | M7 | `i32→u32` cast wraps negative values in DealDamage/GainLife/LoseLife/DrawCards | OPEN |

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
| MR-M6-11 | M6 | Menace check counts raw entries not unique creatures — depends on MR-M6-02 | OPEN |
| MR-M6-12 | M6 | Redundant `calculate_characteristics` for vigilance in declare_attackers | OPEN |
| MR-M7-06 | M7 | `ForEachTarget::EachPlayer/EachOpponent` returns empty vec — player iteration broken | OPEN |
| MR-M7-07 | M7 | `EffectAmount::CardCount` always returns 0 — unimplemented stub | OPEN |
| MR-M7-08 | M7 | Supreme Verdict "can't be countered", Negate "noncreature" restrictions not modeled | OPEN |
| MR-M7-09 | M7 | AddManaAnyColor/AddManaChoice default to colorless — 4 cards produce wrong mana | OPEN → M9 |
| MR-M7-10 | M7 | `dest_tapped` takes ZoneId not ZoneTarget — ignores "enters tapped" flag | OPEN |
| MR-M7-11 | M7 | Brainstorm only draws 3 — does not put 2 cards back on library | OPEN |
| MR-M7-12 | M7 | Path to Exile's search unconditional — should be "may" | OPEN → M9 |

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
| MR-M6-06 | M6 | `apply_combat_damage` is 312 lines — extract attacker/blocker helpers | OPEN |
| MR-M6-08 | M6 | Test gap: no combat game script in replay harness | OPEN |
| MR-M6-13 | M6 | Test gap: blocker-removed-before-damage (CR 509.1h) untested | OPEN |
| MR-M6-14 | M6 | `blockers_for()` rebuilds list on every call — O(n) in hot path | OPEN |
| MR-M7-13 | M7 | Equipment definitions (Lightning Greaves, Swiftfoot Boots) have empty ability lists | OPEN → M8+ |
| MR-M7-14 | M7 | "No maximum hand size" ability not modeled (Thought Vessel, Reliquary Tower) | OPEN → M8+ |
| MR-M7-15 | M7 | Test gap: no CreateToken or CounterSpell effect unit tests | OPEN |
| MR-M7-16 | M7 | Test gap: no combat game script in replay harness | OPEN |
| MR-M7-17 | M7 | Shuffle in effects uses `from_entropy()` — non-deterministic across replays | OPEN |
| MR-M7-18 | M7 | `try_as_tap_mana_ability` doesn't convert multi-mana abilities (Sol Ring) | OPEN |

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
| MR-M6-04 | M6 | `is_blocked` correctly implements CR 509.1h — blocked status persists (stub resolved) | — |
| MR-M6-05 | M6 | i32→u32 cast safe — `power <= 0` guard prevents negative values (stub resolved) | — |
| MR-M6-07 | M6 | Defender tested in keywords.rs — `test_702_3_defender_cannot_attack` (stub resolved) | — |
| MR-M6-15 | M6 | Blocked status persistence design correct per CR 509.1h | — |
| MR-M6-16 | M6 | Two-phase collect+apply prevents use-after-free; simultaneous damage per CR 510.2 | — |
| MR-M6-17 | M6 | Combat state lifecycle correct — init at BeginningOfCombat, cleared at EndOfCombat | — |

---

## Statistics

| Metric | Value |
|--------|-------|
| Total unique issue IDs | 123 (MR-M5-02 cross-refs MR-M4-02; MR-M3-05/06 cross-ref M7; MR-M6-11 depends on MR-M6-02) |
| CRITICAL | 0 |
| HIGH (OPEN) | 22 |
| HIGH (DEFERRED) | 2 |
| MEDIUM (OPEN) | 25 |
| MEDIUM (DEFERRED) | 4 |
| LOW (OPEN) | 27 |
| LOW (DEFERRED) | 5 |
| INFO | 37 |
| Milestones fully reviewed | 8 (M0, M1, M2, M3, M4, M5, M6, M7) |
| Milestones not started | 2 (M8, M9) |

**Engine source LOC (M0-M7)**: ~12,500 lines
**Engine test LOC (M1-M7)**: ~14,600 lines
**Total test count**: 303 (all passing)
