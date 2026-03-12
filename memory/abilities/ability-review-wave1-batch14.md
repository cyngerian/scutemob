# Wave 1 Batch 14 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 0 HIGH, 3 MEDIUM, 3 LOW

---

## Card: Swiftwater Cliffs
- card_id: OK (`swiftwater-cliffs`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK (matches Scryfall exactly)
- abilities: **MEDIUM** -- All three abilities are implementable in the current DSL but left as empty TODO:
  1. "This land enters tapped" -- implementable via `ReplacementModification::EntersTapped`
  2. "When this land enters, you gain 1 life" -- implementable via ETB triggered ability with `GainLife { amount: 1, player: PlayerTarget::Controller }`
  3. "{T}: Add {U} or {R}" -- implementable via `ManaAbility`
- Verdict: **MEDIUM** (all abilities implementable but skipped)

## Card: Godless Shrine
- card_id: OK (`godless-shrine`)
- name: OK
- types/subtypes: OK (Land -- Plains Swamp)
- oracle_text: OK (matches Scryfall exactly)
- abilities: Skeleton with TODOs. The mana ability TODO is unnecessary -- Plains/Swamp subtypes grant intrinsic mana abilities via engine rules (no CardDef needed). The shock ETB is a known DSL gap (`shock_etb`). TODOs are accurate.
- Verdict: **PASS**

## Card: Bojuka Bog
- card_id: OK (`bojuka-bog`)
- name: OK
- types/subtypes: OK (Land, no subtypes)
- oracle_text: OK (matches Scryfall exactly)
- abilities: **MEDIUM** -- Two of three abilities are implementable but left as TODO:
  1. "This land enters tapped" -- implementable via `ReplacementModification::EntersTapped`
  2. "When this land enters, exile target player's graveyard" -- DSL gap (`targeted_trigger`), TODO is correct
  3. "{T}: Add {B}" -- implementable via `ManaAbility`
- Verdict: **MEDIUM** (ETB tapped + mana ability implementable but skipped; exile trigger correctly identified as DSL gap)

## Card: Overgrown Tomb
- card_id: OK (`overgrown-tomb`)
- name: OK
- types/subtypes: OK (Land -- Swamp Forest)
- oracle_text: OK (matches Scryfall exactly)
- abilities: Skeleton with TODOs. Same as Godless Shrine -- intrinsic mana from subtypes, shock ETB is DSL gap. TODOs are accurate.
- Verdict: **PASS**

## Card: Blood Crypt
- card_id: OK (`blood-crypt`)
- name: OK
- types/subtypes: OK (Land -- Swamp Mountain)
- oracle_text: OK (matches Scryfall exactly)
- abilities: Skeleton with TODOs. Same as Godless Shrine/Overgrown Tomb -- intrinsic mana from subtypes, shock ETB is DSL gap. TODOs are accurate.
- Verdict: **PASS**

---

## Summary

HIGH: 0 | MEDIUM: 2 | LOW: 0

**MEDIUM findings:**

- F1 (MEDIUM): **Swiftwater Cliffs** -- All three abilities (ETB tapped, gain 1 life ETB trigger, tap for U/R mana) are implementable in the current DSL but left as empty `vec![]` with TODOs. This is a gain-land pattern; other gain-lands in the codebase should be checked for the same issue.

- F2 (MEDIUM): **Bojuka Bog** -- ETB tapped and tap-for-B mana ability are both implementable in the current DSL but left as TODO. The exile-graveyard triggered ability is correctly identified as a DSL gap (`targeted_trigger`). Per W5 policy, since the card cannot be fully implemented, `abilities: vec![]` is acceptable -- but the two implementable abilities could still be added (ETB tapped + mana) while leaving the triggered ability as a TODO. This is a judgment call on partial implementation.

**Clean cards:** Godless Shrine, Overgrown Tomb, Blood Crypt (all shock lands with correct subtypes and accurate DSL gap TODOs)
