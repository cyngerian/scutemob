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
- [x] 5. Review phase — `/review` (Opus, read-only) wrote `memory/primitives/pb-review-DX3.md`:
      **1 MEDIUM / 5 LOW, 0 HIGH.** Both flips justified clause-by-clause against oracle
      text + all 8 rulings; all 4 hard gates green at review time.
- [ ] 6. Close-out bookkeeping (CLAUDE.md snapshot delta, workstream-state, seed-rerank §4 banner)

## Fix cycle (2026-08-01, `scutemob-164`)

Applied all 6 review findings from `pb-review-DX3.md`.

- **MEDIUM-1** — T1's pre-fix hand-count claim was unreproducible (fixture had no library
  object; drawing from an empty library sets `has_lost`, not hand+1). Gave T1 a real
  library card (mirroring T2/T3), then **empirically re-derived** the pre-fix behaviour:
  temporarily reverted `garruks_uprising.rs`'s ETB `intervening_if` to `None`, swapped T1's
  two panicking assertions for `eprintln!` so execution could reach the hand-count read, ran
  it, observed `stack_objects()` held exactly 1 object and `hand_before=0 → hand_after=1`
  (matches the original claim exactly, now grounded in a real run), then restored both
  files. Also re-verified T3/T5/T6/T7/T8's pre-fix notes against the same standard — T3 was
  already genuinely observed (implement-phase fail-before run); T5-T8 needed
  `inventors_fair.rs`'s upkeep-trigger block and `activation_condition` temporarily reverted
  and re-run the same way. All five held with `RE-VERIFIED` markers added to the module doc;
  none needed correction, all were either already-genuine fail-before observations or
  directly-readable static facts.
- **LOW-2** — T4 rewritten from a single `check_triggers`-on-synthetic-event assertion into
  a real cast-through-`Command::CastSpell` + `pass_all` end-to-end test with three
  sub-scenarios: (a) power-3 creature under the controller must NOT trigger
  (`min_power: Some(4)` pinned), (b) power-4 creature under the controller MUST trigger and
  the net hand-count delta proves the fired effect is genuinely a draw, (c) power-4+
  creature under an OPPONENT's control must NOT trigger (`controller: You` pinned). New
  `plain_creature_def` + `garruks_uprising_third_ability_fixture` helpers.
- **LOW-3** — T9 now also asserts the un-chosen candidate remains in the library (not swept
  along with the announced one) and that a `GameEvent::LibraryShuffled` event for p1 is
  actually present in `AnswerEffectChoice`'s returned events, plus a comment explaining WHY
  `candidates[1]` provably isn't the pre-PB-DP9 lowest-ObjectId auto-pick.
- **LOW-4** — Added an in-def comment next to `inventors_fair.rs`'s `reveal: true` pointing
  at OOS-DP9-9 (the flag is inert; `effects/mod.rs` destructures `reveal: _`). No marker or
  flag change.
- **LOW-5** — Reordered `inventors_fair.rs`'s header comment (upkeep, mana, search) to match
  the `abilities` vec's oracle-text order.
- **LOW-6** — `pb-plan-DX3.md`'s `matches_filter` cite now names the symbol, not a line
  number.

**Gates after the fix cycle**: `git diff main --stat -- crates/engine/src crates/card-types/src`
empty; `git diff main -- crates/engine/src/rules/protocol.rs crates/engine/src/state/hash.rs`
empty (PROTOCOL 32 / HASH 69 unmoved); `cargo build --workspace` clean; `cargo clippy
--workspace --all-targets -- -D warnings` clean; `cargo fmt --check` clean;
`tools/check-defs-fmt.sh` clean (1,804 defs); `cargo test --all` **3,998 passing / 0
failing** (10 tests in the new module, all green, up from the implement-phase pin).
