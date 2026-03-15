# Card Review: Batch D

**Date**: 2026-03-10
**Reviewer**: ability-impl-reviewer (Opus)
**Files reviewed**:
- `crates/engine/src/cards/defs/path_of_ancestry.rs`
- `crates/engine/src/cards/defs/reassembling_skeleton.rs`
- `crates/engine/src/cards/defs/reflecting_pool.rs`
- `crates/engine/src/cards/defs/pitiless_plunderer.rs`
- `crates/engine/src/cards/defs/mother_of_runes.rs`

## Card 1: Path of Ancestry

- **Oracle match**: YES -- oracle_text field matches the official text. Types (Land), no mana cost, no P/T all correct.
- **DSL correctness**: YES (within documented limitations) -- ETB tapped via `ReplacementModification::EntersTapped` is the standard pattern (matches Dimir Guildgate, Windbrisk Heights, etc.). Mana ability uses `Effect::AddManaAnyColor` which is the standard simplification for commander-identity-restricted mana. The conditional scry-on-spend mechanic is a genuine DSL gap.
- **Findings**:
  - F1 (LOW): **Conditional scry on mana spend not modeled.** The triggered ability "When that mana is spent to cast a spell that shares a creature type with your commander, scry 1" is omitted. Documented as TODO. This is a genuine DSL gap -- mana-spend triggers with creature-type comparison are not expressible. Acceptable as documented.

## Card 2: Reassembling Skeleton

- **Oracle match**: YES -- mana cost {1}{B}, types Creature - Skeleton Warrior, P/T 1/1 all correct. Oracle text matches.
- **DSL correctness**: YES (within documented limitations) -- the creature body is correctly defined. The activated ability "{1}{B}: Return ~ from your graveyard to the battlefield tapped" requires zone-specific activated abilities (activatable from graveyard), which is a documented DSL gap (`return_from_graveyard` in W5 known gaps). Empty `abilities: vec![]` is correct per W5 policy (no simplifications).
- **Findings**:
  - F1 (LOW): **Graveyard-zone activated ability not modeled.** Documented as TODO. Known DSL gap. The `back_face: None` field is explicitly set rather than using `..Default::default()` -- minor style inconsistency with other cards but not wrong.

## Card 3: Reflecting Pool

- **Oracle match**: YES -- Land type, no mana cost, no P/T, oracle text matches.
- **DSL correctness**: YES (within documented limitations) -- uses `Effect::AddManaAnyColor` as simplification. The actual ability should only produce mana types that other lands you control could produce, which requires querying mana capabilities of your own lands. Documented as TODO. This is the same simplification pattern used for Exotic Orchard and Fellwar Stone.
- **Findings**:
  - F1 (LOW): **Mana type restriction not modeled.** `AddManaAnyColor` is strictly more permissive than the oracle text -- it allows producing any color even if no land you control could produce that color. Documented as TODO. Acceptable given DSL limitations.

## Card 4: Pitiless Plunderer

- **Oracle match**: PARTIAL -- mana cost {3}{B} correct, types Creature - Human Pirate correct, P/T 1/4 correct, oracle text matches. However, the trigger semantics are broader than the oracle text specifies.
- **DSL correctness**: PARTIAL -- `TriggerCondition::WheneverCreatureDies` triggers on ANY creature dying, not just "another creature you control." Two distinct filter gaps exist: (1) should exclude self ("another"), and (2) should only trigger on creatures the controller owns ("you control"). The `treasure_token_spec(1)` usage is correct. The TODO comment acknowledges this gap.
- **Findings**:
  - F1 (MEDIUM): **Trigger fires too broadly -- any creature death vs. another creature you control.** `WheneverCreatureDies` triggers on all creature deaths across all players, including self. Oracle text says "another creature you control" which requires two filters: exclude-self AND controller-only. This will create excess Treasure tokens during gameplay (e.g., when opponents' creatures die). The card's TODO acknowledges the issue. However, this is a semantic correctness problem that affects game state -- unlike the mana production simplifications which are overly permissive but rarely matter in practice, death triggers in Commander fire constantly. Per W5 policy ("no simplifications -- a card is complete only when its full oracle text is faithfully expressible in the DSL"), this card should arguably have `abilities: vec![]` until `death_trigger_filter` is added to the DSL, since the current implementation produces incorrect game behavior. **Fix:** Either (a) change to `abilities: vec![]` with a TODO noting the DSL gap, matching W5 policy, or (b) add a filter field to `WheneverCreatureDies` (e.g., `{ filter: Option<DeathTriggerFilter> }` with controller_only + exclude_self booleans) as part of W1. Note: Zulaport Cutthroat uses the same pattern but its oracle says "or another creature" (any creature, broader), so the gap is less severe there.
  - F2 (LOW): **`back_face: None` explicitly set.** Minor style note -- could use `..Default::default()` for consistency with cards that omit this field.

## Card 5: Mother of Runes

- **Oracle match**: YES -- mana cost {W}, types Creature - Human Cleric, P/T 1/1 all correct. Oracle text matches.
- **DSL correctness**: YES (within documented limitations) -- the activated ability requires interactive color choice and dynamic protection granting, which is a genuine DSL gap. Empty `abilities: vec![]` is the correct approach per W5 policy (no simplifications). Properly documented as TODO.
- **Findings**:
  - F1 (LOW): **Activated ability with color choice not modeled.** Documented as TODO. Requires interactive player choice (color selection) plus `EffectDuration::UntilEndOfTurn` protection grant with dynamic `ProtectionQuality`. Known DSL gap.
  - F2 (LOW): **`back_face: None` explicitly set.** Same minor style note as Reassembling Skeleton and Pitiless Plunderer.

## Summary

| Card | Oracle Match | DSL Correct | Issues |
|------|-------------|-------------|--------|
| Path of Ancestry | YES | YES (simplified) | 1 LOW |
| Reassembling Skeleton | YES | YES (empty abilities) | 1 LOW |
| Reflecting Pool | YES | YES (simplified) | 1 LOW |
| Pitiless Plunderer | PARTIAL | PARTIAL | 1 MEDIUM, 1 LOW |
| Mother of Runes | YES | YES (empty abilities) | 2 LOW |

**Verdict**: 4 of 5 cards are clean. Pitiless Plunderer has a MEDIUM finding -- its trigger implementation is overly broad (fires on all creature deaths rather than "another creature you control"), producing incorrect game behavior. Per W5 policy against simplifications, the abilities vec should either be emptied or the DSL should be extended with death trigger filtering.
