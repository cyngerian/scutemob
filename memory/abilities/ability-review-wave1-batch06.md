# Wave 1 Batch 6 Review

**Reviewed**: 2026-03-12
**Cards**: 5
**Findings**: 0 HIGH, 4 MEDIUM, 2 LOW

---

## Card 1: Creeping Tar Pit

- **card_id**: PASS (`creeping-tar-pit`)
- **name**: PASS
- **types/subtypes**: PASS (Land, no subtypes)
- **oracle_text**: PASS (matches Scryfall exactly)
- **mana_cost**: PASS (None for land)
- **P/T**: PASS (absent, correct for land)
- **abilities**: Skeleton with TODOs
- **Findings**:
  - F1 (MEDIUM): ETB tapped is implementable. `AbilityDefinition::Replacement { trigger: WouldEnterBattlefield { filter: ObjectFilter::Any }, modification: ReplacementModification::EntersTapped, is_self: true }` works -- see `lonely_sandbar.rs` for the pattern. Left as TODO.
  - F2 (MEDIUM): `{T}: Add {U} or {B}` mana ability is implementable via `AbilityDefinition::Activated` with `Cost::Tap` and `Effect::AddMana`. Left as TODO.
  - F3 (LOW): The creature-land animation ability (`{1}{U}{B}: becomes 3/2 Elemental`) is a genuine DSL gap (no type-changing / P/T-setting / unblockable-granting activated ability). TODO is accurate.
- **Verdict**: MEDIUM (2 implementable abilities left as TODO)

---

## Card 2: Castle Ardenvale

- **card_id**: PASS (`castle-ardenvale`)
- **name**: PASS
- **types/subtypes**: PASS (Land, no subtypes)
- **oracle_text**: PASS (matches Scryfall exactly)
- **mana_cost**: PASS (None for land)
- **P/T**: PASS (absent)
- **abilities**: Skeleton with TODOs
- **Findings**:
  - F4 (LOW): Conditional ETB tapped ("unless you control a Plains") is a DSL gap -- `ReplacementModification::EntersTapped` has no condition field. TODO is accurate.
  - F5 (MEDIUM): `{T}: Add {W}` mana ability is implementable. Left as TODO.
  - F6 (MEDIUM): `{2}{W}{W}, {T}: Create a 1/1 white Human creature token` is likely implementable. The engine has `Effect::CreateToken` and activated abilities with complex mana costs. Left as TODO.
- **Verdict**: MEDIUM (2 implementable abilities left as TODO)

---

## Card 3: Oathsworn Vampire

- **card_id**: PASS (`oathsworn-vampire`)
- **name**: PASS
- **types/subtypes**: PASS (Creature -- Vampire Knight). Note: def has `&["Knight", "Vampire"]` while Scryfall order is "Vampire Knight". Subtype order is not functionally significant.
- **oracle_text**: PASS (matches Scryfall exactly)
- **mana_cost**: PASS (`generic: 1, black: 1`)
- **P/T**: PASS (2/2)
- **abilities**: Skeleton with TODOs
- **Findings**:
  - F7 (LOW): "This creature enters tapped" is likely implementable with the same `EntersTapped` replacement pattern used for lands. "Cast from graveyard if you gained life" is a DSL gap (conditional graveyard casting). TODOs are accurate.
- **Verdict**: LOW

---

## Card 4: Mishra, Claimed by Gix

- **card_id**: PASS (`mishra-claimed-by-gix`)
- **name**: PASS
- **types/subtypes**: PASS (Legendary Creature -- Phyrexian Human Artificer). Def has `&["Human", "Artificer", "Phyrexian"]` vs Scryfall "Phyrexian Human Artificer" -- order differs but not functionally significant.
- **oracle_text**: PASS (matches Scryfall exactly)
- **mana_cost**: PASS (`generic: 2, black: 1, red: 1`)
- **P/T**: PASS (3/5)
- **abilities**: Skeleton with TODOs
- **Findings**:
  - None beyond skeleton status. The entire ability (attack trigger with X = attacking creatures, drain, meld condition) is a complex DSL gap involving meld, which is not implemented. TODO is accurate.
- **Verdict**: PASS (skeleton appropriate for DSL gaps)

---

## Card 5: Thundering Falls

- **card_id**: PASS (`thundering-falls`)
- **name**: PASS
- **types/subtypes**: PASS (Land -- Island Mountain via `types_sub`)
- **oracle_text**: PASS (matches Scryfall exactly)
- **mana_cost**: PASS (None for land)
- **P/T**: PASS (absent)
- **abilities**: Skeleton with TODOs
- **Findings**:
  - F8 (MEDIUM): All three abilities are implementable in the current DSL. (1) Basic land subtypes grant implicit mana abilities, but explicit `AddMana` would also work. (2) ETB tapped uses the `EntersTapped` replacement pattern (see `lonely_sandbar.rs`). (3) ETB surveil 1 trigger: `Effect::Surveil { amount: 1 }` exists (see `consider.rs`) and ETB triggers work via `queue_carddef_etb_triggers()`. All left as TODO. Same pattern seen in `underground_mortuary.rs` and `undercity_sewers.rs` -- this appears to be a systematic gap in Phase 1 template generation for surveil lands.
- **Verdict**: MEDIUM (all abilities implementable but left as TODO)

---

## Summary

- **Cards with issues**: Creeping Tar Pit (MEDIUM), Castle Ardenvale (MEDIUM), Thundering Falls (MEDIUM), Oathsworn Vampire (LOW)
- **Clean cards**: Mishra, Claimed by Gix

**Pattern note**: The MEDIUM findings (F1, F2, F5, F6, F8) are all cases where Phase 1 template generation left implementable abilities as empty TODOs. ETB tapped replacement, basic mana abilities, token creation, and surveil ETB triggers all have working DSL patterns in other card defs. These are Phase 2 authoring tasks, not bugs -- but they represent cards that could be more functional today.
