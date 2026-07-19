# Primitive Batch Plan: PB-OS6 — DFC flip-condition sub-batch (OOS-EF5-4 a/b/c/d/g)

**Generated**: 2026-07-19
**Primitive**: A three-primitive sub-batch that makes the *surviving* (non-transform) clause of
four roster DFCs expressible, so their already-shipped `TransformSelf` (PB-EF5) half can reach a
`Complete` def. Ships: (a) a `Condition` for "top card of your library is an instant or sorcery";
(b) a `Condition` for "you attacked with N or more creatures" backed by a captured per-turn
attacker count on `PlayerState`; (g) `Effect::RemoveFromCombat { target }` + a
`GameEvent::RemovedFromCombat` and a shared `remove_from_combat` helper factored out of
regeneration.
**CR Rules**: 506.4 (removal from combat), 508.1 (declare attackers / "whenever you attack"),
508.4 (put-onto-battlefield-attacking does not count), 603.4 (intervening-if), 614.1c (peek at top
of library), 701.28/712.18 (Transform).
**Cards affected**: 5 candidates → **3 flip to Complete** (delver_of_secrets partial→Complete,
thaumatic_compass partial→Complete, legions_landing NEW→Complete). 2 stay as-is with honest
status: growing_rites_of_itlimoc stays `partial` (d deferred to PB-OS8), westvale_abbey stays
UNAUTHORED (c deferred to new seed OOS-OS6-1).
**Dependencies**: PB-EF5 (`Effect::TransformSelf`) — present. PB-OS4b (face-aware ability
gathering) — present, so back-face abilities register.
**Deferred items from prior PBs**: OOS-EF5-4 (this batch). See §"Scope decisions" for the (c)/(d)
carry-forward.

---

## TODO sweep (roster-recall gate)

Ran the mandatory pre-existing-TODO sweep over `crates/card-defs/src/defs/` for each sub-primitive's
keywords:

- `TODO`/blocker comments naming "top card ... instant or sorcery" / Delver reveal → only
  `delver_of_secrets.rs` (already on roster).
- "attacked with ... creatures" / `WheneverYouAttack` count → only `legions_landing` (unauthored,
  on roster). No OTHER def carries a "attack with N creatures" transform TODO.
- multi-count sacrifice (`Sacrifice five creatures`) → `westvale_abbey` (unauthored) **plus
  `kellogg_dangerous_mind.rs`** ("'Sacrifice five Treasures:' — `Cost::Sacrifice(TargetFilter)` has
  no count field") — this is the SAME missing primitive as (c). Recorded under the OOS-OS6-1
  deferral below; both cards ride the deferred (c) primitive, not this batch.
- remove-from-combat → only `thaumatic_compass.rs` (on roster). No other def references a
  standalone combat-removal blocker.
- look-at-top-N-take-one → `growing_rites_of_itlimoc.rs` (on roster) + the OS8 family
  (`birthing_ritual` etc., out of scope). Deferred to PB-OS8.

**Sweep result**: 1 additional card surfaced (`kellogg_dangerous_mind`), and it is bound to the
DEFERRED (c) primitive, so it does not change this batch's shipped roster. No forced adds to the
three shipped primitives.

---

## Scope decisions (the two deferrals — read these first)

### (d) growing_rites_of_itlimoc — DEFER to PB-OS8 (recorded per AC 5069)

**Decision: DEFER.** Do NOT implement a minimal look-at-top-N-take-one effect in PB-OS6.
growing_rites_of_itlimoc stays `partial` (its end-step `TransformSelf` half already works; the ETB
"look at top four, may reveal a creature and put it into hand, rest to bottom" clause remains the
truthfully-marked blocker).

**Reasoning**: PB-OS8 (`oos-retriage-plan-2026-07-18.md` §3, ~line 363) is *designed* to ship the
general `Effect::LookAtTopThenPlace { count: EffectAmount, filter: TargetFilter, destination,
rest_to: BottomRandomOrder | Graveyard, optional: bool }`. growing_rites's ETB is a plain instance of
that family (count 4, filter creature, destination hand, rest_to bottom, optional). Building a
near-duplicate minimal effect here would (1) add a throwaway wire type to the SR-8 closure, forcing a
PROTOCOL bump now, and (2) force a SECOND PROTOCOL bump plus a migration/removal when OS8 lands the
general effect. Deferring keeps the wire surface minimal and lets growing_rites flip to Complete
inside PB-OS8 with zero throwaway. The only cost is growing_rites stays `partial` one more batch,
which is honest (its blocker is real).

**Action in this batch**: update growing_rites_of_itlimoc.rs's `TODO` comment and `completeness`
note to cite **PB-OS8 / `LookAtTopThenPlace`** as the closing primitive (currently they cite the
stale "OOS-EF5-4(f)"). No behavioral change; keep `partial`.

### (c) westvale_abbey — DEFER to a new dedicated seed OOS-OS6-1

**Decision: DEFER.** Do NOT implement a multi-count sacrifice *cost* in PB-OS6. westvale_abbey stays
UNAUTHORED (no def file — its honest current state).

**Reasoning (grounded in source)**: Westvale's transform ability is a normal activated ability
(CR 602, goes on the stack) whose cost is `{5}, {T}, Sacrifice five creatures`. A *correct*
implementation needs BOTH:
1. A count carried into the cost. `Cost::Sacrifice(TargetFilter)` (card_definition.rs:1249) has no
   count; `ActivationCost` (game_object.rs:294) carries `sacrifice_filter`/`sacrifice_exclude_self`
   but no count. Every `ActivationCost` construction site is a **fully-explicit struct literal with
   no `..Default::default()`** (verified: ~40 sites across `tests/` + 3 in card_definition.rs), so
   adding an `ActivationCost` field is a ~43-site mechanical edit (the same tax PB-EF1 paid for
   `sacrifice_exclude_self`).
2. A way for the player to supply **five** distinct ObjectIds. `Command::ActivateAbility`
   (command.rs:78) has only `sacrifice_target: Option<ObjectId>` (singular); the payment path
   (abilities.rs:934) validates exactly one. Supplying five requires a plural Command field, and
   `sacrifice_target` is referenced in **50 files** — a ~50-site mechanical edit + a `Command` wire
   reshape.

That is ~90 mechanical site edits and TWO extra wire-closure shape changes (`ActivationCost` on the
HASH side, `Command` on the PROTOCOL side) for a **single-card** yield. The one cheap alternative —
auto-selecting the five sacrifice victims deterministically — is rejected: the existing singular
`Cost::Sacrifice` path *requires* the caller to name the victim (errors if `sacrifice_target` is
None, abilities.rs:935), so auto-selecting cost fodder would break the engine's explicit-cost-choice
convention (a legal-but-suboptimal smell the batch guardrail warns against). Better to give the
multi-count sacrifice cost its own focused micro-PB.

**Action in this batch**: file seed **OOS-OS6-1** (below) capturing the full design so the follow-up
PB starts from a plan, and note that `kellogg_dangerous_mind` ("Sacrifice five Treasures") rides the
same primitive. westvale_abbey remains absent.

> **New seed to record in `oos-retriage-plan-2026-07-18.md` (and workstream-state "Last Handoff"):**
> **OOS-OS6-1 (capability) — multi-count sacrifice *activation cost*.** Add a count to the
> activated-ability sacrifice cost so "Sacrifice five creatures"/"Sacrifice five Treasures" is
> payable. Fix shape: `ActivationCost.sacrifice_count: u32` (default 1) + a plural
> `Command::ActivateAbility.sacrifice_targets: Vec<ObjectId>` (keep singular `sacrifice_target` for
> the count==1 fast path, or migrate) + a multi-target validation loop in `handle_activate_ability`
> (count == N, all distinct, all match the filter, `object_cant_be_sacrificed` honored). Candidates
> (2): `westvale_abbey` (new; back = Ormendahl, Profane Prince 9/7 Demon), `kellogg_dangerous_mind`
> (partial→Complete). New wire type (Command reshape) → PROTOCOL bump. ~90-site mechanical churn.

---

## Primitive Specification (the three SHIPPED sub-primitives)

### (a) delver_of_secrets — `Condition::TopCardIsInstantOrSorcery`

The upkeep flip is modeled the same way `heralds_horn.rs` models its top-card peek: an unconditional
`AtBeginningOfYourUpkeep` trigger whose effect is `Effect::Conditional { condition, if_true, if_false }`.
The "you may reveal … if an instant or sorcery is revealed, transform" clause is faithfully a
mandatory-if-true transform because revealing to transform is **strictly beneficial** (a 1/1 becomes a
3/2 flier with no downside), so optimal play always reveals — unlike Herald's Horn (marked
`known_wrong` because "put into hand" can be undesirable), Delver has no reason to decline, so
Conditional-in-effect is faithful and the def reaches **Complete**.

New engine type: `Condition::TopCardIsInstantOrSorcery` (unit). Evaluated by peeking the top of the
effect controller's library and reading printed card types (CR 400.2 — library cards use printed
characteristics; CR 614.1c — the peek is hidden info, deterministic engine sees all), mirroring
`Condition::TopCardIsCreatureOfChosenType` (effects/mod.rs:9011).

### (b) legions_landing — `Condition::YouAttackedWithNOrMore(u32)` + captured attacker count

`TriggerCondition::WheneverYouAttack` (card_definition.rs:3498) is a bare unit that fires once when
the controller declares ≥1 attacker (→ `TriggerEvent::ControllerAttacks`, abilities.rs:4147-4169).
It carries no count, and — critically — the shared lowering `build_face_ability_vectors`
(replay_harness.rs:3142-3166) hardcodes `intervening_if: None`, **dropping any DSL
`intervening_if`** on a WheneverYouAttack trigger. `check_intervening_if` (abilities.rs:8851) also
only accepts the limited runtime `InterveningIf` enum (2 variants), NOT the full DSL `Condition`.

Therefore the count gate MUST live inside the effect (same proven pattern as (a)/Herald's Horn), not
in `intervening_if`. Legion's Landing's attack trigger fires unconditionally on any attack and
self-gates via `Effect::Conditional { condition: Condition::YouAttackedWithNOrMore(3), if_true:
TransformSelf, if_false: Nothing }`.

The count is captured on `PlayerState`: a new `attackers_declared_this_turn: u32` set in
`handle_declare_attackers` (combat.rs:624-628, right where `attacked_this_turn = true` is set —
`attackers` in scope is exactly the declared set, all controlled by `player`) and reset in
`reset_turn_state` (turn_actions.rs:1463, next to `attacked_this_turn = false`). Per the Legion's
Landing rulings (confirmed via MCP): it counts only creatures **declared** as attackers (CR 508.4
tokens-entering-attacking do not count — `handle_declare_attackers` is the only setter, matching the
existing `attacked_this_turn` semantics), and it transforms even if those creatures later leave — the
captured count does not decrease, so the effect still sees ≥3 at resolution.

Fidelity note (acceptable, matches engine-wide convention): the trigger goes on the stack even when
you attack with <3 (then resolves to `Nothing`), rather than not triggering at all. This is the same
"trigger-always-fires, effect-self-gates" pattern the engine already uses for every top-card/peek
conditional trigger (Herald's Horn, delver). Produces correct game state (no transform with <3).

New engine type: `Condition::YouAttackedWithNOrMore(u32)`, evaluated against
`PlayerState.attackers_declared_this_turn` for the effect controller. `PlayerState` is inside
`GameState` (NOT on the Command/GameEvent wire), so the new field is a HASH-schema change only, not a
PROTOCOL one; only the `Condition` variant touches the PROTOCOL closure.

### (g) thaumatic_compass — `Effect::RemoveFromCombat { target }` (+ shared helper, + GameEvent)

The Spires of Orazca back face is `{T}: Untap target attacking creature an opponent controls and
remove it from combat.` The untap + `is_attacking`/`controller: Opponent` target already exist in the
def; only the combat-removal clause has no primitive. CR 506.4: "A permanent is removed from combat
if … an effect specifically removes it from combat … A creature that's removed from combat stops
being an attacking, blocking, blocked, and/or unblocked creature."

The removal mechanism already exists *inside* `apply_regeneration` (replacement.rs:2728-2749, "step
3") and is duplicated at abilities.rs:2243. Factor it into a shared helper and reuse it:

```
// new: crates/engine/src/rules/combat.rs (or replacement.rs)
pub(crate) fn remove_from_combat(state: &mut GameState, object_id: ObjectId) -> bool
```
Removes `object_id` from `combat.attackers`, `combat.blockers`, `combat.blocked_attackers`, and every
`combat.damage_assignment_order` slot (attacker key + blocker lists), returning whether anything was
removed. `apply_regeneration` step 3 is rewritten to call it (no behavior change — regression-guard
via the existing regenerate tests).

New engine types: `Effect::RemoveFromCombat { target: EffectTarget }` (placed next to
`Effect::UntapPermanent`, card_definition.rs:1488) and `GameEvent::RemovedFromCombat { object_id }`
(events.rs:63, for invariant #4 / diagnosability — the tools match GameEvent with a catch-all, so no
view-model arm is required; verified `GameEvent::Regenerated` has no replay-viewer/TUI arm).

The def's Spires untap ability becomes a two-step `Effect::Sequence` (card_definition.rs:1754):
`Effect::Sequence(vec![ UntapPermanent { target: DeclaredTarget{0} }, RemoveFromCombat { target:
DeclaredTarget{0} } ])`.

---

## CR Rule Text (from MCP)

**506.4** — "A permanent is removed from combat if it leaves the battlefield, if its controller
changes, if it phases out, **if an effect specifically removes it from combat**, if it's a
planeswalker that's being attacked and stops being a planeswalker, … or if it's an attacking or
blocking creature that regenerates (see rule 701.19), stops being a creature, or becomes a battle. A
creature that's removed from combat stops being an attacking, blocking, blocked, and/or unblocked
creature." (506.4b: tapping/untapping an already-declared attacker/blocker does NOT by itself remove
it from combat — so the untap and the removal are genuinely two separate effects, confirming the
two-step Sequence.)

**508.4** — a creature put onto the battlefield attacking (or stated to be attacking) is "attacking"
but never "attacked" — corroborates that only `handle_declare_attackers` feeds the (b) count.

**508.1 / 603.4 / 614.1c / 400.2** — as cited inline above.

**Legion's Landing rulings (MCP, 2017-09-29):** "The last ability … only counts creatures that you
declare as attacking creatures. Creatures that enter the battlefield attacking won't count." / "Once
you've attacked with three or more creatures, Legion's Landing will transform even if some of those
creatures leave the battlefield or are removed from combat." — both satisfied by the captured
`attackers_declared_this_turn` design.

---

## Engine Changes

### Change 1 — `Condition::TopCardIsInstantOrSorcery` (a)
**File**: `crates/card-types/src/cards/card_definition.rs` (`enum Condition`, near line 3796).
**Action**: add unit variant `TopCardIsInstantOrSorcery`.
**File**: `crates/engine/src/effects/mod.rs` — add eval arm in `check_condition` next to
`Condition::TopCardIsCreatureOfChosenType` (~line 9011): peek `ZoneId::Library(ctx.controller).top()`,
read `card.characteristics.card_types` for `Instant` OR `Sorcery`, `false` if empty library.
**CR**: 400.2 / 614.1c.

### Change 2 — `Condition::YouAttackedWithNOrMore(u32)` + `PlayerState.attackers_declared_this_turn` (b)
**File**: `crates/card-types/src/cards/card_definition.rs` (`enum Condition`) — add
`YouAttackedWithNOrMore(u32)`.
**File**: `crates/card-types/src/state/player.rs` (~line 437, next to `attacked_this_turn`) — add
`pub attackers_declared_this_turn: u32` (with `#[serde(default)]`).
**File**: `crates/engine/src/state/builder.rs:273` — initialize `attackers_declared_this_turn: 0` in
the `PlayerState` literal (the single central construction site; runner: `cargo build` will flag any
other explicit `PlayerState { … }` literal).
**File**: `crates/engine/src/rules/combat.rs:624-628` — inside the `if !attackers.is_empty()` block,
set `ps.attackers_declared_this_turn = attackers.len() as u32;` alongside `attacked_this_turn = true`.
**File**: `crates/engine/src/rules/turn_actions.rs:1463` — reset `p.attackers_declared_this_turn = 0;`
next to `attacked_this_turn = false`.
**File**: `crates/engine/src/effects/mod.rs` `check_condition` — add eval arm:
`p.attackers_declared_this_turn >= n` for the effect controller.
**CR**: 508.1 / 508.4.

### Change 3 — `Effect::RemoveFromCombat { target }` + `GameEvent::RemovedFromCombat` + shared helper (g)
**File**: `crates/card-types/src/cards/card_definition.rs` (`enum Effect`, next to `UntapPermanent`
~line 1488) — add `RemoveFromCombat { target: EffectTarget }`.
**File**: `crates/engine/src/rules/events.rs` (`enum GameEvent`, near 63) — add
`RemovedFromCombat { object_id: ObjectId }`.
**File**: `crates/engine/src/rules/combat.rs` (or `replacement.rs`) — add
`pub(crate) fn remove_from_combat(state, object_id) -> bool`; rewrite `apply_regeneration` step 3
(replacement.rs:2728-2749) to call it (no behavior change).
**File**: `crates/engine/src/effects/mod.rs` — add `Effect::RemoveFromCombat { target }` dispatch
next to `Effect::Regenerate` (~line 3639): `resolve_effect_target_list` → for each
`ResolvedTarget::Object(id)` on the battlefield, call `remove_from_combat(state, id)`, push
`GameEvent::RemovedFromCombat { object_id: id }`.
**CR**: 506.4 / 508.

### Change 4 — Exhaustive match / wire updates

| File | Match expression | Action |
|------|-----------------|--------|
| `crates/engine/src/state/hash.rs` | `Condition` HashInto (~5881) | hash arm for `TopCardIsInstantOrSorcery` (new discriminant) |
| `crates/engine/src/state/hash.rs` | `Condition` HashInto | hash arm for `YouAttackedWithNOrMore(u32)` (hash the u32) |
| `crates/engine/src/state/hash.rs` | `Effect` HashInto (~6293) | hash arm for `RemoveFromCombat { target }` (hash the target) |
| `crates/engine/src/state/hash.rs` | `GameEvent` HashInto (~4751) | hash arm for `RemovedFromCombat { object_id }` |
| `crates/engine/src/state/hash.rs` | `PlayerState` HashInto (~1941) | hash `attackers_declared_this_turn` next to `attacked_this_turn` |
| `crates/engine/src/effects/mod.rs` | `check_condition` | 2 eval arms (Changes 1,2) |
| `crates/engine/src/effects/mod.rs` | `execute_effect` | 1 dispatch arm (Change 3) |
| `crates/engine/src/rules/events.rs` / `engine.rs` | any exhaustive `GameEvent` match (application, `private_to()`, Debug) | `cargo build --workspace` will flag; `RemovedFromCombat` is public (not player-private) |
| `crates/engine/src/rules/abilities.rs`, `resolution.rs` | any exhaustive `Effect` classification match | `cargo build --workspace` flags (Regenerate/UntapPermanent appear in abilities.rs+events.rs — check for a mirror arm) |
| `crates/engine/src/testing/replay_harness.rs` | JSON `Effect`/`Condition` lowering (only if these appear in scripts) | not required — no golden script uses these; add only if compile demands |

> Tools (`tools/replay-viewer/src/view_model.rs`, `tools/tui/src/play/panels/stack_view.rs`)
> exhaustively match `StackObjectKind` + `KeywordAbility`, NOT `Condition`/`Effect`/`GameEvent`
> (verified: `GameEvent::Regenerated` has no arm there). No tool edits expected — still run
> `cargo build --workspace` (the #1 miss).

---

## WIRE ANALYSIS (SR-8) — single batched bump PROTOCOL 20→21, HASH 57→58 (both forced)

**In the `PROTOCOL_SCHEMA_FINGERPRINT` closure (declared shape moves; type COUNT unchanged — no new
types join, all are existing closure enums gaining variants):**
- `Condition` — 2 new variants (`TopCardIsInstantOrSorcery`, `YouAttackedWithNOrMore`). In closure via
  `Effect::Conditional` (history rows 5, 9 confirm `Condition` is in the closure).
- `Effect` — 1 new variant (`RemoveFromCombat`). In closure (rows 10, 19, 20).
- `GameEvent` — 1 new variant (`RemovedFromCombat`). `GameEvent` is a wire frame (SR-8 root).

**HASH-only (inside `GameState`, NOT the PROTOCOL closure):**
- `PlayerState.attackers_declared_this_turn` — HASH schema change, no PROTOCOL impact.

**Verdict: ONE batched PROTOCOL 20→21 (four closure-shape moves) + HASH 57→58 forced** (five new
`HashInto` arms: 2 Condition, 1 Effect, 1 GameEvent, 1 PlayerState field — the hash-schema digest
moves regardless). (c)/(d) are deferred, so no `Cost`/`Command`/`ActivationCost`/`LookAtTopThenPlace`
wire change in this batch.

**Sentinel / fingerprint files the runner MUST update (get the new digests by running the failing
gate tests — they print the computed value):**

*PROTOCOL:*
- `crates/engine/src/rules/protocol.rs` — `PROTOCOL_VERSION` 20→21 (line 183); add a `- 21: PB-OS6 …`
  History line describing the four shape moves; set `PROTOCOL_SCHEMA_FINGERPRINT` (line 200) to the
  value printed by `protocol_schema_fingerprint_is_pinned`; **APPEND** (do not edit) a `PROTOCOL_HISTORY`
  row.
- `crates/engine/tests/core/protocol_schema.rs` — `protocol_version_sentinel` (872) 20→21; re-pin
  `FROZEN_HISTORY_PREFIX_DIGEST` (printed by `frozen_prefix`-style failure) after the append.
- `PROTOCOL_VERSION, 20` sentinels to bump to 21:
  `tests/primitives/pb_ef7_modal_activated.rs:242`, `pb_ef12_any_color_choice.rs:363`,
  `pb_ef10_sacrifice_driven_amounts.rs:1595`, `pb_os5_relative_attacker_count.rs:716`,
  `tests/core/protocol_schema.rs:872`.

*HASH:*
- `crates/engine/src/state/hash.rs` — `HASH_SCHEMA_VERSION` 57→58; add a `- 58:` History line; APPEND a
  `HASH_SCHEMA_HISTORY` row with the new fingerprints.
- `crates/engine/tests/core/hash_schema.rs` — `hash_schema_version_sentinel` (1194) 57→58; re-pin
  `FROZEN_HISTORY_PREFIX_DIGEST`.
- `HASH_SCHEMA_VERSION, 57u8` sentinels to bump to 58 (~38 files — the full grep list; the runner
  should `rg "HASH_SCHEMA_VERSION, 57u8" crates/engine/tests` and edit each): incl.
  `optional_cost_and_counter_tax.rs:1139`, `effect_sacrifice_permanents_filter.rs:136`,
  `loyalty_target_validation.rs:355`, `pbp_power_of_sacrificed_creature.rs:787`,
  `primitive_pb_xa.rs:93`, `pb_ef7_modal_activated.rs:237`, `primitive_pb_xs.rs:69`,
  `primitive_pb_ewcd.rs:143`, `pb_ef6_target_opponent.rs:280`, `primitive_pb_cc_c_followup.rs:402`,
  `primitive_pb_oos_lki_power_3.rs:66`, `pb_ac8_restrictions_and_wingame.rs:165`,
  `primitive_pb_lki_power.rs:389`, `pb_ef11_wheel_greatest_discarded.rs:91`,
  `pb_ac3_dynamic_pt_counts.rs:885`, `pb_ef2_create_token_recipient.rs:263`,
  `pbn_subtype_filtered_triggers.rs:568`, `primitive_pb_eat.rs:140`, `pb_ac9_wheel_and_misc.rs:127`,
  `pb_ac6_phase_action_conditions.rs:182`, `pb_ac1_untap_counter.rs:94`,
  `pb_ef1_exclude_self_enforcement.rs:165`, `primitive_pb_xa2.rs:107`, `primitive_pb_ewc.rs:400`,
  `primitive_pb_xs_e.rs:165`, `pb_ef10_sacrifice_driven_amounts.rs:1600`,
  `pb_ac4_per_mode_targeting.rs:693`, `pbt_up_to_n_targets.rs:412,866`,
  `pb_ef11_spell_single_target.rs:336`, `primitive_pb_ts.rs:369`,
  `pb_os5_relative_attacker_count.rs:721`, `pb_ac7_type_change_ability_removal.rs:961`,
  `primitive_pb_lki_cc.rs:443`, `pb_ac5_alt_costs.rs:408`, `pbd_damaged_player_filter.rs:611`,
  `primitive_pb_cc_a.rs:101`, `tests/core/hash_schema.rs:1194`.

---

## Card Definition Fixes

### delver_of_secrets.rs (partial → **Complete**)
**Oracle**: "At the beginning of your upkeep, look at the top card of your library. You may reveal
that card. If an instant or sorcery card is revealed this way, transform Delver of Secrets."
(front). Back Insectile Aberration 3/2 flying, blue via color indicator — already present.
**Fix**: replace the placeholder comment ability (lines 28) with an
`AbilityDefinition::Triggered { trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep, effect:
Effect::Conditional { condition: Condition::TopCardIsInstantOrSorcery, if_true:
Box::new(Effect::TransformSelf), if_false: Box::new(Effect::Nothing) }, intervening_if: None, targets:
vec![], modes: None, trigger_zone: None }`. Keep `KeywordAbility::Transform`. Update header comment
and set `completeness: Completeness::Complete` (drop the "DSL gap" note).

### thaumatic_compass.rs (partial → **Complete**)
**Oracle back (Spires of Orazca)**: "{T}: Add {C}.\n{T}: Untap target attacking creature an opponent
controls and remove it from combat." Front already Complete (search + end-step `TransformSelf`).
**Fix**: change the back-face untap ability's `effect` from the single `Effect::UntapPermanent { … }`
to `Effect::Sequence(vec![ Effect::UntapPermanent { target: EffectTarget::DeclaredTarget { index: 0 }
}, Effect::RemoveFromCombat { target: EffectTarget::DeclaredTarget { index: 0 } } ])`. Keep the
existing `targets: vec![TargetRequirement::TargetCreatureWithFilter(… is_attacking, controller:
Opponent …)]`. Update the two "OMITS that clause / partial" comments and set `completeness:
Completeness::Complete`.

### growing_rites_of_itlimoc.rs (stays **partial** — doc-only)
**Fix**: no behavioral change. Update the `TODO` (line 55) and `completeness` note to cite **PB-OS8 /
`Effect::LookAtTopThenPlace`** as the closing primitive instead of "OOS-EF5-4(f)". Keep the end-step
`TransformSelf` trigger and both back-face mana abilities as-is.

---

## New Card Definitions

### legions_landing.rs (NEW → **Complete**)
**Oracle (established Oracle text — MCP `lookup_card` returns only combined DFC type/keywords for
this card, NOT per-face text; the runner MUST cross-check the exact wording against `cards.sqlite`
`card_faces` at implement time, per the PB-EF5 grist/bloodline lesson):**
- Front — Legion's Landing (Legendary Enchantment, `{W}`): "When Legion's Landing enters the
  battlefield, create a 1/1 white Vampire creature token with lifelink. Whenever you attack with
  three or more creatures, transform Legion's Landing."
- Back — Adanto, the First Fort (Legendary Land): "{T}: Add {W}.\n{1}{W}, {T}: Create a 1/1 white
  Vampire creature token with lifelink." (MCP keywords confirm the pair is `["Transform"]`; color
  identity W.)

**CardDefinition sketch**:
- `types: supertypes(&[SuperType::Legendary], &[CardType::Enchantment])`, `mana_cost: {W}`.
- `abilities`:
  - `KeywordAbility::Transform`.
  - ETB trigger: `AbilityDefinition::Triggered { trigger_condition: <WhenEntersBattlefield/
    SelfEntersBattlefield per the DSL used elsewhere>, effect: Effect::CreateToken { <1/1 white
    Vampire, lifelink, recipient: controller> }, … }`. Mirror an existing 1/1 white Vampire-with-
    lifelink token producer for the exact `TokenSpec` (search `crates/card-defs/src/defs/` for a
    white Vampire lifelink token, e.g. Bishop of the Bloodstained / Call to the Feast family).
  - Attack trigger: `AbilityDefinition::Triggered { trigger_condition:
    TriggerCondition::WheneverYouAttack, effect: Effect::Conditional { condition:
    Condition::YouAttackedWithNOrMore(3), if_true: Box::new(Effect::TransformSelf), if_false:
    Box::new(Effect::Nothing) }, intervening_if: None, targets: vec![], modes: None, trigger_zone:
    None }`.
- `back_face: Some(CardFace { name: "Adanto, the First Fort", types: supertypes(Legendary, Land),
  abilities: [ Activated {T}: Add {W} ; Activated Cost::Sequence([Mana {generic:1, white:1}, Tap]):
  Effect::CreateToken(<same 1/1 white Vampire lifelink token>) ], color_indicator: None })`.
- `completeness: Completeness::Complete`.
- **Register** the new module in `crates/card-defs/src/defs/mod.rs` (or the generated registry list)
  — verify via `all_cards()`, not grep (SR-36).

No gated-stub effects anywhere (no `Effect::Choose`/`AddManaChoice`); both faces are fully
expressible with existing + this batch's primitives.

---

## Unit Tests

**New file**: `crates/engine/tests/primitives/pb_os6_dfc_flip_conditions.rs` (add its `mod` line to
the `primitives` group per SR-9a — never a top-level `tests/*.rs`). All probes are by EXECUTION
(SR-34/36): drive real Commands, assert transformed status / combat state, not source-tracing.

Decoy pairs (each decoy fails on exactly the field under test):
- `test_delver_flips_when_top_is_instant` — stack an instant on top of library, advance to the
  controller's upkeep, resolve the trigger, assert Delver is now transformed (Insectile Aberration
  face active). CR 400.2/701.28.
- `test_delver_no_flip_when_top_is_creature` — **decoy**: top card is a creature; same upkeep path;
  assert NOT transformed (the `TopCardIsInstantOrSorcery` condition is false → `Nothing`).
- `test_legions_landing_transforms_on_three_attackers` — battlefield: Legion's Landing + 3 attackers;
  `DeclareAttackers` with all 3; resolve the attack trigger; assert transformed to Adanto (a Land).
- `test_legions_landing_no_transform_on_two_attackers` — **decoy**: declare only 2 attackers; resolve;
  assert NOT transformed (`YouAttackedWithNOrMore(3)` false at count 2). Confirms the count gate, not
  a bare "you attacked".
- `test_legions_landing_fires_once_not_per_creature` — declare 3; assert exactly one transform (the
  trigger is per-combat, not per-attacker).
- `test_legions_landing_transforms_even_if_attacker_leaves` — declare 3, remove one from combat before
  the trigger resolves; assert still transforms (captured count, CR ruling).
- `test_thaumatic_spires_untaps_and_removes_from_combat` — an opponent's tapped attacker is in
  `combat.attackers`; activate Spires' `{T}` ability targeting it; assert the creature is (1) untapped
  AND (2) no longer in `combat.attackers` (and a `GameEvent::RemovedFromCombat` was emitted).
- `test_thaumatic_spires_decoy_untap_only_leaves_in_combat` — **decoy**: assert that BEFORE the ability
  resolves the target IS in `combat.attackers`, and that only the two-step Sequence removes it (i.e.
  a lone `UntapPermanent` would leave it attacking — CR 506.4b). Pins that the removal comes from the
  new effect, not the untap.
- Engine-level: `test_remove_from_combat_helper_clears_attacker_and_damage_order` — direct helper call
  clears `attackers`/`blockers`/`damage_assignment_order`; `test_regenerate_still_removes_from_combat`
  — regression guard that refactored `apply_regeneration` is unchanged.
- Sentinels in this file: `assert_eq!(PROTOCOL_VERSION, 21)` and `assert_eq!(HASH_SCHEMA_VERSION, 58)`
  with a comment naming PB-OS6 and the four closure moves.

**Patterns to follow**: `tests/primitives/pb_os5_relative_attacker_count.rs` (attack-count + sentinel
layout), `tests/mechanics_a_d/chosen_creature_type.rs` (top-of-library condition), the existing
`tests/mechanics_m_z/regenerate.rs` (combat-removal assertions).

---

## Verification Checklist

- [ ] `cargo check -p mtg-card-types -p mtg-engine`
- [ ] delver_of_secrets, thaumatic_compass flip to `Complete`; legions_landing authored `Complete` &
      registered in `all_cards()`
- [ ] growing_rites_of_itlimoc note re-pointed to PB-OS8; westvale_abbey left absent; OOS-OS6-1 seed
      recorded in `oos-retriage-plan-2026-07-18.md` + workstream-state
- [ ] PROTOCOL 20→21 + HASH 57→58 done in lockstep; both gate tests (`protocol_schema`, `hash_schema`)
      green with re-pinned fingerprints; all `…, 20` / `…, 57u8` sentinels bumped
- [ ] `cargo build --workspace` (catches every exhaustive-match miss — GameEvent/Effect/Condition)
- [ ] `cargo test --all` (incl. `core card_defs_fmt` / `tools/check-defs-fmt.sh`, SR-35)
- [ ] `cargo clippy -- -D warnings`
- [ ] No remaining TODOs in delver_of_secrets.rs / thaumatic_compass.rs; growing_rites TODO reworded
      (not removed)

---

## Risks & Edge Cases

- **WheneverYouAttack intervening_if is dropped in lowering** (replay_harness.rs:3156 hardcodes
  `None`; `check_intervening_if` only takes the limited runtime `InterveningIf`). This is WHY (b) gates
  inside `Effect::Conditional`, not via `intervening_if` — do not "simplify" it back to intervening_if
  (it would silently no-op). Same reasoning fixes (a).
- **Trigger-fires-then-does-Nothing** for legions/delver when the condition is false: a minor
  observability deviation (a fizzling trigger reaches the stack) but correct game state. Consistent
  with the engine's established peek-conditional pattern (Herald's Horn) — accepted as `Complete`.
- **Multi-combat turns** (Aggravated Assault etc.): `attackers_declared_this_turn` is overwritten to
  the most recent declaration's count. Legion's Landing transforms on the first ≥3 declaration and
  becomes a Land (loses the trigger), so a later smaller combat can't mis-fire it. Benign.
- **`remove_from_combat` refactor** must be behavior-identical for regeneration — guard with existing
  regenerate tests; keep the exact set of maps cleared (attackers, blockers, blocked_attackers,
  damage_assignment_order attacker key + blocker lists).
- **`GameEvent::RemovedFromCombat`** is a public event — ensure `private_to()` (if exhaustively
  matched) returns `None` for it. Confirm no tool exhaustive `GameEvent` match breaks (catch-all
  expected; `cargo build --workspace` is the gate).
- **Delver reveal "may"**: modeled mandatory-if-true. Faithful ONLY because transform is strictly
  beneficial for Delver (no card-advantage tradeoff like Herald's Horn); do not copy this to a card
  where declining the reveal can be correct.
- **Legion's Landing / Adanto token spec**: verify the token is 1/1 WHITE Vampire WITH lifelink and
  `recipient` = the ability's controller; mirror an existing def rather than hand-rolling the
  `TokenSpec`. Cross-check both faces' oracle wording against `cards.sqlite` (MCP DFC face-text gap).
