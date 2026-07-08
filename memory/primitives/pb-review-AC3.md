# Primitive Batch Review: PB-AC3 — Dynamic P/T & count amounts (CDA residual)

**Date**: 2026-07-08
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 508.1/509 (attackers), CR 613.4a/b/c (Layer 7 sublayers), CR 613.1d/205.3m
(type-changing), CR 608.2h/107.3k (X locked at resolution), CR 611.2c (membership re-eval),
CR 702.26d (phased-out), CR 400 (hand size), CR 111.1 (token count), CR 122 (counters).
**Engine files reviewed**: `cards/card_definition.rs`, `effects/mod.rs`, `rules/layers.rs`,
`state/continuous_effect.rs`, `state/combat.rs`, `state/hash.rs`.
**Card defs reviewed** (11): keep_watch, throne_of_the_god_pharaoh, mirror_entity,
krenko_tin_street_kingpin, ulvenwald_hydra, wight_of_the_reliquary, storm_kiln_artist
(CLEAN roster); galadhrim_ambush, mishra_claimed_by_gix, ashaya_soul_of_the_wild,
multani_yavimayas_avatar (PARTIAL).

## Verdict: needs-fix

0 HIGH, 1 MEDIUM, 4 LOW. The engine surface is correct and complete: both new count
`EffectAmount` variants are wired in **lockstep** across `resolve_amount` and
`resolve_cda_amount`; `SetBothDynamic` is substituted at resolution (CR 608.2h lock-in)
and has the exhaustive apply arm at Layer 7b; the hash schema is clean (EffectAmount
disc 0–21 unique, LayerModification disc 0–29 unique) with the real disc-26 collision fixed
(`RemoveSuperType`→29) and `HASH_SCHEMA_VERSION` bumped 29→30 with changelog. The 19-test
file is substantive and non-vacuous, with genuine layer-ordering and lock-at-resolution
assertions. The one MEDIUM is a wrong-sublayer registration in **mirror_entity.rs**
(AddAllCreatureTypes at Layer 6 instead of Layer 4), which produces incorrect type-line
ordering against other Layer-4 effects and contradicts both CR 613.1d and the engine's own
Changeling CDA handling. The LOWs are pre-existing/systemic and out of PB-AC3 scope. Fix the
MEDIUM (one-line layer change + comment correction) and the batch is ready to close.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| — | — | — | No engine-code findings. Lockstep, substitution, apply arm, hash, and collision-fix all verified correct. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | **MEDIUM** | `mirror_entity.rs:37` | **AddAllCreatureTypes registered at wrong layer.** `EffectLayer::Ability` (Layer 6) but "gain all creature types" is a type-changing effect = Layer 4. **Fix:** change to `EffectLayer::TypeChange`; fix the two comments (card line 20, test line 569/600). |
| 2 | LOW | `tools/authoring-report.py:151` | Pre-existing regex misclassifies Krenko as "empty". Out of PB-AC3 scope. |
| 3 | LOW | `storm_kiln_artist.rs` | Certified clean; Magecraft "or copy" clause unimplemented, no marker. Consistent with existing pattern (archmage_emeritus). |
| 4 | LOW | `wight_of_the_reliquary.rs` | Retains SacrificeAnother TODO (can sac self); correctly left blocked. |
| 5 | LOW | `ulvenwald_hydra.rs` | "may search" modeled as mandatory (deterministic fallback); pre-existing convention. |

### Finding Details

#### Finding 1 (MEDIUM): Mirror Entity — AddAllCreatureTypes registered at Layer 6 instead of Layer 4

**Status**: ✅ RESOLVED 2026-07-08. Changed `EffectLayer::Ability` → `EffectLayer::TypeChange`
in `mirror_entity.rs:37` and the mirroring effect at `pb_ac3_dynamic_pt_counts.rs:600`;
corrected the card comment (line 20) and test comment (line 569) to name Layer 4 / CR 613.1d.
All 19 PB-AC3 tests still pass.

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/mirror_entity.rs:37` (also test comment
`crates/engine/tests/pb_ac3_dynamic_pt_counts.rs:569` and the effect at line 600)
**CR Rule**: CR 613.1d / 205.3m — adding creature types is a **type-changing effect
(Layer 4)**. `LayerModification::AddAllCreatureTypes` lives in the Layer-4 section of the
`LayerModification` enum (`continuous_effect.rs:291-296`, before the Layer-5 color block).
**Issue**: Mirror Entity's second `ApplyContinuousEffect` registers `AddAllCreatureTypes`
with `layer: EffectLayer::Ability` (Layer 6). The layer loop applies effects strictly by
their `.layer` field (`layers.rs:344`, `e.layer == layer`), so this type change is applied
during Layer 6 rather than Layer 4. This is inconsistent with:
- **CR 613.1d** (type changes belong in Layer 4);
- the engine's own Changeling CDA, which inserts all creature subtypes in Layer 4
  (`layers.rs:252`, `if layer == EffectLayer::TypeChange && ... Changeling`);
- the PB-AC3 plan itself, which specified `layer: Layer4` for this effect (plan §Card
  Definition Fixes, mirror_entity "effect B").

Concrete wrong game state: a later Layer-4 `SetTypeLine`/`LoseAllSubtypes` on a creature you
control (e.g. Blood Moon-style or "loses all creature types" effect with a later timestamp)
should, per timestamp order within Layer 4, be able to override the Mirror Entity type-add.
Because Mirror Entity's add is at Layer 6, it always applies *after* any Layer-4 effect,
so the creature ends up with all creature types even when a later Layer-4 effect removed
them. The P/T outcome (Layer 7b/7c) is unaffected — this only mis-orders the type line — so
the common case (Mirror Entity alone, tribal-anthem interaction) is correct, which is why
the test suite passes. The card comment ("Layer 6, AddAllCreatureTypes", line 20) is an
aspirationally-wrong comment per the conventions hazard and must be corrected too.
**Fix**: change `layer: EffectLayer::Ability` → `layer: EffectLayer::TypeChange` at
mirror_entity.rs:37; update the card comment at line 20 and the two test comments/effects
(pb_ac3 lines 569, 600) to say Layer 4 / TypeChange.

#### Finding 2 (LOW): authoring-report.py Krenko misclassification is pre-existing, out of scope

**Severity**: LOW
**File**: `tools/authoring-report.py:151`
**Issue**: `re.search(r"abilities:\s*vec!\[\s*\]\s*,", text)` has no word boundary before
`abilities`, so it substring-matches `mana_abilities: vec![],` / `activated_abilities:
vec![],` inside Krenko's `TokenSpec` literal (krenko_tin_street_kingpin.rs:46-47),
classifying Krenko as "empty" both before and after this batch. This fully explains the
+5 (not +6) clean delta the runner reported. The Krenko card def is functionally correct
(matches oracle, covered by `test_krenko_tokens_equal_power`). Genuinely pre-existing (the
regex is not in the PB-AC3 diff) and correctly out of scope (tooling, not engine/card/test).
**Fix** (optional, separate tooling task): anchor the regex, e.g.
`(?m)^\s*abilities:\s*vec!\[\s*\]\s*,` or add `\b`.

#### Finding 3 (LOW): Storm-Kiln Artist certified clean with unmarked Magecraft copy gap

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/storm_kiln_artist.rs`
**Oracle**: "Magecraft — Whenever you cast or copy an instant or sorcery spell, create a
Treasure." The def uses `WheneverYouCastSpell`, which does not fire on spell *copies*.
**Issue**: The card is certified clean (no TODO marker, so counted clean by
authoring-report), yet the "or copy" half is unimplemented — a wrong-game-state gap on
copy triggers. This is **systemic and pre-existing**: the existing shipped clean card
`archmage_emeritus.rs` uses the identical treatment (comment "the 'or copy' half is a known
gap", no TODO). PB-AC3 is consistent with the established convention; the PB-AC3-authored
part (Layer 7c `CdaModifyPowerToughness` artifact count) is correct.
**Fix**: none required for PB-AC3 (consistent with baseline). If honest accounting is
desired, file a systemic OOS seed for a copy-aware Magecraft trigger and mark all Magecraft
cards together — do not single out storm_kiln.

#### Finding 4 (LOW): Wight of the Reliquary retains "sacrifice another" gap

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/wight_of_the_reliquary.rs:7,50`
**Oracle**: "{T}, Sacrifice another creature: ...". The def approximates with
`Cost::Sacrifice(creature filter)`, which may allow sacrificing Wight itself.
**Issue**: The plan listed wight in the "CLEAN residual" bucket and expected both TODOs
removed. The runner correctly declined (no `Cost::SacrificeAnother` exists) and left the
TODO, so the card is honestly flagged as incomplete (has TODO → not counted clean → will
not enter a game per invariant 9). The PB-AC3 part (base 2/2 + Layer 7c per creature card
in graveyard) is correct. Disposition is acceptable.
**Fix**: none for PB-AC3. The SacrificeAnother DSL gap is a separate OOS seed.

#### Finding 5 (LOW): Ulvenwald Hydra "may search" modeled as mandatory

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/ulvenwald_hydra.rs:43-55`
**Oracle**: "you **may** search your library for a land card...". The def models the search
unconditionally (deterministic fallback), forcing the search + shuffle.
**Issue**: Minor wrong game state (an optional search is made mandatory) but this is a
widespread, plan-sanctioned codebase convention, not PB-AC3-specific. The PB-AC3 part
(`CdaPowerToughness` = lands you control, `*/*` with `power/toughness: None`) is correct.
**Fix**: none for PB-AC3; tracked by the systemic "may" modeling limitation.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 508.1/509 AttackingCreatureCount | Yes (effect + CDA lockstep) | Yes | test_attacking_creature_count_basic / _zero_outside_combat / _ignores_nonattacking / test_keep_watch |
| 613/status TappedCreatureCount | Yes (effect + CDA lockstep) | Yes | test_tapped_creature_count_basic / _controller_scope / _excludes_phased_out / test_throne |
| 702.26d phased-out exclusion | Yes | Yes | test_tapped_creature_count_excludes_phased_out |
| 400 HandSize alias | Yes (delegates to CardCount, both paths) | Yes | test_hand_size_matches_card_count_hand |
| 613.4b SetBothDynamic (Layer 7b set) | Yes (PtSet + apply arm) | Yes | test_set_both_dynamic_sets_base_pt |
| 608.2h/107.3k X locked at resolution | Yes (substitution arm effects/mod.rs:2854) | Yes | test_set_both_dynamic_sets_base_pt (asserts concrete SetPowerToughness stored) / _locked_at_resolution |
| 613.4b vs 613.4c set-then-modify order | Yes | Yes | test_set_both_dynamic_then_counter_layer_order / _then_anthem |
| 611.2c membership re-eval | Yes | Yes | test_set_both_dynamic_locked_at_resolution (late-entering creature gets locked X) |
| 205.3m AddAllCreatureTypes | Partial (wrong layer — Finding 1) | Yes | test_set_both_dynamic_with_all_creature_types / test_mirror_entity — pass but enshrine wrong layer |
| 111.1 power-based token count | Yes (already-shipped) | Yes | test_power_based_token_count / test_krenko |
| Hash determinism + disc-26 collision fix | Yes | Yes | test_hash_schema_version_is_30 / test_hash_distinguishes_new_variants_and_fixes_collision |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| keep_watch | Yes | 0 | Yes | AttackingCreatureCount{EachPlayer,None} — CR-correct "all attackers" scope |
| throne_of_the_god_pharaoh | Yes | 0 | Yes | Triggered end-step + TappedCreatureCount{Controller,None}; generic end-step sweep fires it |
| mirror_entity | Yes | 0 | **No (layer)** | SetBothDynamic 7b correct; AddAllCreatureTypes at Layer 6 not 4 (Finding 1) |
| krenko_tin_street_kingpin | Yes | 0 | Yes | PowerOf(Source) after AddCounter in Sequence; misclassified "empty" by tooling only (Finding 2) |
| ulvenwald_hydra | Yes | 0 | Yes* | CdaPowerToughness=lands; "may" search mandatory (Finding 5) |
| wight_of_the_reliquary | Yes | 1 (SacrificeAnother) | No (sac-self) | PB-AC3 part correct; honestly blocked (Finding 4) |
| storm_kiln_artist | Yes | 0 | No (copy gap) | CDA correct; Magecraft copy unmarked, systemic (Finding 3) |
| galadhrim_ambush | Yes | Yes (prevention filter) | Blocked (empty abilities) | TODO names prevention-shield gap — correct |
| mishra_claimed_by_gix | Yes | Yes (Meld) | Blocked (Fixed(1) known-wrong) | TODO names Meld; placeholder marked KNOWN-WRONG — correct |
| ashaya_soul_of_the_wild | Yes | Yes (nontoken filter) | Blocked | Pre-existing Static type-grant untouched; TODO names nontoken gap — correct |
| multani_yavimayas_avatar | Yes | Yes (graveyard-return cost) | Blocked | Pump/CDA not authored (blocked on 2nd clause); TODO names return-lands cost — correct |

## Verification notes

- **Lockstep invariant (card_definition.rs:2366)**: both `AttackingCreatureCount` and
  `TappedCreatureCount` added to BOTH `resolve_amount` (effects/mod.rs:6736,6770) AND
  `resolve_cda_amount` (layers.rs:1660,1690). CDA arms correctly use BASE characteristics;
  effect arms use `calculate_characteristics` (W3-LC compliant). No raw
  `obj.characteristics.power/toughness` battlefield reads introduced.
- **`state.combat == None` → 0**: both paths early-return 0 (effect 6738, CDA 1665).
- **Phased-out**: both paths gate on `obj.is_phased_in()`.
- **HashInto**: EffectAmount arms 19/20/21 and LayerModification arm 28 present; all
  discriminants unique within each enum's impl; `HASH_SCHEMA_VERSION = 30` with changelog.
- **`RemoveSuperType` 26→29 collision fix**: verified 29 collides with nothing (0–29 now
  each used exactly once in LayerModification HashInto).
- **`is_attacking` helper**: added (combat.rs) and used by both count arms — not dead code.
- **Exhaustiveness**: `cargo build --workspace` green (per wip) covers all exhaustive
  `EffectAmount`/`LayerModification` match sites; TUI/replay-viewer matches (StackObjectKind/
  KeywordAbility) are not affected by this batch.
- **Gates**: not independently re-run by reviewer; runner reports build/clippy/fmt clean,
  2938 tests passing (2919 + 19).

## Recommendation

Fix Finding 1 (one-line `EffectLayer::Ability` → `EffectLayer::TypeChange` in mirror_entity.rs
plus the aspirational comments). LOWs 2–5 are pre-existing/systemic and out of PB-AC3 scope;
no fix required to close. After Finding 1 is applied, batch is ready to close.
