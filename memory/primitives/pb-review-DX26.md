# Primitive Batch Review: PB-DX26 — the equip surface, one link earlier (21 equipment defs)

**Date**: 2026-08-11
**Reviewer**: primitive-impl-reviewer (Opus)
**Task / branch**: `scutemob-206` / `feat/pb-dx26-the-equip-surface-one-link-earlier-21-equipment-defs`
**Seeds claimed closed**: `OOS-CARDS1-3`, `OOS-CARDS1-1`, `OOS-DX3b-1`
**CR rules**: 702.6a / 702.6b / 702.6c / 702.6d (Equip), 702.67a / 702.67b (Fortify),
702.151a (Reconfigure), 601.2c, 603.3d, 111.1, 400.7
**Engine source files reviewed**: none changed (verified — see "Scope" below)
**Test files reviewed**: `crates/engine/tests/core/pb_dx26_attach_keyword_roster.rs` (new, R1–R6),
`crates/engine/tests/primitives/pb_dx26_equip_surface.rs` (new, T1–T9),
`crates/engine/tests/core/cards1_equip_target_roster.rs` (re-pinned 17→38, walk made recursive),
`crates/engine/tests/primitives/cards1_equip_target_repair.rs` (t7b strengthened, walk made recursive),
`crates/engine/tests/core/cards2_printed_field_fidelity.rs` (R7 extended to Equip/Fortify),
`crates/engine/tests/core/completeness_deviation_scan.rs` (floor 670→669),
`crates/simulator/tests/pb_dx32_fuzz_output.rs` (`CORPUS_COMPLETE` 1133→1134),
`tools/play-server/src/main.rs` (`UI3_SPLIT_COMBAT_SEED` 21→28)
**Card defs reviewed**: all 23 — the 21 equip defs, plus `darksteel_garrison` and
`guardian_project`; plus `quietus_spike` and `sting_the_glinting_dagger` (the R4 residual) and
`cryptic_coat` (census cross-check) = 26 defs read in full.

---

## Verdict: needs-fix

The core of this batch is right, and I verified the parts that matter most **by an independent
method rather than by re-reading the batch's own evidence**. The census numbers (21 markers / 10
deck-legal `Complete` / 38 post-fix attach roster / 42 Equipment defs / exactly 2 residuals) are
**correct** — I re-derived them from the printed-oracle axis, a third method the batch did not use,
and got 38 exactly. **There is no third def** beyond `quietus_spike` and
`sting_the_glinting_dagger`. All 21 authored equip costs plus the fortify cost are MCP-correct,
including the four the brief singled out. The `sword_of_body_and_mind` flip is justified clause by
clause. The three re-dealt fixtures are honest re-measurements, not numbers bumped until green
(and `COMMANDER_POOL` correctly did **not** move, which is the tell). `t3`'s index pin is the right
shape, and `umezawas_jitte` really is the only one of the 21 with a pre-existing `Activated`
ability — I checked all 21.

What is wrong is concentrated in three places. **(1) The recursive walk and its R6 gate overclaim:
an eleventh `Effect` nesting site already exists today** (`Effect::RollDice`), invisible to both the
walk and the count, while R6's stated residual names a form (`Option<Box<Effect>>`) that is
actually *visible*. **(2) The batch's flagship new gate — extending SR-37's R7 to check equip costs
— was added with its non-vacuity floor left at 9 while it now performs ~46 comparisons, and with no
revert row proving it discriminates**; all 38 equip costs it was written to protect can silently
stop being compared and R7 stays green. **(3) The "blocker notes are dated claims" discipline the
batch applied to `completeness` fields was not applied to the file-header/inline `TODO` comments in
the same files** — six defs now contain a TODO that their own completeness note, thirty lines
below, declares false, and `OOS-DX26-1` plus the workstream handoff both state that `sting`'s
header "was corrected in place" when it demonstrably was not. Separately, two `Complete` deck-legal
defs inside the batch's own R3 pin produce wrong game state for reasons the batch did not look for.

---

## Engine Change Findings

**None — and the "0 engine-source lines" claim is accurate as stated but incomplete as scoped.**
`crates/engine/src`, `crates/card-types/src`, `crates/view-model/src` and `crates/simulator/src`
carry no change. `tools/play-server/src/main.rs` **was** changed (one constant, `UI3_SPLIT_COMBAT_SEED`
21 → 28, with a re-measured derivation in its doc). That is legitimate and honestly documented in
the handoff item 5, but the headline scope sentence in `CLAUDE.md:139-141` and
`workstream-state.md:25-27` lists four crate paths and omits `tools/`, so a reader takes "0 engine
source lines" as "nothing outside card defs and tests moved". Fold `tools/` into the enumerated
scope line. (LOW — recorded in the table below as L11.)

---

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `crates/card-defs/src/defs/sword_of_light_and_shadow.rs:73-76` | **A `Complete`, deck-legal def declares a MANDATORY target for a printed "up to one target", so with an empty graveyard the whole trigger — including "you gain 3 life" — is lost.** `UpToN` exists and is used by this batch's own roster-mate. **Fix:** wrap the requirement in `TargetRequirement::UpToN { count: 1, inner: Box::new(TargetCardInYourGraveyard(..)) }`, or demote with an oracle citation. |
| 2 | **MEDIUM** | `crates/engine/tests/core/pb_dx26_attach_keyword_roster.rs:36-38, 72-91, 443-494` | **The recursive walk misses an existing eleventh nesting site and R6 cannot see it; R6's stated residual names the wrong hole.** `Effect::RollDice { results: Vec<(u32, u32, Effect)> }` is inside the enum body and matches neither `Box<Effect>` nor `Vec<Effect>`. **Fix:** add a `RollDice` arm to all three walkers, extend R6's count, correct the doc and `OOS-DX26-5`. |
| 3 | **MEDIUM** | `crates/engine/tests/core/cards2_printed_field_fidelity.rs:849-862` | **The batch's flagship gate extension is unprotected against silent vacuity and unproven by revert.** `compared >= 9` was measured for five keywords; the batch added ~37 comparisons and did not raise it. **Fix:** re-measure and raise the floor, add per-keyword floors for `Equip`/`Fortify`, and execute a revert row. |
| 4 | **MEDIUM** | 6 card defs (see detail) | **Six defs carry a file-header/inline `TODO` that their own newly-written `completeness` note contradicts, and `OOS-DX26-1` + the handoff falsely claim `sting`'s was "corrected in place".** **Fix:** delete/rewrite each TODO; correct both false claims. |
| 5 | **MEDIUM** | `crates/engine/tests/primitives/pb_os10_singleton_cleanup.rs:424-427` | **A constant's doc comment is now a lie, in the file `t3`'s own failure message cites.** "Jitte's modal ability is its only entry in `activated_abilities` (Equip is not wired through the indexed activated-ability path)" — after this batch there are two and Equip is index 1. **Fix:** rewrite the comment. |
| 6 | **MEDIUM** | `crates/engine/tests/core/cards1_equip_target_roster.rs:70-101` + `cards2_printed_field_fidelity.rs:768` | **Every shape/cost gate walks `def.abilities` only, while the new census walks both faces — a back-face Equipment with `targets: vec![]` passes all six gates.** **Fix:** make `roster_r1`, `equip_targets_for` and `def_ability_cost` chain `def.back_face`, or state the front-face-only scope as an asserted residual. |
| 7 | **MEDIUM** | `crates/card-defs/src/defs/the_reaver_cleaver.rs:46-60` | **A `Complete`-by-derive def under-fires vs. its printed card**: `WhenEquippedCreatureDealsCombatDamageToPlayer` for a printed "…to a player **or planeswalker**". **Fix:** demote to `partial` with the oracle citation and a named blocker (neither existing `TriggerCondition` variant is exact), or add the exact variant. |
| 8 | LOW | `crates/engine/tests/core/cards2_printed_field_fidelity.rs:701-712, 823-825` | **`EQUIP_VARIANT_COST_DEFS` is an unasserted excusal list** — the opposite of the standard the batch applied at R4 in the same batch. **Fix:** assert each member still prints a CR 702.6c variant line ahead of the plain one, so the entry expires with its reason. |
| 9 | LOW | `crates/engine/tests/primitives/cards1_equip_target_repair.rs:702, 758`; `cards2_printed_field_fidelity.rs:783, 791` | **Four flat `matches!` attach checks survive the batch that made three others recursive** — the §2.7 hazard, left in place at a fourth and fifth site. `def_ability_cost`'s is the dangerous one: it returns `None` and the caller `continue`s **silently**. **Fix:** use a recursive finder in `def_ability_cost`; state the fortify sites' loud-failure reasoning in-source. |
| 10 | LOW | `crates/engine/tests/primitives/pb_dx26_equip_surface.rs:539, 548, 586, 599` | **`t7`/`t8` hardcode `ability_index = 0`** instead of locating the `AttachFortification` ability the way `equip_ability_index` does — inconsistent with the batch's own `OOS-DX26-3`. **Fix:** add a `fortify_ability_index` helper. |
| 11 | LOW | `CLAUDE.md:139-141`, `memory/workstream-state.md:25-27` | **The "0 engine-source lines" scope line omits `tools/`,** which did change. **Fix:** enumerate `tools/` and state its `+N -M`. |
| 12 | LOW | `crates/engine/tests/primitives/pb_dx26_equip_surface.rs` (T1–T8) | **No behavioural probe covers a non-generic or zero equip cost.** `glimmer_lens` `{1}{W}` and `umbral_mantle` `{0}` are covered only by the static (unreverted) R7 comparison. **Fix:** add one activation probe for each. |
| 13 | LOW | `crates/engine/tests/core/cards1_equip_target_roster.rs:183-185, 220-223` | **Two doc blocks stale after the re-pin**: `r3`'s doc claims it checks `mana_cost` (it does not) and R2's doc still says "any of these 17 cards (all 17 were MCP-verified)" after the pin moved to 38. **Fix:** update both. |
| 14 | LOW | `memory/primitives/pb-DX26-fail-before-2026-08-11.md:216-223, 267-289` and the `OOS-CARDS1-3` closure row | **V6a (a positive control that MUST be green) carries the same alarm banner as V4b (a genuine non-discriminator), and the published count "13 of 14 rows red" drops V6a from the denominator of 15.** **Fix:** relabel V6a `CONTROL — MUST BE GREEN`; restate as 13 red / 1 control / 1 undiscriminated of 15. |
| 15 | LOW | `memory/primitives/pb-DX26-equip-spec.md:94`, `memory/workstream-state.md:150-152`, `crates/card-defs/src/defs/umbral_mantle.rs:7,27,56` | **Two wording defects in an otherwise-correct blocker claim**: "appear ONLY in this def's own TODO comment" is false (both strings also live in `pb-plan-S.md`, `marker-sweep-2026-07-16.md` and the audit registry), and the named type `ActivationCost` does not exist — it is `Cost`. The substantive claim (the enums lack the field) I verified and it holds. **Fix:** reword both. |
| 16 | LOW | `crates/card-defs/src/defs/umbral_mantle.rs:29-30` | **A misleading cross-reference**: "see `bone_saw.rs` for another Equip {0} card" — Bone Saw's *equip* is `{1}`; its *mana cost* is `{0}`. **Fix:** say "another {0}-mana-cost artifact", or drop the reference. |
| 17 | LOW | `crates/card-defs/src/defs/darksteel_garrison.rs:80-83` | **The completeness note is truncated mid-sentence** ("…target…"). Pre-existing, but the batch rewrote the def around it. **Fix:** complete the sentence. |
| 18 | LOW | `crates/card-defs/src/defs/mask_of_memory.rs:68-70` | **The only repaired def whose note was not given the "Equip {N} is now authored" line** that all ten of its siblings received. **Fix:** append it, for uniformity of the worklist `OOS-DX26-6` builds on. |

---

## Finding Details

### Finding 1 — `sword_of_light_and_shadow` loses its life gain (HIGH)

**File**: `crates/card-defs/src/defs/sword_of_light_and_shadow.rs:73-76`
**Oracle (MCP, verified this review)**: "Whenever equipped creature deals combat damage to a
player, you gain 3 life and **you may return up to one target** creature card from your graveyard
to your hand."
**CR**: 601.2c (target requirements) / 603.3d (a trigger with no legal target is removed from the
stack).

The def declares:

```rust
targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
    has_card_type: Some(CardType::Creature),
    ..Default::default()
})],
```

— a **mandatory** single target for a printed *"up to one target"*. Two consequences, both wrong
game state on a def that is `Complete` by the `#[default]` derive and therefore deck-legal today:

1. With no creature card in the controller's graveyard, the trigger has no legal target and is
   removed under CR 603.3d, so **"you gain 3 life" never happens** — the printed card gains the
   life unconditionally.
2. The return becomes mandatory rather than "you may".

`TargetRequirement::UpToN` exists and this batch's own roster-mate `sword_of_sinew_and_steel`
(`:70-79`) uses it twice for exactly the "up to one target" shape. So this is expressible today and
is not a DSL gap.

I raise this as HIGH rather than "out of scope" because the batch modified this def, and its new
`pb_dx26_attach_keyword_roster::r3` pin now positively asserts the def's `Complete` marker as a
deliberate, reviewed fact ("A completeness FLIP is a deliberate act"). R3 makes the marker a
checked claim; the claim is wrong.

**Fix**: change the requirement to
`TargetRequirement::UpToN { count: 1, inner: Box::new(TargetRequirement::TargetCardInYourGraveyard(..)) }`
mirroring `sword_of_sinew_and_steel:71-78`, and pin it with a probe that fires the trigger with an
empty graveyard and asserts the 3 life still arrives. If the "you may" half is judged
inexpressible, demote to `partial` with the oracle citation instead of leaving `Complete`.

---

### Finding 2 — the walk and R6 both miss an existing eleventh nesting site (MEDIUM)

**Files**: `crates/engine/tests/core/pb_dx26_attach_keyword_roster.rs:36-38` (claim),
`:72-91` (`contains_attach`), `:443-494` (R6);
mirrored in `cards1_equip_target_roster.rs:46-65` and
`cards1_equip_target_repair.rs:586-605`.

The module doc states:

> `contains_attach` walks all ten `Box<Effect>` / `Vec<Effect>` nesting sites in the `Effect` enum,
> and **R6 fails if an eleventh is ever added**, so the walk cannot silently go shallow.

I read the `Effect` enum body (`crates/card-types/src/cards/card_definition.rs:1365-2532`) directly.
The (8, 2) pin is exactly right *for the two forms it counts* — I confirmed all ten by line:
1731/1732 `Conditional`, 1740 `Repeat`, 1746 `ForEach`, 1784 `MayPayOrElse`, 1795 `MayPayThenEffect`,
1998/2000 `CoinFlip` (Box); 1765 `Choose`, 1768 `Sequence` (Vec). But there is an **eleventh nesting
site already in the enum**:

```rust
// card_definition.rs:2012-2017
RollDice {
    sides: u32,
    /// Mapping from result ranges to effects. Evaluated in order; first match wins.
    results: Vec<(u32, u32, Effect)>,
},
```

`contains_attach` falls through to `_ => false` on it, and R6's substring count of `"Box<Effect>"`
/ `"Vec<Effect>"` cannot see `Vec<(u32, u32, Effect)>`. So an attach nested inside a `RollDice`
branch drops out of every census in this batch **silently** — precisely the `seed-rerank-2026-08-02.md`
§2.7 failure mode the batch exists to close, surviving inside the fix.

Second half of the same finding: the stated residual in R6's doc (`:437-442`) and in `OOS-DX26-5`
names `Option<Box<Effect>>` as a form the gate "cannot see". That is **backwards** —
`Option<Box<Effect>>` *contains* the substring `Box<Effect>`, so it would move the count from 8 to 9
and R6 would fire. (`AbilityDefinition::Sunburst`-adjacent `on_cast_effect: Option<Box<Effect>>` at
`:1106` is outside the enum body, which is why the current count is unaffected.) The residual
therefore documents a hole that does not exist while missing the one that does.

Practical blast radius today: **zero** — no Equipment attaches from a dice roll. The finding is
about the gate's advertised reach, which is the batch's own stated subject.

**Fix**: (a) add
`Effect::RollDice { results, .. } => results.iter().any(|(_, _, e)| contains_attach(e, kind)),`
(and the analogous arms in `reaches_attach_equipment` and `find_attach_equipment_target`);
(b) extend R6 to count a third form — `code_only.matches("u32, Effect)").count()` or, better,
count occurrences of `Effect)` / `Effect>` and pin the triple; (c) rewrite the R6 doc and
`OOS-DX26-5` to name `Vec<(_, _, Effect)>` (a tuple inside a `Vec`) as the real invisible form and
drop the false `Option<Box<Effect>>` example.

---

### Finding 3 — the new R7 equip-cost gate can go vacuous and was never proven red (MEDIUM)

**File**: `crates/engine/tests/core/cards2_printed_field_fidelity.rs:849-862` (floor and comment),
`:691-699` (`ABILITY_COST_KEYWORDS`), `:774-797` (the new `Equip`/`Fortify` arms).

This is the strongest thing the batch added — its own comment says it well:

> Before this, **38 authored equip costs and 1 fortify cost were checked by nothing** … A def
> charging Equip {1} for a printed Equip {3} sailed past every gate.

Two problems with how it landed.

**(a) The non-vacuity floor was not raised.** It still reads:

```rust
// MEASURED, not guessed: 9 definitions declare one of these five variants today
// (Bestow 1, Morph 4, Megamorph 2, Disguise 1, Craft 1). …
assert!(compared >= 9, "R7 compared only {compared} ability costs — the extraction has stopped
        matching, which would make this rule silently vacuous");
```

`ABILITY_COST_KEYWORDS` now has **seven** entries, not five, and `compared` is now approximately
9 + 36 + 1 = 46. If the whole-word guard at `:724-742` regresses, or `def_ability_cost`'s new
`Equip` arm stops matching (see Finding 9 — it is a flat `matches!`), `compared` falls back to 9
and **the assertion the batch wrote this extension to satisfy stays green with all 38 equip costs
unchecked**. The comment also still says "these five variants", which is now false in the same
breath as claiming the number was "MEASURED, not guessed".

**(b) No revert row proves R7 discriminates for Equip or Fortify.** The executed matrix
(`pb-DX26-fail-before-2026-08-11.md` §2, rows V1–V8 + V1b/V1c/V2b/V2c/V4b/V4c/V6a/V6b) contains no
row touching `cards2_printed_field_fidelity`. Acceptance criterion 5 requires every new gate to be
proven red by an executed revert; this one was not. I did verify the *outputs* independently — all
21 authored equip costs and the fortify cost match MCP, including `glimmer_lens {1}{W}`,
`umbral_mantle {0}`, `blackblade_reforged {7}`, `commanders_plate {5}` — so the gate is currently
telling the truth. What is unproven is that it would *notice* if it stopped.

**Fix**: re-measure `compared` and pin it at the measured value with a dated derivation comment
(and correct "five variants" → "seven"); add separate per-keyword floors so a regression confined to
`Equip` cannot hide behind the Morph/Craft comparisons; and execute a revert row (set
`bone_saw.rs`'s equip cost to `generic: 2` and watch R7 name the card, the printed value and the
def value) and record it in the fail-before doc.

---

### Finding 4 — six defs carry a TODO their own note declares false, and two documents claim otherwise (MEDIUM)

The batch applied `OOS-DX3-1`'s "a blocker note is a dated claim" discipline rigorously to the
`completeness` field — every one of the eleven non-`Complete` notes was re-verified and rewritten,
and I confirmed the substance of each against the live enums (see "Blocker-claim spot-checks"
below). It was **not** applied to the file-header and inline `TODO` comments in the same files,
which are what an author reads first. Six defs now contradict themselves:

| def | stale TODO | contradicted by | ground truth |
|---|---|---|---|
| `sting_the_glinting_dagger.rs:8-9` | "Equip {2} is a keyword but Equipment Equip activated ability requires target-creature activated ability which is **also a DSL gap**" | its own `Completeness::inert` note at `:30-36` ("Three of four clauses are now expressible … **Equip {2}**") | 21 defs in this very batch prove it is expressible |
| `glimmer_lens.rs:7` and `:26` | "'For Mirrodin!' — ETB token + auto-attach **not expressible**" | its note at `:59-60` ("'For Mirrodin!' is **NOT** blocked — `Effect::CreateTokenAndAttachSource` … expresses it") | `Effect::CreateTokenAndAttachSource` exists — `card_definition.rs:1922`, executed at `effects/mod.rs:1532` |
| `blackblade_reforged.rs:24-25` | "DSL gap — dynamic +1/+1 per land you control. `LayerModification` needs `EffectAmount`, not fixed i32" | its note at `:57-59` ("now expressible … `ModifyBothDynamic` + `EffectAmount::PermanentCount`") | both exist — `effects/mod.rs:4101` and `card_definition.rs:2683` |
| `empyrial_plate.rs:18-19` | "DSL gap — dynamic +1/+1 per card in hand … Needs dynamic `LayerModification`" | its note at `:48-51` (gives the exact `ModifyBothDynamic` + `EffectAmount::HandSize` rewire) | `EffectAmount::HandSize` exists — `card_definition.rs:2925` |
| `blade_of_the_bloodchief.rs:21-24` | "`EffectTarget::EquippedCreature` **does not exist**" | its note at `:53-56` ("`EffectTarget::EquippedCreature` and `WheneverCreatureDies` **both exist**") | exists — `card_definition.rs:2604` |
| `quietus_spike.rs:5-6, 23-25` | "TODO: Equip {3} activated ability"; "deathtouch requires equipment continuous effect" | its note at `:26-32` ("Deathtouch grant … and Equip {3} **are expressible today**") | both expressible; only `EffectAmount` half-rounded-up is missing |

`sword_of_body_and_mind.rs` shows the correct treatment — the stale header TODO was deleted, which
is exactly what the spec's rule 6 asked for. The other six did not get it.

**And two documents state that one of them did.** `docs/audits/decision-point-audit.md:1379`
(`OOS-DX26-1`) says *"Sting's stale header claim that the equip ability 'is also a DSL gap' was
corrected in place"*, and `memory/workstream-state.md:152-156` repeats it under the heading
"Confirmed **stale and corrected in place**". Lines 8-9 of `sting_the_glinting_dagger.rs` still
carry the claim verbatim. This is the batch's own subject matter — a dated claim that outlived its
truth — recurring inside the batch's own closure prose. `glimmer_lens`'s entry on that same list is
half-true: the "Equip {1}{W} cost is also not modeled" clause was struck from the note, but the two
`For Mirrodin!` TODOs the batch itself proved false were left standing.

`OOS-DX26-6` (`:1384`) already names three of these six ("Three of these name a blocker their own
note calls expressible today"). It is four, not three — `blade_of_the_bloodchief` belongs on the
list — and the header TODOs are the artefact that will actually mislead the next author.

**Fix**: delete or rewrite all six TODO blocks so each states only what is still blocked; add
`blade_of_the_bloodchief` to `OOS-DX26-6`'s "expressible today" list; and correct the "corrected in
place" claim in `OOS-DX26-1` and `workstream-state.md` — either by doing the correction or by
striking the claim.

---

### Finding 5 — `JITTE_MODAL_ABILITY_INDEX`'s doc is now false (MEDIUM)

**File**: `crates/engine/tests/primitives/pb_os10_singleton_cleanup.rs:424-427`

```rust
/// Jitte's modal ability is its only entry in `activated_abilities` (Equip is not
/// wired through the indexed activated-ability path; the counters trigger is a
/// `TriggeredAbilityDef`, also not indexed there).
const JITTE_MODAL_ABILITY_INDEX: usize = 0;
```

Both parenthetical claims are now wrong: after this batch Jitte has **two** entries in
`activated_abilities`, and Equip *is* wired through the indexed path — that is the whole point of
`pb_dx26_equip_surface::t3`, whose failure message cites this very file by name (`:342-343`). The
constant's *value* is still correct, but its justification is the reason a future reader would
believe reordering is safe. `memory/conventions.md`'s "aspirationally-wrong code comments are
correctness hazards" rule applies directly, and `OOS-DX26-3` is filed about exactly this class.

**Fix**: rewrite to state that Jitte now exposes two activated abilities, that the modal one is
index 0 **because PB-DX26 appended equip at index 1**, and cross-reference
`pb_dx26_equip_surface::t3` as the gate.

---

### Finding 6 — the shape and cost gates are front-face-only while the census is both-faces (MEDIUM)

**Files**: `cards1_equip_target_roster.rs:70-101` (`roster_r1`, `equip_targets_for`);
`cards2_printed_field_fidelity.rs:768` (`def_ability_cost`);
`cards1_equip_target_repair.rs:562, 699-717, 752`.
Contrast `pb_dx26_attach_keyword_roster.rs:108-141`, which deliberately chains
`def.back_face.iter().map(|f| &f.abilities)` in all four of its helpers.

Trace a hypothetical new DFC Equipment whose equip ability lives on the **back face** with
`targets: vec![]` (the CARDS-1 defect):

* `pb_dx26_attach_keyword_roster::r4` — `has_subtype` sees the back face, `has_activated_attach`
  sees the back face and finds the attach → **not a violation** → green.
* `r1`/`r2` — only fire if the def carries the `Equip` marker; if it does not (as CARDS-1's 17 do
  not), → green.
* `cards1_equip_target_roster::r1` — `roster_r1` walks `def.abilities` only → the def is absent from
  the measured set, and absent from the pinned set → **equality holds** → green.
* `r2` (the shape gate, "the gate that makes OOS-M11-10 unable to recur") — iterates the R1 roster,
  which does not contain it → **never examines it** → green.
* `cards2_printed_field_fidelity::r7` — `def_ability_cost` walks `def.abilities` only → `None` →
  `continue` → green.

So the exact defect this whole file family exists to prevent passes six gates. There are no DFC
Equipment in the corpus today, so this is latent, not live. But R4 was explicitly widened to both
faces and the others were not, which means the asymmetry is an oversight rather than a decision.

**Fix**: chain `def.back_face` in `roster_r1`, `equip_targets_for`, `def_ability_cost` and
`equip_activated_attach_equipment_roster` (a one-line `std::iter::once(&def.abilities).chain(..)`,
copied from the new file), or state front-face-only scope as an **asserted** residual (a gate that
fails if any `Equipment`-subtyped def ever gains a `back_face`).

---

### Finding 7 — `the_reaver_cleaver` under-fires vs. its printed card (MEDIUM)

**File**: `crates/card-defs/src/defs/the_reaver_cleaver.rs:46-60`
**Oracle (MCP)**: "Equipped creature … has 'Whenever this creature deals combat damage to a player
**or planeswalker**, create that many Treasure tokens.'"

The def uses `TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer`, which by its own
enum doc (`card_definition.rs:3512-3518`) "fires only on damage to a player". The sibling
`WhenEquippedCreatureDealsCombatDamage` is *any recipient* (creatures included) and would
over-fire, so **neither variant is exact** — this is a genuine gap. The def is nevertheless
`Complete` by the `#[default]` derive with no marker at all, i.e. nobody has ever decided it.

Same class as Finding 1: `r3`'s new exact pin now positively blesses this marker, and the batch's
handoff item 1 explicitly reasoned "the ten deck-legal defs were *already* `Complete`, so repairing
them flips nothing" — which assumes the marker was right rather than checking it. (The narrower
in-def note also claims the trigger is on the *equipment*, where the printed card grants it to the
*creature*; that distinction matters only if the equipment leaves mid-combat, and I would not fix
it here.)

**Fix**: demote `the_reaver_cleaver` to `Completeness::partial` naming the planeswalker half and the
absent exact `TriggerCondition` variant, and re-pin `r3` accordingly — or add
`WhenEquippedCreatureDealsCombatDamageToPlayerOrPlaneswalker` in a follow-up batch and keep the
marker. Do not leave a `Complete` marker on a known under-fire.

---

## What I verified independently and found correct

Spending length on what is wrong is the instruction, so these are compressed — but each was
re-derived, not read off the batch's evidence.

**The census is right, by a third method.** The batch used a grep census (the seed) and an
`all_cards()` census (R1/R2) and a type-line census (R4/R5). I ran a fourth axis — the **printed
oracle text stored in the defs** — and it reconciles exactly: 42 def files carry an
`Equip {` / `Fortify {` / `Reconfigure {` literal; minus `darksteel_garrison` (Fortify) and
`lizard_blades` (Reconfigure) leaves **40** defs printing an Equip line; minus `quietus_spike` and
`sting_the_glinting_dagger` (no ability) leaves **38** — exactly `cards1_equip_target_roster` R1's
re-pin. `cryptic_coat` is the 42nd Equipment and MCP confirms it genuinely prints no Equip line.
**There is no third def beyond the two the inverse census found.** 21 markers, 10 deck-legal
`Complete` (nine with no `completeness` field at all — I confirmed each file), 38 post-fix: all
correct.

**All 21 equip costs + the fortify cost are oracle-correct.** MCP-verified per def, independently
of the spec table, with particular attention to the four the brief named: `glimmer_lens {1}{W}`
(authored as `generic: 1, white: 1` — correct, and it is the only non-generic one),
`umbral_mantle {0}` (`ManaCost::default()` — correct), `blackblade_reforged {7}` (correct; the
`{3}` legendary variant is deliberately unmodelled and allowlisted in R7), `commanders_plate {5}`
(same). Every one carries `TimingRestriction::SorcerySpeed` (CR 702.6d) and the identical
`TargetCreatureWithFilter { controller: You, ..default }` (CR 702.6a) with the marker retained
beside it, and none of the 21 prints a CR 702.6c quality restriction on its plain line.

**The `sword_of_body_and_mind` flip is justified clause by clause.** Printed: +2/+2 → `ModifyBoth(2)`
on `AttachedCreature`; protection from green and blue → two `AddKeyword(ProtectionFrom(FromColor(..)))`
statics; the combat-damage trigger → `WhenEquippedCreatureDealsCombatDamageToPlayer` with a 2/2
green Wolf `CreateToken` and `MillCards { player: DamagedPlayer, count: 10 }`; Equip {2} → now
authored. Nothing printed is missing. The `partial` note really did name Equip {2} as the sole
blocker, and the stale header TODO was deleted as the spec required.

**The three re-dealt fixtures are honest re-measurements.** `completeness_deviation_scan`'s floor
670 → 669 carries a dated derivation naming the flipped card and states "RE-MEASURED DIRECTLY, not
derived: `all_cards()` reports 1,134 Complete / 669 non-Complete of 1,803"; `CORPUS_COMPLETE`
1133 → 1134 says "Re-measured by executing this gate, not predicted". The tell that these are real
rather than bumped-until-green is `COMMANDER_POOL`, which correctly stayed at **90** — the flipped
card is not a Legendary Creature, so `deck.rs:40-47`'s three-clause filter cannot see it. And
`UI3_SPLIT_COMBAT_SEED` 21 → 28 records a full sweep of 0..40 (9/26/28/29/30 split the attack; only
28 and 29 also reach a declared blocker) and explicitly names a *second* filter the original
constant's own doc never mentioned. That is a better re-observation than the one it replaces.

**`t3` is the right shape, and the index-hazard claim is exact.** Asserting `descriptions.len() == 2`
plus `equip_idx == 1` forces the modal ability to index 0 by elimination, so no separate identity
assertion is needed. I read all 21 defs: `umezawas_jitte` really is the only one with a pre-existing
`AbilityDefinition::Activated`, so no other def in the batch acquired a renumbering. (Note for the
future: `sword_of_the_paruns`'s unauthored "{3}: You may tap or untap equipped creature" will be the
next member of this class.)

**R6's (8, 2) pin is arithmetically exact for the two forms it counts** — I enumerated the enum body
line by line rather than trusting the printout, and the bracket-matcher's range
(`pub enum Effect {` at 1365 to the closing brace at 2532) is correct. Its comment-stripping is
genuinely load-bearing, per PB-DX32's `OOS-DX32-6`. The only defect is the eleventh site (Finding 2).

**`t4`'s undiscriminated status is handled correctly.** V4b really is shadowed by `OOS-DX20-7`'s
legacy `Effect::AttachEquipment` guard, the doc comment says so in the test itself rather than only
in a memo, and the compensating coverage (T1/V4c on the offer side, `r2`/V4 on the shape side) is
real — I checked both go red under the same reversion by reading what each asserts. This is the
right way to handle a shadowed probe.

**Blocker-claim spot-checks (criterion: "spot-check at least three").** I checked five against the
live enums, not the notes:
`TriggerCondition::WhenEquippedCreatureAttacks` — **absent**, confirmed; the enum has only
`WhenEquippedCreatureDealsCombatDamageToPlayer` (`:3512`) and `WhenEquippedCreatureDealsCombatDamage`
(`:3518`). `requires_untap_self` / a `{Q}` cost — **absent** from `Cost` (`:1240-1292`), confirmed
(the note's type name is wrong; see L15). `Condition::EquippedCreatureIsTapped` and
`EffectFilter::TappedCreaturesYouControl` — **absent**, confirmed; `EffectFilter`
(`state/continuous_effect.rs:81`) has 30-odd creature-scoping variants and no tapped/untapped one.
`EffectTarget::EquippedCreature` — **present** (`:2604`), so `blade_of_the_bloodchief`'s *new note*
is right and its *old TODO* is wrong (Finding 4). `Effect::CreateTokenAndAttachSource` — **present**
(`:1922`), so `glimmer_lens`'s new note is right and its TODOs are wrong (Finding 4). No blocker
claim in the `completeness` fields is substantively wrong.

**`darksteel_garrison` gets CR 702.67a right, not the equip repair.**
`TargetPermanentWithFilter(has_card_type: Land + controller: You)`, with an in-def comment stating
why `TargetCreatureWithFilter` would be wrong, and `t7` asserts the offer excludes both an
opponent's land **and** the controller's own creature. `t7b` was genuinely strengthened from a
name-set pin to a requirement-shape pin with a `checked == 1` non-vacuity floor. Revert row V2c
executes the copy-paste mistake and watches it fail. This rider is clean.

**`guardian_project` is correct and `t9` is non-vacuous in both directions.** `is_nontoken: true`
on the trigger's `TargetFilter`, honoured by the `creature_filter.is_nontoken && entering_obj.is_token`
pre-check; the def correctly stays `known_wrong` (CR-wise, "doesn't have the same name as another
creature you control **or a creature card in your graveyard**" — the def's stored oracle text
matches MCP exactly); `t9` casts a real nontoken creature whose own ETB makes a token, so both the
fire and the no-fire halves ride one flow with library count as the observable, and V3 reddens the
no-fire half. The header TODO here **was** properly rewritten.

**Registration and hygiene.** Both new test files are wired into their group `main.rs`
(`core/main.rs:36`, `primitives/main.rs:41`) — SR-9a satisfied. The queue-memo row
(`seed-rerank-2026-08-02.md:726`) is struck with an accurate yield correction. The six `OOS-DX26-*`
rows exist in the registry and are well-argued; `OOS-DX26-1`'s only defect is the false
"corrected in place" clause (Finding 4).

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 702.6a — "attach to target creature you control" | Yes (21 defs) | Yes | `t1` (offer side), `cards1_equip_target_roster::r2` (shape); `t4` shadowed, disclosed |
| 702.6b — equip is an activated ability | Yes | Yes | `t1`/`t2`/`t3` activate through `Command::ActivateAbility`; `r2` pins existence |
| 702.6c — variant "equip [quality] {N}" | **No — deliberate** | n/a | `blackblade_reforged`, `commanders_plate`; filed `OOS-DX26-2`, allowlisted in R7 (see L8) |
| 702.6d — sorcery speed only | Yes | **No** | Every def carries `TimingRestriction::SorcerySpeed`, but no probe attempts an instant-speed equip. Gap. |
| 702.67a — "attach to target land you control" | Yes | Yes | `t7`/`t8`, `t7b` shape pin, V2/V2b/V2c |
| 702.67b — fortify is an activated ability | Yes | Yes | `t8` end-to-end |
| 702.151a — Reconfigure "another target creature" | Pre-existing (PB-DX20) | Pinned | `t7b` reconfigure roster; counted as reachable by `has_activated_attach` |
| 601.2c — zero-target activation rejected | Yes | Yes | `t5` |
| 603.3d — trigger with no legal target | n/a (not this batch) | — | **but see Finding 1** — `sword_of_light_and_shadow` is on the wrong side of it |
| 111.1 — nontoken | Yes | Yes | `t9`, both directions; V3 |
| 400.7 | n/a | — | No zone-change surface in this batch |

**One CR gap worth naming**: 702.6d is authored on all 21 defs and asserted by **no** probe. A def
that dropped `timing_restriction` would pass every gate in this batch (R2 checks existence, `r2`
checks `targets`, R7 checks `cost` — none checks timing). Cheap fix: extend
`cards1_equip_target_roster::r2` to also assert
`timing_restriction == Some(TimingRestriction::SorcerySpeed)` for every roster member, or add a
probe that attempts a Bone Saw equip during the opponent's turn. Recommended alongside the fix
cycle.

---

## Card Def Summary

| Card | Equip cost MCP-verified | Oracle match (whole def) | Stale TODOs left | Game state correct | Notes |
|---|---|---|---|---|---|
| `bone_saw` | {1} ✓ | Yes | 0 | Yes | Reference implementation; T1/T2/T4/T5/T6 all ride it |
| `kite_shield` | {3} ✓ | Yes | 0 | Yes | |
| `paradise_mantle` | {1} ✓ | Yes | 0 | Yes | R7 whole-word guard was needed for this def |
| `the_reaver_cleaver` | {3} ✓ | **No** | 0 | **No** | Finding 7 — under-fires on planeswalker damage while `Complete` |
| `sword_of_feast_and_famine` | {2} ✓ | Yes | 0 | Yes | |
| `sword_of_light_and_shadow` | {2} ✓ | **No** | 0 | **No** | **Finding 1 (HIGH)** — mandatory target for "up to one target" |
| `sword_of_sinew_and_steel` | {2} ✓ | Yes | 0 | Yes | Correct `UpToN` usage — the model Finding 1 should copy |
| `sword_of_truth_and_justice` | {2} ✓ | Approximation, documented | 0 | Documented deviation | `OOS-DX4-6` targeting axis stated in-def; acceptable |
| `sword_of_war_and_peace` | {2} ✓ | Yes | 0 | Yes | |
| `umezawas_jitte` | {2} ✓ | Yes | 0 | Yes | Equip appended last, `t3` pins it; see Finding 5 for the stale doc it left behind |
| `sword_of_body_and_mind` | {2} ✓ | Yes | 0 | Yes | **Flip `partial` → `Complete` is justified clause by clause** |
| `blade_of_the_bloodchief` | {1} ✓ | partial, honest | **1** | Yes (partial) | Finding 4 — TODO says `EffectTarget::EquippedCreature` absent; it exists |
| `blackblade_reforged` | {7} ✓ | partial, honest | **1** | Yes (partial) | Finding 4; `OOS-DX26-2` variant cost |
| `commanders_plate` | {5} ✓ | partial, honest | 0 | Yes (partial) | `OOS-DX26-2` variant cost |
| `empyrial_plate` | {2} ✓ | partial, honest | **1** | Yes (partial) | Finding 4 |
| `glimmer_lens` | {1}{W} ✓ | partial, honest | **2** | Yes (partial) | Finding 4 — the only coloured equip cost; no behavioural probe (L12) |
| `illusionists_bracers` | {3} ✓ | partial, honest | 0 | Yes (partial) | |
| `mask_of_memory` | {1} ✓ | known_wrong, honest | 0 | Documented | L18 — note not given the "equip now authored" line |
| `sword_of_the_animist` | {2} ✓ | partial, honest | 0 | Yes (partial) | Blocker re-verified by me: variant genuinely absent |
| `sword_of_the_paruns` | {3} ✓ | partial, honest | 0 | Yes (partial) | Blocker re-verified; next index-hazard candidate |
| `umbral_mantle` | {0} ✓ | partial, honest | 0 | Yes (partial) | L15/L16 wording; no behavioural probe for the {0} path (L12) |
| `darksteel_garrison` | Fortify {3} ✓ | partial, honest | 0 | Yes | L17 truncated note |
| `guardian_project` | n/a | known_wrong, honest | 0 | Yes (nontoken half now right) | Header TODO correctly rewritten — the model the other six should follow |
| `quietus_spike` | not authored (Inert) | Inert, honest | **2** | Withheld | Finding 4; R4 residual with the excusal asserted — good shape |
| `sting_the_glinting_dagger` | not authored (Inert) | Inert, honest | **3** | Withheld | Finding 4 — and `OOS-DX26-1`/handoff falsely claim this was fixed |

---

## Acceptance Criteria Verdict

| # | Criterion | Met? | Note |
|---|---|---|---|
| 1 | Pre-edit baseline + itemised delta | **Yes** | 4,491 → 4,506 (+15 = R1–R6 + T1–T9), 46 targets, residual empty |
| 2 | Roster from `all_cards()` (SR-36), 10 deck-legal repaired, all partials dispositioned one at a time | **Yes** | Re-derived by me on a fourth axis and confirmed; all 11 non-`Complete` rows dispositioned; every equip line MCP-verified |
| 3 | `darksteel_garrison` gets `TargetPermanentWithFilter(Land + You)`, t7b updated | **Yes** | And strengthened past the row's own instruction |
| 4 | `guardian_project` `is_nontoken: true`, stays non-`Complete`, blocker re-dated | **Yes** | Cleanest rider in the batch |
| 5 | Gate integrity: R1 re-pinned, recursive walk, every new gate proven red by executed revert | **Partial** | R1/walk done (Findings 2, 6, 9 qualify the reach). **The R7 extension has no revert row and no raised floor — Finding 3.** |
| 6 | Inverse-method census; new finds fixed or seeded with measurements | **Yes** | Two found, seeded with a measured 0 blast radius and an asserted excusal |
| 7 | PROTOCOL/HASH gate-executed and unmoved, clippy/fmt/defs-fmt clean, coverage regenerated, seeds filed, queue row struck, handoff appended | **Yes, with two false claims** | 35/74 unmoved; coverage 62.8→62.9 re-measured; row struck. But the "corrected in place" claim (Finding 4) and the `tools/`-omitting scope line (L11) are both inaccurate. |

---

## Previous Findings

None — this is the first review of PB-DX26.
