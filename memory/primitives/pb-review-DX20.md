# Primitive Batch Review: PB-DX20 — the offer layer cannot see a keyword-carried target requirement

**Date**: 2026-08-04
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 303.4a, 303.4d, 303.4g, 601.2c, 602.2c, 700.2c, 702.5a, 702.5c, 702.5d,
702.11b, 702.16b, 702.18a, 702.103b, 702.151a, 205.4a
**Engine files reviewed**: `crates/engine/src/rules/casting.rs` (`enchant_target_to_requirement`,
`aura_spell_target_requirements`, the CR 303.4a gate at `:3728-3798`, the Step-2 fold at
`:3627-3637`), `crates/engine/src/rules/queries.rs` (`spell_target_requirements`),
`crates/engine/src/testing/replay_harness.rs` (Reconfigure synth site),
`crates/engine/src/state/keyword_registry.rs` (SR-5)
**Test / prose files reviewed**:
`crates/engine/tests/primitives/pb_dx20_keyword_carried_target_requirements.rs` (13 probes),
`crates/engine/tests/mechanics_e_l/enchant.rs:498-556`,
`crates/engine/tests/primitives/cards1_equip_target_repair.rs:634-707`,
`tools/play-server/src/main.rs` (`KNOWN_FALSE_OFFERS` deletion + T6),
`crates/simulator/src/targeting.rs`, `crates/simulator/src/report.rs`,
`crates/simulator/src/setup.rs`, `docs/audits/decision-point-audit.md:1183-1191`
**Card defs reviewed**: **23** (every def carrying `KeywordAbility::Enchant(...)`), of which the
**13 `Complete`** were each checked against MCP oracle text. **0 card-def lines changed by the batch.**

---

## Verdict: needs-fix

The primitive itself is right. I re-derived the `EnchantTarget -> TargetRequirement` mapping
independently — all 9 variants and all 6 `EnchantFilter` fields — against
`sba::matches_enchant_target` / `sba::enchant_filter_matches` on one side and
`validate_object_satisfies_requirement` / `effects::matches_filter` on the other, and it is
**exact in both directions**: no variant is stricter, none is looser. Three plan claims I was asked
to distrust also hold up in source: hexproof/shroud/protection already applied to Aura casts before
this batch (§4.3), `exclude_self` is honoured on the *activated-ability* path (`abilities.rs:501-508`
passes `source` as `self_id`), and the ordering asymmetry between the cast and query paths is gated
by T4's "no Aura def carries an `AbilityDefinition::Spell`" assertion, because `spell_mode_selection`
can only find modes inside that ability. The `KNOWN_FALSE_OFFERS` deletion is a strict
strengthening and loses no coverage.

What needs fixing is one **HIGH card-def finding the batch's own scope should have caught**
(`imprisoned_in_the_moon` — `Complete`, deck-legal, one of the 13 Auras this batch exists to make
playable, and its declared restriction is *wider* than its printed line, so the browser now offers
illegal targets and the engine accepts them), plus a cluster of MEDIUM test/evidence gaps: T1 is
structurally blind to two whole classes of mapping error and exercises only one of the six
`EnchantFilter` fields; T6's failure-under-revert was watched on a deleted scratch function rather
than on the committed probe; and two filed seeds (`OOS-DX20-7`, `OOS-DX20-8`) make claims about the
tree and about the divergence set that are false as written.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | MEDIUM | `pb_dx20_...rs:191-299` | **T1 cannot see a strictly-narrower mapping, nor any player-side mapping error.** The plan's claim that T1 "is the only assertion that can catch a stricter-or-looser mapping in either direction" is false in the stricter direction. **Fix:** add an exact-shape pin for all 9 arms and a player-target row. |
| E2 | MEDIUM | `pb_dx20_...rs:201-206` | **Only one `EnchantFilter` instantiation is probed.** `basic`, `nonbasic`, `has_subtypes`, `controller: Opponent`/`Any` are never exercised — and `basic: true` is live in two deck-legal `Complete` defs. **Fix:** add `Filtered` rows for those fields. |
| E3 | MEDIUM | `docs/audits/decision-point-audit.md:1190` | **`OOS-DX20-8` misstates the divergence set.** The non-empty per-mode slice case does *not* agree; it is a **new hard cast rejection**. **Fix:** reword the seed. |
| E4 | MEDIUM | `docs/audits/decision-point-audit.md:1189` | **`OOS-DX20-7` claims a comment that does not exist.** Plan §5 step 5's comment at `abilities.rs:539-582` was never written. **Fix:** write the comment, or strike the claim. |
| E5 | MEDIUM | `tools/play-server/src/main.rs:9979-10031` | **T6's failure-under-revert is unverified for the committed probe.** Only a deleted scratch function was watched red. **Fix:** re-run the T2-class revert against the committed test and record the observed failure. |
| E6 | LOW | `tools/play-server/src/view.rs:2399-2401` | Comment now false for a modal Aura (0 corpus exposure, same shape as E3). **Fix:** add a pointer to `OOS-DX20-8`. |
| E7 | LOW | `crates/engine/tests/primitives/pb_dx20_...rs:766-789` | `test_dx20_t5_5`'s mana assertion is structurally incapable of failing. **Fix:** delete it or replace with a real post-rejection observation. |
| E8 | LOW | `tools/play-server/src/main.rs:9995-10006` | Two near-tautological assertions in T6. **Fix:** tighten to `contains("Llanowar Elves")` and drop the redundant emptiness check. |
| E9 | LOW | `pb_dx20_...rs:408-434`, `:361-376` | T2.4 and T2.2's second half assert only `is_err()`. **Fix:** match the error variant/message substring. |
| E10 | LOW | `crates/simulator/src/mana_solver.rs:292-294`, `legal_actions.rs:809`, `tests/sim2_mana_intelligence.rs:958` | Three comments cite `KNOWN_FALSE_OFFERS` constants that no longer exist anywhere. **Fix:** mark them historical or name the batch that deleted them. |
| E11 | LOW | `docs/mtg-engine-feedback-engineering.md:536, 537, 732` | Still documents `OOS-CARDS2-4` as live ("13 `Complete` Auras that 422 on first contact"). **Fix:** annotate as closed by PB-DX20. |
| E12 | LOW | `memory/primitive-wip.md`, `memory/workstream-state.md:18` | WIP file still describes PB-DX32; no PB-DX20 handoff appended; row 18 still reads "next PB-DX20". **Fix:** plan §9's final checklist item. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | **HIGH** | `imprisoned_in_the_moon.rs:24` | **Declared `EnchantTarget::Permanent` for a printed "Enchant creature, land, or planeswalker".** `Complete` (by the `#[default]` derive), deck-legal, and one of the 13 Auras this batch makes offerable. The browser now offers artifacts and enchantments as legal targets and the cast **succeeds**. **Fix:** file the seed; add a T4 roster pin. |

The other **12** `Complete` Auras were each read against MCP oracle text and are **correct** — see
the Card Def Summary table.

---

### Finding Details

#### Finding C1: `imprisoned_in_the_moon` declares a restriction wider than its printed line, and PB-DX20 turns that into a human-reachable wrong state

**Severity**: HIGH
**File**: `crates/card-defs/src/defs/imprisoned_in_the_moon.rs:24`
**Oracle** (MCP, 2026-08-04): "Enchant creature, land, or planeswalker"
**CR Rule**: 702.5a — "The enchant ability restricts what an Aura spell can target and what an Aura
can enchant."

**Issue**: the def declares
`AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Permanent))`. `Permanent` is
strictly wider than the printed restriction — it additionally admits artifacts, enchantments and
battles. The def carries **no `completeness` field**, so it is `Complete` by the `#[default]`
derive (the silent-defect generator CLAUDE.md already names twice), it passes `validate_deck`, and
it is one of the exact 13 defs the T4 roster gate counts as this batch's yield.

Before PB-DX20 the mis-declaration was *half* live: `sba::matches_enchant_target`'s `Permanent` arm
is a bare `true` (`sba.rs:1026`), so a hand-built `CastSpell` naming an artifact already succeeded —
but no offer surface could see it, so no human could reach it. **PB-DX20 opens exactly that door.**
`enchant_target_to_requirement(Permanent) = TargetRequirement::TargetPermanent`, whose object arm is
`on_battlefield` alone (`casting.rs:6537`), so `legal_targets_per_slot` now enumerates *every*
permanent on the board into `target_slots[0].candidates`, the browser renders them, the human
clicks an artifact, and both the requirement check and the redundant CR 303.4a gate accept it. The
Aura resolves attached to an object CR 702.5a forbids, and CR 704.5m's SBA — which uses the same
over-wide `matches_enchant_target` — will not clean it up either.

This is the same class the batch *did* file as `OOS-DX20-5` for `kayas_ghostform` ("narrower in
type, wider in controller… PB-DX20 makes the error VISIBLE"). The difference is that
`kayas_ghostform` is `partial` and therefore not deck-legal, while this one is `Complete` and
deck-legal — i.e. the batch filed the harmless instance and missed the live one. Nothing checks
this class: SR-37's printed-field fidelity gate covers mana cost, P/T, type line, ability-embedded
costs and oracle text, and the Enchant restriction is none of those.

**Fix**:
1. File a seed in `docs/audits/decision-point-audit.md` §8.1 (suggest `OOS-DX20-10`) recording the
   card, the oracle text, the widened restriction, and the fact that PB-DX20 made it
   human-reachable. State the blocker honestly: it shares `OOS-DX20-5`'s root — `EnchantFilter` has
   `has_card_type` (single) and `has_subtypes` (OR over *sub*types) but **no** OR-vector over card
   types, and `EnchantTarget` has `CreatureOrPlaneswalker` but nothing that admits Land as well. So
   the correct fix is a filter-field addition (`has_card_types`) or a new `EnchantTarget` variant,
   and both seeds should be closed by the same successor.
2. Add a fifth T4 roster assertion pinning the set of Aura defs declaring
   `EnchantTarget::Permanent` (measured: `{"Imprisoned in the Moon"}`), with a message saying that
   `Permanent` is almost always a widening of a printed multi-type restriction and that a new member
   must be checked against its oracle line. This makes the class machine-visible instead of
   depending on the next reviewer noticing.
3. If a def edit stays out of scope (the batch pins `git diff -- crates/card-defs/` EMPTY), say so
   at the def in a comment — do not leave a deck-legal `Complete` Aura's widened restriction with no
   record anywhere in the tree.

---

#### Finding E1: T1's equivalence claim is materially overstated — it is blind to narrowing and to every player-side error

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_dx20_keyword_carried_target_requirements.rs:191-299`
**CR Rule**: 702.5d — "Auras that can enchant a player can target and be attached to players. Such
Auras can't target permanents and can't be attached to permanents."

**Issue**: T1 asserts `legal_targets_per_slot(...).contains(cand) == process_command(...).is_ok()`.
Both sides now run the **same** synthesized requirement — the offer side through
`validate_targets_inner`, the cast side through `validate_targets_with_source`. The only
Aura-specific cross-check left on the cast side is the redundant CR 303.4a gate at
`casting.rs:3746-3798`, and it does two things that bound what T1 can see:

* it can only ever **reject** — so any mapping that is a strict *subset* of the incumbent leaves
  both sides rejecting, and T1 stays green;
* its loop is `if let Target::Object(target_id) = st.target` (`:3758`) — so **player targets are
  never cross-checked at all**, and any player-side mapping error leaves both sides agreeing.

Two concrete, plausible regressions that ship silently under T1:

1. `EnchantTarget::Permanent -> TargetCreature` (a "tightening" a future author might make while
   looking at Rancor). Offer lists creatures only; a cast naming an artifact fails the requirement,
   and the gate's `Permanent => true` arm never disagrees. T1 green, and every "Enchant permanent"
   Aura silently stops being able to enchant artifacts, lands and enchantments. **No other probe
   pins `Permanent`'s mapping** — T2.1 pins only `Creature`, T5.1 only Reconfigure.
2. `CreatureOrPlaneswalker -> TargetAny` — **the exact mistake the plan's §3.2 warns against in
   bold**. `validate_player_satisfies_requirement` accepts a player for `TargetAny`
   (`casting.rs:6388-6391`), and the gate skips player targets, so offer and cast both accept a
   player. T1 green, CR 702.5d violated, and the resulting attachment is unrepresentable.

T1 is still valuable — the executed revert (`Filtered.controller -> Any`) reddened exactly as
predicted, and it does catch every *loosening* on the object side. The problem is the claim, not
the probe.

**Fix**: add a table-driven exact-shape probe alongside T1 —
`assert_eq!(<the requirement spell_target_requirements returns>, <expected>)` for all 9 variants,
listing the expected `TargetRequirement` literally (T2.1's shape, generalised). That is the only
assertion that discriminates narrowing. Additionally, assert per-variant on the **player** rows
specifically: for the 8 non-`Player` variants, both `Target::Player(p1)` and `Target::Player(p2)`
must be rejected; for `Player`, both must be accepted and all 9 objects rejected. Correct T1's doc
comment and plan §6 T1's "in either direction" wording to state the two blind classes.

---

#### Finding E2: five of the six `EnchantFilter` fields are never instantiated, and one of them is live in the corpus

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_dx20_keyword_carried_target_requirements.rs:201-206`
**CR Rule**: 205.4a (basic/nonbasic supertype)

**Issue**: T1's single `Filtered` row is
`{ has_card_type: Land, has_subtype: Mountain, controller: You }`. It therefore exercises 3 of the
6 fields the §3.3 mapping table argues about. Never exercised: `basic`, `nonbasic`, `has_subtypes`
(the OR vector), `EnchantControllerConstraint::Opponent`, `EnchantControllerConstraint::Any`.

`basic` is not hypothetical. Measured from the corpus:

* `crates/card-defs/src/defs/ossification.rs:12-17` — `basic: true, controller: You`, printed
  "Enchant basic land you control", **`Complete`**;
* `crates/card-defs/src/defs/dimensional_exile.rs:13-18` — identical shape, **`Complete`**.

Both are in this batch's 13. A transposition in the mapping (`nonbasic: f.basic`, or the two lines
swapped) inverts the restriction: the browser would offer every *nonbasic* land the caster controls
for Ossification, and the cast would be refused by the gate — the 422 this batch exists to remove,
reintroduced through the door T1 was supposed to be watching. The board fixture already contains the
discriminating pair (`Own Mountain`, Basic; `Own Karoo`, nonbasic), so this is cheap to close.

**Fix**: add three more `Filtered` rows to T1's `variants` vector — `{ has_card_type: Land,
basic: true, controller: You }` (mirrors Ossification exactly), `{ has_card_type: Land,
nonbasic: true }`, and `{ has_subtypes: [Mountain, Forest] }` — plus one row with
`controller: Opponent` (the board's `Opp Mountain` discriminates it against `Own Mountain`). Each
keeps T1's existing non-vacuity floor, so no new assertion machinery is needed.

---

#### Finding E3: `OOS-DX20-8` states the wrong divergence, and the divergence it omits is a new hard rejection

**Severity**: MEDIUM
**File**: `docs/audits/decision-point-audit.md:1190`
**CR Rule**: 700.2c / 303.4a

**Issue**: the seed says the two paths "agree for every reachable shape (`mode_targets: None`, or a
non-empty per-mode slice), and differ only if `per_mode_target_requirements` returns
`Some(vec![])`". Traced against source, the non-empty per-mode slice case does **not** agree:

* **Cast** (`casting.rs:3636`, then `:3696-3708`): the synthesis runs *first*, so `requirements` is
  `[<enchant req>]` — non-empty. `mode_targets_active` is then `Some(non-empty)`, and the guard at
  `:3704` fires: `InvalidCommand("modal spell has both Spell.targets and ModeSelection.mode_targets
  set …")`. The cast is impossible, for any target list.
* **Query** (`queries.rs:136-142`): `per_mode_target_requirements` returns the non-empty slice,
  guard 3 (`base.is_empty()`) suppresses the synthesis, and the query reports the per-mode list as
  announceable.

Pre-batch this shape was castable (`requirements` was `vec![]`, so the guard never fired). So the
batch introduces a **new unconditional rejection** for it, which is worse than a target-list
mismatch. The plan *did* anticipate this — §7 R6, "the modal guard now has a new way to fire — a
modal Aura" — but the seed that is supposed to be the durable record contradicts it. A successor
reading only the seed would fix the `Some(vec![])` case and leave the more consequential one open.

The "gated by T4" half of the seed is **correct**, and I verified it independently: `ModeSelection`
is reachable only through `AbilityDefinition::Spell { modes: Some(..) }` (`casting.rs:5540`), which
T4 assertion 4 pins as empty across `all_cards()`. Corpus exposure for both shapes is 0.

**Fix**: reword `OOS-DX20-8` to name **both** shapes, and state that the non-empty-slice case is a
behaviour change (castable before, hard-rejected after) rather than a query/cast disagreement. Cite
plan §7 R6 and `casting.rs:3704`.

---

#### Finding E4: `OOS-DX20-7` asserts a comment that was never written

**Severity**: MEDIUM
**File**: `docs/audits/decision-point-audit.md:1189` vs `crates/engine/src/rules/abilities.rs:533-582`

**Issue**: the seed says the legacy `Effect::AttachEquipment` guard "is deliberately **kept** (it
still covers card-def-authored equip abilities) and **now carries a comment saying so**". Plan §5
step 5 instructed exactly that ("add a comment saying so rather than removing it"). The comment at
`abilities.rs:533-538` is the pre-existing one — "Legacy special-case check for AttachEquipment
effects. Cards with proper TargetRequirement declarations will be validated by the general check
above." It says nothing about the empty-vec permissiveness that is the seed's actual subject, and a
`rg 'DX20' crates/engine/src/rules/abilities.rs` returns nothing.

This is precisely the aspirationally-wrong-comment class the batch's own plan invokes
(`memory/conventions.md`), inverted: a *seed* asserting a code state that does not exist. It matters
because the seed is the register a successor greps.

**Fix**: either write the comment at `abilities.rs:539` (one sentence: the guard is redundant with
declarative validation for every ability that carries a `TargetRequirement`, and silently permissive
for any that does not, because `targets.first()` on an empty vec is `None` — `OOS-DX20-7`), or strike
the claim from the seed row. Writing the comment is the better half; the guard is the last place a
Fortify-shaped def can still fizzle with the cost paid.

---

#### Finding E5: the committed T6 was never watched failing; a deleted scratch function was

**Severity**: MEDIUM
**File**: `tools/play-server/src/main.rs:9979-10031`
**Evidence**: `scratchpad/dx20-reverts.md` §T6

**Issue**: acceptance criterion 1 asks for a discriminating browser-path probe "watched failing by
revert". The recorded T6 revert used a throwaway `scratch_dx20_t6_revert_observer` that searched for
"Cast Rancor" **by label alone** and submitted `"params": {}`, because under the revert
`target_slots` never populates. That function was deleted. The committed
`test_dx20_t6_rancor_castable_with_a_real_target_over_http` searches for Rancor **with a non-empty
`target_slots[0].candidates`** (`:9919-9926`), so under the same revert it would fail in the *drive
loop* (timeout or "the game ended … without Rancor ever being offered"), not at the 200-vs-422
assertion the test is named for. The revert log records only that the committed test was re-run
**green after restore**.

It is very likely the committed test does redden — but "very likely" is the standard this project
explicitly rejects (the plan's own §7 R5 insists on confirming `Compiling` in every revert before
trusting it). Nothing in the tree records the committed probe's failure mode.

**Fix**: re-apply the T2-class revert (`aura_spell_target_requirements` body → `let _ = chars; base`)
with `#[allow(dead_code)]` on `enchant_target_to_requirement`, run
`cargo test -p play-server test_dx20_t6 -- --nocapture`, confirm `Compiling` in the output, and
record the verbatim failure in the revert log. If it fails in the drive loop rather than at the
status-code assertion, say so — and consider splitting the drive predicate (find Rancor by label,
then assert `target_min == 1` and the non-empty slot) so the committed probe reddens on the
assertion it advertises.

---

#### Finding E7: `test_dx20_t5_5`'s CR 602.2c assertion cannot fail

**Severity**: LOW
**File**: `crates/engine/tests/primitives/pb_dx20_keyword_carried_target_requirements.rs:766-789`
**CR Rule**: 602.2c

**Issue**: the probe is billed as "the discriminating one" and its doc says "critically the mana was
NOT spent". `activate_attach(state.clone(), ...)` moves a *clone* into `process_command`, which takes
`GameState` by value and returns `Err` with no state. Re-reading the caller's `state` at `:784`
therefore observes an object no code path could have touched. The assertion is structurally
guaranteed. The in-test comment is honest about the mechanism but the doc comment above it still
claims the property is being tested.

Note this is not a hole in the engine — CR 602.2c's rewind is guaranteed *by the API shape*, which
is a stronger property than any test could assert. The problem is a decorative assertion carrying a
CR citation.

**Fix**: delete the assertion and replace the doc line with the correct statement — "`process_command`
takes `GameState` by value and returns `Err` without a state, so CR 602.2c's rewind is structural
here and is not what this probe tests; what it tests is that a zero-target attach is `Err` at all,
where before PB-DX20 it was `Ok` with a silent fizzle." Optionally add a real observation: after the
rejection, activating the same ability *with* a legal target on the same `state` still succeeds,
proving no cost was consumed by the failed attempt.

---

#### Finding E8: two near-tautological assertions in T6

**Severity**: LOW
**File**: `tools/play-server/src/main.rs:9995-10006`

**Issue**: (a) `assert!(!candidates.is_empty(), ...)` at `:9995` cannot fail — the drive loop's own
search predicate at `:9923-9925` already required a non-empty `target_slots[0].candidates` before
returning. (b) `target_label.contains("Llanowar Elves") || !target_label.is_empty()` at `:10003`
reduces to `!target_label.is_empty()`; the left disjunct never decides anything. The fixture is
constructed so the only creature on the human's battlefield *is* Llanowar Elves, so the strong form
is available for free.

**Fix**: drop (a); change (b) to `assert!(target_label.contains("Llanowar Elves"), ...)`.

---

#### Finding E9: two probes assert only `is_err()`

**Severity**: LOW
**Files**: `pb_dx20_...rs:429-434` (T2.4 hexproof), `:361-376` (T2.2 two-target half)

**Issue**: T2.4 exists to pin that CR 702.11b applies to Aura targets and was unaffected by this
batch. `assert!(result.is_err())` would also pass if the cast failed for an entirely different
reason — e.g. if the `Creature` mapping broke so that no creature were a legal target at all. The
same probe would then report "hexproof still works" while the batch's core mapping was broken. T2.2's
two-target half has the same shape. (Its zero-target half correctly matches on `InvalidTarget`.)

**Fix**: match `GameStateError::InvalidTarget(msg)` and assert `msg` contains a hexproof/protection
substring in T2.4; assert `InvalidTarget` and the `"expected 1..=1 target(s) but got 2"` substring in
T2.2's second half.

---

## What the plan called a hazard and is in fact correct

These were checked in source rather than taken on the plan's word, and each holds:

* **§4.3 — hexproof / shroud / protection.** `validate_mapped_targets` runs
  `super::validate_target_protection` unconditionally for any `Target::Object` in
  `Battlefield | Stack` (`casting.rs:6324-6336`) and the player-protection checks for any
  `Target::Player` (`:6253`+). `req` is consulted only afterwards at `:6340-6342`. **The synthesis
  changes nothing here**, exactly as claimed, and T2.4 is a correct (if weak — E9) pin.
* **§3.2 / §3.3 — the mapping is exact.** All 9 `EnchantTarget` variants and all 6 `EnchantFilter`
  fields re-derived by hand against `sba.rs:1014-1092` and `casting.rs:6536-6641` +
  `effects/mod.rs:9721-9848`. `matches_filter` checks nothing beyond the six mapped fields when the
  rest are at their defaults (every unmapped field is `None`/empty/`false` and short-circuits), and
  `TargetPermanentWithFilter`'s controller/self/combat/tapped conjuncts are all no-ops at the
  defaults. `TargetAny` is correctly **not** used for `CreatureOrPlaneswalker`.
* **§5 step 5 — Reconfigure.** `replay_harness.rs:4004-4009` matches CR 702.151a verbatim
  ("another target creature you control": `controller: You` + `exclude_self: true`), and correctly
  does **not** copy CARDS-1's `exclude_self`-free equip repair. The detach ability at `:4026-4039`
  correctly stays `targets: vec![]` — CR 702.151a's second ability takes no target. `exclude_self`
  **is** honoured on the activated-ability path: `abilities.rs:501-508` calls
  `validate_targets_with_source(..., source)`, i.e. `self_id = Some(source)`, which
  `validate_object_satisfies_requirement`'s `passes_self` (`casting.rs:6567`) consumes.
* **The ordering asymmetry is genuinely gated.** `spell_mode_selection` reads
  `AbilityDefinition::Spell { modes: Some(m), .. }` only, so T4 assertion 4 ("no Aura def carries an
  `AbilityDefinition::Spell`") is a true precondition for *both* modal divergences. The seed's
  gating claim is correct; only its enumeration is wrong (E3).
* **The `KNOWN_FALSE_OFFERS` deletion loses no coverage.** The old loop's fall-through only ran
  *after* a refusal that the register excused; an unlisted refusal already panicked. The new form
  panics on any refusal, which is strictly stronger. The surviving `assert!(advanced, ...)` is now
  reachable only when a decision offers zero actions (`candidates` always includes
  `actions.iter().take(1)`) — narrower than before, but still a real degenerate-state check, not
  dead code.
* **Step 2's placement (the `else` branch) is right.** Both paths return `vec![]` for overload
  before any synthesis, so an overloaded Aura cannot acquire a requirement the query does not report.
* **Guard ordering avoids a per-cast allocation.** `aura_spell_target_requirements` tests
  `card_types.contains(&Enchantment)` *before* `subtypes.contains(&SubType("Aura".to_string()))`, so
  the `String` allocation is short-circuited away for every non-enchantment spell.
* **No `.unwrap()` in new library code**; no new `TargetRequirement` variant, so PROTOCOL/HASH are
  unmoved by construction as well as by gate execution.
* **SR-5 is respected**: `keyword_registry.rs:82-97` adds `queries.rs` as an `Enchant` handling site
  with a stated justification ("a query that reads the keyword IS a handling site").
* **`plan_targets`'s "first legal candidate" policy already names Auras** as a case where the policy
  is strategically wrong (`targeting.rs:184-186`), so `OOS-SIM5-1`'s scope did not need widening when
  bots started casting them.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 303.4a | Yes | Yes | T1, T2.1, T2.2, T6; count check now live via `validate_targets_inner` |
| 303.4d | n/a (SBA, unchanged) | No | Aura in hand/stack is never `on_battlefield`, so self-targeting is unreachable at cast |
| 303.4g | No (out of scope) | No | `OOS-DX20-3` — an Aura with no legal target is still offered |
| 601.2c | Yes | Yes | T2.2 zero-target → `InvalidTarget`; `enchant.rs:507` variant change is the batch's own evidence |
| 602.2c | Structural (API shape) | **No** — E7 | `process_command` by value; the assertion cannot fail |
| 700.2c | Partial | Gated | E3 — modal Aura, 0 corpus exposure, gated by T4 assertion 4 |
| 702.5a | Yes | Yes | T1 over all 9 variants (with the blind classes of E1/E2) |
| 702.5c | **No, deliberately** | Yes (gate) | `get_enchant_target`'s `find_map`; `OOS-DX20-1`; T4 pins exposure at 0 |
| 702.5d | Yes (object side) | Partial | Player side mapped but never cross-checked by T1 — E1; `OOS-DX20-2` gates the corpus |
| 702.11b / 702.16b / 702.18a | Yes (pre-existing) | Weakly — E9 | Verified unconditional in `validate_mapped_targets` |
| 702.103b | Yes | Yes | T3, both directions (`None` → `vec![]`, `Some(Bestow)` → `[TargetCreature]`) |
| 702.151a | Yes | Yes | T5.1-T5.6; `exclude_self` revert executed and correctly partitioned |
| 205.4a | Yes (mapped) | **No** — E2 | `basic`/`nonbasic` never instantiated in any probe |

---

## Card Def Summary

All 23 `KeywordAbility::Enchant`-carrying defs enumerated; the 13 `Complete` ones read against MCP
oracle text on 2026-08-04. **0 defs were edited by this batch** (correct — the brief pins that).

| Card | Completeness | Declared | Oracle | Match | Notes |
|------|-------------|----------|--------|-------|-------|
| `rancor` | Complete | `Creature` | "Enchant creature" | Yes | T6's fixture card |
| `imprisoned_in_the_moon` | Complete (derive) | `Permanent` | "Enchant creature, land, or planeswalker" | **NO** | **C1 — HIGH**, wider than printed |
| `kasminas_transmutation` | Complete | `Creature` | "Enchant creature" | Yes | |
| `awaken_the_ancient` | Complete | `Filtered{Land, Mountain}` | "Enchant Mountain" | Yes | no controller clause, correctly `Any` |
| `darksteel_mutation` | Complete | `Creature` | "Enchant creature" | Yes | |
| `ossification` | Complete | `Filtered{Land, basic, You}` | "Enchant basic land you control" | Yes | exercises `basic` — untested, E2 |
| `sigil_of_sleep` | Complete | `Creature` | "Enchant creature" | Yes | |
| `wild_growth` | Complete | `Land` | "Enchant land" | Yes | |
| `chained_to_the_rocks` | Complete | `Filtered{Land, Mountain, You}` | "Enchant Mountain you control" | Yes | the shape T1's `Filtered` row mirrors |
| `hyena_umbra` | Complete | `Creature` | "Enchant creature" | Yes | the card that stopped CARDS-2's driver |
| `kenriths_transformation` | Complete | `Creature` | "Enchant creature" | Yes | |
| `dimensional_exile` | Complete | `Filtered{Land, basic, You}` | "Enchant basic land you control" | Yes | second `basic` user |
| `eaten_by_piranhas` | Complete | `Creature` | "Enchant creature" | Yes | |
| `kayas_ghostform` | partial | `Creature` | "Enchant creature or planeswalker you control" | No | already filed as `OOS-DX20-5` |
| `aqueous_form`, `elvish_guidance`, `curiosity`, `smoke_shroud`, `shiny_impetus`, `breath_of_fury`, `ophidian_eye`, `bear_umbra`, `crown_of_skemfar` | partial (9) | — | — | n/a | not deck-legal; unrelated trigger/DSL gaps |
| `animate_dead`, `curse_of_opulence` | inert (2) | *no `Enchant` keyword* | — | n/a | T4 assertion 1 pins the set and the `inert` marker |
| `lizard_blades` | Complete | (Reconfigure, not Enchant) | "Reconfigure {2}" | Yes | requirement now CR 702.151a-correct |
| `boon_satyr` | Complete | (Bestow → `Creature` at cast) | "Bestow {4}{G}{G}" | Yes | T3; query-side transform mirrors `casting.rs:983-988` |

---

## Acceptance Criteria Assessment

| # | Criterion | Met? | Note |
|---|-----------|------|------|
| 1 | Offer + `casting.rs` derive from literally the same function; 13 Auras castable; discriminating probe watched failing | **Partial** | The shared-function half is fully met (`casting::aura_spell_target_requirements`, one definition, two callers). The revert-watched half is met for T1/T2/T3/T4/T5 but **not** for the committed T6 (E5). And one of the 13 is offered *wrongly* (C1). |
| 2 | Reconfigure requirement + t7b pin + probe watched failing | **Yes** | Requirement matches CR 702.151a; t7b prose rewritten with the Fortify half deliberately left open; the `exclude_self: false` revert reddened T5.2 and left T5.3/T5.4/T5.5 green, exactly the partition the plan demanded. |
| 3 | Second-failure-mode probe (Aura defs with no `Enchant`) | **Yes** | T4 assertion 1 pins `{"Animate Dead", "Curse of Opulence"}` **and** their `inert` markers, correcting the brief's "4" to the measured 2. |
| 4 | `KNOWN_FALSE_OFFERS` deleted with a staleness assertion proving it | **Yes** | Constant gone from the tree; the refusal path is now an unconditional panic naming the label and reason. Strictly stronger. |
| 5 | Full-workspace tests, PROTOCOL/HASH gate-executed | **Yes** (coordinator-measured) | 4,387 / 0 / 5, PROTOCOL 35 / HASH 72 unmoved, clippy + both fmt gates clean, `crates/card-defs/` diff empty. |
| 6 | Seeds dispositioned and filed | **Partial** | `OOS-DX20-1..9` filed in the audit registry and largely excellent. Two are wrong as written (E3, E4) and one live-wrong `Complete` card is unfiled (C1). Plan §9's `primitive-wip.md` / `workstream-state.md` bookkeeping is outstanding (E12). |

---

## Summary Counts

**1 HIGH / 5 MEDIUM / 7 LOW.**

The HIGH is a card def, not the primitive: the primitive is correct, and I verified its equivalence
argument independently rather than accepting it. The MEDIUMs are concentrated in evidence quality —
T1 proves less than it claims, two seeds describe a tree that does not exist, and the headline
browser probe's discrimination is unrecorded. None of the MEDIUMs indicates a live wrong game state;
C1 does.
