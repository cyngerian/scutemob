# Primitive Batch Plan: PB-OS7 — Defending-player-scoped continuous EffectFilter

**Generated**: 2026-07-19
**Primitive**: A DSL placeholder `EffectFilter::CreaturesControlledByDefendingPlayer` that the
`Effect::ApplyContinuousEffect` handler stamps at effect-creation time into the pre-existing
locked filter `EffectFilter::CreaturesControlledBy(<captured defending player>)`. Lets a
*continuous* effect scope its affected set to "creatures the defending player of this attack
controls," per-attacker, without the layer system needing to read the resolving `EffectContext`.
**Finding**: OOS-EF3-1 (filed by PB-EF3, `ef-batch-plan-2026-07-17.md` §6).
**CR Rules**: CR 508.4 (defending player), CR 611.2a/611.3 (continuous effect from a resolving
ability; affected set fixed at resolution), CR 613.1d/613.4c (Layer 7c P/T modification),
CR 514.2 (until-end-of-turn cleanup expiry), CR 704.5f (0-toughness SBA), CR 205.3m (Dragon
subtype). Ruling: Silumgar 2014-11-24 (affected set is relative to the specific attacking Dragon).
**Cards affected**: 1 (0 existing fixes + 1 new `Complete`). Karazikar honestly BLOCKED (remainder
named below, filed OOS-OS7-1).
**Dependencies**: PB-EF3 (`PlayerTarget::DefendingPlayer`, per-attacker `defending_player_id`
capture — SHIPPED), PB-N (`TriggerCondition::WheneverCreatureYouControlAttacks { filter }` — the
per-attacker subtype-filtered attack trigger — SHIPPED), PB-OS1/existing EOT cleanup machinery
(`expire_end_of_turn_effects` — SHIPPED).
**Deferred items from prior PBs**: none carried into this PB. This PB does NOT close the
target-side/declared-target-player siblings (kogla, karazikar, polymorphists_jest,
great_oak_guardian) — see "Adjacency notes & new seeds."

---

## Chosen wire shape (decision + justification)

**Stored/locked filter**: the pre-existing `EffectFilter::CreaturesControlledBy(PlayerId)`
(continuous_effect.rs:101). It is ALREADY handled by the layer system
(`rules/layers.rs:646`, `filter_matches`) and ALREADY hashed (`state/hash.rs:2077`,
discriminant 9). No change to its handling. The stored `ContinuousEffect` after substitution
carries a concrete `CreaturesControlledBy(defending_pid)` — this is the brief's preferred
"general locked filter that the builder stamps with the captured defending player."

**DSL placeholder**: a NEW unit variant `EffectFilter::CreaturesControlledByDefendingPlayer`.
Card defs write this placeholder; `Effect::ApplyContinuousEffect` substitutes it into
`CreaturesControlledBy(ctx.defending_player)` at execution. It never appears in a stored
`ContinuousEffect`. This is the exact precedent set by `AllCreaturesExcludingChosenSubtype`
(continuous_effect.rs:248-258, substituted at effects/mod.rs:3170-3175) and by
`Source`/`TriggeringCreature` (unit placeholders substituted in the same match).

**Rejected alternative (Option A — overload `CreaturesControlledBy(PlayerId(0))` as a
defending-player sentinel)**: rejected because the codebase's universal `PlayerId(0)`-placeholder
convention binds to the RESOLVING CONTROLLER ("you") everywhere else
(`WhileYouControlSource`, `UntilYourNextTurn`, `ObjectFilter::ControlledBy`/`CreatureControlledBy`
all bind `PlayerId(0) -> ctx.controller`). Re-using `PlayerId(0)` to instead mean the DEFENDING
player would be a silent, surprising inconsistency (a documented footgun class per
`memory/gotchas-rules.md` and the "aspirationally-wrong" discipline in `memory/conventions.md`).
For a -1/-1 continuous effect, a mis-bind to the controller would debuff your OWN creatures — an
actively-wrong game state, not a harmless degenerate. A self-documenting placeholder variant makes
the "defending player" intent unmistakable at the card-def site.

### PROTOCOL / HASH impact (verify empirically; do not assume)

- **PROTOCOL: NOT forced. No `PROTOCOL_VERSION` bump, no `PROTOCOL_HISTORY` row, no fingerprint
  re-pin.** `EffectFilter` is OFF the SR-8 wire closure — it lives inside `GameState.continuous_effects`,
  not `Command`/`GameEvent`/`ReplayLog`. This is stated verbatim in the PB-EF4 history note at
  `rules/protocol.rs:109-112`: PB-EF4 added `EffectFilter::TriggeringCreature` and the
  `PROTOCOL_SCHEMA_FINGERPRINT` gate did **not** move for it (the v8→9 bump that PB shipped was
  driven by the `Effect::DealDamage.source` reshape, a *different* closure type). **The brief's
  anticipated "PROTOCOL 21→22" is therefore incorrect for this primitive** — reported honestly.
  **Runner MUST confirm**: after adding the variant, run the protocol gate
  (`cargo test -p mtg-engine protocol` / the fingerprint parity test). It must stay green at
  version 21 with the same fingerprint. If it unexpectedly moves, STOP AND FLAG (do not silently
  bump) — that would mean the closure roots changed and needs re-analysis.

- **HASH: machine-forced, 58 → 59.** `EffectFilter` IS in the `GameState` serde/hash closure (via
  `continuous_effects -> ContinuousEffect.filter`). Adding a variant moves the machine-computed
  `decl_fingerprint` in `HASH_SCHEMA_HISTORY`, which is a hard gate. Per the `memory/conventions.md`
  hash-bump rule ("bump on every change to a serialized type's variant shape; default action:
  bump"), and confirmed by the PB-EF4 precedent (HASH 46→47 for exactly this kind of change), the
  bump is required.

---

## CR Rule Text (authoritative excerpts)

- **CR 508.4**: "A permanent is a legal attacker for a player if that player is the defending
  player." / defending player = the player being attacked. (Per-attack; each attacking creature
  has its own attack target — a player or a planeswalker whose controller is the defending player.)
- **CR 611.2a**: "The continuous effects of a spell or ability that isn't a static ability … are
  locked in when that spell or ability resolves … the set of objects it affects is determined at
  that time." (For Silumgar: the "creatures defending player controls" set is fixed at trigger
  resolution to whoever controls creatures at that moment — but see the note below; we bind the
  PLAYER at resolution and let the layer filter re-evaluate membership live, which is correct
  because "-1/-1 until end of turn" from a permanent's triggered ability affects the set that
  matches the description at each moment — CR 611.2c only fixes the *player*, not the membership.)
- **CR 514.2**: "…all 'until end of turn' … effects end." (Cleanup.)
- **CR 704.5f**: "…a creature with toughness 0 or less is put into its owner's graveyard."
- **Silumgar ruling 2014-11-24**: "The creatures affected … are determined relative to the Dragon
  that attacked. For example, if Silumgar attacked one opponent and two other Dragons attacked
  another opponent, creatures controlled by the first opponent would get -1/-1 and creatures
  controlled by the second opponent would get -2/-2 until end of turn."

**Membership semantics (design note for the runner) — CORRECTED post-review (PB-OS7 review, Finding
1)**: We store `CreaturesControlledBy(pid)` where `pid` is locked at resolution, but the
*membership* (which creatures that player controls) is evaluated live by `filter_matches` at each
`calculate_characteristics`. **This is NOT CR-correct.** CR 611.2c: "If a continuous effect
generated by the resolution of a spell or ability modifies the characteristics ... of any objects,
the set of objects it affects is determined when that continuous effect begins. After that point,
the set won't change." Silumgar's -1/-1 is a resolution-generated Layer 7c effect, so per 611.2c
the affected *set of creatures* must be locked at resolution, not just the player. Live
re-evaluation means a creature the defending player gains control of later in the turn is wrongly
swept into the debuff, and a debuffed creature that changes controller away from the defending
player wrongly loses the -1/-1 (CR 611.2c says it should keep it). **This is a known, pre-existing,
engine-wide simplification** — every resolution-generated mass P/T effect in the corpus has the
same divergence (e.g. `golgari_charm.rs` via `AllCreatures`, `eyeblight_massacre.rs` via
`AllCreaturesExcludingSubtype`), so PB-OS7 correctly follows existing precedent rather than
introducing a new bug. A real fix requires an engine-wide resolution-time affected-set snapshot
mechanism (out of scope for this PB) — tracked as **OOS-OS7-2** in
`memory/primitives/oos-retriage-plan-2026-07-18.md`. Only the PLAYER identity is intentionally
locked at resolution here; that is exactly what the per-attacker `defending_player_id` capture
supplies — do not confuse that with the (absent) set-lock.

---

## Engine Changes

### Change 1 — Add the placeholder variant

**File**: `crates/card-types/src/state/continuous_effect.rs`
**Action**: Add a unit variant to `enum EffectFilter` (place it immediately after
`TriggeringCreature`, ~line 141, or after `CreaturesControlledBy` at ~line 101 — keep it adjacent
to the locked variant it substitutes into). Doc-comment it as a DSL-only placeholder that never
appears in a stored `ContinuousEffect`, substituted at `Effect::ApplyContinuousEffect` execution
into `CreaturesControlledBy(ctx.defending_player)`; if `defending_player` is `None` the effect is
skipped (applies to nothing) — never falls back to the controller (would wrongly debuff own
creatures). Cite CR 508.4 / 611.2a. Model the doc comment on `AllCreaturesExcludingChosenSubtype`.

```rust
/// DSL placeholder: "creatures the DEFENDING player controls." Substituted at
/// `Effect::ApplyContinuousEffect` execution time into
/// `CreaturesControlledBy(ctx.defending_player)` using the per-attacker defending
/// player captured by the attack trigger (CR 508.4 / 611.2a). If no defending player
/// was captured (`ctx.defending_player == None`), the effect is skipped entirely —
/// it must NEVER fall back to the controller (that would debuff the caster's own
/// creatures). Never appears in a stored `ContinuousEffect`; the layer arm returns
/// `false` as an unreached guard (mirrors `Source` / `TriggeringCreature`).
CreaturesControlledByDefendingPlayer,
```

### Change 2 — Substitute the placeholder at resolution (the "builder stamp")

**File**: `crates/engine/src/effects/mod.rs`
**Action**: In the `Effect::ApplyContinuousEffect` handler's `resolved_filter` match (currently
lines 3144-3177), add an arm BEFORE the `other => other.clone()` catch-all (this is a behavior
add; the wildcard means it is not a compile requirement, but it is functionally load-bearing):

```rust
// CR 508.4 / 611.2a (PB-OS7, OOS-EF3-1): stamp the captured defending player into
// the locked filter at effect creation. The layer system cannot read ctx, so the
// player must be baked in now. None => apply to nothing (never fall back to
// ctx.controller — that would debuff the caster's own creatures).
CEFilter::CreaturesControlledByDefendingPlayer => match ctx.defending_player {
    Some(pid) => CEFilter::CreaturesControlledBy(pid),
    None => return,
},
```

**CR**: 508.4 / 611.2a. `ctx.defending_player: Option<PlayerId>` is already populated for attack
triggers (threaded PendingTrigger -> StackObject -> EffectContext; see `rules/abilities.rs:4090-4113`
and the existing reads at effects/mod.rs:3865/3927/6886). Pattern mirrors the `TriggeringCreature`
arm (effects/mod.rs:3162-3165: `None => return`).

### Change 3 — Layer-system unreached guard (EXHAUSTIVE match — compile-forced)

**File**: `crates/engine/src/rules/layers.rs`
**Action**: In `filter_matches` (the `match &effect.filter` starting ~line 611; it is exhaustive,
no wildcard), add an arm next to the `Source`/`TriggeringCreature`/`DeclaredTarget => false`
guards (~line 657-664):

```rust
// CreaturesControlledByDefendingPlayer is a DSL placeholder resolved to
// CreaturesControlledBy(pid) at ApplyContinuousEffect execution. If it reaches
// here unresolved, treat as non-matching (same pattern as Source/TriggeringCreature).
EffectFilter::CreaturesControlledByDefendingPlayer => false,
```

### Change 4 — HashInto arm (EXHAUSTIVE match — compile-forced) + HASH bump

**File**: `crates/engine/src/state/hash.rs`
**Actions** (all four):
1. `impl HashInto for EffectFilter` (exhaustive match ending at discriminant 35,
   `TriggeringCreature`, ~line 2148). Add discriminant 36:
   ```rust
   // PB-OS7 (OOS-EF3-1): DSL placeholder "creatures defending player controls" — discriminant 36
   EffectFilter::CreaturesControlledByDefendingPlayer => 36u8.hash_into(hasher),
   ```
2. Bump the sentinel: `pub const HASH_SCHEMA_VERSION: u8 = 59;` (was 58, line 523).
3. Add a `- 59:` History doc line above the const (after the `- 58:` block, ~line 512-522),
   in the established prose form: name the enum, the new variant + discriminant, cite OOS-EF3-1 /
   CR 508.4, note "`decl_fingerprint` MOVES (a new enum variant); `stream_fingerprint` moves per
   the v40 mechanism."
4. Append a `HashSchemaEpoch { version: 59, decl_fingerprint: <re-pin>, stream_fingerprint: <re-pin> }`
   row to `HASH_SCHEMA_HISTORY` (after the version-58 row at ~line 768-778). Re-pin BOTH
   fingerprints from the failing-gate output (do NOT hand-compute).

### Change 5 — Test sentinel sweep (~40 sites)

**Files**: every `crates/engine/tests/**/*.rs` containing `assert_eq!(HASH_SCHEMA_VERSION, 58…)`
(and the `mtg_engine::HASH_SCHEMA_VERSION, 58u8` variant). These are canary assertions that all
track the CURRENT version. Grep `HASH_SCHEMA_VERSION, 58` and replace `58` -> `59` at every site
(~40 files — full list below, plus `crates/engine/tests/core/hash_schema.rs:1194`). This is the
deliberate tripwire from `state/hash.rs:551` ("sentinels scattered through the test suite").
Known sites include (non-exhaustive; grep to be sure): `casting/optional_cost_and_counter_tax.rs`,
`core/hash_schema.rs`, `rules/loyalty_target_validation.rs`, and ~37 files under
`tests/primitives/` (pb_ac1/ac3/ac4/ac5/ac6/ac7/ac8/ac9, pb_ef1/ef2/ef6/ef7/ef10/ef11×2,
pb_os5/os6, primitive_pb_* xa/xa2/xs/xs_e/ewc/ewcd/ts/eat/cc_a/cc_c_followup/lki_power/lki_cc/
oos_lki_power_3, pbn_/pbt_×2/pbd_/pbp_, mechanics_e_l/effect_sacrifice_permanents_filter).

### Change 6 — Exhaustive-match audit (confirmed complete)

Only TWO exhaustive `EffectFilter` matches exist engine-wide; both are handled above:
| File | Match expression | ~Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/rules/layers.rs` | `filter_matches` `match &effect.filter` | 611 | add `=> false` guard (Change 3) |
| `crates/engine/src/state/hash.rs` | `impl HashInto for EffectFilter` | 2061 | add discriminant 36 (Change 4) |

Confirmed NON-exhaustive (have `_`/`other` wildcards — no change needed): `rules/copy.rs:111`
(`_ => false`), `rules/face.rs:156` (`other => other.clone()`), `effects/mod.rs:3144`
(`other => other.clone()` — Change 2 adds a real arm before it), `rules/layers.rs:1598` &
`:1666` (`_ => None` filter_maps). Confirmed constructors-only (no match): `rules/resolution.rs`,
`rules/abilities.rs`, `state/ability_definition_registry.rs`. Confirmed unrelated type
(`ObjectFilter`, not `EffectFilter`): `rules/replacement.rs`. Confirmed NO `EffectFilter` match in
`tools/` (replay-viewer, tui) or `crates/card-types/` — so NO display arm needed there. Runner MUST
still run `cargo build --workspace` after Change 1 to catch anything this audit missed.

---

## Card Definition — New (1)

### `crates/card-defs/src/defs/silumgar_the_drifting_death.rs` (NEW, `Complete`)

**Oracle text** (MCP-verified 2026-07-19): "{4}{U}{B} Legendary Creature — Dragon 3/7. Flying,
hexproof. Whenever a Dragon you control attacks, creatures defending player controls get -1/-1
until end of turn."

**Structure** (templates: `dromoka_the_eternal.rs` for the per-Dragon attack trigger;
`massacre_wurm.rs` for the boxed `ApplyContinuousEffect`/`ModifyBoth`/`UntilEndOfTurn` shape;
`olivia_voldaren.rs` for the `Completeness::Complete` + helpers-prelude idiom):

- `types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Dragon"])`
- `power: Some(3), toughness: Some(7)`
- `mana_cost: Some(ManaCost { generic: 4, blue: 1, black: 1, ..Default::default() })`
- abilities:
  1. `AbilityDefinition::Keyword(KeywordAbility::Flying)`
  2. `AbilityDefinition::Keyword(KeywordAbility::Hexproof)`
  3. `AbilityDefinition::Triggered { once_per_turn: false,
     trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks {
       filter: Some(TargetFilter { has_subtype: Some(SubType("Dragon".to_string())),
       ..Default::default() }) },
     effect: Effect::ApplyContinuousEffect { effect_def: Box::new(ContinuousEffectDef {
       layer: EffectLayer::PtModify,
       modification: LayerModification::ModifyBoth(-1),
       filter: EffectFilter::CreaturesControlledByDefendingPlayer,
       duration: EffectDuration::UntilEndOfTurn,
       condition: None }) },
     intervening_if: None, targets: vec![], modes: None, trigger_zone: None }`
- `completeness: Completeness::Complete`
- Header comment: cite CR 508.4/611.2a/205.3m, the 2014-11-24 ruling (per-Dragon, per-defender),
  and PB-OS7 / OOS-EF3-1.

**Why Complete (all four proven by the tests below)**: per-Dragon trigger (dromoka template +
abilities.rs:4096 per-attacker loop), per-defender scope (Change 2 stamp), EOT expiry
(`expire_end_of_turn_effects`), 0-toughness SBA (CR 704.5f). No gated-stub effects. Do NOT use
`Effect::Choose`/`MayPayOrElse`/`AddMana*` (barred from Complete per §5).

---

## Karazikar — SHIP/BLOCK decision: **BLOCKED** (not authored this PB)

**Card name (MCP-verified)**: "Karazikar, the Eye Tyrant" — {3}{B}{R} Legendary Creature —
Beholder 5/5. (The brief's "scarlet_throat" was a mislabel.) File would be `karazikar_the_eye_tyrant.rs`.

**Oracle**: (1) "Whenever you attack a player, tap target creature that player controls and goad
it." (2) "Whenever an opponent attacks another one of your opponents, you and the attacking player
each draw a card and lose 1 life."

**Decision — stays honestly blocked. Do NOT author.** Neither ability is expressible after this PB;
both need NEW primitives distinct from the continuous layer filter this PB adds:

- **Remainder R1 (ability 1)** — a defending-player-scoped *target filter*. "tap target creature
  that player controls" requires target-legality validation to restrict candidates to creatures
  controlled by the just-attacked player, resolved from the trigger's captured defending player at
  target-selection time. This lives in the target-VALIDATION path (`casting.rs` / targeting), a
  different subsystem from the layer-system continuous filter — it is the "target-selection sibling"
  the OOS-EF3-1 seed named, NOT covered by `CreaturesControlledByDefendingPlayer`. (Goad itself
  exists: `Effect::Goad { target }`.)
- **Remainder R2 (ability 2)** — an opponent-attacks-another-opponent `TriggerCondition`
  ("an opponent attacks another one of your opponents": attacking player ≠ you AND is your
  opponent AND defending player ≠ you AND is your opponent). No such trigger condition exists.

Folding either into PB-OS7 would balloon it past the single-primitive scope (per
`memory/conventions.md` "implement-phase default-to-defer" and `feedback_pb_yield_calibration`).
**File new seed OOS-OS7-1** (below).

---

## Adjacency notes & new seeds (do NOT author here)

The mandatory pre-existing TODO sweep (`grep TODO.*defend / CreaturesControlledBy` over
`crates/card-defs/src/defs/`) returned several "defending player controls" cards, but **0 forced
adds for THIS primitive** — each needs a *different* defending/target-player primitive:
- `kogla_the_titan_ape.rs` — "destroy target artifact/enchantment defending player controls" →
  needs R1 (defending-player target filter), approximated as opponent today. Rides OOS-OS7-1.
- `kazuul_tyrant_of_the_cliffs.rs` — "if you're the defending player" → needs a defender-side
  intervening-if / per-defending-player check. Different primitive.
- `polymorphists_jest.rs`, `great_oak_guardian.rs`, `naya_charm.rs` — want a *declared/target*-
  player-scoped continuous filter ("creatures TARGET player controls"), i.e. a
  `CreaturesControlledByTargetPlayer { index }` placeholder (same substitution mechanism, but
  reads `ctx.targets[index]` instead of `ctx.defending_player`). Adjacent, cheap follow-on, but
  out of this PB's defending-player scope.

**Record in plan preamble**: TODO sweep ran; 0 cards self-identify as blocked on the
defending-player *continuous* filter specifically.

**New seed OOS-OS7-1 (capability)** — defending-player-scoped *target filter* + opponent-vs-opponent
attack trigger. Unblocks Karazikar (both halves) and the R1 half of `kogla_the_titan_ape`. File in
the OOS retriage plan §3/§8 with the R1/R2 breakdown above.

**Optional adjacent seed OOS-OS7-3 (capability, low priority)** — `CreaturesControlledByTargetPlayer
{ index }` continuous placeholder for polymorphists_jest / great_oak_guardian / naya_charm. Same
one-line substitution shape as this PB. Note only; do not author. **Renumbered from OOS-OS7-2 during
the PB-OS7 fix phase** (review Finding 1) — OOS-OS7-2 was reassigned to the CR 611.2c
resolution-time affected-set snapshot seed (a correctness finding, filed ahead of this
still-unfiled, lower-priority capability note) to avoid an id collision.

> ⚠️ **OOS-OS7-3 was NEVER FORMALLY FILED, and the ID is contested** (verified `scutemob-142`,
> 2026-07-19). It appears in exactly two non-canonical places: this note, and `pb-review-OS7.md:118`
> — which simultaneously proposed the *same* ID for the 611.2c content. The 611.2c content won and
> shipped as OOS-OS7-2; this filter note was orphaned. The canonical inventory
> `oos-retriage-plan-2026-07-18.md` has entries for OOS-OS7-1 (`:414-422`) and OOS-OS7-2
> (`:424-444`) and **no OOS-OS7-3**. The underlying gap is real (2 cards, not 3 — `naya_charm` is
> `inert` and needs a mass-*tap*, not a continuous filter, so this seed would not flip it).
> **Refiled under a fresh, uncontested ID as OOS-RS-5**; do not reuse `OOS-OS7-3`.
> See `memory/primitives/rider-seed-triage-2026-07-19.md` §1b / §3 (folded into rank R7).

---

## Unit Tests

**File**: `crates/engine/tests/primitives/pb_os7_defending_player_continuous_filter.rs`
**Register**: add `mod pb_os7_defending_player_continuous_filter;` to
`crates/engine/tests/primitives/main.rs` (after line 39, `mod pb_os6_dfc_flip_conditions;`).
**SR-9a**: never add a top-level `tests/*.rs`; the module MUST be registered in `main.rs` or the
coverage silently vanishes.
**Patterns**: mirror `tests/primitives/pb_os5_relative_attacker_count.rs` and `pb_os6_dfc_flip_conditions.rs`
(attack-count setup, per-attacker triggers, EOT stepping). Use `GameStateBuilder`. Every test cites
CR. Probe by EXECUTION (SR-34/36) — build state, declare attackers, resolve triggers, read
`calculate_characteristics` for the post-layer P/T, assert. Do NOT source-trace.

Tests to write:
- `test_os7_defending_player_creatures_debuffed` — 2-player: Silumgar + one other Dragon attack
  the opponent; assert every creature the opponent controls is at P-1/T-1 via layer-resolved
  characteristics; assert the attacker's own creatures are unaffected. CR 508.4/611.2a.
- `test_os7_four_player_bystander_decoy` — 4-player: Silumgar's Dragon attacks player B only;
  assert B's creatures get -1/-1 but C's and D's creatures (bystanders) are untouched
  (the wedge property is *controller == defending player*, non-vacuous: C/D each have a
  1-toughness creature that must survive). CR 508.4.
- `test_os7_multi_attack_same_defender_stacks` — two Dragons attack the SAME opponent → two
  stacked `CreaturesControlledBy(sameB)` effects → that opponent's creatures at -2/-2.
  Source: Silumgar ruling 2014-11-24. CR 611.2a.
- `test_os7_multi_attack_different_defenders_scoped` — Dragon1 attacks B, Dragon2 attacks C →
  B's creatures -1/-1, C's creatures -1/-1, each in its own scope; a third opponent D untouched.
  Directly encodes the 2014-11-24 ruling example.
- `test_os7_until_end_of_turn_expiry` — after the debuff, advance to cleanup; assert
  `expire_end_of_turn_effects` removed the `ModifyBoth(-1)` effects and the opponent's creatures
  are back to base P/T. CR 514.2. (Assert the continuous_effects vector no longer contains the
  UntilEndOfTurn ModifyBoth entries.)
- `test_os7_toughness_death_sba_defender_vs_bystander` — defending opponent has a 1-toughness
  creature (dies to -1/-1 as SBA, CR 704.5f) AND a bystander opponent has a 1-toughness creature
  (survives). Assert the defender's creature is in the graveyard and the bystander's is on the
  battlefield after SBA. This is the load-bearing correctness test.
- `test_os7_no_defending_player_applies_to_nothing` — negative/guard test: exercise the
  `None => return` skip path (e.g. resolve `ApplyContinuousEffect { CreaturesControlledByDefendingPlayer }`
  with `ctx.defending_player == None`) and assert NO creature (including the controller's own) is
  debuffed. Prevents the controller-fallback footgun.
- `test_os7_hash_schema_sentinel` — `assert_eq!(mtg_engine::HASH_SCHEMA_VERSION, 59u8);` (strict
  equality per `memory/conventions.md` hash-sentinel convention).

---

## Verification Checklist

- [ ] `EffectFilter::CreaturesControlledByDefendingPlayer` added; `cargo check -p mtg-card-types`
- [ ] Substitution arm in `effects/mod.rs` ApplyContinuousEffect (`None => return`)
- [ ] `layers.rs` filter_matches `=> false` guard arm
- [ ] `hash.rs` HashInto discriminant 36 + HASH_SCHEMA_VERSION 58→59 + `- 59:` History line + v59
      HashSchemaEpoch row (both fingerprints re-pinned from failing-gate output)
- [ ] All ~40 `assert_eq!(HASH_SCHEMA_VERSION, 58)` test sentinels bumped to 59
- [ ] PROTOCOL gate re-run: confirm PROTOCOL_SCHEMA_FINGERPRINT did NOT move (stays v21); if it
      moved, STOP AND FLAG
- [ ] `silumgar_the_drifting_death.rs` authored `Complete`, no gated stubs, no TODOs
- [ ] Karazikar NOT authored; OOS-OS7-1 filed in the retriage plan
- [ ] New test module registered in `tests/primitives/main.rs`; all 8 tests pass
- [ ] `cargo build --workspace` (catches any missed exhaustive match / tool arm)
- [ ] `cargo test --all` (incl. `tools/check-defs-fmt.sh` via `core card_defs_fmt`)
- [ ] `cargo clippy -- -D warnings`; `cargo fmt --check` + `tools/check-defs-fmt.sh`

---

## Risks & Edge Cases

- **Controller-fallback footgun**: the single most important correctness point. `None => return`
  (NOT `unwrap_or(ctx.controller)`). `PlayerTarget::DefendingPlayer` uses `unwrap_or(ctx.controller)`
  for *point* effects (harmless degenerate), but for a -1/-1 continuous that fallback debuffs the
  caster's own board. The `test_os7_no_defending_player_applies_to_nothing` test guards this.
- **Membership vs player capture (CR 611.2c divergence, CORRECTED post-review)**: we lock the
  PLAYER at resolution but let the filter re-evaluate membership live. **This is NOT CR-correct**
  per CR 611.2c (the affected *set*, not just the player, must be locked at resolution) — it is a
  known, pre-existing, engine-wide simplification shared by every resolution-generated mass P/T
  filter (`golgari_charm.rs`, `eyeblight_massacre.rs`), so PB-OS7 correctly follows precedent rather
  than introducing a new divergence. Do NOT attempt to snapshot the creature set in this PB — that
  would be an out-of-scope engine-wide fix (implement-phase-default-to-defer); tracked as
  **OOS-OS7-2** in `memory/primitives/oos-retriage-plan-2026-07-18.md`.
- **Per-attacker independence**: relies on the abilities.rs:4096-4113 loop assigning each attacker's
  triggers their own `defending_player_id`. This is PB-EF3 machinery; the multi-defender test
  proves it end-to-end. If a future refactor batches attack triggers, this scope would break —
  the tests are the regression guard.
- **Planeswalker attack target**: if a Dragon attacks a planeswalker, `defending_player` resolves
  to that planeswalker's controller (abilities.rs:4107-4109). Silumgar's oracle says "defending
  player," which per CR 508.4 is the planeswalker's controller — correct. (Not separately tested;
  covered by the existing DefendingPlayer capture logic. Optional extra test if cheap.)
- **HASH sentinel churn**: ~40 files. Mechanical find/replace, but a missed one fails CI — run the
  full suite, not a filtered subset, before signaling ready.
- **PROTOCOL non-bump surprise**: the brief anticipated a PROTOCOL bump. It is NOT forced (EffectFilter
  off wire closure). The runner must not "helpfully" bump PROTOCOL to match the brief — that would
  desync the fingerprint gate. Trust the gate.
