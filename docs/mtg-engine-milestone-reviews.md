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
> **Last Updated**: 2026-02-23 (M9 reviewed)

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
- [M8: Replacement & Prevention Effects](#m8-replacement--prevention-effects) **(REVIEWED)**
- [M9: Commander Rules Integration](#m9-commander-rules-integration) **(REVIEWED)**
- [Cross-Milestone Issue Index](#cross-milestone-issue-index)

---

## M0: Project Scaffold & Data Foundation

**Review Status**: RE-REVIEWED (2026-02-22) — original review covered tools only; re-review adds engine state module coverage and corrects one false positive

### Files Introduced

**Tools & Infrastructure** (original review scope):

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

**Engine State Module** (added in re-review — M0 scaffolded these types, later milestones expanded):

| File | Lines (current) | Purpose |
|------|-----------------|---------|
| `crates/engine/src/state/mod.rs` | 268 | GameState struct, zone/object accessors, move_object_to_zone |
| `crates/engine/src/state/types.rs` | 105 | Color, ManaColor, SuperType, CardType, SubType, CounterType, KeywordAbility |
| `crates/engine/src/state/player.rs` | 83 | PlayerId, CardId, ManaPool, PlayerState |
| `crates/engine/src/state/zone.rs` | 185 | ZoneType, ZoneId, Zone (Ordered/Unordered storage) |
| `crates/engine/src/state/game_object.rs` | 221 | ObjectId, ManaCost, ManaAbility, Characteristics, GameObject |
| `crates/engine/src/state/turn.rs` | 121 | Phase, Step, TurnState |
| `crates/engine/src/state/stack.rs` | 64 | StackObject, StackObjectKind |
| `crates/engine/src/state/targeting.rs` | 36 | Target, SpellTarget |
| `crates/engine/src/state/combat.rs` | 105 | CombatState, AttackTarget |
| `crates/engine/src/state/continuous_effect.rs` | 207 | ContinuousEffect, EffectLayer, LayerModification |
| `crates/engine/src/state/error.rs` | 75 | GameStateError enum (15 variants) |
| `crates/engine/src/state/stubs.rs` | 43 | DelayedTrigger, ReplacementEffect, PendingTrigger |
| `crates/engine/src/state/builder.rs` | 676 | GameStateBuilder, ObjectSpec, PlayerBuilder |
| `crates/engine/src/state/hash.rs` | 1223 | blake3 state hashing, HashInto trait |

**Source total**: ~4,688 lines (tools ~1,276 + state ~3,412) | **Tests**: rules_db.rs inline, schema.rs inline

### CR Sections Implemented

None directly — M0 is infrastructure. CR text is parsed and indexed for lookup.
State types cite CR sections in documentation (109.3, 400.7, 500-514, 611, 613, etc.)
but enforce no rules — that's M1+.

### Findings

**Status changes from re-review**:

| ID | Original | Revised | Reason |
|----|----------|---------|--------|
| MR-M0-01 | HIGH | ~~CLOSED — FALSE POSITIVE~~ | Both `import_cards` and `import_rulings` wrap DELETE+INSERT in an explicit `conn.transaction()` (lines 142/230). Any failure between DELETE and INSERT causes the transaction to roll back, preserving the prior data. WAL mode further protects against process-kill corruption. The original finding's recommendation ("wrap in explicit transaction with rollback") is already implemented. |

**Confirmed findings** (no change):

| ID | Severity | File | Description | Status |
|----|----------|------|-------------|--------|
| MR-M0-02 | **HIGH** | mcp-server/main.rs | **FTS5 MATCH operator injection.** User queries passed directly to `WHERE rules_fts MATCH ?1`. FTS5 interprets operators (`AND`, `OR`, `NOT`, quotes, parentheses). Malformed queries can cause FTS5 parse errors. Parameterized queries protect against SQL injection but not FTS syntax injection. Fix: wrap input in double-quotes to force literal matching. | CLOSED — fix session 9 |
| MR-M0-03 | **MEDIUM** | mcp-server/rules_db.rs | **Multi-line CR rules not fully captured.** Parser treats each line independently. CR rules spanning 2+ lines in the text file have only the first line captured as `rule_text`. Affects completeness of FTS index. | CLOSED — fix session 9 |
| MR-M0-04 | **MEDIUM** | mcp-server/rules_db.rs | **CR format assumptions fragile.** Parsing relies on "Glossary" and "Credits" as exact case-sensitive stop markers, position-based detection (after `seen_rules`). No CR version metadata captured. If Wizards changes the format or casing, import silently produces fewer rules with no validation. | CLOSED — fix session 9 |
| MR-M0-05 | **MEDIUM** | mcp-server/main.rs | **FTS index probe is fragile.** Lines ~441-454: probes for the word "the" to detect if FTS index is populated. Not guaranteed to exist in all future CR revisions. Should query table count or metadata directly. | CLOSED — fix session 9 |
| MR-M0-06 | **MEDIUM** | scryfall-import/main.rs | **JSON parse errors lose context.** `serde_json::from_reader()` on a ~200MB file gives no indication of which card or line caused the failure. Debugging reimport failures is painful. | CLOSED — fix session 9 |
| MR-M0-07 | **MEDIUM** | scryfall-import/main.rs | **No download integrity check.** Bulk downloads have no timeout, resumption, or checksum validation. Corrupted download produces silent parse failures. | CLOSED — fix session 9 |
| MR-M0-08 | **LOW** | card-db/schema.rs | **No ON DELETE CASCADE** for `card_faces.card_id` FK. Orphaned card_faces possible if cards are deleted without cascading. Not a practical issue with current delete-all pattern, but schema doesn't enforce it. | OPEN |
| MR-M0-09 | **LOW** | card-db/schema.rs | **JSON columns stored as TEXT.** `colors`, `color_identity`, `keywords`, `legalities` are TEXT not JSON type. Requires callers to always serialize correctly. Risk of accidental string matching instead of JSON queries. | OPEN |
| MR-M0-10 | **LOW** | mcp-server/main.rs | **Partial card name matching too broad.** `name LIKE '%' || ?1 || '%'` matches substrings ("Sol" matches "Sollen's Zendikon"). Single-letter queries return hundreds of results before LIMIT. | OPEN |
| MR-M0-11 | **INFO** | card-db/lib.rs | Clean error types, WAL mode, foreign keys enabled. No issues. | — |
| MR-M0-12 | **INFO** | mcp-server/rules_db.rs | Good test coverage: section headers, rule lines, parent computation, TOC edge cases. | — |

**New findings from re-review**:

| ID | Severity | File | Description | Status |
|----|----------|------|-------------|--------|
| MR-M0-13 | **MEDIUM** | state/mod.rs | **`move_object_to_zone` non-atomic — removes source before validating destination.** Lines 188-201: removes object from source zone (line 193) and objects map (line 198), then checks destination zone exists (line 201). If destination is invalid, the object is gone and the state is corrupted. In practice, `process_command` clones state before mutation so the caller's state is safe, but `move_object_to_zone` is a public method — any direct caller without pre-cloning gets silent corruption. Fix: validate destination zone existence before removing from source. | CLOSED — fix session 5 |
| MR-M0-14 | **MEDIUM** | scryfall-import/main.rs | **Entire ~200MB JSON deserialized into memory.** Line 136: `serde_json::from_reader(reader)` materializes the full `Vec<ScryfallCard>` (~37K cards). With Serde overhead this can consume 1-2GB RAM. A streaming/iterative parser (serde_json `StreamDeserializer` or `simd-json`) would be more robust for CI/automated use. | CLOSED — fix session 9 |
| MR-M0-15 | **LOW** | rules_db.rs | **`rulings_fts` missing UPDATE/DELETE triggers.** Lines 65-68: only an INSERT trigger is defined for `rulings_fts`, unlike `rules_fts` which has INSERT, DELETE, and UPDATE triggers. Individual ruling updates/deletes would leave the FTS index stale. Currently mitigated by bulk `rebuild_rulings_fts()`, but inconsistent and fragile if usage patterns change. | OPEN |
| MR-M0-16 | **LOW** | scryfall-import/main.rs | **Cards without `oracle_id` stored as empty string.** Line 165: `oracle_id.as_deref().unwrap_or("")`. Non-game cards (tokens, art series, emblems) without an oracle_id all share `oracle_id = ""`, so ruling lookups could cross-contaminate. Low impact because these cards are filtered out by the MCP server's layout exclusion. | OPEN |
| MR-M0-17 | **INFO** | state/ (all) | **Engine state module type design is strong.** `ZoneId` makes invalid per-player/shared zone combinations unrepresentable. `Zone` enum separates ordered vs unordered storage. `OrdMap`/`OrdSet` ensure deterministic iteration. `ObjectId` newtype prevents ID confusion. Commander-specific fields (tax, damage tracking, partner support) are first-class. | — |
| MR-M0-18 | **INFO** | state/hash.rs | **State hashing framework well-designed.** Length-prefixed strings prevent concatenation collisions. Discriminant bytes on all enums. Public hash excludes hidden zones (hand/library contents) but includes their sizes. Private hash per-player. Cross-platform determinism via explicit `to_le_bytes()`. | — |

### Notes

- M0 files are **tools/binaries**, not core engine. `unwrap()`/`expect()` and `anyhow` are
  acceptable per project conventions. Findings focus on data integrity and parsing fragility.
- `card-db` is a library crate using `thiserror` — correct pattern.
- The MCP server is consumed by Claude Code only (trusted input), so the FTS injection risk
  is low-probability but should be fixed for defense-in-depth.
- **Re-review coverage note**: The original M0 review examined only the tools/infrastructure
  files (~1,276 lines). The engine state module (~3,412 lines) was scaffolded in M0 but not
  reviewed until this re-review. Many state files evolved significantly in M1-M7; findings
  from those later changes are attributed to their respective milestone reviews (e.g.,
  MR-M3-03 for `has_summoning_sickness` hash omission, MR-M3-05/06 for `effect` field hash
  omissions). The state module findings here (MR-M0-13 through MR-M0-18) cover issues
  present since M0 that were not caught in later reviews.
- **MR-M0-01 false positive**: The original finding recommended wrapping deletes in a
  transaction — but the code already does exactly this (`conn.transaction()` at lines 142
  and 230). Both `import_cards` and `import_rulings` DELETE within an explicit transaction
  that only commits on success. This was the only HIGH-severity tool finding; with it
  closed, the remaining HIGH is MR-M0-02 (FTS injection).

---

## M1: Game State & Object Model

**Review Status**: RE-REVIEWED (2026-02-22) — original review covered all M1 files; re-review adds 12 new findings (1 MEDIUM, 8 LOW, 3 INFO) and confirms all original findings still open

### Files Introduced

**Source files (line counts reflect current state after M1-M7 evolution):**

| File | Lines | Purpose |
|------|-------|---------|
| `crates/engine/src/state/types.rs` | 105 | Color, ManaColor, SuperType, CardType, SubType, CounterType, KeywordAbility enums |
| `crates/engine/src/state/player.rs` | 83 | PlayerId, CardId, ManaPool, PlayerState (Commander fields) |
| `crates/engine/src/state/game_object.rs` | 221 | ObjectId, ManaCost, ManaAbility, ActivatedAbility, TriggeredAbilityDef, Characteristics, ObjectStatus, AbilityInstance, GameObject |
| `crates/engine/src/state/zone.rs` | 185 | ZoneId, ZoneType, Zone (Ordered/Unordered), operations (insert, remove, shuffle, insert_at, top) |
| `crates/engine/src/state/turn.rs` | 121 | Phase, Step (with phase mapping, priority, next), TurnState |
| `crates/engine/src/state/error.rs` | 75 | GameStateError enum (15 variants, thiserror) |
| `crates/engine/src/state/builder.rs` | 676 | GameStateBuilder (6 player/config methods), ObjectSpec (6 constructors + 17 fluent setters), PlayerBuilder |
| `crates/engine/src/state/stubs.rs` | 43 | DelayedTrigger, ReplacementEffect, PendingTrigger (placeholders) |
| `crates/engine/src/state/mod.rs` | 268 | GameState struct (13 fields), next_object_id, accessors, add_object, move_object_to_zone, active_players, objects_in_zone |
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
| CR 106.4 | ManaPool with per-color tracking (`state/player.rs`) |
| CR 110.5 | ObjectStatus: tapped, flipped, face_down, phased_out (`state/game_object.rs`) |
| CR 302.6 | Summoning sickness set on battlefield entry (`state/mod.rs:224`) |
| CR 500-514 | Phase/Step enums with correct ordering (`state/turn.rs`) |
| CR 903.7/903.10a | Commander starting life (40), commander_tax, commander_damage_received (`state/player.rs`) |

### Findings

**Original findings status (all confirmed still open)**:

| ID | Severity | File | Description | Status |
|----|----------|------|-------------|--------|
| MR-M1-01 | **HIGH** | state/mod.rs:159 | **`.unwrap()` in `add_object()`.** `self.zones.get_mut(&zone_id).unwrap()` after a `contains_key` guard. Provably safe in isolation (im-rs prevents concurrent mutation between check and access), but violates the architectural constraint: "engine crate uses typed errors — never `unwrap()` or `expect()` in engine logic." If a future refactor removes the guard or reorders code, this panics the engine. Should use `.ok_or(GameStateError::ZoneNotFound(zone_id))?`. | CLOSED — fix session 1 |
| MR-M1-02 | **HIGH** | state/mod.rs:228 | **`.unwrap()` in `move_object_to_zone()`.** Same pattern as MR-M1-01: `self.zones.get_mut(&to).unwrap()` after earlier validation. Same fix needed. | CLOSED — fix session 1 |
| MR-M1-03 | **MEDIUM** | state/builder.rs:318 | **`.expect()` in `build()`.** `state.add_object(object, zone).expect("failed to add object in builder")`. Builder is documented as a test utility that panics on invalid configuration, so the convention is arguably acceptable. However, `build()` is public API and could be used outside tests. Consider returning `Result` or documenting the panic contract. | CLOSED — fix session 8 |
| MR-M1-04 | **MEDIUM** | state/mod.rs | **Check-then-access pattern.** Both `add_object` and `move_object_to_zone` use `contains_key()` + `get_mut().unwrap()` instead of the idiomatic `get_mut().ok_or()?` pattern. Creates maintenance risk — the guard and the access can drift apart during refactoring. | CLOSED — fix session 1 (subsumed by MR-M1-01/02) |
| MR-M1-05 | **MEDIUM** | state/builder.rs:181 | **Panics on 0 players.** `build()` panics if `self.players.is_empty()`. Could return `Result` for consistency with engine error handling philosophy. Currently has `#[should_panic]` test in builder_tests.rs so it's tested, but violates typed-error convention. | CLOSED — fix session 8 |
| MR-M1-06 | **LOW** | structural_sharing.rs | **Uses mock types, not real GameState.** Tests im-rs principle with stand-in structs using `im::HashMap` (not `im::OrdMap` which the real code uses). Real structural sharing validated in `snapshot_perf.rs`, so this is redundant — not wrong, just low-value. | OPEN |
| MR-M1-07 | **LOW** | state_foundation.rs | **ManaPool tests thin.** Only 1 test (`test_mana_pool_operations`) covering basic add/total/empty. No colored mana allocation, insufficient mana, or complex scenarios. Adequate for M1 (pool used properly starting M3). | OPEN |
| MR-M1-08 | **INFO** | object_identity.rs | **Exemplary CR citation.** All 10 tests directly reference CR 400.7 with specific sub-behaviors. Model for other test files. | — |
| MR-M1-09 | **INFO** | state_invariants.rs | **Good property-based foundation.** 5 proptest tests covering zone integrity, unique IDs, move semantics. Could expand with state determinism properties in M3+. | — |
| MR-M1-10 | **INFO** | — | **Commander format compliance verified.** `PlayerState` defaults: life=40, commander_tax tracking, commander_damage_received matrix, poison_counters. All correct for Commander. | — |
| MR-M1-11 | **INFO** | — | **Type safety is strong.** PlayerId, ObjectId, CardId are distinct types. ZoneId enum prevents invalid zone references. No accidental ID confusion possible. | — |

**New findings from re-review**:

| ID | Severity | File | Description | Status |
|----|----------|------|-------------|--------|
| MR-M1-12 | **MEDIUM** | game_object.rs, player.rs, mod.rs | **Core types lack `PartialEq`/`Eq` — blocked by `Effect` not deriving it.** `GameObject`, `Characteristics`, `PlayerState`, `TurnState`, `GameState` cannot be compared for equality. Root cause: `ActivatedAbility` and `TriggeredAbilityDef` contain `Option<crate::cards::card_definition::Effect>`, and `Effect` only derives `Clone, Debug, Serialize, Deserialize` — no `PartialEq`. This blocks the entire derive chain through `Characteristics` (via `activated_abilities`/`triggered_abilities`) up to `GameObject` and `GameState`. Tests must check fields individually or use blake3 hashes for equality. Adding `PartialEq` to `Effect` (a 30+ variant recursive enum) and its transitive dependencies (`EffectTarget`, `EffectAmount`, `PlayerTarget`, `TargetFilter`, `TokenSpec`, `TriggerCondition`, `Condition`, etc.) would unblock derives on all core types. | CLOSED — fix session 7 |
| MR-M1-13 | **LOW** | player.rs:32-33,36-44 | **`ManaPool::total()` and `ManaPool::add()` can overflow u32.** `total()` chains 6 additions: `self.white + self.blue + self.black + self.red + self.green + self.colorless` without checked arithmetic. `add()` uses `+=`. Panics in debug builds, wraps silently in release. Practically unreachable (requires >4 billion mana), but `saturating_add` would be more defensive. | OPEN |
| MR-M1-14 | **LOW** | error.rs:22-23 | **`InvalidZoneTransition` error variant is dead code.** Defined but never constructed anywhere in the codebase. `move_object_to_zone` does not validate whether a zone transition is legal (e.g., hand→battlefield vs. hand→stack). Zone transition legality is enforced at the Command level, not in the state module. The variant suggests planned validation that was never added. | OPEN |
| MR-M1-15 | **LOW** | player.rs | **`ManaPool` has no `spend`/`pay` method.** Only `add()` and `empty()` exist. Mana spending logic lives in `rules/mana.rs` via direct field manipulation (`pool.white -= amount` etc.). Not encapsulated in the type, increasing risk of inconsistent mana handling across call sites. | OPEN |
| MR-M1-16 | **LOW** | builder.rs:101-128 | **Builder player setters silently no-op for unknown PlayerId.** `player_life(id, life)`, `player_poison(id, counters)`, `player_mana(id, pool)` iterate `self.players` looking for a matching ID. If no player has that ID, the method silently returns `self` unchanged. Could cause confusing test failures where a test thinks it configured a player but the ID was wrong. A `debug_assert!` on match would catch configuration errors. | OPEN |
| MR-M1-17 | **LOW** | builder.rs:76-87 | **Builder `add_player` allows duplicate PlayerIds.** Pushes a new `PlayerConfig` without checking for existing IDs. In `build()`, `OrdMap::insert` silently overwrites on key collision — the first player's configuration is discarded. Zones are also created twice (second overwrites first). Produces a valid state but with silently lost configuration. | OPEN |
| MR-M1-18 | **LOW** | zone.rs:103-143 | **`Zone::Ordered` has O(n) `contains` and `remove`.** `contains` on `im::Vector` is a linear scan (line 105). `remove` calls `position()` (O(n)) + `remove(pos)` (O(log n)) (line 135). For libraries with 90+ cards, potentially slow if called in hot paths. `Zone::Unordered` uses `OrdSet` which is O(log n) for both. Currently not a bottleneck — `contains` on ordered zones is rare, and `remove` happens once per zone change. | OPEN |
| MR-M1-19 | **LOW** | tests/ | **No test for same-zone move (CR 400.7 edge case).** Moving an object to the same zone type (e.g., battlefield → battlefield, as with certain flickering effects) should still create a new object per CR 400.7. The current implementation handles this correctly (remove + re-add with new ObjectId), but no test explicitly confirms this behavior. | OPEN |
| MR-M1-20 | **LOW** | tests/ | **No test for valid object moved to invalid destination zone.** Only `move_nonexistent_object_errors` is tested (invalid object, valid zone). The reverse case — valid object, non-existent destination zone (e.g., `Library(PlayerId(99))`) — is untested. This path triggers the non-atomic corruption documented in MR-M0-13: the object is removed from the source zone before the destination is validated, so the error path leaves a corrupted state. | OPEN |
| MR-M1-21 | **INFO** | game_object.rs, builder.rs | **Files evolved significantly since M1.** `game_object.rs` (221 lines) now includes M3-M7 additions: `ActivationCost`, `ActivatedAbility` (with `effect: Option<Effect>`), `TriggerEvent`, `InterveningIf`, `TriggeredAbilityDef` (with `effect: Option<Effect>`), `deathtouch_damage`, `has_summoning_sickness`. `builder.rs` (676 lines) gained `with_activated_ability`, `with_triggered_ability`, `with_damage`, `with_deathtouch_damage`, `with_registry`, `add_continuous_effect`. Line counts in original review reflect current state — re-review covers this appropriately. | — |
| MR-M1-22 | **INFO** | zone.rs:156-165 | **`Zone::shuffle` correctly implements Fisher-Yates.** Deterministic with seeded RNG. Well-tested in `zone_integrity.rs`. Good implementation. | — |
| MR-M1-23 | **INFO** | types.rs:58, types.rs:77, player.rs:18 | **String-wrapped newtypes allow empty/invalid content.** `SubType(pub String)`, `CounterType::Custom(String)`, `CardId(pub String)` accept arbitrary strings including empty. Conscious design tradeoff — enum-based SubType is impractical with 280+ creature types. Defense-in-depth concern only. | — |

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
| Same-zone move | **Missing** | MR-M1-19: battlefield→battlefield not tested |
| Invalid destination zone | **Missing** | MR-M1-20: valid object + non-existent zone not tested |
| Builder duplicate players | **Missing** | MR-M1-17: add_player with same ID twice not tested |
| Builder wrong PlayerId | **Missing** | MR-M1-16: player_life/poison/mana with unknown ID not tested |

### Notes

- `state/stubs.rs` (43 lines) contains placeholder types (`PendingTrigger`) that are later
  filled in by M3-E. Not a finding — intentional forward declaration.
- `builder.rs` at 676 lines is the largest M1 file. The fluent API is well-designed
  but the `expect()` in `build()` should be addressed.
- `im::OrdMap` used consistently for deterministic iteration — correct per CLAUDE.md.
- **Re-review coverage note**: The original M1 review was solid but was conducted against
  the evolved file contents (M1-M7 growth) without explicitly noting file evolution. The
  re-review adds 8 additional CR sections to the "CR Sections Implemented" table, 4 new
  test gap rows, and 12 new findings. The strongest new finding is MR-M1-12 (`PartialEq`
  blocked by `Effect`), which is a cross-cutting concern originating in M7's addition of
  `Option<Effect>` to M1/M3-era structs.
- **MR-M0-13 overlap**: The non-atomic `move_object_to_zone` was captured in MR-M0-13
  during the M0 re-review. MR-M1-20 identifies the corresponding test gap (no test
  exercises the invalid-destination-zone error path that exposes MR-M0-13).

---

## M2: Turn Structure & Priority

**Review Status**: RE-REVIEWED (2026-02-22)

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
| MR-M2-01 | **HIGH** | priority.rs:54 | **`.expect()` in engine library code.** `next_priority_player(state, player).expect(...)` in `pass_priority`. Logically unreachable (the `all_passed` check on line 46 guarantees at least one player hasn't passed), but violates the "no unwrap/expect in engine" convention. If state becomes inconsistent (e.g., a bug in `active_players()` vs `players_passed`), this panics the engine instead of returning an error. **Fix:** `.ok_or(GameStateError::NoActivePlayers)?` | CLOSED — fix session 1 |
| MR-M2-02 | **HIGH** | turn_structure.rs:78 | **`.expect()` in engine library code.** `next_player_in_turn_order(state, turn.last_regular_active).expect("no active players remaining")` in `advance_turn`. Called when there are no extra turns. If all players are eliminated, this panics. Currently unreachable because `is_game_over` checks in `enter_step` prevent reaching `advance_turn` when ≤1 players remain, but the panic is a landmine for future refactors. **Fix:** change `advance_turn` to return `Result<...>` and propagate error. | CLOSED — fix session 1 |
| MR-M2-03 | **HIGH** | engine.rs:292-323 | **Concede while active player: step-advance then turn-advance.** When the conceding player is both the active player AND the priority holder, and all other players have already passed, the code calls `handle_all_passed` (line 305) which advances the step and executes turn-based actions, then ALSO calls `advance_turn` (line 316) because `active_player == conceding player`. This executes one step's turn-based actions for a player who has already conceded (e.g., `draw_for_turn` draws a card for the conceded player if we advance through Draw — see MR-M2-04). Should skip `handle_all_passed` when the conceding player is the active player and go straight to `advance_turn`. | CLOSED — fix session 5 |
| MR-M2-04 | **MEDIUM** | turn_actions.rs:90 | **`draw_card` has no concession/elimination guard.** `draw_card(state, player)` draws for any player regardless of `has_conceded` or `has_lost` status. Reachable via MR-M2-03: conceded active player's turn advances through the Draw step. The drawn card is pointless (player is eliminated) but modifies dead state. **Fix:** add `if p.has_lost \|\| p.has_conceded { return Ok(vec![]); }` guard. | CLOSED — fix session 5 |
| MR-M2-05 | **HIGH** | engine.rs:269-327 | **Concede doesn't clean up owned objects (CR 800.4a).** When a player leaves a multiplayer game, "all objects owned by that player leave the game and any effects which give that player control of any objects or players end." The concede handler marks `has_conceded = true` but doesn't: (1) exile/remove the player's permanents from the battlefield, (2) remove the player's spells from the stack, (3) end control-change effects. **Note:** may be intentionally deferred — this is a complex interaction that needs the full effect/replacement system. Mark as deferred to M9 (Commander rules integration). | DEFERRED → M9 |
| MR-M2-06 | **MEDIUM** | turn_actions.rs:158 | **`DiscardedToHandSize` event uses wrong ObjectId.** `object_id: new_id` where `new_id` is the NEW graveyard ObjectId (from `move_object_to_zone`). The event also has `zone_from: hand_zone`, but the object at `new_id` was never in the hand zone — it's the post-zone-change identity (CR 400.7). Should include both old hand ID and new graveyard ID, or at minimum use the old ID for `object_id`. | CLOSED — fix session 5 |
| MR-M2-07 | **LOW** | turn_invariants.rs | **Proptest lacks library cards.** `run_pass_sequence` builds a 4-player state with no library cards. Players hit empty-library loss within 1-2 turns, limiting the turn-cycle coverage of the proptest. Adding 10+ library cards per player would exercise more turn cycles. | OPEN |
| MR-M2-08 | **LOW** | concede.rs | **No test for concede when active player + all others passed.** The complex code path at `engine.rs:302-307` (active player concedes, no next priority player → `handle_all_passed` + `advance_turn`) has zero test coverage. This is the path where MR-M2-03 manifests. | OPEN |
| MR-M2-09 | **LOW** | turn_actions.rs:142 | **`unwrap_or(7)` for max_hand_size lookup.** If `state.players.get(&active)` returns None (active player missing from map), silently defaults to 7. Should be unreachable but a `.ok_or()` would be more defensive. Minor since the scenario requires a corrupted state. | OPEN |
| MR-M2-10 | **INFO** | engine.rs | **Loop-based step advancement (not recursion).** `enter_step` correctly uses a loop for no-priority steps (Untap, Cleanup), avoiding stack overflow on long chains of auto-advancing steps. Good design. | — |
| MR-M2-11 | **INFO** | priority.rs | **`pass_priority` doesn't mutate state.** The function builds a local `passed` set and returns `PriorityResult`; the caller (`handle_pass_priority`) applies the state change. Clean separation of query vs mutation. | — |
| MR-M2-12 | **INFO** | turn_structure.rs | **Extra turns correctly use LIFO with `pop_back`.** `advance_turn` pops from the back of `extra_turns` (most recently added goes first per CR 614.10), and `last_regular_active` correctly tracks normal order resumption. 4 tests verify this behavior. | — |
| MR-M2-13 | **INFO** | turn_actions.rs:52 | **Summoning sickness cleared at untap.** CR 302.6 implementation: `has_summoning_sickness = false` for all active player's permanents during untap. Correct. | — |
| MR-M2-14 | **MEDIUM** | turn.rs:70, engine.rs:201 | **CR 514.3a: Cleanup step never checks SBAs or grants conditional priority.** `Step::has_priority()` unconditionally returns `false` for Cleanup. But CR 514.3a (confirmed by CR 704.3) says: after cleanup turn-based actions, check SBAs and triggered abilities; if any fire, perform them, grant priority to the active player, and when all pass start a NEW cleanup step. Currently, if an "until end of turn" effect expires during cleanup and causes a creature's toughness to drop to 0, the SBA check won't fire until the next turn's first priority-granting step. **Fix:** replace the `has_priority()` gate with cleanup-specific logic: run SBAs + check triggers; if any events produced, grant priority (and loop back to a new cleanup step when all pass); if none, auto-advance as today. | CLOSED — fix session 5 |
| MR-M2-15 | **MEDIUM** | engine.rs:312-322 | **Active player concede during combat leaks stale `state.combat`.** When the active player concedes mid-combat, `handle_concede` calls `advance_turn` (which only modifies `TurnState`, not `GameState.combat`) then `enter_step`. The stale `CombatState` (with the old attacking player) persists into the next player's turn. When the next player's `BeginningOfCombat` fires, `begin_combat` checks `state.combat.is_none()` → false → skips creating a new `CombatState`. Consequence: stale combat data from the conceded player's turn. **Fix:** clear `state.combat = None` in `handle_concede` before calling `advance_turn` when `active_player == player`. | CLOSED — fix session 5 |
| MR-M2-16 | **LOW** | turn_actions.rs:178-179, engine.rs:158,250 | **Redundant triple mana-pool empty + unconditional event.** During End→Cleanup→NewTurn: (1) `handle_all_passed` empties pools at step transition, (2) `cleanup_actions` empties again AND unconditionally pushes `ManaPoolsEmptied` regardless of whether pools were non-empty, (3) the auto-advance loop empties a third time. The unconditional event at line 179 is also misleading — `empty_all_mana_pools` returns conditional events but the return value is discarded. **Fix:** use the return value from `empty_all_mana_pools` in `cleanup_actions` instead of the unconditional push. | OPEN |
| MR-M2-17 | **LOW** | concede.rs | **No test for concede during active combat phase.** The stale combat state bug (MR-M2-15) has zero test coverage. No concede test sets up a state at a combat step (e.g., `DeclareAttackers`) with `state.combat = Some(...)`. | OPEN |
| MR-M2-18 | **INFO** | events.rs:49 | **`LossReason::Conceded` defined but never emitted.** The concede handler emits `PlayerConceded` (a separate event variant), not `PlayerLost { reason: LossReason::Conceded }`. The variant only appears in hash.rs for completeness. Harmless but dead code. | — |

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
| Concede during combat | **Missing** | MR-M2-17: stale `state.combat` leak has no coverage |
| Cleanup SBA/trigger priority | **Missing** | MR-M2-14: CR 514.3a exception never tested |
| Empty library loss | Thin (indirect) | Proptest may trigger it but no dedicated test |
| Proptest invariants | Good (4 tests) | State validity, holder validity, monotonicity, eliminated check |

### Notes (original)

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

### Re-Review Notes (2026-02-22)

**Re-review context**: Calibration test — M2 was the first milestone reviewed in a dedicated
session. Comparing new finding count/severity against M0/M1 re-reviews to assess whether
M3-M7 also need re-reviewing.

**New findings**: 5 (2 MEDIUM, 2 LOW, 1 INFO). All original 9 actionable findings confirmed
still open and accurate.

**Key new findings**:
- MR-M2-14 (MEDIUM): CR 514.3a is a genuine rules compliance gap — cleanup must conditionally
  check SBAs and grant priority. This is the kind of subtle CR interaction that requires deep
  rules knowledge to catch. It's latent for now (no current cards trigger SBAs during cleanup)
  but will become a real bug as more continuous effects are implemented (M8+).
- MR-M2-15 (MEDIUM): Stale `state.combat` on active-player concede during combat. This is a
  cross-milestone interaction bug (M2 concede × M6 combat state lifecycle) that wasn't visible
  during the original M2 review because combat didn't exist yet.

**Calibration assessment**: The original M2 review was solid for its scope. New findings are:
one CR rules subtlety requiring deep MTG knowledge (514.3a) and one cross-milestone interaction
bug not visible at original review time. Neither reflects a systematic gap in review depth.
Fewer total new findings (5) than M1 re-review (12), with comparable severity profile (no new
HIGHs). **Recommendation: M3-M7 re-reviews are NOT warranted.** The original dedicated-session
reviews appear adequate for their scope.

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
| MR-M3-01 | **HIGH** | resolution.rs:79-80,122-129 | **Partial fizzle: targets not filtered.** When some (but not all) targets are illegal at resolution time, `resolve_top_of_stack` passes ALL original targets to `EffectContext` via `stack_obj.targets.clone()` (line 126), including the illegal ones. Effects execute against illegal targets instead of skipping them. Comment on line 80 says "Illegal targets will be unaffected when effects are implemented (M7+)" but M7 is now complete and this filtering was not added. **Fix:** filter `stack_obj.targets` to only legal targets before passing to `EffectContext`. | CLOSED — fix session 4 |
| MR-M3-03 | **HIGH** | hash.rs:349-365, game_object.rs:220 | **GameObject hash omits `has_summoning_sickness`.** The `HashInto` impl for `GameObject` (hash.rs:349-365) hashes 14 fields but skips `has_summoning_sickness: bool` (game_object.rs:220). Two game states differing only in summoning sickness produce identical public hashes, breaking the distributed verification model. **Fix:** add `self.has_summoning_sickness.hash_into(hasher);` to the GameObject impl. | CLOSED — fix session 7 |
| MR-M3-04 | **HIGH** | abilities.rs:148 | **Non-existent object target silently accepted in ability activation.** In `handle_activate_ability`, hexproof/shroud target validation (lines 146-171) uses `if let Some(obj) = state.objects.get(id)` which silently skips non-existent objects. A `Target::Object` with a bogus ObjectId passes validation — no existence check, no zone snapshot. Compare with `casting.rs:187-192` which explicitly returns `ObjectNotFound` for missing targets. **Fix:** return `GameStateError::ObjectNotFound(*id)` when object doesn't exist. | CLOSED — fix session 4 |
| MR-M3-02 | **MEDIUM** | casting.rs:108 | **ManaCostPaid not emitted for {0} cost spells.** The `if cost.mana_value() > 0` guard (line 108) skips the entire payment block for zero-cost spells, including the `ManaCostPaid` event. Zero-cost spells like Ornithopter ({0}) and Memnite ({0}) never emit `ManaCostPaid`. Matters if triggers key on `ManaCostPaid` (e.g., "whenever a player casts a spell" that checks mana paid). **Fix:** emit `ManaCostPaid` even when cost is {0} (no pool deduction needed, just the event). | CLOSED — fix session 4 |
| MR-M3-05 | **MEDIUM** | hash.rs:585-589, game_object.rs:89 | **ActivatedAbility hash omits `effect` field.** The `HashInto` impl (hash.rs:585-589) only hashes `cost` and `description`, but `ActivatedAbility` also has `effect: Option<Effect>` (game_object.rs:89, added in M7). In the distributed verification model, if one peer loaded a different card definition with a different effect, the hash wouldn't detect the mismatch. **Cross-milestone:** M3 wrote the hash impl; M7 added the field without updating the hash. **Fix:** add `self.effect.hash_into(hasher);` (requires `HashInto` impl for `Effect`). | CLOSED — fix session 7 |
| MR-M3-06 | **MEDIUM** | hash.rs:658-663, game_object.rs:140 | **TriggeredAbilityDef hash omits `effect` field.** Same pattern as MR-M3-05: `HashInto` for `TriggeredAbilityDef` hashes `trigger_on`, `intervening_if`, `description` but not `effect: Option<Effect>` (game_object.rs:140). Same cross-milestone gap as MR-M3-05. **Fix:** add `self.effect.hash_into(hasher);`. | CLOSED — fix session 7 |
| MR-M3-07 | **MEDIUM** | abilities.rs:146-171, casting.rs:196-216 | **Hexproof/shroud target validation duplicated.** `handle_activate_ability` (abilities.rs:146-171) and `validate_targets` (casting.rs:196-216) both implement hexproof/shroud checks with nearly identical code. The abilities version is weaker (silently skips non-existent objects per MR-M3-04). **Fix:** extract a shared `validate_target_protection(state, target, controller)` helper used by both paths. | CLOSED — fix session 4 |
| MR-M3-08 | **MEDIUM** | targeting.rs:591-594 | **`matches!` used as bare statement — silent no-op assertion.** In `test_601_insufficient_mana_fails`, line 591-594: `matches!(result.unwrap_err(), mtg_engine::GameStateError::InsufficientMana);` is a bare expression, not wrapped in `assert!()`. The `matches!` macro returns a `bool` that is silently discarded — the test passes regardless of the error variant. **Fix:** wrap in `assert!(matches!(...));`. | CLOSED — fix session 4 |
| MR-M3-09 | **LOW** | hash.rs:955-965 | **LegendaryRuleApplied event hash missing length prefix.** The `for (old_id, new_id) in put_to_graveyard` loop (lines 961-964) hashes pairs without prefixing the vec length. The generic `Vec<T>` HashInto impl uses a length prefix for unambiguous framing. Inconsistent pattern — low collision risk in practice since the discriminant byte and subsequent events provide implicit framing. **Fix:** add `(put_to_graveyard.len() as u64).hash_into(hasher);` before the loop. | CLOSED — fix session 7 |
| MR-M3-10 | **LOW** | targeting.rs:224-269 | **Incomplete test discards results.** `test_608_2b_fizzle_player_target_concedes` constructs a fizzle scenario but the test body ends with `let _ = (final_state, events);` (line 268) and a comment "will redo below." The test has no assertions. The replacement test (`test_608_2b_fizzle_all_targets_illegal` at line 275) covers the fizzle case properly, making this test dead code. **Fix:** delete the incomplete test or complete it for the concede-specific path. | CLOSED — fix session 7 |
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
| MR-M4-01 | **HIGH** | sba.rs:120 | **`.unwrap()` in `check_player_sbas`.** `state.players.get(&id).unwrap()` for the commander damage check. The player IS guaranteed to exist (iterated from `state.players.keys()` on line 85, checked with `state.players.get(&id)` on line 87), but violates the "no unwrap/expect in engine" convention. If a future refactor between lines 87-120 invalidates the guarantee, this panics. **Fix:** `if let Some(p) = state.players.get(&id) { ... }` with early-continue on None. | CLOSED — fix session 1 |
| MR-M4-02 | **HIGH** | sba.rs:189-226 | **SBAs don't use `calculate_characteristics` for P/T or keyword checks.** `check_creature_sbas` reads raw `obj.characteristics.toughness` and `obj.characteristics.keywords` instead of the layer-calculated values. A creature with base 2/2 under a continuous -2/-2 effect has `raw_toughness == 2` but `effective_toughness == 0` — the SBA would miss it. Same for indestructible: Humility removes keywords from the raw characteristics level, but the SBA reads from raw. The layer system (M5) provides `calculate_characteristics()` which returns the correct values. **Cross-milestone:** M4 code, M5+ fix. | DEFERRED → M5 review |
| MR-M4-03 | **HIGH** | sba.rs:340 | **`.unwrap()` in `check_legendary_rule`.** `ids.last().unwrap()` — safe because `ids.len() < 2` guard on line 334 guarantees ≥2 elements, but violates convention. **Fix:** `let Some(&kept) = ids.last() else { continue; };` | CLOSED — fix session 1 |
| MR-M4-04 | **MEDIUM** | sba.rs:217 | **`u32 as i32` cast in lethal damage comparison.** `obj.damage_marked as i32 >= toughness` — `damage_marked` is `u32`. If damage exceeds `i32::MAX` (2,147,483,647), the cast wraps to negative, making the comparison incorrect. Practically unreachable in normal gameplay but formally unsound. **Fix:** convert toughness to u32 instead: `toughness > 0 && obj.damage_marked >= toughness as u32`. | CLOSED — fix session 6 |
| MR-M4-05 | **MEDIUM** | sba.rs:280 | **`u32 as i32` cast in planeswalker loyalty.** `loyalty_counter.map(|c| c as i32)` — same overflow pattern as MR-M4-04. If a planeswalker somehow had >2B loyalty counters, the cast wraps. **Fix:** compare in u32 space: if counter > 0, alive; if counter == 0, dead. | CLOSED — fix session 6 |
| MR-M4-06 | **MEDIUM** | sba.rs (missing) | **CR 704.5b not implemented as SBA.** "If a player attempted to draw from an empty library since the last SBA check, that player loses." Currently handled as immediate loss in `turn_actions.rs:99-108` (CR 104.3b). The timing difference matters if replacement effects (M8) could replace the draw — the current approach loses the player before replacements can apply. Acceptable for M4; should be revisited when replacement effects are implemented in M8. | DEFERRED → M8 |
| MR-M4-07 | **LOW** | sba.rs (missing) | **CR 704.5e (spell/card copies) not implemented.** "If a copy of a spell is in a zone other than the stack, it ceases to exist." No `is_copy` field on `GameObject`. Copy effects are M8+ territory — expected omission. | DEFERRED → M8+ |
| MR-M4-08 | **LOW** | sba.rs (missing) | **CR 704.5k (world rule) not implemented.** "If two or more permanents have the supertype world..." World is an extremely rare supertype (handful of old cards, none Commander staples). Reasonable to defer indefinitely. | DEFERRED |
| MR-M4-09 | **LOW** | sba.rs:329 | **`String::clone()` allocation in legendary rule hot path.** `obj.characteristics.name.clone()` for every legendary permanent on every SBA pass. Creates many small allocations if many legends are on the battlefield. Minor performance concern — could use `&str` references in the grouping map. | OPEN |
| MR-M4-10 | **LOW** | sba.rs:391,449,453 | **`SubType("...".to_string())` allocates on every comparison.** Aura/Equipment/Fortification checks create new `String` allocations on every object iteration. Should use a static or pre-allocated `SubType`. Same pattern in `check_aura_sbas` and `check_equipment_sbas`. | OPEN |
| MR-M4-11 | **LOW** | sba.rs:281 | **`unwrap_or(1)` default for missing planeswalker loyalty.** If a planeswalker has no Loyalty counter AND no `characteristics.loyalty`, effective loyalty defaults to 1 (survives). Correct for well-constructed states (planeswalkers always have `characteristics.loyalty`), but silently hides construction bugs. A `unwrap_or(0)` or logging would catch incorrectly built test states. | OPEN |
| MR-M4-12 | **LOW** | tests/sba.rs | **Test gap: no test for planeswalker with Loyalty counters (vs characteristics.loyalty).** All planeswalker tests use `ObjectSpec::planeswalker(p, name, loyalty)` which sets `characteristics.loyalty`. No test verifies the `CounterType::Loyalty` counter path (sba.rs:278-279) which is the runtime path for planeswalkers that have used loyalty abilities. | CLOSED — fix session 8 |
| MR-M4-13 | **LOW** | tests/sba.rs | **Test gap: no test for aura whose target left the battlefield.** The only aura test (704.5m) tests an unattached aura (`attached_to == None`). No test for the `target.zone != Battlefield` branch (sba.rs:400-404) where an aura is attached to an object that moved zones. | OPEN |
| MR-M4-14 | **LOW** | tests/sba.rs | **Test gap: no test for 3+ legendary copies.** Only tests 2 copies of a legendary. No test verifying that with 3+ copies, all but the newest are removed simultaneously. The grouping logic (sba.rs:333-340) should handle it, but it's unverified. | CLOSED — fix session 8 |
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
| MR-M5-01 | **HIGH** | layers.rs:95 | **`.expect()` in engine library code.** `state.objects.get(&object_id).expect("object exists")` in the counter P/T block (layer 7c). Provably safe — the object was retrieved at line 36 and state is `&` (immutable), so it cannot have been removed. However, violates the "no unwrap/expect in engine" convention. **Fix:** wrap counter logic in `if let Some(obj_ref) = state.objects.get(&object_id) { ... }`. | CLOSED — fix session 1 |
| MR-M5-02 | **HIGH** | sba.rs:193,200,204-207,469,472,585 | **SBAs use raw characteristics, not `calculate_characteristics()`.** Forwarded from MR-M4-02, now validated. Seven call sites in `sba.rs` read `obj.characteristics.{toughness,card_types,keywords}` directly. Impact: (1) creature under -X/-X continuous effect: raw toughness unmodified, SBA won't kill it; (2) Humility removes Indestructible keyword: raw still has it, SBA incorrectly skips lethal-damage destroy; (3) Opalescence makes enchantment a creature: raw card_types lacks Creature, equipment attachment SBA misses it. **Fix:** call `calculate_characteristics` for each battlefield object at the start of each SBA pass. | CLOSED — fix session 6 |
| MR-M5-03 | **MEDIUM** | layers.rs:435 | **`ptr::eq` for effect identity in cycle fallback.** Uses `std::ptr::eq(*e, *effect)` to check whether an effect was already emitted during Kahn's cycle recovery. Correct in current usage — all references point into the same `im::Vector`, so each element has a unique address. Fragile: if refactored to use cloned values or indices, `ptr::eq` silently breaks. **Fix:** compare by `EffectId`: `result.iter().any(\|e\| e.id == effect.id)`. | CLOSED — fix session 6 |
| MR-M5-04 | **MEDIUM** | continuous_effect.rs:46-57 | **`EffectDuration` missing general condition-based durations.** Only `WhileSourceOnBattlefield`, `UntilEndOfTurn`, `Indefinite`. Missing: (1) `UntilEndOfNextTurn`/`UntilYourNextTurn` — common in Commander ("until your next turn"); (2) general `AsLongAs(Condition)` — "as long as you control a creature." Roadmap M5 deliverable says "as long as" is covered; only the most common case (`WhileSourceOnBattlefield`) is implemented. | DEFERRED → M8+ |
| MR-M5-05 | **MEDIUM** | layers.rs:184-192 | **`AllPermanents` filter over-checks card types.** Checks `card_types.contains(Creature\|Artifact\|Enchantment\|Land\|Planeswalker\|Battle)` instead of just `obj_zone == Battlefield`. Per CR 110.4, any card on the battlefield is a permanent. If layer 4 removes all card types, `AllPermanents` incorrectly excludes the object. Extremely rare but technically wrong. **Fix:** `obj_zone == ZoneId::Battlefield`. | CLOSED — fix session 6 |
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
| MR-M6-01 | **HIGH** | combat.rs:67 | **Attack target not validated.** `handle_declare_attackers` validates each attacker (battlefield, controller, creature type, defender, tapped, summoning sickness) but completely ignores the `AttackTarget`. The `_target` variable on line 67 is never inspected. Accepts: (1) attacking yourself `AttackTarget::Player(self)`, (2) attacking a non-existent player, (3) attacking your own planeswalker, (4) attacking a planeswalker not on the battlefield. **Fix:** validate that `Player(pid)` is an opponent and exists; validate that `Planeswalker(pw_id)` is on the battlefield and controlled by an opponent. | CLOSED — fix session 2 |
| MR-M6-02 | **HIGH** | combat.rs:282-284 | **Same creature can block multiple attackers.** `combat.blockers.insert(*blocker_id, *attacker_id)` uses OrdMap insert which silently overwrites. If the `blockers` Vec parameter contains the same `blocker_id` twice with different attacker_ids (e.g., `[(B, A1), (B, A2)]`), the last entry wins. CR 509.1a: "For each of the chosen creatures, the defending player chooses one creature for it to block." A creature blocks exactly one attacker. **Fix:** before inserting, check `combat.blockers.contains_key(blocker_id)` or validate no duplicate blocker_ids in the input Vec. | CLOSED — fix session 2 |
| MR-M6-03 | **HIGH** | combat.rs:356-363 | **Partial blocker ordering accepted.** `handle_order_blockers` validates that every blocker in `order` is blocking the attacker (line 356-362) but does NOT validate that every blocker OF the attacker is in `order`. Submitting a partial order (e.g., ordering 1 of 3 blockers) silently accepts; the other 2 blockers are excluded from `damage_assignment_order` and receive no attacker damage. CR 509.2: the attacking player orders ALL blocking creatures. **Fix:** add `if order.len() != blocking_this.len() { return Err(...) }` after line 363. | CLOSED — fix session 2 |
| MR-M6-09 | **HIGH** | combat.rs:196-230 | **Blocker can block attacker targeting a different player.** `handle_declare_blockers` validates that the attacker is a declared attacker (line 225) but never checks that the attacker is targeting the declaring player or their planeswalker. In multiplayer, p2 could declare their creature blocks an attacker that's attacking p3. CR 509.1a: "the defending player chooses one creature for it to block that's attacking that player or a planeswalker that player controls." **Fix:** after line 230, resolve the attacker's `AttackTarget` and verify it matches the declaring player. | CLOSED — fix session 2 |
| MR-M6-10 | **HIGH** | combat.rs:172-305 | **`defenders_declared` tracked but never enforced.** The `CombatState::defenders_declared` set is populated on line 286 but never checked as a guard at the top of `handle_declare_blockers`. The same player can call `DeclareBlockers` multiple times, overwriting earlier blocker assignments (OrdMap insert). This allows: (1) changing which attacker a creature blocks mid-step, (2) adding new blockers after already finishing declaration. **Fix:** add guard: `if combat.defenders_declared.contains(&player) { return Err(...) }` after line 193. | CLOSED — fix session 2 |
| MR-M6-11 | **MEDIUM** | combat.rs:249-276 | **Menace check counts Vec entries, not unique creatures.** The menace check sums `blocker_count_for_attacker` from both existing combat blockers and the new `blockers` Vec. If MR-M6-02 allows the same blocker_id to appear twice for the same attacker, the count would be 2 (satisfying menace) even though only 1 unique creature is blocking. **Dependency:** fixing MR-M6-02 (duplicate blocker prevention) also fixes this. | CLOSED — fix session 2 (via MR-M6-02) |
| MR-M6-12 | **MEDIUM** | combat.rs:98,121 | **Redundant `calculate_characteristics` for vigilance.** Vigilance is checked twice in `handle_declare_attackers`: first via `chars.keywords.contains(&Vigilance)` (line 98, from the full `calculate_characteristics` call at line 81), then again via the `has_keyword` helper (line 121, which calls `calculate_characteristics` a second time). The second call is unnecessary — `has_vigilance` from line 98 should be reused. Not a correctness bug (both calls see the same immutable state) but wasteful and fragile. | CLOSED — fix session 2 |
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
| MR-M7-01 | **HIGH** | effects/mod.rs:509 | **`MoveZone` effect always emits `ObjectExiled` regardless of destination.** Line 509: `events.push(GameEvent::ObjectExiled { ... })` fires for every MoveZone, even when the destination is Battlefield, Hand, Graveyard, or Library. A creature bounced to hand would emit `ObjectExiled`. **Fix:** match on destination zone and emit the appropriate event (`PermanentEnteredBattlefield` for battlefield, `CardDrawn` or a generic `ObjectMoved` for hand, etc.). | CLOSED — fix session 3 |
| MR-M7-02 | **HIGH** | casting.rs:162 / card_definition.rs:358 | **`TargetRequirement` filters not validated at cast time.** `validate_targets` (casting.rs:162) only checks existence, zone snapshot, and hexproof/shroud — it does NOT check `TargetRequirement` type restrictions. Comment on line 160 says "Full type-restriction validation deferred to M7" but M7 is now complete and this was not implemented. Doom Blade (`TargetCreatureWithFilter`) can target non-creatures. Negate (`TargetSpell`) can counter creature spells. Any `TargetRequirement` variant beyond basic existence is unenforced. **Fix:** validate each target against its `TargetRequirement` (creature type, permanent type, color filter, etc.) during `validate_targets`. | CLOSED — fix session 4 |
| MR-M7-03 | **HIGH** | card_definition.rs:639-641 / effects/mod.rs:1090 | **Doom Blade's "non-black" filter semantically inverted.** `TargetFilter { colors: Some([Color::Black]) }` combined with `matches_filter` (line 1090) means "object MUST be black." Doom Blade says "destroy target NON-black creature" — the filter selects the exact opposite set. Additionally, `TargetFilter` has no negation field for colors (unlike `non_land: bool`). **Fix:** add a `non_colors: Option<OrdSet<Color>>` field to `TargetFilter`, or a `negate_colors: bool` flag, and use it for Doom Blade. Even once MR-M7-02 is fixed, this filter would admit only black creatures. | CLOSED — fix session 4 |
| MR-M7-04 | **HIGH** | effects/mod.rs:914-927 | **`resolve_zone_target` ignores `owner` field — always uses spell controller.** Lines 917-923: all three owner-bearing `ZoneTarget` variants (Graveyard, Hand, Library) discard the `owner: PlayerTarget` field and use the passed `controller` instead. Comment on line 918 says "For simplicity, use controller." For effects like "return target permanent to its owner's hand" (e.g., Unsummon), this puts the card in the CASTER's hand, not the card owner's. Currently no card definition in the 50 uses `MoveZone` for cross-player returns, but this will break when added. **Fix:** resolve the `PlayerTarget` in the `owner` field using `resolve_player_target_list`, or accept a resolved `PlayerId` directly. | CLOSED — fix session 3 |
| MR-M7-05 | **HIGH** | effects/mod.rs:112 | **`i32` to `u32` cast wraps negative values in DealDamage.** `resolve_amount` returns `i32`; line 112 casts `as u32`. If `EffectAmount::PowerOf` resolves to a negative value (creature with negative power from layer effects), the cast wraps to ~4 billion, dealing massive damage. The `if dmg == 0 { return; }` guard on line 113 doesn't catch wrapping. Same pattern in GainLife (line 179), LoseLife (line 196), DrawCards (line 214: `as usize`). **Fix:** clamp to 0 before casting: `let dmg = resolve_amount(...).max(0) as u32;`. | CLOSED — fix session 3 |
| MR-M7-06 | **MEDIUM** | effects/mod.rs:1206-1207 | **`ForEachTarget::EachPlayer` and `EachOpponent` return empty vec.** `collect_for_each` returns `Vec<ObjectId>` — player-based ForEach targets return `vec![]` with a comment "players aren't ObjectIds." Effects like "each player draws a card" via ForEach can't work. The `ForEach` combinator only handles object iteration, not player iteration. **Fix:** either refactor `collect_for_each` to return an enum of ObjectId/PlayerId collections, or handle player ForEach variants separately in `execute_effect_inner`. | CLOSED — fix session 3 |
| MR-M7-07 | **MEDIUM** | effects/mod.rs:900-907 | **`EffectAmount::CardCount` always returns 0.** The match arm has a comment "M7+: implement card counting if needed. Defaults to 0." Any card definition using CardCount for damage/draw amounts produces zero effect. No current definition uses it, but the variant exists and silently fails. **Fix:** implement the zone-card-count logic or remove the variant until needed. | CLOSED — fix session 3 |
| MR-M7-08 | **MEDIUM** | definitions.rs:697, 738, 788 | **Supreme Verdict "can't be countered" not modeled.** Definition encodes only the destroy-all-creatures effect; the uncounterable restriction is in oracle text but not in abilities. Similarly, Negate's "noncreature spell" restriction uses `TargetSpell` without a filter (no `TargetSpellWithFilter` variant exists). Arcane Denial's delayed draw triggers simplified to immediate draw. These are known simplifications but may confuse script authors expecting correct behavior. | CLOSED — fix session 8 |
| MR-M7-09 | **MEDIUM** | effects/mod.rs:432-445 | **`AddManaAnyColor` and `AddManaChoice` default to colorless.** Both variants add 1 colorless instead of letting the player choose a color. Affects 4 definitions: Arcane Signet, Command Tower, Birds of Paradise, Darksteel Ingot. Documented as "M9+: interactive mana color choice" but means these cards produce wrong mana (colorless instead of any-color). In most games this would produce incorrect mana pool states. | OPEN → M9 |
| MR-M7-10 | **MEDIUM** | effects/mod.rs:930-936, 542-547 | **`dest_tapped` helper ignores `ZoneTarget::Battlefield { tapped: true }` flag.** `dest_tapped()` takes a `ZoneId` (not `ZoneTarget`) and always returns `Some(false)` for Battlefield. The `tapped: true` flag from `ZoneTarget::Battlefield { tapped: true }` is only captured in the SearchLibrary handler via a separate check (lines 542-548 manually apply `tapped`). If other effects use `dest_tapped` expecting it to detect "enters tapped," they'll get the wrong answer. The function is misleading. **Fix:** pass `ZoneTarget` instead of `ZoneId`, or remove the function and inline the tapped check. | CLOSED — fix session 3 |
| MR-M7-11 | **MEDIUM** | definitions.rs:946-961 | **Brainstorm only draws 3 — does not put 2 cards back.** Oracle text says "Draw three cards, then put two cards from your hand on top of your library in any order." The effect is `DrawCards { count: 3 }` only — the put-back is missing entirely. Comment says "simplified as draw without scry" (confusing Brainstorm with a scry card). This makes Brainstorm strictly better than intended — pure card advantage instead of card selection. | CLOSED — fix session 8 |
| MR-M7-12 | **MEDIUM** | definitions.rs:509-539 | **Path to Exile's search is unconditional but should be "may".** Oracle says the exiled creature's controller "may search their library." Currently encoded as an unconditional `SearchLibrary`, which always fetches a land. Requires interactive choice (`MayPayOrElse` pattern) for correctness. Note: `MayPayOrElse` already exists in the Effect enum and defaults to "don't pay → apply or_else" — could model as `MayPayOrElse { cost: ..., payer: ControllerOf, or_else: Nothing }` with SearchLibrary in the "may" branch once M9 interactive choice is implemented. | OPEN → M9 |
| MR-M7-13 | **LOW** | definitions.rs:1083-1108 | **Equipment definitions have empty ability lists.** Lightning Greaves and Swiftfoot Boots have `abilities: vec![]` — they are blank permanents on the battlefield. The equip cost, keyword grants (haste+shroud / haste+hexproof) are described only in oracle text. These cards do nothing until the equipment/attach system is implemented. Expected for M7; flagged for completeness. | OPEN → M8+ |
| MR-M7-14 | **LOW** | definitions.rs:160-177, 362-396 | **"No maximum hand size" ability not modeled.** Thought Vessel, Reliquary Tower, and Rogue's Passage second ability ({4},{T}: unblockable) are described in oracle text but not encoded in abilities. Only the tap-for-mana ability exists on each. | OPEN → M8+ |
| MR-M7-15 | **LOW** | tests/effects.rs | **No test for CreateToken or CounterSpell effects.** The 8 effect tests cover DealDamage (2), ExileObject+GainLife (1), DrawCards (1), Nothing (1), Sequence (1), Conditional (2). Token creation and spell countering are exercised only by game scripts, not by direct effect unit tests. | CLOSED — fix session 8 |
| MR-M7-16 | **LOW** | tests/ | **No combat game script.** All 7 approved scripts cover baseline and stack interactions. No script exercises the combat system (declare attackers/blockers, damage step) through the replay harness. Combat is tested via unit tests in `tests/combat.rs` and `tests/keywords.rs`, but not via the engine-independent script format. | OPEN |
| MR-M7-17 | **LOW** | effects/mod.rs:567 | **Shuffle uses `from_entropy()` — non-deterministic in effect execution.** `SearchLibrary` and `Shuffle` effects call `rand::rngs::StdRng::from_entropy()` (line 567), making the shuffle non-deterministic across replays. Tests use seeded RNG at the `GameState` level, but effect-level shuffles bypass this. For replay determinism, the shuffle should use the state's seeded RNG. | CLOSED — fix session 3 |
| MR-M7-18 | **LOW** | script_replay.rs:700-718 | **`try_as_tap_mana_ability` only converts single-mana single-color abilities.** Sol Ring ({T}: Add {C}{C}) is NOT converted to a ManaAbility by `enrich_spec_from_def` because it produces 2 colorless. Scripts must use ActivateAbility instead of TapForMana for Sol Ring. Documented in CLAUDE.md but creates an asymmetry. | CLOSED — fix session 8 |
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

**Review Status**: REVIEWED (2026-02-22)

### Files Introduced

| File | Lines | Purpose |
|------|-------|---------|
| `state/replacement_effect.rs` | 160 | Type definitions: `ReplacementEffect`, `ReplacementId`, `ReplacementTrigger` (5 variants), `ReplacementModification` (6 variants), `ObjectFilter` (7 variants), `PlayerFilter` (3 variants), `DamageTargetFilter` (4 variants), `PendingZoneChange` |
| `rules/replacement.rs` | 991 | Core replacement/prevention logic: `find_applicable`, `determine_action`, `check_zone_change_replacement`, `apply_etb_replacements`, `apply_self_etb_from_definition`, `register_permanent_replacement_abilities`, `apply_damage_prevention`, `resolve_pending_zone_change`, `handle_order_replacements` |
| `tests/replacement_effects.rs` | 2346 | 50+ tests across Sessions 1-5: serde round-trip, builder wiring, find_applicable, determine_action, loop prevention, self-replacement priority, OrderReplacements command, zone-change interception, commander replacement, draw replacement, ETB replacement, prevention shields |
| `test-data/generated-scripts/replacement/*.json` | 6 files | Game scripts: RiP simple redirect, multiple ordered, self-first, commander die, ETB tapped, Kalitas-style |

### Files Modified

| File | Delta | Changes |
|------|-------|---------|
| `state/mod.rs` | +30 | New fields: `replacement_effects`, `next_replacement_id`, `pending_zone_changes`, `prevention_counters`. Added `next_replacement_id()` method. |
| `state/hash.rs` | +200 | `HashInto` impls for all new M8 types: `ReplacementId`, `ObjectFilter`, `PlayerFilter`, `DamageTargetFilter`, `ZoneType`, `ReplacementTrigger`, `ReplacementModification`, `ReplacementEffect`, `PendingZoneChange`. Hash for new `GameEvent` variants. M8 fields added to `public_state_hash`. |
| `state/builder.rs` | +80 | `with_replacement_effect()`, `with_prevention_counter()` builder methods. `register_commander_zone_replacements()`. New fields initialized in `build()`. |
| `rules/engine.rs` | +10 | `Command::OrderReplacements` handler calling `replacement::handle_order_replacements()`. |
| `rules/command.rs` | +5 | New `Command::OrderReplacements { player, ids }` variant. |
| `rules/events.rs` | +15 | New events: `ReplacementEffectApplied`, `ReplacementChoiceRequired`, `DamagePrevented`. |
| `rules/sba.rs` | +90 | Zone-change replacement interception in creature and planeswalker SBA checks. Pending zone change skip logic. |
| `rules/combat.rs` | +25 | Damage prevention integration in `apply_combat_damage`. |
| `rules/resolution.rs` | +30 | ETB replacement integration: `apply_self_etb_from_definition`, `apply_etb_replacements`, `register_permanent_replacement_abilities` after permanent ETB. |
| `rules/lands.rs` | +25 | Same ETB replacement integration for PlayLand path. |
| `rules/turn_actions.rs` | +30 | WouldDraw replacement check in `draw_card`. |
| `effects/mod.rs` | +120 | `apply_damage_prevention` calls in `DealDamage`. `check_zone_change_replacement` in `DestroyPermanent` and `ExileObject`. `draw_one_card` helper with WouldDraw replacement. |
| `cards/definitions.rs` | +120 | 4 new card definitions: Dimir Guildgate (ETB tapped), Rest in Peace, Leyline of the Void, Darksteel Colossus. |
| `cards/card_definition.rs` | +10 | `AbilityDefinition::Replacement { trigger, modification, is_self }` variant. |
| `state/stubs.rs` | ~0 | Comment update: "ReplacementEffect has moved to state/replacement_effect.rs (M8)". |

### CR Sections Implemented

| CR Section | Implementation |
|------------|---------------|
| CR 614 | `replacement.rs` -- replacement effect framework (find, apply, chain) |
| CR 614.5 | `replacement.rs:60` -- `already_applied` HashSet prevents re-application |
| CR 614.10 | `turn_actions.rs:119`, `effects/mod.rs:1255` -- SkipDraw modification |
| CR 614.11 | `turn_actions.rs:101-130`, `effects/mod.rs:1237-1263` -- WouldDraw replacement check |
| CR 614.12 | `replacement.rs:677-776` -- ETB replacement effects (EntersTapped, EntersWithCounters) |
| CR 614.15 | `replacement.rs:71-82` -- self-replacements partitioned first in find_applicable |
| CR 615.1 | `replacement.rs:960-974` -- PreventAllDamage zeroes damage, not consumed |
| CR 615.7 | `replacement.rs:927-958` -- PreventDamage shields with counter decrement and depletion |
| CR 616.1 | `replacement.rs:92-138` -- determine_action: NoApplicable/AutoApply/NeedsChoice |
| CR 616.1a | `replacement.rs:107-130` -- self-replacement auto-apply when alone, choice when multiple |
| CR 616.1e | `replacement.rs:132-138` -- non-self multiple replacements: player chooses among all |
| CR 616.1f | `replacement.rs:559-616` -- re-check after applying one replacement |
| CR 903.9 | `builder.rs:register_commander_zone_replacements` -- commander graveyard/exile redirects |

### Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| MR-M8-01 | **HIGH** | `replacement.rs:569` | **Hardcoded `ZoneType::Battlefield` as "from" zone in `resolve_pending_zone_change` re-check.** When re-checking remaining replacements after applying one (CR 616.1f), the `from` zone is hardcoded to `Battlefield`. This is incorrect for zone changes originating from other zones (e.g., a card moving from Library to Graveyard intercepted by Rest in Peace -- Darksteel Colossus from anywhere). The `from` zone should be preserved from the original event context. Currently, `PendingZoneChange` does not store the `from` zone at all. **Fix:** Add an `original_from: ZoneType` field to `PendingZoneChange`, populate it at all creation sites (sba.rs, effects/mod.rs), and use it in the re-check call instead of `ZoneType::Battlefield`. | CLOSED — fix session 1 |
| MR-M8-02 | **HIGH** | `effects/mod.rs:323-332,401-410` | **`DestroyPermanent` and `ExileObject` ignore `ChoiceRequired` from `check_zone_change_replacement`.** Both effect handlers match `ZoneChangeAction::Redirect` but treat `ChoiceRequired` as `_ => default_destination`, proceeding with the original zone move. When a commander is destroyed with Rest in Peace active (two competing replacements), the object moves to the default zone without player choice, violating CR 616.1. SBAs correctly handle this case by creating pending zone changes, but effect-driven zone moves do not. **Fix:** Add `ZoneChangeAction::ChoiceRequired` match arms in both `DestroyPermanent` and `ExileObject` that create `PendingZoneChange` entries and emit `ReplacementChoiceRequired` events, matching the SBA pattern. | CLOSED — fix session 1 |
| MR-M8-03 | **HIGH** | `turn_actions.rs:216`, `rules/layers.rs:490-497` | **`UntilEndOfTurn` replacement effects never expire.** `expire_end_of_turn_effects` (called during cleanup) only filters `state.continuous_effects` by `EffectDuration::UntilEndOfTurn`. It does not touch `state.replacement_effects`. Prevention shields with `UntilEndOfTurn` duration (e.g., "prevent the next 3 damage this turn") persist indefinitely across turns, violating CR 514.2. This affects any `ReplacementEffect` or `PreventDamage` shield registered with `UntilEndOfTurn` duration. **Fix:** Add a parallel expiration step in `cleanup_actions` that filters `state.replacement_effects` to remove entries with `UntilEndOfTurn` duration, and removes corresponding `prevention_counters` entries. | CLOSED — fix session 1 |
| MR-M8-04 | **MEDIUM** | `replacement.rs:779-796` | **`zone_change_events` uses hardcoded `PlayerId(0)` for `ObjectExiled` events.** The helper function that produces zone-change events after a pending replacement resolves hardcodes `player: PlayerId(0)` in the `ObjectExiled` variant (line 786). This produces incorrect event data. The actual owner/controller of the exiled object is available at all call sites but not passed to this helper. **Fix:** Add an `owner: PlayerId` parameter to `zone_change_events` and use it in the `ObjectExiled` event construction. Update the call site at line 595. | CLOSED — fix session 1 |
| MR-M8-05 | **MEDIUM** | `replacement.rs:790-793` | **`ReplacementId(u64::MAX)` sentinel for commander redirect events.** `zone_change_events` emits a `ReplacementEffectApplied` event with `effect_id: ReplacementId(u64::MAX)` as a sentinel when the destination is the command zone. This violates the invariant that `ReplacementEffectApplied.effect_id` references a real replacement effect. Any consumer of this event (triggers, UI, replay) that tries to look up the effect by ID will fail silently. A dedicated `GameEvent::CommanderSentToCommandZone` variant would be cleaner. **Fix:** Either define a `CommanderZoneRedirect` event variant or use the actual replacement effect ID (which is available from the `check_zone_change_replacement` result). | CLOSED — fix session 1 |
| MR-M8-06 | **MEDIUM** | `replacement.rs:781-784` | **`zone_change_events` always emits `CreatureDied` for graveyard destinations.** When an object is redirected to the graveyard (e.g., a non-creature permanent), the helper unconditionally emits `CreatureDied`. Non-creature permanents going to the graveyard should emit `PermanentDestroyed` or `ObjectPutInGraveyard` instead. The SBA code in `sba.rs` correctly distinguishes creature vs non-creature, but this helper does not. **Fix:** Check the object's card types before choosing the event variant, matching the pattern used in `sba.rs` and `effects/mod.rs`. | CLOSED — fix session 1 |
| MR-M8-07 | **MEDIUM** | `turn_actions.rs:101-130`, `effects/mod.rs:1236-1263` | **Duplicated WouldDraw replacement logic in two draw paths.** `draw_card` (turn_actions.rs) and `draw_one_card` (effects/mod.rs) both contained identical 30-line blocks checking WouldDraw replacement effects. Both checked `find_applicable`, `determine_action`, and handled `SkipDraw`. **Fix:** Extracted `check_would_draw_replacement` into `replacement.rs` returning `DrawAction` enum; both paths now call it. | CLOSED — fix session 2 |
| MR-M8-08 | **MEDIUM** | `turn_actions.rs:128`, `effects/mod.rs:1262` | **`NeedsChoice` result for WouldDraw replacements silently falls through to normal draw.** **Fix:** `check_would_draw_replacement` now returns `DrawAction::NeedsChoice(ReplacementChoiceRequired)` when multiple WouldDraw replacements apply; both `draw_card` and `draw_one_card` emit the event and defer the draw. | CLOSED — fix session 2 |
| MR-M8-09 | **MEDIUM** | `definitions.rs:1383-1394` | **Leyline of the Void uses `ObjectFilter::Any` instead of opponent-only filter.** **Fix:** Added `ObjectFilter::OwnedByOpponentsOf(PlayerId)` variant; `object_matches_filter` checks `obj.owner != player_id`; `register_permanent_replacement_abilities` binds `OwnedByOpponentsOf` to the actual controller at registration time; Leyline definition updated to use it. | CLOSED — fix session 2 |
| MR-M8-10 | **MEDIUM** | `state/turn.rs:125`, `state/hash.rs:401-416` | **`TurnState::cleanup_sba_rounds` field not included in hash.** **Fix:** Added `self.cleanup_sba_rounds.hash_into(hasher)` to `TurnState::hash_into`. | CLOSED — fix session 2 |
| MR-M8-11 | **LOW** | `replacement.rs:891-991` | **`apply_damage_prevention` applies all shields in registration order (CR 615.7 gap).** CR 615.7 states: "the player or the controller of the permanent chooses which damage the shield prevents" when two or more applicable sources deal damage simultaneously. The current implementation applies shields in registration order without player choice. For single-source damage this is correct, but for simultaneous multi-source damage (e.g., two blockers dealing damage to an attacker), the player should choose which shield applies to which source. **Fix:** For M8 this is acceptable since multi-source simultaneous damage with multiple shields is extremely rare. Document as a known limitation and plan to revisit in M9+. | OPEN |
| MR-M8-12 | **LOW** | `replacement.rs:633-666` | **Self-ETB replacements from card definitions bypass the replacement effect framework.** `apply_self_etb_from_definition` reads abilities directly from the card definition and applies modifications inline, without going through `find_applicable`/`determine_action`. This means self-ETB replacements do not participate in CR 616.1f re-checking, and do not respect `already_applied` loop prevention (CR 614.5). For current use cases (EntersTapped, EntersWithCounters) this is correct since they modify state directly. But if a card has both a self-ETB and a global ETB replacement, the ordering might not follow CR 614.15 exactly. | OPEN |
| MR-M8-13 | **LOW** | tests/ | **No test for `UntilEndOfTurn` replacement effect expiration.** Tests verify that `UntilEndOfTurn` effects are "always active" but do not test that they are removed during cleanup. This gap is directly related to MR-M8-03 (they are NOT removed). | CLOSED — fix session 1 (covered by MR-M8-03 tests) |
| MR-M8-14 | **LOW** | `definitions.rs:1407` | **Darksteel Colossus "shuffle into library" simplified to `RedirectToZone(Library)`.** The oracle text says "shuffle it into its owner's library instead" but the implementation just moves it to the library top without shuffling. Documented simplification. **Fix:** Add a shuffle step after the zone move, or add a `ShuffleIntoLibrary` replacement modification variant. | OPEN |
| MR-M8-15 | **LOW** | tests/ | **No test for multiple ETB replacements interacting.** Tests cover single global ETB, single self-ETB, and filter mismatch. No test with both a self-ETB (e.g., enters tapped) and a global ETB (e.g., enters with counters) on the same permanent to verify ordering per CR 614.15. | OPEN |
| MR-M8-16 | **LOW** | `replacement.rs:803-877` | **`register_permanent_replacement_abilities` does not clean up stale effects when permanent leaves.** Effects are registered with `WhileSourceOnBattlefield` duration so `is_effect_active` will return false, but the effect objects remain in `state.replacement_effects` permanently (until the game ends). Over a long game with many ETB permanents, this grows unbounded. **Fix:** Add cleanup in `move_object_to_zone` or SBA checks to remove effects whose source has left the battlefield, or add periodic GC. | OPEN |
| MR-M8-17 | **INFO** | `replacement.rs` | **Well-structured replacement effect architecture.** The separation of concerns between `find_applicable` (matching), `determine_action` (decision), and `check_zone_change_replacement` (interception site integration) is clean and extensible. The `already_applied` HashSet for CR 614.5 loop prevention is correct. The self-replacement priority partition in `find_applicable` correctly implements CR 614.15. The `ZoneChangeAction` enum provides a clear protocol for interception sites. |
| MR-M8-18 | **INFO** | `state/replacement_effect.rs` | **Type definitions are well-designed.** `ReplacementTrigger`, `ReplacementModification`, `ObjectFilter`, `PlayerFilter`, and `DamageTargetFilter` enums are comprehensive for M8 scope. `PendingZoneChange` correctly tracks all state needed for deferred choices. All types derive `Clone, Debug, PartialEq, Eq, Serialize, Deserialize`. |
| MR-M8-19 | **INFO** | `tests/replacement_effects.rs` | **Thorough test coverage for core framework.** 50+ tests organized by session. Covers serde round-trip for all modification types, find_applicable with zone/trigger/duration/filter matching, determine_action decision logic, loop prevention (CR 614.5), self-replacement ordering (CR 614.15), OrderReplacements command validation, zone-change interception (SBA + effects), commander replacement with choice, draw replacement, ETB replacement with filter validation, and prevention shields (partial, exhaustion, prevent-all, sequential). All tests cite CR rules. |
| MR-M8-20 | **INFO** | `state/hash.rs` | **Hash coverage complete for M8 types.** All new M8 types have `HashInto` implementations: `ReplacementId`, `ObjectFilter` (7 variants), `PlayerFilter` (3 variants), `DamageTargetFilter` (4 variants), `ZoneType` (7 variants), `ReplacementTrigger` (5 variants), `ReplacementModification` (6 variants), `ReplacementEffect` (all 7 fields), `PendingZoneChange` (4 fields). New `GameEvent` variants (discriminants 53-55) are hashed. `public_state_hash` includes `replacement_effects`, `pending_zone_changes`, `prevention_counters`, and `next_replacement_id`. Exception: MR-M8-10 (`cleanup_sba_rounds` omission is a pre-M8 gap surfaced during this review). |
| MR-M8-21 | **INFO** | `rules/sba.rs` | **SBA zone-change interception is well-integrated.** Both creature and planeswalker SBA paths correctly call `check_zone_change_replacement` before moving objects, handle all three `ZoneChangeAction` variants (Proceed, Redirect, ChoiceRequired), and skip objects with pending zone changes in subsequent SBA passes. |
| MR-M8-22 | **INFO** | CR 903.9 | **Commander zone replacement implemented as replacement effects, not as SBAs.** The current CR 903.9a specifies that commanders returning to the command zone from graveyard/exile is a state-based action, while CR 903.9b says library/hand redirects are replacement effects. The M8 implementation models both graveyard and exile redirects as replacement effects (via `register_commander_zone_replacements`). This produces equivalent gameplay for the common case (commander dies/exiled -> player chooses command zone), but the mechanism differs from the current CR. The SBA-based approach for graveyard/exile and replacement-based approach for library/hand should be reconciled in M9 when commander rules are formalized. |

### Test Coverage Assessment

| Behavior | Coverage | Notes |
|----------|----------|-------|
| ReplacementEffect serde round-trip | Full | 7 tests covering all 6 modification types + WouldGainLife |
| Builder wiring | Full | 4 tests: single/multiple effects, ID counter, default state |
| find_applicable matching | Full | 10 tests: no effects, single match, trigger mismatch, zone mismatch, wildcard from, duration (active/inactive), already_applied (CR 614.5), self-replacement priority (CR 614.15), creature/enchantment filter, opponent filter |
| determine_action decisions | Full | 5 tests: 0 applicable, single auto-apply, 1 self among many, multiple self, multiple non-self |
| OrderReplacements command | Full | 4 tests: valid ordering, empty IDs error, nonexistent ID error, wrong player error |
| Zone-change interception (SBA) | Full | 5 tests: baseline no-replacement, exile redirect, commander auto-redirect, commander+RiP choice, pending skip |
| Zone-change interception (effects) | Good | 2 tests: ExileObject + commander redirect, DestroyPermanent + RiP redirect. **Gap:** no test for ChoiceRequired in effect paths (MR-M8-02) |
| OrderReplacements choice resolution | Good | 1 test: commander+RiP player chooses command zone. Tests both the pending creation and resolution paths. |
| Draw replacement (WouldDraw) | Full | 3 tests: baseline draw, SkipDraw fires, SkipDraw only affects target player |
| ETB replacement (WouldEnterBattlefield) | Good | 4 tests: global EntersTapped via PlayLand, EntersWithCounters, filter mismatch, self-ETB from card definition (Dimir Guildgate). **Gap:** no multi-ETB interaction test (MR-M8-15) |
| Prevention shield (PreventDamage) | Full | 3 tests: partial reduce, depletion/removal, sequential two-shield |
| Prevention (PreventAllDamage) | Full | 1 test: zeroes damage, not consumed, persists for second event |
| Game scripts | Good | 6 scripts covering: RiP simple redirect, multiple ordered, self-first priority, commander die, ETB tapped, Kalitas-style exile-on-death |
| UntilEndOfTurn replacement expiration | **None** | No test verifies cleanup removes UntilEndOfTurn replacement effects (MR-M8-03/13) |
| WouldGainLife interception | **None** | Trigger type defined and serializable but no interception site exists; no card uses it yet |
| Damage prevention in combat | Indirect | Prevention is tested via `apply_damage_prevention` unit tests; combat integration tested via the combat damage pipeline in `apply_combat_damage` but no dedicated replacement+combat test |

### Notes

- **MR-M8-01, MR-M8-02, and MR-M8-03 are the most impactful findings.** MR-M8-01 (hardcoded `from` zone) would cause incorrect re-checks for any non-battlefield zone change replacement. MR-M8-02 (ignored ChoiceRequired in effects) silently bypasses player choice when multiple replacements compete in effect-driven zone moves. MR-M8-03 (UntilEndOfTurn replacement effects never expire) means prevention shields persist across turns.
- **The replacement effect architecture is well-designed.** The `ReplacementTrigger` / `ReplacementModification` / `ReplacementResult` / `ZoneChangeAction` type hierarchy provides clean separation between what to watch for, what to do, and how interception sites integrate. The `PendingZoneChange` system for deferred player choices is a good solution for the async nature of CR 616.1.
- **CR 903.9 implementation note:** The current Comprehensive Rules split commander zone replacement into two mechanisms: SBA-based for graveyard/exile (CR 903.9a) and replacement-effect-based for hand/library (CR 903.9b). M8 models all four as replacement effects, which is functionally equivalent for graveyard/exile but differs from the CR mechanism. M9 should reconcile this.
- **Cross-milestone context:** MR-M4-06 (CR 704.5b empty library not SBA) remains open and is tangentially related to draw replacement. MR-M8-10 (cleanup_sba_rounds hash omission) is a pre-M8 gap surfaced during this review. MR-M7-13 and MR-M7-14 (equipment/hand-size) remain open as expected.

---

## M9: Commander Rules Integration

**Review Status**: REVIEWED (2026-02-23)

### Files Introduced

**Engine Source** (new/modified):

| File | Lines | Purpose |
|------|-------|---------|
| `crates/engine/src/rules/commander.rs` | 691 | **NEW** — Deck validation, commander tax, zone return SBA, partner validation, mulligan, companion |
| `crates/engine/src/rules/command.rs` | 164 | Added 4 new Command variants: ReturnCommanderToCommandZone, TakeMulligan, KeepHand, BringCompanion |
| `crates/engine/src/rules/casting.rs` | 521 | Modified: command zone casting, commander tax integration (CR 903.8) |
| `crates/engine/src/rules/engine.rs` | 486 | Modified: dispatch for 4 new Command variants |
| `crates/engine/src/rules/events.rs` | 571 | Added 5 new GameEvent variants + `reveals_hidden_info()` method |
| `crates/engine/src/rules/sba.rs` | 785 | Modified: integrated commander zone return SBA call (CR 903.9a / CR 704.6d) |
| `crates/engine/src/rules/mod.rs` | 51 | Added `pub mod commander` |
| `crates/engine/src/state/player.rs` | 96 | Added fields: companion, companion_used, mulligan_count |
| `crates/engine/src/state/hash.rs` | 2075 | Added hashing for 3 new PlayerState fields + 5 new GameEvent variants |
| `crates/engine/src/state/builder.rs` | 795 | Modified: PlayerState init for new fields, `register_commander_zone_replacements` refactored (M8 graveyard/exile replacements removed; hand/library only) |
| `crates/engine/src/state/types.rs` | 108 | Added `KeywordAbility::Partner` variant |
| `crates/engine/src/lib.rs` | 37 | Added public re-exports for commander module types |

**Engine Tests** (new/modified):

| File | Lines | Purpose |
|------|-------|---------|
| `crates/engine/tests/commander.rs` | 1881 | **NEW** — Command zone casting, tax, zone return SBA, partner tax, mulligan, companion, full 4-player integration |
| `crates/engine/tests/commander_damage.rs` | 432 | **NEW** — Commander damage SBA (21+), per-commander tracking, copy exclusion, zone-change persistence |
| `crates/engine/tests/deck_validation.rs` | 522 | **NEW** — Deck size, singleton, color identity, banned list, commander type validation |
| `crates/engine/tests/six_player.rs` | 479 | **NEW** — 6-player priority rotation, combat, APNAP, elimination, concession, `reveals_hidden_info()` |
| `crates/engine/tests/replacement_effects.rs` | 3018 | Modified: M8 commander replacement tests updated to M9 SBA model |

**Game Scripts** (new):

| File | Lines | Purpose |
|------|-------|---------|
| `test-data/generated-scripts/commander/cc26_commander_copy_damage.json` | 116 | Draft: CC #26 — copy doesn't deal commander damage |
| `test-data/generated-scripts/commander/cc27_partner_tax.json` | 161 | Draft: CC #27 — partner commander independent tax |
| `test-data/generated-scripts/commander/cc28_commander_dies_exile_replacement.json` | 161 | Draft: CC #28 — commander dies + exile replacement |

**Source total**: ~1,400 new/changed lines (src/) | **Test total**: ~3,700 new/changed lines (tests/) | **Scripts**: 438 lines

### CR Sections Implemented

| CR Section | Implementation |
|------------|---------------|
| CR 903.3 | `commander.rs:validate_deck()` — commander must be legendary creature |
| CR 903.4 | `commander.rs:compute_color_identity()` — mana cost color extraction |
| CR 903.5a | `commander.rs:validate_deck()` — 100-card deck size check |
| CR 903.5b | `commander.rs:validate_deck()` — singleton rule with basic land exemption |
| CR 903.5c | `commander.rs:validate_deck()` — color identity subset validation |
| CR 903.6 | `builder.rs` — command zone created per player; objects placed via `ZoneId::Command` |
| CR 903.8 | `casting.rs:handle_cast_spell()` + `commander.rs:apply_commander_tax()` — tax applied on cast |
| CR 903.9a | `commander.rs:check_commander_zone_return_sba()` — SBA for graveyard/exile return |
| CR 903.9b | `builder.rs:register_commander_zone_replacements()` — replacement for hand/library redirect |
| CR 903.10a | `combat.rs:789-810` — per-commander per-opponent damage tracking |
| CR 704.6c | `sba.rs:check_player_sbas()` — 21+ commander damage SBA loss check |
| CR 702.124 | `commander.rs:validate_partner_commanders()` — partner keyword validation |
| CR 702.124c | `commander.rs:validate_deck()` — combined partner color identity |
| CR 702.124d | `casting.rs` + `commander.rs` — independent tax per partner commander |
| CR 103.5 | `commander.rs:handle_take_mulligan()` — shuffle + draw 7 |
| CR 103.5c | `commander.rs:handle_keep_hand()` — free first mulligan in multiplayer |
| CR 702.139a | `commander.rs:handle_bring_companion()` — {3} special action |
| M10 prep | `events.rs:reveals_hidden_info()` — hidden info classification for network layer |

### Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| MR-M9-01 | **HIGH** | `commander.rs:229` | **Commander zone return SBA is auto-applied without player choice.** CR 903.9a says the owner "**may** put it into the command zone" -- it is optional. The implementation in `check_commander_zone_return_sba` unconditionally moves every commander from graveyard/exile to the command zone with no player choice. This allows a player's commander to be force-returned even when they want it to remain in graveyard/exile (e.g., for reanimation, Flashback synergy, or exile-matters effects). The doc comment on line 225 acknowledges this simplification ("auto-applied... deferred to M10+"), but this is a functional correctness issue: the engine produces illegal game states when a player would prefer their commander to stay. **Fix:** Add a `ChoiceRequired` path analogous to the replacement effect framework. The SBA should emit a choice event; the player responds with `ReturnCommanderToCommandZone` or a new `LeaveCommanderInZone` command. Until then, document as KNOWN SIMPLIFICATION and track. | CLOSED — fix session 1 |
| MR-M9-02 | **HIGH** | `commander.rs:183-195` | **`compute_color_identity` only extracts colors from mana cost, not rules text.** CR 903.4 explicitly states color identity includes "any mana symbols in that card's... rules text" (e.g., Alesha, Who Smiles at Death has {W/B} in its activated ability text, making it Mardu despite costing {2}{R}). The function ignores `oracle_text` entirely. This causes `validate_deck` to accept decks with cards that violate color identity when the commander or deck cards have rules-text-only mana symbols. The doc comment on line 182 acknowledges this as "deferred to a future milestone" but it silently permits illegal decks. **Fix:** Parse `CardDefinition.oracle_text` for mana symbols (regex: `\{[WUBRG]\}` patterns) and include those colors. Alternatively, add a `color_identity` field to `CardDefinition` populated from Scryfall data. | CLOSED — fix session 1 |
| MR-M9-03 | **MEDIUM** | `commander.rs:209` | **Commander tax overflow: `base_cost.generic + tax * 2` can overflow u32.** If a commander is cast many times (theoretically, `tax >= u32::MAX / 2 + 1`), `tax * 2` overflows in debug mode (panic) or wraps in release mode (produces nonsensical cost). While unlikely in practice (would require 2 billion casts), this violates the no-panic invariant for engine library code. **Fix:** Use `tax.checked_mul(2).and_then(|t| base_cost.generic.checked_add(t))` and return a saturating result or an error. | CLOSED — fix session 1 |
| MR-M9-04 | **MEDIUM** | `commander.rs:491` | **`cards_to_bottom.len() as u32` truncates on 64-bit platforms.** If `cards_to_bottom` has more than `u32::MAX` elements (impossible in practice but violates the project's safe-cast policy established in MR-M7-05), `len() as u32` silently truncates. The comparison `cards_to_bottom.len() as u32 != required_bottom` could succeed when it should fail. **Fix:** Compare as `usize`: `cards_to_bottom.len() != required_bottom as usize`. | CLOSED — fix session 2 |
| MR-M9-05 | **MEDIUM** | `commander.rs:417-465` | **Mulligan draws from library via `draw_card` which can trigger library-empty loss during pregame.** `handle_take_mulligan` calls `turn_actions::draw_card` in a loop, which triggers `PlayerLost { reason: LibraryEmpty }` if the library is exhausted. During the pregame mulligan procedure, drawing from an empty library should not cause a game loss (the game hasn't started yet; CR 103.5 describes mulligan as a pregame procedure). A player mulliganing with fewer than 7 cards in their library would incorrectly lose. **Fix:** Either use a dedicated `draw_for_mulligan` helper that doesn't trigger the library-empty loss, or check library size before drawing and handle the edge case gracefully. | CLOSED — fix session 1 |
| MR-M9-06 | **MEDIUM** | `commander.rs:586-628` | **Companion mana deduction uses fixed priority order, not player choice.** `handle_bring_companion` hardcodes the order of mana color consumption for the {3} generic cost (colorless first, then green, red, black, blue, white). In a real game, the player chooses which mana to spend for generic costs. This is the same pattern as `casting.rs:pay_cost()` (pre-existing), but the companion code duplicates the logic rather than calling the shared `pay_cost()` function. Both implementations produce deterministic but potentially suboptimal mana spending. **Fix:** Call `casting::pay_cost()` instead of duplicating the logic. The underlying "player doesn't choose mana spending order" issue is pre-existing (LOW) but the duplication is a MEDIUM code quality concern. | CLOSED — fix session 2 |
| MR-M9-07 | **MEDIUM** | `commander.rs:53-173` | **`validate_deck` silently skips cards not in registry.** Line 131-132: `None => continue` when a card_id is not found in the registry. This means an unknown card silently passes all validation (size, singleton, color identity, banned list). If a deck contains a card_id typo or a card not loaded into the registry, `validate_deck` will report the deck as valid. Architecture Invariant 9 requires every card to have a definition; silently skipping unknown cards undermines this. **Fix:** Return a new `DeckViolation::UnknownCard { card_id }` instead of `continue`. | CLOSED — fix session 1 |
| MR-M9-08 | **MEDIUM** | `casting.rs:61-81` | **Command zone casting validation uses raw `card_obj.characteristics` for type checks.** Lines 84-91 check `card_obj.characteristics.card_types` directly instead of using `calculate_characteristics()`. If a continuous effect changes the card's types (e.g., removing the Land type from a card), the raw characteristics would be stale. On the command zone, continuous effects rarely apply (objects in command zone are not on the battlefield), so this is low risk, but it's inconsistent with the convention established in MR-M5-02 for all type checks. **Fix:** Use `calculate_characteristics()` with fallback to raw characteristics for the type check on line 84-91. | CLOSED — fix session 2 |
| MR-M9-09 | **LOW** | `events.rs:560-570` | **`reveals_hidden_info()` incomplete: `MulliganTaken` and `LibraryShuffled` reveal/commit hidden info.** Mulligan draws reveal cards (7 cards move from hidden library to hand). `LibraryShuffled` changes the library order (hidden information). `CompanionBroughtToHand` moves a card to hand. These events should arguably return `true` since they involve hidden information transitions. Currently the catch-all `_ => false` covers them. **Fix:** Add `MulliganTaken`, `LibraryShuffled`, and `CompanionBroughtToHand` to the `true` arm. Note: `CardDrawn` events are emitted for each draw during mulligan, so draws are partially covered — but the shuffle and companion events are not. | OPEN |
| MR-M9-10 | **LOW** | `commander.rs:126-128` | **Duplicate counting uses `HashMap` in an engine library file.** `validate_deck` uses `std::collections::HashMap` (line 126) instead of `im::OrdMap` for the `name_counts` accumulator. While `validate_deck` operates on pre-game data (not `GameState`), using `HashMap` in the engine crate is inconsistent with the `im-rs` policy (Architecture Invariant 2). Additionally, `HashMap` iteration order is non-deterministic, meaning `DeckViolation::DuplicateCard` entries appear in random order across runs. **Fix:** Use `im::OrdMap` for deterministic violation ordering, or document `validate_deck` as exempt from the im-rs policy since it doesn't touch game state. | OPEN |
| MR-M9-11 | **LOW** | `commander.rs:53-173` | **Deck validation reports multiple violations but no dedup.** If the same card has both a banned-list violation and a color identity violation, both are reported. This is arguably correct behavior (show all problems), but if the same color identity violation could be reported for the same card multiple times in edge cases (e.g., a card with color identity {R}{G} in a mono-white deck would trigger the break on line 158 but could theoretically be iterated multiple times if multiple card_ids map to the same name), it could produce confusing output. **Fix:** Minor — consider deduplicating violations by card name or documenting that multiple violations per card are expected. | OPEN |
| MR-M9-12 | **LOW** | `commander.rs:447` | **Mulligan shuffles hand back to library via `move_object_to_zone` but ignores errors with `let _ =`.** Line 447: `let _ = state.move_object_to_zone(obj_id, lib_zone_id)` silently drops errors. If a card in hand can't be moved (e.g., zone mismatch from a concurrent state modification), the error is lost. In practice, this shouldn't happen during a pre-game mulligan, but the silent error suppression pattern invites bugs. **Fix:** Propagate the error with `?` or at minimum log/count failures. | OPEN |
| MR-M9-13 | **LOW** | `commander.rs:655-659` | **Companion `BringCompanion` emits event even if companion was not in command zone.** Lines 655-668: if `companion_obj_id` is `None` (companion not found in command zone), the code still emits `CompanionBroughtToHand` and sets `companion_used = true`. This means a player could "use" the companion action without the card actually moving anywhere. The event would indicate a state change that didn't happen, violating Architecture Invariant 4 (events must describe what actually happened). **Fix:** Return an error if `companion_obj_id` is `None` — the companion card should always be in the command zone when `BringCompanion` is issued. | OPEN |
| MR-M9-14 | **LOW** | tests | **Test gap: no test for 3+ mulligans (London mulligan escalation).** Tests cover 0, 1, and 2 mulligans. No test exercises taking 3+ mulligans to verify the escalating bottom-of-library count (3rd mulligan → 2 cards to bottom, 4th → 3, etc.). CR 103.5 allows mulligans until the opening hand would be 0 cards. **Fix:** Add a test with 3+ mulligans verifying correct `cards_to_bottom` counts. | OPEN |
| MR-M9-15 | **LOW** | tests | **Test gap: no test for companion with non-empty stack (should be rejected).** The companion timing validation checks `!state.stack_objects.is_empty()` but no test exercises this path with items on the stack. **Fix:** Add a test placing a spell on the stack before `BringCompanion` and verify rejection. | OPEN |
| MR-M9-16 | **LOW** | tests | **Test gap: commander damage from partner commanders tracked independently.** While `test_partner_commanders_separate_tax_tracking` tests tax independence and `test_commander_damage_10_from_a_plus_11_from_b_no_loss` tests per-commander damage thresholds, no test exercises a partner pair where one partner reaches 21+ damage and the other doesn't. **Fix:** Add a test with two partner commanders dealing damage to the same player, verifying only the one reaching 21 triggers loss. | OPEN |
| MR-M9-17 | **LOW** | `commander.rs:63-78` | **Partner validation has no upper bound check on `commander_card_ids` length.** `validate_deck` rejects `> 2` commanders but the partner validation on lines 63-78 only fires for `len() == 2`. If someone passes 0 commander_card_ids, no commander validation occurs at all (no legendary creature check, no color identity computation). **Fix:** Add a check for `commander_card_ids.is_empty()` with an appropriate violation. | OPEN |
| MR-M9-18 | **INFO** | — | **M8 INFO note MR-M8-22 reconciled.** MR-M8-22 noted that M8 modeled all four commander zone-change paths as replacement effects, while the CR splits them into SBA (graveyard/exile) and replacement (hand/library). M9 correctly reconciles this: `check_commander_zone_return_sba` handles graveyard/exile as SBAs (CR 903.9a), and `register_commander_zone_replacements` handles hand/library as replacement effects (CR 903.9b). | — |
| MR-M9-19 | **INFO** | — | **Well-structured commander.rs module.** Clear section separation (deck validation, commander tax, zone return SBA, partner, mulligan, companion), consistent CR citations, and thorough doc comments. The 691-line single-file approach is appropriate given the cohesive domain. | — |
| MR-M9-20 | **INFO** | — | **Comprehensive hash coverage for all M9 types.** All 3 new `PlayerState` fields (`companion`, `companion_used`, `mulligan_count`) and all 5 new `GameEvent` variants are included in `hash.rs`. Discriminant numbering (57-61) is sequential and collision-free. | — |
| MR-M9-21 | **INFO** | — | **Commander damage tracking is CardId-based, surviving zone changes.** The `commander_damage_received: OrdMap<PlayerId, OrdMap<CardId, u32>>` design correctly tracks damage by physical card identity (CardId) rather than zone-ephemeral ObjectId. This means damage persists when a commander dies and is re-cast (new ObjectId, same CardId). Tested by `test_commander_damage_survives_zone_change`. | — |
| MR-M9-22 | **INFO** | — | **Thorough test coverage for core paths.** 44 new tests across 4 test files (commander.rs, commander_damage.rs, deck_validation.rs, six_player.rs), plus replacement_effects.rs updates. The full 4-player integration test (`test_full_four_player_commander_game`) exercises casting, death, SBA return, re-cast with tax, and 21-damage elimination in a single scenario. | — |
| MR-M9-23 | **INFO** | — | **Three deferred issues from previous milestones remain unresolved.** MR-M2-05 (HIGH: concede doesn't clean up owned objects, CR 800.4a), MR-M7-09 (MEDIUM: AddManaAnyColor/AddManaChoice default to colorless), and MR-M7-12 (MEDIUM: Path to Exile search unconditional) were tagged "OPEN -> M9" but were not addressed in this milestone. These should be re-tagged to M10+. | — |

### Test Coverage Assessment

| Behavior | Coverage | Notes |
|----------|----------|-------|
| Cast from command zone (first time) | Full | `test_cast_commander_from_command_zone_first_time` |
| Cast from command zone (2nd, 3rd) | Full | `test_cast_commander_from_command_zone_second_time`, `third_time` |
| Cast with insufficient mana | Full | `test_cast_commander_from_command_zone_insufficient_mana` |
| Non-commander from command zone | Full | `test_cast_non_commander_from_command_zone_rejected` |
| Sorcery speed enforcement | Full | `test_cast_commander_sorcery_speed_enforced` |
| Commander dies → SBA return | Full | `test_commander_dies_returns_to_command_zone_sba` |
| Commander exiled → SBA return | Full | `test_commander_exiled_returns_to_command_zone_sba` |
| Hand replacement registered | Full | `test_commander_bounced_to_hand_replacement_redirects` |
| Library replacement registered | Full | `test_commander_tucked_to_library_replacement_redirects` |
| Tax increments on cast only | Full | `test_commander_tax_increments_on_cast_not_zone_change` |
| Partner independent tax | Full | `test_partner_commanders_separate_tax_tracking` |
| Partner combined color identity | Full | `test_partner_commanders_combined_color_identity` |
| 21 commander damage → loss | Full | `test_commander_damage_21_from_one_commander_kills` |
| 20 damage → no loss | Full | `test_commander_damage_20_from_one_commander_no_loss` |
| Per-commander damage tracking | Full | `test_commander_damage_10_from_a_plus_11_from_b_no_loss` |
| Copy doesn't deal cmdr damage | Full | `test_commander_damage_from_copy_does_not_count` |
| Damage survives zone change | Full | `test_commander_damage_survives_zone_change` |
| Deck size validation | Full | `test_deck_validation_rejects_99_cards`, `101_cards` |
| Singleton rule | Full | `test_deck_validation_rejects_duplicate_nonbasic` |
| Basic land exemption | Full | `test_deck_validation_allows_basic_land_duplicates` |
| Color identity violation | Full | `test_deck_validation_rejects_off_color_identity` |
| Banned card | Full | `test_deck_validation_rejects_banned_card` |
| Non-legendary commander | Full | `test_deck_validation_rejects_non_legendary_commander` |
| Valid 100-card deck | Full | `test_deck_validation_accepts_valid_100_card_deck` |
| Color identity computation | Full | 3 tests: colorless, single, multicolor |
| Free mulligan (CR 103.5c) | Full | `test_free_mulligan_then_london_mulligan` |
| London mulligan (2nd) | Full | Part of `test_free_mulligan_then_london_mulligan` |
| Wrong bottom count rejected | Full | `test_mulligan_keep_wrong_count_rejected` |
| 4-player independent mulligans | Full | `test_mulligan_sequence_four_players` |
| Companion costs {3} | Full | `test_companion_special_action_costs_3_mana` |
| Companion main phase only | Full | `test_companion_only_during_main_phase_stack_empty` |
| Companion once per game | Full | `test_companion_only_once_per_game` |
| 6-player priority rotation | Full | `test_six_player_priority_rotation` |
| 6-player combat 5 defenders | Full | `test_six_player_combat_five_defenders` |
| 6-player APNAP ordering | Full | `test_six_player_apnap_ordering` |
| 6-player skip eliminated | Full | `test_six_player_turn_advancement_skips_eliminated` |
| 6-player concession mid-game | Full | `test_six_player_concession_mid_game` |
| 6-player commander game start | Full | `test_six_player_game_start_all_commanders_correct` |
| `reveals_hidden_info()` | Partial | CardDrawn = true, PriorityGiven = false (see MR-M9-09 for gaps) |
| 3+ mulligans | None | See MR-M9-14 |
| Companion with non-empty stack | None | See MR-M9-15 |
| Full integration test | Full | `test_full_four_player_commander_game` |

### Notes

- **Most significant finding (MR-M9-01):** The commander zone return SBA auto-applies without player choice, violating CR 903.9a ("owner **may** put it into the command zone"). This is documented as intentional simplification but produces incorrect game states when a player wants their commander to stay in graveyard/exile. Requires a choice mechanism.
- **Color identity gap (MR-M9-02):** `compute_color_identity` only reads mana cost, missing rules text mana symbols. This means commanders like Alesha, Who Smiles at Death and Kenrith, the Returned King will have incorrect color identities, silently permitting illegal decks.
- **Strong architecture:** The M9 implementation correctly splits CR 903.9 into SBA-based (graveyard/exile) and replacement-based (hand/library) paths, reconciling the M8 model with the actual CR. Commander damage tracking via `CardId` is robust and well-tested.
- **Test quality is excellent:** 44+ new tests with CR citations, a comprehensive integration test, and 6-player scalability verification. The test-to-source ratio (~2.6:1) is appropriate for rules engine code.
- **Three prior-milestone deferred issues (MR-M2-05, MR-M7-09, MR-M7-12) remain unresolved** and should be re-tagged to M10+.
- **M9 simplifications are well-documented** in code comments (auto-return, mana cost-only color identity, fixed mana spending order). Each includes a "deferred to M10+" note.

---

## Cross-Milestone Issue Index

All findings across all milestones, sorted by severity then milestone.

### CRITICAL

(None identified — all engine panics classified as HIGH per current assessment)

### HIGH

| ID | Milestone | Summary | Status |
|----|-----------|---------|--------|
| MR-M0-01 | M0 | ~~Delete-then-import data loss risk in scryfall-import~~ | CLOSED — FALSE POSITIVE (already in transaction) |
| MR-M0-02 | M0 | FTS5 MATCH operator injection in MCP server | CLOSED — fix session 9 |
| MR-M1-01 | M1 | `.unwrap()` in `add_object()` (state/mod.rs:159) | CLOSED — fix session 1 |
| MR-M1-02 | M1 | `.unwrap()` in `move_object_to_zone()` (state/mod.rs:228) | CLOSED — fix session 1 |
| MR-M2-01 | M2 | `.expect()` in priority.rs:54 — `next_priority_player` | CLOSED — fix session 1 |
| MR-M2-02 | M2 | `.expect()` in turn_structure.rs:78 — `next_player_in_turn_order` | CLOSED — fix session 1 |
| MR-M2-03 | M2 | Concede while active: step-advance then turn-advance overlap | CLOSED — fix session 5 |
| MR-M2-05 | M2→M9→M10+ | Concede doesn't clean up owned objects (CR 800.4a) | DEFERRED → M10+ |
| MR-M3-01 | M3 | Partial fizzle: targets not filtered — effects execute against illegal targets | CLOSED — fix session 4 |
| MR-M3-03 | M3 | GameObject hash omits `has_summoning_sickness` — breaks distributed verification | CLOSED — fix session 7 |
| MR-M3-04 | M3 | Non-existent object target silently accepted in ability activation | CLOSED — fix session 4 |
| MR-M4-01 | M4 | `.unwrap()` in `check_player_sbas` (sba.rs:120) — commander damage path | CLOSED — fix session 1 |
| MR-M4-02 | M4→M5 | SBAs don't use `calculate_characteristics` for P/T or keyword checks | CLOSED — fix session 6 (via MR-M5-02) |
| MR-M4-03 | M4 | `.unwrap()` in `check_legendary_rule` (sba.rs:340) — `ids.last()` | CLOSED — fix session 1 |
| MR-M5-01 | M5 | `.expect()` in layers.rs:95 — counter P/T block | CLOSED — fix session 1 |
| MR-M5-02 | M5 | SBAs use raw characteristics, not `calculate_characteristics` (7 call sites) | CLOSED — fix session 6 |
| MR-M6-01 | M6 | Attack target not validated — accepts self, non-existent, own planeswalker | CLOSED — fix session 2 |
| MR-M6-02 | M6 | Double-blocking not prevented — same creature can block multiple attackers | CLOSED — fix session 2 |
| MR-M6-03 | M6 | Partial blocker ordering accepted — OrderBlockers doesn't require completeness | CLOSED — fix session 2 |
| MR-M6-09 | M6 | Cross-player blocking — blocker can block attacker targeting a different player | CLOSED — fix session 2 |
| MR-M6-10 | M6 | `defenders_declared` tracked but never enforced — same player can re-declare | CLOSED — fix session 2 |
| MR-M7-01 | M7 | `MoveZone` always emits `ObjectExiled` regardless of destination zone | CLOSED — fix session 3 |
| MR-M7-02 | M7 | `TargetRequirement` filters not validated at cast time — all type restrictions unenforced | CLOSED — fix session 4 |
| MR-M7-03 | M7 | Doom Blade "non-black" filter semantically inverted — `colors` field is inclusion-only | CLOSED — fix session 4 |
| MR-M7-04 | M7 | `resolve_zone_target` ignores `owner` field — always uses spell controller | CLOSED — fix session 3 |
| MR-M7-05 | M7 | `i32→u32` cast wraps negative values in DealDamage/GainLife/LoseLife/DrawCards | CLOSED — fix session 3 |
| MR-M8-01 | M8 | `resolve_pending_zone_change` hardcodes `ZoneType::Battlefield` as "from" zone in re-check | CLOSED — fix session 1 |
| MR-M8-02 | M8 | `DestroyPermanent` and `ExileObject` ignore `ChoiceRequired` — bypass player choice | CLOSED — fix session 1 |
| MR-M8-03 | M8 | `UntilEndOfTurn` replacement effects never expire — prevention shields persist across turns | CLOSED — fix session 1 |
| MR-M9-01 | M9 | Commander zone return SBA auto-applied without player choice (CR 903.9a) | CLOSED — fix session 1 |
| MR-M9-02 | M9 | `compute_color_identity` only reads mana cost, not rules text mana symbols (CR 903.4) | CLOSED — fix session 1 |

### MEDIUM

| ID | Milestone | Summary | Status |
|----|-----------|---------|--------|
| MR-M0-03 | M0 | Multi-line CR rules not captured | CLOSED — fix session 9 |
| MR-M0-04 | M0 | CR format assumptions fragile | CLOSED — fix session 9 |
| MR-M0-05 | M0 | FTS index probe fragile | CLOSED — fix session 9 |
| MR-M0-06 | M0 | JSON parse errors lose context | CLOSED — fix session 9 |
| MR-M0-07 | M0 | No download integrity check | CLOSED — fix session 9 |
| MR-M0-13 | M0 | `move_object_to_zone` non-atomic — removes source before validating dest | CLOSED — fix session 5 |
| MR-M0-14 | M0 | Entire ~200MB JSON deserialized into memory | CLOSED — fix session 9 |
| MR-M1-03 | M1 | `.expect()` in builder.rs:318 | CLOSED — fix session 8 |
| MR-M1-04 | M1 | Check-then-access pattern in state/mod.rs | CLOSED — fix session 1 (subsumed by MR-M1-01/02) |
| MR-M1-05 | M1 | Panics on 0 players instead of Result | CLOSED — fix session 8 |
| MR-M1-12 | M1 | Core types lack `PartialEq`/`Eq` — blocked by `Effect` not deriving it | CLOSED — fix session 7 |
| MR-M2-04 | M2 | `draw_card` has no concession/elimination guard | CLOSED — fix session 5 |
| MR-M2-06 | M2 | `DiscardedToHandSize` event uses wrong ObjectId (new graveyard ID instead of old hand ID) | CLOSED — fix session 5 |
| MR-M2-14 | M2 | CR 514.3a: Cleanup never checks SBAs or grants conditional priority | CLOSED — fix session 5 |
| MR-M2-15 | M2 (cross: M6) | Active player concede during combat leaks stale `state.combat` | CLOSED — fix session 5 |
| MR-M3-02 | M3 | ManaCostPaid not emitted for {0} cost spells | CLOSED — fix session 4 |
| MR-M3-05 | M3 (cross: M7) | ActivatedAbility hash omits `effect` field — M7 added field, hash not updated | CLOSED — fix session 7 |
| MR-M3-06 | M3 (cross: M7) | TriggeredAbilityDef hash omits `effect` field — same gap as MR-M3-05 | CLOSED — fix session 7 |
| MR-M3-07 | M3 | Hexproof/shroud target validation duplicated between casting.rs and abilities.rs | CLOSED — fix session 4 |
| MR-M3-08 | M3 | `matches!` bare statement in test — silent no-op assertion | CLOSED — fix session 4 |
| MR-M4-04 | M4 | `u32 as i32` cast in lethal damage comparison (sba.rs:217) — wraps on overflow | CLOSED — fix session 6 |
| MR-M4-05 | M4 | `u32 as i32` cast in planeswalker loyalty counter (sba.rs:280) | CLOSED — fix session 6 |
| MR-M4-06 | M4→M8 | CR 704.5b not implemented as SBA — empty library loss is immediate, not SBA-checked | DEFERRED → M8 |
| MR-M5-03 | M5 | `ptr::eq` for effect identity in cycle fallback — correct but fragile | CLOSED — fix session 6 |
| MR-M5-04 | M5→M8+ | `EffectDuration` missing `UntilEndOfNextTurn` and general `AsLongAs(Condition)` | DEFERRED → M8+ |
| MR-M5-05 | M5 | `AllPermanents` filter over-checks card types instead of just checking Battlefield zone | CLOSED — fix session 6 |
| MR-M6-11 | M6 | Menace check counts raw entries not unique creatures — depends on MR-M6-02 | CLOSED — fix session 2 (via MR-M6-02) |
| MR-M6-12 | M6 | Redundant `calculate_characteristics` for vigilance in declare_attackers | CLOSED — fix session 2 |
| MR-M7-06 | M7 | `ForEachTarget::EachPlayer/EachOpponent` returns empty vec — player iteration broken | CLOSED — fix session 3 |
| MR-M7-07 | M7 | `EffectAmount::CardCount` always returns 0 — unimplemented stub | CLOSED — fix session 3 |
| MR-M7-08 | M7 | Supreme Verdict "can't be countered", Negate "noncreature" restrictions not modeled | CLOSED — fix session 8 |
| MR-M7-09 | M7 | AddManaAnyColor/AddManaChoice default to colorless — 4 cards produce wrong mana | OPEN → M10+ |
| MR-M7-10 | M7 | `dest_tapped` takes ZoneId not ZoneTarget — ignores "enters tapped" flag | CLOSED — fix session 3 |
| MR-M7-11 | M7 | Brainstorm only draws 3 — does not put 2 cards back on library | CLOSED — fix session 8 |
| MR-M7-12 | M7 | Path to Exile's search unconditional — should be "may" | OPEN → M10+ |
| MR-M8-04 | M8 | `zone_change_events` uses hardcoded `PlayerId(0)` for `ObjectExiled` events | CLOSED — fix session 1 |
| MR-M8-05 | M8 | `ReplacementId(u64::MAX)` sentinel for commander redirect events | CLOSED — fix session 1 |
| MR-M8-06 | M8 | `zone_change_events` always emits `CreatureDied` for graveyard destinations | CLOSED — fix session 1 |
| MR-M8-07 | M8 | Duplicated WouldDraw replacement logic in `draw_card` and `draw_one_card` | CLOSED — fix session 2 |
| MR-M8-08 | M8 | `NeedsChoice` for WouldDraw replacements silently falls through to normal draw | CLOSED — fix session 2 |
| MR-M8-09 | M8 | Leyline of the Void uses `ObjectFilter::Any` instead of opponent-only filter | CLOSED — fix session 2 |
| MR-M8-10 | M8 | `TurnState::cleanup_sba_rounds` field not included in hash | CLOSED — fix session 2 |
| MR-M9-03 | M9 | Commander tax overflow: `base_cost.generic + tax * 2` can overflow u32 | CLOSED — fix session 1 |
| MR-M9-04 | M9 | `cards_to_bottom.len() as u32` truncates on 64-bit platforms | CLOSED — fix session 2 |
| MR-M9-05 | M9 | Mulligan draws from library via `draw_card` can trigger library-empty loss during pregame | CLOSED — fix session 1 |
| MR-M9-06 | M9 | Companion mana deduction duplicates `pay_cost` logic with fixed priority order | CLOSED — fix session 2 |
| MR-M9-07 | M9 | `validate_deck` silently skips cards not in registry (Architecture Invariant 9) | CLOSED — fix session 1 |
| MR-M9-08 | M9 | Command zone casting uses raw characteristics for type checks | CLOSED — fix session 2 |

### LOW

| ID | Milestone | Summary | Status |
|----|-----------|---------|--------|
| MR-M0-08 | M0 | No ON DELETE CASCADE for card_faces FK | OPEN |
| MR-M0-09 | M0 | JSON columns stored as TEXT | OPEN |
| MR-M0-10 | M0 | Partial card name matching too broad | OPEN |
| MR-M0-15 | M0 | `rulings_fts` missing UPDATE/DELETE triggers | OPEN |
| MR-M0-16 | M0 | Cards without `oracle_id` stored as empty string | OPEN |
| MR-M1-06 | M1 | structural_sharing.rs uses mock types (im::HashMap, not OrdMap) | OPEN |
| MR-M1-07 | M1 | ManaPool tests thin (1 test) | OPEN |
| MR-M1-13 | M1 | `ManaPool::total()` and `add()` can overflow u32 | OPEN |
| MR-M1-14 | M1 | `InvalidZoneTransition` error variant is dead code — never constructed | OPEN |
| MR-M1-15 | M1 | `ManaPool` has no `spend`/`pay` method — spending logic not encapsulated | OPEN |
| MR-M1-16 | M1 | Builder player setters silently no-op for unknown PlayerId | OPEN |
| MR-M1-17 | M1 | Builder `add_player` allows duplicate PlayerIds — first config silently lost | OPEN |
| MR-M1-18 | M1 | `Zone::Ordered` has O(n) `contains` and `remove` | OPEN |
| MR-M1-19 | M1 | No test for same-zone move (CR 400.7 edge case) | OPEN |
| MR-M1-20 | M1 | No test for valid object + invalid destination zone (exercises MR-M0-13) | OPEN |
| MR-M2-07 | M2 | Proptest lacks library cards — limited turn coverage | OPEN |
| MR-M2-08 | M2 | Test gap: concede while active + all others passed | OPEN |
| MR-M2-09 | M2 | `unwrap_or(7)` for max_hand_size in cleanup | OPEN |
| MR-M2-16 | M2 | Redundant triple mana-pool empty + unconditional ManaPoolsEmptied event in cleanup | OPEN |
| MR-M2-17 | M2 | Test gap: concede during active combat phase (stale state.combat) | OPEN |
| MR-M3-09 | M3 | LegendaryRuleApplied event hash missing length prefix for put_to_graveyard | CLOSED — fix session 7 |
| MR-M3-10 | M3 | Incomplete test discards results (test_608_2b_fizzle_player_target_concedes) | CLOSED — fix session 7 |
| MR-M3-11 | M3 | `apnap_order` silently defaults position with `unwrap_or(0)` | OPEN |
| MR-M3-12 | M3 | `NotController` error used for ownership check in lands.rs — misleading | OPEN |
| MR-M4-07 | M4 | CR 704.5e (spell/card copies) not implemented — no `is_copy` field | DEFERRED → M8+ |
| MR-M4-08 | M4 | CR 704.5k (world rule) not implemented — irrelevant for Commander | DEFERRED |
| MR-M4-09 | M4 | `String::clone()` allocation in legendary rule hot path (sba.rs:329) | OPEN |
| MR-M4-10 | M4 | `SubType("...".to_string())` allocates on every SBA comparison (sba.rs:391,449,453) | OPEN |
| MR-M4-11 | M4 | `unwrap_or(1)` default for missing planeswalker loyalty — hides construction bugs | OPEN |
| MR-M4-12 | M4 | Test gap: no planeswalker with Loyalty counters (only characteristics.loyalty) | CLOSED — fix session 8 |
| MR-M4-13 | M4 | Test gap: no aura whose target left battlefield (only tests unattached) | OPEN |
| MR-M4-14 | M4 | Test gap: no 3+ legendary copies test | CLOSED — fix session 8 |
| MR-M5-06 | M5 | `ready.remove(0)` is O(n) in Kahn's algorithm — use VecDeque | OPEN |
| MR-M5-07 | M5→M8+ | Missing `AddSupertypes`/`RemoveSupertypes` layer 4 variants | DEFERRED → M8+ |
| MR-M5-08 | M5 | CDA partition test uses different sublayers — same-layer CDA priority untested | OPEN |
| MR-M6-06 | M6 | `apply_combat_damage` is 312 lines — extract attacker/blocker helpers | OPEN |
| MR-M6-08 | M6 | Test gap: no combat game script in replay harness | OPEN |
| MR-M6-13 | M6 | Test gap: blocker-removed-before-damage (CR 509.1h) untested | OPEN |
| MR-M6-14 | M6 | `blockers_for()` rebuilds list on every call — O(n) in hot path | OPEN |
| MR-M7-13 | M7 | Equipment definitions (Lightning Greaves, Swiftfoot Boots) have empty ability lists | OPEN → M8+ |
| MR-M7-14 | M7 | "No maximum hand size" ability not modeled (Thought Vessel, Reliquary Tower) | OPEN → M8+ |
| MR-M7-15 | M7 | Test gap: no CreateToken or CounterSpell effect unit tests | CLOSED — fix session 8 |
| MR-M7-16 | M7 | Test gap: no combat game script in replay harness | OPEN |
| MR-M7-17 | M7 | Shuffle in effects uses `from_entropy()` — non-deterministic across replays | CLOSED — fix session 3 |
| MR-M7-18 | M7 | `try_as_tap_mana_ability` doesn't convert multi-mana abilities (Sol Ring) | CLOSED — fix session 8 |
| MR-M8-11 | M8 | `apply_damage_prevention` applies shields in registration order, not player choice | OPEN |
| MR-M8-12 | M8 | Self-ETB replacements bypass the replacement effect framework (CR 616.1f) | OPEN |
| MR-M8-13 | M8 | No test for `UntilEndOfTurn` replacement effect expiration | OPEN |
| MR-M8-14 | M8 | Darksteel Colossus "shuffle into library" simplified to RedirectToZone(Library) | OPEN |
| MR-M8-15 | M8 | No test for multiple ETB replacements interacting (CR 614.15) | OPEN |
| MR-M8-16 | M8 | Stale replacement effects grow unbounded when source leaves battlefield | OPEN |
| MR-M9-09 | M9 | `reveals_hidden_info()` incomplete: MulliganTaken, LibraryShuffled, CompanionBroughtToHand not covered | OPEN |
| MR-M9-10 | M9 | `validate_deck` uses `HashMap` instead of `im::OrdMap` — non-deterministic violation ordering | OPEN |
| MR-M9-11 | M9 | Deck validation reports multiple violations but no dedup | OPEN |
| MR-M9-12 | M9 | Mulligan `move_object_to_zone` errors silently dropped with `let _ =` | OPEN |
| MR-M9-13 | M9 | `BringCompanion` emits event even if companion not found in command zone | OPEN |
| MR-M9-14 | M9 | Test gap: no test for 3+ mulligans (London mulligan escalation) | OPEN |
| MR-M9-15 | M9 | Test gap: no test for companion with non-empty stack rejection | OPEN |
| MR-M9-16 | M9 | Test gap: partner commander damage independence (one at 21+, one not) | OPEN |
| MR-M9-17 | M9 | Partner validation has no check for empty `commander_card_ids` | OPEN |

### INFO

| ID | Milestone | Summary | Status |
|----|-----------|---------|--------|
| MR-M0-11 | M0 | card-db/lib.rs clean | — |
| MR-M0-12 | M0 | rules_db.rs good test coverage | — |
| MR-M0-17 | M0 | Engine state module type design strong (ZoneId, Zone enum, OrdMap/OrdSet) | — |
| MR-M0-18 | M0 | State hashing framework well-designed (length-prefix, discriminants, public/private split) | — |
| MR-M1-08 | M1 | object_identity.rs exemplary CR citation | — |
| MR-M1-09 | M1 | state_invariants.rs good property-based foundation | — |
| MR-M1-10 | M1 | Commander format compliance verified | — |
| MR-M1-11 | M1 | Type safety is strong | — |
| MR-M1-21 | M1 | Files evolved significantly since M1 (M3-M7 additions to game_object.rs, builder.rs) | — |
| MR-M1-22 | M1 | `Zone::shuffle` correctly implements Fisher-Yates with deterministic RNG | — |
| MR-M1-23 | M1 | String-wrapped newtypes (SubType, CounterType::Custom, CardId) allow empty content — conscious design tradeoff | — |
| MR-M2-10 | M2 | Loop-based step advancement (good design) | — |
| MR-M2-11 | M2 | `pass_priority` query/mutation separation (good design) | — |
| MR-M2-12 | M2 | Extra turns LIFO with correct normal-order resumption | — |
| MR-M2-13 | M2 | Summoning sickness cleared at untap (CR 302.6) | — |
| MR-M2-18 | M2 | `LossReason::Conceded` defined but never emitted — dead code | — |
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
| MR-M6-17 | M6 | Combat state lifecycle correct — init at BeginningOfCombat, cleared at EndOfCombat | — (but see MR-M2-15: concede during combat leaks stale state) |
| MR-M8-17 | M8 | Well-structured replacement effect architecture (find/determine/intercept separation) | — |
| MR-M8-18 | M8 | Type definitions well-designed: comprehensive enums, all derive Serialize/Deserialize | — |
| MR-M8-19 | M8 | Thorough test coverage: 50+ tests across 5 sessions with CR citations | — |
| MR-M8-20 | M8 | Hash coverage complete for all M8 types (except pre-M8 gap MR-M8-10) | — |
| MR-M8-21 | M8 | SBA zone-change interception well-integrated (all 3 ZoneChangeAction variants) | — |
| MR-M8-22 | M8 | CR 903.9 implementation note: replacement effects vs SBAs for commander zone changes | — |
| MR-M9-18 | M9 | M8 INFO note MR-M8-22 reconciled — M9 correctly splits SBA vs replacement paths | — |
| MR-M9-19 | M9 | Well-structured commander.rs module (691 lines, cohesive domain, thorough CR citations) | — |
| MR-M9-20 | M9 | Comprehensive hash coverage for all M9 types (3 PlayerState fields + 5 GameEvent variants) | — |
| MR-M9-21 | M9 | Commander damage tracking is CardId-based, surviving zone changes (robust design) | — |
| MR-M9-22 | M9 | Thorough test coverage: 44+ new tests across 4 files, full 4-player integration test | — |
| MR-M9-23 | M9 | Three deferred issues from prior milestones (MR-M2-05, MR-M7-09, MR-M7-12) remain unresolved | — |

---

## Statistics

| Metric | Value |
|--------|-------|
| Total unique issue IDs | 191 (146 M0-M7 + 22 M8 + 23 M9) |
| CRITICAL | 0 |
| HIGH (OPEN) | 0 |
| HIGH (CLOSED) | 30 (1 false positive + 23 closed by fix sessions 1-7 + 1 closed by fix session 9 MR-M0-02 + 3 closed by M8 fix session 1 + 2 closed by M9 fix session 1: MR-M9-01, MR-M9-02) |
| HIGH (DEFERRED) | 1 (MR-M2-05 -> M10+) |
| MEDIUM (OPEN) | 2 (pre-M8: MR-M7-09, MR-M7-12) |
| MEDIUM (CLOSED) | 40 (27 closed by fix sessions 1-9 + 3 closed by M8 fix session 1 + 4 closed by M8 fix session 2 + 3 closed by M9 fix session 1: MR-M9-03, MR-M9-05, MR-M9-07 + 3 closed by M9 fix session 2: MR-M9-04, MR-M9-06, MR-M9-08) |
| MEDIUM (DEFERRED) | 4 (MR-M4-06 -> M8, MR-M5-04 -> M8+, MR-M7-09 -> M10+, MR-M7-12 -> M10+) |
| LOW (OPEN) | 51 (36 pre-M8 + 6 M8: MR-M8-11 through MR-M8-16 + 9 M9: MR-M9-09 through MR-M9-17) |
| LOW (CLOSED) | 3 (MR-M3-09, MR-M3-10 -- fix session 7; MR-M7-17 -- fix session 3) |
| LOW (DEFERRED) | 5 |
| INFO | 55 (43 pre-M8 + 6 M8: MR-M8-17 through MR-M8-22 + 6 M9: MR-M9-18 through MR-M9-23) |
| Milestones reviewed | 10 (M0 re-reviewed, M1 re-reviewed, M2 re-reviewed, M3, M4, M5, M6, M7, M8, M9) |
| Milestones not started | 0 |
| Fix phase progress | M0-M7 fix sessions 1-9 complete; M8 fix phase complete (sessions 1-2: 3 HIGH + 7 MEDIUM closed); M9 fix phase complete (session 1: 2 HIGH + 3 MEDIUM closed; session 2: 3 MEDIUM closed: MR-M9-04, MR-M9-06, MR-M9-08) |

**Engine source LOC (M0-M9)**: ~15,100 lines (+1,400 M9)
**Engine test LOC (M1-M9)**: ~20,700 lines (+3,700 M9)
**Total test count**: 448 (all passing, as of M9 completion)
