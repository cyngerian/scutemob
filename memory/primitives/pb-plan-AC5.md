# Primitive Batch Plan: PB-AC5 — Alt-Costs & Timing Keywords

**Generated**: 2026-07-08
**Primitive**: Four DSL capabilities — (1) Warp (`AltCostKind::Warp` + `KeywordAbility::Warp`
+ end-step exile delayed trigger + recast-from-exile), (2) Transmute (`KeywordAbility::Transmute`
marker over existing DiscardSelf/SearchLibrary composition), (3) Exert (keyword *action*:
`KeywordAbility::Exert` attack-cost marker + `Cost::Exert` activation cost + `Designations::EXERTED`
+ `TriggerCondition::WhenExertedAsAttacks`), (4) `Cost::ExileFromHand` pitch cost (`AltCostKind::Pitch`
+ `AltCastDetails::Pitch` + `AdditionalCost::ExileFromHand`).
**CR Rules**: 702.185 (Warp), 702.53 (Transmute), 701.43 (Exert), 118.9/118.9a-d (alt costs),
508.1g (optional attack cost), 607.2h (linked triggers), 608.3g (stack-static delayed trigger),
601.2b/f-h (announcing alt costs), 502.3 (untap), 400.7 (object identity on zone change).
**Cards affected**: 9 in roster. **Confirmed full yield: 6** (7 if the recommended
`CounterSpell.exile_instead` micro-add is accepted). **2 remain BLOCKED on unrelated
out-of-scope 2nd gaps** (see "Roster reality check").
**Dependencies**: PB-AC1 (DoesNotUntap keyword + untap loop at `turn_actions.rs:1205`),
RC-1/RC-3 consolidations (AdditionalCost, AltCastAbility). All present.
**Deferred items from prior PBs**: none targeting these four primitives.

---

## Roster reality check (READ FIRST — contradicts the dispatch brief's yield framing)

The brief and `pb-ac5-scope-verified.md` list 9 cards. Verified against oracle text + card
defs, only **6 are fully authorable by PB-AC5's four primitives**. Two carry a SECOND,
unrelated gap that PB-AC5 does not close; per W6 policy ("no partial implementations, no
wrong game state") they must stay BLOCKED, not shipped half-done.

| Card | Primitive | Fully authorable? | Blocking 2nd gap (out of scope) |
|---|---|---|---|
| `timeline_culler` | Warp | **YES** | — (Haste + graveyard-warp handled) |
| `dimir_infiltrator` | Transmute | **YES** | — |
| `combat_celebrant` | Exert (attack) | **YES** | — |
| `arena_of_glory` | Exert (activation) | **YES** | — |
| `force_of_will` | Pitch | **YES** | — |
| `force_of_vigor` | Pitch | **YES** | — (UpToN already done) |
| `force_of_negation` | Pitch | **only if** `CounterSpell.exile_instead` added | "exile it instead of graveyard" on counter — `Effect::CounterSpell` has no exile-instead flag (verified `card_definition.rs:1339`). RECOMMEND small in-scope add (Engine Change P5). |
| `starfield_shepherd` | Warp | **NO — stays BLOCKED** | ETB "search for a basic Plains card OR a creature with MV ≤ 1": `TargetFilter` cannot express disjunction across two independent constraint-groups (basic+Plains-subtype OR creature-type+max_cmc). Separate primitive. |
| `force_of_despair` | Pitch | **NO — stays BLOCKED** | "Destroy all creatures that entered this turn": `DestroyAll`/`TargetFilter` has no entered-this-turn predicate. Separate primitive. |

**Action for the two blocked cards**: do NOT author them to completion. Update their TODO
comments to record that the alt-cost/warp primitive now exists (PB-AC5) but the named 2nd
gap still blocks full authoring. This keeps the history honest and avoids shipping wrong
state (see conventions.md "aspirationally-wrong comments are correctness hazards").

**TODO sweep result (mandatory gate)**: grepped `cards/defs/` for
`TODO.*(warp|transmute|exert|pitch|exile.*card from hand)`. Hits = exactly the 9 roster
cards, **0 cards outside the brief**. Broader oracle-text sweep for `Transmute|Warp|Exert`
additionally surfaced `susurian_voidborn` (death-trigger filter gap, NOT warp — confirmed
out of scope) and `chaos_warp` (name false positive). **Positive assertion: no roster-recall
additions.**

---

## CR Rule Text (verbatim from MCP)

**702.185 Warp**
- 702.185a: two static abilities functioning while the card is on the stack, one of which may
  create a delayed triggered ability. "Warp [cost]" = "You may cast this card from your hand by
  paying [cost] rather than its mana cost" AND "If this spell's warp cost was paid, exile the
  permanent this spell becomes at the beginning of the next end step. Its owner may cast this
  card after the current turn has ended for as long as it remains exiled." Alt cost per 601.2b/601.2f-h.
- 702.185b: a "warped card in exile" is one exiled by the delayed triggered ability created by a
  warp ability. (Needs a distinguishing flag — a bare "in exile" bit is insufficient.)
- 702.185c: "a spell was warped this turn" = cast for its warp cost this turn.

**702.53 Transmute**
- 702.53a: activated ability functioning **only while the card is in a player's hand**.
  "Transmute [cost]" = "[Cost], Discard this card: Search your library for a card with the same
  mana value as the discarded card, reveal that card, and put it into your hand. Then shuffle
  your library. Activate only as a sorcery."
- 702.53b: the ability continues to exist in all other zones; objects with transmute count as
  "having an activated ability."

**701.43 Exert**
- 701.43a: to exert a permanent, you choose to have it not untap during your next untap step.
- 701.43b: a permanent can be exerted even if untapped or already exerted; all such effects
  expire during the SAME untap step.
- 701.43c: an object not on the battlefield can't be exerted.
- 701.43d: "You may exert [this creature] as it attacks" is an optional cost to attack (508.1g);
  a "when you do" trigger in the same paragraph is linked (607.2h).

**118.9 alt costs** — 118.9a: only ONE alternative cost per spell. 118.9c: alt cost does not
change the spell's mana cost (effects reading mana cost see the original). 118.9d: additional
costs/increases/reductions apply to the alt cost.

**508.1g**: active player chooses which optional "as it attacks" costs to pay.
**607.2h**: static + same-paragraph triggered ability are linked.
**608.3g**: a stack-static that creates a delayed triggered ability creates it as the permanent
enters (groups Dash 702.109, Blitz 702.152, Warp 702.185 — structurally identical machinery).

---

## Discriminant chain (verified against current code)

- `KeywordAbility`: next free = **163**. Assign **Warp=163, Transmute=164, Exert=165**
  (hashed at `state/hash.rs` ~955-962; enum tail `state/types.rs:1685`).
- `AltCostKind` hash discriminants end at **29** (`hash.rs:3301`). Assign **Warp=30, Pitch=31**.
- `AltCastDetails`: currently `Escape`, `Prototype` (`card_definition.rs:1073`). Add `Warp`, `Pitch`.
- `AdditionalCost`: add `ExileFromHand` (`state/types.rs:200`, hash `hash.rs:3306`).
- `Cost`: add `ExileFromHand`, `Exert` (`card_definition.rs:1125`, hash `hash.rs:5215`).
- `Designations` (u16, `game_object.rs:18`): last used bit `SOLVED = 1<<9`. Assign
  **EXERTED = 1<<10, WARPED = 1<<11**.
- `TriggerCondition`: add `WhenExertedAsAttacks` (`card_definition.rs:2836`).
- `PendingTriggerKind`: add `WarpExile` (`state/stubs.rs:63`).
- `StackObjectKind`: **NO new variant** — warp exile reuses `KeywordTrigger { keyword: Warp,
  data: DelayedZoneChange }`; transmute/arena are normal `ActivatedAbility`; the exert linked
  trigger is a normal `TriggeredAbility`. (This is a simplification vs. the brief, which only
  recorded the count 27.)
- `HASH_SCHEMA_VERSION`: **31 → 32** (`hash.rs:235`; also update the changelog doc-comment
  and the parity test `assert_eq!(HASH_SCHEMA_VERSION, 32)`).

---

## Engine Changes — Primitive 1: WARP (CR 702.185)

Model against Foretell (exile-then-recast-later; `game_object.rs` `FORETOLD`/`foretold_turn`,
casting gate `casting.rs:304-319`) and Dash/Blitz (end-step delayed trigger; `turn_actions.rs:618-690`,
resolution `resolution.rs:3255-3315`, `abilities.rs:7558-7573`).

### W1 — `KeywordAbility::Warp` (disc 163)
- `state/types.rs` after `DoesNotUntap` (1685): add `Warp` with doc comment citing 702.185.
- `state/hash.rs` ~962: add `KeywordAbility::Warp => 163u8.hash_into(hasher),`.
- `tools/replay-viewer/src/view_model.rs` `keyword_ability_label` ~889: add
  `KeywordAbility::Warp => "Warp".to_string(),`.

### W2 — `AltCostKind::Warp` (hash disc 30)
- `state/types.rs:189` (end of enum): add `Warp`.
- `state/hash.rs:3301`: add `AltCostKind::Warp => 30,`.

### W3 — `AltCastDetails::Warp { costs: Vec<Cost>, from_graveyard: bool }`
- `cards/card_definition.rs:1073` enum: add variant. `costs` holds NON-mana components of the
  warp cost (e.g. `[Cost::PayLife(2)]` for Timeline Culler; `[]` for Starfield). The MANA part
  lives in `AltCastAbility.cost: ManaCost` (mirrors Dash/Blitz). `from_graveyard=true` grants
  the extra "cast from graveyard using its warp ability" permission (Timeline Culler).
- `state/hash.rs` (find the `impl HashInto for AltCastDetails`, near the `AltCostKind` block):
  add an arm hashing a fresh discriminant + `costs` (len + each Cost) + `from_graveyard`.

### W4 — `GameObject.warped_turn: u32` + `Designations::WARPED` (1<<11)
- `game_object.rs`: add `#[serde(default)] pub warped_turn: u32,` (mirror `foretold_turn:793`),
  doc: "turn number when this card was warped into exile; 0 = not warped. Recastable only when
  `state.turn.turn_number > warped_turn` (CR 702.185a 'after the current turn has ended')."
- `game_object.rs:18` Designations: add `const WARPED = 1 << 11;` (CR 702.185b distinguishing flag).
- `state/hash.rs`: add `self.warped_turn.hash_into(hasher);` in the GameObject hash body (near
  `foretold_turn`). Designations bitfield is already hashed as bits — no shape change, schema
  bump covers the new bit's semantics.
- **CR 400.7 clearing**: verify `state/game_object.rs` / `move_object_to_zone` resets
  `warped_turn = 0` and clears `Designations::WARPED` on any zone change out of exile (mirror
  the FORETOLD/foretold_turn reset). On recast the card becomes a new object; the flag must not
  leak. If foretold clearing is centralized, add WARPED there; else clear it in the recast path.

### W5 — Casting a warp spell (`rules/casting.rs`) — CR 702.185a, 118.9a
Mirror the `cast_with_foretell` plumbing (`casting.rs:149,304-319,999-1035`):
- `let cast_with_warp = alt_cost == Some(AltCostKind::Warp);`
- **Legal origin zones**: Hand (initial), Exile (recast), and Graveyard IFF the card's
  `AltCastDetails::Warp.from_graveyard` (Timeline Culler). Relax the `casting_from_exile`
  guard (`casting.rs:246,450-455,597-604`) to admit warp like it admits foretell/adventure.
- **Exile recast gate** (mirror foretell 304-319): require the exile object has
  `Designations::WARPED` set AND `warped_turn < state.turn.turn_number` (reject same-turn recast:
  "after the current turn has ended").
- **Cost payment**: pay `AltCastAbility.cost` (ManaCost) as mana; then iterate
  `AltCastDetails::Warp.costs` and pay each (`Cost::PayLife(n)` → deduct n life). Mana cost the
  spell reports elsewhere stays the printed cost (118.9c) — do NOT overwrite mana_cost.
- **Mutual exclusion (118.9a)**: add the standard "cannot combine warp with {foretell, flashback,
  evoke, dash, blitz, …}" guards mirroring the foretell block (`casting.rs:999-1035` and the
  per-alt-cost `casting_with_foretell` guards). Warp is an alternative cost.
- **Set `cast_alt_cost = Some(AltCostKind::Warp)`** on the resulting spell/permanent (see W6) and
  (deferred, see note) do not add `spell_warped_this_turn` unless a card needs it.

### W6 — Set `cast_alt_cost` on the resolved permanent (`rules/resolution.rs`)
Mirror how Dash/Blitz stamp `cast_alt_cost` on the permanent as the warp spell resolves into a
permanent, so the end-step sweep (W7) finds it. (Grep `resolution.rs` for where `cast_alt_cost`
is copied from the stack object to the new battlefield object for Dash.)

### W7 — End-step delayed exile trigger (`rules/turn_actions.rs`, alongside Dash/Blitz 618-690)
Add a block: collect battlefield objects with `cast_alt_cost == Some(AltCostKind::Warp)`, and for
each push a `PendingTrigger { kind: PendingTriggerKind::WarpExile, source: obj_id, controller, .. }`
(clone the Dash block verbatim, swapping the kind). CR 702.185a "at the beginning of the next end step."

### W8 — `PendingTriggerKind::WarpExile` (`state/stubs.rs:63`)
Add the variant with doc citing 702.185a. If `PendingTriggerKind` is hashed anywhere, add its
arm (check — most PendingTrigger data flows through the stack object, not the persisted hash;
verify with `cargo build`).

### W9 — Flush WarpExile → stack object (`rules/abilities.rs` ~7558, next to DashReturn)
```
PendingTriggerKind::WarpExile => StackObjectKind::KeywordTrigger {
    source_object: trigger.source,
    keyword: KeywordAbility::Warp,
    data: TriggerData::DelayedZoneChange,
}
```

### W10 — Resolve WarpExile (`rules/resolution.rs`, next to Dash arm 3255)
Add arm for `KeywordTrigger { keyword: Warp, data: DelayedZoneChange }`:
- CR 400.7 guard: only if `source_object` still on Battlefield (mirror Dash).
- `state.move_object_to_zone(source_object, ZoneId::Exile)?;`
- On the new exile object: set `Designations::WARPED` and `warped_turn = state.turn.turn_number`
  (this is the "current turn"; recast allowed when a later turn's number exceeds it).
- Emit an exile event + `AbilityResolved`. If the permanent already left the battlefield (never
  became a permanent, or died), do nothing (CR 702.185a targets "the permanent this spell becomes").

**Deferred (702.185c)**: `spell_warped_this_turn` per-turn GameState flag is NOT needed by any
roster card. Per default-to-defer, do NOT add unused hashed state. Record the deferral in the
plan close note.

---

## Engine Changes — Primitive 2: TRANSMUTE (CR 702.53)

**Key finding: Transmute needs almost no new engine surface.** The activated ability is fully
expressible today via `Cost::DiscardSelf` (which the activation path already treats as
"activated from hand" via `ActivationCost.discard_self` → `abilities.rs:172-179`),
`TimingRestriction::SorcerySpeed`, and `Effect::SearchLibrary`. The "same mana value as this
card" search is a FIXED property per card (Dimir Infiltrator MV=2), expressed as
`min_cmc = max_cmc = <MV>` on the `SearchLibrary` `TargetFilter`.

### T1 — `KeywordAbility::Transmute` (disc 164) — display/presence marker only
- `state/types.rs` after Warp: add `Transmute` (doc cites 702.53b — objects with transmute count
  as "having an activated ability"; satisfied inherently by the `Activated` ability, marker is
  for display and future "cares about transmute" effects).
- `state/hash.rs`: `KeywordAbility::Transmute => 164u8.hash_into(hasher),`.
- `view_model.rs`: `KeywordAbility::Transmute => "Transmute".to_string(),`.

### T2 — Verify (no code) that the composition works end to end
- `Cost::Sequence(vec![Cost::Mana(...), Cost::DiscardSelf])` flattens to `discard_self=true`
  (confirm `flatten_cost_into`/`ActivationCost` handles Sequence-with-DiscardSelf).
- `SearchLibrary { reveal: true, destination: Hand, .. }` shuffles the library after search
  (standard; confirm shuffle is unconditional in the SearchLibrary effect path).
- `TimingRestriction::SorcerySpeed` enforced for the activated ability.
No new Effect/Cost variants for transmute. If T2 uncovers that DiscardSelf-from-hand does not
compose with a `Cost::Sequence` (only bare `Cost::DiscardSelf`), STOP and flag — do not invent
new surface; that would be a scope decision.

---

## Engine Changes — Primitive 3: EXERT (CR 701.43 — keyword ACTION)

Two shapes. Model the attack-cost shape against **Enlist** (`KeywordAbility::Enlist` marker +
`DeclareAttackers.enlist_choices` field 508.1g + linked trigger 607.2h + post-processing at
`abilities.rs:3671`).

### E1 — `KeywordAbility::Exert` (disc 165) — "may exert as it attacks" marker (701.43d)
- `state/types.rs`, `hash.rs` (`=> 165u8`), `view_model.rs` (`=> "Exert".to_string()`).

### E2 — `Cost::Exert` — exert the source as an activation/attack cost (arena_of_glory)
- `cards/card_definition.rs:1125` Cost enum: add `Exert` (doc: 701.43a/c — set EXERTED on the
  source permanent; source must be on the battlefield).
- `state/hash.rs:5215` Cost hash: add arm.
- `testing/replay_harness.rs:3445` `flatten_cost_into`: add `Cost::Exert => { ... }` — flatten
  into a new `ActivationCost.exert: bool` field (add it; mirror `discard_card`/`discard_self`).
- `rules/abilities.rs` activated-ability cost payment: when the flattened cost has `exert=true`,
  validate source on battlefield (701.43c) and set `Designations::EXERTED` on the source. Place
  next to where Tap/other costs are paid.

### E3 — `Designations::EXERTED` (1<<10)
- `game_object.rs:18`: `const EXERTED = 1 << 10;` (CR 701.43a — set when exerted; cleared at the
  controller's next untap step).

### E4 — `DeclareAttackers.exert_choices: Vec<ObjectId>` (508.1g)
- `rules/command.rs:138` `Command::DeclareAttackers`: add `#[serde(default)] exert_choices: Vec<ObjectId>,`.
- `rules/engine.rs:170` dispatch: destructure `exert_choices` and thread into
  `combat::handle_declare_attackers(...)`.
- `rules/combat.rs:33` `handle_declare_attackers` signature: add `exert_choices: Vec<ObjectId>`.
  After the enlist validation block (369+): validate each exert choice — attacker is a declared
  attacker; has `KeywordAbility::Exert` (layer-aware via `calculate_characteristics`); is NOT
  already `EXERTED` (this satisfies combat_celebrant's "if this creature hasn't been exerted this
  turn"); on battlefield (701.43c). Then set `Designations::EXERTED` and queue the linked trigger
  (E5). Update the two card-def callers of `handle_declare_attackers`
  (`karlach_fury_of_avernus.rs`, `grand_warlord_radha.rs`) and `replay_harness.rs`,
  `builder.rs`, `turn.rs`, `turn_structure.rs` for the new field/arg.

### E5 — `TriggerCondition::WhenExertedAsAttacks` (linked trigger, 607.2h)
- `cards/card_definition.rs:2836`: add variant (doc: fires only when the creature is exerted as
  it attacks; linked to the Exert static, 607.2h).
- `state/hash.rs` TriggerCondition hash (~4909 region): add arm.
- `rules/abilities.rs` `check_triggers`: when an attacker is exerted in E4, queue its card-def
  `Triggered` abilities whose condition is `WhenExertedAsAttacks` as a `PendingTrigger`
  (Normal/CardDef path). This is cleaner than Enlist's placeholder-removal dance because the
  effect is card-specific. Alternatively, follow Enlist's post-processing pattern
  (`abilities.rs:3671-3710`) — planner recommends the direct queue-on-exert approach.

### E6 — Untap-step expiry (`rules/turn_actions.rs:1205`, adjacent to DoesNotUntap)
In the per-object untap loop, after computing `does_not_untap`, add: if the object has
`Designations::EXERTED`, do NOT untap it AND clear the EXERTED flag (`obj.designations.remove(
Designations::EXERTED)`). CR 701.43a/b: expires during this untap step; because it is a single
boolean, exerting twice still expires in one step (701.43b), which is correct. Do NOT reuse
`skip_untap_steps` (it is a decrementing counter and would wrongly stack multiple exerts).

---

## Engine Changes — Primitive 4: PITCH (`Cost::ExileFromHand`, CR 118.9)

### P1 — `Cost::ExileFromHand { color: Color }`
- `cards/card_definition.rs:1125` Cost enum: add `ExileFromHand { color: Color }` (CR 118.9 pitch;
  exile a card of this color from hand as (part of) an alternative cost). Single color covers all
  roster cards (blue/green/black). Composes via `Vec<Cost>`/`Cost::Sequence` with `Cost::PayLife`.
- `state/hash.rs:5215` Cost hash: add arm hashing discriminant + color.
- `testing/replay_harness.rs:3445` `flatten_cost_into`: no ActivationCost representation needed
  (pitch is a spell alt cost, not an activated ability cost) — but keep a no-op arm to satisfy
  exhaustiveness (mirror the existing `Cost::PayLife(_) => {}`).

### P2 — `AltCostKind::Pitch` (hash disc 31)
- `state/types.rs:189`: add `Pitch`. `state/hash.rs:3301`: `AltCostKind::Pitch => 31,`.

### P3 — `AltCastDetails::Pitch { costs: Vec<Cost>, opponents_turn_only: bool }`
- `cards/card_definition.rs:1073`: add variant. `costs` = full alternative-cost components, e.g.
  FoW `[Cost::PayLife(1), Cost::ExileFromHand { color: Blue }]`; FoV `[Cost::ExileFromHand {
  color: Green }]`. `opponents_turn_only=true` for FoV/FoN/FoD ("If it's not your turn").
- `state/hash.rs` AltCastDetails HashInto: add arm.

### P4 — `AdditionalCost::ExileFromHand { card: ObjectId }` (the chosen pitched card)
- `state/types.rs:200` `AdditionalCost` enum: add `ExileFromHand { card: ObjectId }` (the specific
  card the caster chose to pitch; carried on `CastSpell.additional_costs`, propagated to the stack
  object via the existing generic Vec).
- `state/hash.rs:3306` `impl HashInto for AdditionalCost`: add arm.

### P5 — Casting a pitch spell (`rules/casting.rs`) — CR 118.9a/c
- `let cast_with_pitch = alt_cost == Some(AltCostKind::Pitch);` (mirror foretell plumbing).
- **Condition gate**: if `AltCastDetails::Pitch.opponents_turn_only` and it IS the caster's turn
  (`state.turn.active_player == player`), reject.
- **Pay `costs`**: iterate `AltCastDetails::Pitch.costs`:
  - `Cost::PayLife(n)` → deduct n life.
  - `Cost::ExileFromHand { color }` → read the chosen card from `additional_costs`
    (`AdditionalCost::ExileFromHand { card }`), validate: card is in `ZoneId::Hand(player)`,
    layer-resolved colors include `color`, and card is NOT the spell being cast (it is on the
    stack, but guard explicitly). Move it to exile.
- Mana cost = {0} for the spell; do NOT alter the spell's reported mana cost (118.9c).
- **Mutual exclusion (118.9a)**: add "cannot combine pitch with {any other alt cost}" guards
  mirroring the foretell block.

### P5b (RECOMMENDED — unblocks force_of_negation) — `Effect::CounterSpell.exile_instead: bool`
- `cards/card_definition.rs:1339`: change `CounterSpell { target }` →
  `CounterSpell { target, #[serde(default)] exile_instead: bool }`.
- `state/hash.rs`: update the CounterSpell effect hash arm (add the bool).
- `rules/effects/mod.rs` (or wherever CounterSpell resolves): when `exile_instead`, move the
  countered spell to exile instead of graveyard (CR 701.5f / Force of Negation).
- Update ALL existing `Effect::CounterSpell { target: ... }` construction sites (grep — several
  card defs) to add `exile_instead: false`, plus any exhaustive match on the Effect variant.
- **Decision point**: this is a small, low-risk addition that converts force_of_negation from
  BLOCKED → fully authorable (yield 6→7). If the coordinator prefers strict four-primitive scope,
  SKIP P5b and leave force_of_negation blocked (update its comment). Planner recommends INCLUDING
  P5b — shipping a knowingly-wrong counter destination would violate the no-wrong-state invariant.

---

## Exhaustive-match / downstream-build sites (the #1 compile-error source)

| File | Match on | Action |
|---|---|---|
| `crates/engine/src/state/hash.rs` | KeywordAbility | add Warp=163, Transmute=164, Exert=165 |
| `crates/engine/src/state/hash.rs` | AltCostKind | add Warp=30, Pitch=31 |
| `crates/engine/src/state/hash.rs` | AltCastDetails | add Warp, Pitch arms |
| `crates/engine/src/state/hash.rs` | AdditionalCost | add ExileFromHand arm |
| `crates/engine/src/state/hash.rs` | Cost | add ExileFromHand, Exert arms |
| `crates/engine/src/state/hash.rs` | TriggerCondition | add WhenExertedAsAttacks arm |
| `crates/engine/src/state/hash.rs` | GameObject body | hash `warped_turn` |
| `crates/engine/src/state/hash.rs` | Effect::CounterSpell | (P5b) hash `exile_instead` |
| `crates/engine/src/state/hash.rs` | `HASH_SCHEMA_VERSION` | 31 → 32 + changelog comment |
| `tools/replay-viewer/src/view_model.rs` | `keyword_ability_label` | add Warp/Transmute/Exert labels (KeywordTrigger display already falls through `_`, no change) |
| `tools/tui/src/play/panels/stack_view.rs` | StackObjectKind | **no change** (no new SOK; KeywordTrigger arm is generic) — but rebuild to confirm |
| `crates/engine/src/rules/command.rs` | Command::DeclareAttackers | add `exert_choices` field |
| `crates/engine/src/rules/engine.rs` | DeclareAttackers dispatch | thread `exert_choices` |
| `crates/engine/src/rules/combat.rs` | `handle_declare_attackers` sig + callers | add param |
| `crates/engine/src/cards/defs/karlach_fury_of_avernus.rs`, `grand_warlord_radha.rs` | DeclareAttackers construction | add `exert_choices: vec![]` |
| `crates/engine/src/state/{builder,turn}.rs`, `rules/turn_structure.rs` | DeclareAttackers construction | add `exert_choices: vec![]` |
| `crates/engine/src/state/stubs.rs` | PendingTriggerKind | add WarpExile |
| `crates/engine/src/rules/effects/mod.rs` + card defs | Effect::CounterSpell construction | (P5b) add `exile_instead: false` at all sites |
| `crates/engine/src/cards/helpers.rs` | prelude | ensure `Color`, `AltCastDetails`, `AdditionalCost` exported (add if a def fails with "undeclared type") |

**Run `cargo build --workspace` after EVERY primitive's engine changes** — replay-viewer and tui
are separate crates; runners miss these ~50% of the time (per MEMORY.md).

---

## Harness wiring (`crates/engine/src/testing/`)

- `script_schema.rs` + `translate_player_action` (`replay_harness.rs`):
  - Cast with alt cost: extend the cast action to carry `alt_cost` (Warp/Pitch) + a
    `pitch_exile_card` / `warp_from_zone` param → build `CastSpell.alt_cost` +
    `additional_costs` (`AdditionalCost::ExileFromHand`) + life payment.
  - `declare_attackers`: add an `exert` list param → `Command::DeclareAttackers.exert_choices`.
  - Transmute uses the EXISTING activate-ability action (Cost::DiscardSelf-from-hand path);
    verify the harness can activate a hand ability (Channel precedent).
- `legal_actions.rs`: add arms so the bot/legal-action provider offers: warp cast (from hand and,
  after the turn ends, from exile), pitch cast (only on a legal turn per `opponents_turn_only`),
  exert-at-declare-attackers, and the arena_of_glory exert activation. Follow the Foretell/Dash
  and Enlist precedents.

---

## Card Definition Fixes

### timeline_culler.rs (Warp — YIELD)
Warp—{B}, Pay 2 life; Haste; castable from graveyard. Keep `KeywordAbility::Haste`. Add:
`AbilityDefinition::Keyword(KeywordAbility::Warp)` + `AbilityDefinition::AltCastAbility { kind:
AltCostKind::Warp, cost: ManaCost { black: 1, ..default }, details: Some(AltCastDetails::Warp {
costs: vec![Cost::PayLife(2)], from_graveyard: true }) }`. Remove TODOs.

### dimir_infiltrator.rs (Transmute — YIELD)
Keep `CantBeBlocked`. Add `KeywordAbility::Transmute` marker + `AbilityDefinition::Activated {
cost: Cost::Sequence(vec![Cost::Mana(ManaCost{generic:1,blue:1,black:1,..default}),
Cost::DiscardSelf]), timing_restriction: Some(SorcerySpeed), targets: vec![], activation_condition:
None, activation_zone: None, once_per_turn: false, effect: Effect::SearchLibrary { player:
Controller, filter: TargetFilter { min_cmc: Some(2), max_cmc: Some(2), ..default }, reveal: true,
destination: Hand, shuffle_before_placing: false, also_search_graveyard: false } }`. (MV of {U}{B}
= 2.) Remove TODOs.

### combat_celebrant.rs (Exert attack — YIELD)
Replace the simplified `WhenAttacks` trigger. Add `KeywordAbility::Exert` marker + a linked
`AbilityDefinition::Triggered { trigger_condition: WhenExertedAsAttacks, intervening_if: None,
effect: Sequence([ForEach EachOtherCreatureYouControl → UntapPermanent, AdditionalCombatPhase {
followed_by_main: false }]), targets: vec![], modes: None, trigger_zone: None, once_per_turn:
false }`. The "hasn't been exerted this turn" restriction is enforced by E4 (exert offered only if
not already EXERTED). Remove TODO.

### arena_of_glory.rs (Exert activation — YIELD)
Keep the EntersTapped replacement + `{T}: Add {R}`. Add the third ability:
`AbilityDefinition::Activated { cost: Cost::Sequence(vec![Cost::Mana(ManaCost{red:1,..default}),
Cost::Tap, Cost::Exert]), effect: <Add {R}{R} + haste-if-spent-on-creature-spell>, ... }`. NOTE:
the "if that mana is spent on a creature spell, it gains haste until end of turn" mana-tag rider
may itself be a DSL gap — verify an `AddMana`-with-spend-restriction/rider primitive exists; if
NOT, arena_of_glory's exert ACTIVATION (the exert primitive) is provable but this rider is a
separate gap. Flag and, if the rider is unexpressible, keep arena_of_glory blocked on the rider
(the Cost::Exert primitive is still validated by the unit test + the plain add-{R}{R} portion).
**Runner: verify the mana-rider before authoring; do not ship a card missing the haste rider.**

### force_of_will.rs (Pitch — YIELD)
Add `AbilityDefinition::AltCastAbility { kind: AltCostKind::Pitch, cost: ManaCost::default(),
details: Some(AltCastDetails::Pitch { costs: vec![Cost::PayLife(1), Cost::ExileFromHand { color:
Color::Blue }], opponents_turn_only: false }) }` alongside the existing CounterSpell Spell ability.
Remove TODO.

### force_of_vigor.rs (Pitch — YIELD)
Keep the UpToN destroy. Add `AltCastAbility { kind: Pitch, cost: default, details: Some(Pitch {
costs: vec![Cost::ExileFromHand { color: Color::Green }], opponents_turn_only: true }) }`. Remove TODO.

### force_of_negation.rs (Pitch — YIELD only with P5b)
If P5b accepted: add the Pitch alt cost `[Cost::ExileFromHand { color: Blue }]`,
`opponents_turn_only: true`, and change the CounterSpell effect to `exile_instead: true`. Remove
TODOs. If P5b rejected: leave blocked; update comment to "PB-AC5 added pitch cost; still blocked on
CounterSpell exile-instead."

### starfield_shepherd.rs (BLOCKED — comment update only)
Do NOT author. Update the Warp TODO to: "Warp primitive shipped in PB-AC5; card still BLOCKED on
ETB disjunctive search (basic Plains OR creature MV≤1) — TargetFilter cannot express cross-group OR."

### force_of_despair.rs (BLOCKED — comment update only)
Do NOT author. Update the pitch TODO to: "Pitch cost primitive shipped in PB-AC5; card still
BLOCKED on 'destroy all creatures that entered this turn' (no entered-this-turn TargetFilter
predicate)."

## New Card Definitions
None — all 9 already have stub defs.

---

## Unit Tests (`crates/engine/tests/`)

New file `pb_ac5_alt_costs.rs` (plus combat/exert tests may live in an existing combat test file).
Each test cites CR and uses `GameStateBuilder`.

**Warp (702.185)**
- `test_warp_cast_from_hand_pays_warp_cost` — cast Starfield's/Timeline's warp cost from hand;
  assert mana+life deducted, permanent enters. (702.185a)
- `test_warp_exiled_at_next_end_step` — warp-cast a creature; advance to end step; assert it is in
  exile with `Designations::WARPED` and `warped_turn == turn`. (702.185a/b)
- `test_warp_not_exiled_if_not_warp_cast` — cast the same creature for its normal mana cost;
  assert NOT exiled at end step (only warp-paid permanents). (702.185a "if warp cost was paid")
- `test_warp_recast_from_exile_after_turn` — after exile, on a later turn recast from exile
  succeeds; same-turn recast rejected. (702.185a "after the current turn has ended")
- `test_warp_countered_spell_not_exiled` — counter the warp spell; assert no exile trigger (it
  never became a permanent). (702.185a, 400.7)
- `test_warp_timeline_culler_from_graveyard` — with `from_graveyard`, warp-cast from graveyard
  succeeds; without it, rejected. (Timeline Culler)
- `test_warp_mutual_exclusion` — attempt warp + another alt cost → rejected. (118.9a)

**Transmute (702.53)**
- `test_transmute_searches_equal_mana_value` — activate Dimir Infiltrator's transmute; assert a
  MV-2 card moved library→hand, library shuffled. (702.53a)
- `test_transmute_only_from_hand` — transmute not activatable from battlefield/graveyard. (702.53a)
- `test_transmute_sorcery_timing` — reject activation with a nonempty stack / on opponent's turn.
- `test_transmute_discards_self` — the source card is discarded as the cost. (702.53a)

**Exert (701.43)**
- `test_exert_combat_celebrant_untaps_and_extra_combat` — declare Combat Celebrant attacking with
  `exert_choices=[celebrant]`; assert linked trigger untaps other creatures + adds a combat phase.
  (701.43d, 607.2h)
- `test_exert_does_not_untap_next_untap_step` — exerted permanent stays tapped through its
  controller's next untap step, then EXERTED cleared. (701.43a)
- `test_exert_twice_expires_same_step` — exert via attack then via arena activation same turn;
  assert only ONE untap step skipped (boolean flag, 701.43b).
- `test_exert_offer_requires_not_already_exerted` — Combat Celebrant can't be exerted as attacker
  twice in one turn (card text) — second offer rejected.
- `test_exert_arena_of_glory_activation` — pay `{R},{T},Exert` → EXERTED set, {R}{R} added.
  (701.43a as activation cost)
- `test_exert_cannot_exert_off_battlefield` — Cost::Exert on a source not on battlefield rejected.
  (701.43c)

**Pitch (118.9)**
- `test_pitch_force_of_will_exile_blue_and_life` — cast FoW paying 1 life + exiling a blue card;
  assert mana untouched (0 paid), 1 life lost, blue card exiled, spell counters target. (118.9)
- `test_pitch_wrong_color_rejected` — attempt to pitch a non-blue card for FoW → rejected.
- `test_pitch_force_of_vigor_opponents_turn_only` — FoV pitch legal on opponent's turn, rejected
  on your own turn. (card text / opponents_turn_only)
- `test_pitch_mana_value_unchanged` — after pitching, an effect reading the spell's mana value/CMC
  sees the printed cost, not 0. (118.9c)
- `test_pitch_mutual_exclusion` — pitch + another alt cost → rejected. (118.9a)
- `test_pitch_cannot_pitch_self` — the card being cast can't be its own pitch (it's on the stack).
- (P5b) `test_force_of_negation_counters_and_exiles` — countered noncreature spell goes to exile,
  not graveyard. (701.5 / Force of Negation)

**Hash parity**
- `test_hash_schema_version_is_32` — `assert_eq!(HASH_SCHEMA_VERSION, 32)` (strict equality per
  conventions hash-sentinel rule).

---

## Verification Checklist

- [ ] `cargo check -p mtg-engine` clean after each primitive
- [ ] `cargo build --workspace` clean after EACH primitive (replay-viewer + tui exhaustiveness)
- [ ] KeywordAbility labels added in view_model.rs (Warp/Transmute/Exert)
- [ ] `HASH_SCHEMA_VERSION == 32`; changelog comment + parity test updated
- [ ] All 6 (or 7 with P5b) YIELD card defs authored, TODOs removed, oracle text preserved
- [ ] 2 BLOCKED cards (starfield_shepherd, force_of_despair) comment-updated, NOT authored
- [ ] arena_of_glory mana-spend-rider verified expressible before authoring (else keep blocked)
- [ ] `cargo test --all` green; `cargo clippy -- -D warnings`; `cargo fmt --check`
- [ ] No remaining `TODO.*(warp|transmute|exert|pitch)` in the 6-7 authored defs

---

## Risks & Edge Cases

- **CR 400.7 flag leakage (Warp)**: `WARPED`/`warped_turn` MUST clear when the card leaves exile
  on recast. If left set, a later normal cast could be wrongly gated or re-exiled. Mirror foretold
  clearing exactly.
- **Warp origin-zone relaxation**: the `casting_from_exile` guards reject unknown alt-cost casts
  from exile — every guard mirroring foretell must also admit warp (grep all `!cast_with_foretell`
  sites at 452/597; add `&& !cast_with_warp`).
- **Exert vs DoesNotUntap**: keep them SEPARATE. EXERTED is a one-shot boolean cleared in the
  untap loop; DoesNotUntap is a standing static keyword. Do not fold exert into DoesNotUntap or
  skip_untap_steps (verification doc §3).
- **Exert linked trigger must be gated on the exert CHOICE**, not on attack. Authoring it as a
  plain `WhenAttacks` trigger (the current stub) fires even when the player declines to exert —
  wrong. Use `WhenExertedAsAttacks` queued only for exerted attackers.
- **Transmute MV hardcode**: `min_cmc=max_cmc=<MV>` is faithful ONLY for fixed-MV cards (all
  current transmute cards). A general "same MV as source, dynamic" filter is deferred; do not
  generalize.
- **Pitch 118.9c**: the pitched spell's mana value must remain the printed value for any effect
  reading it (Force of Will pitched still has MV 5). Do not zero the stored mana cost; only the
  paid cost is 0.
- **arena_of_glory haste rider** is a possible separate DSL gap ("if that mana is spent on a
  creature spell…"). The Exert primitive is validated regardless; but the CARD is only shippable
  if the rider is expressible. Verify first.
- **force_of_negation / force_of_despair / starfield_shepherd**: shipping any of them with a
  known-missing clause violates the no-wrong-state invariant. Keep blocked unless the 2nd gap is
  genuinely closed (only force_of_negation's is trivially closable, via P5b).
- **DeclareAttackers field fan-out**: `exert_choices` touches many construction sites (engine
  dispatch, combat sig, 2 card defs, builder, turn, turn_structure, harness). Use `#[serde(default)]`
  and `cargo build` to enumerate them.
- **PendingTriggerKind::WarpExile hashing**: confirm whether PendingTriggerKind participates in
  the state hash; if so add its arm (schema already bumping).
