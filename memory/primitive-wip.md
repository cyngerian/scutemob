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
- **Phase**: implement
- **Plan**: `memory/primitives/pb-plan-DX3b.md` (premise fully re-verified there, §0)
- **Review file**: `memory/primitives/pb-review-DX3b.md`
- **Wire prediction**: PROTOCOL **32** / HASH **69** unmoved. Falsifier is trivial — a non-empty
  `git diff` over `crates/engine/src` **or** `crates/card-types/src`.
- **Baseline**: 3,998 / 0 on this branch (main pin at the `scutemob-164` merge).

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
      no behavior change). `guardian_project`: found the note's (a) half (is_nontoken
      "ignored by matches_filter") is ITSELF now stale — PB-AC0 wired
      `triggering_creature_filter` (including `is_nontoken`) through the creature-ETB
      dispatch path in `rules/abilities.rs`, confirmed by reading the forwarding site in
      `testing/replay_harness.rs`. (b) half (name-uniqueness Condition) re-verified as
      still genuinely absent. Corrected the note to be accurate rather than perpetuating
      the stale claim; did NOT apply the (a) fix (stays `known_wrong`, outside this
      batch's declared 4-def scope, flagged as a candidate for a follow-up micro-batch —
      see close-out report).
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
      check-defs-fmt/full tests, PB-DP10 `decision_gate` suites green (plan §6). See
      close-out report for exact results. **Deviation from plan §6's "none of the four
      appears in BASELINE today" assumption**: `emeria_the_sky_ruin`'s demotion legitimately
      moved TWO pinned test floors (both lower-bound `>=` assertions on `Complete`-def
      counts, not the `BASELINE` map itself) — `core::decision_gate::
      canonical_walk_reproduces_pb_dp8_roster` (77→76, `triggered_targets` row) and
      `core::completeness_deviation_scan::the_marker_detector_is_not_vacuous` (662→661,
      plus its hardcoded "669" message text corrected to "661" — that number had ALREADY
      gone stale between PB-OS11 and this batch, independent of PB-DX3b). Both are
      pre-existing pinned-floor gates whose whole job is to track the corpus, updated with
      dated derivation comments per this file's own established convention (3rd and 4th
      such update respectively), not silently patched. Full derivation and both diffs are
      in the close-out report.
- [ ] 9. Review (`primitive-impl-reviewer`) → fix cycle → close-out (plan §7).
