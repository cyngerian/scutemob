# Primitive Batch Review: PB-DX3b — OOS-DX3-1, the stale-blocker bucket (remainder)

**Date**: 2026-08-01
**Reviewer**: primitive-impl-reviewer (Opus)
**Task / commit**: `scutemob-166` · `bc035684`
**CR Rules**: 603.1, 603.4, 109.1, 111.1/111.5, 205.3, 400.7, 605.x (n/a), 702.147a, 704.5j
**Engine files reviewed**: none changed — verified read-only against `crates/engine/src/effects/mod.rs`
(`check_static_condition`, `check_condition`, `matches_filter`, `condition_is_queue_time_evaluable`),
`crates/engine/src/rules/abilities.rs` (`carddef_intervening_if_holds_at_queue_time`,
`InterveningIfMoment`, `is_nontoken` dispatch), `crates/engine/src/rules/turn_actions.rs` (upkeep +
end-step sweeps), `crates/engine/src/rules/replacement.rs` (self-ETB queue site),
`crates/engine/src/rules/protocol.rs`, `crates/engine/src/state/hash.rs`
**Test files reviewed**: `crates/engine/tests/primitives/pb_dx3b_stale_blocker_bucket.rs` (new),
`crates/engine/tests/primitives/main.rs` (mod line), `crates/engine/tests/core/decision_gate.rs`,
`crates/engine/tests/core/completeness_deviation_scan.rs`
**Golden script reviewed**: `test-data/generated-scripts/combat/191_decayed_jadar_zombie_token_eoc_sacrifice.json`
**Card defs reviewed (7)**: `jadar_ghoulcaller_of_nephalia`, `ophiomancer`, `dwynen_s_elite`,
`emeria_the_sky_ruin` (fixed) · `vampire_socialite`, `thousand_faced_shadow`, `guardian_project` (deferred)

## Verdict: needs-fix

**0 HIGH / 5 MEDIUM / 7 LOW.** All four completeness moves are justified against MCP oracle text
and every ruling, verified clause-by-clause and — where the def's claim is about *engine* semantics
rather than *card* semantics — traced through the full dispatch chain to the match arm. Jadar,
Ophiomancer and Dwynen's Elite are correct as authored: the filters say what the printed cards say,
the controller scoping is real (not merely named), `exclude_self` is genuinely enforced,
`Decayed`/`Snake`/`Elf` are read from layer-resolved characteristics, and both halves of CR 603.4 fire
at every trigger moment used. Emeria's intervening-if is right (7+ Plains, and she genuinely does not
count herself), and the demotion is the correct call: I independently reproduced the runner's search
for a free-optional mechanism and it comes up empty in a stronger way than the runner claimed —
`Effect::Choose`'s own doc comment says *"**Do not reach for this to express 'you may X'**"* and both
it and `MayPayOrElse` are gated by `effect_choose_gate.rs` from ever appearing on a `Complete` def, and
`EffectChoiceQuestion` has no generic "you may [effect]" variant. The pre-fix observation discipline is
materially better than PB-DX3's: every "pre-fix, X happened" sentence is backed by a fixture that could
actually produce the stated number, and the three vacuous cases are labelled rather than manufactured.
The five MEDIUMs are: one oracle mismatch on Emeria the batch did not notice while re-certifying her
(spurious `Legendary` supertype), one missing probe for the single most load-bearing claim in Jadar's
def, two stale-prose residues left inside the reconciled golden script (the exact class this batch
exists to close), and one pinned floor that has been converted into a tripwire whose failure message
will lie.

---

## Engine Change Findings

**None.** The batch's zero-engine claim holds on every check available to me:

| Gate | Result |
|---|---|
| `PROTOCOL_VERSION` | `rules/protocol.rs:335` = **32**, unmoved |
| `HASH_SCHEMA_VERSION` | `state/hash.rs:679` = **69**, unmoved |
| Any `DX3b` token under `crates/engine/src` or `crates/card-types/src` | **none** (only `crates/card-defs/src/defs/*` and `crates/engine/tests/*`) |
| New DSL variant / new `Effect` / new `Condition` | none — every construct used (`Condition::Not`, `Condition::YouControlNOrMoreWithFilter`, `TargetFilter.exclude_self`, `TriggerCondition::WhenEntersBattlefield`, `zombie_decayed_token_spec`) pre-dates the batch |

Dispatch chain independently walked and confirmed:

- `Condition::Not` → `effects/mod.rs:10109` (queue-time evaluable, delegates to inner) and
  `effects/mod.rs:9731` (`!check_condition(inner)`).
- `Condition::YouControlNOrMoreWithFilter` → `effects/mod.rs:10151` (queue-time evaluable) and
  `effects/mod.rs:10201-10233`. That arm checks `obj.zone == Battlefield && obj.is_phased_in() &&
  obj.controller == controller`, then `matches_filter(&expect_characteristics(state, obj.id), filter)`
  — i.e. **layer-resolved**, as Jadar's comment claims — plus `check_has_counter_type` and
  `(!filter.exclude_self || obj.id != source)` (PB-EF1, marker EF-5).
- `check_condition`'s own `YouControlNOrMoreWithFilter` arm (`effects/mod.rs:9909`) **delegates** to
  `check_static_condition`, so there is no second, divergent implementation on the
  `Not(...)` path. This was the most plausible way for the four defs to be silently wrong; it is clean.
- Queue-time gating reaches all three trigger moments used: `turn_actions.rs:299-317` covers
  `AtBeginningOfYourUpkeep` **and** `AtBeginningOfEachUpkeep` in one arm (Ophiomancer, Emeria),
  `turn_actions.rs:781` covers `AtBeginningOfYourEndStep` (Jadar), `replacement.rs:2131` covers
  self-ETB and passes `new_id` as `source` (Dwynen — this is what makes `exclude_self` meaningful).
- Resolution-time re-check is `InterveningIf::CardDef` + `InterveningIfMoment::Resolution`
  (`abilities.rs:10360-10390`), pinned by T3 and T10.

---

## Card Definition Findings

| # | Severity | Card / file | Description |
|---|----------|------|-------------|
| 1 | MEDIUM | `emeria_the_sky_ruin.rs:1,13` | **Spurious `Legendary` supertype.** MCP type line is `Land`, not `Legendary Land`. **Fix:** `types: types(&[CardType::Land])`, fix the leading comment, and extend the new `partial` note (which currently enumerates the remaining gap as *only* the "you may"). |
| 2 | MEDIUM | `pb_dx3b_stale_blocker_bucket.rs` (Jadar) | **The def's biggest claim is unpinned.** No probe that an *opponent's* decayed creature fails to suppress Jadar. **Fix:** add T13 mirroring T7. |
| 3 | MEDIUM | `combat/191...json:239,243,254` | **Stale prose contradicting the batch's own new assertions.** `step_note` still says the CreateToken effect "is not executed due to remaining engine gap"; two notes still say "PARTIAL FIX confirmed". **Fix:** rewrite all three to current behaviour. |
| 4 | MEDIUM | `combat/191...json:38-41` | **Dispute #1 left `resolution: null`** while dispute #2 states it was closed by B14. **Fix:** fill `resolution`/`resolved_by`/`resolved_date` on dispute #1 (leave `description` verbatim). |
| 5 | MEDIUM | `completeness_deviation_scan.rs:311-315` | **Zero-margin floor with a failure message that will lie.** `marked >= 661` at `marked == 661`; the next Complete flip reddens a test that says "MARKER_FRAGMENTS is broken". **Fix:** restore a margin or rewrite the message to name both causes. |
| 6 | LOW | `pb_dx3b_stale_blocker_bucket.rs:245-270` | `count_snakes` / `count_elf_warriors` read base `obj.characteristics`, while the gate reads layer-resolved. **Fix:** use `calculate_characteristics` as `count_decayed_creatures` already does. |
| 7 | LOW | `pb_dx3b_stale_blocker_bucket.rs` (Emeria) | No probe that an opponent's Plains don't count toward the 7. **Fix:** add a fixture giving p2 two Plains and p1 six. |
| 8 | LOW | `memory/primitive-wip.md:60,86` | Steps 5 and 8 defer their derivations to a "close-out report" that exists nowhere in the tree. **Fix:** inline the derivation or name the real artefact. |
| 9 | LOW | `combat/191...json` filename + `cr_sections_tested` | Claims `701.17a` / `704.3` / an "eoc_sacrifice" the script never runs (no combat). **Fix:** trim `cr_sections_tested` to what is exercised; note the filename mismatch. |
| 10 | LOW | `vampire_socialite.rs:41-44` | `partial` marker string names only blocker (a); the def carries two TODOs. **Fix:** name the conditional-ETB-replacement gap in the marker string too. |
| 11 | LOW | `guardian_project.rs:46-67` | The now-authorable `is_nontoken` half is deliberately deferred but has no seed row. **Fix:** file it in audit §8.1 at close-out. |
| 12 | LOW | close-out | audit §8.1 `OOS-DX3-1` not yet closed; `docs/authoring-status.md` + CLAUDE.md still say 1,142. Expected (step 9 open) — listed so it is not dropped. |

### Finding Details

#### Finding 1: Emeria is not a Legendary Land

**Severity**: MEDIUM
**File**: `crates/card-defs/src/defs/emeria_the_sky_ruin.rs:1` (comment), `:13` (`types:`)
**Oracle (MCP)**: `Emeria, the Sky Ruin` — `Type: **Land**`
**Issue**: the def declares `types: supertypes(&[SuperType::Legendary], &[CardType::Land])`. MCP is
not eliding supertypes here — I control-tested it: `Gaea's Cradle` returns `Legendary Land` and
`Valakut, the Molten Pinnacle` (the same Zendikar mythic-land cycle) returns `Land`. Emeria returns
`Land`. A spurious `Legendary` supertype means CR 704.5j would destroy a duplicate Emeria that the
real card permits, i.e. wrong game state, not cosmetics.

This is almost certainly pre-existing rather than introduced here, which is exactly why it belongs in
this review: the batch **re-certified this def**, replaced its marker, and wrote a fresh completeness
note that says the *only* remaining gap is the printed "you may". That note is now itself a stale
claim of the kind this batch exists to eliminate. It is MEDIUM and not HIGH solely because the
demotion to `partial` makes the def deck-illegal, so no live game state is produced today; if the
def ever flips back to `Complete` without this fix, it is HIGH.

**Fix**: change to `types: types(&[CardType::Land])`; fix the file's leading comment
(`// Emeria, the Sky Ruin — Legendary Land`); and either fix silently *and* say so in the
`Completeness::partial(...)` string, or leave the string honest by naming the corrected type line.

#### Finding 2: nothing pins "an opponent's decayed creature must not suppress Jadar"

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_dx3b_stale_blocker_bucket.rs` (no such test)
**CR Rule**: 603.4 — the intervening-if is *"if **you** control no creatures with decayed"*
**Issue**: `jadar_ghoulcaller_of_nephalia.rs:54-60` carries the longest comment in the def, arguing
that `TargetFilter.controller` is deliberately left at `TargetController::Any` because the
`YouControlNOrMoreWithFilter` evaluator does its own `obj.controller == controller` check and
`matches_filter` cannot see a controller field. **I verified that claim and it is true** —
`card_definition.rs:3241-3244` (`#[default] Any`), `effects/mod.rs:10208`
(`obj.controller == controller`), `effects/mod.rs:9496` (`matches_filter(&Characteristics, …)`).

But nothing in the suite tests it. The suite tests exactly this shape for *Ophiomancer* — T7 puts the
Snake on p2 and asserts p1's trigger still fires — and that test is the reason I can be confident the
mechanism works at all. Jadar has the identical exposure (a Decayed token is a common board presence
in a multiplayer pod; Wilhelt, Tainted Adversary and Ghoulish Procession all mint them) and no probe.
A future refactor that "helpfully" set `controller: TargetController::You` here, or that started
honouring `TargetFilter.controller` inside `matches_filter`, would silently suppress Jadar every end
step and the suite would stay green.

**Fix**: add T13 — p2 controls a creature with `Decayed`, p1 controls Jadar and no decayed creature;
assert Jadar's trigger **does** queue at p1's end step and one Zombie is created. Cite CR 603.4 and
point at `effects/mod.rs`'s `obj.controller == controller` line the way T7's docstring does.

#### Finding 3: the reconciled golden script still narrates the bug it no longer has

**Severity**: MEDIUM
**File**: `test-data/generated-scripts/combat/191_decayed_jadar_zombie_token_eoc_sacrifice.json:239`
(`script[1].step_note`), `:243` (`actions[0].description`), `:254` (`actions[0].note`)
**Issue**: the batch strengthened the final `assert_state` (correctly — see the Positive Verifications
section) and rewrote `metadata.description` and `metadata.generation_notes`, but three prose fields
*inside the script body* were left untouched and now contradict the assertions two objects below them:

- `:239` — *"Trigger resolves — but CreateToken effect is not executed due to remaining engine gap
  (resolution reads characteristics.triggered_abilities which is empty). See disputes."*
- `:243` — *"PARTIAL FIX confirmed: end_step_actions() generic sweep places the trigger on the stack."*
- `:254` — *"Stack count = 1 — this assertion PASSES with the partial engine fix."*

The script now asserts the token exists. Its own step note says the token is never created. This is
the doc-vs-code class the whole PB-DX line is closing, reproduced inside the artefact the batch
reconciled — the same self-referential failure PB-DX3's single MEDIUM recorded.

**Fix**: rewrite all three to current behaviour, matching the language already used in
`metadata.description`. Do not delete the history — it lives, correctly, in the two dispute entries.

#### Finding 4: dispute #1 is still open on a gap the batch says is closed

**Severity**: MEDIUM
**File**: `test-data/generated-scripts/combat/191_...json:38-41`
**Issue**: dispute #1's `description` begins *"REMAINING ENGINE GAP (after partial fix)"* and its
`resolution` / `resolved_by` / `resolved_date` are all `null`. Dispute #2 — appended by this batch —
states plainly that this gap *"was ALREADY FIXED by the B14 engine fix"*. The corpus therefore now
carries an approved script asserting an open engine defect that the same file, four lines later,
says does not exist.

The plan's append-only instruction (§4.3) was about not deleting or editing the existing
`description`. Filling the `resolution` field is not an edit of the record; it is the field's purpose,
and every other resolved dispute in the corpus uses it.

**Fix**: set dispute #1's `resolution` to a short pointer at dispute #2 and at the B14 fix, with
`resolved_by: "primitive-impl-runner (PB-DX3b, scutemob-166)"` and `resolved_date: "2026-08-01"`.
Leave `description` byte-for-byte as it is.

#### Finding 5: the non-vacuity floor has become a tripwire with a lying message

**Severity**: MEDIUM
**File**: `crates/engine/tests/core/completeness_deviation_scan.rs:311-315`
**Issue**: the floor moved `662 → 661` and is now pinned at the **exact** measured value, deliberately
and with a documented argument ("ANY fixed margin silently erodes"). The derivation itself is sound and
I verified the numbers independently: CLAUDE.md's `1,142/1,804` gives `1804 − 1142 = 662` non-Complete
on the merge base, and `1804 − 1143 = 661` here, matching the comment's claim and confirming that the
previous `669` in the message was indeed stale. Good work.

The problem is what the assertion now *does*. `marked >= 661` with `marked == 661` fails the moment
any future batch flips one more def to `Complete` — i.e. on ordinary card-authoring work that has
nothing to do with this gate — and the failure message it prints is
*"MARKER_FRAGMENTS is broken and the gate would spuriously flag marked defs"*, which will be false.
The sibling file states the opposite convention explicitly at `decision_gate.rs:923-924`:
*"Assertions are `>=` floors only (the PB-DP9 convention: an `==` pin reddens on unrelated
authoring)."* A `>=` pinned at the current value is an `==` in the only direction that moves.

The stated purpose of this test is detector health, not corpus size. A margin does not "erode" into
false failures; it erodes into reduced sensitivity, which is the correct trade for a detector-health
guard.

**Fix**: either (a) restore a margin (the historical convention was ~9; `>= 640` would survive the next
several batches while still catching a detector that has stopped matching entirely), or (b) keep the
exact pin and rewrite the message to name both causes and instruct re-measurement, e.g.
*"marker detector matched {marked} files; expected >= 661. Either MARKER_FRAGMENTS stopped matching, or
the corpus legitimately moved — re-measure the non-`Complete` count against `all_cards()` and update
this floor with a dated derivation."* (b) is closer to the runner's stated intent; (a) is closer to the
test's stated purpose. Either is acceptable; the current combination is not.

---

## Oracle-vs-Filter Semantic Gate (the #1 primitive-review failure mode)

Every filter walked card def → enum variant → match arm → runtime behaviour.

| Card | Printed clause | DSL | Dispatch verdict |
|---|---|---|---|
| Jadar | "if you control no **creatures with decayed**" | `Not(YouControlNOrMoreWithFilter{1, has_card_type Creature + has_keywords[Decayed]})` | **Correct.** `matches_filter` checks `has_card_type` at `:9507` and iterates `has_keywords` at `:9512`, both against `expect_characteristics` (layer-resolved), so a Humility/Dress Down strip of `Decayed` re-enables the trigger as the def claims. Controller scoping is done by the arm, not the filter — verified, see Finding 2. **Jadar's own token counts** (it is a Creature with `Decayed` under the same controller — `zombie_decayed_token_spec` at `card_definition.rs:4272` puts `Decayed` in `keywords`), so the card is self-limiting exactly as printed; T3 proves the token is visible to the gate end-to-end. |
| Ophiomancer | "if you control no **Snakes**" | `Not(YouControlNOrMoreWithFilter{1, has_subtype Snake})` | **Correct, and the deviation is the right call.** The def's former note proposed `Not(ControlCreatureWithSubtype(Snake))`; I read that arm — `effects/mod.rs:9867-9876` — and it does hard-require `chars.card_types.contains(&CardType::Creature)`. CR says "Snakes", i.e. permanents with the subtype (CR 205.3), which is a strict superset (Kindred/Tribal noncreature permanents are the live counterexample class). `has_subtype` alone is the exact translation, and it agrees with ruling 2013-10-17 #3 in every 2013-legal board. |
| Ophiomancer | "At the beginning of **each** upkeep" + "if **you** control" | `AtBeginningOfEachUpkeep` + controller-scoped gate | **Correct and genuinely tested.** `turn_actions.rs:299-306` fires the arm for `AtBeginningOfEachUpkeep` on every player's upkeep and passes the *permanent's* `controller`, not the active player, into `carddef_intervening_if_holds_at_queue_time`. T7 is a real discriminator: p2 (active) holds the Snake, p1 (controller) does not, and the trigger must queue — an active-player-scoped gate would produce `stack.len() == 0` and fail. |
| Dwynen's Elite | "if you control **another** Elf" | `YouControlNOrMoreWithFilter{1, has_subtype Elf, exclude_self: true}` | **Correct.** CR 109.1. `exclude_self` is enforced at `effects/mod.rs:10228`, and the self-ETB queue site passes the entering object's `new_id` as `source` (`replacement.rs:2135`), which is what makes the exclusion refer to the right object. T8 discriminates: drop `exclude_self` and Dwynen (an Elf) satisfies `count >= 1` alone, the trigger queues, and T8 reddens. |
| Dwynen's Elite | trigger variant | `TriggerCondition::WhenEntersBattlefield` | **Correct.** `card_definition.rs:3259-3260` documents it as *"When ~ enters the battlefield — self-referential ETB"*, and `replacement.rs:2117` matches exactly that variant on the self-ETB path. Not the "any permanent enters" variant (`WheneverPermanentEntersBattlefield`, `:3344`). |
| Dwynen's Elite | "1/1 green Elf Warrior creature token" | `TokenSpec` name `"Elf Warrior"`, `[Creature]`, `[Elf, Warrior]`, `[Green]`, 1/1, no keywords | **Correct**, matches the printed token exactly. |
| Ophiomancer | "1/1 black Snake creature token with deathtouch" | `TokenSpec` name `"Snake"`, `[Creature]`, `[Snake]`, `[Black]`, 1/1, `[Deathtouch]` | **Correct.** |
| Emeria | "if you control **seven or more Plains**" | `YouControlNOrMoreWithFilter{7, has_subtype Plains}` | **Correct.** Subtype-only is right (a Plains is a Plains whether basic, dual or animated), and Emeria genuinely does not count herself — she has no land subtypes, so `exclude_self` really is unnecessary rather than merely omitted. Ruling 2009-10-01 names both CR 603.4 halves and both are wired. |

---

## Test Review

**Non-vacuity**, checked test by test rather than taken from the module doc:

| Probe | Fails before the def edit? | Discriminates against the mutation it names? |
|---|---|---|
| T1 jadar / decayed present → no queue | **Yes** (`stack empty` would see len 1) | yes |
| T2 jadar / no decayed → queue + token | no (regression guard, labelled) | n/a |
| T3 jadar / decayed appears before resolution | **Yes** (count would be 2) | yes — pins the resolution re-check |
| T4 jadar `oracle_text` | **Yes** | yes |
| T5 ophiomancer / Snake present → no queue | **Yes** | yes |
| T6 ophiomancer / no Snake → token | no (guard, labelled) | n/a |
| T7 ophiomancer / opponent's upkeep + opponent's Snake | no (passes pre-fix) | **yes** — reddens if the gate reads the active player's board |
| T8 dwynen alone → no token | vacuous pre-fix, **labelled** | **yes** — reddens if `exclude_self` is dropped |
| T9 dwynen + another Elf → token | **Yes** (ability did not exist) | yes |
| T10 dwynen / Elf leaves before resolution | vacuous pre-fix, **labelled** | yes — pins the resolution re-check |
| T11 emeria / 6 Plains → no queue | **Yes** | yes |
| T12 emeria / 7 Plains → reanimates | no (guard, labelled) | n/a |

**Pre-fix observation claims — the PB-DX3 MEDIUM standard.** I read the module doc sceptically and
checked each claim against the fixture that is supposed to have produced it. All four hold, and
this is a genuine improvement over PB-DX3:

- **T1** claims `stack len = 1` and `decayed count = 2` pre-fix. The fixture has a real
  `Old Zombie` with `Decayed` on p1's battlefield, so "2" is a number that fixture can actually
  produce (Old Zombie + the wrongly-created token) and `count_decayed_creatures` reads it through
  `calculate_characteristics`, which sees the token's keyword. **Checkable and consistent.**
- **T3** claims the trigger queued and resolution created *another* Zombie. The fixture creates a
  decayed token between queue and resolution, so pre-fix the post-state is 2 and post-fix it is 1 —
  the assertion is `== 1` and would have read 2. **Consistent.**
- **T5** claims `stack len = 1` and `snake count = 2`. The fixture has a real `Garter Snake`.
  **Consistent.**
- **T11** claims `stack len = 1` (target auto-filled) and *"Dead Beast was on the battlefield and NOT
  in the graveyard"*. The fixture has a real `Dead Beast` creature card in p1's graveyard and exactly
  one legal target, so CR 601.2c auto-fill is available and the reanimation is observable.
  **Consistent** — and specifically not the PB-DX3 T1 failure mode, where the claimed number could not
  have arisen from the fixture at all.
- **T8/T9/T10** are labelled vacuous with the reason stated (`abilities` was empty). Correct and
  honest; no number was manufactured.
- T2/T4/T6/T7/T12 make no pre-fix claims, so there is nothing to go stale.

**T3's documented deviation from the plan** is correct and the reasoning is right: the plan's
row wrote the standard "true at queue, false at resolution" scenario without inverting for the
negated condition, which would have described a case that never queues. Recording the correction in
the module doc rather than silently following the plan is the right call.

**Test gaps**: Findings 2 (Jadar / opponent's decayed creature — MEDIUM) and 7 (Emeria / opponent's
Plains — LOW). Finding 6 (base vs layer-resolved characteristics in two helpers) is a fidelity nit,
not a correctness hole, on these fixtures.

---

## The Two Moved Floors

Both verified by derivation rather than by reading the comment that claims them (I have no shell in
this session, so "direct measurement" here means re-deriving the arithmetic from independent sources
and checking the mechanism, not re-running `cargo test`).

| Floor | Move | Verdict |
|---|---|---|
| `decision_gate.rs:1045` `triggered_targets` | 77 → 76 | **Sound.** Emeria is the only one of the four defs whose `AbilityDefinition::Triggered` carries a non-empty `targets` (`TargetCardInYourGraveyard`), and `is_effectively_complete` is exactly `completeness == Complete` (`decision_site_walk.rs:537-539`), so demoting her removes precisely one match. Jadar/Ophiomancer/Dwynen all have `targets: vec![]` and cannot add to this row. 77 − 1 = 76. |
| `completeness_deviation_scan.rs:312` | 662 → 661 | **Arithmetic sound, gate design regressed.** `1804 − 1142 = 662` (merge base) and `1804 − 1143 = 661` (here) corroborate the comment from CLAUDE.md's independently-maintained coverage figures, and confirm the old `669` really had gone stale. See Finding 5 for the zero-margin problem. |

Neither of the four defs appears in `decision_gate.rs`'s `BASELINE`, and neither new `Complete` def
introduces a decision site (both are fixed `Effect::CreateToken` with no targets, modes, or choice
variants), so PB-DP10's gate is untouched in substance — consistent with the runner's report.

---

## The Three Defers

All three were genuinely re-verified, not copied forward. I re-derived the current `Condition` variant
list from `card_definition.rs:3678-3858` and checked each blocker against it:

| Def | Blocker as re-stated | Verdict |
|---|---|---|
| `vampire_socialite` | `Condition::OpponentLostLifeThisTurn` absent; `AbilityDefinition::Replacement.unless_condition` is an opt-**out** gate, wrong polarity | **Confirmed.** No such variant in the enum. The polarity observation about `unless_condition` is a new, correct addition the old note did not have. |
| `thousand_faced_shadow` | no zone-of-origin `Condition`; no "the source itself is attacking" `Condition` (`TargetFilter.is_attacking` is target-side only) | **Confirmed.** Neither exists. The distinction drawn between target-side `is_attacking` (which PB-XA does enforce, and this def already uses) and a source-side intervening-if is correct and is the kind of chain-walk the old note lacked. |
| `guardian_project` | (a) `is_nontoken` **is** honoured — note was stale; (b) name-uniqueness `Condition` genuinely absent | **The runner's claim is correct and I verified it independently.** `rules/abilities.rs:6964` reads `creature_filter.is_nontoken && entering_obj.is_token` in the creature-ETB dispatch, and `replay_harness.rs:2946` forwards the def's full `TargetFilter` as `triggering_creature_filter` for exactly this `WheneverCreatureEntersBattlefield` shape. `crates/engine/tests/rules/etb_trigger_subtype_filter.rs` has a dedicated revert-proof for it (`:1519-1684`). (b) is confirmed absent. Correcting the note without applying the fix is the right call given the declared 4-def scope — but it needs a seed (Finding 11). |

The `partial`/`known_wrong` markers and the `completeness_deviation_scan` needle set stay consistent:
none of the four edited defs contains deviation language (`simplif` / `modeled as` / `deviation` /
`approximat`), so none newly needs an `ALLOWLIST` entry — I checked, and the plan's "deliberate
deviation" phrasing correctly did **not** make it into `ophiomancer.rs`'s shipped comment, which
would have forced that def out of `Complete` by the gate's own rule.

---

## Positive Verifications Worth Recording

- **The Emeria demotion is correct and the falsifier search is genuine.** The plan invited the
  reviewer to find a free-optional mechanism it missed. I looked, and the DSL is more emphatic than
  the runner's own note: `Effect::Choose`'s doc comment at `card_definition.rs:1755-1756` says
  *"**Do not reach for this to express 'you may X'** — `Choose{[X, Nothing]}` always does X"*, and
  both `Choose` and `MayPayOrElse` are barred from `Complete` defs by `effect_choose_gate.rs`
  (`:1758`, `:1777`). `MayPayThenEffect` (`:1786-1796`) is documented as *"Deterministic
  non-interactive path: the payer pays when able"* — so a free cost always pays and `then` always
  fires, byte-identical to the unconditional effect, exactly as the def's comment says.
  `EffectChoiceQuestion` (`card-types/src/state/stubs.rs:906`) has no generic "you may" variant.
  **There is no free-optional mechanism. The demotion stands.**
- **The golden script was strengthened, not weakened.** The prior final `assert_state` checked only
  "Jadar on battlefield, stack empty, life totals unchanged" — three facts that hold whether or not
  the token exists. It now asserts `zones.battlefield.p1 includes [Jadar, Zombie]`. That is a real
  assertion on a real object, and the script's own scenario (P1 controls no decayed creature at any
  point) genuinely satisfies the new gate at both checkpoints, so it is a true positive-path witness
  rather than a green-by-construction one.
- **All four `oracle_text` fields match MCP verbatim**, including Jadar's reminder text and Emeria's
  three-line body. Jadar's corrected text ("no creatures with decayed") is exactly the printed clause,
  and T4 pins both directions (contains the new, does not contain "Shambling Ghast").
- **Mana costs, P/T and (except Emeria) type lines all match MCP**: Jadar `{1}{B}` Legendary Creature
  — Human Wizard 1/1; Ophiomancer `{2}{B}` Creature — Human Shaman 2/2; Dwynen's Elite `{1}{G}`
  Creature — Elf Warrior 2/2.
- **The `mod` line is registered** (`crates/engine/tests/primitives/main.rs:33`), so SR-9a's silent
  coverage-deletion hazard is closed.
- **No TODOs or placeholders remain** in any of the four fixed defs.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 603.4 s1 (queue-time) | Yes (pre-existing, PB-DP6) | Yes | T1, T5, T11; T8 via `exclude_self` |
| 603.4 s2 (resolution re-check) | Yes (pre-existing, PB-DX1) | Yes | T3, T10 |
| 603.1 (trigger fires) | Yes | Yes | T2, T6, T9, T12 |
| 109.1 ("another") | Yes | Yes | T8 — `exclude_self` |
| 205.3 (subtypes, "Snakes"/"Plains") | Yes | Yes | T5/T6, T11/T12 |
| 702.147a (Decayed) | Yes (pre-existing) | Yes | T1/T2/T3 via `zombie_decayed_token_spec` |
| 111.1/111.5 (tokens) | Yes | Yes | all token probes + `combat/191` |
| "each upkeep" controller scoping | Yes | Yes | **T7** — the best test in the file |
| "you control" (opponent's board excluded) — Jadar | Yes (engine) | **No** | **Finding 2** |
| "you control" (opponent's board excluded) — Emeria | Yes (engine) | **No** | Finding 7 |
| 704.5j (legend rule) | **Wrongly applicable to Emeria** | No | **Finding 1** |
| 400.7 (new object on reanimation) | Yes | Yes | T12's second assertion |

---

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Marker move justified | Notes |
|------|-------------|-----------------|-------------------|---|-------|
| `jadar_ghoulcaller_of_nephalia` | **Yes** | 0 | **Yes** | n/a (stays `Complete`) | Filter, controller scoping and layer-resolution all verified; live-wrong behaviour genuinely closed. Untested claim → Finding 2 |
| `ophiomancer` | **Yes** | 0 | **Yes** | **Yes** — `partial` → `Complete` correct | `has_subtype`-only deviation verified against `ControlCreatureWithSubtype`'s creature hard-requirement; T7 is a real discriminator |
| `dwynen_s_elite` | **Yes** | 0 | **Yes** | **Yes** — `inert` → `Complete` correct | Ability authored from scratch; token spec and `exclude_self` both verified end-to-end |
| `emeria_the_sky_ruin` | **No — type line** | 0 | Gate correct; "may" unimplemented (declared) | **Yes** — `Complete`-by-default → explicit `partial` correct | Finding 1 |
| `vampire_socialite` | Yes | 2 (declared) | n/a (`partial`) | n/a | Defer re-affirmed; marker string incomplete (Finding 10) |
| `thousand_faced_shadow` | Yes | 1 (declared) | n/a (`partial`) | n/a | Defer re-affirmed with a sharper chain-walk than the old note |
| `guardian_project` | Yes | 2 (one now stale-corrected) | n/a (`known_wrong`) | n/a | (a) half now authorable — needs a seed (Finding 11) |

---

## Close-out Items (not findings — step 9 is legitimately open)

1. Close `OOS-DX3-1` in `docs/audits/decision-point-audit.md` §8.1. **Record the seed's own error**:
   the row dispositions `emeria_the_sky_ruin` into the "genuinely does not exist yet" pile, and that
   was wrong twice over — the condition *was* expressible, and the def was silently `Complete` by the
   `#[default]` derive trap while carrying a stale blocker note. That is a new instance of the
   `aurelia_the_warleader` class and is the most transferable thing this batch found.
2. File the `guardian_project` `is_nontoken` half as a seed (Finding 11).
3. Regenerate `tools/authoring-report.py`; update `docs/authoring-status.md` and the CLAUDE.md
   snapshot to **1,143/1,804 = 63.4%** — and report the delta honestly as **+1 net (+2 / −1)**, as the
   plan's own corrected §5 requires.
4. Note in the handoff that the seed row's own count is off by one ("the other three" then lists four
   defs) — trivial, but this batch is about doc claims that nobody re-reads.

---

## Previous Findings

Not a re-review; no prior `pb-review-DX3b.md` existed.
