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
