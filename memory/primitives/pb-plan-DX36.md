# PB-DX36 implementation plan

Seed: **OOS-CARDS2-6** (both halves). Task `scutemob-228`. Stage-0 measurements, the wire
prediction and the design rationale are in `memory/primitives/pb-DX36-execution-notes.md` §0 —
**read that first; it is binding.** This file is the step list.

Merge base `e7d7ae31`. Baseline **5,097 / 0 / 5**, 63 targets. PROTOCOL **41**, HASH **82**.

---

## The two defects

**Half A (correctness, live on a deck-legal `Complete` def).**
`TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer { combat_only }`
(`crates/card-types/src/cards/card_definition.rs:3692`) is dispatched at
`crates/engine/src/rules/abilities.rs:~5889-5945`, **inside the `GameEvent::CombatDamageDealt` arm
only**, under a `TODO(PB-37)`. `combat_only` is destructured away by the lowering
(`crates/engine/src/testing/replay_harness.rs:~3590` — `{ .. }`) and is **read in exactly one place
in the whole workspace: `crates/engine/src/state/hash.rs:6848`**, so `true` and `false` are
behaviourally identical. `sigil_of_sleep` (`Complete` by derive, deck-legal) declares
`combat_only: false` and silently drops the noncombat half of its printed trigger.
`curiosity` and `ophidian_eye` (`partial`) carry the same arm plus an approximation of
*"an opponent"* as *any player*.

**Half B (card yield).** No general *"whenever this permanent deals damage"* `TriggerCondition`
and no damage-dealt `EffectAmount` for *"that much"*, so `exalted_angel`'s printed triggered
ability is unauthored.

---

## Step 1 — `DamageRecipient` (new, card-types)

In `crates/card-types/src/cards/card_definition.rs`, beside `TriggerCondition`:

```rust
/// Which recipient of a damage event a "deals damage" trigger cares about.
///
/// CR 603.2 — the recipient clause of a damage trigger ("…deals damage",
/// "…deals damage to a player", "…deals damage to an opponent"). The check is
/// made at trigger-collection time against the trigger source's controller.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DamageRecipient {
    /// Any recipient — player, creature, planeswalker or battle.
    #[default]
    Any,
    /// Any player.
    Player,
    /// A player who is an opponent of the trigger source's controller (CR 102.2).
    Opponent,
}
```

Same derive set / serde attributes as its neighbours in that file. Export it from
`crates/card-types/src/cards/helpers.rs`'s prelude if the card defs need to name it (they do).

**It must not be placed on `TriggerEvent` or on any type in either closure** — see execution
notes §0.3: keeping it off both closures is a *predicted* outcome, and putting it on a wire type
would falsify the prediction that both type counts stay at 98 / 132.

## Step 2 — `TriggerCondition` (card-types)

1. `WhenEnchantedCreatureDealsDamageToPlayer` — **keep** `combat_only: bool`, **add**
   `recipient: DamageRecipient` (`#[serde(default)]`). Rewrite the doc: `combat_only` is now
   genuinely read (name the two lowering arms); on this variant `DamageRecipient::Any` and
   `::Player` are equivalent, because the dispatch site only ever fires on damage to a player —
   say so, and `t`-pin it.
2. **New** `WhenDealsDamage { recipient: DamageRecipient }` —
   *"Whenever this permanent deals damage"*, CR 603.2. Any damage, combat or noncombat.
   Deliberately carries **no** `combat_only` flag: a combat-only-to-player shape is already
   `WhenDealsCombatDamageToPlayer`, and adding a second unread flag is the defect this batch
   closes. Say that in the doc.

## Step 3 — `TriggerEvent` (card-types `state/game_object.rs`)

**Remove** `EnchantedCreatureDealsDamageToPlayer`. **Add seven** unit variants:

| variant | fired when |
|---|---|
| `EnchantedCreatureDealsCombatDamageToPlayer` | attached creature deals **combat** damage to any player |
| `EnchantedCreatureDealsCombatDamageToOpponent` | …to an opponent of the **Aura's** controller |
| `EnchantedCreatureDealsAnyDamageToPlayer` | attached creature deals **any** damage to any player |
| `EnchantedCreatureDealsAnyDamageToOpponent` | …to an opponent of the **Aura's** controller |
| `SelfDealsDamage` | this permanent deals damage to any recipient |
| `SelfDealsDamageToPlayer` | …to any player |
| `SelfDealsDamageToOpponent` | …to an opponent of **this permanent's** controller |

Each variant's doc must state which dispatch arm fires it. `TriggerEvent` is in **both** closures
(execution notes §0.2), so this is the PROTOCOL bump and part of the HASH bump.

## Step 4 — lowering (`crates/engine/src/testing/replay_harness.rs`,
`build_face_triggered_abilities`)

Replace the existing `WhenEnchantedCreatureDealsDamageToPlayer` arm and add a `WhenDealsDamage`
arm. **Both must select `trigger_on` with an exhaustive `match` and no wildcard arm** — a new
`DamageRecipient` value must be a compile error, not a silent drop:

```rust
let trigger_on = match (combat_only, recipient) {
    (true,  DamageRecipient::Any) | (true,  DamageRecipient::Player)  => TriggerEvent::EnchantedCreatureDealsCombatDamageToPlayer,
    (true,  DamageRecipient::Opponent)                                => TriggerEvent::EnchantedCreatureDealsCombatDamageToOpponent,
    (false, DamageRecipient::Any) | (false, DamageRecipient::Player)  => TriggerEvent::EnchantedCreatureDealsAnyDamageToPlayer,
    (false, DamageRecipient::Opponent)                                => TriggerEvent::EnchantedCreatureDealsAnyDamageToOpponent,
};
```

and for the self family `match recipient { Any => SelfDealsDamage, Player => SelfDealsDamageToPlayer,
Opponent => SelfDealsDamageToOpponent }`.

**One ability lowers to exactly one `TriggeredAbilityDef`.** Do NOT emit two entries for
`combat_only: false` — that would grow `characteristics.triggered_abilities`, shift the runtime
index space `PendingTriggerKind::Normal` addresses, and perturb `OOS-DX35-1`'s alignment roster.

No registry scan anywhere (PB-DX47): the runtime lowering is the single dispatcher.

## Step 5 — the amount channel

Three structs gain **one** `u32` field each, all named `damage_dealt_amount`:

* `PendingTrigger` (`crates/card-types/src/state/stubs.rs`) — `#[serde(default)]`, initialised to
  `0` in `PendingTrigger::blank`.
* `StackObject` (`crates/card-types/src/state/stack.rs`) — beside `combat_damage_amount`;
  update the ~10 literal construction sites (`copy.rs`, `engine.rs`, `casting.rs`,
  `resolution.rs`) that already list `combat_damage_amount: 0`.
* `EffectContext` (`crates/engine/src/effects/mod.rs`) — carried through the same
  `PendingTrigger → StackObject → EffectContext` chain `combat_damage_amount` already uses
  (`resolution.rs:2481` and `:2600` are the two `ctx.combat_damage_amount = …` sites; add the
  sibling assignment at both).

Doc on each: *"CR 603.10a: the amount of damage in the triggering damage event, combat or
noncombat. Read by `EffectAmount::DamageDealt`. 0 for triggers that are not damage triggers."*

**New `EffectAmount::DamageDealt`** (`card_definition.rs`), resolved in
`effects/mod.rs::resolve_amount` as `ctx.damage_dealt_amount as i32`, beside
`EffectAmount::CombatDamageDealt`. **Do not delete or rename `CombatDamageDealt`** — the two are
distinguishable (execution notes §0.5(d)) and the doc of each must point at the other and say
when each is correct.

Also **widen the doc** (not the name) of `PendingTrigger::damaged_player` and
`EffectContext::damaged_player`: they now carry the damaged player for noncombat damage too.
`TargetController::DamagedPlayer` reads this and `sigil_of_sleep`'s target filter uses it, so the
noncombat arm **must** populate it.

## Step 6 — ONE shared dispatch arithmetic (`crates/engine/src/rules/abilities.rs`)

Extract the attachment walk currently inlined in the `CombatDamageDealt` arm into one free
function, and call it from **both** damage arms:

```rust
/// CR 510.3a / CR 603.2: queue every "deals damage" trigger for one damage event.
///
/// `is_combat` is a property of the EVENT, not of any ability: `GameEvent::CombatDamageDealt`
/// passes `true`, `GameEvent::DamageDealt` passes `false`. Combat damage is emitted only as
/// `CombatDamageDealt` (verified: `rules/combat.rs:2382` is the sole combat emit site and it
/// emits no `DamageDealt`), so the two arms are **disjoint by construction** and a given
/// ability — which lowers to exactly one `trigger_on` — fires exactly once per damage event.
fn queue_damage_source_triggers(
    state: &GameState,
    triggers: &mut Vec<PendingTrigger>,
    source: ObjectId,
    target: &CombatDamageTarget,
    amount: u32,
    is_combat: bool,
)
```

Behaviour:

* CR 603.2g — return immediately if `amount == 0`.
* Return if `source` is not on the battlefield (the existing `creature_on_bf` guard).
* **Self family**, on `source`: always `SelfDealsDamage`; if `target` is a player, also
  `SelfDealsDamageToPlayer`, and if that player `!= source.controller`, also
  `SelfDealsDamageToOpponent`.
* **Attachment family**, over `state.expect_object(source).attachments`, per attachment:
  * `EquippedCreatureDealsCombatDamageToPlayer` — **only when `is_combat`** and the target is a
    player (CR 510.3a: the printed text is *"deals combat damage"*). Unchanged behaviour.
  * when the target is a player: `EnchantedCreatureDealsAnyDamageToPlayer` always; plus
    `EnchantedCreatureDealsCombatDamageToPlayer` when `is_combat`; plus the two `…ToOpponent`
    siblings when the damaged player is an opponent **of that attachment's controller** (the check
    is per attachment, which is why it lives inside the loop — the same shape
    `TriggerEvent::SelfBecomesTargetByOpponent`'s doc describes).
* For every trigger queued in this call, set `damaged_player` (when the target is a player),
  `damage_dealt_amount = amount`, `entering_object_id = Some(source)`, and — **only when
  `is_combat`** — `combat_damage_amount = amount`.

Then:

* In the `GameEvent::CombatDamageDealt` arm, replace the inlined walk with a call per assignment
  (`is_combat: true`). The other collectors in that arm (batch triggers,
  `EquippedCreatureDealsCombatDamage`, …) stay where they are — this extraction is scoped to the
  per-assignment attachment/self walk.
* In the `GameEvent::DamageDealt { source, target, amount }` arm, call it with `is_combat: false`,
  **beside** the existing `SelfIsDealtDamage` (Enrage) collection, not replacing it.
* **Delete `TODO(PB-37)`** and the three in-def comments that echo it (`curiosity` ×2,
  `ophidian_eye` ×2 including its `completeness` sentence) — `OOS-DX47-6`: a false comment
  outlives the commit that falsifies it.

## Step 7 — card defs

| def | change | marker |
|---|---|---|
| `sigil_of_sleep` | `recipient: DamageRecipient::Player` | `Complete` (unchanged, marker-less) |
| `curiosity` | `recipient: DamageRecipient::Opponent`; delete both `TODO(PB-37)` comments | `partial`, note rewritten to the **costless "you may"** blocker alone |
| `ophidian_eye` | same | `partial`, note keeps its (2) and drops (1) and the PB-37 sentence |
| `exalted_angel` | author the printed trigger: `WhenDealsDamage { recipient: Any }` → `Effect::GainLife { player: Controller, amount: EffectAmount::DamageDealt }`; delete the header TODO and the in-`abilities` TODO | **`Complete`** — the batch's ONE flip |
| `goblin_lackey` | `WhenDealsCombatDamageToPlayer` → `WhenDealsDamage { recipient: Player }` (its own marker names this as blocker (c)) | `partial`, note drops (c) |
| `warren_instigator` | note rewritten: the trigger CONDITION is now expressible; the filtered hand→battlefield put and the costless "may" survive | `partial` |

Check every oracle line against MCP (`lookup_card`) before editing; the header comment of each
edited def must end up true.

## Step 8 — tests (all new files; SR-9a — never a top-level `tests/*.rs`)

* `crates/engine/tests/primitives/pb_dx36_damage_trigger_dispatch.rs` — behavioural probes:
  * `sigil_of_sleep` end-to-end on a **noncombat** damage event (a ping from the enchanted
    creature) — trigger fires **exactly once**, target announced on CR 603.3d's channel, the
    named creature returns to hand.
  * the same on a **combat** damage event — **exactly once** (the count is the verdict;
    `>= 1` passes on PB-DX47's double-push shape).
  * `exalted_angel` life gain equals the damage on a **combat** event and on a **noncombat**
    event.
  * `combat_only: true` fires on combat and **not** on noncombat; `combat_only: false` fires on
    both — the arm that was dead.
  * `recipient` discriminates: an `Opponent` trigger does not fire on damage to its own
    controller; a `Player` trigger does.
  * `EffectAmount::CombatDamageDealt` reads 0 on a noncombat trigger while
    `EffectAmount::DamageDealt` reads the amount (execution notes §0.5(d)).
  * on this variant `DamageRecipient::Any` and `::Player` behave identically (the stated
    equivalence, asserted rather than commented).
* `crates/engine/tests/core/pb_dx36_deals_damage_roster.rs` — the census, **PRINTED by a test**
  walking `all_cards()` (SR-36 — never grep source):
  * every def whose oracle text (front face **and** every `CardFace`) prints a *"deals damage"*
    trigger, classified: **new condition** / **existing narrower condition** / **still blocked**,
    with the blocker named. Print the table; ratchet the counts.
  * reconcile with PB-DX47's existing inverse ratchet
    (`core::pb_dx47_dispatch_path_roster`) rather than double-counting — cite it and state the
    partition.
  * a mechanism gate: **no second dispatcher** for any of the seven new `TriggerEvent`s
    (PB-DX47's `r3` shape, keyed on the mechanism — an ability-list walk near the variant name —
    not on one syntactic form).
  * a gate that the lowering's two `match`es are exhaustive with no `_ =>` arm.
* `crates/simulator/tests/pb_dx36_damage_trigger_channel.rs` — the human channel: the
  `sigil_of_sleep` noncombat trigger offered and answered through `LocalGame`/`HumanChoice`.

Every new gate and probe must be proven RED by an **executed** revert; record the matrix in
`memory/primitives/pb-DX36-execution-notes.md`. Disclose any UNDISCRIMINATED row **in the test's
own doc comment**, not only in `memory/`.

## Step 9 — wire

Only after steps 1-8 are green. Take both numbers from the failing gates' own output —
never invent them. Expect **PROTOCOL 41 → 42** and **HASH 82 → 83**, closure type counts
**98 / 132 unchanged** (execution notes §0.3). Append history rows (never edit a shipped row),
re-pin `FROZEN_HISTORY_PREFIX_DIGEST`, re-pin every scattered sentinel **by symbol**, then
**survivor-scan with a differently-shaped regex** covering the multi-line and `u8`-suffix
spellings (`OOS-DX20b`/`OOS-DX50`), and read every changed line of the diff for an
**over**-replacement (`OOS-DX18-3`). `history_is_append_only` and `frozen_prefix_is_pinned` must
be green on both gates.

## Step 10 — the corpus re-deal

`exalted_angel`'s marker moves, so `CORPUS_COMPLETE` moves and **every seeded fixture must be
re-observed by an EXECUTED sweep** (`OOS-CARDS2-3`) — `UI3_SPLIT_COMBAT_SEED` and its siblings.
Re-measure `COMMANDER_POOL` rather than reasoning about it. Budget **two** reconciliations if the
`/review` moves a marker (PB-DX27).
