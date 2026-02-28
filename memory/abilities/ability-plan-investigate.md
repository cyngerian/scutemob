# Ability Plan: Investigate

**Generated**: 2026-02-28
**CR**: 701.16 (NOT 701.36 -- the ability-coverage doc has a typo; 701.36 is Populate)
**Priority**: P3
**Similar abilities studied**: Surveil (Effect::Surveil, tests/surveil.rs), Connive (Effect::Connive), Food tokens (food_token_spec)

## CR Rule Text

701.16. Investigate

  701.16a "Investigate" means "Create a Clue token." See rule 111.10f.

111.10f A Clue token is a colorless Clue artifact token with "{2}, Sacrifice this token:
Draw a card."

No CR 701.16b exists in the current rules (the "investigate N times" semantics from the
user prompt referenced 701.36b, which is actually Populate 701.36b). However, cards like
Teysa, Opulent Oligarch say "investigate for each opponent who lost life" and Tamiyo Meets
the Story Circle says "investigate twice for each card discarded." Per ruling on Tamiyo
(2024-06-07): "If you're instructed to investigate multiple times, those actions are
sequential, meaning you'll create that many Clue tokens one at a time."

## Key Edge Cases

- **Sequential token creation**: When instructed to investigate N times, create N separate
  Clue tokens one at a time (ruling 2024-06-07). Each creation is a separate event that
  can be responded to (e.g., for "whenever you create a token" triggers).
- **Investigate is a keyword action, not a keyword ability**: It appears in oracle text as
  an action verb, not a static/triggered/activated keyword. Similar to Scry and Surveil.
- **Fizzled spells don't investigate**: If a spell with "investigate" has targets and all
  targets become illegal, the spell doesn't resolve and you don't investigate (multiple
  rulings from 2016-04-08 on cards like Expose Evil, Confront the Unknown, Press for
  Answers, etc.).
- **Clue tokens have NO tap requirement**: Unlike Food tokens ({2}, {T}, Sacrifice), Clue
  tokens only require {2} + sacrifice. A tapped Clue can still be activated. This is
  already correctly implemented in `clue_token_spec()` at `card_definition.rs:862-864`.
- **"Whenever you investigate" triggers**: Some future cards may have this pattern. Adding
  a dedicated `GameEvent::Investigated` and `TriggerEvent::ControllerInvestigates` enables
  this pattern, following the Surveil/Scried precedent.
- **Multiplayer**: No special multiplayer considerations. Each player investigates
  independently. Teysa, Opulent Oligarch investigates once per opponent who lost life
  (counts eliminated players too -- ruling 2024-02-02).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant / keyword action
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

### Existing Infrastructure

- `clue_token_spec(count: u32)` at `crates/engine/src/cards/card_definition.rs:852-882`
  -- fully implemented, correct characteristics per CR 111.10f.
- Thraben Inspector at `crates/engine/src/cards/definitions.rs:2918-2936` -- uses inline
  `Effect::CreateToken { spec: clue_token_spec(1) }` for its ETB investigate trigger.
- Comprehensive Clue token tests at `crates/engine/tests/clue_tokens.rs` -- 11 tests
  covering spec characteristics, activated ability, sacrifice-as-cost, draw on resolution,
  SBA cleanup (CR 704.5d), multiplayer, and `Effect::CreateToken` integration.
- Game script at `test-data/generated-scripts/stack/098_thraben_inspector_clue_token_draw.json`
  (pending_review) -- covers the full cast-ETB-investigate-activate-draw flow.

## Implementation Steps

### Step 1: Add Effect::Investigate Variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `Effect::Investigate { count: EffectAmount }` variant to the `Effect` enum.
Place it in the "Permanents" section after `CreateToken` (around line 324).

```rust
/// CR 701.16a: "Investigate" means "Create a Clue token."
/// Creates `count` Clue tokens sequentially (ruling 2024-06-07:
/// "If you're instructed to investigate multiple times, those actions
/// are sequential, meaning you'll create that many Clue tokens one
/// at a time.").
Investigate { count: EffectAmount },
```

**Pattern**: Follow `Effect::Surveil` at line 389-392 (keyword action with player-implicit
count parameter). The player is implicitly the controller (from `EffectContext`), so no
`player` field is needed -- unlike Surveil, Investigate always creates tokens under the
controller's control.

**Why EffectAmount**: Supports `EffectAmount::Fixed(1)` for simple "investigate" and
`EffectAmount::Fixed(2)` for "investigate twice." Also supports dynamic counts like
`EffectAmount::CountOf(...)` for Teysa's "investigate for each opponent who lost life."

### Step 2: Hash Support

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `Effect::Investigate` to the `HashInto` impl for `Effect`. Use discriminant
36 (next available after Connive at 35).

```rust
// CR 701.16a: Investigate (discriminant 36)
Effect::Investigate { count } => {
    36u8.hash_into(hasher);
    count.hash_into(hasher);
}
```

**Location**: After the `Effect::Connive` arm (around line 2535).

### Step 3: GameEvent::Investigated

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add `GameEvent::Investigated { player: PlayerId, count: u32 }` variant.
Place it after `Surveilled` (around line 703).

```rust
/// A player performed an investigate action (CR 701.16a).
///
/// Emitted by `Effect::Investigate` when the player creates Clue tokens.
/// Enables "whenever you investigate" triggers (future cards).
Investigated { player: PlayerId, count: u32 },
```

**Hash**: Add to `state/hash.rs` `GameEvent` `HashInto` impl with discriminant 82
(next available after PoisonCountersGiven at 81).

```rust
// CR 701.16a: Investigated (discriminant 82)
GameEvent::Investigated { player, count } => {
    82u8.hash_into(hasher);
    player.hash_into(hasher);
    count.hash_into(hasher);
}
```

### Step 4: Effect Resolver

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add handler for `Effect::Investigate` in `execute_effect`. Delegate to
`clue_token_spec` + the existing `CreateToken` logic. Place after `Effect::CreateToken`
handler (around line 415).

```rust
// CR 701.16a: Investigate — create N Clue tokens sequentially
Effect::Investigate { count } => {
    let n = resolve_amount(state, count, ctx).max(0) as u32;
    if n > 0 {
        let spec = crate::cards::card_definition::clue_token_spec(1);
        // Create tokens one at a time (ruling 2024-06-07)
        for _ in 0..n {
            let obj = make_token(&spec, ctx.controller);
            if let Ok(id) = state.add_object(obj, ZoneId::Battlefield) {
                events.push(GameEvent::TokenCreated {
                    player: ctx.controller,
                    object_id: id,
                });
                events.push(GameEvent::PermanentEnteredBattlefield {
                    player: ctx.controller,
                    object_id: id,
                });
            }
        }
        events.push(GameEvent::Investigated {
            player: ctx.controller,
            count: n,
        });
    }
}
```

**Key design decision**: Create each token individually with `clue_token_spec(1)` in a
loop, rather than using `clue_token_spec(n)` with `count: n`. This is because
`Effect::CreateToken` uses `spec.count` for the loop, but the ruling says tokens are
created "one at a time" -- semantically they should be individual create-token events.
In practice both approaches produce the same result since `CreateToken` already loops
over `spec.count`, but the one-at-a-time approach is clearer about the ruling.

**Note**: Import `clue_token_spec` at the top of `effects/mod.rs` or use full path.

### Step 5: TriggerEvent::ControllerInvestigates (Optional, Recommended)

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `ControllerInvestigates` variant to `TriggerEvent` enum (after
`SourceConnives`, around line 190).

```rust
/// CR 701.16a: Triggers when the controller of this permanent investigates.
/// Used by "whenever you investigate" cards (e.g., future cards).
/// The controller match is done at trigger-collection time in `rules/abilities.rs`.
ControllerInvestigates,
```

**Hash**: Add to `state/hash.rs` `TriggerEvent` `HashInto` impl with discriminant 16.

```rust
// CR 701.16a: ControllerInvestigates trigger -- discriminant 16
TriggerEvent::ControllerInvestigates => 16u8.hash_into(hasher),
```

**Trigger wiring** in `crates/engine/src/rules/abilities.rs`: Add a `GameEvent::Investigated`
arm to `check_triggers` that collects `ControllerInvestigates` triggers from permanents
controlled by the investigating player. Follow the `GameEvent::Surveilled` /
`TriggerEvent::ControllerSurveils` pattern at `abilities.rs:1426-1445`.

```rust
GameEvent::Investigated { player, .. } => {
    // CR 701.16a: "Whenever you investigate" triggers on all permanents
    // controlled by the investigating player.
    let controller_sources: Vec<ObjectId> = state
        .objects
        .iter()
        .filter(|(_, obj)| {
            obj.controller == *player
                && obj.zone == ZoneId::Battlefield
        })
        .map(|(id, _)| *id)
        .collect();

    for obj_id in controller_sources {
        collect_triggers_for_event(
            state,
            &mut triggers,
            TriggerEvent::ControllerInvestigates,
            Some(obj_id),
            None,
        );
    }
}
```

### Step 6: Update Thraben Inspector Card Definition (Optional)

**File**: `crates/engine/src/cards/definitions.rs`
**Action**: Update Thraben Inspector's ETB effect from:
```rust
effect: Effect::CreateToken { spec: clue_token_spec(1) },
```
to:
```rust
effect: Effect::Investigate { count: EffectAmount::Fixed(1) },
```

**Location**: Line 2932.

**Rationale**: Using the semantic `Investigate` effect enables `GameEvent::Investigated`
to fire, which powers "whenever you investigate" triggers. The inline `CreateToken` form
works but does not emit the `Investigated` event.

**CR citation update**: Change the comment at line 2920 from `CR 701.36a` to `CR 701.16a`.

### Step 7: Unit Tests

**File**: `crates/engine/tests/investigate.rs` (new file)
**Tests to write**:

1. **`test_investigate_creates_clue_token`** -- CR 701.16a: `Effect::Investigate { count: Fixed(1) }`
   creates exactly one Clue token on the controller's battlefield. Verify the token has
   the Clue subtype, Artifact type, is colorless, and has the correct activated ability
   (no tap required, {2} generic + sacrifice, draws 1 card).

2. **`test_investigate_twice_creates_two_clues`** -- Ruling 2024-06-07: `Effect::Investigate
   { count: Fixed(2) }` creates two separate Clue tokens. Verify two `TokenCreated`
   events and two Clue tokens on the battlefield.

3. **`test_investigate_emits_investigated_event`** -- CR 701.16a: After investigating,
   `GameEvent::Investigated { player, count }` is emitted. Verify the event appears
   in the returned events with correct player and count.

4. **`test_investigate_zero_does_nothing`** -- Edge case: `Effect::Investigate { count:
   Fixed(0) }` creates no tokens and emits no `Investigated` event. Follow Surveil's
   pattern (CR 701.25c: surveil 0 is a no-op).

5. **`test_investigate_multiplayer_correct_controller`** -- In a 4-player game, when
   player 3 investigates, the Clue token is created under player 3's control (not the
   active player). Verify `TokenCreated.player == p3` and the token's controller is p3.

6. **`test_investigate_clue_can_be_activated`** -- Integration test: investigate creates
   a Clue, then the controller activates the Clue's ability (pay {2}, sacrifice) to
   draw a card. Verify the full flow works end-to-end.

**Pattern**: Follow `crates/engine/tests/surveil.rs` for structure, helpers (find_object,
pass_all), and `execute_effect` direct-call pattern. Follow `crates/engine/tests/clue_tokens.rs`
for Clue-specific assertions.

### Step 8: Match Arm Audit

When adding `Effect::Investigate`, grep for all `Effect::` match expressions to ensure
exhaustive matching. Key files to check:

- `crates/engine/src/effects/mod.rs` -- main execute_effect match (Step 4)
- `crates/engine/src/state/hash.rs` -- Effect HashInto impl (Step 2)
- `crates/engine/src/rules/` -- any file that pattern-matches on Effect

Run: `grep -rn "Effect::" crates/engine/src/ | grep -v "test" | grep "=>"` to find all
match arms.

Similarly for `GameEvent::Investigated`:
- `crates/engine/src/state/hash.rs` -- GameEvent HashInto impl (Step 3)
- `crates/engine/src/rules/abilities.rs` -- check_triggers match (Step 5)
- `tools/replay-viewer/src/view_model.rs` -- may need updating if it matches GameEvent

And for `TriggerEvent::ControllerInvestigates`:
- `crates/engine/src/state/hash.rs` -- TriggerEvent HashInto impl (Step 5)
- `crates/engine/src/rules/abilities.rs` -- collect_triggers_for_event callers
- `crates/engine/src/testing/replay_harness.rs` -- TriggerCondition → TriggerEvent mapping
  (follow the `ControllerSurveils` pattern at replay_harness.rs:878)

### Step 9: Replay Harness TriggerCondition Mapping

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add mapping from `TriggerCondition::WhenControllerInvestigates` (if added to
card_definition.rs) to `TriggerEvent::ControllerInvestigates`. Follow the
`ControllerSurveils` pattern at line 878.

**Note**: This is only needed if a `TriggerCondition` variant for investigate is added.
For now, Investigate cards use `TriggerCondition::WhenEntersBattlefield` (Thraben Inspector)
or other existing trigger conditions with `Effect::Investigate` as the effect payload.
This step can be deferred.

### Step 10: Card Definition (Later Phase)

**Suggested card**: Magnifying Glass ({3}, Artifact)
- `{T}: Add {C}.`
- `{4}, {T}: Investigate.`

This is a good candidate because:
1. It has two abilities (mana ability + activated ability that investigates).
2. The investigate is an activated ability, not ETB, testing a different trigger path.
3. Simple enough for a card definition without complex interactions.

**Alternative**: Tireless Tracker ({2}{G}, Creature -- Human Scout 3/2)
- `Landfall -- Whenever a land you control enters, investigate.`
- `Whenever you sacrifice a Clue, put a +1/+1 counter on this creature.`

This is more complex (requires Landfall trigger + sacrifice-Clue trigger) but tests
the `ControllerInvestigates` / `Investigated` event pathway and is a Commander staple.

### Step 11: Game Script (Later Phase)

**Suggested scenario**: "Magnifying Glass investigate and activate Clue"
- Player casts Magnifying Glass
- Player activates Magnifying Glass's investigate ability ({4}, {T})
- Clue token appears on battlefield
- Player activates Clue's ability ({2}, sacrifice) to draw a card
- Final state: Magnifying Glass tapped, Clue gone, player drew 1 card

**Subsystem directory**: `test-data/generated-scripts/stack/`

### Step 12: Ability Coverage Doc Update

**File**: `docs/mtg-engine-ability-coverage.md`
**Action**: Update Investigate row from `none` to `validated`. Fix CR number from
`701.36` to `701.16`.

## Interactions to Watch

- **Token doubling effects (Doubling Season, Anointed Procession)**: Investigate creates
  tokens via `CreateToken` semantics. If token doubling is implemented, it should naturally
  apply since the implementation delegates to the same `add_object` path as `CreateToken`.
  The `Investigated` event count should reflect the intended count, not the doubled count
  (following how the CR describes it -- the instruction is "investigate N times," and each
  investigation creates a Clue; doubling applies per-creation).

- **Replacement effects on token creation**: Effects like "if one or more tokens would be
  created, instead create twice that many" apply at the `add_object` level, not at the
  `Investigate` level. The current implementation will naturally inherit these behaviors.

- **"Whenever you investigate" triggers**: Currently no cards in the engine use this
  pattern, but the infrastructure (`TriggerEvent::ControllerInvestigates` +
  `GameEvent::Investigated`) prepares for future cards like Lonis, Cryptozoologist.

- **Clue token activated ability interaction with Stax effects**: Effects like
  "artifacts don't untap" or "activated abilities cost {2} more" interact with Clue
  tokens. These are handled by existing infrastructure (cost modification, untap
  restrictions) and are not Investigate-specific.

## Complexity Assessment

This is a **low-complexity** implementation:
- The core mechanic (create Clue token) is already fully working via `clue_token_spec`.
- Adding `Effect::Investigate` is a thin wrapper around existing `CreateToken` logic.
- The hash/event/trigger infrastructure follows well-established patterns (Surveil, Connive).
- No new game systems or complex interactions are introduced.

Estimated touch points: 6 files modified, 1 new test file, ~80 lines of production code,
~200 lines of test code.
