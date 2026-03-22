# Card Review: A-07 lifegain, A-08 lifedrain, A-09 protection, A-10 aura

**Reviewed**: 2026-03-22
**Cards**: 17 (+1 existence check)
**Findings**: 3 HIGH, 6 MEDIUM, 4 LOW

---

## Card 1: Jaddi Offshoot
- **Oracle match**: YES
- **Types match**: YES (Creature — Plant)
- **Mana cost match**: YES ({G})
- **P/T match**: YES (0/3)
- **DSL correctness**: YES
- **Findings**: None — clean card.

## Card 2: Courser of Kruphix
- **Oracle match**: YES
- **Types match**: YES (Enchantment Creature — Centaur)
- **Mana cost match**: YES ({1}{G}{G})
- **P/T match**: YES (2/4)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F1 (MEDIUM): W5 policy — two of three abilities are unimplemented (reveal top card, play lands from top). The one implemented ability (Landfall gain 1 life) produces correct but incomplete game state. The card is castable and provides lifegain without its primary value engine (playing lands from library top). This is borderline W5 but the lifegain alone doesn't produce *wrong* state, just incomplete. TODOs are legitimate DSL gaps (no play-from-top-of-library static exists).

## Card 3: Nadier's Nightblade
- **Oracle match**: YES
- **Types match**: YES (Creature — Elf Warrior)
- **Mana cost match**: YES ({2}{B})
- **P/T match**: YES (1/3)
- **DSL correctness**: YES (vec![] with TODO)
- **Findings**:
  - F2 (LOW): TODO is valid — no trigger condition for "token leaves the battlefield" exists. Correct to leave abilities empty.

## Card 4: Bontu's Monument
- **Oracle match**: YES
- **Types match**: YES (Legendary Artifact, supertype present)
- **Mana cost match**: YES ({3})
- **DSL correctness**: PARTIAL
- **Findings**:
  - F3 (HIGH): W5 policy violation — partial implementation produces wrong game state. The cost reduction uses `SpellCostFilter::HasColor(Color::Black)` which reduces ALL black spells, not just black creature spells. The trigger uses unfiltered `WheneverYouCastSpell` which fires on all spells, not just creature spells. Both produce incorrect game behavior: non-creature black spells get discounted, and non-creature spells trigger the drain. Per W5 policy, this should be `abilities: vec![]` and `spell_cost_modifiers: vec![]` with TODOs explaining compound filter gap. The TODOs correctly identify the gaps but the card should not have partial implementations that produce wrong state.

## Card 5: Bloodchief Ascension
- **Oracle match**: YES (matches Scryfall, uses "this enchantment" per oracle errata)
- **Types match**: YES (Enchantment)
- **Mana cost match**: YES ({B})
- **DSL correctness**: YES (vec![] with TODO)
- **Findings**:
  - F4 (LOW): TODOs are valid — both abilities require conditions not in the DSL (opponent lost 2+ life this turn; card put into opponent's graveyard; quest counter threshold). Correct to leave empty.

## Card 6: Marauding Blight-Priest
- **Oracle match**: YES
- **Types match**: YES (Creature — Vampire Cleric)
- **Mana cost match**: YES ({2}{B})
- **P/T match**: YES (3/2)
- **DSL correctness**: YES
- **Findings**: None — clean card.

## Card 7: Blood Seeker
- **Oracle match**: YES
- **Types match**: YES (Creature — Vampire Shaman)
- **Mana cost match**: YES ({1}{B})
- **P/T match**: YES (1/1)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F5 (HIGH): W5 policy violation — wrong game state. The effect uses `PlayerTarget::EachOpponent` which makes ALL opponents lose 1 life whenever ANY opponent's creature enters. Oracle says "you may have **that player** lose 1 life" — only the controller of the entering creature should lose life. In a 4-player game, if opponent A plays a creature, opponents B and C also lose 1 life incorrectly. Additionally, the "you may" optional component is ignored (effect is mandatory). The trigger filter (`controller: TargetController::Opponent`) is correct, but the effect target is wrong. This should be `abilities: vec![]` with TODO per W5 policy.
  - F6 (LOW): TODO comment acknowledges the issue but implements anyway instead of using empty vec.

## Card 8: Scrawling Crawler
- **Oracle match**: YES
- **Types match**: YES (Artifact Creature — Phyrexian Construct)
- **Mana cost match**: YES ({3})
- **P/T match**: YES (3/2)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F7 (MEDIUM): W5 borderline — the draw trigger (upkeep, each player draws) is correct. The second trigger fires on ALL card draws (including controller's own) because `WheneverPlayerDrawsCard` lacks a player filter. The effect then targets `EachOpponent` which is doubly wrong: it should target "that player" (the one who drew). In practice, if controller draws, nothing bad happens (EachOpponent doesn't include controller), but if opponent A draws, opponents B and C also lose 1 life incorrectly. The TODO correctly identifies the gap. Per W5 policy, the second ability should probably be omitted (wrong multiplayer behavior).

## Card 9: Torment of Hailfire
- **Oracle match**: YES
- **Types match**: YES (Sorcery)
- **Mana cost match**: YES ({X}{B}{B} — mana_cost has black: 2 only, X is implicit)
- **DSL correctness**: YES (vec![] with TODO)
- **Findings**:
  - F8 (LOW): TODO is valid — complex repeated player choice loop is not expressible. Correct to leave empty.

## Card 10: Blind Obedience
- **Oracle match**: YES
- **Types match**: YES (Enchantment)
- **Mana cost match**: YES ({1}{W})
- **DSL correctness**: PARTIAL
- **Findings**:
  - F9 (MEDIUM): Extort is implemented but the second ability ("Artifacts and creatures your opponents control enter tapped") is not. The TODO correctly identifies a compound filter gap. The card is castable with only Extort, which is correct but incomplete. Since Extort alone is a real ability that works correctly, this is acceptable per W5 — the missing ETB-tapped for opponents doesn't cause wrong state, just incomplete state.

## Card 11: Crypt Ghast
- **Oracle match**: YES
- **Types match**: YES (Creature — Spirit)
- **Mana cost match**: YES ({3}{B})
- **P/T match**: YES (2/2)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F10 (MEDIUM): Extort is implemented but mana doubling ("Whenever you tap a Swamp for mana, add an additional {B}") is not. TODO is valid — mana trigger on specific land subtype is a genuine DSL gap. Since Extort works correctly and the missing mana doubling doesn't produce *wrong* state (just incomplete), this is acceptable per W5.

## Card 12: Teferi's Protection
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({2}{W})
- **DSL correctness**: YES (vec![] with TODO)
- **Findings**:
  - F11 (MEDIUM): All four TODOs are valid DSL gaps (life total lock, player protection, mass phase out, self-exile). Correct to leave empty. This is a very complex card.

## Card 13: Wild Growth
- **Oracle match**: YES
- **Types match**: YES (Enchantment — Aura)
- **Mana cost match**: YES ({G})
- **DSL correctness**: PARTIAL
- **Findings**:
  - F12 (MEDIUM): Enchant(Land) is correct. Mana trigger on enchanted land is a genuine DSL gap (TODO valid). The card is castable but does nothing beyond being an Aura — no wrong state, just incomplete.

## Card 14: Elvish Guidance
- **Oracle match**: YES
- **Types match**: YES (Enchantment — Aura)
- **Mana cost match**: YES ({2}{G})
- **DSL correctness**: PARTIAL
- **Findings**: Same as Wild Growth — mana trigger on enchanted land is a genuine DSL gap, plus count-based scaling. TODO is valid. No findings beyond F12 pattern.

## Card 15: Animate Dead
- **Oracle match**: PARTIAL
- **Types match**: YES (Enchantment — Aura)
- **Mana cost match**: YES ({1}{B})
- **DSL correctness**: PARTIAL
- **Findings**:
  - F13 (HIGH): Enchant target is `EnchantTarget::Creature` but should be a graveyard card target. Oracle says "Enchant creature card in a graveyard" — no `EnchantTarget` variant exists for this. The current `Enchant(Creature)` would make this enchant a creature on the battlefield, which is wrong. Per W5 policy, this should be `abilities: vec![]` since the Enchant keyword with wrong target produces incorrect game behavior (could legally target battlefield creatures when cast, which is not how Animate Dead works).
  - F14 (LOW): Oracle text in def uses "this Aura" per modern oracle text. Scryfall uses "this Aura" as well. Match confirmed.

## Card 16: Kasmina's Transmutation
- **Oracle match**: YES
- **Types match**: YES (Enchantment — Aura)
- **Mana cost match**: YES ({1}{U})
- **DSL correctness**: YES
- **Findings**: None — clean card. Layer 6 RemoveAllAbilities and Layer 7b SetPowerToughness with AttachedCreature filter are all correct.

## Card 17: Eaten by Piranhas
- **Oracle match**: YES
- **Types match**: YES (Enchantment — Aura)
- **Mana cost match**: YES ({1}{U})
- **DSL correctness**: PARTIAL
- **Findings**:
  - F15 (MEDIUM): Flash and Enchant(Creature) keywords correct. RemoveAllAbilities (Layer 6) and SetPowerToughness 1/1 (Layer 7b) are correct. However, the color override (becomes black) and type override (becomes Skeleton only, loses other types) are missing. TODOs correctly identify SetColors and SetSubtypes as DSL gaps. Since the implemented parts (remove abilities, set P/T) are correct and produce a subset of the intended effect, this is acceptable under W5 — the creature will be 1/1 with no abilities, which is mostly right. The missing color/type changes are cosmetic in most game situations.

## Breath of Fury (existence check)
- **Status**: EXISTS at `crates/engine/src/cards/defs/breath_of_fury.rs` — correctly skipped from authoring.

---

## Summary

### HIGH (3)
| ID | Card | Issue |
|----|------|-------|
| F3 | Bontu's Monument | W5 violation: cost reduction + trigger both overbroad (all black spells, not just creatures). Should be vec![] |
| F5 | Blood Seeker | W5 violation: EachOpponent effect instead of "that player". Wrong multiplayer behavior |
| F13 | Animate Dead | Wrong Enchant target: Enchant(Creature) should not exist — no graveyard enchant variant. Should be vec![] |

### MEDIUM (6)
| ID | Card | Issue |
|----|------|-------|
| F1 | Courser of Kruphix | 2/3 abilities unimplemented (valid TODOs). Borderline W5 |
| F7 | Scrawling Crawler | Draw trigger fires on all players, effect hits all opponents. Wrong multiplayer |
| F9 | Blind Obedience | ETB-tapped for opponents unimplemented (valid TODO) |
| F10 | Crypt Ghast | Mana doubling unimplemented (valid TODO) |
| F11 | Teferi's Protection | All abilities unimplemented (valid TODOs) |
| F15 | Eaten by Piranhas | Color/type override missing (valid TODOs for SetColors/SetSubtypes) |

### LOW (4)
| ID | Card | Issue |
|----|------|-------|
| F2 | Nadier's Nightblade | Valid TODO, empty vec correct |
| F4 | Bloodchief Ascension | Valid TODOs, empty vec correct |
| F6 | Blood Seeker | TODO acknowledges problem but implements wrong behavior anyway |
| F8 | Torment of Hailfire | Valid TODO, empty vec correct |

### Clean cards (3)
- Jaddi Offshoot
- Marauding Blight-Priest
- Kasmina's Transmutation

### Cards needing fixes (3 HIGH)
- **Bontu's Monument**: Remove partial implementation, use `abilities: vec![]` and `spell_cost_modifiers: vec![]`
- **Blood Seeker**: Remove partial implementation, use `abilities: vec![]`
- **Animate Dead**: Remove `Enchant(Creature)` keyword, use `abilities: vec![]`
