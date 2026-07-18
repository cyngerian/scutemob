# Primitive Batch Plan: PB-EF7 — modal `AbilityDefinition::Activated { modes }`

**Generated**: 2026-07-18
**Primitive**: Add `modes: Option<ModeSelection>` to `AbilityDefinition::Activated` (DSL) and to
the runtime `ActivatedAbility` struct, plus `modes_chosen: Vec<usize>` to
`Command::ActivateAbility`, so a "Choose one —" **activated** ability announces its mode at
activation (CR 601.2b via 602.2b) and resolves ONLY the chosen mode — reusing the existing
PB-AC4 `ModeSelection.mode_targets` per-mode target machinery — instead of always executing
`Effect::Choose`'s first branch (a gated stub).
**CR Rules**: 601.2b, 602.2 / 602.2b, 700.2 / 700.2a / 700.2c / 700.2d / 700.2f
**Cards affected**: 2 flips (Goblin Cratermaker `known_wrong`→Complete, Cankerbloom
`known_wrong`→Complete) + 1 honest note/seed (Umezawa's Jitte stays `known_wrong`, OOS seed).
**Dependencies**: PB-AC4 (`ModeSelection.mode_targets` + `validate_targets_positional`) — exists.
**Deferred items from prior PBs**: EF-W-PB2-4 (this batch closes it).

---

## Recon corrections to `memory/primitive-wip.md`

The WIP recon is accurate on file/line locations. Three corrections/refinements:

1. **RESOLUTION.rs:1841 does NOT need modification** under the recommended approach (a). The
   WIP says "Replace `ability_effect` with the chosen mode effects from `ModeSelection` when
   `modes_chosen` non-empty, mirroring the Triggered modal path at resolution.rs:2009-2049."
   That describes **approach (b)**. This plan recommends **approach (a)** — resolve the chosen
   mode into a concrete `embedded_effect` **at activation time** and slice the mode's targets
   into `stack_obj.targets` there. The existing ActivatedAbility resolution arm then resolves
   it **unchanged** (it already reads `embedded_effect` + `stack_obj.targets`). See
   "SacrificeSelf hazard" below for why (a) is correct and simpler.

2. **The DSL enum field addition is a corpus-wide mechanical change the WIP did not quantify.**
   Card defs write the **full** `AbilityDefinition::Activated { … once_per_turn: … }` literal
   with **no `..` rest** (verified: `drudge_skeletons.rs`, `goblin_cratermaker.rs`). Adding
   `modes` therefore forces `modes: None,` into **every** activated-ability def literal
   (~600–800 occurrences across ~500+ files) — exactly as `once_per_turn` did when PB-AC1 added
   it. This is the #1 mechanical surface and #1 compile-error risk. Scripted recipe below.

3. **`Command::ActivateAbility.modes_chosen` breaks ~180 test literals.** Struct-variant
   literals must name every field (`#[serde(default)]` helps only deserialization, not Rust
   literals). All ~180 `Command::ActivateAbility { … }` sites (nearly all in `tests/`) need
   `modes_chosen: vec![],`. Second big mechanical surface.

The runtime `ActivatedAbility` struct **derives `Default`**, so its `modes` field is absorbed by
the `..Default::default()` most construction sites already use — LOW churn (only the all-fields
`enrich` site + hash arm are affected).

---

## CR Rule Text (from MCP)

**601.2b** — "If the spell is modal, the player announces the mode choice (see rule 700.2). …
If the spell has a variable cost … the player announces the value of that variable. …"

**602.2** — "To activate an ability is to put it onto the stack and pay its costs … If, at any
point during the activation …, a player is unable to comply …, the activation is illegal; the
game returns to the moment before that ability started to be activated (see rule 732)."
**602.2b** — "The remainder of the process for activating an ability is **identical to the
process for casting a spell listed in rules 601.2b–i**. Those rules apply to activating an
ability just as they apply to casting a spell." → 601.2b (mode announcement) applies to
activation. This is the whole legal basis for the primitive.

**700.2** — "A spell or ability is modal if it has two or more options in a bulleted list
preceded by instructions for a player to choose a number of those options, such as 'Choose
one —.' Each of those options is a mode."
- **700.2a** — "The controller of a modal spell or **activated ability** chooses the mode(s) as
  part of … activating that ability. If one of the modes would be illegal (due to an inability
  to choose legal targets …), that mode can't be chosen."
- **700.2c** — "If a spell or ability targets … only if a particular mode is chosen …, its
  controller will need to choose those targets only if they chose that mode. Otherwise, the …
  ability is treated as though it did not have those targets." → the Cankerbloom fix: choosing
  Proliferate requires NO artifact/enchantment target.
- **700.2d** — duplicate modes only if "You may choose the same mode more than once" (both our
  cards are choose-exactly-one → `allow_duplicate_modes: false`).
- **700.2f** — "Modal spells and abilities may have different targeting requirements for each
  mode. Changing a … target can't change its mode." → per-mode `mode_targets`.

---

## Recommended design: approach (a) — resolve chosen mode at activation

### Why (a), not (b)

Both eligible cards cost `Cost::SacrificeSelf`, so at resolution `state.objects.get(source)` is
`None` (CR 400.7 — the sacrificed object's ObjectId is dead). The ActivatedAbility SOK already
solves this by capturing `embedded_effect` at activation (`abilities.rs:1116`,
`resolution.rs:1847-1850`). Approach (a) piggybacks on that exact mechanism:

- At activation, after validating modes + targets, compute the chosen-mode effect and store it
  as `embedded_effect`. Store the mode's target slice as `stack_obj.targets`.
- Resolution arm (`resolution.rs:1841`) is **unchanged**: it reads `embedded_effect` (the chosen
  mode) and `stack_obj.targets` (the mode's slice) and executes.

Approach (b) would need the `ModeSelection` **at resolution**, but the source (and its runtime
`ActivatedAbility.modes`) is gone for sacrifice-cost cards, and the SOK carries no ModeSelection.
Embedding the ModeSelection into the SOK is extra wire surface for zero benefit. (a) also gives
the **strongest LKI guarantee**: the chosen mode is frozen into the effect at activation, so no
intervening board change between activation and resolution can alter which mode resolves.

### Per-mode `DeclaredTarget` index threading (both cards are choose-exactly-one)

Both cards have `min_modes: 1, max_modes: 1`. Exactly one mode is chosen, so:
- `mode_targets_active` = `mode_targets[chosen_idx]` (a single mode's requirement list).
- Announced `targets` are validated **positionally** against that slice →
  `stack_obj.targets` = exactly that mode's targets.
- `embedded_effect` = `modes.modes[chosen_idx]`, whose `DeclaredTarget { index }` values are
  **LOCAL** to that mode's slice (index 0 = the mode's first target). Because only one mode is
  chosen, local == global: `ctx.targets` at resolution IS the single slice, so
  `DeclaredTarget { index: 0 }` → `stack_obj.targets[0]`. Correct with **no** offset math.

This exactly mirrors the Spell per-mode loop (`resolution.rs:458-474`), which sets
`ctx.targets = slice` per mode; here the slice is the whole list because there's one mode.

**Multi-mode handling (future-proofing, flag-don't-extend):** In `handle_activate_ability`,
- single chosen mode → `embedded_effect = modes.modes[idx].clone()`.
- multiple chosen modes **with `mode_targets: None`** → `embedded_effect =
  Effect::Sequence(chosen effects)` sharing one `ctx.targets` (global indices — identical to the
  Triggered path at `resolution.rs:2029-2048`). Legal today, no card uses it, low risk.
- multiple chosen modes **with `mode_targets: Some`** → **hard-reject** with a typed
  `InvalidCommand`, exactly as casting hard-rejects Escalate+mode_targets
  (`casting.rs:3669-3674`). A flat Sequence would break local per-mode indexing; extending both
  ladders together is a future micro-PB (`memory/conventions.md` "implement-phase
  default-to-defer"). Neither eligible card hits this branch.

---

## Engine Changes

### Change 1 — DSL enum: `AbilityDefinition::Activated { modes }`

**File**: `crates/card-types/src/cards/card_definition.rs` (variant at L285-313)
**Action**: add as the **last** field (after `once_per_turn`):
```rust
/// CR 700.2a: Modal activated ability. When `Some`, the controller chooses mode(s) as
/// the ability is activated (Command::ActivateAbility.modes_chosen); the chosen mode's
/// effect replaces `effect` at resolution. Per-mode targets ride `ModeSelection.mode_targets`
/// (PB-AC4), LOCAL to each mode's slice. `effect` should be `Effect::Sequence(vec![])` when
/// this is `Some`. Mirrors `Spell`/`Triggered` `modes`.
#[serde(default)]
modes: Option<ModeSelection>,
```
**Pattern**: `AbilityDefinition::Spell.modes` (same file, L330 on the Triggered variant; Spell
variant likewise). `ModeSelection` already exists (L3716) with `mode_targets` (L3749).
**Consequence**: corpus-wide `modes: None,` insertion — see Change 8.

### Change 2 — runtime struct: `ActivatedAbility { modes }`

**File**: `crates/card-types/src/state/game_object.rs` (struct at L352-388, derives `Default`)
**Action**: add field (Default-absorbed at most call sites):
```rust
/// CR 700.2a: Modal activated ability. Propagated from
/// `AbilityDefinition::Activated::modes` by `enrich_spec_from_def`. Read by
/// `handle_activate_ability` to validate `modes_chosen` and slice per-mode targets.
#[serde(default)]
pub modes: Option<crate::cards::card_definition::ModeSelection>,
```

### Change 3 — Command: `Command::ActivateAbility { modes_chosen }`

**File**: `crates/engine/src/rules/command.rs` (variant at L67-87)
**Action**: add field:
```rust
/// CR 601.2b/700.2a (via 602.2b): mode indices chosen for a modal activated ability.
/// Empty = non-modal, or auto-select mode 0 (bot/backward-compat). Validated in
/// handle_activate_ability.
#[serde(default)]
modes_chosen: Vec<usize>,
```
**Consequence**: ~180 `Command::ActivateAbility { … }` literals need `modes_chosen: vec![],` —
see Change 9. **Wire change → PROTOCOL bump.**

### Change 4 — dispatch: thread `modes_chosen` into the handler

**File**: `crates/engine/src/rules/engine.rs` (L147-171)
**Action**: destructure `modes_chosen` and pass it to `handle_activate_ability`.

### Change 5 — `enrich_spec_from_def` propagation

**File**: `crates/engine/src/testing/replay_harness.rs` (Activated loop L2135-2174)
**Action**: add `modes,` to the `AbilityDefinition::Activated { … }` destructure (L2136-2145,
currently ends with `..` — replace/extend to capture `modes`) and set `modes: modes.clone(),`
on the runtime `ActivatedAbility { … }` literal (L2155-2172). This all-fields literal is one of
the few runtime `ActivatedAbility` sites that must gain the field explicitly.

### Change 6 — `handle_activate_ability` mode validation + per-mode target announce + effect bake

**File**: `crates/engine/src/rules/abilities.rs` (`handle_activate_ability`, L130+; signature L130-139)
**Action**:
1. Signature: add `mut modes_chosen: Vec<usize>` parameter.
2. Read `ab.modes.clone()` alongside cost/effect/targets at the capture block (L308-326). Add a
   4th/5th binding `ability_modes: Option<ModeSelection>` from `ab.modes`.
3. **Mode validation (CR 700.2a/700.2d)** — mirror `casting.rs:3506-3559` verbatim: if
   `modes_chosen` non-empty, `ability_modes` must be `Some` (else `InvalidCommand` "…has no
   modal structure (CR 700.2a)"); every index `< modes.len()`; no dup unless
   `allow_duplicate_modes` (CR 700.2d); `min_modes <= len <= max_modes`; `sort_unstable()`
   (ascending printed order). Produce `validated_modes_chosen`. **Empty `modes_chosen` on a
   modal ability → auto-select `vec![0]`** for backward-compat / bots (mirror casting's
   `else if !modes.modes.is_empty() { vec![0] }`).
4. **Per-mode target requirements (CR 700.2c/700.2f)** — mirror `casting.rs:3627-3660`: if
   `ability_modes.mode_targets` is `Some`, `mode_targets_active` =
   `chosen indices flat_map mode_targets[idx]`; debug_assert `mode_targets.len() == modes.len()`.
5. **Target validation split** — replace the current single validation (L330-351):
   - if `mode_targets_active` is `Some`: require the flat `ab.targets` be empty and no `UpToN`
     in the slice (mirror `casting.rs:3686-3698`), then call
     `crate::rules::casting::validate_targets_positional(state, &targets, active_reqs, player,
     source_chars.as_ref(), Some(source))` (signature confirmed at `casting.rs:5915`).
   - else: existing `validate_targets_with_source(…)` path (unchanged, threads `exclude_self`).
   - hard-reject `mode_targets_active.is_some() && validated_modes_chosen.len() > 1` (typed
     `InvalidCommand`, mirror `casting.rs:3669`).
   **Ordering**: all of this is BEFORE any cost payment (payment begins ~L489), satisfying CR
   602.2's "illegal activation rewinds" — no mana/sacrifice is spent on an illegal mode/target.
6. **Bake the chosen-mode effect** — override the captured `embedded_effect` when
   `ability_modes` is `Some` and `validated_modes_chosen` non-empty:
   - 1 index → `Some(modes.modes[idx].clone())`.
   - >1 index + `mode_targets: None` → `Some(Effect::Sequence(chosen effects))`.
   - (the >1 + `mode_targets: Some` case is already hard-rejected above.)
7. Set `stack_obj.modes_chosen = validated_modes_chosen;` after the existing
   `stack_obj.targets = spell_targets;` (L1119) — for LKI/replay/hash observability. `StackObject`
   already has `modes_chosen: Vec<usize>` (`stack.rs:413`); `trigger_default` inits it `vec![]`
   (`stack.rs:554`).
**CR**: 601.2b, 602.2b, 700.2a/700.2c/700.2d/700.2f.

### Change 7 — Hash arms (HASH bump)

**File**: `crates/engine/src/state/hash.rs`
- **DSL arm** L6617-6634 (`AbilityDefinition::Activated { … }`, no `..`): add `modes` to the
  destructure and `modes.hash_into(hasher);`.
- **Runtime arm** L2816-2832 (`impl HashInto for ActivatedAbility`): add
  `self.modes.hash_into(hasher);`.
`ModeSelection: HashInto` already exists (L5780). **Runtime `ActivatedAbility.modes` reaches
GameState → HASH bump.**

### Change 8 — corpus-wide `modes: None,` on DSL literals (MECHANICAL, ~600–800 sites)

**Files**: every `crates/card-defs/src/defs/*.rs` containing `AbilityDefinition::Activated {`.
**Action**: insert `modes: None,` as the final field of each such literal. **Do NOT sed on
`once_per_turn:`** — `AbilityDefinition::Triggered` also ends with `once_per_turn` and must NOT
receive `modes: None`. Use a brace-matching script (Python): for each `AbilityDefinition::Activated {`
occurrence, track brace depth to its matching `}` and insert `modes: None,` immediately before
it. Then run `tools/check-defs-fmt.sh --fix` and `cargo build --workspace`. SR-35 caveat: verify
no def now overflows 100 cols (the script's own canary / `error_on_line_overflow`); `modes: None,`
is short so this is unlikely. The two flipped defs (Change 11) get real `modes: Some(...)` instead.

### Change 9 — `modes_chosen: vec![],` on Command literals (MECHANICAL, ~180 sites)

**Files**: all `Command::ActivateAbility { … }` literals — engine `tests/` (see roster below),
`simulator/src/random_bot.rs:169`, `replay_harness.rs:669`. Add `modes_chosen: vec![],`.
Engine-source sites: `random_bot.rs` (bot always mode 0 via empty), `replay_harness.rs:669`
(translate — read from a new optional script field `modes_chosen`, default `vec![]`).
Test-file roster (each needs the field on every literal): `tests/casting/{cost_primitives,
animated_creature_sacrifice_cost,spell_cost_modification,x_cost_spells}.rs`,
`tests/rules/{grant_activated_ability,split_second,targeted_abilities,restrictions,abilities,
activation_condition}.rs`, `tests/mechanics_a_d/{blood_tokens,adapt,clue_tokens,channel,ascend}.rs`,
`tests/mechanics_e_l/{living_weapon,investigate,land_animation,forage,graveyard_abilities,fortify,
hideaway,equip,keywords,food_tokens}.rs`, `tests/mechanics_m_z/{outlast,meld,ward,reconfigure}.rs`,
`tests/primitives/{pbp_power_of_sacrificed_creature,primitive_pb_x,primitive_pb_xa,primitive_pb_xs,
primitive_pb_xa2,primitive_sr36_scaled_mana_and_life_costs,pb_ac5_alt_costs,pb_ac6_card_integration,
pb_ac6_phase_action_conditions,pb_ac8_restrictions_and_wingame,pb_ef1_exclude_self_enforcement}.rs`,
`tests/scripts/harness_equivalence.rs` (its `Move::ActivateAbility → Command::ActivateAbility` at
L482). Verify complete via `cargo build --workspace` (E0063 "missing field `modes_chosen`").

### Change 10 — test literals constructing `AbilityDefinition::Activated { … }` (~15 files)

Non-def sites that construct the DSL literal need `modes: None,`:
`crates/engine/src/testing/replay_harness.rs` (a few builder sites), `tests/primitives/{primitive_pb_xa,
primitive_pb_xa2,primitive_pb_xs,primitive_sr34_composite_mana_costs,
primitive_sr36_scaled_mana_and_life_costs,primitive_sr37_conditioned_mana_abilities,
pb_ac3_dynamic_pt_counts,pb_ac5_alt_costs,pb_ef1_exclude_self_enforcement}.rs`,
`tests/mechanics_m_z/meld.rs`, `tests/mechanics_e_l/{graveyard_abilities,hideaway}.rs`,
`tests/core/{card_def_fixes,effect_choose_gate}.rs`. (Ignore doc-comment mentions in
`card-types/src/state/types.rs`.) `cargo build --workspace` enumerates any missed.

### Change 11 — Exhaustive match sites (no new arms needed)

| File | Match | Note |
|------|-------|------|
| `tools/tui/src/play/panels/stack_view.rs` | `StackObjectKind` | UNCHANGED — modes ride outer `StackObject.modes_chosen`; SOK `ActivatedAbility` unchanged. Still run `cargo build --workspace`. |
| `tools/replay-viewer/src/view_model.rs` | `StackObjectKind` / `KeywordAbility` | UNCHANGED, same reason. Verify with workspace build (gotcha: runners miss this). |

`StackObjectKind::ActivatedAbility` (`stack.rs:584`) is **not** modified. No new enum variant is
introduced anywhere, so no new match arm is required beyond the hash arms (Change 7).

---

## Wire bumps (read digests from the FAILING gate, never hand-guess)

- **PROTOCOL_VERSION 11 → 12** (`crates/engine/src/rules/protocol.rs:123`). Forced by
  `Command::ActivateAbility.modes_chosen` (a wire frame) AND the DSL `AbilityDefinition::Activated.modes`
  entering the SR-8 transitive type closure. `tests/core/protocol_schema.rs` recomputes
  `PROTOCOL_SCHEMA_FINGERPRINT` from source and fails with the new digest — copy it from the
  failure into `protocol.rs`. Append a history row.
- **HASH_SCHEMA_VERSION 49 → 50** (`crates/engine/src/state/hash.rs:441`). Forced by runtime
  `ActivatedAbility.modes` reaching `GameState`. `tests/core/hash_schema.rs` asserts the exact
  version; the parity assertion fails and states the number. Bump the const + the parity test.
Both are machine-forced; do not guess digests. Document both bumps in the implement commit.

---

## Card Definition Fixes

### goblin_cratermaker.rs — `known_wrong` → **Complete**
**Oracle**: "{1}, Sacrifice this creature: Choose one — • Goblin Cratermaker deals 2 damage to
target creature. • Destroy target colorless nonland permanent."
**Fix**: replace `effect: Effect::Choose { … }` + flat `targets` with:
- `effect: Effect::Sequence(vec![])` (placeholder; passes `effect_choose_gate`).
- `targets: vec![]`.
- `modes: Some(ModeSelection { min_modes: 1, max_modes: 1, allow_duplicate_modes: false,
  mode_costs: None, modes: vec![ <mode0 DealDamage index 0>, <mode1 DestroyPermanent index 0> ],
  mode_targets: Some(vec![ vec![TargetCreature], vec![TargetPermanentWithFilter{ non_land: true,
  exclude_colors: Some({W,U,B,R,G}), ..Default }] ]) })`.
- **Both `DeclaredTarget` indices become `index: 0`** (LOCAL to each mode's slice). Mode 1's
  index changes from the old global `1` to local `0`.
- `source: None` on the DealDamage → keep (self-source semantics unchanged from current def).
**exclude_colors is honored** (`effects/mod.rs:8249`, enforced at cast-time target validation —
proven by `tests/rules/targeting.rs:966`; `doom_blade`/`shriekmaw`/`snuff_out` ship it). A
colorless permanent has empty `chars.colors`, so `exclude_colors: {all 5}` passes it and rejects
any colored permanent; `non_land: true` rejects lands. This is a **pure def fix — no engine
work** for the colorless filter. Set `completeness: Completeness::Complete`.

### cankerbloom.rs — `known_wrong` → **Complete**
**Oracle**: "{1}, Sacrifice this creature: Choose one — • Destroy target artifact. • Destroy
target enchantment. • Proliferate."
**Fix**: `effect: Effect::Sequence(vec![])`, `targets: vec![]`, and
`modes: Some(ModeSelection { min_modes: 1, max_modes: 1, allow_duplicate_modes: false,
mode_costs: None, modes: vec![ DestroyPermanent{DeclaredTarget index 0}, DestroyPermanent{
DeclaredTarget index 0}, Effect::Proliferate ], mode_targets: Some(vec![ vec![TargetArtifact],
vec![TargetEnchantment], vec![] ]) })`. Mode 2's empty target slice is the CR 700.2c fix:
activating Proliferate no longer requires a legal artifact+enchantment on the board.
Set `completeness: Completeness::Complete`.

### umezawas_jitte.rs — stays `known_wrong` (out of scope)
**Second blocker** (per WIP sweep): the counter-adding trigger fires only on combat-damage-**to
players**, but the oracle is "deals combat damage" (any recipient). Needs a new trigger variant —
NOT this PB. **Action**: update the marker note to cite the real surviving blocker and file
**OOS-EF7-1** ("modal-activated primitive now exists; Jitte still blocked on a
combat-damage-to-any-recipient trigger variant") in the EF findings doc. Do not touch its
abilities.

---

## Unit Tests

**File**: `crates/engine/tests/primitives/pb_ef7_modal_activated.rs` (add `mod pb_ef7_modal_activated;`
to `tests/primitives/main.rs`).
**Tests** (all cite CR; use `GameStateBuilder` + `enrich_spec_from_def`; each decoy must fail on
exactly the field under test):
- `test_602_2b_modal_activated_resolves_only_chosen_mode` — activate a 2-mode ability choosing
  mode 0; assert mode-0 effect happened. **Forward decoy**: put on the board an object that ONLY
  mode 1 could legally affect; assert it is UNTOUCHED after resolution (proves mode 1 did not
  resolve). CR 602.2b/700.2a.
- `test_700_2a_modal_activated_reverse_decoy` — same ability, choose mode 1; assert mode-1 effect
  happened and mode-0's legal target is untouched. (Reverse of the above — pins both directions.)
- `test_700_2a_invalid_mode_index_rejected` — `modes_chosen: vec![5]` on a 2-mode ability →
  `InvalidCommand` (out of range); no cost paid (mana/permanent unchanged). CR 700.2a.
- `test_700_2a_modes_chosen_on_nonmodal_rejected` — `modes_chosen` non-empty on a non-modal
  activated ability → `InvalidCommand`. CR 700.2a.
- `test_601_2b_modal_choice_survives_intervening_change` (LKI) — activate Goblin Cratermaker
  choosing mode 1 (destroy target colorless permanent); before it resolves, change the board
  (e.g. another player casts/moves something, or the chosen target's neighbors change); resolve;
  assert the chosen colorless permanent is destroyed (not mode 0's damage). Approach (a) freezes
  the mode, so this must hold. CR 400.7/601.2b.
- `test_goblin_cratermaker_mode0_deals_damage` — full activation (pay {1}, sacrifice self), mode
  0, 2 damage to a target creature; **decoy**: a second creature is undamaged.
- `test_goblin_cratermaker_mode1_destroys_colorless_only` — mode 1 destroys a colorless artifact;
  **exclude_colors decoy**: a COLORED nonland permanent is NOT a legal target (activation with it
  as target → `InvalidTarget`), and a colorless one IS. Deleting `exclude_colors` must flip this.
- `test_cankerbloom_mode0_destroys_artifact` / `_mode1_destroys_enchantment` — each mode destroys
  only its type; decoy of the other type untouched.
- `test_cankerbloom_mode2_proliferate_needs_no_target` (CR 700.2c) — activate choosing mode 2
  (proliferate) with an empty board of artifacts/enchantments; assert activation SUCCEEDS (no
  target required) and a counter is proliferated. This is the headline CR 700.2c fix vs the old
  `known_wrong` behavior that demanded artifact+enchantment.
- `test_ef7_hash_and_protocol_versions` — `assert_eq!(HASH_SCHEMA_VERSION, 50)` and
  `assert_eq!(PROTOCOL_VERSION, 12)` (sentinel convention).

**Pattern**: follow `tests/primitives/pb_ac5_alt_costs.rs` (sacrifice-cost activation) +
`tests/primitives/pb_ef1_exclude_self_enforcement.rs` (activated-ability decoy discipline) +
the Spell modal tests for mode-index validation shape.

---

## Verification Checklist

- [ ] `cargo build --workspace` (proves GameState seal + enumerates every missing-field literal)
- [ ] DSL `modes` added; all ~600–800 def literals carry `modes: None,` (or real `modes:` for the 2 flips)
- [ ] `Command::ActivateAbility.modes_chosen` added; all ~180 literals updated
- [ ] `handle_activate_ability` validates modes (CR 700.2a/d), splits target announce, bakes chosen effect, hard-rejects multi-mode+mode_targets
- [ ] resolution.rs UNCHANGED (approach a) — confirm the ActivatedAbility arm still passes
- [ ] Goblin Cratermaker + Cankerbloom flipped `Complete`; no `Effect::Choose` remains (effect_choose_gate)
- [ ] Umezawa's Jitte note rewritten + OOS-EF7-1 filed
- [ ] PROTOCOL 11→12 + HASH 49→50 from FAILING gate output; history rows appended
- [ ] `cargo test --all` (incl. `core protocol_schema`, `core hash_schema`, `core effect_choose_gate`, `core card_defs_fmt`)
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `tools/check-defs-fmt.sh` (SR-35 — `cargo fmt` checks zero defs)

## Risks & Edge Cases

- **Two corpus/test-wide mechanical surfaces** (Change 8 + Change 9). Highest compile-error risk.
  Mitigation: brace-matching script for defs (never sed `once_per_turn`), `cargo build --workspace`
  to enumerate misses. This is the dominant cost of the PB; the semantic engine change is small.
- **Wrong `once_per_turn` sed** would corrupt Triggered literals (shared field name). Use the
  Activated-scoped brace matcher only.
- **DeclaredTarget index must be LOCAL** (0) in the flipped mode effects. Copying the old global
  index (mode 1 = `index: 1`) would read the wrong / a missing slot. Pinned by the mode-1 decoy test.
- **effect_choose_gate** walks the serde tree for the `"Choose"` key; the flipped defs must use
  `Effect::Sequence(vec![])` placeholder + mode effects that are none of Choose/MayPayOrElse/AddManaChoice.
- **exclude_colors** confirmed honored — but re-run `test_goblin_cratermaker_mode1_destroys_colorless_only`
  with the field deleted to prove the decoy is non-vacuous (CLAUDE.md: several TargetFilter fields
  are silently ignored; this one is not, but prove it).
- **SR-35 line overflow**: `modes: None,` insertion is short; verify no def crosses 100 cols after
  `check-defs-fmt.sh --fix`.
- **Simulator**: `random_bot.rs` sends empty `modes_chosen` → handler auto-selects mode 0 (bots
  don't yet choose modes). `LegalActionProvider` is unaffected (no `modes_chosen` in `LegalAction`).
  Acceptable; note as a follow-up if modal-activated bot play is ever wanted.
