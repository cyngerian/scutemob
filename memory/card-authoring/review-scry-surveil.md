# Card Review: Scry/Surveil Batch

**Reviewed**: 2026-03-22
**Cards**: 7
**Findings**: 2 HIGH, 3 MEDIUM, 2 LOW

---

## Card 1: Faerie Seer
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None -- clean card.

## Card 2: Doom Whisperer
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None -- clean card.

## Card 3: Woe Strider
- **Oracle match**: NO
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (abilities are correct for what is implemented)
- **Findings**:
  - F1 (LOW / KI-18): Oracle text mismatch on last line. MCP returns `"This creature escapes with two +1/+1 counters on it."` (modern templating). Card def has `"Woe Strider escapes with two +1/+1 counters on it."` using the card name instead of "this creature". Should match Scryfall exactly.
  - F2 (LOW / KI-19): TODO for escape ETB counters is valid. The DSL has no mechanism for "escapes with N +1/+1 counters" (a conditional replacement that only applies when cast via Escape). Correctly documented, correctly omitted.

## Card 4: Umbral Collar Zealot
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: MAYBE
- **Findings**:
  - F3 (MEDIUM): Sacrifice cost filter uses `has_card_types: vec![CardType::Creature, CardType::Artifact]` which has OR semantics per the field doc (line 1730 of card_definition.rs). This correctly models "creature or artifact". However, the `has_card_type` (singular) field is not set, so no AND conflict. The filter looks correct. One concern: `Cost::Sacrifice(TargetFilter { ... })` with `has_card_types` -- verify the sacrifice cost resolution code actually checks `has_card_types` (plural) and not just `has_card_type` (singular). If the engine only checks the singular field, this filter would match anything. Needs engine-side verification.
  - F4 (MEDIUM): Oracle says "Sacrifice another creature or artifact" -- the word "another" means the source creature itself cannot be sacrificed. The `TargetFilter` used here has no `exclude_self` or similar field. If `Cost::Sacrifice` inherently excludes the source permanent, this is fine. If not, the zealot can sacrifice itself to its own ability, which is wrong. Needs engine-side verification. (Note: Woe Strider and Viscera Seer use the same pattern for "sacrifice another creature" -- if those work correctly, this does too.)

## Card 5: Retreat to Coralhelm
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities vec is empty)
- **Findings**:
  - F5 (MEDIUM): TODO claims modal triggered abilities are not supported. This is partially valid: `Effect::Choose` exists for modal effects (line 1207 of card_definition.rs) and could be nested inside a triggered ability's effect. However, the first mode "You may tap or untap target creature" requires targeting within a modal choice of a triggered ability, which is a genuine DSL gap (targets are declared at the trigger level, not per-mode). The second mode (Scry 1) is trivially expressible. The landfall trigger condition would need `TriggerCondition::WhenALandEntersUnderYourControl` or similar -- checking if that exists would be needed. Overall the `vec![]` is justified given the targeting-within-modes gap, but the TODO explanation could be more precise.

## Card 6: Aqueous Form
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F6 (HIGH / KI-2): W5 policy concern -- partial implementation. The card implements Enchant + CantBeBlocked static grant but omits the scry-on-attack trigger. While the missing scry is only missing upside (not incorrect behavior per se), the card is partially functional. Under strict W5 policy ("partial impl that produces wrong game state should use vec![]"), this is borderline. The unblockable grant without scry means the enchantment still provides significant correct value. However, if W5 policy is interpreted strictly, the card should be `abilities: vec![]` since the trigger is genuinely not expressible (no `WhenEnchantedCreatureAttacks` trigger condition). Recommend keeping as-is with a TODO noting the missing trigger, since the implemented portion is fully correct and provides the card's primary effect. Reclassify to MEDIUM if W5 policy allows partial-but-correct implementations.
  - F7 (LOW): TODO correctly identifies the DSL gap -- no `WhenEnchantedCreatureAttacks` trigger condition exists in `TriggerCondition`. This is a valid gap.

## Card 7: Hermes, Overseer of Elpis
- **Oracle match**: YES
- **Types match**: YES (Legendary supertype present)
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F8 (HIGH / KI-2): W5 policy violation -- the first ability triggers on ALL spells cast by the controller, not just noncreature spells. This produces incorrect game state: casting a creature spell creates a Bird token when it should not. The `WheneverYouCastSpell` trigger condition has no spell type filter field, so this cannot be correctly expressed. Per W5 policy, a trigger that fires too broadly and produces wrong game state should use `abilities: vec![]` rather than an overbroad implementation. The second ability (attack with Birds) is correctly omitted with a TODO.
  - F9 (LOW): TODOs are valid. `WheneverYouCastSpell` genuinely lacks a spell type filter (noncreature), and there is no trigger condition for "whenever you attack with one or more [subtype]". Both are real DSL gaps.

---

## Summary

- **Cards with issues**: Woe Strider (2 LOW), Umbral Collar Zealot (2 MEDIUM), Retreat to Coralhelm (1 MEDIUM), Aqueous Form (1 HIGH + 1 LOW), Hermes Overseer of Elpis (1 HIGH + 1 LOW)
- **Clean cards**: Faerie Seer, Doom Whisperer
- **HIGH findings**: 2
  - F6: Aqueous Form -- partial implementation (W5 borderline, Enchant+CantBeBlocked correct but scry trigger missing)
  - F8: Hermes -- overbroad trigger produces wrong game state (creates Bird tokens on creature spells)
- **MEDIUM findings**: 3
  - F3: Umbral Collar Zealot -- verify engine checks `has_card_types` (plural) on sacrifice cost
  - F4: Umbral Collar Zealot -- "another" exclusion on sacrifice cost (self-sacrifice possible?)
  - F5: Retreat to Coralhelm -- `vec![]` justified, TODO wording could be more precise
- **LOW findings**: 4
  - F1: Woe Strider oracle text uses card name instead of "this creature"
  - F2: Woe Strider escape counters TODO is valid
  - F7: Aqueous Form trigger gap TODO is valid
  - F9: Hermes both TODOs are valid

### Recommended Actions
1. **Hermes** (F8): Change `abilities` to `vec![]` with TODO explaining both gaps. The overbroad trigger is wrong game state.
2. **Aqueous Form** (F6): Decision needed on W5 strictness. If strict, revert to `vec![]`. If "correct partial is OK", keep as-is with TODO for the missing scry trigger.
3. **Woe Strider** (F1): Update oracle_text to use "This creature" instead of "Woe Strider" to match current Scryfall templating.
4. **Umbral Collar Zealot** (F3/F4): Verify engine-side that `Cost::Sacrifice` checks `has_card_types` and excludes source permanent.
