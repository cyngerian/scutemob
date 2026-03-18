# Primitive Batch Review: PB-10 -- Return From Zone Effects (Graveyard Targeting)

**Date**: 2026-03-18
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 115.1 (targeting), CR 608.2b (fizzle), CR 702.11b (hexproof scope), CR 400.7 (new object identity)
**Engine files reviewed**: `card_definition.rs` (TargetRequirement variants, TargetFilter::has_subtypes), `casting.rs` (validate_object_satisfies_requirement graveyard path, validate_target_protection), `effects/mod.rs` (matches_filter has_subtypes, MoveZone handler), `hash.rs` (TargetFilter/TargetRequirement hashing), `abilities.rs` (auto-targeting for triggers), `resolution.rs` (fizzle check), `state/mod.rs` (move_object_to_zone controller reset)
**Card defs reviewed**: 10 (bloodline_necromancer, buried_ruin, emeria_the_sky_ruin, hall_of_heliods_generosity, nullpriest_of_oblivion, reanimate, teneb_the_harvester, bladewing_the_risen, den_protector, grim_harvest) + 2 bonus (haven_of_the_spirit_dragon, connive.rs)

## Verdict: needs-fix

Two HIGH findings affect game correctness. (1) Hexproof/shroud/protection validation runs unconditionally on all object targets including graveyard cards, which would illegally block targeting of hexproof creatures in graveyards (CR 702.11b says hexproof applies to permanents only). (2) Reanimate and Teneb say "under your control" but `move_object_to_zone` always resets controller to owner, so reanimating an opponent's creature gives it back to the opponent. Several MEDIUM findings for remaining TODOs and a missing "permanent card" filter.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `casting.rs:5161-5167` | **Hexproof/shroud blocks graveyard targeting.** `validate_target_protection` runs for ALL object targets before requirement dispatch. A creature with Hexproof in a graveyard (characteristics preserved by `move_object_to_zone`) would be untargetable by reanimation spells. CR 702.11b: hexproof applies only to permanents on the battlefield. **Fix:** Gate the `validate_target_protection` call on `obj.zone == ZoneId::Battlefield` (or additionally `ZoneId::Stack` for shroud on spells). Graveyard/exile/hand cards should skip this check entirely. |
| 2 | **HIGH** | `state/mod.rs:384` | **"Under your control" not supported by MoveZone.** `move_object_to_zone` always sets `controller: old_object.owner`. Reanimate says "under your control" meaning the caster controls the reanimated creature. Targeting an opponent's graveyard creature correctly puts it on the battlefield but under the WRONG player's control. **Fix:** Add a `controller_override: Option<PlayerId>` parameter to `MoveZone` effect (or a `ZoneTarget::Battlefield` field), and use `ctx.controller` when set. Alternatively, add `Effect::MoveZoneUnderYourControl` variant. Update Reanimate and Teneb card defs to use it. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | **MEDIUM** | `bladewing_the_risen.rs` | **Missing "permanent card" filter.** Oracle says "target Dragon permanent card" but filter has no card type restriction. Would incorrectly allow targeting Dragon tribal instants/sorceries if they existed. **Fix:** Add `has_card_types: vec![CardType::Creature, CardType::Artifact, CardType::Enchantment, CardType::Land, CardType::Planeswalker]` to the TargetFilter, or add a `permanent_card: bool` field to TargetFilter. |
| 4 | **MEDIUM** | `emeria_the_sky_ruin.rs:27,35` | **Intervening-if condition TODO.** Oracle: "if you control seven or more Plains" is an intervening-if per CR 603.4. Currently fires unconditionally, which produces wrong game state (free reanimation every upkeep). The condition `Condition::YouControlNOrMorePermanentsWithSubtype` does not exist yet. **Fix:** Implement the condition or document this as a known DSL gap with a tracking reference. |
| 5 | **MEDIUM** | `teneb_the_harvester.rs:23-26` | **Optional mana payment TODO.** Oracle: "you may pay {2}{B}. If you do..." The trigger fires unconditionally (always returns a creature). Wrong game state: Teneb returns a creature for free on combat damage. **Fix:** Implement `Effect::PayManaOrElse` or `Cost` on triggered abilities, or document as known DSL gap. |
| 6 | **MEDIUM** | `reanimate.rs:17-18` | **Life loss TODO.** Oracle: "You lose life equal to its mana value." Missing entirely. Wrong game state: free reanimation with no life cost. **Fix:** Needs `EffectAmount::ManaValueOfTarget` or similar to express dynamic life loss. Document as known DSL gap. |
| 7 | **MEDIUM** | `connive.rs:14-20,60` | **Concoct half still uses Effect::Nothing.** The PB-10 primitive (TargetCardInYourGraveyard + MoveZone) now exists, so Concoct's "return a creature card from your graveyard to the battlefield" can be implemented as `Effect::Sequence([Surveil{3}, MoveZone{target: DeclaredTarget{0}, to: Battlefield}])` with a graveyard target. **Fix:** Update Concoct to use `TargetCardInYourGraveyard(TargetFilter { has_card_type: Some(Creature) })` and `Effect::Sequence`. |
| 8 | LOW | `bladewing_the_risen.rs:41-42` | **Activated ability TODO.** "{B}{R}: Dragon creatures get +1/+1 until end of turn" not implemented. Known DSL gap (creature-type-filtered temporary pump). |
| 9 | LOW | `den_protector.rs:22-23` | **Static evasion TODO.** "Creatures with power less than this creature's power can't block it" not implemented. Known DSL gap. |
| 10 | LOW | `haven_of_the_spirit_dragon.rs:7-8,40` | **"or Ugin planeswalker card" not targetable.** Target filter lacks name-or-type union support. Known DSL gap. |

### Finding Details

#### Finding 1: Hexproof/shroud blocks graveyard targeting

**Severity**: HIGH
**File**: `crates/engine/src/rules/casting.rs:5161-5167`
**CR Rule**: 702.11b -- "Hexproof on a permanent means 'This permanent can't be the target of spells or abilities your opponents control.'"
**Issue**: The `validate_target_protection` call at line 5162 runs unconditionally for all `Target::Object` cases, including objects in the graveyard. When a creature with Hexproof (e.g., Carnage Tyrant) dies and goes to the graveyard, `move_object_to_zone` preserves `characteristics.keywords` (line 383 in `state/mod.rs`). The Hexproof keyword remains in the graveyard object's characteristics. If an opponent tries to target that card with Reanimate (`TargetCardInGraveyard`), `validate_target_protection` fires first and rejects the target with "object has hexproof and cannot be targeted by opponents". Per CR 702.11b, hexproof only applies to permanents on the battlefield; cards in graveyards are not permanents. Same issue applies to Shroud (CR 702.18) and Protection (CR 702.16).
**Fix**: Gate the `validate_target_protection` call on `obj.zone == ZoneId::Battlefield`. Cards in graveyard, exile, hand, library, and command zone should not have hexproof/shroud/protection checked for targeting purposes. The graveyard targeting comment at line 5300 ("No hexproof/shroud check") is correct in intent but the check already ran 140 lines earlier.

#### Finding 2: "Under your control" not implemented

**Severity**: HIGH
**File**: `crates/engine/src/state/mod.rs:384`, `crates/engine/src/effects/mod.rs:1343`
**Oracle**: Reanimate: "Put target creature card from a graveyard onto the battlefield **under your control**." Teneb: "put target creature card from a graveyard onto the battlefield **under your control**."
**Issue**: `move_object_to_zone` always sets `controller: old_object.owner` (line 384). When Reanimate (cast by Player A) targets a creature in Player B's graveyard, the creature enters the battlefield with `controller = Player B` (the owner). Oracle text requires `controller = Player A` (the caster). This produces fundamentally wrong game state -- the opponent gets their creature back instead of the caster stealing it. This affects Reanimate, Teneb, and any future "under your control" reanimation effects.
**Fix**: Either (a) add a `controller_override: Option<PlayerId>` field to `Effect::MoveZone` and apply it after `move_object_to_zone`, or (b) add a new `ZoneTarget::BattlefieldUnderYourControl` variant, or (c) follow `MoveZone` with an `Effect::SetController` effect in a Sequence. Option (a) is cleanest. Update `effects/mod.rs` MoveZone handler to set `new_obj.controller = override` after the move. Update Reanimate and Teneb card defs.

#### Finding 3: Missing "permanent card" filter on Bladewing

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/bladewing_the_risen.rs:36-39`
**Oracle**: "you may return target Dragon **permanent card** from your graveyard to the battlefield"
**Issue**: The TargetFilter only specifies `has_subtypes: vec![Dragon]` without any card type restriction. "Permanent card" means a card with a permanent type (creature, artifact, enchantment, land, planeswalker). Without this filter, the targeting would also accept Dragon tribal instants/sorceries in the graveyard. While no Dragon instants/sorceries currently exist in Standard, tribal Dragon cards exist in older formats and could appear in Commander.
**Fix**: Add `has_card_types: vec![CardType::Creature, CardType::Artifact, CardType::Enchantment, CardType::Land, CardType::Planeswalker]` to the TargetFilter. Or consider adding a `permanent_card: bool` convenience field to TargetFilter that checks the same set.

#### Finding 7: Connive // Concoct not updated

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/connive.rs:14-20,60-66`
**Oracle**: Concoct: "Surveil 3, then return a creature card from your graveyard to the battlefield."
**Issue**: The TODO at line 14 says the return-creature effect needs PB-10 primitives. PB-10 is now complete (TargetCardInYourGraveyard + MoveZone to Battlefield works). The Concoct half still uses `Effect::Surveil` alone with `targets: vec![]` and no MoveZone. This card was not listed in PB-10's affected cards but could have been fixed.
**Fix**: Update Concoct to use `targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() })]` and `effect: Effect::Sequence(vec![Effect::Surveil { player: PlayerTarget::Controller, count: EffectAmount::Fixed(3) }, Effect::MoveZone { target: EffectTarget::DeclaredTarget { index: 0 }, to: ZoneTarget::Battlefield { tapped: false } }])`.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 115.1 (targeting declarations) | Yes | Yes | test_115_1_target_creature_in_your_graveyard_valid, + 4 more |
| CR 608.2b (fizzle on illegal target) | Yes | Yes | test_608_2b_fizzle_gy_target_exiled_before_resolution |
| CR 400.7 (new object identity) | Yes | Yes | test_115_1_resolve_return_creature_from_gy_to_battlefield checks new ID |
| CR 702.11b (hexproof scope) | **NO** | No | Finding 1 -- hexproof check runs on graveyard targets |
| "under your control" | **NO** | No | Finding 2 -- controller not overridden for any-GY reanimation |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Bloodline Necromancer | Yes | 0 | Yes | Correct: your GY, Vampire or Wizard creature, lifelink |
| Buried Ruin | Yes | 0 | Yes | Correct: sacrifice + tap cost, artifact from your GY to hand |
| Emeria, the Sky Ruin | Partial | 2 | No | Missing intervening-if (fires unconditionally) |
| Hall of Heliod's Generosity | Yes | 0 | Yes | Correct: enchantment from your GY to top of library |
| Nullpriest of Oblivion | Yes | 0 | Yes | Correct: kicked intervening-if, creature from your GY |
| Reanimate | Partial | 1 | No | Missing life loss + "under your control" (Finding 2) |
| Teneb, the Harvester | Partial | 1 | No | Missing optional mana payment + "under your control" (Finding 2) |
| Bladewing the Risen | Partial | 1 | Partial | Missing "permanent card" filter + activated ability TODO |
| Den Protector | Partial | 1 | Yes | Graveyard targeting correct; static evasion TODO is unrelated |
| Grim Harvest | Yes | 0 | Yes | Correct: creature from your GY to hand, Recover keyword |
| Haven of the Spirit Dragon | Partial | 2 | Partial | Missing "Ugin planeswalker card" union target |
| Connive // Concoct | No | 2 | No | Concoct half still Effect::Nothing; could use PB-10 now |

## Test Coverage

The test file `graveyard_targeting.rs` has 9 tests covering:
- Positive: valid creature target in your GY (cast + resolve)
- Negative: opponent's GY rejected by TargetCardInYourGraveyard
- Positive: opponent's GY accepted by TargetCardInGraveyard
- Negative: wrong card type rejected by filter
- Negative: battlefield card rejected by GY targeting
- Positive/Negative: has_subtypes OR-filter (Wizard matches, Goblin rejected)
- Positive: artifact to hand (Buried Ruin pattern)
- Fizzle: target exiled before resolution (CR 608.2b)

**Missing test coverage:**
- No test for hexproof creature in graveyard being targetable (would catch Finding 1)
- No test for "under your control" when targeting opponent's GY creature (would catch Finding 2)
- No card integration test using actual Reanimate/Bloodline Necromancer card defs
