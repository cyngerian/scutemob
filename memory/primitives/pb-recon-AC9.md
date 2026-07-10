# PB-AC9 Recon — misc & mana (worker, pre-plan)

Per PB-AC8 close-out recommendation: **stale-marker sweep before scoping.**
All five briefed primitives were checked against live code. **None already exist.**
Unlike PB-AC8 (where 3/3 backfilled cards were mis-triaged stale markers), the
AC9 markers appear to name *real* gaps. But co-blocking is heavy — see roster.

## Primitive existence check (recon-first, hazard: do NOT re-add)

| Primitive | Exists? | Evidence |
|---|---|---|
| `Effect::WheelHand` | **NO** | `grep -rE "WheelHand\|DiscardHand"` → 0 hits in `crates/engine/src` |
| multi-output filter mana | **NO** | `ManaAbility` (`state/game_object.rs:165`) has `produces: OrdMap<ManaColor,u32>`, `requires_tap`, `sacrifice_self`, `any_color`, `damage_to_controller`. **No activation mana cost field and no output-mode list.** Filter lands (Mystic Gate: `{W/U}, {T}: Add {W}{W}, {W}{U}, or {U}{U}`) need BOTH. |
| `SearchLibrary` multi-name | **NO** | `TargetFilter` (`cards/card_definition.rs:2687`) has no name field at all. `SearchLibrary` (`:1580`) takes `filter: TargetFilter`. |
| token-doubling replacement | **NO** | No `TokenDoubl*` / token replacement variant. See chokepoint analysis below. |
| d20 + tiered outcome | **NO** | No `d20` / die-roll anything in `rules/`, `effects/`, `state/`. |

## Determinism: the established pattern (criterion 4424 hazard)

**An RNG pattern already exists — extend it, do not invent Command injection.**

`effects/mod.rs` seeds from game state, not entropy:
```rust
// MR-M7-17: use timestamp_counter as seed instead of from_entropy() so
// shuffles are deterministic given the same game state sequence.
let seed = state.timestamp_counter;
let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
zone.shuffle(&mut rng);
```
Sites: `effects/mod.rs:2679`, `:2782`, `:3530`. `Zone::shuffle(&mut impl Rng)` at `state/zone.rs:138`.

Implication for die rolls: a roll seeded from `state.timestamp_counter` is
already replay-deterministic (same state sequence → same roll) and needs **no
new Command variant**, preserving architecture invariants 1–4 (pure library, no
IO/entropy). Two things the planner MUST resolve explicitly:

1. **Seed collision.** `timestamp_counter` may not advance between two rolls in
   one effect resolution (e.g. Ancient Gold Dragon rolls once, but CR 706.3c
   "roll again" and multi-die effects roll N times). Seeding each roll off the
   same counter yields **identical rolls**. Need a per-roll salt (roll index)
   or an explicit `state.roll_counter` that increments per roll.
2. **Hashable state.** If a `roll_counter` (or stored results, CR 706.8a) is
   added to `GameState`/`GameObject`, hazard 1 applies: `state/hash.rs`
   `HashInto` + `HASH_SCHEMA_VERSION` bump + mutation-verified test.

Also emit a `GameEvent` for the roll (invariant 4: all state changes are Events)
— "whenever a player rolls one or more dice" triggers (CR 706.7) will need it.

## Token doubling: the chokepoint problem (hazard 4 — replacement, NOT trigger)

`state/mod.rs:361-369` `add_object()` is a chokepoint for the *`created_token_this_turn`
flag* only — it sees one token at a time, already-decided.

The **count** is decided independently at **8+ `GameEvent::TokenCreated` emission
sites** in `rules/resolution.rs` (`:4739, :4991, :5674, :6348, :6563, :6793, :7697, :7718`).
Doubling Season replaces the *event that would create N tokens* with one creating
2N (CR 614.1 — "shields around" the event; no stack, not a trigger).

Planner must decide: introduce a single `create_tokens(state, controller, spec, count)`
helper that applies token-count replacements once, and refactor all 8 sites through
it — or the primitive will ship half-wired (the exact `feedback_verify_full_chain`
failure mode that PB-AC8 review caught as E1).

Precedents to study: PB-CD counter-doubling, `state/replacement_effect.rs`
(`ControlledBy(PlayerId)` at `:252`, `AnyCreature`+`ControlledBy` combo at `:267`).

## Discriminant / hash baseline (verified from live code, hazard 2)

- `HASH_SCHEMA_VERSION = 35` (`state/hash.rs:296`) → next is **36**
- `KeywordAbility` hash discriminants: max **165** (`Exert`, `state/hash.rs:1029`) → next **166**
- Enums: `KeywordAbility` in `state/types.rs`; `AbilityDefinition` + `Effect` in
  `cards/card_definition.rs:1259`; `StackObjectKind` in `state/stack.rs`
- Note: KW discriminants are assigned **in `hash.rs`**, not as enum `= N` on the
  variant. Do not trust MEMORY.md's "KW 158" — it is stale; 165 is live.

## CR verification (via mtg-rules MCP — brief's refs were advisory)

| Claim | Verdict |
|---|---|
| Die rolls = **CR 706** | ✅ CONFIRMED. 706.1a defines dN / d20 explicitly. |
| Tiered outcomes = results table | ✅ **CR 706.3a** — ranges `N`, `N1–N2`, `N+`. |
| "Roll again" | ✅ CR 706.3c — same kind/number of dice + modifiers. |
| Roll is one ability | ✅ CR 706.3b — roll + modifiers + table are ONE ability (do not split into a reflexive trigger unless oracle says "when you roll"). |
| Ignored roll | CR 706.6 — never happened, no triggers. |
| Token doubling = replacement | ✅ CR 614.1 (continuous replacement, "shields", applies as events happen). Not 614.1c (that's `enters with`). |
| Brief said "605" | ⚠️ 605 is *mana abilities* — relevant to filter mana, not to search. |

## Real card roster (markers verified per-file, NOT trusted from brief)

Brief's discounted yield: ~16. **Actual fully-unblockable count is far lower** —
most candidates carry a second, out-of-scope co-blocking gap (the AC8 pattern).

**Token doubling** (cleanest yield):
- `parallel_lives.rs` — 1 marker, token doubling ONLY → **clean unblock**
- `anointed_procession.rs` — 1 marker, token doubling ONLY → **clean unblock**
- `doubling_season.rs` — 2 markers: token doubling + **counter doubling**. Counter
  doubling = PB-CD; VERIFY whether PB-CD shipped it. If yes → clean unblock.
- `adrix_and_nev_twincasters.rs`, `elspeth_storm_slayer.rs` — **0 markers** (already
  authored?). Verify: may be mis-triaged or may silently lack doubling.

**Wheel** — note the brief conflates two DIFFERENT effects:
- `incendiary_command.rs` — mode 3 "each player discards all cards, draws that many"
  = **true WheelHand**. ENGINE-BLOCKED.
- `reforge_the_soul.rs` — wheel (discard hand, draw 7) but **co-blocked on Miracle**
  (KeywordAbility::Miracle unimplemented) → will NOT fully unblock.
- `winds_of_change.rs`, `echo_of_eons.rs` — **"shuffle hand INTO LIBRARY, draw that
  many"**. This is *not* a discard-based wheel. Needs a distinct effect
  (hand→library shuffle). Planner: decide whether WheelHand covers this via a
  destination param, or scope it out.

**d20 dragons** (heavy co-blocking):
- `ancient_copper_dragon.rs` — 1 marker, d20 → treasure count. **Likely clean unblock.**
- `ancient_gold_dragon.rs` — d20 → token count. Clean IF token-count-from-roll wired.
- `ancient_silver_dragon.rs` — co-blocked: "no maximum hand size **for the rest of the
  game**" permanent player designation (AC8 dropped NoMaximumHandSize as
  already-expressible — verify whether the *permanent/rest-of-game* form exists).
- `ancient_brass_dragon.rs` — co-blocked: variable-count graveyard reanimation with
  cumulative mana-value budget. **Out of scope.**
- `ancient_bronze_dragon.rs` — co-blocked: combat-damage trigger + **reflexive trigger**.

**Search multi-name**:
- `emergency_eject.rs`, `replicating_ring.rs`, `sengir_autocrat.rs` — grep-matched on
  "named"; markers NOT yet confirmed as multi-name search. Planner must read each.

**Filter mana**: grep for filter-mana markers returned unrelated files (matched on
generic "mana"). **No confirmed filter-land card def carries a blocking marker.**
Planner MUST confirm a real card roster exists before building this primitive, or
it is a 0-yield defensive primitive → record as OOS seed instead.

## Recommendation to planner

Order by yield-per-risk:
1. **token doubling** (3-4 clean cards) — but demands the `create_tokens` refactor.
2. **d20 + results table** (1-2 clean cards) — extend the `seed_from_u64` pattern.
3. **WheelHand** (1 clean card: Incendiary Command mode 3).
4. **SearchLibrary multi-name** — confirm roster first.
5. **filter mana** — likely 0 yield; consider OOS seed (AC8 precedent: do not build
   primitives that unblock zero cards).
