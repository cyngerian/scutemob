# Primitive Batch Plan: PB-AC8 — Static restrictions, win-cons, no-max-hand

**Generated**: 2026-07-09
**Primitive**: 5 requested — `GameRestriction::{NoMaximumHandSize, MustAttack, CantAttackOwner, CantBeSacrificed}` + `Effect::WinGame`
**CR Rules (VERIFIED via MCP)**: 104.1, 104.2b, 104.3e/f/h, 119.6, 402.2, 508.1c/d, 514.1, 701.21a, 704.5, 800.4, 801.14
**Cards affected**: honest roster below — **2 FULLY unblocked** (both mis-triaged, fixable via *existing* keywords, no new primitive needed); **0 fully unblocked by the 4 genuinely-new primitives** (every candidate has a co-blocking DSL gap → PARTIAL)
**Dependencies**: none (all machinery — `GameRestriction`, `ActiveRestriction`, `has_lost`, `check_game_over`, `check_condition` — already exists)
**Deferred items from prior PBs**: none specific to these primitives

---

## TL;DR for the coordinator (read this first)

Roster discovery produced a **much smaller honest yield than the ~14 estimate**, exactly as
`feedback_pb_yield_calibration` predicts. Three load-bearing findings:

1. **`NoMaximumHandSize` already exists** as `KeywordAbility::NoMaxHandSize` +
   `PlayerState.no_max_hand_size` + the cleanup recompute at `turn_actions.rs:1487-1526`.
   **Recommend DROPPING `GameRestriction::NoMaximumHandSize`** — it would be a redundant
   second path for the same behavior (violates "don't invent a parallel path"). The blocked
   "no max hand" cards split into: (a) mis-triaged self-static creatures that the *existing
   keyword* already covers, and (b) "for the rest of the game" / emblem cards that need a
   *persistent player designation surviving the cleanup recompute* — a **different** primitive,
   out of scope. See "CR / scope corrections."

2. **`MustAttack` (self) already exists** as `KeywordAbility::MustAttackEachCombat`
   (`combat.rs:334-368`). Toski and Alexios's must-attack clause use it. The only genuinely-new
   must-attack case is the **group-filtered** form (Goblin Rabblemaster: "Other Goblin creatures
   you control attack each combat if able"), and Rabblemaster is **PARTIAL** (co-blocked by a
   count-based pump gap). So `GameRestriction::MustAttack` has **0 immediate card yield**.

3. **`Effect::WinGame`, `CantAttackOwner`, `CantBeSacrificed`** are real gaps but their only
   roster cards are all **PARTIAL** (each blocked by a *second*, out-of-scope DSL gap). They are
   valid infrastructure/prerequisite primitives with full unit tests, but they fully unblock
   **zero** cards in this batch.

**Net card wins available right now (backfill via existing keywords, no new engine code):**
- **Nezahal, Primal Tide** — add `KeywordAbility::NoMaxHandSize` (its trigger + discard-exile-return ability are already authored). Mis-triaged.
- **Toski, Bearer of Secrets** — add `KeywordAbility::MustAttackEachCombat` (its combat-damage-draw trigger is already authored). Mis-triaged.

**Recommendation**: build `Effect::WinGame` + `CantBeSacrificed` + `CantAttackOwner` as
prerequisite infrastructure with comprehensive wiring and unit tests (they are on the critical
path to Alexios and the win-con cards); **defer or minimally-stub `GameRestriction::MustAttack`**
(group form, 0 yield, and its self-form already exists); **drop `GameRestriction::NoMaximumHandSize`**.
Land the two mis-triage backfills regardless. This is the "smaller honest roster is the correct
outcome" case.

---

## CR corrections / scope corrections (MCP-verified)

The advisory CR refs were mostly correct, with these clarifications:

- **CR 514.1 / 402.2** — verified. 402.2: "as part of their cleanup step, the player must
  discard excess cards down to the maximum hand size." 514.1: the discard turn-based action.
  Already implemented at `turn_actions.rs:1501-1526`; skipped when `no_max_hand_size`.
- **CR 508.1c vs 508.1d** — the brief lumped "508.1" but the two halves are distinct and matter:
  - **508.1c = RESTRICTIONS** ("can't attack" / "can't attack unless…"): CantAttackOwner lives here.
  - **508.1d = REQUIREMENTS** ("attacks if able"): MustAttack lives here. 508.1d also states the
    interaction: the number of obeyed requirements must be the max possible *without disobeying
    any restriction* — i.e. a requirement yields to a restriction. This is the CantAttackOwner
    vs MustAttack interaction for Alexios.
- **CR 104.2b + 104.1 + 104.3h/801.14 (WinGame multiplayer)** — CORRECTED against the brief:
  - 104.2b: "An effect may state that a player wins the game." 104.1: "A game ends immediately
    when a player wins."
  - **104.3h and 801.14 apply ONLY under the limited-range-of-influence option (rule 801).**
    **Commander does NOT use limited range of influence** (that option is for Emperor / 5+ player
    variants). So in a normal 4-player Commander game, "you win the game" is NOT "each opponent in
    range loses and the game continues." It is: **that player wins → the game ends immediately →
    all other players lose** (104.1). The brief's phrasing ("with the limited-range option absent,
    a player winning causes each remaining opponent to lose") reaches the right *end state* but via
    the wrong rule — the correct citation is 104.1 (game ends immediately), not 104.3h/801.14.
    Both 104.3h and 801.14 are explicitly gated on rule 801 being in effect, which it is not here.
  - **104.3f**: "If a player would both win and lose the game simultaneously, that player loses."
    → WinGame must no-op if the controller is already `has_lost`.
- **CR 701.21a (Sacrifice)** — verified (rule number shifted to 701.21 in the current CR; the old
  701.16 no longer refers to Sacrifice). "To sacrifice a permanent, its controller moves it from
  the battlefield directly to its owner's graveyard. A player can't sacrifice something that isn't
  a permanent, or something that's a permanent they don't control." There is **no dedicated
  "can't be sacrificed" CR rule** — it is an ordinary continuous restriction, enforced at every
  sacrifice site.
- **CR 704.5 (SBA list)** — verified: winning-via-effect is **NOT** in the SBA list. WinGame is an
  effect that resolves from the stack, not an SBA. Losing (704.5a/b/c) IS an SBA. This is the crux
  of hazard #4: WinGame must set opponents' `has_lost` atomically inside its own resolution and let
  the **existing** post-resolution SBA + `check_game_over` pass finalize — it must NOT be spliced
  into the `sba.rs` batch.

---

## Discriminant chain (verified FROM SOURCE)

**No new variants are added to `KeywordAbility`, `AbilityDefinition`, or `StackObjectKind`.**

- `KeywordAbility` (`crates/engine/src/state/types.rs:411`) — order-based, no explicit
  discriminants; `NoMaxHandSize` at L487, `MustAttackEachCombat` at L1665 already exist. Untouched.
- `AbilityDefinition` (`crates/engine/src/cards/card_definition.rs:215`) — the 4 static
  restrictions register through the **existing** `AbilityDefinition::StaticRestriction { restriction }`
  (disc 65) arm in `register_static_continuous_effects` (`replacement.rs:2090`). Untouched.
- `StackObjectKind` (`crates/engine/src/state/stack.rs:563`) — WinGame resolves inside existing
  spell/triggered-ability stack objects; no new SOK. Untouched.

**Consequence — exhaustive-match relief**: the usual hazard sites
`tools/tui/src/play/panels/stack_view.rs` (StackObjectKind) and
`tools/replay-viewer/src/view_model.rs` (StackObjectKind + KeywordAbility) are **NOT impacted** —
verified they contain no `Effect::` or `GameRestriction::` match arms. `cargo build --workspace`
still required, but no tool-crate edits are expected.

---

## Hash impact (hazard F — mutation-verified)

`HASH_SCHEMA_VERSION` is currently **34** (`state/hash.rs:287`). Bump to **35**.

| Change | hash.rs site | Action |
|--------|--------------|--------|
| 3 new `GameRestriction` variants | `impl HashInto for GameRestriction` L1876-1901 (currently arms `0u8..8u8`) | Add `MustAttack => 9u8`, `CantAttackOwner => 10u8`, `CantBeSacrificed => 11u8` (drop `NoMaximumHandSize` per recommendation) |
| `Effect::WinGame` variant | `impl HashInto for Effect` L5433 | Add a WinGame arm (trivial — no payload if the `condition` field is dropped; see design note) |
| new `LossReason` variant (if added) | `impl HashInto for LossReason` L1087-1096 (arms `0u8..4u8`) | Add `OpponentWonGame => 5u8` (or reuse an existing reason — see design note) |
| schema bump | `HASH_SCHEMA_VERSION` L287 + parity test | 34 → 35, update `assert_eq!(HASH_SCHEMA_VERSION, 35)` |

**New mutable/runtime state fields requiring `HashInto`**: **NONE beyond the enum-variant arms
above.** WinGame reuses the existing `PlayerState.has_lost: bool` (already hashed at
`hash.rs:1412`) to eliminate opponents — it does **not** introduce a `has_won` field. The
restriction variants are carried in the already-hashed `ActiveRestriction` (L1904). This is the
clean path around the PB-AC1/AC5 hash HIGH: no new struct fields, only new enum arms. The mutation-
verified tests required by hazard F are therefore the *game-state* tests below (they mutate
`has_lost` / registered restrictions and assert the hash changes), not new-field tests.

---

## Primitive Specification (per-primitive)

### P1 — `GameRestriction::NoMaximumHandSize` → **RECOMMEND DROP**

Already expressible via `KeywordAbility::NoMaxHandSize`. Adding a `GameRestriction` variant would
duplicate the cleanup-discard skip. Instead:
- **Fix mis-triaged cards with the existing keyword** (see roster: Nezahal is fully unblocked this
  way; Curiosity Crafter / Niv-Mizzet get their no-max clause fixed but stay PARTIAL on their other
  gap; author them with the keyword and keep the *other* TODO).
- The "no maximum hand size **for the rest of the game**" cards (Sea Gate Restoration, Ancient
  Silver Dragon) and the **emblem** case (Wrenn and Seven) need a *persistent player designation*
  that survives the cleanup recompute at `turn_actions.rs:1487-1499` (which currently overwrites
  `ps.no_max_hand_size` from battlefield keywords every cleanup). That is a **separate primitive**
  (e.g. `PlayerState.permanent_no_max_hand_size: bool` set by a resolution effect, OR'd into the
  recompute) — **out of PB-AC8 scope**; flag as a follow-up seed (OOS-AC8-1).

If the coordinator insists on the variant for uniformity, it is a ~10-line addition (variant +
hash arm + a `state.restrictions` scan OR'd into `has_no_max` at `turn_actions.rs:1490`), but it
yields no card that the keyword doesn't already cover. Planner recommends **not** adding it.

### P2 — `GameRestriction::MustAttack` (group form)  — 0 yield, recommend defer/minimal

- Self form ("this attacks each combat if able") is already `KeywordAbility::MustAttackEachCombat`,
  enforced at `combat.rs:334-368`. Toski/Alexios/Ulamog/Dauthi use it.
- Only the **group** form is new (Rabblemaster). If built: add
  `GameRestriction::MustAttack { affected: TargetFilter, exclude_source: bool }`; in `combat.rs`,
  extend the requirement loop (334-368) with a second pass that scans `state.restrictions` for
  `MustAttack`, and for each on-battlefield source, requires every creature controlled by
  `restriction.controller` that (a) matches `affected` via layer-resolved characteristics, (b) is
  not the source when `exclude_source`, and (c) is *able* to attack (same tapped/sickness/Defender
  gate already coded at 355-361) to be in `declared_attacker_ids`. Interacts with restrictions per
  CR 508.1d (requirement yields to restriction) — reuse the CantAttackOwner check ordering.
- **Yield = 0** (Rabblemaster is PARTIAL: "+1/+0 for each other attacking Goblin" is a separate
  count-pump gap). Recommend **defer** unless the coordinator wants the infra landed now with a
  standalone unit test.

### P3 — `GameRestriction::CantAttackOwner` (CR 508.1c) — build; 0 immediate yield (Alexios PARTIAL)

- Self-referential restriction registered from the creature's `StaticRestriction`. The affected
  object is the `ActiveRestriction.source`.
- **Enforcement** (`combat.rs`, new block after the goad directional check ~333, before/with the
  requirement pass): for each declared `(attacker_id, AttackTarget::Player(pid))`, if there is an
  on-battlefield `ActiveRestriction { source: attacker_id, restriction: CantAttackOwner }` and
  `pid == state.objects[attacker_id].owner` → `InvalidCommand` (CR 508.1c). Use **owner**, not
  controller (Alexios changes control every upkeep; it can't attack its *owner*).
- Interaction with MustAttackEachCombat (Alexios has both): the must-attack pass at 334-368 must
  treat a creature as "unable to attack a given player" if CantAttackOwner forbids it. Concretely:
  a goaded/must-attack creature whose *only* legal defender is its owner is **not** forced to attack
  (CR 508.1d — obey the requirement only to the extent it doesn't violate the restriction). Add an
  owner-exclusion when computing "is able to attack any legal player."

### P4 — `GameRestriction::CantBeSacrificed` (CR 701.21a) — build; full dispatch wiring; 0 immediate yield (Alexios PARTIAL)

- Self-referential restriction; affected object = `ActiveRestriction.source`.
- **Single choke-point helper** (add to `effects/mod.rs` or a shared `rules` module):
  `fn object_cant_be_sacrificed(state: &GameState, obj_id: ObjectId) -> bool` = true iff some
  `ActiveRestriction { source: obj_id, restriction: CantBeSacrificed }` exists with source on the
  battlefield. Wire it into **every** sacrifice site (full chain per `feedback_verify_full_chain`):

  | # | Site | file:line | Action |
  |---|------|-----------|--------|
  | 1 | `eligible_sacrifice_targets` (covers `Effect::SacrificePermanents` **and** PB-AC2 optional-cost sac via `try_pay_optional_cost`) | `effects/mod.rs:7067-7118` | add `if object_cant_be_sacrificed(state,*id) { return false }` to the filter |
  | 2 | Board-wipe "sacrifice all creatures" selection | `effects/mod.rs:5484-5497` | exclude ids where `object_cant_be_sacrificed` |
  | 3 | Activated-ability cost — sacrifice self (`ability_cost.sacrifice_self`) | `rules/abilities.rs:619` | reject payment if source can't be sacrificed |
  | 4 | Activated-ability cost — sacrifice another (`ability_cost.sacrifice_filter`) | `rules/abilities.rs:734-802` | add to the target-validity check (~795) |
  | 5 | Cast additional cost — Emerge | `rules/casting.rs:~1873-1923` | validate sac target can be sacrificed |
  | 6 | Cast additional cost — Bargain | `rules/casting.rs:~3194-3236` | same |
  | 7 | Cast additional cost — Casualty | `rules/casting.rs:~3242+` | same |
  | 8 | Cast additional cost — Devour (`AdditionalCost::Sacrifice`) | `rules/casting.rs` (devour arm) + `replay_harness.rs:1588` | same |
  | 9 | Forage — "sacrifice a Food" | `rules/abilities.rs:866-913` | skip Food that can't be sacrificed (edge) |
  | 10 | Exploit — "you may sacrifice a creature" (CR 702.110) | resolution of Exploit trigger (grep `Exploit` in `resolution.rs`/`effects`) | exclude can't-be-sac creatures from the choice |
  | 11 | Evoke self-sacrifice on ETB (CR 702.74b) | Evoke sac trigger (`state/stack.rs:195`, resolution) | if evoked creature can't be sacrificed, it stays (edge) |

  Sites 1-2 are the ones that matter for real cards (edicts, board wipes). Sites 3-11 are wired for
  correctness completeness (the reviewer will check the full chain per the memo). If a site proves
  to route through the same `eligible_sacrifice_targets`/`sacrifice_permanents_for_player` helpers,
  document that it's covered transitively rather than duplicating the guard.
- **Semantics**: "can't be sacrificed" means an effect telling the controller to sacrifice it does
  nothing *for that permanent* (it is simply not a legal sacrifice), and a cost that can only be
  paid by sacrificing it can't be paid. It is NOT indestructible and does NOT stop destruction.

### P5 — `Effect::WinGame` (CR 104.1 / 104.2b / 104.3f) — build; 0 immediate yield (all win-cons PARTIAL)

- **Design note (deviation from the brief's `{ condition }`)**: recommend `Effect::WinGame`
  **without** a `condition` field. Gating is already available via (a) `intervening_if` on the
  `Triggered` ability (re-checked at resolution, CR 603.4 — used by Hellkite/Simic upkeep triggers)
  and (b) `Effect::Conditional { condition, then, else }` for inline gates (Thassa's Oracle).
  Adding a `condition` field would duplicate `check_condition` logic and complicate the hash arm.
  If the coordinator prefers the self-contained form, use `Effect::WinGame { condition: Option<Condition> }`
  and evaluate via `check_condition(state, cond, ctx)` at resolution; note that this is redundant
  with existing gating for every roster card. **Planner recommends the no-field form.**
- **Dispatch** (`effects/mod.rs`, new arm near the `Effect::SacrificePermanents` arm ~3085):
  1. Let `winner = ctx.controller` (all roster cards say "*you* win").
  2. **CR 104.3f guard**: if `state.players[winner].has_lost` → do nothing (they can't win while
     lost).
  3. Otherwise, for every `pid != winner` with `!has_lost && !has_conceded`, set
     `has_lost = true` and push `GameEvent::PlayerLost { player: pid, reason: <see below> }`.
  4. Optionally push a `GameEvent::PlayerWon { player: winner }` marker (only if we add that event;
     otherwise the existing `check_game_over` emits `GameOver { winner: Some(winner) }` on the next
     pass since exactly one active player remains).
- **SBA batch discipline (hazard #4)**: this is an EFFECT, resolving atomically. It sets opponents'
  `has_lost` in one shot, then returns. The **existing** post-resolution flow
  (`engine.rs:1621-1691` `is_game_over` / `check_game_over`) finalizes with `GameOver`. Do **NOT**
  add a win check into `sba.rs`. Do not remove players from `state.players` mid-effect.
- **Multiplayer (4-player)**: because step 3 marks **all** opponents (not just the next one), a
  single "you win the game" in a 4-player Commander pod eliminates all 3 opponents and ends the
  game — the correct behavior with the limited-range option absent (CR 104.1). This is the
  mandatory 4-player test below.
- **LossReason**: either add `LossReason::OpponentWonGame` (cleaner; needs the hash arm at
  `hash.rs:1087` + a display arm anywhere `LossReason` is rendered — grep showed **no** TUI/replay
  match on `LossReason`, only construction sites, so churn is limited to `events.rs` + `hash.rs`),
  or reuse an existing reason. Recommend the new variant for auditability.
- **Edge (out of scope, note only)**: Platinum Angel ("you can't lose / opponents can't win") would
  require a can't-win/can't-lose guard — defer (OOS-AC8-2). No roster card needs it.

---

## Engine Changes (summary table)

| # | File | Action |
|---|------|--------|
| 1 | `state/stubs.rs:584` `GameRestriction` | add `MustAttack {…}` (if built), `CantAttackOwner`, `CantBeSacrificed` (NOT `NoMaximumHandSize`) |
| 2 | `state/hash.rs:1876` GameRestriction HashInto | add arms 9/10/11 |
| 3 | `state/hash.rs:5433` Effect HashInto | add `WinGame` arm |
| 4 | `state/hash.rs:1087` LossReason HashInto | add `OpponentWonGame => 5u8` (if new reason) |
| 5 | `state/hash.rs:287` | `HASH_SCHEMA_VERSION` 34 → 35 (+ lib.rs re-export + parity test) |
| 6 | `rules/events.rs:~` `LossReason` | add `OpponentWonGame` variant |
| 7 | `cards/card_definition.rs` `Effect` enum | add `WinGame` variant |
| 8 | `effects/mod.rs` ~3085 (dispatch) | add `Effect::WinGame` arm (marks opponents `has_lost`) |
| 9 | `effects/mod.rs:7067` `eligible_sacrifice_targets` | CantBeSacrificed guard |
| 10 | `effects/mod.rs:5484` board-wipe sac selection | CantBeSacrificed guard |
| 11 | `effects/mod.rs` (new fn) | `object_cant_be_sacrificed` helper |
| 12 | `rules/abilities.rs:619, 734` | CantBeSacrificed at sac-self + sac-filter cost payment |
| 13 | `rules/casting.rs` emerge/bargain/casualty/devour | CantBeSacrificed on sac-target validation |
| 14 | `rules/combat.rs` after 333 | CantAttackOwner restriction check + owner-exclusion in the must-attack pass |
| 15 | `rules/combat.rs:334-368` | (if group MustAttack built) restriction-scan pass |
| 16 | `cards/helpers.rs` prelude | ensure any new type (e.g. WinGame condition) is exported if referenced by card defs |

No `register_static_continuous_effects` change needed — the existing `StaticRestriction` arm
(`replacement.rs:2090`) already registers any `GameRestriction` variant into `state.restrictions`.

---

## Card Definition Fixes (honest roster)

### FULLY unblocked NOW (mis-triaged — fix via EXISTING keywords, no new primitive)

| Card | File | Fix | Note |
|------|------|-----|------|
| **Nezahal, Primal Tide** | `cards/defs/nezahal_primal_tide.rs` | add `AbilityDefinition::Keyword(KeywordAbility::NoMaxHandSize)`; delete the "No maximum hand size static not in DSL" TODO | trigger + discard-exile-return already authored; NoMaxHandSize was the only blocker |
| **Toski, Bearer of Secrets** | `cards/defs/toski_bearer_of_secrets.rs` | add `AbilityDefinition::Keyword(KeywordAbility::MustAttackEachCombat)`; delete the "MustAttack restriction not in DSL" TODO | combat-damage-draw trigger already authored; must-attack was the only blocker |

Both are **added via the mandatory TODO sweep** — their source self-identified as needing the
primitive, but the primitive already exists as a keyword. These are pure re-triage wins.

### PARTIAL — a PB-AC8 primitive helps but a SECOND out-of-scope gap remains (do NOT delete TODOs)

| Card | File | PB-AC8 helps with | Remaining blocker (out of scope) |
|------|------|-------------------|----------------------------------|
| Alexios, Deimos of Kosmos | `alexios_deimos_of_kosmos.rs` | CantBeSacrificed + CantAttackOwner (+ MustAttack already keyword'd) | "each player's upkeep: gain control + untap + +1/+1 + haste" — ForEachPlayer upkeep trigger gap |
| Hellkite Tyrant | `hellkite_tyrant.rs` | WinGame + `Condition::YouControlNOrMoreWithFilter{20, artifacts}` for the upkeep win | "gain control of all artifacts a damaged player controls" — combat-damage gain-control effect gap |
| Simic Ascendancy | `simic_ascendancy.rs` | WinGame + `Condition::SourceHasCounters{growth,20}` for the upkeep win | "put THAT MANY growth counters" — `EffectAmount` reading counters-placed-count gap (ENGINE-BLOCKED, PB-AC1 noted) |
| Goblin Rabblemaster | `goblin_rabblemaster.rs` | group `MustAttack` (if built) | "+1/+0 for each other attacking Goblin" — count-based pump gap |
| Curiosity Crafter | `curiosity_crafter.rs` | `NoMaxHandSize` keyword for the no-max clause | "creature TOKEN deals combat damage" trigger gap |
| Niv-Mizzet, Visionary | `niv_mizzet_visionary.rs` | `NoMaxHandSize` keyword for the no-max clause | "any source you control deals noncombat damage → draw that many" trigger+amount gap |
| Thassa's Oracle | `thassas_oracle.rs` | `Effect::WinGame` for the win clause | ETB "look at top X" + "devotion ≥ library size" comparison gap |
| Call the Spirit Dragons | `call_the_spirit_dragons.rs` | `Effect::WinGame` for the win clause | per-color counter distribution + "5 Dragons this way" counting gap |
| Laboratory Maniac | `laboratory_maniac.rs` | — | draw-from-empty-library **replacement** → win — different mechanism (replacement, not `Effect::WinGame`); out of scope |
| Sea Gate Restoration | `sea_gate_restoration.rs` | — | "no max hand size **for the rest of the game**" — persistent player designation (OOS-AC8-1) |
| Ancient Silver Dragon | `ancient_silver_dragon.rs` | — | same persistent-designation gap + d20 roll |
| Wrenn and Seven | `wrenn_and_seven.rs` | — | emblem "no maximum hand size" — emblem support + persistent designation |
| Howlsquad Heavy | `howlsquad_heavy.rs` | — | combat-trigger token creation + forced attack — needs both |

**For PARTIAL cards, the runner may apply the in-scope half** (e.g. add the `NoMaxHandSize` keyword
to Curiosity Crafter / Niv-Mizzet, or the WinGame-gated upkeep trigger to Hellkite/Simic) **but MUST
leave the remaining TODO/ENGINE-BLOCKED marker in place** describing the still-open gap. Do not mark
these cards "done." Doing the in-scope half is optional and lower priority than landing the two full
wins and the primitives with tests.

---

## Unit Tests

**Files**: `crates/engine/tests/restrictions.rs` (existing GameRestriction tests) and a new/existing
`crates/engine/tests/win_loss.rs` (or `game_end.rs`). Every test cites a VERIFIED CR section and
uses `GameStateBuilder`.

### WinGame (mandatory multiplayer coverage)
- `test_wingame_1v1_controller_wins_opponent_loses` — CR 104.1/104.2b. Resolve `Effect::WinGame`;
  assert opponent `has_lost` and `GameOver { winner }`.
- **`test_wingame_4player_all_three_opponents_lose`** — CR 104.1 (limited-range option absent).
  4-player Commander pod; resolve `Effect::WinGame` for P1; assert P2/P3/P4 all `has_lost`, exactly
  one active player remains, `GameOver { winner: Some(P1) }`. **Mandatory (invariant #5).**
- `test_wingame_controller_already_lost_is_noop` — CR 104.3f. Controller `has_lost = true` before
  resolution; assert WinGame does nothing (no opponent eliminated).
- `test_wingame_hashes_change_on_elimination` — hazard F mutation-verification: hash before vs after
  the WinGame resolution differ; and `assert_eq!(HASH_SCHEMA_VERSION, 35)`.
- `test_wingame_via_intervening_if_upkeep_trigger` — integration using a Hellkite-style
  `Triggered{ intervening_if: YouControlNOrMoreWithFilter{20,artifacts}, effect: WinGame }`: false
  when <20 artifacts, wins at 20. (Validates the no-field WinGame + existing gating design.)

### CantBeSacrificed (full chain)
- `test_cant_be_sacrificed_edict_skips_protected` — CR 701.21a. `Effect::SacrificePermanents{count:1}`
  when the only permanent is protected → nothing sacrificed.
- `test_cant_be_sacrificed_choice_excludes_from_eligible` — protected + one normal permanent →
  `eligible_sacrifice_targets` returns only the normal one.
- `test_cant_be_sacrificed_activation_cost_cannot_pay` — a "Sacrifice a creature:" ability whose only
  creature can't be sacrificed → activation rejected (`abilities.rs:734`).
- `test_cant_be_sacrificed_board_wipe_all_creatures` — the "sacrifice all creatures" path leaves the
  protected creature (`effects/mod.rs:5484`).
- `test_cant_be_sacrificed_negative_normal_permanent_is_sacrificed` — control: without the
  restriction, the permanent is sacrificed normally.

### CantAttackOwner
- `test_cant_attack_owner_illegal_declaration` — CR 508.1c. Attacker with CantAttackOwner declaring
  an attack on its owner → `InvalidCommand`.
- `test_cant_attack_owner_can_attack_other_player` — same attacker attacking a non-owner is legal.
- `test_cant_attack_owner_yields_mustattack_requirement` — CR 508.1d. A must-attack + CantAttackOwner
  creature whose only opponent is its owner is NOT forced to attack (requirement yields to
  restriction). Wedge property = attack target controller, per gotcha #39 discipline.

### MustAttack (only if the group form is built)
- `test_mustattack_group_forces_filtered_creatures` — CR 508.1d. Rabblemaster-style restriction
  forces other Goblins (able to attack) into `declared_attackers`; a non-Goblin and the source are
  not forced.
- `test_mustattack_group_unable_creature_not_forced` — a tapped/summoning-sick Goblin isn't forced.

**Pattern**: follow the goad tests and the Propaganda attack-tax tests in
`crates/engine/tests/restrictions.rs` and the combat tests for the requirement/restriction loop.

---

## Verification Checklist

- [ ] Engine primitives compile (`cargo check`)
- [ ] `HASH_SCHEMA_VERSION` bumped 34→35; parity test uses `assert_eq!`; re-exported in `lib.rs`
- [ ] hash arms added for new GameRestriction variants, `Effect::WinGame`, new `LossReason`
- [ ] CantBeSacrificed guard wired at ALL sites in the P4 table (or transitively-covered ones documented)
- [ ] `Effect::WinGame` marks ALL opponents in 4-player; game-over finalized by existing pass (no `sba.rs` edit)
- [ ] Two full backfills authored (Nezahal, Toski) and their stale TODOs deleted
- [ ] PARTIAL cards: in-scope half optional; remaining markers LEFT IN PLACE
- [ ] `cargo build --workspace` (TUI + replay-viewer expected untouched — verify)
- [ ] Unit tests pass incl. **`test_wingame_4player_all_three_opponents_lose`** (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`), `cargo fmt --check`
- [ ] `card-batch-reviewer` on Nezahal + Toski
- [ ] `authoring-report.py` rerun; coverage delta posted as task comment
- [ ] OOS seeds recorded: OOS-AC8-1 (persistent no-max-hand designation), OOS-AC8-2 (can't-win/can't-lose)

---

## Risks & Edge Cases

- **Yield honesty**: the headline risk is treating PB-AC8 as a "~14 card" batch. It is a
  **prerequisite/infrastructure** batch: 2 mis-triage full wins + 4 primitives with 0 immediate
  full-card yield. Report this explicitly; do not pad the roster.
- **Scope drift toward the co-gaps**: it will be tempting to also build the ForEachPlayer upkeep
  trigger (Alexios), the gain-control-of-artifacts effect (Hellkite), or the multi-counter
  `EffectAmount` (Simic) to "finish" a card. Per `conventions.md` "implement-phase default-to-defer":
  STOP and flag — each is its own micro-PB.
- **CantBeSacrificed half-wiring** (the exact `feedback_verify_full_chain` failure mode): the review
  will check every site in the P4 table. Cost-payment sites (emerge/bargain/casualty/devour, sac-self,
  sac-filter) are easy to miss because they live in `casting.rs`/`abilities.rs`, not `effects/`.
- **owner vs controller** for CantAttackOwner: must key on `owner` (Alexios changes control). A
  controller-based check would be wrong precisely for the one card that needs it.
- **WinGame vs SBA ordering**: adding a win check to `sba.rs` (instead of resolving as an effect)
  would violate CR 704.5 (win-by-effect is not an SBA) and hazard #4. Keep it in `effects/mod.rs`.
- **CR 104.3f**: forgetting the "already lost → no win" guard lets a dying player win.
- **`GameRestriction::NoMaximumHandSize` redundancy**: if built despite the recommendation, ensure
  the cleanup recompute ORs the restriction scan with the keyword check rather than replacing it,
  or Thought Vessel/Reliquary Tower regress.
- **Group MustAttack filter semantics**: "other … you control" needs both an `exclude_source` flag
  and controller scoping; a naive filter would force the source itself or opponents' creatures.
