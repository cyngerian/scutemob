# Card Review: Wave 3 Batch 08 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 2 MEDIUM, 0 LOW

## Card 1: Karn's Bastion
- **Oracle match**: YES
- **Types match**: YES (Land, no supertypes)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: {T}: Add {C} correctly implemented. Proliferate ability left as TODO with accurate description (generic mana cost in activated ability + Proliferate effect not in DSL). Clean.

## Card 2: Kher Keep
- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: {T}: Add {C} correctly implemented. Token creation ability left as TODO with accurate description (named token "Kobolds of Kher Keep" not expressible in TokenSpec, plus mana+tap composite cost). Clean.

## Card 3: Kor Haven
- **Oracle match**: YES
- **Types match**: YES (Legendary Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: {T}: Add {C} correctly implemented. Prevention ability left as TODO with accurate description (prevention effects not in DSL). Uses `full_types` with empty subtypes instead of `supertypes` -- functionally equivalent, not an issue. Clean.

## Card 4: Llanowar Wastes
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): Second mana ability (Add {B} or {G}) is implemented without the "This land deals 1 damage to you" part. The card currently produces colored mana with zero downside, which is functionally wrong -- it is strictly better than the actual card. Per W5 policy, approximate behavior corrupts game state. The second ability should either include the damage as part of Effect::Choose branches (e.g., Effect::Sequence with AddMana + DealDamage to controller) or be removed entirely and left as a TODO. The TODO comment acknowledges the gap but the ability is still present and functional without the damage penalty.

## Card 5: Maelstrom of the Spirit Dragon
- **Oracle match**: YES
- **Types match**: YES (Land)
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F2 (MEDIUM): Second ability (Add one mana of any color) is implemented via AddManaAnyColor without the spending restriction "Spend this mana only to cast a Dragon spell or an Omen spell." The card currently produces unrestricted any-color mana, which is functionally wrong -- it is strictly better than the actual card. Per W5 policy, approximate behavior corrupts game state. The TODO comment before the ability acknowledges the gap but the ability is still present and functional without the restriction. Either implement the restriction or remove the ability and leave only the TODO.

## Summary
- Cards with issues: Llanowar Wastes (F1), Maelstrom of the Spirit Dragon (F2)
- Clean cards: Karn's Bastion, Kher Keep, Kor Haven
- **Common pattern**: Both MEDIUM findings are the same class of issue -- a mana ability is implemented without its drawback/restriction, making the card strictly better than intended. The implemented ability should be removed and replaced with a TODO-only comment, or the drawback must be included.
