# PB-DX44 — the casts you cannot make

> Task `scutemob-215`. v4 queue rank 2 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 2,
> §2.4). Closes `OOS-DX29-3` / `-9` / `-12` / `-14`.
>
> This file is written **incrementally, in execution order**. §1 (wire predictions) was written
> and committed BEFORE any source line changed — that ordering is the point of the section, and
> the commit that adds it touches nothing under `crates/` or `tools/`.

---

## 0. Baseline, measured on this branch BEFORE any edit

`cargo test --workspace --no-fail-fast` to a file, at `3fb932ef` (branch fork point):

| figure | value |
|---|---|
| passing | **4,753** |
| failing | **0** |
| ignored | **5** |
| result-producing targets | **50** |
| distinct test NAMEs captured | **4,757** (0 duplicates) |

Reproduces PB-DX43's close pin (4,753 / 0 / 5, 50 targets) exactly. The 4,757-name set is
retained for the end-of-batch set-diff, so the delta is itemised by NAME rather than by
arithmetic (PB-DX7's rule).

---

## 1. Wire predictions — WRITTEN BEFORE ANY CODE CHANGE

The v4 memo publishes four wire cells for this row, each with a confidence. AC 6730 requires the
prediction to exist before the code and makes a mismatch a stop-and-re-scope event rather than a
number to be adjusted after the fact. PB-DX27 is the precedent that makes this worth doing
honestly: its brief predicted "expected wire impact NONE" and the gate refuted it.

| half | memo prediction | **this batch's prediction** | reasoning, stated before the edit |
|---|---|---|---|
| **Spree mode costs** (`OOS-DX29-14`) | none, HIGH | **PROTOCOL none / HASH none** | The whole change is `crates/simulator/src/legal_actions.rs::effective_cast_cost_with_additional` plus its callers. The chosen modes must *reach* that function, which is a signature change on a **simulator** function — `LegalAction`, `ActionParams` and `AdditionalCostPlan` are all simulator types, outside the `Command`/`GameEvent`/`Effect`/`Characteristics` closure `protocol_schema` walks. `ModeSelection.mode_costs` is already hashed (`hash.rs:6894`) and already read by `casting.rs`; nothing about it changes. |
| **Fuse targets** (`OOS-DX29-12`) | none, HIGH | **PROTOCOL none / HASH none** | `casting.rs` concatenates the existing `AbilityDefinition::Fuse { targets }` into the requirement list it already derives. `StackObject.target_requirements` is an existing hashed `Vec<TargetRequirement>` (PB-DX25c) whose **content** grows, not its type. `legal_actions::fused_right_half_declares_targets` is a simulator predicate being deleted. No type, no variant, no field. |
| **Half selector** (`OOS-DX29-9`) | **PROTOCOL**, HIGH | **PROTOCOL BUMP / HASH BUMP** | Two moves, and the memo names only the first. (a) `CastSpellData` gains a half selector field → it is `Command::CastSpell`'s payload, squarely inside the PROTOCOL closure. (b) Resolution must know which half to execute, and the precedent set for every other cast-mode flag (`was_overloaded`, `was_bargained`, `was_cleaved`, `cast_with_aftermath`, `was_cast_as_adventure`) is a field on `StackObject` — which is hashed. **So a HASH bump is predicted too, and the memo's cell is short by that half.** Recorded here as a prediction, not as a discovery after the gate. |
| **Pitch** (`OOS-DX29-3`) | none, HIGH | **PROTOCOL none / HASH none** | `CastSpellData.alt_cost` and the pitch payment path (`casting.rs:4209-4260`) both already exist and are already on the wire; `AltCostKind::Pitch` is an existing variant. What is missing is entirely above the engine: `params.rs` hard-codes `alt_cost: None`, `ActionParams` has no channel to carry the choice, and the offer layer emits no pitch alternative. `ActionParams` is a simulator type; the play-server request DTO is a play-server type. |

**Aggregate prediction: exactly ONE PROTOCOL bump and ONE HASH bump for the whole PB**, both
owned by the half-selector half. Every other half is wire-neutral.

**Stop condition, stated in advance**: if `protocol_schema` reports a bump the half-selector does
not explain, or reports **no** bump at all, the batch stops and re-scopes rather than re-writing
this table. Same for `hash_schema`.

### 1.1 Why the half selector is a field and not an `AdditionalCost`

`OOS-DX29-9`'s own row says the fix is "a half-selector on the cast action, not another
additional cost", and the CR agrees: CR 702.102a and CR 709.4 make each half of a split card a
spell with its own characteristics that you choose *at announcement* (CR 601.2a), which is not
what CR 118 additional costs are. Modelling it as an `AdditionalCost` would also put it in the
same bag as `AdditionalCost::Fuse`, and the two must be mutually exclusive — a "fused right half"
is not a thing.

### 1.2 The target-index hazard, named before it is hit

`turn.rs`'s right half declares `EffectTarget::DeclaredTarget { index: 1 }` — a **globally
offset** index, correct for a fused cast where the left half's one target occupies index 0. A
right-half-**only** cast announces one target, which lands at index 0, and the effect would read
index 1 and find nothing. This is a real consequence of the def-authoring convention
`resolution.rs:338-345` documents, and any half-selector that ignores it ships a spell that
resolves at nothing — the silent-wrong-game-state failure, not a refusal. Handling is designed in
§3 and gated by a roster assertion over every `AbilityDefinition::Fuse` carrier.

---

## 2. Census — every population re-derived at HEAD, on two axes, and PRINTED

`crates/engine/tests/core/pb_dx44_uncastable_roster.rs` (6 gates). Every figure below is
**copied from that test's own output**, not transcribed from a memo — PB-DX8's correction, which
PB-DX27 had to make a second time. `t_census_report` prints the whole membership under
`--nocapture`; if it ever disagrees with this table, it wins.

Each population is measured **forward** (the DSL construct `casting.rs` actually reads) and
**inverse** (the printed text, or the printed name). Dispatch hygiene 6, and four consecutive
batches that learned it: *a roster derived from one declaration construct measures that
construct.*

| population | forward | deck-legal | inverse | the filed row said | verdict |
|---|---|---|---|---|---|
| **Pitch** (CR 118.9) | 4 | **4** | 14 (4 deck-legal) | 4, named | **reproduces EXACTLY** |
| **Split right half** (CR 702.102a/709.4) | 3 | **3** | 39 (20 deck-legal) | "reachable population is 2" | **short by one** |
| **Fusable** (Fuse keyword) | 2 | **2** | — | 2 | reproduces |
| **Spree** (CR 702.172a) | 3 | **1** | 2 (1 deck-legal) | 1 | reproduces |

### 2.1 The pitch list is exact, and the batch's own grep was not

**`OOS-DX29-3` names four defs — `force_of_will`, `force_of_negation`, `force_of_vigor`,
`misdirection` — and the corpus walk returns exactly those four**, with no `partial` or `inert`
member hiding behind the deck-legal filter. "The filed site list is a FLOOR" has held for four
consecutive batches; this is a counterexample, the same way `OOS-ENG2-1`'s five-site census was
exact. Worth saying out loud, because a rule that is *usually* true is the kind that gets applied
without checking.

**And the census's first draft got it wrong in the direction the project has a rule against.**
`grep -l "AltCostKind::Pitch" crates/card-defs/src/defs/*.rs` returns **five** files, so this
batch's hand census recorded a fifth member (`force_of_despair`, `inert`) and `r1` was written to
assert five. The gate went red on its first run: `force_of_despair` declares **no** pitch
ability — `force_of_despair.rs:5` merely *mentions* `AltCostKind::Pitch` in a comment recording
what PB-AC5 shipped. **A source grep counts the token; `all_cards()` counts the declaration.**
That is SR-36's whole content, committed by this batch inside its own census, and caught only
because the figure was written as an executed assertion rather than as prose. The correction is
recorded in `r1`'s doc comment rather than silently fixed.

The inverse axis is 14 defs, of which 10 print CR 118.9's phrase and declare no pitch construct.
**Every one is non-deck-legal**, so none is a live-wrong card today — and `r1` asserts that
emptiness wrong-way-round, so the day one is promoted the gate says so. Two of the ten (`Gush`,
`Mindbreak Trap`) the hand grep also missed.

### 2.2 The half selector serves THREE defs, and `OOS-DX29-9` says two

`OOS-DX29-9` states "Population: 3 deck-legal `Complete` fuse-cost defs … so the reachable
population is 2", excluding `connive_concoct` because it is a deliberate data carrier with no
`Fuse` marker. That subtraction is right about **fusing** and wrong about **this batch**: the
half selector is CR 702.102a's *other* half, and `connive_concoct`'s right half (Concoct) is
exactly as uncastable as Burn and Tear. The keyword governs whether you may cast **both**; it has
nothing to do with whether you may cast the **right one**. So the row's two figures answer
different questions and its own summary sentence conflates them. `r2` pins both sets separately
for that reason.

The inverse `//`-name axis returns **39** defs, 20 of them deck-legal — MDFCs, Adventures, Rooms
and Aftermath split cards. Deliberately over-broad: the question is "which printed two-halved
cards can a player not cast a half of", and a needle narrowed to `AbilityDefinition::Fuse` could
never surface a two-halved card the engine models some other way. What it shows is that the
right-half gap is confined to the DSL's `Fuse` carrier — the Aftermath and Adventure halves have
their own (separately reachable, separately gapped) channels.

### 2.3 A new finding the inverse Spree axis produced

**`smugglers_surprise` carries `KeywordAbility::Spree` and declares no `mode_costs` at all.**
`casting.rs:2983-2987` refuses that cast outright ("spree spell has no per-mode costs defined in
ModeSelection"). It is `partial`, so the defect is latent — but it is `galadhrim_brigade`'s
marker-without-cost shape recurring on the one mode-cost mechanic that lives under a different
enum and was therefore outside `pb_dx29_additional_cost_roster::R2`'s eight-kind table. *A gate
written for one variant measures that variant*, arriving one enum over from the gate written to
generalise it. Pinned wrong-way-round by `r4`, filed as **`OOS-DX44-1`**.

---

## 3. Rider disposition — every seed naming PB-DX44 as host, decided in writing

The brief names seven riders across three groups. AC 6732 requires each to be TAKEN with a probe
or DEFERRED with a reason written into its registry row. Nothing is left implicit.

| rider | disposition | reason |
|---|---|---|
| **`OOS-DX29-13`** (a wrong `CardId` is a silently rider-less offer) | **TAKEN** — `pb_dx44_uncastable_roster::r6` | This batch builds its fuse and right-half fixtures against `turn.rs`, the exact def PB-DX29 lost three probes to. Without the gate this batch pays that cost a fourth time. **And executing the row's own prescribed fix refuted it**: the equality it asks for fails on **50** defs, in four classes, so the gate ships as a pinned floor and the row's prescription is corrected. Two genuine typos found (`skrevls-hive`, `laez-el-…`), filed as `OOS-DX44-2`. |
| **`OOS-DX29-4`** (hybrid/Phyrexian pips free in the additive rider arms) | **DEFERRED**, gap RECORDED | The Spree arm this batch mirrors is a **ninth** member of that class, and the mirror is deliberate for the reason the existing arms document: `effective_cast_cost_with_additional` must predict what `casting.rs` charges, and a provider that "corrected" the engine here would over-tap, fail to spend the surplus, and turn a silent undercharge into a visible refusal. Fixing it means teaching all nine `casting.rs` arms and *then* un-mirroring the provider, in that order — an engine behaviour change that has no deck-legal member today and no business sharing a commit with this batch's wire bump. |
| **`OOS-DX29-10`** (a hybrid rider pip makes `repeated_cost_max_count`'s bound an under-report) | **DEFERRED**, coupled | The row says it explicitly: teaching `casting.rs` to charge hybrid rider pips is what makes this one LIVE, so the two close in one commit or neither does. Deferring `-4` therefore defers `-10` by construction, not by choice. |
| **`OOS-DX29-11`** (escalate count and `modes_chosen` are two unreconciled channels) | **DEFERRED** | Escalate's count and Spree's per-mode costs are different mechanics that happen to both read `modes_chosen`; this batch makes the modes reach the **cost** function and changes nothing about the escalate derivation. 0 deck-legal escalate defs, pinned by `pb_dx29_additional_cost_roster` R4. |
| **`OOS-DX29-17`** (an over-announced escalate count is charged in full, clamped in effect) | **DEFERRED** | Same 0-member population. The fix is a clamp in `casting.rs`'s escalate charge, adjacent to but not inside the mode-cost region this batch touches, and it would move an engine charge with no member to prove it on. |
| **`OOS-DX29-6`** (four mechanics share one `Sacrifice` entry with no arbitration) | **DEFERRED** to PB-DX57 | Nothing in this batch reads `AdditionalCost::Sacrifice`. |
| **`OOS-DX29-15`** (`casting.rs` and `resolution.rs` make the entwine decision from two sources) | **DEFERRED** to PB-DX57 | This batch *does* mirror `casting.rs`'s entwine branch into the mode-cost arm — but that mirror is a third reader of the same decision only in the sense that the auto-tap now agrees with the charge, which is the defect being fixed rather than a new instance of `-15`. The `casting.rs`/`resolution.rs` divergence is untouched and still latent. |

---

## 4. Stage 1 — Spree mode costs, and CR 702.102d fuse targets

Commit `0a14c42c`. **PROTOCOL 37 / HASH 76 both gate-executed and UNMOVED**, exactly as §1
predicted for these two halves.

### 4.1 `OOS-DX29-14` — the eighth site of PB-DX29's own seven

`effective_cast_cost_with_additional` gains `modes_chosen: &[usize]` and charges
`ModeSelection.mode_costs`, mirroring `casting.rs:2940-2991` clause for clause — **including
the `entwine_paid` override that charges EVERY mode rather than the chosen ones**, which a
mirror written from the seed's one-sentence description would have missed.

**The load-bearing link is not the arithmetic, it is the argument.** `auto_tap_commands_for`
passes `&cast.modes_chosen` **verbatim off the `Command` it is about to apply**, which is the
same value for the human path (`submit`) and the bot path (`advance`, where `params.rs` has
already substituted `spell_default_modes`). Passing `&[]` there — the obvious first draft, since
the caller is "just funding a cast" — leaves the defect alive on both paths, and the revert row
proves it: that one substitution reddens three end-to-end probes with `InsufficientMana`. This is
PB-DX29's own lesson arriving one function over: *the function auto-tap asks is where the defect
lives, and the brief that names only the arithmetic has named half of it.*

`insatiable_avarice`, the only deck-legal `Complete` Spree def and previously uncastable from
**every** channel, now casts — proven by resolution effect (mode 1 makes a player draw 3 and lose
3 life) rather than by the offer.

**The hybrid/Phyrexian omission is mirrored deliberately** and the Spree arm is recorded as a
**ninth** member of `OOS-DX29-4`'s undercharge class (§3). A provider that "corrected" the engine
here would over-tap, fail to spend the surplus, and turn a silent undercharge into a visible
refusal — the trade the existing rider arms already document.

### 4.2 `OOS-DX29-12` — one derivation, two consumers

`card_def_target_requirements` — the function `handle_cast_spell` and
`queries::spell_target_requirements` **share** so they cannot drift — appends the
`AbilityDefinition::Fuse { targets }` after the left half's, preserving the global index contract
`resolution.rs:338-345` documents. `legal_actions::fused_right_half_declares_targets` and its
suppression site are DELETED with their mechanism, per the PB-DX20/PB-DX21 precedent.

**The SR-5 keyword registry caught `queries.rs` as an unregistered `Fuse` handling site on the
first full run** — the third consecutive batch in which that gate finds what static reading of
the brief missed (PB-DX20's `queries.rs`/Enchant, PB-DX23's `queries.rs`/Dredge, now this).

### 4.3 The hole Stage 1 left, found by the coordinator and not by Stage 1's own gate

**Deleting the suppression is not the same as making the offer honest, and the difference is a
guaranteed 422.** `view.rs::action_target_requirements` calls
`spell_target_requirements(state, card, &[], None, **false**)`, and `ActionBar.svelte`'s
`resolvedTargetSlots` returns that static list for any card without per-mode targeting. The
browser's stage order is `ValuePrompt` → `CostPicker` → `TargetPicker`, so a human ticks Fuse in
stage 2 and is then asked in stage 3 for **one** target while `casting.rs` now demands **two**.
Clean offer, server rejection — **the exact SR-38 defect this batch exists to delete, created by
this batch**, which is what PB-DX29 recorded about itself when it chose to gate the offer instead.

Stage 1's own `t4` does not catch it, and the reason is worth naming: it asserts
`spell_target_requirements(.., fuse: true).len() == 2` and
`(.., fuse: false).len() == 1`. **Both are true, and neither is about the channel** — nothing on
the browser path ever passes `true`. *A differential between two arguments of one function proves
the function, not the caller.* PB-DX20's durable lesson, in the file that cites it.

Closed in Stage 2b together with the pitch and half-selector client work, since all three need the
same offer-side plumbing.

---

## 5. Stage 2a — the right-half cast, and the batch's one wire bump

Commit `8eec3696`. **PROTOCOL 37 → 38 / HASH 76 → 77**, both taken from the failing gates' own
output and both **predicted in writing in §1 before any code changed**. §1's stop condition (a
gate that moves in a way the half selector does not explain, or does not move at all) never fired.

### 5.1 Why the selector is an `AltCostKind` variant and not a `CastSpellData` field

Two reasons, and the second is the one that makes it right rather than merely cheap.

**Cheap**: `CastSpellData` derives no `Default` and its **793** construction sites each list every
field, so a sixteenth field is a 793-line mechanical diff — a change that would bury this batch's
actual content under churn and make the `/review` unreadable.

**Right**: `AltCostKind` is already the engine's *which face/half/mode am I casting*
discriminator, not merely its alternative-cost list, and it says so itself. Three of its members
carry doc comments denying they are alternative costs — `Prototype` ("NOT an alternative cost,
ruling 2022-10-14"), `Adventure`, and above all **`Aftermath`, which is literally "cast the other
half of a split card"**. `SplitRightHalf` is Aftermath's sibling, one zone over. And
`OOS-DX29-9`'s own text asks for "a half-selector on the cast action, not another additional
cost", which an `AdditionalCost` variant would have violated while also colliding with
`AdditionalCost::Fuse` — a "fused right half" is not a thing.

`StackObject` gains `cast_right_half: bool`, which is the shape every other cast-mode flag on that
struct already has (`was_overloaded`, `was_bargained`, `was_cleaved`, `cast_with_aftermath`,
`was_cast_as_adventure`). It is hashed; that is the HASH half of the bump, and §1 predicted it
while the v4 memo's wire cell did not.

### 5.2 The stage's real risk was never the cost arm

The cost arm is four lines. **The target index is where a wrong answer is silent.**

`turn.rs`'s right half declares `EffectTarget::DeclaredTarget { index: 1 }` — a **globally
offset** index, correct for a fused cast where the left half's single target occupies index 0.
Cast alone, the spell announces one target at index 0, the effect reads index 1, and it resolves
**at nothing**: no error, no refusal, wrong game state. This is the "legal-but-wrong" class the
project ranks as its biggest pre-alpha risk, and it is reachable from a legal deck.

`resolution.rs` pads the **effect context** by the LEFT half's declared requirement count, using
the pre-existing `SpellTarget::unchosen_slot()` idiom. Three constraints, each of which a naive
padding would have broken:
* pad **after** the `is_target_legal` filter, or a dropped target shifts every index behind it;
* pad the **context only, never `stack_obj.targets`** — that vector is what CR 608.2b's fizzle
  check reads and what `GameEvent::TargetsAnnounced` publishes to clients, so a duplicated entry
  there would both mis-report and mis-fizzle;
* the offset is **computed** from `r3`'s pinned per-half counts, not remembered — which is the
  whole reason `r3` pins them by value rather than by a floor.

`connive_concoct` is the control case that makes the padding path observable on an empty
announcement: its right half declares **zero** targets.

### 5.3 A guard deliberately NOT written

The brief asked for a `SplitRightHalf` + `AdditionalCost::Fuse` rejection. It is **dead code**:
the pre-existing fuse block already rejects `alt_cost.is_some()` unconditionally, and
`cast_right_half` is *derived from* `alt_cost`, so the combined command has already returned
`Err` before that point. Documented in place rather than added — a second check guarding an
unreachable state is a claim about the code that is false.

### 5.4 A stated design residual, recorded as a decision rather than left to be found

`card_def_target_requirements` now takes **three booleans**
(`casting_with_aftermath`, `casting_with_fuse`, `casting_right_half`), of whose eight
combinations only four are legal, and the function's own doc says it does not re-validate the
mutual exclusion its callers guarantee. An enum would make the illegal states unrepresentable and
would be the better shape. It is **not** taken in this batch: the refactor touches the function,
`queries::spell_target_requirements`' public signature and ~10 call sites, and no fourth flag is
coming (pitch does not change a spell's target requirements), so the churn buys nothing this
batch needs. Recorded here as a decision, and filed, so that it is a choice rather than an
oversight if the `/review` raises it.

---

## 6. Stage 2b — the client half, and the pitch channel

Commits `0ebbd0b1`, `0efbcf0e`, plus the deferral pins. **PROTOCOL 38 / HASH 77 both gate-executed
and UNMOVED** — one wire bump for the whole PB, taken in Stage 2a.

### 6.1 The one line that made a whole engine branch unreachable

`params.rs` hard-coded `alt_cost: None` on every `CastSpell`. `casting.rs`'s pitch payment path
(`:4209-4260`) has existed since PB-AC5 and **nothing could reach it**. Four deck-legal `Complete`
defs printed an alternative cost no client could pay, and the fix on that axis is one field
forwarded.

`LegalAction::CastSpell` carries the alt cost so the hand loop emits **one action per castable
mode** — the `PayEcho` / `ChooseDredge` idiom, and PB-DX29's `(target, on_top)` pair idiom.
`AdditionalCostPlan.pitch` carries the eligible exile candidates, with eligibility **mirroring**
`casting.rs`'s own gate rather than re-derived (colour, not-the-spell-itself, in hand, life
payable, CR 118.9's `opponents_turn_only`).

### 6.2 Two affordability traps, either of which silently suppresses the feature

The hand loop gates casts on `can_afford(obj.characteristics.mana_cost)`.

* **Pitch**: a pitch cast's mana cost is `{0}`, and Force of Will's entire point is casting it
  when you *cannot* afford `{3}{U}{U}`. An offer left behind that gate appears exactly when it is
  least wanted and vanishes exactly when it matters.
* **Right half**: `Turn // Burn`'s printed `mana_cost` is the LEFT half's `{2}{U}`, while Burn
  costs `{1}{R}`. A player holding `{1}{R}` can legally cast Burn and would be offered nothing.

So `effective_cast_cost_with_additional` learns the alt cost too. **This is the same class of link
Stage 1 hit**: the function auto-tap asks is where the defect lives, and a brief that names only
the offer has named half of it.

### 6.3 The fuse hole closed on the precedent that already existed

`ActionOptionView` gains fused slot/min/max fields and `ActionBar.svelte` selects on whether
`paramsAcc.additional_costs` carries a Fuse entry. **`ModeOptionView::target_slots` exists for
exactly this reason** — slots that depend on an earlier stage's answer — so this is an instance of
a shape the file already had, not new machinery.

The regression probe is a differential over **the DTO the browser actually receives**. Stage 1's
`t4` compared `fuse: true` against `fuse: false` on the query; both assertions were true and the
channel was broken between them.

### 6.4 Two acceptance gaps the coordinator closed rather than accepted

* **Misdirection (T6).** Stage 2b recorded the fourth pitch member as a stated floor. The
  criterion names four and three is not four. Writing it then produced a second lesson: the
  probe's own doc justified it as *"the only member whose cost list has no `Cost::PayLife`"*, and
  an executed revert — making a life component mandatory in `offerable_pitch_plan` — reddened
  **four** tests instead of one. `force_of_will` is the only member that **pays** life; the claim
  was inverted 1-vs-3. Corrected in the test's own doc, along with an honest statement that T6
  isolates no branch T1-T5 miss.
* **The graveyard half (T7 + `r8`).** Deferred, with the coupling reason stated *and measured on
  both axes*: nothing is offered today, and the deck-legal `Complete` Escape population is
  **zero**. The seed's row argues the coupling and omits the population, and the distinction —
  latent versus unreachable — is the figure PB-DX29 learned to publish when its own "13 of 15
  kinds invisible" proved materially misleading.

---

## 7. Final measurements

| gate | value |
|---|---|
| tests | **4,797 / 0 / 5**, **53** result-producing targets (baseline 4,753 / 50) — includes the `/review` fix cycle's +2 |
| delta | **+44**, itemised by NAME below |
| removals | **0** genuine; **1 rename**, mandated by the criterion |
| PROTOCOL | **37 → 38**, gate-computed, predicted in writing |
| HASH | **76 → 77**, gate-computed, predicted in writing |
| coverage | **1,136 / 1,803 = 63.0%**, **0 flips** as predicted, self-dating churn reverted |
| clippy | `--workspace --all-targets -- -D warnings` clean |
| fmt | `cargo fmt --check` clean; `tools/check-defs-fmt.sh` clean (1,803 defs) |

### 7.1 The delta, by test NAME (set-diff of the two run logs)

4,757 unique names → 4,801. **45 additions, 1 rename, 0 removals.** (43 + the fix cycle's 2: the
frontend source gate and the pitch HTTP probe.)

| count | file |
|---|---|
| 10 | `crates/engine/tests/core/pb_dx44_uncastable_roster.rs` (new — `r1`-`r9` + `t_census_report`) |
| 8 | `crates/engine/tests/rules/pb_dx44_split_half_cast.rs` (new) |
| 7 | `crates/simulator/tests/pb_dx44_pitch_channel.rs` (new — T1-T7) |
| 6 | `crates/simulator/tests/pb_dx44_spree_mode_costs.rs` (new) |
| 4 | `crates/engine/tests/rules/pb_dx44_fuse_targets.rs` (new) |
| 4 | `tools/play-server/src/main.rs`'s `#[cfg(test)]` module |
| 3 | `crates/simulator/tests/pb_dx44_split_half_channel.rs` (new — R1-R3) |
| 1 | `pb_dx29_cost_kind_surface::p1e` (the rename's successor) |

**The one name that left the passing set is not a removal**:
`p1e_fuse_is_suppressed_while_its_right_half_targets_cannot_be_announced` became
`p1e_fuse_is_offered_and_its_target_count_matches_what_the_cast_validates` — same file, same
position, **subject inverted**, because the suppression it pinned is the thing this batch deleted.
The acceptance criterion mandated exactly this re-pointing. Stated rather than netted out, because
"+44 with zero removals" would have been a true number hiding a real edit.

---

## 8. `/review` — 1 HIGH, 3 MEDIUM, 8 LOW, all 12 taken

An Opus reviewer with a shell re-derived every headline figure against the shipped code (all
reproduced), ran the full suite (4,795 / 0 / 5 at 53 targets — the pre-fix-cycle figure, reproduced exactly), and executed five reverts
of its own — four confirming the claimed discrimination, including the "3 probes redden" figure for
the `modes_chosen` link.

### 8.1 The HIGH was the batch shipping its own subject matter, a third time

**The entire browser half of two of the four seeds had ZERO gate coverage.** The reviewer deleted
both `isFusedCast` branches from `ActionBar.svelte`'s slot/range derivation **and** the
`pitch={activeOption.costs.pitch}` prop, then ran `cargo test -p play-server`: **112 passed, 0
failed.**

That is worse than a missing test in two distinct ways. §4.3 of this file calls the fused-slot hole
*"the exact SR-38 defect this batch exists to delete, created by this batch"* — and the fix closing
it silently reopens on the next frontend edit. Dropping the `pitch` prop is a different failure:
`CostPicker` never renders, `merge_required_additional_costs` substitutes `plan.pitch.default`, and
**a human never chooses which card to pitch** — the acceptance criterion's "NON-DEFAULT pitched
card" becomes unreachable from the browser with everything green.

The project has a settled precedent (UI-4's `frontend/src` walk, UI-5's four gates, UI-6's needle
set), and this batch reached for it for the ENGINE half and not the CLIENT half. *A batch that
gates the half it finds interesting has gated half its work.*

### 8.2 A comment citing a gate that does not make the claim

`resolution.rs` justified the target-index padding with "`r3` pins that every corpus right half's
declared indices start at or after `left_count`". `r3` pins `(name, left_len, right_len)`
**counts**; it never walks the `Effect` tree for `DeclaredTarget` values. The invariant *is* covered
— behaviourally, by `split_half_cast::t1`/`t2`, which the reviewer proved by setting `wear_tear`'s
right half to `index: 0` and watching three tests redden — so this is a **wrong cite, not an
uncovered invariant**, sitting in the one comment guarding the batch's stated highest-risk line.
PB-DX7's `/review` M5 in the same shape: *a reason asserting documentation that does not exist.*

### 8.3 The riders were dispositioned in `memory/` and the criterion said registry

All seven rider rows were **byte-unchanged** on the branch; §3 of this file held the dispositions.
That is precisely the failure the v4 re-rank made its headline finding — 61 of 208 seeds unrowed
because work was filed into handoff prose — and dispatch hygiene 5's "the registry is ground truth".
Now written into each row. Note `OOS-DX29-13` is **TAKEN**, not deferred, and its refuted
prescription is corrected in its OWN row rather than only in `OOS-DX44-2`'s, so a reader of `-13`
no longer sees a prescription this batch disproved.

### 8.4 The refusal-channel A/B, measured because the reviewer asked and worth the answer

SIM-5, SIM-6 and PB-DX29 each published one; this batch published none, in a batch whose entire
framing is SR-38. Measured now.

**The standing gate** (`sim5_bot_cast_discipline`, seeds 0/7/42, 26 turns, 4 heuristic bots):

| class | seed | PB-DX29 | PB-DX44 |
|---|---|---|---|
| `InsufficientMana` on **activate** (auto-tap covers `CastSpell` alone) | `OOS-SIM6-3` | 76 | **76** |
| blocker refusals (`CrossPlayerBlock` + attacker-declaring-blockers) | `OOS-SIM5-3` | 27 | **26** |
| modal per-mode target slices unqueryable | `OOS-SIM5-5` | 2 | **4** |
| **cast-side refusals of any kind** | — | **0** | **0** |
| total | | 105 | **106** |

**The number that matters is the last-but-one row: cast-side refusals are still ZERO.** This batch
opened four new cast channels and none of them produces a refusal — which is the SR-38 property,
and the only one of these figures that is a claim about this batch rather than about the bot.

**The two that moved are not evidence of anything, and saying why is the point.** This batch grows
the offer list (up to three `CastSpell` actions per card where there was one), so a bot drawing
uniformly over that list **diverges from the first affected offer and plays a different game**.
Per-seed counts across an action-space change are not a like-for-like comparison — `OOS-DX22`'s own
recorded lesson, *a random bot's seed choice is not evidence*. Both moved classes are pre-existing
and named; no new class appears.

**The fuzz figures, reproduced from the reviewer's run**: RandomBot, 8 games from seed 44, 60 turns
— `250 / 19365 = 12.910‰` on branch against `206 / 19346 = 10.648‰` on main, threshold **30‰** on
the aggregate. Seeds 44-50 are byte-identical; **all +44 land in seed 51** (20/2216 → 64/2235 =
28.6‰), i.e. exactly one game diverged, for the reason above. Its rejection classes are
`InsufficientMana`, `InvalidTarget`, `AlreadyDeclaredBlockers`, `InvalidCommand`, `CrossPlayerBlock`
— all pre-existing. The heuristic bot moved the other way (1487 → 1395). **Recorded rather than
tuned away**: a single seed at 28.6‰ against a 30‰ aggregate threshold is a thing a future reader
should know is understood, not a thing to discover.

### 8.5 The remaining findings

* **LOW 5** — a right-half cast still runs the LEFT half's per-mode target branch: both
  `queries.rs` and `casting.rs` short-circuit `mode_targets` on `casting_with_aftermath` only. The
  two **agree**, so there is no offer-vs-cast drift; a shared latent CR 709.4 gap with zero corpus
  members. Recorded in `OOS-DX44-3`.
* **LOW 6** — **the dual of SR-38, and worth naming as a class**: `offerable_pitch_plan` requires an
  `ExileFromHand` component, so a life-only pitch ability would never be OFFERED though
  `casting.rs` would accept it. Every seed in the registry describes an offer the engine refuses;
  this is an offer withheld that the engine would honour. Latent (all four members exile).
  Recorded in `OOS-DX44-3`.
* **LOW 8** — a right half cannot express "can't be countered" (`AbilityDefinition::Fuse` has no
  such field, so the branch falls back to `def.cant_be_countered`). Correct today, now documented as
  a residual — the CR 709.4 timing residual got a comment and a seed row and this did not.
* **LOW 9** — the identity-guard widening's one behavioural change: a modal cast of a card with **no
  registry def** now yields `None` (unfunded) where it previously yielded the base cost. No member.
* **LOW 10** — the v4 memo stated two different "next dispatch" answers three lines apart
  (pre-existing; PB-DX43 left it, and this batch edited the adjacent line). Struck, not deleted,
  because the sentence after it explains why the memo names a dispatch at all.
* **LOW 11** — **criterion 3's "fused … from bot paths" is structurally unmet and is now stated as
  such rather than reported as met.** `merge_required_additional_costs` only ever defaults
  *required* costs, so no bot can announce the optional `AdditionalCost::Fuse`. Left-only and
  right-only ARE bot-reachable; fused is not, by construction, and making it so would mean teaching
  bots to want optional riders — a different batch. Documented at `targeting.rs`.
* **LOW 12** — `p4_fuse_is_offered_from_hand_and_never_from_the_command_zone` survives unchanged and
  green, preserving the CR 702.102a **zone** clause's independence from the target clause. The
  criterion asked for "preserved or explicitly re-homed" **in writing**; it was preserved and not
  written down. Written down here.

### 8.6 The fix cycle's outcome, and the defeat re-executed by the coordinator

`crates/engine/src/rules/{casting,queries}.rs`, `crates/simulator/src/legal_actions.rs`,
`tools/play-server/src/main.rs`. **+2 tests** (4,795 → **4,797**), no removals, PROTOCOL 38 /
HASH 77 gate-executed and unmoved.

* `test_frontend_action_bar_keeps_the_fused_slot_and_pitch_wiring` — the UI-4/UI-5/UI-6 source-scan
  idiom, pinning both the Fuse-first branch (twice: `resolvedTargetSlots` and `resolvedTargetRange`)
  and the `pitch={activeOption.costs.pitch}` thread to `CostPicker`.
* `test_dx44_pitch_cast_is_offered_and_resolved_over_http` — the full HTTP drive pitch was missing,
  with a NON-DEFAULT `ExileFromHand`. `view.rs`'s `pitch_prompt` / `color_word` had **no coverage at
  all** before it.

**The defeat was re-executed by the coordinator rather than taken on report**, three ways: the
reviewer's exact combined deletion, then the `pitch` prop alone, then the two `isFusedCast` branches
alone. All three redden; the tree was restored and `git diff --stat` over the frontend is empty.
Each half catches independently, which is what stops the gate from being satisfied by whichever
half a future edit happens to leave behind.

**LOW 5 was taken as code rather than as a note**: `cast_right_half` now joins
`casting_with_aftermath` in the per-mode `mode_targets` short-circuit at **both** sites
(`casting.rs` and `queries.rs`), verified behaviour-neutral by the full suite — no shipped card is
both modal and a fuse right half. The two sites had to move together for the same reason they were
worth fixing: they agree today, and fixing one would have created the offer-vs-cast drift the
absence of the fix did not have.
