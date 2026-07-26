# Primitive Batch Review: PB-DP3 — Mode announcement is mandatory (DP-4)

**Date**: 2026-07-26
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 601.2 / 601.2b / 601.2f / 601.2h / 601.2i, 602.2b, 700.2, 700.2a, 700.2b, 700.2d,
702.42a/b (entwine), 702.120a (escalate), 702.172a (spree), 613.1f
**Engine files reviewed**: `crates/engine/src/rules/casting.rs`,
`crates/engine/src/rules/abilities.rs`, `crates/engine/src/rules/resolution.rs`,
`crates/engine/src/state/ability_definition_registry.rs`,
`crates/engine/src/testing/replay_harness.rs`
**Non-engine files reviewed**: `crates/simulator/src/legal_actions.rs`,
`crates/simulator/src/random_bot.rs`, `crates/simulator/src/heuristic_bot.rs`,
`tools/tui/src/play/input.rs`
**Tests reviewed**: `crates/engine/tests/primitives/pb_dp3_modal_mode_announcement.rs` (16),
`crates/simulator/src/legal_actions.rs` `mod tests` (4), `tests/rules/modal.rs`,
`tests/mechanics_e_l/{entwine,escalate}.rs`, `tests/primitives/main.rs`
**Golden scripts reviewed**: `stack/147_entwine_promise_of_power.json`,
`stack/148_escalate_blessed_alliance.json` (+ negative check across the whole corpus)
**Card defs reviewed**: 0 modified (correct — this is an engine fix). Oracle-verified the 3
live-wrong `Complete` cards and the corpus `min_modes` census (41 defs).

## Verdict: ship after fixes

The core engine change is **CR-correct and well-executed**. I re-derived CR 601.2b, 700.2a-d and
702.120a from the MCP independently: CR 700.2a says the controller "chooses the mode(s) **as part
of** casting that spell or activating that ability," CR 601.2b places that announcement in step
601.2b — before total cost is locked in (601.2f) and before payment (601.2h) — and **nothing in
CR 700.2 or 601.2 supplies a default or engine-side mode choice.** The lift (validate whenever the
object is modal, rather than only when a non-empty vector arrives) is the right structural fix
rather than a bolt-on, and it is positioned so no cost is spent: `process_command` takes
`GameState` by value (`rules/engine.rs:67-70`) and returns `Err` by dropping it, and the mana
payment site (`casting.rs:~4005`) / activation cost payment (`abilities.rs`, after `:408`) both
follow the guard. **No HIGH findings.** Zero card defs needed edits and none were made; the three
`Complete` commands' oracle text, mode order and `min_modes: 2` all check out against Scryfall.
Wire neutrality is real — nothing in the diff touches the `Command`/`GameEvent`/`GameState` type
closure, so PROTOCOL 27 / HASH 63 correctly did not move.

Two MEDIUM findings block a clean ship: (1) the `resolution.rs:335-351` retained-fallback comment
— the single most load-bearing piece of documentation this PB produces, and the text that seed
OOS-DP3-3 will hand to whoever fixes DP-20 — **names four sites that are not Spell producers at
all and misses two that are** (suspend and cipher free-casts); (2) the two brand-new escalate
derived-count rejection branches have **zero test coverage**. Five LOW findings follow. Both the
escalate exemption and the `abilities.rs` in-scope expansion are **explicitly upheld** below.

---

## Rulings the plan asked for

### Ruling 1 — the escalate exemption (plan §3.4, §11 risk 1): **UPHELD**

The plan's reading ("the player elected to pay the escalate additional cost, therefore the *count*
is announced") is a **weak but acceptable** reading of CR 702.120a. Strictly, 702.120a defines the
cost as a *consequence* of the mode choice ("for each mode you choose beyond the first…"), not as
an announcement of it; CR 601.2b still requires the identities. So escalate casts remain
CR-incomplete after PB-DP3. I uphold the exemption anyway, for reasons the plan did not fully
state:

- **No `Complete` card is live-wrong through this path.** I checked both escalate defs directly:
  `blessed_alliance.rs:102` and `collective_resistance.rs:99` are **both `Completeness::partial`**,
  so `validate_deck` (SR-2) refuses them in a real game. The residual agency loss is unreachable
  from legal play. That is the fact that makes strict rejection deferrable, and it is stronger
  than the plan's "it costs 9 test edits" argument.
- The derived-count bounds check is genuine, not theatre: `casting.rs:3538`
  `((escalate_modes as usize) + 1).min(ms.modes.len())` is **character-for-character identical**
  to resolution's derivation at `resolution.rs:332-333`, and both are gated identically
  (`escalate_modes > 0` / `.filter(|&c| c > 0)`, both only when `modes_chosen` is empty).
  **Cast-time validation and resolution-time derivation cannot disagree** for a card whose
  `ModeSelection` is reached identically at both sites. The one way they *can* disagree is the
  face-awareness divergence in Finding 5 (`casting.rs` reads `def.abilities`; `resolution.rs:246`
  reads `adventure_face` when applicable) — latent, no corpus card.
- The exemption's boundary (`escalate_modes == 0` is not exempt) is pinned by a dedicated test
  (`test_702_120a_escalate_count_zero_requires_explicit_mode`) and is what forced the single
  `escalate.rs:244` edit. That is the right boundary.

**Do not reverse it in a fix cycle.** OOS-DP3-1 correctly owns the residual; see also the new
OOS-DP3-6 (count over-payment is clamped, not rejected), which strengthens the case that escalate
needs its own PB rather than a patch here.

### Ruling 2 — the `min_modes == 0` Spell hard-reject: **UPHELD, claims verified**

- **Unrepresentability verified by reading, not taken on trust.** `StackObject.modes_chosen` is
  `Vec<usize>` (`crates/card-types/src/state/stack.rs:413`), not `Option<Vec<usize>>`; there is no
  `modes_announced` flag. `resolution.rs:310-357` reaches `vec![0]` for any Spell stack object with
  `spell_modes: Some`, empty `modes_chosen`, no entwine and no escalate — **and it cannot tell
  "controller announced zero" from a free-cast that never announced anything.** Confirmed: a
  discriminator would be a `StackObject` field ⇒ HASH bump. Correctly deferred.
- **Corpus claim verified independently.** `grep -rn "min_modes:" crates/card-defs/src/defs/` →
  44 hits, of which 3 are comments (`hullbreaker_horror.rs:8`, `:33`, `akromas_will.rs:27`) ⇒ 41
  real fields: 37 × `min_modes: 1`, 3 × `min_modes: 2` (`cryptic_command`, `austere_command`,
  `incendiary_command`), 1 × `min_modes: 0`. I opened the last one:
  `hullbreaker_horror.rs:35-59` is `AbilityDefinition::Triggered`, not `Spell` and not `Activated`.
  **There is no `min_modes: 0` modal Spell or Activated ability in the corpus.** Claim holds.
- **The Spell-vs-Activated asymmetry is documented in code, not only in the plan**, as required:
  `casting.rs:3552-3567` (Spell reject, names OOS-DP3-2 and the two cascade/discover producers) and
  `abilities.rs:390-397` (Activated accept-and-resolve-nothing, explicitly contrasted with the
  Spell path). Both comments are legible to a future reader who never sees the plan. Requirement
  met. See LOW Finding 6 for one over-claim inside the `abilities.rs` comment.

### Ruling 3 — `abilities.rs` Change 4, the in-scope expansion: **UPHELD, keep it in PB-DP3**

Verified rather than accepted:
- **Same defect, not an adjacent one.** Audit §4.2 line 214 reads verbatim
  `| Modes | 700.2a | **B** | rules/abilities.rs:386-397 — empty ⇒ vec![0]; same min_modes bypass
  as **DP-4** |`. The lift is what turns that row into class A.
- **CR-correct.** CR 700.2a covers "modal spell **or activated ability**" in one sentence; CR
  602.2b governs the activation announcement. The message cites both.
- **Costs are not spent before the rejection.** I read `abilities.rs:150-408`: everything before
  the guard is read-only validation (zone/controller `:180-229`, sorcery-speed `:241-257`,
  activation condition `:259-295`, once-per-turn `:297-308`, layer-resolved ability capture
  `:313-332`). Mana, tap, sacrifice and discard payment all follow. Probe 6
  (`test_602_2b_modal_activated_ability_empty_modes_rejected`) asserts the Cratermaker is still on
  the battlefield, i.e. `SacrificeSelf` was not paid.
- **Blast radius is genuinely zero.** `grep '"modes"' test-data/generated-scripts/` returns 5 hits
  and **none is on an `activate_ability` action**; every in-repo modal activation
  (`pb_ef7_modal_activated.rs`, `pb_os10_singleton_cleanup.rs`) already passes explicit modes. The
  suite needed no edits from this half. Splitting it out would have shipped half a fix and filed
  the other half as a seed against a file already open. **No finding.**
- **Index namespace is correct** — `ability_modes` comes from
  `expect_characteristics(state, source).activated_abilities[ability_index]` (`:313-331`), the
  same layer-resolved list `LegalAction::ActivateAbility` indexes. This is the PB-RS4 bug class and
  it was avoided on both the engine and the simulator side.

### Ruling 4 — the un-enumerated SR-15 registry edit: **CORRECT USE OF THE GATE, not silencing**

`crates/engine/src/state/ability_definition_registry.rs:110-119` adds
`crates/simulator/src/legal_actions.rs` to the `Spell` variant's `sites`, with a comment naming
the new dispatch. The module doc (`:22-30`) states that `Handled` sites "must equal the set of
scanned source files that mention `AbilityDefinition::<Variant>` … **adding a read in a new file
fails the test**", and the test's `SCAN_ROOTS`
(`crates/engine/tests/core/ability_definition_registry.rs:30-34`) explicitly includes
`crates/simulator/src` with an SR-20 rationale, with six variants already declaring that exact
path (`LoyaltyAbility`, `Bloodrush`, `MutateCost`, `Morph`, `Megamorph`, `Disguise`). The edit is
set-equality maintenance, which is what the gate exists to force. **Declaring the site is the only
way to keep the suite green without deleting a real dispatch — this is the intended use.** No
finding. Good call flagging it in the WIP file rather than patching it silently (plan §4.7).

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `rules/resolution.rs:340-350` | **The retained-fallback comment names four producers that cannot reach it and misses two that can.** `engine.rs:2112/2176/2686/2853` build **RingAbility / RoomAbility / LoyaltyAbility / ClassLevelAbility** stack objects, not `StackObjectKind::Spell`; meanwhile `resolution.rs:5167-5173` (cipher copy) and `resolution.rs:5837-5844` (suspend free-cast) build **real Spell** objects via `StackObject::trigger_default` (`stack.rs:555` ⇒ `modes_chosen: vec![]`) and do reach it. **Fix:** rewrite the comment (and seed OOS-DP3-3) to the four true producers — `copy.rs:386` cascade, `copy.rs:614` discover, `resolution.rs:5167` cipher, `resolution.rs:5837` suspend. |
| 2 | MEDIUM | `rules/casting.rs:3539-3550` | **The two new escalate derived-count rejection branches have zero test coverage.** No test in the batch (or the pre-existing escalate suite) drives `derived < min_modes` or `derived > max_modes`. **Fix:** add one probe on a synthetic escalate card with `max_modes` below `modes.len()` (e.g. 3 modes, `max_modes: 2`, `count: 2` ⇒ `derived == 3`), asserting `Err` with `"at most"` + `"702.120a"`. |
| 3 | LOW | `rules/casting.rs:2936-2937` | **Stale line reference in an adjacent pre-existing comment**: "Validation happens below at line ~2874" — the block now lives at `:3510-3620`. Pre-existing, but the PB moved the referent. **Fix:** change to "below, at the `validated_modes_chosen` match". |
| 4 | LOW | `rules/casting.rs:3510-3516` | **The `entwine_paid` arm is now the only wholly unvalidated arm.** With entwine paid, a caller may pass `modes_chosen: vec![99]`; it is stored on the `StackObject` unsorted and out of range. Harmless today (resolution ignores it under entwine, `:313-316`), and the plan's risk 8 correctly says don't touch it. **Fix:** none in this PB — add a one-line comment noting the deliberate non-validation, or file as OOS-DP3-8. |
| 5 | LOW | `rules/casting.rs:3495-3506` | **The guard's `ModeSelection` lookup is not face-aware and ignores the aftermath/adventure half** — this is the plan's own OOS-DP3-5, and it has a second sub-case the seed does not state: when `casting_with_aftermath` is true the guard still reads the **front** half's `ModeSelection` (the `requirements` lookup 120 lines below *does* branch on `casting_with_aftermath`, `:3630`), so an aftermath cast of a card with a modal front half would now be **rejected outright**. Latent: I intersected the 41 `min_modes` files with the 6 files carrying `adventure_face: Some` / `AbilityDefinition::Aftermath` — **empty intersection**. **Fix:** extend OOS-DP3-5's text with the aftermath sub-case; no code change. |
| 6 | LOW | `rules/abilities.rs:391-396` | **Over-claim in the `min_modes: 0` comment.** "leaves `embedded_effect` as the ability's own base effect, which is the correct 'no mode chosen' behaviour" is only correct when the base `effect` is `Effect::Nothing`. A modal activated ability authored with a non-trivial base effect would execute it on a zero-mode activation. The synthetic test uses `Effect::Nothing` (`pb_dp3_…:411`), so the general case is untested. **Fix:** qualify the comment ("correct because a modal ability's base `effect` is `Effect::Nothing` by authoring convention"), or `debug_assert!(matches!(ab.effect, None \| Some(Effect::Nothing)))` in that branch. |
| 7 | LOW | `src/testing/replay_harness.rs:498` vs `:543`, `:566`, `:589`, … (~28 arms) | **Harness cast-action parity is now asymmetric.** `cast_spell` honours `modes`; `cast_spell_modal`/`_entwine`/`_escalate` already did; the ~28 alt-cost arms (`cast_spell_flashback`, `_evoke`, `_bestow`, `_miracle`, `_escape`, `_foretell`, `_plot`, `_warp`, `_pitch`, `_overload`, `_retrace`, `_jump_start`, `_aftermath`, prototype, `_dash`, `_blitz`, `_impending`, `_emerge`, `_spectacle`, `_surge`, `_cleave`, `_mutate`, morph, commander-free-cast, …) still hard-code `modes_chosen: vec![]`. Post-PB-DP3 that is no longer "discard the field" — it is "**this action can never cast a modal card again**". No corpus card is both modal and alt-cost-castable, so it is latent. **Fix:** file as a seed (OOS-DP3-7) and add a one-line note at `:491-498` that the other arms are knowingly mode-blind. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| — | — | — | **None.** Zero card defs were modified, which is correct: the 41 modal defs already declare the constraints the engine was failing to enforce. Oracle-verified `cryptic_command` (Choose two — counter / return / tap all opponents' creatures / draw), `austere_command` (Choose two — artifacts / enchantments / MV ≤3 / MV ≥4, def `austere_command.rs:31-66` matches printed order exactly, `Complete`), `incendiary_command` (Choose two — 4 dmg player/PW / 2 dmg each creature / destroy nonbasic land / wheel). The mode **indices** the positive tests use ([2,3], [0,1], [1,3]) map to the correct printed modes in every case. |

## Test Quality Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 8 | LOW | `pb_dp3_modal_mode_announcement.rs:652-657`, `:687-692`, `:723-728`, `:799-809` | **The "no cost was paid" assertions are vacuous.** Each probe does `cast_modal(state.clone(), …)` and then asserts against the *original, never-passed* `state`. The assertion cannot fail regardless of engine behaviour. The property is nonetheless **structurally true** (`process_command` consumes `GameState` and drops it on `Err`), which is why this is LOW and not MEDIUM. **Fix:** replace with a meaningful sequel — after the rejection, cast the same object again on the same `state` with explicit legal modes and assert it succeeds with the full mana pool available; or drop the assertions and cite the ownership guarantee in the doc comment instead of pretending to test it. |

**Otherwise the test suite is strong** and clears every bar in the brief:
- Pass-after assertions check **real game state**, not "no error":
  `test_700_2a_cryptic_command_modes_2_and_3_both_resolve` asserts `opp_creature.status.tapped`
  **and** hand-count restoration; `…austere_command_modes_0_and_1…` asserts artifact in
  `Graveyard(p1)`, enchantment in `Graveyard(p2)`, **and** the creature surviving (proving modes
  2/3 did not fire); `…incendiary_command_modes_1_and_3…` asserts the 2/2 in the graveyard and
  both players' hands wheeled to 2. That genuinely satisfies **ESM criterion 5524** — two modes,
  two independent board consequences, on three real `Complete` cards.
- Rejection tests assert the **CR-citing message**, not merely `is_err()`: every one checks both
  the `"at least N mode"` substring and the literal `"601.2b"` / `"602.2b"` / `"OOS-DP3-2"`.
  A generic error would not satisfy them.
- Every test carries a CR citation in its doc comment (Architecture Invariant 8), and the module
  header enumerates the covered rules.
- SR-9a honoured: `mod pb_dp3_modal_mode_announcement;` is registered at
  `tests/primitives/main.rs:23`, alphabetically after `pb_dp1_actor_priority`. No new top-level
  `tests/*.rs`.
- The simulator tests are discriminating, not tautological:
  `test_dp3_ability_default_modes_uses_layer_resolved_index` uses Umezawa's Jitte, whose
  `def.abilities[0]` is **not** the activated ability — a `def.abilities`-based implementation
  would return `[]` and fail. That is a real proof of the layer-resolved read.

## Finding Details

### Finding 1: The retained-fallback comment names the wrong producers

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:340-350` (and plan §9 seed OOS-DP3-3, and
`memory/primitive-wip.md:73`)
**Issue**: The plan's central justification for keeping the `vec![0]` arm is "six free-cast
producers build a `StackObject { modes_chosen: vec![] }` without calling `handle_cast_spell`."
The **conclusion is right** — the arm must stay — but the enumeration is wrong in both directions,
and the wrong list is now baked into a code comment that will outlive the plan:

| plan's claim | what I found by reading |
|---|---|
| `copy.rs:430` cascade | ✅ `copy.rs:383-430`, `kind: StackObjectKind::Spell` |
| `copy.rs:646` discover | ✅ `copy.rs:614-646`, `kind: StackObjectKind::Spell` |
| `engine.rs:2112` | ❌ `ring_ability_stack_object`, `StackObjectKind::RingAbility` (`:2068-2083`) |
| `engine.rs:2176` | ❌ `room_ability_stack_object`, `StackObjectKind::RoomAbility` (`:2132-2147`) |
| `engine.rs:2686` | ❌ `StackObjectKind::LoyaltyAbility` (`:2649-2654`) |
| `engine.rs:2853` | ❌ `StackObjectKind::ClassLevelAbility` (`:2817-2823`) |
| *(missed)* | ⚠️ `resolution.rs:5167-5173` — **cipher copy**, `StackObjectKind::Spell` via `trigger_default` |
| *(missed)* | ⚠️ `resolution.rs:5837-5844` — **suspend free-cast**, `StackObjectKind::Spell` via `trigger_default` |

Method: `grep -n 'StackObjectKind::Spell {' crates/engine/src/` — the only *production*
constructions are `casting.rs:4574/4579` (inside `handle_cast_spell` itself), `copy.rs:386`,
`copy.rs:614`, `resolution.rs:5170`, `resolution.rs:5841`. (`casting.rs:7977` is a
`#[cfg(test)]` helper; `stack.rs:517-555` `trigger_default` is a constructor, not a producer.)

**Failure scenario**: whoever implements DP-20 reads this comment or OOS-DP3-3, "fixes" mode
announcement at four `engine.rs` sites that are Ring/Room/Loyalty/ClassLevel abilities and can
never reach the Spell branch, concludes the arm is now dead, deletes it — and **every suspended or
ciphered modal spell silently resolves nothing.** Suspend in particular is a live path
(`resolution.rs:5837` is reached from the suspend upkeep trigger, not from a test).

**Fix**: replace the producer list at `resolution.rs:340-350` with `copy.rs:386` (cascade),
`copy.rs:614` (discover), `resolution.rs:5167` (cipher copy), `resolution.rs:5837` (suspend
free-cast); state that they all go through `StackObject::trigger_default`
(`crates/card-types/src/state/stack.rs:517-555`), which zero-fills `modes_chosen`; and correct
seed OOS-DP3-3's text identically before filing it in audit §8.1.

### Finding 2: New escalate bounds branches are untested

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:3539-3550`
**CR Rule**: 702.120a — "For each mode you choose beyond the first as you cast this spell, you pay
an additional [cost]" (with CR 700.2a supplying the printed `min`/`max`)
**Issue**: `derived < ms.min_modes` is effectively unreachable (the branch requires
`escalate_modes >= 1`, so `derived >= 2`, and no corpus card has `min_modes > 2`), but
`derived > ms.max_modes` is a **real** guard: on a card with 3 modes and `max_modes: 2`, paying
escalate twice yields `derived == 3` and is now correctly refused where it previously resolved 3
modes. Neither branch has a test. New rejection code with no probe is exactly what the DP suite's
fail-before/pass-after discipline exists to prevent, and a future refactor could delete or invert
either branch with a green suite.
**Fix**: add `test_702_120a_escalate_derived_count_over_max_modes_rejected` to
`pb_dp3_modal_mode_announcement.rs` — a synthetic escalate spell with 3 modes and
`min_modes: 1, max_modes: 2`, cast with `AdditionalCost::EscalateModes { count: 2 }` and empty
modes; assert `Err` containing `"at most"` and `"702.120a"`.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 601.2b (modes announced as part of casting) | Yes | Yes | `casting.rs:3568-3574`; probes 1-4, 16, `modal.rs::test_601_2b_modal_empty_modes_chosen_rejected` |
| 601.2f / 601.2h (rejection precedes cost lock-in and payment) | Yes (by construction) | Weakly | Guard at `:3510` precedes payment `:~4005`; `process_command` ownership makes it atomic. See LOW #8 — the assertions meant to prove this are vacuous |
| 602.2b (activated-ability announcement) | Yes | Yes | `abilities.rs:398-404`; probe 6 |
| 700.2a (controller chooses; range; min/max) | Yes | Yes | `casting.rs:3579-3614`, `abilities.rs:346-381`; tests 11, 13 |
| 700.2b (modal **triggered**, "if no mode is chosen…") | **No** — out of scope | n/a | `abilities.rs:8419-8429` still forces `vec![0]` and has two identical branches; OOS-DP3-4 (verified, see below) |
| 700.2d (no duplicate modes) | Yes (pre-existing) | Yes | `casting.rs:3590-3600`; test 12. No corpus card sets `allow_duplicate_modes: true` (verified: 0 hits) |
| 702.42a/b (entwine overrides the count) | Yes (preserved) | Yes | `casting.rs:3510-3516`; test 14 asserts *both* modes' board effects |
| 702.120a (escalate) | Partial — count validated, identities engine-derived | Partial | Tests 15, 16; the two new bounds branches untested (Finding 2). Residual = OOS-DP3-1 / new OOS-DP3-6 |
| 702.172a (spree) | Yes (untouched) | Yes (pre-existing) | `casting.rs:2938-2945` verified byte-unchanged; owns its own message and fires first |
| 613.1f (layer-resolved ability index) | Yes | Yes | `abilities.rs:313-331`; `legal_actions.rs:1332-1344`; simulator test 20 |
| 700.2g (copies carry chosen modes) | Pre-existing | Yes | `modal.rs` copy tests unaffected |

## Blast-Radius Verification (independent of the plan's table)

| check | method | result |
|---|---|---|
| Other `CastSpellData` / `Command::ActivateAbility` construction sites outside tests | grep workspace, excluding `crates/engine/tests/**` and `memory/**` | Exactly 4 non-doc code sites: `random_bot.rs`, `tools/tui/src/play/input.rs`, `replay_harness.rs`, plus engine internals. **No benches, no fuzz target, no `tools/replay-viewer`, no `local_game.rs` construction** (`local_game.rs:555` is a doc comment). Nothing missed. |
| `heuristic_bot.rs` cast path | read `heuristic_bot.rs:16,127` | Delegates to `random_bot::action_to_command` — the single chokepoint. **Covered.** All 4 `random_bot` sites (`:154`, `:194`, `:302`, `:330`) are wired. |
| `ability_default_modes` index namespace | read `legal_actions.rs:1332-1344` | Uses `calculate_characteristics(...).activated_abilities[ability_index]` with an `o.characteristics` fallback — **layer-resolved, correct** (plan risk 6 avoided). |
| Do the defaults produce a **legal** mode set for every modal card? | `default_modes_chosen = (0..min_modes.min(modes.len()))` + corpus census | Yes. Never duplicates, never out of range, never over `max_modes` (`min ≤ max`), and `min_modes ≤ modes.len()` for all 41 defs — including the three `min_modes: 2` commands (4 modes each ⇒ `[0,1]`). No card sets `allow_duplicate_modes`. **Criterion 5525 satisfied at the mode-set level.** (It does not follow that every bot modal cast now *succeeds* — targets are still `Vec::new()`, so mode-targeted casts are rejected as before. That is not a regression and is M11 R4's scope.) |
| Golden-script corpus sweep for modal cards | grep all 42 modal card names across `test-data/generated-scripts/` | Exactly the plan's 5 files: `stack/147`, `stack/148`, `stack/169` (retired, already modal-explicit), `stack/173` (spree, `cast_spell_modal`), `baseline/112` (modal **trigger**, untouched path). Nothing missed. |
| Harness change is a no-op for existing scripts | `grep '"modes":' test-data/generated-scripts/` | 5 hits: `173:116` and `169:142/247` on `cast_spell_modal` (pre-existing), `147:94` and `148:92` newly added on `cast_spell`. **No `"modes"` on a non-modal card's action.** The pre-edit safety grep in plan §4.4 held. |
| Were any approved scripts retired or weakened? | read both scripts' metadata | **No.** `147` and `148` are both still `"review_status": "approved"`, both now list `"601.2b"` in `cr_sections_tested`, both carry a CR 601.2b note on the edited action, and **no assertion was removed or loosened** — the only diff is an added `"modes": [0]` plus prose. `148`'s scenario-2 `cast_spell_escalate` action (`:148-150`, `escalate_modes: 1`) is correctly untouched (exempt). |
| Wire neutrality | inspected every changed file for type-closure edits | Nothing adds/removes/reshapes a `Command`, `GameEvent`, `Effect`, `GameState`, `StackObject` or `CardDefinition` field or variant. The two new `pub fn`s live in `crates/simulator`, outside the wire closure. **PROTOCOL 27 / HASH 63 correctly unmoved; nothing in the diff should have moved them.** |

## Seeds to file in `docs/audits/decision-point-audit.md` §8.1

The plan proposes OOS-DP3-1..5. My adjudication, plus three additions:

| seed | verdict | notes |
|---|---|---|
| **OOS-DP3-1** (escalate derives a contiguous `0..=count`) | **VALID — file as written** | Confirmed by reading `resolution.rs:321-334`. CR 702.120a permits any `count + 1` distinct modes. Add the fact I verified: **both escalate defs are `partial`**, so no `Complete` card is affected — that is what makes it deferrable. |
| **OOS-DP3-2** (`min_modes: 0` Spell unrepresentable) | **VALID — file as written** | Both claims independently verified (see Ruling 2). Correctly identifies the fix as a HASH bump. |
| **OOS-DP3-3** (free-cast producers bypass announcement) | **VALID IN SUBSTANCE, SITE LIST WRONG — correct before filing** | See Finding 1. True list: `copy.rs:386`, `copy.rs:614`, `resolution.rs:5167`, `resolution.rs:5837`. The four `engine.rs` sites must be struck. |
| **OOS-DP3-4** (modal **triggered** auto-selects mode 0; dead "up to one" branch) | **VALID — file, with a line correction** | Verified at `abilities.rs:8419-8429` (plan said `:8408-8421`): `if min_modes == 0 { vec![0] } else { vec![0] }` — two identical branches, so `hullbreaker_horror` ("choose up to one") can never decline, and CR 700.2b's "if no mode is chosen, the ability is removed from the stack" is never performed. Correctly bundled with PB-DP8. |
| **OOS-DP3-5** (cast-time `ModeSelection` lookup not face-aware) | **VALID — file, with the aftermath sub-case appended** | See Finding 5. Add: `casting.rs:3495` also ignores `casting_with_aftermath`, so an aftermath cast validates against the **front** half's `ModeSelection` and would now be hard-rejected; latent (empty intersection between the 41 modal defs and the 6 adventure/aftermath defs). |
| **OOS-DP3-6** *(new)* | **file** | **An escalate `count` larger than `modes.len() - 1` is silently clamped, not rejected.** `casting.rs:3538` and `resolution.rs:332-333` both `.min(modes.len())`, so paying escalate ×5 on a 3-mode spell costs 5 extra payments and yields 3 modes. CR 702.120a's cost is "for each mode you choose beyond the first", and a player cannot choose 6 modes of a 3-mode spell — the announcement is illegal, not clamp-able. Pinned as intended by `escalate.rs::test_escalate_modes_exceed_available_clamped`. Fix belongs with OOS-DP3-1's escalate PB. Latent for `Complete` cards (both escalate defs are `partial`). |
| **OOS-DP3-7** *(new)* | **file** | **Replay-harness cast-action mode parity (DP-24 class).** Only `cast_spell`, `cast_spell_modal`, `cast_spell_entwine` and `cast_spell_escalate` can announce modes; ~28 alt-cost cast arms (`replay_harness.rs:543`, `:566`, `:589`, `:613`, `:637`, `:666`, `:1078`, `:1113`, `:1140`, `:1165`, `:1195`, `:1222`, `:1249`, `:1273`, `:1297`, `:1320`, `:1343`, `:1366`, `:1393`, `:1431`, `:1459`, `:1486`, `:1507`, `:1536`, `:1571`, `:1604`, `:1644`, `:1669`, `:1697`, `:1945`, `:1972`) hard-code `vec![]`. Post-PB-DP3 that means a modal card is **unscriptable** through those actions (hard-rejected, not silently mode-0'd). Latent: no corpus card is both modal and alt-cost-castable. |
| **OOS-DP3-8** *(new, optional)* | **file or drop** | **Entwine short-circuits all mode validation.** `casting.rs:3510-3516` passes `modes_chosen` through unsorted and unchecked when `entwine_paid`, so out-of-range or duplicate indices reach the `StackObject`. Harmless today (resolution ignores them under entwine), but it is now the only unvalidated arm of the new match. |

## Close-out obligations still outstanding (not defects — bookkeeping)

Verified as **not yet applied** at review time; the plan §10 requires all of them:

- `docs/audits/decision-point-audit.md` **§4.1 line 186** still reads class **D**,
  site `rules/casting.rs:3555-3559`; **§4.2 line 214** still reads class **B**,
  site `rules/abilities.rs:386-397`. Both must flip to **A** with the escalate caveat.
- **§5 DP-4 row (line 431)** still describes the open defect; needs the `SHIPPED (PB-DP3,
  scutemob-151)` prefix. **§5 DP-20 row** needs the corrected cross-reference (Finding 1).
- **§8 PB-DP3 row (line 572)** still reads as a proposal; note that its "Mirror the Spree guard"
  prescription was deliberately **not** followed (the guard was kept, the fix is a lift).
- **§8.1** contains no `OOS-DP3-*` rows yet — file all five as adjudicated above, plus the three
  additions.
- **§9 recommendation 4** annotation (superseded by PB-DP3).
- `CLAUDE.md` Current State / Last Updated, `memory/workstream-state.md` close-out,
  `memory/primitive-wip.md` phase advance.

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `cryptic_command` | Yes (verified vs Scryfall) | 0 | **Yes, now** | `min_modes: 2, max_modes: 2`; `Complete`. Was live-wrong (full cost, one mode). Fixed engine-side, 0 def edits. |
| `austere_command` | Yes (mode order verified line-by-line, `austere_command.rs:31-66`) | 0 | **Yes, now** | Same. |
| `incendiary_command` | Yes (verified vs Scryfall) | 0 | **Yes, now** | Same. |
| 37 × `min_modes: 1` defs | n/a (unmodified) | n/a | **Yes, now** | The broad half of the fix the audit headline understated — all 37 previously accepted an unannounced cast. |
| `hullbreaker_horror` (`min_modes: 0`, **triggered**) | Yes | 0 | No change | Correctly untouched; still auto-selects mode 0 at queue time (OOS-DP3-4). |
| `blessed_alliance`, `collective_resistance` (escalate) | n/a | pre-existing | Unchanged | Both `Completeness::partial` — the escalate residual (OOS-DP3-1/6) is unreachable from a legal deck. |
| `goblin_cratermaker`, `cankerbloom`, `umezawas_jitte` (modal activated) | n/a | 0 | **Yes, now** | All `min_modes: 1`; every in-repo activation already passed explicit modes, so zero test churn. |

## Recommended fix list (in order)

1. **Finding 1** — rewrite the `resolution.rs:340-350` producer list and OOS-DP3-3's text to
   `copy.rs:386`, `copy.rs:614`, `resolution.rs:5167`, `resolution.rs:5837`. *(MEDIUM, ~10 lines)*
2. **Finding 2** — add the `derived > max_modes` escalate probe. *(MEDIUM, ~40 lines of test)*
3. **Finding 8** — de-vacuum the three "no cost was paid" assertions. *(LOW)*
4. **Findings 3, 5, 6, 7** — comment corrections + seed-text amendments. *(LOW, comments only)*
5. File the eight seeds and apply the §10 audit bookkeeping.

None of these touch the guard's control flow, so no re-run of the fail-before probes is required.
