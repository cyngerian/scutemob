# Card Review: Counter Batch 3

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 2 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Keep Safe
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-2): W5 wrong game state. The card uses `TargetRequirement::TargetSpell` which allows countering ANY spell, but oracle says "target spell that targets a permanent you control." This is overbroad — the card can counter spells it should not be able to target (e.g., a Wrath of God, a spell targeting an opponent's creature). The DSL has `TargetSpellWithFilter` but lacks a "targets a permanent you control" sub-filter, so the TODO is valid regarding the DSL gap. However, the current implementation should use `abilities: vec![]` per W5 policy because it produces wrong game state (illegal targeting). A counterspell that can target the wrong spells is functionally incorrect.
  - F2 (LOW): TODO correctly identifies the DSL gap — no "spell that targets a permanent you control" filter exists on `TargetFilter`. Valid TODO.

## Card 2: Memory Lapse
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM / KI-2): Partial implementation — counters to graveyard instead of top of library. The TODO correctly identifies that `CounterSpell` always sends to graveyard (CR 701.5a default). This is arguably wrong game state: putting a card in the graveyard vs. top of library is a meaningful difference (graveyard access vs. redraw). However, the targeting is correct (any spell), so the counter effect itself is valid — it's the destination that's wrong. Borderline W5. The card is still "counter target spell" with correct targeting; the wrong-destination is a secondary effect. Flagging as MEDIUM rather than HIGH because the card's primary function (countering) works correctly, and the wrong destination doesn't give the caster an advantage they shouldn't have.
  - F2 (LOW): TODO is valid — `CounterSpell` has no destination parameter.

## Card 3: Mental Misstep
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (Phyrexian mana correctly encoded as `PhyrexianMana::Single(ManaColor::Blue)`)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-3): Stale TODO. The TODO claims "requires mana-value-filtered TargetSpell. TargetSpell currently has no MV filter." This is FALSE. The DSL has `TargetRequirement::TargetSpellWithFilter(TargetFilter)` and `TargetFilter` has `max_cmc: Option<u32>` and `min_cmc: Option<u32>`. The correct implementation is: `targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter { max_cmc: Some(1), min_cmc: Some(1), ..Default::default() })]`. Without this filter, the card can counter spells of any mana value, which is wrong game state (KI-2 as well).

## Card 4: Flare of Denial
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES ({1}{U}{U} = generic 1, blue 2)
- **DSL correctness**: YES (with valid TODO)
- **Findings**:
  - No issues. The counterspell effect is correctly implemented (counter target spell, full stop). The TODO about the pitch-sacrifice alt cost ("sacrifice a nontoken blue creature rather than pay this spell's mana cost") is valid — `AltCostKind` has no pitch-sacrifice variant with color+nontoken filtering. The card is still functional at its regular mana cost; missing the alt cost means it's less powerful but not wrong (you can still cast it for {1}{U}{U} and it does the right thing).

## Card 5: Rewind
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES ({2}{U}{U} = generic 2, blue 2)
- **DSL correctness**: NO
- **Findings**:
  - F1 (LOW): Missing "Untap up to four lands" effect. The TODO is valid — this requires targeting up to 4 lands and untapping them, and "up to" choice is not trivially expressible. However, this is NOT wrong game state per W5: the card still costs {2}{U}{U} and counters a spell. Missing the untap is missing upside (the caster gets less benefit than they should), which does not produce incorrect game behavior — it just makes the card weaker. The counter effect itself is correct.

## Summary
- **Cards with issues**: Keep Safe (1 HIGH), Mental Misstep (1 HIGH), Memory Lapse (1 MEDIUM)
- **Clean cards**: Flare of Denial, Rewind

### Action Items
1. **Mental Misstep** (HIGH, easy fix): Replace `TargetRequirement::TargetSpell` with `TargetRequirement::TargetSpellWithFilter(TargetFilter { max_cmc: Some(1), min_cmc: Some(1), ..Default::default() })`. Remove stale TODO.
2. **Keep Safe** (HIGH): Should use `abilities: vec![]` per W5 policy until DSL supports "spell that targets a permanent you control" filter. Current implementation allows illegal targeting.
3. **Memory Lapse** (MEDIUM): Counter-to-top-of-library is a DSL gap. Current implementation is borderline W5 — consider `abilities: vec![]` if policy is strict, or keep with TODO if counter-to-graveyard is acceptable as approximation.
