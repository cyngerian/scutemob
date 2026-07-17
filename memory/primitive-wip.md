# Primitive WIP: SR-34 — Mana abilities with additional costs are never registered

batch: SR-34
title: Composite-cost mana abilities (CR 605.1a) — `enrich_spec_from_def` only lowers bare `Cost::Tap`
task: scutemob-90
branch: feat/sr-34-mana-abilities-with-additional-costs-are-never-registe
cards_affected: 39 defs w/ non-`Cost::Tap` mana-producing abilities (23 `Complete`) + 10 bare-`Cost::Tap`
  defs that register nothing. Roster: `memory/primitives/sr34-affected-defs.md` (enumerated from
  `all_cards()`). `ManaAbility { mana_cost, life_cost }` fixes 26 of 39.
started: 2026-07-17
phase: implement
plan_file: memory/primitives/pb-plan-sr34.md
roster_file: memory/primitives/sr34-affected-defs.md

## Plan headlines (2026-07-17) — read these before touching code

- **Finding A — the naive widening BREAKS Cabal Coffers.** `try_as_tap_mana_ability`'s
  `AddManaScaled` arm registers `produces = {color: 1}` and calls it "a marker; actual production
  is dynamic". Nothing makes it dynamic — the dynamic evaluation is only reachable via stack
  resolution. Cabal Coffers is correct *today only because the `Cost::Tap` gate excludes it*.
  `AddManaScaled` must be **actively excluded** from the widened path, with a test pinning it.
- **CR 605.1a has three criteria and the lowering loop only checks one.** It drops `targets` on
  the floor; it avoids making Deathrite Shaman a mana ability by luck. Widening without adding
  `targets.is_empty()` silently violates CR 605.1a.
- **CR 118.3, not 118.4**, is "can't pay what you can't pay". CR 119.4b makes a 0-life payment
  always legal, so the life check must short-circuit on `life_cost > 0`; the boundary is `>=`.
- All three version constants move: `PROTOCOL_VERSION` 2→3, `HASH_SCHEMA_VERSION` 40→41
  (30 sentinels across 29 files), and both fingerprints.
- New findings to FILE, not fix: **SF-8** (`Cost::Tap` + `AddManaScaled` → Gaea's Cradle taps for
  exactly 1 green today, HIGH), **SF-9** (`Cost::PayLife` silently unpaid for non-mana activated
  abilities), **SF-10** (`ManaAbility` has no `activation_condition` — Tainted Field taps for `{W}`
  with no Swamp).

## Origin

SF-1 in `memory/card-authoring/sr33-engine-findings-2026-07-17.md` (empirically proven).
CR 605.1a makes an ability a mana ability based on **what it does**, not what it costs.
`enrich_spec_from_def` (`crates/engine/src/testing/replay_harness.rs:2117`) gates on
`matches!(cost, Cost::Tap)`, so every mana source with an additional cost is treated as a
stack-using activated ability: it cannot be found by `TapForMana` and cannot be activated
while casting a spell (CR 605.3b) — which is what a Signet is *for*.

## Known affected shapes (from SF-1; must be re-derived from the registry)

| Shape | Cards |
|---|---|
| `{1}, {T}: Add {C}{C}` | Signets, Cluestones, Viridescent Bog, Darkwater Catacombs, Magnifying Glass |
| `{T}, Pay 1 life: Add {B} or {G}` | horizon lands (Fiery Islet, Nurturing Peatland, Silent Clearing) |
| `{W/B}, {T}: Add …` | filter lands (accepted-limitation note in `tests/casting/mana_filter.rs` — that note understates the gap) |
| `{2}, {T}: Add {B} for each Swamp` | Cabal Coffers, Cabal Stronghold |

## Scope notes

- `ManaAbility` (`crates/card-types/src/state/game_object.rs:165`) has no field for an
  additional cost — only `requires_tap`, `sacrifice_self`, `any_color`,
  `damage_to_controller`. `handle_tap_for_mana` (`crates/engine/src/rules/mana.rs:29`) has
  no cost-payment step at all. Both need one.
- The three horizon lands are blocked on SF-1 **and** SF-3 (`Effect::AddManaChoice` is a stub
  that adds one `{C}` and ignores `count`). SR-33 demoted them to `known_wrong`; this task
  un-demotes them. The SR-33 fix shape applies: one activated ability per printed colour
  (`tainted_field` pattern), which makes SF-3 unnecessary for them.
- `Effect::AddManaChoice` is gated out of `Complete` by `tests/core/effect_choose_gate.rs`
  — do not reintroduce it.
- SF-6: `enrich_spec_from_def` excludes tap-mana abilities from `activated_abilities` so
  `ability_index` does not shift. Widening the mana-ability gate **moves abilities out of
  `activated_abilities`**, so indices shift again on affected defs. That exclusion list
  (`is_tap_mana_ability`, `replay_harness.rs:2141`) must be widened in lockstep, and any
  test/script referencing an affected card by ability index must be re-checked.
- SF-7: `cargo fmt` reaches **zero** card defs (they live behind `include!`/`#[path]` from
  `OUT_DIR`). Run `rustfmt` over touched def files by name, and note that rustfmt exits 0
  while silently abandoning a macro body that exceeds `max_width`.

## Implementation complete (2026-07-17, scutemob-90)

Plan §3 steps 1–7, §6 tests, and §9's card-def work (horizon lands + `mana_filter.rs` +
`effect_choose_gate.rs`) are DONE. §3 step 8's broader roster reconciliation was
explicitly out of this agent's scope (per its brief) — see the "Roster items not
reconciled" section of the findings doc below for what that leaves open.

- `ManaAbility` gained `mana_cost: Option<ManaCost>` / `life_cost: u32`
  (`crates/card-types/src/state/game_object.rs`), hashed in `state/hash.rs`.
- `handle_tap_for_mana` (`rules/mana.rs`) gained cost-legality (step 5b) and payment
  (step 6b) steps; new `GameStateError::InsufficientLife`.
- `mana_ability_lowering` (`testing/replay_harness.rs`) is the single predicate for both
  mana-ability registration and the `activated_abilities` exclusion; widened from bare
  `Cost::Tap` to any cost payable through `Command::TapForMana`; actively excludes
  `Effect::AddManaScaled` from every cost but bare `Cost::Tap` (Finding A).
  `targets.is_empty()` gate protects Deathrite Shaman.
- Three horizon lands (Fiery Islet, Nurturing Peatland, Silent Clearing) rewritten to
  the `tainted_field.rs` one-ability-per-colour pattern and un-demoted to `Complete`.
- `tests/casting/mana_filter.rs` rewritten (filter lands now activate via `TapForMana`,
  hybrid-cost non-enforcement documented explicitly, not silently).
- `tests/core/effect_choose_gate.rs`'s `printed_tap_mana_colors` widened to cover
  composite-cost tap-mana clauses (Signets, horizon lands, filter lands), with the
  `AddManaScaled`/dynamic-amount blind spot and the sacrifice-another exclusion both
  documented in the function's doc comment.
- New test file `crates/engine/tests/primitives/primitive_sr34_composite_mana_costs.rs`
  (14 tests, T1–T13 from the plan's §6, T12 split into two).
- Version bumps done last, in dedicated commits: `PROTOCOL_VERSION` 2→3,
  `HASH_SCHEMA_VERSION` 40→41, both fingerprints, both `FROZEN_HISTORY_PREFIX_DIGEST`s
  re-pinned, 30 sentinels across 29 files updated.
- SF-6 sweep (3 passes, mechanical probe deleted before commit): only Magnifying Glass
  (script 099 fixed) and Staff of Compleation (unreferenced, no action) actually shift
  indices among the roster's `Complete` defs.
- SF-8, SF-9, SF-10 filed, not fixed:
  `memory/card-authoring/sr34-engine-findings-2026-07-17.md`.

All gates green: `cargo build --workspace`, `cargo test --all` (3298 passing, up from
3284 baseline), `cargo clippy --all-targets -- -D warnings` (0 warnings),
`cargo fmt --check` + `rustfmt` over every touched def by name,
`tests/scripts/run_all_scripts.rs` (210/210 approved scripts pass).
