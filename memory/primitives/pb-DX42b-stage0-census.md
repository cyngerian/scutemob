# PB-DX42b — stage 0 measurements (`scutemob-233`)

All figures below were taken at the merge base (`ffa4ee7e`) **before any production line changed**.
Every one is produced by walking `mtg_card_defs::all_cards()` (SR-36 — never a source grep) or by
executing a gate and reading its own output.

## 1. Baseline suite

**5,231 passed / 0 failed / 5 ignored** across **68** result-producing targets, residual list
empty. This **REPRODUCES PB-DX54's published close pin EXACTLY** — the fifth consecutive batch in
which an inherited pin reproduces with no correction owed (`OOS-DX51-5`'s failure mode did not
recur). Log retained for the byte-exact NAME set difference at close.

## 2. The DEMAND side — layer-querying `ContinuousEffectDef.condition`, re-measured

`pb_dx42a_continuous_condition_roster::t4_conditioned_census_report`, executed:

| quantity | at HEAD |
|---|---|
| total `ContinuousEffectDef` nodes | **382** |
| …with `condition: Some(_)` | **18 instances / 16 distinct cards** |
| …distinct `Condition` variants among them | **9** |
| …**layer-querying** (`YouControlNOrMoreWithFilter`) | **2 instances / 2 cards** |
| …layer-querying **and deck-legal `Complete`** | **1** — `indomitable_archangel` |

The second layer-querying member is **`the_world_tree`** (`Partial`, authored by PB-DX27), and its
filter reads **`Land`** where the Archangel's reads **`Artifact`** — so the supply census below does
**not** carry over to it. That coupling is what the v4 memo §3.1 finding 4 records and it still
holds: the deck-legal `Complete` demand population is **1**, unmoved since the adjudication measured
it, and it moves off 1 the day PB-DX9 promotes The World Tree.

The v4 memo's row-18 cell says the population is **2 at HEAD — re-measure**. It re-measures at
**2 instances / 2 cards**, and the cell is right. What the cell does not say, and the census does,
is that only **one** of the two is deck-legal, which is the number the seven pairs are counted
against.

## 3. The SUPPLY side — Artifact-moving Layer-4 modifications, re-derived as a FLOOR **and** a CEILING

### 3a. Every `LayerModification` that writes `chars.card_types`, enumerated from the apply site

> **↻ CORRECTED by the `/review`.** The heading below is accurate and §3d's conclusion drawn from
> it was over-stated: *"there is no sixth way for a continuous effect to move `card_types`"* is
> true of the modification `match`, and `layers.rs` writes `chars.card_types` at **six further
> sites OUTSIDE it**. Two were accounted for (the DFC face swap in §3c, the face-down class in
> §3d) and **four were mentioned nowhere: meld, Impending, Reconfigure and Living Metal.** All
> four were measured during the review and none moves `Artifact` on this corpus — meld's only
> corpus pair (`Hanweir`) produces a plain Creature, and Living Metal / Impending / Reconfigure
> insert or remove `Creature` only. **So the answer of 7 survives, but by measurement rather than
> by the published argument**, and the ceiling below is a ceiling on the **continuous-effect**
> class specifically, not on every way `card_types` can move.

Read off `rules/layers.rs`'s modification `match` rather than assumed — five arms write
`chars.card_types`:

| arm | layer | line |
|---|---|---|
| `Copy` | 1 | `:2139` (`chars.card_types = target_chars.card_types`) |
| `SetTypeLine` | 4 | `:2165` |
| `AddCardTypes` | 4 | `:2170` |
| `RemoveCardTypes` | 4 | `:2178` |
| `SetCardTypes` | 4 | `:2229` |

**`SetLandTypes` is NOT one of them** and its own doc comment says so (`:2243-2248`) — that is
PB-DX27's `OOS-ADJ-7` repair, and it is the reason `blood_moon` / `magus_of_the_moon` are correctly
absent from the census below. A batch that re-derived this list from the adjudication's four names
would have got the same answer for the corpus and the wrong answer for the *mechanism*.

`Copy` (the fifth) contributes **zero** corpus supply: `rules/copy.rs:110-112` records the measured
fact that `crates/card-defs/src/defs` contains zero occurrences of `EffectLayer::Copy`.

### 3b. The corpus census, by payload rather than by variant name

Walking `all_cards()` and collecting every `SetCardTypes` / `SetTypeLine` / `AddCardTypes` /
`RemoveCardTypes` payload found at any depth in the serialized def gives **18** defs. Of those,
the ones whose payload actually **moves `CardType::Artifact`**:

| card | modification | completeness | direction |
|---|---|---|---|
| `blinkmoth_nexus` | `AddCardTypes=["Artifact","Creature"]` | **Complete** | **adds** → under-count |
| `inkmoth_nexus` | `AddCardTypes=["Artifact","Creature"]` | **Complete** | adds → under-count |
| `darksteel_mutation` | `SetCardTypes=["Artifact","Creature"]` | **Complete** | adds → under-count |
| `eaten_by_piranhas` | `SetCardTypes=["Creature"]` | **Complete** | **removes** → over-count |
| `kenriths_transformation` | `SetCardTypes=["Creature"]` | **Complete** | removes → over-count |
| `imprisoned_in_the_moon` | `SetTypeLine={"card_types":["Land"],…}` | **Complete** | removes → over-count |
| `vraska_betrayals_sting` | `SetCardTypes=["Artifact"]` | `Partial` | **excluded — not deck-legal** |
| `oko_thief_of_crowns` | `SetTypeLine={"card_types":["Creature"],…}` | `KnownWrong` | **excluded — not deck-legal** |

The remaining defs in the 18 move `Creature` or `Land` and never `Artifact`:
`awaken_the_ancient`, `creeping_tar_pit`, `den_of_the_bugbear`, `purphoros_god_of_the_forge`,
`arixmethes_slumbering_isle`, `athreos_god_of_passage`, `destiny_spinner`,
`iroas_god_of_victory`, `tatyova_steward_of_tides`, `wrenn_and_realmbreaker` — **ten names
carrying eleven payload instances**, because `arixmethes_slumbering_isle` carries two.

> **↻ TWO CORRECTIONS by the `/review`, and the first is this document's own subject matter.**
> The list above originally included **`druid_class`**, which carries **no type-writing payload
> at all**: its only `AddCardTypes` occurrence is inside a `Completeness::partial(...)` **note
> string**. That is `OOS-DX53-2`'s exact shape — **compiled prose counted as a declaration** —
> committed inside a document whose own method paragraph says *"No def source was grepped
> (SR-36)"*, because the serde walk that produced it descends into every string field. And with
> `druid_class` removed the reconciliation inverts: the parenthetical read *"eleven names for ten
> rows"* and the truth is **ten names for eleven instances**. Neither correction moves 18 / 8 / 7.

**For a REPLACEMENT modification the payload is the whole answer, so "does not contain Artifact" is
as much a change to Artifact as "contains Artifact" is.** That is the adjudication §2.5's own
recorded blind spot, obeyed here rather than re-learned: a "payload contains `Artifact`" filter
would have found three and called it measured.

### 3c. The non-continuous-effect supply source

`thaumatic_compass` — **Complete**, a DFC whose front face is an `Artifact` and whose back face
(Spires of Orazca) is a `Land`. The face swap happens at `layers.rs:219`, a **pre-loop base rewrite**,
not a Layer-4 continuous effect, so no `LayerModification` census of any shape can see it. It is
its own test for exactly that reason.

### 3d. The answer

**Deck-legal `Complete` live-wrong pairs: 7 — the adjudication's figure REPRODUCES EXACTLY**, and
this is the first supply cell in this queue's recent history that is a **ceiling** as well as a
floor rather than a floor alone:

* **floor** — every one of the seven was re-derived here, from the payloads, without consulting the
  adjudication's list;
* **ceiling** — the enumeration in 3a is over the *apply site's own `match` arms*, so there is no
  sixth **`LayerModification`** way to move `card_types`, and `Copy` (the one arm the corpus does
  not use) is measured at zero. **Scope of that ceiling, corrected by the `/review`**: it bounds
  the CONTINUOUS-EFFECT class. `layers.rs` also writes `chars.card_types` at six sites outside the
  `match` — the DFC face swap (§3c, and its `Complete` Artifact-mover enumerates to exactly one),
  meld, face-down, Impending, Reconfigure and Living Metal — and the last four were measured and
  move `Creature` only on this corpus.

The ceiling holds **only for the bounded class**. The CR 708.2a face-down class is genuinely
unbounded — any `Complete` card can be played face down, and `layers.rs:333` replaces `card_types`
with exactly `{Creature}` — so it is **stated and not tallied**, exactly as the adjudication has it.

**Hosts, measured**: `Complete` printed artifact creatures = **28**; `Complete` printed artifact
lands = **2**. Both reproduce §2.3's figures. Four of the seven supply cards are Auras and need one
of these.

## 4. Coverage flip prediction, NAMED before regeneration

**0 flips. Live coverage stays at 1,140 / 1,803 = 63.2%.**

The reason, per def rather than in aggregate: all seven supply cards and the one demand card are
**already `Complete`**, so none of them can flip up; nothing in this batch removes an engine
capability, so none can honestly flip down; and the batch authors no card, so no `partial` def
becomes newly satisfiable. `the_world_tree` stays `Partial` — its blocker is
`Effect::SearchLibrary`'s missing count field (PB-DX9), which this batch does not touch.

If any card-def file is edited at all it will be **comment-only** (a blocker note this batch
falsifies — PB-DX27's rule that a note is a claim), and the `Completeness` marker will be checked
by `git diff` over the marker rather than inferred from an unchanged total (PB-DX26's lesson that a
stable COUNT is not a stable SET).

## 5. Wire

See `pb-DX42b-wire-prediction.md`, committed at `d90b7994` before this file. HASH **85** /
PROTOCOL **44** both predicted UNMOVED; closure type counts MEASURED at **132** / **98** by raising
each gate's `MIN_CLOSURE_TYPES` to 9999 and reading its own panic text; the counterfactual for the
rejected stored-field design verified by execution.
