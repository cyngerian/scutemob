# Primitive WIP: PB-EF7 — modal `AbilityDefinition::Activated { modes }` (EF-W-PB2-4)

batch: PB-EF7
title: Add `modes: Option<ModeSelection>` + per-mode targets to `AbilityDefinition::Activated` (and the runtime `ActivatedAbility`), mirroring the `Spell`/`Triggered` modal announce/validate/resolve path so "Choose one —" activated abilities resolve the CHOSEN mode (CR 601.2b mode choice at activation; CR 602.2 activation mirrors casting; CR 700.2 modal spells/abilities). The chosen mode rides the stack object (`StackObject.modes_chosen` already exists) and survives LKI.
task: scutemob-108
branch: feat/pb-ef7-modal-abilitydefinitionactivated-ef-w-pb2-4
started: 2026-07-18
phase: implement

## Source findings
- memory/primitives/ef-batch-plan-2026-07-17.md — PB-EF7 section (line ~409); §1 table (EF-W-PB2-4, line 190)
- memory/card-authoring/w-pb2-engine-findings-2026-07-17.md — EF-W-PB2-4 (line 90)

## Corpus sweep (DONE 2026-07-18 by worker — all_cards() enumeration, NOT source grep)
Enumerated every `AbilityDefinition::Activated` whose `effect` serde-tree contains `Effect::Choose`.
Modal-activated cohort = **3**:
| Card | current | modes | per-mode targets | verdict |
| --- | --- | --- | --- | --- |
| Goblin Cratermaker | known_wrong | 2 (deal 2 dmg to target creature / destroy target colorless nonland permanent) | YES — mode0 TargetCreature, mode1 colorless-nonland | **ELIGIBLE → Complete** (also fix colorless filter: `exclude_colors: {W,U,B,R,G}` + `non_land`) |
| Cankerbloom | known_wrong | 3 (destroy target artifact / destroy target enchantment / proliferate) | YES — mode0 TargetArtifact, mode1 TargetEnchantment, mode2 NO target | **ELIGIBLE → Complete** |
| Umezawa's Jitte | known_wrong | 3 (on the "remove a counter" ability) | YES | **INELIGIBLE** — second blocker: the counters trigger fires only on combat-damage-to-PLAYERS; oracle is "deals combat damage" (any recipient) → needs a new trigger variant. Stays known_wrong; update note + file OOS seed. |

**Discounted ship: 2 flips** (Goblin Cratermaker + Cankerbloom). Both need PER-MODE targets, so this PB must mirror PB-AC4's `ModeSelection.mode_targets` machinery onto the activation path, not just simple mode selection.

## Architecture recon (worker — verify + extend during plan)
- `AbilityDefinition::Activated` — `crates/card-types/src/cards/card_definition.rs:285`. Add `modes: Option<ModeSelection>` (`#[serde(default)]`). Per-mode targets ride the EXISTING `ModeSelection.mode_targets: Option<Vec<Vec<TargetRequirement>>>` (card_definition.rs:3749) — no new field there.
- `ModeSelection` — card_definition.rs:3716. Goblin Cratermaker = choose exactly 1 of 2; Cankerbloom = choose exactly 1 of 3.
- Runtime `ActivatedAbility` struct — `crates/card-types/src/state/game_object.rs:352`. Add `modes: Option<ModeSelection>`.
- `enrich_spec_from_def` — `crates/engine/src/testing/replay_harness.rs:2135` (the non-mana Activated loop, ~2136). Propagate `modes`.
- `Command::ActivateAbility` — `crates/engine/src/rules/command.rs:67`. Add `modes_chosen: Vec<usize>` (`#[serde(default)]`). WIRE CHANGE → PROTOCOL bump. Dispatch site: `engine.rs:147`.
- `handle_activate_ability` — `crates/engine/src/rules/abilities.rs:130`. Add: mode validation (CR 700.2a min/max; ascending-sort; dup rule per `allow_duplicate_modes`), and mode_targets-aware target announcement/validation. Study `casting.rs` modal path: `validated_modes_chosen` (~3487-3560), `mode_targets_active`/`spell_targets` split (~3620-3700), and the split helper referenced at casting.rs ~5908.
- StackObject already carries `modes_chosen: Vec<usize>` (stack.rs:413) and `targets`. `StackObjectKind::ActivatedAbility` (stack.rs:584) is UNCHANGED — modes ride the outer StackObject, so TUI/replay-viewer exhaustive `StackObjectKind` matches need no new arm (still run `cargo build --workspace`).
- Resolution — `crates/engine/src/rules/resolution.rs:1841` (`ActivatedAbility` arm). Replace `ability_effect` with the chosen mode effects from `ModeSelection` when `modes_chosen` non-empty, mirroring the Triggered modal path at resolution.rs:2009-2049 (single mode → that effect; multiple → `Effect::Sequence`).
  **KEY HAZARD (SacrificeSelf cost):** Goblin Cratermaker + Cankerbloom both cost `Cost::SacrificeSelf`, so `state.objects.get(source)` is None at resolution and the CardDef must be looked up via a still-available handle. The ActivatedAbility arm already uses `embedded_effect` (captured at activation) for this exact reason (stack.rs:587). Decide: either (a) resolve the chosen modes into a concrete `embedded_effect` AT ACTIVATION and store that, or (b) embed the ModeSelection alongside and read `modes_chosen` at resolution. (a) is simpler and matches how sacrifice-cost activated abilities already capture their effect — planner picks and justifies. Whatever the choice, the `mode_targets` DeclaredTarget indices in the chosen mode effects must be LOCAL to that mode's target slice (as in the Spell path) — verify the target-context threading.
- Hash — `crates/engine/src/state/hash.rs:6617` (`AbilityDefinition::Activated` arm) needs `modes` hashed; `ModeSelection` HashInto already exists (hash.rs:5780).
- Wire bumps: PROTOCOL (Command::ActivateAbility.modes_chosen + Activated.modes reaches the Effect/DSL closure) and HASH (runtime ActivatedAbility.modes reaches GameState). Both machine-forced; read digests from FAILING gate output, never hand-guess. Read current consts from `rules/protocol.rs` (PROTOCOL_VERSION) and `state/hash.rs` (HASH_SCHEMA_VERSION).

## COORDINATOR SCOPING DECISIONS (constraints for planner/runner)
1. Scope = the 2 eligible flips + honest Jitte note (+ OOS seed for Jitte's trigger blocker). Do NOT attempt Jitte's trigger-variant work.
2. Reuse `ModeSelection.mode_targets` — do NOT invent a parallel per-mode-target field on Activated.
3. `Effect::Choose` stays a gated stub (effect_choose_gate). Flipped defs MUST use `modes: Some(ModeSelection)` and MUST NOT retain `Effect::Choose` anywhere (the gate walks the serde tree and will catch it).
4. Decoy discipline: the pinning test MUST fail if the UNCHOSEN mode resolves. A decoy target only mode-1 could legally affect, with mode-0 chosen, must remain untouched after resolution. Also add the reverse (choose mode-1, mode-0's target untouched).
5. Verify `matches_filter` actually honors `exclude_colors` before relying on it for Goblin Cratermaker's colorless filter (CLAUDE.md warns several TargetFilter fields are silently ignored). If NOT honored, that is a secondary in-scope fix OR Goblin Cratermaker stays partial with a truthful note — planner decides + justifies.
6. LKI persistence: mode choice must survive an intervening state change between activation and resolution (add a test where something changes on the board between activation and resolution and the chosen mode still resolves).

## PLAN RESOLUTIONS (2026-07-18)
- **SacrificeSelf approach: (a) — resolve chosen mode into `embedded_effect` at activation.** resolution.rs:1841 is left UNCHANGED (recon correction: the WIP's "replace at resolution mirroring Triggered" is approach (b), NOT chosen). Both cards are choose-exactly-one, so the single chosen mode's `DeclaredTarget` LOCAL indices == global (one slice); `stack_obj.targets` holds that slice, `ctx.targets` at resolution is that slice. Multi-mode+mode_targets is hard-rejected (flag-don't-extend, mirrors casting Escalate+mode_targets reject).
- **exclude_colors IS honored** (effects/mod.rs:8249, cast-time enforced per tests/rules/targeting.rs:966; doom_blade/shriekmaw/snuff_out ship it). Goblin Cratermaker colorless filter = pure def fix, no engine work.
- **Wire bumps: PROTOCOL 11→12, HASH 49→50** (read exact digests from failing protocol_schema/hash_schema gates).
- **Two corpus/test-wide mechanical surfaces flagged**: DSL `modes: None,` on ~600–800 def literals (brace-match script; do NOT sed `once_per_turn` — shared with Triggered); `modes_chosen: vec![],` on ~180 `Command::ActivateAbility` literals. Runtime `ActivatedAbility.modes` is Default-absorbed (low churn).

## Steps (planner fills detail in pb-plan-EF7.md)
plan_file: memory/primitives/pb-plan-EF7.md
plan_complete: true

## Implementation (DONE 2026-07-18, worker scutemob-108)

- [x] Change 1 — DSL: `AbilityDefinition::Activated::modes: Option<ModeSelection>`
      (`crates/card-types/src/cards/card_definition.rs`).
- [x] Change 2 — runtime `ActivatedAbility::modes` (`crates/card-types/src/state/game_object.rs`).
- [x] Change 3 — `Command::ActivateAbility::modes_chosen: Vec<usize>` (`crates/engine/src/rules/command.rs`).
- [x] Change 4 — dispatch thread-through (`crates/engine/src/rules/engine.rs`).
- [x] Change 5 — `enrich_spec_from_def` propagation (`crates/engine/src/testing/replay_harness.rs`).
- [x] Change 6 — `handle_activate_ability` mode validation + per-mode target split + effect
      bake (`crates/engine/src/rules/abilities.rs`) — approach (a), resolution.rs UNCHANGED
      (confirmed: `ActivatedAbility` resolution arm untouched).
- [x] Change 7 — hash arms (DSL + runtime) (`crates/engine/src/state/hash.rs`).
- [x] Change 8 — corpus-wide `modes: None,` on 772 `AbilityDefinition::Activated {}` def
      literals across 499 files (brace-matching Python script; 2 files hand-excluded and
      hand-authored: goblin_cratermaker.rs, cankerbloom.rs). One extra fix:
      `bootleggers_stash.rs` (a raw runtime `ActivatedAbility` literal via `AddActivatedAbility`).
- [x] Change 9 — `modes_chosen: vec![],` on ~200 `Command::ActivateAbility {}` literals
      (test files, `random_bot.rs`, `tui/input.rs`, `replay_harness.rs` translate site).
- [x] Change 10 — `modes: None,` on test/engine-side `AbilityDefinition::Activated` /
      `ActivatedAbility` literals not in card-defs (~53 files via brace-matching script
      with pattern-vs-literal detection + function-return-type exclusion).
- [x] Change 11 — exhaustive matches verified unchanged (`StackObjectKind` in
      `stack_view.rs` / `view_model.rs`) — confirmed via `cargo build --workspace`.
- [x] Wire bumps: PROTOCOL 11→12 (fingerprint `05eaa04b...`), HASH 49→50 (decl
      `3812156d...`, stream `76ebf655...`) — both read from FAILING gate output, history
      rows appended, frozen-prefix digests re-pinned.
- [x] Card fixes: `goblin_cratermaker.rs` + `cankerbloom.rs` → `Completeness::Complete`
      (real `modes: Some(ModeSelection)`, no `Effect::Choose` remaining).
      `umezawas_jitte.rs` note rewritten + **OOS-EF7-1** filed
      (`memory/card-authoring/w-pb2-engine-findings-2026-07-17.md`, EF-W-PB2-4 closed there too).
- [x] Fixed collateral test: `effect_choose_gate::sr33_demoted_cards_carry_truthful_markers`
      (Cankerbloom removed from the "must stay known_wrong" roster).
- [x] Tests: `crates/engine/tests/primitives/pb_ef7_modal_activated.rs` (11 tests, all CR-cited),
      registered in `tests/primitives/main.rs`. Non-vacuity verified by two canary breaks
      (always-bake-mode-0; delete `exclude_colors`) — both reddened the expected tests, reverted.

**Gates**: `cargo build --workspace` clean; `cargo test --all` 3416 passed / 0 failed;
`cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --all -- --check` clean;
`tools/check-defs-fmt.sh` clean.

phase: done (pending coordinator review)
