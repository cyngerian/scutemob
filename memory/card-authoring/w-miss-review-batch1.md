# W-MISS Review — Batch 1 (12 cards)

**Reviewed**: 2026-07-17
**Worktree**: scutemob-97
**Cards**: 12
**Findings**: 0 HIGH, 0 MEDIUM, 0 LOW — all 12 clean, all correctly `Complete`

All defs verified against Scryfall oracle text, mana cost, type line, P/T, and DSL
semantics. Every referenced Effect/Cost/filter variant was traced to a real engine
implementation (no gated stubs: no `Effect::Choose`, `MayPayOrElse`, `AddManaChoice`,
`AddManaAnyColor*` in this batch). None carry a completeness marker, so all default to
`Complete`, which is correct.

## Card 1: Arbor Elf
- Oracle/Types/Mana/P-T match: YES ({G}, Elf Druid, 1/1)
- `{T}: Untap target Forest` correctly modeled as a normal `AbilityDefinition::Activated`
  with `Effect::UntapPermanent`, NOT a mana ability (effect is not AddMana, so
  `try_as_tap_mana_ability` won't pick it up). Target filter `has_subtype: Forest`. Correct.

## Card 2: Contagion Clasp
- Oracle/Types/Mana match: YES (Artifact, {2}; oracle incl. reminder text verbatim)
- ETB `-1/-1` counter: `Triggered / WhenEntersBattlefield / AddCounter{MinusOneMinusOne, count 1}`
  targeting `DeclaredTarget{0}` with `TargetCreature`. Correct.
- `{4},{T}: Proliferate`: `Sequence(Mana{generic 4}, Tap)` → `Effect::Proliferate` (real impl
  at effects/mod.rs:3467). Correct.

## Card 3: Moggcatcher
- Oracle/Types/Mana/P-T match: YES ({2}{R}{R} = generic 2 + red 2, Human Mercenary, 2/2)
- "Goblin permanent card": filter is `has_subtype: Goblin` AND `has_card_types: [all 6
  permanent types]`. Verified `matches_filter` (effects/mod.rs) enforces `has_subtype` as a
  hard AND-gate and `has_card_types` as OR-within — so subtype Goblin AND at-least-one
  permanent type. A tribal instant/sorcery cannot be fetched to the battlefield. SearchLibrary
  routes the filter through `matches_filter` (line 2806). `shuffle_before_placing: false`
  matches "put it onto the battlefield, then shuffle." Correct.

## Card 4: Skyshroud Poacher
- Oracle/Types/Mana/P-T match: YES ({2}{G}{G} = generic 2 + green 2, Human Rebel, 2/2)
- Same permanent-type-constrained tutor pattern as Moggcatcher, subtype Elf. Correct.

## Card 5: Sakura-Tribe Scout
- Oracle/Types/Mana/P-T match: YES ({G}, Snake Shaman Scout, 1/1)
- `{T}: You may put a land card from your hand onto the battlefield` →
  `Effect::PutLandFromHandOntoBattlefield{tapped:false}` (real impl at 5337); "may"
  optionality embedded in the effect. Correct.

## Card 6: Timberwatch Elf
- Oracle/Types/Mana/P-T match: YES ({2}{G} = generic 2 + green 1, Elf, 1/2)
- Multiplayer scope CORRECT: `EffectAmount::PermanentCount{filter: subtype Elf, controller:
  EachPlayer}`. `resolve_cda_player_target(EachPlayer)` returns the full turn order (all
  players), so X = Elves on the whole battlefield. Filter carries no controller restriction.
- X lock-in CORRECT: `ModifyBothDynamic` is substituted to a concrete `ModifyBoth(v)` at
  `Effect::ApplyContinuousEffect` execution time (effects/mod.rs:3008), so X is locked at
  resolution (CR 608.2h) — matches the 2016-06-08 ruling. Single target creature. Correct.

## Card 7: Wellwisher
- Oracle/Types/Mana/P-T match: YES ({1}{G} = generic 1 + green 1, Elf, 1/1)
- Multiplayer scope CORRECT: `GainLife` amount = `PermanentCount{subtype Elf, EachPlayer}`.
  `resolve_amount` → `resolve_player_target_list(EachPlayer)` counts Elves controlled by all
  players. Gains life to `Controller`. Correct.

## Card 8: Goblin Chirurgeon
- Oracle/Types/Mana/P-T match: YES ({R}, Goblin Shaman, 0/2)
- `Sacrifice a Goblin: Regenerate target creature`: `Cost::Sacrifice(subtype Goblin)` with NO
  `exclude_self` — source is a legal Goblin to sac (correct per ruling). `Effect::Regenerate`
  on `DeclaredTarget{0}`, `TargetCreature`. Correct.

## Card 9: Goblin Lookout
- Oracle/Types/Mana/P-T match: YES ({1}{R}, Goblin, 1/2)
- Cost `Sequence(Tap, Sacrifice(subtype Goblin))` — `{T}, Sacrifice a Goblin`, no
  `exclude_self` (source is a legal sac). Correct.
- "Goblin creatures get +2/+0" (ALL Goblins, no controller restriction):
  `ApplyContinuousEffect / ModifyPower(2) / AllCreaturesWithSubtype(Goblin)`. +0 toughness
  correctly modeled by using `ModifyPower` only. Correct.

## Card 10: Spore Frog
- Oracle/Types/Mana/P-T match: YES ({G}, Frog, 1/1)
- `Sacrifice this creature: Prevent all combat damage...`: `Cost::SacrificeSelf` →
  `Effect::PreventAllCombatDamage` (real impl at 5193). Correct.

## Card 11: Culling the Weak
- Oracle/Types/Mana match: YES (Instant, {B})
- Additional cost: `spell_additional_costs: [SacrificeCreature]` (enforced; CR 118.8). Correct.
- "Add {B}{B}{B}{B}": `Effect::AddMana` with `mana_pool(0,0,4,0,0,0)`. Verified signature
  `mana_pool(white,blue,black,red,green,colorless)` → 4 black. Amount and color correct.

## Card 12: Whirlpool Warrior
- Oracle/Types/Mana/P-T match: YES ({2}{U} = generic 2 + blue 1, Merfolk Warrior, 2/2)
- ETB clause scope CORRECT: `Triggered / WhenEntersBattlefield / WheelHand{player:
  Controller, ShuffleHandIntoLibrary, ThatMany}` — only the controller wheels ("your hand").
- Sac-activated clause scope CORRECT: `Activated / Sequence(Mana{red 1}, SacrificeSelf) /
  WheelHand{player: EachPlayer, ...}` — each player wheels. Two distinct scopes, both right.

## Summary
- Cards needing fixes: NONE
- Cards to demote to blocked: NONE
- Clean cards (all 12): Arbor Elf, Contagion Clasp, Moggcatcher, Skyshroud Poacher,
  Sakura-Tribe Scout, Timberwatch Elf, Wellwisher, Goblin Chirurgeon, Goblin Lookout,
  Spore Frog, Culling the Weak, Whirlpool Warrior
