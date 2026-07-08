---
pb: PB-AC2
title: Optional-cost wrapper & counter-tax primitives
phase: closed
plan_file: memory/primitives/pb-plan-AC2.md
review_file: memory/primitives/pb-review-AC2.md
---

# PB-AC2 — Optional-cost (beneficial-pay) wrapper & counter-tax

## Scope
1. Optional-cost **beneficial-pay** wrapper on triggered effects: the general
   "you may pay/sacrifice/discard X. If you do, <effect>." pattern.
   - Distinct from existing `Effect::MayPayOrElse` which has TAX semantics
     ("pay or suffer or_else"). This is the beneficial counterpart
     (proposed name in card TODOs: `Effect::MayPayThenEffect`).
2. `Effect::CounterUnlessPays { cost }` — caster-side counter-tax (Mana Leak
   pattern: "Counter target spell unless its controller pays {cost}").

## CR refs (VERIFY via mtg-rules MCP — advisory)
- 118.8 (paying costs), 603.2 (triggered abilities), 701.5 (counter)

## Candidate blocked cards (verify roster from oracle text)
- Beneficial-pay: crossway_troublemakers, miara_thorn_of_the_glade,
  ruthless_technomancer, temur_sabertooth, leaf_crowned_visionary,
  tainted_observer, call_of_the_ring, hazorets_monument
- Counter-tax: mana_leak, make_disappear, spell_pierce, mana_tithe,
  flusterstorm, izzet_charm

## Hazards (from task brief)
1. Verify KW/AbilDef/SOK discriminant chain from current code before adding variants.
2. New mutable runtime fields + new struct fields MUST be added to state/hash.rs
   (PB-AC1 HIGH finding was exactly this — check HashInto for every extended struct).
3. Exhaustive matches in tools/tui/src/play/panels/stack_view.rs AND
   tools/replay-viewer/src/view_model.rs need arms for every new enum variant —
   verify with cargo build --workspace.
4. Do not commit phantom .claude/skills deletions in the worktree.

## Phases
- [x] plan  (primitive-impl-planner → pb-plan-AC2.md)
- [x] implement (primitive-impl-runner, 2026-07-07)
  - Added `Effect::MayPayThenEffect { cost, payer, then }` (CR 118.12) and
    `Effect::CounterUnlessPays { target, cost }` (CR 118.12a) to
    `crates/engine/src/cards/card_definition.rs`.
  - `crates/engine/src/effects/mod.rs`: execution arms for both variants; new
    `try_pay_optional_cost`/`can_pay_optional_cost`/`pay_optional_cost` helper family
    (Mana/PayLife/DiscardCard/Sacrifice/Sequence; Sequence atomic pre-check-then-pay).
    Extracted `Effect::SacrificePermanents`'s inline sacrifice logic into shared
    `eligible_sacrifice_targets` / `sacrifice_permanents_for_player` helpers so the
    optional-cost Sacrifice path reuses the exact same dies-trigger / replacement-effect
    zone-move code (no second sacrifice path).
  - `crates/engine/src/state/hash.rs`: hash arms for discriminants 88 (MayPayThenEffect)
    and 89 (CounterUnlessPays); `HASH_SCHEMA_VERSION` bumped 28→29 + history entry.
    Updated the `HASH_SCHEMA_VERSION, 28u8` sentinel in all 20 existing test files to 29u8.
  - New test file `crates/engine/tests/optional_cost_and_counter_tax.rs` — 15 tests
    (PayLife/DiscardCard/Sacrifice/Mana pay+decline pairs, Sequence atomicity pay/decline,
    CounterUnlessPays decline-counters + flashback-exile regression + noncreature-filter
    target-validation integration, hash schema sentinel + hash-distinguishes-variants).
  - Deviation: the 5 card-integration tests named in the plan (crossway_troublemakers,
    hazorets_monument, springbloom_druid, nadir_kraken, mana_leak) are deferred to the
    backfill phase — card defs are untouched in this implement phase per task scope, and
    those defs still contain TODOs/stubs that would make the tests fail today.
  - Gates: `cargo build --workspace` clean (no TUI/replay-viewer arms needed, confirmed),
    `cargo test --all` 2908 passed / 0 failed, `cargo clippy --all-targets -- -D warnings`
    clean, `cargo fmt --check` clean.
- [x] review (primitive-impl-reviewer → pb-review-AC2.md, 2026-07-07)
  - 0 HIGH, 2 MEDIUM, 2 LOW. Hash/PayLife-doubling/resolution-timing/CounterSpell
    delegation all CR-correct. MEDIUM #1: Cost::Sequence pre-check ignored cumulative
    resource depletion. MEDIUM #4: no real-card integration tests (deferred to backfill).
- [x] fix (primitive-impl-runner, 2026-07-07)
  - MEDIUM #1 FIXED: Sequence payability probe now simulates against a scratch GameState
    clone with cumulative depletion; all-or-nothing preserved. +6 tests (2914 total).
  - LOW #2 FIXED: `then` runs with ctx.controller rebound to the actual payer.
  - LOW #3: note-only, left as-is. MEDIUM #4: deferred to backfill.
- [x] backfill (bulk-card-author + card-batch-reviewer, 2026-07-07)
  - 12 CLEAN cards authored (crossway_troublemakers, miara, hazorets_monument [was
    wrong-state unconditional draw], tainted_observer, springbloom_druid, nadir_kraken,
    mana_leak, mana_tithe, spell_pierce, flusterstorm, make_disappear, izzet_charm);
    8 PARTIAL with precise ENGINE-BLOCKED markers (ezuri [no proliferate-trigger DSL
    variant], stubborn_denial [Ferocious authored], ruthless_technomancer, vampire_gourmand,
    mana_vault, temur_sabertooth [bounce-as-cost out of scope], leaf_crowned_visionary,
    call_of_the_ring).
  - card-batch-reviewer: 0 HIGH / 0 MEDIUM / 1 LOW (make_disappear Casualty reminder text — fixed).
  - MEDIUM #4 closed: crates/engine/tests/pb_ac2_card_integration.rs (5 real-card tests).
  - Authoring-report: clean 934→946 (+12), 53.4%→54.1%.
- [x] close (2026-07-07) — /review PASS on all 4 acceptance criteria; gates green
      (build/clippy/fmt clean, 2919 tests pass).
