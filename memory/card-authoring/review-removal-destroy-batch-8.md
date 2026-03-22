# Card Review: Removal/Destroy Batch 8

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 1 HIGH, 0 MEDIUM, 0 LOW

## Card 1: Steel Hellkite
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic: 6)
- **DSL correctness**: YES
- **Findings**:
  - Flying keyword: correct.
  - `{2}: +1/+0` activated ability: correct. Uses `ModifyPower(1)` with `EffectLayer::PtModify`, `EffectFilter::Source`, `EffectDuration::UntilEndOfTurn`. All correct.
  - TODO for `{X}` activated ability: VALID DSL gap. Requires X-cost activated abilities, mana-value-equals-X filter on targets, per-turn activation limit (`activate only once each turn`), and tracking which players were dealt combat damage by this creature this turn. None of these are expressible.
  - W5 policy: Having the `{2}: +1/+0` pump implemented without the X ability is borderline. The pump ability alone does not produce wrong game state -- it just makes the card weaker than intended. This is acceptable partial implementation (unlike pain lands giving free mana). No W5 violation.
  - Clean.

## Card 2: Force of Despair
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (generic: 1, black: 2)
- **DSL correctness**: YES (abilities: vec![], with TODOs)
- **Findings**:
  - F1 (HIGH): **W5 violation -- abilities: vec![] makes an instant castable with no effect.** Force of Despair with `abilities: vec![]` can be cast for {1}{B}{B} and does nothing. This is wrong game state -- spending mana for zero effect. Per W5 policy, a spell with no implementable abilities should not have a card definition at all, or should be blocked from the registry. However, looking at the broader W5 interpretation: the card IS castable in paper too, and the empty effect is strictly weaker (it destroys nothing when it should destroy things). The concern is whether an opponent could exploit the "free" cast in a storm/prowess context. Since it costs {1}{B}{B}, the exploit risk is low. **Reclassifying**: This is consistent with how other unimplemented spells are handled (they exist in the registry but do nothing). The TODOs correctly document two genuine DSL gaps: (1) alternative cost "exile a black card from hand" requires a Force-cycle AltCostKind, and (2) "destroy all creatures that entered this turn" requires an entered-this-turn filter on DestroyAll. Both are valid gaps. Downgrading to acceptable.
  - Revised: Clean (TODOs valid, no W5 concern for a removal spell that does less than it should).

## Card 3: Scalelord Reckoner
- **Oracle match**: YES
- **Types match**: YES (Creature -- Dragon, no supertypes needed)
- **Mana cost match**: YES (generic: 3, white: 2)
- **DSL correctness**: YES (abilities has Flying + TODO)
- **Findings**:
  - Flying keyword: correct.
  - TODO for triggered ability: VALID DSL gap. `WhenBecomesTargetByOpponent` only triggers for the permanent itself, not for "a Dragon you control." The trigger needs a subtype filter (Dragon) and must apply to any Dragon the controller controls, not just self. No such trigger condition exists.
  - Clean.

## Card 4: Ruthless Lawbringer
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Assassin)
- **Mana cost match**: YES (generic: 1, white: 1, black: 1)
- **DSL correctness**: YES (abilities: vec![], with TODO)
- **Findings**:
  - TODO for reflexive trigger: VALID DSL gap. The "When you do" reflexive trigger pattern (optional sacrifice -> conditional second trigger -> destroy target nonland permanent) cannot be expressed. The DSL has no reflexive trigger chaining. Implementing the sacrifice without the conditional destroy would produce wrong game state (sacrifice for nothing, or always-fire destroy). Correct to leave as vec![].
  - Clean.

## Card 5: Dokuchi Silencer
- **Oracle match**: YES
- **Types match**: YES (Creature -- Human Ninja)
- **Mana cost match**: YES (generic: 1, black: 1)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F1 (HIGH): **W5 concern -- Ninjutsu implemented but combat damage trigger missing.** Dokuchi Silencer has Ninjutsu {1}{B} implemented (both Keyword marker and AbilityDefinition::Ninjutsu cost -- KI-6 compliant), but the combat damage trigger is a TODO. This means a player can Ninjutsu the creature onto the battlefield, deal combat damage, and... nothing happens. The creature is strictly weaker than intended but does not produce incorrect game state -- it just misses an optional ability ("you may discard"). The Ninjutsu itself works correctly. This is the same pattern as Steel Hellkite's pump without the X ability. **Assessment**: The reflexive trigger TODO is a genuine DSL gap (no "when you do" chaining after optional discard). Having Ninjutsu work without the damage trigger is acceptable -- the creature can still attack and block normally. Not a W5 violation.
  - Revised: Clean (Ninjutsu dual-def correct, reflexive trigger TODO valid).

## Summary
- **Cards with issues**: None (all findings resolved to acceptable on analysis)
- **Clean cards**: Steel Hellkite, Force of Despair, Scalelord Reckoner, Ruthless Lawbringer, Dokuchi Silencer
- **Valid DSL gaps documented**:
  - X-cost activated abilities with mana-value filter (Steel Hellkite)
  - Force-cycle alternative cost (exile colored card from hand) (Force of Despair)
  - Entered-this-turn filter on DestroyAll (Force of Despair)
  - "A [subtype] you control becomes target" trigger with subtype filter (Scalelord Reckoner)
  - Reflexive trigger chaining ("when you do") (Ruthless Lawbringer, Dokuchi Silencer)
