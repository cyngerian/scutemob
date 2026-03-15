# Card Review: Wave 3 Batch 15 (mana-land)

**Reviewed**: 2026-03-13
**Cards**: 8
**Findings**: 2 HIGH, 0 MEDIUM, 1 LOW

## Card 1: Voldaren Estate
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): File header comment says "{T}: Add {B} or {R}" but the actual oracle text is "Add one mana of any color" (with Vampire-only spend restriction). The `oracle_text` field itself is correct; this is a misleading comment only.
  - TODOs accurately describe two DSL gaps: life-payment cost + mana spend restriction, and variable cost reduction based on board state.

## Card 2: War Room
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. TODO accurately describes the commander-color-identity-scaled life payment gap.

## Card 3: Wasteland
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. TODO accurately describes the sacrifice-as-cost gap.

## Card 4: Wastewood Verge
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F2 (HIGH): The second ability ({T}: Add {B}) is implemented without its activation restriction ("Activate only if you control a Swamp or a Forest"). The TODO explicitly acknowledges this produces wrong behavior ("producing black unconditionally") but claims it is "acceptable per W5 policy." This directly contradicts W5 policy, which states: "no simplifications -- a card is complete only when its full oracle text is faithfully expressible in the DSL. Empty abilities: vec![] is fine; wrong/approximate behavior corrupts game state." The unrestricted {B} ability should be removed entirely and replaced with a TODO-only comment. As implemented, this card illegally produces black mana when the controller has no Swamp or Forest.

## Card 5: Wirewood Lodge
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. TODO accurately describes the untap-target-Elf gap.

## Card 6: Yavimaya Coast
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: NO
- **Findings**:
  - F3 (HIGH): The second ability ({T}: Add {G} or {U}) is implemented as a color-choice mana ability but omits the mandatory 1-damage rider ("This land deals 1 damage to you"). The TODO acknowledges the self-damage is missing. Per W5 policy, wrong behavior (free colored mana with no drawback) corrupts game state. The partial implementation should be removed entirely, leaving only the {C} ability and a TODO comment describing the full painland ability as a DSL gap. As implemented, the controller gets {G} or {U} without paying any life, which is strictly better than the real card.

## Card 7: Yavimaya Hollow
- **Oracle match**: YES
- **Types match**: YES (Legendary supertype correctly set via `full_types`)
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. TODO accurately describes the Regenerate gap.

## Card 8: Zhalfirin Void
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none)
- **DSL correctness**: YES
- **Findings**: None. ETB scry 1 is fully and correctly implemented. Both abilities present.

## Summary
- Cards with issues: Wastewood Verge (1 HIGH), Yavimaya Coast (1 HIGH), Voldaren Estate (1 LOW)
- Clean cards: War Room, Wasteland, Wirewood Lodge, Yavimaya Hollow, Zhalfirin Void

### Issue Details

| ID | Severity | Card | Description |
|----|----------|------|-------------|
| F1 | LOW | Voldaren Estate | Misleading file header comment (says B/R, oracle says any color) |
| F2 | HIGH | Wastewood Verge | Black mana ability implemented without activation restriction -- violates W5 policy (wrong behavior). Remove the second `AbilityDefinition::Activated` and leave as TODO only. |
| F3 | HIGH | Yavimaya Coast | Painland colored mana implemented without 1-damage rider -- violates W5 policy (strictly better than real card). Remove the second `AbilityDefinition::Activated` and leave as TODO only. |
