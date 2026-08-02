# UI-2 — Additional-cost surfacing (sacrifice + Squad): implementation spec

Task `scutemob-178`. Evidence: `memory/playtest-triage-2026-08-02.md` **F9**.

## 0. What is and is not broken

The **request** wire already exists and is complete for these two cost kinds:

* `CastSpellData.additional_costs: Vec<AdditionalCost>` (`rules/command.rs:824`).
* `ActionParams.additional_costs` is forwarded verbatim by `params.rs`'s `CastSpell`
  arm (`params.rs:276`), and `CastSpell` is inside the nine-arm allowlist, so an
  announced value is *not* refused.
* `ActionParamsDto.additional_costs` deserializes (`view.rs:658`).

A hand-crafted `POST /api/game/action` could pay a sacrifice **today**. Two things are
missing, and they are both on the **offer** side:

1. **`StubProvider` is blind.** `legal_actions.rs` has zero references to
   `spell_additional_costs` or `KeywordAbility::Squad`. So:
   * Life's Legacy (`lifes_legacy.rs:26`, `SpellAdditionalCost::SacrificeCreature`) is
     offered on mana affordability alone, and `casting.rs:3311-3316` then refuses it —
     the observed **422**, and a straight **SR-38** violation ("never offer an action the
     engine rejects").
   * Galadhrim Brigade casts at `squad_count = 0` with the optional cost silently lost
     (CR 702.157).
2. **`ActionOptionView` has no field** saying a cost is required or available, or what is
   eligible to pay it — so no client can render a picker even though the answer channel
   is open.

**Zero engine lines.** Everything below is `crates/simulator`, `tools/play-server` and the
Svelte frontend. `crates/engine/src` and `crates/card-types/src` must diff empty.

## 1. `crates/simulator/src/legal_actions.rs` — the descriptor

### 1.1 New types

```rust
/// CR 118.8 / CR 601.2b (UI-2): the additional costs a `CastSpell` offer must or may
/// pay. Built by the provider, consumed by `params.rs` (for the bot default) and by
/// `tools/play-server` (to render a picker).
#[derive(Clone, Debug, Default)]
pub struct AdditionalCostPlan {
    /// CR 118.8: the spell's REQUIRED sacrifice, when its `CardDefinition` declares
    /// one. `None` for every other spell.
    pub sacrifice: Option<SacrificeCostOption>,
    /// CR 702.157: the OPTIONAL squad cost, when the spell has `KeywordAbility::Squad`.
    pub squad: Option<SquadCostOption>,
}

#[derive(Clone, Debug)]
pub struct SacrificeCostOption {
    /// The engine's own requirement, verbatim — for labelling only.
    pub requirement: SpellAdditionalCost,
    /// Battlefield permanents this player controls that `casting.rs`'s own gate will
    /// accept. **Never empty**: an empty set suppresses the whole offer (§1.3).
    pub eligible: Vec<ObjectId>,
    /// The deterministic default a bot submits: `eligible[0]` (lowest `ObjectId`, since
    /// `objects_in_zone` yields the engine's own order). Kept as its own field rather
    /// than re-derived at the two consumer sites, so the two cannot drift.
    pub default: ObjectId,
}

#[derive(Clone, Debug)]
pub struct SquadCostOption {
    /// The per-copy cost from `AbilityDefinition::Squad { cost }`.
    pub cost: ManaCost,
    /// The largest N this player can currently afford on top of the spell's own cost.
    /// 0 means "offerable but not payable right now" — the client must still be able to
    /// cast the spell, declining is always legal (CR 702.157a "any number of times",
    /// including zero).
    pub max_count: u32,
}
```

`LegalAction::CastSpell` gains one field:

```rust
CastSpell {
    card: ObjectId,
    from_zone: ZoneId,
    /// CR 118.8 / CR 702.157 (UI-2). `AdditionalCostPlan::default()` — both fields
    /// `None` — for the overwhelming majority of spells.
    additional_costs: AdditionalCostPlan,
},
```

Every existing match site already uses `..` **except** `crates/simulator/tests/commander_cast.rs:1049`, which constructs one; update that one site.

### 1.2 Eligibility mirrors `casting.rs`, gate for gate

`casting.rs:3300-3369` is the authority. Its checks, in its order, are exactly:

1. `sac_obj.zone == ZoneId::Battlefield`
2. `sac_obj.controller == player`
3. `!effects::object_cant_be_sacrificed(state, sac_id)` — CR 701.21a / PB-AC8
4. the filter, against **layer-resolved** `Characteristics`
   (`SacrificeCreature` → `card_types.contains(Creature)`; `SacrificeLand` → `Land`;
   `SacrificeArtifactOrCreature` → `Artifact || Creature`; `SacrificeSubtype(s)` →
   `subtypes.contains(s)`; `SacrificeColorPermanent(c)` → `colors.contains(c)`)

It does **not** check `is_phased_in` (unlike `effects::eligible_sacrifice_targets`), and
this mirror must not either — mirroring the *sacrifice-effect* helper instead of the
*cast* gate would offer a different set from the one that will be validated.

`object_cant_be_sacrificed` is `pub(crate)` to the engine and cannot be called from here
without an engine line. It is a two-line predicate over **public** state
(`state.restrictions()`, `GameRestriction::CantBeSacrificed`, plus the source's zone), so
mirror it locally with a doc comment naming it a *necessary* duplicate — the same
category as `multiply_mana_cost`, and explicitly not the category of
`effective_cast_cost` (whose engine copy IS public and must be consumed).

**Only `spell_additional_costs.first()` is read.** `casting.rs` says so in its own words
("For now, we support exactly one mandatory sacrifice cost") and then validates
`required_costs[0]` alone and consumes one sacrifice id. Offering a second requirement
the engine will never check would be an offer that means nothing. The roster gate (§6)
pins that no def in the corpus declares more than one, with a non-vacuity floor, so if
one ever does the gate fails loudly rather than this silently under-asking.

### 1.3 Offer gating (SR-38, criterion 5999)

In **both** cast loops (hand, `legal_actions.rs:526-550`; command zone, `:594-631`):

* Build the plan before pushing.
* If the def declares a required sacrifice and `eligible.is_empty()`, **do not push the
  `CastSpell` action at all.** The engine would refuse it (`casting.rs:3311`,
  `GameStateError::InvalidCommand`), and offering it is the F9 defect.
* Squad never gates: it is optional, `count: 0` is always legal, so a spell with Squad and
  no spare mana is still cast.

### 1.4 `max_count` has a real bound, not a magic cap

Loop `n = 1..` while `can_afford(state, player, &(base + n × squad))`, stopping at a
**genuine** upper bound rather than an arbitrary one — a silent cap would read as "this is
all you can pay" when it is not. The bound:

```
available = pool.total() + <count of untapped battlefield permanents this player
                            controls that have at least one mana ability>
```

Every mana ability produces at least one mana, so no payment plan can exceed `available`;
therefore `n_max <= available.saturating_sub(base.mana_value()) / squad.mana_value()`.
If `squad.mana_value() == 0` set `max_count = 0` and say why in the comment (the choice
would be unbounded, and the roster gate pins that no def in the corpus has one).

## 2. `crates/simulator/src/params.rs` — default and forward

`CastSpell` arm:

```rust
LegalAction::CastSpell { card, additional_costs: plan, .. } => {
    let additional_costs = merge_required_additional_costs(plan, &params.additional_costs);
    ...
}
```

`merge_required_additional_costs` **appends** the plan's required-sacrifice default when
the caller announced no `AdditionalCost::Sacrifice`, and otherwise leaves the caller's
vector untouched. Two properties, both load-bearing:

* **`ActionParams::default()` still produces an engine-accepted command.** `random_bot`
  and `heuristic_bot` reach this arm with the default params, so a bot casting Life's
  Legacy now sacrifices `eligible[0]` instead of being refused. This is the
  minimum-viable bot policy the brief asks for; the cost of "sacrifices the lowest-id
  creature, which may be its best" is filed as a seed, not hidden.
* **A human's explicit choice is never overwritten**, only a missing one filled in.

Squad is **not** defaulted: absent means declined (`count: 0`), which is what the engine
already does and what keeps a bot's command byte-identical to the pre-UI-2 one on every
non-sacrifice spell.

### 2.1 The mana cost must include the squad payment at all three sites

SIM-1's rule: the offer gate, `LocalGame::submit`'s human auto-tap and `advance()`'s bot
auto-tap must not disagree about what is charged. Squad is a cost **increase**
(`casting.rs:2766-2779` adds it N times to the total), so add a sibling to
`effective_cast_cost`:

```rust
pub fn effective_cast_cost_with_additional(
    state: &GameState,
    player: PlayerId,
    card: ObjectId,
    additional_costs: &[AdditionalCost],
) -> Option<ManaCost>
```

= `effective_cast_cost(..)` plus `count × squad_cost` for an `AdditionalCost::Squad`
entry. `effective_cast_cost` stays as-is and this one calls it, so the commander-tax
arithmetic is still `mtg_engine::apply_commander_tax` and is not re-derived.

`get_squad_cost` is private to `casting.rs`; reading `AbilityDefinition::Squad { cost }`
off `state.card_registry()` here is a **necessary** duplicate (same category as §1.2).

Call sites changed: `local_game.rs:449` (bot auto-tap) and `local_game.rs:630`
(`auto_tap_commands_for`), each passing `cast.additional_costs`. Identity for every cast
that announces no Squad — which is every bot cast and every cast today.

## 3. `tools/play-server/src/view.rs` — the descriptor on the wire

`ActionOptionView` gains:

```rust
/// CR 118.8 / CR 702.157 (UI-2): the additional costs this cast must or may pay.
/// `None` on every action that has none — which is nearly all of them.
pub costs: Option<AdditionalCostsView>,
```

```rust
pub struct AdditionalCostsView {
    /// Which `ActionParamsDto` field carries the answer. Sent rather than inferred,
    /// for the same reason `BlockingDecisionView::answer_field` is.
    pub answer_field: String,          // "additional_costs"
    pub prompt: String,
    pub sacrifice: Option<SacrificeCostView>,
    pub squad: Option<SquadCostView>,
}

pub struct SacrificeCostView {
    pub prompt: String,
    /// Battlefield permanents, so every label comes through `NameIndex` — the
    /// seat-redacted channel, NOT `question_card_label` (which exists only for library
    /// cards the engine has told this seat to look at). CR 400.1 makes the battlefield
    /// public, so nothing here can be hidden in the first place.
    pub candidates: Vec<CardOptionView>,
    pub default: u64,
    /// The engine's own `AdditionalCost`, serialized verbatim, holding the default —
    /// the client clones it and replaces the array named by `ids_key`. Same argument
    /// as `TargetOptionView::value` and `AnswerShapeView::Partition::template`: the
    /// externally-tagged encoding of `AdditionalCost` stays known in exactly one place.
    pub template: AdditionalCost,      // Sacrifice { ids: [default], lki: [] }
    pub ids_key: String,               // "ids"
}

pub struct SquadCostView {
    pub prompt: String,
    /// Compact MTG notation, rendered server-side (`{1}{G}`).
    pub cost_label: String,
    pub max_count: u32,
    pub template: AdditionalCost,      // Squad { count: 0 }
    pub count_key: String,             // "count"
}
```

`lki` is deliberately sent empty and must stay empty: `casting.rs:4269-4285` **patches**
it from the layer-resolved characteristics captured before the zone move (CR
608.2b/608.2h/608.2i). A client-supplied `lki` would be a second opinion about LKI.

## 4. `tools/play-server/src/api.rs` — the 400/422 boundary

A new `validate_additional_cost_params`, called beside `validate_combat_params` /
`validate_decision_params` under the same `seq`-matches guard:

* a submitted `AdditionalCost::Sacrifice` must carry **exactly one** id and that id must
  be in the offered `eligible` set → else 400 `bad_params`;
* a submitted `AdditionalCost::Squad { count }` must have `count <= max_count`, and the
  action must actually offer Squad → else 400;
* **every other `AdditionalCost` variant falls through to the engine's 422.** This is
  deliberate and is said out loud: UI-2 surfaces two cost kinds, so it can only speak
  authoritatively about two. Claiming to validate the rest would be a check written
  against no offer.

## 5. Frontend — `CostPicker.svelte`, stage `'costs'`

Inserted into `ActionBar.pickerNeeded` **after `'value'` and before `'targets'`**.

CR 601.2b announces additional costs, and CR 601.2c announces targets, so costs-before-
targets is required. Within 601.2b the printed order is modes → splice → **additional
costs** → X, so a strict reading would split `ValuePrompt` (which bundles modes *and* X)
around this stage. That is not done, and the reason is checkable rather than assumed: the
roster gate (§6) pins that **no def in the corpus declares both an additional sacrifice
cost / Squad and an `{X}` or a mode**, so the sub-ordering inside 601.2b is not observable
today. If the gate ever fails, this is the paragraph to re-read.

The picker builds its own fragment off `costs.answer_field` and the two templates, so it
never spells `"Sacrifice"`, `"Squad"` or `"additional_costs"` itself.

## 6. Tests

| Id | Where | What |
|----|-------|------|
| **R1–R5** | `crates/engine/tests/core/ui2_additional_cost_roster.rs` | Permanent roster gate enumerating `all_cards()` (**SR-36 — never grep**): R1 the required-sacrifice roster with a non-vacuity floor; R2 no def declares >1 requirement; R3 the Squad roster; R4 every Squad def has a non-zero `AbilityDefinition::Squad { cost }`; R5 no def co-declares an additional cost with `{X}`/modes (§5's premise). |
| **T1–T6** | `legal_actions.rs` unit tests | eligibility mirrors each of the five filters; `CantBeSacrificed` excluded; **offer suppressed** when nothing is eligible; squad `max_count` bound. |
| **T7–T9** | `params.rs` unit tests | default appended for a bot; a human's choice forwarded verbatim and not overwritten; no Squad defaulted. |
| **P1** | `tools/play-server/src/main.rs` | **Life's Legacy over HTTP** — the descriptor is present, the pre-fix 422 is reproduced (submit with the sacrifice stripped), and a real cast resolves with the chosen creature in the graveyard. |
| **P2** | same | **Squad 0 / 1 / N** — declining casts plain; paying N charges N × cost and produces N token copies. |
| **P3** | same | **SR-38** — with no eligible permanent the option is absent from the payload entirely. |

Every probe must be **watched failing** by reverting the fix, not argued to discriminate.

## 7. Known limitations to state, not paper over

* Only two of `AdditionalCost`'s sixteen variants are surfaced. Kicker, Replicate,
  Offspring, Escalate, Splice, Entwine, Gift, Assist, Escape and Collect Evidence are all
  still invisible to the offer, and a `Complete` card carrying one still casts without it
  (or is still refused, for the mandatory ones). Seed this.
* The bot's required-sacrifice default is `eligible[0]` — the lowest `ObjectId`, which is
  usually its **oldest and often best** creature. That is a real strategic cost of making
  the cast legal at all. Seed this.
* No frontend test harness exists (plan §8 R7), so `CostPicker.svelte` has no automated
  test. The *channel* is covered end-to-end by P1/P2; its rendering is not.
