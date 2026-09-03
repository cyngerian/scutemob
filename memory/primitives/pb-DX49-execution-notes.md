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
