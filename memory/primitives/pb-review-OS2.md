# Primitive Batch Review: PB-OS2 — optional-cost sacrifice power (EF-EF1-A)

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**Commit**: `95c8a632` on `feat/pb-os2-optional-cost-sacrifice-power-ef-ef1-a-maypaytheneffe`
**CR Rules**: 608.2h, 608.2i, 603.10a, 608.2c, 613.1d, 701.21a, 118.12, 109.1
**Engine files reviewed**: `crates/engine/src/effects/mod.rs` (`MayPayThenEffect` executor 3393-3426; `pay_optional_cost` 8111-8175; `try_pay_optional_cost` 8182-8195; `can_pay_optional_cost` scratch probe 8082-8092; `sacrifice_permanents_for_player` capture site 7844-7901; `PowerOfSacrificedCreature` resolve 7380-7384; `GainLife`/`DrawCards` clamp 562-579, 652-662)
**Card defs reviewed**: 1 flip — `disciple_of_freyalise.rs`; roster spot-checks: `springbloom_druid.rs`, `birthing_ritual.rs`, `ziatora_the_incinerator.rs`
**Tests reviewed**: `crates/engine/tests/primitives/pb_ef10_sacrifice_driven_amounts.rs` (3 new tests + `anthem_power_effect` helper)

## Verdict: clean

The engine change threads the already-layer-resolved, pre-zone-move `Vec<SacrificedCreatureLki>`
(captured inside `sacrifice_permanents_for_player` at 7869-7887, BEFORE `move_object_to_zone`) up
through `pay_optional_cost` → `try_pay_optional_cost` → the `Effect::MayPayThenEffect` executor,
which sets `ctx.sacrificed_creature_lki` / `ctx.sacrifice_fired` before running `then`. This is a
faithful mirror of the mandatory `Effect::SacrificePermanents` executor (3457-3488) and closes
EF-EF1-A. The change is CR-correct (608.2h/608.2i LKI look-back, 613.1d layer resolution), touches
only private unserialized fns + runtime `EffectContext` scratch (no PROTOCOL/HASH bump, correctly),
the `exclude_self` source threading and payer/controller rebind are preserved intact, and the card
def matches oracle text. Tests genuinely pin layer-resolution, correct-creature capture, and the
decline/no-leak path. Zero findings at any severity. Two informational NITs recorded below (neither
requires a fix; one is a pre-existing PB-EF10 cosmetic issue, not introduced here).

## Engine Change Findings

None.

## Card Definition Findings

None.

## Detailed verification against the task's correctness checks

**1. Ctx set BEFORE `then`, scoped correctly (no cross-contamination).** Confirmed
(3411-3422): on the `Some(sacrificed)` branch the executor sets `ctx.sacrifice_fired` and
`ctx.sacrificed_creature_lki` *before* `execute_effect_inner(state, then, ctx, events)`. On the
`None` (decline) branch ctx is left untouched — the decline test (`test_may_pay_sacrifice_declined_
no_capture_no_leak`) pins this by placing a sibling `GainLife{PowerOfSacrificedCreature}` after the
`MayPayThenEffect` in the same `Sequence` and asserting it reads 0, plus asserting
`ctx.sacrifice_fired == false`. That sibling assertion is a genuine decoy for "executor writes ctx
unconditionally" — it would fail if the writes were hoisted above the `if let Some`.
- **Multi-payer loop cross-contamination**: judged **defensively irrelevant**. The field is
  most-recent-sacrifice-wins and is NOT restored after the loop, identical to the mandatory
  `Effect::SacrificePermanents` executor. Each paying payer's own `then` reads its own just-captured
  LKI (overwrite happens immediately before that payer's `then`); a declining payer's `then` never
  runs. The only latent stale read is a *sibling effect placed AFTER the whole MayPayThenEffect* that
  reads `PowerOfSacrificedCreature` on the success path — see NIT-1. No card in the 1,798-def corpus
  exhibits that pattern (only two Complete cards use `MayPayThenEffect{Cost::Sacrifice}` at all;
  neither has such a sibling). Verdict: correct and consistent with the mandatory path.

**2. `exclude_self` + controller rebind preserved.** Confirmed. `try_pay_optional_cost(state, pid,
cost, Some(ctx.source), events)` still passes `Some(ctx.source)` (3408-3409), so
`Cost::Sacrifice(exclude_self:true)` excludes the source ("another creature", CR 109.1). The
`let original_controller = ctx.controller;` capture (3403), per-payer `ctx.controller = pid` (3411),
and `ctx.controller = original_controller;` restore after the loop (3425) are all intact — no
regression to the PB-EF1 threading the plan flagged.

**3. `Cost::Sequence` accumulation.** Confirmed (8154-8160): the `Cost::Sequence(costs)` arm builds
`acc`, extends it with each sub-cost's returned vec, and returns `acc` — a sequence containing a
sacrifice plus another cost correctly propagates the sac LKI. The `can_pay_optional_cost` scratch
probe's inner `pay_optional_cost` call (8089) correctly discards via `let _ =` (throwaway payability
simulation on a `state.clone()`).

**4. No wire/hash change.** Confirmed correct. Only two private, unserialized fn signatures widened
(`pay_optional_cost` → `Vec<SacrificedCreatureLki>`, `try_pay_optional_cost` → `Option<Vec<..>>`);
`EffectContext` is runtime resolution scratch (not hashed, not on the wire). `SacrificedCreatureLki`
and the ctx fields already existed (shipped by PB-EF10). Version sentinels unchanged: PROTOCOL == 18,
HASH == 55 (`pb_ef10_sacrifice_driven_amounts.rs:1595-1601`); `core/protocol_schema.rs` +
`core/hash_schema.rs` untouched. This is right.

**5. Test quality.** Strong.
- `test_may_pay_sacrifice_captures_layer_resolved_power` (DECOY): a `+2/+0` anthem makes Fodder's
  layer-resolved power 4 (base 2) and Decoy's 7 (base 5). Asserts life gained == 4 and cards drawn ==
  4 (layer-resolved, not base 2, not Decoy 7), Fodder in graveyard, Decoy still on battlefield. Pins
  BOTH layer-resolution (a result of 2 fails) AND correct-creature capture (7/5 fails). Runner
  attests it FAILS against the pre-fix engine (revert-and-rerun). Exercises the real pay path via
  `execute_effect`, not a hand-built ctx — the code under test.
- `test_may_pay_sacrifice_declined_no_capture_no_leak` (DECLINE + no-leak decoy): zero eligible
  targets (only the `exclude_self` source), asserts source not sacrificed, `then` does not run (0
  life, 0 draw), sibling reads 0, `sacrifice_fired == false`. Correctly passes both pre- and post-fix
  (it guards the new decline branch, not the main bug).
- `test_disciple_of_freyalise_front_face_gains_and_draws_power` (card integration): casts the real
  def, drains the ETB trigger, auto-pays the optional sacrifice of a 3/3 fodder, asserts +3 life / 3
  draws / fodder in graveyard / Disciple on battlefield. Proves DSL wiring end-to-end.
- All three cite CR sections (invariant #8): 613.1d/608.2h, 608.2c/118.12, 608.2h.

**Negative-power safety** (task-requested): `PowerOfSacrificedCreature` returns `l.power`
(`i32`, `unwrap_or(0)`); both `GainLife` (564) and `DrawCards` (654) clamp with
`resolve_amount(...).max(0)` before the `usize`/`u32` cast, so a 0-power or negative-power sacrifice
draws/gains 0 — no crash, no wrap. Confirmed.

## Card Def: disciple_of_freyalise.rs

Oracle (MCP-confirmed via ruling "Use the power of the sacrificed creature as it last existed on the
battlefield to determine the value of X" — an explicit LKI/look-back statement matching CR 608.2h/i).
Front face now wired as `Triggered{ WhenEntersBattlefield, MayPayThenEffect{ Cost::Sacrifice(creature,
exclude_self:true), payer: Controller, then: Sequence[GainLife{PowerOfSacrificedCreature},
DrawCards{PowerOfSacrificedCreature}] } }`. Verified:
- "you may sacrifice **another** creature" → `exclude_self: true`, `has_card_type: Some(Creature)`. ✓
- "If you do" → implicit `then`-arm gate (runs only on paid cost); no separate `Condition`. ✓ (CR 608.2c)
- "gain X life **and** draw X cards, X = that creature's power" → both `GainLife` and `DrawCards` use
  `PowerOfSacrificedCreature`. ✓
- Back face (Garden of Freyalise) untouched, still Complete (pay-3-life-or-tapped replacement + `{T}:
  Add {G}`). MDFC structure (`back_face: Some(CardFace{ color_indicator: None, ... })`) intact. ✓
- `completeness: Completeness::Complete`; header comment updated (removes "surviving blocker"
  language). 0 TODOs remaining. ✓

## Roster / regression spot-check

Full-corpus grep confirms the sweep conclusion:
- **MayPayThenEffect ∩ (Power/Toughness/ManaValue)OfSacrificedCreature** = exactly
  `disciple_of_freyalise.rs` and `birthing_ritual.rs`. Birthing Ritual stays `inert`/partial
  (blocked on the top-7 conditional-battlefield **dig**, OOS-EF10-1 / PB-OS8 — NOT the optional-cost
  power capture). Correct to leave.
- **`MayPayThenEffect{Cost::Sacrifice}`** = exactly `disciple_of_freyalise.rs` and
  `springbloom_druid.rs`. Springbloom's `then` is `SearchLibrary`+`Shuffle` (sacrifices a land) and
  reads NO sacrificed-creature amount and has no `SacrificeFired`-gated sibling — so the newly
  populated `ctx.sacrificed_creature_lki`/`sacrifice_fired` are harmless there. **No existing Complete
  card changes behavior in a wrong direction.** ✓
- **ziatora_the_incinerator** stays partial — its optional sacrifice is inside a *triggered* ability
  with a reflexive "when you do" and `Triggered` has no `may` field; a distinct gap, not EF-EF1-A.
  Correct to leave.
- The other `PowerOfSacrificedCreature` users (momentous_fall, life's_legacy, greater_good,
  eldritch_evolution, birthing_pod, altar_of_dementia, miren, diamond_valley) use the mandatory /
  activated-cost path, not `MayPayThenEffect`; unaffected by this change.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 608.2h/608.2i (LKI look-back) | Yes | Yes | decoy + integration; capture pre-zone-move at 7869-7887 |
| 603.10a (leaves-battlefield look-back) | Yes | Yes (via decoy) | layer-resolved power read as LKI |
| 613.1d (layer-resolved chars) | Yes | Yes | anthem decoy: 4 not base 2 |
| 608.2c (sequential "if you do") | Yes | Yes | decline test; implicit `then`-arm gate |
| 109.1 (exclude_self "another") | Yes (preserved) | Covered by PB-EF1 | `Some(ctx.source)` threaded |
| 118.12 (beneficial optional cost) | Yes | Yes | decline + decoy |
| 701.21a (sacrifice, not destruction) | Yes (pre-existing) | Yes | goes to graveyard |

## Card Def Summary

| Card | Oracle Match | TODOs | Game State Correct | Notes |
|------|-------------|-------|--------------------|-------|
| disciple_of_freyalise | Yes | 0 | Yes | front flipped to Complete; back untouched |

## Informational NITs (no fix required)

- **NIT-1 (latent, unexercised)**: after a *successful* optional sacrifice, the executor does not
  restore `ctx.sacrificed_creature_lki`/`sacrifice_fired` after the payer loop (most-recent-wins,
  documented in the plan's Risks section and the in-code comment). A hypothetical card with a sibling
  effect *after* a `MayPayThenEffect{Cost::Sacrifice}` in the same resolution that reads
  `PowerOfSacrificedCreature` would read the stale value. This exactly mirrors the mandatory
  `Effect::SacrificePermanents` executor and no card in the corpus triggers it. Not a bug; recorded
  only so a future card that introduces the pattern re-checks this scoping. The in-code comment's
  "no stale value leaks to sibling effects" phrasing is scoped correctly to the *decline* branch, but
  a reader could over-generalize it — optional clarifying edit, not required.
- **NIT-2 (pre-existing, not introduced here)**: the version-sentinel test's doc comment
  (`pb_ef10_sacrifice_driven_amounts.rs:1589-1601`) says "== 15" / "== 53" while the actual asserts
  are `18` / `55u8`. Stale comment carried over from PB-EF10; cosmetic; outside this batch's scope.

## Previous Findings

N/A — first review of PB-OS2.
