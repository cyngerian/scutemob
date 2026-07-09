# Primitive Batch Plan: PB-AC4 — Modal & Optional Targeting

**Generated**: 2026-07-08
**Primitive**: Per-mode `TargetRequirement` on `ModeSelection` (modal spells choose targets
only for the modes actually chosen, CR 700.2c). Plus verification/cleanup of the already-shipped
`TargetRequirement::UpToN` (PB-T) — no re-implementation.
**CR Rules**: 601.2c, 700.2 (700.2a–700.2i), 608.2b, 115.1 (verified via MCP — see below)
**Cards affected**: ~8–11 (mostly existing card-def fixes; 1–3 new-enablement candidates with caveats)
**Dependencies**: PB-T (`UpToN`), Batch 11 (`ModeSelection`, `modes_chosen`), Spree (`mode_costs`) — all present.
**Deferred items from prior PBs**: none specific to AC4 in Last Handoff; this PB is the AC4 row of the
campaign plan §2.

---

## CR Rule Text (verified via mtg-rules MCP — quote authoritative, plan brief was advisory)

**CR 601.2c** — "The player announces their choice of an appropriate object or player for each
target the spell requires. **A spell may require some targets only if … a particular mode was
chosen for it; otherwise, the spell is cast as though it did not require those targets.** … If the
spell has a variable number of targets, the player announces how many targets they will choose
before they announce those targets. … The same target can't be chosen multiple times for any one
instance of the word 'target' …"

**CR 700.2** — "A spell or ability is modal if it has two or more options in a bulleted list
preceded by instructions for a player to choose a number of those options …"
- **700.2a** — controller chooses the mode(s) as part of casting; an illegal mode (e.g. no legal
  targets) can't be chosen.
- **700.2c** — "**If a spell or ability targets one or more targets only if a particular mode is
  chosen for it, its controller will need to choose those targets only if they chose that mode.
  Otherwise, the spell or ability is treated as though it did not have those targets.**" (This is
  the exact rule AC4 implements.)
- **700.2d** — no duplicate modes unless "you may choose the same mode more than once"; a mode
  chosen multiple times is treated as appearing that many times; **each instance may have its own
  targets**.
- **700.2f** — "Modal spells and abilities may have different targeting requirements for each mode.
  Changing a spell or ability's target can't change its mode."
- **700.2g** — a copy copies the chosen modes.
- **700.2h** — per-mode additional costs (Spree; already modeled by `mode_costs`).

**CR 608.2b** — on resolution, each declared target is re-checked for legality; a target no longer
in its cast-time zone is illegal. "**If all its targets, for every instance of the word 'target',
are now illegal, the spell or ability doesn't resolve** … Illegal targets … won't be affected by
parts of a resolving spell's effect for which they're illegal. Other parts of the effect … may
still affect [legal targets]."

**CR 115.1 / 115.1a** — instants/sorceries are targeted via "target [something]"; targets chosen as
the spell is cast (601.2c). (No correction needed; the plan brief's CR numbers were all correct
this batch.)

---

## Current State of the Target System (traced chain — do NOT stop at "the enum exists")

### A. `UpToN` (PB-T) — chain trace: WORKS for spells & activated abilities; residual gap on triggered abilities

`TargetRequirement::UpToN { count, inner }` — `cards/card_definition.rs:2599–2626`.

1. **Card def → cast announcement.** `Command::CastSpell.targets: Vec<Target>` (`command.rs:59`) and
   `Command::ActivateAbility.targets` (`command.rs:115`) carry a flat announced-target list.
2. **Cast-time validation.** `casting.rs:5371 validate_targets_inner` computes the legal count range
   via `target_count_range` (`casting.rs:5354`) — `UpToN` contributes `0..=count`, mandatory reqs
   contribute `1..=1`. Assignment is a **two-pass best-fit** (mandatory slots pass 1, UpToN slots
   pass 2), correctly implementing the CR 601.2c "declaration order is not slot order" rule
   (`casting.rs:5395–5480`). Verified: this is exactly the fix recorded in `gotchas-rules.md`
   ("Multi-target validators cannot greedily match slots").
3. **StackObject.** Flat `targets` stored on the `StackObject` (`casting.rs:4170`).
4. **Resolution.** `resolution.rs:283–288` filters `stack_obj.targets` to legal ones → `legal_targets`,
   passes into `ctx.targets`. Effects read `EffectTarget::DeclaredTarget { index }`, resolved by
   `resolve_effect_target_list_indexed` (`effects/mod.rs:5866`), which indexes `ctx.targets.get(idx)`
   and returns `vec![]` for missing/nonexistent (partial-fizzle skip, CR 608.2b).
5. **Live proof.** `force_of_vigor.rs` ("destroy up to two artifacts/enchantments") uses
   `UpToN { count: 2, inner: TargetPermanentWithFilter }` with two `DestroyPermanent
   DeclaredTarget{0}/{1}` and is shipped/tested (`tests/pbt_up_to_n_targets.rs`). **UpToN is done for
   player-declared targets (spells + activated abilities).**

**Residual gap (document, OUT of AC4 scope):** *triggered*-ability `UpToN` with a permanent inner
**auto-selects 0 targets**. `abilities.rs:6964–6999`: the trigger auto-target loop routes a
player-inner `UpToN` to the player-picker but returns `None` for permanent-inner `UpToN` ("skip
optional slots"). Cards whose "up to N target" is on a *triggered* ability (kogla ETB fight,
marang/moonsnare/skyclave/endurance ETB bounce, sword_of_sinew/sword_of_light combat triggers,
carmen attack trigger, ancient_bronze_dragon, drakuseth) therefore always resolve with 0 optional
targets. That is a **legal choice** (choosing 0 is allowed) but never exercises the optional target.
Player-declared triggered-ability targeting is a broader feature (choice injection into
`flush_pending_triggers`) and is **not** part of AC4. Note in the plan; do not implement.

**Known latent bug adjacent to modal (do not rely on it):** for a *multi-target* spell where an
earlier declared target becomes illegal before resolution, `resolution.rs:283` **compacts**
`legal_targets`, shifting later `DeclaredTarget` indices. This is masked today because the modal
union-workaround always declares every slot. The AC4 design below **sidesteps** this by using
per-mode raw slices (never compacted) and per-target existence checks in `resolve_effect_target_list`.

### B. Modal per-mode targeting — chain trace: MISSING (this is the real AC4 gap → wrong game state)

`ModeSelection { min_modes, max_modes, modes: Vec<Effect>, allow_duplicate_modes,
mode_costs: Option<Vec<ManaCost>> }` — `card_definition.rs:3397`. **There is no per-mode target
field.** Modes are a bare `Vec<Effect>`; a mode's effects obtain targets by reading
`EffectTarget::DeclaredTarget { index }` into the **single flat** `Spell.targets` list (a *global*
index across the union of all modes).

- **Cast validation is union-based.** `validate_targets_inner` runs against the full flat `Spell.targets`
  requirement list. For a "choose one" charm with two mandatory target modes, `target_count_range`
  yields `min=2,max=2` → **the caster must declare targets for modes they did not choose.**
- **Resolution runs one shared ctx.** `resolution.rs:302–430`: chosen modes' effects are collected into
  `effects_to_run` and executed under a **single** `ctx` whose `ctx.targets` is the filtered flat list.

**Concrete wrong game state (why this matters):**
- `casualties_of_war.rs` — "Choose one or more —" of five "Destroy target [type]". Flat targets =
  `[Artifact, Creature, Enchantment, Land, Planeswalker]` → `min=5`. To cast choosing only "destroy
  target creature," the caster must **also** declare a target artifact, enchantment, land, AND
  planeswalker. In most board states those don't exist → **the card is uncastable** (dead card).
- `izzet_charm.rs`, `abzan_charm.rs`, `blessed_alliance.rs` — same union-declaration workaround, each
  with an explicit `TODO: per-mode target lists are not supported`.
- `cryptic_command.rs`, `incendiary_command.rs`, `archmages_charm.rs` — stubbed `Effect::Nothing`
  with `TODO: requires … per-mode targets`.

No-target modal cards (mass removal like `farewell`, `austere_command`, `merciless_eviction`; Entwine
spells; Spree spells without targets) are unaffected — they declare no targets and work today. AC4
must leave those on the existing path untouched.

---

## Chosen Design

**Add one card-definition field:** on `ModeSelection`

```
/// CR 700.2c / 700.2f: per-mode target requirements. When `Some`, `mode_targets[i]` is the
/// (fixed-length) target-requirement list for mode `i`; each mode's effects use DeclaredTarget
/// indices LOCAL to that mode's target slice. `Spell.targets` MUST be empty when this is `Some`.
/// `None` = legacy behavior (flat union targets, single shared ctx). Length == modes.len().
pub mode_targets: Option<Vec<Vec<TargetRequirement>>>,
```

**Storage model: flat announce + resolution-time per-mode slicing. No new runtime fields.**
The caster still announces a single flat `Command::CastSpell.targets` list, but ordered by
*ascending chosen-mode index*, each chosen mode's targets contiguous. Everything needed at resolution
(`stack_obj.targets` flat list + `stack_obj.modes_chosen` + the card-def `mode_targets`) already
exists on the stack object. This means **no new field on `Command::CastSpell`, no new field on
`StackObject`, no harness-schema change, no new enum variants/discriminants.**

Because each chosen mode consumes exactly `mode_targets[m].len()` announced targets (fixed count),
the per-mode slice offsets are recomputable deterministically at both cast and resolution.

**Cast time (`casting.rs`):** when the spell's `ModeSelection.mode_targets` is `Some`, build
`active_requirements = concat( mode_targets[m] for m in sorted(modes_chosen) )` and validate the
announced flat `targets` **positionally** against `active_requirements` (position `k` must satisfy
`active_requirements[k]`). Positional (not best-fit) validation is required here so the per-mode slice
offsets are unambiguous. Store the validated flat `SpellTarget`s on the stack object as today.

**Resolution (`resolution.rs`):** when `mode_targets.is_some()`, replace the "collect
`effects_to_run` + single ctx" branch with a per-mode loop over `sorted(modes_chosen)`:
for each chosen mode `m`, compute its slice `stack_obj.targets[offset .. offset + mode_targets[m].len()]`
(raw, uncompacted), set `ctx.targets = slice.to_vec()`, then run `modes[m]` under the *same* `ctx`
(reusing all other ctx setup: `x_value`, `was_overloaded`, gift, etc.). Advance `offset`. Mode effects
read `DeclaredTarget { index }` **local** to the slice (index 0 = first target of that mode).
Per-target legality/existence is enforced inside `resolve_effect_target_list` (CR 608.2b partial skip);
the pre-existing full-fizzle "all targets illegal" check on the flat list still runs earlier and is
unchanged. `PlayerTarget::DeclaredTarget { index }` resolves against the same per-mode `ctx.targets`
slice, so player-target modes (e.g. Blessed Alliance "target player gains 4 life") also become local.

### Why this design (vs. the two alternatives the brief named)

- **Rejected: `Mode { effect, targets, cost }` struct replacing `modes: Vec<Effect>`.** Cleanest data
  model, but forces migrating **every** modal card (including the ~30 no-target mass-removal / Entwine /
  Spree cards) and folds `mode_costs` into the struct (extra Spree churn). Largest blast radius and
  highest regression risk for a single PB.
- **Rejected: `mode_target_slots: Option<Vec<Vec<usize>>>` mapping modes to *global* flat slots.**
  Preserves existing global `DeclaredTarget` indices (no per-card effect-index rewrite), but requires a
  **sparse/gap** representation of unchosen slots in stored targets and in `ctx.targets` — either
  `Vec<Option<SpellTarget>>` (ripples into every `resolve_effect_target_list` / `EffectContext` site)
  or a parallel slot-map carried through ctx. It also keeps the latent compaction bug alive.
- **Chosen: `mode_targets: Option<Vec<Vec<TargetRequirement>>>` + resolution slicing.** Parallels the
  existing `mode_costs: Option<Vec<ManaCost>>` shape (familiar to reviewers); adds **zero** new runtime
  storage/enums; reuses `resolve_effect_target_list` unchanged (dense per-mode `ctx.targets`); truly
  isolates each mode's targets (CR 700.2c/700.2f, and 700.2d duplicate-mode instances get independent
  slices for free); sidesteps the compaction hazard. Migration is bounded to only the cards that
  actually have per-mode targets (~6–8), each a mechanical requirement-relocation + local-index rewrite.

### Scope boundary (fixed-count) and the one variable-count residual

AC4 supports **fixed-count** per-mode target lists: `mode_targets[m].len()` is the exact number of
targets mode `m` consumes. **Author invariant:** a `mode_targets[m]` entry must NOT itself contain
`UpToN` (that would make the slice length variable and the flat split ambiguous). Enforce/validate
this and document it on the field.

`blessed_alliance.rs` mode 1 ("Untap **up to two** target creatures") is the single variable-count
case. AC4 handles Blessed Alliance's modes 0 and 2 (fixed) but mode 1's "up to two" cannot be
expressed under the fixed-count rule. **Keep Blessed Alliance's mode 1 as its current two-required
approximation (documented residual), or model mode 1 as two fixed optional-less targets** — do NOT
extend the design for it in AC4. General UpToN-inside-a-mode requires a per-mode declared-count field
(a future micro-PB); per `conventions.md` "implement-phase default-to-defer," flag it, do not build it.

---

## Engine Changes (ordered)

### Change 1 — Add `mode_targets` to `ModeSelection`
**File**: `crates/engine/src/cards/card_definition.rs` (`ModeSelection`, ~line 3413, after `mode_costs`)
**Action**: add `#[serde(default)] pub mode_targets: Option<Vec<Vec<TargetRequirement>>>,` with the
doc comment above. `#[serde(default)]` keeps existing JSON/data back-compatible.
**Follow**: the existing `mode_costs` field for shape and doc style.

### Change 2 — Hash the new field + bump schema version
**File**: `crates/engine/src/state/hash.rs`
**Action**:
1. In `impl HashInto for ModeSelection` (line 5248) add, after the `mode_costs` block, an
   `Option<Vec<Vec<TargetRequirement>>>` hash (mirror the `mode_costs` Some/None + nested-loop
   pattern; `TargetRequirement` already implements `HashInto`).
2. Bump `pub const HASH_SCHEMA_VERSION: u8` from **30 → 31** (line 230) and add a changelog comment
   ("31: ModeSelection.mode_targets (PB-AC4 per-mode targeting)").
**Sentinel updates (per `conventions.md` hash-sentinel convention)**: update every
`assert_eq!(HASH_SCHEMA_VERSION, 30)` parity assertion to `31`. Find them:
`Grep pattern="HASH_SCHEMA_VERSION, 30" path="crates/engine"` (and the re-export in `lib.rs` needs no
value change, just confirm it compiles).

### Change 3 — Cast-time per-mode target validation
**File**: `crates/engine/src/rules/casting.rs`
**Action**: at the point where `requirements`/`spell_targets` are computed (~line 3390–3397) and/or in
`validate_targets_inner`, add a branch: if the casting spell's `ModeSelection.mode_targets` is `Some`,
compute `active_requirements = concat(mode_targets[m] for m in sorted validated_modes_chosen)` and
validate the announced flat `targets` **positionally** against it (position k satisfies
`active_requirements[k]`; count must equal `active_requirements.len()`). This replaces the flat-union
`Spell.targets` validation for these spells. Also assert `Spell.targets` is empty when `mode_targets`
is `Some` (author invariant) and reject any `UpToN` appearing inside a `mode_targets[m]` entry.
**CR**: 601.2c + 700.2c (targets only for chosen modes); 700.2a (illegal-mode gating already at
`casting.rs:4084–4131`).
**Note**: `validated_modes_chosen` is already sorted ascending at `casting.rs:4128`, so slice offsets
match execution order.

### Change 4 — Resolution per-mode ctx slicing
**File**: `crates/engine/src/rules/resolution.rs` (spell-modal branch, ~line 295–430)
**Action**: when the resolving spell's `ModeSelection.mode_targets` is `Some`, take the per-mode path:
iterate `stack_obj.modes_chosen` (ascending), maintain a running `offset`, and for each mode `m` set
`ctx.targets = stack_obj.targets[offset .. offset + mode_targets[m].len()].to_vec()` (raw slice, not
`legal_targets`), execute `modes.modes[m]` under the shared `ctx`, then advance `offset`. Leave the
existing Entwine / Escalate / explicit-`modes_chosen` / auto-mode[0] branches untouched for the
`mode_targets == None` case. Splice/Ascend tail logic below is unchanged.
**CR**: 700.2c/700.2f (per-mode targets), 700.2d (duplicate-mode instances slice independently),
608.2b (per-target skip handled by `resolve_effect_target_list`).

### Change 5 — Exhaustive-match & construction-site audit (expected: NONE required)
This design adds **no new enum variant** (no `StackObjectKind`, `KeywordAbility`,
`AbilityDefinition`, `Effect`, `TargetRequirement` variant) and **no new struct field on
`Command`/`StackObject`**. Therefore:

| File | Match / construction | Expected action |
|------|----------------------|-----------------|
| `tools/tui/src/play/panels/stack_view.rs` | `StackObjectKind` exhaustive match | **No new arm** (no new variant) |
| `tools/replay-viewer/src/view_model.rs` | `StackObjectKind` + `KeywordAbility` matches | **No new arm** |
| `state/stack.rs` `trigger_default`, `casting.rs`, `copy.rs`, `engine.rs` | `StackObject { … }` literals | **No change** (no new field) |
| `testing/replay_harness.rs` (~40 `Command::CastSpell` literals) | Command construction | **No change** (no new field) |
| `rules/copy.rs:215` | copies `modes_chosen` | **No change** (copy already carries flat `targets` + `modes_chosen`, CR 700.2g) |

**Mandatory verification anyway**: the runner MUST run `cargo build --workspace` after the impl phase
(memory note: runners miss view_model.rs ~50% of the time). If a `ModeSelection { … }` literal exists
without `..` anywhere (grep the card defs — most use struct-literal without `..Default`), each such
literal needs `mode_targets: None,` added. Find them:
`Grep pattern="ModeSelection \{" path="crates/engine/src/cards/defs"` — every modal card def constructs
`ModeSelection` with an explicit field list, so **each of the ~40 modal card defs needs
`mode_targets: None,` added unless it is being migrated to `Some`.** This is the primary mechanical
churn of AC4; `cargo check` catches any miss immediately.

---

## Card Definition Fixes (honest roster — MCP oracle authoritative; discounted per feedback_pb_yield_calibration)

### Pre-existing TODO sweep (roster-recall gate)
Ran `Grep` for `per-mode|mode-scoped|700.2c|up to.*target|TODO.*modal|ENGINE-BLOCKED` across
`cards/defs/`. **Forced adds** (source self-identifies as needing per-mode targeting), verified against
oracle text below. TODO sweep result: the four "per-mode target lists are not supported" TODOs
(izzet/abzan/blessed_alliance/casualties) and three stubbed `Effect::Nothing` modal TODOs
(cryptic/incendiary/archmages) are all forced candidates.

| Card | Oracle pattern | Unblocked by AC4? | Notes |
|------|----------------|-------------------|-------|
| `izzet_charm` | Choose one; 2 target-modes + 1 no-target | **YES** (fixed-count) | migrate flat→`mode_targets`, local indices |
| `casualties_of_war` | Choose one-or-more of 5 single-target destroys | **YES** — strongest fix | currently *uncastable* without all 5 target types present |
| `cryptic_command` | Choose two of 4 (counter / bounce / tap-all / draw) | **YES (new)** — all sub-effects exist | replace `Effect::Nothing` stub |
| `abzan_charm` | Choose one of 3 | **PARTIAL** | modes 0 & 2 single-target unblocked; mode 2 "distribute among one or two" counter-split still approximated (separate primitive) |
| `blessed_alliance` | Choose one-or-more of 3 (Escalate) | **PARTIAL** | modes 0 & 2 unblocked; mode 1 "untap up to two" = variable-count residual (keep approx / defer) |
| `boros_charm`, `golgari_charm`, `rakdos_charm`, `evolution_charm`, `archdruids_charm`, `collective_resistance` | Charm "choose one" families | **VERIFY each** | migrate to `mode_targets` only where a mode actually targets; some modes are targetless. Runner checks oracle per card. |
| `bridgeworks_battle` | "…fights **up to one** target creature you don't control" | **YES (UpToN cleanup, card-only)** | STALE TODO; migrate mandatory 2nd target → `UpToN{1}`. No engine change. |
| `skullsnatcher` | "exile **up to two** target cards …" (triggered) | **YES (UpToN cleanup, card-only)** | STALE TODO; migrate two-required → `UpToN{2}`. NB: triggered → auto-targets 0 (document); still removes wrong-game-state approximation |
| `incendiary_command` | Choose two of 4; mode 4 = wheel (discard hand, draw that many) | **CANDIDATE — likely partial** | modes 1/2/3 expressible; mode 4 wheel effect may be a separate gap. Runner verifies; if blocked, leave stub + note. |
| `archmages_charm` | Choose one of 3; mode 3 = gain control of nonland perm MV≤1 | **CANDIDATE — likely blocked** | needs a gain-control effect + MV filter; per-mode targeting alone insufficient. Runner verifies; if blocked, leave stub + note. |

For each **migrated** card: move the mode's requirement(s) from the flat `Spell.targets` into
`ModeSelection.mode_targets[m]`, set `Spell.targets: vec![]`, and rewrite that mode's effect
`DeclaredTarget`/`PlayerTarget::DeclaredTarget` indices to be **local** (0-based within the mode).
Set `mode_targets: None` on every non-migrated modal card def (compile requirement).

## New Card Definitions
Only `cryptic_command` is a confident new enablement (its stub becomes a real 4-mode / choose-two
definition). `incendiary_command` and `archmages_charm` are candidates gated on secondary primitives
(wheel effect; gain-control) — author only if the runner confirms those effects exist; otherwise leave
the stub with an updated TODO naming the true blocker.

---

## Unit Tests

**File**: `crates/engine/tests/modal.rs` (extend) and/or new `crates/engine/tests/pb_ac4_per_mode_targeting.rs`.
Pattern-follow: existing `tests/modal.rs`, `tests/pbt_up_to_n_targets.rs`, and the `cast_spell_modal`
harness action (`replay_harness.rs:1547`).

- `test_601_2c_modal_targets_only_for_chosen_mode` — cast Casualties of War choosing only "destroy
  target creature" while NO artifact/enchantment/land/planeswalker exists; cast succeeds declaring a
  single creature target; only that creature is destroyed. **CR 601.2c / 700.2c.**
- `test_700_2c_unchosen_mode_targets_not_required` — Izzet Charm cast choosing the "2 damage to
  target creature" mode with no spell on the stack; succeeds (no noncreature-spell target demanded).
  **CR 700.2c.**
- `test_700_2a_illegal_mode_still_castable_via_other_mode` — Izzet Charm: counter-mode illegal (no
  spell to counter) but damage-mode legal → damage mode is castable. **CR 700.2a.**
- `test_601_2c_wrong_type_target_rejected_per_mode` — Casualties "destroy target creature" mode with a
  land declared as its target → rejected (positional per-mode type check). **CR 601.2c.**
- `test_700_2f_two_modes_two_targets_sliced_independently` — Cryptic Command choose two (counter +
  bounce): each mode affects its own declared target; targets do not cross-contaminate. **CR 700.2f.**
- `test_608_2b_modal_partial_illegal_target_skips_only_that_mode` — Casualties choose creature+land;
  the creature target leaves the battlefield in response; on resolution the land is still destroyed,
  the creature mode is skipped (not full fizzle). **CR 608.2b.**
- `test_608_2b_modal_all_targets_illegal_fizzles` — single-target-mode spell whose only declared
  target becomes illegal → spell removed from stack, no effect. **CR 608.2b.**
- `test_700_2c_multiplayer_casualties_choose_subset` — 4-player: Casualties choosing a subset of modes
  targeting different opponents' permanents resolves correctly. **CR 700.2c** (multiplayer coverage).
- `test_pbt_up_to_n_regression_still_passes` — confirm `force_of_vigor` UpToN path unaffected by the
  cast/resolution changes (guard against regressing the `mode_targets == None` path). **CR 601.2c.**

Each test must use `GameStateBuilder` + the public command/harness API and cite its CR in a doc
comment (per `conventions.md`).

---

## Verification Checklist
- [ ] `mode_targets` added to `ModeSelection`; `cargo check` clean
- [ ] `HASH_SCHEMA_VERSION` bumped 30→31; all `assert_eq!(HASH_SCHEMA_VERSION, …)` sentinels updated
- [ ] `ModeSelection` HashInto hashes `mode_targets`
- [ ] Every modal card def has `mode_targets: None` (or `Some` if migrated) — `cargo build` clean
- [ ] Cast-time positional per-mode validation + author-invariant checks (empty flat targets; no
      nested UpToN in a mode)
- [ ] Resolution per-mode ctx slicing for `mode_targets.is_some()`; legacy path unchanged
- [ ] Migrated cards use local `DeclaredTarget` indices; flat `Spell.targets` emptied
- [ ] `cargo build --workspace` (TUI + replay-viewer compile — verify even though no new variants)
- [ ] `cargo test --all` green (new tests + full suite)
- [ ] `cargo clippy -- -D warnings` clean; `cargo fmt --check`
- [ ] No remaining "per-mode target lists are not supported" TODOs in migrated cards
- [ ] Residual gaps documented (triggered-UpToN auto-0; Blessed Alliance mode-1 variable count;
      incendiary wheel / archmages gain-control if left stubbed)

---

## Risks & Edge Cases
- **Primary churn = adding `mode_targets: None` to ~40 modal `ModeSelection` literals.** Purely
  mechanical; `cargo check` flags any miss. This is the #1 compile-error source for AC4.
- **Positional vs best-fit validation.** The per-mode path MUST validate positionally (slice offsets
  depend on order). Do not reuse the flat two-pass best-fit for `mode_targets` spells — it could
  reorder assignments and break slicing.
- **Sorted-mode order.** Slice offsets assume `modes_chosen` is ascending (it is, post
  `casting.rs:4128`). If any future path sets `modes_chosen` unsorted, slicing breaks — assert sorted.
- **Duplicate modes (CR 700.2d).** With `allow_duplicate_modes`, a repeated mode index appears twice in
  `modes_chosen`; the slice logic gives each instance its own contiguous target slice automatically —
  add a test if any AC4 card uses duplicate modes (none currently do).
- **Copy (CR 700.2g).** Copies already clone flat `targets` + `modes_chosen` (`copy.rs:215`); since
  per-mode slicing is derived at resolution, copies inherit correct per-mode targets with no extra work.
- **Latent compaction bug** (`resolution.rs:283`) is avoided by the per-mode path (raw slices), but it
  still exists for the legacy flat path — out of AC4 scope; do not "fix while here" (default-to-defer).
- **Blessed Alliance mode 1 / Abzan mode 2** are variable-count / distribution cases the fixed-count
  design intentionally does not cover — keep approximations, document, do not extend the primitive.
- **incendiary_command / archmages_charm** may remain blocked on secondary primitives (wheel effect;
  gain-control). If so, they are NOT AC4 yield — update their TODOs to name the true blocker rather
  than claiming per-mode targeting is missing.
- **Discriminant chain**: AC4 allocates **no** new `KeywordAbility` / `AbilityDefinition` /
  `StackObjectKind` / `Effect` / `TargetRequirement` variant (verified: design is field-only), so the
  discriminant chain end is unchanged — no discriminant bookkeeping needed.
