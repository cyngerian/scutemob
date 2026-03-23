# Card Review: A-20 Pump-Buff Batch 3

**Reviewed**: 2026-03-23
**Cards**: 6
**Findings**: 1 HIGH, 3 MEDIUM, 0 LOW

---

## Card 1: Ezuri, Renegade Leader

- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Elf Warrior)
- **Mana cost match**: YES ({1}{G}{G})
- **P/T match**: YES (2/2)
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): Missing `exclude_self: true` on the Regenerate ability's TargetFilter. Oracle says "Regenerate **another** target Elf" but the filter uses `..Default::default()` which leaves `exclude_self: false`. This allows Ezuri to target itself with Regenerate, which is wrong. Add `exclude_self: true` to the TargetFilter.
  - F2 (MEDIUM -- valid TODO): The second ability ("{2}{G}{G}{G}: Elf creatures you control get +3/+3 and gain trample until end of turn") is left as a TODO. The DSL has `OtherCreaturesYouControlWithSubtype(SubType)` but NOT `CreaturesYouControlWithSubtype(SubType)` -- this ability includes the source. The TODO is legitimate. However, the TODO text says "EffectFilter::CreaturesYouControlWithSubtype does not exist (only Other variant)" which is accurate.

---

## Card 2: Beastmaster Ascension

- **Oracle match**: YES
- **Types match**: YES (Enchantment, no supertypes needed)
- **Mana cost match**: YES ({2}{G})
- **DSL correctness**: YES (empty abilities with TODOs)
- **Findings**:
  - F3 (MEDIUM -- valid TODO): Both TODOs are legitimate. (1) "Whenever a creature you control attacks" trigger condition does not exist in the DSL -- there is no `WheneverCreatureYouControlAttacks` trigger. (2) Conditional static ability gated on counter count (7+ quest counters) also requires a DSL pattern that doesn't exist (`Condition::HasCountersAtLeast` + static continuous effect). Both are genuine gaps.

---

## Card 3: Quest for the Goblin Lord

- **Oracle match**: YES
- **Types match**: YES (Enchantment)
- **Mana cost match**: YES ({R})
- **DSL correctness**: PARTIAL
- **Findings**:
  - F4 (MEDIUM -- valid TODO): The triggered ability (ETB Goblin counter placement) is correctly implemented using `WheneverCreatureEntersBattlefield` with a Goblin subtype filter and `TargetController::You`. The TODO for the conditional static ("As long as this has 5+ quest counters, creatures you control get +2/+0") is legitimate -- same gap as Beastmaster Ascension (condition-gated static continuous effect). Note however that this gives +2/+0 to ALL creatures you control (not just Goblins), so `CreaturesYouControl` filter would be correct here (which DOES exist). The gap is the condition gating, not the filter.

---

## Card 4: Vampire Nocturnus

- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire, no supertypes)
- **Mana cost match**: YES ({1}{B}{B}{B})
- **P/T match**: YES (3/3)
- **DSL correctness**: YES (empty abilities with TODOs)
- **Findings**:
  - No actionable findings. Both TODOs are legitimate:
    1. "Play with the top card of your library revealed" -- hidden information reveal is a genuine DSL gap.
    2. Conditional static based on top card color + self-inclusive tribal filter -- genuine gap (needs `Condition::TopCardOfLibraryIsColor` which doesn't exist, plus a filter that includes self AND other Vampires).

---

## Card 5: Goblin Bushwhacker

- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin Warrior)
- **Mana cost match**: YES ({R})
- **P/T match**: YES (1/1)
- **DSL correctness**: NO
- **Findings**:
  - F5 (MEDIUM): Oracle says "When this creature enters, **if it was kicked**" -- this is an intervening-if condition (CR 603.4). The condition should be checked both when the trigger would go on the stack AND at resolution. The def uses `Effect::Conditional { condition: Condition::WasKicked }` inside the effect with `intervening_if: None`, which only checks at resolution and still places a needless trigger on the stack when not kicked. Should use `intervening_if: Some(Condition::WasKicked)` and move the pump/haste effects to be the direct `effect` (not wrapped in Conditional). Reference: `nullpriest_of_oblivion.rs` uses the correct `intervening_if` pattern. The current implementation is functionally nearly identical (effect does nothing if not kicked) but is technically incorrect per CR 603.4.

---

## Card 6: Goblin Sledder

- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin)
- **Mana cost match**: YES ({R})
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**:
  - No findings. The activated ability correctly uses `Cost::Sacrifice(TargetFilter { has_subtype: Goblin })` for the sacrifice cost and `ApplyContinuousEffect` with `ModifyBoth(1)` on `DeclaredTarget { index: 0 }` for the +1/+1 effect. Target requirement is `TargetCreature` which is correct (any creature). Clean implementation.

---

## Summary

- **Cards with issues**: Ezuri Renegade Leader (F1 HIGH, F2 MEDIUM), Beastmaster Ascension (F3 MEDIUM), Quest for the Goblin Lord (F4 MEDIUM), Goblin Bushwhacker (F5 MEDIUM)
- **Clean cards**: Vampire Nocturnus, Goblin Sledder

### Issue Summary

| ID | Sev | Card | Issue |
|----|-----|------|-------|
| F1 | HIGH | Ezuri, Renegade Leader | Missing `exclude_self: true` on Regenerate target -- oracle says "another target Elf" but filter allows self-targeting |
| F2 | MEDIUM | Ezuri, Renegade Leader | Valid TODO -- `CreaturesYouControlWithSubtype` (self-inclusive) does not exist in EffectFilter |
| F3 | MEDIUM | Beastmaster Ascension | Valid TODOs -- attack trigger and condition-gated static are genuine DSL gaps |
| F4 | MEDIUM | Quest for the Goblin Lord | Valid TODO -- condition-gated static is a genuine DSL gap (trigger impl is correct) |
| F5 | MEDIUM | Goblin Bushwhacker | Kicker intervening-if should use `intervening_if: Some(Condition::WasKicked)` not `Effect::Conditional` wrapping (CR 603.4); ref: nullpriest_of_oblivion.rs |
