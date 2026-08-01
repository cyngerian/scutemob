# Primitive WIP — PB-DX3 (two stale blocker notes: garruks_uprising + inventors_fair)

<!-- last_updated: 2026-08-01 -->

> Previous occupant: **PB-DX2 (gate the resolution-time commands nothing gates) — SHIPPED**,
> `scutemob-162`, PROTOCOL 32 / HASH 69 both unmoved, tests **3,974** on the branch.
> Its WIP file is preserved verbatim at `memory/primitives/pb-wip-DX2-archive.md`.
> Authoritative queue: `memory/primitives/seed-rerank-2026-07-27.md` §4, **PB-DX1..PB-DX18**.

- **PB**: PB-DX3 — rank 3 of the PB-DX queue. Seed **OOS-DP6-3**.
- **Task**: `scutemob-164`
- **Branch**: `feat/pb-dx3-two-stale-blocker-notes-garruksuprising-inventorsfair`
- **Class**: **CARD YIELD, ZERO ENGINE.** 2 flips (`partial` → `Complete`), 0 engine lines.
- **Phase**: implement
- **Plan**: `memory/primitives/pb-plan-DX3.md` (premise fully re-verified there, §1)
- **Review file**: `memory/primitives/pb-review-DX3.md`
- **Wire prediction**: PROTOCOL **32** / HASH **69** unmoved. Unlike PB-DX1/DX2 this is not a
  hypothesis about engine consequences — the batch touches **no** engine file, so the falsifier
  is trivial: a non-empty `git diff` over `crates/engine/src`.

## Steps

- [x] 1. `garruks_uprising.rs` — ETB `intervening_if`, drop TODO, flip to `Complete` (plan §2.1).
      Done exactly as planned: `intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
      count: 1, filter: TargetFilter { has_card_type: Some(CardType::Creature), min_power:
      Some(4), ..Default::default() } })` on the first (ETB) ability; stale TODO deleted;
      `completeness: Completeness::Complete`.
- [x] 2. `inventors_fair.rs` — add the missing upkeep trigger, add `activation_condition`,
      drop 4 TODOs, flip to `Complete` (plan §2.2). Upkeep `AbilityDefinition::Triggered`
      (`TriggerCondition::AtBeginningOfYourUpkeep`, `Effect::GainLife` Fixed(1),
      `intervening_if: Some(Condition::YouControlNOrMoreWithFilter { count: 3, filter: {
      has_card_type: Some(Artifact) } })`) inserted FIRST in `abilities` per plan; search
      ability gained the identical `activation_condition`; all 4 stale TODOs removed;
      `completeness: Completeness::Complete`.
- [x] 3. New `crates/engine/tests/primitives/pb_dx3_stale_blocker_notes.rs` + `main.rs` `mod`
      line; T1..T10 per plan §3, every required probe fail-before, every one CR-cited.
      **Found and fixed a pre-existing bug in the runner's own T1/T2/T3 hand-count math**
      (baseline captured BEFORE casting Garruk's Uprising double-counted the -1 from the
      spell leaving hand against the +1 draw, which would have silently vacated T1's and
      T3's negative assertions) — rebaselined AFTER the cast command, and gave T3 a real
      library card so "no draw" isn't a silent empty-library no-op. Confirmed genuinely
      fail-before, pre-edit, for T1/T3/T6(disambiguates T5)/T7/T8; T5 itself is *vacuously
      satisfied* pre-fix (the upkeep ability didn't exist at all) exactly as plan §3
      predicts, not a "fail". 10/10 tests pass post-edit.
- [x] 4. Gates per plan §4 (zero-engine-diff, wire unmoved, build/clippy/fmt/check-defs-fmt,
      full workspace tests, authoring-report regen) — see close-out report.
- [ ] 5. Review phase (`card-batch-reviewer` — oracle-text risk is the batch's whole risk surface)
- [ ] 6. Close-out bookkeeping (CLAUDE.md snapshot delta, workstream-state, seed-rerank §4 banner)
