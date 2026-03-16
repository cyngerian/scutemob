# Primitive Batch Review: PB-1 -- Mana With Damage (Pain Lands)

**Date**: 2026-03-16
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 605 (Mana Abilities), 605.1a (activated mana ability criteria), 605.5a (triggered ability not a mana ability)
**Engine files reviewed**: `card_definition.rs` (TriggerCondition::WhenSelfBecomesTapped), `abilities.rs` (trigger dispatch), `mana.rs` (handle_tap_for_mana pain land damage), `hash.rs` (hash support), `game_object.rs` (TriggerEvent::SelfBecomesTapped), `replay_harness.rs` (try_as_tap_mana_ability, WhenSelfBecomesTapped enrichment), `effects/mod.rs` (AddMana execution)
**Card defs reviewed**: battlefield_forge, caves_of_koilos, city_of_brass, llanowar_wastes, shivan_reef, sulfurous_springs, underground_river, yavimaya_coast (8 total)

## Verdict: needs-fix

One MEDIUM finding: all 7 pain lands add both mana colors simultaneously (2 mana) instead of 1 mana of the player's choice. This is documented in a test comment (pain_lands.rs:235) as a known limitation pending interactive mana choice (M10). The engine and trigger plumbing for City of Brass is correct. Oracle text matches for all 8 cards. No TODOs remain. Hash support present for both new enum variants.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `card_definition.rs:1683` | **WhenSelfBecomesTapped doc comment references dispatched path.** Comment says "Dispatched via GameEvent::PermanentTapped -> TriggerEvent::SelfBecomesTapped in check_triggers" which is accurate. No issue, just noting the dispatch chain is documented inline. |

No engine-level issues found. The `WhenSelfBecomesTapped` trigger condition is:
- Defined in `card_definition.rs:1683`
- Hashed in `hash.rs:3889` (discriminant 27)
- Mapped to `TriggerEvent::SelfBecomesTapped` in `replay_harness.rs:2404`
- `TriggerEvent::SelfBecomesTapped` defined in `game_object.rs:289`
- Hashed in `hash.rs:1805` (discriminant 3)
- Dispatched from `GameEvent::PermanentTapped` in `abilities.rs:3907-3915`
- `GameEvent::PermanentTapped` emitted from: `mana.rs:89`, `combat.rs:433/446`, `abilities.rs:432/7224/7508`, `casting.rs:4851/4965`, `replacement.rs:1548`, `effects/mod.rs:938`

All exhaustive match arms are covered. No missing dispatch sites.

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | MEDIUM | All 7 pain lands | **Adds 2 mana instead of 1.** Oracle says "Add {X} or {Y}" (choose one color), but `AddMana` with both colors set to 1 adds both simultaneously. **Fix:** Replace the `AddMana` in the Sequence with `AddManaChoice { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) }` to correctly produce 1 mana (currently falls back to colorless pending M10 interactive choice). Alternatively, split into two separate activated abilities, one per color, each with `Effect::Sequence([AddMana{single_color}, DealDamage])`. The split approach is more correct pre-M10 because each ability would add exactly 1 mana of a deterministic color. |

### Finding Details

#### Finding 1: Pain lands produce 2 mana instead of 1

**Severity**: MEDIUM
**Files**: `crates/engine/src/cards/defs/battlefield_forge.rs:26`, `caves_of_koilos.rs:23`, `llanowar_wastes.rs:23`, `shivan_reef.rs:23`, `sulfurous_springs.rs:23`, `underground_river.rs:23`, `yavimaya_coast.rs:23`
**Oracle**: "{T}: Add {R} or {W}. This land deals 1 damage to you." (Battlefield Forge example)
**Issue**: The colored mana ability uses `Effect::AddMana { mana: mana_pool(1, 0, 0, 1, 0, 0) }` which sets both White=1 and Red=1. When `handle_tap_for_mana` iterates `ability.produces`, it adds ALL non-zero entries to the player's mana pool (mana.rs:139-146). Result: player gets 1W + 1R = 2 mana total. Oracle text says "Add {R} or {W}" meaning the player picks exactly ONE color. This is documented in test comment `pain_lands.rs:235`: "Currently adds both until interactive choice is implemented."

**Fix**: The best pre-M10 approach is to split each pain land's second ability into two separate activated abilities, each producing exactly 1 mana of one color plus dealing 1 damage. For Battlefield Forge this would be:
```rust
// Ability 2a: {T}: Add {R}. Deals 1 damage.
AbilityDefinition::Activated {
    cost: Cost::Tap,
    effect: Effect::Sequence(vec![
        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 1, 0, 0) },
        Effect::DealDamage { target: EffectTarget::Controller, amount: EffectAmount::Fixed(1) },
    ]),
    timing_restriction: None, targets: vec![],
},
// Ability 2b: {T}: Add {W}. Deals 1 damage.
AbilityDefinition::Activated {
    cost: Cost::Tap,
    effect: Effect::Sequence(vec![
        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(1, 0, 0, 0, 0, 0) },
        Effect::DealDamage { target: EffectTarget::Controller, amount: EffectAmount::Fixed(1) },
    ]),
    timing_restriction: None, targets: vec![],
},
```
This gives the player the correct choice (pick which ability to activate) without needing interactive mana selection. Tests would need updating: `ability_index: 1` becomes "tap for color A", `ability_index: 2` becomes "tap for color B". The `all_pain_lands_deal_damage_on_colored_tap` test should test both indices.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 605.1a (activated mana ability criteria) | Yes | Yes | Pain lands + City of Brass colorless ability |
| 605.3b (mana abilities don't use stack) | Yes | Yes | `handle_tap_for_mana` resolves immediately |
| 605.5a (triggered != mana ability) | Yes | Yes | City of Brass triggered ability correctly NOT a mana ability; test verifies no DamageDealt in mana events |
| 605.5 (mana ability special action, priority retained) | Yes | Implicit | `handle_tap_for_mana` does not reset `players_passed` |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Battlefield Forge | Yes | 0 | No | Produces 2 mana (W+R) instead of 1 |
| Caves of Koilos | Yes | 0 | No | Produces 2 mana (W+B) instead of 1 |
| City of Brass | Yes | 0 | Yes | Triggered damage + any-color mana correctly modeled |
| Llanowar Wastes | Yes | 0 | No | Produces 2 mana (B+G) instead of 1 |
| Shivan Reef | Yes | 0 | No | Produces 2 mana (U+R) instead of 1 |
| Sulfurous Springs | Yes | 0 | No | Produces 2 mana (B+R) instead of 1 |
| Underground River | Yes | 0 | No | Produces 2 mana (U+B) instead of 1 |
| Yavimaya Coast | Yes | 0 | No | Produces 2 mana (G+U) instead of 1 |

## Test Coverage

| Test | What it covers | Adequate? |
|------|---------------|-----------|
| `battlefield_forge_colorless_tap_no_damage` | Colorless ability: mana added, no damage | Yes |
| `battlefield_forge_colored_tap_deals_damage` | Colored ability: damage dealt, life decreases | Yes |
| `all_pain_lands_deal_damage_on_colored_tap` | All 7 pain lands deal damage on colored tap | Yes |
| `city_of_brass_tap_produces_mana` | City of Brass: mana produced, PermanentTapped emitted, no DamageDealt in mana events | Yes |
| `shivan_reef_produces_blue_and_red_with_damage` | Specific mana colors + damage | Documents known 2-mana bug |
| (missing) | City of Brass: trigger resolves and deals damage | No -- test only verifies trigger fires, not that damage resolves |
| (missing) | City of Brass: tapped by opponent's effect still triggers | No -- CR 605.5a edge case |
| (missing) | Negative: colorless tap on any pain land other than Battlefield Forge | Partial -- only tests Battlefield Forge colorless |
