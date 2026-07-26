# Primitive WIP — PB-DP3 (DP-4 · empty `modes_chosen` bypasses `min_modes`) · PLAN

<!-- last_updated: 2026-07-26 -->

- **PB**: PB-DP3 — a modal spell cast with an empty `modes_chosen` skips every mode
  legality check and silently resolves mode 0 at full price. **CR 601.2b** (modes are
  announced as part of casting, before costs) + **CR 700.2a** (the caster chooses; the
  count must satisfy the printed "choose N").
- **Task**: `scutemob-151`
- **Branch**: `feat/pb-dp3-empty-modeschosen-bypasses-minmodes-modal-spells-reso`
- **Class**: CORRECTNESS (live-wrong on 3 `Complete` cards; Tier 0)
- **Phase**: fix — COMPLETE. Review cycle closed 2026-07-26: **0 HIGH, 2 MEDIUM, 6 LOW**,
  verdict "ship after fixes"; all fixes applied (5 fixed, 1 declined-with-reason as
  seed-text-only, 2 folded into seeds). See `memory/primitives/pb-review-DP3.md` §317 for the
  fix list and the dispositions. Tests 3,725 → **3,747**; PROTOCOL 27 / HASH 63 unmoved.
  Seeds **OOS-DP3-1..8** filed in `docs/audits/decision-point-audit.md` §8.1; audit rows
  §4.1 L186 (D→A), §4.2 L214 (B→A), §5 DP-4, §5 DP-20, §8, §8.1, §9 rec 4 all updated.
- **Binding spec**: `docs/audits/decision-point-audit.md` §4.1 (mode-announcement rows,
  lines 185-186), §4.2 (line 214, activated-ability modal, class B, "same `min_modes`
  bypass as DP-4"), §5 (DP-4 row, line ~431), §8 (PB-DP3 row, line ~572), §9
  recommendation 4 (line ~702, the M11-local play-server consequence)
- **Plan file**: `memory/primitives/pb-plan-DP3.md`
- **Review file**: `memory/primitives/pb-review-DP3.md`

## The defect (as filed by the audit)

`crates/engine/src/rules/casting.rs:3507` gates **all** mode validation behind
`if !modes_chosen.is_empty() && !entwine_paid`. Inside that branch, range (`:3516-3524`),
duplicates (`:3526-3536`), `min_modes` (`:3539-3544`) and `max_modes` (`:3545-3550`) are
all checked correctly. The `else` arm (`:3556-3560`) passes the empty vector straight
through as "non-modal spell or auto-select mode[0] (backward compatible)".

Downstream, both consumers re-derive `vec![0]` from the empty vector:

- cast-time per-mode target slicing — `casting.rs:3646-3654`
- resolution — `resolution.rs:335-341`

So `Command::CastSpell { modes_chosen: vec![], .. }` on **Cryptic Command**
(`min_modes: 2, max_modes: 2`), **Austere Command** or **Incendiary Command** — all three
`Complete` — pays the full mana cost and resolves exactly one mode.

The Spree path already hard-rejects an empty `modes_chosen`
(`casting.rs:2941-2945`, CR 702.172a); the general modal path has no equivalent.

## Known adjacent facts (coordinator survey, pre-plan)

- 41 card defs carry `min_modes`: **37** `min_modes: 1`, **3** `min_modes: 2` (the three
  commands), **1** `min_modes: 0` (`hullbreaker_horror` — a modal *triggered* ability,
  "choose up to one", so empty is legal there and must stay legal).
- Two backward-compat paths deliberately send an empty `modes_chosen` and must not
  regress: **entwine** (`entwine_paid` ⇒ all modes) and **escalate**
  (`resolution.rs:321-334` derives `0..=escalate_modes_paid`).
- Golden scripts: `test-data/generated-scripts/stack/169_modal_choice_abzan_charm.json`
  is the only script naming a modal card; the harness surfaces `modes_chosen` only on
  the `cast_spell_modal` action (`replay_harness.rs:745`) — every other cast action
  hard-codes `vec![]`.
- Tests constructing `ModeSelection` directly: `rules/modal.rs`, `rules/modal_triggers.rs`,
  `primitives/pb_ac4_card_integration.rs`, `primitives/pb_ac4_per_mode_targeting.rs`,
  `primitives/pb_ef7_modal_activated.rs`, `primitives/pb_os1_gain_control_reversion.rs`,
  `mechanics_e_l/entwine.rs`, `mechanics_e_l/escalate.rs`, `mechanics_m_z/spree.rs`.
- **No wire change expected** — PROTOCOL 27 / HASH 63 must be unmoved. Stop and re-scope
  if that is contradicted.

## Implementation complete (runner close-out)

All five engine changes (§3), all six blast-radius edits (§4.2-§4.6), and all 20 unit
tests (§7: 7 fail-before/pass-after probes + 9 positive regression guards in
`crates/engine/tests/primitives/pb_dp3_modal_mode_announcement.rs`, + 4 simulator tests
in `crates/simulator/src/legal_actions.rs`) are done, per the plan exactly:

- **Change 1** (`casting.rs`): the emptiness gate is lifted into a 3-way match on
  `(entwine_paid, mode_selection_opt, modes_chosen.is_empty())`, with the escalate
  count-derivation exemption and the `min_modes == 0` fail-safe reject, as specified.
- **Change 2** (`casting.rs` `mode_targets_active`): comment-only reconciliation; the
  `vec![0]` arm is retained as a documented fail-safe.
- **Change 3** (`resolution.rs:335-341`): comment-only; the `vec![0]` fallback is
  **retained** (six free-cast producers depend on it — cascade/discover/4×engine.rs).
- **Change 4** (`abilities.rs`): the modal-activated lift, with the `min_modes == 0`
  legal-no-op branch (representable here, unlike the Spell side).
- **Change 5** (Spree guard, `casting.rs:2938-2945`): verified unchanged, still fires
  first and owns its own CR 702.172a message.
- **§4.2-4.6**: 3 engine test edits, 2 golden scripts (147, 148) + `cr_sections_tested`
  additions, 1 replay-harness line (`cast_spell` now honours `modes`), 2 new simulator
  helpers (`spell_default_modes` / `ability_default_modes`) wired into 4 `random_bot.rs`
  call sites + 2 `tools/tui/src/play/input.rs` call sites.
- **Un-enumerated finding (§4.7)**: `crates/engine/tests/core/ability_definition_registry.rs`
  (SR-15 gate) failed after the simulator edit — `legal_actions.rs::spell_default_modes`
  is a new real dispatch site on `AbilityDefinition::Spell` that the plan's §4 blast-radius
  table did not enumerate. Fixed by adding the site to the `Spell` declaration in
  `crates/engine/src/state/ability_definition_registry.rs` (the registry's own documented
  purpose is to force exactly this kind of edit — treated as a required companion edit,
  not a silent patch). Flagging here per the plan's §4.7 negative-space clause for the
  reviewer's visibility.
- **Test count**: 3,725 (pin) → **3,745** (+16 engine `pb_dp3_modal_mode_announcement.rs`
  + 4 simulator `legal_actions.rs`), all green. `cargo build --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`,
  `tools/check-defs-fmt.sh` all clean. **PROTOCOL 27 / HASH 63 unmoved**, confirmed by
  reading the constants directly, matching the plan's prediction exactly.
- **7 fail-before probes**: each was verified failing against the pre-fix engine by
  temporarily reverting Change 1 (`casting.rs`) and/or Change 4 (`abilities.rs`) and
  re-running the affected test; see the task/session report for the observed pre-fix
  behaviour table.
