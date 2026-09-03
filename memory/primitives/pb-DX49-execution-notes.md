# PB-DX49 — every Saga site reads the printed def; a blanked Saga is still sacrificed

Task `scutemob-220`; v4 queue rank 7 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 7,
derivation §1g). Closes the **engine** half of corner-case audit #36 (the audit's last open GAP).

Seeds: **OOS-RR4-1** (subject), **OOS-RR4-3** (rider, doc rot).

---

## 0. Wire prediction — WRITTEN BEFORE ANY CODE CHANGED

**Prediction: PROTOCOL 39 UNMOVED / HASH 78 UNMOVED.** Stated with its reason rather than
asserted:

- `PROTOCOL_SCHEMA_FINGERPRINT` closes over the `Command` / `GameEvent` / `Effect` /
  `Characteristics` type closure. This batch adds **free functions and one engine-internal
  struct that is not reachable from any of those four roots** (it is a return value of a
  read-only query, never a field of a `Command`, an `Effect` payload or an event). No enum
  gains a variant; no struct in the closure gains a field.
- `HASH_SCHEMA_FINGERPRINT` hashes **declarations** of the hashed state types. This batch adds
  no field to `GameState`, `Object`, `CombatState` or any hashed type: the whole change is a
  *read* of `state.continuous_effects` (already hashed) and `obj.status.face_down` (already
  hashed) at five decision points.
- No history row is owed and no `FROZEN_HISTORY_PREFIX_DIGEST` re-pin is owed, on either gate.

The counterfactual is stated because §1g's row says it: **lowering `AbilityDefinition::SagaChapter`
into `Characteristics` would move BOTH fingerprints** (`Characteristics` is a PROTOCOL root and a
hashed type). The continuous-effect-scan design was mandated for exactly that reason and is what
ships. Gate-computed result recorded in §9.

## 0b. Pre-edit baseline — measured on this branch BEFORE any edit

`cargo test --workspace --no-fail-fast` to a file:
**4,900 passed / 0 failed / 5 ignored**, **57** result-producing targets, residual list empty.
This **reproduces PB-DX48's close pin exactly** (`4,900 / 0 / 5`, 57 targets).
Name set captured for the by-NAME delta (4,905 lines = 4,900 ok + 5 ignored).

---

## 1. Scoping facts established BEFORE the implementation, by reading code and the CR

Recorded here rather than in prose later, because three of them correct the seed row.

### 1.1 CR 714.3a has no chapter-ability clause, and the seed row assumes it does

MCP `get_rule 714`, verbatim:

- **714.3a** — *"As a Saga **without the read ahead ability** enters the battlefield, its
  controller puts a lore counter on it."*
- **714.3b** — *"…each Saga they control **with one or more chapter abilities**."*
- **714.4** — *"…a Saga permanent **with one or more chapter abilities** …"*

`OOS-RR4-1` says *"a fix to the first three alone leaves a blanked Saga still taking its ETB
counter and still firing chapter I"*, i.e. it treats the surviving ETB counter as part of the
defect. **That is right for one blanking channel and CR-wrong for the other:**

| channel | still a Saga? | CR 714.3a ETB counter | CR 714.2b chapters | CR 714.3b counter | CR 714.4 sacrifice |
|---|---|---|---|---|---|
| Layer-6 `RemoveAllAbilities` (CR 613.1f) | **yes** — an ability wipe touches no subtype | **YES, keep it** | none | none | **exempt** |
| CR 708.2a face-down | **no** — *"no text, no name, **no subtypes**"* | **no** | none | none | **exempt** |

And keeping it is not merely permitted, it is the only outcome that stays correct downstream: if
the blanker leaves, a Saga that entered with one lore counter must resume at chapter **II**.
Chapter I never triggered while blanked, because CR 714.2b needs the ability to exist at the
instant counters are put on. Suppressing the ETB counter would make the same Saga fire chapter I
after the blanker leaves — a second wrong outcome produced by "fixing" a right one.

So site 4 asks **two** questions, and the shipped query answers both from two fields.

### 1.2 The face-down conjunct at site 4 is reachable through the morph cast path and NOT through manifest

Stated because the seed's Pair B (`reality_shift` × `binding_the_old_gods`) reads as if the ETB
counter were the live symptom. It is not:

- `Effect::Manifest` (`effects/mod.rs:5247-5262`) and `Effect::Cloak` (`:5310-5325`) move the
  card to the battlefield, set `face_down` / `face_down_as` **themselves**, and emit
  `PermanentEnteredBattlefield` directly. **Neither calls `apply_self_etb_from_definition` at
  all**, so site 4 never runs on the manifest/cloak path — and neither does CR 306.5b starting
  loyalty or CR 716.2d Class level. (For a face-down permanent all three are the *correct*
  outcome under CR 708.2a, so this is right by accident rather than by design; recorded as
  `OOS-DX49-2` so a later batch that wires self-ETB replacements into those arms knows what it
  is turning on.)
- The morph/megamorph/disguise cast path **does** reach site 4 with `face_down` already
  restored — `resolution.rs:853-859` deliberately restores `face_down_as` *"before any ETB
  processing"*. So the conjunct is structurally reachable and is latent only because no corpus
  Saga prints morph.

**Therefore Pair B's live symptom is at sites 1/2/3/5, not site 4**: a manifested Saga takes a
precombat lore counter (CR 714.3b, `turn_actions.rs`), fires chapter I from a face-down 2/2, and
is sacrificed three turns later (CR 714.4, `sba.rs`). That is the defect, and it is worse than
"a counter appears" — a face-down creature resolves *"Destroy target nonland permanent an
opponent controls"*.

### 1.3 The blanker census in the seed row is wrong in BOTH halves, and its own correction fixed only one

`OOS-RR4-1` as filed: *"a Layer-6 `LayerModification::RemoveAllAbilities` (13 corpus defs, **8**
deck-legal `Complete`)"*. Its 2026-08-14 correction re-measures the **numerator** — 13 is a bare
`RemoveAllAbilities` grep that counts two comment mentions; the qualified path returns **9** at
HEAD — and leaves the **denominator** untouched.

**↻ CORRECTED by the `all_cards()` walk (`r3`). The paragraph above was this batch's own first
draft and it was wrong, in the direction that matters most.** The measured population is
**11 defs / 8 deck-legal `Complete`**, and every figure in the chain — the row's 13, the row's own
correction to 9, and this batch's orientation figure of 6 deck-legal — was produced by grepping
for the string `RemoveAllAbilities`. **That is the wrong question.** PB-DX43 moved CR 305.7's
ability loss into `LayerModification::SetLandTypes`, so `blood_moon` and `magus_of_the_moon` are
blankers again *through a different variant*, and no `RemoveAllAbilities` grep can see them. Only
"decide by **calling** `modification_blanks_abilities`" — which `r3` does — counts a blanker as a
blanker.

**And the deck-legal `8` agrees with the row by coincidence, not by measurement**, which is the
sharper finding: the row's 8 was 8-of-13 `RemoveAllAbilities` defs; the true 8 is **six**
`RemoveAllAbilities` defs plus **the two moons**. *An agreeing number is not an agreeing
membership* — and a batch that had checked only the total would have recorded the row as
confirmed.

So this batch reproduced SR-36's failure mode in its own §1.3 orientation pass, one batch after
PB-DX48 reproduced it and two after PB-DX47 filed `OOS-DX47-2` for it. The authoritative figures
are PRINTED by `core::pb_dx49_saga_blanking_roster::t_census_report`; nothing in this document is
transcribed from a grep any more.

### 1.4 A standing gate fired on this batch's own work, and it was right

`state::ability_definition_registry::handling` (SR-5's ability-definition sibling) declares, per
`AbilityDefinition` variant, **which source files name it**. `A::SagaChapter` listed four:
`replacement.rs`, `resolution.rs`, `sba.rs`, `turn_actions.rs`. After this batch, three of those
stop naming the variant at all — they ask `rules::saga::saga_view` — so the roster is now
`saga.rs` (the one derivation) **plus `resolution.rs`, which still names it deliberately** under
CR 113.7a. The gate forced that update rather than letting the roster rot, which is the third
consecutive batch in which a standing declaration gate caught its own author.

---

## 2. The engine half — what shipped, and the one delta the plan called behaviour-identical and was not

Commit `cc9d8dc3`. `git diff --numstat` over `crates/engine/src`: **+253 / −180** tracked, plus
the new untracked `rules/saga.rs` (**178** lines) — stated separately rather than netted out,
because `--numstat` cannot see an untracked file and "+253/−180" alone would understate the
change by a whole module.

### 2.1 One predicate, verified by enumeration rather than asserted

`layers::modification_blanks_abilities` has exactly two real call sites (the layer walk itself
and the new predicate). `layers::abilities_are_blanked` has exactly two (`saga_view` and IG-1 in
`replacement.rs`). Grepped across `crates/engine/src`, `crates/simulator/src`,
`crates/view-model/src` and `tools/`. **There is one ability-blanking predicate in the tree.**

### 2.2 A behavioural delta at IG-1 — disclosed, not netted out

The plan's §2a says collapsing IG-1's two early-return blocks into `abilities_are_blanked` is
behaviour-identical. **It is not quite.** The old IG-1 block handled a *missing* `new_id` by
running the scan anyway with `obj_zone = Exile` and `chars = Characteristics::default()`;
`abilities_are_blanked` returns `false` for a missing object. So a `SingleObject(new_id)`-filtered
`RemoveAllAbilities` naming an already-departed id used to suppress ETB triggers and now does not.
Narrow — every zone-scoped filter (`AllPermanents`, `AllCreatures`, …) already answered false
against an exiled object with empty characteristics — and no test in the tree covers it (the full
suite is green either way). Filed as `OOS-DX49-7` rather than left inside a sentence claiming
identity.

### 2.3 Two standing gates fired on this batch's own work; both were answered, neither weakened

1. `core::ability_definition_registry::registry_sites_match_the_source_tree` went RED with
   *"SagaChapter: declared Handled at {replacement.rs, resolution.rs, sba.rs, turn_actions.rs} but
   the source tree says {resolution.rs, saga.rs}"*. **That failure is the refactor's success
   signal** — three of the four files stopped naming the variant because they now ask the query.
   Roster rewritten to the true two-site set, with CR 113.7a's reason for `resolution.rs`'s
   retention stated in-line.
2. SR-25's `core::bare_lookup_ratchet` went RED with *"`sba.rs` is down to 6 bare lookups from the
   pinned 7 — good, you converted some. Lower its ceiling"* (the chapter-on-stack guard's
   `state.objects.get(&saga_id)` is gone; the view resolves through `fizzle_object`). Ceiling
   **lowered** 7 → 6, deliberately not left at 7: **a stale-high ceiling is slack a regression
   hides in**, which is PB-DX47's `r2` finding applied to a gate this batch merely touched.

### 2.4 Borrow restructuring at three sites, and why the line counts are larger than the change

`saga_view` takes `&GameState`; all three of `check_saga_sbas`, its stack guard and
`precombat_main_actions` were iterating `state.objects` / `state.stack_objects` out of a
`&mut GameState`. Each now materialises its candidate list first and then asks the query. That is
most of the `−55` / `−29` and it is behaviour-neutral; the reason is written at each site.

**One line in `saga_view` is load-bearing for that restructuring's cost and is an identity rather
than an optimisation**: `if printed.is_empty() { return SagaView::default(); }`, placed before the
continuous-effect scan. `chapters` is either `printed` or empty and `is_saga_permanent` requires
`!printed.is_empty()`, so a def with no printed chapters yields the default view down both arms —
retained chapters are a **subset** of printed ones, the query can only remove. Without it,
`check_saga_sbas` would clone `Characteristics` and walk every active continuous effect for
**every phased-in battlefield permanent on every SBA check** to answer a question only a Saga can
answer yes to.

### 2.5 Revert matrix — engine probes

| row | revert | result |
|---|---|---|
| **R-A** | `saga_view` stops consulting `abilities_are_blanked` (`chapters = printed` always) | **7 of 10 RED** — t1, t2, t3, t4, t5, t6, t9. Green: t7 (gated on `is_saga_permanent`, untouched by R-A), **t8 (the CR 113.7a CONTROL — it must stay green)**, t10 (gated on the missing-object early return) |
| **R-B** | drop the `face_down` conjunct from `is_saga_permanent` | **t2 and t7 RED**, all others green — which is what proves t7 discriminates a different line from t1-t6 rather than riding on R-A |

Both restored and the file re-run green. **No UNDISCRIMINATED row**: every probe reddens under at
least one revert, and t8's greenness under both is the *stated* control rather than a gap.

### 2.6 Cite drift, recorded

The plan named `resolution.rs:2194` / `:2225` (taken from the registry row, which was written
2026-08-14). At HEAD they are ~`:2267` and ~`:2298` — the same two
`AbilityDefinition::SagaChapter { effect, .. }` arms. Both carry the CR 113.7a comment now, so the
next reader finds them by symbol rather than by a line number that will drift again.

---

## 3. The census (AC 7282) — every figure walked from `all_cards()` and PRINTED by a test

`crates/engine/tests/core/pb_dx49_saga_blanking_roster.rs`, 20 tests, registered at
`core/main.rs:47`. `--test core` goes 632 → **652** passing. Everything below is printed by
`t_census_report` under `--nocapture`; nothing here is transcribed from a grep.

### 3.1 Saga side — the population is **3**, not the 4 every prior document says

| def | completeness | chapters |
|---|---|---|
| `binding_the_old_gods` | **`Complete`** (by `#[default]` derive) | front I / II / III |
| `fable_of_the_mirror_breaker` | `partial` | front I / II / III |
| `urzas_saga` | `partial` | front I / II / III |

Deck-legal subset: **1** — `{binding_the_old_gods}`, exactly as §1g predicted.

**`song_of_freyalise` is not a member.** It declares `abilities: vec![]` and names `SagaChapter`
only in two `// TODO`s and its `Completeness::inert` note. §1g's 4, `OOS-RR4-1`'s 4, this batch's
plan and this batch's own orientation pass all counted it, all from a source grep. **This is
SR-36's failure mode for the fourth consecutive batch in this queue** — `OOS-CARDS2-7` →
`OOS-DX47-2` → PB-DX48 → here. It is `r2`'s member, not `r1`'s.

### 3.2 Saga side, INVERSE (oracle-text axis) — 4 defs, residual **1**

The residual is `song_of_freyalise`: it *prints* a Saga and *declares* no chapters. The two axes
do not nest, which is the whole reason both exist.

### 3.3 Blanker side — **11 defs / 11 modification sites / 8 deck-legal `Complete`**

`blood_moon` (`SetLandTypes` / `AllNonbasicLands` / ✔), `darksteel_mutation`
(`RemoveAllAbilities` / `AttachedCreature` / ✔), `eaten_by_piranhas` (✔), `final_showdown`
(`AllCreatures` / ✘ `partial`), `imprisoned_in_the_moon` (**`AttachedPermanent`** / ✔),
`kasminas_transmutation` (✔), `kenriths_transformation` (✔), `magus_of_the_moon`
(`SetLandTypes` / ✔), `oko_thief_of_crowns` (`DeclaredTarget` / ✘ `known_wrong`), `turn`
(`DeclaredTarget` / ✔), `vraska_betrayals_sting` (✘ `partial`).

See §1.3 for why every previous figure — 13, 9 and this batch's own 6 — was measuring the wrong
thing, and why the deck-legal 8 agreeing with the row is a coincidence of totals rather than of
membership.

### 3.4 Pairs — both reproduce exactly, and there is a **fourth** blanker nobody named

- **Pair A** (`imprisoned_in_the_moon` × `binding_the_old_gods`) reproduces. Its dependency on
  **`OOS-DX20-10`** is keyed structurally on the declared
  `KeywordAbility::Enchant(EnchantTarget::Permanent)` plus `EffectFilter::AttachedPermanent`, so
  **fixing that seed reddens `r4a`** rather than silently vacating a probe. Stated in the test's
  own doc comment, not only here.
- **Pair B** (`reality_shift` × `binding_the_old_gods`) reproduces and is **unconditional** —
  `reality_shift` is `Complete` and declares `Effect::Manifest`; CR 708.2a does the rest.
- **`oko_thief_of_crowns` is a fourth blanker that can reach an enchantment, and no document names
  it.** Its +1 prints *"target artifact or creature"* and declares a bare
  `TargetRequirement::TargetPermanent`, so it can blank a Saga. It is `known_wrong` (its own marker
  cites the missing `has_card_types`), so the deck-legal blast radius is **0** — but promoting it
  without narrowing the target creates a third blanker × Saga pair. Filed `OOS-DX49-4`.
- **The moons reach an enchantment only through an enchantment LAND** — i.e. Urza's Saga, corner
  case #36's own pair — so they are gated on `OOS-RR4-2`, which is what `r4d` pins.
- **"Five creature-only blankers" is seven**, by three distinct restricting mechanisms
  (`AttachedCreature`, `AllCreatures`, `TargetCreature`), and `r4c` asserts all three stay
  represented so a widening of any one is visible.

### 3.5 `urzas_saga` authoring is explicitly NOT taken

`r4d` pins `urzas_saga`'s completeness as **not** `Complete`, with the reason in the test: the
famous Blood Moon × Urza's Saga pair fails `validate_deck`, the card half is **`OOS-RR4-2`**,
ranked separately, and this batch does not author or promote it. That is why corner case #36 goes
to **PARTIAL** and not to COVERED.

### 3.6 A false claim in CLAUDE.md's PB-DX48 narrative, corrected in place

That narrative states *"`KeywordAbility::Cloak` **does not exist** (Cloak is `Effect::Cloak`)"*.
**False at HEAD**: it is a unit variant at `card-types/src/state/types.rs:1696`, discriminant 157,
beside `KeywordAbility::Manifest` at `:1689`. **PB-DX48's conclusion survives and its measurement
was right** — zero corpus defs declare either marker — but the stated *reason* is wrong, and a
reason is the half the next batch reuses. Corrected in CLAUDE.md with attribution; `r5c` proves
the discrimination on synthetic input, because the corpus cannot.

### 3.7 A second face-down channel that behaves oppositely, pinned

`r5` measures the face-down **effect** channel: **3** defs — `cryptic_coat` (`Effect::Cloak`,
`Complete`), `reality_shift` (`Effect::Manifest`, `Complete`), `write_into_being`
(`Effect::Manifest`, not `Complete`). Those arms never reach `apply_self_etb_from_definition`
(§1.2). The morph/megamorph/disguise **cast** path *does*. `r5d` pins that no corpus Saga declares
a face-down cast keyword, on **both** spellings — the `KeywordAbility` marker and the
`AbilityDefinition::Morph { cost }` carrier — since checking one would measure one.

### 3.8 Gate-integrity work the roster carries beyond the ask

- **Clause-scoped target attribution.** `ability_targets` is read per-`AbilityDefinition`, not
  per-def, and `r4e` proves it load-bearing by measuring both readings of `Turn // Burn`:
  def-scoped sees `{TargetCreature, TargetAny}`, clause-scoped sees `{TargetCreature}`. A
  def-scoped read would let the **blanking** half be widened to `TargetPermanent` while the Burn
  half's `TargetCreature` kept `r4c` green — PB-DX45's `/review` finding, applied pre-emptively.
- **Two executed revert probes on the site roster `r6`, both RED, both restored** (verified by an
  empty `git diff --stat`): (a) a `saga_view(` occurrence planted in `rules/combat.rs` — a file no
  hardcoded list would contain — reddens the pinned set, which closes **PB-DX48's `SITE_SRCS`
  defeat**; (b) a **duplicated** `saga_view` call inside the already-pinned `check_saga_sbas`
  reddens the offset-carrying count (6 vs 5), which closes **PB-DX48's set-collapse defeat**. That
  second one matters concretely: a duplicated CR 714 query is how a Saga takes two lore counters in
  one main phase.
- `strip_comments` handles `//` **and** `/* */` with offsets preserved (`OOS-DX32-6`), and
  `r6b_comment_stripping_is_load_bearing` proves each half separately, plus that a real call
  survives and that `fn saga_view(` is not a call. `r6b_resolution_is_not_a_consumer` additionally
  asserts `resolution.rs` still *mentions* `saga_view` in prose, so its zero cannot become a
  **vacuous** zero by someone deleting the CR 113.7a comment.
- `r3b` proves the `CardType::Land` fixture is load-bearing (a default `Characteristics` drops both
  moons from the blanker set) and that a **nonbasic** `SetLandTypes` payload is correctly not a
  blanker.
- Every roster carries a non-vacuity floor or a synthetic discrimination test; `r2`'s
  empty-residual risk is bounded by an `oracle.len() >= 4` floor.

---

## 4. Reachability (AC 7281) — both directions, human seat and bot path

`crates/simulator/tests/pb_dx49_saga_blanking_channel.rs`, 4 probes, all green.
`binding_the_old_gods` — the only deck-legal `Complete` corpus Saga — via `card_name_to_id` +
`enrich_spec_from_def`, never a stand-in. Printed final chapter **3**, derived from the def rather
than hard-coded. Two seedings per probe: **leg A `lore = 1`** (the 1 → 2 crossing fires chapter II)
and **leg B `lore = 3`** (the only seeding at which CR 714.4's own comparison is reached).

| probe | channel actually driven | lore `CounterAdded` | chapter `AbilityTriggered` | Saga zone | resolution effect |
|---|---|---|---|---|---|
| c1 blanked, leg A | human seat, `LocalGame` + `HumanChoice` | **0** | **0** | battlefield ×1 | Forest still in library |
| c1 blanked, leg B | same | **0** (lore 3 ≥ final 3) | — | battlefield ×1, graveyard ×0 | — |
| c2 un-blanked, leg A | human **answers** the CR 701.19a search | **1** | **1** exactly | battlefield ×1 | Forest **on the battlefield, tapped**; library ×0 |
| c2 un-blanked, leg B | same | — | **0** | battlefield ×0, **graveyard ×1** | — |
| c3 blanked / un-blanked | pure bot: `StubProvider` + `Bot::choose_action` + `process_command` | **0** / **1** | **0** / **1** | survives / sacrificed | Forest in library / on the battlefield |
| c4 face-down manifest | human seat, against a face-up control differing in one input | **0** vs **1** | **0** vs **1** | never sacrificed | — |

Every drive stops at **turn 1, `Step::PreCombatMain`, stack and `pending_triggers` both empty**, and
re-asserts that settlement as an explicit precondition.

### 4.1 Channel revert matrix

| row | revert | result |
|---|---|---|
| **R-A** | `layers::abilities_are_blanked` short-circuited to `false` | **c1, c3, c4 RED**, each on its own first assertion (`CounterAdded left: 1, right: 0`) |
| **R-B** | site 1 alone re-reads the printed def's max chapter (sites 3 and 5 keep the fix) | **c1, c3, c4 RED on leg B**, each naming CR 714.4 — which proves leg B load-bearing **independently of site 3** |

**`c2` is GREEN under both, and that is a stated CONTROL rather than an undiscriminated row**: c2
has no blanking, so a revert of the blanking predicate must not move it.

R-B's first run failed on a *lore* assertion (`left: 0, right: 3`) rather than on the sacrifice
claim — a sacrificed Saga's `ObjectId` is dead, so `lore()` reads 0. Leg B was reordered to assert
battlefield membership first and re-run, so the messages now name the rule they are about. **"All
rows RED" is a true sentence the wrong assertion can produce** — PB-DX48's lesson, applied.

### 4.2 Three things this file could NOT prove, stated rather than worked around

1. **Site 5 in isolation is honestly UNDISCRIMINATED here, and the module doc says so.** Sites 3 and
   5 are chained on this path — `turn_actions.rs` only calls `fire_saga_chapter_triggers` for a Saga
   it just placed a counter on — so with site 3 fixed a blanked Saga never reaches site 5 and no
   site-5-only revert can redden anything in this file. `primitives::…::t5` exercises site 5 alone.
2. **CR 714.3a (site 4) is not exercised by this file at all.** The fixture uses `GameStateBuilder`,
   not `setup::build_initial_state`, because the production pregame path cannot place a *named* Saga
   on the battlefield with a *chosen* lore count — which is the independent variable. The Saga
   therefore never *enters*. Cost stated in the doc: no deck validation, no mulligan, no opening
   hand.
3. **The face-down state is poked, not created through a channel.** `GameStateBuilder` has no
   face-down setter, so c4 sets `status.face_down` + `face_down_as = Some(FaceDownKind::Manifest)`
   directly — the exact conjunct the engine reads. Everything c4 *asserts about* runs on the real
   command path.

### 4.3 The trap this file fell into and closed

**`GameStateBuilder::build()` defaults to `Step::PreCombatMain`** — which is exactly the stopping
point every settle-detecting drive hunts for. The bot-path drive, which unlike `LocalGame::start`
does not call `start_game`, satisfied *"settled at turn-1 precombat main with an empty stack"*
**before issuing a single command**, and asserted its verdict against a board no command had
touched. **This is PB-DX48's shape reached through a different door** — a drive that stopped because
it never started, wearing the same assertion as one that stopped because resolution finished, except
that here the vacuity came from the *fixture's default* rather than the drive's endpoint. Closed
three ways (seed `Step::Untap`, call `start_game`, and assert the drive has **not** already arrived
*before* the loop in both drives) and filed as a class in `OOS-DX49-8`, because every
`GameStateBuilder` fixture in the tree inherits the same default.

### 4.4 A live defect found while choosing the verdict — `OOS-DX49-1`

`binding_the_old_gods`' chapter I destroys nothing. See the registry row; the important
methodological point is that it was found **by execution while looking for something else**, not by
a code read, and that it is filed with **no probe** on purpose.

Chapter III was rejected as the observable effect for a related reason worth recording: its
deathtouch grant is an `EffectFilter::CreaturesYouControl` continuous effect that resolves its
controller through `state.objects.get(&source_id)` at layer-application time, and chapter III is the
*final* chapter, so CR 714.4 sacrifices the Saga in the same window — the source id is gone and the
filter matches nothing. A fact about `EffectFilter` and a departed source, not about CR 714.

---

## 5. Gates, against the FINAL tree

- Tests **4,941 / 0 / 5**, **58** result-producing targets (57 → 58), residual list empty.
  **41 additions / 0 removals / 0 leavers / 0 renames**, by set-diffing the two run logs — 10 engine
  probes, **24** roster rows (20 shipped, +4 in the `/review` fix cycle), 4 channel probes, and 3 in
  `tools/tui`'s new `#[cfg(test)]` module. "0 leavers" is literal: the three
  `fire_saga_chapter_triggers` call sites lost a parameter and were edited **in place**, so no test
  name changed.
- **PROTOCOL 39 / HASH 78 both UNMOVED**, gate-executed (`protocol.rs:427` = 39, `hash.rs:886` = 78)
  and predicted in writing at `57d1dc42` before any code changed. `history_is_append_only` and
  `frozen_prefix_is_pinned` green; no pin edited and no history row appended, because none was owed.
- Coverage **1,137/1,803 = 63.1%** by regeneration, **0 flips** as predicted (clean 1,137 / todo 519
  byte-identical), self-dating churn reverted. **0 card-def edits of any kind.**
- `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs).
- **Benches — see §6. The first version of this line was WRONG and the `/review` refuted it by
  running the A/B this batch had not.**

---

## 6. The `/review` fix cycle — 2 MEDIUM, 1 LOW-MEDIUM, 4 LOW, 1 NIT; all 8 taken, none declined

The reviewer had a shell and used it. **Three findings are defeats by execution of this batch's own
claims**, and one of those is a claim the batch printed in bold in production source.

### 6.1 MEDIUM — "there is exactly one ability-blanking predicate in this tree" was UNGATED

`replacement.rs` asserted it in bold; §2.1 above said "verified by enumeration". Both were **true
and unenforced**. The reviewer appended a second hand-rolled predicate to `turn_actions.rs` —
`matches!(e.modification, LayerModification::RemoveAllAbilities) && effect_applies_to_object(..)`,
**the exact pre-PB-DX43 shape whose 26-def regression this batch's own doc comment narrates** — and
all 652 core tests stayed **GREEN**. That is `OOS-DX49-6`'s own shape (a comment asserting a
property the code does not enforce) inside the batch that filed it.

**Fixed** by `r7_blanking_variant_naming_sites_are_pinned`, keyed on the mechanism: every
comment-stripped occurrence of a blanking variant name across **workspace** source, keyed
`(file, enclosing fn, variant)`, set-compared to an allowlist whose every entry carries a `kind` and
a reason — **plus a second conjunct** that re-checks each allowlisted site's enclosing function body
for the tokens a predicate needs beyond naming the variant (`effect_applies_to_object`,
`continuous_effects`), because set equality alone cannot catch a predicate added **inside** an
already-allowlisted function. Both defeats executed RED, including that in-place one.

**Three things the finding got wrong, found by re-deriving instead of trusting it.** Its sketched
allowlist was short by three (`layers.rs::depends_on` and `::representative_modifications` — the
CR 613.8 dependency machinery — and the enum declaration itself in `crates/card-types`); there is
**no** test fixture in `src`, so "plus one if present" is empty; and **its prescribed needle was
itself PB-DX47's defect** — keying on the *qualified* `LayerModification::RemoveAllAbilities` is
evaded by `use LayerModification::RemoveAllAbilities;` + `matches!(m, RemoveAllAbilities)`. `r7`
keys on the **bare name at word boundaries**, and `r7b` proves both spellings are seen.

Measured allowlist: **7 `(file, fn)` sites / 11 triples**.

### 6.2 MEDIUM — the bench claim was refuted by an A/B this batch had not run

§5's first draft reported branch-only figures against a *remembered* historical number and concluded
*no regression*. **That is PB-DX28's "re-take the measured table" MEDIUM, which PB-DX45 already
repeated once.** The reviewer built the merge base in an isolated worktree with its own
`CARGO_TARGET_DIR` and measured **~+6% `sba_check` / ~+2.4% `full_turn_4p`**, non-overlapping
confidence intervals, twice.

**The finding was right, and the mechanism was the one this batch had already identified and then
not acted on**: `check_saga_sbas` materialised a `Vec` of *every* phased-in battlefield permanent
before asking the query. That `Vec` was never necessary — `saga_view` takes `&GameState` and the
function holds a `&mut`, so **one immutable reborrow** (`let s: &GameState = state;`) lets the walk
and the query share it. All three walk sites now do that; the walk stays lazy and nothing is
materialised.

**The honest A/B, re-run after the fix — matched set, same hardware, same session, merge base
`be7f29a5` in an isolated worktree vs this branch:**

| bench | merge base | branch | delta |
|---|---|---|---|
| `sba_check` | 14.685-14.751 µs | 14.954-14.989 µs | **+1.7%** (non-overlapping — REAL) |
| `priority_cycle_4p` | 24.185-24.434 µs | 24.634-24.808 µs | **+1.7%** (non-overlapping — REAL) |
| `priority_cycle_6p` | 38.156-38.416 µs | 38.860-39.170 µs | **+1.9%** (non-overlapping — REAL) |
| `full_turn_4p` | 216.72-218.91 µs | 217.18-218.51 µs | **noise** (intervals overlap) |
| `full_turn_6p` | 345.85-346.84 µs | 344.36-346.96 µs | **noise** (intervals overlap) |
| `board_wipe_4p` | 122.88-125.20 µs | 117.57-118.03 µs | **−5%** (branch FASTER) |

**Stated plainly: there is a real ~1.7% regression on the SBA and priority-cycle benches, and it is
published as a regression rather than as "inside the historical band".** The reborrow took
`full_turn_4p` from a measured ~+2.4% to noise and `sba_check` from ~+6% to +1.7%. The residual is
inherent to the mandated design: `saga_view` re-resolves the object through `fizzle_object` because
it takes an `ObjectId` rather than the caller's `&GameObject`, which is one hash probe per
battlefield permanent per SBA check. **Threading the object through instead would shave it and would
re-create the drift this batch exists to remove** — five sites deriving CR 714 from their own local
view is the defect, not the cost. The `printed.is_empty()` short-circuit (§2.4) is doing its job:
without it the same walk would clone `Characteristics` and scan every active continuous effect.

### 6.3 LOW-MEDIUM — a production doc comment claimed a seed that did not exist

`rules/saga.rs` said *"Stated residual (seeded, deliberately not fixed here)"*. The reviewer grepped
the registry: `OOS-DX49-1..8` contained nothing about the Saga-ness proxy — `OOS-DX49-3` covers a
**different** residual. **The batch's own headline shape, one layer down.** Filed as
**`OOS-DX49-9`**; the comment now names the row.

### 6.4 LOW — `r6`'s reach was one crate while `saga_view` is `pub`

A `saga_view` consumer planted in `crates/simulator/src/lib.rs` left `r6` **green** — PB-DX48's
`SITE_SRCS` defeat one crate up rather than one directory up, and the module doc overclaimed
("the whole crate", as if that answered it). Walk widened to **workspace** source
(`crates/*/src` + `tools/*/src`, minus `crates/card-defs`), with executing non-vacuity floors
(≥ 8 roots, engine root present, ≥ 100 files) so a broken path returning `[]` cannot make either
`r6` or `r7` pass. Measured: **14 roots / 148 files**, printed by the census. Defeat re-run RED.

### 6.5 LOW — the classifier could be widened silently for any zero-corpus variant

Moving `SwitchPowerToughness` into the `true` arm left the **entire** engine test set green: `r3`
gates the classifier only where the corpus reaches, and PB-DX43's
`f3_..._and_no_others` pins 2 positives against 5 hand-picked negatives out of 33 — an overclaim
this batch inherited and then made load-bearing at a second site. **Fixed** by
`r8_modification_blanks_abilities_is_exhaustively_classified`: one instance of **all 33** variants,
with the constructed name set gated against the variant names **parsed from the enum's own
declaration** so a 34th cannot arrive unclassified; positives asserted as exactly
`{RemoveAllAbilities, SetLandTypes}`; nonbasic `SetLandTypes` asserted false; both positives
re-asserted against a default `Characteristics` so neither is a fixture artefact. Defeat RED — and
under it **`r3` stayed green**, corroborating the finding's premise by execution.

### 6.6 LOW — `r5b`'s 4,000-byte window failed open, and was ALREADY over-scanning

The finding framed this as conditional (*"if either arm grows past 4,000"*). **It is not
conditional.** Measured and now printed: the `Effect::Manifest` arm body is **3,413** bytes and
`Effect::Cloak` **2,820**, so the old window ran **520** and **1,116** bytes *past* each arm's own
closing brace into the next arm — while sitting 520 bytes of growth from failing open. Replaced with
`match_arm_body_span`, which brace-matches the arm's own body (skipping string literals) and
**panics with a fail-closed message** on unbalanced or non-block input; no fixed backstop remains.
**Both halves proven by execution**: a planted `apply_self_etb_from_definition` call behind ~1.2 kB
of filler reddens the new gate, and with that identical plant in place the *superseded* window
**passed** — the old gate failing open on the exact call it exists to catch, demonstrated rather
than argued.

### 6.7 LOW — the `tools/tui` parser repair had no test

`OOS-DX49-6`'s own fix shipped untested in a crate with two `#[test]`s, neither touching it. The
parse is split into `parse_corner_case_audit_content(&str)` and three tests added: the sum
(RED-proven by reverting `total` to `covered + gap` → `left: 35, right: 36`), a non-vacuity case
(rows outside `## Summary` must not count, so the sibling's 35/1/0/0 is evidence the section was
found), and the stop-at-next-heading case.

### 6.8 NIT — `CLAUDE.md` said `fire_saga_chapter_triggers` is called "for a Saga it just countered"

Should be "just placed a lore counter on"; a reader takes "countered" as a stack action. Fixed.

### 6.9 What the reviewer verified and did NOT find a problem with

Recorded because a clean result is evidence too: the **CR 714.3a correction is right** (rule text
pulled independently); **no probe is vacuous** (the face-down conjunct reddens `t2`/`t4` from one
site and `t2`/`t7` from the other, both executed); **both channel reverts reproduce exactly**; corner
case #36 is genuinely PARTIAL; the `KeywordAbility::Cloak` correction to PB-DX48 is itself correct;
the `corner-cases.md` CR 305.7 rewrite is verbatim-accurate; and **no CR 714 reader was missed** —
`grep -rn SagaChapter` over `crates/` + `tools/` leaves only the deliberate CR 113.7a pair, the
registry declaration and comments. **Every published figure reproduced except the benches.**
