# Primitive Batch Review: PB-13 -- Specialized Mechanics

**Date**: 2026-03-18
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 702.52 (Dredge), 702.27 (Buyback), 702.131 (Ascend), 724 (Monarch), 702.92 (Living Weapon), 702.34 (Channel), 702.11 (Hexproof on players)
**Engine files reviewed**: replacement.rs (Dredge), casting.rs (Buyback, HexproofPlayer), sba.rs (Ascend, Monarch transfer), turn_actions.rs (Monarch EOT draw, combat steal), effects/mod.rs (BecomeMonarch, CreateTokenAndAttachSource), builder.rs (LivingWeapon trigger wiring), events.rs (Dredge/Monarch events), command.rs (ChooseDredge), engine.rs (ChooseDredge dispatch), card_definition.rs (Cost::DiscardSelf, TimingRestriction, AbilityDefinition::Buyback), state/types.rs (KeywordAbility variants), state/hash.rs (all new discriminants), abilities.rs (HexproofPlayer targeting), state/mod.rs (monarch field), state/player.rs (has_citys_blessing)
**Card defs reviewed**: 14 (golgari_grave_troll, searing_touch, eomer_king_of_rohan, batterskull, wayward_swordtooth, arch_of_orazca, otawara_soaring_city, boseiju_who_endures, sokenzan_crucible_of_defiance, takenuma_abandoned_mire, eiganjo_seat_of_the_empire, blinkmoth_nexus, inkmoth_nexus, crystal_barricade, twilight_prophet, serra_ascendant, hammer_of_nazahn, monster_manual)

## Verdict: needs-fix

PB-13 implemented 8 of 14 planned sub-batches (13a, 13b, 13c, 13e, 13f, 13g, 13k, 13n). Six sub-batches (13d, 13h, 13i, 13j, 13l, 13m) were explicitly deferred per project-status.md. The implemented engine changes are solid: Dredge has comprehensive draw-replacement logic with player choice; Buyback has full casting/resolution integration; Ascend/City's Blessing has correct SBA-based checking per CR 702.131b; Monarch has EOT draw, combat steal, and player-leaves-game transfer; Living Weapon has atomic token creation and equipment attachment; Channel uses Cost::DiscardSelf for hand-based activation; HexproofPlayer is enforced in both casting.rs and abilities.rs targeting. Tests are thorough (13 Dredge, 9 Buyback, 8 Ascend, 6 Monarch, 6 Living Weapon, 4 Channel = 46 total). However, several card definitions have significant issues: Golgari Grave-Troll has wrong P/T (0/4 vs oracle 0/0), Batterskull is missing its bounce ability, multiple Channel lands have TODOs for cost reduction and target filters, and several cards are missing abilities entirely.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `arch_of_orazca.rs:27` | **Arch of Orazca activation restriction modeled as conditional effect, not legal_actions gate.** Comment acknowledges this: "Proper activation restriction requires legal_actions filter (PB-18 stax framework)." Since PB-18 is now complete, this should be re-evaluated. **Fix:** Check if PB-18 stax framework now supports "activate only if" restrictions; if so, replace the Conditional wrapper with a proper activation restriction. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 2 | **HIGH** | `golgari_grave_troll.rs` | **Wrong P/T.** Oracle says 0/0, def says 0/4. **Fix:** Change `toughness: Some(4)` to `toughness: Some(0)`. |
| 3 | **HIGH** | `batterskull.rs` | **Missing bounce ability.** Oracle says "{3}: Return this Equipment to its owner's hand." Def has no such ability. **Fix:** Add an `AbilityDefinition::Activated` with `Cost::Mana(ManaCost { generic: 3 })` and `Effect::MoveZone { target: EffectTarget::Source, to: ZoneTarget::Hand { owner: PlayerTarget::Controller }, controller_override: None }`. |
| 4 | MEDIUM | `wayward_swordtooth.rs` | **Missing 2 abilities.** Oracle: "You may play an additional land on each of your turns" and "can't attack or block unless you have the city's blessing." Only Ascend keyword is present. **Fix:** Add static continuous effects for additional land play (if DSL supports it) and attack/block restriction conditioned on city's blessing (needs `Condition::DoesNotHaveCitysBlessing` or similar). |
| 5 | MEDIUM | `otawara_soaring_city.rs:23-25` | **Two TODOs remain.** Target filter should exclude lands (oracle: "artifact, creature, enchantment, or planeswalker"). Cost reduction per legendary creature missing. **Fix:** Replace `TargetRequirement::TargetPermanent` with a filter that excludes lands. Add self_cost_reduction or spell_cost_modifiers for the legendary creature discount. |
| 6 | MEDIUM | `boseiju_who_endures.rs:29-30` | **Two TODOs remain.** Target filter should be "artifact, enchantment, or nonbasic land an opponent controls." Cost reduction per legendary creature missing. **Fix:** Add proper target filter. Add cost reduction. Note: the SearchLibrary filter uses `basic: true` which should be checking for basic land type (Forest/Plains/etc), not the Basic supertype -- verify this matches oracle "land card with a basic land type." |
| 7 | MEDIUM | `sokenzan_crucible_of_defiance.rs:23-24` | **Two TODOs remain.** Tokens missing haste until EOT. Cost reduction missing. **Fix:** Add haste keyword to token spec or add a temporary continuous effect granting haste until end of turn after creation. Add cost reduction. |
| 8 | MEDIUM | `takenuma_abandoned_mire.rs:24-27` | **Two TODOs remain.** Only mills 3, missing "return a creature or planeswalker card from your graveyard to your hand." Cost reduction missing. **Fix:** Add a MoveZone from graveyard with creature/planeswalker type filter after the mill. Add cost reduction. |
| 9 | MEDIUM | `eiganjo_seat_of_the_empire.rs:22-23` | **Two TODOs remain.** Target should be "attacking or blocking creature" not any creature. Cost reduction missing. **Fix:** Add attacking/blocking filter to target requirement. Add cost reduction. |
| 10 | MEDIUM | `eomer_king_of_rohan.rs:26-29` | **Two TODOs, card is mostly empty.** Missing ETB counter placement (X +1/+1 counters = other Humans) and ETB trigger (monarch + damage). Only has DoubleStrike. **Fix:** Implement ETB with ForEach counter placement and Monarch+DealDamage trigger when primitives allow. |
| 11 | MEDIUM | `twilight_prophet.rs:23-24` | **TODO remaining.** Missing upkeep trigger conditioned on city's blessing (reveal top, draw, drain life). Only has Flying + Ascend. **Fix:** Add TriggeredAbilityDef for upkeep with intervening-if on HasCitysBlessing, drawing from top + DrainLife based on mana value. |
| 12 | MEDIUM | `crystal_barricade.rs:22` | **TODO remaining.** Missing "prevent all noncombat damage that would be dealt to other creatures you control." Has Defender + HexproofPlayer only. **Fix:** Add replacement effect for noncombat damage prevention when DSL supports it. |
| 13 | MEDIUM | `serra_ascendant.rs:7-10` | **TODO remaining.** Missing conditional static: "As long as you have 30 or more life, gets +5/+5 and has flying." Only has Lifelink. **Fix:** Implement with conditional static when EffectDuration supports life-total conditions. |
| 14 | MEDIUM | `hammer_of_nazahn.rs:22-23` | **TODO remaining.** Missing ETB trigger: "Whenever Hammer of Nazahn or another Equipment you control enters, you may attach that Equipment to target creature you control." Has the equip/static effects but not the key trigger. **Fix:** Add TriggeredAbilityDef watching for Equipment ETB (not just self). |
| 15 | MEDIUM | `monster_manual.rs` | **Card is essentially empty.** Only has card_id, name, mana_cost, and types. Missing all abilities: the activated ability "{1}{G}, {T}: You may put a creature card from your hand onto the battlefield" and the Adventure half "Zoological Study" (sorcery side). **Fix:** Implement the activated ability and Adventure framework (13m was deferred but the card was authored). |
| 16 | LOW | `golgari_grave_troll.rs:8-12` | **Three TODOs for deferred abilities.** ETB counter placement, regeneration activated ability, and CDA power (should be `power: None` for 0/0 base since it's counter-dependent). These are DSL gaps but should be tracked. **Fix:** Change `power: Some(0)` to `power: None` to match the */* CDA pattern if applicable, otherwise keep Some(0) for the base stats of the 0/0 creature. Actually oracle says P/T is 0/0 so keep Some(0) for both once toughness is fixed. |

### Finding Details

#### Finding 2: Golgari Grave-Troll wrong P/T

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/golgari_grave_troll.rs:34`
**Oracle**: "P/T: 0/0"
**Issue**: The card def has `toughness: Some(4)` but oracle text says the card is 0/0. The comment at line 8 says "Power is fixed at 0 (real card is 0/0)" but then sets toughness to 4. This produces wrong game state: SBAs would not kill this creature when it should die as a 0/0 without counters.
**Fix**: Change `toughness: Some(4)` to `toughness: Some(0)`.

#### Finding 3: Batterskull missing bounce ability

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/batterskull.rs:44`
**Oracle**: "{3}: Return this Equipment to its owner's hand."
**Issue**: The card def is missing Batterskull's third ability. Oracle text shows four abilities: Living Weapon, +4/+4 with vigilance/lifelink, {3} bounce, and Equip {5}. The bounce ability is completely absent. This is a wrong game state issue since a key interaction (bouncing Batterskull to reset the Germ) is unavailable.
**Fix**: Add after the static abilities:
```rust
AbilityDefinition::Activated {
    cost: Cost::Mana(ManaCost { generic: 3, ..Default::default() }),
    effect: Effect::MoveZone {
        target: EffectTarget::Source,
        to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
        controller_override: None,
    },
    timing_restriction: None,
    targets: vec![],
},
```

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 702.52 (Dredge) | Yes | Yes (13 tests) | Draw step + effect draw, decline, insufficient library, multiple options, milled-card-for-next-draw |
| 702.52b | Yes | Yes | Insufficient library blocks dredge, exact-count edge case |
| 702.27 (Buyback) | Yes | Yes (9 tests) | Paid/unpaid, countered, fizzled, flashback override, mana validation |
| 702.131 (Ascend) | Yes | Yes (8 tests) | Permanent static, spell ability, permanent once gained, no source, multiple players, tokens |
| 702.131a (spell) | Yes | Yes | Resolution-time check |
| 702.131b (permanent) | Yes | Yes | SBA check, priority over other SBAs |
| 702.131c (permanent designation) | Yes | Yes | Cannot be lost |
| 724.1 (Monarch designation) | Yes | Yes | BecomeMonarch effect |
| 724.2 (EOT draw + combat steal) | Yes | Yes | Both inherent triggers implemented |
| 724.3 (Single monarch) | Yes | Yes | Replacement semantics |
| 724.4 (Monarch leaves) | Yes | Yes | Active player inherits, fallback logic |
| 702.92 (Living Weapon) | Yes | Yes (6 tests) | ETB trigger, Germ characteristics, Equipment survives, re-equip |
| 702.34 (Channel as DiscardSelf) | Yes (via Cost::DiscardSelf) | Yes (4 tests) | Hand activation, battlefield rejection, owner-only, mana check |
| 702.11c (Player hexproof) | Yes | No dedicated test | Enforced in casting.rs + abilities.rs targeting, uses layer-resolved characteristics |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| golgari_grave_troll | **No** | 3 | **No** | P/T 0/4 should be 0/0; ETB counters, regen, CDA deferred |
| searing_touch | Yes | 0 | Yes | Clean |
| eomer_king_of_rohan | **No** | 2 | **No** | Only DoubleStrike; missing ETB counters + monarch trigger |
| batterskull | **No** | 0 (but missing ability) | **No** | Missing {3} bounce ability |
| wayward_swordtooth | **No** | 0 (implicit) | **No** | Missing extra land play + attack/block restriction |
| arch_of_orazca | Partial | 0 | Partial | Activation restriction modeled as conditional (works but imprecise) |
| otawara_soaring_city | Partial | 2 | Partial | Target filter too broad, cost reduction missing |
| boseiju_who_endures | Partial | 2 | Partial | Target filter too broad, cost reduction missing |
| sokenzan_crucible_of_defiance | Partial | 2 | **No** | Tokens missing haste, cost reduction missing |
| takenuma_abandoned_mire | **No** | 2 | **No** | Only mills, missing graveyard return |
| eiganjo_seat_of_the_empire | Partial | 2 | Partial | Target filter should be attacking/blocking only |
| blinkmoth_nexus | Yes | 0 | Yes | Clean -- land animation via continuous effects |
| inkmoth_nexus | Yes | 0 | Yes | Clean -- land animation via continuous effects |
| crystal_barricade | Partial | 1 | Partial | Missing noncombat damage prevention |
| twilight_prophet | **No** | 1 | **No** | Missing upkeep drain trigger |
| serra_ascendant | **No** | 1 | **No** | Missing conditional static (+5/+5 + flying) |
| hammer_of_nazahn | **No** | 1 | **No** | Missing Equipment ETB auto-attach trigger |
| monster_manual | **No** | 0 (but empty) | **No** | Missing all abilities (activated + Adventure) |

## Deferred Sub-Batches Status

| Sub-batch | Status | Now Unblocked? | Notes |
|-----------|--------|---------------|-------|
| 13d: Equipment auto-attach | Deferred | Partially | Hammer of Nazahn needs it; triggered ability watching Equipment ETB exists in DSL |
| 13h: Coin flip / d20 | Deferred | No | No randomness primitives in engine |
| 13i: Timing restriction | Done | Yes | TimingRestriction::SorcerySpeed exists on AbilityDefinition::Activated; used by Equip abilities |
| 13j: Clone / copy ETB | Deferred | No | Requires copy.rs extensions for ETB choice |
| 13l: Flicker (exile + return) | Deferred | Partially | MoveZone to Exile + MoveZone back exists; no atomic flicker effect yet |
| 13m: Adventure | Deferred | No | Requires AltCostKind::Adventure + split-card exile casting |
| 13n: Living Weapon | Done | Yes | Fully implemented with CreateTokenAndAttachSource |

## Previous Findings (first review -- no previous)

N/A -- this is the initial retroactive review.
