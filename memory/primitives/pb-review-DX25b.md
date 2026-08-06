# Primitive Batch Review: PB-DX25b — the announced-target → stack-entry id space

**Date**: 2026-08-05
**Reviewer**: primitive-impl-reviewer (Opus)
**Branch**: `feat/pb-dx25b-validatetargetrequirement-spell-target-id-space-con` (`scutemob-204`)
**CR Rules**: 115.7 / 115.7a / 115.7b / 115.7d, 115.10 (cited to refute), 601.2a, 601.2c, 608.2b,
707.10 / 707.10a, 400.7, 702.21a — all verified verbatim against the MCP rules server.
**Engine files reviewed**: `crates/engine/src/state/stack_registry.rs`,
`crates/engine/src/rules/casting.rs`, `crates/engine/src/effects/mod.rs`
**Test files reviewed**: `crates/engine/tests/primitives/pb_dx25b_announced_stack_target_space.rs`,
`crates/engine/tests/core/pb_dx25b_announced_target_roster.rs`,
`crates/engine/tests/primitives/pb_ef11_spell_single_target.rs`,
`crates/engine/tests/core/pb_dx25_stack_registry_roster.rs`,
`crates/engine/tests/rules/copy_redirect.rs` (unmodified, but in class)
**Card defs reviewed**: 4 in class, 0 modified — `misdirection`, `bolt_bend`,
`untimely_malfunction`, `deflecting_swat` (plus `plumb_the_forbidden`,
`complete_the_circuit`, `hydroelectric_specimen` checked as R2/R3 walker false-positive
candidates)

## Verdict: needs-fix

**The primitive itself is correct and the census is complete — I re-derived both independently and
found nothing wrong with either.** `stack_index_for_announced_target` encodes exactly the rule its
doc claims, the five consumers are the right five, the three deliberately-untouched sites
(`effects/mod.rs:7720`, `effects/mod.rs:7991`, `resolution.rs:8337`) are each correctly classified,
`copy.rs`'s other three callers genuinely pass internal stack ids, and I found **no sixth site** —
an exhaustive grep for `stack_objects … .id ==` across all 43 engine source files returns exactly
five raw comparisons, every one of which the plan classified correctly. `Effect::CounterUnlessPays`
is not a blindness here: it delegates into `Effect::CounterSpell`'s arm, which was refactored
behaviour-preservingly, so it inherits the helper for free; and nothing delegates into the
`ChangeTargets` or `CopySpellOnStack` arms. I also confirmed rather than accepted that
`GameEvent::TargetsChanged` could never fire in production before this batch (all four
`Effect::ChangeTargets` defs are `AbilityDefinition::Spell` with `DeclaredTarget`, and
`ctx.target_remaps` only ever holds card ids), and that no consumer reads
`TargetsChanged.stack_object_id` at all.

**The findings are one HIGH about what the batch made reachable and did not pin, plus a cluster of
gate-coverage and claim problems.** The HIGH is the batch's own stated "single biggest hazard"
(plan §8 R2): the CR 115.7a object-target redirect picks the smallest `ObjectId` in the recorded
zone with no legality check, was unreachable before this batch, and is now reachable from two
`Complete` deck-legal defs in a human-playable browser game and in the fuzzer — and the
wrong-way-round probe the plan explicitly required was not written, while the execution notes
assert (incorrectly) that the plan deferred it. Three further findings are claim problems on
`Complete` card defs the batch's own census touched and left. Everything the coordinator
independently re-ran (counts, PROTOCOL/HASH, coverage, fmt/clippy, the helper-clause revert) I
accept without re-derivation; I challenge none of it.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | **HIGH** | `crates/engine/src/effects/mod.rs:7619-7654` | **A CR 115.7a wrong-answer path became reachable on two `Complete` defs and was not pinned.** The plan mandated a wrong-way-round probe; none exists, and the execution notes misstate the plan as having deferred it. **Fix:** add the probe, correct the notes, decide the completeness question explicitly. |
| E2 | MEDIUM | `crates/engine/tests/core/pb_dx25b_announced_target_roster.rs:434-513` (R5), `:388-424` (R4) | **The defect can be reintroduced at a new site with every gate green,** and R5 is defeatable three ways by innocent formatting. **Fix:** widen R5 to the defect shape, or state the residual precisely instead of calling it "the closest thing to a wide net". |
| E3 | MEDIUM | `crates/engine/tests/core/pb_dx25b_announced_target_roster.rs:206-323` | **R1/R2/R3 walkers have undocumented `_ => false` blind spots** over four recursive `Effect` variants and four `AbilityDefinition` variants; the plan mandated documenting them. R3's "expected EMPTY" is not guaranteed by its walker. **Fix:** document the blind spot in the file doc (plan §5.3), or switch to the sanitized-Debug walker the plan preferred. |
| E4 | LOW | `crates/engine/tests/core/pb_dx25_stack_registry_roster.rs:238-289` | **Re-aimed G2 no longer forbids a re-open-coded scan** inside the `Effect::CounterSpell` arm. **Fix:** add R4's zero-`stack_objects.iter()` conjunct to G2. |
| E5 | LOW | `crates/engine/src/effects/mod.rs:7570-7573`; `pb_ef11_spell_single_target.rs:537-541`; `pb_dx25b_…:288-290` | **Right decision, false reason.** Three sites justify the stack-entry-id choice with "view-model/replay consumers read it as one"; no consumer reads the field. **Fix:** re-justify from the field's own doc. |
| E6 | LOW | `crates/engine/src/rules/casting.rs:8258` | **Comment labels the `TargetSpellOrAbilityWithSingleTarget` arm "C2"** — that is C1. The execution notes caught this exact confusion in the plan and the shipped comment repeats it. **Fix:** `C2` → `C1`. |
| E7 | LOW | `crates/engine/src/state/stack_registry.rs:155` | **The `!so.is_copy` guard is now shared by five consumers and discriminated by exactly one synthetic probe** (T5). **Fix:** record this in T5's doc, or add a real-cast copy probe. |
| E8 | LOW | `crates/engine/src/rules/casting.rs:8364-8371` | **The replacement CR grounding contains a misapplied clause.** CR 115.7a's "another legal target" governs the victim's new target, not which spell may be targeted. **Fix:** drop that sentence; 601.2c + the requirement's own definition carry the argument. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | MEDIUM | `bolt_bend.rs` | **`Complete`, no note, printed "or ability" half provably unreachable.** This batch is what proved it (`OOS-DX25b-1`). **Fix:** demote to `Completeness::partial` citing OOS-DX25b-1, or add a marked note; re-measure coverage. |
| C2 | MEDIUM | `untimely_malfunction.rs:74-80` | **The shipped `partial` note asserts "Modes 0 and 1 are complete"** — false at HEAD (mode 1 was refused on every cast) and still unverified after the fix. Plan §8 R5 required a modal-index probe; none was written. R1's assertion message repeats the claim. **Fix:** write the probe, then correct or keep the note on evidence. |
| C3 | MEDIUM | `deflecting_swat.rs:32,39` | **Oracle/def mismatch on a `Complete` def the batch's own census (F-A) examined.** Printed "target spell **or ability**"; def declares `TargetRequirement::TargetSpell` (spell-only); the def's own comment claims "can target ANY spell or ability". **Fix:** file a seed and add a marked note; do not silently widen the requirement. |
| C4 | — | `misdirection.rs` | **Verified correct.** Oracle text, mana cost `{3}{U}{U}`, Instant, CR 118.9 pitch cost and `TargetSpellWithSingleTarget` all match. No finding. |

## Test Findings

| # | Severity | File | Description |
|---|----------|------|-------------|
| T1 | LOW | `crates/engine/tests/rules/copy_redirect.rs` | **Eight tests announce stack-entry ids into `execute_effect`** — the exact "green while testing a fiction" class the batch identified and repaired in `pb_ef11`. Not censused, not repaired, not disclosed. Includes `test_bolt_bend_redirects_single_target_spell`, named after a card this batch repaired. **Fix:** repair or annotate; at minimum add the file to the batch's census record. |

---

## Finding Details

### Finding E1: The object-target redirect's CR 115.7a violation is now reachable on two `Complete` defs and is not pinned

**Severity**: HIGH
**File**: `crates/engine/src/effects/mod.rs:7619-7654` (unchanged by this batch); reachability
created by `casting.rs:6485` / `:6542` and `effects/mod.rs:7553`
**CR Rule**: 115.7a — "each target can be changed only to **another legal target**. If a target
can't be changed to another legal target, the original target is unchanged."

**Issue.** The `Target::Object` branch of `Effect::ChangeTargets` builds its candidate set as
*every* `state.objects` entry in the recorded `zone_at_cast` except the current target, sorts by
`ObjectId`, and takes the first. There is no requirement check, no protection/hexproof/shroud
check, no controller check, no card-type check. The source self-documents this as a KNOWN
LIMITATION at `:7624-7629`.

Before this batch that branch was unreachable, because nothing could announce a target at all.
After it, both `misdirection` and `bolt_bend` are `Complete`, deck-legal, offerable through
`rules::queries::legal_targets_per_slot` (which delegates to `casting::validate_targets_inner`, so
it inherits the repair automatically), and therefore reachable by a human in the browser **and by
the bots** — `crates/simulator/src/targeting.rs::plan_targets` routes through the same query, and
the fuzzer has cast real spells since PB-DX22.

Concrete failure scenario, entirely within the repaired path: p2 casts "Destroy target creature"
targeting p1's creature. p1 casts Misdirection announcing that spell's card id — legal, and it now
succeeds. At resolution, `zone_at_cast` is `Battlefield`, so the candidate set is every battlefield
object other than the original target, and the redirect lands on the lowest `ObjectId` — routinely a
basic land. `Effect::DestroyPermanent` performs no type check at resolution (CR 608.2b's
`is_target_legal` compares zone only), so **the land is destroyed by a "destroy target creature"
spell.** The same shape can redirect onto a hexproof permanent, onto a permanent the caster
controls, or — when the victim targeted a spell (`zone_at_cast == Stack`) — onto Misdirection's own
card. This is "card def produces wrong game state" on two `Complete` deck-legal defs.

The batch **argued** this correctly (plan §8 R2, option (iii): ship, and pin the deviation
wrong-way-round). I agree with the scope decision — implementing CR 115.7a legality needs the
victim spell's `TargetRequirement` list on `StackObject`, which is a hashed field and a batch of its
own. But option (iii) had two halves and only one shipped. The plan's own words:

> One additional probe asserts the object-target branch's illegal redirect **as the current
> behaviour**, cites CR 115.7a, names **`OOS-DX25b-3`**, and tells the successor batch to invert it
> — the `blinkmoth_nexus` pattern PB-DX19 established.

No such probe exists. T1–T8 all use **player** targets (deliberately, per the plan) or synthetic
copy/lookup fixtures. The execution notes (§10.3) state:

> No dedicated probe was added in this batch for the illegal-redirect behavior itself (the plan
> scoped this as future work, not this batch's test surface)

**That parenthetical is false as to the plan's text** — §8 R2 scoped the *fix* as future work and
the *probe* as this batch's. So this is two defects in one: a missing deliverable, and a
self-report that misdescribes the plan it is reporting against. Under this project's own precedent
(`blinkmoth_nexus`, PB-DX19; the "assert it wrong-way-round so the successor inverts it"
discipline), nothing in the tree now records the wrong answer, and nothing will go red the day
someone fixes it.

**Fix (three parts, all required):**
1. Add `t9_object_target_redirect_ignores_the_original_requirement` (or equivalent) to
   `pb_dx25b_announced_stack_target_space.rs`: a real `Command::CastSpell` chain in which
   Misdirection redirects a "destroy target creature" spell onto a **land**, asserting the land is
   destroyed. Cite CR 115.7a verbatim, name `OOS-DX25b-3`, and state in the test doc that the
   successor batch must **invert** this assertion.
2. Correct `memory/primitives/pb-DX25b-execution-notes.md` §10.3 — the plan required the probe;
   it was not written.
3. Decide the completeness question **explicitly** rather than by omission. The plan's "0
   completeness flips — this batch makes the marker *true* rather than changing it" is sound for
   the player-target branch only. Either demote `misdirection`/`bolt_bend` to `partial` with the
   CR 115.7a note (the PB-DX4 "the corpus got truer" precedent; coverage moves 1,133 → 1,131), or
   record in the close-out, in writing, why `Complete` survives a live CR 115.7a violation. Do not
   let the close-out say "the cards now work" without the qualifier the plan itself demanded.

---

### Finding E2: The defect can be reintroduced at a new site with every gate green; R5 is defeatable three ways

**Severity**: MEDIUM
**File**: `crates/engine/tests/core/pb_dx25b_announced_target_roster.rs:434-513` (R5), `:388-424`
(R4)
**Architecture invariant**: SR-36 / the batch's own thesis ("the rule is encoded once", plan §3.5)

**Issue.** I tried to defeat each gate, as instructed. Both fall.

**(a) R5 does not police the defect — only the cure.** R5 fires when someone writes a *second copy
of the correct rule* (`card_in_stack_zone(` joined by `||` to an `so.id ==`/`s.id ==` comparison in
one statement). The defect `OOS-DX25-3` actually is is a **bare** `stack_objects.iter().find(|so|
so.id == announced)` on an announced id, with no `card_in_stack_zone` anywhere. R5 is structurally
blind to it. So: add a new `Effect` arm in `effects/mod.rs`, or a new `TargetRequirement` arm in
`casting.rs`, that takes a declared target and looks it up with `so.id ==` — R5 passes (no
`card_in_stack_zone` in the expression), R4 passes (it extracts only the two arms it names by
literal), and **casting.rs has no source gate at all**. The defect is reintroducible verbatim with
a fully green suite.

**(b) R5's `same_statement` heuristic is defeated by a preceding statement.** The check is
`!before.contains(';')` over a 150-byte window *before* the `card_in_stack_zone(` match. A faithful
re-open-coding written as

```rust
let announced = id;
let pos = state.stack_objects.iter().position(|so| {
    so.id == announced || (!so.is_copy && card_in_stack_zone(&so.kind) == Some(announced))
});
```

puts the previous statement's `;` inside that window, so `same_statement` is false and the genuine
duplicate is **not** flagged. This is not an adversarial construction — it is what any author
naming a local would write.

**(c) R5 is clause-order sensitive.** The window is scanned *backwards* from
`card_in_stack_zone(`. Writing the two disjuncts the other way round —
`card_in_stack_zone(&so.kind) == Some(announced) || so.id == announced` — puts the id comparison
*after* the match index, so `has_id_eq` is false. Missed.

**(d) R4's zero-`stack_objects.iter()` conjunct is evadable** by `for so in &state.stack_objects`
or an index loop, though (a)'s helper-call conjunct still holds for those two arms.

The execution notes' §9.6 record of R5's tightening is honest and the false positive on
`resolution.rs::counter_stack_object` was real. But the tightening narrowed the gate onto a shape
so specific that it now catches essentially only a verbatim copy-paste of the helper body. R4's doc
comment states its own residual honestly and then says "R5 below is the closest thing to a wide
net" — which, given (a), materially overstates R5.

**Fix (pick one, do not do nothing):**
- Preferred: replace R5's shape test with a **provenance-scoped** gate — scan the two
  `casting.rs` requirement arms (`TargetSpellOrAbilityWithSingleTarget`,
  `TargetSpellWithSingleTarget`) the same way R4 scans the two `effects/mod.rs` arms: each must
  call `stack_index_for_announced_target` at least once and contain zero
  `stack_objects.iter()`/`.iter_mut()`. That is the only place with a behavioural regression test
  today but no structural one, and it costs ~20 lines of the idiom already in the file.
- Additionally: make R5 order-insensitive (scan a symmetric ±150 window and drop the `;`
  heuristic in favour of "no `let ` / no `}` between the two literals"), or delete the `has_or` /
  `same_statement` conjuncts and accept the `counter_stack_object` hit as an explicit
  path-allowlist entry.
- At minimum: correct R4's doc so it does not describe R5 as a wide net, and say plainly that
  **no gate in this tree detects a newly-introduced bare `so.id ==` lookup on an announced id**
  (this is plan §8 R8's residual, and it should be restated at the gate rather than only in the
  plan).

---

### Finding E3: R1/R2/R3's hand-written walkers have undocumented blind spots; the plan required them documented

**Severity**: MEDIUM
**File**: `crates/engine/tests/core/pb_dx25b_announced_target_roster.rs:100-137` (R1),
`:206-231` (R2), `:277-323` (R3)
**Plan**: §5.3 — "**Walker construction — mandated, with the trap named.** … Option (b) is
preferred: it is total over the effect tree by construction and immune to a new recursive `Effect`
variant, which a hand-written walker with a `_ => {}` arm is not. **Whichever is chosen, the choice
and its blind spot go in the file's doc comment.**"

**Issue.** The runner chose option (a), the hand-written structural walker, which the plan
explicitly de-preferred — a defensible call — but **did not write the blind spot into the file's
doc comment**, which the plan made unconditional. The blind spots I measured against
`crates/card-types/src/cards/card_definition.rs`:

*Recursive `Effect` carriers the R2/R3 walkers do not descend into* (they handle `Sequence`,
`Conditional`, `ForEach`, `Choose`, then `_ => false`):

| Variant | Field | Line |
|---|---|---|
| `Effect::Repeat` | `effect: Box<Effect>` | 1739 |
| `Effect::MayPayOrElse` | `or_else: Box<Effect>` | 1781 |
| `Effect::MayPayThenEffect` | `then: Box<Effect>` | 1792 |
| `Effect::CoinFlip` | `on_win`/`on_lose: Box<Effect>` | 1996 |

*`AbilityDefinition` variants none of the three walkers examine* (all three match only
`Spell`/`Activated`/`Triggered`, then `_ => false`):

| Variant | Carries | Line |
|---|---|---|
| `LoyaltyAbility` | `effect`, **`targets: Vec<TargetRequirement>`** | 493 |
| `SagaChapter` | `effect`, **`targets: Vec<TargetRequirement>`** | 505 |
| `ClassLevel` | `abilities: Vec<AbilityDefinition>` (nested) | 518 |
| `Forecast` | `effect: Effect` | 757 |

All three rosters are **correct today** — I verified `hydroelectric_specimen`,
`plumb_the_forbidden` and `complete_the_circuit` are prose/comment mentions only, so the walkers
correctly exclude them and R3's refutation of the dispatch brief stands. The problem is that these
are ratchets whose whole job is to detect *future* movement, and R3's assertion message makes an
affirmative claim the walker cannot back:

> "PB-DX25b R3 (Effect::CopySpellOnStack roster) moved from the expected EMPTY … C4's fix is no
> longer purely synthetic if this is non-empty — re-derive its completeness impact."

Concrete failure scenario: a future author writes Plumb the Forbidden properly as
`Effect::MayPayThenEffect { then: Box::new(Effect::CopySpellOnStack { … }) }` — the natural DSL
shape for "you may pay {X}; if you do, copy that spell". R3 stays green asserting EMPTY, the
"re-derive its completeness impact" instruction never fires, and C4's path is now live from a
corpus def that the tree believes does not exist. Same shape for R1: a planeswalker authored as
`LoyaltyAbility { targets: vec![TargetSpellWithSingleTarget], effect: ChangeTargets{..} }` widens
the class this batch repaired while R1 and R2 both stay green with unchanged rosters. Note R3's
`DrawCards` liveness control does **not** cover this: the control shares the walker's blind spots
exactly, so a `DrawCards` nested under `MayPayThenEffect` is equally invisible and the control still
passes off the many top-level `DrawCards` defs.

**Fix:** either (a) switch R2/R3 to the plan's preferred sanitized-Debug scan (clear `oracle_text`
on both faces, set `completeness: Complete`, then `format!("{:?}", def).contains("CopySpellOnStack")`)
— total by construction and immune to new variants; or (b) keep the walkers and add the missing
arms (`Repeat`, `MayPayOrElse`, `MayPayThenEffect`, `CoinFlip`; `LoyaltyAbility`, `SagaChapter`,
`ClassLevel`, `Forecast`). In **either** case, add the plan-mandated blind-spot paragraph to the
file's module doc naming the variants that are and are not walked, so the next reader can re-derive
the coverage instead of trusting the roster.

---

### Finding E4: Re-aimed G2 no longer forbids a re-open-coded scan in the `Effect::CounterSpell` arm

**Severity**: LOW
**File**: `crates/engine/tests/core/pb_dx25_stack_registry_roster.rs:238-289`

**Issue.** The re-aim itself is exactly right and I endorse it: `card_in_stack_zone >= 1` (the CR
701.6a zone-move) **and** `stack_index_for_announced_target >= 1` (the lookup), both conjuncts
individually revert-proven (G2-1/G2-2), forbidden literals byte-unchanged. Refusing to weaken it to
`>= 1 card_in_stack_zone` alone was the right call.

But the CounterSpell arm — unlike the two arms R4 covers — has no zero-`stack_objects.iter()`
conjunct. A future edit can add `state.stack_objects.iter().find(|so| so.id == id)` *alongside* the
helper call (e.g. a "fast path" or a second target lookup) and G2 stays green on both conjuncts.
That is the identical defect class this batch exists to prevent, in the one arm that has been
through two batches about it.

**Fix:** add R4's conjunct to `g2_counter_spell_arm_does_not_reclassify_by_kind`:
`body.matches("stack_objects.iter()").count() + body.matches("stack_objects.iter_mut()").count() == 0`.
Prove it discriminates by planting the scan, as V11 did for R4.

---

### Finding E5: Three sites justify the `TargetsChanged.stack_object_id` decision with a consumer that does not exist

**Severity**: LOW — this is a **claim** problem, not a behaviour problem.
**Files**: `crates/engine/src/effects/mod.rs:7570-7573`;
`crates/engine/tests/primitives/pb_ef11_spell_single_target.rs:537-541`;
`crates/engine/tests/primitives/pb_dx25b_announced_stack_target_space.rs:288-290`

**Issue.** All three say some form of "`GameEvent::TargetsChanged.stack_object_id` is documented as
a stack-object id **and the view-model/replay consumers read it as one**". The second half is
false. `crates/view-model/src/event_view.rs:927-948` destructures the event as
`GameEvent::TargetsChanged { old_targets, new_targets, .. }` — the field is discarded. Its only
other appearance is the or-pattern at `:1066` (tier classification, field not bound). Nothing in
`tools/play-server`, `tools/tui`, `tools/replay-viewer` or the frontend reads it. So the batch's
own §4 claim, and the task's item 4, resolve as: **no consumer read it as a card id, because no
consumer reads it at all.**

The decision is still correct — the field's own doc at `events.rs:1421` says "The stack object
whose targets changed", and naming the card id there would have been a documented-contract
violation on an event that will eventually be consumed. The reason given is just fabricated, and it
is the kind of reason a future reader will rely on when deciding whether the field is safe to
change again.

**Fix:** replace the justification at all three sites with the true one — the field's own doc
comment at `rules/events.rs:1421-1422` — and add "no consumer reads this field today
(`event_view.rs:927` discards it), so this is a contract correction, not a compatibility fix."

---

### Finding E6: `casting.rs:8258` labels the `TargetSpellOrAbilityWithSingleTarget` arm "C2"

**Severity**: LOW — claim problem.
**File**: `crates/engine/src/rules/casting.rs:8258`

**Issue.** The doc comment on `test_target_spell_single_target_self_targeting_prevented` reads
"…made the lookup this test exercises (**C2**, `casting.rs`'s
**`TargetSpellOrAbilityWithSingleTarget`** arm) pass regardless…". In the plan's census C1 is
`casting.rs:6476` = `TargetSpellOrAbilityWithSingleTarget` and C2 is `casting.rs:6502` =
`TargetSpellWithSingleTarget`. The label and the variant name contradict each other.

This is the *same* confusion the execution notes §9.2 caught in the plan and corrected by
execution — the runner proved V5 does **not** redden this test and re-attributed the discriminator
to the sibling. The correction landed in the notes but the shipped source comment reproduces the
error, so a reader following the comment back to the revert matrix lands on the wrong row.

**Fix:** `C2` → `C1` at `casting.rs:8258`. While there, confirm the sibling comment at `:8410`
("this discriminates C2's lookup") — that one is correct, since that test *is* the
`TargetSpellWithSingleTarget` test.

---

### Finding E7: The shared `!so.is_copy` guard has exactly one discriminating probe, and it is synthetic

**Severity**: LOW
**File**: `crates/engine/src/state/stack_registry.rs:155`

**Issue.** The guard was written for one consumer (`Effect::CounterSpell`, PB-DX25, where it also
closes the CR 702.99c cipher-copy exile hole) and now serves five. I checked each consumer against
CR 707.10 and found no CR problem: a copy of a spell IS a spell, but a copy has no `state.objects`
row, so the offer layer (`queries::legal_targets_per_slot`, `state.objects()`-only) and the
validator's opening `state.objects.get(&id)?` already make a copy unannounceable **independently of
this guard**. The guard therefore imposes nothing new on `ChangeTargets` or `CopySpellOnStack`;
`OOS-DX25b-2` is genuinely a pre-existing consequence of the offer layer, exactly as claimed.
Reaching a copy by its *own* stack-entry id still works through the direct-id clause, so no path
that worked before has narrowed. I re-verified `deflecting_swat` is unchanged: `must_change: false`
hits `continue` at `:7560-7565` after the lookup, exactly as it hit `continue` on the failed lookup
before — no event, no mutation, either way.

The residual is test coverage, not correctness. V2 (delete the guard) reddens only
`t5_copy_is_not_announceable`, and the execution notes correctly and honestly record that PB-DX25's
own copy probes **do not** discriminate it (their scenarios keep the original present, so
`position()`'s first-match-wins lands on the original regardless; and
`test_dx25_countering_a_copy_moves_no_card` exercises the *other* `is_copy` guard, the one at
`effects/mod.rs:7791`). T5 is a hand-built fixture whose second half removes a `StackObject` while
deliberately leaving the card in `state.objects` — a configuration it labels synthetic itself. So
one guard shared by five consumers rests on one synthetic assertion.

**Fix:** state this in T5's own doc (currently it explains the fixture's synthetic-ness but not
that it is the sole discriminator). Optionally add a real-cast variant: cast a spell, copy it via
`Effect::CopySpellOnStack`, counter the original with a real Counterspell, then assert the announced
card id resolves to nothing.

---

### Finding E8: The replacement CR grounding contains a misapplied clause

**Severity**: LOW — claim problem.
**File**: `crates/engine/src/rules/casting.rs:8364-8371` (and the mirror in
`pb_ef11_spell_single_target.rs`'s module doc)
**CR Rule**: 115.7a — verified verbatim

**Issue.** The CR 115.10 mis-citation finding is **correct and well made** — I verified CR 115.10
verbatim and it is the affects-vs-targets rule ("Just because an object or player is being affected
by a spell or ability doesn't make that object or player a target"), with nothing to do with
self-targeting. Correcting it is right, and matches `OOS-DX25-6`'s class.

The replacement contains one bad sentence: "CR 115.7a's 'another legal target' excludes a target
that was never the spell's own target to begin with." CR 115.7a's "another legal target" is about
the *new* target chosen for the **victim's** target slot; it says nothing about which spell may be
chosen as Misdirection's own target. The load-bearing argument is the one the comment already makes
two sentences earlier and does not need this: under CR 601.2c the announced object must be
"appropriate" for the requirement, and a spell that has chosen no targets yet does not have "a
single target".

**Fix:** delete the "CR 115.7a's 'another legal target' excludes…" sentence at
`casting.rs:8367-8368`. Keep 601.2a + 601.2c. Verify the mirrored prose in
`pb_ef11_spell_single_target.rs`'s module doc does not carry the same sentence.

---

### Finding C1: `bolt_bend` stays `Complete` with no note while its printed "or ability" half is provably unreachable

**Severity**: MEDIUM
**File**: `crates/card-defs/src/defs/bolt_bend.rs` (unmodified)
**Oracle**: "Change the target of target spell **or ability** with a single target."

**Issue.** The def is `Complete` by derive (`..Default::default()`, no `completeness` field, no
note), which R1's `expected_complete` assertion pins. This batch **proved** the ability half cannot
work: an activated or triggered ability's stack entry is minted at `abilities.rs:1381` and never
added to `state.objects`, so `queries::legal_targets_per_slot` cannot enumerate it and
`validate_object_satisfies_requirement`'s opening `state.objects.get(&id).ok_or(ObjectNotFound)?`
rejects it. T3 pins all three legs of this wrong-way-round and I verified the mechanism directly at
`abilities.rs:1381`/`:1396`/`:1415` — there is no `add_object`/`objects.insert` on that path.

Plan §8 R1 says: "**no comment, test name, doc line or card-def note in this batch may say Bolt
Bend's 'or ability' half works.**" Nothing in the diff violates that — I checked, and
`casting.rs:6534-6536` even states the two requirement variants are behaviourally identical on the
production path today. But the untouched card def makes exactly that claim by the strongest means
the project has: a `Complete` marker on a printed line half of which the engine refuses. Under
Architecture Invariant 9 and the CARDS-2 / PB-DX4 honest-demotion precedent, that is a completeness
overclaim, and this batch is the first one in a position to know it.

Concrete failure scenario: a deck containing Bolt Bend passes `validate_deck` and enters a game.
A player holds it up against an opponent's activated ability — the printed use case — and the
browser offers zero targets. The engine's own `authoring-status` report counts the card as clean.

**Fix:** convert `bolt_bend.rs` to
`completeness: Completeness::partial("CR 115.7a: the 'or ability' half is unreachable — an
activated/triggered ability's stack entry is never added to state.objects (abilities.rs:1381), so
neither queries::legal_targets_per_slot nor validate_object_satisfies_requirement can see it.
Closing it needs a Target::StackObject id space (wire change). See OOS-DX25b-1; pinned
wrong-way-round by pb_dx25b_announced_stack_target_space.rs::t3_ability_half_is_still_unreachable.
The spell half works as of PB-DX25b.")`, update R1's `expected_complete` set to `{Misdirection}`
(or `{}` if E1's demotion also lands), and re-measure coverage by regeneration. If the coordinator
prefers to keep the marker, the reason must be written down in the close-out, not left implicit.

---

### Finding C2: `untimely_malfunction`'s note asserts "Modes 0 and 1 are complete" — false at HEAD, unverified now

**Severity**: MEDIUM
**File**: `crates/card-defs/src/defs/untimely_malfunction.rs:74-80` (unmodified)
**Plan**: §8 R5 — "Verify by probe whether announcing only mode 1's target lands at
`ctx.targets[1]` or `ctx.targets[0]`; if it lands at 0, mode 1 is still broken after this batch. …
**do not claim it works without a probe.**"

**Issue.** The def's shipped `Completeness::partial` note ends: "Modes 0 and 1 are complete."
That sentence was **false at HEAD** — mode 1 declares `TargetSpellOrAbilityWithSingleTarget`, which
this batch's whole premise is that no cast could satisfy — and it is **still unverified** after the
fix, because the plan-mandated modal-index probe was not written. Mode 1's effect is
`ChangeTargets { target: DeclaredTarget { index: 1 } }` against a *pooled* three-requirement
`targets` list with `mode_targets: None`, and the modal slicing at `abilities.rs:433-458` is a
known-rough area (`OOS-SIM5-5`). If a single announced target lands at `ctx.targets[0]` rather than
`[1]`, mode 1 is still broken.

The batch then **repeats** the unverified claim in a shipped assertion message: R1's
`r1_single_target_spell_requirement_roster_is_pinned` says "Untimely Malfunction is `partial` for an
**unrelated** reason (mode 2's variable target count)". "Unrelated" is precisely the thing that
needed the probe.

Concrete failure scenario: a future reader (or a coverage audit) trusts "Modes 0 and 1 are
complete", writes a card-authoring note or a completeness promotion off it, and mode 1 has never
functioned in either direction.

**Fix:** write the probe — cast `untimely_malfunction` with `modes_chosen: vec![1]` and one
announced target, assert a `GameEvent::TargetsChanged` fires and the victim's targets actually
change. Then either correct the def's note (a card-def line, so re-measure coverage) or keep it on
evidence. Soften R1's message to state what was measured rather than "unrelated".

---

### Finding C3: `deflecting_swat` is `Complete` with a spell-only requirement against printed "spell or ability", and its own comment says the opposite

**Severity**: MEDIUM
**File**: `crates/card-defs/src/defs/deflecting_swat.rs:32,39` (unmodified)
**Oracle**: "You may choose new targets for target spell **or ability**."

**Issue.** The def declares `targets: vec![TargetRequirement::TargetSpell]` — spell-only — against
a printed "target spell or ability", and its adjacent comment at `:32` asserts "Deflecting Swat can
target ANY spell or ability (not just single-target ones)", which the code one line below refutes.
The card is `Complete` by derive. Separately, `must_change: false` (CR 115.7d) means the
`ChangeTargets` effect is a deterministic total no-op — the batch recorded this half as F-A and its
R2 roster message says so, and I confirmed the behaviour is genuinely unchanged by this batch
(both the pre- and post-fix paths reach a `continue` before any mutation).

The requirement/oracle mismatch is the half the batch's census did **not** record. It is
pre-existing, but F-A is exactly the census entry that examined this def, and it stopped at
`must_change`.

**Fix:** file a seed (candidate `OOS-DX25b-5`) for the `TargetSpell`-vs-"spell or ability"
mismatch, noting it is blocked by the same missing id space as `OOS-DX25b-1` (an ability is not
announceable at all, so widening the requirement today would change nothing). Correct the
contradictory comment at `:32`. Consider demoting to `partial` alongside the `must_change: false`
no-op — the card as shipped does nothing at all, on any target.

---

### Finding T1: `copy_redirect.rs`'s eight tests announce stack-entry ids — the same fiction the batch repaired elsewhere

**Severity**: LOW — this is a **census/claim** problem, not a coverage hole.
**File**: `crates/engine/tests/rules/copy_redirect.rs` (unmodified)

**Issue.** The batch's headline test-hygiene finding (plan §1 fact 11, §5.2) is that existing tests
were "green while testing a fiction" because they fed a **stack-entry** id into `execute_effect`'s
`ctx.targets` — a path no real cast can produce. It censused three locations (`casting.rs`'s two
in-src tests, `pb_ef11`) and repaired all three, well.

`crates/engine/tests/rules/copy_redirect.rs` is a fourth location and was not censused. Its helper
`push_spell_targeting_player` (`:118-136`) returns the **StackObject's** id, and eight tests
announce it directly:
`test_copy_spell_on_stack_basic`, `test_copy_spell_on_stack_twice`,
`test_change_targets_must_change_redirects_to_new_player`,
`test_change_targets_no_alternative_leaves_unchanged`,
`test_change_targets_may_choose_new_leaves_unchanged`,
`test_change_targets_accepts_single_target_spell`, `test_change_targets_object_redirect`,
`test_bolt_bend_redirects_single_target_spell`.

They are not vacuous in the old sense — `make_stack_spell` does mint separate `id` and `source`
values — but they exercise only the direct-id clause, so every one of them was green throughout the
entire period Bolt Bend and Misdirection could not resolve a legal target. `test_bolt_bend_
redirects_single_target_spell` is the sharpest instance: it is named after the card this batch
repaired, is the *only* place in the tree describing itself as a "Bolt Bend integration test", and
proves nothing about Bolt Bend.

No coverage is missing (V7 confirmed T1 and the repaired `pb_ef11` test both catch a C3 regression),
so this is about what the tree *claims*, not what it *catches*.

**Fix:** at minimum, add a module-doc paragraph to `copy_redirect.rs` stating that its fixtures
announce stack-entry ids directly and therefore exercise only the direct-id clause of
`stack_index_for_announced_target` — pointing at
`pb_dx25b_announced_stack_target_space.rs::t1`/`t2` for the real-cast coverage. Better: rename
`test_bolt_bend_redirects_single_target_spell` to drop the card name, since T2 now owns that claim.
Also record `copy_redirect.rs` in the execution notes' §2 census so the next batch does not
re-discover it.

---

## Census Verification (independent re-derivation — no sixth site)

The task's highest-value ask. I did **not** trust the plan's method; I ran the inverse check
(every `stack_objects` id-comparison in the engine, classified by the id's provenance) across all
18 engine source files that mention `stack_objects` (105 occurrences).

**Exactly five raw `so.id ==` / `s.id ==` comparisons survive** outside `stack_registry.rs`:

| Site | Id provenance | Verdict |
|---|---|---|
| `rules/copy.rs:150` (`copy_spell_on_stack`) | callee; all four callers now pass genuine stack ids — casualty (`TriggerData::CasualtyCopy.original_stack_id`, `resolution.rs:2562`), `create_storm_copies` (`copy.rs:289`), replicate/storm via the same, and C4's new `real_stack_id` | correctly left alone |
| `rules/resolution.rs:8337` (`counter_stack_object`) | stack-entry id by contract; **zero production callers** (verified by grep — only `tests/core/resolution.rs` and `pb_dx25_counterspell_stack_shapes.rs`) | correctly declined |
| `rules/abilities.rs:6747` | `targeting_stack_id`, minted at `abilities.rs:1381`/`casting.rs:4425` | internal id, correctly left |
| `effects/mod.rs:7720` (`resolve_effect_target_list_indexed`) | deliberately accepts **both** spaces; the only reason Ward's trigger target resolves | correctly left |
| `effects/mod.rs:7991` (`PlayerTarget::ControllerOf`) | objects-first then stack fallback; an announced card id hits the `state.objects` branch, a Ward stack-entry id hits the fallback — correct for both | correctly left |

I additionally checked the two classes the plan's caller-walk alone would miss:

- **The offer layer** (`rules/queries.rs:214-259`) does **not** re-derive the requirement — it
  delegates to `casting::validate_targets_inner` one candidate at a time, so it inherits the repair
  for free. No second arithmetic, no PB-DX20/DX23-style drift site. Good.
- **The delegation class** the task flagged (`Effect::CounterUnlessPays` → `Effect::CounterSpell`):
  present and covered for free by the refactor, exactly as plan §2.3 says. I searched for the
  analogous shape into the two new arms — `Effect::ChangeTargets` and `Effect::CopySpellOnStack`
  appear in `crates/engine/src` **only** at their own arms and in `state/hash.rs`. **Nothing
  delegates into them**, so R4's two-arm scope is not blind in the PB-DX25 way. The R1/R2/R3
  blindness is a different mechanism (Finding E3).
- `rules/abilities.rs:7625/7628` and `casting.rs:6763/6765` return `false` for these two
  requirements in their respective secondary matchers; both are documented dead/early-return arms
  and neither performs a stack lookup.

**Conclusion: the census is complete. There is no sixth site.**

I also re-checked the two index-safety questions the refactor raises and found both handled: C4
captures `real_stack_id` (an id, not an index) before its `for _ in 0..n` loop, so the repeated
`push_back` cannot invalidate it; and C3's read (`state.stack_objects[pos]`) and write
(`state.stack_objects.get_mut(pos)`) share one `pos` with no intervening mutation, which is
strictly better than the two independent `find`s it replaced.

---

## CR Coverage Check

| CR Rule | Verified verbatim | Implemented? | Tested? | Notes |
|---|---|---|---|---|
| 115.7a | Yes | Partially | Player branch: yes (T1/T2). Object branch: **no** | Object-target redirect ignores "another **legal** target" — **Finding E1** |
| 115.7b | Yes | Yes | T1/T2 | Single-target case coincides with 115.7a |
| 115.7d | Yes | Deterministic no-op | `copy_redirect.rs` (fiction path) | `deflecting_swat`; documented, unchanged |
| 115.10 | Yes | n/a | n/a | Correctly identified as a mis-citation and removed (2 sites) |
| 601.2a | Yes | Yes (with a recorded pre-existing ordering deviation) | T1/T2 implicitly | Engine validates before moving the card to the stack |
| 601.2c | Yes | Yes | T1/T2/T3/T6 | The announced-id-is-a-card-id invariant, now encoded once |
| 608.2b | Yes | Yes | T4 | Fizzle on the newly reachable path; T4's `stack_object_id` correction is right |
| 707.10 | Yes | Yes (deviation recorded) | T5 | Copy is a spell but unannounceable — `OOS-DX25b-2`, genuinely pre-existing |
| 707.10a | Yes | Yes | (PB-DX25) | Unchanged |
| 400.7 | Yes | Yes | T4 | New ObjectId on zone change; drives the fizzle |
| 702.21a | (Ward) | Yes | T7 | The direct-id clause's only production consumer, correctly regression-guarded |

## Card Def Summary

| Card | Oracle Match | Lines changed | Game State Correct | Notes |
|---|---|---|---|---|
| `misdirection` | Yes | 0 | Player targets: yes. Object targets: **no** | `Complete`; E1 applies |
| `bolt_bend` | Spell half yes; **ability half unreachable** | 0 | Spell + player targets: yes. Object targets: **no**. Abilities: refused | `Complete`, no note — **C1**; E1 also applies |
| `untimely_malfunction` | Yes | 0 | **Unverified** (mode 1 index) | `partial`; its note claims "Modes 0 and 1 are complete" — **C2** |
| `deflecting_swat` | **No** (`TargetSpell` vs "spell or ability") | 0 | Total no-op (`must_change: false`) | `Complete`; **C3** |
| `plumb_the_forbidden` | — | 0 | — | Prose-only mention; R3 walker correctly excludes it |
| `complete_the_circuit` | — | 0 | — | Comment-only mention; correctly excluded |
| `hydroelectric_specimen` | — | 0 | — | `partial` prose mention; correctly excluded from R2 |

## What I verified and did not find fault with

Recorded so the fix phase does not re-litigate these:

- `stack_index_for_announced_target`'s body is exactly the rule its doc describes; the index return
  type is the right choice for its three call shapes; keeping it a `&Vector<StackObject>` parameter
  rather than `&GameState` preserves `stack_registry` as a dependency-free classification module.
- `card_in_stack_zone` is untouched, still exhaustive over 27 kinds with no wildcard.
- The `is_spell` two-variant `matches!` at `casting.rs:6548-6553` is correctly **kept** and
  correctly **not** re-expressed through `card_in_stack_zone` (CR 707.10 — a copy IS a spell), with
  the reason written at the call site and the counterpart note in the module header.
- Sub-case (iii) of `test_target_spell_with_single_target_self_and_kind_check` says in its own doc
  that it does **not** discriminate the `is_spell` guard, and the code agrees — the lookup returns
  `None` before the guard is reached. The task's item 6 check passes.
- Sub-case (ii)'s deliberately-collapsed fixture is the only remaining configuration that reaches
  the guard with a found object, and is labelled as such rather than "cleaned up".
- `pb_ef11`'s Test 3 doc was rewritten to **retract** its old discrimination claim rather than
  leave it standing — the right handling under `memory/conventions.md`.
- `GameEvent::TargetsChanged` provably never fired in production before this batch (all four
  `ChangeTargets` defs announce; `ctx.target_remaps` only ever carries card ids), so the field's
  meaning change is unobservable.
- SR-9a satisfied: both `mod` lines present (`tests/primitives/main.rs:39`,
  `tests/core/main.rs:34`).
- R4's `extract_match_arm_body` markers resolve to the correct arms after comment-stripping (the
  only earlier occurrences of both literals in `effects/mod.rs` are inside comments).
- R3's `DrawCards` liveness control is a genuine improvement over PB-DX25's T6 self-comparison —
  it is a real control, subject to E3's shared blind spot but not to T6's tautology.
- The execution notes' six self-reported plan corrections (§9.1-§9.6) are each accurate and each
  was found by execution rather than prediction. §9.2 (the V5 test-name slip) and §9.6 (R5's
  `counter_stack_object` false positive) are exactly the kind of finding the discipline exists to
  produce.

## Previous Findings

None — this is the first review of PB-DX25b.
