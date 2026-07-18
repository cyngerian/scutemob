# PB-EF12 Plan — granted `any_color` ManaAbility color choice (EF-W-PB2-3) — CLOSES THE EF QUEUE

Task scutemob-114. Finding: `memory/card-authoring/w-pb2-engine-findings-2026-07-17.md` EF-W-PB2-3.
Coordinator decision recorded in `memory/decisions.md` (2026-07-18). This closes the EF queue.

## The defect (verified)

`handle_tap_for_mana` (`crates/engine/src/rules/mana.rs`) resolves an `any_color: true` `ManaAbility`
by unconditionally adding `ManaColor::Colorless` (steps 7b base_preview L379 + step 8 L396-407). CR
106.1a/106.1b: colorless is a mana *type*, not a colour, so "add one mana of **any color**" producing
`{C}` is outside the legal option set — wrong game state, not a degraded choice.

This is the **ManaAbility path**. It is reached by:
- Intrinsic `any_color: true` abilities: Command Tower, City of Brass, Mana Confluence, Chromatic
  Lantern, Arcane Signet, Birds of Paradise, Treasure tokens (`ManaAbility::treasure()`), …
  — authored as `AbilityDefinition::Activated { cost: Cost::Tap, effect: Effect::AddManaAnyColor }`
  and **lowered** to `ManaAbility { any_color: true }` by `try_as_tap_mana_ability`
  (`testing/replay_harness.rs:3842`).
- **Granted** `any_color: true` abilities via `LayerModification::AddManaAbility(ManaAbility{any_color:true})`
  + `EffectFilter::CreaturesYouControl`: Cryptolith Rite, Citanul Hierophants, Paradise Mantle,
  Bootleggers Stash — **these ship `Completeness::Complete` today while silently producing colorless**,
  a latent wrong-game-state bug no gate catches (the grant is a `ManaAbility` struct, not an `Effect`
  key, so `effect_choose_gate` misses it; the land-colour gate skips granted clauses). Elven Chorus
  (EF-W-PB2-3's named instance) is honestly `partial` because its author found this stub.

**NOT the ManaAbility path** (out of scope — stays gated / OOS):
- `Effect::AddManaAnyColor` on a **non-tap** cost (spell effect, triggered/ETB effect, or a
  sacrifice-a-DIFFERENT-permanent cost that `mana_ability_cost_components` refuses) — resolves through
  `execute_effect` (`effects/mod.rs:2255`), which has no Command channel to carry a colour and still
  adds `Colorless`.
- `Effect::AddManaAnyColorRestricted`, `Effect::AddManaOfAnyColorAmount`, `Effect::AddManaChoice` —
  `try_as_tap_mana_ability` does NOT lower them, so they are never served by `TapForMana`; each still
  adds `Colorless` in `effects/mod.rs`.
- Colour-of-other-permanents lands/rocks that (mis)use `AddManaAnyColor` as an approximation:
  **Fellwar Stone, Exotic Orchard, Reflecting Pool** ("mana of any type/colour a land an opponent
  could produce", CR 106.7) — a DIFFERENT mechanic. **Do NOT restore these.**

## Design (per coordinator decision — CR 605.3b)

A mana ability resolves immediately and never uses the stack (CR 605.3b), so any choice is made at
activation. The colour rides the activation Command:

```
Command::TapForMana { player, source, ability_index, chosen_color: Option<ManaColor> }
```
`#[serde(default)]` so old logs decode (as `None`). Validated in `handle_tap_for_mana`:
- ability `any_color == true`: require `chosen_color == Some(c)` where `c ∈ {White,Blue,Black,Red,Green}`.
  Reject `None` → new `GameStateError` (no silent Colorless default). Reject `Some(Colorless)` → error
  (CR 106.1b). Produce `c`.
- ability `any_color == false`: `chosen_color` must be `None`. If `Some(_)` supplied, reject with an
  error ("colour choice supplied for a fixed-colour mana ability") — catches caller bugs; keeps the
  channel honest. (All 227 existing fixed-colour call sites pass `None`.)

**Wire**: `Command` is inside the SR-8 fingerprint closure → **PROTOCOL 17→18**, machine-forced,
re-pin `PROTOCOL_SCHEMA_FINGERPRINT` from the failing-gate output, append a history row, bump the 3
`PROTOCOL_VERSION` sentinels (protocol_schema.rs:869, pb_ef10:1263, pb_ef7:242 — grep for the exact
current set). **HASH does NOT bump**: `Command` is not in the GameState hash closure and no `GameState`
field / card-DSL type changes (the colour lands in `ManaPool`, already per-colour). If the hash_schema
gate somehow reddens, STOP and investigate — it should stay 55. Justify the no-HASH in the commit.

## Steps

### 1-3. Engine core (`crates/engine/src/rules/command.rs`, `engine.rs`, `mana.rs`)
- `command.rs`: add `chosen_color: Option<ManaColor>` (import `ManaColor` from `crate::state::types`)
  with a `#[serde(default)]` + doc comment (CR 605.3b/111.10a/106.1b).
- `engine.rs:88` dispatch: destructure `chosen_color`, pass to `handle_tap_for_mana`.
- `mana.rs handle_tap_for_mana`: add `chosen_color: Option<ManaColor>` param. After the ability is
  fetched (~L144), add a validation block:
  - if `ability.any_color`: `let color = match chosen_color { Some(c) if c != ManaColor::Colorless => c,
    Some(ManaColor::Colorless) => return Err(...), None => return Err(...) }`. Thread `color` down.
  - else: `if chosen_color.is_some() { return Err(...) }`.
  - Add a `GameStateError` variant e.g. `MissingColorChoice { object_id }` /
    `InvalidColorChoice { object_id }` — OR reuse `InvalidCommand(String)` with a clear CR-cited
    message (simpler, no new error type, no wire impact — PREFER this unless a typed error is cheap).
    Decision: use `GameStateError::InvalidCommand` with CR 605.3b/106.1b messages (no new error variant,
    keeps the change tight). Confirm `GameStateError` is not itself on the wire in a way that forces a
    bump — it is a `Result` error, not part of `Command`/`GameEvent`, so it is fine.
  - step 7b base_preview (L379): `if ability.any_color { base_preview.push((color, 1)); }` (was
    `Colorless`). This makes a colour-filter mana replacement (Caged Sun "add one for each {chosen}")
    match the real chosen colour.
  - step 8 (L396-407): add `color` (not `Colorless`) to the pool + `ManaAdded` event.
- Non-any_color paths unchanged.

### 4. Backfill `chosen_color: None,` to all existing `Command::TapForMana { .. }` literals (~227 sites)
Mechanical, like PB-EF7's `modes: None` backfill. Every `Command::TapForMana { player, source,
ability_index }` literal must gain `chosen_color: None`. Approach: a targeted `sed`/`perl` that inserts
`chosen_color: None,` after the `ability_index: <expr>,` line **only within TapForMana blocks** — but
`ability_index` also appears in `ActivateAbility`/`LegalAction`, so a naive sed is unsafe. Safer: use a
perl one-liner that matches the multi-line `Command::TapForMana {` … `}` block and inserts before its
closing `}`. Verify with `cargo build --workspace` (missing-field errors will list any missed site) and
iterate until zero. Do NOT touch `LegalAction::TapForMana` yet (step 8 handles it). Files with the most
sites: mana_triggers.rs(16), primitive_sr34(13), mana_filter.rs(10), mana_and_lands.rs(10),
treasure_tokens.rs(9), pain_lands.rs(8), primitive_sr36(7), pb_ef8(5), … (full list from
`grep -rn "Command::TapForMana {" crates tools`).

### 5. elven_chorus + grant-based cards
- `elven_chorus.rs`: replace the TODO ability (index 1) with the Cryptolith Rite grant pattern
  (`AbilityDefinition::Static { continuous_effect: ContinuousEffectDef { layer: EffectLayer::Ability,
  modification: LayerModification::AddManaAbility(ManaAbility{ requires_tap:true, any_color:true, ..}),
  filter: EffectFilter::CreaturesYouControl, duration: WhileSourceOnBattlefield, condition: None }}`).
  Verify oracle via MCP. Flip `completeness` → remove the `partial` marker (Complete) — its other two
  clauses (look-at-top, cast-from-top) already ship.
- Verify (test, don't just eyeball) Cryptolith Rite / Citanul Hierophants / Paradise Mantle /
  Bootleggers Stash now produce the chosen colour end-to-end. They are already `Complete`; no flip,
  but add coverage so the latent bug can't return.

### 6. Restore demoted served tap-cost `AddManaAnyColor` cards (CONSERVATIVE, per-card verified)
Restore a demoted def to `Complete` **iff ALL hold** (verify each; honest markers beat over-restoration
per `feedback_pb_yield_calibration.md`):
  (a) `enrich_spec_from_def(...)` registers ≥1 `ManaAbility { any_color: true }` for the card (served —
      programmatic check, not eyeball);
  (b) the card's printed any-colour clause is a plain "{T}: Add one mana of any color" tap ability
      (NOT "any colour a land could produce" — exclude Fellwar Stone / Exotic Orchard / Reflecting Pool;
      NOT a restricted/amount/spell/triggered variant);
  (c) the def has **no other unimplemented clause** (its `known_wrong`/`partial` note cited only the
      any-colour stub — re-read the note; if a second blocker survives, keep it non-Complete with the
      note rewritten to the real blocker, and DO NOT restore);
  (d) oracle text confirmed via MCP `lookup_card`.
Likely restorable (VERIFY each): command_tower, city_of_brass, mana_confluence, chromatic_lantern,
arcane_signet, birds_of_paradise, commanders_sphere, darksteel_ingot, ornithopter_of_paradise,
dragons_hoard, patriars_seal, patchwork_banner, decanter_of_endless_water, mox_amber/mox_opal/mox_jasper
(if their only blocker was the stub), gemstone_array, dragonstorm_globe, replicating_ring, path_of_ancestry.
DO NOT restore without (a)-(d): fellwar_stone, exotic_orchard, reflecting_pool, food_chain
(Restricted), phyrexian_altar/goldhound (sacrifice-cost — verify servedness — likely stub),
deathrite_shaman/druids_repository/lotus_cobra/faeburrow_elder/incubation_druid/nissa_resurgent_animist/
sarkhan_unbroken/abstergo_entertainment/staff_of_compleation/glistening_sphere/outcaster_trailblazer
(second blocker or non-tap — verify; keep non-Complete if any (a)-(c) fails).
**This is the batch's real yield. Discount hard; a wrong restore ships wrong game state (worse than a
partial).** Record the final restored list + the held-back list with reasons in the review file.

### 7. Gate refinement (`crates/engine/tests/core/effect_choose_gate.rs`)
- `registered_colors`: an `any_color: true` ManaAbility now produces any of WUBRG. Change the mapping
  from `push(Colorless)` to extending WUBRG (the five real colours). This makes Command Tower (prints 5,
  registers 5) PASS `every_complete_land_registers_each_printed_tap_mana_color`.
- `land_color_gate_is_not_blind_to_any_color_lands`: Command Tower is being restored to Complete, so it
  will no longer be a `known_wrong` example. Rewrite this non-vacuity test to assert the NEW correct
  behaviour: an any_color land registers all five printed colours (printed==registered, gate passes) —
  and keep a synthetic/decoy proving the parser still detects the "any color" phrasing. Ensure the test
  still proves the machinery is not blind.
- `no_complete_def_uses_an_any_color_mana_stub`: refine so it flags a Complete def iff it contains an
  UNSERVED any-color-family effect:
    * `AddManaAnyColorRestricted` / `AddManaOfAnyColorAmount` → always flag (never lowered).
    * plain `AddManaAnyColor` → flag ONLY if the def registers NO `any_color: true` mana ability (every
      occurrence is a stack/triggered stub). If it registers ≥1, treat the plain usage as the served tap
      ability and don't flag. **Document the known hole** (a def with both a served tap any_color AND a
      separate stack `AddManaAnyColor` would pass) and assert the corpus currently has no such case, OR
      handle it if one exists. Keep `AddManaChoice` gated by its own test unchanged.
  Update `stub_gates_are_not_vacuous` so the restricted/amount probes still fail-closed, and add a
  positive case proving a served plain-`AddManaAnyColor` Complete def is NOT flagged (e.g. Command Tower).
- `sr33_demoted_cards_carry_truthful_markers`: unaffected (Path to Exile / Rhystic Study), keep.

### 8. Simulator (`crates/simulator/src/legal_actions.rs`, `mana_solver.rs`, `random_bot.rs`)
- `LegalAction::TapForMana` gains `chosen_color: Option<ManaColor>` (or the provider emits one legal
  colour per any_color ability; deterministic WUBRG order — first legal = White). Wherever a
  `LegalAction::TapForMana` is converted to a `Command::TapForMana`, populate `chosen_color`
  correctly: `None` for fixed, `Some(White)` (or a deterministic pick) for any_color. A bot must never
  emit a colour the engine rejects (SR-38 precedent). `mana_solver.rs` (3 sites): when solving for a
  specific colour, an any_color source can satisfy any colour — emit `Some(needed_color)`.
  `heuristic_bot.rs` scoring arm unaffected (still matches the variant).
- Add a simulator test proving every emitted `TapForMana` is engine-legal for an any_color source.

### 9. Tests (`crates/engine/tests/primitives/pb_ef12_any_color_choice.rs`, add `mod` to primitives/main.rs)
- happy: an intrinsic any_color source (Command Tower) + `TapForMana{ chosen_color: Some(c) }` adds
  exactly `c` to the pool, for each of WUBRG; stack stays empty (CR 605.3b).
- granted happy: Cryptolith Rite (or Elven Chorus) grants "{T}: Add any color" to a creature you
  control; tap it with `Some(Red)` → red in pool. End-to-end via `process_command`.
- decoy A (non-vacuous): `chosen_color: Some(Colorless)` on an any_color source → `Err`. Prove
  non-vacuous (removing the Colorless check makes it pass).
- decoy B (non-vacuous): `chosen_color: None` on an any_color source → `Err` (no silent Colorless).
- decoy C: `chosen_color: Some(Green)` on a FIXED source (Forest) → `Err`.
- replacement interaction (optional but strong): Caged Sun naming the chosen colour adds its +1 to the
  chosen colour, not Colorless.
- version sentinel: `assert_eq!(PROTOCOL_VERSION, 18)` + `assert_eq!(HASH_SCHEMA_VERSION, 55)`.
- Add PROTOCOL_VERSION sentinel to this file; update the 3 existing sentinels to 18.

### 10. Wire bump
PROTOCOL 17→18, append protocol.rs history row (Command::TapForMana gains chosen_color; CR 605.3b),
re-pin `PROTOCOL_SCHEMA_FINGERPRINT` from `cargo test --test core protocol_schema` failing output. Bump
all `PROTOCOL_VERSION` sentinels. HASH stays 55 (justify; if hash_schema reddens, investigate — should not).

### 11. Bookkeeping + OOS
- Close EF-W-PB2-3 in `w-pb2-engine-findings-2026-07-17.md` (✅ CLOSED header) + roster.
- File **OOS-EF12-1**: the unserved any-color family (`AddManaAnyColorRestricted`,
  `AddManaOfAnyColorAmount`, `AddManaChoice`, and plain `AddManaAnyColor` on spell/triggered/
  sacrifice-a-different-permanent costs) still stubs to Colorless — needs either a resolution-time
  colour channel or per-variant lowering. List the held-back cards. File in
  `w-pb2-engine-findings-2026-07-17.md`.
- `ef-batch-plan-2026-07-17.md`: mark PB-EF12 ✅ DONE + **THE EF QUEUE COMPLETE** (queue summary table
  + a status header). 
- Rerun `python3 tools/authoring-report.py`; record coverage delta in the review + collection report.

### 12. Gates + review
`cargo build --workspace`, `cargo test --all`, `cargo clippy --all-targets -- -D warnings`,
`cargo fmt --check` + `tools/check-defs-fmt.sh`. Then `/review` (primitive-impl-reviewer). Address findings.

## Gotchas
- `Command::TapForMana` literal churn (~227) — build errors enumerate misses; iterate.
- The stub gate must stay non-vacuous in BOTH directions after refinement.
- Restore is the risk surface: a wrong restore ships wrong game state. Verify servedness programmatically
  + oracle via MCP; keep honest markers on any card with a surviving blocker.
- Treasure tokens (`ManaAbility::treasure()`) are `any_color: true` — they now REQUIRE a chosen_color.
  Any test/script/harness path that sacs a Treasure for mana via TapForMana must pass `Some(colour)`.
  Check `treasure_tokens.rs` (9 sites) and any script harness Treasure activation.
