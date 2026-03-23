# Card Review: A-20 Pump-Buff Batch 2

**Reviewed**: 2026-03-23
**Cards**: 6
**Findings**: 2 HIGH, 2 MEDIUM, 0 LOW

## Card 1: Captivating Vampire
- **Oracle match**: YES
- **Types match**: YES (Creature — Vampire)
- **Mana cost match**: YES ({1}{B}{B})
- **P/T match**: YES (2/2)
- **DSL correctness**: YES
- **Findings**:
  - Static +1/+1 to other Vampires: correct EffectFilter::OtherCreaturesYouControlWithSubtype, Layer 7c, ModifyBoth(1). Clean.
  - TODO for "Tap five untapped Vampires you control" activated ability: genuine gap. Cost::TapNCreaturesWithSubtype does not exist, and the effect needs SetController + AddSubtype. Valid TODO.
  - No W5 concern: the lord ability alone is correct; the missing activated ability is additional upside, not a wrong-state issue.

## Card 2: Rising of the Day
- **Oracle match**: YES
- **Types match**: YES (Enchantment)
- **Mana cost match**: YES ({2}{R})
- **DSL correctness**: YES
- **Findings**:
  - Static haste grant: correct EffectFilter::CreaturesYouControl, Layer 6, AddKeyword(Haste). Clean.
  - TODO for "Legendary creatures you control get +1/+0": genuine gap. No EffectFilter variant for creatures filtered by supertype (Legendary). Valid TODO.
  - No W5 concern: haste is the primary ability; missing +1/+0 to legendaries is an incomplete upside, not wrong game state.

## Card 3: Dwynen, Gilt-Leaf Daen
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature — Elf Warrior, Legendary supertype present)
- **Mana cost match**: YES ({2}{G}{G})
- **P/T match**: YES (3/4)
- **DSL correctness**: YES
- **Findings**:
  - Keyword Reach: correct.
  - Static +1/+1 to other Elves: correct EffectFilter::OtherCreaturesYouControlWithSubtype("Elf"), Layer 7c, ModifyBoth(1). Clean.
  - TODO for attack trigger with attacking-Elf count: genuine gap. EffectAmount lacks an AttackingCreatureCount or subtype-filtered variant. Valid TODO.
  - No W5 concern: existing abilities are correct; the missing trigger is life gain upside only.

## Card 4: Elesh Norn, Grand Cenobite
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature — Phyrexian Praetor, Legendary supertype present)
- **Mana cost match**: YES ({5}{W}{W})
- **P/T match**: YES (4/7)
- **DSL correctness**: YES (for what is implemented)
- **Findings**:
  - F1 (HIGH / KI-2 / W5): **Missing -2/-2 to opponents' creatures produces wrong game state.** Elesh Norn without the debuff is a fundamentally different card -- the debuff is what makes her a board-warping threat that kills opponent X/2s and shrinks everything else. A game with Elesh Norn giving only +2/+2 to your creatures but NOT shrinking opponents' creatures produces incorrect combat outcomes and incorrect SBA deaths. The card should use `abilities: vec![]` per W5 policy until EffectFilter::CreaturesOpponentsControl is added.
  - The TODO correctly identifies the gap (EffectFilter::CreaturesOpponentsControl does not exist) and correctly notes that AllCreatures would be wrong.
  - Vigilance keyword: correct.
  - Static +2/+2 to other creatures you control: correct filter/layer/modification.

## Card 5: Eldrazi Monument
- **Oracle match**: YES
- **Types match**: YES (Artifact)
- **Mana cost match**: YES ({5})
- **DSL correctness**: YES (for what is implemented)
- **Findings**:
  - F2 (HIGH / KI-2 / W5): **Missing upkeep sacrifice trigger produces wrong game state.** Eldrazi Monument gives +1/+1, flying, and indestructible to all your creatures with NO downside. The upkeep sacrifice is the balancing cost -- without it, the card is strictly better than intended and produces incorrect game states (no creature attrition, no self-sacrifice of the monument). The card should use `abilities: vec![]` per W5 policy until the upkeep sacrifice trigger can be expressed.
  - Static +1/+1: correct EffectFilter::CreaturesYouControl, Layer 7c, ModifyBoth(1).
  - Static flying grant: correct Layer 6, AddKeyword(Flying).
  - Static indestructible grant: correct Layer 6, AddKeyword(Indestructible).
  - TODO correctly identifies the gap: needs upkeep trigger with mandatory sacrifice choice.

## Card 6: Invigorate
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({2}{G})
- **DSL correctness**: YES
- **Findings**:
  - F3 (MEDIUM): TODO for alternative cost ("opponent gains 3 life") is a genuine gap. AltCostKind does not have a "pay life to opponent" variant with a condition check (controlling a Forest). Valid TODO.
  - Spell effect: correct. ApplyContinuousEffect with Layer 7c, ModifyBoth(4), DeclaredTarget{0}, UntilEndOfTurn. Target: TargetCreature. Clean.
  - No W5 concern: the card is castable for its normal mana cost and the +4/+4 effect is correct. The missing alt cost just means you can't cast it for free -- the card still does the right thing when cast normally.

## Card 7: Elesh Norn, Grand Cenobite (additional note)
- F4 (MEDIUM): The +2/+2 static ability uses `OtherCreaturesYouControl` which is correct ("Other creatures you control get +2/+2" -- excludes self). If the card is zeroed out per F1, this becomes moot.

## Summary

- **Cards with issues**:
  - Elesh Norn, Grand Cenobite: 1 HIGH (W5 wrong game state -- partial implementation gives buff without debuff)
  - Eldrazi Monument: 1 HIGH (W5 wrong game state -- all upside without sacrifice downside)
  - Invigorate: 1 MEDIUM (missing alt cost, but normal cast works correctly)
- **Clean cards**:
  - Captivating Vampire (valid TODO, existing abilities correct)
  - Rising of the Day (valid TODO, existing ability correct)
  - Dwynen, Gilt-Leaf Daen (valid TODO, existing abilities correct)

### Action Items
1. **Elesh Norn, Grand Cenobite**: Change `abilities` to `vec![]` with TODO explaining W5 policy -- partial implementation without -2/-2 debuff produces wrong game state.
2. **Eldrazi Monument**: Change `abilities` to `vec![]` with TODO explaining W5 policy -- all upside without sacrifice cost produces wrong game state.
3. All other TODOs are genuine DSL gaps -- no stale claims (KI-3 clean).
