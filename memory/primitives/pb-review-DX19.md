# Primitive Batch Review: PB-DX19 — the unbounded `calculate_characteristics` recursion

**Date**: 2026-08-02
**Reviewer**: primitive-impl-reviewer (Opus)
**Branch**: `feat/pb-dx19-the-unbounded-characteristics-recursion-oos-sim2-6-h` (`ee7a55b4`, `a0d977e5`, `79b94a58`)
**CR Rules**: 604.2, 603.4, 613.1 (613.1d/f/g), 613.4c, 613.8 (613.8b/c), 702.73a, 708.2 (708.2a), 712.8d/e, 729.2a, 608.2h, 208.1
**Engine files reviewed**: `crates/engine/src/effects/mod.rs` (`check_static_condition`, `check_condition`, `matches_filter`, `check_has_counter_type`, `resolve_amount`, the CR 608.2h substitution block), `crates/engine/src/rules/layers.rs` (`calculate_characteristics`, `is_effect_active`, `apply_layer_modification`, `resolve_cda_amount`)
**Test files reviewed**: `crates/engine/tests/primitives/pb_dx19_characteristics_recursion.rs` (589 lines, 10 tests), `crates/engine/tests/rules/static_grants.rs` (`archangel_battlefield` + 2 tests)
**Card defs reviewed**: `greymond_avacyns_stalwart` (edited), `indomitable_archangel`, `blinkmoth_nexus`, `inkmoth_nexus`, `mox_opal`, `mox_jasper`, `inventors_fair`, `bloodline_keeper`, `garruks_uprising`, `jadar_ghoulcaller_of_nephalia`, `dwynen_s_elite`, `scute_swarm`, `dispatch`, `docent_of_perfection`, `omnath_locus_of_the_roil`, `revel_in_riches`, `hellkite_tyrant`, `growing_rites_of_itlimoc`, `dragonmaster_outcast`, `thaumatic_compass`, `birthing_ritual`, `emeria_the_sky_ruin`, `ophiomancer`, `case_of_the_locked_hothouse`, `stubborn_denial`, `dawnstrike_vanguard`, `changeling_outcast`, `universal_automaton` (28 defs; 1 edited, 27 read to establish blast radius)

## Verdict: needs-fix

The recursion fix is **correct and sufficient for termination** — I traced every hop myself and confirm
there is no surviving edge from `check_static_condition`'s `YouControlNOrMoreWithFilter` arm back into
`calculate_characteristics`, including through the combinator route. The arithmetic conversions are real
and the six P/T probes discriminate. The `static_grants.rs` repair is exactly right. **But the batch
changed a shared evaluator and reasoned about only one of its five call paths.** The same arm serves
`activation_condition`, `intervening_if` and `Effect::Conditional`, which are **not** on the
`is_effect_active` path, never had the recursion hazard, and were **CR-correct before this batch**.
They are now wrong, on ordinary boards, on `Complete` deck-legal cards, with zero termination benefit —
`Garruk's Uprising` + a creature pumped to power 4 by +1/+1 counters no longer draws a card. That is
Finding 1, and it is HIGH. Finding 2 is that the deviation is understated even on the layer path: base
characteristics also drop Changeling (CR 702.73a) and the transformed-DFC face, and they *add* a false
positive for face-down permanents (CR 708.2a), none of which the comment, the pinned test, the greymond
note or `OOS-DX19-2` mention. The remaining findings are smaller: two counter `as i32` widenings on P/T
paths survive the very hardening pass whose own durable lesson says not to leave them, the promised
`devilish_valet` card-level test is absent, and the `memory/decisions.md` entry the plan mandated was
never written.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `effects/mod.rs:10303` | **The base-characteristics read reaches four call paths, three of which never had the hazard and were correct before.** `check_condition:9953` delegates this variant to `check_static_condition`, so every `activation_condition` (`abilities.rs:260`, `mana.rs:330`), every `intervening_if` (`abilities.rs:10358`) and every `Effect::Conditional` now reads printed characteristics. `garruks_uprising` (Complete) stops drawing off a counter-pumped creature. **Fix:** scope the base read to the layer path — pass a flag or use a re-entrancy guard set by `calculate_characteristics` — and keep `expect_characteristics` for the non-layer callers. |
| 2 | MEDIUM | `effects/mod.rs:10283-10302` | **The documented deviation names one of five divergences and misses the only one that produces a FALSE positive.** Base characteristics also omit Changeling (CR 702.73a), the transformed-DFC back face (CR 712.8d/e), meld (712.8g) and the mutate topmost component (729.2a), and they retain the real card's types for a **face-down** permanent, which CR 708.2a says is a colorless 2/2 creature with no other types. **Fix:** enumerate all five in the comment, in `OOS-DX19-2`, and in the greymond note; add a face-down false-positive pin next to the Nexus pin. |
| 3 | MEDIUM | `layers.rs:2418`, `effects/mod.rs:8342` | **Two `u32 -> i32` counter widenings on P/T write paths survive the pass whose stated purpose was to close exactly that class.** `resolve_cda_amount`'s `CounterCount` and `resolve_amount`'s `CounterCount` feed `SetPtDynamic`/`ModifyPowerDynamic` -> `ModifyPower`. Also unfixed: `EffectAmount::Sum` bare `+` (`layers.rs:2445-2446`), `.sum()` (`:2409`, `:2439`), `.count() as i32` (`:2359`, `:2381`, `:2518`, `:2546`), `mana_value as i32` (`:1673`). **Fix:** convert the two `CounterCount` casts to `try_into().unwrap_or(i32::MAX)` and `Sum` to `saturating_add`; state the rest as deliberately out of scope. |
| 4 | LOW | `effects/mod.rs:10303-10308` | **`exclude_self` was not reordered first**, though plan §4.1/§4.2 specified it and the verification checklist lists it as an acceptance item. Behaviour-neutral; the plan's own argument was legibility only. **Fix:** apply the reorder or strike the checklist item with a reason. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | **HIGH** | `garruks_uprising.rs:35-42` | **Complete; live-wrong after this batch.** `intervening_if` uses `min_power: Some(4)`, and P/T lives in Layer 7. **Fix:** covered by Engine Finding 1; add a pumped-creature test either way. |
| 2 | **HIGH** | `bloodline_keeper.rs:69`, `mox_opal.rs:29`, `inventors_fair.rs:31,94` | **Complete; live-wrong after this batch** on changeling / animated-artifact boards. **Fix:** Engine Finding 1. |
| 3 | MEDIUM | `jadar_ghoulcaller_of_nephalia.rs:44-47` | **In-def comment is now false.** It states the check runs against LAYER-RESOLVED characteristics "so a Humility-style effect that strips Decayed correctly re-enables the trigger". It no longer does. **Fix:** correct the comment (and the behaviour, via Engine Finding 1). |
| 4 | LOW | `greymond_avacyns_stalwart.rs:37-48` | Note says the cost is "a Human created by another continuous effect's type change is not counted"; it also drops changelings (a changeling IS a Human, CR 702.73a) and would falsely count a face-down permanent. **Fix:** widen the sentence per Engine Finding 2. |

## Test Findings

| # | Severity | File | Description |
|---|----------|------|-------------|
| 1 | MEDIUM | `pb_dx19_characteristics_recursion.rs` | **The plan's §8.5 card-level vehicle `devilish_valet_doubling_saturates` is absent.** Plan called it "the card-level proof that the seed's 'silently wraps to negative power' claim was real"; the claim now rests on synthetic `Fixed(i32::MAX)` probes only. **Fix:** add it, or record in the close-out that the card-level proof was dropped. |
| 2 | LOW | `pb_dx19_characteristics_recursion.rs:431-435` | Doc says "the amount is the creature's own power"; the amount is `EffectAmount::Fixed(i32::MAX)`. **Fix:** reword. |
| 3 | LOW | `pb_dx19_characteristics_recursion.rs:550` | "exactly as blinkmoth_nexus does" — the layer and modification match, but the def uses `EffectFilter::Source` + `UntilEndOfTurn`, the test `SingleObject` + `WhileSourceOnBattlefield`. Immaterial to the assertion; the word "exactly" is not earned. **Fix:** say "the same Layer-4 modification as". |
| 4 | MEDIUM | — | **No test pins the non-layer paths.** `pb_dx3_stale_blocker_notes.rs:377` uses a base 4/4, so the Garruk regression is invisible to the suite. **Fix:** add a counter-pumped-creature test for `garruks_uprising`'s intervening-if. |

## Documentation / Bookkeeping Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `memory/decisions.md` | **The P/T ceiling deviation entry plan §8.4 mandated is not there.** §8.4 argued at length for `decisions.md` over `engine-invariants.md`; only the `layers.rs` doc comment landed. Checklist item unmet. **Fix:** write the entry. |
| 2 | LOW | `CLAUDE.md:246-249`, `memory/workstream-state.md:921-924` | **A false claim, self-contradicted three lines later in CLAUDE.md.** "all 97 `condition: Some(..)` occurrences enumerated; every one is an `activation_condition`/`unless_condition`/`intervening_if`, none on the `is_effect_active` path" — `indomitable_archangel`, `serra_ascendant`, `bloodghast`, `dragonlord_ojutai`, `iroas`, `athreos`, `purphoros`, `skyhunter_strike_force`, `beastmaster_ascension`, `nadaar`, `arixmethes`, `quest_for_the_goblin_lord`, `radha`, `razorkin_needlehead`, `triumphant_adventurer` are all `ContinuousEffectDef.condition`. `docs/audits/decision-point-audit.md:1085` has the correct 57/17 framing. **Fix:** delete the false sentence from both files. |
| 3 | LOW | `CLAUDE.md:237-240` | "sixteen edits ... ten `+=` sites, six negations, and two `as i32` counter widenings" sums to **18** and omits the `saturating_sub`. The real total is 19 (16 in `layers.rs` + 3 in `effects/mod.rs`). **Fix:** restate as "16 in `layers.rs` plus 3 at the CR 608.2h substitution site". |
| 4 | LOW | `CLAUDE.md:224-225,234-237`; handoff §"Claims" | **`OOS-DP3-9`/`OOS-M11-3`'s stack-overflow half is declared closed from one 15-game, single-seed A/B without the backtrace classification plan §10.4 trap 1 required.** The post-fix arm's 15/15 is good evidence; the pre-fix arm's overflows were never shown to carry the four-hop cycle, which is the discriminator the plan itself insisted on because `OOS-M11-3` predates this defect. **Fix:** downgrade to "closed for the population sampled" or attach the backtrace. |
| 5 | LOW | `memory/primitive-wip.md` | Still headed "Primitive batch WIP — PB-DX6". **Fix:** roll to PB-DX19. |

### Finding Details

#### Engine Finding 1: the fix changes three call paths that never had the hazard

**Severity**: HIGH
**File**: `crates/engine/src/effects/mod.rs:10303`
**CR Rules**: 603.4, 613.1d, 613.1g, 613.4c, 702.73a; Garruk's Uprising ruling 2024-11-08
**Issue**:

`Condition::YouControlNOrMoreWithFilter` is not evaluated only from the layer path. `check_condition`
routes it back to `check_static_condition` at `effects/mod.rs:9953`, and `check_condition` is the entry
point for:

| consumer | entry point | previously | now |
|---|---|---|---|
| `activation_condition` on an activated ability (CR 602.5b) | `rules/abilities.rs:260` -> `check_condition` | layer-resolved (correct) | base (wrong) |
| `activation_condition` on a lowered mana ability | `rules/mana.rs:330` -> `check_condition` | layer-resolved (correct) | base (wrong) |
| `intervening_if` (CR 603.4), queue time and resolution | `rules/abilities.rs:10358` `carddef_intervening_if_holds_at_queue_time` -> `check_condition` | layer-resolved (correct) | base (wrong) |
| `Effect::Conditional` at resolution | `check_condition` | layer-resolved (correct) | base (wrong) |
| `ContinuousEffectDef.condition` | `layers.rs:575` `is_effect_active` -> `check_static_condition` | **SIGABRT** | base (wrong, but terminating) |

Only the last row had the recursion. On the other four, `expect_characteristics` recursed **only** if a
conditional continuous effect of this variant was simultaneously in `state.continuous_effects` — i.e.
only if `indomitable_archangel` was on the battlefield, in which case the game was already dead. On every
board without the Archangel, those four paths were terminating **and CR-correct**, and this batch made
them wrong for nothing.

Affected `Complete` defs, all verified by reading the def:

- `garruks_uprising.rs:35-42` (`completeness: Complete`, `:78`) — `intervening_if`, `min_power: Some(4)`.
- `mox_opal.rs:29` (Complete by derive) — Metalcraft activation.
- `inventors_fair.rs:31` and `:94` (`completeness: Complete`, `:106`) — Metalcraft intervening-if and activation.
- `bloodline_keeper.rs:69` (`completeness: Complete`, `:130`) — five-or-more-Vampires activation.
- Plus `hellkite_tyrant`, `dwynen_s_elite`, `revel_in_riches`, `growing_rites_of_itlimoc`, `dragonmaster_outcast`, `thaumatic_compass`, `birthing_ritual`, `emeria_the_sky_ruin`, `jadar` (`intervening_if`) and `scute_swarm`, `dispatch`, `docent_of_perfection`, `omnath_locus_of_the_roil`, `ophiomancer`, `case_of_the_locked_hothouse` (`Effect::Conditional`).

**Concrete failure scenario A (no exotic cards at all).**
State: P1 controls a 2/2 creature with two `+1/+1` counters. P1 casts Garruk's Uprising.
CR: `calculate_characteristics` applies the counters in Layer 7c (CR 613.1g / 613.4c), so the creature's
power is 4; CR 603.4's intervening-if is true; the trigger goes on the stack and P1 draws a card.
Engine post-PB-DX19: `carddef_intervening_if_holds_at_queue_time` -> `check_condition` ->
`check_static_condition` -> `matches_filter(&obj.characteristics, filter)` reads printed power 2 ->
`min_power: Some(4)` fails -> count 0 -> **the trigger never queues and no card is drawn**.
Engine pre-PB-DX19: correct. The card's own ruling (2024-11-08) makes the layer-resolved read explicit:
"If one or more static abilities that apply to a creature entering change its power, those abilities are
considered."

**Concrete failure scenario B (two `Complete` cards, no interaction).**
State: P1 controls Bloodline Keeper, three base-Vampires, and Universal Automaton (`completeness: Complete`,
`:29`) or Changeling Outcast (Complete by derive). CR 702.73a: Changeling is a CDA, "this object is every
creature type", working everywhere — so P1 controls five Vampires and may pay `{B}` to transform.
Engine post-PB-DX19: `matches_filter` reads `obj.characteristics.subtypes`, which for a changeling is just
`Shapeshifter` — the Changeling expansion is applied inside `calculate_characteristics`
(`layers.rs:255-259`), not in base characteristics. Count 4 -> **activation rejected**.

**Concrete failure scenario C (false positive, CR 708.2a).**
State: P1 controls Mox Opal, one other artifact, and a face-down manifested card whose real card is an
artifact (`cryptic_coat` manifests the top card of the library — any card type).
CR 708.2a: the face-down permanent is a 2/2 colorless creature with no subtypes and **no other card
types**, so P1 controls two artifacts and Mox Opal cannot be activated.
Engine post-PB-DX19: `obj.characteristics` still holds the real card's types (the face-down override at
`layers.rs:219-240` runs on the local `chars` clone inside `calculate_characteristics`, which is why the
override exists at all), so the count is three and **the ability is wrongly available**.

**Fix**: make the base read a property of the *call path*, not of the *variant*. Options, in order of
preference:
1. Give `check_static_condition` a `LayerPath`/`ResolvePath` discriminator (or a
   `matches_filter_for_path` helper) and pass the layer flavour only from `is_effect_active`;
   `check_condition:9953` keeps `expect_characteristics`.
2. Implement `OOS-DX19-4`'s `thread_local` re-entrancy flag now: `calculate_characteristics` sets it,
   `expect_characteristics` falls back to base only when it is set. One guard fixes this arm, the ten
   `OOS-DX19-1` siblings, and preserves every non-layer path's correctness.
Either way, add the discriminating tests from scenarios A and B, and re-scope `OOS-DX19-2` to the layer
path only.

#### Engine Finding 2: the deviation is understated by four divergences and one false positive

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:10283-10302`; mirrored in `greymond_avacyns_stalwart.rs:44-47`,
`docs/audits/decision-point-audit.md:1086` (`OOS-DX19-2`), and
`pb_dx19_characteristics_recursion.rs:505-521`
**CR Rules**: 702.73a, 708.2a, 712.8d/e, 712.8g, 729.2a, 613.1a-f
**Issue**: `calculate_characteristics` does five things to `chars` before the layer loop that
`obj.characteristics` does not contain:

| `layers.rs` | what it does | CR | direction of the error under a base read |
|---|---|---|---|
| `:97-139` | transformed DFC -> back-face name/types/subtypes/P/T | 712.8d/e | misses the back face; **counts the front face instead** |
| `:149-204` | melded permanent -> melded back face | 712.8g | same |
| `:219-240` | face-down -> 2/2 colorless creature, no subtypes, no other types | **708.2a** | **FALSE POSITIVE** — counts the hidden real card |
| `:246-248` | merged/mutate -> topmost component's characteristics | 729.2a | counts the wrong component |
| `:255-259` | Changeling -> every creature type | **702.73a** | misses every changeling on subtype filters |

Plus the whole layer loop (Layers 1-6), of which the comment names only Layer 4 `AddCardTypes`.

The face-down row is qualitatively different from all the others and from the documented Nexus case: it
makes the condition **more** true than CR allows, and it does so by reading a hidden card's real
characteristics into a publicly-observable count. That is a category the batch never considered.

**Fix**: enumerate all five in the fix-site comment and in `OOS-DX19-2`; add a face-down pin next to
`deviation_animated_nexus_does_not_count_toward_metalcraft`; widen the greymond note's one-clause
description.

#### Engine Finding 3: the counter-widening class is half-closed

**Severity**: MEDIUM
**Files**: `crates/engine/src/rules/layers.rs:2418`, `crates/engine/src/effects/mod.rs:8342`
**Issue**: the batch's own durable lesson #3 (`workstream-state.md:880-884`) reads: "If a hardening pass
converts `+=` to `saturating_add` and leaves the `as` casts, it has hardened the sites that were already
loud and skipped the silent one." Two `as` casts of exactly that shape remain on P/T write paths:

- `layers.rs:2418` — `resolve_cda_amount`'s `EffectAmount::CounterCount`:
  `.and_then(|obj| obj.counters.get(counter).copied()).unwrap_or(0) as i32`. Feeds
  `SetPtDynamic` / `SetBothDynamic` / `Modify*Dynamic` directly (`layers.rs:1667`, `:1690`, `:1734`,
  `:1753`, `:1768`), i.e. every "power equal to the number of counters on it" CDA.
- `effects/mod.rs:8342` — `resolve_amount`'s `CounterCount`:
  `.map(|obj| *obj.counters.get(counter).unwrap_or(&0) as i32)`. Feeds the CR 608.2h substitution at
  `:3892-3913` and thus `ModifyPower(v)`.

`counters` is `OrdMap<CounterType, u32>`; both wrap to a negative modifier above `2^31-1`, in every
profile, exactly as the fixed sites did. Also unconverted, in decreasing order of interest:
`EffectAmount::Sum`'s bare `+` (`layers.rs:2445-2446`), the two `.sum()` accumulators (`:2409`, `:2439`),
four `.count() as i32` (`:2359`, `:2381`, `:2518`, `:2546`), and `mana_value as i32` (`:1673`).
All are unreachable in a real game; so was the site the batch did fix, which it described honestly as
"hardening, not a repair of an observed defect" (plan §11 risks). The inconsistency, not the reachability,
is the finding.

**Fix**: convert the two `CounterCount` casts and `Sum`; add one sentence to
`apply_layer_modification`'s deviation doc listing what was deliberately left.

## Verification of the review brief's six specific questions

**1. Is the base read sufficient to terminate?** **Yes.** Traced in full:
`matches_filter` (`effects/mod.rs:9540-9667`) reads only the `&Characteristics` it is handed — every
branch touches `chars.power/toughness/card_types/keywords/colors/supertypes/subtypes/name/mana_cost`
and nothing else; it takes no `&GameState` and cannot reach the layer system.
`check_has_counter_type` (`:9529-9537`) reads `obj.counters` only. `is_effect_active`
(`layers.rs:518-585`) reaches `check_static_condition` and nothing else.
`resolve_cda_amount` (`layers.rs:2325-2578`), the other function called from inside the layer loop, is
base-characteristics throughout with a `_ => 0` catch-all. No surviving edge.

**2. The combinator hole.** **Covered.** `check_static_condition`'s `_` arm builds a minimal
`EffectContext` and calls `check_condition`; `Not` (`:9775`), `Or` (`:9784`) and `And` (`:9976`) recurse
into `check_condition`, and `check_condition`'s `YouControlNOrMoreWithFilter` arm (`:9953`) delegates
straight back to the **fixed** `check_static_condition` arm. So
`Not(YouControlNOrMoreWithFilter{..})` as a `ContinuousEffectDef.condition` terminates.

**3. §4.3's latency claim — re-verified independently, and it holds.** I enumerated every
`condition: Some(` in `crates/card-defs/` and every `Condition::` occurrence of the ten sibling variants.
Result: all ten appear only as `unless_condition` (the check/battle/slow/fast-land family, `minas_tirith`,
`shifting_woodland`, `arena_of_glory`, `spymasters_vault`, `mistrise_village`), `activation_condition`
(`blazemire_verge`, `wastewood_verge`, `gloomlake_verge`, `bleachbone_verge`, `tainted_field`,
`tainted_isle`, `tainted_wood`, `temple_of_the_false_god`, `mox_jasper`), `intervening_if`
(`land_tax`, `tatyova_steward_of_tides`), a `Box` inside `Or`/`Not` on those same fields
(`temple_of_the_dragon_queen`, `den_of_the_bugbear`), or a bare non-`Option` `Effect::Conditional`
field (`stubborn_denial:24`). **None** is a `ContinuousEffectDef.condition`. The 15 defs that do put a
condition on a `ContinuousEffectDef` use `IsYourTurn`, `SourceIsUntapped`, `SourceHasCounters`,
`ControllerLifeAtLeast`, `OpponentLifeAtMost`, `CompletedADungeon`, `YouControlYourCommander`,
`DevotionToColorsLessThan`, `ControllerGainedLifeThisTurn`, `SourceIsSolved`, and
`YouControlNOrMoreWithFilter` (Archangel alone). Engine source registers `condition: None` at every
`ContinuousEffect` construction site; `crates/engine/tests/rules/conditional_statics.rs` uses only
non-recursive variants. **No live counterexample. `OOS-DX19-1` is correctly scoped and correctly
described in the registry** — note that the *registry* text (`decision-point-audit.md:1085`) is precise
and the *CLAUDE.md/workstream-state* restatement is not (Doc Finding 2).

**4. Correctness of the deviation.** CR 613.1d verified by MCP: "Layer 4: Type-changing effects are
applied." CR 604.2 verified: it is about static abilities creating continuous effects and does **not**
literally say the condition is evaluated against layer-resolved state — the real support is CR 613.1
("the values of an object's characteristics are determined by ... then all applicable continuous effects
are applied"). The comment's `CR 604.2 / CR 613.1d` pairing is inherited from the pre-existing code and
is acceptable shorthand, not a miscite worth a finding. CR 613.8b verified verbatim, including the
dependency-loop -> timestamp-order sentence the comment quotes.
`blinkmoth_nexus.rs:39-51` and `inkmoth_nexus.rs:40-52` do produce
`EffectLayer::TypeChange` + `LayerModification::AddCardTypes([Artifact, Creature])`; both are lands with
`mana_cost: None`, both `Complete` by derive (`..Default::default()`, no `completeness` key). The test's
hand-built effect matches on layer and modification (it differs on `filter`/`duration`, immaterial —
Test Finding 3). **The test genuinely flips**: base count is 2 (`P1 Artifact 0`, `P1 Artifact 1`; the
Archangel is an Angel), layer-resolved count is 3, so a CR 613.8b fixpoint turns Metalcraft on and
`P1 Artifact 0` gains Shroud, failing the `!contains(Shroud)` assertion. The precondition assertion at
`:571-575` prevents it from passing for the unrelated reason of the animation silently not working.

**5. Do the tests discriminate?** Yes, all six P/T probes, checked by hand:
- `counter_path_saturates...` — `i32::MAX` power + 5 counters; pre-fix `*p += net` panics under the
  default `overflow-checks` (the workspace `Cargo.toml` declares only `[profile.fuzz]`, so cargo's dev
  defaults apply and `test` inherits dev — plan §1.5 verified, and I re-confirmed there is no
  `[profile.dev]`/`[profile.test]` block).
- `counter_widening_saturates_instead_of_wrapping_negative` — **tests what its name claims.**
  `3_000_000_000u32 as i32` = `-1294967296`; `2 + (-1294967296)` = `-1294967294`, which is exactly the
  quoted pre-fix `left:` value. Post-fix `try_into().unwrap_or(i32::MAX)` -> `2.saturating_add(i32::MAX)`
  -> `i32::MAX`. It is correctly identified as the only probe that fails by assertion rather than panic.
- `modify_both_arm_saturates_instead_of_overflowing` / `..._downward` — `ModifyBoth(±100)` on
  `i32::MAX-1` / `i32::MIN+1`; both hit `layers.rs:1705-1712`.
- `dynamic_arm_saturates_instead_of_overflowing` — **the correction is real**: it uses
  `LayerModification::ModifyPowerDynamic { amount: Fixed(i32::MAX), negate: false }`, reaching
  `layers.rs:1747-1759`, not the concrete `ModifyPower` arm. `is_cda: false` is fine here because the
  effect is pushed directly onto `continuous_effects` and never passes the `effects/mod.rs:3892`
  substitution. (Its doc sentence about "the creature's own power" is inaccurate — Test Finding 2.)
- `dynamic_arm_negation_saturates` — **tests what its name claims.** `Fixed(i32::MIN)` + `negate: true`
  hits `let delta = if *negate { raw.saturating_neg() }` at `:1755`; pre-fix `-i32::MIN` panics with
  "attempt to negate with overflow", a site none of the other probes reaches. Post-fix
  `saturating_neg()` -> `i32::MAX`, then `3.saturating_add(i32::MAX)` -> `i32::MAX`.

**6. The `static_grants.rs` repair.** Correct. `archangel_battlefield` (`:723-766`) builds the object
from the real def via `enrich_spec_from_def` + `with_card_id` and registers through
`rules::replacement::register_static_continuous_effects`; the hand-built `condition: None` block is gone.
The three original assertions survive unchanged in meaning (`:797-816`: P1 artifact has Shroud — filter
positive; P1 creature does not — filter axis; P2 artifact does not — controller axis).
`test_artifacts_you_control_shroud_requires_metalcraft` (`:827-838`) is genuinely discriminating: with
`artifact_count = 2` the helper builds `P1 Artifact` + one `P1 Filler`, base count 2 < 3, and the old
`condition: None` shape would have granted Shroud unconditionally and failed it.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 613.1 (base + layers) | Yes | Yes | the fix deliberately reads pre-613.1 values; see Finding 2 |
| 613.1d (Layer 4) | **Deviates, documented** | Yes | `deviation_animated_nexus_does_not_count_toward_metalcraft` |
| 613.1f (Layer 6 grant) | Yes | Yes | `recursion_metalcraft_on_grants_shroud_and_terminates`, `static_grants.rs:790` |
| 613.1g / 613.4c (Layer 7) | Yes | Yes | four `Modify*` probes + counter path |
| 613.8b/c (fixpoint) | No — deferred | n/a | `OOS-DX19-2`; CR text verified verbatim against the comment |
| 604.2 (conditional statics) | Yes | Yes | but see the citation note in question 4 |
| 603.4 (intervening-if) | **Regressed** | **No** | Finding 1 scenario A; `pb_dx3` uses a base 4/4 and cannot see it |
| 602.5b (activate only if) | **Regressed** | **No** | Finding 1 scenarios B/C |
| 702.73a (Changeling) | **Regressed on this arm** | No | Finding 1 scenario B / Finding 2 |
| 708.2a (face-down) | **False positive on this arm** | No | Finding 2 |
| 712.8d/e, 712.8g, 729.2a | Not considered | No | Finding 2 |
| 608.2h (lock-in) | Yes | Indirectly | three `saturating_neg` at `effects/mod.rs:3902/3907/3912` — beyond plan §8, a good catch |
| 208.1 (no P/T ceiling) | Deviates, documented | Yes | `layers.rs:1463-1490`; `memory/decisions.md` entry missing (Doc Finding 1) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `indomitable_archangel` | Yes (MCP-verified: `{2}{W}{W}` Angel 4/4, Flying, Metalcraft/3) | 0 | Yes on the layer path | unchanged by the batch; no longer aborts |
| `greymond_avacyns_stalwart` | Yes; still correctly `inert` | 3 (unchanged, `:6`, `:34`, `:35`) | n/a (inert) | note-string-only edit as required; understates the deviation (Card Finding 4) |
| `blinkmoth_nexus`, `inkmoth_nexus` | Yes | 0 | Yes in themselves | the deviation's live carriers; correctly identified |
| `garruks_uprising` | Yes (MCP-verified, incl. the 2024-11-08 ruling) | 0 | **No — regressed** | Card Finding 1 |
| `mox_opal`, `inventors_fair`, `bloodline_keeper` | Yes | 0 | **No — regressed** | Card Finding 2 |
| `jadar_ghoulcaller_of_nephalia` | Yes | 0 | No (Humility case) + false comment | Card Finding 3 |
| `dwynen_s_elite`, `hellkite_tyrant`, `revel_in_riches`, `growing_rites_of_itlimoc`, `dragonmaster_outcast`, `thaumatic_compass`, `birthing_ritual`, `emeria_the_sky_ruin` | Yes | 0 | Degraded (subtype/type/count filters now base) | collateral of Finding 1 |
| `scute_swarm`, `dispatch`, `docent_of_perfection`, `omnath_locus_of_the_roil`, `ophiomancer`, `case_of_the_locked_hothouse` | Yes | 0 | Degraded | `Effect::Conditional` path |
| `mox_jasper`, `stubborn_denial` | Yes | 0 | Unaffected | use `YouControlPermanent`, a different arm (still layer-resolved) |
| `dawnstrike_vanguard` | Yes; correctly `partial` | 1 (pre-existing) | n/a | its blocked note is about `is_tapped`, unaffected |

## Claims I judge overstated

1. `CLAUDE.md:246-249` / `workstream-state.md:921-924`: *"all 97 `condition: Some(..)` occurrences in
   the defs were enumerated and every one is an `activation_condition` / `unless_condition` /
   `intervening_if`, none on the `is_effect_active` path."* **False as written** — at least 15 defs put a
   condition on a `ContinuousEffectDef`, which is the `is_effect_active` path, and one of them is the
   card this batch exists for. The registry row (`decision-point-audit.md:1085`) states it correctly and
   even flags the trap ("state it that way and no wider"); the summary docs then restate it wrongly.
2. `workstream-state.md:914-919` and `CLAUDE.md:241-245`: *"The fix has a known, live cost"* framed as a
   single two-def Nexus interaction. The cost is at least five divergences plus a false-positive class
   (Finding 2) and it lands on paths that were previously correct (Finding 1). The word "known" is doing
   more work than the analysis supports.
3. `CLAUDE.md:224-225`: *"OOS-DP3-9 / OOS-M11-3's stack-overflow half closes with them, measured."* The
   measurement is real and impressive (0/15 -> 15/15), but the plan's own §10.4 trap 1 required that
   every pre-fix overflow be classified by backtrace before merging that seed's half into this batch,
   and no backtrace classification is recorded. Doc Finding 4.
4. Plan §3.3's table, reproduced into the comment: *"An artifact animated into a creature — still
   Metalcraft fuel? yes ... unaffected"* is true, but the table's fourth column ("live in corpus?")
   answers "no corpus def" for the removal and copy rows without considering face-down, DFC, meld, merge
   or Changeling, all of which are in the corpus.

## What the batch got right (worth preserving in any fix)

- The one-line termination fix is the correct call for the layer path, and the argument in §3 for
  deferring the CR 613.8b fixpoint is sound.
- The comment rewrite is a genuine improvement: it states the mechanism, names the disproved invariant,
  and refuses the "performance" framing that hid the bug for 4.5 months.
- `static_grants.rs` is repaired in exactly the way the finding demanded — real def, production
  registrar, plus the below-threshold case the old shape could not express.
- The `effects/mod.rs:3902/3907/3912` `saturating_neg` conversions were **not** in the plan and close a
  real hole one hop upstream of the layer fix.
- Every pre-fix failure was watched by an executed, compiling revert, and the two reverts were run
  *independently* (P/T-only and recursion-only) so neither fix carries the other's evidence. That is
  better discipline than the plan asked for.
