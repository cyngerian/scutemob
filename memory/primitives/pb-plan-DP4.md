# Primitive Batch Plan: PB-DP4 — Costs checked but never collected (DP-10 + DP-11)

**Generated**: 2026-07-26
**Primitive**: not a new DSL variant — two **cost-collection** fixes of the same shape.
(a) **DP-10**: the Propaganda / Ghostly Prison attack tax becomes a real, colour-correct
`ManaCost` debit against the declaring player's pool (`rules/combat.rs`), and CR 508.1d's
"you are never required to pay an attack cost" is honoured by the must-attack "able" test.
(b) **DP-11**: echo / cumulative upkeep / recover gain a **deadline** — an unanswered
pay-or-lose-it payment is closed out with the CR-mandated "otherwise" branch at the point the
game would leave the priority round in which the ability resolved (`rules/engine.rs`), and the
three payments gain simulator `LegalAction`s so a bot / an M11-local seat can actually answer.
**CR Rules**: 508.1, 508.1c, 508.1d, 508.1f, 508.1g, 508.1h, 508.1i, 508.1j, 106.6, 107.4e,
107.4f, 118.12, 118.12a, 119.4, 119.4b, 608.2d, 608.2g, 702.30a, 702.24a, 702.24b, 702.59a,
701.21a, 400.7, 117.3b, 117.3c, 117.4, 101.4
**Cards affected**: **5 `Complete` defs are live-wrong today and are made right with 0 edits**
— `propaganda.rs`, `ghostly_prison.rs` (tax never charged), `mogg_war_marshal.rs`,
`avalanche_riders.rs` (echo never enforced), `grim_harvest.rs` (recover never exiles). Plus
2 non-`Complete` defs whose notes stop being aspirational: `mystic_remora.rs` (`known_wrong`,
its note's "Cumulative upkeep {1} … are correct" claim becomes true), `tombstone_stairwell.rs`
(`partial`, unrelated blockers survive). **Expected card-def edits: 0** (one optional
comment-accuracy touch, §6).
**Dependencies**: PB-DP1 (`scutemob-149`, actor priority — supplies the CR 117.3c reasoning
this PB uses to *delete* three priority writes), PB-DP2 (`scutemob-150`), PB-DP3
(`scutemob-151`), PB-RS2 (`scutemob-144`, hybrid/Phyrexian flattening contract), SR-36
(`life_cost` payment paths + `GameStateError::InsufficientLife`), SR-38 (the
"provider only offers what the engine accepts" rule). **All present.**
**Deferred items from prior PBs in scope**: **OOS-DP1-1** (the three
`priority_holder = Some(active_player)` bodges — *closed by deletion*, §3 Change 2d) and
**OOS-RS3-4** (`memory/primitives/rider-seed-triage-2026-07-19.md`, the Goblin Rabblemaster
must-attack-vs-attack-tax deadlock — *closed*, §3 Change 1c). The RS queue stays paused; no
other RS item is touched.

---

## 0. STOP-CHECK: wire prediction — **NONE**. PROTOCOL 27 / HASH 63 unmoved.

**Prediction and the reasoning that supports it:**

| surface | why it is untouched |
|---|---|
| `Command` | `PayEcho`, `PayCumulativeUpkeep`, `PayRecover` and `DeclareAttackers` all already exist with the fields this plan needs. **No new field is added to `DeclareAttackers`** — the tax is derived entirely from `state.restrictions` + the declared `attackers` vector, so the in-code claim at `combat.rs:197-198` ("Interactive payment is deferred to post-alpha (requires a new `DeclareAttackers` command field)") is **falsified**: a full auto-debit needs no command field. |
| `GameEvent` | The attack-tax debit reuses the existing `GameEvent::ManaCostPaid { player, cost }` (`rules/events.rs:183`), which is already the universal payment event at 14 engine sites. The three `*PaymentRequired` events and `EchoPaid` / `CumulativeUpkeepPaid` / `RecoverPaid` / `RecoverDeclined` / `CreatureDied` / `ObjectExiled` all already exist and are reused verbatim. |
| `GameState` | No field added. `pending_echo_payments` / `pending_cumulative_upkeep_payments` / `pending_recover_payments` already exist (`state/mod.rs:263`, `:271`, `:283`), already have read accessors (`:535-549`) and `_mut` escape hatches (`:774-792`), and are already hashed (`state/hash.rs:7736-7754`). **The seal (SR-3) is not widened** — the sweep lives in `rules/engine.rs`, which is inside the crate and uses the private fields directly, exactly as the existing handlers do. |
| `Effect` / `StackObject` / `CardDefinition` | untouched. |
| `LegalAction` | 3 new variants — `crates/simulator`, **outside** the engine's wire closure. PB-RS2 / PB-DP3 precedent. |

`PROTOCOL_SCHEMA_FINGERPRINT` (`rules/protocol.rs:277`) digests the transitive type closure of
`Command`/`GameEvent`; `HASH_SCHEMA_VERSION` covers the serialized `GameState` field shape.
Neither closure moves. **`PROTOCOL_VERSION` stays 27, `HASH_SCHEMA_VERSION` stays 63. Do not
edit either constant.**

**Stop-and-re-scope instruction.** If the runner reaches a point where a new `Command` field, a
new `GameEvent` variant, or a new `GameState` field seems necessary — the three most likely
temptations are enumerated below — **STOP, do not bump the constants, and report in the ESM task
comment plus the review file**:

1. *"The attack tax has hybrid/Phyrexian pips and `DeclareAttackers` has nowhere to put the
   choice."* → **do not add the field.** Reject the declaration (§3 Change 1a step 1) and file
   seed OOS-DP4-1.
2. *"CR 508.1i wants a mana-ability window between total-cost determination and payment."* →
   **do not add a two-phase declaration.** That is a new command; the deviation is pre-existing
   and is recorded as seed OOS-DP4-2.
3. *"The forced decline needs to distinguish 'the player declined' from 'the engine declined on
   their behalf' in the event stream."* → **do not add an event or a flag.** Reuse
   `EchoPaid`-family events as-is and file seed OOS-DP4-5. (This is the DP-suite-wide
   "engine chose this for you" annotation, audit §9 rec 8 — an M11-local Session 7 item.)

---

## 1. The defects, verified by reading (pre-survey corrections in bold)

### 1.1 DP-10 — the attack tax is inspected and never debited

`crates/engine/src/rules/combat.rs:185-265` is one bare `{ … }` block inside
`handle_declare_attackers` (`:32-685`). Verified line by line:

| lines | what it does |
|---|---|
| `:185-198` | comment. `:194-198` states the deferral: *"Alpha implementation: reject the declaration if the attacking player's mana pool … cannot cover the total cumulative attack tax. … Interactive payment is deferred to post-alpha (requires a new `DeclareAttackers` command field)."* |
| `:202-203` | `tax_per_attacker: std::collections::HashMap<PlayerId, u32>` — **a `u32`, not a `ManaCost`** |
| `:204-229` | scans `state.restrictions` for live-source `CantAttackYouUnlessPay { cost_per_creature }`, keyed on `restriction.controller`; **flattens colour** at `:221-227` by summing `generic + white + blue + black + red + green + colorless`. `hybrid` / `phyrexian` / `x_count` are **silently dropped** |
| `:230-240` | counts attackers per taxed defender. Only `AttackTarget::Player(pid)` is counted (`:235`) — planeswalker attacks are untaxed, which is *correct* per the Propaganda ruling and must be preserved |
| `:241-247` | `total_tax: u32 = Σ cost_per × attacker_count` |
| `:248-263` | reads `ps.mana_pool.total_with_restricted()` and returns `Err` if `available_mana < total_tax`. Error message contains the substring **`"attack tax"`**, asserted by `tests/rules/restrictions.rs:690` |
| — | **and that is the entire block. `mana_pool` is never mutated anywhere in the 600-line handler.** Confirmed: the only `mana_pool` occurrence in `combat.rs` is `:252`. |

Three distinct bugs, all reachable from a legal deck (`propaganda.rs` and `ghostly_prison.rs`
carry no `completeness` field ⇒ `Completeness::default() == Complete`):

1. **Never debited.** Float `{2}`, attack, keep the `{2}`. Also means the same floating `{2}`
   pays for every extra combat phase in the turn.
2. **Colour lost.** A hypothetical `{W}{W}` tax is payable with `{C}{C}`.
3. **Restricted mana counts toward affordability.** `total_with_restricted()`
   (`crates/card-types/src/state/player.rs:61-64`) adds every `RestrictedMana` entry.
   `ManaPool::spend(cost, None)` would spend **none** of it (`:186-245`: `spell_restricted` is
   only consulted when `spell.is_some()`), so today's check is affordability-generous and the
   payment that never happens would have been impossible anyway. **This is the inconsistency
   the brief flags, and it is resolved in the strict direction — see §2.3.**

**Pre-survey corrections:**

- **`goblin_rabblemaster.rs` does NOT carry `GameRestriction::CantAttackYouUnlessPay`.** Verified
  by reading the whole file: it has exactly three abilities (a `Static`
  `AddKeyword(MustAttackEachCombat)` grant, an `AtBeginningOfCombat` token trigger, and a
  `WhenAttacks` pump). The restriction appears only inside the long comment at `:35-55`. The
  corpus roster of the restriction is exactly **two** defs: `propaganda.rs:20`,
  `ghostly_prison.rs:20`. Both are `{2}` generic.
- The Rabblemaster deadlock is **already a filed seed — OOS-RS3-4** — not an unfiled note.
  `goblin_rabblemaster.rs:53-55` says so. It is closed by this PB (§3 Change 1c).
- The brief's *"restricted mana (CR 106.12)"* cite is **wrong**. Live CR 106.12 is
  *"To 'tap [a permanent] for mana' is to activate a mana ability … that includes the {T}
  symbol"*. The restricted-mana rule is **CR 106.6**. `crates/card-types/src/state/player.rs`
  repeats the stale cite at `:20`, `:46`, `:147`, `:181`, `:209`, `:295`(implicitly) — same
  class as PB-DP2's `103.4b`→`103.5` correction. Fix the two comments PB-DP4 actually edits;
  the rest is seed OOS-DP4-6.
- The audit's cite of **CR 508.1g** for the Propaganda tax is imprecise. CR 508.1g covers
  *optional* costs ("costs a player may pay 'as' a creature attacks", i.e. exert). Propaganda is
  a **restriction** (CR 508.1c: "effects that say a creature … can't attack unless some
  condition is met"); the payment machinery is **CR 508.1h** (total cost, locked in),
  **CR 508.1i** (mana-ability window) and **CR 508.1j** ("Once the player has enough mana in
  their mana pool, they pay all costs in any order. **Partial payments are not allowed.**").
  Use the precise cites in code and tests; correct the audit row on close-out (§10).

### 1.2 DP-11 — nothing ever calls the "otherwise" branch

**Producers** — `crates/engine/src/rules/resolution.rs`, inside `resolve_top_of_stack`:

| keyword | lines | what it does |
|---|---|---|
| Echo | `:2799-2842` | CR 400.7 still-on-battlefield check (`:2820-2824`) → `EchoPaymentRequired` (`:2827-2831`) → `pending_echo_payments.push_back((controller, permanent, cost))` (`:2833-2835`) → `AbilityResolved` |
| Cumulative upkeep | `:2843-2903` | same shape, plus the CR 702.24a age counter at `:2873-2877` and `age_counter_count` on the event |
| Recover | `:2904-~2960` | CR 400.7 still-in-graveyard check → `RecoverPaymentRequired` → `pending_recover_payments.push_back(...)` |

The comment at **`:2804-2805`** reads *"The game pauses until a `Command::PayEcho` is
received."* **Confirmed false.** Grep of the three field names across `crates/engine/src`
returns exactly: `state/mod.rs:263/271/283` (declarations), `:535-549` (read accessors),
`:774-792` (`_mut` escape hatches), `state/builder.rs:337-339` (init),
`state/hash.rs:7736-7754` (hashing), the three producers above, the three handlers below, and
`testing/replay_harness.rs:912`. **`rules/priority.rs`, `rules/engine.rs::handle_all_passed`,
`rules/turn_structure.rs` and `rules/sba.rs` contain zero occurrences.** The audit's claim
holds exactly as filed.

**Consumers** — `crates/engine/src/rules/engine.rs`:

| handler | lines | decline branch | trailing priority write |
|---|---|---|---|
| `handle_pay_echo` | `:590-767` | `:667-750` — `check_zone_change_replacement` → move to graveyard/exile/command zone → `CreatureDied`/`ObjectExiled` (CR 701.21a: bypasses indestructible) | **`:763-765`** |
| `handle_pay_cumulative_upkeep` | `:776-976` | `:877-959` — same shape | **`:972-974`** |
| `handle_pay_recover` | `:1013-1098` | `:1073-1081` — move to `ZoneId::Exile` → `RecoverDeclined` | **`:1094-1096`** |

**The consequence logic exists and is correct.** Verified. Nothing calls it. So today: an echo
permanent survives every upkeep for free, a cumulative-upkeep permanent accrues age counters
forever without ever being sacrificed, and a recover card sits in the graveyard un-exiled and
re-triggers on the next creature death.

**Compounding**: grep of `crates/simulator` for `PayEcho|PayCumulativeUpkeep|PayRecover|
pending_echo|pending_cumulative|pending_recover` returns **zero matches** — no `LegalAction`, so
a bot or an M11-local seat never sends the command at all. And `crates/engine/src/testing/
replay_harness.rs` implements only a **`pay_recover`** action (`:906-923`); there is no
`pay_echo` or `pay_cumulative_upkeep` script action.

**Pre-survey corrections:**

- **`bala_ged_recovery.rs` does not carry `KeywordAbility::Recover`.** Grepping the corpus for
  the three keywords (both `KeywordAbility::` and `AbilityDefinition::` spellings) returns
  exactly **5 files**: `mogg_war_marshal.rs`, `avalanche_riders.rs` (echo),
  `tombstone_stairwell.rs`, `mystic_remora.rs` (cumulative upkeep), `grim_harvest.rs` (recover).
  Bala Ged Recovery is an MDFC sorcery with no Recover keyword.
- **Three of those five are `Complete`** (no `completeness` field): `mogg_war_marshal.rs`,
  `avalanche_riders.rs:61` (explicit `Completeness::Complete`), `grim_harvest.rs`. They are
  **live-wrong today**. `tombstone_stairwell.rs:53` is `partial` for unrelated reasons
  (token provenance) and cannot upgrade; `mystic_remora.rs:59` is `known_wrong` for
  `MayPayOrElse` (DP-12 scope) — its note's clause *"Cumulative upkeep {1} and the
  noncreature-spell trigger filter are correct"* is **false today and becomes true** after this
  PB, with no edit.
- `state/mod.rs:260-261`'s *"Only one echo payment can be pending at a time"* is **false** — two
  echo permanents at the same upkeep queue two triggers, each of which pushes an entry
  (`tests/mechanics_a_d/cumulative_upkeep.rs:631-691` pins exactly this for CU, CR 702.24b).
  Comment correction prescribed in §3 Change 2f.

---

## 2. CR rule text (verbatim, from the mtg-rules MCP) and what it settles

### 2.1 CR 508.1 — declaring attackers (the operative sub-rules)

> **508.1.** First, the active player declares attackers. This turn-based action doesn't use the
> stack. To declare attackers, the active player follows the steps below, in order. **If at any
> point during the declaration of attackers, the active player is unable to comply with any of
> the steps listed below, the declaration is illegal; the game returns to the moment before the
> declaration** (see rule 732, "Handling Illegal Actions").
>
> **508.1c** The active player checks each creature they control to see whether it's affected by
> any restrictions (effects that say a creature can't attack, **or that it can't attack unless
> some condition is met**). If any restrictions are being disobeyed, the declaration of attackers
> is illegal.
>
> **508.1d** … **If a creature can't attack unless a player pays a cost, that player is not
> required to pay that cost, even if attacking with that creature would increase the number of
> requirements being obeyed.** …
>
> **508.1f** The active player taps the chosen creatures. Tapping a creature when it's declared
> as an attacker isn't a cost; attacking simply causes creatures to become tapped.
>
> **508.1g** If there are any **optional** costs to attack with the chosen creatures (expressed
> as costs a player may pay "as" a creature attacks), the active player chooses which, if any,
> they will pay.
>
> **508.1h** If any of the chosen creatures **require paying costs to attack**, or if any
> optional costs to attack were chosen, the active player determines the total cost to attack.
> Costs may include paying mana, tapping permanents, sacrificing permanents, discarding cards,
> and so on. Once the total cost is determined, it becomes "locked in." If effects would change
> the total cost after this time, ignore this change.
>
> **508.1i** If any of the costs require mana, the active player then has a chance to activate
> mana abilities (see rule 605, "Mana Abilities").
>
> **508.1j** Once the player has enough mana in their mana pool, they pay all costs in any order.
> **Partial payments are not allowed.**

### 2.2 What CR 508.1 settles for DP-10

1. **The tax is a real cost that is really paid**, out of the mana pool, in step 508.1j. There is
   no reading under which "checked but not collected" is correct.
2. **Colour matters.** 508.1h says the cost is *determined*, not *summed to a mana value*. A
   coloured attack tax must be paid with matching colours (CR 202/106.1). Flattening to a `u32`
   is a rules violation; it is *latent* today (both real cards are `{2}` generic) so the test
   must construct the coloured case synthetically.
3. **Payment order is: restrictions (508.1c) → requirements (508.1d) → tap (508.1f) → total cost
   (508.1h) → pay (508.1j).** So the debit belongs **after** the tapping loop in the handler's
   mutation section, not in the validation block. Affordability is still checked in the
   validation block, before any mutation, which is what makes the CR 508.1 / CR 732
   "the declaration is illegal, rewind" property hold with an immutable-state engine.
4. **CR 508.1d is a hard carve-out.** A must-attack requirement can *never* be satisfied by
   forcing a payment. This is the CR basis for closing OOS-RS3-4 (§3 Change 1c), and the
   2014-07-18 Goblin Rabblemaster ruling says the same thing in as many words.
5. **CR 508.1i cannot be honoured without a new command.** The engine determines the total cost
   and pays it inside one `DeclareAttackers`, with no intervening mana-ability window. The tax
   must therefore already be floating. That is today's behaviour and it is **preserved**; it is
   recorded as seed OOS-DP4-2, not fixed.

### 2.3 CR 106.6 — restricted mana, and the answer to the brief's question 4

> **106.6.** Some spells or abilities that produce mana **restrict how that mana can be spent**,
> have an additional effect that affects the spell or ability that mana is spent on, or create a
> delayed triggered ability … that triggers when that mana is spent. This doesn't affect the
> mana's type.

**Decision: restricted mana does NOT pay an attack tax.** Argued from the engine's own model,
not just from CR text:

- Every `ManaRestriction` variant (`crates/card-types/src/cards/card_definition.rs`, matched in
  `player.rs::restriction_matches:293-313`) is *spell*-scoped: `CreatureSpellsOnly`,
  `SubtypeOnly`, `SubtypeOrSubtype`, `CreatureWithSubtype`, `ChosenTypeCreaturesOnly`,
  `ChosenTypeSpellsOnly`. Each one reads `spell.is_creature` or `spell.subtypes`.
- An attack tax is **not a spell** and has no `SpellContext`. There is no restriction in the
  corpus that an attack tax could satisfy, so no restricted mana in this engine is spendable on
  one.
- Therefore the correct API is `casting::can_pay_cost(pool, &cost)` / `casting::pay_cost(pool,
  &cost)` — the `spell: None` pair (`casting.rs:6596-6601`, `:6976-6981`), which resolves to
  `ManaPool::can_spend(cost, None)` / `spend(cost, None)`.
- **This resolves the `total_with_restricted()` inconsistency in the strict direction**:
  affordability stops counting restricted mana, which is exactly what `spend(cost, None)` can
  actually take. Check and payment become the *same predicate*, which is the whole point.
- **Behaviour flip**: a player whose only mana is restricted can no longer attack past a
  Propaganda. That is correct, and it is a fail-before/pass-after probe (§7 probe 3).

### 2.4 CR 107.4e / 107.4f — hybrid and Phyrexian

`ManaPool::can_spend` and `spend` both open with `debug_assert_flattened(cost)`
(`player.rs:153`, `:191`, guard at `:281-291`). Its doc block is explicit that reaching it
unflattened is an **engine bug**, not an LKI fizzle, and that in a *release* build the assert
compiles out and the pips are paid **for free** — the exact silence that made every filter land
free for the life of the project (OOS-RS-2 / PB-RS2). `Command::DeclareAttackers` has no
`hybrid_choices` / `phyrexian_life_payments` field and adding one is a PROTOCOL bump, so:

**Decision: a hybrid / Phyrexian / X attack tax is REJECTED, not paid.** Explicit pre-check
before summing (§3 Change 1a step 1), not a reliance on the `debug_assert`. Latent — both real
cards are pure generic — and seeded as OOS-DP4-1.

### 2.5 CR 702.30a / 702.24a / 702.59a — the three pay-or-lose-it abilities

> **702.30a** Echo is a triggered ability. "Echo [cost]" means "**At the beginning of your
> upkeep**, if this permanent came under your control since the beginning of your last upkeep,
> **sacrifice it unless you pay [cost]**."
>
> **702.24a** Cumulative upkeep is a triggered ability that imposes an increasing cost on a
> permanent. "Cumulative upkeep [cost]" means "**At the beginning of your upkeep**, if this
> permanent is on the battlefield, put an age counter on this permanent. Then **you may pay
> [cost] for each age counter on it. If you don't, sacrifice it**." If [cost] has choices
> associated with it, each choice is made separately for each age counter, then either the entire
> set of costs is paid, or none of them is paid. **Partial payments aren't allowed.**
>
> **702.24b** If a permanent has multiple instances of cumulative upkeep, each triggers
> separately. However, the age counters are not connected to any particular ability; each
> cumulative upkeep ability will count the total number of age counters on the permanent at the
> time that ability resolves.
>
> **702.59a** Recover is a triggered ability that functions only while the card with recover is
> in a player's graveyard. "Recover [cost]" means "**When a creature is put into your graveyard
> from the battlefield, you may pay [cost]. If you do, return this card from your graveyard to
> your hand. Otherwise, exile this card.**"

### 2.6 CR 118.12 / 118.12a — "unless" is "may; if you don't"

> **118.12.** Some spells, activated abilities, and triggered abilities read, "[Do something]. If
> [a player] [does, doesn't, or can't], [effect]." Or "[A player] may [do something]. If [that
> player] [does, doesn't, or can't], [effect]." **The action [do something] is a cost, paid when
> the spell or ability resolves.** The "If [a player] [does, doesn't, or can't]" clause checks
> **whether the player chose to pay** an optional cost or started to pay a mandatory cost,
> regardless of what events actually occurred.
>
> **118.12a** … "[Do something] unless [a player does something else]." This means the same thing
> as "[A player may do something]. If [that player doesn't], [do something]."

### 2.7 CR 608.2d / 608.2g — when the choice is made, and the mana-ability window

> **608.2d** If an effect of a spell or ability offers any choices other than choices already made
> as part of casting the spell, activating the ability, or otherwise putting the spell or ability
> on the stack, **the player announces these while applying the effect.** …
>
> **608.2g** **If an effect gives a player the option to pay mana, they may activate mana
> abilities before taking that action.** … No other spells can normally be cast and no other
> abilities can normally be activated during resolution.

### 2.8 CR 119.4 — life payments

> **119.4.** If a cost or effect allows a player to pay an amount of life greater than 0, the
> player may do so **only if their life total is greater than or equal to the amount** of the
> payment. …
> **119.4b** Players can always pay 0 life, no matter what their … life total is …

### 2.9 What the CR settles for DP-11 (answers the brief's question 1)

1. **The decision belongs at resolution (CR 608.2d), not at a later priority window.** All three
   abilities are triggered abilities whose payment is a resolution-time cost (CR 118.12).
2. **Not answering is "doesn't" (CR 118.12a).** So the CR-correct default for an unanswered
   payment is the **decline** branch — sacrifice / exile — not an auto-pay. Auto-paying when
   affordable would be the DP-19 (`MayPayThenEffect`) bug class: force-feeding the player a cost
   they never elected. **Declining is CR-faithful; auto-paying is not.**
3. **CR 608.2g is the only reason a pause is needed at all.** The player may want to activate
   mana abilities before paying. That is what makes design (3) below — decide instantly at
   resolution — CR-wrong as well as agency-destroying: the player never gets the 608.2g window.
4. **CR 101.4** supplies APNAP as the ordering discipline when multiple players' choices are
   outstanding. Cited for determinism (SR-9b) as much as for the rule.

---

## 3. Engine changes

### Change 1 — `rules/combat.rs`: charge the attack tax, and honour CR 508.1d

**File**: `crates/engine/src/rules/combat.rs`
**Imports to add**: `ManaCost` to the existing
`use crate::state::game_object::{Designations, ObjectId};` (`:14`); `use super::casting;`
alongside `use super::abilities;` (`:9`); `std::collections::{BTreeMap, BTreeSet}`.

#### Change 1a — turn the block at `:199-265` into a real cost computation

Replace the bare `{ … }` block with a bound expression so its results survive into the
requirement blocks and the mutation section:

```rust
// CR 508.1c / 508.1h / 508.1j: attack-cost restrictions (Propaganda, Ghostly Prison).
//
// "Creatures can't attack you unless their controller pays {N} for each creature they
// control that's attacking you" is a RESTRICTION (CR 508.1c), not an optional cost
// (CR 508.1g — that rule covers exert-style "as it attacks" costs). The payment
// machinery is CR 508.1h (determine the total, lock it in), CR 508.1i (mana-ability
// window) and CR 508.1j ("they pay all costs in any order. Partial payments are not
// allowed"). Costs from multiple sources are cumulative (Propaganda ruling).
//
// Affordability is checked HERE, before any state is mutated, so an unaffordable
// declaration is rejected with the game untouched (CR 508.1 / CR 732: "the declaration
// is illegal; the game returns to the moment before the declaration"). The DEBIT happens
// after the tapping loop, matching CR 508.1f -> 508.1j order. See Change 1b.
//
// PB-DP4 / DP-10: before this, the pool was READ ONCE (total_with_restricted) and never
// debited, and the cost was flattened to a u32 generic total so colour was lost.
let (attack_tax, taxed_defenders): (Option<ManaCost>, BTreeSet<PlayerId>) = { … };
```

Body, step by step (the runner implements exactly this logic; comment wording is theirs, every
CR citation shown must appear):

1. **Per-defender per-creature cost, as a `ManaCost`.** `let mut tax_per_creature:
   BTreeMap<PlayerId, ManaCost> = BTreeMap::new();` — **`BTreeMap`, not `HashMap`**: iteration
   order feeds the summed cost and the error message, and SR-9b requires determinism. Walk
   `state.restrictions` with the existing live-source guard (`:206-213`, keep verbatim). For each
   `GameRestriction::CantAttackYouUnlessPay { cost_per_creature }`:
   - **Reject unflattenable pips before touching them** (CR 107.4e / 107.4f, §2.4):
     ```rust
     if !cost_per_creature.hybrid.is_empty()
         || !cost_per_creature.phyrexian.is_empty()
         || cost_per_creature.x_count > 0
     {
         return Err(GameStateError::InvalidCommand(format!(
             "attack tax: a hybrid, Phyrexian or X attack cost is not payable — \
              Command::DeclareAttackers carries no payment-choice field, so the engine \
              cannot ask which half to pay (CR 107.4e/107.4f via CR 508.1h). \
              Restriction source {:?}; see OOS-DP4-1.",
             restriction.source
         )));
     }
     ```
     The message **must** contain the substring `"attack tax"` (see §4.2).
   - Accumulate field-wise into `tax_per_creature.entry(restriction.controller)` via the new
     `add_mana_cost` helper (Change 1d) with `times = 1`.
2. **Attackers per taxed defender.** Keep `:230-240` unchanged in behaviour, switching the map to
   `BTreeMap<PlayerId, u32>`. **Preserve the `AttackTarget::Player(pid)`-only match** and add a
   comment: *"Only player-attacks are taxed. A creature attacking a planeswalker is attacking
   that planeswalker, not its controller, so Propaganda does not apply (CR 508.1c + the
   Propaganda ruling). Keep this narrow."*
3. **`taxed_defenders`** = `tax_per_creature.keys().copied().collect::<BTreeSet<_>>()`. Built
   from the *restriction* map, not the attacker map, because Change 1c needs to know which
   players are taxed whether or not anyone is currently declared against them.
4. **Total.** `let mut total = ManaCost::default();` then for each `(defender, count)` in the
   attacker map, `add_mana_cost(&mut total, &tax_per_creature[defender], *count)`.
5. **Affordability (CR 508.1h/508.1j).** If `total != ManaCost::default()`:
   ```rust
   let affordable = state
       .expect_player(player)
       .map(|ps| casting::can_pay_cost(&ps.mana_pool, &total))
       .unwrap_or(false);
   if !affordable {
       return Err(GameStateError::InvalidCommand(format!(
           "attack tax: the attacking player must pay {:?} in total for the declared \
            attackers but cannot pay it from their mana pool (CR 508.1h/508.1j, \
            Propaganda/Ghostly Prison). Restricted mana (CR 106.6) cannot pay an attack \
            tax — no ManaRestriction in this engine matches a non-spell cost.",
           total
       )));
   }
   ```
   Substring `"attack tax"` preserved. `can_pay_cost` = `can_spend(cost, None)`, so restricted
   mana is excluded — §2.3.
6. Yield `(if total == ManaCost::default() { None } else { Some(total) }, taxed_defenders)`.

**Delete** the now-false comment at `:194-198` and replace it with the block above. Per
`memory/conventions.md` ("aspirationally-wrong code comments are correctness hazards") the
"requires a new `DeclareAttackers` command field" claim must not survive — it is falsified by
this change.

#### Change 1b — actually debit, in the mutation section

**Site**: immediately after the enlist tapping loop (`:602-612`) and **before** "Record attackers
in combat state" (`:613-618`).

```rust
// CR 508.1j: pay all attack costs. Partial payments are not allowed, and the total was
// locked in during validation (CR 508.1h) before any state was mutated, so this cannot
// fail. Placed after the CR 508.1f tapping loop to match the CR's own step order
// (508.1f tap -> 508.1h total -> 508.1i mana abilities -> 508.1j pay).
//
// CR 508.1i is NOT honoured: the engine determines and pays the total inside a single
// DeclareAttackers command, so the player has no window to activate mana abilities
// between the two. The tax must already be floating. Pre-existing deviation, preserved
// (fixing it needs a two-phase declaration = a new Command). Seed OOS-DP4-2.
if let Some(tax) = &attack_tax {
    if let Some(ps) = state.expect_player_mut(player) {
        casting::pay_cost(&mut ps.mana_pool, tax);
    }
    // Architecture Invariant 4: a pool debit is a state change and must be evented.
    // Reuses the existing universal payment event — no wire change.
    events.push(GameEvent::ManaCostPaid {
        player,
        cost: tax.clone(),
    });
}
```

`events` is declared at `:588`, so the push is in scope. `casting::pay_cost` takes
`&mut ManaPool` — no `SpellContext`, per §2.3.

#### Change 1c — CR 508.1d: a must-attack requirement can never force a payment (closes OOS-RS3-4)

**Decision: FIX IN SCOPE.** Reasoning, stated so the reviewer can overrule it:

- CR 508.1d is verbatim on point: *"If a creature can't attack unless a player pays a cost, that
  player is not required to pay that cost, even if attacking with that creature would increase
  the number of requirements being obeyed."* The engine's must-attack "able" test never reads
  `CantAttackYouUnlessPay`, so the engine violates that sentence directly. It is a class-D
  finding, not a nicety.
- The failure mode is a **hang**: with every viable opponent taxed and the player unable to pay,
  declaring the forced creature is illegal (tax check) *and* omitting it is illegal (must-attack
  check). A hang is precisely the failure class this PB's own hard constraint (b) forbids for
  DP-11. It would be incoherent to design DP-11 against deadlock while shipping a DP-10 change
  that makes an existing deadlock cheaper to reach (real debit ⇒ the same floating `{2}` no
  longer funds a second combat phase).
- Cost is bounded and the code is already open: Change 1a hands us `taxed_defenders` for free,
  and the fix **deduplicates** two copy-pasted `no_legal_target` computations
  (`:304-321` for goad, `:403-420` for `MustAttackEachCombat`) into one helper.
- `goblin_rabblemaster.rs` is `Complete` and manufactures a forced attacker every combat, so the
  gap is reachable every turn, not only when a fixed forced attacker happens to be out.

**New private helper** in `combat.rs` (place it just below `handle_declare_attackers`, before
`remove_from_combat` at `:702`):

```rust
/// CR 508.1d: can a must-attack requirement actually force `creature` to attack?
///
/// "If a creature can't attack unless a player pays a cost, that player is not required to
/// pay that cost, even if attacking with that creature would increase the number of
/// requirements being obeyed." (CR 508.1d; Goblin Rabblemaster ruling 2014-07-18: "If
/// there's a cost associated with having a creature attack, you're not forced to pay that
/// cost.") So a requirement is only *obeyable* if some attack target exists that costs the
/// controller nothing.
///
/// Returns true iff at least one such target exists:
///   * a live opponent that is neither this creature's owner-under-CantAttackOwner
///     (CR 508.1c) nor a member of `taxed_defenders`, OR
///   * any opponent-controlled planeswalker on the battlefield — attacking a planeswalker
///     is not "attacking you", so a CantAttackYouUnlessPay tax never applies to it
///     (CR 508.1c + the Propaganda ruling), and CantAttackOwner is about players.
///
/// Generalises the two hand-copied `no_legal_target` computations PB-DP4 replaced
/// (`handle_declare_attackers`, the goad block and the MustAttackEachCombat block).
/// PB-DP4 / DP-10, closes OOS-RS3-4.
fn has_uncosted_attack_target(
    state: &GameState,
    player: PlayerId,
    creature: ObjectId,
    taxed_defenders: &BTreeSet<PlayerId>,
) -> bool
```

Body:
1. `has_cant_attack_owner` — lift the existing predicate verbatim from `:304-312` (source ==
   `creature`, restriction is `CantAttackOwner`, source still on the battlefield).
2. `let owner = state.expect_object(creature).map(|o| o.owner);`
3. Any live opponent target: `state.players.keys().any(|pid| *pid != player && (!has_cant_attack_owner || Some(*pid) != owner) && !taxed_defenders.contains(pid) && state.expect_player(*pid).map(|p| !p.has_lost && !p.has_conceded).unwrap_or(false))`.
4. Else any opponent planeswalker: scan `state.objects` for
   `zone == ZoneId::Battlefield && controller != player` whose
   `layers::expect_characteristics(state, id).card_types` contains
   `CardType::Planeswalker`. (Layer-resolved — W3-LC contract; do **not** read
   `obj.characteristics`.)

**Call sites** — in both blocks, replace the local `has_cant_attack_owner` + `no_legal_target`
computation with one line, keeping the surrounding `cannot_attack` expression intact:

| block | current lines | replacement |
|---|---|---|
| goad, CR 701.15b (`:272-335`) | `:304-321` | `let no_legal_target = !has_uncosted_attack_target(state, player, goaded_id, &taxed_defenders);` |
| `MustAttackEachCombat`, CR 508.1d (`:377-432`) | `:403-420` | `let no_legal_target = !has_uncosted_attack_target(state, player, *obj_id, &taxed_defenders);` |

Update the comment blocks at `:298-303` and `:398-402` to add the CR 508.1d cost carve-out and
name OOS-RS3-4 as closed. **Do not touch** the goad *directional* check at `:336-374` — "must
attack a player other than the goading player if able" is a separate requirement and adding a
tax carve-out there is out of scope (seed OOS-DP4-3).

#### Change 1d — `add_mana_cost` helper

```rust
/// CR 508.1h: accumulate `addend` x `times` into `total`, field by field.
///
/// Attack taxes from multiple sources are cumulative (Propaganda ruling), and a defender's
/// total is `cost_per_creature` x the number of creatures attacking that defender. Hybrid,
/// Phyrexian and X components are rejected by the caller before this is reached
/// (CR 107.4e/107.4f) and are debug-asserted here so the guard cannot drift.
fn add_mana_cost(total: &mut ManaCost, addend: &ManaCost, times: u32) {
    debug_assert!(
        addend.hybrid.is_empty() && addend.phyrexian.is_empty() && addend.x_count == 0,
        "unflattened / X attack tax reached add_mana_cost: {addend:?}"
    );
    total.white += addend.white * times;
    total.blue += addend.blue * times;
    total.black += addend.black * times;
    total.red += addend.red * times;
    total.green += addend.green * times;
    total.colorless += addend.colorless * times;
    total.generic += addend.generic * times;
}
```

Note: `engine.rs:978-1002` already has a `multiply_mana_cost` for cumulative upkeep. **Do not
reuse or move it** — it is private to `engine.rs`, it also multiplies `hybrid`/`phyrexian`/
`x_count` (correct for CU, wrong for a path that rejects them), and hoisting it would widen a
module boundary for no gain. Deduplicating the two is seed OOS-DP4-7.

---

### Change 2 — DP-11: a deadline for the three pay-or-lose-it payments

#### 2.0 The design decision (answers the brief's questions 1 and 2)

**Chosen: the wip file's option (2) — auto-resolve at the advancement boundary, declining.**
Precisely: **an unanswered pay-or-lose-it payment is closed out with its `pay: false` branch at
the moment the game would leave the priority round in which the ability resolved** — i.e. at the
top of `handle_all_passed`'s stack-empty branch, before mana pools empty and before the step
advances.

**Why not (1) "gate advancement".** It is the most CR-faithful *shape* but it violates hard
constraint (b): a fuzzer seat, `GameDriver`, a golden script or an M11-local human that never
sends `Pay*` hangs the game forever. Worse, the hang is silent in this codebase — `driver.rs`
answers a rejected command with a `PassPriority`, so a refused `PassPriority` produces an
infinite retry loop with no error. A hang is strictly worse than today's free survival, and any
"forced-resolution backstop" bolted onto (1) *is* design (2) with an extra failure mode.

**Why not (3) "decide inside trigger resolution".** It has zero deadlock risk but (a) it deletes
the player's agency on a genuine CR 118.12 choice, which is the exact class of bug the whole
PB-DP suite exists to remove — shipping it inside the DP suite would be self-defeating; (b) it
makes `Command::PayEcho` / `PayCumulativeUpkeep` / `PayRecover` **unreachable**, directly
contradicting acceptance criterion 3 (5529) which requires the provider to expose the choice;
and (c) it is *also* CR-wrong, because CR 608.2g guarantees the player a mana-ability window
before paying, and an instant decision at resolution never offers one.

**Why (2) is right, and where the boundary sits.** `handle_all_passed` (`engine.rs:1693-1742`)
has two **disjoint** branches:

- `:1695-1710` stack non-empty ⇒ resolve the top object and return. **This is where the pending
  entry is created.**
- `:1711-1740` stack empty ⇒ empty mana pools (CR 500.4) and advance the step / turn.

`resolve_top_of_stack` ends by clearing `players_passed` and granting priority to the active
player (`resolution.rs:7768-7772`), so **every player is guaranteed at least one priority window
between the entry's creation and the deadline** — including a non-active recover controller. In
that window the owing player can activate mana abilities (CR 608.2g, honoured), cast, respond,
and send the `Pay*` command. Because the two branches are disjoint, **the sweep can never fire in
the same `handle_all_passed` call that created the entry**, which is what keeps every existing
test and golden script green (§4.2 / §4.3).

**The CR deviation this design accepts, stated explicitly:**

> CR 608.2d requires the pay-or-decline choice to be announced *while the triggered ability's
> effect is applied*. This engine defers it to the end of the priority round that follows the
> ability's resolution. Within that window the permanent still exists (and a recover card still
> sits in its graveyard) although the CR would already have sacrificed / exiled it. Observable
> consequences inside the window: the permanent can be tapped for mana, sacrificed to another
> cost, targeted, counted by a static ability, or blocked with; the recover card can be milled,
> exiled by something else, or re-trigger. The **outcome** is not deviated from: at the boundary
> the CR-mandated "otherwise" branch runs, and an unanswered payment is treated as "the player
> didn't pay" exactly as CR 118.12a directs. A second deviation is the mirror of the first: a
> player may hold priority with the payment outstanding, which the CR never permits.

**Auto-decline, not auto-pay.** CR 118.12a makes "didn't answer" ≡ "didn't pay". Auto-paying when
affordable would silently consume the player's floating mana or life for a cost they never
elected — DP-19's bug class. Declining is the CR-correct default and is also the one that cannot
destroy resources the player was saving.

**Termination (no infinite extra-round loop).** Each sweep consumes *every* outstanding entry.
The extra-round branch is only taken when the sweep **produced events**; each such event is a
zone change (sacrifice / exile) or a life loss. A new entry can only be created by a *new*
trigger resolving, which requires a *new* permanent to leave the battlefield or a *new* creature
to hit a graveyard — a strictly decreasing resource. In the Cleanup step the existing
`cleanup_sba_rounds` cap (100, `engine.rs:1773`) and `loop_detection::check_for_mandatory_loop`
(`:1784`) provide a second backstop. State this argument as a code comment.

#### Change 2a — the sweep

**File**: `crates/engine/src/rules/engine.rs`. New private fn, placed immediately after
`handle_pay_recover` (i.e. after `:1098`), so it sits next to the three handlers it calls.

```rust
/// CR 702.30a / 702.24a / 702.59a + CR 118.12a: close out any pay-or-lose-it payment that
/// was not answered before the game left this priority round.
///
/// PB-DP4 / DP-11. Before this, the three `pending_*` vectors were inert queues: nothing in
/// `rules/priority.rs`, `handle_all_passed`, `rules/turn_structure.rs` or `rules/sba.rs`
/// consulted them, so passing priority left an echo permanent neither paid for nor
/// sacrificed, a cumulative-upkeep permanent accruing age counters forever, and a recover
/// card sitting un-exiled in its graveyard. `rules/resolution.rs`'s claim that "the game
/// pauses until a Command::PayEcho is received" described a pause that did not exist.
///
/// **Why decline and not auto-pay.** CR 118.12a: "[Do something] unless [a player does
/// something else]" means "[a player may do something else]. If [that player doesn't], [do
/// something]." Not answering is "doesn't". Auto-paying an affordable cost would spend mana
/// or life the player never elected to spend — the DP-19 (`MayPayThenEffect`) bug class.
///
/// **Deviation from CR 608.2d, deliberate.** The CR makes this choice during the ability's
/// resolution. This engine defers it to the end of the following priority round, which is
/// the earliest boundary reachable without a new `Command` (SR-8) and without a design that
/// can hang a fuzzer, the `GameDriver`, a golden script or an M11-local seat that never
/// sends the command. The permanent therefore survives, observably, for the rest of that
/// round. The outcome at the boundary is CR-correct. See `memory/primitives/pb-plan-DP4.md`
/// §3 2.0 for the rejected alternatives.
///
/// **Ordering.** Players are visited in APNAP order (CR 101.4, `abilities::apnap_order`);
/// within a player, echo then cumulative upkeep then recover, each in insertion order
/// (which is the order the triggers resolved). Deterministic, as SR-9b requires.
///
/// **Termination.** Every call drains every entry. A new entry needs a new trigger to
/// resolve, which needs a permanent to leave the battlefield or a creature to reach a
/// graveyard, so the extra-round chain is bounded by the object count.
///
/// SR-4 classification: every `Err` from the three handlers is an **engine bug**, not an LKI
/// fizzle — the entry was read out of the vector one statement earlier, and each handler
/// removes it before any fallible step, so a failure cannot loop and cannot be a legal
/// CR 400.7 fizzle (the handlers already return `Ok(vec![])` for that case). Mechanism is a
/// `debug_assert!`, mirroring `state::diagnostics`' `expect_*` family.
fn force_resolve_overdue_payments(state: &mut GameState) -> Vec<GameEvent>
```

Body:

```rust
let mut events = Vec::new();
if state.pending_echo_payments.is_empty()
    && state.pending_cumulative_upkeep_payments.is_empty()
    && state.pending_recover_payments.is_empty()
{
    return events;
}
for owing in abilities::apnap_order(state) {
    // Snapshot before mutating: each handler removes its own entry from the vector.
    let echoes: Vec<ObjectId> = state
        .pending_echo_payments
        .iter()
        .filter(|(p, _, _)| *p == owing)
        .map(|(_, obj, _)| *obj)
        .collect();
    for permanent in echoes {
        match handle_pay_echo(state, owing, permanent, false) {
            Ok(evs) => events.extend(evs),
            Err(e) => debug_assert!(false, "engine invariant: forced echo decline for \
                {owing:?}/{permanent:?} failed ({e}); the entry was read from \
                pending_echo_payments one statement earlier"),
        }
    }
    // … identical shape for pending_cumulative_upkeep_payments -> handle_pay_cumulative_upkeep
    // … identical shape for pending_recover_payments      -> handle_pay_recover
}
events
```

**`abilities::apnap_order`** is `pub fn apnap_order(state: &GameState) -> Vec<PlayerId>`
(`abilities.rs:8477-8491`). It does **not** filter eliminated players — that is what we want
here, so an entry belonging to a player who has since lost still resolves instead of orphaning.

**Do not name `KeywordAbility::Echo` / `::CumulativeUpkeep` / `::Recover` in executable code in
this file.** See §4.5 — that is a registry-gate failure.

#### Change 2b — hook it into `handle_all_passed`

**File**: `crates/engine/src/rules/engine.rs:1711-1740` (the stack-empty branch).
**Site**: immediately after `state.maybe_clear_lki_objects();` at `:1715` and **before**
`turn_actions::empty_all_mana_pools` at `:1717`.

```rust
// PB-DP4 / DP-11 (CR 702.30a / 702.24a / 702.59a, CR 118.12a): a pay-or-lose-it
// payment must not survive the priority round in which its ability resolved. Every
// player has had priority since the entry was created — `resolve_top_of_stack`
// re-grants it with an empty pass set — so an unanswered payment is a decline.
//
// This runs only in the stack-EMPTY branch, so it can never fire in the same
// `handle_all_passed` call that created the entry (that call takes the stack-non-empty
// branch and returns). That disjointness is what keeps `mechanics_e_l/echo.rs`,
// `mechanics_a_d/cumulative_upkeep.rs`, `mechanics_m_z/recover.rs` and golden script
// stack/153 green: all of them send `Pay*` immediately after the resolving pass round.
let mut payment_events = force_resolve_overdue_payments(state);
if !payment_events.is_empty() {
    // The sacrifice/exile can produce dies-triggers. They belong on the stack in THIS
    // step, so re-grant priority here instead of advancing — the same
    // "run another round, don't advance" shape `enter_step` uses for CR 514.3a.
    check_and_flush_triggers(state, &mut payment_events);
    events.extend(payment_events);
    if is_game_over(state) {
        events.extend(check_game_over(state));
        return Ok(events);
    }
    // CR 117.3b: grant priority to the active player (if still alive) for the new round.
    // Same idiom as `enter_step` (engine.rs:1836-1858).
    let active = state.turn.active_player;
    let is_alive = state
        .players
        .get(&active)
        .map(|p| !p.has_lost && !p.has_conceded)
        .unwrap_or(false);
    if is_alive {
        let (passed, priority_events) = priority::grant_initial_priority(state);
        state.turn.players_passed = passed;
        state.turn.priority_holder = Some(active);
        events.extend(priority_events);
    } else if let Some(next) = priority::next_priority_player(state, active) {
        state.turn.players_passed = imbl::OrdSet::new();
        state.turn.priority_holder = Some(next);
        events.push(GameEvent::PriorityGiven { player: next });
    } else {
        state.turn.priority_holder = None;
    }
    return Ok(events);
}
```

Two behaviours the runner must get right:

- **The guard is `!payment_events.is_empty()`, not "did the sweep consume anything".** A payment
  whose permanent already left the battlefield (CR 400.7) is consumed and produces **no** events
  (`handle_pay_echo:637-641` returns `Ok(vec![])`). In that case fall straight through to the
  normal advance — the vectors are already drained, so a second sweep would find nothing and an
  extra round would be pure churn.
- **Placement before `empty_all_mana_pools` matters.** The decline branch never spends mana, but
  a future auto-pay variant would, and CR 500.4's pool-emptying is the boundary itself. Keeping
  the sweep on the pre-emptying side of that line keeps the ordering honest.

`priority` and `check_and_flush_triggers` are already in scope in this file
(`priority::grant_initial_priority` at `:1798`, `check_and_flush_triggers` at `:32`).

#### Change 2c — make the recover decline branch infallible (SR-4)

**File**: `crates/engine/src/rules/engine.rs:1075`.
**Action**: `state.move_object_to_zone(recover_card, ZoneId::Exile)?` →
`state.expect_move_object_to_zone(recover_card, ZoneId::Exile)`, handling the `Option`:

```rust
// CR 702.59a: the player declined -- exile the card from the graveyard.
// SR-4 (engine-bug side): the card was proven to be in a graveyard 30 lines above, and
// zones are never removed, so every error variant here is corrupted state, not a CR 400.7
// fizzle. `expect_move_object_to_zone` debug-asserts and returns None in release.
// Making this branch infallible is load-bearing for PB-DP4's forced sweep: the pending
// entry is already removed at this point, so a propagated Err would abandon a mutated
// state and (via handle_all_passed -> handle_pass_priority -> process_command) make every
// subsequent PassPriority fail forever — a deadlock.
if let Some((new_exile_id, _old)) =
    state.expect_move_object_to_zone(recover_card, ZoneId::Exile)
{
    events.push(GameEvent::RecoverDeclined { player, recover_card, new_exile_id });
}
```

**Leave the `pay: true` branch's `?` at `:1067` alone.** The sweep never reaches it, and there
returning `Err` to a caller that discards the state is correct.

#### Change 2d — delete the three priority bodges (closes OOS-DP1-1; answers brief question 3)

**Decision: the mechanism removes the need for them, so they are DELETED.**

**Files / sites**: `engine.rs:763-765` (echo), `:972-974` (cumulative upkeep), `:1094-1096`
(recover) — the `state.turn.players_passed = imbl::OrdSet::new(); … priority_holder =
Some(active);` pairs — plus the comment blocks that justify them (`:754-762`, `:963-971`,
`:1085-1093`). Replace all three with the same block, adapted per keyword:

```rust
// CR 702.30a / CR 608.2d / CR 117.3c — PB-DP4, closes OOS-DP1-1.
//
// Paying or declining is a resolution-time cost choice (CR 118.12 / 608.2d), not an
// action that grants priority, so there is no actor for CR 117.3c to hand priority to.
// This site used to write `priority_holder = Some(active_player)` and clear
// `players_passed` as a bodge standing in for the payment pause DP-11 said was never
// implemented (PB-DP1 correctly left it alone; it is exactly the OOS-DP1-1 seed). The
// pause now exists as a DEADLINE (`force_resolve_overdue_payments`), so the bodge is gone:
//
//  * echo / cumulative upkeep: the controller IS the active player (the trigger reads "at
//    the beginning of YOUR upkeep"), and `resolve_top_of_stack`
//    (rules/resolution.rs:7768-7772) already cleared `players_passed` and granted priority
//    to the active player before this command could arrive. Removing the write is a
//    behaviour no-op.
//  * recover: the controller can be a NON-active player ("when a creature is put into YOUR
//    graveyard" fires on any player's turn), and the old write yanked priority away from
//    whoever legitimately held it and restarted the pass round. Removing it is a fix.
//
// CR 117.4 is not engaged: answering an out-of-band resolution-time payment is not "taking
// an action" between passes, so the pass set is left exactly as it is. Leaving it alone is
// also what makes the deadline work — a player must send `Pay*` BEFORE passing, and a
// spurious pass-set reset would silently buy them an extra round.
```

Verified safe against the existing suite: the write is reached only after
`resolve_top_of_stack` has already set `players_passed = OrdSet::new()` and
`priority_holder = Some(active)`, and every existing test sends `Pay*` immediately after the
resolving `pass_all`. `mechanics_e_l/echo.rs:564-570` (which forces `step = Untap` and
`priority_holder = Some(p1)` after paying and then calls `pass_all`) depends on
`players_passed` being empty at that point — and it is, from `resolve_top_of_stack`, not from
the deleted write.

#### Change 2e — CR 119.4 on the cumulative-upkeep life cost

**File**: `crates/engine/src/rules/engine.rs:859-870` (the
`CumulativeUpkeepCost::Life(amount)` arm of the `pay` branch).
**Defect found in planning**: the arm computes `total_life = amount * age_count` and subtracts it
with **no affordability check**, so `PayCumulativeUpkeep { pay: true }` can take a player below
0 life. The mana arm right above it (`:848-854`) does check. CR **119.4**: *"the player may do so
only if their life total is greater than or equal to the amount of the payment"* (CR 119.4b: 0 is
always payable).

**Action**: before mutating, reject:

```rust
// CR 119.4: a life payment greater than 0 is legal only if life_total >= the amount
// (CR 119.4b: 0 is always payable). PB-DP4: the mana arm above already checked
// affordability; this one did not, so a declined-by-inability upkeep silently drove the
// controller below 0 instead of sacrificing the permanent (CR 702.24a's "if you don't").
if total_life > 0 {
    let life_total = state
        .players
        .get(&player)
        .ok_or(GameStateError::PlayerNotFound(player))?
        .life_total;
    if (life_total as i64) < (total_life as i64) {
        return Err(GameStateError::InsufficientLife {
            player,
            required: total_life,
            actual: life_total,
        });
    }
}
```

`GameStateError::InsufficientLife { player, required: u32, actual: i32 }` already exists
(`state/error.rs:85-89`) and is the SR-36 idiom (`rules/mana.rs:298-302`). **No new error
variant.** Note that this arm is reached only from the interactive command — the sweep always
passes `pay: false` — so the `?` is safe here.

#### Change 2f — comment corrections (aspirationally-wrong comments are correctness hazards)

| file:line | current | change |
|---|---|---|
| `rules/resolution.rs:2804-2805` | *"The game pauses until a `Command::PayEcho` is received."* | Replace: the pause is a **deadline**, not a block — `rules/engine.rs::force_resolve_overdue_payments` applies CR 702.30a's "otherwise" at the end of the priority round if no `PayEcho` arrives. Name PB-DP4 / DP-11 and the CR 608.2d deviation. |
| `rules/resolution.rs` CU + recover arms (`:2843-2856`, `:2904-2912`) | same "pauses" framing in the arm doc blocks | same correction, per keyword |
| `state/mod.rs:256-261` | *"The game pauses until a `Command::PayEcho` is received for each entry."* + *"Only one echo payment can be pending at a time"* | Correct **both**: the deadline framing, and the multiplicity claim — two echo permanents at one upkeep queue two triggers and therefore two entries (CR 702.24b makes the same point explicitly for cumulative upkeep, pinned by `tests/mechanics_a_d/cumulative_upkeep.rs:631-691`). |
| `state/mod.rs:264-283` | same "pauses" framing on the CU + recover fields | same correction |
| `card-types/src/state/player.rs:147`, `:181` | *"(CR 106.12)"* on `can_spend` / `spend`'s restricted-mana doc | → **CR 106.6**. Live CR 106.12 is "tap for mana". Only these two (the two docs that govern the API DP-10 now calls); the other four occurrences (`:20`, `:46`, `:209`, and the `restriction_matches` region) are seed OOS-DP4-6. |
| `combat.rs:194-198` | the "requires a new `DeclareAttackers` command field" deferral | deleted by Change 1a |

---

### Change 3 — simulator: expose the three payment choices (criterion 3 / 5529)

#### Change 3a — three new `LegalAction` variants

**File**: `crates/simulator/src/legal_actions.rs`, appended to `pub enum LegalAction`
(`:16-120`), after `CastMorphFaceDown` (`:114-119`).

```rust
/// CR 702.30a (PB-DP4 / DP-11): answer an outstanding echo payment. `pay: false` is
/// always offered (declining is always legal — CR 118.12a); `pay: true` is offered only
/// when `casting::can_pay_cost` says the engine will accept it, mirroring
/// `handle_pay_echo`'s own check (SR-38 precedent: never offer a payment the engine
/// rejects).
PayEcho { permanent: ObjectId, pay: bool },
/// CR 702.24a (PB-DP4 / DP-11): answer an outstanding cumulative upkeep payment. The
/// total is `per_counter_cost` x the permanent's age counters (CR 702.24b counts ALL age
/// counters on the permanent, not per-ability). `pay: true` is gated on the mana pool for
/// `CumulativeUpkeepCost::Mana` and on the life total for `::Life` (CR 119.4).
PayCumulativeUpkeep { permanent: ObjectId, pay: bool },
/// CR 702.59a (PB-DP4 / DP-11): answer an outstanding recover payment. `pay: false`
/// exiles the card; declining is always legal.
PayRecover { recover_card: ObjectId, pay: bool },
```

#### Change 3b — enumerate them in `StubProvider::legal_actions`

**File**: `crates/simulator/src/legal_actions.rs`.
**Site**: immediately after `actions.push(LegalAction::PassPriority);` at **`:200`**.

**Design call — append, do NOT early-return.** The commander-zone (`:174-183`) and mulligan
(`:185-190`) blocks early-return, because those decisions genuinely exclude everything else. A
payment must **not**: CR 608.2g explicitly lets the player activate mana abilities before paying,
and the engine's payment path reads only the pool (it never auto-taps), so early-returning would
make `pay: true` reachable only when the pool happens to already be funded. Appending after
`PassPriority` keeps `TapForMana` in the list and is the only shape that lets a bot or a human
fund the payment.

Logic (all three read the **pending vector**, which already carries the cost — see §4.5 for why
that is mandatory, not merely convenient):

```rust
// PB-DP4 / DP-11: an outstanding pay-or-lose-it payment. Offered as ordinary
// priority-window actions rather than a separate blocking decision, because the engine's
// deadline is the end of this priority round (rules/engine.rs::
// force_resolve_overdue_payments) and because CR 608.2g lets the player activate mana
// abilities first -- so TapForMana must stay available alongside these.
//
// Not answering is a legal decline (CR 118.12a); the engine applies it at the boundary.
for (owing, permanent, cost) in state.pending_echo_payments().iter() {
    if *owing != player { continue; }
    actions.push(LegalAction::PayEcho { permanent: *permanent, pay: false });
    if mtg_engine::rules::casting::can_pay_cost(&pool, cost) {
        actions.push(LegalAction::PayEcho { permanent: *permanent, pay: true });
    }
}
```

- `pool` = `state.player(player).map(|p| p.mana_pool.clone())` — hoist it once next to
  `life_total` at `:218`.
- **Cumulative upkeep**: compute `age_count` from the permanent's `CounterType::Age` counter
  (CR 702.24b: the total on the permanent), then
  - `CumulativeUpkeepCost::Mana(mc)`: gate on `can_pay_cost(&pool, &multiply(mc, age_count))`.
    The provider needs its own multiply — `engine.rs::multiply_mana_cost` is private. Write a
    small private `fn multiply_mana_cost(cost: &ManaCost, times: u32) -> ManaCost` in
    `legal_actions.rs` that mirrors `engine.rs:978-1002` **exactly**, including
    `hybrid`/`phyrexian`/`x_count`, or the gate will disagree with the engine (SR-38). Note the
    duplication in seed OOS-DP4-7.
  - `CumulativeUpkeepCost::Life(amount)`: gate on `life_total >= (amount * age_count) as i32`
    (CR 119.4 / 119.4b — mirrors Change 2e).
- **Recover**: gate `pay: true` on `can_pay_cost(&pool, cost)`.
- **`pay: false` is always pushed**, for all three, unconditionally.

`mtg_engine::rules::casting::can_pay_cost` is `pub` (`casting.rs:6596`). Verify it is reachable
through the `mtg_engine` facade; if not, use the fully-qualified path the crate already exposes
(the file already reaches into `mtg_engine::rules::layers::calculate_characteristics` at `:319`
and `:462`, so `rules::` is reachable).

#### Change 3c — `action_to_command` arms (the one exhaustive match)

**File**: `crates/simulator/src/random_bot.rs`, `pub(crate) fn action_to_command`
(`:128-349`). **This match has no catchall** — the last arm is `ActivateLoyaltyAbility` at
`:338-347` and the match closes at `:348`. Three new arms are a **compile error until added**:

```rust
LegalAction::PayEcho { permanent, pay } => Command::PayEcho {
    player,
    permanent: *permanent,
    pay: *pay,
},
LegalAction::PayCumulativeUpkeep { permanent, pay } => Command::PayCumulativeUpkeep {
    player,
    permanent: *permanent,
    pay: *pay,
},
LegalAction::PayRecover { recover_card, pay } => Command::PayRecover {
    player,
    recover_card: *recover_card,
    pay: *pay,
},
```

`RandomBot::choose_action` (`:56-57`) picks a uniformly random index, so both branches get
exercised. `HeuristicBot` shares the same chokepoint (`heuristic_bot.rs:127`) — **no separate
edit**. `driver.rs` and `local_game.rs` construct no `Pay*` command themselves.

#### Change 3d — `LocalGame`: nothing to add (answers audit §9 rec 3)

**No change to `crates/simulator/src/local_game.rs`.** Audit §9 recommendation 3 asked that
`advance()` yield `AwaitingHuman` for a non-empty pending-payment vector. **That premise is
falsified by this design**: because the deadline is the end of the priority round, the payment
*is* a priority-window action, so it arrives in the existing `PendingDecision` whose actions come
from `self.provider.legal_actions(&self.state, acting_player)` (`local_game.rs:329`), classified
by `decision_kind_for` (`:575-592`) as `DecisionKind::Priority`. **No new `DecisionKind` variant,
no reshaping of `PendingDecision`, no new DTO for M11-local sessions 5/7.** Record this in the
audit row (§10) — it is a cheaper outcome than the recommendation predicted.

#### Change 3e — TUI: out of scope, deliberately

`tools/tui/src/play/input.rs` and `panels/action_menu.rs` reach into the action list only via
`iter().any(matches!(…))` / `.find(…)` (`input.rs:49/59/78/122/140/164/193/570`,
`action_menu.rs:118-130`) — **no exhaustive `LegalAction` match**, so nothing breaks.

**Decision: add a minimal keybinding, or nothing at all — the runner's call, but no more than
this.** The TUI's play loop is a W2 surface with no payment prompt UI, and building one is M11
work. If a keybinding is added, it must offer both branches and must read the pending vector, not
guess. If the runner adds nothing, say so in the task comment; criterion 5529 names
`LegalActionProvider` and the bots, not the TUI. **Do not build a modal prompt here.**

---

## 4. Blast radius — enumerated, not estimated

Method: (a) grep the corpus for the two `GameRestriction::CantAttackYouUnlessPay` defs and the
five echo/CU/recover defs; (b) grep every reader of the three `pending_*` fields across
`crates/`; (c) grep `test-data/generated-scripts/` for all seven card names plus the words
`echo` / `cumulative` / `Propaganda` / `Ghostly Prison`; (d) trace each exhaustive match on the
enums touched.

### 4.1 Engine source

| file:line | change |
|---|---|
| `crates/engine/src/rules/combat.rs:9`, `:14` | imports: `super::casting`, `ManaCost`, `BTreeMap`/`BTreeSet` |
| `crates/engine/src/rules/combat.rs:185-265` | **Change 1a** — replace the block with a bound `(Option<ManaCost>, BTreeSet<PlayerId>)` computation; delete the false `:194-198` deferral comment |
| `crates/engine/src/rules/combat.rs:304-321` | **Change 1c** — replace with `has_uncosted_attack_target` |
| `crates/engine/src/rules/combat.rs:403-420` | **Change 1c** — same |
| `crates/engine/src/rules/combat.rs:298-303`, `:398-402` | comment updates (CR 508.1d cost carve-out, OOS-RS3-4 closed) |
| `crates/engine/src/rules/combat.rs` after `:612` | **Change 1b** — the debit + `ManaCostPaid` |
| `crates/engine/src/rules/combat.rs` after `:685` | **Change 1c/1d** — two new private fns |
| `crates/engine/src/rules/engine.rs:754-766` | **Change 2d** — delete the echo priority writes |
| `crates/engine/src/rules/engine.rs:859-870` | **Change 2e** — CR 119.4 life gate |
| `crates/engine/src/rules/engine.rs:963-975` | **Change 2d** — delete the CU priority writes |
| `crates/engine/src/rules/engine.rs:1075` | **Change 2c** — `expect_move_object_to_zone` |
| `crates/engine/src/rules/engine.rs:1085-1097` | **Change 2d** — delete the recover priority writes |
| `crates/engine/src/rules/engine.rs` after `:1098` | **Change 2a** — `force_resolve_overdue_payments` |
| `crates/engine/src/rules/engine.rs:1715-1717` | **Change 2b** — the hook |
| `crates/engine/src/rules/resolution.rs:2804-2805`, `:2843-2856`, `:2904-2912` | **Change 2f** — comment corrections |
| `crates/engine/src/state/mod.rs:255-283` | **Change 2f** — comment corrections (**doc comments only — do not touch the field declarations or widen the SR-3 seal**) |
| `crates/card-types/src/state/player.rs:147`, `:181` | **Change 2f** — CR 106.12 → CR 106.6 |

**Not touched, verified**: `state/hash.rs` (both the `CantAttackYouUnlessPay` arm at `:2514`,
the three pending-vector arms at `:7736-7754`, and the `ManaCostPaid` arm at `:4304` already
exist and need no edit — nothing about their shape changes); `state/builder.rs:337-339`;
`rules/priority.rs`; `rules/turn_structure.rs`; `rules/sba.rs`; `rules/casting.rs:6763` (the
"attack restrictions don't affect casting" no-op arm stays correct).

### 4.2 Engine tests

| file:line | test | expected effect |
|---|---|---|
| `crates/engine/tests/rules/restrictions.rs:650-694` | `test_restriction_cant_attack_you_unless_pay_blocks_broke_attacker` | **passes unchanged.** Asserts `is_err()` + `err.contains("attack tax")` — Change 1a preserves the substring. |
| `:699-742` | `..._allows_funded_attacker` | **passes unchanged** (asserts `is_ok()`), but **add a debit assertion** (§7 probe 1): `mana_pool.total() == 0` after. Fails pre-fix. |
| `:747-803` | `..._stacked_costs` | passes unchanged (2 mana vs stacked {4}). |
| `:808-848` | `..._does_not_affect_other_targets` | passes unchanged. |
| `crates/engine/tests/mechanics_e_l/echo.rs` (7 tests, `:163-790`) | all | **all pass unchanged.** Traced: every test either never resolves the trigger (`:283-326`, `:591+`, `:683+`, `:738+`) or sends `PayEcho` immediately after the resolving `pass_all` with no intervening pass (`:361→388`, `:456→462`, `:525→544`). The sweep lives in the stack-**empty** branch and cannot fire in the resolving call. `:564-570`'s manual `step = Untap` + `pass_all` relies on `players_passed` being empty, which `resolve_top_of_stack:7769` supplies. |
| `crates/engine/tests/mechanics_a_d/cumulative_upkeep.rs` (8 tests, `:159-691`) | all | **all pass unchanged**, same trace. `:398-441` (`escalating_cost`, second upkeep after paying) and `:631-691` (`multiple_instances_share_counters`, two triggers queued but never resolved) both verified individually. |
| `crates/engine/tests/mechanics_m_z/recover.rs` (8 tests, `:149-600`) | all | **all pass unchanged**, same trace (`:239→264`, `:322→335`, `:390-408` fizzle case). |
| `crates/engine/tests/core/keyword_registry.rs` | site-scan tests | **pass only if no new file names `KeywordAbility::Echo`/`::CumulativeUpkeep`/`::Recover` in executable code.** See §4.5. |
| `crates/engine/tests/core/ability_definition_registry.rs` | site-scan tests | same, for `AbilityDefinition::` variants. See §4.5. |
| `crates/engine/tests/scripts/run_all_scripts.rs` | golden corpus | §4.3 |
| **new** `crates/engine/tests/primitives/pb_dp4_attack_tax_and_payment_deadline.rs` | — | §7 |
| `crates/engine/tests/primitives/main.rs` | module list | **add `mod pb_dp4_attack_tax_and_payment_deadline;`** alphabetically after `mod pb_dp3_…;`. SR-9a: a missing `mod` line silently deletes the whole file's coverage. **Never** create a top-level `crates/engine/tests/*.rs`. |

### 4.3 Golden scripts

Search: grep the whole corpus for `Propaganda` / `Ghostly Prison` / `echo` / `Echo` /
`cumulative` / `Cumulative` / all seven card names.

| file | status | effect |
|---|---|---|
| `test-data/generated-scripts/stack/151_echo_avalanche_riders.json` | **`review_status: "retired"`** (`:22`) — does not run | no edit |
| `test-data/generated-scripts/stack/152_cumulative_upkeep_mystic_remora.json` | `approved` (`:20`) | **no edit.** Traced: it casts Mystic Remora in a main phase and ends at `zones.battlefield.p1 includes Mystic Remora` (`:150-166`). It never reaches an upkeep, so no CU trigger and no pending entry. Its own note at `:147` says so. |
| `test-data/generated-scripts/stack/153_recover_grim_harvest.json` | `approved` (`:27`) | **no edit.** `priority_round … all_pass` (`:228-235`) resolves the RecoverTrigger, then `pay_recover` (`:256-265`) answers it with **no intervening priority round**. The sweep never sees the entry. Its `:234`/`:253` notes ("Game pauses until `Command::PayRecover` is received") are now accurate for the first time. |
| `test-data/generated-scripts/stack/161_amass_dreadhorde_invasion.json` | matched only on the substring "echo" inside unrelated prose | no edit |
| **No script anywhere uses Propaganda or Ghostly Prison** (only `test-data/test-decks/` and `test-data/test-cards/` inventory files match) | — | no edit |

**Runner gate**: run `SCRIPT_FILTER=152 cargo test --test run_all_scripts -- --nocapture` and the
same for `153`. **Do not start the replay-viewer HTTP server** — agent-launched it gets SIGKILL
(137).

### 4.4 Replay harness

| file:line | change |
|---|---|
| `crates/engine/src/testing/replay_harness.rs:906-923` | **no change required.** The `pay_recover` action already resolves `pay: true` from `card_name` and `pay: false` from the pending vector (`:911-917`). |
| — | **Deliberate non-change**: there is **no** `pay_echo` or `pay_cumulative_upkeep` script action. A future script that wants to pay echo or CU cannot. Latent (no script needs it today; the two echo/CU scripts are retired / never reach an upkeep). **Seed OOS-DP4-4.** Do not add them speculatively — a script action with no script is dead code the SR-9c triage would flag. |

### 4.5 Registry gates — the trap that bit PB-DP3

`crates/engine/tests/core/keyword_registry.rs` and
`crates/engine/tests/core/ability_definition_registry.rs` both walk
**`SCAN_ROOTS = ["crates/engine/src", "crates/card-types/src", "crates/simulator/src"]`**
(`keyword_registry.rs:31-35`, `ability_definition_registry.rs:30-34`), strip comments and string
literals, and assert that each variant's declared `sites` list **exactly equals** the set of files
that mention it. Adding a mention in an undeclared file is a hard test failure — this is what cost
PB-DP3 an unplanned edit.

**The specific hazard for PB-DP4** (verified by reading `state/keyword_registry.rs`):

| variant | declared `sites` | risk |
|---|---|---|
| `K::Echo(..)` (`:474-480`) | `rules/lands.rs`, `rules/resolution.rs`, `rules/turn_actions.rs` — **not `rules/engine.rs`, not `crates/simulator/src`** | naming it in the sweep or the provider breaks the gate |
| `K::CumulativeUpkeep(..)` (`:481-486`) | `rules/resolution.rs`, `rules/turn_actions.rs` — same omissions | same |
| `K::Recover` (`:454-459`) | `rules/abilities.rs`, `rules/resolution.rs` — same omissions | same |
| `K::MustAttackEachCombat` (`:649`) | `rules/combat.rs` | **already declared** — Change 1c is safe |

**Mandatory prescription**: read the payment kind and cost from the **pending vector**
(`(PlayerId, ObjectId, ManaCost)` / `(PlayerId, ObjectId, CumulativeUpkeepCost)`), which already
carries everything both the sweep and the provider need. **Do not name
`KeywordAbility::Echo` / `::CumulativeUpkeep` / `::Recover`, or any `AbilityDefinition::`
variant, in executable code in `rules/engine.rs` or `crates/simulator/src/legal_actions.rs`.**
Comments and doc comments are stripped before the scan and are safe — cite the keywords freely
there. `CumulativeUpkeepCost` (`state/types.rs`) is a plain enum with no registry; matching on
`Mana(..)` / `Life(..)` is safe.

If a site genuinely must be added, declare it in `state/keyword_registry.rs` in the same commit
and say so in the task comment.

### 4.6 Exhaustive matches — the complete list

| enum | file:line | shape | action |
|---|---|---|---|
| `LegalAction` | `crates/simulator/src/random_bot.rs:134-348` (`action_to_command`) | **exhaustive, no catchall** | **3 new arms (Change 3c)** |
| `LegalAction` | `crates/simulator/src/heuristic_bot.rs:127` | delegates to `action_to_command` | no change |
| `LegalAction` | `crates/simulator/src/local_game.rs:575-592` (`decision_kind_for`) | `iter().any(matches!)` | no change |
| `LegalAction` | `tools/tui/src/play/input.rs` (8 sites), `panels/action_menu.rs:118-130` | `any(matches!)` / `find` | no change |
| `GameEvent` | `tools/replay-viewer/src/view_model.rs`, `tools/tui/src/play/panels/stack_view.rs` | exhaustive on `StackObjectKind` + `KeywordAbility` | **no change — no new variant is added anywhere.** `ManaCostPaid` is reused. Build the workspace anyway. |
| `GameRestriction` | `state/hash.rs:2514`, `rules/casting.rs:6763`, `rules/combat.rs` | exhaustive in hash + casting | **no change — no new variant** |
| `Command` / `Effect` / `StackObjectKind` | — | — | **no change — no new variant** |

### 4.7 Negative-space clause — READ THIS

The blast radius above is **falsifiable by construction**, and PB-DP3's under-enumeration of
exactly this section is why the clause exists.

**Any compile error, test failure, clippy warning, `check-defs-fmt.sh` failure, registry-gate
failure, golden-script failure, or fingerprint/hash mismatch at a site NOT listed in §4.1-§4.6 is
an un-enumerated site.** When you hit one:

1. **Report it** — in the ESM task comment *and* in `memory/primitives/pb-review-DP4.md`, naming
   the file, line, and the mechanism that caught it.
2. **Then** fix it minimally.
3. **Do not** silently patch it, and do not widen the change's scope to accommodate it without
   saying so.

Specific things to watch for that this plan may have missed:
- a `HashInto` allowlist entry (SR-33-era gate) if any hashed shape shifts;
- `crates/engine/tests/core/` gate tests other than the two registries;
- a property test in `crates/engine/tests/properties/` that asserts a permanent's survival across
  a turn;
- `crates/simulator/tests/local_game.rs` (10 tests) if the new actions change an action count;
- `tools/replay-viewer` or `tools/tui` compile breakage from the `LegalAction` addition.

---

## 5. Card definition fixes

**None required.** The five `Complete` defs that are live-wrong today are made correct by the
engine changes with **zero edits** — the defs already declare exactly the right thing and the
engine simply did not enforce it:

| def | keyword / restriction | marker | wrong today | after PB-DP4 |
|---|---|---|---|---|
| `crates/card-defs/src/defs/propaganda.rs:19-26` | `CantAttackYouUnlessPay { generic: 2 }` | `Complete` (no field ⇒ default) | tax never charged | correct |
| `crates/card-defs/src/defs/ghostly_prison.rs:20` | same | `Complete` | tax never charged | correct |
| `crates/card-defs/src/defs/mogg_war_marshal.rs` | `Echo` | `Complete` | never sacrificed when unpaid | correct |
| `crates/card-defs/src/defs/avalanche_riders.rs:61` | `Echo` | `Completeness::Complete` (explicit) | never sacrificed when unpaid | correct |
| `crates/card-defs/src/defs/grim_harvest.rs` | `Recover` | `Complete` | never exiled when unpaid | correct |
| `crates/card-defs/src/defs/mystic_remora.rs:59-64` | `CumulativeUpkeep(Mana{1})` | `known_wrong` (MayPayOrElse, DP-12) | its note's clause *"Cumulative upkeep {1} … are correct"* is false | **note becomes true**; marker stays `known_wrong` for the unrelated `MayPayOrElse` gap |
| `crates/card-defs/src/defs/tombstone_stairwell.rs:53` | `CumulativeUpkeep` | `partial` (token provenance) | CU unenforced | CU correct; marker **cannot** upgrade (unrelated blockers survive) |

**Optional single-line touch (runner's call, low value):** `mystic_remora.rs:60` cites
`effects/mod.rs:3196` for `MayPayOrElse`; the live site is `effects/mod.rs:3425-3428` (audit
§4.8). Line drift in a `known_wrong` note about a *different* primitive. If touched, say so; if
not, it is seed material for a doc pass, not for this PB.

### 5.1 MANDATORY TODO sweep (roster-recall gate)

Grepped `crates/card-defs/src/defs/` for
`TODO.*(attack tax|Propaganda|Ghostly|echo|Echo|cumulative upkeep|CumulativeUpkeep|recover|Recover|pay-or-sac|unless.*pays)`.

**Result: 1 hit, 0 forced adds.** `crates/card-defs/src/defs/mystic_remora.rs:34` —
*"TODO: 'may draw unless that player pays {4}' — MayPayOrElse still a gap."* That TODO names
**`Effect::MayPayOrElse`** (audit §4.8 / **DP-12**), a different primitive with a different fix
(a `Command`-level optional-payment channel). It is **not** the echo/CU/recover payment deadline
and **not** the attack tax. **Not a forced add.**

Positive assertion: **the gate was run and produced no roster additions.** The independent
keyword grep (both `KeywordAbility::` and `AbilityDefinition::` spellings) found the complete
5-def echo/CU/recover roster and the complete 2-def restriction roster listed in §5, which is
the same roster the brief predicted, minus `bala_ged_recovery.rs` (see §1.2) and minus
`goblin_rabblemaster.rs` (see §1.1).

---

## 6. New card definitions

**None.** Both attack-tax cards and all five payment cards already exist. The coloured attack
tax, the hybrid attack tax, the restricted-mana case and the CU life-cost overpayment are all
**latent** — no shipped card reaches them — so every test for those must build its
`GameRestriction` / `KeywordAbility` synthetically (the `add_restriction` helper in
`tests/rules/restrictions.rs:665` and the `ObjectSpec::…with_keyword(…)` idiom in
`tests/mechanics_e_l/echo.rs:146-156`).

---

## 7. Test plan

**Primary file (new)**: `crates/engine/tests/primitives/pb_dp4_attack_tax_and_payment_deadline.rs`
**Registration (mandatory, SR-9a)**: add `mod pb_dp4_attack_tax_and_payment_deadline;` to
`crates/engine/tests/primitives/main.rs`, alphabetically immediately after the PB-DP3 line.

**Patterns to copy verbatim**: `crates/engine/tests/rules/restrictions.rs:638-694` for the
attack-tax scaffolding (`declare_cmd`, `add_restriction`, `find_by_name`,
`GameStateBuilder::four_player().player_mana(…)`); `crates/engine/tests/mechanics_e_l/echo.rs:
26-156` for the payment scaffolding (`pass_all`, `find_in_zone`, `on_battlefield`,
`in_graveyard`, the `Designations::ECHO_PENDING` seed, and the `at_step(Step::Untap)` +
`priority_holder = Some(p1)` idiom that walks Untap→Upkeep without touching the library).

**Gotcha (mandatory)**: `ObjectSpec::card()` creates naked objects — call `enrich_spec_from_def()`
(or use `ObjectSpec::creature` / `.with_keyword(…)` as the existing tests do) so the object
actually carries the keyword, or the probe passes vacuously.

### 7.1 DP-10 fail-before / pass-after probes

For each, the **observable pre-fix behaviour** is stated so the runner can confirm the probe
really fails against the pre-fix engine. Record every before/after in the task comment.

| # | test | CR | pre-fix behaviour the probe pins | post-fix |
|---|---|---|---|---|
| 1 | `test_508_1j_attack_tax_is_debited_from_the_pool` | 508.1j | p2 has `{C}{C}`, p1 has Propaganda `{2}`, p2 attacks p1 with one Bear. Declaration **succeeds and `p2.mana_pool.total() == 2`** — the mana is still there. | succeeds and `total() == 0`; a `GameEvent::ManaCostPaid { player: p2, cost: {generic:2} }` is in the returned events |
| 2 | `test_508_1h_attack_tax_colour_is_not_flattened_to_generic` | 508.1h, 106.1 | synthetic restriction `cost_per_creature: ManaCost { white: 2, ..default }`; p2's pool is `{C}{C}`. Declaration **succeeds** (`total_with_restricted() == 2 >= total_tax == 2`) — the wrong colours paid a coloured cost. | `Err`, message contains `"attack tax"` |
| 2b | `test_508_1j_coloured_attack_tax_paid_with_correct_colours` | 508.1j | same restriction, pool `{W}{W}`: **succeeds, pool still `{W}{W}`** | succeeds, `white == 0` |
| 3 | `test_106_6_restricted_mana_cannot_pay_an_attack_tax` | 106.6, 508.1j | p2's pool holds **only** `add_restricted(Green, 2, ManaRestriction::CreatureSpellsOnly)` and zero unrestricted. Propaganda `{2}`. Declaration **succeeds** (`total_with_restricted() == 2`) and nothing is spent — the tax was affordable on paper and unpayable in fact. | `Err`, message contains `"attack tax"` and `"106.6"` |
| 4 | `test_508_1h_attack_tax_sums_per_defender_and_per_attacker` | 508.1h | 4-player: p1 has Propaganda `{2}`, p3 has Ghostly Prison `{2}`; p2 declares 2 attackers at p1 and 1 at p3 with `{6}` floating. **Succeeds, pool still 6.** | succeeds, `total() == 0`; `ManaCostPaid { cost.generic == 6 }` |
| 5 | `test_508_1c_planeswalker_attack_is_not_taxed` | 508.1c | (regression guard, passes before **and** after) p1 has Propaganda and a planeswalker; p2 with **zero** mana attacks the planeswalker → `Ok`. Label it a guard. |
| 6 | `test_107_4e_hybrid_attack_tax_is_rejected_not_paid_free` | 107.4e/f, 508.1h | synthetic `cost_per_creature` with a non-empty `hybrid` vector. Pre-fix: the hybrid pip is **invisible** to `:221-227`'s sum, so a `{2/W}` tax contributes **0** and the declaration succeeds for free. | `Err`, message contains `"attack tax"` and `"OOS-DP4-1"` |
| 7 | `test_508_1d_must_attack_creature_is_not_forced_to_pay_an_attack_tax` | **508.1d** | 2-player: p2 controls a `MustAttackEachCombat` creature and **zero** mana; p1 has Propaganda `{2}`. **Both** `DeclareAttackers { attackers: vec![] }` **and** `DeclareAttackers { attackers: vec![(creature, Player(p1))] }` return `Err` — assert both, which *is* the deadlock. | the empty declaration returns **`Ok`** (the requirement is not obeyable without paying), and the paying declaration still returns `Err` for lack of mana |
| 8 | `test_508_1d_goaded_creature_is_not_forced_to_pay_an_attack_tax` | 508.1d, 701.15b | same shape with `goaded_by` instead of the keyword: pre-fix both declarations are `Err`. | empty declaration `Ok` |
| 9 | `test_508_1d_must_attack_still_forced_when_an_untaxed_opponent_exists` | 508.1d | (regression guard) 4-player: only p1 has Propaganda; p3/p4 untaxed ⇒ the empty declaration must still be **rejected** before and after. Pins that Change 1c did not simply disable must-attack. |
| 10 | `test_508_1d_must_attack_still_forced_when_only_an_opponent_planeswalker_is_untaxed` | 508.1d | (guard) all opponents taxed but one controls a planeswalker ⇒ still forced. Pins the planeswalker clause of `has_uncosted_attack_target`. |

### 7.2 DP-11 fail-before / pass-after probes

| # | test | CR | pre-fix behaviour the probe pins | post-fix |
|---|---|---|---|---|
| 11 | `test_702_30a_unanswered_echo_is_sacrificed_at_the_round_boundary` | 702.30a, 118.12a | echo permanent with `ECHO_PENDING`, no mana. Resolve the trigger (`pass_all`), then `pass_all` **again** without sending `PayEcho`. Pre-fix: permanent **still on the battlefield**, `pending_echo_payments().len() == 1`, and `turn().step` has advanced to `Draw`. | permanent in `Graveyard(p1)`, a `CreatureDied` event, `pending_echo_payments()` empty, and `turn().step == Upkeep` (the extra round) |
| 12 | `test_702_24a_unanswered_cumulative_upkeep_is_sacrificed_at_the_round_boundary` | 702.24a | same shape with `CumulativeUpkeep(Mana{1})`: pre-fix survives with 1 age counter and the step advances. | sacrificed; pending vector empty |
| 13 | `test_702_59a_unanswered_recover_card_is_exiled_at_the_round_boundary` | 702.59a | Grim-Harvest-shaped recover card in `Graveyard(p1)`, a creature dies, trigger resolves, then `pass_all` again. Pre-fix: card **still in the graveyard**, `pending_recover_payments().len() == 1`. | card in `ZoneId::Exile`, a `RecoverDeclined` event |
| 14 | `test_702_30a_echo_paid_before_the_boundary_still_survives` | 702.30a | (regression guard, passes before **and** after) fund the pool, send `PayEcho { pay: true }`, then `pass_all` twice → still on the battlefield. Pins that the deadline does not eat a *paid* echo. |
| 15 | `test_dp11_boundary_sweep_does_not_deadlock_the_priority_round` | 117.4 | **the anti-deadlock pin, mandatory.** From probe 11's post-sweep state, `pass_all` once more and assert the step **advances** (Upkeep → Draw) and `priority_holder` is `Some(active)` in the new step. Pre-fix this test is vacuous (the step already advanced); post-fix it proves the extra round terminates. Also assert `pending_*` all empty. |
| 16 | `test_101_4_multiple_outstanding_payments_resolve_in_apnap_order` | 101.4 | 4-player, p1 active: p1 owes an echo and p3 owes a recover simultaneously (queue both by resolving both triggers). Pre-fix: neither resolves. | both resolve in one sweep and p1's `CreatureDied` appears **before** p3's `RecoverDeclined` in the returned event vector |
| 17 | `test_dp11_answering_a_payment_does_not_reassign_priority` | 117.3c, 608.2d | **OOS-DP1-1.** p3 (non-active) owes a recover payment; p2 holds priority. Send `PayRecover { player: p3, pay: false }`. Pre-fix: `priority_holder` is yanked to `Some(p1)` (the active player) and `players_passed` is cleared. | `priority_holder` is still `Some(p2)` and `players_passed` is unchanged |
| 18 | `test_119_4_cumulative_upkeep_life_cost_beyond_life_total_is_rejected` | 119.4 | `CumulativeUpkeepCost::Life(3)` with 2 age counters (total 6) and `life_total == 5`. Pre-fix: `PayCumulativeUpkeep { pay: true }` **succeeds** and `life_total` becomes `-1`. | `Err(GameStateError::InsufficientLife { required: 6, actual: 5 })` and the permanent is untouched |
| 18b | `test_119_4b_cumulative_upkeep_zero_life_cost_is_always_payable` | 119.4b | (guard) `Life(0)` is payable at any life total. |
| 19 | `test_702_24b_two_cumulative_upkeep_instances_both_reach_the_boundary` | 702.24b | two CU instances on one permanent (the `Mana{1}` + `Mana{2}` distinct-value idiom from `cumulative_upkeep.rs:639-654` — `imbl::OrdSet` dedupes equal values). Resolve both triggers, then pass: pre-fix the permanent survives with 2 age counters and both entries stranded. | the **first** forced decline sacrifices it; the second entry is consumed and produces no events (CR 400.7 — the permanent already left), and the pending vector ends empty. Pins the sweep's snapshot-then-mutate discipline. |
| 20 | `test_608_2g_mana_ability_during_the_payment_window_still_funds_the_payment` | 608.2g | (guard) with the trigger resolved and the payment outstanding, `TapForMana` then `PayEcho { pay: true }` succeeds. Pins that the deadline design preserves the CR 608.2g window the design section argues for. |

### 7.3 Simulator tests

Inline in `crates/simulator/src/legal_actions.rs`'s existing `#[cfg(test)] mod tests` (that
file's own convention — `:1370`, `:1390`, `:1616`).

| # | test | assertion |
|---|---|---|
| 21 | `provider_offers_both_echo_branches_when_the_cost_is_affordable` | pending echo `{2}` + pool `{C}{C}` ⇒ both `PayEcho { pay: true }` and `{ pay: false }` present |
| 22 | `provider_omits_echo_pay_when_the_cost_is_unaffordable` | pool empty ⇒ only `{ pay: false }` (SR-38) |
| 23 | `provider_gates_cumulative_upkeep_mana_on_age_counter_multiple` | `Mana{1}` with 3 age counters + pool `{C}{C}` ⇒ `pay: true` **absent**; with `{C}{C}{C}` ⇒ present. Pins CR 702.24b's total-counter multiply. |
| 24 | `provider_gates_cumulative_upkeep_life_on_life_total` | `Life(3)` × 2 counters at 5 life ⇒ `pay: true` absent (CR 119.4); at 6 life ⇒ present |
| 25 | `provider_offers_recover_decline_always` | pending recover, empty pool ⇒ `PayRecover { pay: false }` present, `{ pay: true }` absent |
| 26 | `provider_still_offers_tap_for_mana_alongside_a_pending_payment` | pins Change 3b's append-don't-early-return decision: with a pending echo **and** an untapped land, both `PayEcho` and `TapForMana` appear (CR 608.2g) |
| 27 | `provider_offers_no_payment_action_to_a_player_who_owes_nothing` | another player's pending entry does not leak into this player's list |
| 28 | `action_to_command_round_trips_the_three_payment_actions` | in `random_bot.rs`'s test module (or `legal_actions.rs` if that's where the existing round-trip tests live): each new `LegalAction` maps to the matching `Command` with the same `pay` flag |

### 7.4 End-to-end smoke — record the result, do not commit it

`crates/simulator/src/bin/fuzzer.rs` is **not currently usable** as a smoke test:
**OOS-DP3-9** records that `mtg-fuzzer --games 15` aborts with a stack overflow (pre-existing on
`main`) and that long games flood `stack_consistency` violations. Do **not** treat a fuzzer crash
as a PB-DP4 regression, and do **not** spend the session on it.

Do run `cargo run --release --bin mtg-fuzzer -- --games 5 --seed 1` once before and once after,
and record in the task comment whether any **new** `InvalidCommand` rejection mentioning
`"attack tax"` or `"echo"`/`"upkeep"`/`"recover"` appears, and whether the turn counts move.
Expect: echo/CU permanents now dying at their controller's upkeep and recover cards being exiled
— that is the fix working, not a regression. Note that `driver.rs` answers a rejected command
with a silent `PassPriority`, so a *simulator* regression here is invisible; the unit tests in
§7.3 are the real gate.

---

## 8. Verification checklist

- [ ] `cargo check -p mtg-engine` clean after Change 1, then after Change 2
- [ ] `cargo build --workspace` clean — catches the `random_bot.rs` exhaustive-match arms, the TUI, and the replay-viewer
- [ ] `cargo test --all` green, **with no failure outside the §4.1-§4.6 enumeration** (§4.7)
- [ ] `cargo clippy --workspace -- -D warnings` clean
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35 — `cargo fmt` checks none of the 1,798 card defs; `cargo test --all` runs the script via `core card_defs_fmt`)
- [ ] `PROTOCOL_VERSION == 27` (`rules/protocol.rs:260`), `PROTOCOL_SCHEMA_FINGERPRINT` unchanged, `HASH_SCHEMA_VERSION == 63` — verified by the existing parity tests. **Do not edit either constant.** If either moves, **STOP** (§0).
- [ ] `cargo test -p mtg-engine --test core keyword_registry` and `… ability_definition_registry` green — the §4.5 gate
- [ ] `SCRIPT_FILTER=152 cargo test --test run_all_scripts -- --nocapture` and the same for `153`. **Do not start the replay-viewer HTTP server.**
- [ ] All **14** fail-before probes (§7.1 #1, 2, 2b, 3, 4, 6, 7, 8; §7.2 #11, 12, 13, 16, 17, 18) demonstrated **failing on the pre-fix engine and passing after**, with the observable pre-fix behaviour recorded in the task comment
- [ ] The 6 regression guards (§7.1 #5, 9, 10; §7.2 #14, 15, 20) pass **before and after**, labelled as guards
- [ ] `tests/rules/restrictions.rs:690`'s `err.contains("attack tax")` assertion still passes unmodified
- [ ] `mechanics_e_l/echo.rs`, `mechanics_a_d/cumulative_upkeep.rs`, `mechanics_m_z/recover.rs` all green **without edits**; if any needs an edit, that is an un-enumerated site (§4.7)
- [ ] `crates/simulator/tests/local_game.rs` green without edits
- [ ] No remaining occurrence of *"The game pauses until"* in `rules/resolution.rs` or `state/mod.rs` (Change 2f)
- [ ] No remaining occurrence of *"requires a new DeclareAttackers command field"* in `rules/combat.rs`
- [ ] Audit rows updated per §10; seeds filed per §9
- [ ] `memory/primitive-wip.md` phase advanced; close-out appended to `memory/workstream-state.md` "PB-DP suite — worker close-outs"; CLAUDE.md "Current State" + "Last Updated" delta

---

## 9. Seeds to file in `docs/audits/decision-point-audit.md` §8.1

Per the §8.1 convention, seeds land in the **audit** (the suite's binding spec), not in
`memory/primitive-wip.md`, which the next `/implement-primitive` run overwrites wholesale.

| seed | finding | class | status |
|---|---|---|---|
| **OOS-DP4-1** | **A hybrid / Phyrexian / X attack tax is unpayable and is now hard-rejected.** `Command::DeclareAttackers` has no `hybrid_choices` / `phyrexian_life_payments` field, so the engine cannot ask which half of a `{2/W}` tax the player pays. Before PB-DP4 the pips were **silently dropped** by `combat.rs:221-227`'s field sum (a `{2/W}` tax contributed 0 and the attack was free — the OOS-RS-2 class); PB-DP4 rejects the declaration instead. Latent: both corpus restriction defs (`propaganda.rs`, `ghostly_prison.rs`) are pure `{2}` generic. Fix needs the two `DeclareAttackers` fields PB-RS2 added to `ActivateAbility`/`TapForMana` ⇒ **PROTOCOL bump**, so it is its own PB. | correctness, latent (wire) | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-2** | **CR 508.1i's mana-ability window between attack-cost determination and payment does not exist.** CR 508.1h locks in the total, 508.1i gives the active player "a chance to activate mana abilities", 508.1j then pays. The engine determines *and* pays inside one `DeclareAttackers`, so the tax must already be floating — a player with untapped lands and an empty pool cannot attack past a Propaganda. Pre-existing (PB-DP4 made the payment real but did not change the window). Fix needs a two-phase declaration ⇒ a new `Command` ⇒ **PROTOCOL bump**. The same gap exists for every "as it attacks" cost. | correctness, deviation (wire) | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-3** | **Goad's *directional* requirement has no CR 508.1d cost carve-out.** PB-DP4 gave both must-attack "able" tests (`combat.rs` goad block and `MustAttackEachCombat` block) the CR 508.1d "not required to pay" carve-out via `has_uncosted_attack_target`. The separate goad check "must attack a player other than the goading player if able" (`combat.rs:336-374`) still computes `has_non_goading_target` from opponent liveness only, so a goaded creature can be forced onto a *taxed* non-goading opponent when the untaxed goading player was the free option. Same root cause, narrower reach. No wire change. | correctness, narrow | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-4** | **The replay harness cannot script an echo or cumulative-upkeep payment.** `testing/replay_harness.rs` implements `pay_recover` (`:906-923`) and **no** `pay_echo` / `pay_cumulative_upkeep` action, so a golden script can never exercise the CR 702.30a / 702.24a payment branch — only PB-DP4's forced *decline* at the boundary is script-reachable. Latent today (`stack/151` is retired, `stack/152` never reaches an upkeep). DP-24 class. Deliberately not added speculatively: a harness action with no script is dead code SR-9c would flag. | test-infrastructure gap, latent | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-5** | **A forced decline is indistinguishable from a player's decline in the event stream.** PB-DP4's boundary sweep calls the same `handle_pay_*(pay: false)` path a real `Command::PayEcho { pay: false }` takes, so the emitted `CreatureDied` / `RecoverDeclined` carry no marker saying "the engine declined this for you because you never answered." A playtester cannot tell a rules outcome from an engine default. Fixing it inside the engine needs an event field ⇒ **HASH/PROTOCOL bump**; the cheap fix is audit §9 rec 8's "engine chose this for you" annotation, derived client-side in M11-local Session 7 from the absence of a preceding `Pay*` command. | diagnosability / agency visibility | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-6** | **`ManaPool`'s restricted-mana docs cite CR 106.12, which is the wrong rule.** Live CR 106.12 is *"To 'tap [a permanent] for mana' is to activate a mana ability … that includes the {T} symbol"*. The restricted-mana rule is **CR 106.6** ("Some spells or abilities that produce mana restrict how that mana can be spent"). Stale cites survive at `crates/card-types/src/state/player.rs:20`, `:46`, `:209` and in `docs/audits/decision-point-audit.md`'s §4.9 `ChooseCreatureType` row. PB-DP4 fixed only `:147` and `:181` (the two docs governing the API DP-10 calls). Same class as PB-DP2's stale `103.4b` and OOS-DP1-3's pre-renumber `116.3a`. Batch into a doc pass. | cosmetic / stale cite | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-7** | **Three near-duplicate mana-cost arithmetic helpers.** `rules/engine.rs:978-1002` `multiply_mana_cost` (multiplies every field including hybrid/Phyrexian/X, correct for cumulative upkeep), `rules/combat.rs`'s new `add_mana_cost` (rejects those fields, correct for an attack tax), and now a third copy in `crates/simulator/src/legal_actions.rs` (which must mirror the engine's exactly or the SR-38 affordability gate disagrees with the engine). A `ManaCost` `impl std::ops::Add`/`Mul` in `crates/card-types` would dedupe all three — but the semantics genuinely differ on hybrid/Phyrexian, so a naive merge would reintroduce the free-pip class (OOS-RS-2). Needs an argued API, not a rename. Cleanup pass. | cosmetic / refactor | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-8** | **`StubProvider` offers attack targets the player cannot pay the tax for.** `LegalAction::DeclareAttackers { eligible, targets }` (`legal_actions.rs:55-58`) lists every live opponent and opponent planeswalker with no reference to `CantAttackYouUnlessPay`, so `RandomBot::choose_attackers` routinely composes a declaration the engine rejects — and `driver.rs` answers the rejection with a silent `PassPriority`, so the bot simply loses its combat. Pre-existing (the affordability check predates PB-DP4), but PB-DP4's real debit makes it bite more often: the same floating mana no longer funds a second combat phase. SR-38 class. Fix = filter `targets`, and/or return a per-target tax so the bot can budget. Simulator-only, no wire change. | simulator move-generation gap | filed by PB-DP4 (`scutemob-152`) |
| **OOS-DP4-9** | **The echo and recover payment paths emit no `ManaCostPaid`.** PB-DP4 gave the attack tax a `GameEvent::ManaCostPaid` (Architecture Invariant 4: a pool debit is a state change and must be evented). `handle_pay_echo:663-666` and `handle_pay_recover:1063-1072` debit the pool and emit only `EchoPaid` / `RecoverPaid`, neither of which carries the cost; `handle_pay_cumulative_upkeep`'s `Life` arm does emit `LifeLost` but its `Mana` arm emits nothing for the debit. Deliberately not changed in PB-DP4 to keep the event-stream delta minimal for a PB whose blast radius already spans two subsystems. Wire-neutral (the variant exists); the only risk is tests that count events exactly. | diagnosability / Invariant 4 | filed by PB-DP4 (`scutemob-152`) |

**Seeds closed by this PB** (mark them, do not delete the rows):
**OOS-DP1-1** (§8.1) — closed by deletion, Change 2d.
**OOS-RS3-4** (`memory/primitives/rider-seed-triage-2026-07-19.md`) — closed by Change 1c;
also strike the accepted-limitation paragraph at `crates/card-defs/src/defs/goblin_rabblemaster.rs:35-55`
and replace it with a one-line "closed by PB-DP4 (`scutemob-152`), CR 508.1d" note. **This is the
one card-def edit PB-DP4 makes, and it is a comment, not behaviour.**

---

## 10. Audit bookkeeping — exact rows to update on close-out

File: `docs/audits/decision-point-audit.md`

| location | current | change |
|---|---|---|
| **§4.5, line 266** | `\| **Attack cost** (Propaganda) \| **508.1g** \| **D** \| rules/combat.rs:248-263 — see **DP-10** \|` | class **D** → **A**; **correct the CR cite**: Propaganda is a CR **508.1c** restriction, and the payment machinery is CR **508.1h / 508.1i / 508.1j** (508.1g covers *optional* "as it attacks" costs like exert, which is the row two below). Note: *"**A** since PB-DP4 — the total cost is built as a real summed `ManaCost` per defending player and debited via `casting::pay_cost` (CR 508.1j), colour-correct, with restricted mana correctly excluded (CR 106.6). Two residual deviations: no CR 508.1i mana-ability window (OOS-DP4-2) and hybrid/Phyrexian taxes rejected rather than paid (OOS-DP4-1)."* Site → `rules/combat.rs:185-265` + the debit site. |
| **§4.5, line 265** | `\| Attack requirements (goad, must-attack) \| 508.1d / 701.15b \| **A** \| rules/combat.rs:272-432, with the correct requirement-yields-to-restriction carve-out at :412-424 \|` | **the "**A**" was wrong** — the carve-out covered only `CantAttackOwner`, never `CantAttackYouUnlessPay`, so CR 508.1d's "that player is not required to pay that cost" was violated and a forced attacker + an unpayable tax on every viable opponent **deadlocked** the declare-attackers step. Reclassify as **A since PB-DP4** and name `has_uncosted_attack_target` + OOS-RS3-4 closed. Residual: the goad *directional* check (OOS-DP4-3). |
| **§4.11, line 393** | `\| Echo / cumulative upkeep / recover pay-or-sacrifice \| 702.30a / 702.24a / 702.59a \| **A** plumbing, **D** enforcement \| see **DP-11** \|` | → **A** plumbing, **A** enforcement (with a stated CR 608.2d timing deviation). Site → `rules/engine.rs::force_resolve_overdue_payments`. |
| **§5, DP-10 row (line 442)** | class D, open | prefix `**SHIPPED (PB-DP4, `scutemob-152`).**` and record: the tax is now a summed per-defender `ManaCost` debited in the mutation section (CR 508.1f→508.1j order); colour preserved; **restricted mana no longer counts toward affordability** (CR 106.6 — `can_pay_cost`/`pay_cost` use `spell: None`, resolving the `total_with_restricted()` vs `spend(cost, None)` inconsistency in the strict direction); hybrid/Phyrexian rejected (OOS-DP4-1); `GameEvent::ManaCostPaid` reused so **no wire change**; the in-code claim that interactive payment "requires a new `DeclareAttackers` command field" is **falsified and deleted**; **CR 508.1g cite corrected to 508.1c/h/i/j**; **CR 508.1d honoured**, closing OOS-RS3-4; `ghostly_prison.rs` and `propaganda.rs` (both `Complete`) stop being live-wrong with **0 def edits**. |
| **§5, DP-11 row (line 443)** | class D, open | prefix `**SHIPPED (PB-DP4, `scutemob-152`).**` and record the design decision and its deviation in full: the three `pending_*` vectors are now consulted by `handle_all_passed`'s stack-empty branch, which applies the CR 118.12a "didn't pay" branch to any unanswered payment before the game leaves the priority round in which the ability resolved. **The audit's own §8 phrasing — "wire: none if the 'otherwise' is applied at resolution rather than gated on priority" — is confirmed, but the boundary chosen is neither**: it is the *end of the priority round after* resolution, because (a) applying it *at* resolution destroys the CR 608.2d/608.2g choice and makes `Command::PayEcho` unreachable, and (b) gating priority deadlocks any seat that never sends the command (`driver.rs` answers a rejection with a silent `PassPriority`). Auto-**decline**, never auto-pay (CR 118.12a; auto-pay is DP-19's bug class). APNAP order (CR 101.4). Extra-round-not-advance so dies-triggers land in the correct step (the CR 514.3a pattern). Three `Complete` defs (`mogg_war_marshal`, `avalanche_riders`, `grim_harvest`) stop being live-wrong with **0 def edits**; `mystic_remora`'s `known_wrong` note becomes accurate. **PROTOCOL 27 / HASH 63 unmoved.** |
| **§8, PB-DP4 row (line 573)** | proposal | → **SHIPPED (`scutemob-152`)**; confirm the predicted `wire impact: none`; note the boundary correction above, and that the row's bundling rationale ("two 'cost checked but never collected' bugs of the same shape") held — the shared lesson is that *an affordability check is not a payment*, and both fixes are "make the check and the payment the same predicate." |
| **§8.1, OOS-DP1-1 row (line 599)** | open, "Correct fix is the pause itself, owned by **PB-DP4**" | → **CLOSED by PB-DP4 (`scutemob-152`)**, by **deletion**: all three `priority_holder = Some(active_player)` / `players_passed = OrdSet::new()` pairs are gone. Record *why* they were harmless for echo/CU (the controller *is* the active player and `resolve_top_of_stack:7768-7772` already granted priority to them with an empty pass set, so the write was an identity write) and *harmful* for recover (a non-active controller's answer yanked priority away from whoever held it). Answering a resolution-time payment is not a CR 117.3c action and not a CR 117.4 action-between-passes. |
| **§8.1** | seed table | append the nine `OOS-DP4-*` rows from §9 |
| **§9, recommendation 3 (lines 704-710)** | *"`advance()` should yield `AwaitingHuman` for a non-empty `pending_echo_payments` / … These need new `LegalAction` variants … Note **DP-11**: because nothing enforces the pending payments, an M11 game today lets every echo permanent stay for free."* | annotate: **the `LegalAction` half is DONE by PB-DP4; the `advance()` half turned out unnecessary.** Because the deadline makes the payment a priority-window choice, the three new actions arrive inside the *existing* `PendingDecision` (`local_game.rs:329`) and are classified `DecisionKind::Priority` by `decision_kind_for` (`:575-592`) — **no new `DecisionKind` variant, no `PendingDecision` reshaping, no `local_game.rs` edit.** Cheaper than this recommendation predicted. The `OrderBlockers` half (DP-13) remains open. |
| **§9, recommendation 6 (lines 731-734)** | *"…it is also the only way a playtester can currently exercise `PayEcho`, `PayCumulativeUpkeep`, `PayRecover` and `OrderBlockers` (no `LegalAction` exists for any of them…)"* | annotate: **superseded for the three payments** — `LegalAction::PayEcho` / `PayCumulativeUpkeep` / `PayRecover` now exist (PB-DP4) and reach a human seat through the normal priority decision. The `--dev` raw-command hatch is still recommended, and `OrderBlockers` is still the only decision with no `LegalAction`. |
| **§7, OOS-M11-2 section (lines 529-542)** | *"`solve_mana_payment` … contains zero references to `mana_pool`"* | add a one-line rider: PB-DP4's attack-tax debit makes the pool-blindness cost real in combat too — a bot that taps for a tax and then has the mana spent will over-tap on the next action. Still an M11-local Session 3 item, not a primitive-queue item. |

Also, per the DP-suite close-out convention: update `CLAUDE.md` "Current State" (PB-DP4 SHIPPED,
test-count delta from 3,747, PROTOCOL 27 / HASH 63 unmoved, the 5-`Complete`-cards-made-right
headline) and "Last Updated"; append a worker close-out to `memory/workstream-state.md`
"PB-DP suite — worker close-outs"; advance `memory/primitive-wip.md`'s phase; and edit
`memory/primitives/rider-seed-triage-2026-07-19.md` to mark **OOS-RS3-4 CLOSED by PB-DP4** (that
file is the RS queue's seed inventory — this is the one cross-queue edit this PB makes, and it is
a status marker, not a re-ranking; **do not touch the RS queue's ordering or its §5 pause
banner**).

---

## 11. Risks & edge cases

1. **The DP-11 boundary choice is the load-bearing judgement call.** If the reviewer prefers
   gating advancement (option 1), the cost is a forced-resolution backstop anyway (i.e. this
   design plus a hang) — flag it in the review file rather than reversing it mid-implement. If
   the reviewer prefers auto-pay-when-affordable, point at CR 118.12a and DP-19 before changing
   anything.
2. **The sweep must live in the stack-EMPTY branch only.** Putting it anywhere that runs in the
   same `handle_all_passed` call that resolved the trigger — e.g. at the top of the function, or
   inside `resolve_top_of_stack`, or in `handle_pass_priority` — **will destroy the payment
   before any test or script can answer it**, and every one of the 23 existing echo/CU/recover
   tests plus golden script `153` will fail. This is the single highest-risk mistake available in
   this PB.
3. **The extra-round branch must not advance the step.** If the runner extends `events` and falls
   through instead of returning, dies-triggers from the forced sacrifice land on the stack in the
   *next* step, which is a CR 603.3 violation and will show up as a bizarre golden-script diff
   rather than a clean failure.
4. **Termination of the extra round.** The guard is `!payment_events.is_empty()`, not
   "consumed > 0" — a CR 400.7 no-op decline consumes an entry and produces nothing, and must
   fall through to the advance. Getting this backwards produces an infinite priority round with
   no error (the `cleanup_sba_rounds` cap only guards the Cleanup step).
5. **Deleting the priority writes is safe *because* `resolve_top_of_stack` already does it.** If
   any test starts failing on `priority_holder` or `players_passed` after Change 2d, do **not**
   restore the write — that would reopen OOS-DP1-1. Trace what set priority instead, and report
   it (§4.7).
6. **The registry site-scan gate (§4.5) will fail loudly if the sweep or the provider names
   `KeywordAbility::Echo`/`::CumulativeUpkeep`/`::Recover` in code.** `crates/simulator/src` is a
   scan root. Read costs from the pending vectors. This is the concrete lesson PB-DP3 paid for.
7. **`random_bot.rs::action_to_command` is the only exhaustive `LegalAction` match** and it has
   no catchall. It will not compile until the three arms exist — that is the gate working. Do not
   add a `_ =>` arm to "fix" it.
8. **Restricted mana becomes a behaviour flip, not just a cleanup.** A player whose only mana is
   restricted can no longer attack past a Propaganda. It is correct (CR 106.6) and it is a probe
   (§7.1 #3), but call it out in the commit message — it is the kind of change that looks like a
   regression to someone reading a fuzzer diff.
9. **CR 508.1d weakens must-attack, and the two guards exist to bound that.** Probes 9 and 10
   (§7.1) pin that an untaxed opponent, or an untaxed opponent planeswalker, still forces the
   attack. Without them, `has_uncosted_attack_target` returning a blanket `false` would silently
   disable must-attack enforcement across every `MustAttackEachCombat` and goaded card and no
   test would notice.
10. **`has_uncosted_attack_target` must read layer-resolved characteristics.** The planeswalker
    scan must use `layers::expect_characteristics(state, id)`, never `obj.characteristics`
    (W3-LC contract, CR 613.1f). An animated planeswalker, or a Humility'd one, must be judged
    on its current types.
11. **Determinism (SR-9b).** Change 1a switches two `HashMap`s to `BTreeMap`, and the sweep
    iterates `apnap_order` then insertion order. Any set/map iteration introduced here that is
    not deterministically ordered will make the replay hash unstable in exactly the 150-200-turn
    regime OOS-M11-3 already flags, and will be very hard to attribute.
12. **`multiply_mana_cost` duplication is a correctness hazard, not just duplication.** The
    provider's copy (Change 3b) must mirror `engine.rs:978-1002` exactly, including
    hybrid/Phyrexian/X, or SR-38's "only offer what the engine accepts" breaks and a bot's
    `PayCumulativeUpkeep { pay: true }` starts getting rejected — silently, via `driver.rs`'s
    `PassPriority` fallback.
13. **Two echo/CU permanents at one upkeep.** `state/mod.rs:260-261` says only one payment can be
    pending at a time; that is false, and probe 19 pins the multi-entry case. The sweep must
    snapshot each player's entries **before** calling any handler — the handlers mutate the
    vector — or it will skip entries or index out of range.
14. **Do not widen the SR-3 seal.** The sweep lives in `rules/engine.rs`, inside the crate, and
    touches the private fields exactly as the existing handlers do. The `_mut` escape hatches at
    `state/mod.rs:774-792` are for out-of-crate callers and are **not needed here**; reaching for
    them would be a smell.
15. **`GameEvent::ManaCostPaid` for the attack tax could break an event-count assertion
    somewhere.** It is a new event in an existing stream. Grep showed only positive-existence
    assertions (`.any(matches!)`), never exact counts, in the 14 files that mention it — but if a
    property test or a golden script counts events, that is an un-enumerated site (§4.7).
