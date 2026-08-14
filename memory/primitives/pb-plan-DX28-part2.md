# PB-DX28 part 2 — the untargeted-choice channel: consumer surface and derivation

Companion to `pb-plan-DX28.md` §1. That section states the design; this one states the surfaces a
new `EffectChoiceQuestion` variant has to reach, measured rather than remembered, so the
implementing run does not discover them one compile error at a time.

## 1. Candidate derivation — the one thing that must NOT be reused

`rules::queries::legal_targets_per_slot` derives candidates by delegating to
`casting::validate_targets_inner`, i.e. **full CR 115 targeting legality** — hexproof, shroud,
protection, "can't be the target of". That is precisely the defect `OOS-DX4-6` names, so the
untargeted channel must NOT route through it, and must not route through
`validate_object_satisfies_requirement` either.

Instead: a dedicated `filter_matches_object_untargeted(state, id, filter, chooser, self_id)` that
applies

* `rules::layers::expect_characteristics(state, id)` → `matches_filter(&chars, filter)` — the
  characteristic axes;
* then, explicitly, every runtime axis `matches_filter` cannot see, each of which is documented at
  its own declaration in `card_definition.rs` as "silently ignored by `matches_filter()`":
  `controller`, the new `owner` (part 1), `exclude_self`, `is_token` / `is_nontoken`,
  `is_tapped` / `is_untapped`, `is_attacking` / `is_blocking`, `has_counter_type`,
  `has_chosen_subtype` / `exclude_chosen_subtype`;
* and **fails closed on the axes it does not implement**: `max_cmc_amount` / `min_cmc_amount` are
  `EffectContext`-resolved and are not supported here, so a filter setting one must be rejected by
  the roster gate rather than silently ignored.

The corpus needs only `has_card_type`, `has_card_types` and `controller` today. Implement the full
list anyway — a filter axis that is silently ignored is the `is_token` failure mode this file's
own citations keep naming — and pin the supported set in the roster test.

## 2. Consumer surface, measured

| surface | file | what it needs |
|---|---|---|
| answer defaults | `crates/engine/src/effects/mod.rs` `default_effect_choice_answer` (+ a `default_choose_object_answer` sibling if the four existing ones each have one) | first `min(count, candidates.len())` candidates — the deterministic recovery of the pre-batch auto-pick |
| answer legality | `effects/mod.rs::handle_answer_effect_choice`'s `(question, answer)` match | no duplicates, subset of `candidates`, `== count` when `!up_to` (clamped by CR 608.2 to `min(count, candidates.len())`), `<= count` when `up_to` |
| mana-ability gate | `crates/engine/tests/primitives/pb_dp9_effect_choice.rs::test_dp9_mana_ability_gate` | asserts no `Complete` def puts one of the **four** asking effects inside a mana ability. There are five channels now — extend it, or the gate silently stops covering the new one |
| replay harness | `crates/engine/src/testing/replay_harness.rs:1118-1160` | one arm per question variant; a missing arm is a silently unanswerable script |
| simulator coverage | `crates/simulator/src/decision_coverage.rs:269-272` | the question→row-name map, and `decision_site_walk.rs`'s `ROWS` if a new row is warranted |
| simulator legal actions | `crates/simulator/src/legal_actions.rs:391` | `LegalAction::AnswerEffectChoice { question, answer }` is generic — verify, do not assume |
| blocking decision | `crates/engine/src/rules/engine.rs:161/199/253` | `BlockingDecision::EffectChoice` is generic over the question — verify |
| play-server view | `tools/play-server/src/view.rs:2170` (`AnswerShapeView::PickN`) | `PickN` is exact-`count`. `up_to` needs a `min_count` field on that DTO (play-server-local, **not** a wire change). Existing uses set `min_count == count` |
| play-server api | `tools/play-server/src/api.rs:530-570` (answer round-trip) and `:912` (`question_kind`) | one arm each |
| frontend | `tools/play-server/frontend/src/lib/DiscardPicker.svelte` | honour `min_count` so a player may submit fewer than `count` for an "up to" choice; the Confirm button's enable predicate is the only thing that changes |

Every one of those line numbers is a **2026-08-14 measurement**, not a promise: re-grep before
editing (`OOS-DX6-5` is the standing seed for exactly this drift).

## 3. Gates this batch owes

* `crates/engine/tests/core/pb_dx28_chosen_object_roster.rs`
  * **R1** — the exact set of corpus defs naming `EffectTarget::ChosenObject`, pinned by name
    (17 after migration). A pin, so an 18th use is a deliberate act.
  * **R2** — every `ChosenObject` filter in the corpus sets only axes
    `filter_matches_object_untargeted` implements. Non-vacuity floor: assert the set is non-empty
    and that at least one member sets `controller`.
  * **R3** — no migrated def declares a `TargetRequirement` for the ability that now uses
    `ChosenObject` (i.e. the migration is complete, not additive) — with `rewind` as the stated
    exception, since its slot 0 `TargetSpell` is a real printed target and stays.
  * **R4** — inverse axis: no `Complete` def still pairs `slots > "target"-word-count` outside a
    named allowlist of the refuted rows in `pb-plan-DX28.md` §0.1. This is the census, frozen, so
    the class cannot silently regrow.
* `crates/engine/tests/primitives/pb_dx28_untargeted_choice.rs` — the behavioural probes in
  `pb-plan-DX28.md` §1.6, each revert-proven red.
