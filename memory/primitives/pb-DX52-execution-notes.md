# PB-DX52 — execution notes

**Task**: `scutemob-229` · v4 queue rank 14 · seeds `OOS-DX25b-1` (headline) + `OOS-DX25b-5` (rider).
**Branch**: `feat/pb-dx52-bolt-bends-printed-or-ability-half-is-unreachable-an`
**Merge base**: `cecf0ba0`.

---

## §0 Stage 0 — measured, BEFORE any production line changed

### §0.1 Pre-edit baseline (executed)

`cargo test --workspace --no-fail-fast` to a file:
**5,117 passed / 0 failed / 5 ignored**, **64** test-result lines.
**Reproduces PB-DX36's published close pin exactly** (5,117 / 0 / 5, 64 targets) — no
discrepancy to report, which is the first time in five batches the inherited pin
reproduces without a correction (`OOS-DX51-5` was the last one that did not).

*Procedural note, recorded because it cost twenty minutes and will recur*: a wait loop
written as `for i in ...; do pgrep -f "cargo test --workspace" || break; done` **never
exits** — `pgrep -f` matches the loop's own `bash -c` command line, which contains the
needle verbatim. It reported STILL RUNNING for 20 minutes after the run had finished.
Filed as `OOS-DX52-1`.

### §0.2 Version / count pins at HEAD (read off source, not remembered)

| Pin | Value at HEAD | Cite |
|---|---|---|
| `PROTOCOL_VERSION` | **42** | `crates/engine/src/rules/protocol.rs:490` |
| `PROTOCOL_SCHEMA_FINGERPRINT` | `9d75f591…02aa47` | `protocol.rs:507-508` |
| `HASH_SCHEMA_VERSION` | **83** | `crates/engine/src/state/hash.rs:1046` |
| PROTOCOL closure type count | **98** | gate output, PB-DX36 |
| HASH closure type count | **132** | gate output, PB-DX36 |
| Coverage | 1,139 / 1,803 = 63.2% | `docs/authoring-status.md` |

### §0.3 THE DESIGN DECISION — `Target::StackObject`, not `state.objects` registration

The brief offers two options and calls the seed's own prescription a claim. Both were
costed by executed census before a line was written.

#### Option (b) — register ability stack entries in `state.objects` (CR 109.1's literal reading): REJECTED

Measured blast radius (241 full-map walk sites; 207 production; 9 with no zone filter at
all; 4 whose zone filter is *negative* or *includes `Stack`*):

1. **HARD BLOCKER — `crates/simulator/src/invariants.rs:79`** (`check_zone_integrity`)
   emits an `InvariantViolation` for every `state.objects` entry that is not in some
   zone's `object_ids()`. So an ability entry cannot merely be inserted; it must also be
   pushed into `zones[ZoneId::Stack]`.
2. Doing that moves **`public_state_hash`** — `hash.rs:8867-8879` is **zone-driven**, so
   the Stack zone's `Vector<ObjectId>` gaining an element changes the digest for *every
   game with an ability on the stack*, and the full `GameObject` is then hashed at
   `:8875`.
3. **`loop_detection.rs:132`** (`compute_mandatory_state_hash`) is map-driven and
   unconditional (`Hand | Library => continue, _ => hash`), and it *already* hashes
   `state.stack_objects` separately at `:145` — so an ability entry would be
   **double-counted** in the mandatory-loop fingerprint.
4. **A TARGETING CORRECTNESS REGRESSION, not a cost**: `queries.rs:428` and
   `retarget.rs:267` both offer `Target::Object(id)` for any object whose zone is
   `Battlefield | Stack | Graveyard(_)`, and `casting.rs`'s `TargetSpell` arm validates
   by `obj.zone != ZoneId::Stack` **alone, with no spell/ability discriminator**. An
   ability entry claiming `ZoneId::Stack` therefore becomes a legal target for
   *"counter target spell"* — CR 115.1a-wrong (a spell is not a permanent, and "counter target spell" names a spell), and a new defect this batch would have
   shipped while closing an old one.
5. **`sba.rs:451`** removes any `is_token` object not on the battlefield from
   `state.objects` — a landmine one field-default away.
6. `GameObject` (`card-types/src/state/game_object.rs:1069`) has **two non-`Option`
   fields that must be fabricated**: `characteristics: Characteristics` (requires a
   name, rules text, four `OrdSet`s, three `Vec`s, two `Vector`s — and **no `CardType`
   fits**, there is no `CardType::Ability`) and `zone: ZoneId`. There is no "kind"
   discriminator on `GameObject` at all: nothing in that map can say *"I am not a card."*
7. The view model already builds its stack view from `state.stack_objects()`
   (`view-model/src/lib.rs:513`), so registration produces a **duplicate
   representation**, not a missing one made present.

**The CR argument for (b) does not survive contact with the model.** CR 109.1's "object"
is a rules concept; `state.objects` is this engine's **card**-object map, and CR 113
abilities are modelled by `state.stack_objects`. Registering them in both makes the model
*doubly* represented, not *more* faithful.

#### Option (a) — `Target::StackObject(ObjectId)` naming the stack ENTRY's own id: TAKEN

Measured blast radius: **≈22 files to compile and pass gates** (13 production + 7
test/`cfg(test)` forced by exhaustive matches, + `protocol.rs` and
`tests/core/protocol_schema.rs`), plus **13 wildcard/`if let`-without-`else` sites across
8 files that the compiler will NOT flag** — enumerated and each dispositioned in §0.5 —
plus the untyped `tools/replay-viewer/frontend/` JS.

**The one shape choice that collapses most of the work**: `Target::StackObject(id)`
carries the **stack-entry** id, and
`state::stack_registry::stack_index_for_announced_target`'s **first clause is already
`so.id == announced`**. So every existing consumer — `Effect::ChangeTargets`,
`Effect::CounterSpell`, `Effect::CopySpellOnStack`, `casting.rs`'s two single-target arms
— resolves a stack-entry id through the shared arithmetic *with no new plumbing at all*.
`resolve_effect_target_list_indexed` (`effects/mod.rs:8337-8348`) likewise **already**
accepts a stack-entry id (its `exists_on_stack` clause, written for CR 702.21a Ward), so
`Target::StackObject(id)` maps to `ResolvedTarget::Object(id)` at that single site.

**`ResolvedTarget` is deliberately NOT given a third variant.** That enum has ~55
`if let ResolvedTarget::Object(..)` sites with no `else` in `effects/mod.rs`; growing it
would create 55 silent-swallow sites to buy nothing, because the id it would carry is the
same id and the one function that consumes it already resolves both spaces. Stated here
so a later batch does not "finish the job".

### §0.4 WIRE PREDICTION — written before any production line changed

**Predicted, per half:**

* **PROTOCOL 42 → 43, ONE bump.** `Target` is in the wire closure: `Command::CastSpell.targets: Vec<Target>` (`rules/command.rs:105`), `:405`, `:749`, `:779`, and `SpellTarget` via `GameEvent::TargetsChanged` (`rules/events.rs:1424/1426`) and `GameEvent::TargetsAnnounced` (`:1571`). Adding a variant is a *shape* change, so `PROTOCOL_SCHEMA_FINGERPRINT` moves.
* **HASH 83 → 84, ONE bump.** `impl HashInto for Target` (`state/hash.rs:4864-4877`) is reached from `StackObject::hash_into` (`:4889`) via `SpellTarget`, which is inside the `GameState` serde closure the DECLARATION fingerprint is taken over. The new `2u8` stream tag is *append-only*, so no existing hash value changes — but the **declaration** digest moves, which is what the gate pins.
* **Closure type counts predicted UNCHANGED at 98 (PROTOCOL) / 132 (HASH)**, and the reason is stated rather than asserted: this batch adds a **variant to an existing type**, not a new type. No `TargetRequirement` variant is added either.
* **Both are ONE bump for the whole PB** — the card-def half (`deflecting_swat`) mints no type, and the offer/validate/re-check halves add only functions.
* **A two-step observation is expected** (v40/v82/v83's pattern): with the variant in the tree and hashed but BEFORE the version bump, `declaration_fingerprint_is_pinned` should be RED while `stream_fingerprint_is_pinned` may be GREEN, because `canonical_fixture()` carries no stack object with a `StackObject` target. To be observed, not assumed.

**Not owed, stated rather than left implied**: **no `GameEvent::PermanentTargeted` is
owed for a stack-entry target.** CR 702.21a Ward is *permanent*-only ("Whenever this
permanent becomes the target of a spell or ability an opponent controls"), and an ability
on the stack is not a permanent. `rules/events.rs::permanent_targeted_events`'s
`_ => None` arm therefore gives the CR-correct answer for the new variant by accident —
so the arm is made **explicit** rather than left to the wildcard, since a wildcard that
happens to be right is not a decision anyone made.

### §0.5 The 13 sites the compiler will NOT flag — disposition table

| # | Site | Verdict |
|---|---|---|
| 1 | `resolution.rs:4294-4311` Modular death-trigger `_ => None` | correct-by-accident → made explicit |
| 2 | `events.rs:1670-1681` `permanent_targeted_events` `_ => None` | **CR-correct** (Ward is permanent-only) → made explicit with the CR cite |
| 3 | `resolution.rs:1912-1918` Aura attach `else { None }` | correct (an Aura cannot enchant a stack entry) → explicit |
| 4 | `resolution.rs:4504-4510` Scavenge `else { None }` | correct → explicit |
| 5 | `casting.rs:3907` Aura battlefield check | correct → explicit |
| 6 | `casting.rs:6506` CR 601.2c inter-target distinctness | **stated residual**: no corpus requirement pairs two stack-object slots; recorded, not silently skipped |
| 7 | `abilities.rs:564` AttachEquipment | correct → explicit |
| 8 | `abilities.rs:634` Fortify | correct → explicit |
| 9 | `view-model/src/redact.rs:246` | **must be handled** — a redaction hole is Invariant 7 |
| 10 | `testing/replay_harness.rs:947-950` | script channel — handled |
| 11 | `targeting.rs:62` `is_unchosen_slot` | correct (the SENTINEL placeholder is always `Target::Object`) |
| 12 | `resolution.rs:7713` equality compare | correct |
| 13 | `pb_dp8_trigger_target_choice.rs:311` (test) | correct |


---

## §1 Coverage prediction — written BEFORE regeneration

**Predicted: 0 flips, coverage unmoved at 1,139 / 1,803 = 63.2%.**

The reason, stated rather than asserted. The whole card-def diff is:

| Def | Change | Marker moves? |
|---|---|---|
| `deflecting_swat.rs` | `TargetRequirement::TargetSpell` → `TargetSpellOrAbility`, plus the falsified note repaired | **no** — it carries no explicit `completeness`, so it is `Complete` by derive before AND after |
| `bolt_bend.rs` | comment only (`OOS-DX25b-1` CLOSED; the note's own instruction to invert `t3` discharged) | **no** — stays `Complete` |
| `misdirection.rs` | comment only | **no** — stays `Complete` |
| `untimely_malfunction.rs` | untouched | **no** — stays `partial`; its blocker is mode 2's "one or two target creatures" variable count, which this batch does not touch |

**No `Completeness` marker moves anywhere in the corpus**, so the `CORPUS_COMPLETE` SET is
unmoved as well as its count — and therefore `OOS-CARDS2-3`'s seeded-fixture re-deal budget
is **not owed**. That is checked by `git diff` over the marker rather than inferred from an
unchanged total: PB-DX26's lesson is that a stable COUNT is not a stable SET, and two
cancelling flips would leave the count still while moving the deal.

## §2 Frontend — a REFUTED premise of the acceptance criterion, reported not skipped

AC 7352 predicts: *"npm run build if `tools/play-server/frontend` moves (it will if the
picker learns a new target kind)"*.

**It does not move, and the parenthetical is false.** `TargetPicker.svelte` echoes each
candidate's `.value` back **verbatim** (the engine's own serialized `Target`) and displays
`.label`, grouping by `.owner`. It never reads `.kind` at all — grepped across the whole
frontend, the only `.kind` consumers are `CostPicker`'s cost tags and `stores.js`'s
`ApiError`. So a stack-entry candidate renders and round-trips through the browser with
**zero frontend production lines**, which is a property of the picker's design (UI-1's
"echoed back verbatim, never rebuilt") rather than luck.

`npm run build` is therefore **N/A**, on the same two grounds every recent batch states:
`git diff main..HEAD --numstat -- tools/play-server/frontend` is EMPTY, and `node_modules`
is absent from this worktree.

`tools/` is **not** zero, and saying "the frontend did not move" must not be read as
saying that: `tools/play-server/src/view.rs` gains the `"stack_object"` wire kind and a
THIRD `NameIndex` map keyed by stack-entry id — deliberately not folded into the
`ObjectId`-keyed one, because that exact fold is the collision `view.rs`'s own `from_view`
comment records as a shipped bug. `tools/tui/` gains the render arm and one Cargo
dependency.

---

## §3 Revert matrix — 8 rows, EXECUTED by the coordinator, 8 discriminating, 0 UNDISCRIMINATED

Every row applied to the real production source, run, and restored; the tree was
re-verified green afterwards and `git diff` over each reverted file is empty. Delegated
agents reported their own rows; **these were re-executed rather than accepted** (PB-DX48's
rule — a delegated "all rows RED" is a true sentence the wrong assertion can produce).

| # | Revert | Reddens |
|---|---|---|
| **R1** | delete the `Target::StackObject` tail in `queries::legal_targets_per_slot` | **9** — `c1`, `c3`, the inverted `t3`, `t1`, `t2`, `t3`, `t7`, `t9`, `r2a` |
| **R2** | `TargetSpellOrAbilityWithSingleTarget` always `Err` | **11** — `c1`, `c2`, `c3`, inverted `t3`, `t1`, `t2`, `t3`, `t5`, `t6`, `t8`, `t9` |
| **R3** | `is_target_legal`'s `StackObject` arm always `true` | **3** — `t6`, `r1c`, `r3a`. Exactly three, which is what proves `t6` is the CR 608.2b probe and nothing else rides on it |
| **R4** | `permanent_targeted_events` emits for a stack entry | **2** — `t8`, and **PB-DX48's own `r2_exactly_one_construction_site`**, a neighbouring batch's gate correctly catching the second construction site |
| **R5** | drop the `is_spell` guard from `TargetSpellWithSingleTarget` | **1** — `t2`. The distinctness probe AC 7348 asks for is the ONLY thing holding that guard |
| **R6** | `plan_target_change` reads `card_in_stack_zone` again | **1 before `t10`, 2 after** — see below |
| **R7** | offer EVERY stack entry, spells included (drop the de-dup predicate) | **2** — `c4` and `r2a`; the predicate is pinned by the control, not by argument |
| **R8** | (delegated, re-executed) the `TargetSpellOrAbilityWithSingleTarget` arm's `Err` re-run against the HTTP probes | both play-server probes RED |

### R6 is the row worth reading, and it is a coverage measurement

Putting `card_in_stack_zone` back — undoing the CR 702.16b protection fix — reddened
**exactly one thing**: `r7b`, a SOURCE gate that reads the call site's text. **No
behavioural probe moved.** A source gate proves a line is spelled a certain way; it cannot
prove the line does anything, and a later batch that "simplifies" the helper while keeping
the name satisfies it completely. So the fix this batch describes at length as *"a defect
this batch would have created"* was, at that moment, standing on a text comparison.

Closed by `t10_protection_from_red_refuses_an_ability_shaped_redirect`, which is RED under
R6 on its own assertion message. **The durable half: a revert matrix is also a coverage
measurement. A row that reddens only a source gate is telling you the behaviour has no
probe — not that the row is uninteresting.**

## §4 Benches — MEASURED, seven runs, verdict NO REGRESSION, and the apparent improvement is NOT claimed

Matched-set A/B against merge base `cecf0ba0`, each revision in its own `git worktree`
with its own `CARGO_TARGET_DIR`.

**The first A/B was thrown away, and saying why is the point.** Base runs 1-2 were taken
while this session's own full test suite and revert matrix were running. Their same-code
band came out at **up to 47%** (`full_turn_4p` 326.58 vs 221.96 µs across two runs of
IDENTICAL code), and the contaminated table read *"HEAD 30% faster on `sba_check`"* — an
effect nothing in this batch can cause, which is the tell. Discarded and re-run on a quiet
machine rather than averaged.

Quiet-machine table (µs, criterion's point estimate):

| bench | base3 | base4 | head1 | head2 | head3 | base band | mean delta |
|---|---|---|---|---|---|---|---|
| `priority_cycle_4p` | 24.77 | 24.64 | 24.65 | 24.23 | 24.29 | 0.54% | −1.26% |
| `priority_cycle_6p` | 39.35 | 39.01 | 38.13 | 38.57 | 38.24 | 0.88% | −2.22% |
| `sba_check` | 14.72 | 14.73 | 15.07 | 14.58 | 14.33 | 0.10% | −0.45% |
| `full_turn_4p` | 222.24 | 218.63 | 217.78 | 218.02 | 216.66 | 1.65% | −1.34% |
| `full_turn_6p` | 347.62 | 349.16 | 344.91 | 342.25 | 339.45 | 0.44% | −1.78% |
| `board_wipe_4p` | 118.38 | 119.60 | 119.01 | 118.16 | 119.17 | 1.03% | −0.18% |

**Verdict: no regression. The uniform 0.2-2.2% improvement is deliberately NOT claimed**,
for three reasons stated rather than waved at:

1. **HEAD's own three-run spread is WIDER than every difference in the table**: `sba_check`
   reads 14.33-15.07 across head1/head2/head3, a **5.2%** same-code band, against a
   base-vs-HEAD difference of 0.45%.
2. **The controls move the same order as everything else.** `priority_cycle_4p`/`6p` and
   `sba_check` execute no line this batch touched, and they shift 0.45-2.22% — the same
   band as `full_turn`. A uniform shift across benches that cannot be affected is a
   build/layout artefact of two separate compilations (PB-DX20b's tell, PB-DX51's too).
3. **The mechanism bound is independent of the numbers and is measured, not argued.**
   `grep -cE "legal_targets_per_slot|retarget|ChangeTargets|CastSpell|StackObject|Target::"
   crates/engine/benches/engine_perf.rs` returns **0** — every path this batch changed is
   off every benched path by construction. And `size_of` was executed AT BOTH REVISIONS and
   is identical: `Target` **16 → 16**, `SpellTarget` **32 → 32**, `TargetRequirement`
   **304 → 304**, `StackObject` **504 → 504**. The new `Target` variant carries exactly one
   `ObjectId`, like the two it joins, so it cannot widen the enum; `TargetSpellOrAbility` is
   a unit variant in an enum whose largest member is a `TargetFilter`. Nothing on the hot
   layer/SBA/priority path got bigger, and nothing copied per mutation grew.

## §5 Fuzz — NOT A/B'd, with the reason

No `Completeness` marker moved anywhere in the corpus (§1), so `CORPUS_COMPLETE` is
unmoved as a SET and no seeded fixture is re-dealt — the usual reason a batch owes a fuzz
A/B (`OOS-CARDS2-3`) does not apply. Beyond that, the change is reachable only through a
cast that announces a stack-entry target, and the fuzzer's bots reach it only if
`plan_targets` picks one; `c3` proves the bot path handles the kind, and the offer set for
every other requirement is unchanged by construction (the new candidates satisfy only the
four stack-object requirements). **Stated as a reason to expect no movement, not as a
measurement** — which is the distinction PB-DX49's `/review` refuted a batch for blurring.

---

## §6 `/review` fix cycle — 12 findings, ALL 12 TAKEN, none declined

Opus reviewer with a shell, briefed to try to defeat the gates. It did, three ways.

### The headline: a FIFTH family of wrong CR cites, and it is the largest

**CR 115.4 was cited 27 times by this branch as the authority for *"target spell"* /
*"target spell or ability"*. CR 115.4 says the opposite.** Verbatim from the rules server:

> **115.4.** Some spells and abilities **that refer to damage** require "any target,"
> "another target," "two targets," or similar rather than "target [something]." These
> targets may be creatures, players, planeswalkers, or battles. **Other game objects, such
> as noncreature artifacts or spells, can't be chosen.**

It is the *"any target"* rule, and its last sentence explicitly forbids choosing a spell.
`main` uses it correctly and only for `TargetAny` (6 sites, all left alone). All 27 new
occurrences are re-pointed at **CR 115.1a** (*"An instant or sorcery spell is targeted if
its spell ability identifies something it will affect by using the phrase 'target
[something]'"*) — which, with CR 109.1 / CR 113.1c making a spell and an ability on the
stack both objects, is the actual authority. This batch had already run a four-family cite
correction on itself; **the correction pass was itself incomplete, and the biggest family
was the one it did not look for.**

Three more wrong rule numbers, each with the RIGHT number already present elsewhere in the
tree: **CR 702.43b → 702.43a** (Modular's trigger; 702.43b is the *multiple instances*
rule), **CR 702.98a → 702.97a** (Scavenge; 702.98 is Unleash), and **CR 702.71a → CR 207.2c**
(bloodrush is an ABILITY WORD with, per 207.2c, *"no individual entries in the
Comprehensive Rules"*; 702.71 is Transfigure). Plus **two survivors of this batch's own
`CR 113.3` pass**, one of them load-bearing production prose, and **CR 707.10b → CR 707.10**.

### Three gates defeated by execution, all three re-keyed and re-proven

**`r1a` fell six ways** — and the reviewer's compile-against-synthetic-inputs method found
a seventh the review did not name: the original matched the CONTIGUOUS literal
`.iter().find(`, which `rustfmt` splits across lines, so it was **effectively vacuous on
real multi-line source** and only ever caught the synthetic single-line probes. Re-keyed on
the mechanism: receiver derivation with the accessor's trailing `()` normalised away and a
FIXPOINT alias pass, plus a **bidirectional** search window (a combinator is written after
the receiver, but a `for` loop's keyword is written before it — that asymmetry was the one
bypass still blind after the first re-key, found by execution). All six re-executed:
**CAUGHT / CAUGHT / CAUGHT / CAUGHT / CAUGHT / CAUGHT.** The widened scan then found **six
real entry-id lookups**, each allowlisted with a reason and a companion `r1d` that
re-checks the reason in source, plus a **per-entry** non-vacuity floor (a total would be
satisfied by one entry matching many times while another matched zero).

**`r6b` and `r7a` fell to a NAMED catch-all.** Both asserted `!body.contains("_ =>")` while
their own messages promised a new variant would be *"a compile error"*. `other => None` is
equally irrefutable, contains no `_ =>`, survives `rustfmt` and compiles. New
`irrefutable_catch_all_arms` collects both spellings; both bypasses planted and
re-executed: **r7a RED, r6b RED**, tree restored green.

### A comment asserting the exact opposite of this batch's headline, 60 lines from the new code

`casting.rs`'s `TargetSpellWithSingleTarget` arm still said the two requirements were
*"behaviourally IDENTICAL on the production path today"* and that its guard *"becomes
load-bearing the day OOS-DX25b-1 is closed"*. **This batch is that day**, so both halves
were false at HEAD — `OOS-DX47-6`'s shape, inside a batch that invokes PB-DX27's *"a
blocker note is a claim"* three times in its own registry rows. Rewritten, along with three
sub-case framings in the same file that described their fixtures as pinning a now-closed
deviation.

### The rest

| # | Finding | Taken as |
|---|---|---|
| 7 | `source_of` returns `None` for five kinds, silently disabling CR 702.16b there | the consequence AND the reachability accident that makes it safe are now written at the arm |
| 8 | `self_id` is asymmetric between a spell victim and an ability victim | recorded at the call site with why passing the entry id would be wrong in the other direction |
| 9 | `DX52_SEED`'s *"byte-identical"* claim is false (8 members vs 5) | corrected to the narrower true claim, wrong version left visible |
| 10 | `Effect::CounterSpell` × `Target::StackObject` has no probe | stated as a bound in the probe file's own module doc, with why no corpus def can reach it |
| 11 | the census needle list misses Disallow/Voidslime's phrasing | recall bound stated beside the list, population measured at 0 |
| 12 | CR 707.10b cited for CR 707.10's last sentence | corrected |
