# Ability Review: Morph Mini-Milestone

**Date**: 2026-03-08
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.37 (Morph), 702.37b (Megamorph), 702.168 (Disguise), 701.40 (Manifest), 701.58 (Cloak), 708 (Face-Down Spells and Permanents)
**Files reviewed**:
- `crates/engine/src/state/types.rs` (FaceDownKind, TurnFaceUpMethod, AltCostKind::Morph, KW 153-157)
- `crates/engine/src/state/game_object.rs` (face_down_as field)
- `crates/engine/src/state/hash.rs` (face_down_as hashing, FaceDownRevealed hashing, TurnFaceUpTrigger hashing)
- `crates/engine/src/state/mod.rs` (zone-change face_down_as clearing)
- `crates/engine/src/state/builder.rs` (face_down_as initialization)
- `crates/engine/src/state/stack.rs` (TurnFaceUpTrigger SOK)
- `crates/engine/src/state/stubs.rs` (PendingTriggerKind::TurnFaceUp)
- `crates/engine/src/rules/command.rs` (Command::TurnFaceUp, CastSpell.face_down_kind)
- `crates/engine/src/rules/engine.rs` (handle_turn_face_up, Command::TurnFaceUp dispatch)
- `crates/engine/src/rules/casting.rs` (AltCostKind::Morph handling, face-down on stack)
- `crates/engine/src/rules/layers.rs` (face-down characteristic override)
- `crates/engine/src/rules/resolution.rs` (face-down spell resolution, TurnFaceUpTrigger resolution)
- `crates/engine/src/rules/events.rs` (PermanentTurnedFaceUp, FaceDownRevealed)
- `crates/engine/src/rules/abilities.rs` (PermanentTurnedFaceUp trigger dispatch)
- `crates/engine/src/effects/mod.rs` (Effect::Manifest, Effect::Cloak)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::Morph/Megamorph/Disguise, TriggerCondition::WhenTurnedFaceUp)
- `crates/engine/tests/morph.rs` (14 tests)
- `tools/replay-viewer/src/view_model.rs` (KW + SOK match arms)
- `tools/tui/src/play/panels/stack_view.rs` (SOK match arm)
- `crates/simulator/src/legal_actions.rs` (checked for TurnFaceUp support)

## Verdict: needs-fix

The Morph mini-milestone is a substantial and well-structured implementation covering five related
face-down mechanics. The core model (FaceDownKind enum, layer override, TurnFaceUp command) is
sound. However, there is one HIGH finding (Manifest/Cloak ETB suppression gap) and two MEDIUM
findings (TurnFaceUpTrigger resolution drops ability_index, FaceDownRevealed event never emitted)
that need to be addressed.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `effects/mod.rs:1986` | **Manifest/Cloak emit PermanentEnteredBattlefield without ETB suppression.** CR 708.3 violation. |
| 2 | MEDIUM | `resolution.rs:6970-6999` | **TurnFaceUpTrigger resolution ignores ability_index, fires first WhenTurnedFaceUp ability.** Multiple triggers fire wrong ability. |
| 3 | MEDIUM | `events.rs:1202` | **FaceDownRevealed event defined but never emitted.** CR 708.9 zone-change reveal missing. |
| 4 | MEDIUM | `legal_actions.rs` | **LegalActionProvider missing TurnFaceUp and morph-cast.** Simulator cannot generate morph actions. |
| 5 | LOW | `engine.rs:1538` | **`.unwrap()` in engine library code.** Convention violation. |
| 6 | LOW | `casting.rs:88` | **`face_down_kind` parameter ignored (prefixed `_`).** Player cannot choose between Morph and Disguise if card has both. |

### Finding Details

#### Finding 1: Manifest/Cloak ETB suppression gap

**Severity**: HIGH
**File**: `crates/engine/src/effects/mod.rs:1986`
**CR Rule**: 708.3 -- "Objects that are put onto the battlefield face down are turned face down before they enter the battlefield, so the permanent's enters-the-battlefield abilities won't trigger (if triggered) or have any effect (if static)."
**Issue**: The `Effect::Manifest` and `Effect::Cloak` handlers emit `GameEvent::PermanentEnteredBattlefield` after setting `face_down = true` and `face_down_as`. When `check_triggers` processes this event, it calls `collect_triggers_for_event(SelfEntersBattlefield, ...)` which reads raw `obj.characteristics.triggered_abilities` (not layer-calculated values). If the manifested card has ETB triggered abilities in its raw characteristics (populated at build time from ObjectSpec), those triggers will fire even though the card is face-down. The morph cast path in resolution.rs correctly suppresses ETB with `is_face_down_entering`, but the Manifest/Cloak effect path has no such suppression.

**Current test cards have no ETB triggers, so this bug is not exercised.**

**Fix**: In `collect_triggers_for_event` (abilities.rs:5585), add a guard after the object lookup: if `obj.status.face_down && obj.face_down_as.is_some()` and `event_type == TriggerEvent::SelfEntersBattlefield`, skip the object (face-down permanents have no abilities, CR 708.2a). Alternatively, add a face-down guard in the Manifest/Cloak effect handlers to not emit PermanentEnteredBattlefield, but the former approach is more robust as it covers any future face-down entry path. Also add a test that manifests a creature with an ETB trigger and verifies the trigger does NOT fire.

#### Finding 2: TurnFaceUpTrigger resolution ignores ability_index

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:6970-6999`
**CR Rule**: 708.8 -- "As a face-down permanent is turned face up, its copiable values revert to its normal copiable values."
**Issue**: `StackObjectKind::TurnFaceUpTrigger` does not carry `ability_index`. At resolution, the code rescans the CardDefinition for the first `WhenTurnedFaceUp` triggered ability and executes it, then `break`s. If `check_triggers` (abilities.rs:5494-5505) creates multiple PendingTriggers for a card with multiple WhenTurnedFaceUp abilities (each with a different `ability_index`), all resulting TurnFaceUpTrigger SOKs would fire the FIRST WhenTurnedFaceUp ability's effect instead of their respective effects. This is a correctness bug for cards with multiple "when turned face up" triggers (rare but rules-legal).

**Fix**: Add `ability_index: usize` to `TurnFaceUpTrigger` SOK. In `abilities.rs:5500`, set `ability_index` from the PendingTrigger. In the flush code (abilities.rs, PendingTriggerKind::TurnFaceUp arm), pass `ability_index` through to the SOK. In resolution.rs, use the stored `ability_index` to index into `def.abilities` directly instead of scanning. Update the hash in hash.rs accordingly.

#### Finding 3: FaceDownRevealed event never emitted

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/events.rs:1202`
**CR Rule**: 708.9 -- "If a face-down permanent or a face-down component of a merged permanent moves from the battlefield to any other zone, its owner must reveal it to all players as they move it."
**Issue**: `GameEvent::FaceDownRevealed` is defined (events.rs:1202) and hashed (hash.rs:3305), but is never emitted anywhere in the engine. When a face-down permanent dies or is bounced, no reveal event fires. The plan (D7) specified that this event should be emitted when a face-down permanent leaves the battlefield. The `state/mod.rs` `move_object_to_zone` correctly clears `face_down_as` on zone change, and the test (`test_face_down_dies_reveal`) only checks that `card_id` is preserved in the graveyard and `face_down` is cleared -- it does not assert on the FaceDownRevealed event.

For the engine-only phase this is low-impact (the engine knows the card identity). However, the network layer (M10) will depend on this event to broadcast reveals to all players, so it should be wired now.

**Fix**: In `state/mod.rs:move_object_to_zone()`, before clearing `face_down_as` on the new object, check if the old object was `face_down && face_down_as.is_some()` and the source zone is Battlefield. If so, look up the card name from `card_id` via the card registry and emit `FaceDownRevealed { player: old_object.controller, permanent: old_id, card_name }`. Return the event from `move_object_to_zone` (may require signature change) or add a post-move check in the callers. Update `test_face_down_dies_reveal` to assert the event is present.

#### Finding 4: LegalActionProvider missing TurnFaceUp and morph-cast

**Severity**: MEDIUM
**File**: `crates/simulator/src/legal_actions.rs`
**CR Rule**: 702.37e -- "Any time you have priority, you may turn a face-down permanent you control with a morph ability face up."
**Issue**: The LegalActionProvider does not generate `Command::TurnFaceUp` actions for face-down permanents, nor does it generate morph-cast actions (`CastSpell` with `alt_cost: Some(AltCostKind::Morph)`). The plan (Step 8) specified this should be implemented. Without this, the simulator's bots cannot interact with morph mechanics.

**Fix**: In `legal_actions.rs`, add a scan for face-down permanents with `face_down_as.is_some()` controlled by the active player. For each, determine valid TurnFaceUpMethod values (MorphCost if card has Morph/Megamorph, DisguiseCost if Disguise, ManaCost if Manifest/Cloak and creature card with mana cost). Check mana availability. Generate a `Command::TurnFaceUp` for each valid method. Also add morph-cast generation for cards in hand with Morph/Megamorph/Disguise abilities when the player has 3+ generic mana.

#### Finding 5: `.unwrap()` in engine library code

**Severity**: LOW
**File**: `crates/engine/src/rules/engine.rs:1538`
**CR Rule**: N/A (conventions.md)
**Issue**: `let face_down_as = obj.face_down_as.clone().unwrap();` -- The guard at lines 1527-1530 ensures `face_down_as` is `Some`, so the `unwrap()` is logically safe. However, per `memory/conventions.md`: "Engine crate uses typed errors -- never `unwrap()` or `expect()` in engine logic."

**Fix**: Replace with: `let face_down_as = obj.face_down_as.clone().ok_or_else(|| GameStateError::InvalidCommand("TurnFaceUp: no face_down_as".into()))?;` — or use `// SAFETY: guarded above` comment if preferring the unwrap for clarity.

#### Finding 6: `face_down_kind` Command field ignored

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:88`
**CR Rule**: 702.37c / 702.168b -- casting procedure says the player announces which ability they are using.
**Issue**: The `CastSpell.face_down_kind` field is accepted in the Command but prefixed with `_` and ignored in `handle_cast_spell`. Instead, the code auto-detects the FaceDownKind from the card's AbilityDefinition (lines 3666-3689), preferring Disguise > Megamorph > Morph. If a card had both Morph and Disguise (unlikely but rules-legal), the player cannot choose. The plan specified using the command's `face_down_kind`.

**Fix**: Use the `face_down_kind` parameter when provided: `let kind = face_down_kind.unwrap_or_else(|| auto_detect_kind(...))`. Validate that the chosen kind matches an ability the card actually has.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.37a (Morph cast) | Yes | Yes | test_morph_cast_face_down_basic |
| 702.37b (Megamorph) | Yes | Yes | test_megamorph_counter |
| 702.37c (Casting procedure) | Yes | Yes | test_morph_cast_face_down_is_creature_spell |
| 702.37d (Can't cast face-down normally) | Yes | No | Implicit -- no other path exists |
| 702.37e (Turn face up special action) | Yes | Yes | test_morph_turn_face_up |
| 702.37f (X in morph cost) | No | No | Deferred (no X-morph cards) |
| 702.168a (Disguise ward {2}) | Yes | Yes | test_disguise_ward |
| 702.168b (Disguise casting) | Yes | Yes | Via AltCostKind::Morph path |
| 702.168d (Disguise turn face up) | Yes | No | No explicit disguise turn-face-up test |
| 701.40a (Manifest) | Yes | Yes | test_manifest_creature_turn_face_up |
| 701.40b (Manifest turn face up) | Yes | Yes | test_manifest_creature_turn_face_up |
| 701.40c (Manifest + morph) | Yes | Yes | test_manifest_with_morph |
| 701.40d (Manifest + disguise) | No | No | Not tested, but code supports it |
| 701.40e (Manifest multiple) | Partial | No | Effect handles one card |
| 701.40f (Manifest blocked) | Partial | No | Empty library handled; prohibition not checked |
| 701.40g (Instant/sorcery stays down) | Yes | Yes | test_manifest_noncreature_stuck |
| 701.58a (Cloak) | Yes | Yes | test_cloak_ward |
| 701.58b (Cloak turn face up) | Yes | No | Same logic as manifest ManaCost |
| 701.58c (Cloak + morph) | Yes | No | Code supports it; no test |
| 701.58d (Cloak + disguise) | Yes | No | Code supports it; no test |
| 701.58g (Cloak instant/sorcery) | Yes | No | Same logic as 701.40g; no separate test |
| 708.2 (Face-down characteristics) | Yes | Yes | test_morph_face_down_characteristics_layer |
| 708.2a (Default characteristics) | Yes | Yes | Multiple tests verify 2/2 no-name |
| 708.2b (Can't turn face-down twice) | No | No | Not implemented |
| 708.3 (ETB suppression) | Partial | Yes | Morph cast path correct; **Manifest/Cloak ETB bug (F1)** |
| 708.4 (Face-down on stack) | Yes | Yes | test_morph_cast_face_down_is_creature_spell |
| 708.5 (Look at own face-down) | N/A | N/A | Network-layer concern |
| 708.6 (Differentiate face-down) | N/A | N/A | UI concern |
| 708.8 (Turn face up revert) | Yes | Yes | test_morph_when_turned_face_up_trigger |
| 708.9 (Reveal on zone change) | Partial | Partial | face_down_as cleared; **FaceDownRevealed event not emitted (F3)** |
| 708.10 (Face-down copies) | No | No | Deferred (stretch goal) |
| 708.11 ("As turned face up") | No | No | Not implemented (requires replacement-like timing) |
