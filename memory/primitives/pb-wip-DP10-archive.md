# Primitive WIP — PB-DP10 (decision-gate widening: stop the 277-def figure growing silently)

<!-- last_updated: 2026-07-27 -->

> Previous occupant: **PB-DP9 (DP-7 / DP-8 / DP-9: search, scry, surveil player choice) — SHIPPED**
> `scutemob-157`, merge `d65e7f1e`, PROTOCOL 30 → **31**, HASH 67 → **68**, tests **3,910** on main.
> Its WIP file is preserved verbatim at `memory/primitives/pb-wip-DP9-archive.md` (this file is
> rewritten wholesale by each `/implement-primitive` run); its plan/review are
> `memory/primitives/pb-plan-DP9.md` / `pb-review-DP9.md`, and its seeds are in
> `docs/audits/decision-point-audit.md` §8.1.

- **PB**: PB-DP10 — the **invariant-level** fix for the whole PB-DP suite. Audit §8's last row.
- **Task**: `scutemob-158`
- **Branch**: `feat/pb-dp10-widen-the-decision-gate-stop-the-277-def-engine-gues`
- **Class**: GATE / INVARIANT (test-only). Rank 10 of the PB-DP suite; **closes it**.
- **Phase**: **closed** — plan → implement → review → fix → close all complete 2026-07-27.
  Review findings applied in full (all 14: 2 HIGH, 6 MEDIUM, 6 LOW); see the
  "Review + fix cycle" section below. **This closes the PB-DP suite (DP1..DP10).**
- **Plan**: `memory/primitives/pb-plan-DP10.md`
- **Review file**: `memory/primitives/pb-review-DP10.md`
- **Baseline**: PROTOCOL **31**, HASH **68**, tests **3,910** (main at merge `d65e7f1e`)
- **Hard constraint**: **NO engine change, NO wire change.** PROTOCOL 31 / HASH 68 must be
  unmoved and `crates/engine/src/` / `crates/card-types/src/` must be untouched. Card-def edits
  are allowed only if a completeness marker/note is itself the deliverable, and each one must be
  argued from oracle text. If the work appears to require an engine change, **stop and
  re-scope** (task brief, explicit).

## The problem

`crates/engine/tests/core/effect_choose_gate.rs` (SR-33/34/37/38 + PB-EF12) bars exactly
**three** DSL variants from `Complete` — `Effect::Choose`, `Effect::MayPayOrElse`,
`Effect::AddManaChoice` (plus the any-color family). Audit §3.1 counts **twenty-one** decision
sites across **277 of 1,139** effectively-`Complete` defs (24.3%) where the engine makes a
player's choice for them. Seventeen of those rows the gate does not name at all, so the figure
grows silently with every card authored. DP-INV (audit §1) is the invariant; the gate is
narrower than the invariant, and PB-DP10 closes that difference *at the corpus level* — it
cannot close it at the engine level, because that is what PB-DP1..DP9 were for and what the
still-open rows (DP-13/14/16/17/18/19/20/25/26/31) remain for.

## Acceptance criteria (ESM `scutemob-158`)

1. **5554** — a machine gate enumerates every def containing an engine-made choice, fails on
   unmarked new instances, and its count reconciles against §3.1's magnitude with discrepancies
   explained.
2. **5555** — decision classes fixed by PB-DP7..DP9 are distinguished from still-auto-chosen
   classes in the gate/marker taxonomy.
3. **5556** — no engine or wire change; PROTOCOL / HASH untouched; gate runs inside
   `cargo test --all`.
4. **5557** — audit PB-DP10 row updated + suite marked complete in §8; §10 re-audit triggers
   updated where the gate mechanizes them.

## Known assets to reuse (do not re-invent)

- **`pb_dp9_effect_choice.rs`'s `roster` module** — a *structurally complete* serde walk of the
  serialized `CardDefinition` (`contains_variant` / `collect`). It exists because PB-DP9's fix
  cycle found a hand-written walk had skipped `AbilityDefinition::{Spell,Triggered,Activated}::modes`,
  `{SagaChapter,LoyaltyAbility}`, split-card halves and `Effect::CoinFlip`. **A hand-written
  tree walk is a reachability claim** (audit §8, PB-DP9 row) — reuse the serde walk, and if it
  must be shared across test targets, share it rather than copying it.
- **`effect_choose_gate.rs`'s `def_uses` / `count_key_occurrences`** — same technique, older,
  plus the served-vs-unserved refinement (`registers_any_color_mana_ability`), which is the
  precedent for criterion 5555: *the same variant can be served on one path and a stub on
  another*, so a variant-name predicate alone is not a decision-class predicate.
- **`Completeness`** (`crates/card-types/src/cards/`) — SR-2's marker, and note that adding a
  variant to it is a `card-types` change, i.e. **inside the wire closure** — check the hash gate
  before assuming a new marker variant is free.

## Step Checklist

- [x] 1. Decide gate-widening vs new marker (plan) — **gate-side name-keyed baseline + union
      ratchet**; a new `Completeness` variant is wire-free (the WIP's own caution falsified,
      plan §1.1) but rejected on Architecture-Invariant-9 grounds and by the hard constraint.
      **Planning's headline finding**: the serde walk this batch inherits is **blind to unit
      variants** (`Effect::Proliferate`, `Effect::TheRingTemptsYou` serialize as bare JSON
      *strings*, and all three existing walks match object keys only), so a verbatim reuse
      would report 0 for Proliferate's ~25 `Complete` defs while looking green — plan §2.1, T2.
- [x] 2. Enumerate the decision-site taxonomy: served (DP7..DP9) / still-auto-chosen / gated —
      `crates/engine/tests/core/decision_site_walk.rs::ROWS`, 22 rows (4 SERVED, 15
      AUTO-CHOSEN, 2 GATED, 1 NO-DECISION; the plan's own tally sentence said "14 AUTO", the
      table itself sums to 15 — a plan arithmetic slip, not a code bug, corrected here).
- [x] 3. Implement the gate + fail-closed allowlist — `crates/engine/tests/core/
      decision_gate.rs`: `BASELINE` (97 entries, name-keyed exact row-set) + `T4` (per-def
      gate) + `T6` (union ratchet, `MAX_AUTO_CHOSEN_COMPLETE_UNION = 97`) + `T5` (allowlist
      liveness) + `T7` (the two hard zeros).
- [x] 4. Non-vacuity probes, both directions, including the nesting case — `T1` (per-row,
      including a `Sequence(Sequence(Proliferate))` nesting probe), `T2` (the unit-variant
      fail-before, pinned against the legacy object-key-only walk), `T3` (`PROSE_FIELDS`
      denylist, both directions), `t4_gate_logic_reddens_on_a_new_unbaselined_auto_chosen_
      complete_def` (T4's own non-vacuity, on a synthetic in-memory corpus, never touching
      `all_cards()`).
- [x] 5. §3.1 reconciliation, printed and explained — `T9`
      (`decision_site_reconciliation_report`): per-row Complete/non-Complete counts,
      all-rows union **267**, still-auto union **97**, live denominator **1,139/1,804
      (63.1%)**. Closes OOS-DP7-7.
- [x] 6. Build / test / clippy / fmt + `tools/check-defs-fmt.sh`; wire-neutrality proof — all
      green (see "What shipped" below); `git diff --name-only main -- crates/engine/src
      crates/card-types/src crates/card-defs/src` empty; PROTOCOL 31 / HASH 68 unmoved.
- [x] 7. Audit §8 / §5 / §10 / §8.1 updates — done; PB-DP10 row SHIPPED, suite marked
      COMPLETE, §3.1/§4.9 CR-cite corrections, §6 bullet, §10 mechanization ledger (3/8
      mechanized), §8.1 closes OOS-DP7-7 + files OOS-DP10-1..7 (widened to **OOS-DP10-1..11**
      by the review/fix cycle below).

## What shipped

**PB-DP10 SHIPPED** (this branch, 2026-07-27; task `scutemob-158`). TEST-ONLY, as scoped —
`git diff --name-only main -- crates/engine/src crates/card-types/src crates/card-defs/src`
is empty. PROTOCOL 31 / HASH 68 unmoved.

**Files.**
- NEW `crates/engine/tests/core/decision_site_walk.rs` — the canonical walk
  (`json_contains_variant`, `find_variant_nodes`, `PROSE_FIELDS`, `DecisionClass`, `ROWS` (22
  entries), `row_hits`/`auto_chosen_row_hits`, `is_effectively_complete`). No `#[test]`, all
  items `pub`, reached from sibling `core/` modules as `crate::decision_site_walk::…`.
- NEW `crates/engine/tests/core/decision_gate.rs` — `BASELINE` (97 entries) + 17 `#[test]`s
  (T1–T14, T16; T15/T15b dropped, see below).
- EDIT `crates/engine/tests/core/main.rs` — `mod decision_gate;` / `mod decision_site_walk;`
  added; `cargo fmt` sorted them BEFORE `mod deck_validation;` (its own alphabetical rule:
  `deci` < `deck` — the plan's placement note had the two files reversed; `cargo fmt` is
  authoritative and was applied).
- EDIT `crates/engine/tests/core/effect_choose_gate.rs` — `contains_key`/`def_uses` rewired
  onto the canonical walk (§2.3 rewire). Abort condition checked: same 14 tests, same
  offender sets, before and after.
- EDIT `crates/engine/tests/core/pb_rs1_roster_sweep.rs` — same rewire. Same 1 test, same
  printed roster (41 cards, unchanged) before and after.
- EDIT `crates/engine/tests/primitives/pb_dp9_effect_choice.rs` — doc-only note on
  `roster::json_contains_variant`'s unit-variant blindness, pointing at the canonical walk;
  no logic changed.
- EDIT `docs/audits/decision-point-audit.md` — §3.1 (superseded-by note, two CR-cite fixes,
  hard-zero framing correction; widened by the fix cycle with the encoding-blindness
  paragraph and the full per-row reconciliation delta table), §4.9 (Bolster cite fix), §5
  (baselined-rows note), §6 (superseded bullet; widened with the two further mechanisms —
  `Completeness`-note-string regex hits, the row-4 split), §8 (PB-DP10 row → SHIPPED,
  suite-COMPLETE banner; widened with the fix-cycle summary and corrected test count), §8.1
  (closes OOS-DP7-7, files **OOS-DP10-1..11** — the last four by the fix cycle; OOS-DP10-6
  gains `put_on_library` and an upper-bound caveat), §10 (mechanization ledger: 3/8
  mechanized, 1 optional-recommended-not-taken, 4 stay human; T15's dropped-digest pointer
  corrected from OOS-DP10-4/7 to **OOS-DP10-11**).

**Step 0 probe results** (via a throwaway `zz_dp10_probe.rs`, deleted before finalizing —
its findings are captured here and in the doc comments of the shipped files instead):
- **P0-a CONFIRMED**: `serde_json::to_value(Effect::Proliferate) == Value::String("Proliferate")`
  and `to_value(Effect::TheRingTemptsYou) == Value::String("TheRingTemptsYou")`; `Effect::Scry{..}`
  serializes to `{"Scry": {...}}`. Premise P4 holds exactly as predicted — the whole §2 design
  is sound.
- **P0-b CONFIRMED**: a real targeted `Triggered` node (Acidic Slime) serializes with
  `targets` as a populated array and `modes: null` both visible (no `#[serde(default)]`
  elision on serialize — confirmed separately: zero `skip_serializing_if` attributes anywhere
  in `card_definition.rs` / `game_object.rs`). Premise P3 holds.
- **P0-c**: not separately probed via Fuse/Saga (P0-b's positive was sufficient evidence
  that the serde walk reaches populated fields on real corpus data); PB-DP9's own fix-cycle
  finding that the walk reaches `modes`/`SagaChapter`/`LoyaltyAbility`/split halves was taken
  as already-established precedent (`decision_site_walk.rs`'s design doc comment cites it).
- **P0-d MEASURED** (this is what populated `BASELINE` and `MAX_AUTO_CHOSEN_COMPLETE_UNION`):
  per-row Complete/non-Complete counts exactly as printed by `T9` (see the audit's updated
  §3.1 and §8 PB-DP10 row for the full table). All-rows union **267**; still-auto union
  (P2's ~110 estimate) **97**, below the ~150-item concern threshold in plan §11 P2 — no
  fallback to ratchet-only needed.
- **P0-e/P0-f MEASURED**: informed `T12`'s collision inventory (`Discover` × 2 declarations,
  `SearchLibrary`/`Scry`/`Surveil` × 3 each) and `T13`'s `PROSE_FIELDS` completeness check —
  T13 found ONE gap the plan's own list omitted (`Characteristics.rules_text`), which turned
  out to be a false positive from an over-broad file-level scan of `game_object.rs` (that
  file also declares the runtime `Characteristics` struct, NOT reachable from
  `CardDefinition`); T13 was narrowed to scan only the `TriggeredAbilityDef` struct body
  within that file, and the plan's original `PROSE_FIELDS` list needed no changes.
- **P0-g/P0-h**: wire-neutrality confirmed empty both before and after; the rewire (§2.3)
  changed neither `effect_choose_gate.rs`'s nor `pb_rs1_roster_sweep.rs`'s printed output —
  abort condition not triggered, rewire kept.

**Falsified/corrected plan premises.**
- The plan's §3 tally sentence ("14 AUTO") does not match its own table, which sums to 15
  AUTO rows. Fixed by direct count; not a code defect, a documentation arithmetic slip in
  the plan.
- T13's design ("scan `card_definition.rs` + `game_object.rs` whole-file") produced ONE
  false positive (`rules_text`) not anticipated by the plan, because `game_object.rs` also
  declares `Characteristics` (a runtime type, not reachable from `CardDefinition`). Fixed by
  brace-matching just the `TriggeredAbilityDef` struct body instead of the whole file.
- The plan's cited reachability path for `TriggeredAbilityDef.description` was
  `Effect::CreateToken { triggered_abilities }`; the actual field lives on
  `Effect::CreateEmblem { triggered_abilities, .. }` (`card_definition.rs:2257`). Noted in
  `T3`'s doc comment; `T3` itself tests the denylist mechanism via raw JSON literals rather
  than building the full `CreateEmblem`/`TriggeredAbilityDef` chain, since the suppression
  logic is a property of the parent-key string, independent of which Rust type produced it.
- `mod` alphabetization: the plan said "between `deck_validation` and `effect_choose_gate`";
  `cargo fmt` places `decision_gate`/`decision_site_walk` BEFORE `deck_validation` (`deci` <
  `deck`). Followed `cargo fmt`, the authoritative source per SR-35's own convention.

**Dropped from the plan's test list (budget).** None fully dropped — T1–T14 and T16 all
shipped (17 tests total at implementation close: T1, T2, T3, T4 + its own non-vacuity probe,
T5, T6, T7, T8, T9, T10, T11, T12 + its non-vacuity probe, T13, T14, T16; the review/fix
cycle below adds an 18th, `t4_failure_message_names_the_bound`). **T15/T15b (the
DSL-enum-declaration roster digest) were NOT shipped** — filed as the honest-framing note in
§10's mechanization ledger. **Correction (review finding PB-DP10 #13, applied in the fix
cycle):** the WIP's original record here claimed T15 was filed as seeds OOS-DP10-4 and
OOS-DP10-7; that was wrong — OOS-DP10-4 is a `Command`-field scan (a different enum, a
different instrument) and OOS-DP10-7 is the `GameEvent` digest (`T15b`, T15's sibling, not
T15 itself). **T15's actual subject — the `Effect`/`AbilityDefinition`/
`ReplacementModification` roster digest — had no owning seed at all** and is now filed as
**OOS-DP10-11**. Per the plan's own R9 risk note, T15/T15b were explicitly the first to drop
if the batch ran long; the effort went to T12/T13/T14/T16 instead because each of those
defends a mechanism this batch actually introduces (string matching, the denylist, the
Gated-row drift check, the residual-seed honesty check), whereas T15's marginal value (per
the plan's own §9 analysis) is the *obligation*, not the *notice* — `Effect` already forces a
wire bump on a new variant regardless.

**Verification.**
- `cargo build --workspace` — clean.
- `cargo test --all` — **3,928 passing / 0 failing** (3,910 baseline + 18 new, after the
  review/fix cycle's 1 additional test). All 30 workspace test binaries green.
- `cargo clippy --all-targets -- -D warnings` — clean (one doc-comment `doc_lazy_continuation`
  lint fixed: a wrapped line starting with `+` was read as a markdown list item; reworded).
- `cargo fmt --check` — clean (after running `cargo fmt`, which reordered the two new `mod`
  lines and reflowed a handful of long `use`/`assert!` lines — no semantic change).
- `tools/check-defs-fmt.sh` — clean, 1,804 defs checked, 0 edited.
- `git diff --name-only main -- crates/engine/src crates/card-types/src crates/card-defs/src`
  — **empty**.
- `PROTOCOL_VERSION == 31`, `HASH_SCHEMA_VERSION == 68` — unmoved; `protocol_schema::` (17
  tests) and `hash_schema::` (19 tests) both green; neither history table's append-only rule
  was touched (T2/T4 checked by direct grep of the constants and by running both gate
  targets standalone).

**Measured numbers, for the record** (2026-07-27, this branch): per-row Complete counts —
`triggered_targets` 77, `search_library` 73, `proliferate` 25, `discard_cards` 13,
`wheel_hand` 10, `scry` 16, `sacrifice_permanents` 11, `may_pay_then_effect` 10,
`choose_color_or_type` 10, `look_at_top_or_route` 10, `surveil` 8, `counter_unless_pays` 7,
`modal_trigger` 4, `change_targets` 3, `put_on_library` 1, `bolster_amass` 3, `connive` 1,
`discover` 1, `may_pay_or_else` 0, `add_mana_filter_choice` 0, `choose_stub` 0,
`the_ring_tempts_you` 0. All-rows union **267**. Still-auto union / `BASELINE` size **97**.
Live denominator **1,139 / 1,804 = 63.1%** effectively-`Complete`. **All of these numbers are
UNCHANGED by the review/fix cycle below** — no `ROWS` predicate, no `BASELINE` row set, and
no `Completeness` marker was touched; the fixes are messaging, denylist-completeness, and
gate-integrity, not measurement.

---

## Review + fix cycle (2026-07-27)

`memory/primitives/pb-review-DP10.md` (commit `76b4f1cd`): 2 HIGH, 6 MEDIUM, 6 LOW. **All 14
applied.** Hard constraints re-verified after the fix cycle: `git diff --name-only main --
crates/engine/src crates/card-types/src crates/card-defs/src` still empty; PROTOCOL 31 /
HASH 68 still unmoved; 0 card-def edits.

| # | Sev | Finding | Disposition |
|---|---|---|---|
| 1 | HIGH | `BASELINE` launders class-D defs (Smuggler's Copter, Shambling Ghast) as class B; plan §5.3 triage never performed | **Fixed.** `BASELINE`'s doc comment now states explicitly that an entry asserts only "hits these rows," not oracle-correctness, and names both defs with the corrected, WORSE Shambling Ghast finding (three deviations, not two — see below). `T4`'s failure message: "reviewed acknowledgement" → "recorded acknowledgement." Filed **OOS-DP10-8** with the orchestrator's corrected facts: Smuggler's Copter is the 20th instance of audit §5's DP-12 class (19 already marked `known_wrong`), not a novel defect; Shambling Ghast has a THIRD deviation the review's own text missed — it grants `KeywordAbility::Decayed`, which the printed card does not have at all, in addition to the permanent-counter and stale-`oracle_text` issues the review found. **Neither card def edited** — per plan §5.3, file, do not demote. |
| 2 | HIGH | The gate is blind to a decision the DSL never encoded (Smuggler's Copter is the live example) and criterion 5554's wording doesn't say so | **Fixed.** Added the bound in the three places the fix specified: `decision_gate.rs`'s module doc (new paragraph after the "cannot stop the growth" one), the audit §8 PB-DP10 row, and §3.1's superseded note. Filed **OOS-DP10-9**. |
| 3 | MEDIUM (test-validity, treated as fix-phase HIGH per convention) | T4's non-vacuity probe never executed T4's offender loop, in particular the subset/superset mismatch arm | **Fixed properly, not minimally**, per the orchestrator's explicit instruction. Extracted `offenders()` (T4's exact logic) and `t4_message()` (T4's exact failure text) as standalone functions; T4 now calls both. Rewrote the probe to build a 3-def synthetic corpus and assert all three outcomes: (a) an unbaselined `Complete` Proliferate def is an offender; (b) a def baselined with a recorded row set that is a SUPERSET of its actual hits (the "tighten the entry" / subset arm — previously uncovered anywhere) is an offender; (c) a non-`Complete` def carrying the identical site is NOT an offender. **Verified non-vacuous by execution**: temporarily neutered the mismatch arm's guard (`Some(recorded) if false && recorded != &hits`), confirmed outcome (b)'s assertion goes red naming the exact synthetic def, restored, confirmed green again. |
| 4 | MEDIUM | Wrong CR cite `106.12` for `ChooseColor` (that's the tap-for-mana definition) | **Fixed.** `decision_site_walk.rs`'s `choose_color_or_type` row now cites `"614.12a (as-enters, ReplacementModification) / 608.2d (resolution-time Effect)"`, applied verbatim per the orchestrator's own CR verification this session. |
| 5 | MEDIUM | Module doc cites `t4_failure_message_names_the_bound`, which didn't exist | **Fixed by writing the test** (the review's preferred option). Extracted `t4_message()` so the new test can assert its text contains the four load-bearing phrases: "CANNOT STOP THE GROWTH," "Mark the def non-Complete," "Add a BASELINE entry," "is NOT an exit for this batch." **Verified non-vacuous by execution**: temporarily corrupted the "Add a BASELINE entry" phrase in `t4_message`'s format string, confirmed the new test goes red naming the missing phrase, restored, confirmed green again. This is the 18th test (+1 over the implementation phase's 17). |
| 6 | MEDIUM | `PROSE_FIELDS`-completeness test recognized only literal `String` types, missing newtype-over-`String` (`SubType`, `CardId`) fields and 3 contributing files | **Fixed via option (a)** (widen, per the orchestrator's explicit preference over documenting the gap). `string_field_name` now recognizes `SubType`/`Option<SubType>`/`Vec<SubType>`/`Option<Vec<SubType>>`/`CardId`/`Option<CardId>` in addition to the three `String` shapes — `SubType`/`CardId` are the ONLY two newtype-over-`String` types in the whole `card-types` crate (verified by grepping every `pub struct X(pub String)` declaration), so this closes the channel completely, not partially. Scan extended to `state/types.rs`, `state/replacement_effect.rs`, `state/targeting.rs` (whole-file, matching the fix's own scope). Cross-checked with a standalone Python replica of the widened scan against all 5 files: found exactly `card_id`, `default`, `exclude_subtypes`, `has_name`, `has_subtype`, `has_subtypes`, `melded_card_id`, `name`, `onto_subtype`, `oracle_text`, `pair_card_id`, `prompt`, `spell_subtype_filter`, `subtype` — all 14 are now in `PROSE_FIELDS` (8 new: `pair_card_id`, `melded_card_id`, `onto_subtype`, `has_subtype`, `has_subtypes`, `exclude_subtypes`, `spell_subtype_filter`, `default`). Audit's over-claim sentence in the §8 row corrected to name the widened scope explicitly. |
| 7 | MEDIUM | §3.1's reconciliation explains a minority of the drift; two mechanisms named, several deltas unexplained | **Fixed properly**, per the orchestrator's explicit instruction. Added the full per-row audit-vs-measured delta table to §3.1's superseded note (15 rows), with a mechanism per non-trivial delta. Two new mechanisms added to §6: (iii) the audit's regex counted variant names inside `Completeness` note strings (`connive` 2→1, `put_on_library` 3→1, both confirmed by finding the actual defs: only `raffines_informant.rs` / `brainstorm.rs` are real, `spymasters_vault.rs` / `witchs_cottage.rs` / `gravepurge.rs` mention the variant only in a note); (iv) the audit's row 4 bundled two predicates, and the split is exact (13 + 10 = 23). Per the orchestrator's explicit instruction, the three deltas with no established mechanism (`search_library` 74→73, `may_pay_then_effect` 11→10, `modal_trigger` 5→4) are written as **"unexplained, ±1, within regex noise"** rather than assigned an invented mechanism. |
| 8 | MEDIUM | `look_at_top_or_route` over-includes (Chaos Warp, Coiling Oracle have no real choice); the row's own `why` claims otherwise | **Minimum fix applied**, per the orchestrator's explicit instruction (do not split the row; `MAX_AUTO_CHOSEN_COMPLETE_UNION`/`BASELINE` stay frozen against 97). `decision_site_walk.rs`'s row `why_not_flagged_is_wrong` rewritten to state the row is an UPPER BOUND, naming both the real-choice members (Goblin Ringleader) and the no-choice members (Chaos Warp, Coiling Oracle). Same caveat added to audit §8.1's **OOS-DP10-6**. |
| 9 | LOW | `wheel_hand`'s NO-DECISION `why` overreaches (CR 404.3 graveyard order IS engine-chosen, just not the pick this row counts) | **Fixed.** Narrowed the `why` text to distinguish "no 'which card' pick (CR 701.9b)" from the still-real CR 404.3 graveyard-order choice, pointing at the new seed. Filed **OOS-DP10-10**. |
| 10 | LOW | Stale doc reference to `contains_key` in `effect_choose_gate.rs` (deleted by the PB-DP10 rewire) | **Fixed.** Retargeted to `def_uses`. |
| 11 | LOW | T12's collision inventory omits the one row whose predicate spans two enums by design (`choose_color_or_type`) and doesn't scan `replacement_effect.rs` | **Fixed.** Added `state/replacement_effect.rs` to the scanned set; pinned `("ChooseColor", 1)` and `("ChooseCreatureType", 2)`, both verified by direct grep before pinning (1 declaration in `replacement_effect.rs` for `ChooseColor`; one each in `card_definition.rs` and `replacement_effect.rs` for `ChooseCreatureType`). |
| 12 | LOW | `OOS-DP10-6` omits `put_on_library` (measured 1) from the successor-queue ranking | **Fixed.** Added `put_on_library 1` to the ranked list in §8.1. |
| 13 | LOW | The dropped T15 has no owning seed; the WIP's record pointed at the wrong two seeds | **Fixed.** Filed **OOS-DP10-11** owning T15's actual subject (the `Effect`/`AbilityDefinition`/`ReplacementModification` roster digest). Corrected the WIP's "Dropped" paragraph (above) and both §10 in-doc pointers that had said "OOS-DP10-4's sibling scope note." |
| 14 | LOW | T9 (and T7/T8/T10/T11) re-serialize each `CardDefinition` once per row instead of once per def | **Fixed.** Hoisted `serde_json::to_value` out of the row loop in T7, T8, T9, T10, T11 — each now serializes the corpus once (`Vec<Value>`, zipped against `defs` for the row-by-row filter) and indexes into it. T9 alone drops from ~40,000 serializations to ~1,804. |

**Seeds filed by this cycle**: **OOS-DP10-8, -9, -10, -11** in `docs/audits/decision-point-audit.md` §8.1, in the existing row format. Every place that counted the seeds (the §8 PB-DP10 row, this WIP's own step-7 line, and the file-inventory bullet) updated from "OOS-DP10-1..7" to "**OOS-DP10-1..11**".

**Test count, re-derived after the fix cycle**: `decision_gate.rs` now has **18** `#[test]`s
(17 at implementation close + 1 new: `t4_failure_message_names_the_bound`). Workspace total
**3,928** (3,910 baseline + 18), verified by `awk` summing every `test result:` line across
all 31 workspace test binaries — see Verification above.

**Non-vacuity proof method** (both F3 and F5, per the orchestrator's explicit request):
temporarily broke the mechanism each test defends, ran the single test, confirmed a red
failure naming the exact defect, reverted, confirmed green again. Both `git diff`s were
inspected after reverting to confirm no `TEMPORARY`/`if false` residue was left behind (see
Verification's `git diff --name-only` line and the full `git diff` review before close).

**What could not be done / deferred**: nothing from the review's 14 findings was deferred —
all were applied. The review's own closing recommendation ("re-run the Finding 1 oracle
spot-check across all 97 `BASELINE` entries") was **not** performed in this cycle — the
orchestrator's brief scoped Finding 1's fix to the seed + doc-comment + message-wording
triple, not a full 97-entry re-triage, and OOS-DP10-8 says so explicitly ("the remaining 95
entries have not been triaged").

## Fail-closed proven end-to-end, on a real card def (orchestrator, 2026-07-27)

The review and the fix cycle both proved T4's logic against a *synthetic* corpus. The
acceptance criterion's claim is about the real one, so the orchestrator ran it end-to-end:
`Effect::Proliferate` was temporarily added to `crates/card-defs/src/defs/lightning_bolt.rs`
— a `Complete` def not in `BASELINE` — and `cargo test -p mtg-engine --test core
decision_gate::` was run against the real `all_cards()`:

```
test decision_gate::no_complete_def_introduces_an_unrecorded_auto_chosen_decision ... FAILED
test decision_gate::auto_chosen_complete_union_is_ratcheted ... FAILED
  Lightning Bolt is NOT in BASELINE but hits {"proliferate"}. Lightning Bolt hits
  proliferate (CR 701.34a, effects/mod.rs (Proliferate) -- auto-selects all eligible)
test result: FAILED. 16 passed; 2 failed
```

The def was restored from a backup and the target went green again (18 passed / 0 failed),
`git status` clean. **Two** tests catch the new instance, not one, and the message names the
card, the row, the CR and the engine site — which is what criterion 5554 asks for. Recorded
in audit §8's PB-DP10 row.
