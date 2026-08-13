# PB-DX8 — execution notes, measurements and revert matrix

`scutemob-208`, 2026-08-12. v3 queue rank 10. Closes **OOS-DP10-9** (recorded, not eliminated)
and **OOS-CARDS2-7**; ships the **PB-DX42a** rider per
`docs/audits/mtg-characteristics-recursion-adjudication.md` §5.1.

Everything below is measured on this branch. Where a number differs from the brief or from a
filed seed, the difference is stated rather than silently adopted.

---

## 0. Baseline

Pre-edit, measured on this branch BEFORE any edit, `cargo test --workspace --no-fail-fast` to a
file: **4,527 passing / 0 failing / 5 ignored**, **46** result-producing targets, residual list
empty. Identical to the PB-DX7 pin in CLAUDE.md, which is the expected result on an unmodified
branch and is stated here because it is evidence the branch was clean, not decoration.

---

## 1. The derivation problem, and three measured failures before the answer

The acceptance criteria forbid a hand-written vocabulary on either axis: a hand list is
`OOS-CARDS2-7`'s own defect recurring. Four derivations were built and measured. **Three failed,
each for a different and separately-instructive reason**, and they are recorded because the
failures are the argument for the rule that shipped.

| attempt | rule | why it failed | evidence |
|---|---|---|---|
| **D-a** | iterated sentence bootstrapping from `{may, choose, up to}`: a phrase joins V if it is ≥8× lifted inside choice-bearing sentences; repeat to fixed point | **semantic drift**. By iteration 3 the vocabulary had absorbed `battlefield`, `library`, `graveyard`, `search`, `shuffle` — the vocabulary of *search effects*, which contain "choose", not the vocabulary of *choice* | iter 0 → 12 new, iter 1 → 7, iter 2 → 16, iter 3 → 19; V grew 3 → 57 in four passes |
| **D-b** | the same rule, single-pass, no iteration | **object-noun noise**. Top survivors were `land card`, `basic land`, `mana cost`, `to two` — collocates of the search idiom, not markers of optionality | 12 candidates at ≥20 sentences / ≥8× lift, ~2 of them plausible |
| **D-c** | contrast against the DSL's own ground truth: learn the vocabulary from the oracle text of defs that DO carry a decision-bearing DSL construct | **self-blinding on the target.** The derived V was `{choose, chosen, one, up, of the}` — **`may` does not appear at all**, because "you may" is precisely the class the DSL cannot encode. A vocabulary learned from what the DSL already expresses cannot rediscover the marker for the choice it never expresses | 104 defs with a decision-bearing element vs 1,699 without; `may` scored below the lift threshold |
| **D-d — SHIPPED** | **morphological closure**: take the first 3 characters of each single-word marker as a stem and admit every whole word in the corpus's own oracle text that begins with it; match the phrase marker as a phrase | derives the inflected family from the corpus, cannot drift (no iteration), and is blind to nothing by construction | `may` → {`may`}; `cho` → {`choice`, `choose`, `chooses`, `chosen`}; one false positive (`mayhem`, 1 occurrence) recorded as a reasoned exclusion |

**The durable lesson**: a statistical derivation over a corpus learns the vocabulary of the
*effects that surround* a marker, not the marker. And a derivation grounded in what a system
already encodes is structurally unable to find what it fails to encode — which is the same shape
as `OOS-CARDS2-7` itself, met a second time, from the opposite direction.

---

## 2. OOS-DP10-9 — the oracle-text-vs-DSL cross-check

`crates/engine/tests/core/pb_dx8_oracle_decision_cross_check.rs`, **16 tests**.

### 2.1 The two axes

* **Oracle axis** — D-d above, computed from `all_cards()` at run time and pinned by
  `t_decision_word_closure_is_pinned`.
* **DSL axis** — identifier stemming over the serialized corpus surface: **617 object keys +
  711 bare variant strings = 1,328 elements**, tokenized on camelCase/snake_case, kept when a
  token starts with the *same stem*. Measured: `may` → {`MayPayOrElse`, `MayPayThenEffect`};
  `cho` → 20 elements; `up to` → {`UpToN`}.

**Channels are paired and are not interchangeable.** A `choose`-shaped construct does not
discharge a printed "you may". This is the whole reason the gate sees Smuggler's Copter where
`decision_gate.rs` did not: the Copter *does* hit a decision row (the incidental
`Effect::DiscardCards` inside its unconditional `Sequence`), so a single "does this def carry any
decision at all" test passes it. `t_channels_are_not_interchangeable` pins both halves of that.

### 2.2 Measured populations

| channel | oracle-positive defs | no DSL evidence | of which effectively-`Complete` |
|---|---:|---:|---:|
| `may` | 285 | 265 | **72** |
| `choose` | 116 | 44 | **2** |
| `up_to` | 70 | 46 | **10** |

Union of effectively-`Complete` defs dropping ≥1 channel: **80** — the `BASELINE` size and the
`COMPLETE_DROPPED_UNION` pin.

### 2.3 Suppressions — every one explicit and reasoned

Six `RECORDED_STRUCTURAL_EVIDENCE` rows, each with a CR cite and a written reason, for DSL
constructs that express optionality **structurally** and carry no morpheme the stem rule can see.
Measured effect on the `may` channel's `Complete` population: **90 → 80** (the three `…Unless…`
variants, chiefly the ten shock lands' `EntersTappedUnlessPayLife`) **→ 72** (`unless_condition`
non-null). So the suppressions account for **18 real defs**, not a rounding.

Two false-positive classes the brief anticipated were **measured** rather than assumed:

* *reminder text* — stripped (CR 207.2). **Load-bearing**: revert row V1 reddens three tests.
* *"may not"* — **zero occurrences corpus-wide**. No suppression written; the absence is pinned by
  `t_may_not_is_measured_absent` so the claim fails the day one is authored instead of rotting.

### 2.4 The inverse-method census found a real recall hole

Per dispatch hygiene 6, the scanner was checked against the *type*, not against the author's
memory of the type. **`CardFace` has its own `oracle_text`** (`card_definition.rs:30-44`) and a
`CardDefinition` can carry two of them (`back_face` :77, `adventure_face` :114). The first draft
read `def.oracle_text` alone and was **blind to every transformed face and every Adventure half**
— the same shape of hole as `OOS-CARDS2-7` itself.

Fixed structurally: harvest every string under an `oracle_text` **key** at arbitrary depth, so a
future face slot joins automatically. Measured: **19** defs expose more than one `oracle_text`.

**And the widening added ZERO offenders** — no effectively-`Complete` def carries a decision
marker on a non-front face its front face does not already carry. That is a fact about today's
corpus, not about the walk (PB-DX26: a stable count is not evidence that nothing changed), and
`t_multi_face_printed_text_is_reached` keeps the walk honest either way.

### 2.5 Fail-closed proven END-TO-END on a real def

Not synthetically. `crates/card-defs/src/defs/lightning_bolt.rs`'s `oracle_text` was temporarily
given `"You may draw a card."`; **both** gates went red:

```
no_complete_def_drops_a_printed_choice_unrecorded  FAILED
  Lightning Bolt is NOT in BASELINE but drops {"may"}. may (CR 608.2 / 601.2b, printed "you
  may …": the controller may decline the action entirely)
t_complete_dropped_union_is_ratcheted              FAILED
  the effectively-Complete dropped-choice union moved to 81 from the pinned 80. GREW: …
```

Restored (`git diff crates/card-defs/` empty), both green.

### 2.6 Revert matrix — 12 rows, executed, watched

| row | what was broken | verdict | tests that fired |
|---|---|---|---|
| V1 | reminder-text stripping removed | **RED** | gate, union ratchet, `t_reminder_text_is_stripped` |
| V2 | `STEM_LEN` 3 → 6 (stem no longer reaches `chosen`) | **RED** | gate, union, closure pin, DSL non-vacuity |
| V3 | bare variant **strings** dropped from the surface walk (object-key-only) | **RED** | gate, union, DSL non-vacuity |
| V4 | `PROSE_FIELDS` suppression removed | **UNDISCRIMINATED** | — |
| V5 | one `RECORDED_STRUCTURAL_EVIDENCE` row deleted | **RED** | gate, union, liveness |
| V6 | `optional` counted by key **presence** instead of value `true` | **RED** (after repair, see below) | `t_optional_false_is_not_evidence` |
| V7 | `modes`/`unless_condition` counted by presence instead of non-null | **RED** | gate, union, baseline liveness |
| V8 | one `BASELINE` entry deleted | **RED** | gate |
| V9 | channels collapsed (any evidence discharges every channel) | **RED** | union, baseline liveness |
| V10 | effectively-`Complete` filter removed from the offender loop | **RED** | gate, synthetic probe |
| V11 | `printed_texts` reverted to front-face-only | **RED** | `t_multi_face_printed_text_is_reached` |
| V12 | `LEXICAL_EXCLUSIONS` emptied (`mayhem` readmitted) | **RED** | closure pin |

**V6 caught a real defect in this batch's own work, and it is PB-DP10 review finding #3 verbatim.**
`t_optional_false_is_not_evidence` was written with a *local* re-implementation of the truthiness
check, so flipping the production `dsl_expresses` to count key presence left it **green**: the
probe never executed the function it claimed to guard. Rewritten to drive `dsl_expresses` directly
with synthetic `DefFacts`; V6 then reddened. The row is also load-bearing for a second measured
reason: **only 5 defs in the whole corpus carry an `optional` key and all 5 have it `true`**, so
the live corpus cannot distinguish the two readings — nothing but that synthetic probe stands
between the gate and the regression.

**V4 is honestly UNDISCRIMINATED, not quietly dropped.** Removing the `PROSE_FIELDS` denylist
changes nothing, because no prose string in today's corpus spells any `may*`/`cho*`/`up to`
identifier (measured: the suppressed and unsuppressed surfaces yield the identical 23-element
channel-relevant set). It is kept because it is correct and free; the module doc says exactly
this rather than implying it was measured to matter.

---

## 3. PB-DX42a rider

See `docs/audits/mtg-characteristics-recursion-adjudication.md` §5.1's shipped banner for the full
record. Headline: **382 / 176 / 17** all re-derived and all matching; layer-querying subset pinned
exactly at one member along **two** independent axes; a third non-vacuity floor added that the
adjudication did not ask for (`>= 176` nodes with no `Static` ancestor — a `Static`-only walk
clears both stated floors while missing the entire nesting class); 10/10 revert rows RED.

**Disclosed correction to the adjudication's premise**: the structural axis is not a general proxy
for "reaches a layer query" — `Condition::ControlLandWithSubtypes` reaches
`characteristics_for_condition` without carrying a `TargetFilter`. It is absent from today's
conditioned population, so the two axes agree by coincidence; `t7` pins the coincidence.

---

## 4. OOS-CARDS2-7

`crates/engine/tests/core/completeness_deviation_scan.rs`, **+906 / −34**, 4 tests → **11**.

Floor reproduction: the seed's **35** `Complete` defs invisible to the shipped needle set is
**reproduced exactly at HEAD** by its own six needles — the filed number is neither stale nor an
estimate. The derivation rule (D1 completeness-note n-grams ≥10 notes; D2 ≥95% concentration in
already-marked defs over ≥8 defs; D3 shortest generative form) yields **34** needles reaching
**31** unmarked defs, and the two sets are **not nested** — 14 defs are seed-only, 10 derived-only,
union **45**.

**The derivation does not rediscover the seed's own six**, and the reason is the batch's second
headline: `todo` and `deferred` live in `// TODO:` **comments**, not in compiled `Completeness`
notes, so a derivation keyed on one declaration construct is short by exactly the failure mode
`OOS-CARDS2-7` names — reproduced inside the fix for it. The shipped needle set is therefore the
measured **union** of both tiers: **5 legacy + 34 Tier A + 3 Tier B net-new = 42 needles**, each
carrying its own measured `(prose_defs, marked, unmarked)` triple.

Three of the seed's six are already Tier A members (`dsl gap`, `not expressible`, and — the brief
got this one wrong — **not** `cannot be expressed`, which is absent from Tier A and is redundant
only because its single unmarked hit is also caught by `dsl gap`). The correction is in the
module doc rather than the brief's imprecise framing being repeated.

The 45 previously-invisible defs are frozen in `RECORDED_BASELINE`, each entry quoting the matched
needle(s) and the substantive fragment of the def's own note, with an exact two-direction ratchet,
a liveness test and a `MIN_REASON_LEN` floor. The freeze is documented as **mechanical, not an
oracle-text adjudication** — the correction PB-DP10's review forced on `decision_gate.rs`'s
`BASELINE`, applied at write time rather than after a review found it.

### 4.1 Two real defects found while deriving the fix

**`OOS-DX8-6` — the scan and the derivation were measuring different text.** The derivation runs
over *author prose* (comment bodies + `Completeness::*` note strings); the shipped
`has_deviation_language` matched the whole lower-cased **file**. Two derived needles collide with
Rust identifiers: `drawcards` is also `Effect::DrawCards`, `partial` is also
`Completeness::partial(`. Measured, and **independently reproduced by two implementations**:

| needle | surface | files | unmarked | precision |
|---|---|---:|---:|---:|
| `drawcards` | prose | 20 | 1 | **0.95** |
| `drawcards` | full source | 203 | 127 | **0.37** |
| `partial` | prose | 52 | 2 | 0.96 |
| `partial` | full source | 441 | 2 | 1.00 |

`drawcards` could never have survived D2's 95% floor under a fair measurement, and shipping it
against full source would have blown the 45-def freeze past 150 silently. Fixed by giving the gate
an `author_prose` extractor so the scan and the derivation measure the same text, plus a permanent
regression test. **The generalisable half: a needle set and the surface it is matched against are
two halves of one instrument, and deriving one while inheriting the other turns a 95% rule into a
37% gate.**

**`OOS-DX8-7` — a ratchet that could never redden.** The first-draft
`recorded_baseline_population_is_ratcheted` filtered its "live population" down to
`baseline.contains(stem)`, making the count vacuously equal `RECORDED_BASELINE.len()` on every
run. Caught by revert row **V4**: the real gate reddened with 106+ new offenders while the ratchet
stayed **GREEN**. Fixed; V4 re-executed post-fix (V4b) reddens on all three.

### 4.2 Revert matrix — 8 rows

| row | what was broken | verdict | tests that fired |
|---|---|---|---|
| V1 | one `RECORDED_BASELINE` entry removed | **RED** | main gate (named it) + ratchet |
| V2 | baseline entry naming a nonexistent file | **RED** | liveness + ratchet |
| V3 | a reason shortened below `MIN_REASON_LEN` | **RED** | liveness |
| V4 | scanner reverted to full-source | **RED** on the gate; **ratchet stayed GREEN** — the `OOS-DX8-7` bug | regression test + main gate |
| V4b | same revert, after the ratchet fix | **RED** | regression test + main gate + ratchet |
| V5 | the 3 Tier-B net-new needles dropped | **RED** | liveness + ratchet |
| V6 | `RECORDED_BASELINE_POPULATION` corrupted to 44 alone | **RED** | ratchet |
| V7 | baseline exclusion removed from `offenders()` | **RED** | synthetic gate-logic test + main gate |
| V8 | `completeness_note_bodies` wiring broken | **RED** | prose-extraction test |

V5's per-tier non-vaciuty assertion did **not** fire and that is disclosed rather than glossed: it
checks that each tier's vocabulary still exists in the corpus, not that `DEVIATION_NEEDLES` is
wired to it — a deliberate scope, stated in the test.

End-to-end proof executed on a real def: `windbrisk_heights.rs`'s only needle hit was edited
(`needs` → `requires`); the liveness test failed **by name** and the ratchet dropped 45 → 44; file
restored, `git diff` empty.

### 4.3 Six baseline entries that were read and frozen anyway — deliberately

`jadar_ghoulcaller_of_nephalia` and `birthing_pod` read as historical *"this gap is now closed"*
notes; `tyrranax_rex` is a collision with unrelated authoring-process prose; `steel_guardian` is a
synthetic non-card fixture; `korvold_fae_cursed_king` and `land_tax` read like the
faithful-decomposition pattern the reviewed `ALLOWLIST` exists for. **Reclassifying six defs on a
single reading is the unmeasured judgement this batch was dispatched to stop**, so each entry's
reason states what was found and the adjudication is filed as `OOS-DX8-8`.

---

## 5. Seeds

**Filed**: `OOS-CARDS2-7` itself — it had **no registry row anywhere** until this batch wrote one
(grep-confirmed absent first, per dispatch hygiene 5; its registry-of-record was the seed re-rank
memo §2.6) — plus `OOS-DX8-1..8`.

**Dispositions**: `OOS-CARDS2-7` **CLOSED**. `OOS-DP10-9` **RECORDED, not closed** — the gate makes
the class visible and says so in its own failure message; the class needs the owning engine PB
(audit §5 DP-12). `PB-DX42a` **SHIPPED**, adjudication §5.1 banner'd.
