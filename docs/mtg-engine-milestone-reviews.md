# MTG Engine — Milestone Code Reviews

> **Purpose**: Per-milestone code review tracking. Records files introduced, CR sections
> implemented, findings (bugs, enforcement gaps, code quality, test gaps), and deferred
> issues. Updated as milestones complete and issues are discovered or resolved.
>
> **Convention**: Every completed milestone MUST have its new/changed files reviewed and
> findings added to this document before the milestone is considered done. This is a
> required step in the Milestone Completion Checklist (see CLAUDE.md).
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

**Review Status**: STUB — to be reviewed

### Files Introduced

**Source files:**

| File | Lines | Purpose |
|------|-------|---------|
| `rules/command.rs` | 113 | Command enum (PassPriority, Concede, + later variants) |
| `rules/engine.rs` | 394 | `process_command()` entry point, `start_game()`, step entry |
| `rules/events.rs` | 398 | GameEvent enum (14+ variants), LossReason |
| `rules/priority.rs` | 104 | APNAP ordering, pass_priority, grant_initial_priority |
| `rules/turn_structure.rs` | 132 | STEP_ORDER, advance_step, advance_turn, next_player |
| `rules/turn_actions.rs` | 273 | Untap, draw, cleanup discard/damage, mana pool empty |
| `rules/mod.rs` | 23 | Module exports |

**Source total**: 1,437 lines

**Test files:**

| File | Lines | Tests | Focus |
|------|-------|-------|-------|
| `tests/turn_structure.rs` | 237 | ~6 | Step advancement, phase transitions |
| `tests/priority.rs` | 151 | ~7 | APNAP ordering, priority pass/grant |
| `tests/turn_actions.rs` | 248 | ~7 | Untap, draw, cleanup, mana empty |
| `tests/extra_turns.rs` | 116 | ~4 | Extra turn LIFO, normal resumption |
| `tests/concede.rs` | 134 | ~5 | Concede, elimination, skipping |
| `tests/turn_invariants.rs` | 104 | ~4 | Proptest invariants |

**Test total**: 990 lines, ~33 tests

### CR Sections Implemented

| CR Section | Implementation |
|------------|----------------|
| CR 500.1-4 | Turn structure, mana pool emptying |
| CR 501-514 | All steps/phases |
| CR 502.2 | Untap step |
| CR 504.1 | Draw step |
| CR 514 | Cleanup (discard, damage clear) |
| CR 103.8 | First player skips first draw |
| CR 104.3b | Empty library → player loses |
| CR 116 | Priority system (APNAP) |

### Known Issues (to be validated during review)

| ID | Severity | File | Description | Status |
|----|----------|------|-------------|--------|
| MR-M2-01 | **HIGH** | priority.rs | **`.expect()` in engine library code.** Exact location and context to be verified during review. | STUB |
| MR-M2-02 | **HIGH** | turn_structure.rs | **`.expect()` in engine library code.** Exact location and context to be verified during review. | STUB |

---

## M3: Stack, Spells & Abilities

**Review Status**: STUB — to be reviewed

### Files Introduced

**Source files — state:**

| File | Lines | Purpose |
|------|-------|---------|
| `state/stack.rs` | 64 | StackObject, StackObjectKind |
| `state/hash.rs` | 1,223 | HashInto trait, blake3 hashing, public/private hash |
| `state/targeting.rs` | 36 | Target, SpellTarget types |

**Source files — rules:**

| File | Lines | Purpose |
|------|-------|---------|
| `rules/mana.rs` | 112 | CR 605 mana ability handler |
| `rules/lands.rs` | 107 | CR 305.1 land play handler |
| `rules/casting.rs` | 302 | CR 601 spell casting, cost payment |
| `rules/resolution.rs` | 355 | CR 608 stack resolution, fizzle, counter |
| `rules/abilities.rs` | 448 | CR 602-603 activated/triggered abilities, APNAP, intervening-if |

**Source files — testing:**

| File | Lines | Purpose |
|------|-------|---------|
| `testing/script_schema.rs` | 325 | GameScript JSON schema types |
| `testing/mod.rs` | 9 | Module exports |

**Source total**: 2,981 lines

**Test files:**

| File | Lines | Tests | Focus |
|------|-------|-------|-------|
| `tests/mana_and_lands.rs` | 660 | ~19 | Mana abilities, land plays |
| `tests/casting.rs` | 550 | ~12 | CastSpell, casting speed, priority reset |
| `tests/resolution.rs` | 626 | ~10 | Stack resolution, permanent ETB |
| `tests/targeting.rs` | 742 | ~13 | Target validation, fizzle, partial fizzle, mana cost |
| `tests/abilities.rs` | 852 | ~15 | Activated/triggered abilities, APNAP, intervening-if |
| `tests/state_hashing.rs` | 477 | ~19 | Determinism, sensitivity, partitioning |
| `tests/script_schema.rs` | 128 | ~3 | JSON round-trip tests |

**Test total**: 4,035 lines, ~91 tests

### CR Sections Implemented

| CR Section | Implementation |
|------------|----------------|
| CR 305.1 | Land plays (sorcery speed, one per turn) |
| CR 601 | Spell casting (speed validation, stack entry, priority reset) |
| CR 601.2c-h | Target selection, mana cost payment |
| CR 602 | Activated abilities |
| CR 603 | Triggered abilities (check at event, flush before priority) |
| CR 603.4 | Intervening-if (checked at trigger AND resolution) |
| CR 605 | Mana abilities (special action, no priority change) |
| CR 608 | Stack resolution (LIFO, permanent vs instant/sorcery) |
| CR 608.2b | Countering a spell |

### Known Issues (to be validated during review)

| ID | Severity | File | Description | Status |
|----|----------|------|-------------|--------|
| MR-M3-01 | **HIGH** | resolution.rs | **Partial fizzle: targets not filtered.** When some (but not all) targets are illegal, `resolve_top_of_stack` proceeds with all original targets instead of filtering to legal ones only. Effect executes against illegal targets. | STUB |
| MR-M3-02 | **MEDIUM** | casting.rs | **ManaCostPaid not emitted for {0} cost spells.** `pay_cost` skips if mana cost is zero, so no ManaCostPaid event fires. Matters if triggers key on ManaCostPaid. | STUB |

---

## M4: State-Based Actions

**Review Status**: STUB — to be reviewed

### Files Introduced

| File | Lines | Purpose |
|------|-------|---------|
| `rules/sba.rs` | 587 | `check_and_apply_sbas` fixed-point loop, all CR 704.5 checks |

**Source total**: 587 lines

**Test files:**

| File | Lines | Tests | Focus |
|------|-------|-------|-------|
| `tests/sba.rs` | 756 | ~28 | All SBA checks, batch behavior, events |

### CR Sections Implemented

| CR Section | Implementation |
|------------|----------------|
| CR 704.5a | Player at 0 or less life loses |
| CR 704.5c | 10+ poison counters |
| CR 704.5d | Token in non-battlefield zone ceases to exist |
| CR 704.5f | Creature toughness ≤ 0 |
| CR 704.5g | Lethal damage on creature |
| CR 704.5h | Deathtouch damage |
| CR 704.5i | Planeswalker at 0 loyalty |
| CR 704.5j | Legendary rule (auto-keeps newest) |
| CR 704.5m | Aura attached to illegal object |
| CR 704.5n | Equipment attached illegally |
| CR 704.5q | +1/+1 and -1/-1 counter annihilation |
| CR 704.5u | Commander damage 21+ |

### Known Issues (to be validated during review)

| ID | Severity | File | Description | Status |
|----|----------|------|-------------|--------|
| MR-M4-01 | **HIGH** | sba.rs | **`.unwrap()` in engine library code.** At least two locations. Exact lines to be verified. | STUB |
| MR-M4-02 | **HIGH** | sba.rs | **SBAs don't use `calculate_characteristics` for P/T checks.** CR 704.5f/g/h check raw `game_object.characteristics.toughness` instead of the layer-calculated value. A creature with base 1/1 and a -1/-1 effect would not be caught. Should use layer system (available since M5). | DEFERRED → M5 review |
| MR-M4-03 | **HIGH** | sba.rs | **Second `.unwrap()` in engine library code.** Same pattern as MR-M4-01. | STUB |

### Notes

- MR-M4-02 is a cross-milestone issue: code is in M4 but the fix requires M5's layer system.
  Should be addressed when the SBA + layer integration is reviewed.

---

## M5: The Layer System

**Review Status**: STUB — to be reviewed

### Files Introduced

| File | Lines | Purpose |
|------|-------|---------|
| `state/continuous_effect.rs` | 207 | EffectId, EffectLayer, EffectDuration, EffectFilter, LayerModification, ContinuousEffect |
| `rules/layers.rs` | 497 | `calculate_characteristics`, dependency detection, toposort, effect expiry |

**Source total**: 704 lines

**Test files:**

| File | Lines | Tests | Focus |
|------|-------|-------|-------|
| `tests/layers.rs` | 1,392 | ~28 | Humility+Opalescence, Blood Moon+Urborg, CDAs, layer ordering, duration expiry |

### CR Sections Implemented

| CR Section | Implementation |
|------------|----------------|
| CR 613 | Full layer system (layers 1-7d) |
| CR 613.3 | CDAs applied before other effects in their layer |
| CR 613.4c | Counter P/T modifications at layer 7c |
| CR 613.7 | Timestamp ordering within layers |
| CR 613.8 | Dependency ordering (overrides timestamp) |
| CR 613.8k | Circular dependencies fall back to timestamp |

### Known Issues (to be validated during review)

| ID | Severity | File | Description | Status |
|----|----------|------|-------------|--------|
| MR-M5-01 | **HIGH** | layers.rs | **`.expect()` in engine library code.** Exact location to be verified. | STUB |
| MR-M5-02 | **MEDIUM** | layers.rs | **`ptr::eq` used for effect comparison.** May have correctness issues with im-rs structural sharing (two logically equal values at different addresses). Context and impact to be verified. | STUB |
| MR-M5-03 | **HIGH** | (cross: sba.rs) | **SBAs don't use `calculate_characteristics`.** Forwarded from MR-M4-02. SBA toughness checks should call `calculate_characteristics` now that the layer system exists. | STUB |

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
| MR-M2-01 | M2 | `.expect()` in priority.rs | STUB |
| MR-M2-02 | M2 | `.expect()` in turn_structure.rs | STUB |
| MR-M4-01 | M4 | `.unwrap()` in sba.rs (first instance) | STUB |
| MR-M4-02 | M4→M5 | SBAs don't use `calculate_characteristics` for P/T | STUB |
| MR-M4-03 | M4 | `.unwrap()` in sba.rs (second instance) | STUB |
| MR-M5-01 | M5 | `.expect()` in layers.rs | STUB |
| MR-M5-03 | M5 | SBAs don't use `calculate_characteristics` (cross-ref MR-M4-02) | STUB |
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
| MR-M3-01 | M3 | Partial fizzle targets not filtered | STUB |
| MR-M3-02 | M3 | ManaCostPaid not emitted for {0} cost | STUB |
| MR-M5-02 | M5 | `ptr::eq` in layers.rs | STUB |
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

---

## Statistics

| Metric | Value |
|--------|-------|
| Total unique issue IDs | 56 (MR-M5-03 cross-refs MR-M4-02) |
| CRITICAL | 0 |
| HIGH (OPEN) | 4 |
| HIGH (STUB) | 13 |
| MEDIUM (OPEN) | 8 |
| MEDIUM (STUB) | 12 |
| LOW (OPEN) | 5 |
| LOW (STUB) | 8 |
| INFO | 6 |
| Milestones fully reviewed | 2 (M0, M1) |
| Milestones with stubs | 5 (M2, M3, M4, M5, M6, M7) |
| Milestones not started | 2 (M8, M9) |

**Engine source LOC (M0-M7)**: ~12,500 lines
**Engine test LOC (M1-M7)**: ~14,600 lines
**Total test count**: 303 (all passing)
