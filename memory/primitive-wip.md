# Primitive WIP: PB-Q — ChooseColor

batch: PB-Q
title: ChooseColor (as-ETB color choice + color-aware downstream effects)
cards_unblocked: 6 in-scope (Caged Sun, Gauntlet of Power, Throne of Eldraine, Temple of the Dragon Queen, Utopia Sprawl, Painter's Servant Tier-1-verify) + 3+ deferred to PB-Q2 (activated-time)
started: 2026-04-11
phase: review
plan_file: memory/primitives/pb-plan-Q.md

## Plan Phase Summary (2026-04-11)

Plan written to `memory/primitives/pb-plan-Q.md`. Key findings:

- **One primitive shipped**: as-ETB ChooseColor as a `ReplacementModification` (CR 614.12), exact parallel of existing `ChooseCreatureType` at `replacement.rs:1440`. Plus dispatch surface for the in-scope cards: 2 new `EffectFilter` variants (`CreaturesYouControlOfChosenColor`, `AllCreaturesOfChosenColor`) and a new mana-production replacement modification (`AddOneManaOfChosenColor`).
- **NO new `LayerModification` variant** in core PB-Q. (Painter's Servant might need one — Tier 1 verify, defer if confirmed.)
- **Coordinator scope notes had two factual errors** (verified via MCP):
  - Cavern of Souls is choose-creature-type, NOT choose-color. Removed from scope.
  - Utopia Sprawl is choose-color, NOT choose-basic-land-type. Stays in scope (not bundled — it IS this primitive).
- **Activated-time choose-color is a different primitive** — Skrelv, Nykthos, Three Tree City, Throne's draw activation deferred to PB-Q2.
- **Painter's Servant flagged Tier 1**: its color-add scope ("nearly every card in nearly every zone") may need a new Layer 5 color-add modification — defer if true.

## Standing Rules (MUST follow — NON-NEGOTIABLE)

(unchanged from plan-phase brief)

1. **"As ~ enters the battlefield, choose X" is a REPLACEMENT effect per CR 614.12, NOT a triggered ability.** PB-X C1 lesson. Templates: Urza's Incubator, Vanquisher's Banner, Morophon, Cavern of Souls, Patchwork Banner, Obelisk of Urd (post-PB-X).
2. **Full-dispatch tests for every new `LayerModification` variant.** PB-Q does not introduce one; the rule still binds the dispatch tests for the new EffectFilter variants and the mana doubling path. Plan tests 5 and 8 are mandatory.
3. **Full-chain verification.** Walk effect → filter → layer → cost/trigger.
4. **CR verification before implement.**

## Implement Phase Summary (2026-04-11)

### Engine Changes Completed

- [x] **`GameObject.chosen_color: Option<Color>`** field added (`state/game_object.rs`) — `#[serde(default)]`, doc citing CR 614.12 / CR 105.1 / CR 400.7
- [x] **`ReplacementModification::ChooseColor(Color)`** — ETB replacement fires deterministic fallback (scan controller permanents by color count; tie-break prefers default_color then highest discriminant) (`state/replacement_effect.rs`)
- [x] **`ReplacementModification::AddOneManaOfChosenColor`** — mana-production replacement modification (`state/replacement_effect.rs`)
- [x] **`ChosenColorRef { SelfChosen, Fixed(Color) }`** — enum for color reference in `ManaWouldBeProduced` (`state/replacement_effect.rs`)
- [x] **`ReplacementManaSourceFilter { Any, BasicLand, AnyLand, EnchantedLand }`** — scopes which tapped permanents trigger replacement (`state/replacement_effect.rs`). Named to avoid conflict with `ManaSourceFilter` in `card_definition.rs`.
- [x] **`ManaWouldBeProduced` extended** with `color_filter: Option<ChosenColorRef>` + `source_filter: Option<ReplacementManaSourceFilter>` fields
- [x] **`EffectFilter::CreaturesYouControlOfChosenColor`** + **`EffectFilter::AllCreaturesOfChosenColor`** — layer filter variants (`state/continuous_effect.rs`)
- [x] **`Effect::AddManaOfChosenColor { player: PlayerTarget, amount: u32 }`** — new effect variant (`cards/card_definition.rs`)
- [x] **`apply_mana_production_replacements` refactored** — new signature `(state, player, source_perm, base_mana) -> (u32, Vec<(ManaColor, u32)>)` handles both MultiplyMana and AddOneManaOfChosenColor paths (`rules/mana.rs`)
- [x] **`effects/mod.rs`** — dispatch for `Effect::AddManaOfChosenColor`; reads `ctx.source.chosen_color`, maps Color→ManaColor
- [x] **`is_mana_producing_effect`** — added `AddManaOfChosenColor` arm so triggered mana ability detection works for Utopia Sprawl
- [x] **`rules/layers.rs`** — two dispatch arms in `is_effect_active` for new EffectFilter variants; read `source.chosen_color` dynamically
- [x] **`rules/replacement.rs`** — `ChooseColor` arm in `emit_etb_modification`; deterministic fallback with tie-break
- [x] **Hash schema sentinel bumped 2→3** with all new fields/variants hashed (`state/hash.rs`)
- [x] **`chosen_color` added to `HashInto for GameObject`** (PB-S H1 discipline)
- [x] **`helpers.rs`** — `ChosenColorRef` + `ReplacementManaSourceFilter` re-exported
- [x] **`mana_reflection.rs` + `nyxbloom_ancient.rs`** — added `color_filter: None, source_filter: None` to old `ManaWouldBeProduced` struct literals
- [x] **19 `chosen_color: None` entries** added to all `GameObject` constructors (state/mod.rs, state/builder.rs, rules/resolution.rs)

### Card Definitions Completed

- [x] **`caged_sun.rs`** (NEW) — ETB ChooseColor(White) + CreaturesYouControlOfChosenColor +1/+1 + ManaWouldBeProduced { SelfChosen, AnyLand } replacement
- [x] **`gauntlet_of_power.rs`** (NEW) — ETB ChooseColor(White) + AllCreaturesOfChosenColor +1/+1 + ManaWouldBeProduced { SelfChosen, BasicLand } replacement
- [x] **`throne_of_eldraine.rs`** (PATCHED) — ETB ChooseColor(White) + {T}: AddManaOfChosenColor{4} + {3}{T}: DrawCards{2}
- [x] **`temple_of_the_dragon_queen.rs`** (PATCHED) — ETB ChooseColor(White) + {T}: AddManaOfChosenColor{1} (added to existing EntersTapped replacement)
- [x] **`utopia_sprawl.rs`** (NEW) — Aura. ChooseColor(Green) ETB + WhenTappedForMana{EnchantedLand} triggered → AddManaOfChosenColor{1}

### Tests Written (primitive_pb_q.rs — 11 tests)

- [x] `test_choose_color_replacement_sets_field` — ETB sets chosen_color (Component 2)
- [x] `test_choose_color_deterministic_fallback_picks_majority` — majority wins (Component 2)
- [x] `test_choose_color_default_when_no_permanents` — empty board uses default (Component 2)
- [x] `test_choose_color_resets_on_zone_change` — CR 400.7: fresh object has None (Component 2)
- [x] `test_caged_sun_full_dispatch_pumps_chosen_color_creatures` — **MANDATORY** full-dispatch (Component 4/5)
- [x] `test_gauntlet_of_power_pumps_all_controllers_chosen_color` — AllCreaturesOfChosenColor (Component 4)
- [x] `test_chosen_color_filter_no_choice_matches_nothing` — None source → no pump (Component 4)
- [x] `test_caged_sun_doubles_chosen_color_land_mana` — **MANDATORY** full-dispatch mana doubling (Component 5)
- [x] `test_caged_sun_does_not_double_other_color_mana` — color filter discrimination (Component 5)
- [x] `test_gauntlet_of_power_only_doubles_basic_lands` — BasicLand vs AnyLand source filter (Component 5)
- [x] `test_chosen_color_hash_field_audit` — PB-S H1 hash defense

### Verification Gates (all PASS)

- [x] `cargo test --all` — ALL PASS (2616 + 11 = 2627 tests)
- [x] `cargo clippy --workspace -- -D warnings` — CLEAN
- [x] `cargo build --workspace` — CLEAN (replay-viewer + TUI compile)
- [x] `cargo fmt --check` — CLEAN

### Open Questions Resolved at Implement Time

- **Q4 verification gate**: Mana Reflection exists (`mana_reflection.rs`) using `ManaWouldBeProduced { controller }` + `MultiplyMana` — Alt A matched exactly.
- **Q7 Utopia Sprawl ETB ordering**: Verified `resolution.rs` — Aura attachment (~line 1547) before ETB replacements (~line 1571). `chosen_color` set atomically with ETB. No STOP-AND-FLAG needed.
- **Tied-color tie-break**: Added deterministic tie-break logic — default_color preferred when tied for max; otherwise highest Color discriminant. Prevents non-determinism from HashMap iteration.

### Deferred Items

- **Painter's Servant**: Deferred (Q2). Scope unclear; may need Layer 5 color-add modification.
- **PB-Q2**: Activated-time choose-color (Skrelv, Nykthos, Three Tree City, Throne's draw activation). Reserve in `docs/primitive-card-plan.md` (Q10 — not yet done, pending oversight).
- **`mana_reflection.rs` TODO comment**: Still references "spending restrictions" — existing behavior correct, comment cleanup deferred.

## Hazards / Carry-forward

- Cavern of Souls is creature-type-choice (oversight WIP error — corrected in plan)
- Utopia Sprawl IS choose-color (not basic-land-type — coordinator note corrected via MCP)
- Painter's Servant deferred
- PB-Q2 scope: Skrelv / Nykthos / Three Tree City / Throne's draw activation
- PB-S H1 hash field-count discipline — DONE for chosen_color

## Next Action

Oversight: trigger `primitive-impl-reviewer` for PB-Q review phase.
