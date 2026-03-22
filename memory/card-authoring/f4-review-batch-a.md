# Card Review: F-4 Session 1 Batch A

**Reviewed**: 2026-03-22
**Cards**: 6
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Castle Ardenvale
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None -- land)
- **DSL correctness**: YES
- **Findings**: None

ETB-tapped-unless pattern uses `EntersTapped` + `unless_condition: Some(Condition::ControlLandWithSubtypes(vec![SubType("Plains")]))`
which matches the reference implementation (Arena of Glory). Mana ability adds white (WUBRG order
correct: `mana_pool(1, 0, 0, 0, 0, 0)`). Token ability cost is `{2}{W}{W}` = generic 2, white 2 --
correct. Token is 1/1 white Human creature -- all fields correct.

## Card 2: Castle Embereth
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None -- land)
- **DSL correctness**: YES
- **Findings**: None

ETB-tapped-unless correctly checks Mountain subtype. Mana ability adds red (`mana_pool(0, 0, 0, 1, 0, 0)`
-- correct WUBRGC order). Pump ability cost `{1}{R}{R}` = generic 1, red 2 -- correct.
`ApplyContinuousEffect` with `ModifyPower(1)`, `CreaturesYouControl` filter, `UntilEndOfTurn` duration
correctly models "+1/+0 until end of turn to creatures you control". Layer is `PtModify` (Layer 7c) --
correct for P/T modification.

## Card 3: Castle Locthwain
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None -- land)
- **DSL correctness**: YES
- **Findings**: None

ETB-tapped-unless correctly checks Swamp subtype. Mana ability adds black (`mana_pool(0, 0, 1, 0, 0, 0)`
-- correct). Draw ability cost `{1}{B}{B}` = generic 1, black 2 -- correct. Effect is
`Sequence([DrawCards(1), LoseLife(CardCount of hand)])` which correctly models "Draw a card, then you
lose life equal to the number of cards in your hand." The `CardCount` checks
`ZoneTarget::Hand { owner: PlayerTarget::Controller }` which is correct -- it counts cards in your hand
(including the card just drawn).

## Card 4: Castle Vantress
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None -- land)
- **DSL correctness**: YES
- **Findings**: None

ETB-tapped-unless correctly checks Island subtype. Mana ability adds blue (`mana_pool(0, 1, 0, 0, 0, 0)`
-- correct). Scry ability cost `{2}{U}{U}` = generic 2, blue 2 -- correct. `Effect::Scry` with count
`Fixed(2)` and `PlayerTarget::Controller` is correct.

## Card 5: Karn's Bastion
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (None -- land)
- **DSL correctness**: YES
- **Findings**: None

No ETB-tapped condition -- correct (oracle has none). Colorless mana ability via
`mana_pool(0, 0, 0, 0, 0, 1)` -- correct. Proliferate ability cost `{4}, {T}` = generic 4 + tap --
correct. `Effect::Proliferate` is the correct DSL variant. No targets needed (Proliferate is a choice,
not a target).

## Card 6: Kher Keep
- **Oracle match**: YES
- **Types match**: YES (Legendary Land -- `supertypes(&[SuperType::Legendary], &[CardType::Land])`)
- **Mana cost match**: YES (None -- land)
- **DSL correctness**: YES
- **Findings**: None

Legendary supertype correctly present (KI-4 check passes). No ETB-tapped condition -- correct.
Colorless mana ability correct. Token ability cost `{1}{R}, {T}` = generic 1, red 1 + tap -- correct.
Token is "0/1 red Kobold creature token named Kobolds of Kher Keep" -- name, P/T, color, creature type,
and subtype all correct.

## Summary
- Cards with issues: (none)
- Clean cards: Castle Ardenvale, Castle Embereth, Castle Locthwain, Castle Vantress, Karn's Bastion, Kher Keep

All 6 cards are correctly implemented with accurate oracle text, proper mana costs, correct type lines
(including Legendary supertype on Kher Keep), correct ETB-tapped-unless conditions on all four Castle
lands, and appropriate DSL patterns for their activated abilities. No stale TODOs found.
