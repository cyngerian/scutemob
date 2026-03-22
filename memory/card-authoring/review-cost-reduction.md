# Card Review: A-05 Cost Reduction Cards

**Reviewed**: 2026-03-22
**Cards**: 12
**Findings**: 3 HIGH, 3 MEDIUM, 2 LOW

---

## Card 1: Jet Medallion
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None -- clean.

## Card 2: Sapphire Medallion
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None -- clean.

## Card 3: Ruby Medallion
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None -- clean.

## Card 4: Emerald Medallion
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None -- clean.

## Card 5: Dragonlord's Servant
- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin Shaman)
- **Mana cost match**: YES ({1}{R})
- **P/T match**: YES (1/3)
- **DSL correctness**: YES
- **Findings**: None -- clean.

## Card 6: Dragonspeaker Shaman
- **Oracle match**: YES
- **Types match**: YES (Creature -- Human Barbarian Shaman)
- **Mana cost match**: YES ({1}{R}{R})
- **P/T match**: YES (2/2)
- **DSL correctness**: YES (change: -2 correctly encodes "cost {2} less")
- **Findings**: None -- clean.

## Card 7: Goblin Electromancer
- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin Wizard)
- **Mana cost match**: YES ({U}{R})
- **P/T match**: YES (2/2)
- **DSL correctness**: YES (SpellCostFilter::InstantOrSorcery is correct)
- **Findings**: None -- clean.

## Card 8: Bolt Bend
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({3}{R})
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-2): W5 policy violation -- uses `Effect::Nothing` as spell effect, making the card castable but doing nothing. The card's entire purpose is "Change the target of target spell or ability with a single target." With `Effect::Nothing`, a player can cast this spell, spend mana, and get no effect -- wrong game state. The cost reduction (`SelfCostReduction::ConditionalPowerThreshold`) is correctly implemented, but the spell effect is a no-op. Per W5 policy, `abilities` should be `vec![]` to prevent casting until `Effect::ChangeTarget` is added to the DSL.
  - F2 (LOW): TODO comment correctly identifies `Effect::ChangeTarget` as a DSL gap. This is a genuine gap -- no `ChangeTarget` variant exists in the Effect enum. TODO is valid.

## Card 9: Shadow of Mortality
- **Oracle match**: YES
- **Types match**: YES (Creature -- Avatar)
- **Mana cost match**: YES ({13}{B}{B} = generic: 13, black: 2)
- **P/T match**: YES (7/7)
- **DSL correctness**: YES (`SelfCostReduction::LifeLostFromStarting` is the correct variant)
- **Findings**: None -- clean.

## Card 10: Curtains' Call
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({5}{B})
- **DSL correctness**: YES
- **Findings**: None -- clean. Undaunted keyword + `SelfCostReduction::PerOpponent` + two targeted DestroyPermanent effects with separate `TargetCreature` requirements. Well structured.

## Card 11: Morophon, the Boundless
- **Oracle match**: NO
- **Types match**: YES (Legendary Creature -- Shapeshifter, uses `full_types` with `SuperType::Legendary`)
- **Mana cost match**: YES ({7} = generic: 7)
- **P/T match**: YES (6/6)
- **DSL correctness**: NO (partially implemented)
- **Findings**:
  - F3 (MEDIUM / KI-18): Oracle text mismatch. Scryfall says "As Morophon enters, choose a creature type." but the def has "As Morophon, the Boundless enters, choose a creature type." The full card name should not appear in the oracle text per Scryfall convention -- self-references use the short name.
  - F4 (HIGH / KI-3): TODO claims "needs ChosenCreatureType infrastructure" but `ReplacementModification::ChooseCreatureType` already exists in the engine and is used by Cavern of Souls, Unclaimed Territory, Secluded Courtyard, and Three-Tree City. The "As this enters, choose a creature type" replacement effect IS expressible. The TODO is stale for that part.
  - F5 (MEDIUM): TODO claims "SpellCostModifier only supports generic change" for the WUBRG colored mana reduction. This TODO IS valid -- `SpellCostModifier.change` is an `i32` that only adjusts generic mana. Morophon's ability reduces colored mana specifically ({W}{U}{B}{R}{G}), which requires a different mechanism. The `SpellCostFilter` enum has no variant for chosen creature type either. Two genuine DSL gaps remain: (1) colored mana reduction on SpellCostModifier, (2) SpellCostFilter variant for chosen creature type.
  - F6 (MEDIUM): TODO claims "+1/+1 to other creatures of chosen type" needs chosen-type dynamic filter. This IS a genuine gap -- `ContinuousEffectDef` filter would need an `EffectFilter::ChosenSubtype` variant which does not exist (confirmed: Etchings of the Chosen has the same gap documented). Valid TODO.

## Card 12: Urza's Incubator
- **Oracle match**: NO
- **Types match**: YES (Artifact)
- **Mana cost match**: YES ({3})
- **DSL correctness**: NO (unimplemented, abilities: vec![])
- **Findings**:
  - F7 (LOW / KI-18): Oracle text mismatch. Scryfall says "As this artifact enters, choose a creature type." but the def says "As Urza's Incubator enters, choose a creature type." The card uses "this artifact" as self-reference, not its own name.
  - F8 (HIGH / KI-3): TODO claims "needs ChosenCreatureType infrastructure" but `ReplacementModification::ChooseCreatureType` already exists. The ETB replacement effect for choosing a creature type IS expressible. Stale TODO for that part. However, `SpellCostFilter` lacks a variant for chosen creature type (e.g., `HasChosenSubtype`), so the cost reduction itself cannot be fully wired. Also note: Urza's Incubator affects ALL players ("cost {2} less to cast" without "you cast"), so `CostModifierScope::AllPlayers` would be needed. The scope gap and the filter gap are both genuine. The TODO should be updated to reflect that only the SpellCostFilter variant is missing, not the ChooseCreatureType infrastructure.

---

## Summary

- **Cards with issues**: Bolt Bend (1 HIGH, 1 LOW), Morophon the Boundless (1 HIGH, 3 MEDIUM), Urza's Incubator (1 HIGH, 1 LOW)
- **Clean cards**: Jet Medallion, Sapphire Medallion, Ruby Medallion, Emerald Medallion, Dragonlord's Servant, Dragonspeaker Shaman, Goblin Electromancer, Shadow of Mortality, Curtains' Call

### Issue Breakdown

| ID | Card | Severity | Pattern | Description |
|----|------|----------|---------|-------------|
| F1 | Bolt Bend | HIGH | KI-2 | W5 policy: Effect::Nothing makes card castable with no effect. Should be `abilities: vec![]` |
| F2 | Bolt Bend | LOW | -- | TODO for Effect::ChangeTarget is a valid DSL gap |
| F3 | Morophon | MEDIUM | KI-18 | Oracle uses "Morophon, the Boundless" but Scryfall uses "Morophon" |
| F4 | Morophon | HIGH | KI-3 | Stale TODO: ChooseCreatureType infrastructure exists (used by Cavern of Souls etc.) |
| F5 | Morophon | MEDIUM | -- | Valid TODO: colored mana reduction + chosen-type SpellCostFilter are genuine gaps |
| F6 | Morophon | MEDIUM | -- | Valid TODO: EffectFilter::ChosenSubtype for +1/+1 grant is a genuine gap |
| F7 | Urza's Incubator | LOW | KI-18 | Oracle uses "this artifact" but def uses "Urza's Incubator" |
| F8 | Urza's Incubator | HIGH | KI-3 | Stale TODO: ChooseCreatureType exists; only SpellCostFilter::HasChosenSubtype is missing |

### Recommended Fixes

1. **Bolt Bend** (F1): Change `abilities` to `vec![]` to prevent casting. Keep the `self_cost_reduction` field (it's correct). Add a TODO explaining that both `Effect::ChangeTarget` and the spell ability are blocked.
2. **Morophon** (F3): Fix oracle_text to use "As Morophon enters" (matching Scryfall).
3. **Morophon** (F4): Add `ReplacementModification::ChooseCreatureType` replacement effect. Update TODO to clarify that only the SpellCostFilter and colored-mana-reduction aspects are DSL gaps.
4. **Urza's Incubator** (F7): Fix oracle_text to use "As this artifact enters" (matching Scryfall).
5. **Urza's Incubator** (F8): Add `ReplacementModification::ChooseCreatureType` replacement effect. Update TODO to clarify that only `SpellCostFilter::HasChosenSubtype` is missing.
