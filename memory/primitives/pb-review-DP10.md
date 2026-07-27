# Primitive Batch Review: PB-DP10 — Widen the decision gate (stop the 277-def figure growing silently)

**Date**: 2026-07-27
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-158` · branch `feat/pb-dp10-widen-the-decision-gate-stop-the-277-def-engine-gues` · commit `76b4f1cd`
**CR Rules verified independently**: 404.3, 608.2d, 614.12/614.12a, 701.9/701.9a/701.9b, 701.39/701.39a, 701.57/701.57a, 115.7d, 118.12/118.12a, 106.12, 105.2
**Files reviewed**:
- NEW `crates/engine/tests/core/decision_site_walk.rs` (514 lines)
- NEW `crates/engine/tests/core/decision_gate.rs` (1,123 lines, 17 `#[test]`)
- EDIT `crates/engine/tests/core/main.rs`, `effect_choose_gate.rs`, `pb_rs1_roster_sweep.rs`, `tests/primitives/pb_dp9_effect_choice.rs`
- EDIT `docs/audits/decision-point-audit.md` (§3.1, §4.9, §5, §6, §8, §8.1, §10)
- Engine sites read for taxonomy verification: `effects/mod.rs` (`Proliferate` 4460, `discard_cards` 9319, `try_pay_optional_cost` 9257, `MayPayThenEffect` 4133, `CounterUnlessPays` 4187, `SacrificePermanents` 4210, `PutOnLibrary` 3444, `SearchLibrary` 3476, `WheelHand` 1214/1236/1266), `card_definition.rs` (`WheelDisposal`/`WheelDraw` 2497-2549)
- Card defs read against oracle text: `smugglers_copter.rs`, `shambling_ghast.rs`, `chaos_warp.rs`, `goblin_ringleader.rs`, `spymasters_vault.rs`, and the `Connive`/`PutOnLibrary` corpus grep

**Card-def edits in this batch**: **0** — claim verified. `rg 'PB-DP10|decision_gate|decision_site_walk'` across `crates/` returns six files, all under `crates/engine/tests/`. Nothing under `crates/engine/src`, `crates/card-types/src` or `crates/card-defs/src`.

## Verdict: **needs-fix**

The batch's headline technical finding is real, correctly diagnosed and correctly fixed: serde's external tagging makes a unit `Effect` variant a bare JSON *string*, every prior walk in this codebase matched object keys only, and a verbatim reuse would have reported **0** for `Effect::Proliferate`'s 25 `Complete` defs while looking green. `decision_site_walk.rs::json_contains_variant` fixes it, `T2` pins it in both directions, and the `PROSE_FIELDS` denylist that the string arm makes necessary is sound (I verified independently that **no** field named in `PROSE_FIELDS` holds an `Effect` or `Vec<Effect>`, so the denylist cannot over-suppress a real hit; and that `crates/card-types/src` carries **zero** `#[serde(rename|untagged|flatten|skip|tag)]` attributes and **zero** map types, so the two other blinding channels the brief asked about do not exist). The ratchet design (per-def exact row-set + exact union) is right, and the union check structurally catches the "walk went blind" failure that a per-def check alone would not. The wire-neutrality and SR-9a claims hold.

What it does **not** do is honour its own plan §5.3 discipline. I spot-checked baselined defs against oracle text and hit a class-D card on the second try: **Smuggler's Copter** encodes "you **may** draw a card. If you do, discard a card." as an unconditional `Effect::Sequence(vec![DrawCards, DiscardCards])` — the "may" is gone, the controller is forced to loot on every attack and block, and the def is `Complete`. It is now recorded in `BASELINE` as merely "the engine chose which card you discard", which is a *weaker* and reassuringly-reviewed-sounding description of a card that is outright live-wrong. That also exposes the gate's structural bound, which nothing in the batch states: every row is keyed on a *decision-bearing DSL variant*, so a choice the DSL never encoded at all is invisible — which is the worst class, not the mildest. Nine further findings follow: one un-run gate-logic probe, a wrong new CR cite, a doc reference to a test that does not exist, an over-claimed denylist-completeness proof, a partial §3.1 reconciliation, and five LOWs.

---

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `decision_gate.rs:346-452` | **`BASELINE` launders a class-D def as class B.** Smuggler's Copter (and Shambling Ghast) are live-wrong vs oracle, not merely un-consulted; plan §5.3 required a seed and none was filed. **Fix:** file the seed, and bound the `BASELINE` doc comment. |
| 2 | **HIGH** | `decision_site_walk.rs:269-487` / audit §8 | **The gate cannot see a decision the DSL never encoded**, which is the most severe class, and criterion 5554's wording ("enumerates every def containing an engine-made choice") is falsified by a def inside the baseline itself. **Fix:** state the bound in three places and seed the missing class. |
| 3 | MEDIUM | `decision_gate.rs:538-571` | **`t4_gate_logic_reddens_…` never runs T4's gate logic.** It re-checks two predicates; the offender loop, the `is_effectively_complete` filter and the row-set-mismatch arm are untested. **Fix:** extract `offenders()` and drive it from the probe. |
| 4 | MEDIUM | `decision_site_walk.rs:351` | **New wrong CR cite: `106.12` for `ChooseColor`.** CR 106.12 is "tap a permanent for mana". **Fix:** `614.12a` (as-enters) / `608.2d` (resolution-time). |
| 5 | MEDIUM | `decision_gate.rs:15-16` | **Module doc cites `t4_failure_message_names_the_bound`, which does not exist** anywhere in the tree — at exactly the R6 spot the plan named as the batch's likeliest harm. **Fix:** write the test or delete the reference. |
| 6 | MEDIUM | `decision_gate.rs:965-1052` | **`prose_field_denylist_covers_every_string_field_in_the_dsl` over-claims** (two files, literal `String` types only; newtype-over-`String` fields uncovered). Audit §8 repeats the over-claim. **Fix:** widen the scan or narrow the name + doc + audit sentence. |
| 7 | MEDIUM | `docs/audits/decision-point-audit.md:134-156, 552-562` | **Criterion 5554's "discrepancies explained" is only partly met.** The two named mechanisms do not explain the largest per-row deltas; two that do are unnamed. **Fix:** per-row delta table in §3.1's note. |
| 8 | MEDIUM | `decision_site_walk.rs:359-368` | **`look_at_top_or_route` over-includes.** Several baselined members (Chaos Warp, Coiling Oracle) have no CR-given choice at all; the row's own `why` asserts otherwise. Inflates the published 97 and OOS-DP10-6's ranking. **Fix:** split the row or mark the count an upper bound. |
| 9 | LOW | `decision_site_walk.rs:311-318` | **`wheel_hand`'s NO-DECISION `why` overreaches.** CR 404.3 gives the owner the graveyard order; the engine picks ascending `ObjectId`. **Fix:** narrow the `why` to "which card" + seed CR 404.3. |
| 10 | LOW | `effect_choose_gate.rs:223-225` | **Stale doc reference to `contains_key`**, deleted by the rewire. **Fix:** retarget to `def_uses`. |
| 11 | LOW | `decision_gate.rs:909-947` | **T12 omits the one row whose predicate spans two enums by design** (`choose_color_or_type`) and does not scan `replacement_effect.rs`. **Fix:** add both. |
| 12 | LOW | audit §8.1 `OOS-DP10-6` | **`put_on_library` (measured 1) omitted** from the successor-queue ranking. **Fix:** add it. |
| 13 | LOW | audit §8.1 / WIP "Dropped" | **The dropped T15 has no owning seed.** OOS-DP10-4 is the `Command` scan, OOS-DP10-7 is the `GameEvent` digest; neither owns the DSL-enum digest. **Fix:** file it. |
| 14 | LOW | `decision_gate.rs:764-793` | **T9 serializes each def once per row** (~40k `CardDefinition` serializations). **Fix:** hoist `to_value` out of the row loop. |

---

### Finding 1 — `BASELINE` launders a class-D def as class B

**Severity**: HIGH
**File**: `crates/engine/tests/core/decision_gate.rs:346-452` (entries `"Smuggler's Copter"` :430, `"Shambling Ghast"` :429)
**Plan clause**: §5.3 — *"If the sweep turns up a def that is **live-wrong** (not merely un-consulted), **file a seed, do not demote**… The distinction that matters: 'the engine chose for you' is class B and is what the baseline records; 'the engine did something the card does not say' is class D and is a seed."*

**Issue.** I spot-checked baselined defs against oracle text via the mtg-rules MCP and found class-D members. No seed was filed for either, and no evidence in the WIP or the audit that the class-B/class-D triage was performed at all on the 97 entries.

1. **Smuggler's Copter** — Oracle: *"Whenever this Vehicle attacks or blocks, **you may** draw a card. If you do, discard a card."* The def (`crates/card-defs/src/defs/smugglers_copter.rs:35-44` and `:54-63`) is:
   ```rust
   effect: Effect::Sequence(vec![
       Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) },
       Effect::DiscardCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) },
   ]),
   ```
   and `completeness: Completeness::Complete` (`:83`). The "may" is not modelled at all — not as `Effect::MayPayThenEffect`, not as anything. The controller is *forced* to loot on every attack and every block. That is a wrong game state, not an un-consulted choice: a player at 0 cards in library dies to it, a player holding a card they must not discard loses it. `BASELINE` records this def as `&["discard_cards"]` — i.e. "the engine picks which card you discard" — which is true but is not the defect.

2. **Shambling Ghast** — Oracle: *"When this creature dies, choose one — • Target creature an opponent controls **gets -1/-1 until end of turn**. • Create a Treasure token."* The def (`shambling_ghast.rs:49-53`) implements mode 1 as `Effect::AddCounter { counter: CounterType::MinusOneMinusOne, count: 1 }` — a **permanent** -1/-1 counter, which persists past end of turn, is proliferate-able, and interacts with `+1/+1` counter annihilation (CR 122.3). Separately its stored `oracle_text` (`:18-19`) says *"When Shambling Ghast **enters**"* while the `TriggerCondition` is `WhenDies`; `oracle_text` is a load-bearing field (`effect_choose_gate.rs::printed_tap_mana_colors` parses it, the viewer displays it). `BASELINE` records this def as `&["modal_trigger"]`.

**Why it matters.** The frozen baseline is published as the reviewed, acknowledged set: T4's own message calls a `BASELINE` entry *"a reviewed acknowledgement that this card ships with the engine choosing for the player until the owning PB lands."* For these two the sentence is false in a way that hides a worse defect behind a milder label — and the audit's §8 row and OOS-DP10-6 then carry the 97 forward as the successor queue's input. This is the same shape as PB-DP8's meta-lesson (iii): *a gate cited as covering something is a claim like any other.*

**Fix.**
1. File **OOS-DP10-8** in audit §8.1: *"PB-DP10's `BASELINE` was populated mechanically; the class-B/class-D triage plan §5.3 required was not performed. Two class-D members found on spot-check: Smuggler's Copter (oracle 'you **may** draw a card' modelled as an unconditional `Sequence(DrawCards, DiscardCards)`; the controller is forced to loot) and Shambling Ghast (oracle 'gets -1/-1 until end of turn' modelled as a permanent `MinusOneMinusOne` counter; stored `oracle_text` also says 'enters' where the trigger is `WhenDies`). Both are `Complete`. The remaining 95 entries have not been triaged."* Classify `correctness, uncounted`.
2. Add to `BASELINE`'s doc comment (`decision_gate.rs:339-345`): *"An entry asserts exactly one thing — that this def hits these `AutoChosen` rows. It asserts **nothing** about whether the def is otherwise oracle-correct; the entries were not triaged against oracle text (OOS-DP10-8)."*
3. Amend T4's failure-message sentence "a reviewed acknowledgement" to "a recorded acknowledgement" so the message does not overstate.

---

### Finding 2 — the gate is blind to a decision the DSL never encoded, and criterion 5554's wording does not say so

**Severity**: HIGH
**File**: `crates/engine/tests/core/decision_site_walk.rs:269-487` (`ROWS`); `docs/audits/decision-point-audit.md` §8 PB-DP10 row; ESM criterion 5554
**CR Rule**: 118.12 — *"'[A player] may [do something]. If [that player] [does…]' … The action is a cost, paid when the spell or ability resolves."*

**Issue.** Every one of the 22 rows is a predicate over a **DSL variant name**. That means the gate can only see a decision that someone *encoded as a decision-bearing variant*. A card whose choice was dropped at authoring time — never expressed as `MayPayThenEffect`, `Choose`, `optional`, a mode, or anything else — produces a serialized tree with no trace of the choice, hits zero rows, and passes T4, T6 and T7 forever.

This is not hypothetical: it is exactly Smuggler's Copter (Finding 1). Its `may` clause is a CR 118.12 optional cost. Had it been authored as `MayPayThenEffect`, it would have hit the `may_pay_then_effect` row and been recorded. Because it was authored as a bare `Sequence`, it is invisible — and the *only* reason it appears in `BASELINE` at all is the incidental `DiscardCards` in its second element.

The batch's own framing makes the opposite impression. ESM criterion 5554 says *"a machine gate **enumerates every def containing an engine-made choice**"*; the audit §8 row says the gate *"fails on a `Complete` def hitting an `AutoChosen` row"* and, correctly, that it does not close DP-INV — but nowhere does either say that the population it scans is *encoded* decision sites, not decision sites. That distinction inverts the severity ordering: the defs the gate misses are worse than the ones it catches, because a recorded auto-choice is at least a legal outcome, while a dropped "may" is not.

**Fix.**
1. `decision_gate.rs` module doc, immediately after the existing "cannot stop the growth; makes it recorded" paragraph, add: *"And it can only see a decision the DSL **encoded**. Every row is a predicate over a variant name, so a card whose choice was dropped at authoring time — a 'you may' written as a bare `Sequence`, a 'choose one' written as a single effect — hits zero rows and passes. That class is strictly worse than the class this file records, and this file does not detect it (OOS-DP10-9)."*
2. Same sentence, condensed, into the audit §8 PB-DP10 row and into §3.1's superseded note.
3. File **OOS-DP10-9**: *"Un-encoded decisions are invisible to PB-DP10's gate. A 'you may X. If you do, Y' authored as an unconditional `Effect::Sequence` (Smuggler's Copter) leaves no variant to key on. Detecting this class needs an oracle-text-vs-DSL cross-check ('may'/'choose'/'up to' in `oracle_text` with no decision-bearing variant in the effect tree), which is a different instrument from a variant walk."* Note this is a *feasible* test-only gate and a strong successor candidate: `oracle_text` is already on the def and already parsed by `effect_choose_gate.rs`.
4. When the ESM criterion 5554 is attested, attest against the corrected wording, not the literal "every def containing an engine-made choice".

---

### Finding 3 — T4's non-vacuity probe does not exercise T4

**Severity**: MEDIUM (fix-phase HIGH per `memory/conventions.md` → "Test-validity MEDIUMs are fix-phase HIGHs")
**File**: `crates/engine/tests/core/decision_gate.rs:538-571`

**Issue.** `t4_gate_logic_reddens_on_a_new_unbaselined_auto_chosen_complete_def` builds a synthetic `Complete` def carrying `Effect::Proliferate` and then asserts only:

```rust
let hits = auto_chosen_row_hits(&fake);
assert!(!hits.is_empty() && !baseline.contains_key(fake.name.as_str()), …);
```

That is a re-check of two things T1 and T5 already cover. It never runs T4's offender loop (`decision_gate.rs:476-514`), which is where the actual gate logic lives: the `is_effectively_complete` filter, the `hits.is_empty()` short-circuit, the `None =>` offender branch, and — the arm with no coverage anywhere — `Some(recorded) if recorded != &hits`, the superset/subset mismatch detection that is half of the ratchet's design rationale (plan §1.3). `fake.completeness = Complete` is assigned at `:559` and never read by the probe.

The test's name promises the gate reddens. Nothing in the file demonstrates that. This is precisely the failure mode the suite has been burned by (PB-DP8 closing-review lesson (ii): *"a test that constructs a hazardous state and does not assert against it is worse than no test, because it reads as coverage"*).

Mitigating: a *systemic* blinding of `auto_chosen_row_hits` would be caught by T6's exact `union == 97` and by T10/T11's floors. The uncovered surface is the per-def bookkeeping, not the walk.

**Fix.** Extract the loop:
```rust
fn offenders(defs: &[CardDefinition], baseline: &HashMap<&str, BTreeSet<&str>>) -> Vec<String>
```
Have T4 call `offenders(&all_cards(), &baseline_map())`. Have the probe call it on a three-element synthetic corpus and assert exactly three outcomes: (a) an unbaselined `Complete` `Proliferate` def **is** an offender; (b) a def present in a synthetic baseline with a *smaller* row set **is** an offender and the message says "subset"; (c) a **non-`Complete`** def carrying the same `Proliferate` is **not** an offender.

---

### Finding 4 — a new wrong CR cite ships in the row table

**Severity**: MEDIUM
**File**: `crates/engine/tests/core/decision_site_walk.rs:351`
**CR Rule**: 106.12 — *"To 'tap [a permanent] for mana' is to activate a mana ability of that permanent that includes the {T} symbol in its activation cost."*

**Issue.** The `choose_color_or_type` row reads:
```rust
cr: "106.12 (ChooseColor) / n/a (ChooseCreatureType)",
```
CR 106.12 has nothing to do with choosing a colour; it is the tap-for-mana definition, and it is cited correctly for that purpose 20 lines away in `effect_choose_gate.rs:508`. The plan's §3 row 8 supplied **no** CR at all, so this was invented at implementation time. `"n/a"` for `ChooseCreatureType` is wrong for the same reason.

The governing rules, verified: **CR 614.12a** — *"If a replacement effect that modifies how a permanent enters the battlefield requires a choice, that choice is made before the permanent enters the battlefield"* — for the `ReplacementModification::ChooseColor`/`ChooseCreatureType` path (Caged Sun, Morophon, Urza's Incubator, Vanquisher's Banner, Patchwork Banner, Etchings of the Chosen, Obelisk of Urd); and **CR 608.2d** for the resolution-time `Effect::ChooseCreatureType` path (Kindred Dominance, Crippling Fear, Pact of the Serpent).

This matters more than a comment typo because `Row::cr` is *printed by T9* on every `cargo test` and *interpolated into T4's failure message* (`decision_gate.rs:496`), so it propagates into whatever a future author reads when the gate reddens. The batch's own contribution to §3.1 was fixing two wrong CR cites; shipping a third undercuts that.

**Fix.** `cr: "614.12a (as-enters, ReplacementModification) / 608.2d (resolution-time Effect)"`.

---

### Finding 5 — the module doc cites a test that does not exist

**Severity**: MEDIUM
**File**: `crates/engine/tests/core/decision_gate.rs:15-16`

**Issue.**
```
//! growth; it makes it recorded** (see `T4`'s failure message, `t4_failure_message_names_
//! the_bound`, and the audit's own PB-DP10 row -- do not read this file as a closure of
//! DP-INV).
```
`rg t4_failure_message_names_the_bound` over the whole worktree returns **nothing**. The file's 17 tests are T1, T2, T3, T4 + its probe, T5, T6, T7, T8, T9, T10, T11, T12 + its probe, T13, T14, T16 — none of them checks the failure message.

Plan §12 R6 names "the gate reading as a closure of DP-INV" as *"the most likely way PB-DP10 does harm"*. The one place the file asserts a machine check against that risk, the check does not exist. Per `memory/conventions.md` ("Aspirationally-wrong code comments are correctness hazards"), the aspirational version must not be left standing.

**Fix.** Preferred: write the test — extract T4's message into `fn t4_message(offenders: &[String]) -> String`, and assert it contains `"CANNOT STOP THE GROWTH"`, `"Mark the def non-Complete"`, `"Add a BASELINE entry"`, and `"is NOT an exit for this batch"`. That makes R6 machine-checked for ~10 lines. Otherwise delete the citation.

---

### Finding 6 — `prose_field_denylist_covers_every_string_field_in_the_dsl` does not cover every string field in the DSL

**Severity**: MEDIUM
**File**: `crates/engine/tests/core/decision_gate.rs:965-1052`; over-claim repeated at `docs/audits/decision-point-audit.md:692` ("a completeness proof that `PROSE_FIELDS` covers every `String`-typed field reachable from the DSL (`T13`)")

**Issue.** Two narrowings, neither reflected in the test name, the doc comment, or the audit sentence:

1. **Type recognition.** `string_field_name` (`:965-980`) accepts only the literal types `String`, `Option<String>`, `Vec<String>`. A serde **newtype struct over `String` serializes to a bare JSON string too**, so a field of such a type is exactly as much a false-positive channel as a `String` field. Reachable-from-`CardDefinition` examples the scan silently ignores, none of which is in `PROSE_FIELDS`:
   - `pair_card_id: CardId` (`card_definition.rs:282`), `melded_card_id: CardId` (`:286`)
   - `onto_subtype: SubType` (`:684`)
   - `has_subtype: Option<SubType>` (`:3064`), `has_subtypes: Vec<SubType>` (`:3068`), `exclude_subtypes: Vec<SubType>` (`:3134`), `spell_subtype_filter: Option<Vec<SubType>>` (`:3383`)
   - `subtype: SubType` in `state/replacement_effect.rs:284`
2. **File scope.** It scans `card_definition.rs` plus the `TriggeredAbilityDef` struct body in `game_object.rs`. `state/types.rs`, `state/replacement_effect.rs` and `state/targeting.rs` all contribute types to the `CardDefinition` tree and are not scanned. (`state/types.rs:342/346` declare a further `has_subtype`/`has_subtypes` pair.)

No live false positive exists today — I checked that no MTG subtype and no `cid(..)` slug in the corpus equals `"Proliferate"`, `"TheRingTemptsYou"` or `"Discover"` — so this is a claim defect, not a measurement defect. But the whole point of T13 was to close the channel *before* it opens, and OOS-DP7-11's lesson is that an unearned coverage claim is the failure mode.

**Fix.** Either (a) widen `string_field_name` to accept `SubType`, `Option<SubType>`, `Vec<SubType>`, `Option<Vec<SubType>>`, `CardId`, `Option<CardId>`, add those field names to `PROSE_FIELDS`, and extend the scanned set to `state/types.rs` + `state/replacement_effect.rs` + `state/targeting.rs`; or (b) rename to `prose_field_denylist_covers_every_literal_string_field_in_card_definition_rs`, say in the doc comment that newtype-over-`String` fields are **not** covered, file the gap as a seed, and correct the audit §8 sentence to match.

---

### Finding 7 — the §3.1 reconciliation explains a minority of the drift (criterion 5554)

**Severity**: MEDIUM
**File**: `docs/audits/decision-point-audit.md:134-156` (superseded note) and `:552-562` (§6 bullet)

**Issue.** Criterion 5554 requires the measured count to *"reconcile against §3.1's magnitude **with discrepancies explained**"*. §6 names exactly two mechanisms — per-node qualification of the compound rows, and unit-variant string matching. Neither explains the largest per-row deltas, and the unit-variant mechanism explains **no** delta at all (`proliferate` measures 25 both ways; the string arm is what *preserves* the 25, not what changes it).

Per-row, audit → measured:

| row | audit `Complete` | measured | mechanism stated? |
|---|---:|---:|---|
| `triggered_targets` | 84 | 77 | yes (compound row, per-node) |
| `search_library` | 74 | 73 | **no** |
| `proliferate` | 25 | 25 | n/a |
| `discard_cards` (+`wheel_hand`) | 23 | 13 (+10) | **no** — but the arithmetic is exact |
| `scry` / `surveil` | 16 / 8 | 16 / 8 | n/a |
| `sacrifice_permanents` | 11 | 11 | n/a |
| `may_pay_then_effect` | 11 | 10 | **no** |
| `choose_color_or_type` | 10 | 10 | n/a |
| `look_at_top_or_route` | 10 | 10 | n/a |
| `counter_unless_pays` | 7 | 7 | n/a |
| `modal_trigger` | 5 | 4 | yes (compound row) |
| `change_targets` / `bolster_amass` | 3 / 3 | 3 / 3 | n/a |
| `put_on_library` | 3 | 1 | **no** |
| `connive` | 2 | 1 | **no** |
| `discover` | 1 | 1 | n/a |

I checked two of the unexplained deltas by finding the actual defs, as the brief asked:

- **`connive` 2 → 1.** Only `raffines_informant.rs:25` carries a real `Effect::Connive`. `spymasters_vault.rs:45` mentions the string **only inside a `Completeness::partial(...)` note** (*"Effect::Connive itself exists (card_definition.rs:1633)"*), and the def is `partial`, not `Complete`. The audit's source regex counted the note.
- **`put_on_library` 3 → 1.** Only `brainstorm.rs:23` carries a real `Effect::PutOnLibrary`. `witchs_cottage.rs:47` and `gravepurge.rs:32` mention it only in `Completeness` note strings.

So the driving mechanism for those rows is the plan's §4 note 2 ("regex hits in comments") extended to `Completeness` note strings — a mechanism the plan enumerated and the audit update dropped. And the single largest delta, `discard_cards` 23 → 13, is fully explained by the plan's own row-4 split: **13 + `wheel_hand` 10 = 23, exactly.** That arithmetic is compelling and nowhere written down; a reader sees "23 became 13" and cannot tell whether ten defs went missing.

**Fix.** Put the table above (or its equivalent) into §3.1's superseded note, with a mechanism per non-trivial row, and add to §6's bullet the two missing mechanisms: *"(iii) the audit's regex counted variant names appearing inside comments and `Completeness` note strings, which a serde walk cannot see — this is the whole of the `connive` 2→1 and `put_on_library` 3→1 deltas; (iv) the audit's row 4 bundled three needles, and the split is exact: `discard_cards` 13 + `wheel_hand` 10 = 23."* The `search_library` 74→73, `may_pay_then_effect` 11→10 and `modal_trigger` 5→4 deltas each need a stated mechanism or an explicit "unexplained, ±1, within regex noise".

---

### Finding 8 — `look_at_top_or_route` over-includes, and the row says the opposite

**Severity**: MEDIUM
**File**: `crates/engine/tests/core/decision_site_walk.rs:359-368`
**CR Rule**: 608.2d — the row claims all its members are 608.2d choices.

**Issue.** The row's justification reads:
> *"LookAtTopThenPlace's `optional` field is inert by construction (OOS-DP10-5) and RevealAndRoute routes deterministically; both are CR 608.2d choices the player should make"*

For `LookAtTopThenPlace` that is right — `optional` is destructured away, so a printed "you may" is dropped. For `RevealAndRoute` it is right for some members and wrong for others, because the *card* determines the routing:

- **Chaos Warp** (`chaos_warp.rs:45-67`, count 1) — Oracle: *"…reveals the top card of their library. If it's a permanent card, they put it onto the battlefield."* Reveal one card, deterministic destination on both branches. **No CR 608.2d choice exists.**
- **Coiling Oracle** (count 1) — Oracle: *"…reveal the top card of your library. If it's a land card, put it onto the battlefield. Otherwise, put that card into your hand."* Same. **No choice.**
- **Goblin Ringleader** (`goblin_ringleader.rs:30-40`, count 4) — Oracle: *"…put all Goblin cards revealed this way into your hand and the rest on the bottom of your library **in any order**."* Here there **is** a CR 608.2d/401.4 order choice, correctly flagged.
- **Risen Reef / Satyr Wayfinder / Grisly Salvage / Birthing Ritual / Growing Rites / Thrasios** — genuine "you may" / "a card from among them" choices, correctly flagged.

So roughly 2 of 10 members carry no player decision at all. Because OOS-DP10-6 ranks the successor queue **by measured def count**, and because 97 is now the published still-auto figure, the over-inclusion propagates.

**Fix.** Minimum: change the row's `why_not_flagged_is_wrong` to *"`LookAtTopThenPlace`'s `optional` is inert by construction (OOS-DP10-5); `RevealAndRoute` covers both real CR 608.2d/401.4 order choices (Goblin Ringleader's 'in any order') **and** defs whose routing the card itself determines (Chaos Warp, Coiling Oracle), so this row's count is an **upper bound** on decisions"*, and add the same caveat to OOS-DP10-6. Better: split into two rows keyed on `LookAtTopThenPlace` and `RevealAndRoute` and qualify the latter on `count > 1` or on distinct `matched_dest`/`unmatched_dest` — the same per-node qualification technique the two compound rows already use.

---

### Finding 9 — `wheel_hand`'s NO-DECISION reason overreaches

**Severity**: LOW
**File**: `crates/engine/tests/core/decision_site_walk.rs:311-318`
**CR Rule**: **404.3** — *"If an effect or rule puts two or more cards into the same graveyard at the same time, the owner of those cards may arrange them in any order."*

**Issue.** The NO-DECISION classification is **correct on the question the audit's row asks** — I read `Effect::WheelHand` (`effects/mod.rs:1214-1272`) and `WheelDisposal` (`card_definition.rs:2525-2535`): all three disposals take the *whole* hand, so there is no "which card" pick and the row is not misclassified in the direction that hides work. The plan's reading is verified.

But the stated reason is wider than the truth:
```rust
why: "the whole hand is discarded, so there is no pick order to observe or hook",
```
`WheelDisposal::Discard` delegates to `discard_cards(state, p, hand_size, events)`, which moves cards one at a time in ascending `ObjectId` (`effects/mod.rs:9321-9327`). Graveyard order **is** observable (top-of-graveyard effects, Volrath's Shapeshifter, Delve/Escape ordering displays) and CR 404.3 gives it to the owner. So the engine does make one CR-given choice here; it is just not the one the row counts. The same is true of the `discard_cards` row for any count ≥ 2, and of every mill effect.

**Fix.** Narrow the `why` to *"the whole hand is discarded, so there is no 'which card' pick (CR 701.9b) to hook. The CR **404.3** graveyard-order choice does still exist and the engine takes it by ascending `ObjectId` — that is a separate, uncounted class (see OOS-DP10-10), not this row."* File **OOS-DP10-10**: *"CR 404.3 simultaneous-graveyard order is engine-chosen everywhere (discard, wheel, mill, sacrifice batches) and is not one of §3.1's 22 rows — an uncounted class-B site with a wide roster."*

---

### Finding 10 — stale doc reference left by the rewire

**Severity**: LOW
**File**: `crates/engine/tests/core/effect_choose_gate.rs:223-225`

**Issue.** `count_key_occurrences`'s doc says *"(unlike [`contains_key`], which only asks whether it appears at all)"*. The §2.3 rewire replaced `contains_key` with `def_uses` delegating to the canonical walk; `contains_key` no longer exists in this file. A dangling intra-doc reference in a file whose whole purpose is documenting what the gates do and do not cover.

**Fix.** `(unlike [`def_uses`], which only asks whether it appears at all)`.

**Rewire verification (positive).** I read the rewire in both files against plan §2.3's abort condition and found it behaviour-neutral by construction, not merely by assertion: `def_uses` (`effect_choose_gate.rs:73-75`) and `contains_key` (`pb_rs1_roster_sweep.rs:26-28`) now call `def_contains_variant`/`json_contains_variant`, whose only semantic addition over the deleted copies is the `Value::String` arm. Every key those two files name — `Choose`, `MayPayOrElse`, `AddManaChoice`, `AddManaAnyColor`, `AddManaAnyColorRestricted`, `AddManaOfAnyColorAmount`, `Scry`, `Surveil`, `RevealAndRoute`, `LookAtTopThenPlace` — is a struct variant (declarations read), and I confirmed by grep that no enum in `crates/card-types/src` declares a **unit** variant of any of those names, so the string arm cannot fire. `PROSE_FIELDS` suppression applies only to `Value::String` and therefore cannot suppress an object-key hit, so no SR-33 offender set can shrink. **No SR-33 assertion or message text was altered** — all fourteen tests, their messages and their `Completeness` pins are intact.

---

### Finding 11 — T12's collision inventory omits the one multi-enum row

**Severity**: LOW
**File**: `crates/engine/tests/core/decision_gate.rs:909-947`

**Issue.** `pinned_collision_counts()` pins four keys (`Discover` 2, `SearchLibrary` 3, `Scry` 3, `Surveil` 3) across three files (`card_definition.rs`, `state/types.rs`, `state/stubs.rs`). I verified all four counts by grep and they are correct today. But the row whose predicate matches **two different enums by design** — `choose_color_or_type`, `json_contains_variant(v, "ChooseColor") || json_contains_variant(v, "ChooseCreatureType")`, hitting `ReplacementModification::ChooseColor` (`state/replacement_effect.rs:219`), `ReplacementModification::ChooseCreatureType` (`:170`) and `Effect::ChooseCreatureType` (`card_definition.rs:1859`) — is **not** pinned, and `replacement_effect.rs` is not one of the scanned files. That is the row most exposed to a new declaration silently changing what the gate counts, and it is the only one left unguarded.

The doc comment is honest about the narrowing ("Scoped inventory (not a generic scan of all 22 row keys)"), so this is a coverage gap, not a false claim.

**Fix.** Add `crates/card-types/src/state/replacement_effect.rs` to `combined`, and pin `("ChooseColor", 1)` and `("ChooseCreatureType", 2)` (verify the counts before pinning).

---

### Finding 12 — `OOS-DP10-6` omits a row from the successor queue's input

**Severity**: LOW
**File**: `docs/audits/decision-point-audit.md:840`

**Issue.** OOS-DP10-6 is explicitly "the successor queue's input, ranked by measured `Complete` count" and lists twelve rows: proliferate 25, discard_cards 13, sacrifice_permanents 11, may_pay_then_effect 10, choose_color_or_type 10, look_at_top_or_route 10, counter_unless_pays 7, modal_trigger 4, change_targets 3, bolster_amass 3, connive 1, discover 1. **`put_on_library` (measured 1, Brainstorm) is missing** — that is 13 non-zero `AutoChosen` rows, not 12. Cross-checked against the WIP's own measured list and against `BASELINE` (Brainstorm, `:355`).

**Fix.** Add `put_on_library 1` to the ranked list.

---

### Finding 13 — the dropped T15 has no owning seed

**Severity**: LOW
**File**: `memory/primitive-wip.md` "Dropped from the plan's test list"; `docs/audits/decision-point-audit.md:838, 1061-1067`

**Judgement on the drop itself: correct.** T15 was a roster digest over the `Effect` / `AbilityDefinition` / `ReplacementModification` declarations, and the plan's own §9 analysis established that a new variant in any of those *already* forces a PROTOCOL and a HASH bump (both are inside the SR-8/SR-17 closures), so the **notice** was never missing — only the **obligation message** ("now classify it in `ROWS`"). Spending the remaining budget on T12/T13/T14/T16, each of which defends a mechanism this batch actually introduced, was the right trade, and the audit §10 ledger records the reasoning honestly (3 of 8 mechanized, with the caveat that the framing of the trigger was wrong).

**The record, however, is inaccurate.** The WIP says T15/T15b were *"filed as … seeds OOS-DP10-4 (Command accepted-and-discarded scan) and OOS-DP10-7 (`GameEvent` sibling-answer roster digest)"*. Neither seed owns T15's actual subject: OOS-DP10-4 is a scan for `_`-bound `Command` fields (a different instrument, a different enum), and OOS-DP10-7 is the `GameEvent` digest (T15**b**). The `Effect`/`AbilityDefinition`/`ReplacementModification` roster digest exists only as prose inside §10's ledger ("recommendation for whoever builds it") with no seed id, so nothing tracks it and it will not surface in a re-triage.

**Fix.** File **OOS-DP10-11**: *"The DSL-enum roster digest (PB-DP10's dropped T15): a count + blake3 of the sorted variant-name lists of `Effect`, `AbilityDefinition` and `ReplacementModification`, failing with 'a new variant landed; classify it in `decision_site_walk.rs::ROWS` and update audit §3.1/§10'. The notice is already free (SR-8/SR-17 force both bumps); the digest supplies the obligation. ~20 lines, test-only. Sibling of OOS-DP10-7, which is the same instrument pointed at `GameEvent`."* Correct the WIP's "Dropped" paragraph to point at it.

---

### Finding 14 — T9 re-serializes the corpus once per row

**Severity**: LOW
**File**: `crates/engine/tests/core/decision_gate.rs:769-778` (also `:683`, `:746`, `:838`, `:857`)

**Issue.** `for row in ROWS { for def in &defs { (row.predicate)(&serde_json::to_value(def).unwrap()) } }` performs 22 × ~1,804 ≈ **40,000** full `CardDefinition` serializations, where 1,804 would do. T7, T8, T10 and T11 repeat the pattern at smaller multiples. `CardDefinition` is a deep tree; this is measurable wall-clock in a suite that runs on every commit and in CI.

**Fix.** Hoist: build `let jsons: Vec<Value> = defs.iter().map(|d| to_value(d).unwrap()).collect();` once at the top of each test and index it in the row loop.

---

## CR Coverage Check

| CR Rule | Cited correctly? | Verified how | Notes |
|---|---|---|---|
| 608.2d | Yes | MCP | Correct unifying rule for `put_on_library`, `look_at_top_or_route` |
| 701.9 / 701.9b | Yes | MCP | Audit's old `701.8` correction is right (701.8 is Destroy) |
| 701.9 (`wheel_hand`) | Partly | MCP + engine read | NO-DECISION verdict correct; the `why` overreaches — Finding 9 |
| 404.3 | **Not cited** | MCP | An uncounted class the batch touched but did not name — Finding 9 |
| 701.21a / 701.22a / 701.23a / 701.25a / 701.34a / 701.47a / 701.50a / 701.54a | Yes | MCP + engine read | |
| 701.39a | Yes | MCP | Audit's `701.29a` correction is right; "among ties the controller chooses" is verbatim CR |
| 701.57a | Yes | MCP | "You may cast that card" — engine always casts, AutoChosen correct |
| 115.7d | Yes | MCP | "may leave any number of the targets unchanged" — engine always declines, AutoChosen correct |
| 118.12 / 118.12a | Yes | MCP + `try_pay_optional_cost` read | Pays iff affordable — AutoChosen correct; **no contradiction** with SR-33's non-gating note (see below) |
| 603.3c / 603.3d | Yes | engine read | |
| 605.1a / 700.2 | Yes | — | Gated rows |
| **106.12** | **NO** | MCP | Wrong rule for "choose a color" — Finding 4 |
| 614.12a | Missing | MCP | The rule that should be there — Finding 4 |

**On the `may_pay_then_effect` framing question the brief raised: the two documents do not contradict each other, and PB-DP10's framing is the correct one.** `effect_choose_gate.rs:103-105` says `MayPayThenEffect` is *"deliberately **not** gated here — it honours its `payer` and pays when able, which is a documented deterministic-but-legal game choice (CR 118.12). It is a weaker claim than these two stubs, not the same defect."* That is a statement about **SR-33's** scope (class C: an effect that does one fixed thing *regardless of what the card prints*), and it remains true — `try_pay_optional_cost` (`effects/mod.rs:9257-9270`) really does honour `payer` and really does execute the card's printed `then`. PB-DP10 classifies it `AutoChosen` (class B: a *legal* outcome that no player chose), which is a different and compatible claim, and plan §5.5 argues the distinction explicitly. The two files are consistent. The only gap is presentational: plan §5.5 recommended *"a 3-line pointer comment in each file's module doc"* so a reader landing on `effect_choose_gate.rs` learns that a second gate now records what this one declines to bar, and that pointer was not added. Worth adding, but not a finding — the substance is right and PB-DP10's `why_not_flagged_is_wrong` string already states the class-B claim precisely.

---

## Verification of the claims the brief flagged as load-bearing

| Claim | Verdict | Evidence |
|---|---|---|
| Walk is blind to unit variants (the headline) | **True, and fixed** | `Effect::Proliferate` (`card_definition.rs:1933`) and `TheRingTemptsYou` (`:2122`) are unit variants; `walk_contains`'s `Value::String` arm handles them; T2 pins both directions |
| Any `#[serde(rename/untagged/flatten/skip/tag)]` reachable from `CardDefinition`? | **No — none** | `rg 'serde\((rename\|untagged\|flatten\|skip\|tag\|content\|with\|other)' crates/card-types/src` → 4 hits, **all prose inside `//` comments in `state/stubs.rs`**. All 310 real `serde` attributes in the crate are `#[serde(default)]`, which does not affect serialization shape. P7 holds |
| Variant name reachable as a **map key**? | **No** | Zero `HashMap`/`BTreeMap`/`IndexMap` in `crates/card-types/src/cards/card_definition.rs` and zero in `crates/card-types/src/state/`. No map-key false-positive channel exists |
| `PROSE_FIELDS` could over-suppress a real `Effect::Proliferate` sitting as a denylisted field's direct value? | **No** | Grepped every field declaration named `name`/`oracle_text`/`subtype`/`prompt`/`first_name`/`second_name`/`has_name`/`card_id`/`description` across `crates/card-types/src`: **all** are `String`, `Option<String>`, `CardId` or `SubType`. None holds an `Effect` or `Vec<Effect>`. The array-inherits-parent-key path (`walk_contains:77`) is therefore also safe |
| T13's denylist-completeness claim | **Over-claimed** | Finding 6 |
| Gate fails closed by construction | **Mostly** | T4's loop is correct as written (`continue` guards are right, `offenders.is_empty()` is the assertion, the mismatch arm exists); T6's exact `== 97` plus `MIN_ROWS`/`MIN_BASELINE`/`MIN_CORPUS` catch systemic blinding; T1's `positive_value_for_row` **panics** on an unregistered row id, so a new `ROWS` entry cannot ship without a probe. **But** the per-def bookkeeping is never executed by a probe — Finding 3 |
| `wheel_hand` NO-DECISION | **Correct** on the question the row asks | All three `WheelDisposal` arms take the whole hand (`effects/mod.rs:1236-1241`, `:1266-1271`) — not misclassified in the work-hiding direction. Reason string overreaches — Finding 9 |
| `search`/`scry`/`surveil` genuinely `Served` | **Yes** | `Effect::SearchLibrary` at `effects/mod.rs:3476` routes through PB-DP9's CR 608.2d channel with residuals correctly named (`OOS-DP9-9` `reveal` inert at `:3479`; `OOS-DP9-3` finds-one). `scry`/`surveil` carry `residual: &[]`, consistent with PB-DP9's close |
| `BASELINE` internally consistent | **Yes** | Counted all 97 entries; per-row tallies from `BASELINE` reproduce the WIP's measured numbers exactly (sacrifice 11, proliferate 25, look_at_top 10, change_targets 3, put_on_library 1, discard 13, bolster 3, choose_color 10, may_pay 10, modal 4, counter_unless 7, discover 1, connive 1 = 99 hits over 97 distinct defs, the two doubles being Izzet Charm and Tainted Observer) |
| `discard_cards` 13 vs audit 23 | **Explained by the row-4 split, exactly** | 13 + `wheel_hand` 10 = 23. Not written down — Finding 7 |
| `connive` 1 vs audit 2, `put_on_library` 1 vs audit 3 | **Explained: regex hit `Completeness` note strings** | Verified by finding the defs (`spymasters_vault.rs:45`, `witchs_cottage.rs:47`, `gravepurge.rs:32`). Mechanism not stated — Finding 7 |
| R6 bound ("recorded, not impossible") | **Present in T4's message and the audit §8 row** | `decision_gate.rs:523` `"THIS GATE CANNOT STOP THE GROWTH; IT MAKES IT RECORDED"`; audit `:692` `"This does not close DP-INV"`. Neither reads as a closure. **But** the file cites a nonexistent test as the machine check (Finding 5) and neither states the *encoding* bound (Finding 2) |
| Hard constraint: no engine/card-types/card-defs edit | **Corroborated, not proven** | `rg 'PB-DP10\|decision_gate\|decision_site_walk' crates/` → 6 files, all under `crates/engine/tests/`. No `git diff` available (see limitations) |
| PROTOCOL 31 / HASH 68 unmoved | **Verified by reading** | `rules/protocol.rs:299` `PROTOCOL_VERSION: u32 = 31`; `state/hash.rs:660` `HASH_SCHEMA_VERSION: u8 = 68` |
| SR-9a satisfied | **Yes** | `core/main.rs:19-20` carries `mod decision_gate;` and `mod decision_site_walk;`, alphabetically correct (`deci` < `deck`); `crates/engine/tests/*.rs` contains only `no_stray_test_binaries.rs`; `decision_site_walk.rs` has no `#[test]`, which SR-9a permits |
| T12's four pinned counts | **Correct today** | Grepped: `Discover` = `card_definition.rs:1950` + `types.rs:1476` = 2; `SearchLibrary` = `:1701` + `stubs.rs:910` + `:931` = 3; `Scry` = `:1664` + `:919` + `:938` = 3; `Surveil` = `:1675` + `:921` + `:944` = 3 |

---

## What I verified by execution vs by reading

**By execution: nothing.** This review environment provides no Bash tool, so `cargo test`, `cargo clippy`, `git show --stat 76b4f1cd`, `git diff 0991999c..HEAD` and the scratch red/green probe the brief asked for were all unavailable. The 3,927-passing figure, the clippy/fmt cleanliness, the rewire's "identical printed counts", and the emptiness of `git diff --name-only main -- crates/engine/src crates/card-types/src crates/card-defs/src` are **unverified by me** and are taken from the WIP's own record.

**By reading, with independent corroboration:**
- Both new files line by line, plus both rewired files and the `pb_dp9_effect_choice.rs` doc note.
- Every engine site named in `ROWS` for the `AutoChosen` and `Served` classes, read in `crates/engine/src/effects/mod.rs` (`Proliferate` 4460, `discard_cards` 9319, `try_pay_optional_cost` 9257, `MayPayThenEffect` 4133, `CounterUnlessPays` 4187, `SacrificePermanents` 4210, `PutOnLibrary` 3444, `SearchLibrary` 3476, `WheelHand` 1214) and `crates/card-types/src/cards/card_definition.rs` (`WheelDisposal`/`WheelDraw`). Every class is right today except the two reason-string overreaches (Findings 8, 9).
- Every CR cite in `ROWS`, against the mtg-rules MCP. One is wrong (Finding 4); the two the batch *corrected* are correctly corrected.
- The serde-shape premises the whole design rests on, by grepping the declarations rather than trusting the plan: no `rename`/`untagged`/`flatten`/`skip`/`tag`, no maps, no `Effect`-typed field under a `PROSE_FIELDS` name, no unit variant colliding with an SR-33 key.
- `BASELINE`'s 97 entries counted by hand and cross-tallied per row against the WIP's measured numbers — internally consistent.
- Five card defs read against oracle text via the mtg-rules MCP (Smuggler's Copter, Shambling Ghast, Coiling Oracle, Chaos Warp, Crossway Troublemakers) plus corpus greps for `Effect::Connive` and `Effect::PutOnLibrary`. This is what produced Findings 1, 7 and 8, and it is the leg I would most want re-run wider: I checked five of ninety-seven and found two class-D defs.

**Recommended before close**: run Finding 3's rewritten probe and Finding 5's message test, and re-run the Finding 1 oracle spot-check across all 97 `BASELINE` entries — at the hit rate I observed, that sweep is the highest-value remaining work in this batch.
