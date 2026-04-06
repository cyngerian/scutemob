# Primitive Batch Plan: PB-I — Grant Flash

**Generated**: 2026-04-05
**Primitive**: Flash grant mechanism — allows "you may cast [spells] as though they had flash"
**CR Rules**: 702.8 (Flash), 601.3b (casting as though flash), 601.5a (flash conditions persist through casting), 304.5 (instant timing), 307.1 (sorcery timing)
**Cards affected**: 4 (3 existing fixes + 1 new)
**Dependencies**: none (PB-37 `EffectDuration::UntilYourNextTurn` already exists)
**Deferred items from prior PBs**: none directly relevant

## Primitive Specification

The engine needs a mechanism to grant "cast as though it had flash" permissions to a player
for specific categories of spells, with varying durations. Currently, `is_instant_speed` in
`casting.rs` (line 537) only checks `CardType::Instant` or `KeywordAbility::Flash` on the
card itself. No mechanism exists for external effects to grant instant-speed casting permission.

Flash grants are NOT continuous effects in the layer sense — they don't modify object
characteristics through the layer system. They modify casting legality, similar to how
`GameRestriction` / `ActiveRestriction` restrict casting. The design mirrors the restriction
pattern: a `Vector<FlashGrant>` on `GameState`, checked at casting-validation time.

Additionally, Teferi Time Raveler's passive ("Each opponent can cast spells only any time
they could cast a sorcery") requires a new `GameRestriction` variant. Per CR 101.2,
restrictions override permissions — a ruling on Teferi confirms this: "If an effect allows
opponents to cast a spell as though it had flash, the restriction of Teferi's first ability
takes precedence over that permission."

## CR Rule Text

**CR 702.8a**: Flash is a static ability that functions in any zone from which you could play
the card it's on. "Flash" means "You may play this card any time you could cast an instant."

**CR 702.8b**: Multiple instances of flash on the same object are redundant.

**CR 601.3b**: If an effect allows a player to cast a spell with certain qualities as though
it had flash, that player may consider any choices to be made during that spell's proposal
that may cause that spell's qualities to change. If any such choices could cause that effect
to apply, that player may begin to cast that spell as though it had flash.

**CR 601.5a**: Once a player has begun casting a spell that had flash because certain
conditions were met or that could be cast as though it had flash because certain conditions
were met (see 601.3d), they may continue to cast that spell as though it had flash even if
those conditions stop being met.

**CR 307.1**: A player who has priority may cast a sorcery card from their hand during a main
phase of their turn when the stack is empty.

**CR 304.5**: If text states that a player may do something "any time they could cast an
instant" or "only as an instant," it means only that the player must have priority.

## Engine Changes

### Change 1: FlashGrant struct and FlashGrantFilter enum in `stubs.rs`

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `FlashGrantFilter` enum and `FlashGrant` struct after `ActiveRestriction` (near line 586).

```rust
/// Filter for which spells a flash grant applies to.
/// CR 601.3b: "a spell with certain qualities"
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlashGrantFilter {
    /// All spells (Borne Upon a Wind: "You may cast spells...")
    AllSpells,
    /// Sorcery spells only (Complete the Circuit, Teferi +1: "sorcery spells")
    Sorceries,
    /// Green creature spells only (Yeva: "green creature spells")
    GreenCreatures,
}

/// An active flash grant allowing a player to cast certain spells at instant speed.
///
/// Follows the same pattern as `ActiveRestriction`:
/// - `source: Option<ObjectId>` for cleanup when the source leaves the battlefield
///   (None for spell-based grants that expire by duration)
/// - Registered by `Effect::GrantFlash` or `AbilityDefinition::StaticFlashGrant`
/// - Checked in `casting.rs` timing validation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlashGrant {
    /// ObjectId of the source (permanent for static grants, None for one-shot spell effects).
    pub source: Option<ObjectId>,
    /// The player who receives the flash permission.
    pub player: PlayerId,
    /// Which spells this grant applies to.
    pub filter: FlashGrantFilter,
    /// How long the grant lasts.
    pub duration: crate::state::continuous_effect::EffectDuration,
}
```

**Pattern**: Follow `ActiveRestriction` at line 578.

### Change 2: `flash_grants` field on `GameState`

**File**: `crates/engine/src/state/mod.rs`
**Action**: Add `flash_grants: Vector<FlashGrant>` field after `restrictions` (near line 123).
Also add `FlashGrant, FlashGrantFilter` to the imports from `stubs` at line 48-49.

```rust
/// Active flash grants (cast-as-though-flash permissions, CR 601.3b).
///
/// Allows players to cast certain spells at instant speed. Checked at casting-validation
/// time in casting.rs. Registered by Effect::GrantFlash or AbilityDefinition::StaticFlashGrant.
/// Cleaned up by duration expiry or source leaving battlefield.
#[serde(default)]
pub flash_grants: Vector<FlashGrant>,
```

**File**: `crates/engine/src/state/builder.rs`
**Action**: Add `flash_grants: Vector::new()` to `GameState` construction (near line 320, after `restrictions`).

### Change 3: `GameRestriction::OpponentsCanOnlyCastAtSorcerySpeed` variant

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add new variant to `GameRestriction` enum (after `MaxNonartifactSpellsPerTurn`).

```rust
/// "Each opponent can cast spells only any time they could cast a sorcery."
/// (Teferi, Time Raveler)
/// CR 307.5: "only as a sorcery" = must have priority, main phase, empty stack.
/// CR 101.2: restriction overrides permission — this beats flash grants.
OpponentsCanOnlyCastAtSorcerySpeed,
```

### Change 4: `Effect::GrantFlash` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `Effect::GrantFlash` variant in the Effect enum (after `RegisterReplacementEffect`).

```rust
/// CR 601.3b: Grant the controller permission to cast certain spells as though
/// they had flash. Registers a `FlashGrant` on GameState.
///
/// Used by Borne Upon a Wind, Complete the Circuit, Teferi's +1 loyalty ability.
GrantFlash {
    filter: crate::state::stubs::FlashGrantFilter,
    duration: crate::state::continuous_effect::EffectDuration,
},
```

### Change 5: `AbilityDefinition::StaticFlashGrant` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::StaticFlashGrant` variant (after `AdditionalLandPlays`, discriminant 72).

```rust
/// Static flash grant — "You may cast [X] spells as though they had flash."
/// (Yeva, Nature's Herald)
///
/// Registers a `FlashGrant` with `WhileSourceOnBattlefield` duration when the
/// permanent enters the battlefield. Cleaned up when source leaves.
///
/// Discriminant 72.
StaticFlashGrant {
    filter: crate::state::stubs::FlashGrantFilter,
},
```

### Change 6: Effect::GrantFlash dispatch in `effects/mod.rs`

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm for `Effect::GrantFlash` in the main `execute_effect` dispatch (after `RegisterReplacementEffect`).

```rust
// CR 601.3b: Register a flash grant for the controller.
Effect::GrantFlash { filter, duration } => {
    use crate::state::continuous_effect::EffectDuration;
    use crate::state::player::PlayerId;
    let controller = ctx.controller;
    // Resolve UntilYourNextTurn(PlayerId(0)) placeholder to controller.
    let resolved_duration = match duration {
        EffectDuration::UntilYourNextTurn(PlayerId(0)) => {
            EffectDuration::UntilYourNextTurn(controller)
        }
        other => *other,
    };
    state.flash_grants.push_back(crate::state::stubs::FlashGrant {
        source: None,
        player: controller,
        filter: filter.clone(),
        duration: resolved_duration,
    });
}
```

### Change 7: StaticFlashGrant registration in `replacement.rs`

**File**: `crates/engine/src/rules/replacement.rs`
**Action**: Add match arm for `AbilityDefinition::StaticFlashGrant` in `register_static_continuous_effects` (before the `_ => {}` catch-all near line 1812).

```rust
// PB-I: Register a static flash grant (Yeva-style).
AbilityDefinition::StaticFlashGrant { filter } => {
    state.flash_grants.push_back(crate::state::stubs::FlashGrant {
        source: Some(new_id),
        player: controller,
        filter: filter.clone(),
        duration: crate::state::continuous_effect::EffectDuration::WhileSourceOnBattlefield,
    });
}
```

### Change 8: Timing validation check in `casting.rs`

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Modify the sorcery-speed timing check at line 3015. Add a flash grant check
before the sorcery-speed restriction. Also add an `OpponentsCanOnlyCastAtSorcerySpeed`
check in `check_cast_restrictions`.

At line 3015, change:
```rust
if !is_instant_speed && !casting_with_madness && !cast_with_miracle {
```
to:
```rust
// CR 601.3b: Check flash grants before enforcing sorcery-speed timing.
let has_flash_grant = has_active_flash_grant(state, player, &chars);
if !is_instant_speed && !casting_with_madness && !cast_with_miracle && !has_flash_grant {
```

Add the helper function (near `check_cast_restrictions`, around line 5370):

```rust
/// CR 601.3b: Check if the player has an active flash grant that applies to this spell.
///
/// Iterates `state.flash_grants` and checks:
/// 1. Source still on battlefield (for WhileSourceOnBattlefield grants)
/// 2. Player matches
/// 3. Filter matches the spell's characteristics
fn has_active_flash_grant(
    state: &GameState,
    player: PlayerId,
    chars: &Characteristics,
) -> bool {
    state.flash_grants.iter().any(|grant| {
        // Check player match
        if grant.player != player {
            return false;
        }
        // Check source validity for WhileSourceOnBattlefield grants
        if let Some(src) = grant.source {
            let on_bf = state
                .objects
                .get(&src)
                .map(|o| matches!(o.zone, ZoneId::Battlefield))
                .unwrap_or(false);
            if !on_bf {
                return false;
            }
        }
        // Check filter match
        match &grant.filter {
            FlashGrantFilter::AllSpells => true,
            FlashGrantFilter::Sorceries => {
                chars.card_types.contains(&CardType::Sorcery)
            }
            FlashGrantFilter::GreenCreatures => {
                chars.card_types.contains(&CardType::Creature)
                    && chars.colors.contains(&Color::Green)
            }
        }
    })
}
```

Add `OpponentsCanOnlyCastAtSorcerySpeed` arm in `check_cast_restrictions` (after `MaxNonartifactSpellsPerTurn`, around line 5363):

```rust
// Teferi, Time Raveler: "Each opponent can cast spells only any time
// they could cast a sorcery."
// CR 307.5: sorcery speed = priority + main phase + stack empty.
// CR 101.2: restriction overrides permission (beats flash grants).
GameRestriction::OpponentsCanOnlyCastAtSorcerySpeed => {
    if player != controller {
        let is_own_main = state.turn.active_player == player
            && matches!(state.turn.step, Step::PreCombatMain | Step::PostCombatMain);
        if !is_own_main || !state.stack_objects.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "restriction: opponents can only cast spells at sorcery speed (CR 101.2)".into(),
            ));
        }
    }
}
```

### Change 9: Flash grant cleanup — duration expiry

Two cleanup sites, matching the existing pattern for continuous effects and replacement effects:

**Site A — UntilEndOfTurn: `expire_end_of_turn_effects` in `layers.rs`**

**File**: `crates/engine/src/rules/layers.rs`
**Action**: In `expire_end_of_turn_effects` (line 1247), add flash grant expiry after the
existing continuous effect and replacement effect cleanup (after line 1277).

```rust
// PB-I: Expire UntilEndOfTurn flash grants (CR 514.2).
let keep_grants: im::Vector<crate::state::stubs::FlashGrant> = state
    .flash_grants
    .iter()
    .filter(|g| g.duration != EffectDuration::UntilEndOfTurn)
    .cloned()
    .collect();
state.flash_grants = keep_grants;
```

This is called from `turn_actions.rs` at line 1411 during the cleanup step, which is the
correct CR 514.2 timing for "until end of turn" effect expiry.

**Site B — UntilYourNextTurn: `expire_until_next_turn_effects` in `layers.rs`**

**File**: `crates/engine/src/rules/layers.rs`
**Action**: In `expire_until_next_turn_effects` (line 1287), add flash grant expiry after
the existing continuous effect and replacement effect cleanup (after line 1305).

```rust
// PB-I: Expire UntilYourNextTurn flash grants for this player (CR 611.2b).
let keep_grants: im::Vector<crate::state::stubs::FlashGrant> = state
    .flash_grants
    .iter()
    .filter(|g| g.duration != EffectDuration::UntilYourNextTurn(active_player))
    .cloned()
    .collect();
state.flash_grants = keep_grants;
```

This is called from `turn_actions.rs` at line 1018 during the untap step.

**Site C — Stale WhileSourceOnBattlefield cleanup: `reset_turn_state` in `turn_actions.rs`**

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: In `reset_turn_state` (line 1454), add periodic stale grant cleanup (after line 1529).
This is a housekeeping sweep — the inline source-validity check in `has_active_flash_grant`
already prevents stale grants from having any effect, but this keeps the Vector clean.

```rust
// PB-I: Clean up WhileSourceOnBattlefield flash grants whose source left.
state.flash_grants.retain(|g| {
    match g.source {
        Some(src) => state.objects.get(&src)
            .map(|o| matches!(o.zone, crate::state::ZoneId::Battlefield))
            .unwrap_or(false),
        None => true,
    }
});
```

### Change 10: Exhaustive match updates

Files requiring new match arms or hash entries:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `state/hash.rs` | `HashInto for GameRestriction` | L1467 | Add `OpponentsCanOnlyCastAtSorcerySpeed => 8u8` |
| `state/hash.rs` | `HashInto for Effect` | L4544 | Add `GrantFlash { filter, duration }` arm, discriminant 78 |
| `state/hash.rs` | `HashInto for AbilityDefinition` | L5189 | Add `StaticFlashGrant { filter }` arm, discriminant 72 |
| `state/hash.rs` | `HashInto for GameState` | ~L5770 | Hash `flash_grants` after `restrictions` |
| `state/hash.rs` | New impl | — | Add `HashInto for FlashGrantFilter` (3 variants: 0/1/2) |
| `state/hash.rs` | New impl | — | Add `HashInto for FlashGrant` (hash source, player, filter, duration) |
| `effects/mod.rs` | main dispatch match | L216 | Add `Effect::GrantFlash` arm |
| `rules/casting.rs` | `check_cast_restrictions` | L5246 | Add `OpponentsCanOnlyCastAtSorcerySpeed` arm |
| `rules/casting.rs` | timing check | L3015 | Add `has_flash_grant` check |

### Change 11: Simulator legal_actions.rs update

**File**: `crates/simulator/src/legal_actions.rs`
**Action**: At line 219, modify the timing check to also consider flash grants. Add a helper
or inline check:

```rust
let can_cast = if is_instant || has_flash {
    true
} else if check_flash_grants(state, player, &obj.characteristics) {
    true
} else {
    is_main_phase && stack_empty && is_active
};
```

Add a local helper `check_flash_grants` that mirrors the engine's `has_active_flash_grant`.
Import `FlashGrantFilter` from the engine crate.

Also add `OpponentsCanOnlyCastAtSorcerySpeed` restriction check if the simulator checks
restrictions (verify by searching for `GameRestriction` in `legal_actions.rs`).

## Card Definition Fixes

### borne_upon_a_wind.rs
**Oracle text**: You may cast spells this turn as though they had flash. Draw a card.
**Current state**: TODO at lines 5, 16 — only draws a card, flash grant missing.
**Fix**: Replace the Spell effect with a `Sequence` of `GrantFlash` + `DrawCards`:
```rust
AbilityDefinition::Spell {
    effect: Effect::Sequence(vec![
        Effect::GrantFlash {
            filter: FlashGrantFilter::AllSpells,
            duration: EffectDuration::UntilEndOfTurn,
        },
        Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
    ]),
    targets: vec![],
    modes: None,
    cant_be_countered: false,
}
```
Remove both TODO comments.

### complete_the_circuit.rs
**Oracle text**: Convoke. You may cast sorcery spells this turn as though they had flash. When you next cast an instant or sorcery spell this turn, copy that spell twice. You may choose new targets for the copies.
**Current state**: TODO at lines 7-9, 24-26 — both flash grant and copy-spell-twice missing. Convoke keyword present.
**Fix**: Add `GrantFlash` effect for sorceries. The copy-spell-twice delayed trigger remains a TODO (separate primitive — `WhenNextCastSpell` delayed trigger + `CopySpellTwice` effect).
```rust
abilities: vec![
    AbilityDefinition::Keyword(KeywordAbility::Convoke),
    AbilityDefinition::Spell {
        effect: Effect::GrantFlash {
            filter: FlashGrantFilter::Sorceries,
            duration: EffectDuration::UntilEndOfTurn,
        },
        // TODO: "When you next cast an instant or sorcery spell this turn, copy that spell
        //   twice" — delayed trigger (WhenNextCastSpell) + copy-on-stack not in DSL.
        targets: vec![],
        modes: None,
        cant_be_countered: false,
    },
],
```
Remove the two flash-related TODOs; keep the copy-spell-twice TODO.

### teferi_time_raveler.rs
**Oracle text**: Each opponent can cast spells only any time they could cast a sorcery. +1: Until your next turn, you may cast sorcery spells as though they had flash. -3: Return up to one target artifact, creature, or enchantment to its owner's hand. Draw a card.
**Current state**: TODO at lines 21-22 — passive stax restriction and +1 flash grant both missing. -3 bounce+draw already implemented.
**Fix**:
1. Add `StaticRestriction { restriction: GameRestriction::OpponentsCanOnlyCastAtSorcerySpeed }` for the passive.
2. Replace +1 `Effect::Nothing` with `Effect::GrantFlash { filter: Sorceries, duration: UntilYourNextTurn(PlayerId(0)) }`.

```rust
abilities: vec![
    // Passive: "Each opponent can cast spells only any time they could cast a sorcery."
    AbilityDefinition::StaticRestriction {
        restriction: GameRestriction::OpponentsCanOnlyCastAtSorcerySpeed,
    },
    // +1: "Until your next turn, you may cast sorcery spells as though they had flash."
    AbilityDefinition::LoyaltyAbility {
        cost: LoyaltyCost::Plus(1),
        effect: Effect::GrantFlash {
            filter: FlashGrantFilter::Sorceries,
            duration: EffectDuration::UntilYourNextTurn(PlayerId(0)),
        },
        targets: vec![],
    },
    // -3: existing bounce+draw (unchanged)
    ...
],
```
Remove both TODO comments. Add `FlashGrantFilter`, `GameRestriction`, `PlayerId` to imports if needed.

## New Card Definitions

### yeva_natures_herald.rs
**Oracle text**: Flash. You may cast green creature spells as though they had flash. 4/4 Legendary Creature -- Elf Shaman. {2}{G}{G}.
**CardDefinition sketch**:
```rust
CardDefinition {
    card_id: cid("yeva-natures-herald"),
    name: "Yeva, Nature's Herald".to_string(),
    mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
    types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elf", "Shaman"]),
    oracle_text: "Flash\nYou may cast green creature spells as though they had flash.".to_string(),
    power: Some(4),
    toughness: Some(4),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Flash),
        AbilityDefinition::StaticFlashGrant {
            filter: FlashGrantFilter::GreenCreatures,
        },
    ],
    ..Default::default()
}
```

Register in card registry (`crates/engine/src/cards/defs/mod.rs`).

## Unit Tests

**File**: `crates/engine/tests/grant_flash.rs`
**Tests to write**:

- `test_grant_flash_borne_upon_a_wind_basic` — Cast Borne Upon a Wind, then cast a sorcery at instant speed (while stack is non-empty or during opponent's turn). CR 601.3b.
- `test_grant_flash_borne_upon_a_wind_all_spells` — Verify creatures, sorceries, and other spell types can all be cast at instant speed after Borne Upon a Wind resolves.
- `test_grant_flash_sorceries_only` — Cast Complete the Circuit, verify only sorceries get flash (not creatures). CR 601.3b filter validation.
- `test_grant_flash_expires_end_of_turn` — Grant expires at cleanup step; next turn the player cannot cast sorceries at instant speed.
- `test_grant_flash_until_your_next_turn` — Teferi +1 grants flash for sorceries until controller's next turn. Verify it persists through opponents' turns and expires at controller's next untap.
- `test_teferi_passive_opponents_sorcery_speed` — Teferi's passive restricts opponents to sorcery speed. Opponent cannot cast instants during the Teferi controller's turn unless it's their main phase with empty stack. CR 307.5.
- `test_teferi_restriction_overrides_grant` — CR 101.2: If opponent has a flash grant AND Teferi's restriction applies, the restriction wins. Opponent still can't cast at instant speed.
- `test_yeva_static_flash_grant_green_creatures` — Yeva on battlefield: controller can cast green creatures at instant speed. Non-green creatures and non-creatures cannot.
- `test_yeva_leaves_battlefield_grant_removed` — When Yeva leaves the battlefield, the flash grant is no longer active.
- `test_grant_flash_multiplayer` — In a 4-player game, flash grant applies only to the controller, not other players. Teferi restriction applies to all opponents.

**Pattern**: Follow tests in `crates/engine/tests/casting_restrictions.rs` or `crates/engine/tests/damage_replacement.rs`.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (3 cards)
- [ ] New card defs authored (Yeva, Nature's Herald)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining flash-related TODOs in affected card defs

## Risks & Edge Cases

- **UntilEndOfTurn cleanup location**: Confirmed — `expire_end_of_turn_effects` in `layers.rs` (line 1247), called from `turn_actions.rs` line 1411 during the cleanup step. Flash grant cleanup MUST go here, NOT in `reset_turn_state`.
- **CR 101.2 restriction vs permission**: Teferi's sorcery-speed restriction on opponents MUST be checked BEFORE flash grants are checked. The `check_cast_restrictions` function runs at line 213 (before timing check at line 3015), so this ordering is already correct — restrictions error out before flash grants are even consulted.
- **FlashGrantFilter::GreenCreatures color check**: Must use the card's colors from characteristics, not color identity. A colorless creature with Devoid that was originally green should NOT match (Devoid removes colors in all zones via CDA). This is correct if we check `chars.colors` which is layer-resolved.
- **Adventure spells**: When casting an Adventure half, the characteristics used for timing are the Adventure half's. Flash grants should apply based on those characteristics. The existing `chars` variable at the timing check point should already reflect this.
- **Stale flash grant cleanup**: `WhileSourceOnBattlefield` grants need cleanup when the source leaves. The inline source-validity check in `has_active_flash_grant` prevents stale grants from having any effect at query time. Periodic cleanup in `reset_turn_state` keeps the Vector tidy.
- **Teferi passive + Teferi +1 interaction**: Teferi's passive restricts opponents. Teferi's +1 grants the controller flash for sorceries. These don't conflict because the passive only affects opponents and the +1 only affects the controller.
- **Copy-spell-twice on Complete the Circuit**: Left as TODO. This is a separate primitive (delayed trigger + spell copying). The card def will be partially complete — flash grant works, copy does not.
- **Teferi's -3 target requirement**: Currently uses `TargetRequirement::TargetPermanent` which is very broad. The oracle text says "up to one target artifact, creature, or enchantment." The "up to one" is not yet expressible (optional targeting), and the type filter should be restricted. These are pre-existing issues not introduced by PB-I.
