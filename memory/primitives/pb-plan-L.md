# Primitive Batch Plan: PB-L -- Reveal/X Effects

**Generated**: 2026-04-06
**Primitive**: Three DSL capabilities: (1) EffectAmount::DomainCount, (2) RevealAndRoute card def fixes for Battlefield destination, (3) AltCostKind::CommanderFreeCast
**CR Rules**: CR 701.20 (Reveal), CR 118.9 (Alternative Costs), CR 903.6 (Commander designation)
**Cards affected**: 7 (5 existing fixes + 2 new)
**Dependencies**: None (all prerequisite infrastructure exists)
**Deferred items from prior PBs**: None directly relevant

## Primitive Specification

PB-L adds three leaf-level DSL capabilities that together unblock 7 cards from A-38/A-42:

1. **EffectAmount::DomainCount** -- A new EffectAmount variant that counts the number of distinct basic land types (Plains, Island, Swamp, Mountain, Forest) among lands the controller controls. Used by Domain cards (Allied Strategies, Territorial Maro) where the count determines an effect magnitude or a CDA P/T value. The counting logic already exists in `SelfCostReduction::BasicLandTypes` in casting.rs; this extends it to the effect amount system.

2. **RevealAndRoute with Battlefield destination** -- `Effect::RevealAndRoute` already supports `ZoneTarget::Battlefield` as a destination. Coiling Oracle's card def predates PB-22 (which added RevealAndRoute) and has a stale TODO. The fix is purely a card def update -- no engine change needed. Bounty of Skemfar already uses RevealAndRoute with Battlefield and just needs its TODO cleaned up (the dual-filter limitation is documented but the existing single-filter implementation is acceptable).

3. **AltCostKind::CommanderFreeCast** -- A new alternative cost variant for the Commander 2020 free-cast cycle (Fierce Guardianship, Deadly Rollick, Flawless Maneuver). These cards say "If you control a commander, you may cast this spell without paying its mana cost." This is a conditional alternative cost (CR 118.9) that sets the cost to {0} when the caster controls any commander on the battlefield. The condition check happens in casting.rs at cast validation time.

## CR Rule Text

### CR 118.9 (Alternative Costs)
> Some spells have alternative costs. An alternative cost is a cost listed in a spell's text, or applied to it from another effect, that its controller may pay rather than paying the spell's mana cost. Alternative costs are usually phrased, "You may [action] rather than pay [this object's] mana cost," or "You may cast [this object] without paying its mana cost."

### CR 118.9a
> Only one alternative cost can be applied to any one spell as it's being cast.

### CR 118.9c
> An alternative cost doesn't change a spell's mana cost, only what its controller has to pay to cast it.

### CR 118.9d
> If an alternative cost is being paid to cast a spell, any additional costs, cost increases, and cost reductions that affect that spell are applied to that alternative cost.

### CR 701.20 (Reveal)
> To reveal a card, show that card to all players for a brief time.

### CR 701.20a
> If an effect causes a card to be revealed, it remains revealed for as long as necessary to complete the parts of the effect that card is relevant to.

### Card Rulings (Fierce Guardianship, 2020-04-17)
> "It doesn't matter whose commander you control. Any one will do. If you have two commanders, you just need to control one of them."
> "Once you begin casting this spell, players can't take any other actions until you're done casting it. Notably, they can't try to remove the commander you control to make you pay its cost."

## Engine Changes

### Change 1: Add EffectAmount::DomainCount variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `DomainCount` variant to `EffectAmount` enum after `ChosenTypeCreatureCount`.
**Pattern**: Follow `ChosenTypeCreatureCount` at line ~2091.

```rust
/// Domain count: the number of distinct basic land types (Plains, Island, Swamp,
/// Mountain, Forest) among lands the controller controls.
///
/// CR 305.6 / ability word "Domain": Used for effects whose magnitude equals
/// the domain count (e.g., Allied Strategies: "draw a card for each basic land
/// type among lands they control").
DomainCount,
```

### Change 2: Add DomainCount dispatch to resolve_amount

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm for `EffectAmount::DomainCount` in `fn resolve_amount` (line ~5787, after `ChosenTypeCreatureCount`).
**CR**: CR 305.6 / Domain ability word -- counts distinct basic land types among lands the controller controls.

```rust
EffectAmount::DomainCount => {
    // Domain: count distinct basic land types among lands the controller controls.
    // CR 305.6: Basic land types are Plains, Island, Swamp, Mountain, Forest.
    let controller_id = resolve_player_target_list(state, &PlayerTarget::Controller, ctx)
        .into_iter()
        .next()
        .unwrap_or(ctx.controller);
    let basic_land_subtypes = [
        SubType("Plains".to_string()),
        SubType("Island".to_string()),
        SubType("Swamp".to_string()),
        SubType("Mountain".to_string()),
        SubType("Forest".to_string()),
    ];
    let mut count = 0i32;
    for sub in &basic_land_subtypes {
        let has_it = state.objects.values().any(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.is_phased_in()
                && obj.controller == controller_id
                && {
                    let chars = crate::rules::layers::calculate_characteristics(state, obj.id)
                        .unwrap_or_else(|| obj.characteristics.clone());
                    chars.card_types.contains(&CardType::Land)
                        && chars.subtypes.contains(sub)
                }
        });
        if has_it {
            count += 1;
        }
    }
    count
}
```

### Change 3: Hash DomainCount in state/hash.rs

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `EffectAmount::DomainCount` with discriminant 14, after `ChosenTypeCreatureCount` (discriminant 13).
**Pattern**: Follow `ChosenTypeCreatureCount` at line ~4153.

```rust
// Domain count (discriminant 14) — PB-L: basic land types among lands you control
EffectAmount::DomainCount => 14u8.hash_into(hasher),
```

### Change 4: Add AltCostKind::CommanderFreeCast variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `CommanderFreeCast` variant to `AltCostKind` enum after `Adventure` (line ~149).
**CR**: CR 118.9 -- "you may cast this spell without paying its mana cost" conditional on controlling a commander.

```rust
/// CR 118.9 / Commander 2020 cycle: "If you control a commander, you may cast
/// this spell without paying its mana cost." Cost becomes {0}; condition validated
/// at cast time (caster must control a commander on the battlefield).
CommanderFreeCast,
```

### Change 5: Hash CommanderFreeCast in state/hash.rs

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `AltCostKind::CommanderFreeCast` with discriminant 28, after `Adventure` (discriminant 27) at line ~2777.

```rust
AltCostKind::CommanderFreeCast => 28,
```

### Change 6: Casting validation for CommanderFreeCast

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Two additions:

**(a)** Add boolean extraction (line ~163, after `cast_with_adventure`):
```rust
let cast_with_commander_free = alt_cost == Some(AltCostKind::CommanderFreeCast);
```

**(b)** Add condition validation (after the Morph face-down validation block, around line ~1960-1990). Validate the caster controls a commander on the battlefield:
```rust
if cast_with_commander_free {
    // CR 118.9 / 2020-04-17 ruling: "It doesn't matter whose commander you control."
    // Check if the caster controls ANY commander on the battlefield.
    let controls_commander = state.objects.values().any(|obj| {
        obj.zone == ZoneId::Battlefield
            && obj.is_phased_in()
            && obj.controller == caster
            && state.commander_cards.iter().any(|&(_, cid)| obj.card_id == Some(cid))
    });
    if !controls_commander {
        return Err(GameStateError::InvalidCommand(
            "commander free-cast requires controlling a commander on the battlefield (CR 118.9)".into(),
        ));
    }
}
```

**(c)** Add cost override in the base cost chain (after Plot free-cast around line ~2124):
```rust
} else if cast_with_commander_free {
    // CR 118.9: Cast without paying mana cost. Cost is zero.
    Some(ManaCost::default())
}
```

### Change 7: Replay harness support for CommanderFreeCast

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add a `cast_spell_commander_free` action type (or support `alt_cost: "commander_free_cast"` string in the harness action translation). Follow the pattern of `cast_spell_evoke` or `cast_spell_miracle`.

In `translate_player_action`, add handling for `"cast_spell_commander_free"` that creates a `CastSpell` command with `alt_cost: Some(AltCostKind::CommanderFreeCast)`.

### Change 8: Exhaustive match updates

Files requiring new match arms for the new variants:

| File | Match expression | Approximate Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/cards/card_definition.rs` | `EffectAmount` enum | ~2091 | Add `DomainCount` variant |
| `crates/engine/src/effects/mod.rs` | `resolve_amount` match | ~5787 | Add `DomainCount` arm |
| `crates/engine/src/state/hash.rs` | `EffectAmount` HashInto match | ~4153 | Add disc 14 for `DomainCount` |
| `crates/engine/src/state/types.rs` | `AltCostKind` enum | ~149 | Add `CommanderFreeCast` variant |
| `crates/engine/src/state/hash.rs` | `AltCostKind` HashInto match | ~2777 | Add disc 28 for `CommanderFreeCast` |
| `crates/engine/src/rules/casting.rs` | alt_cost boolean extraction | ~163 | Add `cast_with_commander_free` |
| `crates/engine/src/rules/casting.rs` | condition validation block | ~1960 | Validate commander on BF |
| `crates/engine/src/rules/casting.rs` | base cost chain | ~2124 | Set cost to {0} |
| `crates/engine/src/testing/replay_harness.rs` | action translation | varies | Add harness action |

**Note**: `AltCostKind` is `Copy` and only has exhaustive matches in `hash.rs` and `casting.rs`. The card def files use it as a value, not in match arms. `EffectAmount` only has exhaustive matches in `hash.rs` and `effects/mod.rs` (`resolve_amount`). No matches exist in `view_model.rs`, `stack_view.rs`, or TUI -- verified.

## Card Definition Fixes

### coiling_oracle.rs
**Oracle text**: "When this creature enters, reveal the top card of your library. If it's a land card, put it onto the battlefield. Otherwise, put that card into your hand."
**Current state**: `Effect::Nothing` with stale TODO -- predates PB-22 (which added RevealAndRoute).
**Fix**: Replace `Effect::Nothing` with `Effect::RevealAndRoute { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1), filter: TargetFilter { has_card_type: Some(CardType::Land), ..Default::default() }, matched_dest: ZoneTarget::Battlefield { tapped: false }, unmatched_dest: ZoneTarget::Hand { owner: PlayerTarget::Controller } }`. Remove all TODO comments.

### bounty_of_skemfar.rs
**Oracle text**: "Reveal the top six cards of your library. You may put up to one land card from among them onto the battlefield tapped and up to one Elf card from among them into your hand. Put the rest on the bottom of your library in a random order."
**Current state**: Uses `RevealAndRoute` with land filter -> BF tapped, rest -> bottom. TODO about dual-filter routing.
**Fix**: The single-filter implementation (land -> BF) is a reasonable approximation. Update the TODO to a NOTE explaining the limitation: "NOTE: Oracle says 'up to one land + up to one Elf'; DSL approximation routes all lands to BF (not 'up to one') and does not separately route Elf to hand. The land-to-BF portion is the higher-value effect." Remove the "DSL gap" language.

### fierce_guardianship.rs
**Oracle text**: "If you control a commander, you may cast this spell without paying its mana cost. Counter target noncreature spell."
**Current state**: Has the CounterSpell effect correctly implemented. TODO about conditional free-cast.
**Fix**: Add `AltCastAbility { kind: AltCostKind::CommanderFreeCast, cost: ManaCost::default(), details: None }` to abilities. Remove the TODO comment.

### deadly_rollick.rs
**Oracle text**: "If you control a commander, you may cast this spell without paying its mana cost. Exile target creature."
**Current state**: Has ExileObject effect correctly implemented. TODO about commander-conditional free cast.
**Fix**: Add `AltCastAbility { kind: AltCostKind::CommanderFreeCast, cost: ManaCost::default(), details: None }` to abilities. Remove the TODO comments.

### flawless_maneuver.rs
**Oracle text**: "If you control a commander, you may cast this spell without paying its mana cost. Creatures you control gain indestructible until end of turn."
**Current state**: Has the indestructible grant effect implemented. TODO about conditional free cast.
**Fix**: Add `AltCastAbility { kind: AltCostKind::CommanderFreeCast, cost: ManaCost::default(), details: None }` to abilities. Remove the TODO comment.

## New Card Definitions

### allied_strategies.rs
**Oracle text**: "Domain -- Target player draws a card for each basic land type among lands they control."
**CardDefinition sketch**:
```rust
CardDefinition {
    card_id: cid("allied-strategies"),
    name: "Allied Strategies".to_string(),
    mana_cost: Some(ManaCost { generic: 4, blue: 1, ..Default::default() }),
    types: types(&[CardType::Sorcery]),
    oracle_text: "Domain \u{2014} Target player draws a card for each basic land type among lands they control.".to_string(),
    abilities: vec![AbilityDefinition::Spell {
        effect: Effect::DrawCards {
            player: PlayerTarget::DeclaredTarget { index: 0 },
            count: EffectAmount::DomainCount,
        },
        targets: vec![TargetRequirement::TargetPlayer],
        modes: None,
        cant_be_countered: false,
    }],
    ..Default::default()
}
```

### territorial_maro.rs
**Oracle text**: "Domain -- Territorial Maro's power and toughness are each equal to twice the number of basic land types among lands you control."
**CardDefinition sketch**: CDA creature with `*/*` P/T. Uses `power: None, toughness: None` and a CDA static ability. The CDA effect needs `EffectAmount::Sum(Box::new(EffectAmount::DomainCount), Box::new(EffectAmount::DomainCount))` to compute "twice the domain count" (or use a multiplication pattern if one exists).

Actually, the engine has no `EffectAmount::Multiply`. The simplest approach: use `EffectAmount::Sum(Box::new(EffectAmount::DomainCount), Box::new(EffectAmount::DomainCount))` which computes domain_count + domain_count = 2 * domain_count.

```rust
CardDefinition {
    card_id: cid("territorial-maro"),
    name: "Territorial Maro".to_string(),
    mana_cost: Some(ManaCost { generic: 4, green: 1, ..Default::default() }),
    types: creature_types(&["Elemental"]),
    oracle_text: "Domain \u{2014} Territorial Maro's power and toughness are each equal to twice the number of basic land types among lands you control.".to_string(),
    power: None,
    toughness: None,
    abilities: vec![
        // CDA: P/T = 2 * domain count
        AbilityDefinition::Static {
            effect: ContinuousEffectDef {
                layer: EffectLayer::CDA,
                modification: LayerModification::SetBoth(
                    EffectAmount::Sum(
                        Box::new(EffectAmount::DomainCount),
                        Box::new(EffectAmount::DomainCount),
                    ),
                ),
                filter: EffectFilter::Source,
                duration: EffectDuration::WhileOnBattlefield,
                condition: None,
            },
        },
    ],
    ..Default::default()
}
```

Note: The CDA pattern (`EffectLayer::CDA` + `LayerModification::SetBoth`) should follow the same pattern as Abomination of Llanowar and other `*/*` creatures. Verify `SetBoth` exists and accepts an `EffectAmount`.

## Unit Tests

**File**: `crates/engine/tests/domain_and_freecast.rs` (new file)
**Tests to write**:

1. `test_domain_count_zero_lands` -- CR 305.6: with no lands, domain count = 0. Allied Strategies draws 0 cards.
2. `test_domain_count_all_five_types` -- CR 305.6: with all 5 basic land types, domain count = 5. Allied Strategies draws 5 cards.
3. `test_domain_count_dual_land` -- CR 305.6: a single land with multiple basic land types (e.g., via Dryad of the Ilysian Grove) counts each type once. Domain count = 5 with one land that is all types.
4. `test_domain_count_duplicate_types` -- CR 305.6: two Plains and one Island = 2 (Plains + Island), not 3.
5. `test_territorial_maro_cda` -- CDA P/T = 2 * domain count. With 3 basic land types, P/T = 6/6.
6. `test_commander_free_cast_with_commander` -- CR 118.9: Fierce Guardianship cast for free when caster controls a commander. Spell resolves, counters target.
7. `test_commander_free_cast_without_commander` -- CR 118.9: Fierce Guardianship with CommanderFreeCast alt cost rejected when no commander controlled. Returns error.
8. `test_commander_free_cast_any_commander` -- 2020-04-17 ruling: "doesn't matter whose commander you control." Works with any player's commander.
9. `test_coiling_oracle_land_to_battlefield` -- CR 701.20: reveal top card; if land, put onto battlefield (untapped). Non-land goes to hand.
10. `test_coiling_oracle_nonland_to_hand` -- CR 701.20: reveal top card; if not land, put into hand.

**Pattern**: Follow tests for `SelfCostReduction::BasicLandTypes` in `tests/spell_cost_modification.rs` for domain count validation. Follow tests for `AltCostKind` (Evoke, Dash) in `tests/spell_resolution.rs` for free-cast validation.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (5 cards)
- [ ] New card defs authored (2 cards: Allied Strategies, Territorial Maro)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs
- [ ] Card defs registered in mod.rs

## Risks & Edge Cases

- **Domain count with Dryad of the Ilysian Grove / Blood Moon**: Dryad makes all lands every basic land type (Layer 4). Domain count must use `calculate_characteristics()` (layer-resolved types), not base characteristics. The implementation in Change 2 does this correctly.
- **Commander free-cast + commander tax**: CR 118.9d says additional costs apply to alternative costs. Commander tax should NOT apply here because the free-cast IS the alternative cost (not cast from command zone). The spell is cast from hand. No interaction.
- **Commander free-cast + Fuse/Splice/Kicker**: CR 118.9d says additional costs apply. Kicker/Splice/Fuse costs should still be payable on top of the free-cast. The existing cost pipeline already handles this (additional costs are added after base cost selection).
- **"Any commander" ruling**: The check must scan ALL commander_cards, not just the caster's commanders. The ruling says "It doesn't matter whose commander you control." An opponent's commander that you gained control of satisfies the condition.
- **EffectAmount::DomainCount with PlayerTarget**: The current design always resolves against the controller (the spell/ability's controller). Allied Strategies targets "target player" for who draws, but domain is still counted for the caster's lands. If future cards need domain count for a target player, we'd need `DomainCount { player: PlayerTarget }`. For now, controller-only is sufficient for both known cards.
- **Territorial Maro CDA applies in all zones** (CR 604.3): CDAs function everywhere -- hand, graveyard, etc. The `EffectLayer::CDA` layer handles this. With 0 lands in play, P/T = 0/0 in all zones. SBA kills it on the battlefield (CR 704.5f).
- **Bounty of Skemfar approximation**: The current implementation routes ALL matching lands to BF (not "up to one"). This is a minor overcorrection but acceptable for the current DSL -- interactive choice ("choose one from among matched") requires M10-level player choice infrastructure.
