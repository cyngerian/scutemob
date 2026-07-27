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
- **Phase**: implement
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
      mechanized), §8.1 closes OOS-DP7-7 + files OOS-DP10-1..7.

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
  hard-zero framing correction), §4.9 (Bolster cite fix), §5 (baselined-rows note), §6
  (superseded bullet), §8 (PB-DP10 row → SHIPPED, suite-COMPLETE banner), §8.1 (closes
  OOS-DP7-7, files OOS-DP10-1..7), §10 (mechanization ledger: 3/8 mechanized, 1
  optional-recommended-not-taken, 4 stay human).

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
shipped (17 tests total: T1, T2, T3, T4 + its own non-vacuity probe, T5, T6, T7, T8, T9, T10,
T11, T12 + its non-vacuity probe, T13, T14, T16). **T15/T15b (the DSL-enum-declaration roster
digest) were NOT shipped** — filed as the honest-framing note in §10's mechanization ledger
and as seeds OOS-DP10-4 (Command accepted-and-discarded scan) and OOS-DP10-7 (`GameEvent`
sibling-answer roster digest). Per the plan's own R9 risk note, T15/T15b were explicitly the
first to drop if the batch ran long; the effort went to T12/T13/T14/T16 instead because each
of those defends a mechanism this batch actually introduces (string matching, the denylist,
the Gated-row drift check, the residual-seed honesty check), whereas T15's marginal value
(per the plan's own §9 analysis) is the *obligation*, not the *notice* — `Effect` already
forces a wire bump on a new variant regardless.

**Verification.**
- `cargo build --workspace` — clean.
- `cargo test --all` — **3,927 passing / 0 failing** (3,910 baseline + 17 new). All 30
  workspace test binaries green.
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
Live denominator **1,139 / 1,804 = 63.1%** effectively-`Complete`.
