# Primitive Batch Plan: PB-Q4 — EnchantTarget::Filtered (bundled land-variant enchant targets)

**Generated**: 2026-04-12
**Primitive**: Generalize `EnchantTarget` to carry a `TargetFilter`, unlocking all four bundled isomorphic land-variant enchant target patterns in ONE variant.
**CR Rules**: 303.4, 702.5, 704.5m, 205.3i (basic land subtypes), 205.4a (Basic supertype)
**Cards affected (planned vs realistic)**: planned ~11; **realistic ship 5-7** after yield discount + gate fallout
**Dependencies**: none (all prerequisite infra exists — see Verification Gate Results)
**Deferred items from prior PBs**: none (PB-Q closed `464d9e79`; `apply_mana_production_replacements` refactor stays)

---

## Verification Gate Results

### GATE 1 — Genju animate-land: **PASS**

Walked the full chain end-to-end. All pieces required to animate a land until end of turn exist and are proven by a live card def.

Evidence:
- `crates/engine/src/state/continuous_effect.rs:251` `LayerModification::AddCardTypes`, `AddSubtypes`, `SetPowerToughness`, `AddKeywords` — all four primitives required by "land becomes an N/N Type creature with haste".
- `crates/engine/src/state/continuous_effect.rs:99` `EffectFilter::AttachedLand` — matches the specific land this Aura/Fortification is attached to (via `attached_to`).
- `crates/engine/src/state/continuous_effect.rs:104` `EffectFilter::AttachedPermanent` — looser variant also usable.
- `crates/engine/src/rules/layers.rs` — layer system already resolves animated lands correctly; `mana.rs:145-173` specifically uses layer-resolved types so animated lands respect summoning sickness.
- **Live precedent**: `crates/engine/src/cards/defs/tatyova_steward_of_tides.rs:37-82` — `Effect::Sequence([AddCardTypes(Creature), AddSubtypes(Elemental), SetPowerToughness{3,3}, AddKeywords(Haste)])` pattern on `DeclaredTarget`. Genju reuses the identical shape targeting `AttachedLand`/`AttachedPermanent` with `Source` as activation filter.

**Yield impact**: Genju cycle remains *technically* expressible, BUT each Genju card also needs a "when enchanted Forest is put into graveyard, return this from graveyard to hand" trigger — which is a self-rebuy-from-graveyard LTB trigger tied to *the attached land* leaving (not self leaving). That trigger condition does NOT exist (`WhenLeavesBattlefield` only fires on self). **Genju cycle is deferred on trigger infrastructure grounds, not on animate-land grounds.** Gate 1 passes the literal test but Genju still exits scope.

### GATE 2 — Chained-to-the-Rocks controller filter on EnchantTarget: **FAIL**

`EnchantTarget` enum (`crates/engine/src/state/types.rs:263-280`) has exactly 8 flat variants: `Creature`, `Permanent`, `Artifact`, `Enchantment`, `Land`, `Planeswalker`, `Player`, `CreatureOrPlaneswalker`. No variant carries subtype, supertype, or controller predicate.

`matches_enchant_target()` at `crates/engine/src/rules/sba.rs:914-931` takes only `&Characteristics` — no access to the aura's controller or target's controller ObjectId. Refactor required.

**Impact**: Chained to the Rocks, "Enchant land you control" cycle (7 cards), "Enchant basic land you control" (Ossification, Dimensional Exile — 2 cards) are all blocked until the enum carries a filter + enforcement gets a controller view.

**Resolution for this PB**: introduce `EnchantTarget::Filtered(TargetFilter)` and extend `matches_enchant_target()` signature to accept `aura_controller: PlayerId` and `target_controller: PlayerId`. `TargetFilter` already has `controller: TargetController`, `basic: bool`, `has_subtype: Option<SubType>`, `has_subtypes: Vec<SubType>` (OR), `has_card_type: Option<CardType>` — no TargetFilter changes needed.

### GATE 3 — Corrupted Roots disjunction ("Enchant Forest or Plains"): **PASS (via existing TargetFilter.has_subtypes)**

`TargetFilter.has_subtypes: Vec<SubType>` (OR semantics) already exists at `crates/engine/src/cards/card_definition.rs:2315-2318`, used by "Vampire or Wizard creature card" (Bloodline Necromancer). Corrupted Roots maps to `EnchantTarget::Filtered(TargetFilter { has_subtypes: vec![Forest, Plains], has_card_type: Some(Land), ..default })`.

Requirement: `matches_enchant_target` must honor `has_subtypes` (OR) in addition to `has_subtype` (AND). Implementation is a local `.any()` check.

**Impact**: no enum changes beyond the single `Filtered` variant.

---

## Primitive Specification

Add **one** new variant to `EnchantTarget`:

```rust
pub enum EnchantTarget {
    // ... existing 8 variants unchanged ...
    /// Filtered enchant restriction — the aura's target and attachment must
    /// match the given `TargetFilter` evaluated against layer-resolved
    /// characteristics AND the relative controllers of aura and target.
    ///
    /// Covers:
    ///   "Enchant Forest"                 → has_subtype: Some(Forest), has_card_type: Some(Land)
    ///   "Enchant Mountain you control"   → + controller: TargetController::You
    ///   "Enchant basic land you control" → basic: true, has_card_type: Some(Land), controller: You
    ///   "Enchant nonbasic land"          → has_card_type: Some(Land), exclude_basic: true  (NEW flag, see below)
    ///   "Enchant land you control"       → has_card_type: Some(Land), controller: You
    ///   "Enchant Forest or Plains"       → has_subtypes: vec![Forest, Plains], has_card_type: Some(Land)
    Filtered(TargetFilter),
}
```

**TargetFilter change required**: add `pub nonbasic: bool` flag to `TargetFilter` (mirrors the existing `basic: bool`). Required for "Enchant nonbasic land" (Uncontrolled Infestation) because `basic: false` is the default "no restriction" state, not "must be nonbasic."

`matches_enchant_target` is refactored to take `(enchant, target_chars, aura_controller, target_controller)` and dispatch to a new helper `filter_matches_enchant(&TargetFilter, &Characteristics, aura_controller, target_controller) -> bool`.

### Why this shape

- **One variant, four oracle patterns.** Bundles PB-Q4 scope per the directive.
- **Reuses existing `TargetFilter`.** No parallel filter hierarchy.
- **Cast-time validation is already correct.** `validate_targets_with_source` (casting.rs:3395) uses `TargetRequirement` + matching — we only need the post-target Enchant restriction check to honor the same filter. The spell's `TargetRequirement` is set from the same filter, so both the targeting and the Enchant SBA check pass/fail consistently.
- **SBA (`check_aura_sbas`, sba.rs:941)** is the only place continuous validation happens. It already iterates battlefield objects and has access to `obj.controller` and `aura.controller` — plumbing the two controllers into `matches_enchant_target` is a one-line change at each of the two call sites.

---

## CR Rule Text (excerpts)

- **CR 303.4a**: "An Aura spell requires a target. The 'enchant [object or player]' keyword ability ... defines what that target can be."
- **CR 303.4c**: "If an Aura is attached to an illegal object or player, or is not attached to an object or player, it's put into its owner's graveyard."
- **CR 702.5a**: "Enchant is a static ability, written 'Enchant [object or player].' The enchant ability restricts what an Aura spell can target and what an Aura can enchant."
- **CR 704.5m**: Aura SBA — illegally-attached Aura goes to graveyard.
- **CR 205.3i / 205.4a**: Basic land types (Plains/Island/Swamp/Mountain/Forest) are subtypes; "Basic" is a supertype.

---

## Engine Changes

### Change 1: Extend `EnchantTarget` enum

**File**: `crates/engine/src/state/types.rs` (line 263)
**Action**: Add `Filtered(TargetFilter)` variant. Import `TargetFilter` from `crates::cards::card_definition` or declare a forward ref.
**Pattern**: Follow existing flat variants; `TargetFilter` is `Clone + Debug + PartialEq + Eq + Hash + Serialize + Deserialize + Ord` — verify `Ord/PartialOrd` on `TargetFilter` (may need to add; several `Vec` fields complicate this — if blocked, derive manually or box).

**Risk**: `EnchantTarget` currently derives `Ord, PartialOrd, Hash`. `TargetFilter` may not. Two paths:
- Path A: Add missing derives to `TargetFilter` (preferred — check for blocking fields).
- Path B: Box into `Filtered(Box<TargetFilter>)` and hand-implement `Ord`/`Hash` by serializing or by tuple-ordering the key fields used in enchant contexts.

**Runner must verify Path A works before falling back to Path B.**

### Change 2: Extend `TargetFilter` with `nonbasic` flag

**File**: `crates/engine/src/cards/card_definition.rs` (around line 2310, next to `pub basic: bool`)
**Action**: Add `#[serde(default)] pub nonbasic: bool,`
**CR**: 205.4a — distinguishes "nonbasic land" from "basic land" and from "land" with no restriction.
**Enforcement**: extend `matches_filter` wherever `basic: true` is checked; add a symmetric `if f.nonbasic && chars.supertypes.contains(Basic) → false`.

### Change 3: Extend `matches_enchant_target` signature

**File**: `crates/engine/src/rules/sba.rs:914-931`
**Action**: Change signature to:
```rust
pub(crate) fn matches_enchant_target(
    enchant: &EnchantTarget,
    target_chars: &Characteristics,
    aura_controller: PlayerId,
    target_controller: PlayerId,
) -> bool
```
Add `EnchantTarget::Filtered(f) =>` arm that evaluates the TargetFilter against `target_chars` plus the two controllers. The existing flat variants ignore the new parameters.

**New helper**: `fn enchant_filter_matches(f: &TargetFilter, chars: &Characteristics, aura_controller: PlayerId, target_controller: PlayerId) -> bool` in sba.rs, checking:
- `has_card_type` / `has_card_types` (any)
- `has_subtype` (single, AND)
- `has_subtypes` (Vec, OR — must match at least one if Vec nonempty)
- `basic` (must have Basic supertype)
- `nonbasic` (must NOT have Basic supertype)
- `controller` (TargetController::You → target_controller == aura_controller; Opponent → !=; Any → always true)
- Reuse logic from existing `matches_filter` where possible; factor out.

### Change 4: Update the two call sites to pass controllers

**File A**: `crates/engine/src/rules/casting.rs:3432`
```rust
// Before: super::sba::matches_enchant_target(&enchant_target, &tc)
// After:  super::sba::matches_enchant_target(&enchant_target, &tc, player /* caster */, tc_controller)
```
Look up `target_id`'s controller from `state.objects.get(&target_id).map(|o| o.controller)` — must be the object's CURRENT controller (layer-resolved if Layer 2 effects are in play; for cast-time simplicity use `obj.controller` since Aura cast validation happens before the Aura hits the battlefield).

**File B**: `crates/engine/src/rules/sba.rs:1000`
```rust
// Before: matches_enchant_target(&enchant_target, &tc)
// After:  matches_enchant_target(&enchant_target, &tc, aura_obj.controller, target_obj.controller)
```
Both objects are already in scope in the SBA loop — just pipe the controller fields through.

### Change 5: TargetRequirement for Aura cast-time selection

**No engine change**, but the card defs authored in this PB set their Aura's `TargetRequirement` to mirror the `EnchantTarget::Filtered` filter. The existing `TargetRequirement::TargetPermanentWithFilter(TargetFilter)` variant already exists — card defs use it to drive the initial target selection, then the Enchant restriction check in casting.rs:3432 re-validates via `matches_enchant_target`. Both must use the same filter so they never disagree.

### Change 6: Exhaustive match sites — audit list

The runner MUST verify each of these builds clean after adding `EnchantTarget::Filtered`:

| File | Line | Match expression | Action |
|------|-----|------------------|--------|
| `crates/engine/src/state/hash.rs` | 262-272 | `impl HashInto for EnchantTarget` | Add arm `EnchantTarget::Filtered(f) => { 8u8.hash_into(hasher); f.hash_into(hasher); }` — requires `HashInto for TargetFilter` (verify exists; add if missing). |
| `crates/engine/src/rules/sba.rs` | 914-931 | `match enchant` inside `matches_enchant_target` | Add `Filtered(f) =>` arm; see Change 3. |
| `crates/engine/tests/enchant.rs` | — | Possible enum exhaustive asserts | Grep for `EnchantTarget::` patterns; add coverage. |
| `crates/engine/tests/sba.rs` | — | Same | Same. |
| `crates/engine/tests/bestow.rs` | — | Bestow uses `EnchantTarget::Creature`; unlikely to break | Grep confirm. |
| `tools/replay-viewer/src/view_model.rs` | — | `KeywordAbility` match may stringify `Enchant(_)` | Verify arm uses `Enchant(_)` wildcard, not inner match. If it matches inner, add `Filtered` arm returning e.g. "Enchant filtered". |
| `tools/tui/src/play/panels/stack_view.rs` | — | Same | Same. |

**The runner runs `cargo build --workspace` after adding the variant — any non-exhaustive match error becomes a required fix in this PB.** Replay-viewer and TUI exhaustive-match drift is the #1 PB compile-error source (see `memory/gotchas-infra.md`).

### Change 7: Helpers re-export

**File**: `crates/engine/src/cards/helpers.rs:37`
**Action**: `TargetFilter` is already exported; `EnchantTarget` is already exported. No new exports needed.

---

## Card Definition Work

### Realistic shipping list (after yield discount)

Per `feedback_pb_yield_calibration.md`: discount 40-50%. Oversight named ~11 cards; after gates, target **5-7 clean ships** and up to 4 deferred to micro-PBs.

**CONFIDENCE HIGH (ship these)**:

1. **Awaken the Ancient** — `{1}{R}{R}{R}` Aura, Enchant Mountain, "Enchanted Mountain is a 7/7 red Giant creature with haste. It's still a land." Static continuous effect on `AttachedLand`: AddCardTypes(Creature), AddSubtypes(Giant), SetColors({Red}), SetPowerToughness{7,7}, AddKeywords(Haste). `EnchantTarget::Filtered({ has_card_type: Land, has_subtype: Mountain })`.
2. **Corrupted Roots** — `{B}` Aura, "Enchant Forest or Plains\nWhenever enchanted land becomes tapped, its controller loses 2 life." **BLOCKED on "whenever enchanted land becomes tapped" trigger (no `WhenTappedForMana` equivalent for non-mana tap events — `WhenSelfBecomesTapped` only fires for the source, not attached)**. **DEFER.**
3. **Chained to the Rocks** — `{W}` Aura, Enchant Mountain you control, ETB exile target creature opponent controls until this leaves. Uses existing `Effect::ExileWithDelayedReturn` (Brutal Cathar precedent). `EnchantTarget::Filtered({ has_card_type: Land, has_subtype: Mountain, controller: TargetController::You })`. **HIGH confidence.**
4. **Ossification** — `{1}{W}` Aura, Enchant basic land you control, ETB exile target creature or planeswalker opponent controls until this leaves. `Filtered({ basic: true, has_card_type: Land, controller: You })`. Exile target: `TargetFilter { has_card_types: [Creature, Planeswalker], controller: Opponent }`. **HIGH confidence.**
5. **Dimensional Exile** — `{1}{W}` Aura, Enchant basic land you control, ETB exile target creature opponent controls until this leaves. Same as Ossification but creatures only. **HIGH confidence.**

**CONFIDENCE MEDIUM**:

6. **Hot Springs** — `{1}{G}` Aura, Enchant land you control, "Enchanted land has '{T}: Prevent the next 1 damage that would be dealt to any target this turn.'" Needs `LayerModification::AddActivatedAbility` granting a prevention-shield activated ability to `AttachedLand`. Prevention-shield primitive exists? Unverified — the runner must check. **If prevention primitive missing → defer.**

7. **Earthlore** — `{G}` Aura, Enchant land you control, "Tap enchanted land: Target blocking creature gets +1/+2 until end of turn. Activate only if enchanted land is untapped." Grants activated ability with tap cost to `AttachedLand`. Tap cost on a granted ability + "only if untapped" (natural state check). **Verify `AddActivatedAbility` supports `untapped_restriction` — if not, defer.**

**CONFIDENCE LOW (defer)**:

8. **Genju cycle (5 cards)** — needs "when enchanted land dies, return THIS from graveyard to hand" trigger — that trigger doesn't exist. Animate-land portion works but rebuy trigger blocks. **DEFER to micro-PB with new `WhenAttachedPermanentLeavesBattlefield` trigger.**
9. **Utopia Sprawl** — needs "as ETB, choose a color" + "add additional mana of chosen color" — chosen_color on Aura + `AddMana` variant for chosen color. **DEFER.**
10. **Spreading Algae / Uncontrolled Infestation** — "when enchanted land becomes tapped, destroy it" — same missing trigger as Corrupted Roots. **DEFER.**
11. **Caribou Range / Crackling Emergence / Harmonious Emergence / Mystic Might / Tourach's Gate** — each needs one or more of: grant-complex-activated-ability, "if would be destroyed instead…" replacement, Cumulative Upkeep + custom sacrifice cost, time-counter sacrifice loops. **DEFER individually; not PB-Q4 scope.**

### Realistic yield: **5 clean ships** (Awaken the Ancient, Chained to the Rocks, Ossification, Dimensional Exile, + 1 of Hot Springs/Earthlore if prevention/tap-ability gates pass).

That is a 5/11 yield rate (≈45%), consistent with calibration feedback.

### New card definitions

For each clean-ship card, the runner authors a new file at `crates/engine/src/cards/defs/<name>.rs` and registers it in the relevant defs module. Oracle text is copied verbatim from MCP lookup results above.

**Template for Awaken the Ancient** (authoritative example; others follow the same shape):

```rust
// Awaken the Ancient — {1}{R}{R}{R} Enchantment — Aura
// Enchant Mountain
// Enchanted Mountain is a 7/7 red Giant creature with haste. It's still a land.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    let enchant_filter = TargetFilter {
        has_card_type: Some(CardType::Land),
        has_subtype: Some(SubType("Mountain".to_string())),
        ..Default::default()
    };
    CardDefinition {
        card_id: cid("awaken-the-ancient"),
        name: "Awaken the Ancient".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 3, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant Mountain\nEnchanted Mountain is a 7/7 red Giant creature with haste. It's still a land.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(
                EnchantTarget::Filtered(enchant_filter.clone())
            )),
            // Static continuous effects animating the attached Mountain.
            // Layer 4 type/subtype additions, Layer 5 color, Layer 7b P/T, Layer 6 keywords.
            // Precedent: tatyova_steward_of_tides.rs lines 37-82 (AddCardTypes+AddSubtypes+SetPT+AddKeywords sequence).
            AbilityDefinition::Static(StaticEffect::ContinuousEffect(ContinuousEffectDef {
                layer: EffectLayer::TypeChange,
                modification: LayerModification::AddCardTypes(
                    [CardType::Creature].into_iter().collect(),
                ),
                filter: EffectFilter::AttachedLand,
                duration: EffectDuration::WhileSourceOnBattlefield,
                condition: None,
            })),
            AbilityDefinition::Static(StaticEffect::ContinuousEffect(ContinuousEffectDef {
                layer: EffectLayer::TypeChange,
                modification: LayerModification::AddSubtypes(
                    [SubType("Giant".to_string())].into_iter().collect(),
                ),
                filter: EffectFilter::AttachedLand,
                duration: EffectDuration::WhileSourceOnBattlefield,
                condition: None,
            })),
            AbilityDefinition::Static(StaticEffect::ContinuousEffect(ContinuousEffectDef {
                layer: EffectLayer::ColorChange,
                modification: LayerModification::SetColors([Color::Red].into_iter().collect()),
                filter: EffectFilter::AttachedLand,
                duration: EffectDuration::WhileSourceOnBattlefield,
                condition: None,
            })),
            AbilityDefinition::Static(StaticEffect::ContinuousEffect(ContinuousEffectDef {
                layer: EffectLayer::PtSet,
                modification: LayerModification::SetPowerToughness { power: 7, toughness: 7 },
                filter: EffectFilter::AttachedLand,
                duration: EffectDuration::WhileSourceOnBattlefield,
                condition: None,
            })),
            AbilityDefinition::Static(StaticEffect::ContinuousEffect(ContinuousEffectDef {
                layer: EffectLayer::Ability,
                modification: LayerModification::AddKeywords([KeywordAbility::Haste].into_iter().collect()),
                filter: EffectFilter::AttachedLand,
                duration: EffectDuration::WhileSourceOnBattlefield,
                condition: None,
            })),
        ],
        ..Default::default()
    }
}
```

The runner must verify `StaticEffect::ContinuousEffect` is the correct shape for static continuous effects on the source (vs. the `Effect::ApplyContinuousEffect` trigger form used by Tatyova's landfall). If Aura static continuous effects don't use `WhileSourceOnBattlefield`, mirror the exact pattern from an existing Aura like Bear Umbra.

**Pattern reference**: Grep `WhileSourceOnBattlefield` + `AttachedLand` for existing precedents. If none exist for `AttachedLand` (only `AttachedCreature`/`AttachedPermanent`), the runner uses `AttachedPermanent` since a land is a permanent.

### Card-side fixes (existing defs)

None in scope. PB-Q4 authors new cards only; no existing def currently uses a land-subtype EnchantTarget placeholder.

---

## Unit Tests

**File**: `crates/engine/tests/enchant.rs` (add to existing file)

Every test is marked **MANDATORY** or **OPTIONAL**. The runner MUST NOT skip mandatory tests — carry-forward from PB-Q implement feedback: the previous session skipped 3 mandatory tests and that cost a re-run.

1. **[MANDATORY] `test_enchant_filtered_land_subtype_cast_time_legal`** — Cast Awaken the Ancient targeting a Mountain you control; verify success. CR 303.4a.
2. **[MANDATORY] `test_enchant_filtered_land_subtype_cast_time_illegal`** — Cast Awaken the Ancient targeting a Forest; verify `InvalidTarget`. CR 303.4a.
3. **[MANDATORY] `test_enchant_filtered_controller_cast_time_legal`** — Cast Chained to the Rocks targeting your own Mountain; verify success. CR 303.4a.
4. **[MANDATORY] `test_enchant_filtered_controller_cast_time_illegal`** — Cast Chained to the Rocks targeting an opponent's Mountain; verify `InvalidTarget`. CR 303.4a.
5. **[MANDATORY] `test_enchant_filtered_basic_land_legal`** — Cast Ossification targeting a basic Plains; verify success. CR 205.4a.
6. **[MANDATORY] `test_enchant_filtered_basic_land_illegal_nonbasic`** — Cast Ossification targeting a non-basic dual land that has Plains subtype; verify `InvalidTarget`. CR 205.4a — the `basic` flag must reject non-basic subtyped lands.
7. **[MANDATORY] `test_enchant_filtered_sba_control_change`** — Attach Chained to the Rocks to your Mountain, then change controller of the Mountain to an opponent (via control-change test helper). SBA must detect the aura is now illegal and put it in graveyard. CR 704.5m.
8. **[MANDATORY] `test_enchant_filtered_sba_land_becomes_nonland`** — Attach Awaken the Ancient to a Mountain, cast an effect that removes the Land type from it. Aura goes to graveyard next SBA check. CR 704.5m.
9. **[MANDATORY] `test_enchant_filtered_disjunction_forest_or_plains`** — (Placeholder card using `has_subtypes: vec![Forest, Plains]`). Cast targeting Forest → legal; targeting Plains → legal; targeting Mountain → illegal. Validates OR semantics. CR 303.4a.
10. **[MANDATORY] `test_enchant_filtered_nonbasic_land`** — (Placeholder card using `nonbasic: true, has_card_type: Land`). Cast targeting a basic Mountain → illegal; targeting a non-basic dual → legal. Validates new `nonbasic` flag.
11. **[MANDATORY] `test_animate_land_pt_and_types_via_chained_or_awaken`** — Attach Awaken the Ancient to a Mountain; read layer-resolved characteristics; verify `card_types` contains Creature, `subtypes` contains Giant + Mountain, power/toughness = 7/7, has Haste. Verify mana ability still present (it's still a land — CR 205.3i, layer system preserves land functionality).
12. **[MANDATORY] `test_animate_land_summoning_sickness_propagation`** — Animate a Mountain that entered this turn; verify Haste prevents summoning sickness from blocking tap-to-attack. CR 302.1 + layer-resolved Haste.
13. **[OPTIONAL] `test_enchant_filtered_preserves_existing_variants`** — Regression: cast a plain "Enchant creature" Aura (Rancor) — existing `EnchantTarget::Creature` path still works. Covers the concern that expanding the enum broke flat dispatch.
14. **[OPTIONAL] `test_enchant_filtered_multiplayer_4p_control_permutation`** — 4-player game; attach "Enchant land you control" aura in a way that triggers illegal-attachment across multiple control-change events; verify SBA picks correct APNAP ordering.

**Pattern**: Follow existing tests in `crates/engine/tests/enchant.rs` and `crates/engine/tests/sba.rs`. Use `harness::setup_game(4)` + manual `spawn_permanent` + `state.objects.get_mut(...).controller = ...` for control-change tests.

Tests 1-12 are MANDATORY. If any test cannot compile or cannot run because of missing infrastructure, the runner STOPS and reports rather than silently skipping. Skipping a mandatory test requires an explicit entry in the review doc justifying the skip.

---

## Verification Checklist

- [ ] `EnchantTarget::Filtered(TargetFilter)` variant added; all exhaustive matches updated.
- [ ] `TargetFilter.nonbasic: bool` added; `matches_filter` honors it; hash + serde derives still compile.
- [ ] `matches_enchant_target` signature extended with `(aura_controller, target_controller)`; both call sites updated.
- [ ] `filter_matches_enchant` helper added in sba.rs, covers all 6 TargetFilter fields enumerated above.
- [ ] `cargo build --workspace` clean (catches replay-viewer + TUI drift).
- [ ] `cargo test -p mtg-engine --test enchant` passes (tests 1-12 all run, 0 skipped).
- [ ] `cargo test --all` passes (2625+ tests, no regressions).
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --check` clean.
- [ ] 5 card defs authored and registered: Awaken the Ancient, Chained to the Rocks, Ossification, Dimensional Exile, (+ Hot Springs OR Earthlore if prevention/tap-ability gates pass).
- [ ] Each authored card has a smoke test in its own module file or in `enchant.rs`.
- [ ] Deferred cards are recorded in `memory/primitive-wip.md` with the exact primitive blocking each.
- [ ] `apply_mana_production_replacements` (PB-Q) is untouched — verify via `git diff 464d9e79..HEAD -- crates/engine/src/effects/mana.rs` shows no changes.

---

## Risks & Edge Cases

1. **`TargetFilter` derive completeness**: `TargetFilter` may not derive `Ord`/`Hash`. Boxing may be required. Adds a line or two to `HashInto` and possibly the enum size constraint.
2. **Aura cast-time target is Layer 2 sensitive**: if a control-change effect is pending between target selection and resolution, the cast-time check sees `obj.controller` (pre-layer) while SBA later sees layer-resolved controller. For Chained to the Rocks this is mostly moot because casting resolves before SBA, but a Mindslaver-style mid-resolution swap could desync. Document; don't try to fix in this PB.
3. **"Enchant land you control" SBA ordering**: when an opponent gains control of your land via theft, your Aura becomes illegal. Which SBA pass? This is standard CR 704.5m and the existing illegal-aura loop handles it — no new ordering concerns.
4. **`nonbasic` flag is redundant with `supertypes: !contains(Basic)`**: intentional minimal change. If a broader supertype-filter refactor is planned elsewhere, coordinate; otherwise the local flag is pragmatic.
5. **Runner temptation to include Genju**: the animate-land gate technically passed but the rebuy trigger blocks. Runner must not author Genju cards "partially" — the deferred-PB policy forbids TODOs in authored card defs.
6. **Runner temptation to include Hot Springs/Earthlore without verifying the granted-activated-ability infra**: mark these explicitly as "verify before authoring; if blocked, defer and carry forward to PB-Q4b."
7. **Exhaustive match in replay-viewer/TUI**: per `gotchas-infra.md`, runners miss this ~50% of the time. `cargo build --workspace` is MANDATORY before claiming completion.

---

## Scope Summary

- **Primitive**: 1 new enum variant (`EnchantTarget::Filtered`), 1 new TargetFilter field (`nonbasic`), 1 refactored function signature (`matches_enchant_target` + 2 call sites), 1 helper (`filter_matches_enchant`).
- **LOC estimate**: ~120-180 engine LOC + ~250 LOC across 5 card defs + ~200 LOC tests.
- **Cards planned**: 11 considered, 5 committed, 6 explicitly deferred with named blocking primitives.
- **Sessions**: 1 plan (this), 1 implement, 1 review, 1 close (≤4).
- **Next action for the runner**: `/start-work W6-PB-Q4-impl`, read this plan, then `cargo check` the current workspace state before editing anything.
