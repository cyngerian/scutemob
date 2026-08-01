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

- [ ] 1. `jadar_ghoulcaller_of_nephalia.rs` — `intervening_if`, corrected `oracle_text`,
      replace stale TODO with a dated note (plan §2.1).
- [ ] 2. `ophiomancer.rs` — `intervening_if`, `partial` → `Complete` (plan §2.2).
- [ ] 3. `dwynen_s_elite.rs` — author the ETB trigger + `exclude_self` gate, `inert` →
      `Complete` (plan §2.3).
- [ ] 4. `emeria_the_sky_ruin.rs` — `intervening_if`, explicit marker + "you may" note (plan §2.4/§5).
- [ ] 5. Re-affirm and date the three DEFER notes (plan §1 rows 5-7).
- [ ] 6. `crates/engine/tests/primitives/pb_dx3b_stale_blocker_bucket.rs` — T1..T12, each
      **observed** pre-fix per plan §3; register the `mod` line (SR-9a).
- [ ] 7. Golden script `combat/191` reconciled by strengthening (plan §4).
- [ ] 8. Gates: empty engine/card-types diff, PROTOCOL 32 / HASH 69, build/clippy/fmt/
      check-defs-fmt/full tests, PB-DP10 `decision_gate` suites green (plan §6).
- [ ] 9. Review (`primitive-impl-reviewer`) → fix cycle → close-out (plan §7).
