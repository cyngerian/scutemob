# Primitive Batch Plan: PB-33 -- Copy/Clone + Exile/Flicker Timing

**Generated**: 2026-03-27
**Primitives**: G-22 (Copy/clone DSL wiring) + G-28 (Exile/flicker timing with delayed triggers)
**CR Rules**: 707.2, 707.5, 707.6, 707.9, 603.7, 610.3, 610.3a-c
**Cards affected**: ~33 (existing fixes; new card defs deferred to backfill)
**Dependencies**: PB-22 S3 (Flicker effect -- DONE), Batch 10 (Soulbond -- DONE)
**Deferred items from prior PBs**:
- Clone/copy ETB choice (PB-13j) -- BecomeCopyOf exists, clone ETB replacement not yet wired
- Scion of the Ur-Dragon copy-self (PB-17) -- needs `EffectTarget::LastSearchResult`
**Sessions**: 2 (session 1: engine changes, session 2: card def fixes + tests)

## Primitive Specification

### G-22: Copy/Clone DSL Wiring

The engine already has `Effect::BecomeCopyOf` (Layer 1 copy of an existing permanent) and
`Effect::CreateTokenCopy` (create a token that copies a permanent). The gaps are:

1. **`CreateTokenCopy` lacks modification support** -- cards like Kiki-Jiki ("except it has
   haste"), Helm of the Host ("except it isn't legendary"), Miirym ("except it isn't
   legendary") need `except_not_legendary: bool` and `gains_haste: bool` fields.
2. **`CreateTokenCopy` lacks delayed sacrifice/exile** -- tokens created by Kiki-Jiki, The
   Fire Crystal, Mirage Phalanx need "sacrifice at beginning of next end step" which requires
   the G-28 delayed trigger infrastructure.
3. **`EffectTarget::EquippedCreature`** -- Helm of the Host needs to reference the equipped
   creature as the copy source. The engine has `EffectFilter::AttachedCreature` for continuous
   effects but no `EffectTarget` variant for it.
4. **Clone ETB replacement** (CR 707.5) -- creatures that "enter as a copy" (Mockingbird) are
   replacement effects on the ETB event, not triggered abilities. This is the most complex
   part and is partially deferred -- full clone support requires interactive target choice at
   ETB time (M10 dependency). For now, `BecomeCopyOf` with `EffectTarget::DeclaredTarget`
   handles the case where the copy target is pre-chosen.

### G-28: Exile/Flicker Timing (Delayed Return Triggers)

The engine has `Effect::Flicker` (immediate exile + return) but lacks delayed return patterns:

1. **"Exile, return at beginning of next end step"** -- The Eternal Wanderer +1, Nezahal's
   activated ability. These create a delayed triggered ability (CR 603.7) that fires at the
   next end step.
2. **"Exile until this leaves the battlefield"** -- Brutal Cathar, Banisher Priest pattern.
   Uses CR 610.3 "until" semantics: exile is linked to the source permanent; when the source
   leaves, the exiled card returns.
3. **"Sacrifice/exile at beginning of next end step"** for created tokens -- Kiki-Jiki, Chandra
   Flamecaller, Mobilize tokens (Voice of Victory, Zurgo Stormrender). These need a delayed
   trigger that targets a specific object.
4. **"Return to hand at beginning of next end step"** -- The Locust God's death trigger.
   Delayed trigger returning an object from a non-battlefield zone.

The `DelayedTrigger` struct is currently a stub (`source: ObjectId` only). It needs to be
expanded into a real system.

## CR Rule Text

### CR 707.2 (Copying Objects)
> When copying an object, the copy acquires the copiable values of the original object's
> characteristics [...] as modified by other copy effects, by its face-down status, and by
> "as . . . enters" and "as . . . is turned face up" abilities that set power and toughness
> (and may also set additional characteristics). Other effects (including type-changing and
> text-changing effects), status, counters, and stickers are not copied.

### CR 707.5 (Enters as a Copy)
> An object that enters the battlefield "as a copy" or "that's a copy" of another object
> becomes a copy as it enters the battlefield. It doesn't enter the battlefield, and then
> become a copy of that permanent. If the text that's being copied includes any abilities
> that replace the enters-the-battlefield event (such as "enters with" or "as [this] enters"
> abilities), those abilities will take effect.

### CR 707.9a (Copy with Modifications)
> Some copy effects cause the copy to gain an ability as part of the copying process. This
> ability becomes part of the copiable values for the copy, along with any other abilities
> that were copied.

### CR 603.7 (Delayed Triggered Abilities)
> An effect may create a delayed triggered ability that can do something at a later time.
> A delayed triggered ability will contain "when," "whenever," or "at," although that word
> won't usually begin the ability.

### CR 603.7b
> A delayed triggered ability will trigger only once -- the next time its trigger event
> occurs -- unless it has a stated duration, such as "this turn."

### CR 610.3 (Exile "Until" Effects)
> Some one-shot effects cause an object to change zones "until" a specified event occurs.
> A second one-shot effect is created immediately after the specified event. This second
> one-shot effect returns the object to its previous zone.

### CR 610.3c
> An object returned to the battlefield this way returns under its owner's control unless
> otherwise specified.

## Engine Changes

### Change 1: Expand `DelayedTrigger` struct

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Replace the stub `DelayedTrigger` with a real struct supporting multiple trigger patterns.

```rust
/// A delayed trigger waiting for a condition (CR 603.7).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DelayedTrigger {
    /// The source that created this delayed trigger (for tracking).
    pub source: ObjectId,
    /// The controller of the delayed trigger (CR 603.7d/e).
    pub controller: PlayerId,
    /// The object this delayed trigger acts upon.
    pub target_object: ObjectId,
    /// What this delayed trigger does when it fires.
    pub action: DelayedTriggerAction,
    /// When this delayed trigger fires.
    pub timing: DelayedTriggerTiming,
    /// Whether this trigger has already fired (CR 603.7b: fires only once).
    pub fired: bool,
}

/// What a delayed trigger does when it fires.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelayedTriggerAction {
    /// Return the target object from exile to the battlefield under its owner's control.
    /// CR 610.3c: returns under owner's control unless otherwise specified.
    ReturnFromExileToBattlefield { tapped: bool },
    /// Return the target object from exile to its owner's hand.
    ReturnFromExileToHand,
    /// Sacrifice the target object.
    SacrificeObject,
    /// Exile the target object.
    ExileObject,
}

/// When a delayed trigger fires.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelayedTriggerTiming {
    /// At the beginning of the next end step (any player's).
    AtNextEndStep,
    /// At the beginning of the target object's owner's next end step.
    AtOwnersNextEndStep,
    /// When the source permanent leaves the battlefield (CR 610.3).
    WhenSourceLeavesBattlefield,
}
```

**Pattern**: Follow `ActiveRestriction` pattern at line ~520 in stubs.rs.

### Change 2: Update `HashInto` for expanded `DelayedTrigger`

**File**: `crates/engine/src/state/hash.rs`
**Action**: Update the `HashInto for DelayedTrigger` impl (line ~1347) to hash all new fields.
Also add `HashInto` impls for `DelayedTriggerAction` and `DelayedTriggerTiming`.

### Change 3: Process delayed triggers in `end_step_actions`

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: In `end_step_actions()` (line ~534), add a new section after the existing
Unearth/Encore/Dash/Blitz scans that processes `state.delayed_triggers` with
`timing == AtNextEndStep` or `timing == AtOwnersNextEndStep` (where owner matches
active player). For each matching trigger:
- Mark `fired = true`
- Queue a `PendingTrigger` with a new `PendingTriggerKind::DelayedAction` variant
- Pass the `DelayedTriggerAction` through `TriggerData::DelayedAction { action, target }`

**CR**: 603.7b -- fires only once.

### Change 4: Process "when source leaves" delayed triggers

**File**: `crates/engine/src/rules/sba.rs` or `crates/engine/src/rules/abilities.rs`
**Action**: In the event handler for `PermanentLeftBattlefield` (or in `check_triggers`),
scan `state.delayed_triggers` for entries with
`timing == WhenSourceLeavesBattlefield` where `source` matches the leaving permanent.
Queue a return trigger for the exiled object.

**CR**: 610.3 -- "immediately after the specified event."

### Change 5: Resolve delayed action triggers

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: In `flush_pending_triggers`, add handling for `PendingTriggerKind::DelayedAction`.
Create a `StackObjectKind::KeywordTrigger { keyword: KeywordAbility::DelayedAction, data }`
or a new SOK variant `DelayedActionTrigger`. At resolution, execute the action:
- `ReturnFromExileToBattlefield`: move from exile to battlefield, register statics, fire ETB
- `ReturnFromExileToHand`: move from exile to hand
- `SacrificeObject`: sacrifice the target
- `ExileObject`: exile the target

**CR**: 603.7 -- delayed triggers use the stack and can be responded to.

### Change 6: Add `PendingTriggerKind::DelayedAction` variant

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add to `PendingTriggerKind` enum:
```rust
/// CR 603.7: A delayed triggered ability fires.
DelayedAction,
```

### Change 7: Add `TriggerData::DelayedAction` variant

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add to `TriggerData` enum:
```rust
/// Delayed trigger action payload (CR 603.7).
DelayedAction {
    action: DelayedTriggerAction,
    target: ObjectId,
},
```

### Change 8: Add new `Effect::ExileWithDelayedReturn` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add after `Effect::Flicker`:
```rust
/// CR 610.3 / CR 603.7: Exile a permanent and create a delayed trigger to return it.
///
/// Unlike `Flicker` (immediate return), this exiles the target and registers a
/// `DelayedTrigger` that will return the object at a later time.
///
/// Used by: The Eternal Wanderer +1, Brutal Cathar ETB, Nezahal activated ability.
ExileWithDelayedReturn {
    target: EffectTarget,
    /// When the exiled object returns.
    return_timing: DelayedTriggerTiming,
    /// Whether the returned object enters tapped.
    return_tapped: bool,
    /// Where the object returns to (battlefield or hand).
    return_to: DelayedReturnDestination,
},
```

Add the destination enum:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelayedReturnDestination {
    Battlefield,
    Hand,
}
```

### Change 9: Dispatch `Effect::ExileWithDelayedReturn` in effects/mod.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add execution logic after the `Effect::Flicker` arm (around line 3972).
Steps:
1. Resolve targets
2. Verify target is on the battlefield (or appropriate zone)
3. Exile the target (using replacement effect check like `Effect::ExileObject`)
4. If exile succeeded, register a `DelayedTrigger` on `state.delayed_triggers` with:
   - `source` = `ctx.source` (the permanent or spell creating the delayed trigger)
   - `target_object` = the new exile ID
   - `timing` = from the effect's `return_timing`
   - `action` = `ReturnFromExileToBattlefield { tapped }` or `ReturnFromExileToHand`

**CR**: 610.3, 603.7a

### Change 10: Add `Effect::CreateTokenCopyWithMods` or extend `CreateTokenCopy`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Extend `Effect::CreateTokenCopy` with optional modification fields:
```rust
CreateTokenCopy {
    /// The permanent to copy.
    source: EffectTarget,
    /// If true, the token enters tapped and attacking.
    enters_tapped_and_attacking: bool,
    /// If true, the token copy is not legendary (CR 707.9b).
    except_not_legendary: bool,
    /// If true, the token gains haste (CR 707.9a).
    gains_haste: bool,
    /// Optional delayed action on the created token.
    delayed_action: Option<(DelayedTriggerTiming, DelayedTriggerAction)>,
},
```

**Pattern**: This extends the existing variant rather than adding a new one. The
existing two call sites (Thousand-Faced Shadow, Mist Syndicate Naga) just need
`except_not_legendary: false, gains_haste: false, delayed_action: None` added.

### Change 11: Update `CreateTokenCopy` execution for modifications

**File**: `crates/engine/src/effects/mod.rs`
**Action**: In the `Effect::CreateTokenCopy` dispatch (line ~3667), after creating the
token and applying the Layer 1 copy effect:
- If `except_not_legendary`: register an additional continuous effect removing
  `SuperType::Legendary` (Layer 4 type modification) on the token
- If `gains_haste`: register a continuous effect adding `KeywordAbility::Haste`
  (Layer 6) on the token
- If `delayed_action` is Some: register a `DelayedTrigger` with the token as
  `target_object`

### Change 12: Add `EffectTarget::EquippedCreature`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add to `EffectTarget` enum:
```rust
/// The creature the source Equipment is attached to (via `attached_to`).
/// Used by Helm of the Host ("copy of equipped creature").
EquippedCreature,
```

### Change 13: Resolve `EffectTarget::EquippedCreature` in effects/mod.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: In `resolve_effect_target_list` (or equivalent), add handling:
```rust
EffectTarget::EquippedCreature => {
    if let Some(obj) = state.objects.get(&ctx.source) {
        if let Some(attached_id) = obj.attached_to {
            vec![ResolvedTarget::Object(attached_id)]
        } else { vec![] }
    } else { vec![] }
}
```

### Change 14: Cleanup fired delayed triggers

**File**: `crates/engine/src/rules/turn_actions.rs` or `crates/engine/src/rules/sba.rs`
**Action**: At the end of `end_step_actions()`, remove all `DelayedTrigger` entries where
`fired == true` from `state.delayed_triggers`. Also clean up triggers whose `source`
no longer exists (for `WhenSourceLeavesBattlefield` after the source has left and the
return has been processed).

### Change 15: Hash and display updates for new types

**Files requiring exhaustive match updates**:

| File | Match expression | Action |
|------|-----------------|--------|
| `crates/engine/src/state/hash.rs` | `HashInto for DelayedTrigger` (L1347) | Rewrite to hash all new fields |
| `crates/engine/src/state/hash.rs` | `HashInto for Effect` (L4317+) | Add arm for `ExileWithDelayedReturn` |
| `crates/engine/src/state/hash.rs` | `HashInto for EffectTarget` (find) | Add arm for `EquippedCreature` |
| `crates/engine/src/state/hash.rs` | `HashInto for CreateTokenCopy` (L4743) | Hash new fields |
| `crates/engine/src/effects/mod.rs` | `execute_effect` match | Add arm for `ExileWithDelayedReturn` |
| `crates/engine/src/effects/mod.rs` | `resolve_effect_target_list` | Add arm for `EquippedCreature` |
| `crates/engine/src/effects/mod.rs` | `CreateTokenCopy` arm | Update for new fields |
| `crates/engine/src/rules/resolution.rs` | `flush_pending_triggers` | Add `DelayedAction` kind |
| `crates/engine/src/rules/turn_actions.rs` | `end_step_actions` | Process delayed triggers |
| `tools/replay-viewer/src/view_model.rs` | `EffectTarget` match (if any) | Add arm |
| `tools/tui/src/play/panels/stack_view.rs` | `StackObjectKind` match | Add arm if new SOK added |

## Card Definition Fixes

### G-22 Cards (Copy/Clone)

#### kiki_jiki_mirror_breaker.rs
**Oracle text**: "Haste. {T}: Create a token that's a copy of target nonlegendary creature you control, except it has haste. Sacrifice it at the beginning of the next end step."
**Current state**: TODO -- copy token + delayed sacrifice not in DSL.
**Fix**: Add `AbilityDefinition::Activated` with `CreateTokenCopy { source: DeclaredTarget{0}, enters_tapped_and_attacking: false, except_not_legendary: false, gains_haste: true, delayed_action: Some((AtNextEndStep, SacrificeObject)) }`. Target: nonlegendary creature you control.

#### the_fire_crystal.rs
**Oracle text**: "{4}{R}{R}, {T}: Create a token that's a copy of target creature you control. Sacrifice it at the beginning of the next end step."
**Current state**: TODO -- copy token + delayed sacrifice.
**Fix**: Add `AbilityDefinition::Activated` with `CreateTokenCopy` + `delayed_action: Some((AtNextEndStep, SacrificeObject))`.

#### helm_of_the_host.rs
**Oracle text**: "At the beginning of combat on your turn, create a token that's a copy of equipped creature, except the token isn't legendary. That token gains haste."
**Current state**: TODO -- no `EffectTarget::EquippedCreature`.
**Fix**: Add `AbilityDefinition::Triggered { trigger_condition: TriggerCondition::AtBeginningOfCombat, effect: CreateTokenCopy { source: EquippedCreature, except_not_legendary: true, gains_haste: true, delayed_action: None, enters_tapped_and_attacking: false } }`.

#### miirym_sentinel_wyrm.rs
**Oracle text**: "Whenever another nontoken Dragon you control enters, create a token that's a copy of it, except the token isn't legendary."
**Current state**: TODO -- "CreateCopyToken not in DSL."
**Fix**: Use `CreateTokenCopy { source: TriggeringCreature, except_not_legendary: true, gains_haste: false, delayed_action: None, enters_tapped_and_attacking: false }`. Trigger: WheneverAnotherCreatureEnters (needs filter: nontoken Dragon you control -- check if this trigger condition exists or if it needs `ETBTriggerFilter` with subtype filter).

#### scion_of_the_ur_dragon.rs (DEFERRED from PB-17)
**Oracle text**: "Search library for Dragon, put into graveyard. Scion becomes a copy of it until end of turn."
**Current state**: TODO -- needs `EffectTarget::LastSearchResult`.
**Fix**: DEFER -- `EffectTarget::LastSearchResult` requires a new tracking mechanism in `EffectContext` to remember the card found by the previous `SearchLibrary` in the same `Sequence`. This is a larger change. Tag as remaining TODO.

#### thespians_stage.rs
**Oracle text**: "{2}, {T}: Thespian's Stage becomes a copy of target land, except it has this ability."
**Current state**: `BecomeCopyOf` is wired. TODO notes "except it has this ability" retained ability not expressible.
**Fix**: Partially addressable -- the "except it has this ability" requires copy-with-retained-ability support. This is a known limitation. Keep TODO but note it's cosmetic (the copy works, it just loses the copy ability).

#### mockingbird.rs (PARTIALLY DEFERRED)
**Oracle text**: "You may have this creature enter as a copy of any creature on the battlefield with mana value <= amount of mana spent, except it's a Bird in addition to its other types and it has flying."
**Current state**: TODO -- clone ETB replacement with X-cost filter.
**Fix**: DEFER -- full clone ETB replacement (CR 707.5) requires interactive choice at ETB time. The X-cost filter on available targets is an additional complexity. This requires M10 interactive choice support.

#### mirage_phalanx.rs
**Oracle text**: "As long as paired, each has 'At beginning of combat, create a token copy with haste, exile at end of combat.'"
**Current state**: TODO -- copy token with haste + delayed exile at end of combat.
**Fix**: The "exile at end of combat" timing is not AtNextEndStep but AtEndOfCombat. This is already handled by `end_combat()` in turn_actions.rs for myriad tokens. Wire the soulbond grant to use `CreateTokenCopy { source: Source, gains_haste: true, delayed_action: Some((AtNextEndStep, ExileObject)) }`. Note: "end of combat" vs "next end step" -- check oracle text. Oracle says "Exile it at end of combat" so this needs end-of-combat timing. Add `DelayedTriggerTiming::AtEndOfCombat` or use the existing myriad pattern.

#### saw_in_half.rs (DEFER)
**Oracle text**: "Create two copy-tokens with halved stats."
**Current state**: TODO -- halved stat copy tokens not expressible.
**Fix**: DEFER -- stat modification on copy tokens requires power/toughness override support beyond simple boolean flags.

#### springheart_nantuko.rs (PARTIALLY BLOCKED)
**Oracle text**: "Landfall -- may pay {1}{G} if attached. If you do, create a token copy of that creature. Otherwise, create 1/1 Insect."
**Current state**: TODO -- Bestow not in DSL, conditional copy-or-Insect branch.
**Fix**: The copy-token part is now expressible with `CreateTokenCopy`. But the conditional branch ("if you paid, copy; otherwise, Insect") and Bestow remain gaps. Keep TODO; add comment noting CreateTokenCopy is available.

#### plumb_the_forbidden.rs (PARTIALLY BLOCKED)
**Oracle text**: "Sacrifice creatures + copy for each sacrificed."
**Current state**: TODO -- sacrifice-as-additional-cost + copy-per-sacrificed.
**Fix**: DEFER -- the "copy this spell for each creature sacrificed" requires spell-copying infrastructure, not `CreateTokenCopy` (permanent copies). This is a spell-copy gap, not a permanent-copy gap.

#### rings_of_brighthearth.rs (DEFER)
**Current state**: TODO -- "copy activated ability" requires ability-copying, not permanent-copying.
**Fix**: DEFER -- ability copying is distinct from G-22.

#### drana_and_linvala.rs (DEFER)
**Current state**: TODO -- "ability copying from opponents' creatures" is a static layer effect.
**Fix**: DEFER -- this is a complex static ability suppression + grant, not a simple copy effect.

#### shiko_paragon_of_the_way.rs (DEFER)
**Current state**: TODO -- "copy-and-cast-from-exile" is spell-copy + free-cast.
**Fix**: DEFER.

#### sunken_palace.rs (DEFER)
**Current state**: TODO -- "copy that spell or ability" requires spell-copying.
**Fix**: DEFER.

### G-28 Cards (Exile/Flicker Timing)

#### the_eternal_wanderer.rs
**Oracle text**: "+1: Exile up to one target artifact or creature. Return that card to the battlefield under its owner's control at the beginning of that player's next end step."
**Current state**: TODO -- +1 exile + return at owner's next end step.
**Fix**: Use `ExileWithDelayedReturn { target: DeclaredTarget{0}, return_timing: AtOwnersNextEndStep, return_tapped: false, return_to: Battlefield }`. Target: artifact or creature (up to one).

#### brutal_cathar.rs
**Oracle text**: "When this creature enters, exile target creature an opponent controls until this creature leaves the battlefield."
**Current state**: DSL gap: ExileUntilLeaves.
**Fix**: Use `ExileWithDelayedReturn { target: DeclaredTarget{0}, return_timing: WhenSourceLeavesBattlefield, return_tapped: false, return_to: Battlefield }`. Add ETB trigger.

#### nezahal_primal_tide.rs
**Oracle text**: "Discard three cards: Exile Nezahal. Return it to the battlefield tapped under its owner's control at the beginning of the next end step."
**Current state**: TODO -- discard 3 + exile + delayed return.
**Fix**: Add activated ability with `Cost::Multiple(vec![Cost::DiscardCards(3)])` and `ExileWithDelayedReturn { target: Source, return_timing: AtNextEndStep, return_tapped: true, return_to: Battlefield }`.

#### the_locust_god.rs
**Oracle text**: "When The Locust God dies, return it to its owner's hand at the beginning of the next end step."
**Current state**: TODO -- "When dies, return to hand at next end step."
**Fix**: Add triggered ability with `TriggerCondition::WhenDies` and effect that registers a delayed trigger. However, this is a death trigger that creates a delayed trigger to return from graveyard to hand -- not an exile effect. Use `ExileWithDelayedReturn` is wrong here. Instead, this needs a new approach: on death, register a `DelayedTrigger { target_object: <graveyard id>, timing: AtNextEndStep, action: ReturnFromGraveyardToHand }`. Add `DelayedTriggerAction::ReturnFromGraveyardToHand` variant. Or handle as a CardDef triggered ability that fires at end step. This is complex -- **assess scope**.

Actually, The Locust God's death trigger is: "when dies, create a delayed trigger that returns it to hand at next end step." The object changes zones (battlefield -> graveyard), so the delayed trigger must track the new ObjectId. The simplest approach: add an `Effect::CreateDelayedTrigger` effect used inside a WhenDies triggered ability. But this adds significant complexity.

**Simpler approach**: Tag the object in the graveyard with a flag `return_to_hand_at_end_step: bool` on `GameObject`, then scan in `end_step_actions()` similarly to how Unearth/Dash are scanned. This follows the existing pattern.

#### chandra_flamecaller.rs
**Oracle text**: "+1: Create two 3/1 red Elemental creature tokens with haste. Exile them at the beginning of the next end step."
**Current state**: TODO -- token creation with delayed exile.
**Fix**: Use `CreateToken` for the tokens, then register `DelayedTrigger` entries for each created token with `action: ExileObject, timing: AtNextEndStep`. This requires the `CreateToken` effect to track created token IDs for subsequent delayed trigger registration. Use `Effect::Sequence([CreateToken{...}, RegisterDelayedTrigger { target: LastCreatedPermanent, ... }])` pattern. Or extend `CreateToken` with an optional `delayed_action` field similar to `CreateTokenCopy`.

**Simpler approach**: Add optional `delayed_action: Option<(DelayedTriggerTiming, DelayedTriggerAction)>` to `CreateToken` as well. This mirrors the `CreateTokenCopy` extension.

#### puppeteer_clique.rs
**Oracle text**: "ETB: put target creature card from opponent's GY onto battlefield under your control. It gains haste. At the beginning of your next end step, exile it."
**Current state**: TODO -- reanimate + haste + delayed exile.
**Fix**: The reanimate (MoveZone from opponent's graveyard) and haste grant are separate gaps. The delayed exile part is `DelayedTrigger { timing: AtOwnersNextEndStep, action: ExileObject }`. Partially fixable -- the delayed exile is expressible but the reanimate-from-opponent-GY with haste grant has additional gaps. Keep partial TODO.

#### voice_of_victory.rs
**Oracle text**: "Mobilize 2: Create two tapped attacking tokens. Sacrifice them at beginning of next end step."
**Current state**: TODO -- delayed sacrifice.
**Fix**: Extend the existing `CreateToken` effect with `delayed_action: Some((AtNextEndStep, SacrificeObject))`.

#### zurgo_stormrender.rs
**Oracle text**: "Mobilize 1: Create one tapped attacking token. Sacrifice at beginning of next end step."
**Current state**: TODO -- delayed sacrifice + token-LTB trigger.
**Fix**: Same as Voice of Victory. The "whenever a creature token leaves" trigger is a separate gap.

#### teferi_hero_of_dominaria.rs
**Oracle text**: "+1: Draw a card. At the beginning of the next end step, untap up to two lands."
**Current state**: TODO -- delayed trigger to untap lands.
**Fix**: This requires `DelayedTriggerAction::UntapLands { count: 2 }` -- a new action variant. Add it if scope permits, or keep as TODO with delayed trigger infrastructure note.

#### pact_of_negation.rs
**Oracle text**: "Counter target spell. At the beginning of your next upkeep, pay {3}{U}{U} or lose the game."
**Current state**: TODO -- delayed upkeep trigger with pay-or-lose.
**Fix**: DEFER -- this is an upkeep delayed trigger, not an end-step one. Different timing entirely.

#### mana_drain.rs
**Oracle text**: "Counter target spell. At the beginning of your next main phase, add {C} equal to its mana value."
**Current state**: TODO -- delayed main phase trigger.
**Fix**: DEFER -- different timing (main phase, not end step).

#### basri_ket.rs
**Oracle text**: "-2: Whenever nontoken creatures attack this turn, create that many tokens."
**Current state**: TODO -- delayed triggered abilities scoped to current turn.
**Fix**: DEFER -- this is a turn-scoped triggered ability, not a delayed return/sacrifice.

## Revised Scope

After analysis, the core engine changes are:

**In scope (fixable with reasonable effort):**
1. Expand `DelayedTrigger` struct
2. Process delayed triggers in `end_step_actions`
3. Process "when source leaves" delayed triggers
4. `Effect::ExileWithDelayedReturn` (exile + delayed return)
5. Extend `CreateTokenCopy` with modifications + delayed action
6. Extend `CreateToken` with optional `delayed_action`
7. `EffectTarget::EquippedCreature`
8. `PendingTriggerKind::DelayedAction` + resolution

**Cards fixable**: ~18 existing defs
**Cards deferred**: ~12 (complex interactions beyond copy/flicker scope)

### Fixable Card Defs (18)

G-22 Copy/Clone:
1. `kiki_jiki_mirror_breaker.rs` -- CreateTokenCopy + gains_haste + delayed sacrifice
2. `the_fire_crystal.rs` -- CreateTokenCopy + delayed sacrifice
3. `helm_of_the_host.rs` -- CreateTokenCopy + EquippedCreature + except_not_legendary + gains_haste
4. `miirym_sentinel_wyrm.rs` -- CreateTokenCopy + TriggeringCreature + except_not_legendary

G-28 Exile/Flicker Timing:
5. `the_eternal_wanderer.rs` -- ExileWithDelayedReturn +1 (AtOwnersNextEndStep)
6. `brutal_cathar.rs` -- ExileWithDelayedReturn ETB (WhenSourceLeavesBattlefield)
7. `nezahal_primal_tide.rs` -- ExileWithDelayedReturn activated (AtNextEndStep, tapped)
8. `chandra_flamecaller.rs` -- CreateToken + delayed_action (ExileObject, AtNextEndStep)
9. `voice_of_victory.rs` -- CreateToken + delayed_action (SacrificeObject, AtNextEndStep)
10. `zurgo_stormrender.rs` -- CreateToken + delayed_action (SacrificeObject, AtNextEndStep)
11. `the_locust_god.rs` -- WhenDies + delayed return to hand (needs flag-based approach)
12. `puppeteer_clique.rs` -- Partial: delayed exile part fixable, reanimate gap remains
13. `mirage_phalanx.rs` -- Partial: CreateTokenCopy + delayed exile at end of combat
14. `thousand_faced_shadow.rs` -- Update existing CreateTokenCopy for new fields
15. `mist_syndicate_naga.rs` -- Update existing CreateTokenCopy for new fields

### Deferred Card Defs

- `scion_of_the_ur_dragon.rs` -- needs `EffectTarget::LastSearchResult` (PB-17 deferred)
- `mockingbird.rs` -- clone ETB replacement with X-cost filter (M10)
- `saw_in_half.rs` -- halved stat copy tokens
- `plumb_the_forbidden.rs` -- spell-copy per sacrifice
- `rings_of_brighthearth.rs` -- ability-copying
- `drana_and_linvala.rs` -- static ability suppression + grant
- `shiko_paragon_of_the_way.rs` -- spell-copy + free-cast
- `sunken_palace.rs` -- spell-copy on mana spend
- `pact_of_negation.rs` -- upkeep delayed trigger
- `mana_drain.rs` -- main phase delayed trigger
- `basri_ket.rs` -- turn-scoped triggered ability
- `teferi_hero_of_dominaria.rs` -- delayed untap (different action type, low priority)

## Unit Tests

**File**: `crates/engine/tests/delayed_triggers.rs` (new file)
**Tests to write**:
- `test_delayed_trigger_sacrifice_at_next_end_step` -- CR 603.7b: token created with delayed sacrifice is sacrificed at end step
- `test_delayed_trigger_fires_only_once` -- CR 603.7b: verify the trigger fires exactly once
- `test_exile_with_delayed_return_at_end_step` -- CR 610.3/603.7: exile and return at next end step
- `test_exile_until_source_leaves` -- CR 610.3: exile until source LTB, object returns when source removed
- `test_exile_until_source_leaves_object_identity` -- CR 400.7: returned object is a new object
- `test_create_token_copy_with_haste` -- CR 707.9a: copy gains haste
- `test_create_token_copy_not_legendary` -- CR 707.9b: copy isn't legendary
- `test_equipped_creature_target` -- EffectTarget::EquippedCreature resolves correctly
- `test_create_token_copy_with_delayed_sacrifice` -- Kiki-Jiki pattern: copy + sacrifice at end step

**Pattern**: Follow tests in `crates/engine/tests/copy.rs` and `crates/engine/tests/abilities.rs`.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (18 fixed, ~12 explicitly deferred with rationale)
- [ ] Existing CreateTokenCopy callers updated for new fields
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in fixable card defs
- [ ] `DelayedTrigger` hash updated
- [ ] Replay viewer and TUI compile with new types

## Risks & Edge Cases

- **CR 400.7 (Object identity)**: When an exiled permanent returns, it is a NEW object. The
  `DelayedTrigger.target_object` stores the exile-zone ObjectId. When returning, must use
  the exile-zone ID (not the original battlefield ID). If the object has already left exile
  (e.g., processed by another effect), the delayed trigger should silently do nothing
  (CR 603.7c).
- **Multiple delayed triggers on same object**: If a token has both a delayed sacrifice AND
  something else happens, both triggers go on the stack independently. APNAP ordering applies.
- **"When source leaves" + zone change identity**: For Brutal Cathar pattern, the source's
  ObjectId must be tracked. If the source is flickered (leaves and returns as a new object),
  the old delayed trigger should fire (the old source left), returning the exiled creature.
  The new Brutal Cathar can then exile a new creature.
- **Stifle interaction**: Delayed triggers use the stack (CR 603.7), so they can be countered
  by Stifle. The implementation must queue them as `PendingTrigger` (not inline execution).
  This is already the design.
- **`CreateTokenCopy` new fields are additive**: Existing callers (Thousand-Faced Shadow,
  Mist Syndicate Naga) just need the new fields set to defaults. No behavior change.
- **`DelayedTrigger` cleanup**: Must remove fired triggers and triggers whose source/target
  no longer exist. Risk of memory leak if not cleaned up properly.
- **End-of-combat timing**: Mirage Phalanx says "exile at end of combat" which is different
  from "at next end step." The existing `end_combat()` in turn_actions.rs handles myriad
  token exile. May need `DelayedTriggerTiming::AtEndOfCombat` or use the same inline pattern.
