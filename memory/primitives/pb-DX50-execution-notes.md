# PB-DX50 — execution notes

**Task** `scutemob-221` | **v4 queue rank 8** | **Seeds** `OOS-DX25-1` + `OOS-DX29-2`
**Branch** `feat/pb-dx50-the-mutate-surface-target-legality-mutate-is-a-targe`

Plan and stage-0 predictions: `memory/primitives/pb-plan-DX50.md`.
15-variant copy audit: `memory/primitives/pb-DX50-additional-cost-copy-audit.md`.

---

## §1 Headline

CR 702.140a says a spell cast for its mutate cost *"**targets** a non-Human creature with the same
owner as this spell."* The engine carried that choice in `AdditionalCost::Mutate` and never put it
into `spell_targets`, so **Ward never fired on a mutate cast** and the mutate validator checked
zone, creature-ness, non-Human and owner **and nothing else** — no hexproof, no shroud, no
protection. CR 702.140c says the on-top/under choice is made **as the spell resolves**; the engine
took it at announcement, so an opponent learned it before responding.

---

## §2 Measurements

### §2.1 Baseline (pre-edit, on this branch, to a file)

**4,941 / 0 / 5**, **58** result-producing targets, residual list empty — reproducing PB-DX49's
close pin exactly. Test-NAME set captured for the delta.

### §2.2 Census — walked, printed, and it refuted the implementer's own first pin

Walked from `all_cards()` (SR-36: never grep source), **printed** by
`core::pb_dx50_mutate_site_roster::r1` rather than transcribed:

```
Brokkos, Apex of Forever  [Complete]         Necropanther            [Complete]
Gemrazer                  [Complete]         Nethroi, Apex of Death  [not-deck-legal]
Glowstone Recluse         [Complete]         Sea-Dasher Octopus      [Complete]
Mindleecher               [not-deck-legal]   Vulpikeet               [Complete]
total 8 / deck-legal Complete 6 / with MutateCost 8
```

**The v4 memo's 6 reproduces exactly** — which is the outcome the re-derivation discipline is FOR,
and is worth stating because it is rare in this queue. The two non-deck-legal defs are
`Mindleecher` and `Nethroi, Apex of Death`, neither previously named anywhere.

**The implementer's own first pin was 8 and the gate refuted it.** Recorded because it is the fifth
consecutive batch in this queue where a guessed census figure did not reproduce.

Inverse oracle axis: **0** defs print "mutate" without declaring the marker — pinned at **0**, not
ceilinged.

### §2.3 Wire — the Half 1 prediction HELD

Predicted in writing before any code: Half 1 moves **neither** fingerprint. Gate-executed after:
`core hash_schema` 36/36, `core protocol_schema` 17/17, **PROTOCOL 39 / HASH 78 UNMOVED**. The
reason was stated rather than asserted — the mutate host is injected into the `StackObject`'s
EXISTING `targets` and `target_requirements` fields and the requirement is built from `TargetFilter`
fields that already exist, so no type, variant or field is added anywhere.

---

## §3 What shipped — Half 1 (`b3aa24c7`, `301c7c37`)

`casting::mutate_target_requirement()` is ONE synthesized `TargetRequirement` —
`TargetCreatureWithFilter { exclude_subtypes: [Human], owner: You, ..Default::default() }`, the
exact analogue of PB-DX20's `enchant_target_to_requirement`. Consumed by three sites and nothing
else.

**The enforcement-site list in both seeds and the queue row was short by one**, and the missing one
is the one that would have broken. See §5.

---

## §4 The CR finding that changed the design

**CR 702.140b is an explicit EXCEPTION to CR 608.2b.** *"As a mutating creature spell begins
resolving, if its target is illegal, it ceases to be a mutating creature spell and continues
resolving as a creature spell."* It does **not** fizzle.

`OOS-DX25-1`'s prescription is "route the mutate target into `spell_targets`". Doing exactly that
hands the host to the generic CR 608.2b fizzle gate and **regresses a behaviour the engine gets
right**. Nothing in the seed, the queue row or the brief says so; only the rule does.

It does not regress as shipped for a **structural** reason, not a checked one: the CR 608.2b gate
lives inside the `StackObjectKind::Spell` arm (`resolution.rs:194-266`) and
`MutatingCreatureSpell` is a **disjoint arm** with no fizzle gate. That is exactly the kind of
load-bearing accident a later batch deletes by "unifying the two arms", so `t7` / `t7b` / `t7c` pin
it — each asserting the 702.140b fallback fires and that **no `SpellFizzled` is emitted** — and
`t7d` pins the undisturbed merge.

---

## §5 Corrections this batch made to its own inputs

1. **The site list was short by one.** Both seeds and the v4 row describe two sites. There are
   three: `casting.rs` (cast), `resolution.rs` (CR 702.140b re-check) and — unnamed anywhere —
   **`legal_actions.rs`'s `non_human_own` offer enumeration**. Tightening the cast path while the
   offer layer keeps a looser predicate is *a clean offer followed by a guaranteed refusal*: the
   SR-38 shape PB-DX29 gated Fuse to avoid, PB-DX44 re-created while fixing it, and PB-DX45 shipped.
   **This batch would have been the fourth.**

2. **THE PLAN'S OWN SITE-2 PRESCRIPTION WAS A REGRESSION, corrected before it shipped.** It said to
   replace the four hand-rolled conjuncts with the shared `is_target_legal`. That function checks,
   for an object target, **only** that it is still in its cast-time zone — so the "shared" thing was
   **weaker than the duplicated thing**, and delegating to it would have deleted three checks.
   *"One arithmetic" is an improvement only when the arithmetic that survives is the right one.*

3. **The plan named the wrong function for the protection layer.** It said
   `validate_object_satisfies_requirement` is "what adds hexproof / shroud / protection". It is not
   — those live in `validate_mapped_targets`, reached only via `validate_targets_inner`.

4. **The implementer overruled the coordinator on site 2, with a better answer.** The coordinator
   recommended the narrower per-object predicate to avoid an engine-wide asymmetry. The implementer
   used `validate_targets_inner` — **literally the function the cast path runs**
   (`validate_targets_with_source` is a two-line wrapper) — so cast-time and resolution-time
   legality are the **same call with the same requirement**, rather than two predicates that agree
   today. Exercised by `t7c`, so it is not an unexercised claim; the asymmetry it creates is stated
   at the site as deliberate and **bounded** (CR 702.140b's consequence is a graceful fallback, never
   a fizzle, so this arm being ahead of the others can turn a merge into an ordinary creature ETB but
   can never silently delete a spell's effect).

5. **The plan said the offer layer's raw-characteristics read was "filed and left".** It is **fixed**,
   as a necessary consequence of routing the offer through the shared query — the offer layer reads
   layer-resolved characteristics for the first time.

6. **The offer set had to become per-CARD and no document anticipated it.** The old code hoisted the
   host list out of the hand loop. Protection (CR 702.16b) is a property of the *(source, target)*
   pair, so two different mutate cards in hand can have different legal host sets. Hoisting would
   have been wrong.

7. **A false comment in production source, corrected.** The `MutatingCreatureSpell` arm's own doc has
   claimed since the mutate subsystem shipped that the resolution re-check covers a host that
   *"gained protection from the mutating spell"*. It did not — the four conjuncts there were the same
   four as at cast time. `t7c` is what makes that sentence true.

8. **A probe of this batch's own refuted one of its own assertions on first run.** `c5`'s first draft
   asserted the cast Gemrazer kept its hand `ObjectId`. **CR 400.7 makes it a new object** — this
   project's self-declared #1 bug class, committed inside a probe about targeting. Corrected in place
   with the refutation stated in-source.

---

## §6 Half 3 — the 15-variant copy audit

Full table: `memory/primitives/pb-DX50-additional-cost-copy-audit.md`. Every sharp claim was
**re-verified by the coordinator rather than accepted**; all four checked claims held.

**`copy.rs`'s comment is refuted by the rule it cites.** It asserts *"CR 707.2: copies copy choices
… but not one-shot additional costs"*. CR 707.10 says verbatim that a copy copies *"all decisions
made for it, including modes, targets, the value of X, **and additional or alternative costs**"*,
and its closing sentence — *"if an effect of the copy refers to objects used to pay its costs, it
uses the objects used to pay the costs of the original"* — makes dropping `Sacrifice`
**affirmatively wrong**. The comment also names 6 of the 12 variants it drops.

**CR 707.10 settles AC 7303 better than the criterion's own wording.** The criterion asks that a
copied mutate spell *"keep its Mutate entry with a defined `on_top` answer"*. CR 707.10 sentence 3
is **"Choices that are normally made on resolution are not copied"** — so once Half 2 makes `on_top`
a resolution-time choice, the copy **must not** inherit it. The entry is kept (the target IS copied,
sentence 2) and the answer is defined by the copy asking at its own resolution.

**`Effect::CopySpellOnStack` has ZERO genuine declarations** — both grep hits are comments
(`plumb_the_forbidden.rs:42`, `complete_the_circuit.rs:6`). SR-36's failure mode for the **fifth
consecutive batch** in this queue.

---

## §7 A rule written down is not a rule applied — TWICE, from two prior batches

1. **`mana.rs:878-879`** says the CR 605.4a gate covers *"four asking effects: SearchLibrary, Scry,
   Surveil, DiscardCards"*. The gate checks **seven**. This is the **same sentence** PB-DX45's
   `/review` caught one channel short in `effects/mod.rs` — and that fix corrected the copy it
   noticed. Nobody asked whether the sentence lived anywhere else. It did, three channels staler.

2. **`rules/engine.rs:181-187`** is PB-DX45's **obligation (8)**, which states the rule exactly:
   *"a wildcard arm that encodes a JUDGEMENT cannot also serve as a fallback for the UNKNOWN, and an
   enum whose growth is expected should be matched exhaustively at every gate that decides what a
   client may send."* It names only `api::validate_decision_params`. Meanwhile
   `effects::handle_answer_effect_choice` has **two** non-compile-forced traps ~30 lines apart — a
   `matches!` over six hardcoded pairs (a seventh variant is silently REJECTED, not a compile error)
   and an `unreachable!()` tail (so fixing only the first turns a rejection into a **release panic**)
   — both in the file PB-DX45 was editing, two functions away.

**One failure mode, two prior batches: a claim corrected where it was noticed rather than where it
lives does not generalise, and neither does a rule.**

---

## §8 `/review` fix cycle — 7 findings, all taken, plus 1 the review did not have

Coordinator-fixed before this cycle and **not** redone here: the HIGH (the `is_copy` guard's
early `return Ok(events);` skipping the shared resolution tail, `5565d588`) and a
LOW-MEDIUM registry-row corruption (`OOS-DX50-11` filed).

### (a) HIGH-adjacent — `r4` was GREEN through the entire hang

**Reproduced first.** `r4` asserted only `body.contains("stack_obj.is_copy")`, which BOTH
the correct `else` shape and the hanging `return` shape satisfy. Re-planting the first
draft (`return Ok(events);` inside the `is_copy` branch) gave, on one command each:
`primitives::pb_dx50_mutate_on_top_timing::t8` **RED** (`priority_holder = None,
players_passed = {P1, P2}`) and `core::pb_dx50_copy_additional_cost_roster` **4/4 GREEN**.

Both halves shipped, and the behavioural one is the one that would have caught it:

* **`t8_a_resolving_copy_of_a_mutate_spell_still_grants_priority`** — mints the copy through
  the production `rules::copy::copy_spell_on_stack` (never hand-built: `copy.rs` cloning
  `original.kind` wholesale IS the defect, so a fixture with a fresh `source_object` cannot
  express it), passes both seats, and asserts `priority_holder.is_some()`, that the ORIGINAL
  survives, and that it goes on to ask its own CR 702.140c question.
* **`r4` gains two conjuncts**: exactly ONE `return Ok(events)` in the arm, and it must sit
  AFTER the `ask_resolution_choice(` call and on a `None =>` line. Comments are stripped
  first and that is **load-bearing, not defensive** — the arm's own doc quotes the defective
  line verbatim, so an unstripped scan fails on the CORRECT code.

**Executed defeats**: (1) the first-draft `return` → `left: 2, right: 1` on the count
conjunct; (2) rewriting the suspend as `None => { return Ok(events); }` → RED on the idiom
conjunct. Defeat (2) is behaviour-NEUTRAL, so the idiom conjunct is a FORM gate; its cost
(it also fires on an honest `if answer.is_none() { return }` refactor) is stated at the
site, with the instruction to widen the accepted idiom rather than delete the check.

### (b) MEDIUM — `r3` policed the DEFINITION; all four copies lived in the CONSUMER

**Both of the reviewer's defeats reproduced GREEN** (4/4 roster tests) before any fix, by
planting in `crates/simulator/src/legal_actions.rs`: (1) a host predicate omitting the
non-Human conjunct; (2) `SubType(String::from("Hum") + "an")`.

Root cause named precisely: `legal_actions.rs` contains **zero** occurrences of
`mutate_target_requirement` — it calls `queries::legal_mutate_hosts` — so conjunct 1's set
equality over files naming the predicate could never see it.

Shipped:

* **`simulator::pb_dx50_mutate_legality_channel::c6`** (behavioural, load-bearing): the
  offered host set must EQUAL `queries::legal_mutate_hosts`' live return value, on a
  four-class board (legal Wolf / Human / **shroud** / opponent-owned), with non-vacuity
  asserted in both directions.
* **`r3` conjunct 3** (structural): every workspace file CONSTRUCTING
  `LegalAction::CastWithMutate` must call `legal_mutate_hosts` exactly once, and the
  identifier it iterates must be the one that call binds — **bound exactly once**, which is
  what catches defeat 2's shadowing rebind.
* **`r3b`** gains synthetic discrimination for the construction-vs-pattern helper.

**Executed re-runs against the fixed gates**: defeat 1 → `r3` RED (`legal_mutate_hosts` 0
times) **and** `c6` RED (`offered={2,3,4} engine={2}`); defeat 2 → `r3` RED (`bound 2 times
… SHADOWING rebind`). **`c6` is GREEN under defeat 2 and that is reported, not hidden**: that
copy is a no-op *today* because `legal_mutate_hosts` already excludes Humans, which is
precisely why the structural conjunct is not redundant — *a redundant second predicate is
not wrong until the first one changes, and then it is wrong silently.*

Conjunct 2's recall bound is **corrected in place**: it claimed *"the only way to express
CR 702.140a's non-Human is a `"Human"` subtype literal"*. False, and refuted by execution. A
string-literal census cannot be made concatenation-proof; it is now labelled a tripwire.

### (c) LOW-MEDIUM — the CR 605.4a gate-site census, defeated two ways at once

**Reproduced**: `rules/abilities.rs`'s `effect_choice_gate_closed: false` → `true`, test
GREEN. Two holes: three hardcoded files (not including `abilities.rs`), and the assignment
spelling only (every `EffectContext` in this tree is a struct LITERAL — five such sites).

Shipped as a NEW core module, **`core::pb_dx50_effect_choice_gate_sites` (g1-g4)**, walking
`workspace_src_files_checked()` and counting both spellings. **The first draft of this fix
extracted the shared walk to `crates/engine/tests/shared/` behind `#[path]`, and SR-9a's
`no_stray_test_binaries` gate refused it in three separate assertions — correctly**
(*"Attributes, `pub mod`, `#[path]`, and inline `mod x { … }` are all ways to look declared
while not being compiled"*). The gate was obeyed, not weakened: the census moved to `core`,
which SR-9a's own layout table calls the home of the machine-checked invariant gates, and
`pb_dp9_effect_choice` keeps parts (a) and (b) plus a pointer. **`g3` links the two files in
both directions** so neither can be deleted while the other claims coverage.

**Executed defeats**: the reviewer's own → `g1` RED naming both sites; deleting the pointer
from `pb_dp9_effect_choice.rs` → `g3` RED.

### (d) LOW — the double-sweep deletion also removed `run_delayed_trigger_cleanup`

Documented at the site, with the **mechanism** rather than the conclusion. No divergence is
constructible and the reason is now stated: `collect_delayed_triggers` opens with
`if dt.fired { continue; }`, and its `WhenSourceLeavesBattlefield` scan is gated on the
CURRENT batch's `left_battlefield` set, which CR 400.7 makes un-repeatable for one id. So
the cleanup is hygiene, not a correctness gate, and being one command late is unobservable.
`handle_all_passed` has always had the same property, so this is the two paths agreeing.
**No divergence was found; if a future change makes `check_triggers` sensitive to a stale
delayed trigger, BOTH sites need the cleanup.**

### (e) LOW — the ~30-space prompt

`view.rs`'s `MutateOnTop` prompt was one physical line with three 30-space gaps. Fixed with
string continuations, and **pinned** by (f)'s new test (`!prompt.contains("  ")`), which was
executed RED against the restored bad literal.

### (f) LOW — nothing constructed a `MutateOnTop` question and looked at the bytes

New `view.rs` unit test **`pb_dx50_binary_choice_wire_shape::test_dx50_mutate_on_top_serializes_the_keys_the_picker_reads`**:
builds a real state, a real `NameIndex`, calls `blocking_decision_view`, serializes, and
asserts `question` / `answer_field` / `shape` / the five keys `ActionBar.svelte` passes into
`BinaryChoicePicker` / `choice_key == "on_top"` / `default == true` / the template's single
variant key by PRESENCE / that `choice_key` names a key inside the template.

**Executed defeats**: `choice_key` `"on_top"` → `"onTop"` → RED; `#[serde(rename)]` on
`false_label` → RED naming the key. **This test also tripped play-server's own
`test_no_socket_symbol_appears_in_the_test_region` gate** (the word "binds" in a comment
under a test attribute); reworded rather than the gate weakened.

### (g) NIT — stale assertion message in `mechanics_m_z/mutate.rs`

Corrected, and it now states WHY (half 2 deleted the field) rather than just dropping the
dead words.

### A finding neither the coordinator nor the reviewer had

While checking (d), `rules/abilities.rs`'s `collect_permanent_becomes_target_triggers` doc
was found to read:

> **Latent for the mutate case today**: the mutate target is never entered into
> `spell_targets` (`OOS-DX25-1`), so no `PermanentBecomesTarget` event is ever raised for a
> mutate cast's own target — this fix only takes effect once that gap closes.

**PB-DX50 half 1 IS that gap closing.** The comment survived the commit that falsified it —
`OOS-DX47-6`/`OOS-DX49-6`'s exact shape, committed by the batch whose own headline is a
false comment, and caught by neither the batch nor its `/review`. Corrected, and pinned
behaviourally rather than replaced with a second sentence:
**`primitives::pb_dx50_mutate_target_legality::test_dx50_t12_whenbecomestarget_fires_for_a_mutate_host`**
fires a `WhenBecomesTarget { scope: creature you control }` trigger off a mutate host,
count `== 1`. Executed revert (stop appending the host to `targets`) → `left: 0, right: 1`.

### Gates, all against the FINAL tree

`cargo test --workspace --no-fail-fast` **4,991 / 0 / 5**, **59** result-producing targets,
residual list empty. **+8 over the 4,983 pre-fix-cycle pin, 0 removals, 0 renames, 0
leavers** — `t8`, `c6`, `t12`, the play-server wire-shape test, and `g1`-`g4`. Targets
unmoved at 59: the new gate is a MODULE in `core`, not a new binary.
**PROTOCOL 40 / HASH 79 both gate-executed and UNMOVED** (`hash_schema` 36/36,
`protocol_schema` 17/17) — nothing here adds a type, variant or field.
`clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
`tools/check-defs-fmt.sh` clean (1,803 defs), `npm run build` green (160 modules).
**0 card-def edits.**


---

## §9 The `/review` — 1 HIGH / 1 MEDIUM / 2 LOW-MEDIUM / 3 LOW / 1 NIT, all eight taken

### §9.1 The HIGH was caused by the coordinator's own instruction

The `is_copy` guard added to the `MutatingCreatureSpell` resolution arm — which the coordinator
ordered, **explicitly overruling the copy audit's advice to defer it** (plan §7.4) — shipped as an
early `return Ok(events);`.

The instruction was *"make the mutate arm agree with `resolution.rs:819`"*. The implementation copied
`:819`'s **condition** and dropped its **control flow**: `:819` is an `if / else if` chain that FALLS
THROUGH to the shared resolution tail. A `return` there leaves `resolve_top_of_stack_inner`
altogether, skipping `check_triggers_with_timing`, `check_and_apply_sbas`, `flush_pending_triggers`
and — fatally — `priority::grant_priority_to_active_player`. Executed by the reviewer:

```
priority_holder = None   players_passed = {P1, P2}   pending_effect_choice = None   stack len = 1
PassPriority(P1) -> Err(NotPriorityHolder { expected: None, actual: PlayerId(1) })
PassPriority(P2) -> Err(NotPriorityHolder { expected: None, actual: PlayerId(2) })
```

An unrecoverable game, and **new** — with the guard disabled the arm suspends legally.

**This is PB-DP8's own recorded lesson, verbatim: *a guard that returns early inherits the obligation
of the statements it skipped*.** That sentence is in this repository, in `docs/audits/`, written by a
prior batch on this same queue, and it was committed against anyway. The durable half is narrower and
more useful than "read the lessons": **"agree with site X" is an instruction about behaviour, and
copying a condition while dropping the control flow around it is not agreement.** When a guard is
added beside an existing one, the thing to copy is the *shape*, and the check is "what runs after the
site I am imitating, and does my version still reach it?"

**The batch's own `r4` gate stayed GREEN through all of it** — it asserted only that the arm's body
contains `stack_obj.is_copy`, which is true of both the correct and the hanging shape. Closed with
two conjuncts (position relative to the ask, and the `None =>` idiom) plus **`t8`**, a behavioural
probe that mints the copy through the production `rules::copy::copy_spell_on_stack` and asserts
`priority_holder.is_some()` — the one that would actually have caught it.

### §9.2 Two of this batch's own gates were defeated by execution

* **`r3`** ("exactly ONE mutate target-legality predicate in the workspace") was defeated **twice**,
  both green with all four roster tests passing: a second host-legality predicate in
  `legal_actions.rs` **omitting the non-Human conjunct** (the literal SR-38 defect), and one spelling
  the subtype as `SubType(String::from("Hum") + "an")`. Cause: the gate does set-equality over
  `mutate_target_requirement` tokens in three NAMED files and keys conjunct 2 on the `"Human"`
  literal — **so it polices the DEFINITION and is blind to the CONSUMER, and the consumer is where
  all four historical hand-rolled copies lived.** Closed with a consumer-keyed conjunct plus `c6`, a
  behavioural probe asserting the offered host set *equals* `queries::legal_mutate_hosts`' live
  answer.
* **The CR 605.4a site census** in `test_dp9_mana_ability_gate` claimed *"exactly one site in the
  tree"* closes the gate, and stayed GREEN when `abilities.rs:290`'s
  `effect_choice_gate_closed: false` was flipped to `true` — defeated **two ways at once**: it read
  three files while `abilities.rs` is also an `EffectContext` construction site, and its needle was
  the assignment form `= true` while five sites use the struct-literal `: true`. Both closed.

### §9.3 The coordinator's registry edit destroyed a word

The `OOS-DX29-2` closure split that row by column. **The row has carried SIX cells in a four-column
table since it was filed** — its own `` `Entwine | Fuse | EscalateModes` `` uses unescaped pipes — so
the edit appended its closure to a fragment ending `` (`Entwine `` and **overwrote the cell holding
`Fuse`**, briefly recording the propagation allowlist as two variants instead of three.

Repaired, pipes escaped, and the incident written into the row itself. A sweep of every `OOS-`/`PB-`
row found **five** carrying the hazard; the other four are deliberately **not** repaired — they are
not this batch's rows, each needs its intended column split *inferred* rather than mechanically
restored, and **a confident mis-repair is worse than a row known to be malformed**. Filed as
`OOS-DX50-11` with the gate that would have caught all five and refused the bad edit. It matters
because the registry is machine-read: PB-DX49 closed `OOS-RR4-3` on precisely the finding that *the
table a tool reads is not the prose a human reads*.

### §9.4 A false comment neither the batch nor the review had seen

Found by the fix-cycle runner while investigating an unrelated finding.
`abilities.rs::collect_permanent_becomes_target_triggers` still read:

> **Latent for the mutate case today**: the mutate target is never entered into `spell_targets`
> (`OOS-DX25-1`), so no `PermanentBecomesTarget` event is ever raised for a mutate cast's own target
> — this fix only takes effect once that gap closes.

**PB-DX50 half 1 *is* that gap closing.** The comment outlived the commit that falsified it — inside
the batch whose headline is a false comment, and missed by the batch AND by the review. Corrected,
and pinned **behaviourally** rather than swapped for another sentence:
`test_dx50_t12_whenbecomestarget_fires_for_a_mutate_host`, revert-proven `left: 0, right: 1`.

### §9.5 Two coordinator prescriptions were refuted while being applied

* **"Use a hexproof host" for `c6`'s board would not have discriminated.** CR 702.11b is *"can't be
  the target of spells **your opponents control**"*, and this mutate is cast by the host's own
  controller — so a hexproof host of the caster's is a perfectly legal target. The engine's own
  answer refuted the first draft (`{Wolf, Hexproof}`, not `{Wolf}`). Switched to **Shroud**
  (CR 702.18a, no controller clause).
* **"Reuse `workspace_src_files_checked()`" could not be applied as written** — that walk lives in
  the `core` test binary and `pb_dp9_effect_choice` is in `primitives`, which are separate crates.
  The first attempt extracted it behind `#[path]` and **SR-9a's `no_stray_test_binaries` gate refused
  it in three separate assertions**, correctly. The gate was obeyed rather than weakened: the census
  moved to `core`, with `g3` linking the two files in both directions.

### §9.6 Dispatch hygiene 8 earned its keep on this batch's own summaries

Re-checking every headline surface against the registry **after** the fix cycle found the v4 memo's
row-8 cell still saying `OOS-DX50-1..10`, because `-11` was filed *by* the fix cycle, after that
summary was written. Corrected. This is the rule's exact case, and it is worth noting that the error
was invisible to every check made before the fix cycle ran.

---

## §10 Final numbers

* Tests **4,991 / 0 / 5**, **59** targets, residual empty. Baseline **4,941 / 58**.
  Delta by NAME: **53 additions, 3 leavers, 0 removals, 0 renames** (leavers = PB-DX29's mutate
  trio: 2 inversions + 1 re-home, each with a named `test_dx50_*` successor).
* **PROTOCOL 39 → 40 / HASH 78 → 79**, one bump each, predicted per half before any code and
  gate-computed after; type counts unchanged at **98 / 131**.
* **47 HASH + 13 PROTOCOL** sentinels re-pinned, **0 stale survivors** (verified with an independent
  multi-line regex, because the plan's own same-line one is what produced the wrong census).
* Coverage **1,137/1,803 = 63.1%**, **0 flips**, **0 card-def edits of any kind**.
* clippy / `fmt --check` / `check-defs-fmt.sh` (1,803) / `npm run build` all clean against the FINAL
  tree.
* **Benches not measured, so nothing claimed.**
