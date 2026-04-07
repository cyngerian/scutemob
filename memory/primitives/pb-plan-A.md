# Primitive Batch Plan: PB-A -- Play from Top of Library

**Generated**: 2026-04-07
**Primitive**: Continuous cast/play-from-top-of-library permission system
**CR Rules**: 601.1a, 601.2, 601.3, 305.1, 305.2a
**Cards affected**: 10 (4 existing fixes + 6 new)
**Dependencies**: PB-I (FlashGrant pattern), PB-K (Case mechanic, AdditionalLandPlays)
**Deferred items from prior PBs**: Case of the Locked Hothouse solved ability (from PB-K)

## Primitive Specification

This batch adds a continuous permission system allowing players to play lands and/or cast
spells from the top of their library. The system follows the same architectural pattern as
`FlashGrant` (PB-I) and `ActiveRestriction` (PB-18):

1. A `PlayFromTopPermission` struct on `GameState` tracks active top-of-library permissions
2. An `AbilityDefinition::StaticPlayFromTop` variant registers permissions when a permanent
   enters the battlefield
3. `casting.rs` checks for an active permission when the card is on top of the player's
   library (new zone check: `ZoneId::Library(player)` at position 0)
4. `lands.rs` checks for an active permission when the card is the top of the library
5. A `PlayFromTopFilter` enum specifies which cards can be played/cast (lands only,
   creatures only, artifacts/colorless, all spells, etc.)
6. For Bolas's Citadel: a new `AltCostKind::PayLifeForManaValue` alternative cost that
   replaces the mana cost with life payment equal to the spell's mana value

Additionally, two informational statics are needed:
- "You may look at the top card of your library any time" -- modeled as a boolean
  permission (not a game-mechanical effect; the engine already knows all cards, but
  this is needed for hidden-info filtering in the network layer)
- "Play with the top card revealed" -- a stronger version where ALL players can see it

## CR Rule Text

**CR 601.1a**: Some effects still refer to "playing" a card. "Playing a card" means playing
that card as a land or casting that card as a spell, whichever is appropriate.

**CR 601.2**: To cast a spell is to take it from where it is (usually the hand), put it on
the stack, and pay its costs, so that it will eventually resolve and have its effect.

**CR 601.3**: A player can begin to cast a spell only if a rule or effect allows that player
to cast it and no rule or effect prohibits that player from casting it.

**CR 305.1**: A player who has priority may play a land card from their hand during a main
phase of their turn when the stack is empty. Playing a land is a special action; it doesn't
use the stack.

**CR 305.2a**: To determine whether a player can play a land, compare the number of lands
the player can play this turn with the number of lands they have already played this turn.

Key rulings from card lookups:
- Normal timing permissions and restrictions still apply (Future Sight 2019-06-14)
- Land play from top counts against your land-play limit (Courser of Kruphix 2021-03-19)
- Cannot suspend, cycle, or activate abilities of the top card (Future Sight 2019-06-14)
- If casting the top card, the new top card is not revealed until you finish (all cards)
- For Bolas's Citadel: paying life is an alternative cost; no alternative costs can combine
  with it; X must be 0; additional costs still apply (Bolas's Citadel 2019-05-03)
- For Mystic Forge: "cast" does not include "play a land" -- artifact lands cannot be
  played from top (Mystic Forge 2019-07-12)

## Engine Changes

### Change 1: PlayFromTopFilter enum (new type in stubs.rs)

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add `PlayFromTopFilter` enum and `PlayFromTopPermission` struct after
`FlashGrant` (line ~619).
**Pattern**: Follow `FlashGrantFilter` / `FlashGrant` pattern.

```
/// Filter for which cards a play-from-top permission applies to.
/// CR 601.1a: "Playing a card" = playing as land OR casting as spell.
pub enum PlayFromTopFilter {
    /// All cards (Future Sight: "play lands and cast spells")
    All,
    /// Lands only (Courser of Kruphix: "play lands")
    LandsOnly,
    /// Creature spells only (Elven Chorus: "cast creature spells")
    CreaturesOnly,
    /// Creature spells with power >= N (Thundermane Dragon: "creature spells with power 4 or greater")
    CreaturesWithMinPower(u32),
    /// Artifact spells and colorless spells (Mystic Forge: "artifact spells and colorless spells")
    ArtifactsAndColorless,
    /// Creature and enchantment spells + lands (Case of the Locked Hothouse solved)
    CreaturesAndEnchantmentsAndLands,
    /// Nonland cards (Bolas's Citadel: "play lands and cast spells" but with life payment)
    /// Note: Bolas's Citadel actually says "play lands and cast spells" -- it's All.
}

pub struct PlayFromTopPermission {
    pub source: ObjectId,
    pub controller: PlayerId,
    pub filter: PlayFromTopFilter,
    /// If true, player may look at top card any time (hidden info permission).
    pub look_at_top: bool,
    /// If true, top card is revealed to all players (stronger than look_at_top).
    pub reveal_top: bool,
    /// Alternative cost: if Some, pay life equal to mana value instead of mana cost.
    /// CR 118.9: This is an alternative cost (Bolas's Citadel).
    pub pay_life_instead: bool,
    /// Optional: condition that must be true for this permission to be active.
    /// Used for Case of the Locked Hothouse (SourceIsSolved).
    pub condition: Option<Condition>,
    /// Optional: bonus effect when a spell is cast from top this way.
    /// Used for Thundermane Dragon (grant haste until end of turn).
    pub on_cast_effect: Option<Box<Effect>>,
}
```

### Change 2: GameState field (play_from_top_permissions)

**File**: `crates/engine/src/state/mod.rs`
**Action**: Add `play_from_top_permissions: Vector<PlayFromTopPermission>` field after
`flash_grants` (line ~131). Add `#[serde(default)]`.

### Change 3: GameStateBuilder initialization

**File**: `crates/engine/src/state/builder.rs`
**Action**: Add `play_from_top_permissions: Vector::new()` after `flash_grants` (line ~321).

### Change 4: AbilityDefinition::StaticPlayFromTop variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant after `StaticFlashGrant` (discriminant 73):

```
/// Static play-from-top-of-library permission (CR 601.3).
///
/// Registers a PlayFromTopPermission when the permanent enters. Cleaned up
/// when the source leaves the battlefield. Follows FlashGrant pattern.
///
/// Discriminant 73.
StaticPlayFromTop {
    filter: PlayFromTopFilter,
    look_at_top: bool,
    reveal_top: bool,
    pay_life_instead: bool,
    condition: Option<Condition>,
    on_cast_effect: Option<Box<Effect>>,
},
```

### Change 5: Registration in replacement.rs

**File**: `crates/engine/src/rules/replacement.rs`
**Action**: Add match arm for `AbilityDefinition::StaticPlayFromTop` in
`register_static_continuous_effects`, after the `StaticFlashGrant` arm (line ~1824).
Push a `PlayFromTopPermission` onto `state.play_from_top_permissions`.

### Change 6: Casting validation in casting.rs -- allow top-of-library as cast zone

**File**: `crates/engine/src/rules/casting.rs`
**Action**: In the zone-validation block (~line 456-476), add a new check for
`ZoneId::Library(player)` when the card is at position 0 (top of library) and an active
play-from-top permission exists that matches the spell being cast.

Specifically:
1. Add `casting_from_top_of_library: bool` to the tuple returned from the card validation block
2. Check: card is in `ZoneId::Library(player)`, card is the FIRST object in that zone
   (top of library), and `has_play_from_top_permission(state, player, &chars)` returns true
3. Add `!casting_from_top_of_library` to the zone-rejection guard at line 472
4. Add a new helper function `has_play_from_top_permission()` following the
   `has_active_flash_grant()` pattern (~line 5429)

The helper function iterates `state.play_from_top_permissions` and checks:
1. Source still on battlefield
2. Controller matches the player
3. Filter matches the card's characteristics (type check, power check, etc.)
4. Condition evaluates to true (if present -- for Case solved)
5. Card is NOT a land (casting only applies to non-land cards; lands use PlayLand)

### Change 7: AltCostKind::PayLifeForManaValue for Bolas's Citadel

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `PayLifeForManaValue` variant to `AltCostKind` after `CommanderFreeCast`.

**File**: `crates/engine/src/rules/casting.rs`
**Action**: In the alt-cost selection chain (~line 2193), add:
```
} else if cast_with_pay_life {
    // Bolas's Citadel: Pay life equal to mana value instead of mana cost.
    // CR 118.9: Alternative cost. Mana cost becomes 0; life payment handled separately.
    // CR 107.3: X is 0 when casting without paying mana cost (2019-05-03 ruling).
    Some(ManaCost::default())
}
```

Then, AFTER the cost is locked in but BEFORE payment, add a life-loss step:
When `cast_with_pay_life`, the caster loses life equal to the original card's mana value.
This happens in the cost-payment section (~line 3342+). The mana cost is {0} (free), but
life equal to the original `base_mana_cost.mana_value()` is deducted from the player.

### Change 8: Land play from top of library in lands.rs

**File**: `crates/engine/src/rules/lands.rs`
**Action**: In `handle_play_land`, modify the zone check (~line 58) to also accept
`ZoneId::Library(player)` when the card is the top card AND an active play-from-top
permission exists with a filter that includes lands.

Add a helper `has_play_from_top_land_permission()` that checks:
1. Source still on battlefield
2. Controller matches
3. Filter includes lands (All, LandsOnly, CreaturesAndEnchantmentsAndLands)
4. Condition evaluates to true (if present)
5. Card is the top card of the library (index 0)

### Change 9: Cleanup sweep in turn_actions.rs

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: In `reset_turn_state` (~line 1526), add cleanup sweep for
`play_from_top_permissions` following the flash_grants pattern. Remove entries whose
source has left the battlefield.

### Change 10: Duration expiry in layers.rs

**File**: `crates/engine/src/rules/layers.rs`
**Action**: Play-from-top permissions use `WhileSourceOnBattlefield` duration (static
abilities), so no end-of-turn expiry is needed. But the cleanup in turn_actions.rs
handles stale sources.

### Change 11: Exhaustive match updates

Files requiring new match arms / hash implementations:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `state/hash.rs` | `HashInto for AbilityDefinition` | ~L5698 | Add hash arm for `StaticPlayFromTop` (disc 73) |
| `state/hash.rs` | `HashInto for PlayFromTopFilter` | NEW | Add `HashInto` impl for new enum |
| `state/hash.rs` | `HashInto for PlayFromTopPermission` | NEW | Add `HashInto` impl for new struct |
| `state/hash.rs` | `HashInto for AltCostKind` | find | Add arm for `PayLifeForManaValue` |
| `state/hash.rs` | GameState `public_state_hash` | ~L5830 | Hash `play_from_top_permissions` |
| `rules/replacement.rs` | `register_static_continuous_effects` | ~L1824 | Add `StaticPlayFromTop` arm |
| `rules/casting.rs` | alt_cost boolean extraction | ~L164 | Add `cast_with_pay_life` |
| `rules/casting.rs` | zone validation block | ~L456-476 | Add `casting_from_top_of_library` |
| `rules/casting.rs` | alt cost chain | ~L2193 | Add `PayLifeForManaValue` branch |
| `rules/casting.rs` | cost payment | ~L3342 | Add life-loss for PayLifeForManaValue |
| `rules/turn_actions.rs` | `reset_turn_state` cleanup | ~L1536 | Add `play_from_top_permissions` retain |
| `cards/helpers.rs` | pub use exports | L27 | Add `PlayFromTopFilter` to imports |

### Change 12: Thundermane Dragon on-cast haste grant

For Thundermane Dragon's "If you cast a creature spell this way, it gains haste until
end of turn", the `on_cast_effect` field on `PlayFromTopPermission` handles this. When
a spell is successfully cast from the top of library via this permission, the engine
should apply `Effect::ApplyContinuousEffect` granting haste until end of turn to the
resulting permanent.

Implementation: In casting.rs, after the spell is successfully put on the stack from
top-of-library, check if the matching permission has `on_cast_effect`. If so, record
it on the StackObject (e.g., a new `top_cast_bonus_effect: Option<Box<Effect>>` field)
so that resolution.rs can apply it when the spell resolves.

Alternatively (simpler): store `was_cast_from_top_with_bonus: Option<ObjectId>` on
the StackObject/GameObject, and have the source card's triggered ability check this.
The simplest approach: model Thundermane Dragon's haste grant as a triggered ability
`WheneverYouCastCreatureSpellFromTop` that checks a flag. But this adds a new
TriggerCondition.

**Recommended approach**: Use `on_cast_effect` on the permission struct. When
`casting.rs` matches a permission with `on_cast_effect.is_some()`, register a
delayed continuous effect granting haste to the spell (by StackObject source) until
end of turn.

## Card Definition Fixes

### courser_of_kruphix.rs
**Oracle text**: "Play with the top card of your library revealed. You may play lands from the top of your library. Landfall -- Whenever a land you control enters, you gain 1 life."
**Current state**: Two TODOs -- missing revealed-top-of-library and play-lands-from-top
**Fix**: Add `AbilityDefinition::StaticPlayFromTop { filter: PlayFromTopFilter::LandsOnly, look_at_top: false, reveal_top: true, pay_life_instead: false, condition: None, on_cast_effect: None }`. The `reveal_top: true` covers "Play with the top card revealed." The Landfall trigger is already implemented.

### elven_chorus.rs
**Oracle text**: "You may look at the top card of your library any time. You may cast creature spells from the top of your library. Creatures you control have '{T}: Add one mana of any color.'"
**Current state**: All three abilities are TODOs
**Fix**: Add `AbilityDefinition::StaticPlayFromTop { filter: PlayFromTopFilter::CreaturesOnly, look_at_top: true, reveal_top: false, pay_life_instead: false, condition: None, on_cast_effect: None }`. The mana grant ability ("creatures you control have {T}: Add any color") is a SEPARATE DSL gap (GrantActivatedAbility / static mana ability grant) -- leave as TODO with note that play-from-top is resolved.

### thundermane_dragon.rs
**Oracle text**: "Flying. You may look at the top card of your library any time. You may cast creature spells with power 4 or greater from the top of your library. If you cast a creature spell this way, it gains haste until end of turn."
**Current state**: Two TODOs
**Fix**: Add `AbilityDefinition::StaticPlayFromTop { filter: PlayFromTopFilter::CreaturesWithMinPower(4), look_at_top: true, reveal_top: false, pay_life_instead: false, condition: None, on_cast_effect: Some(Box::new(Effect::ApplyContinuousEffect { ... haste until end of turn ... })) }`.

### case_of_the_locked_hothouse.rs
**Oracle text**: "You may play an additional land on each of your turns. To solve -- You control seven or more lands. Solved -- You may look at the top card of your library any time, and you may play lands and cast creature and enchantment spells from the top of your library."
**Current state**: TODO for solved play-from-top ability (deferred from PB-K)
**Fix**: Add `AbilityDefinition::StaticPlayFromTop { filter: PlayFromTopFilter::CreaturesAndEnchantmentsAndLands, look_at_top: true, reveal_top: false, pay_life_instead: false, condition: Some(Condition::SourceIsSolved), on_cast_effect: None }`. The condition ensures this only activates when solved.

## New Card Definitions

### future_sight
**Oracle text**: "Play with the top card of your library revealed. You may play lands and cast spells from the top of your library."
**CardDefinition sketch**:
- card_id: "future-sight", name: "Future Sight"
- mana_cost: {2}{U}{U}{U}
- types: Enchantment
- abilities: `[StaticPlayFromTop { filter: All, look_at_top: false, reveal_top: true, pay_life_instead: false, condition: None, on_cast_effect: None }]`

### bolas_s_citadel
**Oracle text**: "You may look at the top card of your library any time. You may play lands and cast spells from the top of your library. If you cast a spell this way, pay life equal to its mana value rather than pay its mana cost. {T}, Sacrifice ten nonland permanents: Each opponent loses 10 life."
**CardDefinition sketch**:
- card_id: "bolass-citadel", name: "Bolas's Citadel"
- mana_cost: {3}{B}{B}{B}
- types: Legendary Artifact
- abilities:
  1. `StaticPlayFromTop { filter: All, look_at_top: true, reveal_top: false, pay_life_instead: true, condition: None, on_cast_effect: None }`
  2. Activated ability: {T}, Sacrifice ten nonland permanents: Each opponent loses 10 life.
     (The sacrifice-10-nonland-permanents cost is a DSL gap -- use TODO for the activated ability, note that play-from-top is resolved.)

### mystic_forge
**Oracle text**: "You may look at the top card of your library any time. You may cast artifact spells and colorless spells from the top of your library. {T}, Pay 1 life: Exile the top card of your library."
**CardDefinition sketch**:
- card_id: "mystic-forge", name: "Mystic Forge"
- mana_cost: {4}
- types: Artifact
- abilities:
  1. `StaticPlayFromTop { filter: ArtifactsAndColorless, look_at_top: true, reveal_top: false, pay_life_instead: false, condition: None, on_cast_effect: None }`
  2. Activated: {T}, Pay 1 life: Exile top card. (Note: Mystic Forge explicitly says "cast" not "play" -- artifact lands CANNOT be played from top per 2019-07-12 ruling.)

### oracle_of_mul_daya
**Oracle text**: "You may play an additional land on each of your turns. Play with the top card of your library revealed. You may play lands from the top of your library."
**CardDefinition sketch**:
- card_id: "oracle-of-mul-daya", name: "Oracle of Mul Daya"
- mana_cost: {3}{G}
- types: Creature -- Elf Shaman, P/T 2/2
- abilities:
  1. `AdditionalLandPlays { count: 1 }`
  2. `StaticPlayFromTop { filter: LandsOnly, look_at_top: false, reveal_top: true, pay_life_instead: false, condition: None, on_cast_effect: None }`

### vizier_of_the_menagerie
**Oracle text**: "You may look at the top card of your library any time. You may cast creature spells from the top of your library. You can spend mana of any type to cast creature spells."
**CardDefinition sketch**:
- card_id: "vizier-of-the-menagerie", name: "Vizier of the Menagerie"
- mana_cost: {3}{G}
- types: Creature -- Snake Cleric, P/T 3/4
- abilities:
  1. `StaticPlayFromTop { filter: CreaturesOnly, look_at_top: true, reveal_top: false, pay_life_instead: false, condition: None, on_cast_effect: None }`
  2. "Spend mana as though it were mana of any color to cast creature spells" -- separate DSL gap (mana spending restriction), leave as TODO.

### radha_heart_of_keld
**Oracle text**: "During your turn, Radha has first strike. You may look at the top card of your library any time, and you may play lands from the top of your library. {4}{R}{G}: Radha gets +X/+X until end of turn, where X is the number of lands you control."
**CardDefinition sketch**:
- card_id: "radha-heart-of-keld", name: "Radha, Heart of Keld"
- mana_cost: {1}{R}{G}
- types: Legendary Creature -- Elf Warrior, P/T 3/3
- abilities:
  1. Conditional first strike (IsYourTurn)
  2. `StaticPlayFromTop { filter: LandsOnly, look_at_top: true, reveal_top: false, pay_life_instead: false, condition: None, on_cast_effect: None }`
  3. Activated: {4}{R}{G}: +X/+X where X = lands you control

## Unit Tests

**File**: `crates/engine/tests/play_from_top.rs`
**Tests to write**:
- `test_play_from_top_basic_land` -- Courser: play a land from top of library, verify land enters battlefield, land play count decremented. CR 305.1/305.2a.
- `test_play_from_top_land_uses_land_play` -- Play land from top counts as land play; cannot play another if at limit. CR 305.2a (Courser 2021-03-19 ruling).
- `test_play_from_top_cast_creature` -- Elven Chorus: cast creature from top, verify goes to stack, mana paid. CR 601.2.
- `test_play_from_top_cast_respects_timing` -- Cannot cast sorcery-speed spell from top during opponent's turn. CR 601.3 (Future Sight 2019-06-14 ruling).
- `test_play_from_top_filter_rejects_wrong_type` -- Mystic Forge: cannot cast non-artifact non-colorless spell from top. CR 601.3.
- `test_play_from_top_mystic_forge_no_lands` -- Mystic Forge: cannot play artifact lands from top (2019-07-12 ruling).
- `test_play_from_top_bolas_citadel_pay_life` -- Bolas's Citadel: cast spell from top, life deducted equal to mana value, no mana spent. CR 118.9.
- `test_play_from_top_bolas_citadel_x_is_zero` -- X spells cast via Bolas's Citadel: X must be 0 (2019-05-03 ruling).
- `test_play_from_top_all_types_future_sight` -- Future Sight: can cast any spell type from top. CR 601.1a.
- `test_play_from_top_power_filter` -- Thundermane Dragon: can cast creature with power 4+, cannot cast creature with power 3.
- `test_play_from_top_haste_grant` -- Thundermane Dragon: creature cast from top gains haste until end of turn.
- `test_play_from_top_case_solved_condition` -- Case of the Locked Hothouse: permission inactive when unsolved, active when solved.
- `test_play_from_top_source_leaves` -- When the source permanent leaves battlefield, permission is removed. Cards on top can no longer be played from there.
- `test_play_from_top_multiple_permissions` -- Two play-from-top sources; removing one doesn't affect the other.
- `test_play_from_top_not_from_second_card` -- Cannot cast the second card from top, only the first (top) card.

**Pattern**: Follow tests for FlashGrant in `tests/flash_grant.rs` (if exists) or `tests/casting_restrictions.rs`.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved
- [ ] New card defs authored (6 new: Future Sight, Bolas's Citadel, Mystic Forge, Oracle of Mul Daya, Vizier of the Menagerie, Radha Heart of Keld)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs (except noted DSL gaps: GrantActivatedAbility for Elven Chorus mana, spend-mana-as-any for Vizier, sacrifice-10 for Bolas's Citadel)

## Risks & Edge Cases

- **Top card identity during casting**: When you begin casting the top card, the new top card should not be visible until casting is complete (all card rulings confirm this). In the engine's deterministic model, this is handled automatically -- the card moves to the stack as part of casting, so the new top card is a different ObjectId. No special handling needed.

- **Bolas's Citadel X=0 enforcement**: When `pay_life_instead` is true and the spell has X in its cost, X must be 0 (2019-05-03 ruling on Bolas's Citadel: "you must choose 0 as the value of X when casting it without paying its mana cost"). Enforce this in casting.rs alongside the existing X-value validation.

- **Multiple play-from-top sources**: A player could have both Future Sight and Mystic Forge. The engine should check if ANY active permission matches -- same as multiple flash grants.

- **Interaction with Morph**: Mystic Forge ruling (2019-07-12): "If the top card has a morph ability, you can cast it face down from the top of your library, even if it's normally not a colorless card." The morph face-down spell IS a colorless spell, so ArtifactsAndColorless filter matches. No special handling needed -- morph already overrides characteristics.

- **Library order**: The engine uses `ZoneId::Library(player)` and objects have implicit ordering. Need to verify that "top of library" corresponds to the first element in the zone's object list. Check `move_object_to_zone` and `draw_card` to confirm index 0 = top.

- **Bolas's Citadel life payment timing**: Life is paid as part of the alternative cost (CR 118.9). This happens during step 601.2h (pay costs). If the player doesn't have enough life, the cast is illegal. Check: player life > mana value (note: life can go to 0 or below from this).

- **PlayFromTopPermission condition evaluation**: For Case of the Locked Hothouse, the condition (`SourceIsSolved`) must be checked at the time the player attempts to cast/play, not at registration time. The `has_play_from_top_permission()` helper must evaluate the condition dynamically using the existing `evaluate_condition()` function from effects/mod.rs.

- **Revealed top card info**: "Play with top card revealed" (Courser, Future Sight) means all players see it. "Look at top card any time" (Mystic Forge, Elven Chorus) means only the controller sees it. The `reveal_top` vs `look_at_top` distinction matters for the network layer's hidden-info filtering. For the engine itself, this is purely informational metadata -- but the fields must exist for correctness.

- **Bolas's Citadel activated ability**: "{T}, Sacrifice ten nonland permanents: Each opponent loses 10 life" requires a new Cost variant for "sacrifice N permanents with filter". This is a separate DSL gap. The play-from-top ability is the primary focus; the activated ability can remain as TODO.
