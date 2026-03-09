# Ability Review: Dungeon Data Model (Session 1)

**Date**: 2026-03-09
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 309 (Dungeons), 725 (The Initiative), 701.49 (Venture into the Dungeon)
**Files reviewed**:
- `crates/engine/src/state/dungeon.rs` (NEW, 569 lines)
- `crates/engine/src/state/mod.rs` (dungeon_state, has_initiative fields)
- `crates/engine/src/state/player.rs` (dungeons_completed field)
- `crates/engine/src/state/builder.rs` (initialization)
- `crates/engine/src/state/hash.rs` (HashInto for DungeonId, DungeonState, new fields)
- `crates/engine/src/cards/helpers.rs` (re-exports)
- `crates/engine/src/lib.rs` (re-exports)
- `crates/engine/tests/dungeon_data_model.rs` (NEW, 3 tests)

## Verdict: needs-fix

Three findings require attention before Session 2. The Tomb of Annihilation room graph has an incorrect edge (Oubliette exits to Cradle instead of Sandfall Cell), The Atropal token cannot be legendary because TokenSpec lacks a supertypes field, and the AddCounter room effects silently do nothing because EffectTarget::Controller resolves to a Player (not an Object).

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `dungeon.rs:415` | **Tomb of Annihilation: Oubliette exits to wrong room.** Room 2 (Oubliette) exits to room 4 (Cradle), should exit to room 3 (Sandfall Cell). **Fix:** change line 451 `exits: &[4]` to `exits: &[3]`. |
| 2 | **HIGH** | `dungeon.rs:167` | **The Atropal token missing Legendary supertype.** TokenSpec has no supertypes field; The Atropal is legendary per card text. **Fix:** add `supertypes: OrdSet<SuperType>` to TokenSpec, propagate in `make_token`, set `Legendary` on atropal_token_spec. |
| 3 | MEDIUM | `dungeon.rs:248-254` | **Storeroom/Forge AddCounter silently no-ops.** EffectTarget::Controller resolves to ResolvedTarget::Player, but AddCounter only acts on ResolvedTarget::Object. Counter is never placed. **Fix:** use a targeted effect variant or document as known no-op simplification with a TODO. |
| 4 | MEDIUM | `dungeon.rs:533-536` | **Arena Goad targets Source (the dungeon itself).** EffectTarget::Source in dungeon room context points at the dungeon card, not an opponent's creature. Goad requires an Object on the battlefield. **Fix:** document as known no-op simplification with TODO, or change to Effect::NoOp / placeholder. |
| 5 | LOW | `dungeon.rs:374-385` | **Muiral's Graveyard uses Sequence of two CreateToken instead of count:2.** Works correctly but is unnecessarily verbose. TokenSpec already has a `count` field. **Fix:** use a single CreateToken with `count: 2` in the spec. |
| 6 | LOW | `dungeon_data_model.rs` | **No test for Tomb of Annihilation reachability.** The structural test checks exits are valid but doesn't verify all rooms are reachable from room 0. Would have caught Finding 1 if Oubliette's only path to Cradle skipped Sandfall Cell. **Fix:** add a reachability traversal assertion. |
| 7 | LOW | `dungeon.rs:277-283` | **Fungi Cavern simplification is misleading.** Card says "Target creature gets -4/-0 until your next turn" but implementation uses LoseLife to each opponent. The fallback isn't even thematically related. **Fix:** consider Effect::NoOp with a comment, or at least note in comment that the fallback is arbitrary. |

### Finding Details

#### Finding 1: Tomb of Annihilation: Oubliette exits to wrong room

**Severity**: HIGH
**File**: `crates/engine/src/state/dungeon.rs:451`
**CR Rule**: 309.5a -- "they move their venture marker from the room it is on to the next room, following the direction of an arrow pointing away from the room"
**Issue**: The Tomb of Annihilation card shows two paths from Trapped Entry (room 0): left to Veils of Fear, right to Oubliette. Both Veils of Fear and Oubliette lead to Sandfall Cell (room 3). Sandfall Cell then leads to Cradle of the Death God (room 4, bottommost). The implementation has Oubliette (room 2) exiting directly to Cradle of the Death God (room 4), skipping Sandfall Cell entirely. This means a player taking the Oubliette path only visits 3 rooms (Trapped Entry -> Oubliette -> Cradle) instead of the correct 4 rooms (Trapped Entry -> Oubliette -> Sandfall Cell -> Cradle), making it faster to complete and changing the room effects experienced.
**Fix**: Change `dungeon.rs:451` from `exits: &[4]` to `exits: &[3]`.

#### Finding 2: The Atropal token missing Legendary supertype

**Severity**: HIGH
**File**: `crates/engine/src/state/dungeon.rs:167-184`
**CR Rule**: The Atropal card text reads "Create The Atropal, a legendary 4/4 black God Horror creature token with deathtouch." The `legendary` supertype affects SBA 704.5j (legend rule).
**Issue**: `TokenSpec` has no `supertypes` field. The `atropal_token_spec()` function cannot set the Legendary supertype. If The Atropal is created, it will not be subject to the legend rule (CR 704.5j). This could allow multiple The Atropal tokens to coexist when they should not.
**Fix**: Add `pub supertypes: OrdSet<SuperType>` to `TokenSpec` in `card_definition.rs`. Update `make_token()` in `effects/mod.rs` to copy supertypes onto the `Characteristics`. Set `supertypes: [SuperType::Legendary].into_iter().collect()` in `atropal_token_spec()`. Update all existing TokenSpec construction sites to include `supertypes: OrdSet::new()`. Add `supertypes` to the `HashInto` impl for `TokenSpec` if one exists.

#### Finding 3: Storeroom/Forge AddCounter silently no-ops

**Severity**: MEDIUM
**File**: `crates/engine/src/state/dungeon.rs:248-254` (Storeroom), `dungeon.rs:511-516` (Forge)
**CR Rule**: Storeroom: "Put a +1/+1 counter on target creature you control." Forge: "Put two +1/+1 counters on target creature you control."
**Issue**: Both rooms use `Effect::AddCounter { target: EffectTarget::Controller, ... }`. `resolve_effect_target_list` resolves `EffectTarget::Controller` to `ResolvedTarget::Player(ctx.controller)`. But the `AddCounter` handler at `effects/mod.rs:1005` only processes `ResolvedTarget::Object(id)`, silently skipping Player targets. These room effects will never place a counter on anything.
**Fix**: Either (a) change to `EffectTarget::Source` and document it relies on the source being a creature (won't work for dungeon rooms since source is the dungeon), or (b) use a targeted effect approach that will be wired in Session 2, or (c) explicitly document these as known no-ops with `// TODO: requires interactive targeting (M10+)` and consider using `Effect::NoOp` or at minimum an `Effect::GainLife { amount: 0 }` placeholder that doesn't mislead.

#### Finding 4: Arena Goad targets Source (the dungeon itself)

**Severity**: MEDIUM
**File**: `crates/engine/src/state/dungeon.rs:533-536`
**CR Rule**: Arena: "Goad target creature an opponent controls"
**Issue**: `Effect::Goad { target: EffectTarget::Source }` will resolve to the dungeon card itself as the source. The dungeon is not a creature on the battlefield, so the Goad handler at `effects/mod.rs:1727` will find no matching object in `state.objects` (dungeons are in the command zone, not tracked as GameObjects). The effect silently does nothing. The comment says "execution resolves to an opponent's creature" which is incorrect.
**Fix**: Document as a known no-op simplification (requires interactive targeting). Fix the misleading comment. Consider using `Effect::NoOp` or a descriptive placeholder instead.

#### Finding 5: Muiral's Graveyard verbose token creation

**Severity**: LOW
**File**: `crates/engine/src/state/dungeon.rs:374-385`
**Issue**: Uses `Effect::Sequence(vec![CreateToken{spec}, CreateToken{spec}])` to create 2 tokens. TokenSpec already has a `count: u32` field specifically for this purpose.
**Fix**: Replace with a single `Effect::CreateToken { spec }` where the spec has `count: 2`.

#### Finding 6: Missing reachability test

**Severity**: LOW
**File**: `crates/engine/tests/dungeon_data_model.rs:25`
**Issue**: `test_dungeon_def_structure` validates exit indices are in-bounds and non-self-looping, but doesn't verify that all rooms are reachable from room 0. A reachability check (BFS/DFS from room 0) would have caught Finding 1, since a broken edge means some rooms might be unreachable or some paths are shorter than intended.
**Fix**: Add a reachability assertion: BFS from room 0 should visit all rooms in each dungeon.

#### Finding 7: Fungi Cavern arbitrary fallback

**Severity**: LOW
**File**: `crates/engine/src/state/dungeon.rs:277-283`
**Issue**: The card effect is "-4/-0 to a target creature" but the fallback implementation is "each opponent loses 1 life." These are completely unrelated effects. Other simplifications in the file (Runestone Caverns -> draw, Throne of the Dead Three -> draw) at least approximate the card advantage aspect. Fungi Cavern's fallback doesn't approximate the card's intent at all.
**Fix**: Consider using `Effect::NoOp` with a descriptive comment, or at least update the comment to say the fallback is an arbitrary placeholder, not a simplification.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 309.1 (dungeon card type) | Yes (DungeonId enum) | Yes | test_dungeon_def_structure |
| 309.2a (bring from outside game) | Partial (static defs) | No | Venture action is Session 2 |
| 309.2b (command zone) | Yes (dungeon_state on GameState) | Yes | test_dungeon_state_default |
| 309.2c (not permanents, can't cast) | N/A | N/A | No casting infrastructure for dungeons |
| 309.3 (one dungeon at a time) | Partial (OrdMap enforces) | No | Venture enforcement is Session 2 |
| 309.4 (rooms + venture marker) | Yes (RoomDef, DungeonState) | Yes | test_dungeon_def_structure |
| 309.4a (topmost room entry) | Yes (room 0 convention) | Implicit | Part of struct design |
| 309.4b (room names = flavor) | Yes (documented) | N/A | |
| 309.4c (room abilities = triggered) | Partial (fn() -> Effect) | Yes | Effects callable without panic |
| 309.5a (venture = follow arrow) | Partial (exits array) | Yes | Exit validity checked |
| 309.6 (SBA: remove completed dungeon) | Not yet | No | Session 2 |
| 309.7 (completing a dungeon) | Partial (dungeons_completed) | Yes | test_dungeon_state_default |
| 725.1 (initiative designation) | Yes (has_initiative field) | Yes | test_dungeon_state_default |
| 725.2 (initiative triggers) | Not yet | No | Session 2+ |
| 725.3 (one player at a time) | Yes (Option type enforces) | Implicit | |
| 725.4 (initiative on player leave) | Not yet | No | Session 2+ |

## Room Graph Correctness

| Dungeon | Room Count | Graph Correct? | Effects Correct? | Notes |
|---------|-----------|----------------|------------------|-------|
| Lost Mine of Phandelver | 7 | Yes | Partial | Storeroom AddCounter no-ops (F3), Fungi Cavern arbitrary (F7) |
| Dungeon of the Mad Mage | 9 | Yes | Partial | Twisted Caverns/Runestone Caverns/Mad Wizard's Lair simplified (documented) |
| Tomb of Annihilation | 5 | **NO** | Partial | Oubliette exits wrong room (F1); Veils/Sandfall/Oubliette simplified (documented) |
| The Undercity | 7 | Yes | Partial | Arena Goad no-ops (F4); Throne simplified (documented); Storeroom-like F3 in Forge |

## Token Correctness

| Token | P/T | Color | Types | Keywords | Legendary? | Correct? |
|-------|-----|-------|-------|----------|-----------|----------|
| Goblin 1/1 | 1/1 | Red | Creature Goblin | -- | No | Yes |
| Treasure | 0/0 | Colorless | Artifact Treasure | -- | No | Yes (has mana ability) |
| Skeleton 1/1 | 1/1 | Black | Creature Skeleton | -- | No | Yes |
| Skeleton 4/1 | 4/1 | Black | Creature Skeleton | Menace | No | Yes |
| The Atropal | 4/4 | Black | Creature God Horror | Deathtouch | **Missing** | **No (F2)** |
