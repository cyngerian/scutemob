# PB-EF12 — Implementation Report

Task scutemob-114. Plan: `memory/primitives/pb-plan-EF12.md`. Coordinator decision:
`memory/decisions.md` (2026-07-18, PB-EF12). This is the **final batch on the EF queue**
(`memory/primitives/ef-batch-plan-2026-07-17.md`).

## Implementation summary (runner)

### 1. Engine core

- `crates/engine/src/rules/command.rs`: `Command::TapForMana` gains
  `chosen_color: Option<ManaColor>` (`#[serde(default)]`), doc-commented with CR
  605.3b/111.10a/106.1b.
- `crates/engine/src/rules/engine.rs:88`: dispatch destructures and threads `chosen_color`
  into `mana::handle_tap_for_mana`.
- `crates/engine/src/rules/mana.rs`:
  - `handle_tap_for_mana` gains the `chosen_color: Option<ManaColor>` parameter.
  - New validation block (inserted right after the ability fetch, ~line 148, i.e. **before**
    the zone/controller legality check — same ordering PB-EF8 established for
    `exile_self_from_hand`): for `ability.any_color == true`, `Some(ManaColor::Colorless)` is
    rejected (CR 106.1b — colorless is a mana type, not a colour), `None` is rejected (no
    silent default), `Some(c)` for `c` ∈ {White,Blue,Black,Red,Green} is accepted and bound to
    `resolved_color`. For `ability.any_color == false`, `Some(_)` is rejected (catches a caller
    supplying a bogus colour on a fixed-colour source). All three rejections use
    `GameStateError::InvalidCommand` (no new error variant, per the plan's stated preference).
  - Step 7b (`base_preview`, feeds the mana-production-replacement filter — Caged Sun etc.):
    for an `any_color` ability, pushes `(resolved_color, 1)` instead of
    `(ManaColor::Colorless, 1)`.
  - Step 8 (pool addition + `GameEvent::ManaAdded`): adds `resolved_color` instead of
    `ManaColor::Colorless`.
  - Removed the stale "Simplified: colorless until interactive color choice is implemented"
    comment.
- No other engine files needed changes — `ManaAbility.any_color`/`.produces` shape is
  unchanged; only the two `handle_tap_for_mana` production sites and the validation block
  moved from "always Colorless" to "the resolved, validated colour."

### 2. Backfill

106 real `Command::TapForMana { .. }` struct-literal sites across 20 files (the plan's ~227
estimate overcounted — consistent with `feedback_pb_yield_calibration.md`'s standing note that
planners overcount by 2-3x). All given `chosen_color: None,` mechanically (a brace-depth-aware
Python script, verified against two comment-only false-positive matches which were fixed by
hand — one in `replay_harness.rs`'s doc comment, corrected rather than corrupted). Then, per
site, any activation of a genuinely `any_color` source was hand-fixed to `Some(colour)`:
`treasure_tokens.rs` (9 sites — Treasure is `any_color: true`; this also **rewrote 4 of that
file's assertions**, which had asserted `mana_pool.colorless` as if that were correct Treasure
behaviour — CR 111.10a says "any color," not colorless, so those assertions were pinning the
pre-fix bug, not the rule), `pain_lands.rs` (City of Brass, 1 site), `grant_activated_ability.rs`
(2 sites, granted Cryptolith-Rite-shaped ability), `mana_triggers.rs` (Forbidden Orchard, 1
site). `LegalAction::TapForMana` (simulator) and 5 golden JSON scripts also needed the new
field/`chosen_color` key — see §5/§9.

### 3. `elven_chorus.rs` — the named EF-W-PB2-3 instance

Replaced the TODO comment with the real grant:
`AbilityDefinition::Static { continuous_effect: ContinuousEffectDef { layer: Ability,
modification: LayerModification::AddManaAbility(ManaAbility{ any_color: true, requires_tap:
true, .. }), filter: EffectFilter::CreaturesYouControl, duration: WhileSourceOnBattlefield } }`
— the identical shape `cryptolith_rite.rs` and `enduring_vitality.rs` already author. Removed
the `partial` marker (now `Complete` via `..Default::default()`); its other two clauses
(look-at-top, cast-from-top) were already shipped.

`cryptolith_rite.rs` and `paradise_mantle.rs` were **already `Complete`** (default) and needed
no card-def change — they were silently producing colorless before this fix and now correctly
produce the chosen colour, since both go through the exact same `ManaAbility.any_color`
dispatch this PB fixed. Verified end-to-end (not eyeballed) by
`test_ef12_granted_any_color_choice_end_to_end`, which builds a Cryptolith-Rite-shaped grant
(the same struct literal `cryptolith_rite.rs` authors) via `state.continuous_effects_mut()`,
confirms `calculate_characteristics` on the recipient creature reports a real `any_color`
`ManaAbility`, then taps it through `process_command` and confirms red (not colorless) lands
in the pool. `citanul_hierophants.rs` grants a **fixed**-colour ability (`{T}: Add {G}`) and is
unaffected by this PB (correct, out of scope). `bootleggers_stash.rs` grants a non-mana
activated ability that creates Treasure tokens; the Treasure's own `any_color` ability is
already covered by the (rewritten) `treasure_tokens.rs` suite.

### 4. Restore sweep — the batch's real yield and its risk

Verified programmatically per plan criteria (a)-(d) for every def using
`Effect::AddManaAnyColor {` (34 candidates, full corpus grep — not the plan's shorter
"likely restorable" list, which was cross-checked against this full scan). Verification for
(a) used `enrich_spec_from_def(...).mana_abilities.iter().any(|ma| ma.any_color)`, not
eyeballing; oracle text for every borderline case was cross-checked against `cards.sqlite`
(the same source the `mtg-rules` MCP server reads — the MCP tool itself was unavailable to
this runner's tool set, so `sqlite3` queries against the shared `cards.sqlite` were used
instead, same authoritative data).

**Restored to `Complete` (17)** — all verified: plain, unrestricted "{T}[, cost]: Add one mana
of any color" with no other unimplemented clause, and the ability's `targets` field is empty
(CR 605.1a) so `mana_ability_lowering` actually lowers it into a real `ManaAbility`:

`birds_of_paradise`, `chromatic_lantern` (self ability + the granted land ability — same
dispatch), `city_of_brass`, `darksteel_ingot`, `decanter_of_endless_water`, `dragons_hoard`,
`dragonstorm_globe`, `elvish_harbinger`, `goldhound` (Tap+SacrificeSelf, served since SR-34),
`mana_confluence` (Tap+PayLife(1), served since SR-34/37), `mox_jasper` (+ its unrelated
"control a Dragon" `activation_condition`, already correctly implemented, unaffected),
`mox_opal` (+ its Metalcraft `activation_condition`, ditto), `ornithopter_of_paradise`,
`patchwork_banner` (oracle-verified via `cards.sqlite`: no "spend only on…" restriction — the
mana ability is plain), `patriars_seal` (its sibling untap ability was already correct),
`staff_of_compleation` (Tap+PayLife(2); its other four abilities already correctly pay via
SR-36), and `elven_chorus` (§3).

**Held back — real second blocker, note rewritten** (7): `command_tower`, `arcane_signet`,
`commanders_sphere`, `path_of_ancestry` — all print "any color **in your commander's color
identity**," a genuine restriction the engine has no runtime mechanism to enforce
(`compute_color_identity` in `rules/commander.rs` is deck-build-validation-only, never
consulted at mana-activation time); restoring these to `Complete` would let a player tap for
any of five colours regardless of their commander, which is *worse* wrong-game-state than the
pre-fix always-colorless bug, not better. `mox_amber` — same defect class, restricted to
colours among legendary creatures/planeswalkers the controller controls. `forbidden_orchard` —
its mana ability is now correctly served, but a **separate, unrelated** blocker (a
`WhenTappedForMana` auto-target dispatch gap, PB-EF6-adjacent, already documented in the file)
survives; note rewritten to say the colour half is fixed and only the trigger gap remains.
`glistening_sphere` — its plain any-color ability is now correctly served, but its "Corrupted"
ability uses the unrelated, still-stubbed `Effect::AddManaChoice`; note rewritten likewise.

**Reverted after being wrongly restored, then caught by the refined gate**: `deathrite_shaman`
— its first ability's oracle text ("{T}: Exile target land card from a graveyard. Add one mana
of any color") looks like a plain any-color clause, but it has a **target**
(`TargetCardInGraveyard`), and CR 605.1a disqualifies any targeted ability from being a mana
ability at all — `mana_ability_lowering` (`testing/replay_harness.rs`) checks
`targets.is_empty()` **first** and returns `None` for this ability regardless of its
`AddManaAnyColor` payload, so it stays a stack-using activated ability whose
`Effect::AddManaAnyColor` still resolves through the unserved `execute_effect` arm (still adds
Colorless). This runner's first pass eyeballed the oracle text and restored it without running
`enrich_spec_from_def` against it; the refined `no_complete_def_uses_an_any_color_mana_stub`
gate (§6) failed with `Offenders: ["Deathrite Shaman"]` on the very next test run and the
restore was reverted with an accurate note. Left as-is (no change needed, blockers unaffected,
notes remain accurate): `phyrexian_altar` (Cost::Sacrifice-of-a-different-permanent, unserved),
`druids_repository`/`gemstone_array` (Cost::RemoveCounter, unserved), `exotic_orchard`/
`fellwar_stone`/`reflecting_pool` (colour-of-other-permanents, explicitly out of scope per
plan), `food_chain` (Restricted/dynamic amount, out of scope), `lotus_cobra`/
`nissa_resurgent_animist`/`outcaster_trailblazer` (triggered/ETB effect, unserved path),
`abstergo_entertainment`/`replicating_ring` (a wholly separate unmodeled ability blocks
`Complete` regardless of the colour fix).

### 5. Gate refinement — `crates/engine/tests/core/effect_choose_gate.rs`

- `registered_colors`: an `any_color: true` `ManaAbility` now maps to the real five-colour
  option set (White/Blue/Black/Red/Green), not `{Colorless}`.
- New `registers_any_color_mana_ability(def, defs)` helper — true iff `enrich_spec_from_def`
  produces ≥1 `any_color: true` mana ability for the def.
- `no_complete_def_uses_an_any_color_mana_stub`: narrowed. `AddManaAnyColorRestricted` /
  `AddManaOfAnyColorAmount` are **always** flagged (never lowered, still stub to Colorless).
  A plain `AddManaAnyColor` is flagged **only if the def registers zero served `any_color`
  abilities** (i.e. every occurrence is an unserved stub). Caught `Deathrite Shaman` live
  during development (§4).
- `land_color_gate_is_not_blind_to_any_color_lands`: rewritten. New correct behaviour: Mana
  Confluence (a real, served, restored-Complete any-color land) has `printed == registered ==
  {W,U,B,R,G}` — both difference sets empty, the gate genuinely matches rather than skipping a
  `known_wrong` card. A synthetic decoy (prints "any color," registers a bare fixed-Green
  ability) still produces a mismatch, proving the parser is not vacuously permissive for any
  any-color-phrased card.
- New `no_complete_def_has_a_mixed_served_and_unserved_any_color_stub`: documents and pins the
  one known hole in the narrowed gate (a def with both a served AND an unserved
  `AddManaAnyColor` occurrence would only be checked for "at least one served," not "every
  occurrence served") and **asserts the corpus currently has none** (verified: no card-def file
  contains `Effect::AddManaAnyColor {` more than once).
- New `served_vs_unserved_any_color_gate_logic_is_not_vacuous`: proves both directions with
  synthetic served (Cost::Tap) and unserved (Triggered) probes, plus a real-corpus positive
  check on Birds of Paradise.
- All decoys/positives ran against the pre-fix logic and correctly failed before the fix landed
  (standard TDD-style verification during authoring, not just post-hoc assertion review).

### 6. Simulator

- `crates/simulator/src/legal_actions.rs`: `LegalAction::TapForMana` gains
  `chosen_color: Option<ManaColor>`. The provider computes it at the offer site: `Some(White)`
  (deterministic WUBRG-first) for an `any_color` ability, `None` for fixed-colour. New test
  `provider_offers_a_concrete_legal_chosen_color_for_any_color_sources` proves the offered
  action, converted straight to a `Command` and run through `process_command`, is accepted by
  the engine.
- `crates/simulator/src/mana_solver.rs`: phase 1 (colour-pip payment) emits
  `Some(needed_color)` for an any_color source satisfying a specific pip; phase 2 (colorless
  payment) is proven unreachable for any_color sources by construction (`produces` is empty for
  `any_color: true`, so the `contains(&Colorless)` filter never matches one) — left `None` with
  a comment explaining why; phase 3 (generic payment) emits `Some(White)` deterministically for
  an any_color source.
- `crates/simulator/src/random_bot.rs`: `LegalAction::TapForMana → Command::TapForMana`
  conversion now threads `chosen_color` through instead of hardcoding `None`.
- `heuristic_bot.rs`'s scoring arm (`LegalAction::TapForMana { .. } => 5`) needed no change —
  it matches on the variant discriminant only.

### 7. Tests — `crates/engine/tests/primitives/pb_ef12_any_color_choice.rs` (+ `mod` line in
   `primitives/main.rs`)

7 tests, all passing:
- `test_ef12_any_color_choice_produces_each_legal_color` — loops WUBRG on a synthetic
  `any_color` source, asserts exactly that colour lands in the pool and the stack stays empty
  (CR 605.3b).
- `test_ef12_elven_chorus_def_authors_the_any_color_grant` — programmatic (serde-tree) check
  that the real `elven_chorus.rs` def now contains the grant.
- `test_ef12_granted_any_color_choice_end_to_end` — the Cryptolith-Rite-shaped grant, described
  in §3.
- `test_ef12_decoy_colorless_choice_rejected` (Decoy A) — `Some(Colorless)` on an any_color
  source → `Err(InvalidCommand)`. **Empirically proven non-vacuous**: temporarily patched
  `handle_tap_for_mana` to accept `Some(Colorless)` and reran — the test failed as expected;
  reverted (verified via `git diff` producing zero unexpected hunks after revert).
- `test_ef12_decoy_missing_choice_rejected` (Decoy B) — `None` on an any_color source →
  `Err(InvalidCommand)`. **Empirically proven non-vacuous** the same way (patched the `None`
  arm to default to White, confirmed the test failed, reverted).
- `test_ef12_decoy_fixed_color_source_rejects_a_choice` (Decoy C) — `Some(Green)` on a real
  Forest → `Err(InvalidCommand)`.
- `test_ef12_protocol_version_sentinel` — `assert_eq!(PROTOCOL_VERSION, 18)`.

### 8. Wire bump

`PROTOCOL_VERSION` **17 → 18** (`Command::TapForMana` is a wire frame; gained a field).
`PROTOCOL_SCHEMA_FINGERPRINT` re-pinned to
`841e4b4130b2e2bfef5b190dc6dc57f18a2ee42a5484a652c2df690358cb115e` (read from the failing-gate
output, per procedure). New `PROTOCOL_HISTORY` row appended (version 18), never editing an
existing row. `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned to
`321ec5d9da45db04da4b1fc2814cea9f1dde733d716dfaf4ff8481d75e4953f3` (the 17-version-16 frozen
prefix, read from the failing-gate output). All 4 `PROTOCOL_VERSION` sentinels bumped to 18
(`core/protocol_schema.rs`, `pb_ef10_sacrifice_driven_amounts.rs`, `pb_ef7_modal_activated.rs`,
and the new `pb_ef12_any_color_choice.rs`).

**`HASH_SCHEMA_VERSION` stays 55** — confirmed, not assumed: `Command` is not part of
`GameState`'s serialized shape (it's an input, not state), and the fix's only `GameState`-side
effect is which `ManaColor` key gets incremented in an already-per-colour `ManaPool` — no new
field, no new variant. `core hash_schema` and its sentinel tests stayed green with zero edits,
confirming the no-bump call.

### 9. Bookkeeping

- **EF-W-PB2-3 closed** in
  `memory/card-authoring/w-pb2-engine-findings-2026-07-17.md` (✅ CLOSED header + full
  closure note, mirroring the doc's established convention for closed findings) and in
  `memory/card-authoring/w-pb2-roster-2026-07-17.md` (patriars_seal and elven_chorus rows
  struck through with their Complete outcome, matching the pattern the doc already used for
  EF-W-PB2-5/EF-W-PB2-8).
- **OOS-EF12-1 filed** (inline in the EF-W-PB2-3 closure note, per that doc's convention of
  filing follow-ups inside the closing entry rather than a separate section): the unserved
  `any_color` family (`AddManaAnyColorRestricted`, `AddManaOfAnyColorAmount`, a plain
  `AddManaAnyColor` on a triggered/ETB effect or a non-`Cost::Tap`-lowerable activation cost)
  still stubs to Colorless; and the commander-color-identity / legendary-colours restriction on
  the 7 held-back cards is unenforced at runtime. Both need a follow-up primitive (a
  resolution-time colour channel for the stack-resolved family; a colour-subset restriction
  mechanism for the identity-restricted family) — neither implemented here, correctly out of
  scope per the plan's "stop and flag" default-to-defer convention.
- `memory/primitives/ef-batch-plan-2026-07-17.md`: **not edited** — the runner brief explicitly
  reserves the queue-complete marker for the coordinator.
- `python3 tools/authoring-report.py` rerun: coverage **61.1% → 62.1%**
  (1,098/1,796 → 1,117/1,798 clean; corpus file count moved 1,796→1,798, unrelated to this
  session — no card-def files were added or removed here, per `git status`). `todo` 527, `empty`
  154 (both roughly unchanged; this batch worked entirely inside the `todo`/`known_wrong`
  bucket via marker flips, not new authoring).

### 10. Gates — all green

- `cargo build --workspace` — clean.
- `cargo test --all` — **3476 passed, 0 failed** (baseline was 3453; +23 net from the new
  `pb_ef12_any_color_choice.rs` tests (7), the new simulator test (1), and the new/refined
  `effect_choose_gate.rs` tests (2 new + narrowing of 1 existing + rewrite of 1 existing, no
  net test-count change from those two)).
- `cargo clippy --all-targets -- -D warnings` — clean (one `nonminimal_bool` lint fixed during
  authoring).
- `cargo fmt --all -- --check` — clean.
- `tools/check-defs-fmt.sh` — clean, 1,798 defs checked.

## Deviations from the plan

- **Yield discipline overrode the plan's specific restore list.** The plan named
  `command_tower` as "likely restorable"; per-card verification (criterion (b): the printed
  clause is genuinely unrestricted) found it is NOT — it prints "in your commander's color
  identity," a real restriction the engine cannot enforce. Held back, contrary to the plan's
  guess but consistent with the plan's own "discount hard" instruction and with
  `arcane_signet.rs`'s *pre-existing* marker (which already correctly named this exact
  blocker for identical oracle text before this task started). `deathrite_shaman` was
  similarly guessed-restorable by the plan and is not, for a different reason (targeted
  ability, CR 605.1a).
- **Test file uses a synthetic same-shape grant rather than casting a literal Cryptolith
  Rite/Citanul Hierophants/Paradise Mantle/Bootleggers Stash through full CastSpell+resolve.**
  Builder-placed permanents don't run `register_static_continuous_effects` (that only fires on
  a real ETB event), so testing the actual card def end-to-end via `process_command` alone
  would require full cast/resolve machinery. Chose the same technique
  `grant_activated_ability.rs` already established (manually push the identical
  `ContinuousEffect` struct literal) — it exercises the identical dispatch path
  (`LayerModification::AddManaAbility` → `calculate_characteristics` →
  `handle_tap_for_mana`), which is what's actually load-bearing for this PB, plus a separate
  serde-tree assertion (`test_ef12_elven_chorus_def_authors_the_any_color_grant`) that the real
  card def contains the grant shape.
- **`no_complete_def_has_a_mixed_served_and_unserved_any_color_stub`** is new, not explicitly
  named in the plan — added because the plan's own text flagged this as a "known hole" to
  document; documenting it as a comment without a machine check would have been the same
  aspirational-comment hazard `memory/conventions.md` warns against, so it's a real assertion
  instead.

## Remaining TODOs / deferred items

None beyond OOS-EF12-1 (filed, not fixed, correctly out of scope per plan). No remaining TODOs
in any of the 17 restored card defs (verified: none contain `// TODO` after the restore).

---

## Review findings (reviewer)

**Reviewer**: primitive-impl-reviewer (Opus) · **Date**: 2026-07-18
**CR rules verified (MCP `get_rule` context + card oracle via `lookup_card`)**: 605.3b (mana
abilities never use the stack), 605.1a (mana ability = no target + could add mana), 106.1a/106.1b
(colorless is a mana *type*, not a colour), 111.10a (Treasure "any color"), 605.5 (special action).
**Engine files reviewed**: `rules/mana.rs`, `rules/command.rs`, `rules/protocol.rs`,
`testing/replay_harness.rs` (lowering + translate_player_action), `tests/core/effect_choose_gate.rs`,
`crates/simulator/src/{legal_actions,mana_solver,random_bot}.rs`.
**Card defs reviewed (all 17 restored + 7 held-back, oracle-verified via MCP)**: birds_of_paradise,
chromatic_lantern, city_of_brass, darksteel_ingot, decanter_of_endless_water, dragons_hoard,
dragonstorm_globe, elvish_harbinger, goldhound, mana_confluence, mox_jasper, mox_opal,
ornithopter_of_paradise, patchwork_banner, patriars_seal, staff_of_compleation, elven_chorus;
held-back: command_tower, arcane_signet, commanders_sphere, path_of_ancestry, mox_amber,
forbidden_orchard, glistening_sphere.

### Verdict: clean

Zero findings at any severity. Every review item in the task brief was checked against CR text
and oracle text independently, and every claim in the runner report that I could verify held.
This is a correct, conservative, well-gated implementation and a clean close to the EF queue.

### What was verified (item by item)

1. **Engine correctness — CORRECT.** `handle_tap_for_mana`'s new validation block (mana.rs
   §2b, ll.146-178) rejects `Some(Colorless)` (CR 106.1b), rejects `None` on an `any_color`
   ability (no silent default — the SR-37 stub being eliminated), and rejects `Some(_)` on a
   fixed ability. The resolved colour flows into **both** the step-7b `base_preview`
   (ll.413-418, so a colour-filter mana replacement such as Caged Sun matches the real choice)
   **and** the step-8 pool addition + `ManaAdded` event (ll.433-445). The two
   `resolved_color.expect("validated above: any_color ability requires Some(c)")` calls
   (ll.417, 435) are provably unreachable: `resolved_color` is `Some` iff `ability.any_color`
   is true (else early `return Err`), `ability` is an immutable local clone (l.138, never
   reassigned), and both `.expect()` sites sit inside `if ability.any_color` guards — so
   `any_color` has the identical value at the validation site and the use sites. The panic
   cannot fire. The validation runs before the zone/controller check (PB-EF8 ordering); the
   only observable consequence is which `InvalidCommand` message an already-illegal activation
   returns — never wrong game state.

2. **Restore roster — CORRECT, all 17.** Oracle text confirmed via MCP `lookup_card` for every
   card. Each genuinely produces one mana of ANY of the five colours (no commander-identity /
   colour-of-other-permanents narrowing), each registers a served `any_color` mana ability (I
   independently confirmed the cost-lowering path: `mana_ability_cost_components` in
   replay_harness.rs accepts Tap / Tap+SacrificeSelf / Tap+PayLife / Tap+Mana sequences, and
   `mana_ability_lowering` requires `targets.is_empty()` — so goldhound's `Sequence[Tap,
   SacrificeSelf]`, staff's `Sequence[Tap, PayLife(2)]`, and mana_confluence's `Sequence[Tap,
   PayLife(1)]` all lower), and — the item I scrutinised hardest — **every other clause on each
   multi-clause card is implemented**:
   - chromatic_lantern: self any_color ability **and** the "Lands you control have …" grant
     (`AddManaAbility` on `LandsYouControl`) — both present.
   - dragons_hoard: all three clauses (Dragon-ETB gold-counter trigger, `Tap+RemoveCounter:
     Draw`, `Tap: any color`) present.
   - dragonstorm_globe: Dragon `EntersWithCounters(+1/+1)` replacement + mana ability.
   - patchwork_banner: `ChooseCreatureType` ETB replacement + chosen-type anthem
     (`CreaturesYouControlOfChosenType` +1/+1) + mana ability.
   - staff_of_compleation: all five abilities (Destroy-you-own w/ target, any-color, Proliferate,
     Draw, {5}:Untap) present; life costs pay via SR-36.
   - patriars_seal: any-color + `{1},{T}: Untap target legendary creature you control` present.
   - city_of_brass: `WhenSelfBecomesTapped → deal 1 to controller` trigger + mana ability.
   - elvish_harbinger: ETB Elf-search-to-top + mana ability. decanter: `NoMaxHandSize` keyword +
     mana ability. darksteel_ingot: Indestructible + mana. ornithopter/birds: Flying + mana.
     mox_opal (Metalcraft `activation_condition`) / mox_jasper ("control a Dragon"
     `activation_condition`) both present and correct.
   No card that should have been held back was restored. The `deathrite_shaman` near-miss
   (targeted ability, CR 605.1a) was correctly caught by the refined gate and reverted.

3. **elven_chorus grant — CORRECT.** Matches oracle: "Creatures you control have '{T}: Add one
   mana of any color.'" via `AddManaAbility(any_color:true)` on `CreaturesYouControl`, the same
   shape as cryptolith_rite/enduring_vitality. Its look-at-top / cast-from-top clauses ship via
   `StaticPlayFromTop { CreaturesOnly, look_at_top: true }`. Now `Complete`.

4. **Held-back — CORRECT.** All 7 are `Completeness::known_wrong` with truthful, CR-cited notes.
   command_tower / arcane_signet / commanders_sphere / path_of_ancestry / mox_amber genuinely
   carry an unenforceable colour-subset restriction (commander identity / legendary-controlled
   colours); forbidden_orchard's surviving blocker is the `WhenTappedForMana` auto-target
   dispatch gap (creates the Spirit for the wrong player); glistening_sphere's is the still-stubbed
   `Effect::AddManaChoice{count:Fixed(3)}` Corrupted ability. Holding these back is the correct
   call — a wrong restore ships worse-than-before game state.

5. **Gate refinement — CORRECT and non-vacuous both directions.** `registered_colors` maps
   `any_color` → WUBRG; `no_complete_def_uses_an_any_color_mana_stub` always-flags the two
   never-lowered variants and flags plain `AddManaAnyColor` only when unserved (via
   `registers_any_color_mana_ability`); `served_vs_unserved_any_color_gate_logic_is_not_vacuous`
   pins both directions with synthetic served/unserved probes plus a real Birds-of-Paradise
   positive; `land_color_gate_is_not_blind_to_any_color_lands` proves Mana Confluence matches
   (printed==registered=={WUBRG}) while a fixed-Green decoy under "any color" phrasing still
   mismatches. The documented "mixed served+unserved" hole is genuinely pinned by
   `no_complete_def_has_a_mixed_served_and_unserved_any_color_stub` (asserts no corpus def has
   >1 `AddManaAnyColor` occurrence). The stub gate correctly does NOT fire on the grant cards
   (elven_chorus/chromatic_lantern grant via a `ManaAbility` struct, not the `Effect` key).

6. **Simulator — CORRECT.** `legal_actions.rs`, `mana_solver.rs` (all 3 phases), and
   `random_bot.rs` emit `Some(colour)` for any_color and `None` for fixed; phase-2's `None` for
   any_color is proven unreachable by construction. New test executes the offered action through
   `process_command` to prove engine-legality (SR-38 precedent honoured — a bot never suggests a
   rejected colour).

7. **Wire — CORRECT.** `PROTOCOL_VERSION == 18` confirmed in protocol.rs (l.171); the fingerprint
   and frozen-prefix re-pins are machine-forced by `protocol_schema.rs`. HASH correctly stays 55:
   `Command` is an engine input, not part of `GameState`'s serialized shape; the only GameState
   effect is which existing `ManaPool` per-colour key increments. The full suite (3476) passing
   with the hash_schema gate untouched confirms `Command` is not in the GameState hash closure.

### Observations (not findings, no action required)

- The golden-script harness path is preserved (SR-9c): `translate_player_action` gained a
  `chosen_color_name` parameter that parses "white/blue/…/green" → `ManaColor`, so a script
  tapping an any_color source can express the colour, and one that omits it fails visibly (the
  engine rejects) rather than silently skipping.
- The two `.expect()` calls in mana.rs are on a locally-proven-unreachable invariant with a
  descriptive message. This is acceptable and not a `.unwrap()`-in-library-code defect. A marginal
  stylistic alternative would bind the concrete colour once in the `any_color` branch and thread
  it, avoiding the re-expect, but the current form is correct and self-documenting — not worth a
  change.
- Grep rendering showed a few doc-comment context lines with a single leading `/` (e.g.
  legal_actions.rs:1216, mana_solver.rs:80/106/124); these are tool display artifacts, not source
  — the workspace builds clean and clippy is green, which a real `/`-prefixed statement line could
  not do.
