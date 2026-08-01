# Primitive WIP — PB-DX3b (OOS-DX3-1: the stale-blocker bucket, remainder)

<!-- last_updated: 2026-08-01 -->

> Previous occupant: **PB-DX3 (two stale blocker notes: `garruks_uprising` + `inventors_fair`) —
> SHIPPED**, `scutemob-164`, PROTOCOL 32 / HASH 69 unmoved, tests **3,998** on the branch.
> Its WIP file is preserved verbatim at `memory/primitives/pb-wip-DX3-archive.md`.
> Authoritative queue: `memory/primitives/seed-rerank-2026-07-27.md` §4, **PB-DX1..PB-DX18**.
> This is a **queue insert ahead of PB-DX4**, taken on the post-DX3 banner's own recommendation
> (live-wrong `Complete`, card-def only — the tier that put PB-DX1 at rank 1).

- **PB**: PB-DX3b. Seed **OOS-DX3-1** (`docs/audits/decision-point-audit.md` §8.1).
- **Task**: `scutemob-166`
- **Branch**: `feat/pb-dx3b-the-oos-dx3-1-insert-jadar-live-wrong-complete-ophio`
- **Class**: **CORRECTNESS (2 live-wrong `Complete` defs) + card yield. ZERO ENGINE LINES.**
- **Phase**: fix — COMPLETE. Review `memory/primitives/pb-review-DX3b.md` (0 HIGH / 5 MEDIUM /
  7 LOW, all four completeness moves ruled justified, nothing reverts); all 12 findings applied
  (see step 9 below for the itemised list and re-run gate results). Close-out bookkeeping
  (CLAUDE.md / workstream-state.md / seed-rerank doc) is owned by the coordinator, not this file.
- **Plan**: `memory/primitives/pb-plan-DX3b.md` (premise fully re-verified there, §0)
- **Review file**: `memory/primitives/pb-review-DX3b.md`
- **Wire prediction**: PROTOCOL **32** / HASH **69** unmoved. Falsifier is trivial — a non-empty
  `git diff` over `crates/engine/src` **or** `crates/card-types/src`.
- **Baseline**: **4,008 / 0** on this branch's merge base (`0eb5a0d4`). *Not* the 3,998 pin from
  the `scutemob-164` merge — `scutemob-165` (M11-local S4) merged after it and brought
  `crates/view-model/src/tests.rs` with it, worth +10. Final: **4,022** (+14: T1..T12 from
  implement, T13/T14 from the fix cycle).

## Scope, in one line each

| def | today | action |
|---|---|---|
| `jadar_ghoulcaller_of_nephalia` | **`Complete`, live-wrong** | gate + fix the stored `oracle_text`; stays `Complete` |
| `ophiomancer` | `partial` | gate; → `Complete` |
| `dwynen_s_elite` | `inert` (ability absent) | **author** the ability + gate; → `Complete` |
| `emeria_the_sky_ruin` | **`Complete` by `#[default]`, live-wrong** — found this batch, seed missed it | gate the live-wrong half; demote to explicit `partial` for the unimplemented "you may" (plan §5) |
| `vampire_socialite` · `thousand_faced_shadow` · `guardian_project` | partial / partial / known_wrong | **DEFER** — blockers re-affirmed against the current `Condition` enum, notes dated |

## Steps

- [x] 1. `jadar_ghoulcaller_of_nephalia.rs` — `intervening_if`, corrected `oracle_text`,
      replace stale TODO with a dated note (plan §2.1). Marker stays `Complete`.
- [x] 2. `ophiomancer.rs` — `intervening_if`, `partial` → `Complete` (plan §2.2).
- [x] 3. `dwynen_s_elite.rs` — author the ETB trigger + `exclude_self` gate, `inert` →
      `Complete` (plan §2.3).
- [x] 4. `emeria_the_sky_ruin.rs` — `intervening_if` fixed (live-wrong half closed);
      explicit `Completeness::partial(...)` set (plan §2.4/§5) — see §5 falsifier note below.
      Genuinely searched for a free-optional DSL mechanism (`MayPayThenEffect` requires a
      `Cost`; even a trivial/free cost buys nothing since the pay-vs-decline choice is
      non-interactive and deterministically always pays; `MayPayOrElse` is a documented
      STUB; PB-DP9's channel is search/scry/surveil-only) — none found, plan's demotion
      applied as written. Net coverage **+1**, not +3: +2 flips up (`ophiomancer`,
      `dwynen_s_elite`) against −1 for `emeria` (measured, `tools/authoring-report.py`:
      **1,142 → 1,143**). The plan's §5 originally said "+2" — its own arithmetic slip,
      corrected there at close-out.
- [x] 5. Re-affirmed and dated the three DEFER notes (plan §1 rows 5-7). `vampire_socialite`
      and `thousand_faced_shadow` re-verified as still genuinely blocked (dated notes added,
      no behavior change; `vampire_socialite`'s note now names BOTH its DSL gaps, not just
      the first — see fix cycle Finding 10). `guardian_project`: found the note's (a) half
      (is_nontoken "ignored by matches_filter") is ITSELF now stale — PB-AC0 wired
      `triggering_creature_filter` (including `is_nontoken`) through the creature-ETB
      dispatch path in `rules/abilities.rs`, confirmed by reading the forwarding site in
      `testing/replay_harness.rs`. (b) half (name-uniqueness Condition) re-verified as
      still genuinely absent. Corrected the note to be accurate rather than perpetuating
      the stale claim; did NOT apply the (a) fix (stays `known_wrong`, outside this
      batch's declared 4-def scope). Filed as its own seed row, **OOS-DX3b-1**
      (`docs/audits/decision-point-audit.md` §8.1, fix cycle Finding 11), rather than left
      as an untracked TODO — a candidate for a follow-up micro-batch, card-def only.
- [x] 6. `crates/engine/tests/primitives/pb_dx3b_stale_blocker_bucket.rs` — T1..T12, each
      **observed** pre-fix per plan §3 (via temporary revert + instrumented re-run, then
      restored); register the `mod` line (SR-9a) — done in
      `crates/engine/tests/primitives/main.rs`. T3's scenario deviates from the plan's
      literal wording (documented in the test module doc as a correction, not silently) —
      Jadar's intervening-if is negated, so the CR-correct "queue true / resolution false"
      analogue is the opposite board-state sequence from what the plan's table describes.
- [x] 7. Golden script `combat/191` reconciled by strengthening — final `assert_state` now
      asserts the Zombie token itself; `generation_notes` rewritten to current behaviour;
      `tags`/`confidence` updated; a NEW dated dispute entry appended (original untouched,
      PB-DX2 precedent) recording that the resolution-time gap was actually already closed
      by the pre-existing B14 card-registry fallback, and this batch closed the def's own
      intervening-if gap. Validated via `SCRIPT_FILTER=191`.
- [x] 8. Gates: empty engine/card-types diff, PROTOCOL 32 / HASH 69, build/clippy/fmt/
      check-defs-fmt/full tests, PB-DP10 `decision_gate` suites green (plan §6). **There is
      no separate close-out report artifact in this tree** — this WIP file and
      `memory/primitives/pb-review-DX3b.md` are the two authoritative records; both
      derivations below are inlined here directly rather than deferred. **Deviation from
      plan §6's "none of the four appears in BASELINE today" assumption**:
      `emeria_the_sky_ruin`'s demotion legitimately moved TWO pinned test floors (both
      lower-bound `>=` assertions on `Complete`-def counts, not the `BASELINE` map itself):
      (1) `core::decision_gate::canonical_walk_reproduces_pb_dp8_roster` 77 → 76 —
      Emeria is the only one of the four defs whose `AbilityDefinition::Triggered` carries
      a non-empty `targets` (`TargetCardInYourGraveyard`), and
      `decision_site_walk.rs::is_effectively_complete` is exactly
      `completeness == Complete`, so demoting her removes precisely one match from the
      `triggered_targets` row; Jadar/Ophiomancer/Dwynen all have `targets: vec![]` and
      cannot add to it. (2) `core::completeness_deviation_scan::
      the_marker_detector_is_not_vacuous` 662 → 661 — `1804 − 1142 = 662` (merge base) and
      `1804 − 1143 = 661` (this branch, `ophiomancer`+`dwynen_s_elite` Complete −2,
      `emeria_the_sky_ruin` Complete → partial +1, net −1), and its hardcoded "669" message
      text corrected to "661" (that number had ALREADY gone stale between PB-OS11 and this
      batch, independent of PB-DX3b — eight intervening batches moved the true count with
      nothing re-deriving the comment). Both are pre-existing pinned-floor gates whose whole
      job is to track the corpus, updated with dated derivation comments per this file's own
      established convention (3rd and 4th such update respectively), not silently patched.
      Fix cycle (Finding 5) additionally rewrote `the_marker_detector_is_not_vacuous`'s
      failure MESSAGE — the zero-margin `>= 661` pin was kept, but the message previously
      named only "MARKER_FRAGMENTS is broken" as the cause; it now names both possible
      causes (detector bug vs. an ordinary unrelated `Complete` flip) per
      `decision_gate.rs:923-924`'s documented `>=`-floor convention.
- [x] 9. Review (`primitive-impl-reviewer`) → fix cycle → close-out (plan §7).
      **Review**: `memory/primitives/pb-review-DX3b.md` — 0 HIGH / 5 MEDIUM / 7 LOW. All
      four completeness moves ruled justified clause-by-clause; nothing reverted.
      **Fix cycle** (`scutemob-166`, 2026-08-01), all 12 findings applied:
      1. (MEDIUM) `emeria_the_sky_ruin.rs`: dropped the spurious `Legendary` supertype
         (`types: types(&[CardType::Land])`), fixed the leading file comment and the
         in-ability comment, extended the `partial` marker note to name this fix.
      2. (MEDIUM) added T13 to `pb_dx3b_stale_blocker_bucket.rs` — Jadar, an opponent's
         decayed creature does not suppress the trigger (mirrors T7's shape).
      3. (MEDIUM) rewrote the three stale-prose sites in golden script `combat/191`
         (`:239` step_note, `:243` description, `:254` note) to current behaviour.
      4. (MEDIUM) filled dispute #1's `resolution`/`resolved_by`/`resolved_date` in
         `combat/191` (description left byte-for-byte verbatim).
      5. (MEDIUM) `completeness_deviation_scan.rs`: kept the exact `>= 661` pin (option
         (b)) and rewrote the failure message to name both possible causes (detector
         bug vs. an ordinary unrelated `Complete` flip), per `decision_gate.rs:923-924`'s
         documented convention.
      6. (LOW) `count_snakes`/`count_elf_warriors` now read `calculate_characteristics`
         (layer-resolved), matching `count_decayed_creatures`'s existing pattern.
      7. (LOW) added T14 — Emeria, an opponent's 2 Plains do not count toward p1's
         threshold of 7 (p1 has 6, board-wide total 8, trigger must not queue).
      8. (LOW) inlined both pinned-floor derivations directly into this file's step 8
         (no separate close-out report exists in this tree; this file + the review file
         are the two authoritative records).
      9. (LOW) trimmed `combat/191`'s `cr_sections_tested` to what the script actually
         exercises (dropped 701.17a/704.3 — no combat occurs) and added a
         `generation_notes` addendum naming the filename/content mismatch
         (`..._eoc_sacrifice.json` implies combat; none occurs).
      10. (LOW) `vampire_socialite.rs`'s `partial` marker string now names BOTH DSL
          gaps (the intervening-if AND the conditional-ETB-replacement wrong-polarity
          gap), not just the first.
      11. (LOW) filed the guardian_project `is_nontoken` half as **OOS-DX3b-1** in
          `docs/audits/decision-point-audit.md` §8.1.
      12. (LOW) close-out bookkeeping (CLAUDE.md / workstream-state.md /
          seed-rerank-2026-07-27.md) is owned by the coordinator, deliberately not
          touched here.
      **Post-fix gates, all re-run and green**: `cargo build --workspace`; `cargo test
      --all` — **4,022 passing / 0 failing** (14/14 `pb_dx3b_stale_blocker_bucket`
      tests including the two new T13/T14; `decision_gate::
      canonical_walk_reproduces_pb_dp8_roster` and `completeness_deviation_scan::
      the_marker_detector_is_not_vacuous` both green); `cargo clippy --workspace
      --all-targets -- -D warnings` — clean; `cargo fmt --check` — clean;
      `tools/check-defs-fmt.sh` — clean (1,804 defs; needed a `--fix` pass after this
      fix cycle's longer marker strings pushed two lines over 100 columns —
      `emeria_the_sky_ruin.rs` and `vampire_socialite.rs`); `SCRIPT_FILTER=191 cargo
      test -p mtg-engine --test scripts run_all_scripts -- --nocapture` — 1 of 271
      discovered scripts ran and passed. `git diff --stat main -- crates/engine/src
      crates/card-types/src` — **empty**. `PROTOCOL_VERSION = 32`,
      `HASH_SCHEMA_VERSION = 69` — both unmoved. Coverage unchanged by the fix cycle
      (no completeness flips): **1,143/1,804 = 63.4%** (already regenerated at
      implement-phase close; `docs/authoring-status.md` confirms).
