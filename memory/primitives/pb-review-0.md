# Primitive Batch Review: PB-0 -- Quick Wins

**Date**: 2026-03-16
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 614.1c (ETB tapped replacement), 702.29 (Cycling), 702.80 (Wither), 508.1d (must attack), 305.6 (basic land subtypes), 204 (color indicator)
**Engine files reviewed**: `state/types.rs` (Wither, MustAttackEachCombat), `state/hash.rs` (hash support), `rules/combat.rs` (Wither damage, MustAttack enforcement), `effects/mod.rs` (Wither non-combat damage), `tools/replay-viewer/src/view_model.rs` (exhaustive match)
**Card defs reviewed**: 18 files (12 ETB-tapped lands, 1 cycling-only land, 1 Flying fix, 1 color indicator, 1 Wither, 3 forced attack)

## Verdict: needs-fix

One MEDIUM finding: Thousand-Faced Shadow is missing the Ninjutsu keyword marker and cost ability, despite Ninjutsu being fully implemented in the DSL since Batch 3. All other cards correctly implement their PB-0 scope. Engine changes for Wither and MustAttackEachCombat are CR-correct with good test coverage for Wither. MustAttackEachCombat has no dedicated tests (LOW).

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/keywords.rs` | **No MustAttackEachCombat tests.** Combat enforcement code exists at `combat.rs:283` but no test validates the keyword forces a creature to attack or rejects declarations where it doesn't. **Fix:** Add test `test_508_1d_must_attack_each_combat_enforced` verifying a creature with MustAttackEachCombat cannot skip attacking. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | **MEDIUM** | `thousand_faced_shadow.rs` | **Missing Ninjutsu keyword + cost ability.** Oracle text has "Ninjutsu {2}{U}{U}" which is fully supported in DSL. Card def has only a TODO comment. **Fix:** Add `AbilityDefinition::Keyword(KeywordAbility::Ninjutsu)` and `AbilityDefinition::Ninjutsu { cost: ManaCost { generic: 2, blue: 2, ..Default::default() } }`. |

### Finding Details

#### Finding 1 (Engine): No MustAttackEachCombat tests

**Severity**: LOW
**File**: `crates/engine/tests/keywords.rs`
**CR Rule**: 508.1d -- "The active player checks each creature they control to see whether it's affected by any requirements (effects that say a creature attacks if able...)"
**Issue**: The `KeywordAbility::MustAttackEachCombat` variant is enforced in `combat.rs:275-299` (validates that creatures with this keyword are declared as attackers unless tapped/sick/defender), but no test exercises this validation path. The keyword is used by 3 card defs (Alexios, Dauthi Slayer, Ulamog's Crusher).
**Fix**: Add at minimum one test to `crates/engine/tests/keywords.rs` that builds a game state with a MustAttackEachCombat creature and verifies that (1) declaring attackers without it produces an error, and (2) declaring attackers with it succeeds.

#### Finding 2 (Card): Thousand-Faced Shadow missing Ninjutsu

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/thousand_faced_shadow.rs:12-14`
**Oracle**: "Ninjutsu {2}{U}{U} ({2}{U}{U}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)"
**Issue**: The card def has `// TODO: Keyword -- Ninjutsu {2}{U}{U}` but the Ninjutsu ability is fully implemented in the DSL since Batch 3 (see `biting_palm_ninja.rs`, `ninja_of_the_deep_hours.rs` for reference). The card is missing both `AbilityDefinition::Keyword(KeywordAbility::Ninjutsu)` and `AbilityDefinition::Ninjutsu { cost }`. This means the card cannot be deployed via Ninjutsu in gameplay, producing wrong game state.
**Fix**: Replace the TODO with:
```rust
AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
AbilityDefinition::Ninjutsu {
    cost: ManaCost { generic: 2, blue: 2, ..Default::default() },
},
```

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 614.1c (ETB tapped) | Yes | Yes (replacement_effects.rs) | Self-replacement pattern used across 11 lands |
| 702.29 (Cycling) | Yes | Yes (existing cycling tests) | 5 lands with Cycling {3} |
| 702.80 (Wither) | Yes | Yes (4 tests in keywords.rs) | Combat, player, persist interaction, redundancy |
| 508.1d (Must attack) | Yes | No | Engine enforcement exists, no dedicated test |
| 305.6 (Basic land subtypes) | Yes | Yes (mana production tests) | Triomes use subtypes correctly |
| 204 (Color indicator) | Yes | Yes (existing tests) | Dryad Arbor uses `color_indicator: Some(vec![Color::Green])` |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| crypt_of_agadeem | Yes | 0 | Yes | ETB tapped + both mana abilities implemented |
| den_of_the_bugbear | Partial | 2 | Partial | Conditional ETB (PB-2) + animation (PB-13) deferred; mana ability correct |
| gruul_turf | Partial | 1 | Partial | ETB tapped done; bounce trigger deferred |
| halimar_depths | Partial | 1 | Partial | ETB tapped done; look-at-top trigger deferred |
| indatha_triome | Yes | 0 | Yes | ETB tapped + Cycling {3} correct |
| mortuary_mire | Partial | 1 | Partial | ETB tapped done; graveyard-to-top trigger deferred |
| oran_rief_the_vastwood | Partial | 1 | Partial | ETB tapped + mana done; counter ability deferred |
| raugrin_triome | Yes | 0 | Yes | ETB tapped + Cycling {3} correct |
| savai_triome | Yes | 0 | Yes | ETB tapped + Cycling {3} correct |
| skemfar_elderhall | Partial | 1 | Partial | ETB tapped + mana done; sac ability deferred |
| sparas_headquarters | Yes | 0 | Yes | ETB tapped + Cycling {3} correct |
| sunken_palace | Partial | 1 | Partial | ETB tapped + mana done; complex mana ability deferred |
| ziatoras_proving_ground | Yes | 0 | Yes | ETB tapped + Cycling {3} correct |
| thousand_faced_shadow | **No** | **2** | **No** | **Missing Ninjutsu keyword + cost (MEDIUM)**; ETB trigger deferred |
| dryad_arbor | Yes | 0 | Yes | Color indicator + Forest Dryad subtypes + mana correct |
| boggart_ram_gang | Yes | 0 | Yes | Haste + Wither + hybrid mana correct |
| alexios_deimos_of_kosmos | Partial | 3 | Partial | MustAttack done; sacrifice restriction, attack-owner restriction, upkeep trigger deferred |
| dauthi_slayer | Yes | 0 | Yes | Shadow + MustAttackEachCombat correct |
| ulamogs_crusher | Yes | 0 | Yes | Annihilator 2 + MustAttackEachCombat correct |

## Notes

- **Den of the Bugbear plan error**: The batch plan lists this card under "Simple ETB tapped (12)" but its oracle text says "If you control two or more other lands, this land enters tapped" -- a conditional ETB that belongs in PB-2. The implementation correctly deferred it. The actual count of unconditional ETB-tapped lands fixed is 11, not 12.
- **Remaining TODOs**: All non-Ninjutsu TODOs are for abilities outside PB-0 scope (triggered abilities, complex activated abilities, land animation, restrictions). These are properly tagged with future PB batch references and do not represent PB-0 failures.
- **Wither engine implementation**: Thoroughly correct. Handles combat damage (`combat.rs:1444-1449`), non-combat damage (`effects/mod.rs:290-309`), Infect overlap, Persist interaction. Hash support at discriminant 58. Four dedicated tests covering positive, negative, and interaction cases.
- **MustAttackEachCombat engine implementation**: Correctly checks for tapped, summoning sickness, and Defender exemptions (`combat.rs:291-297`). Hash at discriminant 158. View model coverage present. No TUI coverage needed (not displayed in stack view).
