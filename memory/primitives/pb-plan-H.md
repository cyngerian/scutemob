# Primitive Batch Plan: PB-H -- Mass Reanimate

**Generated**: 2026-04-06
**Primitive**: Mass zone-change effects moving cards from graveyards to the battlefield
**CR Rules**: 400.7, 101.4, 603.6a, 701.17a
**Cards affected**: 5 (4 existing fixes + 1 new)
**Dependencies**: None (all prerequisite infrastructure exists)
**Deferred items from prior PBs**: None applicable to this batch

## Primitive Specification

The engine currently has `DestroyAll`, `ExileAll`, and `BounceAll` for mass
battlefield-to-other-zone effects, and `MoveZone` for single-target zone changes.
It lacks a mass graveyard-to-battlefield effect. This batch adds two new Effect
variants:

1. **`Effect::ReturnAllFromGraveyardToBattlefield`** -- Finds all cards in one or
   more graveyards matching a filter, then puts them all onto the battlefield
   simultaneously. Handles "under their owners' control" vs "under your control"
   via a controller override parameter. Handles "enters tapped" via a tapped flag.
   This covers Splendid Reclamation, Open the Vaults, World Shaper, and Eerie
   Ultimatum.

2. **`Effect::LivingDeath`** -- A specialized three-step mass zone-change that
   implements Living Death's unique "exile from GY, sacrifice all, return exiled"
   sequence. Too unique to generalize; a dedicated variant is cleaner than trying
   to compose from existing primitives.

Both effects must replicate the full ETB chain for each permanent put onto the
battlefield: move_object_to_zone, apply tapped status, apply_self_etb_from_definition,
apply_etb_replacements, register_permanent_replacement_abilities,
register_static_continuous_effects, emit PermanentEnteredBattlefield, and
queue_carddef_etb_triggers.

## CR Rule Text

### CR 400.7 (Object Identity)
> An object that moves from one zone to another becomes a new object with no memory
> of, or relation to, its previous existence.

### CR 101.4 (APNAP Simultaneous Actions)
> If multiple players would make choices and/or take actions at the same time, the
> active player makes any choices required, then the next player in turn order makes
> any choices required, followed by the remaining nonactive players in turn order.
> Then the actions happen simultaneously.

### CR 603.6a (ETB Triggers for Simultaneous Entries)
> Each time an event puts one or more permanents onto the battlefield, all permanents
> on the battlefield (including the newcomers) are checked for any enters-the-
> battlefield triggers that match the event.

### CR 701.17a (Sacrifice)
> To sacrifice a permanent, its controller moves it from the battlefield to its
> owner's graveyard. A player can't sacrifice an object that isn't a permanent,
> and a player can't sacrifice a permanent they don't control. Sacrificing a
> permanent doesn't destroy it, so regeneration or other effects that replace
> destruction can't affect this action.

### Living Death Ruling (2018-03-16)
> As Living Death resolves, all players exile their creature cards from graveyards
> at the same time. Then all players sacrifice all creatures they control at the
> same time. Then all players put all creatures they exiled onto the battlefield
> at the same time.

> Only cards exiled by Living Death's first instruction are put onto the
> battlefield. If a replacement effect (such as that of Leyline of the Void)
> causes any of the sacrificed creatures to be exiled instead of put into a
> graveyard, those cards aren't returned to the battlefield.

## Engine Changes

### Change 1: Add `Effect::ReturnAllFromGraveyardToBattlefield` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant to the `Effect` enum after `GrantFlash` (line ~1929).
**Pattern**: Follows `DestroyAll`/`ExileAll`/`BounceAll` pattern of filter + parameters.

```rust
/// CR 400.7, 603.6a: Return all cards matching the filter from one or more
/// graveyards to the battlefield simultaneously.
/// All permanents enter at the same time; ETB triggers are queued for all of them
/// before any resolve.
ReturnAllFromGraveyardToBattlefield {
    /// Which graveyards to search. `PlayerTarget::Controller` = your GY only.
    /// `PlayerTarget::AllPlayers` = all graveyards.
    graveyards: PlayerTarget,
    /// Filter applied to cards in the graveyard (has_card_type, has_card_types, etc.).
    filter: TargetFilter,
    /// If true, permanents enter tapped (Splendid Reclamation, World Shaper).
    tapped: bool,
    /// Controller of the entering permanents.
    /// `None` = owner (default for "under their owners' control").
    /// `Some(PlayerTarget::Controller)` = "under your control".
    #[serde(default)]
    controller_override: Option<PlayerTarget>,
    /// If true, only return one card per unique name (Eerie Ultimatum).
    /// Deterministic: for duplicate names, lowest ObjectId wins.
    #[serde(default)]
    unique_names: bool,
    /// If true, only return permanent cards (non-instant, non-sorcery).
    /// Used by Eerie Ultimatum. Checked in addition to `filter`.
    #[serde(default)]
    permanent_cards_only: bool,
},
```

### Change 2: Add `Effect::LivingDeath` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant to the `Effect` enum after `ReturnAllFromGraveyardToBattlefield`.

```rust
/// Living Death (no CR — unique card text):
/// Step 1: Each player exiles all creature cards from their graveyard.
/// Step 2: Each player sacrifices all creatures they control.
/// Step 3: Each player puts all cards they exiled in step 1 onto the battlefield.
/// All steps happen simultaneously per-player. Cards exiled by replacement effects
/// during step 2 are NOT returned in step 3 (only step-1 exiled cards).
LivingDeath,
```

### Change 3: Dispatch `ReturnAllFromGraveyardToBattlefield` in effects/mod.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm after `Effect::GrantFlash` (line ~4621).
**CR**: 400.7 (new object identity), 603.6a (simultaneous ETB triggers)

Implementation steps:
1. Determine which graveyards to search based on `graveyards` PlayerTarget.
2. Collect all cards in those graveyards matching the filter (using layer-resolved
   characteristics where available, falling back to base characteristics for
   non-battlefield objects).
3. If `unique_names`, deduplicate by name (lowest ObjectId wins).
4. If `permanent_cards_only`, exclude instants and sorceries.
5. Sort by ObjectId for deterministic ordering.
6. For each card: `move_object_to_zone(id, ZoneId::Battlefield)`.
7. For each new permanent: apply tapped status if `tapped == true`.
8. For each new permanent: resolve `controller_override` if specified.
9. For each new permanent: run the full ETB chain:
   - `apply_self_etb_from_definition`
   - `apply_etb_replacements`
   - `register_permanent_replacement_abilities`
   - `register_static_continuous_effects`
   - Emit `PermanentEnteredBattlefield`
   - `queue_carddef_etb_triggers`

**Pattern**: Follow `Effect::PutLandFromHandOntoBattlefield` (line ~4444) for the
per-permanent ETB chain. The mass collection pattern follows `Effect::DestroyAll`
(line ~839) for snapshot-then-iterate.

### Change 4: Dispatch `Effect::LivingDeath` in effects/mod.rs

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm after `ReturnAllFromGraveyardToBattlefield`.
**CR**: 101.4 (APNAP simultaneous), 701.17a (sacrifice)

Implementation steps:
1. **Step 1 -- Exile creature cards from all graveyards:**
   For each player (APNAP order): collect all creature cards in their graveyard,
   move each to exile, track the set of exiled ObjectIds per player in a local
   `Vec<(PlayerId, Vec<ObjectId>)>`. Emit `ObjectExiled` for each.

2. **Step 2 -- Sacrifice all creatures:**
   Snapshot all creatures on the battlefield. For each (sorted by ObjectId for
   determinism): move to owner's graveyard. Emit `CreatureDied` for each.
   NOTE: sacrificed creatures that get redirected to exile by replacement effects
   (Leyline of the Void) should NOT be added to the step-1 exiled set.

3. **Step 3 -- Return exiled cards to battlefield:**
   For each player's exiled set from step 1: look up the cards in exile by their
   new ObjectId (after step-1 zone change). Move each from exile to battlefield.
   Run the full ETB chain per permanent (same as Change 3 steps 7-9).
   Emit `PermanentEnteredBattlefield` for each.

Key correctness point: Step 1 exiles cards BEFORE step 2 sacrifices. The exiled
cards get new ObjectIds when moved to exile. Step 3 must track these new IDs
(returned by `move_object_to_zone`) to find the cards in exile. Cards that were
sacrificed in step 2 and redirected to exile by replacement effects are in exile
with DIFFERENT ObjectIds -- they must NOT be returned.

### Change 5: Hash new Effect variants

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add match arms after `Effect::GrantFlash` (line ~5211).
**Discriminants**: 79u8 for `ReturnAllFromGraveyardToBattlefield`, 80u8 for `LivingDeath`.

```
Effect::ReturnAllFromGraveyardToBattlefield {
    graveyards, filter, tapped, controller_override, unique_names, permanent_cards_only,
} => {
    79u8.hash_into(hasher);
    graveyards.hash_into(hasher);
    filter.hash_into(hasher);
    tapped.hash_into(hasher);
    controller_override.hash_into(hasher);
    unique_names.hash_into(hasher);
    permanent_cards_only.hash_into(hasher);
}
Effect::LivingDeath => {
    80u8.hash_into(hasher);
}
```

### Change 6: Exhaustive match updates

Files requiring new match arms for the new Effect variants:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/effects/mod.rs` | `execute_effect` match | ~4621 | Add dispatch for both variants (Changes 3-4) |
| `crates/engine/src/state/hash.rs` | `Effect` HashInto | ~5211 | Hash both variants (Change 5) |

No other files exhaustively match on `Effect` (verified: replay-viewer and TUI
do not match on Effect variants).

## Card Definition Fixes

### splendid_reclamation.rs

**Oracle text**: "Return all land cards from your graveyard to the battlefield tapped."
**Current state**: Empty abilities with TODO (line 13-16).
**Fix**: Replace TODO with:
```rust
AbilityDefinition::Spell {
    effect: Effect::ReturnAllFromGraveyardToBattlefield {
        graveyards: PlayerTarget::Controller,
        filter: TargetFilter {
            has_card_type: Some(CardType::Land),
            ..Default::default()
        },
        tapped: true,
        controller_override: None,
        unique_names: false,
        permanent_cards_only: false,
    },
}
```

### open_the_vaults.rs

**Oracle text**: "Return all artifact and enchantment cards from all graveyards to the battlefield under their owners' control. (Auras with nothing to enchant remain in graveyards.)"
**Current state**: Empty abilities with TODO (line 14-20).
**Fix**: Replace TODO with:
```rust
AbilityDefinition::Spell {
    effect: Effect::ReturnAllFromGraveyardToBattlefield {
        graveyards: PlayerTarget::AllPlayers,
        filter: TargetFilter {
            has_card_types: vec![CardType::Artifact, CardType::Enchantment],
            ..Default::default()
        },
        tapped: false,
        controller_override: None, // owners' control
        unique_names: false,
        permanent_cards_only: false,
    },
}
```

NOTE: "Auras with nothing to enchant remain in graveyards" is a complex
interactive constraint (Aura placement requires choosing what to enchant). The
deterministic fallback skips Aura placement -- Auras enter the battlefield
unattached and will be put into graveyard by SBA (CR 704.5m) at next SBA check.
This is functionally close but not perfect for Auras. Document with a comment.

### eerie_ultimatum.rs

**Oracle text**: "Return any number of permanent cards with different names from your graveyard to the battlefield."
**Current state**: Empty abilities with TODO (line 14-20).
**Fix**: Replace TODO with:
```rust
AbilityDefinition::Spell {
    effect: Effect::ReturnAllFromGraveyardToBattlefield {
        graveyards: PlayerTarget::Controller,
        filter: TargetFilter::default(),
        tapped: false,
        controller_override: None,
        unique_names: true,
        permanent_cards_only: true,
    },
}
```

NOTE: "Any number" means the player chooses which cards. Deterministic fallback:
return ALL permanent cards with unique names (maximum greed -- lowest ObjectId
per name wins). Interactive choice deferred to M10+.

### world_shaper.rs

**Oracle text**: "Whenever this creature attacks, you may mill three cards. When this creature dies, return all land cards from your graveyard to the battlefield tapped."
**Current state**: Empty abilities with two TODOs (lines 16-21).
**Fix**: Replace both TODOs. The attack trigger uses existing Effect::Mill. The
dies trigger uses ReturnAllFromGraveyardToBattlefield.
```rust
// Attack trigger: mill 3
AbilityDefinition::Triggered {
    trigger: TriggerCondition::WhenAttacks,
    effect: Effect::Mill {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(3),
    },
    optional: true,
    intervening_if: None,
    targets: vec![],
},
// Dies trigger: return all lands from GY to BF tapped
AbilityDefinition::Triggered {
    trigger: TriggerCondition::WhenDies,
    effect: Effect::ReturnAllFromGraveyardToBattlefield {
        graveyards: PlayerTarget::Controller,
        filter: TargetFilter {
            has_card_type: Some(CardType::Land),
            ..Default::default()
        },
        tapped: true,
        controller_override: None,
        unique_names: false,
        permanent_cards_only: false,
    },
    optional: false,
    intervening_if: None,
    targets: vec![],
},
```

## New Card Definitions

### living_death.rs

**Oracle text**: "Each player exiles all creature cards from their graveyard, then sacrifices all creatures they control, then puts all cards they exiled this way onto the battlefield."

**CardDefinition sketch**:
```rust
// Living Death -- {3}{B}{B} Sorcery
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("living-death"),
        name: "Living Death".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each player exiles all creature cards from their graveyard, then sacrifices all creatures they control, then puts all cards they exiled this way onto the battlefield.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::LivingDeath,
            },
        ],
        ..Default::default()
    }
}
```

**Registration**: Add `living_death` to `crates/engine/src/cards/defs/mod.rs` module
list and `crates/engine/src/cards/registry.rs` registration.

## Unit Tests

**File**: `crates/engine/tests/mass_reanimate.rs` (new test file)
**Tests to write**:

- `test_return_all_lands_from_graveyard_basic` -- Splendid Reclamation: put 3 land
  cards in GY, cast spell, verify all 3 are on battlefield tapped. CR 400.7.

- `test_return_all_lands_enters_tapped` -- Verify all returned lands are tapped, not
  untapped. CR 614.1c (enters-tapped replacement).

- `test_return_all_from_all_graveyards` -- Open the Vaults pattern: put artifacts in
  multiple players' graveyards, return all under owners' control. CR 101.4.

- `test_return_all_controller_is_owner` -- Verify returned permanents are controlled
  by their owner (not the caster). Tests `controller_override: None`.

- `test_return_all_unique_names` -- Eerie Ultimatum pattern: put 2 cards with the same
  name in GY, verify only 1 returns. Deterministic: lowest ObjectId wins.

- `test_return_all_permanent_cards_only` -- Verify instants/sorceries in GY are NOT
  returned when `permanent_cards_only: true`.

- `test_living_death_basic` -- Standard Living Death scenario: P1 has creatures on BF
  and creature cards in GY. After resolution: BF creatures are in GY, GY creatures are
  on BF. CR 400.7.

- `test_living_death_empty_graveyards` -- All players have creatures on BF but no
  creature cards in GY. Result: all creatures sacrificed, nothing returns.

- `test_living_death_no_creatures_on_battlefield` -- Players have creature cards in GY
  but no creatures on BF. Result: creature cards return to BF, nothing sacrificed.

- `test_living_death_replacement_exile_not_returned` -- Creature sacrificed in step 2
  gets exiled by a replacement effect (simulated). That exiled card should NOT return
  in step 3 (only step-1 exiled cards return). Tests the 2018-03-16 ruling.

- `test_mass_reanimate_etb_triggers_fire` -- Return multiple creatures with ETB
  triggers. Verify all ETB triggers are queued (CR 603.6a).

- `test_mass_reanimate_multiplayer` -- 4-player game: each player has different cards
  in graveyards. Verify correct owner assignment. CR 101.4 APNAP.

**Pattern**: Follow tests in `crates/engine/tests/bounce_all.rs` or
`crates/engine/tests/flash_grant.rs` for builder setup and assertion style.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (4 files)
- [ ] New card def authored (living_death.rs)
- [ ] living_death registered in mod.rs + registry.rs
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs

## Risks & Edge Cases

- **Aura placement (Open the Vaults)**: Auras entering without being cast need to
  choose what to enchant as they enter. The deterministic engine cannot make this
  choice interactively. Fallback: Auras enter unattached and are put into graveyard
  by SBA 704.5m. Document with a TODO comment on the card def. This is acceptable
  for pre-alpha -- the non-Aura cards work correctly.

- **Eerie Ultimatum "any number" choice**: The deterministic fallback returns ALL
  qualifying cards (maximum greed). In a real game, the player might want to return
  fewer cards. Interactive selection deferred to M10+.

- **Living Death step-1 exile tracking**: Must track the NEW ObjectIds created by
  `move_object_to_zone` in step 1, not the original graveyard IDs. Step 3 looks up
  these new IDs in exile. If this tracking is wrong, the wrong cards (or no cards)
  return.

- **Living Death replacement effects on sacrifice**: If a card like Leyline of the Void
  exiles sacrificed creatures instead of putting them in the graveyard, those exiled
  cards must NOT be included in step 3's return set. The implementation must maintain
  a clear separation between "step-1 exiled IDs" and any other exiled cards.

- **Simultaneous ETB triggers**: When N permanents enter simultaneously, all N
  PermanentEnteredBattlefield events occur before any trigger resolution. The existing
  `queue_carddef_etb_triggers` + pending trigger system handles this correctly because
  triggers queue and don't resolve until a player receives priority.

- **Token handling**: Tokens that were sacrificed in Living Death step 2 will briefly
  exist in the graveyard (CR 704.5d), then cease to exist as an SBA. They are NOT
  creature "cards" so they won't be in the step-1 exile set (they were on the
  battlefield, not in the graveyard). No special handling needed.

- **Non-creature cards in graveyard**: Living Death only exiles creature CARDS from
  graveyards (step 1). Non-creature cards in graveyards are untouched. The filter
  must check `has_card_type: Creature` against the card's characteristics, not the
  battlefield state (cards in graveyard have their printed characteristics).
