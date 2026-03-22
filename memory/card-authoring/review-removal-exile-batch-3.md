# Card Review: Removal/Exile Batch 3

**Reviewed**: 2026-03-22
**Cards**: 3
**Findings**: 1 HIGH, 3 MEDIUM, 2 LOW

---

## Card 1: Teysa, Orzhov Scion

- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Human Advisor)
- **Mana cost match**: YES ({1}{W}{B})
- **P/T match**: YES (2/3)
- **DSL correctness**: N/A (abilities empty per W5 policy)
- **Findings**:
  - F1 (LOW): TODO claims "sacrifice N permanents of a given type" not expressible -- this is correct, `Cost::Sacrifice(TargetFilter)` only sacrifices one permanent, not three with a color filter. TODO is valid.
  - F2 (LOW): TODO claims WheneverCreatureDies is overbroad for "another black creature you control" -- this is correct, no color filter exists on the trigger condition. TODO is valid.
  - **Verdict**: Clean. W5 policy correctly applied -- both abilities have genuine DSL gaps.

---

## Card 2: Shiko, Paragon of the Way

- **Oracle match**: YES (includes reminder text for copy)
- **Types match**: YES (Legendary Creature -- Spirit Dragon)
- **Mana cost match**: YES ({2}{U}{R}{W})
- **P/T match**: YES (4/5)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F3 (HIGH / KI-2 / W5): **Partial implementation produces wrong game state.** The ETB trigger currently exiles the target card from the graveyard but does NOT copy-and-cast it. Oracle says "exile target nonland card ... Copy it, then you may cast the copy without paying its mana cost." The implemented effect only exiles the card, which is a real downside (removes it from graveyard) without providing the intended benefit (free cast). This is strictly worse than the card should be -- the opponent loses a card from their graveyard for no upside to the controller. Per W5 policy, a partial implementation that produces wrong game state should use `abilities: vec![]` (keeping only the keywords). The exile-only trigger should be removed or the entire triggered ability should be left as a TODO with empty effect.
  - F4 (MEDIUM): The `TargetFilter { non_land: true, max_cmc: Some(3) }` on `TargetCardInYourGraveyard` is a reasonable filter, but oracle says "nonland card" not "nonland permanent." The `non_land` field was designed for permanents. Verify that `non_land` on graveyard targeting correctly excludes land cards (it likely does since land is a card type, but the field name is ambiguous). Minor correctness concern.

---

## Card 3: Teferi, Hero of Dominaria

- **Oracle match**: YES (uses Unicode minus \u{2212} correctly)
- **Types match**: YES (Legendary Planeswalker -- Teferi)
- **Mana cost match**: YES ({3}{W}{U})
- **Starting loyalty**: YES (4)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F5 (MEDIUM / KI-2 / W5): **+1 loyalty ability is partial -- draws a card but does not untap lands.** Oracle: "Draw a card. At the beginning of the next end step, untap up to two lands." The TODO correctly notes that delayed triggers are not in the DSL. However, the +1 currently gives the player a card draw for only +1 loyalty, which is already quite powerful and close to correct value. The missing land untap is upside the player loses. This is borderline W5 -- the player gets the card but not the tempo. Flagging as MEDIUM rather than HIGH because the partial effect (draw a card) is never harmful to the controller, just incomplete. Compare to pain lands (free colored mana without damage = actively wrong). Still, strict W5 interpretation would make this `vec![]`.
  - F6 (MEDIUM / W5): **-3 loyalty ability uses `LibraryPosition::Top` instead of third from top.** Oracle: "Put target nonland permanent into its owner's library third from the top." The DSL lacks `NthFromTop(u32)`. Placing it on top is STRONGER than the card should be (opponent draws it sooner = worse for them). This is a wrong game state -- the removal is more punishing than intended. The TODO acknowledges this gap. Per strict W5, this ability should not be implemented with the wrong position.
  - F7 (MEDIUM): **-8 emblem has empty triggered_abilities.** The TODO correctly notes `WheneverYouDrawCard` trigger is not in the DSL. The emblem currently does nothing, which makes the -8 a pure loyalty sink. This is the correct W5 approach for a loyalty ability -- spending -8 for nothing is never advantageous, so it doesn't produce wrong game state (player would simply never use it).

---

## Summary

- **Cards with issues**: Shiko, Paragon of the Way (1 HIGH); Teferi, Hero of Dominaria (2 MEDIUM)
- **Clean cards**: Teysa, Orzhov Scion

### Required Fixes

| Priority | Card | Finding | Fix |
|----------|------|---------|-----|
| HIGH | Shiko, Paragon of the Way | F3: Exile-only ETB is wrong game state | Remove the ExileObject effect from the triggered ability (keep trigger shell as TODO, or move entire triggered ability to TODO with `vec![]` for non-keyword abilities) |
| MEDIUM | Teferi, Hero of Dominaria | F5: +1 draw-only is partial | Borderline -- document as known partial. Strict W5 would empty all abilities. |
| MEDIUM | Teferi, Hero of Dominaria | F6: -3 puts on top instead of 3rd | Wrong game state (stronger removal than intended). Consider emptying the -3 effect. |
| MEDIUM | Teferi, Hero of Dominaria | F7: -8 emblem is empty | Correct W5 approach -- no fix needed. |
