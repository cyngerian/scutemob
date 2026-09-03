# PB-DX20b — revert matrix for the two ENGINE test files

Rows executed by `scutemob-222`'s engine-test runner. **Every row below was executed**, its
failure line captured verbatim from the test binary's own output, and then restored; the
matrix closes with a green re-run of both targets.

## Where these ran, and why it is not this worktree

**A parallel worker was mutating `crates/card-defs/src/defs/imprisoned_in_the_moon.rs` while
this matrix was being planned** — `git status` in `/home/skydude/projects/scutemob/.worktrees/
scutemob-222` flipped that file between `Filtered`, `Permanent` and `Creature` across three
consecutive commands, and one of this file's early runs read `Permanent` off a def whose
committed text is `Filtered`. Two agents doing revert matrices on one card-def corpus cannot
both restore safely.

So the whole matrix ran in an isolated `git worktree` at the batch commit **`0be8d904`**, with
its own `CARGO_TARGET_DIR`:

```
git worktree add <scratch>/dx20b-iso 0be8d904
# + the two new test files + their two `mod` lines
```

`git status --short` in that worktree at the end of the matrix shows only the two new test
files and the two `mod` registrations — **no `src/` or `card-defs/` path is modified**. The
shared worktree was never touched by a revert.

Cleanup: `git worktree remove <scratch>/dx20b-iso --force`.

## Matrix

Legend: **RED** = the row fails as required. **green (control)** = the row must stay green and
does; a control that reddened would mean the revert broke something other than its subject.

| # | revert | primitives | core |
|---|---|---|---|
| R1 | `imprisoned_in_the_moon` widened back to `EnchantTarget::Permanent` (the merge-base declaration) | **t4, t5, t6 RED**; t1/t2/t3/t6b/t7/t8/t9 green (controls) | **r1, r2, r3, r4 RED**; r1b/r5 green |
| R2 | `has_card_types: f.has_card_types.clone()` deleted from `casting::enchant_filter_to_target_filter` | **t4, t5, t6 RED**; t1/t2/t3/t6b/t7/t8 green; **t9 green — see below** | **r5 RED**; r1/r1b/r2/r3/r4 green |
| R3 | the `matches_filter` call deleted from `sba::enchant_filter_matches` | **t6, t9 RED**; t4/t5 green — see below | all 7 green |
| R4 | `controller: EnchantControllerConstraint::You` dropped from `kayas_ghostform` | **t7 RED**; rest green | **r1, r2, r3 RED**; r4/r5/r1b green |
| R5 | an eighth field planted on `EnchantFilter`, pin NOT updated | all 10 green (an inert field changes no behaviour) | **r5 RED** at the field-list pin |
| R5b | the same field planted **and** added to `KNOWN_ENCHANT_FILTER_FIELDS`, lowering untouched | — | **r5 RED** at the *unlowered* assertion |
| R6 | `controller: EnchantControllerConstraint::You` dropped from `breath_of_fury` | **t8 RED**; rest green | **r1, r3 RED**; r2/r4/r5/r1b green |
| R7 | `curse_of_opulence` given `AbilityDefinition::Keyword(Enchant(EnchantTarget::Player))` | — | **r1 RED at the allowlist REASON check**, plus r3, r4 |
| R8 | the parser's UNCLASSIFIED branch changed to accept any lowercase token as a subtype | — | **r1b RED**; r1/r2/r3/r4/r5 green |
| R9 | `has_card_type: Some(CardType::Land)` dropped from `awaken_the_ancient` | — | **r1, r3 RED** (the CR 205.3i implication is load-bearing) |
| R10 | `sba::matches_enchant_target`'s `Filtered` arm made to return `false` unconditionally — the "detach everything" bug | **t6b RED**, and t1/t2/t3/t7/t8/t9 RED; **t6, t4, t5 GREEN** | — |

**Every row is discriminating. There is no UNDISCRIMINATED row in this matrix.**

## Verbatim failure lines

### R1 — `imprisoned_in_the_moon` back to `EnchantTarget::Permanent`

```
thread 'pb_dx20b_enchant_card_type_or::t4_imprisoned_refuses_an_artifact' panicked at
crates/engine/tests/primitives/pb_dx20b_enchant_card_type_or.rs:266:18:
OOS-DX20-10: Imprisoned in the Moon prints "Enchant creature, land, or planeswalker" and must
NOT accept `Board Artifact`. This is the live HIGH: at the merge base the def declared
EnchantTarget::Permanent, whose matches_enchant_target arm is a bare `true`, and PB-DX20 made
the widened offer human-reachable.
```

```
thread 'pb_dx20b_enchant_card_type_or::t5_imprisoned_refuses_an_enchantment' panicked at
crates/engine/tests/primitives/pb_dx20b_enchant_card_type_or.rs:266:18:
OOS-DX20-10: Imprisoned in the Moon prints "Enchant creature, land, or planeswalker" and must
NOT accept `Board Enchantment`. ...
```

```
thread 'pb_dx20b_enchant_card_type_or::t6_imprisoned_falls_off_an_artifact' panicked at
crates/engine/tests/primitives/pb_dx20b_enchant_card_type_or.rs:389:5:
CR 704.5m: Imprisoned in the Moon attached to an artifact is illegally attached ("Enchant
creature, land, or planeswalker") and must be put into its owner's graveyard
```

```
thread 'pb_dx20b_enchant_line_roster::r3_...' panicked at ...:934:9:
assertion `left == right` failed: PB-DX20b r3: the `EnchantTarget::Filtered` population moved.
live only: []; pinned only: ["Imprisoned in the Moon"]
  left: {"Awaken the Ancient", "Breath of Fury", "Chained to the Rocks", "Dimensional Exile",
         "Kaya's Ghostform", "Ossification"}
 right: {"Awaken the Ancient", "Breath of Fury", "Chained to the Rocks", "Dimensional Exile",
         "Imprisoned in the Moon", "Kaya's Ghostform", "Ossification"}
```

```
thread 'pb_dx20b_enchant_line_roster::r4_populations_are_pinned_by_count' panicked at ...:1055:5:
assertion `left == right` failed: PB-DX20b r4: the EnchantTarget::Filtered population moved:
["Awaken the Ancient", "Breath of Fury", "Chained to the Rocks", "Dimensional Exile",
 "Kaya's Ghostform", "Ossification"]
  left: 6
 right: 7
```

Also RED: `r1` (mismatch set non-empty) and `r2` (`Imprisoned in the Moon` stops declaring
`Filtered`).

### R2 — the new field deleted from the lowering

```
thread 'pb_dx20b_enchant_line_roster::r5_every_enchant_filter_field_is_lowered' panicked at
crates/engine/tests/core/pb_dx20b_enchant_line_roster.rs:1294:5:
CR 702.5a / `OOS-DX28-1`: EnchantFilter field(s) declared but never read by
`casting::enchant_filter_to_target_filter`: ["has_card_types"]. ...
Declared: {"basic", "controller", "has_card_type", "has_card_types", "has_subtype",
"has_subtypes", "nonbasic"}; lowered: {"basic", "controller", "has_card_type", "has_subtype",
"has_subtypes", "nonbasic"}
```

**`t9` stays GREEN under R2, and that is the file's own thesis executed.** Deleting the field
from the single lowering makes the offer, the cast and the SBA *all* wrong in the same
direction, so the consistency probe agrees perfectly while the engine is broken —
`t4`/`t5`/`t6` are what catch it. That is PB-DX20's durable lesson (*a differential probe
between two consumers of one function proves consistency, not correctness*) demonstrated by
execution rather than quoted, and it is why the module doc calls `t9` the structural half and
`t1`-`t8` the correctness half.

### R3 — `matches_filter` deleted from the SBA predicate

```
thread 'pb_dx20b_enchant_card_type_or::t9_offer_lowering_cast_and_sba_agree_across_the_filter_matrix'
panicked at crates/engine/tests/primitives/pb_dx20b_enchant_card_type_or.rs:941:13:
assertion `left == right` failed: PB-DX20b t9 surface 4: CR 303.4a (cast) and CR 704.5m (SBA)
disagree about the SAME (filter, permanent) pair. This is exactly the drift PB-DX20 left open by
keeping two hand-written copies of the field arithmetic.
[has_card_types [Creature, Land, Planeswalker] (imprisoned_in_the_moon)] x [own artifact]:
cast_legal=false fell_off=false
  left: true
 right: false
```

**`t4` and `t5` stay GREEN under R3**, and the reason is a finding — see "Findings" below.

### R4 — `kayas_ghostform` loses its controller clause

```
thread 'pb_dx20b_enchant_card_type_or::t7_...' panicked at ...:516:5:
OOS-DX20-5 (controller half): "you control" was dropped by the merge-base declaration, so an
opponent's creature was a legal target; got Ok("Ok")
```

```
thread 'pb_dx20b_enchant_line_roster::r1_printed_and_declared_enchant_lines_agree' panicked at
crates/engine/tests/core/pb_dx20b_enchant_line_roster.rs:574:5:
CR 702.5a (`OOS-DX20-10` / `OOS-DX20-5`): 1 def(s) declare an EnchantTarget that does not say
what their printed Enchant line says:
  Kaya's Ghostform [not-legal]: printed "Enchant creature or planeswalker you control" ->
  types=Creature|Planeswalker subtypes=- basic=false nonbasic=false controller=You BUT declared
  Some(Filtered(EnchantFilter { has_card_type: None, has_card_types: [Creature, Planeswalker],
  has_subtype: None, has_subtypes: [], basic: false, nonbasic: false, controller: Any })) ->
  types=Creature|Planeswalker subtypes=- basic=false nonbasic=false controller=Any
```

### R5 / R5b — an eighth `EnchantFilter` field

**`cargo build --workspace` FINISHED CLEAN with the planted field.** Verbatim:

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.91s
```

That reproduces the stage-1 runner's claim by execution: **zero** compile errors anywhere in the
workspace, because every `EnchantFilter` construction site in the corpus and in the test tree
uses `..Default::default()`. All 10 primitives probes also stayed green — an unlowered field is
behaviourally inert, which is precisely why it is dangerous.

R5 (pin not updated):

```
thread 'pb_dx20b_enchant_line_roster::r5_every_enchant_filter_field_is_lowered' panicked at ...:1284:5:
assertion `left == right` failed: PB-DX20b r5: EnchantFilter's field list moved.
live only: ["planted_eighth_field"]; pinned only: []. If a field was ADDED, lower it in
casting::enchant_filter_to_target_filter (nothing else will tell you — see this test's doc) and
add it here.
```

R5b (pin updated, lowering not) — this is the realistic failure, an author who edited the
constant and stopped:

```
thread 'pb_dx20b_enchant_line_roster::r5_every_enchant_filter_field_is_lowered' panicked at ...:1295:5:
CR 702.5a / `OOS-DX28-1`: EnchantFilter field(s) declared but never read by
`casting::enchant_filter_to_target_filter`: ["planted_eighth_field"]. ...
```

### R6 — `breath_of_fury` loses its controller clause

```
thread 'pb_dx20b_enchant_card_type_or::t8_breath_of_fury_is_your_creature_only' panicked at ...:580:5:
CR 702.5a: Breath of Fury prints "Enchant creature you control"; the merge-base
`EnchantTarget::Creature` dropped "you control" and accepted an opponent's creature; got Ok("Ok")
```

### R7 — the allowlist's stated REASON made false

```
thread 'pb_dx20b_enchant_line_roster::r1_printed_and_declared_enchant_lines_agree' panicked at ...:532:9:
PB-DX20b r1: `Curse of Opulence` is allowlisted as INEXPRESSIBLE (CR 702.5d —
`EnchantTarget::Player` EXISTS on the enum; what does not exist is the attachment path
(`GameObject.attached_to` has no player variant) and `sba.rs` rejects it. `OOS-DX20-2`.), yet it
now DECLARES an EnchantTarget (Some(Player)). If the construct became expressible, delete the
allowlist row and let the census compare the two sides.
```

### R8 — the parser's UNCLASSIFIED branch

```
thread 'pb_dx20b_enchant_line_roster::r1b_the_printed_line_parser_discriminates' panicked at ...:632:5:
assertion `left == right` failed: the parser must REFUSE an unclassifiable lowercase token rather
than mint a phantom SubType — that refusal is what makes `animate_dead` a reported residual
  left: Ok(EnchantSpec { types: {}, subtypes: {"creature card in a graveyard"}, basic: false,
        nonbasic: false, controller: "Any", player: false })
 right: Err("creature card in a graveyard")
```

### R9 — the CR 205.3i basic-land-type implication

```
thread 'pb_dx20b_enchant_line_roster::r1_printed_and_declared_enchant_lines_agree' panicked at ...:574:5:
CR 702.5a (`OOS-DX20-10` / `OOS-DX20-5`): 1 def(s) declare an EnchantTarget that does not say
what their printed Enchant line says:
  Awaken the Ancient [Complete]: printed "Enchant Mountain" -> types=Land subtypes=Mountain
  basic=false nonbasic=false controller=Any BUT declared Some(Filtered(EnchantFilter {
  has_card_type: None, has_card_types: [], has_subtype: Some(SubType("Mountain")), has_subtypes:
  [], basic: false, nonbasic: false, controller: Any })) -> types=- subtypes=Mountain basic=false
  nonbasic=false controller=Any
```

### R10 — "detach everything"

```
thread 'pb_dx20b_enchant_card_type_or::t6b_imprisoned_stays_on_a_creature' panicked at ...:425:5:
CR 704.5m: a creature IS one of Imprisoned in the Moon's three printed classes; detaching it
would mean the fix detaches everything, which `t6` alone cannot distinguish from a correct fix
```

`t6`, `t4` and `t5` are the three GREEN rows under R10, and that is the point of the control:
`t6` alone would have reported a healthy engine.

## Findings — things execution refuted

1. **The brief's "the message names the Enchant restriction" does not hold, and asserting it
   would have been a test passing for a reason nobody intended.** `casting.rs` does carry
   `InvalidTarget("target does not match Enchant restriction (…)")`, but its own block comment
   calls that gate *"a DELIBERATELY REDUNDANT second check"*: PB-DX20 synthesizes the
   announceable `TargetRequirement` upstream, so the declaration is refused first at CR 601.2c
   slot assignment. The verbatim message `t4`/`t5` receive is
   `InvalidTarget("declared 1 target(s) but 1 could not be matched to a requirement slot")`.
   Recorded in the helper's own doc; the assertion was re-keyed on the **mechanism** (refusal is
   `InvalidTarget`, **and** the offer layer excludes this permanent while still offering the
   printed-legal creature), which is the same claim without the phantom needle.

2. **R3 shows the CR 303.4a gate's `matches_enchant_target` call is NOT load-bearing in the
   accepting direction at HEAD.** With `matches_filter` deleted from `sba::enchant_filter_matches`,
   `t4`/`t5` still refuse — the requirement rejects them upstream — while `t6` (the SBA) and `t9`
   (cast-vs-SBA agreement) go red. R10 shows the reverse: the gate **is** load-bearing in the
   refusing direction (making the arm return `false` reddens `t1`/`t2`/`t3`, which the
   requirement had accepted). So the two paths are not redundant in both directions, and any
   later batch tempted to delete the gate as "already covered upstream" should read R3 and R10
   together.

3. **`r2`'s population is SEVEN, not the six the brief predicted.** `awaken_the_ancient` prints
   *"Enchant Mountain"* — no `" or "`, no `", "`, no controller clause — and still cannot be
   declared as any bare `EnchantTarget` variant. A substring axis would have pinned six and
   called the population measured; keying on the parsed mechanism found the seventh. Both axes
   are now pinned: `NEEDS_FILTER_DEFS` (7, needs an `EnchantFilter`) and
   `NEEDS_OR_OR_CONTROLLER_DEFS` (6, the narrower one the seed rows are about).

4. **`curse_of_opulence`'s in-source `// TODO` is false and outlived the commit that falsified
   it.** `curse_of_opulence.rs:20` says *"'Enchant player' not in EnchantTarget enum"*;
   `EnchantTarget::Player` is declared at `card-types/src/state/types.rs`, and the def's own
   `Completeness::inert` note three lines below says so correctly. `OOS-DX47-6`'s shape. **Not
   fixed here** — this task must not edit card defs — but the correct sentence is now recorded
   in `r1`'s allowlist reason, which `r1` checks.

5. **`t6b`'s first draft failed for the wrong reason.** The victim was
   `ObjectSpec::card(..).with_types([Creature])` — a creature with **no toughness**, which
   CR 704.5f destroys on the same sweep, so the control reported "the Aura fell off a creature"
   when what had happened was that the creature died first. A control that fails for the wrong
   reason is worse than no control; the fixture now takes a whole `ObjectSpec` and the incident
   is recorded in `attached_board`'s own doc.

6. **CR 400.7 kills the naive graveyard assertion.** `t6`'s first draft looked up the Aura's
   battlefield `ObjectId` after the SBA; the detached Aura is a **new** object, so the lookup
   panicked. It now asserts on the destination ZONE by name. Worth flagging because the
   `AuraFellOff` event carries the OLD id, so an event-only assertion never notices.

## Green re-run after the last restore

```
$ cargo test -p mtg-engine --test primitives -- pb_dx20b
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1297 filtered out

$ cargo test -p mtg-engine --test core -- pb_dx20b
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 668 filtered out
```
