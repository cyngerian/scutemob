# Primitive Batch Plan: PB-OS8 — look-at-top-N, place-one, rest-to-bottom (`Effect::LookAtTopThenPlace`)

**Generated**: 2026-07-19
**Primitive**: a new `Effect::LookAtTopThenPlace` (look at the top N cards of a library,
optionally pay an interposed cost, place AT MOST ONE matching card to a zone, send the
rest to another zone deterministically) + a new runtime `TargetFilter.min_cmc_amount`
lower-bound MV cap (mirror of the existing `max_cmc_amount`).
**CR Rules**: 120 / 121 (look/draw), 202.3 / 608.2h (runtime mana value), 118.12 (optional
"you may pay" cost), 601.2 (choose ≤1), 401 (library order), 400.7 (new object on zone
change), 603.3/603.4 (ETB triggers + intervening-if).
**Cards affected**: 6 verified against oracle → **2 flip to Complete**
(`birthing_ritual` inert→Complete, `growing_rites_of_itlimoc` partial→Complete); 4 stay
truthfully-marked (`birthing_pod`, `muxus_goblin_grandee`, and sweep cards) with a named,
verified remaining blocker. `min_cmc_amount` ships task-directed + unit-tested but its
intended live card (`birthing_pod`) does NOT flip this PB (Phyrexian-mana-in-activated-cost
blocker discovered — new seed OOS-OS8-1).
**Dependencies**: PB-EF10 (`TargetFilter.max_cmc_amount`, `EffectAmount::ManaValueOfSacrificedCreature`,
`try_pay_optional_cost` / `Cost::Sacrifice` LKI capture) — present. PB-22 S3
(`Effect::RevealAndRoute`, `ZoneTarget`, `LibraryPosition`) — present. PB-EF5
(`Effect::TransformSelf`, growing_rites end step) + PB-OS4b (face-aware ability gathering) — present.
**Deferred items from prior PBs**: OOS-EF10-1 (this PB's seed) → close. PB-OS6 deferred
sub-primitive (d) growing_rites ETB → close. (OS6 plan §"Scope decisions (d)" folded here.)

---

## TODO sweep (roster-recall gate — MANDATORY)

Ran the pre-existing-TODO / blocker-comment sweep across `crates/card-defs/src/defs/` for
the LookAtTopThenPlace family ("look at (the) top", "rest on the bottom", "from among them",
"OOS-EF10", "PB-OS8", "LookAtTopThenPlace"). 67 files reference the family; the tell-tale
"rest on the bottom" remainder clause narrows to the put-≤1 / put-multiple candidates below.
**Forced adds (self-identified as needing PB-OS8 / LookAtTopThenPlace):**

- `birthing_ritual.rs` — TODO(OOS-EF10-1) names this exact primitive. **Forced add → flips.**
- `growing_rites_of_itlimoc.rs` — completeness note + TODO cite "PB-OS8 / Effect::LookAtTopThenPlace"
  (re-pointed in OS6). **Forced add → flips.**
- `muxus_goblin_grandee.rs` — note cites "OOS-EF10 / PB-OS8". **Verified NOT this primitive**
  (put-MULTIPLE; see §Verdicts). Misattributed marker → re-point to `RevealAndRoute`.
- `birthing_pod.rs` — note cites the `min_cmc_amount` follow-up "alongside OOS-EF10-1".
  **Verified stays partial** (second blocker: Phyrexian mana in an activated cost).

Sweep result: 2 forced-add flips; 2 forced-add cards verified stay-partial with named delta;
the remaining sweep files (`carth_the_lion`, `nissa_resurgent_animist`, `narset_parter_of_veils`,
`harald_king_of_skemfar`, `bounty_of_skemfar`, `satoru_umezawa`, `dragonlord_ojutai`,
`stock_up`, `telling_time`, `goblin_ringleader`, `sylvan_messenger`, …) each carry an
independent blocker or a non-matching dig shape (see §Sweep verdicts) and stay partial.

---

## Key architecture finding — `Effect::RevealAndRoute` already exists

`Effect::RevealAndRoute { player, count, filter, matched_dest, unmatched_dest }`
(card_definition.rs:2020; dispatch effects/mod.rs:4957; hash disc 62) already:
reveals/looks at the top N (`zone.object_ids().take(n)`), partitions by `filter`, moves
**ALL** matched to `matched_dest`, unmatched to `unmatched_dest`, deterministic ObjectId-ascending.
It is used by `narset_parter_of_veils` and `bounty_of_skemfar`.

`LookAtTopThenPlace` is the **put-AT-MOST-ONE** sibling of `RevealAndRoute`, plus two things
`RevealAndRoute` lacks: (a) an **interposed `place_cost`** paid after the look (Birthing
Ritual's sacrifice, which both gates and parameterizes the placement), and (b) honoring the
**runtime** `max_cmc_amount` / `min_cmc_amount` caps (resolved from `ctx`). A new variant is
warranted (rather than extending `RevealAndRoute`) because: the whole PB-OS queue, OS6 plan,
growing_rites' note, and the retriage doc all name `Effect::LookAtTopThenPlace` by design; and
bolting `place_cost` + ≤1 semantics onto `RevealAndRoute` would change the semantics of the
2 cards already using it. **Model the new executor directly on `RevealAndRoute`'s proven code**
(reuse `resolve_zone_target`, `dest_tapped`, `zone_move_event`, the `object_ids().take(n)`
top-N convention, and ObjectId-ascending determinism).

> **RevealAndRoute vs LookAtTopThenPlace is the reveal(public)/look(private) axis on paper,
> but the engine has NO privacy layer yet** — `GameEvent::private_to()` does not exist
> (grep: 0 hits; it is the deferred M10 design in invariant #7). `RevealAndRoute`/`Scry`/
> `SearchLibrary` all emit ordinary zone-move events and leak no identities today. So
> `LookAtTopThenPlace` **emits NO distinct "look" event and no card identities** — same
> privacy posture as the existing look/reveal family; hidden-info filtering is deferred to
> M10's event-broadcast filter exactly as it is for those effects. Do NOT invent a
> `private_to` mechanism in this PB.

---

## CR Rule Text (from MCP + card oracles)

- **Birthing Ritual** ({1}{G} Enchantment): "At the beginning of your end step, if you
  control a creature, look at the top seven cards of your library. Then you may sacrifice a
  creature. If you do, you may put a creature card with mana value X or less from among those
  cards onto the battlefield, where X is 1 plus the sacrificed creature's mana value. Put the
  rest on the bottom of your library in a random order." Rulings: X uses the LKI mana value of
  the sacrificed creature (2024-06-07); {X}-cost cards among the seven have MV computed with
  X=0; if the intervening-if fails at resolution the ability does nothing.
- **Growing Rites of Itlimoc** (front): "When ~ enters, look at the top four cards of your
  library. You may reveal a creature card from among them and put it into your hand. Put the
  rest on the bottom of your library in any order." (End-step transform-if-4-creatures already
  shipped via PB-EF5; back face Itlimoc mana abilities shipped.)
- **Birthing Pod** ({3}{G/P} Artifact): "{1}{G/P}, {T}, Sacrifice a creature: Search your
  library for a creature card with mana value equal to 1 plus the sacrificed creature's mana
  value, put that card onto the battlefield, then shuffle. Activate only as a sorcery."
  → an activated **tutor** (whole-library SearchLibrary), **not** a top-N dig.
- **Muxus, Goblin Grandee** (ETB): "reveal the top six cards of your library. Put all Goblin
  creature cards with mana value 5 or less from among them onto the battlefield and the rest
  on the bottom of your library in a random order." → put-**MULTIPLE** by filter = exactly
  `RevealAndRoute`, NOT `LookAtTopThenPlace`.

---

## Library top/bottom convention (load-bearing — read before implementing)

`Zone::Ordered(Vector<ObjectId>)`. Authoritative "top of library" is `zone.top()` = `v.last()`
(used by `draw_card` turn_actions.rs:1195 and Herald's Horn peek). `push_front` = index 0 =
**bottom** (move_object_to_zone comment mod.rs:1787: "front (= bottom)").

**However**, the look/reveal family (`RevealAndRoute` effects/mod.rs:4969, `Scry` :3066) treats
"top N" as `zone.object_ids().take(n)` = the FIRST n = the bottom end. This is internally
inconsistent with `draw_card`'s `top()=last()`, a **pre-existing discrepancy** that is OUT OF
SCOPE here. **Model `LookAtTopThenPlace` on `RevealAndRoute` (`object_ids().take(n)`)** so the
new effect is consistent with the shipped effect it derives from and the cards already using
it. In tests, place the intended cards at `object_ids()[0..n]` (the positions the executor
picks) and assert against actual behavior — do NOT try to reconcile with `draw_card`. Logged as
Risk R1.

---

## Engine Changes

### Change 1 — `Effect::LookAtTopThenPlace` variant (card DSL)

**File**: `crates/card-types/src/cards/card_definition.rs` — add to `enum Effect`
**immediately after `RevealAndRoute` (line 2020)**:

```rust
/// CR 120/601.2 + CR 202.3/608.2h: Look at the top `count` cards of `player`'s library,
/// optionally pay an interposed `place_cost` (CR 118.12), place AT MOST ONE card matching
/// `filter` (honoring runtime `max_cmc_amount`/`min_cmc_amount`) to `destination`, and send
/// the remaining looked-at cards to `rest_to`. The put-≤1 sibling of `Effect::RevealAndRoute`
/// (which routes ALL matches and has no cost/gate); distinct from `Effect::SearchLibrary`
/// (whole library, not a top-N subset).
LookAtTopThenPlace {
    player: PlayerTarget,
    count: EffectAmount,
    filter: TargetFilter,
    /// Optional cost paid AFTER the look, BEFORE placing (deterministic "pay when able",
    /// CR 118.12). When Some, placement happens only if paid; a `Cost::Sacrifice` here
    /// populates `ctx.sacrificed_creature_lki` so `filter`'s `*_cmc_amount = 1 + sac MV`
    /// resolves (Birthing Ritual). None = no interposed cost (Growing Rites).
    place_cost: Option<Box<Cost>>,
    /// Placed-card destination. Battlefield{tapped} → emits PermanentEnteredBattlefield
    /// (ETB triggers fire). Hand{owner} for "into your hand".
    destination: ZoneTarget,
    /// Remainder destination — `ZoneTarget::Library { owner, position: Bottom }`.
    /// Deterministic ObjectId-ascending order approximates "in a random order"/"in any order".
    rest_to: ZoneTarget,
    /// Whether placing is a "may" (M7 fallback places the best candidate when one exists;
    /// reserved for M10+ interactive decline).
    optional: bool,
},
```

`Box<Cost>` avoids `large_enum_variant` clippy (gotcha: box `Option<Cost>` on stack-ish
enums). `Cost`, `ZoneTarget`, `PlayerTarget`, `EffectAmount`, `TargetFilter` are all already
in the SR-8 PROTOCOL closure and already `HashInto` — no NEW type joins the closure.

### Change 2 — `TargetFilter.min_cmc_amount` runtime lower-bound (card DSL)

**File**: `crates/card-types/src/cards/card_definition.rs` — add to `struct TargetFilter`
**immediately after `max_cmc_amount` (line 3044)**:

```rust
/// Runtime-computed MIN mana value cap (inclusive). None = no runtime floor. CR 202.3/608.2h.
/// Mirror of `max_cmc_amount`: resolved from `EffectContext` at execution, therefore honored
/// ONLY by executors that hold `ctx` (`Effect::SearchLibrary`, `Effect::LookAtTopThenPlace`),
/// NOT by `matches_filter`. Paired with `max_cmc_amount` set to the SAME amount, expresses
/// "mana value EQUAL TO N + the sacrificed creature's mana value" (Birthing Pod).
#[serde(default)]
pub min_cmc_amount: Option<Box<EffectAmount>>,
```
(Note: a STATIC `min_cmc: Option<u32>` already exists at line 3047 — the new field is the
RUNTIME analog, distinct.)

### Change 3 — `LookAtTopThenPlace` resolution logic (executor)

**File**: `crates/engine/src/effects/mod.rs` — new match arm in `execute_effect_inner`,
placed next to `Effect::RevealAndRoute` (~line 4957). **CR 120/601.2/118.12/202.3/400.7.**
Algorithm (per resolved player from `resolve_player_target_list(state, player, ctx)`):

1. `let n = resolve_amount(state, count, ctx).max(0) as usize;`
2. Collect the top-N looked-at set: `let top_ids: Vec<ObjectId> = state.zones.get(&Library(p))
   .map(|z| z.object_ids()).unwrap_or_default().into_iter().take(n).collect();`
   (identical to `RevealAndRoute`). If empty, `continue`.
3. **Interposed cost** (CR 118.12): if `place_cost` is `Some(cost)`:
   `match try_pay_optional_cost(state, p, cost, Some(ctx.source), events) {`
   - `Some(lki) =>` set `ctx.sacrifice_fired = !lki.is_empty(); ctx.sacrificed_creature_lki = lki;`
     (so `filter`'s `*_cmc_amount` referencing `ManaValueOfSacrificedCreature` resolve) →
     proceed to placement.
   - `None =>` cost not paid: **skip placement**, fall through to bottoming all of `top_ids`.
   `}` If `place_cost` is `None`: proceed to placement unconditionally.
4. **Resolve runtime caps once** (outside the candidate loop; they don't depend on the card):
   `let max_cap = filter.max_cmc_amount.as_ref().map(|a| resolve_amount(state, a, ctx));`
   `let min_cap = filter.min_cmc_amount.as_ref().map(|a| resolve_amount(state, a, ctx));`
5. **Placement (≤1)** — among `top_ids`, keep candidates where
   `matches_filter(&obj.characteristics, filter) && check_has_counter_type(obj, filter)`
   AND `max_cap.is_none_or(|c| mv(obj) <= c)` AND `min_cap.is_none_or(|c| mv(obj) >= c)`
   (mirror SearchLibrary effects/mod.rs:2981-2988 for the mv() helper on
   `obj.characteristics.mana_cost`). Pick the deterministic winner `min_by_key(|id| id.0)`
   (consistent with `RevealAndRoute`/`SearchLibrary`). If a winner exists AND placement is
   allowed (step 3): `let dest = resolve_zone_target(destination, state, ctx);` move via
   `expect_move_object_to_zone`; apply `dest_tapped(destination)` if `Some(tap)`; push
   `zone_move_event(ctx.controller, old, new, dest)` (this yields `PermanentEnteredBattlefield`
   for Battlefield → ETB triggers fire for Birthing Ritual's placed creature). Record the
   placed `old` id so it is excluded from bottoming.
6. **Bottom the rest** (CR 401): `let rest_zone = resolve_zone_target(rest_to, state, ctx);`
   collect `top_ids` minus the placed id, `sort_by_key(|id| id.0)`, and for each
   `expect_move_object_to_zone(id, rest_zone)` pushing `zone_move_event`. `rest_to` MUST be
   `ZoneTarget::Library { position: Bottom }`; `move_object_to_zone` front-inserts (=bottom),
   matching `RevealAndRoute`'s unmatched path (Top not supported by this move path — Risk R2).

Deterministic-randomness note (CR 401.3): the "random order"/"any order" of the remainder is
realized as ObjectId-ascending placement — the M7 deterministic precedent already used by
`RevealAndRoute`, `Scry`, and `PutOnLibrary` (NO `rand`/RNG introduced; the only RNG precedent,
`SearchLibrary`'s `shuffle_before_placing`, is deliberately NOT used).

### Change 4 — honor `min_cmc_amount` in `Effect::SearchLibrary`

**File**: `crates/engine/src/effects/mod.rs` — in `Effect::SearchLibrary` (line 2941), add a
runtime **min** cap alongside the existing `runtime_cap` (max) at 2953-2956 and the candidate
filter at 2981-2988: resolve `filter.min_cmc_amount` once, then AND
`min_runtime_cap.is_none_or(|c| mv(obj) >= c)` into the candidate predicate. (Needed so the
new field is honored wherever `max_cmc_amount` is; used by Birthing Pod's def even though the
card stays partial for a separate reason.)

### Change 5 — exhaustive match / wire updates

| File | Match | Action |
|------|-------|--------|
| `crates/engine/src/state/hash.rs` | `Effect` HashInto (ends L6844, exhaustive, NO catch-all) | add `Effect::LookAtTopThenPlace { player, count, filter, place_cost, destination, rest_to, optional } => { 96u8.hash_into(hasher); … hash each field }` — **next free Effect discriminant is 96** (current max 95 = RemoveFromCombat) |
| `crates/engine/src/state/hash.rs` | `TargetFilter` HashInto (L5246, after `max_cmc_amount`) | `self.min_cmc_amount.hash_into(hasher);` |
| `crates/engine/src/effects/mod.rs` | `execute_effect_inner` `Effect` match (exhaustive) | Change 3 dispatch arm |
| `crates/engine/src/effects/mod.rs` | `Effect::SearchLibrary` | Change 4 min-cap |
| `crates/engine/src/rules/*` (abilities.rs / resolution.rs) | any exhaustive `Effect` classifier | `cargo build --workspace` flags; `RevealAndRoute` has no such extra arm, so likely none — verify |
| `crates/engine/tests/primitives/pb_os1_gain_control_reversion.rs` | test helper `collect_gain_control_durations` matches `Effect` | verify it has a `_ =>` catch-all (it recurses `MayPayThenEffect`/`Sequence`); add no arm if catch-all present |
| `crates/engine/src/testing/replay_harness.rs` | JSON `Effect` lowering | NOT required — no golden script uses `LookAtTopThenPlace`; add only if compile demands |

> **Tools** (`tools/replay-viewer/src/view_model.rs`, `tools/tui/src/play/panels/stack_view.rs`)
> exhaustively match `StackObjectKind` + `KeywordAbility`, **not** `Effect`/`TargetFilter`
> (verified: `RevealAndRoute`/`RemoveFromCombat` have no tool arm). No tool edits expected —
> still run `cargo build --workspace` (the #1 recurring miss).

## WIRE ANALYSIS (SR-8) — single batched PROTOCOL 22→23, HASH 59→60 (both forced)

**PROTOCOL closure** (type COUNT unchanged at 90 — no new type joins; existing closure enums/
structs change shape): `Effect` gains 1 variant (`LookAtTopThenPlace`); `TargetFilter` gains 1
field (`min_cmc_amount`). Both `Effect` and `TargetFilter` are in the closure → the declared-
shape digest moves → PROTOCOL bump forced.

**HASH**: 2 new `HashInto` arms (Effect variant, TargetFilter field) → HASH digest moves → bump
forced. (No `GameState`/`PlayerState`/`EffectContext` field added; `sacrifice_fired` /
`sacrificed_creature_lki` are existing non-serialized ctx scratch — reused, not added.)

**Sentinel / fingerprint sites the runner MUST update** (get the printed digests by running the
failing gate tests):

*PROTOCOL:* `crates/engine/src/rules/protocol.rs` — `PROTOCOL_VERSION` 22→23 (L213); add a
`- 23: PB-OS8 …` History line; re-pin `PROTOCOL_SCHEMA_FINGERPRINT` (L230) to the value printed
by `protocol_schema_fingerprint_is_pinned`; **APPEND** (never edit) a `PROTOCOL_HISTORY` row.
`crates/engine/tests/core/protocol_schema.rs` — `protocol_version_sentinel` (L872) 22→23; re-pin
`FROZEN_HISTORY_PREFIX_DIGEST` after the append. Other `PROTOCOL_VERSION, 22` sentinels to bump
(`rg "PROTOCOL_VERSION, 22" crates/engine/tests`): `pb_os6_dfc_flip_conditions.rs:874`,
`pb_ef7_modal_activated.rs:242`, `pb_os5_relative_attacker_count.rs:716`,
`pb_ef10_sacrifice_driven_amounts.rs:1595`, `pb_ef12_any_color_choice.rs:363`,
`pb_os7_defending_player_continuous_filter.rs:697`, `core/protocol_schema.rs:872`.

*HASH:* `crates/engine/src/state/hash.rs` — `HASH_SCHEMA_VERSION` 59→60; add a `- 60:` History
line; APPEND a `HASH_SCHEMA_HISTORY` row with the new fingerprints; update the module doc
comment (L433/501 style) noting `LookAtTopThenPlace` disc 96 + `TargetFilter.min_cmc_amount`.
`crates/engine/tests/core/hash_schema.rs` — `hash_schema_version_sentinel` 59→60; re-pin its
`FROZEN_HISTORY_PREFIX_DIGEST`. All `HASH_SCHEMA_VERSION, 59u8` sentinels
(`rg "HASH_SCHEMA_VERSION, 59u8" crates/engine/tests` — ~46 files including
`pb_os7_defending_player_continuous_filter.rs:692`, `pb_os6_dfc_flip_conditions.rs:878`,
`pbp_power_of_sacrificed_creature.rs:787`, `optional_cost_and_counter_tax.rs:1139`,
`effect_sacrifice_permanents_filter.rs:136`, `loyalty_target_validation.rs:355`, and the full
PB-EF/PB-AC/PB-OS/primitive_pb_* set) → bump each to `60u8`. The new test file asserts 23/60u8.

---

## Card Definition Fixes

### birthing_ritual.rs (inert → **Complete**) — FLIP
**Oracle**: see §CR. **Fix**: replace the empty `abilities` with the end-step trigger:
```rust
AbilityDefinition::Triggered {
    once_per_turn: false,
    trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
    intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
        count: 1,
        filter: TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() },
    }),
    effect: Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(7),
        place_cost: Some(Box::new(Cost::Sacrifice(TargetFilter {
            has_card_type: Some(CardType::Creature), ..Default::default() }))),
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            max_cmc_amount: Some(Box::new(EffectAmount::Sum(
                Box::new(EffectAmount::Fixed(1)),
                Box::new(EffectAmount::ManaValueOfSacrificedCreature)))),
            ..Default::default()
        },
        destination: ZoneTarget::Battlefield { tapped: false },
        rest_to: ZoneTarget::Library { owner: PlayerTarget::Controller, position: LibraryPosition::Bottom },
        optional: true,
    },
    targets: vec![], modes: None, trigger_zone: None,
}
```
Set `completeness: Completeness::Complete`; rewrite the header/inline comments to describe the
shipped behavior (drop the OOS-EF10-1 TODO). **Faithfulness note** (document in the def):
`place_cost` uses the deterministic "pay when able" convention (CR 118.12, invariant #9) — it
auto-sacrifices whenever a creature is available (even into a whiff), the same M7 limitation
every `MayPayThenEffect` Complete card carries; the intervening-if re-check at resolution and
the {X}=0 MV rule are handled by existing infrastructure (`ManaValueOfSacrificedCreature`).

### growing_rites_of_itlimoc.rs (partial → **Complete**) — FLIP
**Fix**: replace the ETB TODO (line 58-61) with an ETB trigger whose effect is:
```rust
Effect::LookAtTopThenPlace {
    player: PlayerTarget::Controller,
    count: EffectAmount::Fixed(4),
    place_cost: None,
    filter: TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() },
    destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
    rest_to: ZoneTarget::Library { owner: PlayerTarget::Controller, position: LibraryPosition::Bottom },
    optional: true,
}
```
Use the same ETB `TriggerCondition` the codebase uses for "When ~ enters" self-triggers (mirror
a neighboring def; SelfEntersBattlefield / WhenEntersBattlefield). Keep the `KeywordAbility::Transform`,
the end-step `TransformSelf` trigger, and both back-face mana abilities as-is. Set
`completeness: Completeness::Complete`; drop the PB-OS8 TODO/note.

### birthing_pod.rs (stays **partial** — NEW blocker, doc + note update)
**Verdict**: NOT a `LookAtTopThenPlace` card — it is an activated **whole-library tutor**
(`Effect::SearchLibrary`). With `min_cmc_amount` it CAN express the exact-MV filter (max =
min = `Sum(Fixed(1), ManaValueOfSacrificedCreature)`), and the activated-ability sacrifice
cost already captures the sacrificed-creature LKI (abilities.rs:1046 → StackObject :1305 → ctx).
**BUT a second, out-of-scope blocker exists**: its cost `{1}{G/P}` has a Phyrexian pip, and
**Phyrexian mana is not handled in the activated-ability payment path** — `rules/abilities.rs`
has ZERO phyrexian references; the "pay {G/P} with 2 life" logic lives only in `rules/casting.rs`
(spell casting). The activated path pays via `mana_pool.can_spend/spend` (abilities.rs:750-753),
which cannot offer the 2-life alternative. Authoring the cost as plain `{1}{G}` would ship wrong
game state (removes the life-payment option). → **stays partial.** **Action**: update the
completeness note to name BOTH blockers (min_cmc_amount now available; Phyrexian-mana-in-
activated-cost still missing) and cite new seed **OOS-OS8-1**. Do NOT author the ability.

### muxus_goblin_grandee.rs (stays **partial** — misattributed marker, doc-only re-point)
**Verdict**: the ETB puts **ALL** Goblin creatures MV≤5 → this is `Effect::RevealAndRoute`
(existing), NOT `LookAtTopThenPlace` (put-≤1). `TargetFilter` already has a STATIC `max_cmc:
Some(5)` and `has_subtype`/`has_card_type`, so the ETB is authorable via `RevealAndRoute {
count: 6, filter: {has_subtype: Goblin, has_card_type: Creature, max_cmc: Some(5)}, matched_dest:
Battlefield{tapped:false}, unmatched_dest: Library{Bottom} }` — using an already-shipped
primitive. **Action (this PB, doc-only)**: re-point muxus's TODO + completeness note from
"OOS-EF10 / PB-OS8" to cite `Effect::RevealAndRoute` as the closing primitive, and file note
**OOS-OS8-2** (muxus ETB is a `RevealAndRoute` instance — authorable in a small follow-up).
Per implement-phase-default-to-defer, do NOT author the ETB here (RevealAndRoute is not this
PB's primitive and its usage needs its own review pass).

---

## Sweep verdicts (remaining put-≤1 / look-family cards — all STAY PARTIAL this PB)

| Card | Shape | Blocker beyond the dig → verdict |
|------|-------|----------------------------------|
| `carth_the_lion` (inert) | look 7, may put ≤1 planeswalker → hand | ALSO needs `WheneverPlaneswalkerYouControlDies` TriggerCondition + a loyalty-cost-modifier static → stays inert |
| `nissa_resurgent_animist` (partial) | reveal-UNTIL-match (variable count) | different dig shape (reveal-until, not look-top-N) + "second resolution this turn" per-ability counter → stays partial |
| `narset_parter_of_veils` (partial) | look 4, may put ≤1 noncreature/nonland → hand | currently uses `RevealAndRoute` (over-puts ALL matches = legal-but-wrong on ≤1); `LookAtTopThenPlace` would CORRECT the -2, but the card stays partial on "opponents draw ≤1" (`MaxCardsDrawnPerTurn`). Optional correctness fix (no status change) — DEFER |
| `harald_king_of_skemfar` (partial) | look 5, may put ≤1 → hand | needs subtype-OR-name UNION filter ("Elf, Warrior, or Tyvar") — TargetFilter ANDs; different primitive → stays partial |
| `bounty_of_skemfar` (known_wrong) | put ≤1 land→BF AND ≤1 Elf→hand | dual-destination multi-place from one look — not covered by single-destination ≤1 → stays known_wrong |
| `satoru_umezawa` (inert) | look 3, put 1 → hand | ALSO `WheneverYouActivateNinjutsu` trigger + grant-ninjutsu-to-hand static → stays inert |
| `dragonlord_ojutai`, `stock_up`, `telling_time`, `goblin_ringleader`, `sylvan_messenger` | split-destination / put-multiple / other-clause | verify at implement — each carries an independent blocker; expected stay-partial |

The runner should re-confirm each at implement time (per the "NOW-EXPRESSIBLE needs per-card
verification" gotcha) but should NOT expand the shipped roster beyond birthing_ritual +
growing_rites without escalating (implement-phase-default-to-defer).

---

## Unit Tests

**New file**: `crates/engine/tests/primitives/pb_os8_look_at_top_then_place.rs` (add its `mod`
line to `tests/primitives/main.rs` per SR-9a — never a top-level `tests/*.rs`). All probes drive
real effect execution and assert observable state (SR-34), NOT source-tracing. Set up libraries
so the intended cards occupy `object_ids()[0..n]` (the `take(n)` window — see §convention).

Decoy-paired tests:
- `test_look_place_creature_to_hand_growing_rites` — 4-card top with 1 creature; execute
  `LookAtTopThenPlace{count:4, filter:creature, dest:Hand, place_cost:None, optional}`; assert
  the creature is in hand (by name) and the other 3 are at the bottom of the library.
- `test_look_place_no_match_leaves_all_bottomed` — **decoy**: top 4 has NO creature; assert
  nothing placed, all 4 bottomed (deterministic ObjectId order), hand unchanged.
- `test_look_place_at_most_one_even_when_two_match` — **decoy vs RevealAndRoute**: top N has 2
  creatures; assert EXACTLY ONE placed (the min-ObjectId) and the other creature is bottomed
  (this is the whole point vs `RevealAndRoute`, which would place both).
- `test_look_place_onto_battlefield_fires_etb` — dest:Battlefield; assert the placed creature is
  on the battlefield AND a `PermanentEnteredBattlefield` event was emitted (ETB path).
- `test_look_place_cost_sacrifice_gates_and_parameterizes` (Birthing Ritual core) — battlefield
  has a MV-2 creature to sacrifice; top 7 has a MV-3 creature (≤ 1+2) and a MV-5 creature (> 3);
  `place_cost:Sacrifice(creature)`, `filter.max_cmc_amount = 1 + ManaValueOfSacrificedCreature`;
  assert the sac creature is in graveyard, the MV-3 creature entered the battlefield, the MV-5
  did NOT (runtime cap), rest bottomed.
- `test_look_place_cost_declined_when_unpayable_skips_placement` — **decoy**: `place_cost:
  Sacrifice(creature)` but controller has NO creature to sacrifice; assert nothing placed AND
  all N bottomed (cost `None` path), even though a matching card was in the top N.
- `test_min_cmc_amount_caps_search_by_runtime_floor` — direct `SearchLibrary` with
  `min_cmc_amount = Fixed(3)` (+ a max), library has MV-2 and MV-4 creatures; assert only the
  MV-4 is fetched (mirrors PB-EF10 `test_search_max_cmc_amount_caps_by_runtime_value`).
- `test_look_place_min_and_max_equal_exact_mv` — `LookAtTopThenPlace` with min=max=Fixed(3);
  top N has MV-2/MV-3/MV-4 creatures; assert only the MV-3 is placeable.
- Card-integration: `test_birthing_ritual_end_step_flip` — build a game, control a creature,
  Birthing Ritual on battlefield, advance to the controller's end step, resolve the trigger,
  assert the placement + bottoming per the oracle. `test_growing_rites_etb_look_four` — cast/ETB
  growing_rites, assert the creature-to-hand + bottoming.
- Hash soundness: `test_lookattopthenplace_hashes_distinctly` (vs `RevealAndRoute` and a
  min/max-swapped filter); `test_min_cmc_amount_hashes_distinctly` (vs `max_cmc_amount` and
  `Fixed(0)`), mirroring PB-EF10's hash tests.
- Sentinels: `assert_eq!(PROTOCOL_VERSION, 23)` and `assert_eq!(HASH_SCHEMA_VERSION, 60u8)` with
  a comment naming PB-OS8 + the two closure moves (Effect variant, TargetFilter field).

**Patterns**: `tests/primitives/pb_ef10_sacrifice_driven_amounts.rs` (runtime cmc cap +
sacrifice LKI + sentinels), the `RevealAndRoute` usage in narset/bounty for library setup, and
`tests/primitives/pb_os6_dfc_flip_conditions.rs` (end-step-trigger card integration + sentinels).

---

## Verification Checklist (numbered — runner executes in order)

1. Add `Effect::LookAtTopThenPlace` variant (Change 1) + `TargetFilter.min_cmc_amount` (Change 2)
   in `card_definition.rs`. `cargo check -p mtg-card-types`.
2. Add the `Effect::LookAtTopThenPlace` HashInto arm (disc **96**) + `min_cmc_amount` HashInto
   line (Change 5) in `state/hash.rs`.
3. Implement the executor (Change 3) in `effects/mod.rs`, reusing `try_pay_optional_cost`,
   `resolve_zone_target`, `dest_tapped`, `zone_move_event`, `object_ids().take(n)`,
   ObjectId-ascending determinism.
4. Add the `min_cmc_amount` runtime floor to `Effect::SearchLibrary` (Change 4).
5. `cargo build --workspace` — resolve every exhaustive-`Effect`-match compile error (incl. the
   pb_os1 test helper if it lacks a catch-all); confirm NO tool (replay-viewer/TUI) arm needed.
6. Author the two card flips: `birthing_ritual.rs` (inert→Complete), `growing_rites_of_itlimoc.rs`
   (partial→Complete). `cargo check -p mtg-card-defs`.
7. Doc-only updates: `birthing_pod.rs` note (two blockers + OOS-OS8-1), `muxus_goblin_grandee.rs`
   note (re-point to `RevealAndRoute` + OOS-OS8-2). No behavioral change to these two.
8. Bump PROTOCOL 22→23 (+ History append + fingerprint re-pin) and HASH 59→60 (+ History append
   + fingerprint re-pin) in lockstep; run `core protocol_schema` + `core hash_schema` and paste
   the printed digests; bump ALL `PROTOCOL_VERSION, 22` and `HASH_SCHEMA_VERSION, 59u8` sentinels
   (grep-driven).
9. Write `tests/primitives/pb_os8_look_at_top_then_place.rs` (+ `mod` line in `primitives/main.rs`).
10. `cargo test --all` (incl. `core card_defs_fmt` / `tools/check-defs-fmt.sh`, SR-35).
11. `cargo clippy -- -D warnings`; `cargo fmt --check` + `tools/check-defs-fmt.sh`.
12. Close-out: OOS-EF10-1 → CLOSED in `ef-batch-plan-2026-07-17.md` §12 + `oos-retriage-plan-2026-07-18.md`
    §3 PB-OS8; PB-OS6 deferred (d) → CLOSED (growing_rites Complete); file OOS-OS8-1 (Phyrexian
    mana in activated costs) + OOS-OS8-2 (muxus via RevealAndRoute) in the retriage doc +
    workstream-state "Last Handoff". Confirm `all_cards()` still enumerates both flipped defs.
13. No remaining `LookAtTopThenPlace`/OOS-EF10-1 TODOs in the two flipped defs; birthing_pod/muxus
    notes reworded (not removed).

---

## Risks & Edge Cases

- **R1 — top/bottom convention**: `LookAtTopThenPlace` uses `object_ids().take(n)` (bottom-end),
  consistent with `RevealAndRoute`/`Scry` but inconsistent with `draw_card`'s `top()=last()`.
  Pre-existing discrepancy; DO NOT fix here. Tests must place cards at `object_ids()[0..n]`.
- **R2 — `rest_to` must be `Library{Bottom}`**: `move_object_to_zone` front-inserts (=bottom).
  `Library{Top}` is not honored by this move path (same limitation as `RevealAndRoute`). No
  candidate needs Top; assert Bottom-only in the def.
- **R3 — deterministic "pay when able"** (birthing_ritual `place_cost`): auto-sacrifices whenever
  a creature is available, even into a whiff. Legal + replayable (invariant #9), identical to
  every `MayPayThenEffect` Complete card. Document in the def; do NOT try to make it optional.
- **R4 — ETB on battlefield placement**: `zone_move_event`→`PermanentEnteredBattlefield` fires
  ETB triggers for the placed creature (correct for Birthing Ritual). Ensure the resolution
  caller runs `check_triggers`/`flush` after the effect (the standard resolution path does).
- **R5 — `min_cmc_amount` ships without a flipping live card** (its intended card, birthing_pod,
  is Phyrexian-blocked). Justified: it is task-directed, symmetric with `max_cmc_amount`, rides
  the already-forced wire bump for free, is unit-tested, and is honored in both executors.
  Report honestly; the live consumer lands when OOS-OS8-1 (Phyrexian-mana-in-activated-cost) does.
- **R6 — no privacy layer**: `LookAtTopThenPlace` emits no distinct "look" event / no card
  identities, matching `RevealAndRoute`/`Scry`. `private_to` is deferred M10 infrastructure
  (does not exist). Do NOT invent it here.
- **R7 — narset legal-but-wrong ≤1**: narset's shipped `RevealAndRoute` -2 over-puts (all matches
  vs "may reveal A [one]"). Correctable to `LookAtTopThenPlace` but the card stays partial on a
  separate clause; deferred as an optional hygiene fix, not part of the roster.
- **R8 — Effect discriminant 96**: current max is 95 (RemoveFromCombat, PB-OS6). Verify from
  `state/hash.rs` before writing (planner-discriminant-drift hazard); use 96.
