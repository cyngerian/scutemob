# Primitive Batch Plan: PB-EF8 — `Cost::ExileSelfFromHand` (activation from hand)

**Generated**: 2026-07-18
**Primitive**: A new activation-cost variant `Cost::ExileSelfFromHand` (+ decorative
`ActivationZone::Hand`) that lets a *mana ability* activate from the owner's **hand**,
exile the source card as the cost (CR 400.7 — the card becomes a new object in exile),
and produce mana **stacklessly** through the mana-ability-lowering path
(`mana_ability_lowering` → `handle_tap_for_mana`), never the stack-using
`handle_activate_ability`.
**CR Rules**: 605.1a, 605.3a/b, 605.5, 602.1, 602.2/602.2a, 118 (cost), 400.7, 106.12
**Cards affected**: 2 (2 existing flips, 0 new) — Simian Spirit Guide (`inert` → Complete),
Elvish Spirit Guide (`known_wrong` → Complete)
**Dependencies**: none beyond shipped engine (mirrors `ActivationZone::Graveyard` from PB-35
and `Cost::DiscardSelf`/Channel; reuses the SR-34/36/37 `handle_tap_for_mana` payment
machinery). Finding: **EF-W-PB2-8**.
**Deferred items from prior PBs**: none carried in. This closes EF-W-PB2-8.

**TODO sweep (mandatory roster-recall gate)**: grepped `crates/card-defs/src/defs/` for
`ExileSelfFromHand`, `exile.*this.*from.*hand`, `activation_zone.*Hand`, and "pitch for
mana". Result: **exactly 2 cards** self-identify (`simian_spirit_guide.rs` inert note names
`Cost::ExileSelfFromHand` + `activation_zone: Hand` verbatim; `elvish_spirit_guide.rs`
known_wrong note names `ActivationZone::Hand` + "exile-self-from-hand cost"). False positives
verified out of scope and recorded in `memory/primitive-wip.md`: `saw_it_coming` (Foretell
keyword, already Complete), `chrome_mox` (Imprint ETB exile, unrelated primitive),
`gemstone_caverns` (Luck-counter ETB payload, battlefield `{T}` ability). **No third card.**
The coordinator's corpus sweep (in the wip) and this sweep agree: **2 flips**.

---

## Primitive Specification

Simian Spirit Guide (`{2}{R}` Ape Spirit 2/2) and Elvish Spirit Guide (`{2}{G}` Elf Spirit
2/2) each print one line: **"Exile this card from your hand: Add {mana}."** By CR 605.1a this
IS a mana ability — it has no target, it could add mana when it resolves, and it is not a
loyalty ability. Per CR 605.3b a mana ability does not use the stack; per CR 605.5 it is a
special action that does not reset priority or `players_passed`. Therefore the ONLY correct
implementation routes through the stackless mana path (`Command::TapForMana` →
`handle_tap_for_mana`), exactly as a Treasure's `{T}, Sacrifice: Add any` does — **not**
`handle_activate_ability` (which puts the ability on the stack and would illegally hand
opponents a response window). Routing a mana ability through the stack was the SR-33 defect;
this batch must not reintroduce it.

The distinguishing feature versus every existing mana ability is the **activation zone**: the
source is in the owner's hand, not on the battlefield, and it is not tapped (a card in hand
cannot tap — CR 106.12 / CR 302). The cost paid is exiling the source card (CR 400.7 makes it
a dead ObjectId afterward), which is the ability's natural exhaustion mechanism: it cannot be
activated twice because the card leaves the hand.

The current defs are actively wrong, not merely incomplete: both previously shipped a **free,
repeatable battlefield `Add {mana}`** (`Cost::Mana(ManaCost::default())`) = unbounded infinite
mana. Elvish is `known_wrong`; Simian was demoted to `inert` (its wrong ability removed) in
W-PB2. This batch replaces both with a faithful, one-shot, from-hand, stackless mana ability.

---

## CR Rule Text (from MCP lookup)

**605.1a** — "An activated ability is a mana ability if it meets all of the following
criteria: it doesn't require a target (see rule 115.6), it could add mana to a player's mana
pool when it resolves, and it's not a loyalty ability." → Both cards qualify: no target, adds
`{R}`/`{G}`, not loyalty. **The classification is by what the ability does, not by its cost or
its activation zone** — so the hand zone does not disqualify it, and the SR-34 lowering
predicate (`mana_ability_lowering`) applies.

**605.3a** — a player may activate a mana ability "whenever they have priority, whenever they
are casting a spell or activating an ability that requires a mana payment, or whenever a rule
or effect asks for a mana payment" — i.e. Spirit Guides can be activated mid-cast to help pay,
which is their entire purpose. (The engine already supports this for any lowered mana ability;
no new work.)

**605.3b** — "An activated mana ability doesn't go on the stack, so it can't be targeted,
countered, or otherwise responded to. Rather, it resolves immediately after it is activated."
→ Must use `handle_tap_for_mana`, which never touches the stack. Test asserts
`stack_objects().is_empty()`.

**605.5** — abilities not meeting 605.1a–b are not mana abilities. (Confirms these two ARE,
and the guardrail: the ability has no target, so it is not knocked out by 605.5a.)
Combined with the stackless-path invariant (mana.rs step 11): `players_passed` and
`priority_holder` are unchanged by activation.

**602.1 / 602.2 / 602.2a** — "Only an object's controller (or its owner, if it doesn't have a
controller) can activate its activated ability." A hand card has no controller; its owner
activates it. 602.2a: activating from a hidden zone reveals the card. (The engine does not
model reveal-from-hidden-zone separately for mana abilities; the exile event makes the card
public. Out of scope to add a Reveal event — note only.)

**400.7** — "An object that moves from one zone to another becomes a new object with no memory
of, or relation to, its previous existence." → After the exile-cost move, `source` is a dead
ObjectId; the exiled card is a new object. The happy-path test asserts `state.object(source)`
is gone and a new object exists in Exile.

**106.12** — "To 'tap [a permanent] for mana' is to activate a mana ability of that permanent
that includes the {T} symbol in its activation cost." → **The Spirit Guides do NOT tap for
mana** (no `{T}` in cost, and a hand card can't tap). Therefore mana-production replacements
(Nyxbloom Ancient, Mana Reflection — `ReplacementTrigger::ManaWouldBeProduced`) and
"whenever you tap a permanent for mana" triggers (CR 106.12a) **do not apply**. This is
correct-by-construction: both effects are gated on `ability.requires_tap` in `handle_tap_for_mana`
(steps 7b and 10), which is `false` here.

**118 (cost)** — the exile is a cost, paid on activation before mana is produced (602.2b →
601.2h), consistent with how `sacrifice_self` (Treasure) is paid at step 7 of
`handle_tap_for_mana`.

---

## Design decision — Scoping Decision #3 resolved

**`Cost::ExileSelfFromHand` is the single source of truth. `ActivationZone::Hand` is added but
purely decorative/documentary; the engine keys ALL behavior off the cost.** This is the
cleanest shape, justified as follows:

- CR 605.1a classifies the ability by what it does; the lowering predicate
  `mana_ability_lowering(targets, cost, effect, activation_condition)` already decides mana-vs-
  stack purely from `targets`/`cost`/`effect` and **is not passed `activation_zone`**
  (replay_harness.rs:2130). Threading `activation_zone` into the mana loop would add a second,
  redundant switch that could drift out of sync with the cost — exactly the failure mode SR-34
  §3 step 5 warns about ("the two lists can never disagree because it's the same call").
- The precedent is `Cost::DiscardSelf`: Channel's hand activation is driven entirely by the
  cost (`abilities.rs:172-188` reads `ab.cost.discard_self`), with no `activation_zone` needed.
  `ActivationZone::Graveyard` exists only because Reassembling Skeleton's ability is a
  **stack-using non-cost-marked** graveyard ability with a plain `Cost::Mana` — there is no cost
  variant that says "from graveyard", so the zone enum carries that fact. Here the cost variant
  IS self-describing ("...FromHand"), so the zone would be pure duplication.
- We still ADD `ActivationZone::Hand` (task requires the concept; future-proofs a *non-mana*
  hand-activated ability that would need a stack-path zone marker analogous to Graveyard) and
  both defs carry `activation_zone: Some(ActivationZone::Hand)` to document intent. But because
  the mana path never reads it, **there is no state to keep in sync** — the zone check in
  `handle_tap_for_mana` keys on `ManaAbility::exile_self_from_hand`, lowered from the cost.

Net: the cost variant drives lowering AND legality; the zone enum is inert metadata that a
future stack-path hand ability could activate. No redundancy in the behavioral path.

---

## Engine Changes (ordered; each names file + approx line + CR)

### Step 1 — DSL enum/variant additions (`mtg-card-types`)

**1a.** `crates/card-types/src/cards/card_definition.rs` — `Cost` enum (ends at L1262, after
`Exert`). **Add** `ExileSelfFromHand`, with a doc comment: "CR 118 + CR 400.7 + CR 605.1a:
exile the source card **from hand** as an activation cost. The from-hand mana-ability analog of
`DiscardSelf` (Channel, CR 702.34) — differs in that the card is exiled, not discarded, and the
effect is mana produced stacklessly. Drives mana-ability lowering
(`mana_ability_cost_components`) and the from-hand zone-legality branch in
`handle_tap_for_mana`; not a stack-path cost." (No payload — the source is `self`, identified by
`Command::TapForMana`'s ObjectId, exactly like `SacrificeSelf`.)

**1b.** Same file — `ActivationZone` enum (L4156, currently only `Graveyard`). **Add** `Hand`,
doc: "CR 602.2: opt-in exception marker for a hand-activated ability. Decorative for mana
abilities (those are driven by `Cost::ExileSelfFromHand`); reserved for a future non-mana
hand-activated stack ability, analogous to `Graveyard`." **This is an exhaustive-match trip
point** — see the match-site table below.

### Step 2 — runtime struct fields (`mtg-card-types`)

**2a.** `crates/card-types/src/state/game_object.rs` — `ManaAbility` struct (fields end L237).
**Add** `#[serde(default)] pub exile_self_from_hand: bool,` with doc citing CR 118/605.1a: the
flag `handle_tap_for_mana` reads to switch to the from-hand exile-cost payment path (mirrors
`sacrifice_self`). Defaults `false`; `ManaAbility` derives `Default`, so every existing
construction site (`::tap_for`, `::treasure`, `..Default::default()`, the `try_as_tap_mana_ability`
literals) is unaffected.

**2b.** Same file — `ActivationCost` struct (fields end L345). **Add** `#[serde(default)] pub
exile_self_from_hand: bool,` with doc: mirrors `discard_self`; set by `flatten_cost_into` so the
exhaustive `Cost` match stays total. (The mana path does not read this struct; it exists so the
stack-path flatten function handles the new `Cost` variant.)

### Step 3 — mana-ability lowering (`crates/engine/src/testing/replay_harness.rs`)

**3a.** `ManaAbilityCost` struct (L3676). **Add** `exile_self_from_hand: bool,`.

**3b.** `mana_ability_cost_components` (L3711):
- Init `acc` (L3751): add `exile_self_from_hand: false,`.
- Match arm: **remove** `Cost::ExileSelfFromHand` from the not-lowerable `... => false` group
  (it does not exist yet — the new variant forces this exhaustive match to be updated) and add
  an **accepting** arm: `Cost::ExileSelfFromHand => { acc.exile_self_from_hand = true; true }`.
  Rationale (CR 605.1a): the source is `self`, identified by `Command::TapForMana`'s ObjectId —
  no caller-supplied payload is needed, exactly like the already-accepted `SacrificeSelf`.
- **Relax the no-tap guard** (L3760): change `if !acc.requires_tap { return None; }` to
  `if !acc.requires_tap && !acc.exile_self_from_hand { return None; }`. **CR justification**: the
  SR-34 guard declines a *no-tap* cost because such a cost has no exhaustion mechanism and would
  register a free, repeatable, stackless mana ability (its doc comment L3699-3710 names
  Elvish/Simian as the exact seam it closed). An `exile_self_from_hand` cost is **inherently
  one-shot and self-consuming** (CR 400.7: the source leaves hand → dead ObjectId → cannot be
  activated again), so the seam does not apply. The relaxation is scoped to *only* this flag —
  a `Cost::Mana`-only or `SacrificeSelf`-only no-tap cost (Food Chain) is still declined. Update
  the function's doc comment to record this exception and cite CR 605.1a/400.7.

**3c.** `mana_ability_lowering` (L3786-3808): after `ma.life_cost = components.life_cost;` add
`ma.exile_self_from_hand = components.exile_self_from_hand;`. (CR 605.1a — carry the cost
component onto the lowered `ManaAbility`.)

**3d.** `flatten_cost_into` (L3952 match): **add arm** `Cost::ExileSelfFromHand =>
ac.exile_self_from_hand = true,`. (Exhaustive `Cost` match — forced.)

### Step 4 — optional-cost catch-alls (`crates/engine/src/effects/mod.rs`)

Two exhaustive `Cost` matches in the CR 118.12 "may pay a cost then effect" path must gain the
new variant (neither is reachable for our cards — mana abilities are not resolution-time
optional costs — but both are exhaustive):
- `can_pay_optional_cost` (L7905): add `| Cost::ExileSelfFromHand` to the `... => false` group
  (an exile-self-from-hand cost has no optional-cost channel).
- `pay_optional_cost` (L7959): add `| Cost::ExileSelfFromHand` to the unreachable `... => {}`
  group.

### Step 5 — `handle_tap_for_mana` core change (`crates/engine/src/rules/mana.rs`)

**5a. Reorder + conditional zone/controller legality.** Currently: step 2 clones `obj` (L121);
step 3 rejects `obj.zone != Battlefield` (L123); step 4 rejects `obj.controller != player`
(L127); step 5 fetches the ability (L133-146). The ability's `exile_self_from_hand` flag is not
known until step 5, so the zone check must move after the fetch. **Move the ability-fetch block
(L133-146) to immediately after step 2 (before the zone check)**, then replace steps 3-4 with:

```rust
if ability.exile_self_from_hand {
    // CR 602.1/602.2/605.1a: a from-hand mana ability (Spirit Guides). The source must be
    // in the activating player's hand (mirrors the Channel check in handle_activate_ability,
    // abilities.rs:179-188 — a hand card's controller is its owner, so check owner).
    if obj.zone != ZoneId::Hand(player) {
        return Err(GameStateError::InvalidCommand(
            "from-hand mana ability can only be activated from hand (CR 602.2, 605.1a)".into(),
        ));                                    // ← decoy A fails here (same card on battlefield)
    }
    if obj.owner != player {
        return Err(GameStateError::InvalidCommand(
            "you can only activate a from-hand mana ability on a card you own".into(),
        ));
    }
} else {
    if obj.zone != ZoneId::Battlefield {
        return Err(GameStateError::ObjectNotOnBattlefield(source)); // ← decoy B fails here
    }
    if obj.controller != player {
        return Err(GameStateError::NotController { player, object_id: source });
    }
}
```

Note the restriction sweep (step 1b, L60-119) already no-ops for a hand source: it computes
`source_on_bf = false`, so Stony Silence / Grand Abolisher checks all skip. No change there.
Note `expect_characteristics` (used by the ability fetch) works for a hand object — the
graveyard-activated path uses it off-battlefield too; a hand card's `mana_abilities` are its
base (enriched) list since no layers apply off-battlefield.

**5b. Skip tap** — already correct: step 6 (L190) is gated on `ability.requires_tap`, which is
`false` for these abilities, so the tapped-status and summoning-sickness checks are correctly
skipped (a hand card cannot tap — CR 106.12/302).

**5c. Pay the exile cost** — insert a new step **after the `sacrifice_self` block (after L325),
before step 7b (L326)**, guarded on `ability.exile_self_from_hand` (mutually exclusive with
`sacrifice_self` — no card has both). Mirror the pitch-exile shape at abilities.rs:2383-2390:

```rust
// CR 118 cost + CR 400.7: exile the source card from hand as the activation cost. After the
// move `source` is a dead ObjectId; the card is a new object in exile. This is the ability's
// exhaustion mechanism (it cannot be activated twice — the card is no longer in hand).
if ability.exile_self_from_hand {
    let (new_exile_id, _) = state.move_object_to_zone(source, ZoneId::Exile)?;
    events.push(GameEvent::ObjectExiled {
        player,
        object_id: source,
        new_exile_id,
        pre_lba_counters: imbl::OrdMap::new(), // hand card: no battlefield counters
        pre_lba_power: None,                   // not a leaves-battlefield event; no LKI power
    });
}
```

**5d. Produce mana / skip tap-gated machinery** — steps 7b (mana-production replacement, L330),
10 (WhenTappedForMana triggers, L413) are gated on `requires_tap` (false) → correctly skipped
(CR 106.12: exile-from-hand is not tapping, so Nyxbloom/Mana Reflection and
Forbidden-Orchard-style triggers do not fire). Step 8 (L349) produces `{R}`/`{G}` from
`ability.produces` unchanged. The `ManaAdded` event's `source: Some(source)` is a dead ObjectId
— identical to the Treasure `sacrifice_self` case, consistent with CR 400.7 LKI treatment
already used there.

**5e. Priority retained** — step 11 (L423): `players_passed` and `priority_holder` unchanged.
This is the function's existing invariant (CR 605.5); the new branches do not touch turn state.

### Step 6 — hashing (`crates/engine/src/state/hash.rs`)

| Site | Line | Action |
|------|------|--------|
| `Cost` HashInto | L5781 (after `Cost::Exert => 12u8`) | Add `Cost::ExileSelfFromHand => 13u8.hash_into(hasher),` |
| `ActivationZone` HashInto | L5352 (after `Graveyard => 0u8`) | Add `ActivationZone::Hand => 1u8.hash_into(hasher),` |
| `ManaAbility` HashInto | after L1569 (`activation_condition`) | Add `self.exile_self_from_hand.hash_into(hasher);` |
| `ActivationCost` HashInto | after L2830 (`sacrifice_exclude_self`) | Add `self.exile_self_from_hand.hash_into(hasher);` |

Both new struct-field hashes satisfy the source-scanning gate
`tests/core/hash_schema.rs::every_hashed_struct_field_is_hashed_or_allowlisted` (it requires
each declared field to appear in the impl body — no `NOT_HASHED` allowlist entry needed).
`MIN_COVERED_STRUCTS`/`MIN_FIELDS_CHECKED` are minimums; they do not need bumping.

### Step 7 — version bumps (read forced values from failing gates; do NOT hand-guess)

- `crates/engine/src/state/hash.rs` L448: `HASH_SCHEMA_VERSION` currently **50**. The new bool
  fields on `ManaAbility`/`ActivationCost` (both inside `GameState`) reach the hash closure, so
  the byte stream changes → bump machine-forced. Read the new value from the failing
  `tests/core/hash_schema.rs` sentinel; add a changelog comment citing PB-EF8.
- `crates/engine/src/rules/protocol.rs` L130: `PROTOCOL_VERSION` currently **12**. `Cost` and
  `ActivationZone` are in the SR-8 wire type closure (Characteristics → Effect → DSL), so adding
  a `Cost` variant + `ActivationZone` variant is a wire-shape change → bump + new
  `PROTOCOL_SCHEMA_FINGERPRINT` machine-forced. Read both from the failing
  `tests/protocol_schema.rs` output. **`Command::TapForMana` is UNCHANGED** (no new payload —
  the hand source is identified by ObjectId exactly as a battlefield source is); only the
  transitive DSL closure moves.

### Change — Exhaustive match sites (the #1 compile-error source)

| File | Match | Action |
|------|-------|--------|
| `crates/engine/src/testing/replay_harness.rs` | `mana_ability_cost_components` walk (L3713) | New accepting arm (Step 3b) |
| `crates/engine/src/testing/replay_harness.rs` | `flatten_cost_into` (L3953) | New arm (Step 3d) |
| `crates/engine/src/effects/mod.rs` | `can_pay_optional_cost` (L7905) | `\| Cost::ExileSelfFromHand` → false (Step 4) |
| `crates/engine/src/effects/mod.rs` | `pay_optional_cost` (L7959) | `\| Cost::ExileSelfFromHand` → {} (Step 4) |
| `crates/engine/src/state/hash.rs` | `Cost` HashInto (L5746) | discriminant 13 (Step 6) |
| `crates/engine/src/state/hash.rs` | `ActivationZone` HashInto (L5350) | `Hand => 1u8` (Step 6) |

Verified there are **no other exhaustive `Cost` matches** in engine source (grep of
`Cost::Exert`/`Cost::ExileSelf` across `crates/`: the only non-test, non-def source matches are
the 6 sites above; `abilities.rs` reads `ability_cost.exile_self` as a bool, not a `Cost`
match). No exhaustive `ActivationZone` match beyond the enum def and its HashInto (the
`abilities.rs` graveyard check uses `if let Some(ActivationZone::Graveyard)`, which is not
exhaustive and needs no change). **The runner must still `cargo build --workspace` after the
enum adds** — the TUI/replay-viewer display matches are on `StackObjectKind`/`KeywordAbility`,
not `Cost`/`ActivationZone`, so they are not expected to break, but build is the proof.

---

## Card Definition Fixes

### simian_spirit_guide.rs  (added via pre-existing TODO sweep — the def's inert note names the exact primitive)
**Oracle**: "Exile this card from your hand: Add {R}."
**Current state**: `inert` (its prior wrong `Cost::Mana(default)` free `Add {R}` was removed in
W-PB2; note explicitly requests `Cost::ExileSelfFromHand` + `activation_zone: Hand`).
**Fix**: replace the empty `abilities: vec![]` + `Completeness::inert(...)` with:
```rust
abilities: vec![
    AbilityDefinition::Activated {
        cost: Cost::ExileSelfFromHand,
        effect: Effect::AddMana {
            player: PlayerTarget::Controller,
            mana: ManaPool { red: 1, ..Default::default() },
        },
        timing_restriction: None,
        targets: vec![],
        activation_condition: None,
        activation_zone: Some(ActivationZone::Hand),
        once_per_turn: false,
        modes: None,
    },
],
```
Drop the `completeness` field (defaults to `Complete`). Update the header comment to cite
PB-EF8. Rewrite the oracle-text line if needed (already correct).

### elvish_spirit_guide.rs
**Oracle**: "Exile this creature from your hand: Add {G}." (Oracle-updated wording; the def's
`oracle_text` currently reads "...this card..." — match the printed card; either is acceptable
but prefer the current Oracle "creature" wording per MCP lookup. Keep consistent with the file's
existing string to avoid churn — a note, not a blocker.)
**Current state**: `known_wrong` — ships a FREE battlefield `Add {G}` (`Cost::Mana(default)`,
`activation_zone: None`) = infinite mana. **This is a live wrong-state bug removed by this fix.**
**Fix**: replace the `Cost::Mana(ManaCost::default())` with `Cost::ExileSelfFromHand`, set
`activation_zone: Some(ActivationZone::Hand)`, keep `effect: Effect::AddMana { green: 1 }`, drop
the `Completeness::known_wrong(...)` marker (defaults to `Complete`), remove the two `TODO`
comments. Update the header comment to cite PB-EF8.

---

## New Card Definitions

None. Scope is exactly the 2 flips (Scoping Decision #1).

---

## Unit Tests

**File**: `crates/engine/tests/primitives/pb_ef8_exile_self_from_hand.rs`
**Register**: add `mod pb_ef8_exile_self_from_hand;` to
`crates/engine/tests/primitives/main.rs` (after L28 `mod pb_ef7_modal_activated;`) — **required
by SR-9a** (a group file with no `mod` line is silently uncompiled;
`no_stray_test_binaries.rs` fails otherwise).
**Pattern**: follow `tests/primitives/primitive_sr34_composite_mana_costs.rs` — the `make_spec`
helper takes a `ZoneId`, so build the source directly in `ZoneId::Hand(p(1))`; use
`Command::TapForMana { player, source, ability_index: 0 }`; assert pool via `pool_amount`; assert
stacklessness via `state.stack_objects().is_empty()`. The synthetic-def decoy pattern is
`sr34_gates_are_not_vacuous` (L810). Every test cites its CR.

Tests to write:

- `simian_activates_from_hand_and_exiles_the_source` — **happy path (CR 605.1a/118/400.7)**.
  Build Simian Spirit Guide in `ZoneId::Hand(p(1))`, priority p(1). `TapForMana` succeeds:
  assert `pool_amount(state, p(1), Red) == 1`; assert `state.object(source).is_none()` (dead
  ObjectId, CR 400.7); assert exactly one new object exists in `ZoneId::Exile` with the card's
  name; assert `state.stack_objects().is_empty()` (CR 605.3b); assert an `ObjectExiled { object_id:
  source, .. }` event was emitted.
- `elvish_activates_from_hand_adds_green` — same for Elvish Spirit Guide → `Green == 1`, source
  exiled. (Locks the second flip and its color.)
- `from_hand_mana_ability_does_not_reset_priority_or_players_passed` — **stackless invariant
  (CR 605.5)**. Seed `state.turn_mut().players_passed` with a non-empty `OrdSet` (e.g.
  `{p(2)}`) and record `priority_holder`. After `TapForMana`, assert `players_passed` is
  **unchanged** (still `{p(2)}`) and `priority_holder == Some(p(1))`.
- `decoy_a_same_card_on_battlefield_cannot_use_from_hand_ability` — **decoy A (must fail on
  exactly the zone check)**. Build Simian on the **battlefield** (still lowers to a mana ability
  with `exile_self_from_hand = true`). `TapForMana` must return `Err` (the from-hand zone error).
  Assert the source is still on the battlefield and the pool is empty. **Non-vacuity note for the
  runner**: deleting the `if obj.zone != ZoneId::Hand(player)` check in the `exile_self_from_hand`
  branch makes this test pass (the ability would proceed to exile a battlefield permanent and add
  `{R}`), so the assertion pins exactly that check.
- `decoy_b_battlefield_only_mana_ability_cannot_be_activated_from_hand` — **decoy B (must fail on
  exactly the battlefield check)**. Take a basic land (Forest — `requires_tap = true`,
  `exile_self_from_hand = false`) and place its source in `ZoneId::Hand(p(1))`. `TapForMana` must
  return `Err(ObjectNotOnBattlefield)`. Assert pool empty, card still in hand. **Non-vacuity**:
  deleting the `else`-branch `if obj.zone != ZoneId::Battlefield` check lets a hand card produce
  mana; this pins that check.
- `pb_ef8_lowering_gate_is_not_vacuous` — **synthetic-def gate test** (mirrors
  `sr34_gates_are_not_vacuous`). (a) A synthetic def with a single `Cost::ExileSelfFromHand` +
  `Effect::AddMana{R}` **lowers** to a `ManaAbility` (`exile_self_from_hand == true`,
  `requires_tap == false`) and is **excluded from `activated_abilities`**. (b) A synthetic def
  with `Cost::Sequence([DiscardCard, ExileSelfFromHand])` does **not** lower (DiscardCard needs a
  caller-supplied card → declined), proving the no-tap-guard relaxation is scoped to the flag,
  not to all no-tap costs.
- `from_hand_mana_ability_does_not_fire_tapped_for_mana_replacements` — **CR 106.12**. With a
  Nyxbloom-Ancient-style `ManaWouldBeProduced` multiplier replacement active for p(1), activate
  Simian from hand and assert the pool is still exactly `{R} == 1` (the multiplier does NOT
  apply, because exiling from hand is not tapping for mana). If wiring a real replacement is
  heavy, assert the narrower invariant that `requires_tap == false` on the lowered ability and
  that step 7b/10 are unreached (comment-cite CR 106.12). Keep this test if it can be made
  non-vacuous cheaply; otherwise fold the CR-106.12 assertion into the happy-path test as a
  comment + `requires_tap` check.

---

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check -p mtg-engine`) — all 6 exhaustive-match sites updated
- [ ] `cargo build --workspace` clean (proves TUI/replay-viewer display matches unaffected + SR-3 seal)
- [ ] simian_spirit_guide.rs + elvish_spirit_guide.rs flipped to Complete (markers dropped, TODOs removed)
- [ ] `mod pb_ef8_exile_self_from_hand;` added to `tests/primitives/main.rs`
- [ ] Unit tests pass (`cargo test --all`) — incl. decoys proven non-vacuous per notes above
- [ ] `hash_schema.rs` field-coverage gate green (new fields hashed, not allowlisted)
- [ ] HASH_SCHEMA_VERSION + PROTOCOL_VERSION + PROTOCOL_SCHEMA_FINGERPRINT bumped to the exact
      values the failing gates print (never hand-guessed); changelog comments cite PB-EF8
- [ ] Clippy clean (`cargo clippy --all-targets -- -D warnings`)
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` clean (SR-35 — the script is the only
      thing that checks the two touched card defs)
- [ ] No remaining TODOs in the two affected card defs
- [ ] Authoring report re-run (`python3 tools/authoring-report.py`) reflects +2 clean

---

## Risks & Edge Cases

- **Reorder in `handle_tap_for_mana` (Step 5a).** The ability fetch must move above the
  zone/controller check because the branch keys on `ability.exile_self_from_hand`. Verify nothing
  between old-step-2 and old-step-5 mutates state (it does not — steps 3-4 are pure validation),
  so the fetch relocation is behavior-preserving for the battlefield path. Re-run the full
  `primitive_sr34/36/37` + `mana_triggers` + `treasure_tokens` suites to confirm no regression on
  the existing tap-mana paths.
- **`ObjectExiled` for a hand→exile move** is consumed by the `WhenSourceLeavesBattlefield`
  delayed-trigger scan (abilities.rs:6139) and by any exile-matters trigger checker. This is
  benign: no `WhenSourceLeavesBattlefield` delayed trigger will ever reference a *hand-card*
  ObjectId (those are created for battlefield permanents), and a card genuinely being exiled is
  correctly described by the event. Do not invent a separate "exiled from hand" event — the
  pitch-cost path (Force of Will, abilities.rs:2383) already uses `ObjectExiled` for hand exile;
  reuse it for consistency.
- **Reveal-from-hidden-zone (CR 602.2a)** is not modeled as a distinct event; the exile makes the
  card public. Acceptable for this batch (mana abilities don't need the reveal machinery Foretell
  uses); note only, no work.
- **`ManaAdded.source` is a dead ObjectId** after the exile. This matches the existing Treasure
  `sacrifice_self` behavior (CR 400.7 LKI); any consumer already tolerates a dead source id.
- **Interaction with `Effect::AddMana` stack path.** Because the ability lowers to a `ManaAbility`
  and is excluded from `activated_abilities`, `handle_activate_ability` will never see it — verify
  the exclusion predicate (`mana_ability_lowering(...).is_some()` at replay_harness.rs:2154) fires
  for these defs so `TapForMana { ability_index: 0 }` finds the ability and `ActivateAbility` does
  not.
- **Non-vacuity discipline (SR-34/SR-9c lesson).** Both decoys must be shown to flip to green when
  their specific check is deleted; the lowering-gate test must show the negative (DiscardCard
  sequence) direction. A decoy that passes for the wrong reason (e.g. rejected on ability-index
  rather than zone) is worthless — the runner must confirm the error variant, not just `is_err()`.
