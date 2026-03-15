# Card Review: Batch C

**Date**: 2026-03-10
**Reviewer**: ability-impl-reviewer (Opus)
**Cards reviewed**: Reanimate, Fellwar Stone, Mana Crypt, Grand Abolisher, Delighted Halfling

## Card 1: Reanimate

- **Oracle text**: "Put target creature card from a graveyard onto the battlefield under your control. You lose life equal to its mana value."
- **Mana cost**: {B} -- definition has `black: 1`. Correct.
- **Types**: Sorcery. Correct.
- **Oracle match**: YES -- oracle_text string matches.
- **DSL correctness**: NO -- placeholder effect is wrong per W5 policy.
- **Findings**:

**F1** (MEDIUM): **Placeholder Spell effect violates W5 policy.** The definition uses
`AbilityDefinition::Spell` with `Effect::GainLife { amount: 0 }` as a placeholder. Per W5
policy: "Empty `abilities: vec![]` is fine; wrong/approximate behavior corrupts game state."
This placeholder makes Reanimate castable and resolvable as a no-op spell. A player could
"cast Reanimate" in-game and it would resolve, gaining 0 life. This is an approximate behavior
-- the card should either have `abilities: vec![]` (preventing casting until the DSL supports
graveyard targeting and return-to-battlefield effects) or the `Spell` effect should use a
`NoOp` / neutral pattern that doesn't trigger life-gain events. The `GainLife { amount: 0 }`
could interact with "whenever you gain life" triggers (though gaining 0 is not actually gaining
life per CR 118.3e, so this is a LOW risk in practice).
**Fix**: Replace the abilities with `abilities: vec![]` and update the TODO to note the DSL
gaps needed (graveyard targeting, return-to-battlefield, dynamic life loss based on mana value).

## Card 2: Fellwar Stone

- **Oracle text**: "{T}: Add one mana of any color that a land an opponent controls could produce."
- **Mana cost**: {2} -- definition has `generic: 2`. Correct.
- **Types**: Artifact. Correct.
- **Oracle match**: YES -- oracle_text string matches.
- **DSL correctness**: YES (with documented simplification).
- **Findings**:

**F2** (LOW): **Mana color restriction simplified to any color.** The definition uses
`Effect::AddManaAnyColor` which allows producing any color, not just colors that opponents'
lands could produce. The TODO correctly documents this as a DSL gap ("same as Exotic Orchard.
Simplified to any color."). This is an acceptable approximation per W5 -- the behavior is
documented and the card is functional, though over-permissive. This matches the existing
pattern used by Exotic Orchard, Command Tower, etc.

## Card 3: Mana Crypt

- **Oracle text**: "At the beginning of your upkeep, flip a coin. If you lose the flip, Mana Crypt deals 3 damage to you. / {T}: Add {C}{C}."
- **Mana cost**: {0} -- definition uses `ManaCost::default()` (all zeros). Correct.
- **Types**: Artifact. Correct.
- **Oracle match**: YES -- oracle_text string matches (uses `\n` separator for two abilities).
- **DSL correctness**: YES (with documented simplification).
- **Findings**:

**F3** (LOW): **Coin flip simplified to always dealing damage.** The upkeep trigger always
deals 3 damage instead of flipping a coin and only dealing on a loss. The TODO correctly
documents this: "coin flip -- should only deal damage 50% of the time." This is a
conservative (worst-case) approximation. Acceptable per W5 since it's documented.

**F4** (LOW): **Mana ability produces {C}{C} correctly.** The `mana_pool(0, 0, 0, 0, 0, 2)`
call produces 2 colorless mana. Parameter order is (white, blue, black, red, green, colorless).
Correct.

## Card 4: Grand Abolisher

- **Oracle text**: "During your turn, your opponents can't cast spells or activate abilities of artifacts, creatures, or enchantments."
- **Mana cost**: {W}{W} -- definition has `white: 2`. Correct.
- **Types**: Creature -- Human Cleric. `creature_types(&["Human", "Cleric"])`. Correct.
- **Power/Toughness**: 2/2. Correct.
- **Oracle match**: YES -- oracle_text string matches.
- **DSL correctness**: YES (ability is a DSL gap, correctly left empty).
- **Findings**:

No issues. The ability (restricting opponent actions during your turn) is a DSL gap that
requires a static restriction system not yet implemented. The definition correctly uses
`abilities: vec![]` with a TODO comment. The card has explicit `back_face: None` instead of
using `..Default::default()` -- this is a style inconsistency but not a bug.

## Card 5: Delighted Halfling

- **Oracle text**: "{T}: Add {G}. If this mana is spent to cast a legendary spell, that spell can't be countered."
- **Mana cost**: {G} -- definition has `green: 1`. Correct.
- **Types**: Creature -- Halfling Citizen. `creature_types(&["Halfling", "Citizen"])`. Correct.
- **Power/Toughness**: 1/2. Correct.
- **Oracle match**: YES -- oracle_text string matches.
- **DSL correctness**: YES (with documented simplification).
- **Findings**:

**F5** (LOW): **Mana tracking / conditional uncounterability simplified.** The definition
models Delighted Halfling as a plain green mana dork, omitting the "can't be countered"
clause for legendary spells. The TODO correctly documents this as a DSL gap: "mana tracking
(conditional uncounterability based on mana source) is not expressible." This is acceptable --
the mana production works correctly, only the rider is missing.

**F6** (LOW): **Style: explicit `back_face: None` instead of `..Default::default()`.** Both
Grand Abolisher and Delighted Halfling use explicit field listing with `back_face: None` instead
of `..Default::default()`. Other cards in the batch (Reanimate, Fellwar Stone, Mana Crypt) use
`..Default::default()`. This is a style inconsistency across card definitions. Not a bug.

## Summary

| # | Severity | Card | Description |
|---|----------|------|-------------|
| F1 | MEDIUM | Reanimate | Placeholder `GainLife{0}` makes card castable as no-op; should use `vec![]` |
| F2 | LOW | Fellwar Stone | Mana color unrestricted (documented DSL gap) |
| F3 | LOW | Mana Crypt | Coin flip always deals damage (documented DSL gap) |
| F4 | -- | Mana Crypt | {C}{C} mana production verified correct |
| F5 | LOW | Delighted Halfling | Mana tracking / uncounterability missing (documented DSL gap) |
| F6 | LOW | Grand Abolisher, Delighted Halfling | Style: explicit `back_face: None` vs `..Default::default()` |

**Verdict**: 1 MEDIUM (F1), 4 LOW. F1 should be fixed per W5 policy. All other items are
properly documented DSL gaps or style nits.
