# Primitive Batch Plan: PB-AC9 — Misc & mana (FINAL AC-chain batch)

**Generated**: 2026-07-10
**Primitive(s)**: `Effect::WheelHand` (new) + `Effect::SetNoMaximumHandSize` (new, persistent player designation). **Three of the five briefed primitives already exist** and collapse to card fixes.
**CR Rules**: 121.1 (draw), 701.9 (discard), 701.24 (shuffle), 706.1a/706.2/706.3a-c (dice + results table), 614.1 (replacement), 402.2 (max hand size), 605.1a (mana abilities — filter lands).
**Cards affected**: **10 fully-clean unblocks** + 1 partial-correctness upgrade (Reforge, still Miracle-blocked). See yield table.
**Dependencies**: none new. Relies on already-shipped PB-CD (counter doubling), the existing token-doubling replacement (`ReplacementModification::DoubleTokens`), and the existing `Effect::RollDice` d20 infra.
**Deferred items from prior PBs**: PB-AC8 explicitly recommended a **stale-marker triage sweep before scoping AC9** (its ~14 briefed cards collapsed to 3 mis-triage wins). That sweep is done here and confirms the same pattern: **5 of the 10 AC9 unblocks are stale markers naming primitives that already exist.**

---

## 0. Scope decisions (the yield-honesty call — READ FIRST)

The AC8 worker predicted AC9's markers might name real gaps *or* be more stale markers.
Recon-first verification against **live code** (not the brief, not MEMORY.md) found:

| Briefed primitive | Verdict | Why |
|---|---|---|
| **d20 + tiered outcome** | **ALREADY EXISTS → DROP primitive** | `Effect::RollDice { sides, results: Vec<(u32,u32,Effect)>}` (`card_definition.rs:1866`, dispatch `effects/mod.rs:3547`), `EffectAmount::LastDiceRoll`, `GameEvent::DiceRolled` all live and **already used by `ancient_silver_dragon.rs`**. Results table already supports single (`N`), range (`N1–N2`), and `N+` (set `high=sides`). Seed-collision is **already solved**: each roll reads `state.timestamp_counter` as seed then `+= 1` (`:3554-3555`), so two rolls in one resolution get distinct seeds. `GameEvent::DiceRolled` already emitted (invariant 4). **The recon grep for `"d20"`/`"die-roll"` missed this — the type is `RollDice`.** |
| **token-doubling replacement** | **ALREADY EXISTS → DROP primitive** | `ReplacementModification::DoubleTokens` (`replacement_effect.rs:186`), `apply_token_creation_replacement()` (`replacement.rs:2887`), `ReplacementTrigger::WouldCreateTokens`. **Already used by `adrix_and_nev_twincasters.rs` and `elspeth_storm_slayer.rs`** (both fully authored, 0 markers). NOTE: it is **wired at only 2 of ~13 token-creation sites** — a real half-wired bug, but it affects **0 AC9 roster cards** (see §5). |
| **multi-output filter mana** | **DROP — M10-gated, cards already function** | Filter lands (`fetid_heath.rs`, `cascade_bluffs.rs`, `flooded_grove.rs`, `rugged_prairie.rs`, …) are **already authored** via `Effect::AddManaFilterChoice` producing the middle option (1+1). Full 3-way choice (2W / WB / 2B) is **interactive player choice = M10**, consistent with every other M10-deferred choice (shockland tapped, any_color→colorless). **No card carries a blocking marker; 0 campaign yield from building it now.** Do NOT build; do NOT seed. Record as M10 interactive-choice item. |
| **SearchLibrary multi-name** | **DROP — no roster → OOS seed** | `TargetFilter.has_name: Option<String>` (single name) already exists. A grep of every def for "named A or B" style search found **zero** cards. The nearest cards (`tiamat.rs` "up to five … with different names", `eerie_ultimatum.rs` "different names") are a **distinct** primitive (all-different uniqueness) that is *itself* M10-interactive-choice-blocked. Per AC-chain precedent ("do not build primitives that unblock zero cards"), **record `OOS-AC9-MULTINAME` seed** and do not build. |
| **`Effect::WheelHand`** | **BUILD** | Genuinely absent (`grep WheelHand/DiscardHand` → 0). Unblocks 4 cards. See §3.1. |
| **`Effect::SetNoMaximumHandSize` (persistent, rest-of-game)** | **BUILD (recon-discovered co-blocker)** | Not in the brief; found while checking Ancient Silver Dragon. `no_max_hand_size` is **recomputed from the battlefield every cleanup** (`turn_actions.rs:1487-1507`), so a *rest-of-the-game* designation is not expressible. Small, self-contained, unblocks 1 card. It is the **sole driver of the HASH bump** — but since WheelHand's hash arm bumps the version by convention anyway, the field rides free. See §3.2. |

### Yield table (honest)

| Card | Unblocked by | Kind |
|---|---|---|
| `incendiary_command.rs` (mode 3) | WheelHand | new primitive |
| `shattered_perception.rs` | WheelHand | new primitive |
| `winds_of_change.rs` | WheelHand (shuffle disposal) | new primitive |
| `echo_of_eons.rs` | WheelHand (shuffle H+GY, Fixed) | new primitive |
| `ancient_silver_dragon.rs` | SetNoMaximumHandSize | new primitive |
| `parallel_lives.rs` | token doubling (already exists) | **stale marker** |
| `anointed_procession.rs` | token doubling (already exists) | **stale marker** |
| `doubling_season.rs` | token + counter doubling (both exist) | **stale marker** |
| `ancient_copper_dragon.rs` | RollDice (already exists) | **stale marker** |
| `ancient_gold_dragon.rs` | RollDice (already exists) | **stale marker** |
| **TOTAL** | **10 fully clean** | 5 new-primitive + 5 stale-marker |
| `reforge_the_soul.rs` | WheelHand body fixed; **still Miracle-blocked** | partial upgrade |

**TODO sweep result (mandatory roster-recall gate)**: swept `crates/engine/src/cards/defs/` for TODO/ENGINE-BLOCKED naming each primitive. Found the 10 above + Reforge (partial). **Co-blocked / out of scope** (confirmed, do NOT author): `ancient_brass_dragon.rs` (M10 interactive variable-count reanimation w/ MV budget), `ancient_bronze_dragon.rs` (reflexive "when you do" trigger + up-to-2 target — a separate primitive), `emergency_eject.rs` (needs a Lander token spec — NOT multi-name search), `tiamat.rs`/`eerie_ultimatum.rs` (different-names uniqueness, M10). No other AC9-primitive TODOs exist.

---

## 1. CR rule text (MCP-verified this session)

- **706.1a** — "d20" = 20-sided die, outcomes 1..20. **706.2** — natural result then modifiers → result. **706.3a** — results table entries are single `N`, range `N1–N2`, or open `N+`; "use the result to determine which effect happens." **706.3b** — the roll + modifiers + table are ONE ability (do NOT split into a reflexive trigger unless oracle says "when you do"). **706.3c** — "Roll again" reuses same dice.
- **614.1 / 614.1a** — replacement effects use "instead", apply continuously as events happen, are not locked in, act as "shields." Token doubling is exactly this (no stack, not a trigger).
- **121.1** — a player draws by putting the top card of their library into their hand.
- **701.9** — Discard (keyword action). **701.24** — Shuffle (keyword action). **701.23** — Search.
- **402.2** — a player's maximum hand size is seven unless an effect sets it otherwise; enforced at cleanup (CR 514.1a) for the active player only.
- **605.1a** — filter-land abilities are mana abilities (relevant only to the DROPPED filter-mana item).

---

## 2. Discriminant / hash baseline (verified live)

- `HASH_SCHEMA_VERSION = 35` (`hash.rs:296`) → **bump to 36**.
- `Effect` hash discriminants: max is **90** (`Effect::WinGame`, `hash.rs:6200`) → `WheelHand = 91`, `SetNoMaximumHandSize = 92`.
- New sub-enums `WheelDisposal`, `WheelDraw` need `HashInto` impls (own discriminants, start at 0).
- **StackObjectKind / KeywordAbility exhaustive matches DO NOT need arms** — WheelHand and SetNoMaximumHandSize are `Effect` variants, and `tools/tui/.../stack_view.rs` + `tools/replay-viewer/.../view_model.rs` match only on `StackObjectKind` and `KeywordAbility` (verified: **no `Effect::` match in `tools/`**). Positive assertion — do not add arms there. Still run `cargo build --workspace`.

---

## 3. Engine changes

### 3.1 `Effect::WheelHand` — the wheel/Timetwister family

**File**: `crates/engine/src/cards/card_definition.rs` (Effect enum + two helper enums)

Add near `Effect::CoinFlip` / `RollDice`:

```rust
/// CR 701.9 / 701.24 / 121.1: "Each player discards/shuffles-away their hand,
/// then draws." Atomic so the "that many" count is snapshotted BEFORE disposal
/// (a naive DiscardCards{HandSize} + DrawCards{HandSize} draws 0 — see the
/// former incendiary_command.rs ENGINE-BLOCKED note). Covers Wheel of Fortune,
/// Timetwister, Windfall, Winds of Change, Echo of Eons, Incendiary Command mode 3.
WheelHand {
    player: PlayerTarget,     // Controller (Shattered Perception) or EachPlayer (the rest)
    disposal: WheelDisposal,
    draw: WheelDraw,
},
```

```rust
/// How the affected player's current hand is disposed of before drawing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WheelDisposal {
    /// CR 701.9: discard the entire hand (to graveyard; Madness→exile via existing path).
    Discard,
    /// CR 701.24: put the entire hand into the library, then shuffle. (Winds of Change)
    ShuffleHandIntoLibrary,
    /// CR 701.24: put the entire hand AND graveyard into the library, then shuffle.
    /// (Echo of Eons / Timetwister)
    ShuffleHandAndGraveyardIntoLibrary,
}

/// How many cards the affected player draws after disposal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WheelDraw {
    /// Draw a number equal to the HAND size snapshotted before disposal. ("that many")
    ThatMany,
    /// Draw a fixed number regardless of hand size. (Wheel of Fortune / Echo of Eons = 7)
    Fixed(u32),
}
```

**File**: `crates/engine/src/cards/helpers.rs` — export `WheelDisposal`, `WheelDraw` in the prelude (card defs construct them). Follow the existing `pub use ...` block.

**File**: `crates/engine/src/effects/mod.rs` — dispatch in `execute_effect_inner`, near `Effect::DiscardCards` (:548) / `Effect::CoinFlip`. Pattern:

```rust
Effect::WheelHand { player, disposal, draw } => {
    for p in resolve_player_target_list(state, player, ctx) {   // APNAP order; EachPlayer supported
        let hand_size = state.objects.values()
            .filter(|o| o.zone == ZoneId::Hand(p)).count();      // CR snapshot BEFORE disposal
        match disposal {
            WheelDisposal::Discard =>
                discard_cards(state, p, hand_size, events),      // reuse existing helper (:7576) — Madness→exile handled
            WheelDisposal::ShuffleHandIntoLibrary =>
                move_zone_all_then_shuffle(state, p, &[ZoneId::Hand(p)], events),
            WheelDisposal::ShuffleHandAndGraveyardIntoLibrary =>
                move_zone_all_then_shuffle(state, p, &[ZoneId::Hand(p), ZoneId::Graveyard(p)], events),
        }
        let n = match draw { WheelDraw::ThatMany => hand_size, WheelDraw::Fixed(k) => *k as usize };
        for _ in 0..n {
            if let Ok(evs) = crate::rules::turn_actions::draw_card(state, p) { events.extend(evs); }
        }
    }
}
```

- **New private helper `move_zone_all_then_shuffle(state, p, from_zones, events)`** in `effects/mod.rs`: collect all object ids in `from_zones` in **ascending ObjectId order** (determinism), `move_object_to_zone(id, ZoneId::Library(p))` each (emit no per-card public event beyond what `move_object_to_zone` gives), then shuffle the library with the **established seed pattern**: `let seed = state.timestamp_counter; state.timestamp_counter += 1; zone.shuffle(&mut StdRng::seed_from_u64(seed));` (mirror `effects/mod.rs:2778-2785`). CR 400.7: moved cards become new objects — do not reuse old ids afterward.
- **CR compliance**: `ThatMany` counts HAND cards only (not graveyard) — matches Winds of Change wording "draws that many". Snapshot must be taken before *any* disposal mutation.
- **No new `GameEvent` variant** — component events (`CardDiscarded`, draw events, shuffle/zone-move events) already fire from the helpers. Keeps the hash surface to the Effect arms only.
- **`draw_card` on empty library** is a safe no-op (see gotcha) — a wheel with a short library simply draws fewer; do not special-case.

**File**: `crates/engine/src/state/hash.rs` — add `Effect::WheelHand { player, disposal, draw } => { 91u8.hash_into; player.hash_into; disposal.hash_into; draw.hash_into; }` in the `Effect` impl (after WinGame:6200); add `HashInto for WheelDisposal` and `HashInto for WheelDraw` impls (discriminants 0.. ; `Fixed(k)` also hashes `k`).

### 3.2 `Effect::SetNoMaximumHandSize` — persistent "rest of the game" designation

**File**: `crates/engine/src/state/player.rs` — add a persistent flag beside `no_max_hand_size` (:320):
```rust
/// CR 402.2: set by a one-shot effect (e.g. Ancient Silver Dragon) that removes
/// the maximum hand size FOR THE REST OF THE GAME, independent of any permanent.
/// OR'd into the per-cleanup recompute so it is never clobbered back to false.
pub no_max_hand_size_permanent: bool,
```
**File**: `crates/engine/src/state/builder.rs:257` — the **only** explicit `PlayerState` literal; add `no_max_hand_size_permanent: false,`.

**File**: `crates/engine/src/cards/card_definition.rs` — Effect enum:
```rust
/// CR 402.2: permanently remove the target player's maximum hand size for the
/// rest of the game (Ancient Silver Dragon). Sets PlayerState.no_max_hand_size_permanent.
SetNoMaximumHandSize { player: PlayerTarget },
```
**File**: `crates/engine/src/effects/mod.rs` — dispatch:
```rust
Effect::SetNoMaximumHandSize { player } => {
    for p in resolve_player_target_list(state, player, ctx) {
        if let Some(ps) = state.players.get_mut(&p) { ps.no_max_hand_size_permanent = true; }
    }
}
```
**File**: `crates/engine/src/rules/turn_actions.rs:1507` — the cleanup recompute currently does `ps.no_max_hand_size = has_no_max;`. Change to:
```rust
ps.no_max_hand_size = has_no_max || ps.no_max_hand_size_permanent;
```
(Do **not** touch the `calculate_characteristics()` scan above it — that layer-correctness fix from AC8 stays.)

**File**: `crates/engine/src/state/hash.rs` — (a) `Effect::SetNoMaximumHandSize { player } => { 92u8.hash_into; player.hash_into; }`; (b) in the `PlayerState` `HashInto` (near `:1430` where `no_max_hand_size` is hashed) add `self.no_max_hand_size_permanent.hash_into(hasher);`; (c) `HASH_SCHEMA_VERSION` **35 → 36** (`:296`) with a new doc-comment line describing PB-AC9.

### 3.3 HASH bump fan-out (hazard 1)

Bump forces the sentinel tripwire in **28 files** (verified list) from `35` → `36`:

`hash.rs` itself + 27 test files: `pb_ac1_untap_counter.rs`, `pb_ac3_dynamic_pt_counts.rs`, `pb_ac4_per_mode_targeting.rs`, `pb_ac5_alt_costs.rs`, `pb_ac6_phase_action_conditions.rs`, `pb_ac7_type_change_ability_removal.rs`, `pb_ac8_restrictions_and_wingame.rs`, `pbd_damaged_player_filter.rs`, `pbn_subtype_filtered_triggers.rs`, `pbp_power_of_sacrificed_creature.rs`, `pbt_up_to_n_targets.rs`, `optional_cost_and_counter_tax.rs`, `loyalty_target_validation.rs`, `effect_sacrifice_permanents_filter.rs`, `primitive_pb_{xs_e,xs,cc_a,cc_c_followup,eat,ewc,ewcd,lki_cc,lki_power,oos_lki_power_3,ts,xa,xa2}.rs`. Grep `HASH_SCHEMA_VERSION, 35|35u8|== 35` under `crates/engine` to confirm none were added since this plan.

### 3.4 Mutation-verified hash test (hazard 1)

In the new test file: construct two states differing only in `players[p].no_max_hand_size_permanent` (false vs true), assert `state_hash(a) != state_hash(b)`. Mutation-verify by temporarily removing the new `hash_into` line and confirming the test fails (per PB-AC8 discipline — not vacuous).

---

## 4. Card definition fixes

### 4.1 New-primitive cards (WheelHand)

- **`incendiary_command.rs`** — replace mode 3 `Effect::Nothing` (:57) with `Effect::WheelHand { player: PlayerTarget::EachPlayer, disposal: WheelDisposal::Discard, draw: WheelDraw::ThatMany }`. Delete the file-level ENGINE-BLOCKED block (:21-30) and the mode-3 comment. Modes 0–2 already authored — leave untouched.
- **`shattered_perception.rs`** — replace the discard+draw `Sequence` (:23-32) with `effect: Effect::WheelHand { player: PlayerTarget::Controller, disposal: WheelDisposal::Discard, draw: WheelDraw::ThatMany }`. Delete TODO (:21-22). Flashback already present.
- **`winds_of_change.rs`** — replace `Effect::Nothing` (:20) with `Effect::WheelHand { player: PlayerTarget::EachPlayer, disposal: WheelDisposal::ShuffleHandIntoLibrary, draw: WheelDraw::ThatMany }`. Delete both TODO blocks.
- **`echo_of_eons.rs`** — replace the discard+draw `Sequence` (:25-34) with `effect: Effect::WheelHand { player: PlayerTarget::EachPlayer, disposal: WheelDisposal::ShuffleHandAndGraveyardIntoLibrary, draw: WheelDraw::Fixed(7) }`. Delete TODO (:22-24). Flashback already present.
- **`reforge_the_soul.rs` (PARTIAL — do NOT mark clean)** — replace the wrong-game-state `DiscardCards{Fixed(7)}`+`DrawCards{Fixed(7)}` `Sequence` with `Effect::WheelHand { player: PlayerTarget::EachPlayer, disposal: WheelDisposal::Discard, draw: WheelDraw::Fixed(7) }` (now correctly discards the *whole* hand then draws 7). **Keep** the Miracle TODO (:14-15) — the card stays PARTIAL, blocked only on `KeywordAbility::Miracle`. This is a correctness upgrade (was: discards exactly 7), not a full unblock.

### 4.2 New-primitive card (SetNoMaximumHandSize)

- **`ancient_silver_dragon.rs`** — the combat-damage trigger effect (currently just `RollDice`) becomes a `Sequence`:
  ```rust
  effect: Effect::Sequence(vec![
      Effect::RollDice { sides: 20, results: vec![(1, 20,
          Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::LastDiceRoll })] },
      Effect::SetNoMaximumHandSize { player: PlayerTarget::Controller },
  ]),
  ```
  Delete both TODOs (:8-9 file header, :43 inline). CR 706.3b: this is all one ability — correct to keep it inside the single triggered ability (idempotent to set the flag each combat).

### 4.3 Stale-marker cards (token doubling — engine already supports; copy `adrix_and_nev_twincasters.rs`)

- **`parallel_lives.rs`** — replace the TODO (:14) with a single `AbilityDefinition::Replacement { trigger: ReplacementTrigger::WouldCreateTokens { controller_filter: PlayerFilter::Specific(PlayerId(0)) }, modification: ReplacementModification::DoubleTokens, is_self: false, unless_condition: None }`. (`PlayerId(0)` is the bound-at-registration placeholder — same as Adrix.) Uses `..Default::default()` tail, so no explicit-struct suffix needed.
- **`anointed_procession.rs`** — identical to Parallel Lives (white; same replacement). Delete TODO (:14).
- **`doubling_season.rs`** — TWO replacements. (a) token clause = the same `WouldCreateTokens`+`DoubleTokens` as above. (b) counter clause "twice that many counters on a permanent you control" = copy the **Vorinclex** pattern (`vorinclex_monstrous_raider.rs:25-35`) but scoped to receiver-you-control:
  ```rust
  AbilityDefinition::Replacement {
      trigger: ReplacementTrigger::WouldPlaceCounters {
          placer_filter: PlayerFilter::Any,                 // "an effect" — any placer
          receiver_filter: ObjectFilter::ControlledBy(PlayerId(0)), // "a permanent you control"
          counter_filter: None,                              // "one or more counters" — any kind
      },
      modification: ReplacementModification::DoubleCounters,
      is_self: false,
      unless_condition: None,
  },
  ```
  Delete both TODOs (:16-17). **Verify** `PlayerFilter::Any` and `ObjectFilter::ControlledBy` exist (recon: `ControlledBy(PlayerId)` at `replacement.rs:252`); if `PlayerFilter::Any` is absent, check the enum for the equivalent "any player" variant.

### 4.4 Stale-marker cards (d20 — engine already supports)

- **`ancient_copper_dragon.rs`** — replace the `CreateToken{treasure_token_spec(10)}` + TODO (:22-23) with:
  ```rust
  effect: Effect::RollDice { sides: 20, results: vec![(1, 20,
      Effect::CreateToken { spec: TokenSpec { count: EffectAmount::LastDiceRoll, ..treasure_token_spec(1) } })] },
  ```
  (`TokenSpec.count` is `EffectAmount`, resolved at execution against `ctx.last_dice_roll`. Any DoubleTokens replacement then correctly stacks on top — Copper + Doubling Season = 2×roll.)
- **`ancient_gold_dragon.rs`** — replace the TODO (:18-21). The token is a **1/1 blue Faerie Dragon with flying**; no helper exists, so construct `TokenSpec` inline (mirror Elspeth's Soldier at `elspeth_storm_slayer.rs:38-53`) with `name: "Faerie Dragon"`, `colors: {Blue}`, `card_types: {Creature}`, `subtypes: {Faerie, Dragon}`, `keywords: {Flying}`, `count: EffectAmount::LastDiceRoll`, then `..Default::default()`. Wrap in `Effect::RollDice { sides: 20, results: vec![(1, 20, Effect::CreateToken { spec })] }`.

---

## 5. Token-doubling completeness pass (correctness — 0 roster yield; recon-mandated enumeration)

**The primitive already ships, but `apply_token_creation_replacement()` is wired at only 2 of the token-creation sites.** This is the exact half-wired shape PB-AC8 review flagged as E1 (`feedback_verify_full_chain`). It does **not** block any AC9 roster card (the roster cards are the *doublers*, which only register the replacement), but a Populate/Myriad/Embalm/Living-Weapon token created while a Doubling Season is out is silently **not** doubled today.

**Every `GameEvent::TokenCreated` site (verified live):**

| Site | Creates | Doubling should apply? | Wired now? |
|---|---|---|---|
| `effects/mod.rs:563` `Effect::CreateToken` | generic tokens | yes | **YES** (:571, keyed `ctx.controller`) |
| `effects/mod.rs:624` `Effect::CreateTokenAndAttachSource` | Living Weapon Germ | yes (first one equipped, rest live/die per SBA) | **NO** |
| `effects/mod.rs:~4520` `Effect::CreateTokenCopy` | token copy | yes | **YES** (:4556, count seeded to 1 then doubled) |
| `resolution.rs:4739` Populate (SOK) | copy of a creature token you control (CR 701.32) | yes | **NO** |
| `resolution.rs:4991` Offspring "except it's 1/1" copy | 1/1 token copy | yes | **NO** |
| `resolution.rs:5674` Myriad | attacking copy per opponent | yes | **NO** |
| `resolution.rs:6348 / 6563 / 6793` Embalm/Eternalize/named-token from `source_card_id` | token copies | yes | **NO** |
| `resolution.rs:7697 / 7718` Gift Food/Treasure | tokens under **recipient** control | yes — key to `recipient`, NOT controller | **NO** |

**Recommended fix (minimal, lower-risk than a full `create_tokens()` extraction)**: at each unwired site, before the creation loop, call `apply_token_creation_replacement(state, <correct controller>, base_count)` and loop the returned `token_count` instead of the hardcoded count. For the Gift sites pass `recipient` (not `_controller`) — Doubling Season keys on the token's controller. For copy/Myriad sites that currently create exactly 1 per opponent/instance, wrap each unit (1 → maybe 2). This keeps each site's bespoke post-creation logic (CopyOf effect, combat registration, P/T override, equip-attach) intact.

**Recommendation to runner**: this is the **highest-risk item and unblocks 0 roster cards**. Do it in a **separate commit** after the roster cards + 2 primitives are green, so a problem here can be reverted without losing the yield. If the batch is running long, it is acceptable to **defer this to seed `OOS-AC9-TOKREPL`** and note it in the close-out — but since AC9 is the final AC batch, closing this latent bug now is the completeness-correct choice. Add one regression test per newly-wired class (Populate, Myriad, Gift-to-opponent, Living Weapon) with a Doubling Season out.

---

## 6. Unit tests

**File**: `crates/engine/tests/pb_ac9_wheel_and_misc.rs` (new). Pattern: follow `pb_ac8_restrictions_and_wingame.rs` (direct `GameStateBuilder` + effect execution; force die rolls by setting `state.timestamp_counter` so `(counter % 20) + 1` is a known value — mirror any existing `RollDice`/`CoinFlip` test).

WheelHand:
- `test_wheel_hand_discard_that_many` — 3-card hand + ≥3 library. Discard/ThatMany → hand back to 3, graveyard +3, library −3. (CR 701.9 + 121.1)
- `test_wheel_hand_fixed_draw` — 2-card hand + ≥7 library. Discard/Fixed(7) → hand 7, graveyard +2. (Wheel of Fortune / Reforge shape)
- `test_wheel_hand_empty_hand_noop` — 0-card hand. Discard/ThatMany → draws 0, no panic. (edge)
- `test_wheel_hand_shuffle_into_library_that_many` — 3-card hand. ShuffleHandIntoLibrary/ThatMany → hand 3, graveyard unchanged, library net unchanged; assert deterministic result across two identical seeded runs (state-hash equal). (CR 701.24)
- `test_wheel_hand_shuffle_hand_and_graveyard_fixed` — 2 hand + 3 graveyard. ShuffleHandAndGraveyardIntoLibrary/Fixed(7) → graveyard 0, hand 7. (Echo of Eons / Timetwister)
- `test_wheel_hand_each_player_multiplayer` — 4-player, EachPlayer/Discard/ThatMany; every player's hand size preserved and every graveyard grows (invariant #5). (CR APNAP order via `resolve_player_target_list`)
- `test_wheel_hand_madness_routes_to_exile` — a Madness card in hand + Discard → exile + Madness trigger queued (reuses `discard_cards` path). (CR 702.35a — regression guard on reuse)

SetNoMaximumHandSize:
- `test_set_no_maximum_hand_size_survives_cleanup` — set flag via effect, no NoMaxHandSize permanent on battlefield, run a full cleanup with a 10-card hand for the active player → no discard. Then a control player *without* the flag with 10 cards *does* discard to 7. (CR 402.2 / 514.1a)
- `test_set_no_max_hand_size_recompute_does_not_clobber` — verify the `|| ps.no_max_hand_size_permanent` OR: run the cleanup recompute directly and confirm the flag stays true even when `has_no_max` scan returns false.
- `test_no_max_hand_size_permanent_hash_mutation` — the mutation-verified hash test from §3.4.

Integration (card defs):
- `test_incendiary_command_mode3_wheels_all_players`
- `test_ancient_copper_dragon_rolls_treasures` — force roll=7, assert 7 Treasure tokens (and 14 with a Doubling Season out, if §5 done). (CR 706 + 614.1)
- `test_ancient_gold_dragon_faerie_tokens` — force a roll, assert that many 1/1 blue Faerie Dragon flyers.
- `test_ancient_silver_dragon_draw_and_no_max` — force roll, assert draw + `no_max_hand_size_permanent == true`.
- `test_doubling_season_doubles_tokens_and_counters` — a token-maker + a counter-placer resolve under Doubling Season → doubled both. (verifies both clauses register)
- `test_parallel_lives_doubles_tokens` — sanity that the copied Adrix pattern registers.

---

## 7. Verification checklist

- [ ] `Effect::WheelHand` + `WheelDisposal` + `WheelDraw` + `Effect::SetNoMaximumHandSize` compile (`cargo check -p mtg-engine`)
- [ ] `no_max_hand_size_permanent` added to `PlayerState`, `builder.rs:257`, `hash.rs` PlayerState impl
- [ ] `HASH_SCHEMA_VERSION` 35→36 + all 28 sentinel files updated
- [ ] Cleanup recompute OR's the persistent flag (`turn_actions.rs`)
- [ ] `WheelDisposal`/`WheelDraw` exported from `helpers.rs`
- [ ] All 9 card defs re-authored; **Reforge stays PARTIAL** with only the Miracle TODO
- [ ] Every deleted TODO/ENGINE-BLOCKED verified gone in the 9 clean cards (`grep -n "TODO\|ENGINE-BLOCKED"`)
- [ ] (if §5 done) token-doubling wired at all listed sites, separate commit, per-class regression tests
- [ ] `cargo test --all` (NOT just `cargo build` — build skips test targets) — includes the mutation hash test
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo build --workspace` (catches TUI / replay-viewer — expected to need NO new arms; confirm)
- [ ] `cargo fmt --check`
- [ ] `python3 tools/authoring-report.py` rerun; post clean-count delta (expect +9 clean, +1 partial→partial)
- [ ] Seed `OOS-AC9-MULTINAME` (multi-name search, no roster) and note the filter-mana M10 deferral in the close-out

---

## 8. Risks & edge cases

- **Snapshot ordering (WheelHand)**: the "that many" count MUST be read before any disposal mutation. A late read after `discard_cards` reads 0 — the exact bug the old Incendiary Command comment documented. The dispatch above snapshots first; keep it that way.
- **CR 400.7 after shuffle disposal**: cards moved to the library are new objects; do not touch their old ids after the move. The draw pulls fresh top-of-library.
- **Determinism**: shuffle disposal must use `seed_from_u64(state.timestamp_counter)` then `+= 1` (never `from_entropy`). Two rolls / two shuffles in one resolution each advance the counter → distinct seeds (this is already why the pre-existing `RollDice` has no collision — the recon's collision worry is moot given the `+= 1`). Include a two-run state-hash-equality assertion in the shuffle test.
- **EachPlayer draw from short library**: `draw_card` is a safe no-op on empty library; a wheel just draws fewer. Do not add game-loss logic here (that is SBA territory at the next check).
- **Doubling Season counter clause scope**: only "a permanent **you** control" — receiver must be `ControlledBy(controller)`, and it does NOT double counters placed on *players* (unlike Vorinclex which is "permanent or player"). Do not copy Vorinclex's `ObjectFilter::Any` verbatim.
- **HASH bump blast radius**: 28 sentinel files. Miss one → a single test fails loudly (tripwire working as designed). Grep both `35u8` and `HASH_SCHEMA_VERSION, 35`.
- **§5 token-doubling completeness is the one genuinely risky change** — bespoke per-site creation logic, Gift-to-recipient controller subtlety, and 11 sites. Isolate in its own commit; it earns 0 roster yield and exists only to close the pre-existing half-wired bug. Defer to `OOS-AC9-TOKREPL` if it threatens the batch.
- **Reforge honesty**: do not let the authoring report count Reforge as clean — it must retain its Miracle TODO. The WheelHand swap fixes its *body* only.
