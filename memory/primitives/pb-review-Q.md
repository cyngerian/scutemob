# Primitive Batch Review: PB-Q — ChooseColor

**Date**: 2026-04-11
**Reviewer**: primitive-impl-reviewer (Opus)
**Implement commit**: `880b7797` ("W6-prim: PB-Q implement — ChooseColor primitive (6 cards)")
**CR Rules**: 614.12, 614.12a, 105.1, 106.6a, 106.12, 605.1b, 605.4a, 613.1e, 400.7
**Engine files reviewed**: `state/game_object.rs`, `state/replacement_effect.rs`, `state/continuous_effect.rs`, `state/hash.rs`, `rules/replacement.rs`, `rules/layers.rs`, `rules/mana.rs`, `rules/resolution.rs`, `effects/mod.rs`, `cards/card_definition.rs`
**Card defs reviewed**: `caged_sun.rs` (NEW), `gauntlet_of_power.rs` (NEW), `utopia_sprawl.rs` (NEW), `throne_of_eldraine.rs` (PATCH), `temple_of_the_dragon_queen.rs` (PATCH)
**Tests reviewed**: `crates/engine/tests/primitive_pb_q.rs` (11 tests)

## Verdict: needs-fix — 2 HIGH, 4 MEDIUM, 5 LOW

Two HIGH findings surfaced. STOP-AND-FLAG: **do NOT start fix-cycle this session** per
review policy; hand back to oversight to schedule a fix session.

**HIGH-1** (Gauntlet of Power controller filter) means Gauntlet of Power produces
semantically wrong game state — the card was authored despite not matching oracle text,
and the card def includes a self-acknowledged TODO comment. This is a direct W6 policy
violation ("No card is authored until its required primitives exist. No partial
implementations, no wrong game state.") and must be resolved before PB-Q closes.

**HIGH-2** (Utopia Sprawl Enchant Forest) — the card attaches to any land, not just
Forest. This is a pre-existing DSL gap (no `EnchantTarget::Forest` variant) but it was
shipped in PB-Q anyway. W6 policy violation. Either a micro-PB for land-subtype
enchant filtering must land first, or Utopia Sprawl must be deferred.

Everything else (8 mandatory gate items #2, #3, #4, #5, #6, #7, #8, the hash audit
discipline, the deterministic tie-break) checks out. Gate #5 (Aura ETB ordering) was
verified by reading `resolution.rs:1546-1581`: attachment at 1548, self-ETB at 1571,
replacement-ability registration at 1586, statics at 1605 — chosen_color lands after
attachment, before static-effect registration, which is the correct order for Utopia
Sprawl's `effect.source.attached_to` lookup. However, no integration test actually
exercises Utopia Sprawl end-to-end — see test gap M-4 below.

## HIGH Findings

### HIGH-1: Gauntlet of Power controller filter rejects opponent basic-land taps

**File**: `crates/engine/src/cards/defs/gauntlet_of_power.rs:46-50`,
`crates/engine/src/rules/mana.rs:310`
**Oracle**: "Whenever a basic land is tapped for mana of the chosen color, **its
controller** adds an additional one mana of that color."
**CR**: 106.6a

The card def pins `controller: PlayerId(0)` which is rebound at registration
(`replacement.rs:1716`) to **the Gauntlet controller**. The dispatch filter at
`mana.rs:310` then requires `*controller == player` (the player tapping for mana).
Result: if an opponent taps their Plains for white mana while Gauntlet of Power (choosing
white) is in play, the replacement **does not fire** — the opponent gets 1 W, not the
2 W the oracle mandates.

The card def comment at lines 46-49 is a self-acknowledged TODO: "Gauntlet of Power
should fire for ANY player's basic land tap, not just the controller's. Full multi-player
mana replacement dispatch deferred to PB-Q2 (requires per-player replacement registration
loop). For now, only the controller's lands benefit."

This directly violates oracle text, violates CR 106.6a (the replacement effect must
apply to the mana being produced regardless of who is tapping), and violates W6 policy:
"No card is authored until its required primitives exist. No partial implementations,
no wrong game state."

Further, the existing test `test_gauntlet_of_power_only_doubles_basic_lands` uses only
P1 taps — it does NOT exercise opponent taps and therefore does not catch this bug. The
test `test_gauntlet_of_power_pumps_all_controllers_chosen_color` is for the P/T anthem
(a Layer 7 static), not the mana replacement, so it too does not cover multi-player mana.

**Fix**: Either (a) extend `apply_mana_production_replacements` to treat Gauntlet's
replacement as "fires for any player, bills the player tapping, not the replacement
controller" — e.g., by introducing a `fires_for: AnyPlayer | SpecificPlayer(PlayerId)`
field on `ManaWouldBeProduced`, or (b) defer Gauntlet of Power to PB-Q2 until multi-player
replacement dispatch exists and remove it from PB-Q's delivered card list. Option (b)
is preferred for session economy because it preserves the PB-Q "narrow and consistent"
directive. Add a regression test with P1 Gauntlet + P2 tapping P2's basic Plains.

### HIGH-2: Utopia Sprawl can enchant non-Forest lands

**File**: `crates/engine/src/cards/defs/utopia_sprawl.rs:17-18`
**Oracle**: "Enchant Forest"
**CR**: 303.4 (Enchant keyword), 702.5

`AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Land))` allows Utopia
Sprawl to target and legally attach to any land. Oracle restricts it to Forest (basic
or non-basic with Forest subtype).

The card def comment at lines 16-18 admits this: "Forest subtype filtering is not yet
in the target filter DSL." This is a pre-existing DSL gap (no `EnchantTarget::Forest`
nor equivalent `EnchantTarget::LandSubtype(SubType)` variant), but shipping Utopia
Sprawl with it is a W6 policy violation: a player could legally cast Utopia Sprawl on
a Mountain and then the trigger would fire on Mountain taps, producing the chosen-color
mana bonus on a source that oracle text does not permit.

The downstream `ManaSourceFilter::EnchantedLand` correctly reads `attached_to`, so the
bug is entirely at cast-time target validation — the Aura can be attached to the wrong
land type, and everything downstream then works "correctly" against a wrong attachment.

**Fix**: Either (a) add an `EnchantTarget::LandSubtype(SubType)` variant (trivial: one
new enum variant, one dispatch arm in target validation, one hash arm) as a micro-PB
inside PB-Q's close-out, or (b) defer Utopia Sprawl to a follow-up PB. Option (a) is
preferred because the primitive surface is tiny and all the PB-Q mana-trigger machinery
is already in place; Utopia Sprawl is otherwise the only PB-Q-delivered card that
exercises the triggered-ability + EnchantedLand dispatch path, and deferring it leaves
that dispatch path untested at the integration level.

## MEDIUM Findings

### MEDIUM-1: Throne of Eldraine produces unrestricted mana in violation of oracle

**File**: `crates/engine/src/cards/defs/throne_of_eldraine.rs:34-45`
**Oracle**: "{T}: Add four mana of the chosen color. Spend this mana only to cast
monocolored spells of that color."

The `{T}` mana ability is authored with `Effect::AddManaOfChosenColor { amount: 4 }` —
no mana-spending restriction. Per oracle, the 4 mana are restricted to "cast monocolored
spells of that color". The card def has a TODO comment (lines 7-10) acknowledging this
as a PB-Q2/spending-restriction primitive gap.

Per W6 policy: "No partial implementations, no wrong game state." Throne of Eldraine
currently produces a WRONG game state — the 4 mana can be spent on anything.

Severity MEDIUM (not HIGH) only because Throne is Legendary and has low deck-inclusion
in common Commander play, so the damage is limited; but the policy violation is real.

**Fix**: Defer Throne of Eldraine to a future PB that lands the mana-spending-restriction
primitive, and remove it from the PB-Q card list. Alternatively, document at PB close
that Throne remains partially authored pending PB-Q2 (and file a tracking entry in
`docs/card-authoring-operations.md`).

### MEDIUM-2: Throne of Eldraine `{3},{T}: Draw two` ignores activation-cost restriction

**File**: `crates/engine/src/cards/defs/throne_of_eldraine.rs:48-62`
**Oracle**: "{3},{T}: Draw two cards. **Spend only mana of the chosen color to activate
this ability.**"

Same gap as MEDIUM-1: the activation cost is `Sequence(Mana(3 generic), Tap)`, with
no restriction that the generic 3 must be paid with mana of the chosen color. A player
can pay {3} with three colorless mana, which oracle forbids. TODO comment at line 47
acknowledges this.

**Fix**: Same as MEDIUM-1 — tie to mana-spending-restriction primitive or defer.

### MEDIUM-3: `Effect::AddManaOfChosenColor` silently produces Colorless when chosen_color is None

**File**: `crates/engine/src/effects/mod.rs:1646-1657`

```rust
let chosen_mana_color = state.objects.get(&ctx.source)
    .and_then(|o| o.chosen_color)
    .map(|c| /* ... */)
    .unwrap_or(ManaColor::Colorless);
```

If a card with `Effect::AddManaOfChosenColor` is activated on a source that never had
its `ChooseColor` replacement fire (e.g., a bug elsewhere, or a test that bypasses ETB),
this produces 1 *colorless* mana silently. No debug_assert, no event distinction from
a legitimate Colorless-producing source. This masks bugs and is inconsistent with the
mana-replacement dispatch path, which in `apply_mana_production_replacements` simply
skips when no chosen color is available (line 363-378 of `mana.rs`).

**Fix**: Either (a) add `debug_assert!(chosen_color.is_some(), "AddManaOfChosenColor
source must have chosen_color set (CR 614.12a)")`, or (b) produce no mana and emit a
tracing warning when chosen_color is None. Option (a) is preferred for fail-fast tests;
production builds are unaffected.

### MEDIUM-4: Test gap — no end-to-end test for Utopia Sprawl, Throne, or Temple

**File**: `crates/engine/tests/primitive_pb_q.rs`

Plan test list (lines 631-646 of `pb-plan-Q.md`) called for card-level integration
tests 12, 13, 14 covering Throne of Eldraine, Temple of the Dragon Queen, and Utopia
Sprawl respectively. The implement phase delivered 11 tests but **none of these three
card integrations**. The review brief explicitly flagged gate #5 (Utopia Sprawl Aura
ETB + triggered mana ability) as HIGHEST-RISK and required end-to-end verification;
that verification is missing.

Specifically, no test currently exercises:
- Utopia Sprawl cast → aura attached to Forest → Forest tapped → triggered mana ability
  fires → chosen-color mana added. The implementer verified the ordering by reading
  `resolution.rs` but did not write the test that would protect against regression.
- Throne of Eldraine `{T}: Add four mana of the chosen color` activation after ETB
  choice.
- Temple of the Dragon Queen ETB + `{T}: Add one of chosen color`.

**Fix**: Add the three integration tests from the plan. Test 14 is especially important:
it validates the triggered-mana-ability dispatch path (which is a runtime-different code
path from the replacement path the other tests exercise).

## LOW Findings

### LOW-1: `ReplacementManaSourceFilter::EnchantedLand` is dead code

**File**: `crates/engine/src/state/replacement_effect.rs:112-114`,
`crates/engine/src/rules/mana.rs:340-349`

The plan (lines 560-561 of `pb-plan-Q.md`) specified Utopia Sprawl would use
`ManaSourceFilter::EnchantedPermanent` or bind source to `attached_to` at registration.
The implementation of Utopia Sprawl instead uses `AbilityDefinition::Triggered` with
`TriggerCondition::WhenTappedForMana { source_filter: ManaSourceFilter::EnchantedLand }`
(the already-existing enum in `card_definition.rs`, not the new `ReplacementManaSourceFilter`).
This is a cleaner choice (reuses the Wild Growth precedent), but leaves
`ReplacementManaSourceFilter::EnchantedLand` defined, hashed, and dispatched in
`apply_mana_production_replacements` with no card def referencing it.

**Fix**: Either remove the variant (and its hash arm + dispatch arm) or document that
it is reserved for future use. Removal is preferred for code-hygiene.

### LOW-2: Comment framing of Caged Sun as "triggered ability" is incorrect

**File**: `crates/engine/src/state/replacement_effect.rs:206-209`

Comment on `AddOneManaOfChosenColor`: "Caged Sun / Gauntlet of Power are triggered
abilities per oracle text, but mana abilities are stackless per CR 605.3, so we
implement the effect as a replacement."

CR 106.6a explicitly categorizes "Some replacement effects increase the amount of mana
produced by a spell or ability" — Caged Sun / Gauntlet of Power are natively replacement
effects per this rule, not triggered abilities shoehorned into the replacement layer.
The implementation is correct, but the justification comment is backwards and will
mislead future maintainers.

**Fix**: Rewrite the comment to cite CR 106.6a and frame Caged Sun / Gauntlet as
replacements from the outset, not as "triggered abilities we converted."

### LOW-3: `test_choose_color_resets_on_zone_change` does not actually test zone change

**File**: `crates/engine/tests/primitive_pb_q.rs:263-290`

The test builds a fresh `GameObject` in Hand and asserts `chosen_color == None`. It does
not actually cast → set chosen_color → bounce → recast, so it only verifies `Default`
propagation, not CR 400.7 zone-change reset. The test name is misleading.

**Fix**: Either rename to `test_choose_color_default_is_none` or actually cast, mutate,
bounce, and recast within the test body.

### LOW-4: ChooseColor fallback self-inclusion is benign but undocumented

**File**: `crates/engine/src/rules/replacement.rs:1485-1495`

The color scan iterates `state.objects.values()` including the newly-entering permanent
(which has already been moved to Battlefield by the time `apply_self_etb_from_definition`
runs). For Caged Sun (artifact, no colors) this is benign. For Utopia Sprawl (green Aura)
this guarantees Green is always counted once, reinforcing the default-Green choice on an
empty board. This is consistent with `ChooseCreatureType` which has the same pattern,
but neither site documents why self-inclusion is intentional.

**Fix**: Add a one-line comment noting the self-inclusion is intentional and benign
because (a) non-colored permanents contribute no colors to the scan and (b) self-inclusion
ties with the default are resolved by default preference anyway.

### LOW-5: CR citation imprecision in PB-Q framing comment

**File**: `crates/engine/src/state/replacement_effect.rs:207-209`

Comment cites CR 605.3 and CR 603.2 for the "mana abilities are stackless" justification.
The accurate citations are CR 605.4 (mana abilities do not use the stack) and CR 605.1a/b
(definition of mana ability). CR 605.3 is about activation procedure; CR 603.2 is about
triggered-ability putting on stack, which is the opposite point.

**Fix**: Replace citations with CR 605.4 and CR 106.6a (the actual legal basis).

## Test Gaps (summary)

| Plan test | Delivered? | Priority |
|-----------|-----------|----------|
| 1. `test_choose_color_replacement_sets_field` | Yes | |
| 2. `test_choose_color_deterministic_fallback_picks_majority` | Yes | |
| 3. `test_choose_color_default_when_no_permanents` | Yes | |
| 4. `test_choose_color_resets_on_zone_change` | Partial (see LOW-3) | Low |
| 5. `test_caged_sun_full_dispatch_pumps_chosen_color_creatures` | Yes (MANDATORY) | |
| 6. `test_gauntlet_of_power_pumps_all_controllers_chosen_color` | Yes | |
| 7. `test_chosen_color_filter_no_choice_matches_nothing` | Yes | |
| 8. `test_caged_sun_doubles_chosen_color_land_mana` | Yes (MANDATORY) | |
| 9. `test_caged_sun_does_not_double_other_color_mana` | Yes | |
| 10. `test_gauntlet_of_power_only_doubles_basic_lands` | Yes | |
| 11. `test_caged_sun_chosen_color_change_via_bounce_recast` | NOT DELIVERED | Low |
| 12. `test_throne_of_eldraine_etb_chosen_color_and_mana_ability` | NOT DELIVERED | **Medium** (see MEDIUM-4) |
| 13. `test_temple_of_the_dragon_queen_etb_chosen_color` | NOT DELIVERED | **Medium** (see MEDIUM-4) |
| 14. `test_utopia_sprawl_aura_etb_chosen_color` | NOT DELIVERED | **High-priority Medium** (gate 5 risk, see MEDIUM-4) |
| 15. `test_chosen_color_hash_field_audit` | Yes (PB-S H1 defense) | |

No test exercises multi-player mana-replacement interaction — this is why HIGH-1
slipped through.

Additional gap: no test exercises Gauntlet-of-Power-with-opponent-basic-tap. This is the
test that would surface HIGH-1; add it as part of the HIGH-1 fix.

## Mandatory Review Focus List — Gate-by-Gate Status

| Gate | Item | Status | Notes |
|------|------|--------|-------|
| 1 | Deterministic tie-break in ChooseColor fallback | PASS | `replacement.rs:1496-1511`. Default preferred at max-count tie; then highest Color discriminant (Green=4 > Red=3 > ...). Deterministic because `std::HashMap` iteration is not used for the tie-break key — `max_by_key` selects exactly one maximum. |
| 2 | `apply_mana_production_replacements` signature refactor ripple | PASS for signature; **FAIL for semantic correctness** on Gauntlet (HIGH-1). Only one caller site in `mana.rs:212`, updated correctly. |
| 3 | `CreaturesYouControlOfChosenColor` vs `AllCreaturesOfChosenColor` symmetry | PASS | `layers.rs:947-980`. Both arms read `source.chosen_color` dynamically; controller-restriction only on the former. Both dispatch through the same layer-5 / layer-7 pipeline. |
| 4 | `GameObject` constructor exhaustiveness (chosen_color: None) | PASS | Grep shows 16 `chosen_color: None` sites vs 19 `chosen_creature_type: None` sites, but the 3 extra `chosen_creature_type` sites (`abilities.rs:268`, `effects/mod.rs:154`, `:185`, `:7038`) are `EffectContext` struct literals, not `GameObject` literals. Verified. All GameObject construction paths set `chosen_color: None`. |
| 5 | Utopia Sprawl Aura attach + ETB replacement ordering (**HIGHEST-RISK**) | PASS for ordering; **test gap (MEDIUM-4)** | `resolution.rs:1546-1581` confirmed: attachment (1548) → self-ETB replacement (1571) → global replacement ability registration (1586) → statics (1605). `chosen_color` is set on Utopia Sprawl after `attached_to` is set, so the triggered mana ability's `mana_source_matches(EnchantedLand)` read via `trigger_source.attached_to` returns the correct enchanted land. But see HIGH-2 for the Forest vs Land targeting issue, and MEDIUM-4 for the missing integration test. |
| 6 | Hash sentinel bump + `HashInto for GameObject` field count | PASS | Sentinel bumped to `3u8` at `hash.rs:6091`. `chosen_color` hashed at `hash.rs:1035-1042` inside `HashInto for GameObject` impl. `test_chosen_color_hash_field_audit` verifies. |
| 7 | Q4 compromise documentation at `AddOneManaOfChosenColor` | PARTIAL | Comment exists at `replacement_effect.rs:203-209` as required, but the framing is wrong (see LOW-2) and citations imprecise (see LOW-5). |
| 8 | `ReplacementManaSourceFilter` naming + scope | PASS (no collision); dead-code finding (LOW-1) | `ReplacementManaSourceFilter` distinct from `ManaSourceFilter` in `card_definition.rs`. `BasicLand`/`AnyLand`/`EnchantedLand`/`Any` variants cover the three cards' needs on paper; `EnchantedLand` unused in practice (LOW-1). `BasicLand` correctly checks `SuperType::Basic`; `AnyLand` correctly checks `CardType::Land`. Oracle match for Caged Sun (any land, OK) and Gauntlet (basic land, OK) verified. |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Caged Sun | Yes | 0 | Yes | Clean |
| Gauntlet of Power | **No** | 1 (self-acknowledged) | **No** (HIGH-1) | Controller filter wrong |
| Throne of Eldraine | Partial | 2 | **No** (MEDIUM-1/2) | Mana restrictions missing |
| Temple of the Dragon Queen | Yes for PB-Q scope | 0 PB-Q-related | Yes for the chosen-color abilities | Pre-existing Dragon-reveal complexity not PB-Q's concern |
| Utopia Sprawl | **No** (HIGH-2) | 1 (Forest subtype) | **No** for enchant target; Yes for downstream mana | Can enchant non-Forest lands |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 614.12 | Yes | Yes (test 1) | ChooseColor replacement fires at ETB |
| 614.12a | Yes | Yes (test 1, 5) | Choice committed before ETB |
| 105.1 | Yes | Yes (test 5, 6) | Five colors mapped to enum |
| 106.6a | Yes | Yes for Caged Sun (tests 8/9); **incomplete for Gauntlet** (test 10 same-controller only) | Additive mana production via replacement |
| 400.7 | Yes | Partial (test 4 — see LOW-3) | chosen_color resets via Default |
| 613.1e | Yes | Indirectly via test 5 | Uses calculate_characteristics for color scan |
| 605.1b | Yes | **NOT TESTED** for Utopia Sprawl | Triggered mana ability path |
| 605.4a | Yes | **NOT TESTED** for Utopia Sprawl | Immediate resolution of triggered mana ability |

## Previous Findings

N/A — first review of PB-Q.

## Recommendations

1. **Before fix-session**: oversight decide whether Gauntlet (HIGH-1) and Utopia Sprawl
   (HIGH-2) should be (a) fixed in a PB-Q fix cycle with minor primitive extensions, or
   (b) deferred to PB-Q2 / a follow-up subtype-enchant micro-PB. Per W6 policy "no wrong
   game state," shipping PB-Q with these cards in their current state is not acceptable.
2. **Add the three missing integration tests** (MEDIUM-4) before closing PB-Q. Test 14
   (Utopia Sprawl end-to-end) is the highest-value — it validates the gate-5 ordering
   claim that the implementer verified by reading but did not test.
3. **Add multi-player mana replacement test** specifically for Gauntlet's HIGH-1 fix
   — opponent taps P1 Plains or P2 taps P2 Plains → Gauntlet's additional mana fires
   for the tapping player.
4. **MEDIUM-3 (debug_assert on chosen_color None)** is a cheap hardening that should be
   bundled with the fix cycle.
5. **LOW items** can be batched at the end of the fix session or deferred to W3 LOW
   cleanup.

## Phase transition

Update `memory/primitive-wip.md` phase: `review` → **`review-blocked`** (HIGH findings
present, fix session required before close).
