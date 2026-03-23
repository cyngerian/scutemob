# Card Review: A-25 Activated-Tap Batch 1

**Reviewed**: 2026-03-23
**Cards**: 19
**Findings**: 5 HIGH, 3 MEDIUM, 2 LOW

---

## Card 1: Maze of Ith
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (no mana cost, land)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): Target filter too broad. Oracle says "target attacking creature" but def uses `TargetRequirement::TargetCreature` which allows targeting any creature. The DSL has `TargetCreatureWithFilter` but there is no "attacking" filter field, so this is a genuine DSL gap. However, implementing UntapPermanent on ANY creature is wrong game state (W5 policy) -- it lets you untap non-attacking creatures, which the card cannot do. The activated ability should be `vec![]` with a TODO, OR the TODO comment should clearly note the overbroad targeting. As implemented, the card does something it should not (untapping arbitrary creatures). **HIGH -- partial impl produces wrong game state (KI-2).**
  - F2 (MEDIUM): The damage-prevention half of the ability is missing (acknowledged in TODO). This is acceptable as a DSL gap, but combined with the overbroad targeting in F1, the partial impl is actively wrong.

## Card 2: Thaumatic Compass // Spires of Orazca
- **Oracle match**: YES (front face), YES (back face)
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F3 (HIGH): Back face second ability targets any creature (`TargetRequirement::TargetCreature`) but oracle says "Tap target creature **an opponent controls**." Should be `TargetCreatureWithFilter(TargetFilter { controller: TargetController::Opponent, ..Default::default() })`. As-is, you can tap your own creatures, which is wrong game state. **(KI-1 variant -- overbroad target filter.)**
  - F4 (MEDIUM): Back face has TWO activated abilities that both cost `Cost::Tap`. Since the land can only tap once, only one can be used per turn -- this is correct Rules behavior. No issue, noting for completeness.
  - F5 (LOW): Transform trigger TODO is acceptable (conditional end-step transform is a genuine DSL gap).

## Card 3: Golos, Tireless Pilgrim
- **Oracle match**: YES
- **Types match**: YES (Legendary Artifact Creature -- Scout)
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F6 (MEDIUM): Oracle says "you **may** search your library" (optional). The triggered ability has no indication of optionality (`intervening_if: None`). If SearchLibrary is always mandatory, this forces the controller to search when they might not want to. This is a minor behavioral difference (usually you want to search), but technically wrong.
  - F7 (LOW): Activated ability TODO is acceptable (ExileTopCards + free-play is a genuine DSL gap).

## Card 4: Birthing Pod
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: NO
- **DSL correctness**: N/A (abilities empty)
- **Findings**:
  - F8 (HIGH): Mana cost is wrong. Oracle cost is `{3}{G/P}` (3 generic + 1 Phyrexian green). The def uses `ManaCost { generic: 3, green: 1, ..Default::default() }` which is `{3}{G}`, ignoring the Phyrexian payment option. Should be `ManaCost { generic: 3, phyrexian: vec![PhyrexianMana::Green], ..Default::default() }`. The TODO comment claims "Phyrexian mana cost also not representable in ManaCost" which is **false** -- the DSL has `phyrexian: Vec<PhyrexianMana>` on ManaCost (PB-9). **(KI-3 -- stale TODO claims gap for now-expressible pattern; KI-7 -- wrong mana cost.)**
  - F9: The activation cost also has `{G/P}` which would similarly need `PhyrexianMana::Green` in the activation Cost::Mana, but since abilities are `vec![]` this is moot until implementation.

## Card 5: Fauna Shaman
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None. Empty abilities with TODO is acceptable -- `Cost::DiscardCard` exists but not `DiscardCardWithType(TargetFilter)` for "discard a creature card". Genuine DSL gap.

## Card 6: Arcanis the Omnipotent
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Wizard)
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F10 (HIGH): Oracle says "Return Arcanis to **its owner's** hand" but def uses `ZoneTarget::Hand { owner: PlayerTarget::Controller }`. In multiplayer, if an opponent controls Arcanis (e.g., via Bribery or Control Magic), Controller != Owner. The bounce should go to the owner's hand, not the controller's hand. **(KI-11 -- PlayerTarget::Controller for "its owner", but this is HIGH because it produces actively wrong game state in multiplayer, not just a documentation issue.)**

## Card 7: Azami, Lady of Scrolls
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Human Wizard)
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None. Genuine DSL gap -- "Tap an untapped Wizard you control" as cost requires a Cost variant that doesn't exist.

## Card 8: Opposition
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None. Genuine DSL gap (same as Azami -- tap-another-creature-as-cost).

## Card 9: Seedborn Muse
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None. Genuine DSL gap -- untap-during-other-players-untap-step is a replacement/static effect not expressible.

## Card 10: Rings of Brighthearth
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None. Genuine DSL gap -- copy-ability-on-stack triggered by activation.

## Card 11: Heartstone
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None. Genuine DSL gap -- activated ability cost reduction (not spell cost reduction).

## Card 12: Training Grounds
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None. Genuine DSL gap (same as Heartstone).

## Card 13: Gemstone Array
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F11: The first ability ({2}: put a charge counter) is correctly implemented. The second ability (remove counter as cost: add any-color mana) is a TODO. The TODO correctly identifies that `Cost` enum lacks a `RemoveCounter` variant. This is a genuine DSL gap. No wrong game state from the partial impl (adding charge counters without being able to remove them is harmless -- no free mana). Clean.

## Card 14: Springleaf Drum
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None. Genuine DSL gap (tap-another-creature-as-cost).

## Card 15: Honor-Worn Shaku
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**: None. First ability (`{T}: Add {C}`) is correctly implemented. Second ability (tap legendary permanent as cost) is a genuine DSL gap. The partial impl is safe -- producing colorless mana without the untap trick is strictly weaker, not wrong.

## Card 16: Cryptolith Rite
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None. Genuine DSL gap -- GrantActivatedAbility not in LayerModification.

## Card 17: Song of Freyalise
- **Oracle match**: YES
- **Types match**: YES (Enchantment -- Saga)
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None. Genuine DSL gap -- Saga framework not in DSL.

## Card 18: Citanul Hierophants
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None. Genuine DSL gap (same as Cryptolith Rite -- GrantActivatedAbility).

## Card 19: Glare of Subdual
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities empty)
- **Findings**: None. Genuine DSL gap (tap-another-creature-as-cost).

---

## Summary

### Findings by Severity

| ID | Severity | Card | Issue |
|----|----------|------|-------|
| F1 | HIGH | Maze of Ith | Overbroad target -- targets any creature, not just attacking creatures. Partial impl produces wrong game state (KI-2). Should be `vec![]`. |
| F3 | HIGH | Thaumatic Compass (back face) | Overbroad target -- targets any creature, should be opponent's creature only (`TargetController::Opponent`). Wrong game state. |
| F8 | HIGH | Birthing Pod | Mana cost uses `green: 1` instead of `phyrexian: vec![PhyrexianMana::Green]`. Stale TODO claims Phyrexian mana is a DSL gap (KI-3, KI-7). |
| F10 | HIGH | Arcanis the Omnipotent | "Its owner's hand" uses `PlayerTarget::Controller` -- wrong in multiplayer (KI-11 escalated to HIGH). |
| F2 | MEDIUM | Maze of Ith | Damage prevention half missing (acknowledged TODO, DSL gap). |
| F6 | MEDIUM | Golos, Tireless Pilgrim | "May search" optionality not represented -- search is always forced. |
| F4 | MEDIUM | Thaumatic Compass (back face) | Noted for completeness -- two tap-cost abilities is correct behavior. Not a real issue. |
| F5 | LOW | Thaumatic Compass | Transform trigger TODO is acceptable. |
| F7 | LOW | Golos, Tireless Pilgrim | Activated ability TODO is acceptable. |

### Cards with Issues
1. **Maze of Ith** -- 1 HIGH (overbroad target, wrong game state), 1 MEDIUM
2. **Thaumatic Compass** -- 1 HIGH (overbroad target on back face), 1 LOW
3. **Birthing Pod** -- 1 HIGH (wrong mana cost + stale TODO)
4. **Arcanis the Omnipotent** -- 1 HIGH (owner vs controller)
5. **Golos, Tireless Pilgrim** -- 1 MEDIUM (forced search), 1 LOW

### Clean Cards (14)
Fauna Shaman, Azami Lady of Scrolls, Opposition, Seedborn Muse, Rings of Brighthearth, Heartstone, Training Grounds, Gemstone Array, Springleaf Drum, Honor-Worn Shaku, Cryptolith Rite, Song of Freyalise, Citanul Hierophants, Glare of Subdual

### Recommended Fixes

1. **Maze of Ith**: Replace the activated ability with `vec![]` and a TODO noting both the "attacking creature" targeting gap and the damage-prevention gap. The partial UntapPermanent on any creature is worse than no implementation.
2. **Thaumatic Compass back face**: Change `TargetRequirement::TargetCreature` to `TargetCreatureWithFilter(TargetFilter { controller: TargetController::Opponent, ..Default::default() })`.
3. **Birthing Pod**: Fix mana cost to `ManaCost { generic: 3, phyrexian: vec![PhyrexianMana::Green], ..Default::default() }`. Remove stale comment about Phyrexian mana being a DSL gap.
4. **Arcanis the Omnipotent**: Change `ZoneTarget::Hand { owner: PlayerTarget::Controller }` to `ZoneTarget::Hand { owner: PlayerTarget::Owner }` (if `PlayerTarget::Owner` exists) or add a TODO noting the multiplayer incorrectness and switch to `vec![]` if no Owner variant exists.
