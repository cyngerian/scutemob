# Primitive Batch Review: PB-DX24 — the lowering drops `trigger_zone`; the two index spaces disagree

**Date**: 2026-08-05
**Reviewer**: primitive-impl-reviewer (Opus)
**Task / branch**: `scutemob-202` · `feat/pb-dx24-the-lowering-drops-triggerzone-the-two-index-spaces-`
**CR Rules verified independently via MCP**: 113.6 (+ 113.6b/113.6k/113.6m), 603.10 (+ 603.10a),
603.6c, 108.4a, 400.7, 111.7, 118.12, 712.2, 712.8a–e, 712.10, 712.13, 712.14, 712.15, **712.16**,
708.2b, 708.3
**Engine files reviewed**: `crates/engine/src/testing/replay_harness.rs`,
`crates/engine/src/rules/abilities.rs`, `crates/engine/src/rules/resolution.rs` (comment only);
corroborating reads in `rules/face.rs`, `rules/sba.rs`, `rules/events.rs`, `rules/casting.rs`,
`effects/mod.rs`, `state/mod.rs`, `state/diagnostics.rs`, `state/builder.rs`,
`crates/simulator/src/setup.rs`
**Test files reviewed**: `crates/engine/tests/primitives/pb_dx24_trigger_zone_and_index_spaces.rs`
(15 tests), `crates/engine/tests/core/pb_dx24_trigger_zone_roster.rs` (5 tests), plus the two
`main.rs` `mod` lines
**Card defs reviewed**: `crates/card-defs/src/defs/nether_traitor.rs` (1, comment-only) —
plus completeness/marker reads on `bloodghast`, `squee_goblin_nabob`, `teysa_karlov`,
`drivnod_carnage_dominus`, `ancient_greenwarden`, `elesh_norn_mother_of_machines`
**Docs reviewed**: `docs/audits/decision-point-audit.md` rows `OOS-DX1-3`, `OOS-DX1-4`,
`OOS-DX24-1..6`

> **Tooling caveat, stated up front**: this reviewer had no shell. Every claim below is derived by
> reading source, not by executing `cargo test`, `git diff --numstat`, or a revert. Where the
> execution notes assert a measurement I could not re-run, I say so; where I could verify the
> underlying source fact independently, I did and say which lines.

---

## Verdict: needs-fix

**0 HIGH / 6 MEDIUM / 7 LOW.** The batch's core is right and I could not find a correctness defect
in the shipped engine behaviour. The CR 113.6m derivation is correct against the rule text; the
graveyard death arm mirrors the battlefield `AnyCreatureDies` arm clause for clause with no
semantic drift (including the `fizzle_object`-vs-`state.objects.get` question, which turns out to
be a distinction without a difference — `diagnostics.rs:373` is literally `self.objects.get(&id)`);
the CR 603.10a look-back guard is applied to the death arm and correctly withheld from the ETB arm;
all six Q-sites read the trigger source's own face; the extraction is behaviour-preserving and the
activated/mana loops still see the unfiltered slice, so graveyard-**activated** abilities
(`activation_zone`, a different field) are untouched; and the fix reaches production, not just the
test harness (`crates/simulator/src/setup.rs:419/433` builds real games through
`enrich_spec_from_def` → `build_face_ability_vectors`). The six MEDIUMs are all in the *justification
and evidence* layer rather than the behaviour layer, which is the layer this project's own history
says kills it: one in-source CR citation is wrong (712.2, should be 712.16) and has already been
copied into the audit doc; one new gate does not pin the invariant its failure message claims to
pin, and the batch's own Q3/Q4/Q6 probes demonstrate the hole; one filed seed states a corpus fact
that is false in the direction that would over-rank it; one roster gate pins a count instead of the
seven-shape measurement it says it backs; one plan-mandated investigation (slice granularity) was
not done and a real over-suppression follows from it; and one test cites CR 118.12 for the exact
opposite of what CR 118.12 says.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `rules/resolution.rs:7677` (+ `docs/audits/decision-point-audit.md:1213`) | **Q5's re-scope cites the wrong rule.** CR 712.2 is about DFC face symbols; the rule that forbids turning a DFC permanent face down is **CR 712.16**. And CR 712.15 explicitly allows a DFC *card* to enter the battlefield face down, so the site IS reachable. **Fix:** re-cite 712.16 and add the engine-level reason (below). |
| 2 | MEDIUM | `tests/primitives/pb_dx24_trigger_zone_and_index_spaces.rs:1426-1481` | **The §4.0 pin does not pin §4.0.** It counts only literal `is_transformed = true` / `: true`; `rules/face.rs:97` writes a computed bool and is how `Command::Transform` sets it true — the batch's own Q3/Q4/Q6 probes assert exactly that. **Fix:** replace with a runtime probe of the real invariant. |
| 3 | MEDIUM | `rules/abilities.rs:2988-2998` + `memory/primitives/pb-DX24-execution-notes.md` | **Plan risk #2 (slice granularity) was never discharged, and it hides a real over-suppression.** `resolution.rs:8118` passes a whole resolution's events. **Fix:** record per-caller granularity at the set's construction and file the deviation. |
| 4 | MEDIUM | `docs/audits/decision-point-audit.md:1358` (`OOS-DX24-1`) | **The seed's corpus fact is wrong and its class is understated.** Both `CreatureDeath` doublers are `Completeness::partial` (deck-illegal), not `Complete`; and `LandETB`/`AnyPermanentETB` are equally source-blind and already double Bloodghast today. **Fix:** correct the marker claim, widen to the predicate. |
| 5 | LOW | `rules/abilities.rs:5006` | The new graveyard sweep runs per `CreatureDied` event and clones every graveyard object's `CardId`. No bench recorded. **Fix:** re-run `full_turn_4p` / `sba_check` and pin. |
| 6 | LOW | `testing/replay_harness.rs:2446-2447` | Change 2's instruction to restate the arm counts *with the counting rule* was not done. **Fix:** add the rule text (I re-derived 34 independently, see below). |
| 7 | LOW | `rules/abilities.rs:4174`, `:5225` (Q3/Q4) | The "queue and read are now the same expression" claim is an expression identity, not a *time* identity — a permanent that transforms between queue and resolution still desyncs, and in the true→false direction the fix converts an accidentally-correct read into a wrong one. Zero corpus exposure. **Fix:** state the residual at the sites or file a seed. |

## Test / Gate Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 8 | MEDIUM | `tests/core/pb_dx24_trigger_zone_roster.rs:214-239` (R2) + `:161-175` (R1) | **R2 pins a count, not the seven-shape measurement it says it backs; R1 walks only front faces.** **Fix:** walk `back_face` in both; assert the 7 per-shape counts. |
| 9 | MEDIUM | `tests/primitives/pb_dx24_trigger_zone_and_index_spaces.rs:412` (T3) | **The citation contradicts the rule.** CR 118.12 makes it a *player choice*; the test asserts "pay-when-able" and pins the engine's auto-pay deviation as if it were CR. **Fix:** re-word + file the plan-risk-#9 observation. |
| 10 | LOW | `tests/primitives/…:222-281` (T7) | The corpus differential feeds only `def.abilities`; back-face inputs are never differentiated. **Fix:** iterate `def.back_face` too. |
| 11 | LOW | `tests/primitives/…:1490-1515` | The Q2/Q7 structural pin scans raw source with no comment stripping and a hard 8-line window. **Fix:** reuse the roster file's `strip_comments`; widen or brace-scan the window. |
| 12 | LOW | `crates/card-defs/src/defs/nether_traitor.rs:71-73` | Stray blank line inside the struct literal between `targets: vec![],` and `modes: None,`; and the execution notes claim `crates/card-defs/` is untouched while the file plainly carries PB-DX24 comments, so `tools/check-defs-fmt.sh` (SR-35) may never have been run against the edit. **Fix:** run it; drop the blank line; correct the notes. |
| 13 | LOW | `docs/audits/decision-point-audit.md:1363` (`OOS-DX24-6`) | Accepted as filed — the `pub(crate)` → `pub` widening is defensible and is already self-reported. Noted here only so the reader knows it was checked, not missed. |

---

## Finding Details

### Finding 1 — Q5's re-scope cites CR 712.2, which says nothing about face-down (MEDIUM)

**File**: `crates/engine/src/rules/resolution.rs:7673-7689`; copied verbatim into
`docs/audits/decision-point-audit.md:1213`.

**Shipped text**: *"CR 712.2 forbids turning a transforming double-faced card face down, so a
`PermanentTurnedFaceUp` source can never be a transformed DFC."*

**CR, read from the MCP**: CR **712.2** is *"Nonmodal double-faced cards have a Magic card face on
each side and include abilities on one or both of their faces that allow the card to either
'transform' or 'convert' …"* — plus 712.2a/b/c on the front-face and back-face **symbols**. It says
nothing about face-down at all. The rule the comment wants is **CR 712.16**: *"Melded permanents and
other double-faced permanents can't be turned face down. If a spell or ability tries to turn a
double-faced permanent face down, nothing happens."*

**And 712.16 alone is still not sufficient.** CR **712.15**: *"If an effect allows a player to cast a
double-faced card as a face-down creature spell, **or if a double-faced card enters the battlefield
face down**, it will have the characteristics given to it by the rule or effect that caused it to be
face down."* A manifested (CR 701.40a) or cloaked (CR 701.58a) DFC therefore *does* reach a
turn-face-up site — 712.16 only stops a face-**up** DFC permanent from being turned face down. So the
site is reachable; what makes `is_transformed` false there is the **engine's** write discipline, not
a CR prohibition:

- `resolution.rs:853` is the only literal `is_transformed = true` (disturb ETB);
- `rules/face.rs:97` is the only other writer, and it returns early at `:67-69` when
  `obj.zone != ZoneId::Battlefield`;
- `state/mod.rs:1404`, `:1558`, `:1678`, `:1900` reset it to `false` on every zone change
  (CR 712.8a / 400.7).

**Why this matters here specifically**: this project's own CLAUDE.md records that *"that comment, not
the code, is why a HIGH survived 4.5 months"* (PB-DX19). A comment that tells a future reader "CR
forbids it" when CR does not is the same failure mode, and it has already been propagated into the
audit registry, where it will be read as settled.

**Fix**: at `resolution.rs:7677` replace the CR 712.2 clause with: (a) CR **712.16** for the
face-up-DFC-can't-be-turned-face-down half; (b) an explicit note that CR 712.15 leaves
manifest/cloak of a DFC reachable, and that the reason `is_transformed` is nonetheless false there
is `face.rs:67-69`'s battlefield gate plus the four `state/mod.rs` zone-change resets — cite those
lines. Apply the same correction to `docs/audits/decision-point-audit.md:1213`.

---

### Finding 2 — `test_dx24_is_transformed_true_assignment_has_exactly_one_site` does not gate what it claims (MEDIUM)

**File**: `crates/engine/tests/primitives/pb_dx24_trigger_zone_and_index_spaces.rs:1426-1481`.

**Its message**: *"`is_transformed = true` / `is_transformed: true` must be assigned at EXACTLY ONE
site in crates/engine/src … a second site means is_transformed is reachable off the battlefield, and
Q2/Q7's 'defensive, zero-behaviour-change' classification is no longer sound as stated."*

**The hole**: the scanner matches only the two literal strings. `rules/face.rs:97` writes
`obj_mut.is_transformed = new_is_transformed;` — a computed bool. That is the site through which
`Command::Transform` makes it true, and **this batch's own Q3/Q4/Q6 probes assert exactly that**
(`assert!(state.objects()[&obj_id].is_transformed)` at `:1115`, `:1238`, `:1374`). So a second
true-producing writer already exists in the tree and the gate is green — its stated trigger
condition ("a second site") is already met and it does not fire.

The real invariant Q2/Q7 rest on is *"`is_transformed` can only become true on a battlefield
permanent, and is reset on every zone change."* That is enforced by `face.rs:67-69` +
`resolution.rs:853` (an ETB path) + `state/mod.rs:1404/1558/1678/1900`. **None of those four facts is
gated by anything.** A patch that deletes `face.rs:67-69`'s zone guard — the single most likely way
to break Q2/Q7's premise — leaves this test green.

Secondary: unlike G-A/G-B in the sibling file, this scanner does **not** strip block comments, so a
`/* obj.is_transformed = true; */` would be counted (false red) and, more importantly, the
inconsistency means the PB-DX32 M8 lesson was applied in one file and not the other.

**Input that exhibits the gap**: delete the `if obj.zone != ZoneId::Battlefield { return; }` guard at
`face.rs:67-69`. Every PB-DX24 test still passes, including this pin, while Q2's and Q7's entire
justification is gone.

**Fix**: replace the source scan with a runtime probe pair in the same file —
(a) `Command::Transform` a permanent, assert `is_transformed`, then move it to
`ZoneId::Graveyard(p)` (and separately to hand and to the stack) and assert `is_transformed == false`
each time, citing CR 712.8a / 400.7; (b) call `apply_face_change` (or drive `Command::Transform`)
against an object whose zone is not `Battlefield` and assert `is_transformed` stays false, citing
`face.rs:63-69`. If a source scan is kept alongside, count **every** write to `is_transformed`
(regex on `is_transformed\s*[:=]` outside comments) and assert the set is exactly
`{resolution.rs (ETB), face.rs (battlefield-gated)}`, and use the roster file's `strip_comments`.

---

### Finding 3 — plan risk #2 was not discharged; the look-back set is coarser than one simultaneous batch at one caller (MEDIUM)

**File**: `crates/engine/src/rules/abilities.rs:2988-2998`; plan §10 risk 2; execution notes
"Findings recorded" §2 (which records only the *variant* decision, not the granularity).

The plan required, verbatim: *"The runner must establish the slice's granularity by reading
`check_triggers`' callers, record the finding, and — if the slice is coarser than one
simultaneous-event batch — **file it as a seed and state the deviation direction** rather than
guessing a refinement."* The execution notes discuss which `GameEvent` variants carry a
`new_grave_id` and never touch granularity; no `OOS-DX24-*` row covers it.

**What the callers actually do** (verified):

| caller | slice | granularity |
|---|---|---|
| `rules/sba.rs:97` | one `apply_sbas_once` pass | **exactly** one simultaneous SBA batch — correct |
| `rules/resolution.rs:8118` | the *whole* resolution's accumulated `events` vec | **coarser** — spans sequential sub-effects of one resolution |
| `rules/combat.rs:846`, `:1743`, `rules/engine.rs:34`, `:2499` | per-action event batches | not audited by the batch |

**The deviation, with an input that exhibits it**: a single resolution whose effects are ordered
"sacrifice a creature, **then** destroy target creature" pushes `CreatureDied{Nether Traitor}` and
`CreatureDied{other}` into one `events` vec. Both `new_grave_id`s land in
`arrived_in_graveyard_this_batch`; when the loop reaches the second event, `lookback_blocks` is true
for the Traitor's graveyard object and the trigger is **suppressed**. Per CR 603.10a it should fire:
the two deaths were sequential, and immediately prior to the *second* event the ability did exist in
the graveyard. Direction: **over-suppression** — the safer direction, and it matches the Gatherer
ruling in the common (SBA) case, but it must be known rather than assumed, which is exactly what the
plan said.

**Fix**: add a per-caller granularity note at the set's construction (`abilities.rs:2964-2987`)
naming `sba.rs:97` as exact and `resolution.rs:8118` as coarse; file `OOS-DX24-7` stating the
over-suppression direction, the reproduction above, and that a refinement would require the set to
be rebuilt per event-prefix rather than per slice.

---

### Finding 4 — `OOS-DX24-1` states a completeness fact that is false, and understates its own class (MEDIUM)

**File**: `docs/audits/decision-point-audit.md:1358`.

Two problems, in order of consequence.

**(a) The severity tag is wrong.** The row reads *"Corpus exposure is **real, not hypothetical**:
`teysa_karlov` and `drivnod_carnage_dominus` both carry `TriggerDoublerFilter::CreatureDeath`"* and
is tagged *"correctness, **live-wrong on 2 `Complete` defs**"*. Verified in source:
`crates/card-defs/src/defs/teysa_karlov.rs:39` and
`crates/card-defs/src/defs/drivnod_carnage_dominus.rs:38` are both
`completeness: Completeness::partial(...)`. Per Architecture Invariant 9 / SR-2, `validate_deck`
**rejects** non-`Complete` cards, so **no legal game can exhibit this**. It is reachable only in a
hand-built `GameState`. Ranking it as "live-wrong on 2 Complete defs" will over-rank it in the queue
against rows that really are deck-legal.

**(b) "This is a NEW instance introduced by this batch, not a pre-existing one it merely uncovered"
is only true of the `CreatureDeath` filter.** The *class* is older. `doubler_applies_to_trigger`
(`rules/abilities.rs:10023-10112`) is source-blind in **all four** arms — `ArtifactOrCreatureETB`,
`CreatureDeath`, `AnyPermanentETB`, `LandETB` — and none of them checks the trigger's source zone or
whether the source is a permanent. `collect_graveyard_carddef_triggers` has dispatched Bloodghast's
graveyard landfall as `TriggerEvent::AnyPermanentEntersBattlefield` since **PB-35**, so
`ancient_greenwarden` (no explicit marker → `Completeness::Complete` by derive) and
`elesh_norn_mother_of_machines` already double a graveyard-sourced trigger today, against printed
text that says *"a triggered ability of a **permanent** you control."* PB-DX24 adds a second instance
of a pre-existing class.

**Was deferring right? Yes — plainly.** The plan's risk #3 said "investigate, file, do not fix here,"
the fix touches a shared predicate every doubler routes through, and doing it inside a batch whose
whole discipline is wire-neutrality and single-mechanism edits would have been scope creep. The
runner investigated it, read the predicate, checked the corpus and wrote it down. That is the right
behaviour. The defect is in the *write-up*, not the decision.

**Fix**: (1) correct the row to say both `CreatureDeath` doublers are `Completeness::partial` and
therefore deck-illegal; retag from "live-wrong on 2 `Complete` defs" to "latent — reachable only in a
hand-built state; the deck-legal instance is the pre-existing `ancient_greenwarden` × `bloodghast`
pair." (2) Re-scope the seed from *"a permanent-source scope on the doubler predicate, which touches
every death doubler"* to *"one source-zone conjunct at the top of `doubler_applies_to_trigger`
(`abilities.rs:10020`), covering all four `TriggerDoublerFilter` arms"* — the ETB half is the same
one-line fix. (3) Name the `ancient_greenwarden` × `bloodghast` precedent so the next reader does not
re-derive it.

---

### Finding 8 — R2 pins a count, not the measurement it says it backs; R1 is front-face-only (MEDIUM)

**File**: `crates/engine/tests/core/pb_dx24_trigger_zone_roster.rs:161-175` (R1), `:214-239` (R2).

Plan §4.3 asked for *"R2 — the §4.2 back-face population, **pinned per site with the measured
numbers**"* — i.e. the seven shape counts (Backup, WhenYouCastThisSpell, WhenExertedAsAttacks,
WhenDealsCombatDamageToPlayer, WhenTurnedFaceUp, WheneverRingTemptsYou, `trigger_zone: Some(_)`),
all measured 0 at stage 1. What shipped asserts `back_face_defs.len() == 15` and then, in its
*failure message*, instructs a human to re-run the shape scan by hand. So the "0 real corpus cards
exercise any Q-shape" finding — the sole justification for every probe in this batch being synthetic
— is pinned by nothing.

**Input that exhibits it**: add `Keyword(Backup(1))` to the back face of any one of the 15 existing
`back_face: Some(_)` defs. The count stays 15, R2 stays green, the finding it backs is now false, and
Q1's probe silently keeps testing a synthetic card while a real one exists.

R1 has the mirror problem: `trigger_zone_population` iterates `def.abilities` only. A def declaring
`trigger_zone: Some(Graveyard)` on its **back** face is absent from the roster, so R1's whole point —
*"a new `trigger_zone` def must ALSO have a dispatch arm in `collect_graveyard_carddef_triggers`, or
it will silently never fire"* — is not delivered for exactly the population `OOS-DX1-4` is about.

**Fix**: (1) in `trigger_zone_population`, chain `def.back_face.iter().flat_map(|f| &f.abilities)`
onto `def.abilities.iter()`. (2) Replace R2's single count assertion with seven
`assert_eq!(count_shape(...), 0, "...")` assertions over back-face abilities (keeping the
`!back_face_defs.is_empty()` floor and the 15-count pin as an eighth), each naming the Q-site it
unblocks and telling the author to convert that probe from synthetic to real-corpus.

---

### Finding 9 — T3's CR citation asserts the opposite of CR 118.12 (MEDIUM)

**File**: `crates/engine/tests/primitives/pb_dx24_trigger_zone_and_index_spaces.rs:410-414`.

Shipped assertion message: *"CR 118.12: with `{B}` available, Nether Traitor's MayPayThenEffect must
pay (**CR 118.12 pay-when-able**) and return it to the battlefield."*

CR 118.12, read from the MCP: *"… 'If [a player] [does, doesn't, or can't]' clause **checks whether
the player chose to pay an optional cost** or started to pay a mandatory cost."* CR 118.12 is
precisely the rule that makes this a **player decision**. There is no "pay-when-able" in it.

The engine does auto-pay: `effects/mod.rs:4302-4345` runs `try_pay_optional_cost` with no suspension,
and `rules/engine.rs:1568` already names this *"the DP-19 (`MayPayThenEffect`) bug class."* So T3 is a
valid probe of current behaviour — but per Architecture Invariant 8 ("tests cite their rules
source"), a test citing a rule for the opposite of what that rule says is worse than an uncited test:
it launders a known deviation into an apparent CR requirement.

Compounding: plan §10 risk 9 required the runner to *"record whether a real choice is offered … file
the observation."* The execution notes record the end-to-end result and not the question; no
`OOS-DX24-*` row covers it.

**Fix**: re-word T3's message to *"the engine auto-pays this CR 118.12 optional cost
(`effects/mod.rs:4302` → `try_pay_optional_cost`, no player decision — the DP-19 /
`OOS-DP10-9` class); this assertion pins that deviation, not CR 118.12, which makes the payment a
choice."* File the risk-#9 observation as an `OOS-DX24-*` row noting `nether_traitor` is `Complete`
while its "you may pay {B}" is engine-chosen, and cross-reference `OOS-DP10-9`.

---

### Finding 5 / 6 / 7 / 10 / 11 / 12 (LOW) — condensed

- **5 — no performance measurement.** `abilities.rs:5006` adds a full graveyard sweep per
  `CreatureDied` event: `state.objects.values()` filtered to `ZoneId::Graveyard`, **cloning
  `obj.card_id` for every graveyard object**, then a registry lookup and an ability walk per object.
  A 4-player late-game board wipe is O(deaths × total graveyard cards) with an allocation per pair.
  The ETB path already had this shape, so the marginal cost is bounded — but no bench was run and
  none of the standard pins (`full_turn_4p`, `sba_check`, `priority_cycle_4p`) is recorded in the
  execution notes. **Fix:** run the three benches, record them next to the test count; if
  `sba_check` moves outside noise, hoist the `gy_objects` collection out of the per-event loop.
- **6 — the arm-count rule is still unstated.** Change 2 required the `intervening_if` /
  `once_per_turn` cells at `replay_harness.rs:2446-2447` to state the count **and the counting
  rule**. They still read "all 34 push sites" / "31 of 34 sites" with no rule. For the record, I
  re-derived the arm count independently and it is **34**: `rg` over `replay_harness.rs` gives 36
  `for ability in abilities {` loops, of which 2 (`:2466`, `:2483`) are the mana/activated loops
  outside the extracted function. **Fix:** append "(counting rule: `for ability in abilities` loops
  inside `build_face_triggered_abilities`; re-measured at PB-DX24 = 34)".
- **7 — queue-time vs consume-time face.** The read side is documented as the *"is_transformed at
  consume time"* contract (`resolution.rs:2177`, `:2209`); the six Q-fixes read it at **queue** time.
  They are the same *expression*, not the same *evaluation*. For a permanent that is transformed when
  the trigger queues and untransformed when it resolves, the pre-batch code was accidentally correct
  and the fixed code is wrong. Zero corpus exposure (0 back-face Q-shapes), and the fix is still
  strictly better on the reachable-today set. **Fix:** state the residual in a comment at Q3/Q4 (the
  two genuinely reachable sites) or file it; the durable answer is to snapshot the face onto
  `PendingTrigger`, which is a HASH bump and correctly out of scope here.
- **10 — T7 is front-face-only.** `build_face_ability_vectors(&def.abilities)` at `:248-249` never
  differentiates `def.back_face`. The filter itself does protect the back-face path (`face.rs:104`
  and `resolution.rs:888` both call `build_face_ability_vectors`), so this is a coverage gap, not a
  behaviour gap. **Fix:** add a second differential over
  `def.effective_abilities(true)` for every def with a back face.
- **11 — the Q2/Q7 structural pin is brittle.** `:1490-1515` reads `abilities.rs` raw (no
  `strip_comments`, unlike the sibling roster file) and uses a hard `anchor_line + 8` window. A
  `cargo fmt` reflow that pushes `effective_abilities(` to the 9th line after the anchor reddens a
  correct tree. **Fix:** import/duplicate the roster file's `strip_comments`, and scan to the end of
  the enclosing statement (or widen to 16 lines with a comment saying why).
- **12 — card-def formatting and a stale note.** `nether_traitor.rs:71-73` has a blank line inside
  the struct literal between `targets: vec![],` and `modes: None,`. More importantly, the execution
  notes assert `git diff main..HEAD --numstat -- crates/card-defs/` is **empty** and that
  `tools/check-defs-fmt.sh` was skipped "no card-def edit to check" — but the file plainly carries
  the PB-DX24 comment block (`:5-20`), so plan §5 *was* executed and SR-35 may never have been run
  against it. `cargo fmt` does not check card defs (SR-35) and this is exactly the class it catches
  (PB-DX19 hit it). **Fix:** run `tools/check-defs-fmt.sh`; remove the stray blank line; correct the
  execution notes' "Not run in this invocation" list.

---

## What I checked and found correct (so the reader knows the coverage)

### CR correctness

- **CR 113.6 / 113.6m — the load-bearing claim, and it holds.** Read from the MCP: 113.6m confines an
  ability whose effect moves the object out of a zone to that zone, *"unless its trigger condition or
  a previous part of its cost or effect specifies that the object is put into that zone."* Nether
  Traitor's effect is "return **this card from your graveyard** to the battlefield"; its trigger
  condition is about *another* creature and therefore does not put the Traitor into the graveyard;
  no earlier part of the effect does either. **So the ability functions only from the graveyard, and
  a battlefield Nether Traitor watching another creature die does nothing.** The plan's derivation is
  correct and 113.6k is correctly ruled out (this trigger condition *can* trigger from the
  battlefield in general; it is 113.6m, not the condition, that confines it).
- **CR 603.6c / 603.10a — correct classification, correct asymmetry.** "Whenever [something] is put
  into a graveyard from the battlefield" is CR 603.6c's own second example of a
  leaves-the-battlefield ability, and CR 603.10a's list of look-back exceptions leads with
  *"leaves-the-battlefield abilities."* CR 603.10a's list does **not** include enters-the-battlefield
  triggers, so withholding `lookback_blocks` from the `PermanentEnteredBattlefield` arm
  (`abilities.rs:7241-7281`) is right, and Bloodghast arriving in the graveyard alongside a land
  entering still triggers. The reasoning is written at the guard (`:7326-7332`), which is where the
  next reader will be tempted to "unify" the arms.
- **CR 108.4a** — the graveyard card has no controller and `owner` stands in; matches the existing
  `owner` binding the ETB arm already used.
- **CR 400.7 / the id spaces** — `obj_id` is a graveyard id, so `new_grave_id` is the comparison that
  can match; `pre_death_id` is a battlefield id and can never equal it. `exclude_self_blocks`
  (`:7319-7320`) compares both. Correct. Its non-discrimination under revert is honestly filed as
  `OOS-DX24-5` rather than papered over — that is exactly the right disposition.

### The `PermanentSacrificed` exclusion — verified independently, argument holds

The brief asked me to verify rather than accept it, so I read the emit sites rather than the two the
runner cited. There are **16** `events.push(GameEvent::PermanentSacrificed {` sites. I read **9**:
`casting.rs:4340` (paired with `CreatureDied` at `:4321` / `PermanentDestroyed` at `:4331`, same
`new_sac_id`), `:4360`, `:4379`, `:4399` (each paired with `ObjectPutInGraveyard` immediately above,
same id); `abilities.rs:988` (paired at `:968`/`:978`), `:1194` (paired at `:1174`/`:1184`);
`effects/mod.rs:7354` (paired at `:7336`/`:7345`), `:9296` (paired at `:9277`/`:9287`);
`resolution.rs:8069` (paired with `CreatureDied` at `:8075`), `:1511`/`:1536`/`:1565` (devour, each
paired with `CreatureDied` or `ObjectExiled`). Every one carries an id already contributed by an
included variant. **The redundancy argument is sound.**

And the third reason the audit row gives is better than either: `rules/events.rs:466-467` documents
`PermanentSacrificed.new_id` as *"New ObjectId in graveyard (**or exile if replaced**)"* — verified —
so feeding it into a graveyard-arrival set would be type-wrong even where it happened to be right.
Good catch, correctly recorded.

### The clause-for-clause mirror — no divergence, including the one I expected to find

I went looking for a lookup-path asymmetry: the battlefield arm uses `state.objects.get(&dying_obj_id)`
(`abilities.rs:4901`, `:4948`) while the graveyard arm uses `state.fizzle_object(*new_grave_id)`
(`:7324`, `:7343`). `state/diagnostics.rs:373` settles it — `fizzle_object` is **literally**
`self.objects.get(&id)` and its doc says explicitly *"It returns **no** last-known information."*
The two arms are semantically identical; the conversion was made only to satisfy the SR-25
bare-lookup ratchet, and it did not change behaviour. `None` handling also matches (battlefield
`continue`s, graveyard yields `false`). `controller_you`/`controller_opponent`, `nontoken_only`
(CR 111.7), the `f.is_token` pre-check and the `pre_death_characteristics`-else-base fallback
(CR 603.10a / 613.1d) all mirror `:4923-4971` exactly. **No clause makes the same card behave
differently in the two zones.**

Two rulings cross-checked against the implementation: *"If multiple creatures are put into your
graveyard at the same time, Nether Traitor's ability triggers for each of them"* — satisfied for free
by `check_triggers`' per-event loop (`abilities.rs:2999`), with the look-back set containing only the
dying creatures' ids and not the Traitor's, so nothing suppresses the N triggers. *"A token you own
that dies is put into your graveyard before it ceases to exist"* — `nontoken_only: false` and
`filter: None` mean the `matched` path returns `true` without needing the object, so a token death
counts. Both correct.

### The extraction and the retyping

- `build_face_triggered_abilities` (`replay_harness.rs:2570`) takes `&[&AbilityDefinition]`; each arm
  is `for ability in abilities { if let AbilityDefinition::Triggered { .. } = ability { … } }` and
  binds through two references by default binding modes, so no arm needed editing. Verified
  structurally: the **only** `abilities.iter()`-style use anywhere in the file is at `:2294`, well
  outside the extracted region, so no arm consumes `abilities` other than as a loop iterand and the
  retyping cannot have changed any arm's behaviour.
- **The activated/mana loops correctly stay unfiltered.** `:2466` and `:2483` run over `abilities`
  *before* the filter is built at `:2534`. A graveyard-activated ability (Reassembling Skeleton's
  shape) is carried by `activation_zone`, a different field, and `lowers_onto_the_battlefield`
  (`:2548-2558`) matches only `AbilityDefinition::Triggered { trigger_zone: Some(_) }` — so nothing
  that should survive is filtered. This was the specific risk in item 4 of the brief and it is clean.
- `lowers_onto_the_battlefield`'s inner `match zone { TriggerZone::Graveyard => false }` is exhaustive
  with no wildcard, so a future `TriggerZone` variant is a compile error requiring classification —
  the SR-5 idiom, correctly applied.
- The per-arm guard at the old `WheneverPermanentEntersBattlefield` site is gone **including the
  `trigger_zone,` binding** (`:3081-3093` no longer destructures it), which is what makes G-A a real
  gate rather than a comment.
- `..PendingTrigger::blank(obj_id, owner, PendingTriggerKind::CardDefETB)` at `:7391-7395` — SR-7
  compliant. The shared `carddef_intervening_if_holds_at_queue_time` call at `:7379` sits after the
  `fired_as` match and therefore covers the new arm with no edit, exactly as plan §3.4 predicted;
  verified by reading, not assumed.

### The six Q-sites — every one reads the trigger source's own face

| site | object bound | line | source of the trigger? |
|---|---|---|---|
| Q1 Backup | `obj` = `fizzle_object(*object_id)`, the entering permanent | `:3182` / `:3191` | yes — and one `eff` binding serves BOTH `eff.iter().enumerate()` (`:3193`) and `eff[idx + 1..]` (`:3201`), so index and slice cannot diverge. This was the riskiest item in the brief; it is right. |
| Q2 `WhenYouCastThisSpell` | `stack_obj` = the cast spell | `:3805` / `:3814` | yes |
| Q3 `WhenExertedAsAttacks` | `src_obj` = `fizzle_object(*attacker_id)` | `:4157` / `:4174` | yes |
| Q4 `WhenDealsCombatDamageToPlayer` | `src_obj` = `fizzle_object(assignment.source)` | `:5211` / `:5225` | yes |
| Q6 `WheneverRingTemptsYou` | `obj` = `expect_object(obj_id)`, the watching permanent | `:6197` / `:6208` | yes |
| Q7 graveyard sweep | the graveyard object itself, `is_transformed` read at collection time | `:7208` / `:7227` | yes |

### The fix reaches production, not just the harness

Worth stating because `build_face_ability_vectors` lives in `crates/engine/src/testing/` and a
comment at `abilities.rs:4146` says the CardDef→runtime conversion "only happens in
`enrich_spec_from_def` for tests". `crates/simulator/src/setup.rs:419` and `:433` build **real**
games through `enrich_spec_from_def`, which calls `build_face_ability_vectors` at
`replay_harness.rs:3993`. So the `OOS-DX1-3` "live-wrong on a deck-legal `Complete` card" claim is
correct and the repair is not test-only. `rules/face.rs:104` and `rules/resolution.rs:888` (the
disturb back-face rebuild) are the other two callers and both route through the same filter, so the
back-face path is covered too.

### Gates

- **G-A** (`pb_dx24_trigger_zone_roster.rs:104-114`) extracts the function body by brace balance over
  comment-stripped source and asserts zero `trigger_zone`. Its non-vacuity sibling (`:121-132`)
  requires ≥30 `trigger_condition:` arms, so a collapsed extraction cannot pass it. Capable of
  catching what it claims.
- **G-B** (`:141-157`) counts `build_face_triggered_abilities(` == 2 and requires the literal
  `(&battlefield_triggers)`. Since the function is private, no out-of-file caller is possible.
  Capable.
- Both strip **line and block** comments (`:24-58`), the PB-DX32 M8 lesson — correctly applied here
  (and, per Finding 2, *not* in the sibling probe file, which is the inconsistency).
- **R1/R2 are built by enumerating `all_cards()`**, not by grepping — SR-36 satisfied. R1 carries a
  real `>= 1_700` non-vacuity floor. Their gap is coverage (Finding 8), not method.
- **SR-9a**: both `mod` lines present — `tests/primitives/main.rs:37`, `tests/core/main.rs:32`. No
  top-level `tests/*.rs`.

### Wire neutrality

No type declaration changed: `TriggeredAbilityDef` gains no field, `PendingTrigger` gains no field,
no `Command`/`GameEvent`/`Effect` variant added, `TriggerZone` unchanged. The new function parameter
is a `&HashSet<ObjectId>` local to `check_triggers`. PROTOCOL 35 / HASH 73 unmoved is therefore
correct **by construction**; the execution notes report both gate-executed, which I cannot re-run but
have no reason to doubt.

### Scope discipline (item 10) — verified by marker scan, not by `git diff`

I have no shell, so I could not run `git diff main..HEAD --numstat`. A repo-wide scan for
`DX24|dx24` returns 16 files: the 5 source/test files, 2 test `main.rs`, `nether_traitor.rs`, 4
`memory/primitives/*` docs, `memory/workstream-state.md`, `CLAUDE.md`, `docs/audits/
decision-point-audit.md`, and `docs/audits/mtg-characteristics-recursion-adjudication.md` (a
pre-existing mention). **No file under `crates/simulator/`, `tools/`, or `crates/card-types/`.**
That is strong but not conclusive (a change there need not mention "DX24"). **Recommend the runner
re-run the three `--numstat` scopes at close-out and paste the output**, since the execution notes'
`crates/card-defs/` claim is already demonstrably stale (Finding 12).

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 113.6 / 113.6b / **113.6m** | Yes — `lowers_onto_the_battlefield`, `replay_harness.rs:2548` | Yes — T1 (behavioural), T7 (corpus differential), G-A/G-B (structural) | Derivation independently re-checked against MCP text; correct |
| 113.6k | N/A by derivation | — | Correctly ruled out in the plan: the condition *can* trigger from the battlefield; 113.6m is what confines it |
| 603.6c | Yes — the new `CreatureDied` arm, `abilities.rs:7287` | Yes — T2 | Trigger is verbatim 603.6c's second example |
| **603.10a** | Yes — `arrived_in_graveyard_this_batch`, `abilities.rs:2988` + guard at `:7333` | Yes — T4 (revert executed red per notes) | Correctly withheld from the ETB arm. **Slice granularity undischarged — Finding 3** |
| 603.4 | Yes — shared `carddef_intervening_if_holds_at_queue_time` at `:7379`, outside the `fired_as` match | Indirectly | Verified by reading; no edit needed, as predicted |
| 108.4a | Yes — `owner` stands in for the absent controller | Yes — T2 (`controller == p1`), T6(a) | Correct |
| 400.7 | Yes — `exclude_self` on the graveyard id space, `:7319` | T5 — **does not discriminate**, honestly filed as `OOS-DX24-5` | Correct in source; gate gap acknowledged |
| 111.7 | Yes — `nontoken_blocks`, `:7322` | Yes — `..._filter_nontoken_only` (synthetic, both polarities) | Mirrors the battlefield arm exactly |
| 613.1d | Yes — `pre_death_characteristics` with base fallback, `:7351` | Yes — `..._filter_subtype_filter` (synthetic, both polarities) | Mirrors |
| 702.165a (Q1) | Yes — one `eff` binding, `:3191`/`:3201` | Yes — disturb-DFC probe, 2 counters vs 0 | Riskiest Q-site; correct |
| 701.43d (Q3), 510.3a (Q4), 701.54d (Q6) | Yes | Yes — synthetic `Command::Transform` probes | Q4 correctly bypasses Channel-A masking |
| **712.16** (Q5) | N/A — comment-only re-scope | No | **Comment cites 712.2, which is the wrong rule — Finding 1.** Conclusion still holds, for an engine reason |
| 712.8a / 400.7 (zone reset) | Pre-existing, `state/mod.rs:1404/1558/1678/1900` | **No** | Q2/Q7's whole premise; not gated — Finding 2 |
| **118.12** | Deviated (engine auto-pays) | T3 pins the deviation | **Cited backwards — Finding 9** |
| 603.2d (doubling) | Unchanged, deliberately | No | Deferred correctly; write-up wrong — Finding 4 |

---

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `nether_traitor` | **Yes** — MCP-verified word for word: `{B}{B}`, Creature — Spirit, 1/1, Haste + Shadow + the graveyard trigger; `oracle_text` field matches printed text exactly | 0 | **Yes** — fires from the graveyard, silent on the battlefield, per CR 113.6m | Comment-only edit; `Complete` correctly retained. Owner-vs-controller note at `:47-51` correctly left alone (`OOS-DX4-1` allowlist). Stray blank line at `:71-73`, and SR-35 may not have been run — Finding 12 |
| `bloodghast` | not edited | — | unchanged | `partial`, deck-illegal. Correctly untouched; the ETB arm's behaviour is provably unchanged (guard withheld) |
| `squee_goblin_nabob` | not edited | — | **still broken, correctly stated** | `known_wrong`, deck-illegal. Neither a lowering arm nor a dispatch arm; filed as `OOS-DX24-3`, which is the right disposition and says so plainly |
| `teysa_karlov`, `drivnod_carnage_dominus` | not edited | — | unchanged | **`Completeness::partial`, i.e. deck-illegal** — contradicting `OOS-DX24-1`'s "2 `Complete` defs" tag. Finding 4 |
| `ancient_greenwarden` | not edited | — | pre-existing defect | `Complete` by derive; with `bloodghast` it is the *pre-existing* deck-legal shape of `OOS-DX24-1`'s class. Finding 4 |

---

## Filed-seed audit (`OOS-DX24-1..6`)

| Seed | Verdict |
|---|---|
| `OOS-DX24-1` (603.2d doubler) | **Correct in substance, wrong in two facts** — Finding 4 |
| `OOS-DX24-2` (look-back variant set) | **Correct and well-argued.** The `events.rs:466-467` "or exile if replaced" reason is independently verified and is the strongest of the three. Should additionally carry the *granularity* half — Finding 3 |
| `OOS-DX24-3` (2 of ~34 conditions dispatched) | **Correct.** The "a no-op is auditable; firing from the wrong zone is not" framing is the right trade and is stated, not glossed |
| `OOS-DX24-4` (dual dispatch of `WhenDealsCombatDamageToPlayer`) | **Correct, and correctly labelled exposure-UNMEASURED.** Honest |
| `OOS-DX24-5` (T5 non-discrimination) | **Correct.** Exactly the disposition this project's own PB-DX23 lesson asks for — a claim's limits stated rather than glossed |
| `OOS-DX24-6` (`pub` widening) | **Correct and appropriately LOW.** Accepted |
| *(missing)* | Plan risk #2 (slice granularity) — Finding 3. Plan risk #9 (`MayPayThenEffect` auto-pay) — Finding 9. Both were explicitly required to be filed |

---

## Recommended fix order

1. Finding 1 (wrong CR citation, 2 places — cheapest, and it is already propagating)
2. Finding 4 (seed's completeness fact + class scope — it mis-ranks the queue)
3. Finding 9 (backwards CR citation in a test + file the risk-#9 seed)
4. Finding 2 (replace the §4.0 pin with a runtime probe)
5. Finding 8 (R1 back-face walk + R2 seven-shape assertions)
6. Finding 3 (granularity note + seed)
7. LOWs 5/6/7/10/11/12
