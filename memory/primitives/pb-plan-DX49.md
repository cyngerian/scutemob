# PB-DX49 — implementation plan

Task `scutemob-220`. Subject seed **OOS-RR4-1**, rider **OOS-RR4-3**.
Read `memory/primitives/pb-DX49-execution-notes.md` §0 first — the wire prediction is already
committed (`57d1dc42`) and must not be re-litigated, only gate-confirmed.

---

## 1. The defect, restated from the CR rather than from the seed

CR 714 verbatim (MCP `get_rule`, June 2025 wording):

- **714.2d** — *"A Saga's final chapter number is the greatest value among chapter abilities it
  has. If a Saga somehow has no chapter abilities, its final chapter number is 0."*
- **714.3a** — *"As a Saga **without the read ahead ability** enters the battlefield, its
  controller puts a lore counter on it."* — **note there is NO "with one or more chapter
  abilities" clause here.**
- **714.3b** — *"As a player's precombat main phase begins, that player puts a lore counter on
  each Saga they control **with one or more chapter abilities**."*
- **714.4** — *"If the number of lore counters on a Saga permanent **with one or more chapter
  abilities** is greater than or equal to its final chapter number, and it isn't the source of a
  chapter ability that has triggered but not yet left the stack, that Saga's controller
  sacrifices it."*

The engine reads the **printed** card definition (`def.effective_abilities(is_transformed)`) at
every site and never consults the layer axis, so a permanent whose abilities are blanked keeps
accruing lore counters (714.3b), keeps firing chapter triggers (714.2b) and is sacrificed anyway
(714.4).

**Two blanking channels:**

1. **Layer-6 `LayerModification::RemoveAllAbilities`** (CR 613.1f) — the permanent keeps its
   subtypes, so it **is still a Saga**, with zero chapter abilities.
2. **CR 708.2a face-down** — *"no text, no name, **no subtypes**, and no mana cost"* — so a
   face-down permanent is **not a Saga at all**.

**(PB-DX43 added a third modification-level channel, CR 305.7 `SetLandTypes` with a basic
payload. It cannot reach a Saga in practice — a Saga is an enchantment, and CR 305.7 is scoped
to lands — but it must be covered by construction, not by that argument. That is why the
blanking classification delegates to `layers::modification_blanks_abilities`, which is
exhaustive over `LayerModification` with no wildcard arm: a fourth channel is a compile error.)**

### 1.1 The correction this batch owes to its own seed row

`OOS-RR4-1` says: *"A fix to the first three alone leaves a blanked Saga still taking its ETB
counter and still firing chapter I."* **The ETB half of that sentence is wrong on the CR for
channel 1 and right for channel 2**, and the difference is load-bearing:

- Channel 1 (Layer-6 blank): the permanent **is** a Saga and lacks read ahead, so **CR 714.3a
  still puts a lore counter on it.** Suppressing that counter would be CR-*wrong*, and
  observably so: if the blanker later leaves, a Saga that entered with 1 lore counter must
  resume at chapter II, not fire chapter I. The chapter never triggered while blanked (CR 714.2b
  needs the ability to exist at the moment counters are put on), which is exactly the correct
  outcome and is what keeping the counter produces.
- Channel 2 (face-down): the permanent has **no subtypes**, so it is not a Saga, so **714.3a
  does not apply** and no counter is placed.

So site 4 splits into two questions that must be answered separately: *"is this a Saga"*
(714.3a, counter) and *"does it retain chapter abilities"* (714.2b, triggers). Both are answered
by the one query, from two of its fields. This is recorded as a correction to `OOS-RR4-1`'s own
claims, per AC 7284.

---

## 2. Shipped shape

### 2a. One blanking predicate — `rules/layers.rs`

```rust
/// CR 613.1f / CR 305.7 / CR 708.2a — does anything leave this permanent with no abilities?
pub fn abilities_are_blanked(state: &GameState, id: ObjectId) -> bool
```

Body, in order:

1. **CR 708.2a**: `state.fizzle_object(id)` → `obj.status.face_down && obj.face_down_as.is_some()`
   → `true`. (The `face_down_as` conjunct is the same one `layers.rs:329` and
   `replacement.rs:2126` already use to distinguish morph/manifest/cloak from Foretell/Hideaway's
   unrelated `face_down` usage — do **not** invent a second spelling.)
2. **Continuous-effect scan**: exactly IG-1's existing block —
   `state.continuous_effects.iter().filter(is_effect_active).any(|e| modification_blanks_abilities(&e.modification, &chars) && effect_applies_to_object(state, e, id, obj_zone, &chars))`,
   with `chars` taken from the object's **stored** characteristics (`obj.characteristics.clone()`),
   never from `calculate_characteristics` — that is IG-1's deliberate choice and this function
   must not change it.
3. A missing object yields `false` (a fizzle, not a blank).

Then **refactor IG-1** in `replacement.rs::queue_carddef_etb_triggers` to call it. Its two
existing early-return blocks (the CR 708.3 face-down return and the IG-1 scan) both
`return Vec::new()` before anything else runs, so collapsing them into
`if layers::abilities_are_blanked(state, new_id) { return Vec::new(); }` is behaviour-identical.
**There must be exactly one blanking predicate in the tree when this lands** — a second one is
the finding, not the fix.

### 2b. The Saga query — new module `crates/engine/src/rules/saga.rs`

```rust
pub struct SagaView {
    /// (ability_index, chapter) for each chapter ability the permanent RETAINS after the
    /// layer axis is consulted. Indices are into `effective_abilities(obj.is_transformed)`
    /// — the same index space every CardDef ability-index consumer resolves against
    /// (CR 712.8d/e, PB-RS4).
    pub chapters: Vec<(usize, u32)>,
    /// CR 714.3a Saga-ness. False for a face-down permanent (CR 708.2a: no subtypes).
    pub is_saga_permanent: bool,
}

impl SagaView {
    pub fn final_chapter(&self) -> Option<u32>;      // CR 714.2d, None == no chapters
    pub fn has_chapters(&self) -> bool;
    pub fn is_chapter_index(&self, i: usize) -> bool;
}

pub fn saga_view(state: &GameState, id: ObjectId) -> SagaView;
```

`saga_view` derivation:

- object absent, no `card_id`, or unregistered → empty view (`chapters: vec![]`,
  `is_saga_permanent: false`).
- `printed` = `def.effective_abilities(obj.is_transformed)` filtered to
  `AbilityDefinition::SagaChapter { chapter, .. }`, carrying its **enumeration index**.
- `face_down` = `obj.status.face_down && obj.face_down_as.is_some()`.
- `is_saga_permanent` = `!printed.is_empty() && !face_down`.
- `chapters` = if `layers::abilities_are_blanked(state, id)` then `vec![]` else `printed`.

Because face-down implies `abilities_are_blanked`, **both blanking channels answer the same
query** and cannot disagree.

**Stated residual (file as a seed, do not fix here):** `is_saga_permanent` uses *printed chapter
abilities* as the Saga-ness proxy, which is the proxy all five sites already use — it is not the
layer-resolved `SubType("Saga")`. A type-setting effect that strips the Saga subtype without
blanking abilities (`imprisoned_in_the_moon`'s `SetTypeLine`) would leave the proxy saying "Saga"
where CR 205.3h says otherwise. Population is 0 at the only site that reads it (714.3a runs as a
permanent *enters*, and no corpus effect attaches an Aura at that instant). Do not widen the
query into `calculate_characteristics` — that is the lowering the design mandate rejects.

### 2c. The five behavioural sites

| # | site | today | becomes |
|---|---|---|---|
| 1 | `sba.rs` `check_saga_sbas` filter (CR 714.4 final chapter) | `def.effective_abilities(..)` max chapter | `saga::saga_view(state, *id).final_chapter()` |
| 2 | `sba.rs` chapter-still-on-stack guard (CR 714.4) | `def.effective_abilities(..).get(idx)` is a `SagaChapter` | `saga::saga_view(state, saga_id).is_chapter_index(*ability_index)` |
| 3 | `turn_actions.rs` `precombat_main_actions` (CR 714.3b) | `.any(SagaChapter)` | `saga::saga_view(state, *id).has_chapters()` |
| 4 | `replacement.rs` `apply_self_etb_from_definition` (CR 714.3a) | `has_saga_chapters` gates **both** the counter and the triggers | counter gated on `view.is_saga_permanent`; triggers gated on the view's `chapters` (via site 5) |
| 5 | `replacement.rs` `fire_saga_chapter_triggers` (CR 714.2b) | enumerates `def.effective_abilities(..)` | enumerates `saga::saga_view(state, saga_id).chapters` |

**Site 5 loses its `def: &CardDefinition` parameter** — the view derives the same list from the
registry, so keeping `def` would leave two parallel enumerations of the index space, which is the
producer/consumer drift `pb_rs4_face_aware_residuals` exists to police. All **three** existing
call sites in the test tree register the def in the state's registry, so the query resolves; they
are edited in place (**modifications, not renames or removals** — disclose them as such in the
delta).

One behavioural delta on a fizzle path, disclosed rather than discovered later: when the object
has already departed, the old body still enumerated the passed `def` and pushed triggers naming a
dead id; the view returns `chapters: vec![]` and pushes nothing. That is the CR-correct answer
(a departed Saga has no ability to trigger) and must be pinned by a probe.

### 2d. Deliberately EXCLUDED — CR 113.7a

`resolution.rs:2194` and `:2225` resolve a chapter ability that is **already on the stack**.
CR 113.7a: *an ability on the stack exists independently of its source* — blanking the Saga after
the trigger went on the stack does not counter or change it. Those two sites keep reading the
printed def, and must **gain a source comment saying so plus a roster row asserting they are not
consumers of the query**, so a later batch cannot "finish the job" by wiring them.

---

## 3. Census (AC 7282) — inverse method, treat §1g's grep as a FLOOR, PRINT from a test

New `crates/engine/tests/core/pb_dx49_saga_blanking_roster.rs`:

- **r1 — Saga-side, structural**: walk `all_cards()` (SR-36: enumerate, never grep) and collect
  every def declaring `AbilityDefinition::SagaChapter` on either face. §1g floor is **4**.
- **r2 — Saga-side, INVERSE (oracle-text axis)**: every def whose `oracle_text` (front face **and**
  every `CardFace`'s own text — PB-DX8's correction) prints a chapter symbol / "Sacrifice after"
  wording, minus r1's set. A non-empty residual is a def that prints a Saga and declares no
  chapters. The two axes do not nest; report both, ratcheted.
- **r3 — blanker-side, structural**: every def carrying a `LayerModification` that
  `modification_blanks_abilities` classifies as blanking, by walking the def, **not** by grepping
  the source. §1g floor is 13 by a bare grep and **9** by the qualified path at HEAD (the row's own
  2026-08-14 correction — the two moons kept the bare word in comments only). Deck-legal subset
  named.
- **r4 — pairs**: for each blanker × Saga pair, decide whether the blanker's `EffectFilter` can
  reach an enchantment. **Pair A** (`imprisoned_in_the_moon` × `binding_the_old_gods`) exists only
  because of **`OOS-DX20-10`** (that Aura declares `EnchantTarget::Permanent` for a printed
  "creature, land, or planeswalker") — assert that dependency **in the test**, keyed on the def's
  declared `EnchantTarget`, so fixing `OOS-DX20-10` reddens r4 rather than silently vacating a
  probe. **Pair B** (`reality_shift` × `binding_the_old_gods`) is unconditional (manifest ⇒ face
  down ⇒ CR 708.2a) and needs no card-def defect.
- **t_census_report** — PRINTS every population with `--nocapture`. Never transcribe a figure into
  prose that a test does not print (PB-DX8's rule, PB-DX28's MEDIUM).

**`urzas_saga` authoring is explicitly NOT taken** — it is `OOS-RR4-2`, ranked separately; the
famous Blood Moon × Urza's Saga pair is not deck-legal (`urzas_saga` is `partial`). State it.

---

## 4. Reachability (AC 7281) — the UI-4 standard

`crates/simulator/tests/pb_dx49_saga_blanking_channel.rs`, on a real `LocalGame` built through
the production pregame path where possible, **verdicts by resolution effects and counts, never by
offers**:

- **c1 — Layer-6 blanked, human channel**: a Saga under an active `RemoveAllAbilities` accrues
  **no** precombat lore counters, queues **zero** chapter triggers, and **survives** with
  `lore >= final_chapter`. Assert the lore count and the pending-trigger count, and assert the
  object is still on the battlefield after SBAs.
- **c2 — the same Saga un-blanked**: takes counters, fires chapters (assert the chapter's
  *resolution effect*, not the trigger's existence), and **is sacrificed** — `left: battlefield`
  → graveyard. c1 and c2 must differ in exactly one thing.
- **c3 — bot path**: the same discrimination with the bot provider driving, so the exemption is
  not an artefact of the human channel.
- **c4 — face-down channel via manifest** (`reality_shift`-shaped fixture): a manifested Saga
  takes **no** precombat lore counter and fires **no** chapter, and is never sacrificed.

Engine-level probes in `crates/engine/tests/primitives/pb_dx49_blanked_saga_sites.rs` cover each
of the five sites individually plus the CR 113.7a exclusion and the fizzle-path delta.

**Every probe asserts a COUNT, never `>= 1`** — PB-DX48's rule: a `>= 1` assertion passes on a
double-dispatch bug.

---

## 5. Rider OOS-RR4-3 (AC 7283) — re-verify FIRST, fix only what is wrong at HEAD

The row carries its own **2026-08-14 correction inverting finding (i)**. Honour it: re-read the
code before touching any document, and **preserve the correction history** rather than
overwriting it.

- **(i)** `docs/mtg-engine-corner-cases.md:468` — "Blood Moon's type-change applies in Layer 4,
  which strips Urza's Saga's printed chapter abilities." Verify at HEAD whether PB-DX43's
  relocation of CR 305.7's ability loss into the Layer-4 `SetLandTypes` arm makes this sentence
  **correct**. If it does, **do not "fix" it**; record that the row's original finding is
  withdrawn and its correction stands.
- **(ii)** `CLAUDE.md`'s "32 COVERED, 4 GAP" — re-measure the audit table and fix whichever
  number is actually stale at HEAD.
- **(iii)** `docs/mtg-engine-corner-case-audit.md:73`'s remediation note ("requires Saga
  gained-ability tracking"). Re-verify against `OOS-RR4-2`'s finding that the primitive exists.
- **Row 36 status**: update to **engine half covered, card half gated on `urzas_saga` authoring**
  — **NOT `COVERED`**.

---

## 6. Gates (AC 7284)

Against the **FINAL** tree, after the `/review` fix cycle, not before:
`cargo test --workspace --no-fail-fast` to a file; delta itemised by test NAME with 0 removals
and every leaver/rename disclosed; `clippy --workspace --all-targets -- -D warnings`;
`cargo fmt --check`; `tools/check-defs-fmt.sh`; coverage regenerated (`tools/authoring-report.py`)
with flips named — **0 predicted, 0 card-def edits expected**; `hash_schema` + `protocol_schema` +
`history_is_append_only` + `frozen_prefix_is_pinned` executed.
Every new gate and probe proven RED by an **executed** revert; the matrix goes in the execution
notes and any honestly UNDISCRIMINATED row is disclosed **in the test itself**, not only in
`memory/`.
