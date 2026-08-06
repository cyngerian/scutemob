# Primitive Batch Review: PB-DX25 — `Effect::CounterSpell`'s three stack-object shapes

**Date**: 2026-08-05
**Reviewer**: primitive-impl-reviewer (Opus)
**Seed**: `OOS-SIM3-5`
**CR Rules**: 701.6a/701.6b (Counter), 702.140a-c (Mutate), 707.10 / 707.10a / 707.10b (Copies),
729.2/729.2b (Merging), 400.7 (object identity), 601.2c, 608.2b, 702.21a (Ward), 702.34a
(Flashback), 702.133a (Jump-start), 118.9a (one alternative cost), 101.2/101.6, 702.99c (Cipher)

**Engine files reviewed**
- `crates/engine/src/state/stack_registry.rs` (new)
- `crates/engine/src/state/mod.rs` (`pub mod stack_registry;`)
- `crates/engine/src/effects/mod.rs` (the `Effect::CounterSpell` arm, `:2724-2853`; `EffectContext`
  doc at `:171-180`)
- `crates/engine/src/rules/resolution.rs` (`counter_stack_object`, `:8298-8456`)
- `crates/engine/src/rules/events.rs` (`SpellCountered` doc, `:159-167`)
- `crates/simulator/src/invariants.rs` (doc-only: `stack_card_of`, `check_stack_consistency`, `t8`)

**Tests reviewed**
- `crates/engine/tests/primitives/pb_dx25_counterspell_stack_shapes.rs` (T1–T7)
- `crates/engine/tests/core/pb_dx25_stack_registry_roster.rs` (G1 ×3, G2 ×2, G3)
- `crates/simulator/tests/pb_dx25_counter_on_mutate_is_consistent.rs` (2 tests)

**Card defs reviewed**: 0 modified (correct — `git`-scope claim of an empty `crates/card-defs/`
diff is consistent with the `PB-DX25` symbol census, which returns no file under `crates/card-defs/`
or `crates/card-types/`). Defs read for verification: `gemrazer`, `counterspell`, `access_denied`,
`mana_leak`, `mana_tithe`, `make_disappear`, `izzet_charm`.

---

## Verdict: needs-fix

**The primitive itself is correct, and I re-derived it rather than trusting the plan.** The
classification `card_in_stack_zone` is CR-correct and complete: there are exactly four sites in the
whole engine that move an object into `ZoneId::Stack` (`casting.rs:4423`, `copy.rs:383`,
`copy.rs:604`, `resolution.rs:6137`), and every one of them constructs a `StackObjectKind::Spell`
except `casting.rs:4528-4545`, which picks `MutatingCreatureSpell` *after* the same single
`move_object_to_zone(card, ZoneId::Stack)` — so `Spell` + `MutatingCreatureSpell` is exactly the
card-owning set, and the other 25 arms' `None` is right. The `position()` rewrite is
character-equivalent to the old second clause for a non-copy `Spell`; `is_copy: true` is set at
exactly one construction site (`copy.rs:165`) plus one override (`resolution.rs:5429`), so the guard
is sound; `move_object_to_zone` really does `self.objects.remove(&object_id)` (`state/mod.rs:1303`),
so the plan's "dead-id filter" unreachability argument for shape (b) holds; and the `else` branch
provably cannot lose a card. All CR citations I checked via MCP (701.6a, 707.10/a/b, 702.140a,
118.9a) are quoted accurately, and the CR 701.5→701.6 corrections are right.

**But there are nine findings, five of them MEDIUM, and the three that matter are all the same
shape the batch was dispatched to eliminate — a claim whose subject is narrower than the claim.**
(1) The batch's own §0.2 F3 census of "four sites that classify a stack object's card/spell-ness"
is **incomplete by two**, and one of the two it missed (`abilities.rs:6736`) is *wrong in the same
direction as the defect being fixed* while its sibling one function over (`casting.rs:6507`) is
right — two implementations of "is this a spell", disagreeing, exactly the F3 argument.
(2) The SR-36 roster's `P = 48`, which Stage 7 is instructed to write into the queue row and the
`OOS-SIM3-5` row as *the measured number*, is an **undercount by at least 20 pairs**: the
enumeration walks `Effect::CounterSpell` only and is structurally blind to `Effect::CounterUnlessPays`,
which `effects/mod.rs:4401-4411` delegates straight into the arm under repair — `mana_leak`,
`mana_tithe` and `make_disappear` are all `Complete`, all semantically "counter target spell", and
all were live-wrong against all six `Complete` mutate defs.
(3) T6's advertised non-vacuity property does not exist: `assert_eq!(variants.len(), 27)` compares
the hand-written fixture against itself and cannot detect a 28th enum variant.
No HIGH: nothing here contradicts a CR rule in shipped behaviour, produces an illegal game state,
or moves a wire fingerprint.

---

## Findings

| # | Severity | Site | Description |
|---|----------|------|-------------|
| 1 | MEDIUM | `crates/engine/src/rules/abilities.rs:6732-6737` | **A fifth classification site, disagreeing with its own sibling.** `targeting_is_spell` matches `StackObjectKind::Spell` alone; CR 702.140a makes a mutating creature spell a spell. `casting.rs:6507` pairs both kinds for the identical question. **Fix:** pair `MutatingCreatureSpell` at `:6736` (mirroring `casting.rs:6507` verbatim, with the CR 702.140a cite), or file it as a new seed and correct the plan/notes' "FOUR sites" census to six. |
| 2 | MEDIUM | `crates/engine/src/rules/casting.rs:7124-7138` | **A sixth site, and it is `card_in_stack_zone`'s exact question left unconverted.** `has_split_second_on_stack` reads `source_object` off `StackObjectKind::Spell` only. **Fix:** rewrite as `crate::state::stack_registry::card_in_stack_zone(&stack_obj.kind).and_then(...)`, or record it in the notes as a known unconverted consumer with the reason. |
| 3 | MEDIUM | `crates/engine/tests/core/pb_dx25_stack_registry_roster.rs:295-331, 536-545` | **`P = 48` is an undercount and the gate's own message is false about a class it cannot see.** `effect_contains_counter_spell` matches `Effect::CounterSpell` only; `Effect::CounterUnlessPays` delegates into the same arm. **Fix:** add `Effect::CounterUnlessPays { target, .. }` to the walk (and to `counter_target_requirement`), re-measure C1/C2/C3/P, and put the corrected number — not 48 — into the queue row and the `OOS-SIM3-5` row at Stage 7. |
| 4 | MEDIUM | `pb_dx25_counterspell_stack_shapes.rs:1191-1207` | **T6's non-vacuity claim is vacuous.** The literal `27` is compared against the fixture's own `Vec` length. **Fix:** either derive the count from source (the G1 `arm_count` scan already does, at `pb_dx25_stack_registry_roster.rs:145`) and assert T6 against *that*, or delete the claim from T6's doc comment and point it at `g1_scan_is_not_vacuous`. |
| 5 | MEDIUM | `crates/engine/src/state/stack_registry.rs:32-34` | **A doc cross-reference to a comment that was never written.** Plan §3.4 mandated a comment at `casting.rs:6503`; there is none. **Fix:** add the §3.4 comment at `casting.rs:6503`, or delete the pointer. |
| 6 | MEDIUM | `crates/engine/src/rules/events.rs:159-167` | **The event's documented contract is now false for two of the three cases that emit it.** **Fix:** restate the doc to cover all three payload shapes, including the `stack_object_id == source_object_id` copy marker (plan §4.3). |
| 7 | LOW | `crates/engine/src/rules/resolution.rs:8368-8439` | **Unplanned, untested behaviour widening on a `pub` API.** The `named` branch is new here; T7 covers neither the ability-naming arm nor a copy of an ability. **Fix:** add a T7 half for `ActivatedAbility`, or record the delta explicitly. |
| 8 | LOW | `pb_dx25_stack_registry_roster.rs:5-11`, `:159-176`; `resolution.rs:8320` | **Two false/rotten claims in shipped comments.** The module doc's "both revert shapes prove the load-bearing property" is wrong for G1; `resolution.rs:8320` cites "PB-DX9", an unshipped queue entry (the precedent is PB-DP9). **Fix:** restate the module doc per-gate; correct PB-DX9 → PB-DP9. |
| 9 | LOW | close-out | **Stage 7 is outstanding while shipped source already asserts its outcome.** `invariants.rs:288` says "**PB-DX25 closes `OOS-SIM3-5`**"; the registry of record still has it open, the queue row still carries the refuted "6 × 24", and `OOS-DX25-1..6` are unfiled. **Fix:** complete Stage 7 before merge. |

### Additional LOW notes (no separate row; fold into the fix pass)

- `pb_dx25_counterspell_stack_shapes.rs:717-891` — T4 never exercises `cast_with_jump_start`
  (CR 702.133a) although `push_spell` carries the parameter and always passes `false`, and never
  exercises the `unwrap_or(controller)` owner fallback. Plan §3.5 named both as "individually
  probed (§6 T4/T5)". Add a fourth sub-case, or narrow the plan's claim in the notes.
- `pb_dx25_counterspell_stack_shapes.rs:678-705` — T3's non-vacuity half is described as a
  "sibling fixture"; it is the *same* `state`, continued after the copy-counter. Functionally it
  still proves the capability; the wording is wrong.
- `effects/mod.rs:2771-2775` — "moving `source_object` here would put someone else's spell in the
  graveyard" is imprecise for the cipher-copy population (`resolution.rs:5418-5430`), whose
  `source_object` is a card in **Exile**. Worth stating, because it is a *positive* the batch did
  not claim: the `is_copy` guard also closes a CR 702.99c hole (countering a cipher copy via the
  `so.id == id` clause would have pulled the encoded card out of exile into the graveyard).
- `invariants.rs:284-286` — the pre-existing sentence "Two live engine defects that legitimately
  trip this check are filed as `OOS-SIM3-5`" is left standing and is now known false: shape (c)
  produces no divergence, and shapes (a)/(b) are unreachable. The new paragraph at `:288` corrects
  it in prose without striking it.
- `pb_dx25_stack_registry_roster.rs:536-545` — G3's P-message hard-codes "expected 48" *and* prints
  `got {p}`, so under the executed revert (pin flipped to 49) it printed "expected 48 ..., got 48".
  Only `assert_eq!`'s left/right made it readable.
- `pb_dx25_stack_registry_roster.rs:275-287, 325-331` — `has_spell_level_target_requirement` and
  `counterspell_defs` walk the **front face only**, while `mutate_defs` walks both. Moot today
  (no DFC mutate def), but the asymmetry is undocumented at M3.
- `strip_line_comments` is naive about `//` inside string literals; G2 brace-balances over the
  stripped text of `effects/mod.rs`. A future literal could break extraction — loudly (panic), not
  silently, so this is a robustness note only.

---

## Finding Details

### Finding 1 — a fifth classification site, wrong in the same direction, missed by the census

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:6732-6737`
**CR**: 702.140a — "it becomes a mutating creature spell"; 601.2c vs 602.2b (the spell-vs-ability
gate this code implements)

```rust
let targeting_is_spell = state
    .stack_objects
    .iter()
    .find(|so| so.id == targeting_stack_id)
    .map(|so| matches!(so.kind, StackObjectKind::Spell { .. }))
    .unwrap_or(false);
```

`collect_permanent_becomes_target_triggers` uses this to gate every
`TriggerEvent::PermanentBecomesTarget` whose `include_abilities` is `false` (CR 601.2c, "becomes the
target of a **spell**"). A mutating creature spell **is** a creature spell (CR 702.140a), so a
"whenever this becomes the target of a spell an opponent controls" trigger must fire off it and
does not.

This matters for the batch for two reasons, not one:

1. It is the **same question** `casting.rs:6504-6509` answers, one function over, and that site
   pairs `Spell { .. } | MutatingCreatureSpell { .. }`. Two implementations of "is this stack object
   a spell", disagreeing — which is verbatim the argument plan §0.2 F3 makes for building the
   registry. The plan's F3 table lists **four** sites; there are at least **six**.
2. Its reachability status is *identical* to shape (a)'s — latent only because roster M3 = 0 and
   `OOS-DX25-1` keeps the mutate target out of `spell_targets`. The batch fixed shape (a) anyway,
   on the stated grounds that a latent CR-wrong branch in the same family is worth one `if`. The
   same reasoning applies here and was not applied.

Note this is *not* a `card_in_stack_zone` consumer — plan §3.4 is right that "is it a spell" must
not be re-expressed through the registry (CR 707.10: a copy is a spell with no card). The fix is
the two-variant pairing, not the registry.

**Fix**: pair `MutatingCreatureSpell { .. }` at `abilities.rs:6736`, mirroring `casting.rs:6507`
with the CR 702.140a citation; add a probe or an explicit latency note. If the fix is judged out of
scope, file it as a new seed and correct the F3 census in `pb-plan-DX25.md` §0.2 and
`pb-DX25-execution-notes.md` from "four sites" to six.

### Finding 2 — `has_split_second_on_stack` is `card_in_stack_zone`'s exact question, unconverted

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:7124-7138`
**CR**: 702.72a (Split Second), 702.140a

```rust
if let StackObjectKind::Spell { source_object } = &stack_obj.kind { ... } else { false }
```

This is not "is it a spell" — it is literally *"which card does this stack entry own, so I can read
its keywords"*, i.e. `card_in_stack_zone`. It is the fifth of the six sites and the only remaining
one that the registry was built to replace. A `MutatingCreatureSpell` with Split Second would not
suppress casting. No printed card makes this live (no mutate card has split second), which is why
it is MEDIUM and not HIGH — but the batch's whole thesis is "the classification is made once", and
this site was left as a second copy of it.

**Fix**: rewrite as
`crate::state::stack_registry::card_in_stack_zone(&stack_obj.kind).and_then(|card| calculate_characteristics(state, card).ok()).map(...)`,
preserving the existing CR 400.7/113.7a LKI comment. If deferred, record it in the execution notes
as a known unconverted consumer with the reason, so the "one classification" claim is scoped
honestly.

### Finding 3 — the roster's `P = 48` is blind to `Effect::CounterUnlessPays`

**Severity**: MEDIUM
**File**: `crates/engine/tests/core/pb_dx25_stack_registry_roster.rs:295-306` (`effect_contains_counter_spell`),
`:345-373` (`counter_target_requirement`), `:531-545` (the P pin)
**CR**: 118.12a (the "unless its controller pays" cost), 701.6a

`effects/mod.rs:4401-4411`:

```rust
Effect::CounterUnlessPays { target, cost: _ } => {
    execute_effect_inner(state, &Effect::CounterSpell { target: target.clone(), exile_instead: false }, ctx, events);
}
```

So every `CounterUnlessPays` def resolves through the arm this batch repaired, and every one of them
was equally a silent no-op against a mutate spell before the fix. The roster's walk never looks for
that variant. Measured from the corpus:

| def | effect | target requirement | completeness | semantically "target spell"? |
|---|---|---|---|---|
| `mana_leak` | `CounterUnlessPays` | `TargetSpellWithFilter(TargetFilter::default())` | `Complete` (derive) | **yes** |
| `mana_tithe` | `CounterUnlessPays` | `TargetSpellWithFilter(TargetFilter::default())` | `Complete` (derive) | **yes** |
| `make_disappear` | `CounterUnlessPays` | `TargetSpellWithFilter(TargetFilter::default())` | `Complete` (derive) | **yes** |
| `izzet_charm` | `CounterUnlessPays` (mode 0) | `TargetSpellWithFilter(TargetFilter { .. })` | — | filtered |
| `flusterstorm`, `spell_pierce`, `stubborn_denial` | `CounterUnlessPays` | filtered | — | no |

`TargetFilter::default()` sets no restriction (`card_definition.rs:3036-3080` — every field is
`None`/`false`/default), so those three are "counter target spell" with an extra cost clause,
exactly like the eight in C3.

So the true live-wrong pair count is at least **6 × (8 + 3) = 66**, plus the ~2 `red_elemental_blast`
pairs the notes already record as unpinned — i.e. **≥ 68, not 48**. The execution notes instruct
Stage 7 to write "the measured live-wrong pair count is 48 (not '~48', not 144)" into
`seed-rerank-2026-08-02.md` §4 row 7 and the `OOS-SIM3-5` row. That would replace one wrong number
with another, and would do it with the authority of an SR-36 enumeration.

G3's own failure message compounds it: *"a new mutate def or a new unrestricted counter def widens
the class"* — a new `CounterUnlessPays` def with a default filter widens the class and this gate
cannot see it.

Two smaller scoping gaps in the same helpers, worth fixing in the same pass:
`ability_contains_counter_spell` matches only `AbilityDefinition::Spell` (a counter on an activated
or triggered ability is invisible), and `counterspell_defs` walks the front face only.

**Fix**: extend `effect_contains_counter_spell` with `Effect::CounterUnlessPays { .. } => true` and
extend `counter_target_requirement` correspondingly; treat `TargetSpellWithFilter(f)` where
`f == TargetFilter::default()` as unrestricted (or add a fourth pinned population `C4` for it and
state why C3 stays syntactic); re-measure C1/C2/C3/P; and write the **re-measured** number into the
queue row and the seed row at Stage 7. Do not ship "48".

### Finding 4 — T6's non-vacuity is a self-comparison

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_dx25_counterspell_stack_shapes.rs:1191-1207`

The doc comment claims: *"the fixture's own variant count is asserted equal to the measured count
(27 at HEAD), so a 28th variant that compiles (because it was classified in the registry) but is
never added to this fixture cannot silently escape T6's coverage."*

`one_of_each_variant()` is a hand-written `vec![...]`. `assert_eq!(variants.len(), 27)` compares
that vec's length with the literal `27`. If a 28th `StackObjectKind` is added and classified in the
registry, `variants.len()` is still 27 and T6 passes. The stated property is held instead by
`g1_scan_is_not_vacuous` (`pb_dx25_stack_registry_roster.rs:142-152`), which counts arms in the
**source file** — a different subject, in a different crate target.

This is precisely the PB-DX24 durable lesson ("a guard, a gate and a claim each have a subject") in
the batch dispatched after it.

**Fix**: make T6's floor source-derived — reuse the G1 arm-count scan, or assert against a constant
that also gates the enum (e.g. a `hash.rs`-side count) — or delete the sentence from T6's doc and
replace it with an explicit pointer to `g1_scan_is_not_vacuous` as the place the property lives.

### Finding 5 — a doc cross-reference to a comment that does not exist

**Severity**: MEDIUM
**File**: `crates/engine/src/state/stack_registry.rs:32-34`

> "See the comment at that call site (`casting.rs`, near line 6503) for the other half of this note."

`casting.rs:6503` reads `// Spell-only: reject activated/loyalty abilities and non-spell stack
objects.` — it names neither `stack_registry` nor CR 707.10, and there is no other-half note
anywhere near it. Plan §3.4 required it: *"a comment at `casting.rs:6503` names the registry and
says why it is not used there."*

The missing comment is the only thing that would stop a future author from "simplifying"
`is_spell` into `card_in_stack_zone(..).is_some()`, which CR 707.10 makes wrong (a copy of a spell
is a spell and owns no card). This is the MR-M11-12 class: a cite pointing at a sentence that does
not exist.

**Fix**: add the §3.4 comment at `casting.rs:6503`, citing CR 707.10 and naming
`state::stack_registry::card_in_stack_zone` as the function this check must *not* be re-expressed
through. (Alternatively delete the pointer in `stack_registry.rs` — but then the guard is prose in
one file only.)

### Finding 6 — `SpellCountered`'s documented contract contradicts its shipped payload

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/events.rs:159-167`

```rust
/// A spell was countered without resolving (CR 608.2b, 701.6a).
///
/// The card is put into its owner's graveyard. `source_object_id` is the
/// card's new ID in the graveyard.
SpellCountered { player, stack_object_id, source_object_id },
```

The first line was edited by this batch (701.5 → 701.6a). The two lines under it are now false for
two of the three shapes that emit the event:

- **a countered copy** (new in this batch, plan §4.3): no card moves at all, and
  `source_object_id == stack_object_id == ` the copy's stack-entry id, deliberately chosen as a
  machine-detectable "this was a copy" marker;
- **a countered activated/triggered ability** (pre-existing in `effects/mod.rs`, and *newly* emitted
  by `counter_stack_object`): `source_object_id` is the ability's **source**, which has not moved
  and is not in a graveyard.

`view-model/src/event_view.rs:791-794` already relies on the copy shape resolving `card_name()` to
`None`. The design is right; the type-level documentation of the wire contract was not updated
alongside it.

**Fix**: restate the doc to enumerate the three payload shapes (card-owning non-copy → post-move
card id; copy of a card-owning kind → own stack-entry id, CR 707.10; activated/triggered ability →
unmoved source, CR 707.10b), and note the `stack_object_id == source_object_id` copy marker.

### Finding 7 — `counter_stack_object` gained an unplanned, untested emission branch

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:8368-8439`

Plan §3.6 authorises exactly three changes to this function: collapse the OR-list onto the
classification, add the `is_copy` guard, fix the stale doc. The shipped rewrite also adds the whole
`named` branch — a `SpellCountered` for countered `ActivatedAbility`/`TriggeredAbility` kinds and
for copies — which the execution notes describe as *"both absent from the function's original
body"*. That is a real behaviour change on a `pub` API, made to align the two paths, and it is a
defensible one, but:

- T7 exercises only the mutate half and the copy-of-a-`Spell` half. The `ActivatedAbility` /
  `TriggeredAbility` naming arm — the genuinely new emission — has **no** test on either path
  through `counter_stack_object`.
- Consequences are nil today (zero production callers), which is why this is LOW, not MEDIUM.

The verbatim survival of the per-keyword "if countered by Stifle …" comment block was checked and
holds: all fourteen `Note: For …` lines are present on the `_ => None` arm at `:8391-8429`. No
information carried by the 20-variant OR-list was lost — it was a flat enumeration with a single
shared body.

**Fix**: add a third T7 half over a countered `ActivatedAbility` (assert the event names the unmoved
source, CR 707.10b), or record the widening explicitly in the notes as an accepted plan divergence.

### Finding 8 — two false claims in shipped comments

**Severity**: LOW

(a) `crates/engine/tests/core/pb_dx25_stack_registry_roster.rs:5-11`:

> "This file's own gates prove that load-bearing property by executing BOTH revert shapes (`//` and
> `/* */`), not just the line-comment one."

For **G1** the block-comment property cannot be load-bearing: a `/* */`-wrapped wildcard is not
compiled, and `card_in_stack_zone`'s match is exhaustive with no wildcard, so the file would fail to
build. The execution notes are candid that the executed block-comment revert tested the *inverse*
property (that `strip_block_comments` does not over-strip a real wildcard sitting after a real
comment) — a different, weaker experiment than the module doc advertises. Stripping *is* genuinely
load-bearing for **G2**'s `card_in_stack_zone_calls >= 2` clause, where unstripped comment mentions
(`effects/mod.rs:2740`, `:2835`) would inflate the count and let comments satisfy a code gate; that
is the sentence worth writing.

Related: `g1_line_comment_stripping_does_not_hide_the_wildcard_it_is_meant_to_find` (`:160-176`)
runs on a synthetic string literal, not on the gated file — it tests the helper, not the gate. Its
own doc says as much; the module doc does not.

(b) `crates/engine/src/rules/resolution.rs:8320` cites *"PB-DX9's precedent for this exact
function's tail"*. **PB-DX9 has not shipped** — it is an unranked/queued entry in the v3 queue. The
precedent named by plan §3.6 and by the function's own pre-existing tail comment (`:8447-8453`) is
**PB-DP9**.

**Fix**: restate the module doc per-gate (G1: compile-forced, stripping is defence-in-depth; G2:
stripping is load-bearing, and here is why); correct `PB-DX9` → `PB-DP9`.

### Finding 9 — Stage 7 outstanding while shipped source asserts its outcome

**Severity**: LOW (blocking for merge, not for correctness)

`crates/simulator/src/invariants.rs:288` ships the sentence **"PB-DX25 closes `OOS-SIM3-5`"**. A
repo-wide `PB-DX25|DX25` census returns **no edit** to `docs/audits/decision-point-audit.md`, and
`memory/primitives/seed-rerank-2026-08-02.md:723` still reads

> `| **7** | **PB-DX25** | ... | 0 flips; 6 `Complete` mutate defs × 24 counter defs | ...`

i.e. unstruck, and still carrying both refuted numbers (the "24", which the batch itself measured as
23, and the implied 144). `OOS-DX25-1..6` are unfiled. Per the standing dispatch-hygiene rule the
registry is ground truth and status prose lags it — here shipped *source* has got ahead of the
registry, which is the same hazard one layer worse.

**Fix**: complete Stage 7 before merge — close `OOS-SIM3-5` in `decision-point-audit.md` with its
own corrections recorded (shape (c) was live and (a) the rider; (a) becomes reachable only once (c)
is fixed; (b) unreachable three ways), strike queue row 7 with the **re-measured** P from Finding 3,
and file `OOS-DX25-1..6` plus new seeds for Findings 1 and 2 if they are not fixed in-batch.

---

## What I checked that holds up

Stated so the fix pass does not re-litigate settled ground.

| claim | how I checked it | verdict |
|---|---|---|
| `Spell` + `MutatingCreatureSpell` is the complete card-owning set | enumerated every `ZoneId::Stack` occurrence in `crates/engine/src` and read all four `move_object_to_zone`/`fizzle_move_object_to_zone`/`expect_move_object_to_zone` destinations (`casting.rs:4423`, `copy.rs:383`, `copy.rs:604`, `resolution.rs:6137`); read the kind chosen at each | **correct** — three build `Spell`, the fourth (`casting.rs:4528-4545`) picks `MutatingCreatureSpell` *after* the same single move |
| 27 variants, no wildcard | `pub enum StackObjectKind` at `card-types/src/state/stack.rs:573`, counted 27 variant heads; `stack_registry.rs:71-109` has 2 `Some` + 25 `None`, no `_` arm | **correct** |
| `position()` is behaviour-preserving for a non-copy `Spell` | `card_in_stack_zone(Spell{s}) == Some(s)`, so clause (ii) is character-equivalent to the old `matches!` | **correct** |
| the `!so.is_copy` guard cannot suppress a legitimate lookup | `is_copy: true` is written at exactly two places (`copy.rs:165`, `resolution.rs:5429`); the Ward clause `so.id == id` is unguarded | **correct** |
| shape (b) unreachability reason 2 (dead-id filter) | `move_object_to_zone` does `self.objects.remove(&object_id)` (`state/mod.rs:1303`) and mints a fresh id (`:1305`); `resolve_effect_target_list_indexed:7680-7686` requires `contains_key` or a live `so.id` | **correct** |
| the `else` branch cannot lose a card | reached only when `card_owned` is `None` (no card exists) or `is_copy` (card belongs to the original) | **correct — diagnostics-only omission** |
| a copy of an **ability** still names `source_object` (CR 707.10b) | `card_owned.is_some()` is false for every ability kind including copies, so control reaches the `ActivatedAbility \| TriggeredAbility` arm | **correct** |
| the "if countered by Stifle …" comment block survived | read `resolution.rs:8391-8429` — all fourteen `Note: For …` lines present, relocated onto `_ => None` | **correct** |
| the OR-list carried no information the registry lacks | it was a flat enumeration with one shared body | **correct** |
| CR 701.6 is "Counter", CR 701.5 is "Cast", no 701.5g | MCP `get_rule 701.6` (two subrules, text as quoted) | **correct**; the three in-batch corrections are right |
| T4 sub-case 4's CR 118.9 citation | MCP `get_rule 118.9` → 118.9a "Only one alternative cost can be applied to any one spell as it's being cast" | **correct** |
| `card_name(copy_stack_id)` renders "<player>'s spell is countered" | `event_view.rs:791-794`; stack-entry ids come from the same monotone `next_object_id` and are never `GameObject` keys | **correct** |
| nothing dispatches a trigger off `SpellCountered` | repo-wide symbol census: producers in `effects/mod.rs` + `resolution.rs`, consumers are `event_view.rs`, `hash.rs` and tests only | **correct — no trigger reads the payload** |
| C1 = 23 and the Transcendent Dragon grep artefact | `Effect::CounterSpell` appears in 24 def files; `transcendent_dragon.rs` carries it only inside a `Completeness::partial` note string | **correct**, and a good SR-36 catch |
| M3 = 0 is a sufficient condition for shape (a)'s corpus-unreachability | Ward needs `PermanentTargeted`, emitted only from `battlefield_targets` built out of `spell_targets`; no spell-level targets at all ⇒ none | **correct** |
| F2 (Ward never fires on a copy) | zero `PermanentTargeted` occurrences in `rules/copy.rs` | **correct** |
| `OOS-DX25-3` is real (deliberately not fixed) | `casting.rs:6426` keys on `state.objects`; `:6476`/`:6502` compare against `so.id` — disjoint id spaces, so `is_spell` is always false and `target_count` always 0 | **correct, and correctly deferred** |
| wire neutrality | no enum/struct shape change in the diff; `stack_registry` is a free function | **consistent with the gate-executed PROTOCOL 35 / HASH 73** |
| SR-9a | `tests/primitives/main.rs:38` and `tests/core/main.rs:33` both carry their `mod` line | **correct** |
| SR-6 | `PB-DX25` census returns no file under `crates/card-defs/` or `crates/card-types/`; the registry was placed in the engine per the `keyword_registry` precedent | **correct** |

### An unclaimed positive

The `is_copy` guard closes a defect outside the three named shapes. `resolution.rs:5418-5430`
builds a cipher copy as `StackObjectKind::Spell { source_object: encoded_object_id }` with
`is_copy = true`, where `encoded_object_id` is a card **in Exile** (CR 702.99c: it stays encoded
there). Before this batch, countering that copy through the `so.id == id` clause would have pulled
the encoded card out of exile into a graveyard. Neither the plan's behaviour-delta list nor the
execution notes record this; it is worth one sentence at `effects/mod.rs:2771-2775`, which currently
says only "would put someone else's **spell** in the graveyard".

---

## Probe / gate assessment

| id | discriminating? | notes |
|---|---|---|
| T1 | **yes** — fail-before executed at HEAD, failure text recorded verbatim; revert re-executed | real corpus pair, `gemrazer` × `counterspell`. `drain_stack` correctly re-reads `priority_holder` each iteration (the runner's own CR 117.3b fixture bug, fixed and recorded) |
| T2 | **yes** — revert (restore `_ => {}`) watched red | synthetic; the doc honestly states the route (hand-built `EffectContext`, not the Ward trigger machinery) and why (M3 = 0, `OOS-DX25-1`). Reads only state the successful call produced — no PB-DX21-class vacuity |
| T3 | **yes** — revert (drop the `is_copy` guard) watched red, and it fails on the *original's* card zone, the right subject | uses the real `pub copy_spell_on_stack`. Non-vacuity half genuinely moves a card. "Sibling fixture" is a misnomer (same state, continued) |
| T4 | partly | sub-case 3 discriminates owner-vs-controller correctly (owner p1 / controller p2). `cast_with_jump_start` and the `unwrap_or(controller)` fallback are named in plan §3.5 as probed and are not. Sub-case 4 is a structural assertion in prose only, correctly cited (CR 118.9a) |
| T5 | **yes** — revert (move the assignment below the `cant_be_countered` check) watched red | genuinely newly reachable: it targets the CARD id, i.e. clause (ii) |
| T6 | classification content: yes. Non-vacuity: **no** — Finding 4 | |
| T7 | **yes** for the two halves it has; **no coverage** of the new ability-naming branch — Finding 7 | |
| G1 | passes; the wildcard scan is real, `g1_scan_is_not_vacuous` pins 27 arms from source | the block-comment framing is overstated — Finding 8(a) |
| G2 | passes; **not** vacuously satisfiable at the current thresholds (2 real calls exactly, 1 fizzle call, `>= 400` chars). Comment-stripping *is* load-bearing here | defeatable by the `use StackObjectKind as K` alias form the registry itself uses. And there is **no source gate at all over `counter_stack_object`** — criterion 6232's "single classification, both paths" half is satisfied by argument plus T7, not by machine |
| G3 | pins real measured values with a name-pinned M1 and a `>= 1_700` floor | subject is narrower than its claim — Finding 3. The `>= 1_700` floor is adequate for its stated job (a broken enumeration returning nothing) |
| File C | headline half non-discriminating at HEAD **by construction**, and the module doc says so; the behavioural half is the fail-before record and was executed red; the mandatory non-vacuity sibling is present and real | correctly built and correctly described |

---

## CR Coverage Check

| CR rule | Implemented? | Tested? | Notes |
|---|---|---|---|
| 701.6a (countered spell → owner's graveyard) | Yes | Yes | T1 (real cards), T2, T7 half 1 |
| 701.6a on a mutating creature spell (702.140a) | Yes | Yes | T1 end-to-end through `process_command`; File C in a real game |
| 707.10 (a copy is a spell with no card) | Yes | Yes | T3, T7 half 2 |
| 707.10a (copy in a non-stack zone ceases to exist) | Yes (entry removed outright) | Indirect | no explicit SBA probe; adequate |
| 707.10b (ability copy keeps its source) | Yes | **No** | reasoned correct; the arm is untested on both paths — Finding 7 |
| 702.34a / 702.133a (flashback / jump-start → exile) | Yes | flashback yes, **jump-start no** | T4 |
| 400.7 (new object identity) | Yes | Yes | T1/T2/T7 locate by name and assert `assert_ne!` on the id |
| 101.2 / 101.6 (`cant_be_countered`, do as much as possible) | Yes | Yes | T5 |
| 118.9a (one alternative cost ⇒ no mutate+flashback) | n/a (structural) | Asserted in prose | T4 sub-case 4 |
| 729.2 (merge must not happen) | Yes | Yes | T1 + File C, on `merged_components` (not on id, per the mutate gotcha) |
| 601.2c / 608.2b (target is the card in `ZoneId::Stack`) | Yes | Yes | T1 announces the card id and it validates |
| 702.21a (Ward-shaped stack-entry lookup) | Yes | Synthetic | T2 — and the real Ward path remains unreachable (`OOS-DX25-1`) |
| 118.12a (`CounterUnlessPays` → the same arm) | Yes (pre-existing delegation) | **No** | not probed against a mutate spell; not in the roster — Finding 3 |
| 702.99c (cipher copy's card stays exiled) | Yes (incidentally, via `is_copy`) | No | unclaimed positive |
| 601.2c/602.2b spell-vs-ability targeting gate | **No** — `abilities.rs:6736` | No | Finding 1 |
| 702.72a (split second) | **No** — `casting.rs:7126` | No | Finding 2 |

---

## Card Def Summary

| Card | Modified? | Oracle match | Game state correct | Notes |
|---|---|---|---|---|
| — | none | n/a | n/a | 0 card-def lines; coverage unmoved 1,133/1,803 = 62.8% by an empty diff, as planned |

Defs read for verification only (unmodified): `gemrazer` (explicit `Completeness::Complete`,
`gemrazer.rs:74` — matches the plan), `counterspell`, `access_denied` (`targets[0] =
TargetRequirement::TargetSpell`, confirming G3's slot-0 assumption for the `Sequence`-nested case),
`mana_leak` / `mana_tithe` / `make_disappear` / `izzet_charm` (Finding 3).

---

## Deliberately-not-fixed items — my judgement

| id | judgement |
|---|---|
| `OOS-DX25-1` (mutate target is not modelled as a target) | **correctly deferred.** Verified: `AdditionalCost::Mutate` never reaches `spell_targets`, and `battlefield_targets` is built from `spell_targets` alone. It is the load-bearing reason shape (a) and Finding 1 are both latent. Its scope is genuinely much larger than this batch |
| `OOS-DX25-2` (Ward never fires on a copy) | **correctly deferred.** Confirmed: zero `PermanentTargeted` in `rules/copy.rs` |
| `OOS-DX25-3` (`TargetSpellWithSingleTarget` unsatisfiable) | **correctly deferred, and correctly characterised.** Re-derived from `casting.rs:6426` vs `:6476`/`:6502` — the two id spaces never intersect |
| `OOS-DX25-4` (`SpellCountered` for 2 of 25 ability kinds) | **correctly deferred**, and the plan's proposed fix shape (a sibling `source_of(&kind)` in `stack_registry`) is the right one. Note it now applies to **two** functions, since `counter_stack_object` grew the same two-arm branch (Finding 7) |
| `OOS-DX25-5` (`counter_stack_object` has no production caller) | **correctly deferred.** The keep-vs-delete question is now sharper, not softer: the function has gained behaviour (Finding 7) and is pinned only by T7 |
| `OOS-DX25-6` (CR 701.5 vs 701.6 rot, ~337 sites) | **correctly deferred.** Confirmed still present in card defs (e.g. `access_denied.rs:20`) and out of scope |
