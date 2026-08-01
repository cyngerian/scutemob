# Primitive Batch Review: PB-DX1 — the intervening-if dropped in the runtime lowering

**Date**: 2026-07-31
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-160` · branch `feat/pb-dx1-the-intervening-if-dropped-in-the-runtime-lowering-oo`
**Merge base**: `3d73763d` · commits `00d2d5d1`, `1fbdce4a`, `bb93d0d4`, `6941fb51`, `03053182`
**CR Rules**: 603.4 (primary), 603.2, 603.2c, 603.2h, 603.10 / 603.10a, 508.1, 508.3a, 500.8/500.10a,
700.4, 702.55c, 708.8, 113.7a, 613.1f
**Engine files reviewed**: `crates/card-types/src/state/game_object.rs` (`InterveningIf`),
`crates/engine/src/rules/abilities.rs` (`check_intervening_if`, `InterveningIfMoment`, 13 queue
sites, haunt queue gate, `flush_sorted`'s once-per-turn gate),
`crates/engine/src/rules/resolution.rs` (registry re-check + harmonisation, runtime re-check,
`TurnFaceUpTrigger`, haunt), `crates/engine/src/testing/replay_harness.rs`
(`build_face_ability_vectors`, 34 push sites), `crates/engine/src/state/hash.rs`,
`crates/engine/src/rules/protocol.rs`, `crates/engine/tests/core/{hash_schema,protocol_schema,
bare_lookup_ratchet}.rs`, `crates/engine/tests/primitives/{main.rs,pb_dx1_lowered_intervening_if.rs}`
**Card defs reviewed (6)**: `aurelia_the_warleader.rs`, `karlach_fury_of_avernus.rs`,
`tatyova_steward_of_tides.rs`, `welcoming_vampire.rs`, `elvish_warmaster.rs`,
`whispering_wizard.rs` (+ `scourge_of_the_throne.rs` checked and confirmed out of scope)

## Verdict: needs-fix

**The engine change is correct and the batch is a large net improvement.** The (a′) shape —
`InterveningIf::CardDef(Box<Condition>)` — is the right call and is compiler-forced at all 14
evaluation sites; I independently re-derived the moment classification from the source that supplies
`source` at each site and **all 14 rows are right** (8 LookBack / 5 TriggerTime / 1 Resolution); the
34 push sites all carry `intervening_if` **and** `once_per_turn`, and the three surviving
`once_per_turn: false` literals really are `ActivatedAbility` structs (Reconfigure ×2 at
`replay_harness.rs:3964`/`:3981`, Outlast at `:4018`); T3 genuinely exercises the resolution end and
would fail against a queue-only fix; the wire/hash bump is self-consistent and machine-gated;
SR-7/SR-9a/SR-25 are untouched; OOS-DP6-2 was correctly left ungated.

**One HIGH.** It is not in the engine — it is `aurelia_the_warleader.rs`, the batch's own headline
card. `WhenAttacks` + `Condition::IsFirstCombatPhase` does not translate "attacks **for the first
time each turn**", and this batch is what makes that divergence behaviourally live — as a
*suppressed* trigger, the single direction of failure the brief and the plan's hard constraint 3
name. The faithful authoring (`once_per_turn: true`, no `intervening_if`) is made expressible by
**phase 7 of this same batch**, and the plan's stated reason for declining it is falsified by the
batch's own T12b. Five MEDIUM and four LOW follow, of which the largest cluster is bookkeeping the
batch itself asked for: **seeds OOS-DX1-1..5 are cited in five source comments and filed nowhere**,
and the batch's own new field-audit table already contradicts the code five lines below it.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 2 | MEDIUM | `abilities.rs:10356-10395` / `tests/primitives/pb_dx1_lowered_intervening_if.rs:482-557` | **The CR 603.10a carve-out is neutralised at the resolution end, and T6 cannot see it.** `TriggerTimeLookBack` returns `true` without evaluating, but the *same* condition is evaluated for real at `Resolution` against the current state, where the source is in the graveyard/exile/hand. A source-scoped condition on a leaves-the-battlefield trigger therefore **queues and then silently fizzles** — the plan §4.3 claim that the deviation "degrades to over-fires" is false end-to-end. T6 asserts only the `AbilityTriggered` count, never the effect. Zero corpus exposure. **Fix:** below. |
| 4 | MEDIUM | `replay_harness.rs:2398` | **The batch's own field-audit table is already wrong.** Row `once_per_turn` reads "dropped at 31/34 sites (propagated at 3) — NOT fixed by this batch, seeded OOS-DX1-6 if left as-is", five lines above 34 sites that all now write `once_per_turn: *once_per_turn`. **Fix:** below. |
| 6 | MEDIUM | `abilities.rs:10367-10394` vs `resolution.rs:2313-2328` | **The `Resolution` arm builds its context with `vec![]` targets; the registry path 80 lines away passes `stack_obj.targets.clone()`.** Benign *only* because the one `Condition` that reads `ctx.targets` is `TargetIsLegal`, which the evaluability guard short-circuits. OOS-DX1-2, implemented as written, would invert it into a guaranteed false negative. **Fix:** below. |
| 7 | LOW | `abilities.rs:4805-4818` vs `resolution.rs:5582-5588` | **Haunt: the new queue gate skips the `haunting_target` clear.** When the gate suppresses, only the resolution path clears `haunting_target`, so the exiled haunt card keeps haunting a dead creature's `ObjectId`. **Fix:** below. |
| 10 | LOW | `abilities.rs:8092-8109` | **`flush_sorted`'s once-per-turn lookup reads base, not layer-resolved, characteristics** while its comment says "layer-resolved". Pre-existing; this batch is what makes that read load-bearing for three `Complete` defs. Inert today. **Fix:** below. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | **HIGH** | `aurelia_the_warleader.rs:51-54` | **`Complete`, deck-legal, and this batch makes an oracle mismatch live as a suppressed trigger.** Oracle: "Whenever Aurelia attacks **for the first time each turn**". Def: `WhenAttacks` + `Condition::IsFirstCombatPhase` (`!turn.in_extra_combat`). If Aurelia's first attack of the turn happens in a later combat granted by another source (Aggravated Assault / Moraug / World at War / Port Razer — ordinary Commander play), the real card triggers and the def now does **not**. Pre-PB-DX1 it fired (the condition was dropped). **Fix:** below. |
| 5 | MEDIUM | `tatyova_steward_of_tides.rs:89` | **Off-by-one, newly live.** Oracle: "if you control **seven** or more lands". Def: `ControlAtLeastNOtherLands(6)`. Because `ctx.source` is Tatyova (a creature, not a land), the "other" exclusion removes nothing (`effects/mod.rs:9809-9823`), so the effective threshold is **6**. The completeness note added by this batch calls the change a "pure behavior repair" and does not record the off-by-one. Deck-blocked (`partial`). **Fix:** below. |
| 3 | MEDIUM | (bookkeeping, all files) | **Seeds OOS-DX1-1..5 are cited in `abilities.rs`, `resolution.rs`, `replay_harness.rs` and `aurelia_the_warleader.rs`, and filed nowhere.** `docs/audits/decision-point-audit.md` §8.1 has zero `OOS-DX1-*` rows, and OOS-DP6-1 (`:861`) / OOS-DP6-5 / OOS-DP6-9 are not marked closed, with the stale cites (`7369`→`7513`, `5351`→`5500`) uncorrected. **Fix:** below. |
| 8 | LOW | (bookkeeping) | `memory/primitive-wip.md` still reads `**Phase**: plan` and predicts "HASH bump if fix (a) is taken"; `memory/workstream-state.md:36` still reads "PB-DX queue not started". **Fix:** below. |
| 9 | LOW | (evidence) | Plan §13 requires the pre-fix T1 failure text and the bench numbers vs. the merge base to be recorded; neither exists anywhere in the tree. **Fix:** below. |

---

### Finding Details

#### Finding 1: Aurelia's `IsFirstCombatPhase` is a proxy, the proxy now under-fires, and the faithful authoring shipped in this same batch

**Severity**: HIGH
**File**: `crates/card-defs/src/defs/aurelia_the_warleader.rs:51-54`
**Oracle (MCP, verified)**: *"Flying, vigilance, haste / Whenever Aurelia attacks **for the first
time each turn**, untap all creatures you control. After this phase, there is an additional combat
phase."* (single ruling, 2024-11-08: no additional main phases — the def's
`followed_by_main: false` is correct)
**CR**: 603.4 (what the def uses), **603.2h** (what the card actually says)

**Issue.** The def authors "for the first time each turn" as `TriggerCondition::WhenAttacks` +
`intervening_if: Some(Condition::IsFirstCombatPhase)`, and `IsFirstCombatPhase` is
`!state.turn.in_extra_combat` (`effects/mod.rs`). Those are different predicates. They agree only
when Aurelia's first attack of the turn is in the turn's *first* combat.

Concrete failure scenario, all cards deck-legal in Commander: p1 casts Aurelia in the postcombat
main phase (or she was tapped / not yet under p1's control during combat 1). Aggravated Assault,
Moraug, World at War or Port Razer grants an extra combat. Aurelia attacks — this is her **first**
attack this turn. The printed card triggers: untap all creatures you control, plus another combat.
This def evaluates `IsFirstCombatPhase` → `in_extra_combat == true` → **false** → the queue gate at
`abilities.rs:7003-7014` drops the trigger. Nothing fires.

This is newly live. Before PB-DX1 the lowering hardcoded `intervening_if: None`, so the ability
fired unconditionally and *this* scenario produced the correct outcome by accident. The batch trades
an unbounded over-fire (correctly, and that is the whole point of the batch) for a bounded
**under-fire** — the one direction hard constraint 3 forbids and the brief singles out — on a def
with **no `completeness` field**, i.e. `Complete` and admitted by `validate_deck`.

The plan (§6.3) and the def's own new comment (`:34-50`) identify the divergence correctly and
decline to fix it for one reason: *"re-authoring would change which mechanism T1 exercises."*
**That reason is falsified by this batch's own test file.** `test_dx1_karlach_extra_combat_once_per_turn`
(`pb_dx1_lowered_intervening_if.rs:1647-1734`) drives the identical
intervening-if × attack-trigger × extra-combat shape end-to-end on a real `Complete` def, through the
real registry, with the same "no third combat" assertion. The CR 603.4 mechanism is not uniquely
carried by T1. And phase 7 of this batch is precisely what makes `once_per_turn: true` expressible
for a lowered `WhenAttacks`: `replay_harness.rs:2503` now propagates it, and the gate at
`abilities.rs:8136-8151` / `:9192-9198` keys on `(source, ability_index)` with a per-turn reset at
`layers.rs:1726-1744`. That is an exact translation of "for the first time each turn".

**Fix**: in `aurelia_the_warleader.rs`, set `once_per_turn: true` and remove
`intervening_if: Some(Condition::IsFirstCombatPhase)`. Keep the def `Complete`. Retarget T1's CR
citation from 603.4 to **CR 603.2h** (the assertions — one trigger, one extra combat, no third —
are unchanged and still pass, because the once-per-turn gate suppresses the second declaration for a
different, correct reason). T12b (Karlach) remains the CR 603.4 probe for the lowered
intervening-if. Add a second Aurelia probe pinning the newly-correct case: `in_extra_combat`
reached without Aurelia having attacked, then Aurelia attacks → the trigger **does** fire. Replace
the OOS-DX1-5 comment block with a two-line note recording that the proxy was replaced and why.
*(If, contrary to the above, the flip is declined, then Aurelia must be demoted to `partial` with
the divergence as its note — a `Complete` def may not knowingly produce wrong game state, Invariant
9 / SR-2. Do not leave it `Complete` and divergent.)*

#### Finding 2: `TriggerTimeLookBack` fails open at the queue end and fails closed at the resolution end

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:10356-10395`; test
`crates/engine/tests/primitives/pb_dx1_lowered_intervening_if.rs:482-557`
**CR Rule**: 603.10a — *"Some zone-change triggers look back in time. These are
leaves-the-battlefield abilities…"*; 603.4 sentence 2

**Issue.** The carve-out is honestly *described* — the doc comment at `:10314-10319` and the arm
comment at `:10357-10362` both say plainly that CR 603.4 sentence 1 is not evaluated there and why,
and seed OOS-DX1-1 is named. That part of the brief's check (b) passes. Sub-checks (a) and (c) also
pass: I re-derived all 8 LookBack rows from the source that supplies `source` (table below) and the
corpus exposure really is zero (T9 enumerates the LOWERED × `intervening_if` set as exactly
aurelia / karlach / tatyova, none of which is a leave-the-battlefield condition).

What does **not** hold is the plan's justification. §4.3 argues the deviation is safe because
*"the trigger still goes on the stack, and the Resolution re-check still runs (site 14)"* — but
site 14 (`resolution.rs:2394-2401`) hands in `InterveningIfMoment::Resolution` **unconditionally**,
and that arm evaluates `check_condition` against the current state with the same LKI source. For a
`WhenDies` + `Condition::SourceOnBattlefield` ability, `effects/mod.rs:9622-9626` reads the
graveyard object's zone and returns `false`, so the ability is removed from the stack with no
effect. The carve-out therefore buys nothing end-to-end for exactly the class of condition it was
built for; the net behaviour is queue-then-fizzle, which is the shape PB-DP6's review flagged and
plan §14 risk 2 names.

T6 does not detect this because it asserts `triggered_count == 1` and never checks whether the
2 life was gained. The test as written would stay green if the resolution arm suppressed 100 % of
LookBack triggers.

Note the engine already has a convention for exactly this: `InterveningIf::SourceHadNoCounterOfType`
returns **`true`** at resolution when `pre_death_counters` is `None` (`:10351-10353`), i.e. it
deliberately does not re-suppress on LKI. The new arm does not follow it.

**Fix**: (1) Extend T6 to assert the effect executed (`life_of(&state, p1) == 42`). It will redden —
that is the point. (2) Then pick a side and pin it: either add
`InterveningIfMoment::ResolutionLookBack` returning `true` (threaded from `resolution.rs:2394` when
the resolving trigger's `triggering_event` is `SelfDies`/`SelfLeavesBattlefield`/`SourceConnives`),
matching the `SourceHadNoCounterOfType` precedent and making the carve-out real end-to-end; **or**,
if the fizzle is judged CR-correct, say so explicitly in the `TriggerTimeLookBack` doc comment
("the queue end is carved out; the resolution end is not, and a source-scoped condition will remove
the ability there") and change T6 to assert the fizzle deliberately. Either way, correct plan §4.3's
"degrades to over-fires" claim and OOS-DX1-1's text when the seed is finally filed (Finding 3).

#### Finding 3: five seeds cited in source, filed nowhere; three audit rows left open

**Severity**: MEDIUM
**Files**: `abilities.rs:10318`, `:10377`; `resolution.rs:5598`, `:7598`;
`replay_harness.rs:2398`, `:2400`; `aurelia_the_warleader.rs:34`;
`docs/audits/decision-point-audit.md:861` (OOS-DP6-1, no closure marker)
**Invariant**: the seed-re-rank's own **N4 re-dispatch hazard** — `memory/primitives/seed-rerank-2026-07-27.md`
recorded that `OOS-RS3-1` was closed by PB-DP6 and advertised as "next dispatch" for a week because
no document was updated.

**Issue.** `rg 'OOS-DX1-'` over the whole worktree returns **five** files: four engine/card-def
source files and `pb-plan-DX1.md`. `docs/audits/decision-point-audit.md` — the file plan §11 names
as the filing destination — has **zero** `OOS-DX1-*` rows. OOS-DP6-1's row at `:861` still reads
`ranked PB-DX1` with no closure; OOS-DP6-5 and OOS-DP6-9 still carry their stale cites (`7369`,
`5351`) that the plan explicitly asked to be corrected on closure. `memory/workstream-state.md:36`
still says "PB-DX queue not started". The next person to grep `OOS-DX1-1` from the
`TriggerTimeLookBack` doc comment finds nothing.

**Fix**: add rows **OOS-DX1-1..5** to `docs/audits/decision-point-audit.md` §8.1 verbatim from plan
§11 (amending OOS-DX1-1 per Finding 2 and OOS-DX1-2 per Finding 6); mark **OOS-DP6-1**,
**OOS-DP6-5**, **OOS-DP6-9** closed by PB-DX1 (`scutemob-160`), correcting `7369`→`resolution.rs:7564`
and `5351`→`resolution.rs:5494` while closing them. Do **not** file OOS-DX1-6 (§10 shipped).
File OOS-DX1-7 as closed-on-arrival (§7.4's `- 25:`/`- 26:` prose corrections did land, and are
accurate — see the CR/wire section below).

#### Finding 4: the batch's own lossy-lowering table contradicts the code it documents

**Severity**: MEDIUM
**File**: `crates/engine/src/testing/replay_harness.rs:2398`
**Convention**: `memory/conventions.md`'s aspirationally-wrong-comment rule — the rule the plan
invoked at §4.1 Edit 1 for the `InterveningIf` enum doc.

**Issue.** The `## PB-DX1: this lowering is lossy` table is the artefact the plan called "the
batch's real lesson". Its `once_per_turn` row says *"dropped at 31/34 sites (propagated at 3) — NOT
fixed by this batch, seeded OOS-DX1-6 if left as-is"*. Phase 7 fixed it: I counted 34
`once_per_turn: *once_per_turn` writes inside `build_face_ability_vectors` (plus one in the
`ActivatedAbility` loop at `:2471`), and zero remaining `false` literals in the triggered pushes. The
table therefore documents the pre-batch state as the post-batch state, in the one comment whose
entire purpose is to stop the next reader from re-discovering a silent drop — and it points at a
seed that does not and should not exist.

**Fix**: change the row to
`| `once_per_turn` | **propagated (this batch, phase 7)** — was hardcoded `false` at 31 of 34 sites; `flush_sorted`'s gate reads the runtime value first, so three `Complete` defs (welcoming_vampire / elvish_warmaster / whispering_wizard) over-fired. CR 603.2c/603.2h. |`

#### Finding 5: Tatyova's threshold is 6, not 7, and the batch just made it live

**Severity**: MEDIUM
**File**: `crates/card-defs/src/defs/tatyova_steward_of_tides.rs:89` (and the comment at `:31`)
**Oracle (MCP, verified)**: *"Whenever a land you control enters, **if you control seven or more
lands**, up to one target land you control becomes a 3/3 Elemental creature with haste."*

**Issue.** `Condition::ControlAtLeastNOtherLands(n)` counts battlefield lands the controller
controls **excluding `ctx.source`** (`effects/mod.rs:9809-9823`). At both PB-DX1 evaluation points
`ctx.source` is Tatyova herself, a Merfolk Druid — not a land — so the exclusion removes nothing and
`(6)` means "6 or more lands". The correct argument for a non-land source is `(7)`. The def's own
inline comment at `:31` calls it an "(Approximation: '7+ lands' → ControlAtLeastNOtherLands(6)
intervening-if)", which is how it survived; before this batch the condition was discarded entirely
so the number never mattered. It matters now.

Marker discipline is correct — Tatyova stays `partial` (I confirmed: `Completeness::partial(...)`
at `:96-105`, with both surviving blockers accurately named — the `EffectFilter` card-type
intersection for the flying grant, and `targets: vec![TargetRequirement::TargetLand]` where oracle
says "up to one target land **you control**"). The plan §13's "Tatyova stays `partial`" box is
correctly ticked. But the new PB-DX1 sentence in that note calls the change a *"pure behavior
repair"*, which is one land short of true.

**Fix**: change `Condition::ControlAtLeastNOtherLands(6)` → `(7)`, delete the "(Approximation…)"
parenthetical at `:31`, and amend the PB-DX1 sentence in the completeness note to record that the
threshold was also corrected 6→7 (source is not a land, so "other" excludes nothing). Marker stays
`partial`.

#### Finding 6: the Resolution context has no targets, and OOS-DX1-2 as written would weaponise that

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:10367-10394` (comment at `:10373-10377`)
**CR Rule**: 603.4 sentence 2; hard constraint 3 (never suppress on an unanswerable condition)

**Issue.** The `Resolution` arm builds `EffectContext::new_with_kicker(controller, source, vec![],
kicker)` — an **empty** target list. Eighty lines away in `resolution.rs:2319-2324`, the registry
path's re-check passes `stack_obj.targets.clone()`. Today the divergence is invisible, and for a
precise reason worth writing down: `TargetIsLegal` (`effects/mod.rs:9650-9669`) is the **only**
`Condition` arm that reads `ctx.targets`, and it is exactly the arm the evaluability guard at
`:10378` short-circuits.

The comment at `:10373-10377` argues the guard is "over-conservative here — deliberately", and
OOS-DX1-2 proposes a `condition_is_resolution_time_evaluable` split that would let `TargetIsLegal`
through at resolution. Implemented literally against the current context, that split makes
`ctx.targets.get(index)` return `None` → `false` → **every** such trigger removed from the stack.
That is a guaranteed false negative delivered by following the batch's own seed. The trap is
undocumented.

**Fix**: thread the resolving stack object's targets into the Resolution evaluation — either add a
`targets: &[SpellTarget]` parameter to `check_intervening_if` (`resolution.rs:2394` has
`stack_obj.targets` in scope; the registry path already does this) or, minimally, add one sentence
to the comment at `:10377` and to OOS-DX1-2's text when it is filed: *"the Resolution context here
carries no targets; closing this seed requires threading `stack_obj.targets` in first, or the split
inverts into a false negative."*

#### Finding 7: the haunt queue gate leaves `haunting_target` dangling

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:4816-4818`
**CR Rule**: 702.55c

**Issue.** The resolution path clears `haunting_target` after the trigger resolves *"regardless of
whether the intervening-if held … so that a recycled ObjectId cannot cause a spurious re-trigger"*
(`resolution.rs:5582-5588`). The new queue gate's `if !gate_holds { continue; }` never reaches that
code, so a suppressed haunt trigger leaves the exiled card still haunting a creature that is now
dead. The stated hazard (recycled `ObjectId`) is the one the resolution comment was written to
prevent. Latent — zero corpus haunt defs carry an `intervening_if` (T9), and `ObjectId` minting
appears monotonic.

**Fix**: clear `haunting_target` in the `!gate_holds` branch as well, with a one-line CR 702.55c
comment mirroring `resolution.rs:5582-5585`.

#### Finding 10: `flush_sorted`'s once-per-turn lookup is base-characteristics, not layer-resolved

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:8092-8109`

**Issue.** The comment says *"Look up the layer-resolved runtime `TriggeredAbilityDef` first"*; the
code reads `obj.characteristics.triggered_abilities.get(...)`, i.e. base characteristics, not
`layers::calculate_characteristics`. Pre-existing, and the batch correctly needed **no** change to
`flush_pending_triggers` (brief item 8 — confirmed: the gate logic was already right; the lowering
was the only defect). But phase 7 is what makes this read load-bearing for three `Complete` defs, so
it is worth recording. Inert today: an ability-removal effect (Humility, CR 613.1f) would suppress
the trigger at `collect_triggers_for_event`, which *does* use layer-resolved characteristics
(`:6731`), so the stale `once_per_turn` is never consulted.

**Fix**: either switch to `layers::calculate_characteristics(state, trigger.source)` with the
existing `.unwrap_or_else(|| obj.characteristics.clone())` fallback, or correct the comment to say
"base characteristics (the runtime vector as lowered) — sufficient because `collect_triggers_for_event`
already applies CR 613.1f".

#### Findings 8 & 9: close-out bookkeeping and evidence

**Severity**: LOW

`memory/primitive-wip.md` still reads `**Phase**: plan` and `**Wire prediction**: HASH bump if fix
(a) is taken` against a shipped PROTOCOL 32 / HASH 69 and eight completed phases;
`memory/workstream-state.md:36` still reads "PB-DX queue not started" and `:40` still says
"Dispatch PB-DX1". Separately, plan §13 requires the **pre-fix T1 failure text** and **bench numbers
against `3d73763d`** (`full_turn_4p` within 5 % of ~229 µs) to be recorded; neither exists anywhere
in the worktree, so neither is verifiable at review time. The tests do carry good in-source
fail-before evidence for the riders (T10 `:960-962` and T11 `:1134-1136` each record that the arm was
reverted and the test failed) — that idiom should be applied to T1 too.

**Fix**: advance `memory/primitive-wip.md` to the review phase with the shipped wire values; update
`memory/workstream-state.md` §36/§40; paste the pre-fix T1 failure text and the three bench numbers
into this review file or the WIP file.

---

## Independent re-derivation: the 14 `check_intervening_if` call sites

Derived by reading the expression that supplies `source` at each site, not from the plan's table.
**All 14 rows confirmed.** 8 LookBack / 5 TriggerTime / 1 Resolution.

| # | site | event | `source` handed in | zone at gate time | moment | verdict |
|---|------|-------|--------------------|-------------------|--------|---------|
| 1 | `abilities.rs:4567` | `SelfDies` | `*new_grave_id` (`fizzle_object` at `:4549`) | Graveyard | LookBack | ✔ |
| 2 | `abilities.rs:4633` | `SelfLeavesBattlefield` | `*new_grave_id` (`:4614`) | Graveyard | LookBack | ✔ |
| 3 | `abilities.rs:4932` | `AnyCreatureDies` | `obj_id` — the **observer**, not the dying creature | Battlefield | TriggerTime | ✔ comment at `:4922-4924` is accurate |
| 4 | `abilities.rs:4985` | `SelfDies`/`SelfLeavesBattlefield` (Aura LTB, `AuraFellOff`) | `*new_grave_id` (`:4965`) | Graveyard | LookBack | ✔ |
| 5 | `abilities.rs:5091` | `SourceConnives` | `*object_id`, explicitly any zone (CR 701.50b, `:5074`) | any | LookBack | ✔ |
| 6 | `abilities.rs:5560` | `AnyCreatureYouControlBatchCombatDamage` | `obj_id` from an `all_bf` scan filtered `zone == Battlefield` (`:5533`) | Battlefield | TriggerTime | ✔ |
| 7 | `abilities.rs:5877` | `SelfLeavesBattlefield` (champion → graveyard) | `*new_grave_id` (`:5859`) | Graveyard | LookBack | ✔ |
| 8 | `abilities.rs:5942` | `SelfLeavesBattlefield` (→ exile) | `*new_exile_id` (`:5924`) | Exile | LookBack | ✔ |
| 9 | `abilities.rs:6007` | `SelfLeavesBattlefield` (bounce) | `*new_hand_id` (`:5989`) | Hand | LookBack | ✔ |
| 10 | `abilities.rs:6479` | `SelfLeavesBattlefield` (sacrifice) | `*new_id` (`:6461`) | Graveyard/Exile | LookBack | ✔ |
| 11 | `abilities.rs:6671` | `PermanentBecomesTarget` | `src.id`, from a scan filtered `zone == Battlefield && is_phased_in` (`:6617-6621`) | Battlefield | TriggerTime | ✔ enclosing scan verified as the plan demanded |
| 12 | `abilities.rs:7010` | **all 34 lowered events** | `obj_id`, guarded twice (`:6707` and the hard `:6715` `zone != Battlefield → continue`) | Battlefield | TriggerTime | ✔ **the headline site** |
| 13 | `abilities.rs:7076` | emblem sweep | `obj_id`, filtered `is_emblem && Command(_)` (`:7048`) | Command | TriggerTime | ✔ comment correctly notes `SourceOnBattlefield` reads false *by design* here (CR 113.6p) |
| 14 | `resolution.rs:2400` | resolution re-check | `source_object` (may be LKI; read via `fizzle_object` inside the arm) | any | Resolution | ✔ — see Finding 2 |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|--------------|---------|-------|
| 603.4 s1 (queue-time gate) | Yes | Yes | T2 (`stack_objects().is_empty()` after `DeclareAttackers`), T1 (real Aurelia def, engine-driven, count over the whole turn). Both fail pre-fix. |
| 603.4 s2 (resolution re-check) | Yes | Yes | **T3** — condition true at declaration, flipped false before resolution, asserts life stays 10 not 15. Would fail against a queue-only fix. Brief item 3 satisfied. |
| 603.4 + hard constraint 3 (unanswerable) | Yes, both ends | Yes | T5 (`TargetIsLegal { index: 0 }`) asserts both queue and resolution. See Finding 6 for the latent trap. |
| 603.10a (look-back carve-out) | Partial — queue end only | **Queue end only** | Finding 2. T6 asserts `AbilityTriggered` count, not the effect. |
| 603.2c / 603.2h (once each turn) | Yes | Yes | T13/T14/T15 on three real `Complete` defs, all oracle-verified via MCP. |
| 508.1 (whenever *you* attack) | Yes (pre-existing) | Yes | `abilities.rs:4314-4337` fires once per source per declaration, not per attacker — independently confirmed. T12a. |
| 508.3a / 500.8 / 500.10a | Yes | Yes | T1, T12b — extra combat produced by `Effect::AdditionalCombatPhase` off the stack, no state pokes. |
| 708.8 (turned face up) | Yes (rider OOS-DP6-5) | Yes | `resolution.rs:7599-7623`; T10, fail-before recorded in-source. |
| 702.55c (haunt) | Yes, both ends (rider OOS-DP6-9) | Yes | queue `abilities.rs:4789-4818`, resolution `resolution.rs:5561-5574`; T11 Parts A+B. See Finding 7. |
| 712.8d/e (face-aware lowering) | Yes | Yes | T8 asserts the back face's `InterveningIf::CardDef` survives `apply_face_change` **and** gates a real declared attack. |
| 613.1f (ability removal suppresses) | Preserved | — | Alternative (b) correctly rejected for exactly this reason; site 12 still reads `expect_characteristics`. |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|--------------|-----------------|--------------------|-------|
| `aurelia_the_warleader` | **No** | 0 | **No** (narrow) | Finding 1 — `Complete`, deck-legal, newly-live false negative. `followed_by_main: false` correct per the 2024-11-08 ruling. |
| `karlach_fury_of_avernus` | **Yes** | 0 | Yes | Flip `known_wrong` → `Complete` is **earned**. Every clause re-verified against MCP oracle by me, not just ruling #11: {4}{R} ✔, Legendary Creature — Tiefling Barbarian ✔, 5/4 ✔, `ChooseABackground` ✔, untap all attacking creatures (`ForEach::EachAttackingCreature` × `UntapPermanent`) ✔, "They gain first strike until end of turn" (`AddKeyword(FirstStrike)` on `EffectFilter::DeclaredTarget` in a second `ForEach`, `UntilEndOfTurn`) ✔, `AdditionalCombatPhase { followed_by_main: false }` ✔ per ruling #1. `WheneverYouAttack { filter: None }` fires **once per combat for the controller** — independently confirmed at `abilities.rs:4314-4337` (the loop runs outside the per-attacker loop) and `:6875-6881` (batch semantics, distinct from `AnyCreatureYouControlAttacks`). Her printed text says "if it's the first combat phase of the turn" literally, so `IsFirstCombatPhase` is a **translation** here, not a proxy — unlike Aurelia. Two probes (T12a no-personal-attack, T12b once-per-turn). |
| `tatyova_steward_of_tides` | Partial (as marked) | 1 (documented, DSL gap) | Threshold off by one | **Stays `partial`** — plan §13's box correctly ticked; the marker and both named blockers are accurate. Finding 5 on the 6-vs-7. |
| `welcoming_vampire` | Yes | 0 | Yes (post-fix) | `once_per_turn: true`, `Complete`, no def edit needed. Oracle MCP-verified. |
| `elvish_warmaster` | Yes | 0 | Yes (post-fix) | Same. The self-reinforcing-cascade characterisation is **confirmed**: the created token is a 1/1 green **Elf** Warrior entering under the same controller, and "other Elves" excludes only the source, so an ungated trigger re-fires on its own token. |
| `whispering_wizard` | Yes | 0 | Yes (post-fix) | Same. |
| `scourge_of_the_throne` | n/a | — | — | Checked because it shares the "first time each turn" shape; `partial`, attack trigger unimplemented, untouched by this batch. |

## Wire, hash and gates

- **PROTOCOL 31 → 32** (`protocol.rs:335`), fingerprint `52e9b37c…`, `- 32:` History line added,
  row appended to `PROTOCOL_HISTORY`, sentinel `protocol_schema.rs:872` = 32. The append-only
  property is **machine-checked** by `FROZEN_HISTORY_PREFIX_DIGEST` (`protocol_schema.rs:152`), which
  digests every row before the tail — a green suite is proof no existing row was edited.
- **HASH 68 → 69** (`hash.rs:679`), row appended with both digests, `HashInto for InterveningIf`
  gains discriminant **2** (`:3567-3571`; 0/1 correctly left in place), sentinel
  `hash_schema.rs:1202` = 69, `NOT_HASHED` still `&[]` (SR-19 is a no-op for an enum variant, as
  predicted). The `- 69:` note's claim that `stream_fingerprint` moves via the **v40 mechanism**
  rather than the new arm's bytes is self-consistent: `public_state_hash` folds
  `HASH_SCHEMA_VERSION` in as its first byte (stated at `hash.rs:365-366`, established at v40), and
  `canonical_fixture()` carries no `CardDef` intervening-if. Both fingerprints are gate-computed by
  construction — a hand-written value cannot pass `hash_schema.rs`.
- **The `- 25:` / `- 26:` prose corrections are accurate and were made the right way.**
  `Characteristics` is a `CLOSURE_MUST_CONTAIN` entry → `triggered_abilities: Vec<TriggeredAbilityDef>`
  → `trigger_on: TriggerEvent`, so `TriggerEvent` was indeed already in the wire closure at v25;
  `TriggerCondition` lives on the card-def `AbilityDefinition::Triggered` and is not reachable from
  `Characteristics`/`Command`/`GameEvent`, so its exclusion is correct. The corrections are made
  **in the doc-comment prose**, and the `PROTOCOL_HISTORY` table is untouched — exactly the
  distinction plan §7.4 drew. The v26 conclusion ("this sub-change touched only `TriggerCondition`,
  so HASH-only still holds even though its stated reason was half wrong") is also right.
- **SR-25 `bare_lookup_ratchet`**: ceilings unmoved — `effects/mod.rs` 110 (`:98`),
  `rules/resolution.rs` 100 (`:112`), `rules/abilities.rs` 75 (`:129`). No ceiling edits. The new
  code uses `fizzle_object` at both new lookup points (`abilities.rs:10384`, and the queue helper's
  pre-existing `:10296`).
- **SR-7**: no new `PendingTrigger`; the haunt queue site still builds through `PendingTrigger::blank`.
- **SR-9a**: `mod pb_dx1_lowered_intervening_if;` registered at `tests/primitives/main.rs:30`, sorted;
  `crates/engine/tests/` still contains exactly one top-level `.rs`, the `no_stray_test_binaries.rs`
  gate itself.
- **SR-36**: T9 is a real `all_cards()` × `effective_abilities(both faces)` enumeration with
  non-vacuity floors on both the numerator and the denominator. I checked its `is_lowered` predicate
  variant-by-variant against the actual push sites (25 direct `trigger_condition:
  TriggerCondition::…` matches plus 9 matched inside inner `match` arms at `:2672`, `:2817`, `:2894`,
  `:2988`, `:3170`, `:3216`, `:3303`, `:3336`, `:3780`): **exactly 34, no omission, no
  over-inclusion.**
- **Scope**: OOS-DP6-2 (`WheneverYouSacrifice` retain post-filter, now `abilities.rs:6347-6397`)
  correctly left ungated. Nothing else crept in.

## Test non-vacuity (16 new tests)

| Test | Would fail pre-fix? | Assessment |
|------|---------------------|------------|
| T1 aurelia once per turn | **Yes** — pre-fix the second declaration re-queues; `count == 2`, `additional_phases == 1` | Real def, real registry, engine-driven extra combat, count over the whole turn. Meets all three of plan §8.1's non-negotiables. |
| T2 queue-time gate | **Yes** (trigger queued pre-fix) | Life 40 vs `ControllerLifeAtLeast(999)`. |
| T3 resolution re-check | **Yes**, and also fails against a queue-only fix | The antidote to plan risk 2. Direct state mutation between queue and resolve is the correct isolation tool here. |
| T4 true still fires | No (by design) | Non-regression twin; guards against over-suppression. |
| T5 unevaluable | No pre-fix (vacuous then) | Post-fix it is the hard-constraint-3 pin at both ends. Value is entirely post-fix, as planned. |
| T6 look-back | No | **Assertion gap — Finding 2.** Pins the queue end only. |
| T7 legacy variants × 3 moments | No | Correct regression pin for the signature change; exercises both legacy variants at all three moments. |
| T8 back-face condition | Cannot run pre-fix (variant absent) | Forward pin, as planned. Asserts **both** the structural carry-through and the functional gate. |
| T9 roster | New | Real enumeration; predicate verified above; both derived sets pinned by name; both floors asserted. |
| T10 turn-face-up | **Yes** — recorded in-source as verified by reverting the arm | Synthetic def is unavoidable (zero corpus exposure). |
| T11 haunt A+B | **Yes** — both mechanisms recorded as verified by reverting | Part A queue end, Part B resolution end. |
| T12a karlach no-personal-attack | n/a (new def shape) | Exercises MCP ruling #11 directly. |
| T12b karlach once per turn | **Yes** | Mirrors T1 on a second real def — and is why Finding 1's fix costs no CR 603.4 coverage. |
| T13/T14/T15 once-per-turn | **Yes** (T14 pre-fix does not even terminate within the pass budget) | Real corpus defs, real registry, engine-driven casts. |

One nit worth carrying forward, not filed as a finding: `count_aurelia_triggers` is reused verbatim
for Karlach at `:1624` and `:1689`. Rename it `count_triggers_from` when Finding 1 is applied.

## Previous Findings

None — first review of PB-DX1.
