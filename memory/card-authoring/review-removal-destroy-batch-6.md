# Card Review: Removal/Destroy Batch 6

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 2 HIGH, 2 MEDIUM, 1 LOW

## Card 1: Feed the Swarm
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None — clean card.

## Card 2: Kindred Dominance
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (empty abilities with valid TODO)
- **Findings**: None — TODO is legitimate. DSL lacks ChooseCreatureType + negated type filter for DestroyAll. Clean.

## Card 3: Elspeth, Sun's Champion
- **Oracle match**: YES
- **Types match**: YES (Legendary Planeswalker — Elspeth)
- **Mana cost match**: YES ({4}{W}{W})
- **DSL correctness**: YES
- **Findings**: None — all three loyalty abilities correctly implemented. Token spec, DestroyAll with min_power filter, and CreateEmblem with static effects are all correct. Clean card.

## Card 4: Staff of Compleation
- **Oracle match**: NO
- **Types match**: YES
- **Mana cost match**: YES ({3})
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-3): TODO on line 72 claims "Effect::UntapPermanent not in DSL" but `Effect::UntapPermanent { target: EffectTarget }` EXISTS in `card_definition.rs:1017` and is used by multiple cards (Combat Celebrant, Blessed Alliance, Minamo, Thousand-Year Elixir). The {5} untap ability should use `Effect::UntapPermanent { target: EffectTarget::Self_ }` instead of `Effect::Nothing`.
  - F2 (MEDIUM / KI-11): First ability targets "permanent you own" but filter uses `controller: TargetController::You` which means "you control". In Commander, ownership and control can differ. `TargetController` enum lacks an `Owner` variant. Should add a TODO noting the owner-vs-controller distinction is a DSL gap, since the current filter is overbroad (matches things you control but don't own) and too narrow (misses things you own but don't control).
  - F3 (LOW / KI-18): Oracle text in def says "Untap Staff of Compleation" but Scryfall oracle says "Untap this artifact." Minor oracle text mismatch.

## Card 5: Rakdos Charm
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES ({B}{R})
- **DSL correctness**: NO
- **Findings**:
  - F4 (HIGH / KI-2): Mode 1 (Destroy target artifact, line 33-35) is implemented but modes 0 and 2 are `Effect::Nothing`. This makes the card castable with partial implementation that produces wrong game state — a player could cast it choosing mode 1 (correct) but modes 0 and 2 do nothing when they should exile a graveyard or deal damage. Per W5 policy, a modal spell where only 1 of 3 modes works should have `abilities: vec![]` to prevent incorrect gameplay. Alternatively, all three modes should be implemented or none.
  - F5 (MEDIUM): Mode 2 TODO claims "no per-creature self-damage effect" — need to verify if `ForEach::EachCreature` + `DealDamage` to controller exists. Regardless, the W5 policy issue (F4) takes precedence.

## Summary
- **Cards with issues**: Staff of Compleation (1 HIGH, 1 MEDIUM, 1 LOW), Rakdos Charm (1 HIGH, 1 MEDIUM)
- **Clean cards**: Feed the Swarm, Kindred Dominance, Elspeth Sun's Champion
