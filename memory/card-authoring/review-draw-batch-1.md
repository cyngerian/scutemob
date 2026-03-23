# Card Review: A-18 Draw Sessions S20-S21

**Reviewed**: 2026-03-22
**Cards**: 24
**Findings**: 7 HIGH, 8 MEDIUM, 3 LOW

---

## Card 1: Unearth
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): TODO for mana-value-3-or-less filter on graveyard target is a genuine DSL gap. Acceptable approximation.

## Card 2: Teferi, Time Raveler
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F2 (HIGH / KI-1): -3 ability uses `TargetRequirement::TargetPermanent` but oracle says "artifact, creature, or enchantment". Should use `TargetPermanentWithFilter` excluding lands and planeswalkers, or a filter for `has_card_type` matching artifact/creature/enchantment. Currently can target any permanent including lands and planeswalkers -- wrong targeting.
  - F3 (LOW): +1 as `Effect::Nothing` is acceptable placeholder for timing-restriction grant.
  - F4 (LOW): Static "opponents can only cast at sorcery speed" correctly left as TODO.
  - Note: -3 return to owner's hand uses `OwnerOf` -- CORRECT for multiplayer.

## Card 3: Clavileno, First of the Blessed
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (empty abilities with TODOs)
- **Findings**: Clean. TODOs for attack trigger + type granting are genuine DSL gaps.

## Card 4: Welcoming Vampire
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F5 (HIGH / KI-2 / W5): Overbroad trigger fires on ALL creatures you control entering, not just "power 2 or less". Also missing "once each turn" limit. Draws extra cards in normal gameplay. Per W5 policy, should be `vec![]` with TODO or at minimum the TODO must explicitly state the trigger is overbroad. The current TODO mentions the gaps but the implementation is still present and will produce wrong game state.

## Card 5: Ezuri, Stalker of Spheres
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F6 (HIGH / KI-2 / W5): ETB unconditionally proliferates twice. Oracle says "you MAY pay {3}. If you do, proliferate twice." Skipping the {3} payment entirely means the card is strictly better than intended -- free proliferate on ETB. Per W5 policy, should be `vec![]` with TODO.
  - F7 (MEDIUM): "Whenever you proliferate, draw a card" correctly left as TODO (genuine DSL gap).

## Card 6: Serum Visions
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: Clean. Fully implemented.

## Card 7: Up the Beanstalk
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F8 (HIGH / KI-2 / W5): Cast trigger fires on ALL spells, not just "mana value 5 or greater". Draws a card on every spell cast. ETB half is correct. The cast trigger half should be removed (use TODO only) or the entire abilities vec should note the overbroad trigger. Producing extra draws on cheap spells is wrong game state.

## Card 8: Uro, Titan of Nature's Wrath
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F9 (HIGH / KI-2 / W5): Missing "sacrifice unless escaped" ETB trigger. When cast normally (not via Escape), Uro should sacrifice itself on ETB. Without this, hardcasting Uro gives a 6/6 that stays on the battlefield -- massively wrong game state. Should be `vec![]` with TODO or the sacrifice-unless-escaped must be implemented.
  - F10 (MEDIUM): "May put a land from hand onto battlefield" correctly noted as TODO (genuine gap).
  - F11 (MEDIUM): Escape cost exiles 5 other cards from graveyard -- the `AltCastAbility` with `AltCostKind::Escape` is present but the exile-5-cards detail is engine-level (not card-def). Acceptable.

## Card 9: Sarkhan, Fireblood
- **Oracle match**: YES (but see below)
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F12 (HIGH / KI-2 / W5): +1 rummage ability draws unconditionally without discarding. Oracle: "You may discard a card. If you do, draw a card." The implementation gives a free draw every turn with no discard cost -- wrong game state. Should be `Effect::Nothing` with TODO or implement the discard-then-draw sequence.
  - F13 (MEDIUM): +1 mana ability adds 2 red mana. Oracle says "two mana in any combination of colors" with Dragon-only restriction. Hardcoded to red is an approximation -- TODO documents the color-choice gap but the Dragon restriction is also missing. Acceptable as approximation if TODO is expanded.
  - F14 (MEDIUM): -7 token creation looks correct -- 4x 5/5 red Dragon with flying. Clean.

## Card 10: Tormenting Voice
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F15 (MEDIUM): Additional cost "discard a card" is noted as TODO but the spell effect (draw 2) is implemented. Since the discard is an additional COST (not part of the effect), the card is castable without discarding -- slightly wrong but the draw-2 effect is correct. This is a casting-cost gap, not an effect gap. Acceptable with the existing TODO.

## Card 11: Curiosity
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (empty trigger with TODO)
- **Findings**: Clean. Enchant keyword present. Damage trigger correctly left as TODO.

## Card 12: Moon-Circuit Hacker
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F16 (MEDIUM): Combat damage trigger draws unconditionally. Oracle says "you may draw a card. If you do, discard a card unless this creature entered this turn." The conditional discard is simplified away -- TODO should document this. The draw is optional ("may") but implemented as mandatory. Minor wrong game state but generally acceptable for draw-focused cards.

## Card 13: Contaminant Grafter
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F17 (MEDIUM): Corrupted end-step trigger correctly uses `intervening_if: Some(Condition::OpponentHasPoisonCounters(3))`. Good.
  - F18 (MEDIUM): "May put a land from hand onto battlefield" correctly noted as TODO gap.
  - F19 (MEDIUM): Combat damage proliferate trigger correctly noted as TODO gap.

## Card 14: Tamiyo, Field Researcher
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F20 (MEDIUM): -7 draws 3 cards but missing emblem creation. DSL supports `Effect::CreateEmblem` (PB-22 S6). The emblem effect ("cast spells without paying mana costs") is complex but the draw-3 alone is a partial implementation. TODO documents the gap. Acceptable since the emblem itself would need a complex continuous effect.

## Card 15: Leaf-Crowned Visionary
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F21 (HIGH / KI-2 / W5): Cast trigger fires on ALL spells you cast, not just "Elf spells". Also skips the "may pay {G}" optional cost. Draws a card on every spell cast without payment -- significantly wrong game state. The static +1/+1 to Elves is correct. The triggered ability should be removed (leave TODO only).

## Card 16: Baral, Chief of Compliance
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: Clean. Cost reduction via `SpellCostModifier` is correct. Counter-trigger correctly left as TODO (genuine gap).

## Card 17: Priest of Forgotten Gods
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F22 (HIGH / KI-2 / W5): Uses `Cost::SacrificeSelf` but oracle says "Sacrifice two OTHER creatures." The card sacrifices itself instead of two other creatures -- completely wrong activation cost. Also missing the "target players each lose 2 life and sacrifice a creature" effect. Currently gives free {B}{B} + draw for sacrificing itself. Should be `vec![]` with TODO.

## Card 18: Elvish Visionary
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: Clean. Fully implemented. Simple ETB draw.

## Card 19: Infectious Inquiry
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: Clean. Fully implemented -- draw 2, lose 2, poison counter to each opponent.

## Card 20: Two-Headed Hellkite
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: Clean. Fully implemented -- flying, menace, haste, attack-trigger draw 2.

## Card 21: Satoru, the Infiltrator
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F23 (MEDIUM): Trigger fires on ALL creatures you control entering. Oracle has complex condition: "if none of them were cast or no mana was spent to cast them." TODO acknowledges the gap but the overbroad trigger draws cards on every creature ETB including normally-cast creatures. This is a borderline W5 case -- the trigger condition is genuinely inexpressible but the approximation is quite overbroad. Consider `vec![]`.

## Card 22: Spectral Sailor
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: Clean. Fully implemented -- Flash, Flying, {3}{U} activated draw.

## Card 23: Crossway Troublemakers
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F24 (MEDIUM / KI-9): `WheneverCreatureDies` triggers on ALL creature deaths, not just "a Vampire you control." Also missing "pay 2 life" cost for the draw. TODO documents the subtype filter gap but the trigger is overbroad. Consider `vec![]`.
  - F25 (MEDIUM): Static deathtouch+lifelink grant to attacking Vampires correctly noted as TODO gap (conditional static grant while attacking).

## Card 24: Baron Bertram Graywater
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F26 (MEDIUM): Token-enter trigger uses `WheneverCreatureEntersBattlefield` with `controller: You` filter. Oracle says "one or more tokens you control enter" -- should only trigger on tokens, not all creatures. Also missing "once each turn" limit. Overbroad -- creates infinite token loops (every creature ETB makes a token, which triggers again). Consider `vec![]`.
  - F27 (MEDIUM): Activated ability uses `Cost::Sacrifice(TargetFilter::default())` -- oracle says "Sacrifice another creature or artifact." Default filter may not restrict to creature-or-artifact, and "another" should exclude self. Check if default TargetFilter allows any permanent sacrifice.

---

## Summary

### Cards with HIGH findings (7):
1. **Welcoming Vampire** (F5) -- overbroad trigger, wrong game state
2. **Ezuri, Stalker of Spheres** (F6) -- free proliferate without {3} payment
3. **Up the Beanstalk** (F8) -- cast trigger fires on all spells
4. **Uro, Titan of Nature's Wrath** (F9) -- missing sacrifice-unless-escaped
5. **Sarkhan, Fireblood** (F12) -- free draw without discard cost
6. **Leaf-Crowned Visionary** (F21) -- cast trigger fires on all spells
7. **Priest of Forgotten Gods** (F22) -- SacrificeSelf instead of sacrifice-two-others

### Cards with MEDIUM-only findings (8):
- Teferi, Time Raveler (F2 is HIGH -- reclassifying)
- Tormenting Voice, Moon-Circuit Hacker, Contaminant Grafter, Tamiyo Field Researcher, Satoru the Infiltrator, Crossway Troublemakers, Baron Bertram Graywater

**Correction**: Teferi F2 is HIGH (wrong targeting). Updated counts:

**Final Counts**: 8 HIGH, 8 MEDIUM, 3 LOW

### Cards with HIGH findings (8):
1. **Teferi, Time Raveler** (F2) -- TargetPermanent without artifact/creature/enchantment filter
2. **Welcoming Vampire** (F5) -- overbroad trigger
3. **Ezuri, Stalker of Spheres** (F6) -- free proliferate without payment
4. **Up the Beanstalk** (F8) -- cast trigger fires on all spells
5. **Uro, Titan of Nature's Wrath** (F9) -- missing sacrifice-unless-escaped
6. **Sarkhan, Fireblood** (F12) -- free draw without discard
7. **Leaf-Crowned Visionary** (F21) -- cast trigger fires on all spells
8. **Priest of Forgotten Gods** (F22) -- SacrificeSelf instead of sacrifice-two-others

### Clean cards (8):
- Unearth (LOW only)
- Serum Visions
- Clavileno, First of the Blessed
- Curiosity
- Baral, Chief of Compliance
- Elvish Visionary
- Infectious Inquiry
- Two-Headed Hellkite
- Spectral Sailor

### Recommended fixes:
All HIGH findings are W5 violations (wrong game state). The fix for each is:
- **Teferi**: Add `TargetPermanentWithFilter` with appropriate filter for artifact/creature/enchantment
- **Welcoming Vampire, Up the Beanstalk, Leaf-Crowned Visionary, Satoru**: Remove overbroad triggered abilities, leave as `vec![]` with TODO (or keep only the correctly-implementable portions)
- **Ezuri**: Remove unconditional proliferate ETB, leave as TODO
- **Uro**: Either implement sacrifice-unless-escaped or use `vec![]`
- **Sarkhan**: Change +1 to `Effect::Nothing` with TODO
- **Priest of Forgotten Gods**: Change to `vec![]` with TODO (Cost::SacrificeSelf is fundamentally wrong)
