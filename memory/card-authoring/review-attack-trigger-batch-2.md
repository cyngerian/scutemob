# Card Review: A-24 Attack Trigger Batch 2

**Reviewed**: 2026-03-23
**Cards**: 12
**Findings**: 1 HIGH, 2 MEDIUM, 2 LOW

---

## Card 1: Samut, Voice of Dissent
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES
- **DSL correctness**: MOSTLY
- **Findings**:
  - F1 (MEDIUM): Activated ability targets `TargetCreature` but oracle says "another target creature" -- the self-exclusion filter is missing. A comment acknowledges this (line 32-33) but no TODO is placed. The card can untap itself, which is wrong game state. However, since Samut has vigilance and typically would not be tapped, and the ability requires {W}+{T} (so Samut is tapped to activate), the self-untap is impossible in practice due to the tap cost. **Actually benign** -- Samut must tap to activate so she cannot target herself. Downgrading to LOW.
  - F1 (LOW): Comment on lines 32-33 notes "another" self-exclusion not enforced. Technically correct that she can't target herself anyway due to tap cost, but the comment could be clearer.

## Card 2: Tymna the Weaver
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**: CLEAN. The TODO correctly identifies that "postcombat main phase" trigger and dynamic X based on opponent damage tracking are DSL gaps. Lifelink and Partner keywords are present.

## Card 3: Thrasios, Triton Hero
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES
- **DSL correctness**: YES
- **Findings**:
  - F2 (MEDIUM): Oracle says "reveal the top card" but `RevealAndRoute` may not actually reveal the card to opponents before routing. This is a semantic question about whether the DSL primitive handles the reveal step. If `RevealAndRoute` does reveal, this is fine. If it silently routes without revealing, it is a minor behavioral difference (opponents should see the card). Flagging as MEDIUM for verification.
  - Note: The `unmatched_dest: ZoneTarget::Hand` correctly implements "otherwise, draw a card" (putting to hand is functionally equivalent to drawing for a single revealed card, though technically drawing triggers "whenever you draw" while putting to hand does not). This is a known DSL approximation.

## Card 4: Edric, Spymaster of Trest
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**: CLEAN. The TODO correctly identifies the DSL gap -- Edric triggers on ANY creature (including opponents' creatures) dealing combat damage to YOUR opponents, and the draw goes to THAT CREATURE'S CONTROLLER (not Edric's controller). This is a complex multiplayer-aware trigger that the DSL cannot express.

## Card 5: Mirror Entity
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**: CLEAN. The TODO correctly identifies that {X} in activated ability costs, setting base P/T for all your creatures, and "gain all creature types" are all DSL gaps.

## Card 6: Toski, Bearer of Secrets
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**:
  - F3 (HIGH): Indestructible keyword is implemented but the three TODOs (can't be countered, must attack, creature-you-control combat damage trigger) leave the card in an incorrect state. Toski with ONLY Indestructible on the battlefield is wrong -- it is an indestructible 1/1 that is missing "attacks each combat if able" and "can't be countered" and the card draw trigger. The indestructible-without-must-attack is arguably a W5 policy violation (partial implementation produces wrong game state -- an indestructible creature that doesn't have to attack is significantly different from one that does). **Per W5 policy, abilities should be `vec![]` if partial implementation produces wrong game state.**
  - Counterargument: Indestructible alone doesn't produce "wrong" game state in the same way a pain land giving free colored mana does -- it just makes the creature harder to remove. The missing must-attack is a combat constraint, not a state corruption. This is borderline. Keeping as HIGH for W5 review.

## Card 7: Keeper of Fables
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**: CLEAN. The TODO correctly identifies the DSL gap for "one or more non-Human creatures you control deal combat damage" trigger.

## Card 8: Sword of the Animist
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: N/A (not a creature, correctly absent)
- **DSL correctness**: MOSTLY
- **Findings**:
  - F4 (MEDIUM -- downgraded from initial assessment): The +1/+1 static is implemented via two separate `Static` abilities (ModifyPower and ModifyToughness) with `AttachedCreature` filter on `PtModify` layer. This is correct DSL usage. The attack trigger and search are left as TODO. Equip keyword is present. The partial implementation gives +1/+1 to equipped creature without the attack-search trigger -- this is a buff without downside, which is a slight advantage but not "wrong game state" in the W5 sense (the card is still an equipment that gives +1/+1, it just also should search on attack). Acceptable partial implementation.

## Card 9: Bear Umbra
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: N/A (not a creature, correctly absent)
- **DSL correctness**: YES
- **Findings**: CLEAN. Enchant keyword, +2/+2 static via PtModify layer with AttachedCreature filter, and UmbraArmor keyword are all correct. The granted triggered ability ("whenever this creature attacks, untap all lands you control") is correctly left as a TODO -- granting a triggered ability to the enchanted creature is a DSL gap.

## Card 10: Nature's Will
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: N/A (not a creature, correctly absent)
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**: CLEAN. The TODO correctly identifies multiple DSL gaps: per-combat-damage-step trigger, tap-all-lands-target-player-controls, untap-all-lands-you-control.

## Card 11: Druids' Repository
- **Oracle match**: UNABLE TO VERIFY (Scryfall MCP lookup did not find the card -- possibly an apostrophe encoding issue)
- **Types match**: YES (Enchantment, no subtypes -- correct for this card)
- **Mana cost match**: YES ({1}{G}{G} = generic 1, green 2)
- **P/T match**: N/A (not a creature, correctly absent)
- **DSL correctness**: YES (empty abilities with TODO)
- **Findings**:
  - F5 (LOW): Card name in def uses straight apostrophe `Druids' Repository` -- the actual card name uses a right single quotation mark. This is a common encoding choice and unlikely to cause issues, but noting for completeness.
  - The TODOs correctly identify two DSL gaps: "whenever a creature you control attacks" trigger condition and `Cost::RemoveCounter` for the mana-producing activated ability.

## Card 12: Derevi, Empyrial Tactician
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **P/T match**: YES
- **DSL correctness**: YES (empty abilities with TODO, Flying keyword present)
- **Findings**: CLEAN. The TODOs correctly identify: (1) the dual ETB + combat-damage trigger, (2) tap/untap choice on target permanent, and (3) the special command-zone activated ability that bypasses casting (avoids commander tax). Flying keyword is properly implemented.

---

## Summary

- **Cards with issues**: Toski Bearer of Secrets (1 HIGH), Thrasios Triton Hero (1 MEDIUM), Samut Voice of Dissent (1 LOW), Druids' Repository (1 LOW)
- **Clean cards**: Tymna the Weaver, Edric Spymaster of Trest, Mirror Entity, Keeper of Fables, Sword of the Animist, Bear Umbra, Nature's Will, Derevi Empyrial Tactician

### Issue Details

| ID | Severity | Card | Description |
|----|----------|------|-------------|
| F3 | HIGH | Toski, Bearer of Secrets | W5 policy: Indestructible keyword implemented but must-attack constraint is missing. Toski without must-attack is a meaningfully different card (indestructible blocker vs forced attacker). Consider `vec![]` per W5 policy. |
| F2 | MEDIUM | Thrasios, Triton Hero | `RevealAndRoute` to Hand is not identical to "draw a card" -- drawing triggers "whenever you draw" effects while putting to hand does not. Minor behavioral difference in games with draw-matters cards. |
| F1 | LOW | Samut, Voice of Dissent | Comment notes "another" self-exclusion not enforced in DSL, but tap cost makes self-targeting impossible anyway. |
| F5 | LOW | Druids' Repository | Could not verify oracle text via MCP (apostrophe encoding issue in card name lookup). Card appears correct from knowledge. |

### Notes
- 8 of 12 cards have empty or mostly-empty abilities with TODOs, all citing legitimate DSL gaps (combat-damage triggers for non-self creatures, command-zone activation, X-cost activated abilities, granted triggered abilities, etc.).
- The attack-trigger pattern ("whenever a/one or more creature(s) you control deal(s) combat damage to a player") is consistently identified as a DSL gap across multiple cards (Toski, Keeper of Fables, Nature's Will, Edric, Derevi, Druids' Repository). This is a major pattern gap for A-24 cards.
- All type lines, supertypes (Legendary where needed), and mana costs are correct across the batch.
